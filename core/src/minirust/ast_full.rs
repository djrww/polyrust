// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Phase3 — 完整 AST：兼容 shim，實際定義已合併至 ast.rs（AST 補齊）
//! 保留 re-export 以兼容舊路徑。

pub use super::ast::{
    SpanInfo, Vis, Attr, Lifetime, GenericParam, Generics, WhereClause, TypeBound,
    FullType, FullPat, FullExpr, MatchArm, FullStmt, FullItem, FnItem, FnSig, FnInput,
    StructItem, StructFields, NamedField, EnumItem, EnumVariant, UnionItem, ImplItem, TraitItem,
    ModItem, UseItem, UseTree, ConstItem, StaticItem, TypeAliasItem, MacroItem,
    ExternBlockItem, FullProgram, FullParser, HandwrittenParser,
    AstStats, collect_stats_full, full_program_to_poly_code,
    // v2 互轉也需要
    ProgramV2, ItemV2,
};

// 為兼容舊代碼中 `From<FullProgram> for ProgramV2` 的路徑，
// 該 From impl 已在 ast.rs 中定義，此處無需重複。
