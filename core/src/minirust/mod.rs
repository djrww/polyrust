//! Mini-Rust 前端：詞法、語法、宏系統（macro_rules! + 衛生）、
//! 借用活性分析、直接型別檢查器（ground truth）、代數約束生成。

pub mod analysis;
pub mod ast;
pub mod checker;
pub mod constraints;
pub mod lexer;
pub mod macros;
pub mod parse;
