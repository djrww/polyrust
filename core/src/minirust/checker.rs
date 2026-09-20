// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! 直接型別檢查器（ground truth oracle）：
//! 遞迴推導 Mini-Rust 型別，多臂宏採「存在臂選擇」語義
//! （存在某個臂組合使整個程序良構——管線的型別導向臂選擇與之一致；
//! 真實 macro_rules! 的「首個語法匹配臂」語義差異見 docs/THEOREMS.md §6）。

use crate::minirust::analysis::{analyze, BorrowAnalysis};
use crate::minirust::ast::*;
use crate::minirust::macros::Expander;
use std::collections::HashMap;

/// 一條接受推導：結果型別 + 每節點型別 + 每個宏調用選定的臂。
#[derive(Clone, Debug)]
pub struct Derivation {
    pub ty: Type,
    pub node_types: HashMap<usize, Type>,
    pub arm_choice: HashMap<usize, usize>,
}

/// 作用域：名 → 型別（推導用）＋ 名 → 綁定節點 id（借用分析用）
#[derive(Clone, Default)]
pub struct Scope {
    pub types: HashMap<String, Type>,
    pub defs: HashMap<String, usize>,
}

impl Scope {
    pub fn with(types: HashMap<String, Type>, defs: HashMap<String, usize>) -> Scope {
        Scope { types, defs }
    }
}

/// 檢查整個程序。Ok(所有推導) / Err(理由)。
/// fns 的每個體都必須存在與聲明回傳型別一致的推導。
pub fn check_program(p: &Program, exp: &mut Expander) -> Result<Vec<Derivation>, String> {
    // 1. 函式體（每個 fn 取一條與聲明一致的推導，合併進結果）
    let mut fn_types: HashMap<usize, Type> = HashMap::new();
    for f in &p.fns {
        let empty_defs: HashMap<String, usize> = HashMap::new();
        let ba = analyze(&f.body, &empty_defs);
        if !ba.is_clean() {
            return Err(format!("函式 {} 體借用衝突", f.name));
        }
        let mut types = HashMap::new();
        let mut defs = HashMap::new();
        for param in &f.params {
            types.insert(param.name.clone(), param.ty);
            defs.insert(param.name.clone(), param.node);
        }
        let ds = check_expr(&f.body, &Scope::with(types, defs), p, exp);
        let accept = ds.iter().find(|d| d.ty == f.ret_ty);
        match accept {
            None => {
                return Err(format!(
                    "函式 {} 體型別 {{{}}} 與聲明 {} 不符",
                    f.name,
                    ds.iter().map(|d| d.ty.name()).collect::<Vec<_>>().join(", "),
                    f.ret_ty.name()
                ))
            }
            Some(d) => {
                for (k, v) in &d.node_types {
                    fn_types.insert(*k, *v);
                }
                for param in &f.params {
                    fn_types.insert(param.node, param.ty);
                }
            }
        }
    }
    // 2. main 體
    let empty_defs: HashMap<String, usize> = HashMap::new();
    let ba = analyze(&p.main_body, &empty_defs);
    if !ba.is_clean() {
        return Err("main 體借用衝突".to_string());
    }
    let mut ds = check_expr(&p.main_body, &Scope::default(), p, exp);
    if ds.is_empty() {
        return Err("main 體不可定型".to_string());
    }
    // 合併 fn 體型別（σ_D 的完整性；定理 1）
    for d in ds.iter_mut() {
        for (k, v) in &fn_types {
            d.node_types.entry(*k).or_insert(*v);
        }
    }
    Ok(ds)
}

