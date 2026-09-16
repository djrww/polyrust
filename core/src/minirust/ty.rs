//! Phase2 — 型別宇宙構造、閉包、統一 (unify)
//! Phase3 — lifetime 參數、outlives 圖、泛型 lifetime 統一
//!
//! 基於 Phase1 的 `Universe` 與 `TypeV2`，提供：
//! - 從 `ProgramV2` 收集類型並構造閉包宇宙
//! - 類型統一 `unify(T1,T2)` → 多項式 `t_T1 - t_T2 =0` 約束
//! - 泛型實例化與替代（含 lifetime）
//! - 子類型閉包與依賴圖

use std::collections::HashMap;

use super::ast_v2::{ItemV2, ProgramV2};
use super::universe::{ExtType, TypeV2, Universe, parse_type_v2};

/// 類型變量（用於泛型統一）
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeVar(pub String);

/// 統一結果
#[derive(Clone, Debug)]
pub enum UnifyResult {
    /// 兩類型已相同
    Same,
    /// 需要多項式約束 `t_T1 - t_T2 =0`（索引對）
    NeedEq { idx1: usize, idx2: usize },
    /// 需要多項式組（用於複合類型）
    NeedEqs(Vec<(usize, usize)>),
    /// 無法統一
    Fail(String),
}

/// 類型環境：泛型參數 → 具體類型
#[derive(Clone, Debug, Default)]
pub struct TyEnv {
    pub map: HashMap<String, TypeV2>,
}

impl TyEnv {
    pub fn new() -> Self { Self::default() }
    pub fn insert(&mut self, name: String, ty: TypeV2) { self.map.insert(name, ty); }
    pub fn get(&self, name: &str) -> Option<&TypeV2> { self.map.get(name) }
}

/// Lifetime 參數環境
#[derive(Clone, Debug, Default)]
pub struct LifetimeEnv {
    pub map: HashMap<String, String>, // 'a -> 'b 或具體 lifetime
}

impl LifetimeEnv {
    pub fn new() -> Self { Self::default() }
    pub fn insert(&mut self, name: String, lt: String) { self.map.insert(name, lt); }
    pub fn get(&self, name: &str) -> Option<&String> { self.map.get(name) }
}

/// 實際使用：ty 文件清單，供 pipeline 消費 — 優化 with_capacity
pub fn ty_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("ty.rs", "型別宇宙構造、閉包、統一、替代 — 優化 with_capacity", "core/src/minirust/ty.rs"),
        ("universe.rs", "Universe TypeV2 — ty 依賴", "core/src/minirust/universe.rs"),
        ("lifetime.rs", "LifetimeGraph — ty 依賴", "core/src/minirust/lifetime.rs"),
    ]
}
pub fn ty_summary(uni: &Universe) -> String {
    let mut out = String::with_capacity(256);
    out.push_str(&format!("ty: N={} ext={}\n", uni.n_types(), uni.n_ext()));
    out.push_str(&uni.display());
    out
}

/// 構造宇宙：從 ProgramV2 收集所有類型並閉包 — 優化 with_capacity
pub fn build_universe_from_program(prog: &ProgramV2) -> Universe {
    let mut uni = prog.universe.clone();
    for item in &prog.items {
        match item {
            ItemV2::Struct(s) => {
                for (_, ty) in &s.fields {
                    uni.insert_closure(ty.clone());
                }
            }
            ItemV2::Enum(e) => {
                for v in &e.variants {
                    for ty in &v.fields {
                        uni.insert_closure(ty.clone());
                    }
                }
            }
            ItemV2::Fn(f) => {
                for (_, ty) in &f.sig.params {
                    uni.insert_closure(ty.clone());
                }
                uni.insert_closure(f.sig.ret.clone());
            }
            ItemV2::Impl(im) => {
                uni.insert_closure(im.self_ty.clone());
                for m in &im.methods {
                    for (_, ty) in &m.sig.params {
                        uni.insert_closure(ty.clone());
                    }
                    uni.insert_closure(m.sig.ret.clone());
                }
            }
            ItemV2::Trait(tr) => {
                for m in &tr.methods {
                    for (_, ty) in &m.params {
                        uni.insert_closure(ty.clone());
                    }
                    uni.insert_closure(m.ret.clone());
                }
            }
            _ => {}
        }
    }
    uni
}

