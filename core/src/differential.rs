// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! v1↔v3 管線差分測試（DEV_PLAN_V03 P0-C1）
//!
//! 對同一批語義樣本同時跑 v1（`pipeline::run_pipeline`，Lazy 默認）與
//! v3（`pipeline_v3::run_pipeline_v3_with_config`），斷言**判定一致**。
//! 規則同 semantic_matrix 嘅 ratchet：
//!   新差異 = 回歸 = CI 紅；修好管線缺口後從 KNOWN_DIVERGENCES 刪行。
//! v1 Err（解析不支援）與 v3 Err 另外逐類統計——屬適用域差距而非判定差異，
//! 以 ratchet 獨立追蹤（見 DIFF_V1_ERR_ALLOWLIST / DIFF_V3_ERR_ALLOWLIST）。

use crate::pipeline;
use crate::pipeline_v3::{PipelineV3Config, run_pipeline_v3_with_config};
use crate::semantic_matrix::all_semantic_cases;

#[derive(Debug)]
pub struct DiffRow {
    pub name: &'static str,
    pub category: &'static str,
    pub expect_sat: bool,
    /// None = v1 Err / panic
    pub v1_unsat: Option<bool>,
    /// None = v3 Err
    pub v3_unsat: Option<bool>,
}

impl DiffRow {
    pub fn is_divergence(&self) -> bool {
        matches!((self.v1_unsat, self.v3_unsat), (Some(a), Some(b)) if a != b)
    }
}

