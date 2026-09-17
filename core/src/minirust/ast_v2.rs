//! Phase1 — 擴展 AST v2：兼容 shim，實際定義已合併至 ast.rs（AST 補齊）
//! 本文件保留為 re-export，以兼容舊 import 路徑。

pub use super::ast::{
    StructDefV2, UnionDefV2, VariantV2, EnumDefV2, FnSigV2, FnDefV2, ImplDefV2, TraitDefV2,
    ModDefV2, ConstDefV2, StaticDefV2, TypeAliasDefV2, ItemV2, ProgramV2,
    collect_stats_v2, program_v2_to_poly_code,
};
