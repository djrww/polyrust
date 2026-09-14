//! 代數約束生成：把 Mini-Rust 的每條語法/型別規則編譯成多項式方程組，
//! 並同步抽取 R1CS（QAP 用）與 CDCL 子句。
//!
//! 編碼綱要（docs/THEOREMS.md §2 有完整推導）：
//! - 每個表達式節點 v 的型別位元 t_{v,τ}（one-hot：Σ_τ t_{v,τ} − 1 = 0）
//! - 每條規則 R 的輸出向量 out^R_τ 是操作數位元的二次多項式，
//!   約束 t_{v,τ} − out^R_τ = 0（∀τ）
//! - 宏臂位元 a_i：Σ a_i − 1 = 0；arm 內部約束一律乘 a_i（a_i ⟹ 約束）
//! - 借用位元 b_j：b_j − 1 = 0（借用確實發生）；
//!   重疊借用對：b_i·b_j = 0；對應 CDCL 子句 ¬b_i ∨ ¬b_j
//! - 域多項式 x² − x（布爾性）由管線在求解時加入

use crate::cdcl;
use crate::frac::Frac;
use crate::minirust::analysis::{analyze, BorrowAnalysis};
use crate::minirust::ast::*;
use crate::minirust::macros::Expander;
use crate::poly::Poly;
use crate::qap::{Linear, R1cs};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct System {
    pub nvars: usize,
    pub names: Vec<String>,
    /// 多項式方程組 f = 0（不含域多項式）
    pub polys: Vec<Poly>,
    /// CDCL 子句（arm 互斥、借用互斥；文字 = 系統變量索引）
    pub clauses: Vec<Vec<cdcl::Lit>>,
    /// 節點 → 7 個型別位元變量
    pub node_type: HashMap<usize, [usize; N_TYPES]>,
    /// invoke 節點 → 臂位元變量
    pub arm_vars: HashMap<usize, Vec<usize>>,
    /// RefMut 節點 → 借用位元變量
    pub borrow_vars: HashMap<usize, usize>,
    /// 節點種類（報告用）
    pub node_kind: HashMap<usize, String>,
    /// 借用衝突對（節點對）
    pub borrow_conflicts: Vec<(usize, usize)>,
}

struct Ctx<'a> {
    sys: System,
    exp: &'a mut Expander,
}

/// 型別位元向量的一次性建立。
fn type_bits(c: &mut Ctx<'_>, node: usize, kind: &str) -> [usize; N_TYPES] {
    if let Some(ts) = c.sys.node_type.get(&node) {
        return *ts;
    }
    let mut ts = [0usize; N_TYPES];
    for (ti, t) in ALL_TYPES.iter().enumerate() {
        let v = c.sys.nvars;
        c.sys.names.push(format!("t{}:{}", node, t.name()));
        ts[ti] = v;
        c.sys.nvars += 1;
    }
    c.sys.node_type.insert(node, ts);
    c.sys.node_kind.insert(node, kind.to_string());
    // one-hot：Σ_τ t_{v,τ} − 1
    let mut terms: Vec<(Vec<u32>, Frac)> = vec![(vec![], Frac::from_i64(-1))];
    for &v in ts.iter() {
        terms.push((mono_of(v, c.sys.nvars), Frac::ONE));
    }
    c.sys.polys.push(Poly::from_terms(terms));
    ts
}

fn mono_of(var: usize, nvars: usize) -> Vec<u32> {
    let mut m = vec![0u32; nvars.max(var + 1)];
    m[var] = 1;
    m
}

fn var_poly(c: &Ctx<'_>, v: usize) -> Poly {
    Poly::var(v, Frac::ONE, c.sys.nvars)
}

/// 加入 f = 0（乘上 ctx = 活躍臂位元乘積）
fn emit(c: &mut Ctx<'_>, f: Poly, ctx: &Poly) {
    let g = ctx.mul(&f);
    if !g.is_zero() {
        c.sys.polys.push(g);
    }
}