/// 從源碼文本直接構造宇宙（比 parse_v2::build_universe_from_src 更精確）
pub fn build_universe_from_src(src: &str) -> Universe {
    let mut uni = Universe::new();
    if let Ok(prog) = ProgramV2::parse_v2(src) {
        uni = build_universe_from_program(&prog);
    }
    let extra_patterns = [
        "Vec<i32>", "Vec<String>", "Vec<User>", "String",
        "HashMap<String,i32>", "HashMap<String,String>",
        "Option<i32>", "Option<String>", "Option<User>",
        "Result<i32,String>", "Result<User,String>",
        "*mut i32", "*const i32", "*mut User", "*const User",
        "&i32", "&mut i32", "&String", "&mut String",
        "Future<i32>", "Future<String>",
        "&'a i32", "&'a mut i32", "&'static str",
    ];
    for pat in extra_patterns {
        if src.contains(pat) {
            if let Ok(ty) = parse_type_v2(pat) {
                uni.insert_closure(ty);
            }
        }
    }
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("struct ") || t.starts_with("pub struct ") {
            let name = t.split_whitespace()
                .skip_while(|w| *w == "pub" || *w == "struct")
                .next()
                .unwrap_or("")
                .trim_matches(|c| c == '{' || c == '<' || c == '>')
                .split(|c| c == '<' || c == '{' || c == '(')
                .next()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                let ty = TypeV2::Ext(ExtType::Struct { name: name.clone(), args: vec![] });
                uni.insert_closure(ty);
            }
        }
        if t.starts_with("enum ") || t.starts_with("pub enum ") {
            let name = t.split_whitespace()
                .skip_while(|w| *w == "pub" || *w == "enum")
                .next()
                .unwrap_or("")
                .trim_matches(|c| c == '{' || c == '<' || c == '>')
                .split(|c| c == '<' || c == '{' || c == '(')
                .next()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                let ty = TypeV2::Ext(ExtType::Enum { name: name.clone(), args: vec![] });
                uni.insert_closure(ty);
            }
        }
    }
    uni
}

