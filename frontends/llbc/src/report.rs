// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! `polyrust-llbc-check` 判定報告（v4/v1 統一 JSON 契約）＋ exit code 映射。
//! 渲染邏輯喺 lib（可測），CLI binary 只做 IO + charon 調度。

use polyrust_core::llbc_lower::{V4Outcome, V4Verdict};

/// v4 outcome → 統一 JSON 報告。
pub fn v4_report(engine: &str, input: &str, src: &str, ms: u128, out: &V4Outcome) -> serde_json::Value {
    let (verdict, reason) = match &out.verdict {
        V4Verdict::Sat => ("Sat", serde_json::Value::Null),
        V4Verdict::Unsat => ("Unsat", serde_json::Value::Null),
        V4Verdict::Unknown(r) => ("Unknown", serde_json::Value::String(r.clone())),
    };
    serde_json::json!({
        "api_version": "0.3",
        "tool": "polyrust-llbc-check",
        "engine": engine,
        "input": input,
        "status": "ok",
        "verdict": verdict,
        "reason": reason,
        "fun_name": out.fun_name,
        "paths": out.paths,
        "nvars": out.nvars,
        "npolys": out.npolys,
        "markers": out.bounded_markers,
        "assert_obligations": out.assert_obligations,
        "contract_report": out.contract_report,
        "gb_stats": out.gb_stats,
        "fuel_annotations": crate::fuel_of(src),
        "ms": ms as u64,
    })
}

/// exit code：Sat=0、Unsat=1、Unknown=2、執行錯誤=3（CLI 契約）。
pub fn exit_code(v: &V4Verdict) -> u8 {
    match v {
        V4Verdict::Sat => 0,
        V4Verdict::Unsat => 1,
        V4Verdict::Unknown(_) => 2,
    }
}

/// 錯誤 JSON（status:error）——無論邊層失敗都統一出口。
pub fn error_report(tool: &str, input: &str, stage: &str, msg: &str) -> serde_json::Value {
    serde_json::json!({
        "api_version": "0.3",
        "tool": tool,
        "input": input,
        "status": "error",
        "stage": stage,
        "reason": msg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyrust_core::llbc_lower::analyze_module_from_source;

    /// C5.1 端到端（唔經 charon）：fixture LLBC × 合約源碼 → Sat + 合約報告。
    #[test]
    fn sqr_with_contract_end_to_end() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../core/tests/charon_fixtures/sqr.llbc");
        let root = crate::parse_llbc_file(&path).expect("sqr parse");
        // sign 驗證用：真 rustc 驗收級源碼形態（含 @fuel 合約註解行）
        let src = "# @intent square\n# @require x == 3\n# @ensure result == x * x\nfn sqr(x: i32) -> i32 { x * x }";
        let out = analyze_module_from_source(&root, src).expect("analyze");
        assert_eq!(out.verdict, V4Verdict::Sat);
        assert!(!out.contract_report.is_empty(), "contract_report 要有東西：{out:?}");
        let j = v4_report("v4", "sqr.llbc", src, 1, &out);
        assert_eq!(j["verdict"], "Sat");
        assert_eq!(exit_code(&out.verdict), 0);
        assert!(j["assert_obligations"].as_array().is_some());
    }

    #[test]
    fn unknown_maps_to_exit_2() {
        assert_eq!(exit_code(&V4Verdict::Unknown("fuel".into())), 2);
        assert_eq!(exit_code(&V4Verdict::Unsat), 1);
    }
}