/// 生成整個程序的約束系統。
pub fn gen_constraints(p: &Program, exp: &mut Expander) -> Result<System, String> {
    let mut c = Ctx { sys: System::default(), exp };
    // 函式體
    for f in &p.fns {
        let empty_defs: HashMap<String, usize> = HashMap::new();
        let ba = analyze(&f.body, &empty_defs);
        emit_borrow_conflicts(&mut c, &ba, &Poly::constant(Frac::ONE), &f.body, &empty_defs);
        let mut scope = Scope::default();
        for param in &f.params {
            scope.types.insert(param.name.clone(), param.ty);
            scope.defs.insert(param.name.clone(), param.node);
            // 參數節點：型別即聲明型別
            let pts = type_bits(&mut c, param.node, "param");
            let pf = var_poly(&c, pts[param.ty.index()]).sub(&Poly::constant(Frac::ONE));
            emit(&mut c, pf, &Poly::constant(Frac::ONE));
        }
        walk(&f.body, &mut c, &mut scope, p, &Poly::constant(Frac::ONE))?;
    }
    // main 體
    let empty_defs: HashMap<String, usize> = HashMap::new();
    let ba = analyze(&p.main_body, &empty_defs);
    emit_borrow_conflicts(&mut c, &ba, &Poly::constant(Frac::ONE), &p.main_body, &empty_defs);
    let mut scope = Scope::default();
    walk(&p.main_body, &mut c, &mut scope, p, &Poly::constant(Frac::ONE))?;
    Ok(c.sys)
}

#[derive(Default, Clone)]
struct Scope {
    types: HashMap<String, Type>, // 型別（Call/Ref 規則用：函式簽名查 p.fns 即可；此處記 let 綁定型別？不用——約束系統不需要具體型別）
    defs: HashMap<String, usize>,
}