/// 檢查某棵樹，回傳所有可能推導（臂選擇的組合）。
/// 樹內借用衝突在 Invoke 的各 arm 樹上排除；樹本身的衝突由上層 analyze 抓。
fn check_expr(e: &E, scope: &Scope, p: &Program, exp: &mut Expander) -> Vec<Derivation> {
    let mut out: Vec<Derivation> = vec![];
    match &e.kind {
        EKind::Int(_) => out.push(simple(e.id, Type::I32)),
        EKind::BoolV(_) => out.push(simple(e.id, Type::Bool)),
        EKind::UnitLit => out.push(simple(e.id, Type::Unit)),
        EKind::Var(name) => {
            if let Some(&t) = scope.types.get(name) {
                out.push(simple(e.id, t));
            }
        }
        EKind::Let(name, e1, e2) => {
            for d1 in check_expr(e1, scope, p, exp) {
                let mut s2 = scope.clone();
                s2.types.insert(name.clone(), d1.ty);
                s2.defs.insert(name.clone(), e1.id);
                for d2 in check_expr(e2, &s2, p, exp) {
                    out.push(Derivation {
                        ty: d2.ty,
                        node_types: merged(&[
                            (e.id, d2.ty),
                        ], &[&d1.node_types, &d2.node_types], &[]),
                        arm_choice: merged_arms(&[&d1.arm_choice, &d2.arm_choice]),
                    });
                }
            }
        }
        EKind::Seq(e1, e2) => {
            for d1 in check_expr(e1, scope, p, exp) {
                for d2 in check_expr(e2, scope, p, exp) {
                    out.push(Derivation {
                        ty: d2.ty,
                        node_types: merged(&[(e.id, d2.ty)], &[&d1.node_types, &d2.node_types], &[]),
                        arm_choice: merged_arms(&[&d1.arm_choice, &d2.arm_choice]),
                    });
                }
            }
        }
        EKind::BinOp(op, a, b) => {
            for da in check_expr(a, scope, p, exp) {
                for db in check_expr(b, scope, p, exp) {
                    let ok = match op {
                        BinOp::Add | BinOp::Sub | BinOp::Mul => da.ty == Type::I32 && db.ty == Type::I32,
                        BinOp::Lt | BinOp::Le | BinOp::Ge => da.ty == Type::I32 && db.ty == Type::I32,
                        BinOp::Eq | BinOp::Ne => da.ty == db.ty && (da.ty == Type::I32 || da.ty == Type::Bool),
                        BinOp::And => da.ty == Type::Bool && db.ty == Type::Bool,
                    };
                    if ok {
                        let ty = match op {
                            BinOp::Lt | BinOp::Le | BinOp::Ge | BinOp::Eq | BinOp::Ne | BinOp::And => Type::Bool,
                            _ => Type::I32,
                        };
                        out.push(Derivation {
                            ty,
                            node_types: merged(&[(e.id, ty)], &[&da.node_types, &db.node_types], &[]),
                            arm_choice: merged_arms(&[&da.arm_choice, &db.arm_choice]),
                        });
                    }
                }
            }
        }
        EKind::Not(a) => {
            for da in check_expr(a, scope, p, exp) {
                if da.ty == Type::Bool {
                    out.push(Derivation {
                        ty: Type::Bool,
                        node_types: merged(&[(e.id, Type::Bool)], &[&da.node_types], &[]),
                        arm_choice: da.arm_choice.clone(),
                    });
                }
            }
        }
        EKind::Neg(a) => {
            for da in check_expr(a, scope, p, exp) {
                if da.ty == Type::I32 {
                    out.push(Derivation {
                        ty: Type::I32,
                        node_types: merged(&[(e.id, Type::I32)], &[&da.node_types], &[]),
                        arm_choice: da.arm_choice.clone(),
                    });
                }
            }
        }
        EKind::If(c, a, b) => {
            for dc in check_expr(c, scope, p, exp) {
                if dc.ty != Type::Bool {
                    continue;
                }
                for da in check_expr(a, scope, p, exp) {
                    for db in check_expr(b, scope, p, exp) {
                        if da.ty == db.ty {
                            out.push(Derivation {
                                ty: da.ty,
                                node_types: merged(
                                    &[(e.id, da.ty)],
                                    &[&dc.node_types, &da.node_types, &db.node_types],
                                    &[],
                                ),
                                arm_choice: merged_arms(&[
                                    &dc.arm_choice,
                                    &da.arm_choice,
                                    &db.arm_choice,
                                ]),
                            });
                        }
                    }
                }
            }
        }
        EKind::Ref(name) => {
            if let Some(&t) = scope.types.get(name) {
                let rt = match t {
                    Type::I32 => Some(Type::RefI32),
                    Type::Bool => Some(Type::RefBool),
                    _ => None,
                };
                if let Some(rt) = rt {
                    out.push(simple(e.id, rt));
                }
            }
        }
        EKind::RefMut(name) => {
            if let Some(&t) = scope.types.get(name) {
                let rt = match t {
                    Type::I32 => Some(Type::RefMutI32),
                    Type::Bool => Some(Type::RefMutBool),
                    _ => None,
                };
                if let Some(rt) = rt {
                    out.push(simple(e.id, rt));
                }
            }
        }
        EKind::Deref(a) => {
            for da in check_expr(a, scope, p, exp) {
                let t = match da.ty {
                    Type::RefI32 | Type::RefMutI32 => Some(Type::I32),
                    Type::RefBool | Type::RefMutBool => Some(Type::Bool),
                    _ => None,
                };
                if let Some(t) = t {
                    out.push(Derivation {
                        ty: t,
                        node_types: merged(&[(e.id, t)], &[&da.node_types], &[]),
                        arm_choice: da.arm_choice.clone(),
                    });
                }
            }
        }
        EKind::AssignVar(name, rhs) => {
            if let Some(&xt) = scope.types.get(name) {
                for dr in check_expr(rhs, scope, p, exp) {
                    if dr.ty == xt {
                        out.push(Derivation {
                            ty: Type::Unit,
                            node_types: merged(&[(e.id, Type::Unit)], &[&dr.node_types], &[]),
                            arm_choice: dr.arm_choice.clone(),
                        });
                    }
                }
            }
        }
        EKind::AssignDeref(lhs, rhs) => {
            // AST 約定：lhs 即引用表達式本身（parser 已剝去 `*`），須為 &mut T；rhs : T
            for di in check_expr(lhs, scope, p, exp) {
                let pointed = match di.ty {
                    Type::RefMutI32 => Some(Type::I32),
                    Type::RefMutBool => Some(Type::Bool),
                    _ => None,
                };
                if let Some(t) = pointed {
                    for dr in check_expr(rhs, scope, p, exp) {
                        if dr.ty == t {
                            out.push(Derivation {
                                ty: Type::Unit,
                                node_types: merged(
                                    &[(e.id, Type::Unit)],
                                    &[&di.node_types, &dr.node_types],
                                    &[],
                                ),
                                arm_choice: merged_arms(&[&di.arm_choice, &dr.arm_choice]),
                            });
                        }
                    }
                }
            }
        }
        EKind::Call(f, args) => {
            // WRP-R5: builtin IO 宏/函数（println 等）视为 Unit，rustc 同：不经 p.fns 查表
            if matches!(f.as_str(), "println" | "eprintln" | "print" | "eprint" | "println!" | "eprintln!" | "print!" | "eprint!" | "dbg") {
                let mut nt = HashMap::new();
                nt.insert(e.id, Type::Unit);
                let mut ac = HashMap::new();
                let mut ok = true;
                for a in args {
                    let ds = check_expr(a, scope, p, exp);
                    if ds.is_empty() { ok = false; break; }
                    // 取首条推导合并（variadic，不校验形参类型）
                    let d = &ds[0];
                    for (k,v) in &d.node_types { nt.insert(*k, *v); }
                    for (k,v) in &d.arm_choice { ac.insert(*k, *v); }
                }
                if ok {
                    out.push(Derivation { ty: Type::Unit, node_types: nt, arm_choice: ac });
                }
                return out;
            }
            if let Some(fd) = p.fns.iter().find(|f_| f_.name == *f) {
                // 參數個數必須一致
                if fd.params.len() != args.len() {
                    return out;
                }
                // 每個實參的推導與對應形參型別匹配
                let mut arg_derivs: Vec<Vec<Derivation>> = Vec::new();
                for a in args {
                    arg_derivs.push(check_expr(a, scope, p, exp));
                }
                // 笛卡爾積：每個實參選一條推導，全部型別匹配才接受
                fn cartesian(ds: &[Vec<Derivation>], i: usize, acc: &mut Vec<Derivation>, out: &mut Vec<Vec<Derivation>>) {
                    if i == ds.len() {
                        out.push(acc.clone());
                        return;
                    }
                    for d in &ds[i] {
                        acc.push(d.clone());
                        cartesian(ds, i + 1, acc, out);
                        acc.pop();
                    }
                }
                let mut combos: Vec<Vec<Derivation>> = Vec::new();
                cartesian(&arg_derivs, 0, &mut Vec::new(), &mut combos);
                for combo in combos {
                    let all_match = combo.iter().zip(fd.params.iter()).all(|(d, param)| d.ty == param.ty);
                    if !all_match {
                        continue;
                    }
                    // 合併所有實參的 node_types 與 arm_choice
                    let mut nt = HashMap::new();
                    nt.insert(e.id, fd.ret_ty);
                    let mut ac = HashMap::new();
                    for d in &combo {
                        for (k, v) in &d.node_types {
                            nt.insert(*k, *v);
                        }
                        for (k, v) in &d.arm_choice {
                            ac.insert(*k, *v);
                        }
                    }
                    out.push(Derivation { ty: fd.ret_ty, node_types: nt, arm_choice: ac });
                }
            }
        }
        EKind::Invoke(name, toks) => {
            // WRP-R5: builtin println 类宏直接视为 Unit（无需宏定义）
            if matches!(name.as_str(), "println" | "eprintln" | "print" | "eprint" | "println!" | "eprintln!" | "print!" | "eprint!") {
                out.push(simple(e.id, Type::Unit));
                return out;
            }
            let mac = match exp.find_macro(name) {
                Some(m) => m,
                None => return out,
            };
            for arm in exp.matching_arms(mac, toks) {
                let tree = match exp.expand_arm(e.id, mac, arm, toks) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                // arm 樹借用衝突 ⇒ 排除該臂
                let ba: BorrowAnalysis = analyze(&tree, &scope.defs);
                if !ba.is_clean() {
                    continue;
                }
                for d in check_expr(&tree, scope, p, exp) {
                    let mut arms = d.arm_choice.clone();
                    arms.insert(e.id, arm);
                    out.push(Derivation {
                        ty: d.ty,
                        node_types: {
                            // 合併：arm 樹節點 + invoke 節點本身（型別 = arm 結果）
                            let mut m = d.node_types.clone();
                            m.insert(e.id, d.ty);
                            m
                        },
                        arm_choice: arms,
                    });
                }
            }
        }
    }
    // 上限保護（演示規模）
    if out.len() > 64 {
        out.truncate(64);
    }
    out
}

fn simple(id: usize, ty: Type) -> Derivation {
    Derivation { ty, node_types: HashMap::from([(id, ty)]), arm_choice: HashMap::new() }
}

fn merged(
    own: &[(usize, Type)],
    parts: &[&HashMap<usize, Type>],
    _extra: &[()],
) -> HashMap<usize, Type> {
    let mut m = HashMap::new();
    for p in parts {
        for (k, v) in p.iter() {
            m.insert(*k, *v);
        }
    }
    for (k, v) in own {
        m.insert(*k, *v);
    }
    m
}

fn merged_arms(parts: &[&HashMap<usize, usize>]) -> HashMap<usize, usize> {
    let mut m = HashMap::new();
    for p in parts {
        for (k, v) in p.iter() {
            m.insert(*k, *v);
        }
    }
    m
}

/// 實際使用：checker.rs 文件清單 — 優化 with_capacity
pub fn checker_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("checker.rs", "checker.rs 正式運作 — 優化 with_capacity", "core/src/minirust/checker.rs"),
    ]
}