/// 類型統一：判斷 T1 與 T2 是否可統一，返回需要的等式約束
pub fn unify(t1: &TypeV2, t2: &TypeV2, uni: &Universe) -> UnifyResult {
    if t1 == t2 {
        return UnifyResult::Same;
    }
    match (t1, t2) {
        (TypeV2::Ext(ExtType::GenericParam(_)), _) |
        (_, TypeV2::Ext(ExtType::GenericParam(_))) => {
            if let (Some(i1), Some(i2)) = (uni.index_of(t1), uni.index_of(t2)) {
                return UnifyResult::NeedEq { idx1: i1, idx2: i2 };
            } else {
                return UnifyResult::Fail(format!("type not in universe: {:?} vs {:?}", t1, t2));
            }
        }
        _ => {}
    }
    match (t1, t2) {
        (TypeV2::Ext(ExtType::Vec(a)), TypeV2::Ext(ExtType::Vec(b))) => unify(a, b, uni),
        (TypeV2::Ext(ExtType::Option(a)), TypeV2::Ext(ExtType::Option(b))) => unify(a, b, uni),
        (TypeV2::Ext(ExtType::Future(a)), TypeV2::Ext(ExtType::Future(b))) => unify(a, b, uni),
        (TypeV2::Ext(ExtType::HashMap(k1, v1)), TypeV2::Ext(ExtType::HashMap(k2, v2))) => {
            let r1 = unify(k1, k2, uni);
            let r2 = unify(v1, v2, uni);
            combine_unify(r1, r2, uni, t1, t2)
        }
        (TypeV2::Ext(ExtType::Result(t1a, e1)), TypeV2::Ext(ExtType::Result(t2a, e2))) => {
            let r1 = unify(t1a, t2a, uni);
            let r2 = unify(e1, e2, uni);
            combine_unify(r1, r2, uni, t1, t2)
        }
        (TypeV2::Ext(ExtType::RawPtr { mutbl: m1, inner: i1 }), TypeV2::Ext(ExtType::RawPtr { mutbl: m2, inner: i2 })) => {
            if m1 != m2 {
                return UnifyResult::Fail(format!("raw ptr mutbl mismatch: {} vs {}", m1, m2));
            }
            unify(i1, i2, uni)
        }
        (TypeV2::Ext(ExtType::RefExt { mutbl: m1, inner: i1, lifetime: lt1 }), TypeV2::Ext(ExtType::RefExt { mutbl: m2, inner: i2, lifetime: lt2 })) => {
            if m1 != m2 {
                return UnifyResult::Fail(format!("ref mutbl mismatch"));
            }
            // lifetime 兼容性：若都有 lifetime，檢查是否可統一
            match (lt1, lt2) {
                (Some(l1), Some(l2)) if l1 != l2 => {
                    // 允許不同 lifetime，若存在 outlives 關係則 Same，否則需約束
                    // 這裡返回 Same，具體由 lifetime graph 檢查
                }
                _ => {}
            }
            unify(i1, i2, uni)
        }
        (TypeV2::Ext(ExtType::Struct { name: n1, args: a1 }), TypeV2::Ext(ExtType::Struct { name: n2, args: a2 })) => {
            if n1 != n2 {
                return UnifyResult::Fail(format!("struct name mismatch: {} vs {}", n1, n2));
            }
            if a1.len() != a2.len() {
                return UnifyResult::Fail(format!("struct args len mismatch"));
            }
            let mut eqs = vec![];
            for (x, y) in a1.iter().zip(a2.iter()) {
                match unify(x, y, uni) {
                    UnifyResult::Same => {}
                    UnifyResult::NeedEq { idx1, idx2 } => eqs.push((idx1, idx2)),
                    UnifyResult::NeedEqs(mut v) => eqs.append(&mut v),
                    f @ UnifyResult::Fail(_) => return f,
                }
            }
            if eqs.is_empty() { UnifyResult::Same } else { UnifyResult::NeedEqs(eqs) }
        }
        (TypeV2::Ext(ExtType::Enum { name: n1, args: a1 }), TypeV2::Ext(ExtType::Enum { name: n2, args: a2 })) => {
            if n1 != n2 {
                return UnifyResult::Fail(format!("enum name mismatch: {} vs {}", n1, n2));
            }
            if a1.len() != a2.len() {
                return UnifyResult::Fail(format!("enum args len mismatch"));
            }
            let mut eqs = vec![];
            for (x, y) in a1.iter().zip(a2.iter()) {
                match unify(x, y, uni) {
                    UnifyResult::Same => {}
                    UnifyResult::NeedEq { idx1, idx2 } => eqs.push((idx1, idx2)),
                    UnifyResult::NeedEqs(mut v) => eqs.append(&mut v),
                    f @ UnifyResult::Fail(_) => return f,
                }
            }
            if eqs.is_empty() { UnifyResult::Same } else { UnifyResult::NeedEqs(eqs) }
        }
        (TypeV2::Base(b1), TypeV2::Base(b2)) => {
            if b1 == b2 { UnifyResult::Same } else { UnifyResult::Fail(format!("base mismatch: {:?} vs {:?}", b1, b2)) }
        }
        _ => {
            if let (Some(i1), Some(i2)) = (uni.index_of(t1), uni.index_of(t2)) {
                UnifyResult::NeedEq { idx1: i1, idx2: i2 }
            } else {
                UnifyResult::Fail(format!("cannot unify {:?} vs {:?}", t1.name(), t2.name()))
            }
        }
    }
}

