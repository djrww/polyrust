//! polyrust-core —— Mini-Rust 宏的代數形式化管線核心
//!
//! CDCL（子句學習）× Buchberger（Gröbner 基化簡）× QAP（算術程序驗證）。
//!
//! **零第三方依賴**：本 crate 只用 `std`。任何第三方依賴都屬於
//! `frontends/*`（見倉庫根 `Cargo.toml`）。
//!
//! 主要入口：
//! * [`pipeline::run_pipeline`] —— 完整代數管線（約束生成 → CDCL(T) → Buchberger → QAP）
//! * [`llm::run_guardrail`] —— 自然語言 → .poly 護欄（三道閘門＋修復回餵）
//! * [`driver`] —— 各子命令（check/expand/gen/nl/funnel/exhaust/brute）
//! * [`formal`] —— Lean 4 形式化庫的靜態嵌入橋（有無工具鏈皆可編譯）

pub mod brute;
pub mod cdcl;
pub mod codegen;
pub mod driver;
pub mod dsl;
pub mod exhaust;
pub mod formal;
pub mod fp;
pub mod frac;
pub mod groebner;
pub mod json;
pub mod llm;
pub mod minirust;
pub mod obligations;
pub mod pipeline;
pub mod pipeline_v2;
pub mod poly;
pub mod qap;
pub mod server;
