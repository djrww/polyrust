// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! PolyIR — Charon LLBC 同 polyrust 下半段（GB／QAP）之間嘅自家 IR（C1 骨架）。
//!
//! 定位（藍圖 §3 分層）：
//! ```text
//!   .rs ──charon──▶ LLBC ──charon_llbc.rs parse──▶ PolyIR ──lowering──▶ 多項式系統
//!        （C2: int/bool 直線+分支；C3: loop/fuel；C4: calls/contracts）
//! ```
//!
//! C1 只定 **結構 lift**：decl 清單 + body 可用性投影，語義 lowering 係 C2 起嘅貨。
//! 呢一層嘅硬承諾（對下游）：
//! - `PolyFun.body_kind == Structured` ⇒　C2 預期可降；`Error` ⇒　判定呈 UNKNOWN(missing_decl)，
//!   口徑同 C0 harness 對齊；`Missing` ⇒ 同 Error（聲明保留、body 缺失）。
//! - 任何 schema 漂移喺 parse 階段已爆（charon_llbc.rs hard error），PolyModule 唔會見到半形狀數據。

use crate::charon_llbc::{BodyKind, LlbcRoot};

/// 一個函數單元喺 PolyIR 嘅投影。
#[derive(Debug, Clone)]
pub struct PolyFun {
    pub def_id: i64,
    pub name: String,
    pub path: Vec<String>,
    pub body_kind: BodyKind,
}

#[derive(Debug, Clone)]
pub struct PolyModule {
    pub crate_name: String,
    pub funs: Vec<PolyFun>,
    pub n_types: usize,
    pub n_globals: usize,
    pub n_trait_decls: usize,
    pub n_trait_impls: usize,
    /// Charon 已報告有 decl 抽唔到（= C0 嘅 ok_with_missing）。
    pub has_missing: bool,
}

impl PolyModule {
    /// 可以進入 C2 lowering 嘅 fun（Structured body）。
    pub fn lowerable_funs(&self) -> impl Iterator<Item = &PolyFun> {
        self.funs.iter().filter(|f| f.body_kind == BodyKind::Structured)
    }
    /// 預計要報 UNKNOWN(missing_decl) 嘅 fun 數。
    pub fn missing_body_count(&self) -> usize {
        self.funs.iter().filter(|f| f.body_kind != BodyKind::Structured).count()
    }
}

/// LLBC root → PolyIR 結構 lift（C1：純投影，唔做語義）。
pub fn lift(root: &LlbcRoot) -> PolyModule {
    PolyModule {
        crate_name: root.crate_name.clone(),
        funs: root
            .funs
            .iter()
            .map(|f| PolyFun {
                def_id: f.def_id,
                name: f.name.clone(),
                path: f.full_path.clone(),
                body_kind: f.body_kind,
            })
            .collect(),
        n_types: root.n_type_decls,
        n_globals: root.n_global_decls,
        n_trait_decls: root.n_trait_decls,
        n_trait_impls: root.n_trait_impls,
        has_missing: root.has_errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charon_llbc::LlbcRoot;

    #[test]
    fn lift_sqr_fixture() {
        let root = LlbcRoot::parse(include_str!("../tests/charon_fixtures/sqr.llbc")).unwrap();
        let m = lift(&root);
        assert_eq!(m.crate_name, "input");
        assert_eq!(m.funs.len(), 1);
        assert!(m.funs[0].path.contains(&"sqr".to_string()));
        assert_eq!(m.lowerable_funs().count(), 1);
        assert_eq!(m.missing_body_count(), 0);
        assert!(!m.has_missing);
    }

    #[test]
    fn lift_async_fixture_marks_missing() {
        let root = LlbcRoot::parse(include_str!("../tests/charon_fixtures/async_simple.llbc")).unwrap();
        let m = lift(&root);
        assert!(m.has_missing);
        assert_eq!(m.missing_body_count(), 1, "async body=Error → 1 missing");
        assert_eq!(m.lowerable_funs().count(), 0);
    }
}