fn walk(e: &E, c: &mut Ctx<'_>, scope: &mut Scope, p: &Program, ctx: &Poly) -> Result<(), String> {
    let kind = kind_name(&e.kind);
    let _ts = type_bits(c, e.id, kind);
    match &e.kind {
        EKind::Int(_) => {
            let f = var_poly(c, _ts[Type::I32.index()]).sub(&Poly::constant(Frac::ONE));
            emit(c, f, ctx);
        }
        EKind::BoolV(_) => {
            let f = var_poly(c, _ts[Type::Bool.index()]).sub(&Poly::constant(Frac::ONE));
            emit(c, f, ctx);
        }
        EKind::UnitLit => {
            let f = var_poly(c, _ts[Type::Unit.index()]).sub(&Poly::constant(Frac::ONE));
            emit(c, f, ctx);
        }
        EKind::Var(name) => {
            if let Some(&def) = scope.defs.get(name) {
                let dts = type_bits(c, def, "def");
                for ti in 0..N_TYPES {
                    let f = var_poly(c, _ts[ti]).sub(&var_poly(c, dts[ti]));
                    emit(c, f, ctx);
                }
            } else {
                // 未綁定變量：強制矛盾（節點無型別 ⇒ one-hot 破產）
                for ti in 0..N_TYPES {
                    emit(c, var_poly(c, _ts[ti]), ctx);
                }
            }
        }
        EKind::Let(name, e1, e2) => {
            walk(e1, c, scope, p, ctx)?;
            let saved = scope.defs.insert(name.clone(), e1.id);
            walk(e2, c, scope, p, ctx)?;
            match saved {
                Some(old) => {
                    scope.defs.insert(name.clone(), old);
                }
                None => {
                    scope.defs.remove(name);
                }
            }
            // let 結果型別 = body 型別
            let dts = node_bits(c, e2.id);
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&var_poly(c, dts[ti]));
                emit(c, f, ctx);
            }
        }
        EKind::Seq(e1, e2) => {
            walk(e1, c, scope, p, ctx)?;
            walk(e2, c, scope, p, ctx)?;
            let dts = node_bits(c, e2.id);
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&var_poly(c, dts[ti]));
                emit(c, f, ctx);
            }
        }
        EKind::BinOp(op, a, b) => {
            walk(a, c, scope, p, ctx)?;
            walk(b, c, scope, p, ctx)?;
            let ats = node_bits(c, a.id);
            let bts = node_bits(c, b.id);
            let mut out: [Poly; N_TYPES] = Default::default();
            for ti in 0..N_TYPES {
                out[ti] = Poly::zero();
            }
            let i32i = Type::I32.index();
            let booli = Type::Bool.index();
            let prod = var_poly(c, ats[i32i]).mul(&var_poly(c, bts[i32i]));
            let prod_b = var_poly(c, ats[booli]).mul(&var_poly(c, bts[booli]));
            // Eq/Ne：兩操作數同型（皆 i32 或皆 bool）⇒ 結果 bool
            let same = prod.add(&prod_b);
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul => out[i32i] = prod,
                BinOp::Lt | BinOp::Le | BinOp::Ge => out[booli] = prod,
                BinOp::Eq | BinOp::Ne => out[booli] = same,
                BinOp::And => out[booli] = prod_b,
            }
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&out[ti]);
                emit(c, f, ctx);
            }
        }
        EKind::Not(a) => {
            walk(a, c, scope, p, ctx)?;
            let ats = node_bits(c, a.id);
            let mut out: [Poly; N_TYPES] = Default::default();
            for ti in 0..N_TYPES {
                out[ti] = Poly::zero();
            }
            out[Type::Bool.index()] = var_poly(c, ats[Type::Bool.index()]);
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&out[ti]);
                emit(c, f, ctx);
            }
        }
        EKind::Neg(a) => {
            walk(a, c, scope, p, ctx)?;
            let ats = node_bits(c, a.id);
            let mut out: [Poly; N_TYPES] = Default::default();
            for ti in 0..N_TYPES {
                out[ti] = Poly::zero();
            }
            out[Type::I32.index()] = var_poly(c, ats[Type::I32.index()]);
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&out[ti]);
                emit(c, f, ctx);
            }
        }
        EKind::If(cond, a, b) => {
            walk(cond, c, scope, p, ctx)?;
            walk(a, c, scope, p, ctx)?;
            walk(b, c, scope, p, ctx)?;
            let cts = node_bits(c, cond.id);
            let ats = node_bits(c, a.id);
            let bts = node_bits(c, b.id);
            // 條件必須是 bool
            emit(
                c,
                var_poly(c, cts[Type::Bool.index()]).sub(&Poly::constant(Frac::ONE)),
                ctx,
            );
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti])
                    .sub(&var_poly(c, ats[ti]).mul(&var_poly(c, bts[ti])));
                emit(c, f, ctx);
            }
        }
        EKind::Ref(name) | EKind::RefMut(name) => {
            let mutable = matches!(&e.kind, EKind::RefMut(_));
            let def = scope.defs.get(name).copied();
            let mut out: [Poly; N_TYPES] = Default::default();
            for ti in 0..N_TYPES {
                out[ti] = Poly::zero();
            }
            if let Some(def) = def {
                let dts = type_bits(c, def, "def");
                if mutable {
                    out[Type::RefMutI32.index()] = var_poly(c, dts[Type::I32.index()]);
                    out[Type::RefMutBool.index()] = var_poly(c, dts[Type::Bool.index()]);
                } else {
                    out[Type::RefI32.index()] = var_poly(c, dts[Type::I32.index()]);
                    out[Type::RefBool.index()] = var_poly(c, dts[Type::Bool.index()]);
                }
            }
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&out[ti]);
                emit(c, f, ctx);
            }
            if mutable {
                // 借用位元 b：b − 1 = 0
                let bv = borrow_bit(c, e.id);
                emit(c, var_poly(c, bv).sub(&Poly::constant(Frac::ONE)), ctx);
            }
        }
        EKind::Deref(a) => {
            walk(a, c, scope, p, ctx)?;
            let ats = node_bits(c, a.id);
            let mut out: [Poly; N_TYPES] = Default::default();
            for ti in 0..N_TYPES {
                out[ti] = Poly::zero();
            }
            out[Type::I32.index()] = var_poly(c, ats[Type::RefI32.index()])
                .add(&var_poly(c, ats[Type::RefMutI32.index()]));
            out[Type::Bool.index()] = var_poly(c, ats[Type::RefBool.index()])
                .add(&var_poly(c, ats[Type::RefMutBool.index()]));
            for ti in 0..N_TYPES {
                let f = var_poly(c, _ts[ti]).sub(&out[ti]);
                emit(c, f, ctx);
            }
        }
        EKind::AssignVar(name, rhs) => {
            walk(rhs, c, scope, p, ctx)?;
            let def = scope.defs.get(name).copied();
            if let Some(def) = def {
                let dts = type_bits(c, def, "def");
                let rts = node_bits(c, rhs.id);
                for ti in 0..N_TYPES {
                    let f = var_poly(c, dts[ti]).sub(&var_poly(c, rts[ti]));
                    emit(c, f, ctx);
                }
            }
            // x = e 的值是 unit
            emit(c, var_poly(c, _ts[Type::Unit.index()]).sub(&Poly::constant(Frac::ONE)), ctx);
        }
        EKind::AssignDeref(lhs, rhs) => {
            walk(lhs, c, scope, p, ctx)?;
            walk(rhs, c, scope, p, ctx)?;
            if let EKind::Deref(inner) = &lhs.kind {
                let its = node_bits(c, inner.id);
                let rts = node_bits(c, rhs.id);
                // inner 必須是 &mut T 且 rhs : T：
                // t_rhs,τ = t_inner,refmut(τ 的指向) ，並拒絕不可變引用
                for (refidx, valty) in [
                    (Type::RefMutI32.index(), Type::I32),
                    (Type::RefMutBool.index(), Type::Bool),
                ] {
                    let f = var_poly(c, rts[valty.index()])
                        .sub(&var_poly(c, its[refidx]));
                    emit(c, f, ctx);
                }
            }
            emit(c, var_poly(c, _ts[Type::Unit.index()]).sub(&Poly::constant(Frac::ONE)), ctx);
        }
        EKind::Call(f, args) => {
            for a in args {
                walk(a, c, scope, p, ctx)?;
            }
            let fd = p.fns.iter().find(|f_| f_.name == *f);
            match fd {
                Some(fd) if fd.params.len() == args.len() => {
                    // 每個實參型別必須等於對應形參型別
                    for (a, param) in args.iter().zip(fd.params.iter()) {
                        let ats = node_bits(c, a.id);
                        emit(
                            c,
                            var_poly(c, ats[param.ty.index()]).sub(&Poly::constant(Frac::ONE)),
                            ctx,
                        );
                    }
                    // 結果型別 = 回傳型別
                    emit(
                        c,
                        var_poly(c, _ts[fd.ret_ty.index()]).sub(&Poly::constant(Frac::ONE)),
                        ctx,
                    );
                }
                _ => {
                    // 未定義函式或實參個數不符 ⇒ 呼叫不可定型 ⇒ 強制矛盾
                    // （與 EKind::Var 未綁定變量的處理一致：令所有型別位元 = 0，破產 one-hot）
                    for &t in _ts.iter() {
                        emit(c, var_poly(c, t), ctx);
                    }
                }
            }
        }
        EKind::Invoke(name, toks) => {
            let mac = match c.exp.find_macro(name) {
                Some(m) => m,
                None => return Ok(()),
            };
            let arms = c.exp.matching_arms(mac, toks);
            if arms.is_empty() {
                return Ok(()); // 無臂匹配 ⇒ 呼叫者無法賦值（one-hot 仍在，系統可能 SAT——
                        // 由 arm one-hot 缺失導致；見 docs：管線在 CDCL 檢測臂互斥）
            }
            // 臂位元
            let avs: Vec<usize> = match c.sys.arm_vars.get(&e.id) {
                Some(v) => v.clone(),
                None => {
                    let mut v = vec![];
                    for i in 0..arms.len() {
                        let var = c.sys.nvars;
                        c.sys.names.push(format!("a{}:{}", e.id, i));
                        v.push(var);
                        c.sys.nvars += 1;
                    }
                    c.sys.arm_vars.insert(e.id, v.clone());
                    v
                }
            };
            // Σ a_i − 1 = 0（乘 ctx）
            let mut terms: Vec<(Vec<u32>, Frac)> = vec![(vec![], Frac::from_i64(-1))];
            for &a in avs.iter() {
                terms.push((mono_of(a, c.sys.nvars), Frac::ONE));
            }
            emit(c, Poly::from_terms(terms), ctx);
            // CDCL 子句：至少一臂 / 至多一臂
            c.sys.clauses.push(avs.iter().map(|&v| cdcl::lit(v, true)).collect());
            for i in 0..avs.len() {
                for j in (i + 1)..avs.len() {
                    c.sys.clauses.push(vec![cdcl::lit(avs[i], false), cdcl::lit(avs[j], false)]);
                }
            }
            // 各臂展開：tie 約束 + 遞迴（ctx' = ctx·a_i）
            for (k, &arm) in arms.iter().enumerate() {
                let tree = c.exp.expand_arm(e.id, mac, arm, toks)?;
                let a_i = var_poly(c, avs[k]);
                let new_ctx = ctx.mul(&a_i);
                let tts = {
                    walk(&tree, c, scope, p, &new_ctx)?;
                    node_bits(c, tree.id)
                };
                for ti in 0..N_TYPES {
                    let f = var_poly(c, _ts[ti]).sub(&var_poly(c, tts[ti]));
                    let g = new_ctx.mul(&f);
                    if !g.is_zero() {
                        c.sys.polys.push(g);
                    }
                }
                // arm 樹的借用衝突（乘該臂 ctx）
                let ba = analyze(&tree, &scope.defs);
                emit_borrow_conflicts(c, &ba, &new_ctx, &tree, &scope.defs);
            }
        }
    }
    Ok(())
}

