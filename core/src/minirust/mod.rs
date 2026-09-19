// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Mini-Rust 前端：詞法、語法、宏系統（macro_rules! + 衛生）、
//! 借用活性分析、直接型別檢查器（ground truth）、代數約束生成。
//! Phase1 擴展：型別宇宙 7+i（universe）、擴展 AST v2
//! Phase2 擴展：ty.rs (統一)、lower.rs (降維)、constraints_v2.rs (可變 N)

pub mod analysis;
pub mod ast;
pub mod ast_v2;
pub mod ast_full;
pub mod checker;
pub mod constraints;
pub mod constraints_v2;
pub mod lexer;
pub mod lower;
pub mod macros;
pub mod parse;
pub mod parse_pat;
pub mod parse_expr;
pub mod ty;
pub mod universe;
pub mod parse_v2;
pub mod parse_full;
// Phase3
pub mod borrowck;
pub mod contracts;
pub mod effects;
pub mod lifetime;
pub mod mir_lower; // MIR 前端第三條 lowering：消失多項式 + L0′
pub mod trait_impl;
pub mod stdlib;
pub mod async_qap;
pub mod unsafe_safety;