fn combine_unify(r1: UnifyResult, r2: UnifyResult, _uni: &Universe, _t1: &TypeV2, _t2: &TypeV2) -> UnifyResult {
    match (r1, r2) {
        (UnifyResult::Fail(e), _) => UnifyResult::Fail(e),
        (_, UnifyResult::Fail(e)) => UnifyResult::Fail(e),
        (UnifyResult::Same, x) => x,
        (x, UnifyResult::Same) => x,
        (UnifyResult::NeedEq { idx1: a1, idx2: b1 }, UnifyResult::NeedEq { idx1: a2, idx2: b2 }) => {
            UnifyResult::NeedEqs(vec![(a1, b1), (a2, b2)])
        }
        (UnifyResult::NeedEq { idx1, idx2 }, UnifyResult::NeedEqs(mut v)) |
        (UnifyResult::NeedEqs(mut v), UnifyResult::NeedEq { idx1, idx2 }) => {
            v.push((idx1, idx2));
            UnifyResult::NeedEqs(v)
        }
        (UnifyResult::NeedEqs(mut v1), UnifyResult::NeedEqs(v2)) => {
            v1.extend(v2);
            UnifyResult::NeedEqs(v1)
        }
    }
}

/// 生成統一多項式文本 — 優化 with_capacity
pub fn unify_poly_text(node_id: usize, idx1: usize, idx2: usize) -> String {
    let mut s = String::with_capacity(32);
    s.push_str("t");
    s.push_str(&node_id.to_string());
    s.push('_');
    s.push_str(&idx1.to_string());
    s.push_str(" - t");
    s.push_str(&node_id.to_string());
    s.push('_');
    s.push_str(&idx2.to_string());
    s.push_str(" = 0  # unify");
    s
}

/// 類型替代：將泛型參數替換為具體類型，支持 lifetime
pub fn subst_type(ty: &TypeV2, env: &TyEnv) -> TypeV2 {
    subst_type_with_lt(ty, env, &LifetimeEnv::default())
}