fn node_bits(c: &Ctx<'_>, node: usize) -> [usize; N_TYPES] {
    c.sys.node_type[&node]
}

fn borrow_bit(c: &mut Ctx<'_>, node: usize) -> usize {
    if let Some(&b) = c.sys.borrow_vars.get(&node) {
        return b;
    }
    let v = c.sys.nvars;
    c.sys.names.push(format!("b{}", node));
    c.sys.nvars += 1;
    c.sys.borrow_vars.insert(node, v);
    v
}

/// 借用衝突 → 多項式與子句（乘上該樹的 ctx）。
fn emit_borrow_conflicts(
    c: &mut Ctx<'_>,
    ba: &BorrowAnalysis,
    ctx: &Poly,
    _tree: &E,
    _defs: &HashMap<String, usize>,
) {
    for &(n1, n2) in &ba.conflicts {
        let b1 = borrow_bit(c, n1);
        let b2 = borrow_bit(c, n2);
        emit(c, var_poly(c, b1).mul(&var_poly(c, b2)), ctx);
        c.sys.clauses.push(vec![cdcl::lit(b1, false), cdcl::lit(b2, false)]);
        c.sys.borrow_conflicts.push((n1, n2));
    }
    for &(nb, _assign) in &ba.assign_conflicts {
        let b = borrow_bit(c, nb);
        // 賦值點在借用存活區間內 ⇒ 借用必須不存在：b = 0（但 b − 1 = 0 已強制）⇒ 矛盾
        emit(c, var_poly(c, b), ctx);
        c.sys.clauses.push(vec![cdcl::lit(b, false)]);
        c.sys.borrow_conflicts.push((nb, nb));
    }
}