/// v1 適用域正規化（語義保持）：
/// v1 parser 要求 (a) 存在 `fn main()`，(b) 每個非 main fn 帶 `-> Type`。
/// 對 unit 回傳函數補 `-> ()`、缺 main 時附加空 main——Rust 語義完全等價。
/// 差分雙方跑**同一份正規化源**，確保比較係判定而非輸入格式。
fn normalize_for_v1(src: &str) -> String {
    // 1) 收集需要插入 ` -> ()` 嘅位置（fn 頭 `)` 之後下一非空白非 `->` 非 `where`）
    let bytes = src.as_bytes();
    let mut inserts: Vec<usize> = vec![];
    let mut i = 0usize;
    while i + 2 < bytes.len() {
        // bytes 比較取代 &str 切片：任意 byte index 於多字節 UTF-8 源會 panic
        if bytes[i] == b'f' && bytes[i + 1] == b'n' && bytes[i + 2] == b' ' {
            let mut j = i + 3;
            while j < bytes.len() && bytes[j] != b'(' && bytes[j] != b'{' && bytes[j] != b';' {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'(' {
                let mut d = 0usize;
                let mut k = j;
                while k < bytes.len() {
                    match bytes[k] {
                        b'(' => d += 1,
                        b')' => {
                            d -= 1;
                            if d == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    k += 1;
                }
                if k < bytes.len() {
                    let mut m = k + 1;
                    while m < bytes.len() && bytes[m].is_ascii_whitespace() {
                        m += 1;
                    }
                    let rest = &src[m..];
                    if !rest.starts_with("->") && !rest.starts_with("where") {
                        // 函數體以 '{' 起 → unit 回傳；補顯式註解
                        inserts.push(k + 1);
                    }
                    i = k;
                }
            }
        }
        i += 1;
    }
    let mut out = src.to_string();
    // 由後往前插，避免位移
    for pos in inserts.iter().rev() {
        out.insert_str(*pos, " -> ()");
    }
    // 2) 附加空 main（v1 入口約束）
    if !out.contains("fn main") {
        out.push_str("\nfn main() { let _polyrust_v1_anchor = 0; }\n");
    }
    out
}

pub fn run_differential() -> Vec<DiffRow> {
    let config = PipelineV3Config::default();
    all_semantic_cases()
        .iter()
        .map(|case| {
            let norm = normalize_for_v1(case.poly_src);
            // v1：鏡像 driver::cmd_check —— dsl::resolve + run_pipeline（Lazy）
            let v1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::dsl::resolve(&norm, None).and_then(|poly| {
                    pipeline::run_pipeline(case.name, &poly.source, false)
                })
            }));
            let v1_unsat = match v1 {
                Ok(Ok(r)) => Some(r.is_unsat),
                _ => None,
            };
            // v3：同一份正規化源
            let v3 = run_pipeline_v3_with_config(case.name, &norm, None, &config);
            let v3_unsat = match v3 {
                Ok(r) => Some(r.final_is_unsat),
                Err(_) => None,
            };
            DiffRow {
                name: case.name,
                category: case.category,
                expect_sat: case.should_sat,
                v1_unsat,
                v3_unsat,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判定差異白名單（v1≠v3）。修好即刪行，絕不新增。
    const KNOWN_DIVERGENCES: &[(&str, &str)] = &[
        ("io_effect", "v1 保守拒收 I/O（pipeline_v1 全貌下 io effect 觸發 UNSAT）；v3 與 rustc/矩陣口徑 SAT 一致 → v1 過嚴，修 v1 屬 P2"),
    ];

    /// v1 Err（解析/適用域差距）白名單——v1 parser 不支援嘅構成子
    /// （struct/enum/trait/Vec/String/match/loop/raw ptr/unsafe/async/commercial…）。
    /// v1→v3 遷移係既定架構方向；v1 parser 擴闊（P2）時逐行下沉。
    /// 基線 2026-09-19：86/100 在 v1 適用域外；可比較 14 例中除 io_effect 全數一致。
    const KNOWN_V1_ERR_REASON: &str = "v1 parser 適用域外（v1 僅支援核心算術/借用子集；擴 parser 屬 P2）";
    const KNOWN_V1_ERR: &[(&str, &str)] = &[
        ("max", KNOWN_V1_ERR_REASON), ("is_even", KNOWN_V1_ERR_REASON),
        ("gcd", KNOWN_V1_ERR_REASON), ("sum_range", KNOWN_V1_ERR_REASON),
        ("struct_point", KNOWN_V1_ERR_REASON), ("enum_option", KNOWN_V1_ERR_REASON),
        ("enum_result", KNOWN_V1_ERR_REASON), ("struct_rect", KNOWN_V1_ERR_REASON),
        ("impl_methods", KNOWN_V1_ERR_REASON), ("trait_display", KNOWN_V1_ERR_REASON),
        ("generic_struct", KNOWN_V1_ERR_REASON), ("enum_list", KNOWN_V1_ERR_REASON),
        ("struct_nested", KNOWN_V1_ERR_REASON), ("enum_color", KNOWN_V1_ERR_REASON),
        ("struct_with_enum", KNOWN_V1_ERR_REASON), ("impl_trait", KNOWN_V1_ERR_REASON),
        ("struct_default", KNOWN_V1_ERR_REASON), ("enum_with_data", KNOWN_V1_ERR_REASON),
        ("struct_methods_chain", KNOWN_V1_ERR_REASON),
        ("vec_new", KNOWN_V1_ERR_REASON), ("vec_len", KNOWN_V1_ERR_REASON),
        ("vec_get", KNOWN_V1_ERR_REASON), ("vec_iter_sum", KNOWN_V1_ERR_REASON),
        ("string_from", KNOWN_V1_ERR_REASON), ("string_len", KNOWN_V1_ERR_REASON),
        ("string_concat", KNOWN_V1_ERR_REASON), ("hashmap_new", KNOWN_V1_ERR_REASON),
        ("hashmap_get", KNOWN_V1_ERR_REASON), ("vec_map", KNOWN_V1_ERR_REASON),
        ("string_chars", KNOWN_V1_ERR_REASON), ("vec_filter", KNOWN_V1_ERR_REASON),
        ("hashmap_len", KNOWN_V1_ERR_REASON), ("string_split", KNOWN_V1_ERR_REASON),
        ("vec_sort", KNOWN_V1_ERR_REASON),
        ("loop_break", KNOWN_V1_ERR_REASON), ("while_loop", KNOWN_V1_ERR_REASON),
        ("for_range", KNOWN_V1_ERR_REASON), ("match_int", KNOWN_V1_ERR_REASON),
        ("match_bool", KNOWN_V1_ERR_REASON), ("loop_continue", KNOWN_V1_ERR_REASON),
        ("nested_loop", KNOWN_V1_ERR_REASON), ("match_guard", KNOWN_V1_ERR_REASON),
        ("loop_return", KNOWN_V1_ERR_REASON), ("while_let", KNOWN_V1_ERR_REASON),
        ("match_option", KNOWN_V1_ERR_REASON), ("match_result", KNOWN_V1_ERR_REASON),
        ("for_enumerate", KNOWN_V1_ERR_REASON), ("loop_invariant", KNOWN_V1_ERR_REASON),
        ("match_tuple", KNOWN_V1_ERR_REASON),
        ("move_semantic", KNOWN_V1_ERR_REASON), ("lifetime_simple", KNOWN_V1_ERR_REASON),
        ("nll_region", KNOWN_V1_ERR_REASON), ("outlives", KNOWN_V1_ERR_REASON),
        ("borrow_in_loop", KNOWN_V1_ERR_REASON), ("move_closure", KNOWN_V1_ERR_REASON),
        ("lifetime_struct", KNOWN_V1_ERR_REASON), ("borrow_match", KNOWN_V1_ERR_REASON),
        ("mut_borrow_split", KNOWN_V1_ERR_REASON), ("lifetime_elision", KNOWN_V1_ERR_REASON),
        ("raw_ptr_valid", KNOWN_V1_ERR_REASON), ("raw_ptr_box", KNOWN_V1_ERR_REASON),
        ("static_mut_exclusive", KNOWN_V1_ERR_REASON), ("static_mut_mutex", KNOWN_V1_ERR_REASON),
        ("union_tag", KNOWN_V1_ERR_REASON), ("unsafe_fn_precond", KNOWN_V1_ERR_REASON),
        ("unsafe_trait_invariant", KNOWN_V1_ERR_REASON), ("raw_ptr_missing_src", KNOWN_V1_ERR_REASON),
        ("static_mut_missing", KNOWN_V1_ERR_REASON), ("union_missing", KNOWN_V1_ERR_REASON),
        ("async_simple", KNOWN_V1_ERR_REASON), ("async_await", KNOWN_V1_ERR_REASON),
        ("async_spawn", KNOWN_V1_ERR_REASON), ("future_combinator", KNOWN_V1_ERR_REASON),
        ("io_file", KNOWN_V1_ERR_REASON), ("async_loop", KNOWN_V1_ERR_REASON),
        ("pure_no_io", KNOWN_V1_ERR_REASON),
        ("password_gen", KNOWN_V1_ERR_REASON), ("text_buffer", KNOWN_V1_ERR_REASON),
        ("file_tree", KNOWN_V1_ERR_REASON), ("reactive_ui", KNOWN_V1_ERR_REASON),
        ("enterprise_ide", KNOWN_V1_ERR_REASON), ("defi_audit", KNOWN_V1_ERR_REASON),
        ("embedded_cert", KNOWN_V1_ERR_REASON), ("llm_guardrail", KNOWN_V1_ERR_REASON),
        ("web3_audit", KNOWN_V1_ERR_REASON), ("self_evolving", KNOWN_V1_ERR_REASON),
    ];

    /// v3 Err 白名單。
    const KNOWN_V3_ERR: &[(&str, &str)] = &[];

    fn contains(list: &[(&str, &str)], name: &str) -> bool {
        list.iter().any(|(k, _)| k == &name)
    }

    #[test]
    fn differential_v1_v3_ratchet() {
        let rows = run_differential();
        let mut new_div = vec![];
        let mut new_v1_err = vec![];
        let mut new_v3_err = vec![];
        let mut fixed = vec![];
        for r in &rows {
            match (r.v1_unsat, r.v3_unsat) {
                (Some(a), Some(b)) => {
                    if a != b && !contains(KNOWN_DIVERGENCES, r.name) {
                        new_div.push(format!(
                            "{} [{}]: v1_unsat={} v3_unsat={} expect_sat={}",
                            r.name, r.category, a, b, r.expect_sat
                        ));
                    }
                }
                (None, _) => {
                    if !contains(KNOWN_V1_ERR, r.name) {
                        new_v1_err.push(format!("{} [{}]", r.name, r.category));
                    }
                }
                (_, None) => {
                    if !contains(KNOWN_V3_ERR, r.name) {
                        new_v3_err.push(format!("{} [{}]", r.name, r.category));
                    }
                }
            }
        }
        // 白名單失效偵測（修好提醒）
        for (k, r) in KNOWN_DIVERGENCES {
            if rows.iter().any(|row| &row.name == k && !row.is_divergence()) {
                fixed.push(format!("✅ 差異已修好（請刪行）：{} — {}", k, r));
            }
        }
        let n_case = rows.len();
        let n_divv = rows.iter().filter(|r| r.is_divergence()).count();
        let n_v1e = rows.iter().filter(|r| r.v1_unsat.is_none()).count();
        let n_v3e = rows.iter().filter(|r| r.v3_unsat.is_none()).count();
        println!(
            "differential: {} cases, {} divergences (whitelist {}), v1_err {}, v3_err {}",
            n_case,
            n_divv,
            KNOWN_DIVERGENCES.len(),
            n_v1e,
            n_v3e
        );
        for f in &fixed {
            println!("{}", f);
        }
        assert!(
            new_div.is_empty(),
            "v1↔v3 出現**新**判定差異（回歸）：\n{}",
            new_div.join("\n")
        );
        assert!(
            new_v1_err.is_empty() && new_v3_err.is_empty(),
            "v1/v3 Err 白名單外出現新案例：\n[新 v1_err]\n{}\n[新 v3_err]\n{}",
            new_v1_err.join("\n"), new_v3_err.join("\n")
        );
    }
}