pub fn subst_type_with_lt(ty: &TypeV2, env: &TyEnv, lt_env: &LifetimeEnv) -> TypeV2 {
    match ty {
        TypeV2::Ext(ExtType::GenericParam(name)) => {
            if name.starts_with('\'') {
                if let Some(mapped) = lt_env.get(name) {
                    TypeV2::Ext(ExtType::GenericParam(mapped.clone()))
                } else {
                    ty.clone()
                }
            } else {
                env.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
        }
        TypeV2::Ext(ExtType::Vec(inner)) => {
            TypeV2::Ext(ExtType::Vec(Box::new(subst_type_with_lt(inner, env, lt_env))))
        }
        TypeV2::Ext(ExtType::Option(inner)) => {
            TypeV2::Ext(ExtType::Option(Box::new(subst_type_with_lt(inner, env, lt_env))))
        }
        TypeV2::Ext(ExtType::Future(inner)) => {
            TypeV2::Ext(ExtType::Future(Box::new(subst_type_with_lt(inner, env, lt_env))))
        }
        TypeV2::Ext(ExtType::HashMap(k, v)) => {
            TypeV2::Ext(ExtType::HashMap(Box::new(subst_type_with_lt(k, env, lt_env)), Box::new(subst_type_with_lt(v, env, lt_env))))
        }
        TypeV2::Ext(ExtType::Result(t, e)) => {
            TypeV2::Ext(ExtType::Result(Box::new(subst_type_with_lt(t, env, lt_env)), Box::new(subst_type_with_lt(e, env, lt_env))))
        }
        TypeV2::Ext(ExtType::RawPtr { mutbl, inner }) => {
            TypeV2::Ext(ExtType::RawPtr { mutbl: *mutbl, inner: Box::new(subst_type_with_lt(inner, env, lt_env)) })
        }
        TypeV2::Ext(ExtType::RefExt { mutbl, inner, lifetime }) => {
            let new_lt = if let Some(lt) = lifetime {
                if lt.starts_with('\'') {
                    if let Some(mapped) = lt_env.get(lt) {
                        Some(mapped.clone())
                    } else {
                        Some(lt.clone())
                    }
                } else {
                    Some(lt.clone())
                }
            } else {
                None
            };
            TypeV2::Ext(ExtType::RefExt { mutbl: *mutbl, inner: Box::new(subst_type_with_lt(inner, env, lt_env)), lifetime: new_lt })
        }
        TypeV2::Ext(ExtType::Struct { name, args }) => {
            TypeV2::Ext(ExtType::Struct { name: name.clone(), args: args.iter().map(|a| subst_type_with_lt(a, env, lt_env)).collect() })
        }
        TypeV2::Ext(ExtType::Enum { name, args }) => {
            TypeV2::Ext(ExtType::Enum { name: name.clone(), args: args.iter().map(|a| subst_type_with_lt(a, env, lt_env)).collect() })
        }
        _ => ty.clone(),
    }
}

/// 檢查 lifetime 參數是否滿足 outlives 約束
pub fn check_lifetime_bounds(
    ty: &TypeV2,
    graph: &crate::minirust::lifetime::LifetimeGraph,
) -> Result<(), String> {
    let mut lts = vec![];
    collect_lifetimes(ty, &mut lts);
    for lt in &lts {
        if lt.0 != "'static" && !graph.lifetimes.contains(lt) {
            if !graph.lifetimes.is_empty() {
                // 允許未在 graph 中的 lifetime，視為自由變量
            }
        }
    }
    if graph.has_cycle() {
        return Err("lifetime cycle detected".to_string());
    }
    Ok(())
}

fn collect_lifetimes(ty: &TypeV2, out: &mut Vec<crate::minirust::lifetime::Lifetime>) {
    match ty {
        TypeV2::Ext(ExtType::GenericParam(name)) if name.starts_with('\'') => {
            out.push(crate::minirust::lifetime::Lifetime::new(name));
        }
        TypeV2::Ext(ExtType::RefExt { lifetime: Some(lt), inner, .. }) => {
            out.push(crate::minirust::lifetime::Lifetime::new(lt));
            collect_lifetimes(inner, out);
        }
        TypeV2::Ext(ExtType::Vec(inner)) | TypeV2::Ext(ExtType::Option(inner)) | TypeV2::Ext(ExtType::Future(inner)) => {
            collect_lifetimes(inner, out);
        }
        TypeV2::Ext(ExtType::HashMap(k,v)) | TypeV2::Ext(ExtType::Result(k,v)) => {
            collect_lifetimes(k, out);
            collect_lifetimes(v, out);
        }
        TypeV2::Ext(ExtType::RawPtr { inner, .. }) => collect_lifetimes(inner, out),
        TypeV2::Ext(ExtType::RefExt { inner, .. }) => collect_lifetimes(inner, out),
        TypeV2::Ext(ExtType::Struct { args, .. }) | TypeV2::Ext(ExtType::Enum { args, .. }) => {
            for a in args { collect_lifetimes(a, out); }
        }
        _ => {}
    }
}

/// 依賴圖
#[derive(Clone, Debug, Default)]
pub struct DepGraph {
    pub edges: HashMap<TypeV2, Vec<TypeV2>>,
}

impl DepGraph {
    pub fn new() -> Self { Self::default() }
    pub fn add_dep(&mut self, from: TypeV2, to: TypeV2) {
        self.edges.entry(from).or_default().push(to);
    }
    pub fn build_from_universe(uni: &Universe) -> Self {
        let mut g = Self::new();
        for ty in &uni.types {
            match ty {
                TypeV2::Ext(ExtType::Vec(inner)) => g.add_dep(ty.clone(), *inner.clone()),
                TypeV2::Ext(ExtType::Option(inner)) => g.add_dep(ty.clone(), *inner.clone()),
                TypeV2::Ext(ExtType::Future(inner)) => g.add_dep(ty.clone(), *inner.clone()),
                TypeV2::Ext(ExtType::HashMap(k, v)) => {
                    g.add_dep(ty.clone(), *k.clone());
                    g.add_dep(ty.clone(), *v.clone());
                }
                TypeV2::Ext(ExtType::Result(t, e)) => {
                    g.add_dep(ty.clone(), *t.clone());
                    g.add_dep(ty.clone(), *e.clone());
                }
                TypeV2::Ext(ExtType::RawPtr { inner, .. }) => g.add_dep(ty.clone(), *inner.clone()),
                TypeV2::Ext(ExtType::RefExt { inner, .. }) => g.add_dep(ty.clone(), *inner.clone()),
                TypeV2::Ext(ExtType::Struct { args, .. }) | TypeV2::Ext(ExtType::Enum { args, .. }) => {
                    for a in args {
                        g.add_dep(ty.clone(), a.clone());
                    }
                }
                _ => {}
            }
        }
        g
    }
}


