//! Phase2 — 前端 ty.rs：TyV2 宇宙構造與統一，封裝 core 的 ty 模組
//! Phase3 — 擴展 lifetime 參數、outlives、泛型統一

use polyrust_core::minirust::ty::{build_universe_from_src, unify, TyEnv, UnifyResult, LifetimeEnv, check_lifetime_bounds, subst_type_with_lt};
use polyrust_core::minirust::universe::{TypeV2, Universe};

pub use polyrust_core::minirust::ty::{build_universe_from_program, subst_type, DepGraph};

/// 前端封裝：從源碼構建宇宙
pub fn build_universe(src: &str) -> Universe {
    build_universe_from_src(src)
}

/// 統一兩個類型，返回約束文本
pub fn unify_types(t1: &TypeV2, t2: &TypeV2, uni: &Universe) -> Result<Vec<String>, String> {
    match unify(t1, t2, uni) {
        UnifyResult::Same => Ok(vec![]),
        UnifyResult::NeedEq { idx1, idx2 } => {
            Ok(vec![format!("t_{} - t_{} =0 # unify {} vs {}", idx1, idx2, t1.name(), t2.name())])
        }
        UnifyResult::NeedEqs(pairs) => {
            Ok(pairs.into_iter().map(|(i1,i2)| format!("t_{} - t_{} =0", i1, i2)).collect())
        }
        UnifyResult::Fail(e) => Err(e),
    }
}

/// lifetime 感知統一
pub fn unify_with_lifetimes(t1: &TypeV2, t2: &TypeV2, uni: &Universe, lt_graph: &polyrust_core::minirust::lifetime::LifetimeGraph) -> Result<Vec<String>, String> {
    // 先檢查 lifetime bounds
    check_lifetime_bounds(t1, lt_graph).map_err(|e| format!("lifetime bound error for t1: {}", e))?;
    check_lifetime_bounds(t2, lt_graph).map_err(|e| format!("lifetime bound error for t2: {}", e))?;
    unify_types(t1, t2, uni)
}

/// 類型環境前端封裝
pub struct FrontendTyEnv {
    inner: TyEnv,
    lt_inner: LifetimeEnv,
}

impl FrontendTyEnv {
    pub fn new() -> Self { Self { inner: TyEnv::new(), lt_inner: LifetimeEnv::new() } }
    pub fn insert(&mut self, name: String, ty: TypeV2) { self.inner.insert(name, ty); }
    pub fn insert_lt(&mut self, name: String, lt: String) { self.lt_inner.insert(name, lt); }
    pub fn get(&self, name: &str) -> Option<&TypeV2> { self.inner.get(name) }
    pub fn get_lt(&self, name: &str) -> Option<&String> { self.lt_inner.get(name) }
    pub fn inner(&self) -> &TyEnv { &self.inner }
    pub fn lt_inner(&self) -> &LifetimeEnv { &self.lt_inner }
    pub fn subst_with_lt(&self, ty: &TypeV2) -> TypeV2 {
        subst_type_with_lt(ty, &self.inner, &self.lt_inner)
    }
}

impl Default for FrontendTyEnv {
    fn default() -> Self { Self::new() }
}

/// 估算並展示宇宙
pub fn display_universe(src: &str) -> String {
    let uni = build_universe(src);
    uni.display()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_universe_frontend() {
        let src = r#"
            struct Point { x: i32, y: i32 }
            fn foo(v: Vec<Point>) -> Option<Point> {}
        "#;
        let uni = build_universe(src);
        assert!(uni.n_types() > 7);
        println!("{}", uni.display());
    }

    #[test]
    fn test_unify_frontend() {
        let mut uni = Universe::new();
        let t1 = polyrust_core::minirust::universe::parse_type_v2("Vec<T>").unwrap();
        let t2 = polyrust_core::minirust::universe::parse_type_v2("Vec<i32>").unwrap();
        uni.insert_closure(t1.clone());
        uni.insert_closure(t2.clone());
        let res = unify_types(&t1, &t2, &uni).unwrap();
        assert!(!res.is_empty());
        println!("{:?}", res);
    }

    #[test]
    fn test_lifetime_env() {
        let mut env = FrontendTyEnv::new();
        env.insert_lt("'a".to_string(), "'b".to_string());
        let ty = polyrust_core::minirust::universe::parse_type_v2("&'a i32").unwrap();
        let substed = env.subst_with_lt(&ty);
        println!("substed: {:?}", substed);
        match substed {
            TypeV2::Ext(polyrust_core::minirust::universe::ExtType::RefExt { lifetime: Some(lt), .. }) => assert_eq!(lt, "'b"),
            _ => panic!("should be ref with lifetime 'b"),
        }
    }
}