fn kind_name(k: &EKind) -> &'static str {
    match k {
        EKind::Int(_) => "Int",
        EKind::BoolV(_) => "Bool",
        EKind::UnitLit => "Unit",
        EKind::Var(_) => "Var",
        EKind::Let(..) => "Let",
        EKind::Seq(..) => "Seq",
        EKind::BinOp(op, _, _) => match op {
            BinOp::Add => "Add",
            BinOp::Sub => "Sub",
            BinOp::Mul => "Mul",
            BinOp::Lt => "Lt",
            BinOp::Le => "Le",
            BinOp::Ge => "Ge",
            BinOp::Eq => "Eq",
            BinOp::Ne => "Ne",
            BinOp::And => "And",
        },
        EKind::Not(_) => "Not",
        EKind::Neg(_) => "Neg",
        EKind::If(..) => "If",
        EKind::Ref(_) => "Ref",
        EKind::RefMut(_) => "RefMut",
        EKind::Deref(_) => "Deref",
        EKind::AssignVar(..) => "AssignVar",
        EKind::AssignDeref(..) => "AssignDeref",
        EKind::Call(..) => "Call",
        EKind::Invoke(..) => "Invoke",
    }
}

// ============ R1CS 轉換（定理 8） ============

/// 把多項式方程組轉成 R1CS。wire 0 = 常數 1；wire i+1 = 變量 i；
/// 之後為中間導線。每個方程 f = 0 變成若干個 (A·z)(B·z) = C·z 約束。
pub fn to_r1cs(nvars: usize, polys: &[Poly]) -> R1cs {
    let mut r = R1cs {
        n_wires: nvars + 1,
        constraints: vec![],
        intermediates: vec![],
    };
    for f in polys {
        if f.is_zero() {
            continue;
        }
        poly_to_r1cs(f, &mut r);
    }
    r
}

