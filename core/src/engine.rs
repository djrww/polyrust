// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! M1 — 單一判定入口。
//!
//! Surface（ast_v2 / pipeline_v2）只負責偵測與 lower。
//! 只要來源能被 Mini-Rust v1 `Parser` 吃進去，**判定必須來自**
//! `pipeline::run_pipeline`（CDCL × Buchberger），不得再用錯誤列表。
//!
//! 解析失敗則保持 `surface-errors`，JSON 必須標明 engine，禁止冒充命題 P。

use crate::minirust::parse::Parser;
use crate::pipeline::{run_pipeline_eager as run_pipeline, PipelineResult};

/// 哪一個判定器簽發了 `is_unsat`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// Kernel v1：`1 ∈ G` / SAT 模型。
    Kernel,
    /// Surface：borrowck / effect / lifetime 錯誤列表。尚未接到代數核。
    SurfaceErrors,
}

impl Engine {
    pub fn as_str(self) -> &'static str {
        match self {
            Engine::Kernel => "kernel",
            Engine::SurfaceErrors => "surface-errors",
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::SurfaceErrors
    }
}

/// 嘗試把來源交給 Kernel。`Ok(None)` 表示不是 v1 子集（應走 surface）。
pub fn try_kernel(name: &str, source: &str) -> Result<Option<PipelineResult>, String> {
    if Parser::parse_program(source).is_err() {
        return Ok(None);
    }
    match run_pipeline(name, source, false) {
        Ok(p) => Ok(Some(p)),
        Err(e) => Err(e),
    }
}

/// 來源是否落在 Kernel 語法裡（不跑 solver）。
pub fn is_kernel_subset(source: &str) -> bool {
    Parser::parse_program(source).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_let_is_kernel_subset() {
        let src = "fn main() { let x = 1; }";
        assert!(is_kernel_subset(src));
    }

    #[test]
    fn struct_is_not_kernel_subset() {
        let src = "struct Point { x: i32, y: i32 }\nfn main() { let p = Point { x: 1, y: 2 }; }";
        assert!(!is_kernel_subset(src));
    }
}
