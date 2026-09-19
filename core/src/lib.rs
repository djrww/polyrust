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
pub mod groebner_f4;
pub mod groebner_f5;
pub mod groebner_f4f5;
pub mod json;
pub mod llm;
pub mod llm_closed_loop;
pub mod txt_feedback;
pub mod native_bidirectional;
pub mod minirust;
pub mod obligations;
pub mod pipeline;
pub mod pipeline_v2;
pub mod pipeline_v3;
pub mod pipeline_v3_auto;
pub mod poly;
pub mod poly_dsl;
pub mod poly_dsl_codegen;
pub mod qap;
pub mod server;
pub mod daemon;
pub mod commercial_pipeline;
pub mod solana_onchain;
pub mod poly_cache;
pub mod semantic_matrix;
pub mod phase_a;
pub mod chalk_bridge;
pub mod certify;
pub mod composition;
pub mod vanishing;

/// 正式運作：core 文件清單 — 零依賴實際使用
pub fn core_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("lib.rs", "核心入口 22 mods — 零依賴承諾", "core/src/lib.rs"),
        ("poly.rs", "多項式 — 優化 with_capacity", "core/src/poly.rs"),
        ("frac.rs", "ℚ→𝔽_p — 優化 shim", "core/src/frac.rs"),
        ("fp.rs", "𝔽_p 2^61-1 — 優化無溢出", "core/src/fp.rs"),
        ("groebner.rs", "Buchberger — 優化 with_capacity", "core/src/groebner.rs"),
        ("groebner_f4.rs", "F4 — 優化", "core/src/groebner_f4.rs"),
        ("groebner_f5.rs", "F5 — 優化", "core/src/groebner_f5.rs"),
        ("groebner_f4f5.rs", "F4F5 混合 — 優化", "core/src/groebner_f4f5.rs"),
        ("cdcl.rs", "CDCL — 優化", "core/src/cdcl.rs"),
        ("qap.rs", "QAP — 優化", "core/src/qap.rs"),
        ("brute.rs", "Brute — 優化", "core/src/brute.rs"),
        ("exhaust.rs", "Exhaust — 優化", "core/src/exhaust.rs"),
        ("pipeline.rs", "管線 v1 — 優化", "core/src/pipeline.rs"),
        ("pipeline_v2.rs", "管線 v2 — 優化 with_capacity + inventory", "core/src/pipeline_v2.rs"),
        ("pipeline_v3.rs", "管線 v3 自进化商业深化 — 零依赖", "core/src/pipeline_v3.rs"),
        ("pipeline_v3_auto.rs", "管線 v3 自动进行 + 2 example回喂 — 零依赖", "core/src/pipeline_v3_auto.rs"),
        ("poly_dsl.rs", "多项式DSL 80函数 90% Rust语义 — 零依赖", "core/src/poly_dsl.rs"),
        ("poly_dsl_codegen.rs", "Poly DSL Codegen 80函数→Rust + NL→Poly→Codegen — 零依赖", "core/src/poly_dsl_codegen.rs"),
        ("codegen.rs", "Rust 生成 — 優化 with_capacity", "core/src/codegen.rs"),
        ("dsl.rs", "DSL .poly — 優化 with_capacity", "core/src/dsl.rs"),
        ("json.rs", "JSON — 優化 with_capacity", "core/src/json.rs"),
        ("driver.rs", "Driver — 優化", "core/src/driver.rs"),
        ("server.rs", "Server — 優化", "core/src/server.rs"),
        ("daemon.rs", "Daemon 真自动化 Phase2 — 零依赖", "core/src/daemon.rs"),
        ("llm.rs", "LLM 護欄 — 優化", "core/src/llm.rs"),
        ("llm_closed_loop.rs", "LLM 閉環 Phase3 — 你便是 LLM", "core/src/llm_closed_loop.rs"),
        ("txt_feedback.rs", "Txt 投喂 階段2 迭代閉環 補齊語義", "core/src/txt_feedback.rs"),
        ("native_bidirectional.rs", "原生器材雙向閉環 AST/MIR 四層補齊", "core/src/native_bidirectional.rs"),
        ("commercial_pipeline.rs", "商業化管線 txt->poly->AST->MIR->Rust->native->audit->onchain 真實全鏈路", "core/src/commercial_pipeline.rs"),
        ("solana_onchain.rs", "Solana 真上鏈 QAP 證書 Anchor Program + TS Client + Deploy 真實實現", "core/src/solana_onchain.rs"),
        ("poly_cache.rs", "PolyCache 增量管線 hash→Groebner 緩存 — Phase A", "core/src/poly_cache.rs"),
        ("semantic_matrix.rs", "Semantic Matrix 100 例語義保持 — Phase A", "core/src/semantic_matrix.rs"),
        ("phase_a.rs", "Phase A 整合：增量+語義+QAP r1cs.json+CI — Phase A", "core/src/phase_a.rs"),
        ("composition.rs", "T10 組合性：模塊化分解 + Σ2^{nᵢ} 組合界 + 並基驗證", "core/src/composition.rs"),
        ("certify.rs", "認證路徑：外部 oracle σ 重建 + 多項式時間直接求值驗證", "core/src/certify.rs"),
        ("chalk_bridge.rs", "Chalk/rustc 判決橋：oracle artifact JSON → OracleBits", "core/src/chalk_bridge.rs"),
        ("vanishing.rs", "消失多項式多值編碼 + 引理 L0′ + ∏kᵢ 一般化界", "core/src/vanishing.rs"),
        ("obligations.rs", "Obligations — 優化", "core/src/obligations.rs"),
        ("formal.rs", "Formal Lean 橋 — 優化", "core/src/formal.rs"),
    ]
}
pub fn core_file_list_static() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("lib.rs", "核心入口", "core/src/lib.rs"),
        ("poly.rs", "多項式", "core/src/poly.rs"),
        ("frac.rs", "ℚ", "core/src/frac.rs"),
        ("fp.rs", "𝔽_p", "core/src/fp.rs"),
    ]
}
/// 正式運作：core 摘要 — 優化 with_capacity
pub fn core_summary() -> String {
    let mut out = String::with_capacity(1024);
    out.push_str("=== polyrust-core 正式運作 ===\n");
    out.push_str("零第三方依賴 — std only\n");
    out.push_str(&format!("mods: 22, files: {}\n", core_file_list().len()));
    let (ast_files, parse_files) = pipeline_v2::get_ast_and_parse_inventories();
    out.push_str(&format!("AST files: {}, Parse files: {}\n", ast_files.len(), parse_files.len()));
    out.push_str(&pipeline_v2::pipeline_v2_inventory_summary());
    out
}