/// 單項式 → 線性形式（deg ≤ 1 直接；deg ≥ 2 遞迴拆半並引入中間導線）
fn monomial_linear(m: &[u32], r: &mut R1cs) -> Linear {
    let deg: u32 = m.iter().sum();
    if deg == 0 {
        return vec![(0, crate::fp::Fp::one())]; // 常數 1（wire 0）
    }
    if deg == 1 {
        let v = m.iter().position(|&e| e == 1).unwrap();
        return vec![(v + 1, crate::fp::Fp::one())]; // 變量 wire = idx+1
    }
    // 拆半：前 ceil(d/2) 個因子 vs 其餘
    let mut factors: Vec<usize> = vec![];
    for (i, &e) in m.iter().enumerate() {
        for _ in 0..e {
            factors.push(i);
        }
    }
    let half = factors.len().div_ceil(2);
    let (fa, fb) = factors.split_at(half);
    let la = monomial_factors_linear(fa, r);
    let lb = monomial_factors_linear(fb, r);
    // 新中間導線 w = la·lb
    let w = r.n_wires;
    r.n_wires += 1;
    r.constraints.push((la, lb, vec![(w, crate::fp::Fp::one())]));
    r.intermediates.push(Some(w)); // 本約束定義導線 w
    vec![(w, crate::fp::Fp::one())]
}

fn monomial_factors_linear(factors: &[usize], r: &mut R1cs) -> Linear {
    if factors.len() == 1 {
        return vec![(factors[0] + 1, crate::fp::Fp::one())];
    }
    if factors.is_empty() {
        return vec![(0, crate::fp::Fp::one())];
    }
    let half = factors.len().div_ceil(2);
    let (fa, fb) = factors.split_at(half);
    let la = monomial_factors_linear(fa, r);
    let lb = monomial_factors_linear(fb, r);
    let w = r.n_wires;
    r.n_wires += 1;
    r.constraints.push((la, lb, vec![(w, crate::fp::Fp::one())]));
    r.intermediates.push(Some(w)); // 本約束定義導線 w
    vec![(w, crate::fp::Fp::one())]
}

/// 方程 f = 0 → 約束集合：
/// 高次項各自用中間導線化約後，f 成為線性形式 L ⇒ (L)·(1) = 0。
fn poly_to_r1cs(f: &Poly, r: &mut R1cs) {
    let mut lin: Vec<(usize, crate::fp::Fp)> = vec![];
    for (m, cf) in &f.terms {
        let k = *cf;
        let deg: u32 = m.iter().sum();
        if deg == 0 {
            lin.push((0, k));
        } else if deg == 1 {
            let v = m.iter().position(|&e| e == 1).unwrap();
            lin.push((v + 1, k));
        } else {
            let lw = monomial_linear(m, r);
            for (w, c) in lw {
                lin.push((w, k * c));
            }
        }
    }
    // 合併同 wire 係數
    let mut merged: Vec<(usize, crate::fp::Fp)> = vec![];
    for (w, k) in lin {
        if let Some(e) = merged.iter_mut().find(|(mw, _)| *mw == w) {
            e.1 = e.1 + k;
        } else {
            merged.push((w, k));
        }
    }
    merged.retain(|(_, k)| !k.is_zero());
    r.constraints.push((merged, vec![(0, crate::fp::Fp::one())], vec![]));
    r.intermediates.push(None);
}
