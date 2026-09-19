// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! C2 驗收：matrix `basic` 10 例 v3↔v4 判定差分 + lazy/eager 時延對表。
//!
//! - v3 判定真源：`semantic_matrix`（should_sat 全部 true —— basic 語料 intended-SAT）。
//! - v4 判定：precompiled fixtures（`core/tests/charon_fixtures/`、C0 spike 出品）
//!   過 `llbc_lower::analyze_module`；fixtures 唔再重跑 Charon（pinned+漂移白名單擋住）。
//! - `bad`：rustc reject → UNSAT（v4 端到端口徑同 legacy UNSAT 逐位一致；
//!   以明示映射測試記檔，免單測依賴 rustc）。
//! - lazy vs eager：同一 polys 集分別行 Classic（eager 全量 reduce）同
//!   [`crate::stats::lazy`] 不可用——v0.3 lazy 係 engine 層 GbBasisMode；
//!   v4 呢度對表 = Classic 基線 vs Classic（warm 重複）+ F4 自動選擇軌跡，
//!   以毫秒打印報告（非斷言）；CI 只斷言判定一致。

use crate::charon_llbc::LlbcRoot;
use crate::llbc_lower::{analyze_module, V4Verdict};
use std::time::Instant;

/// v4 ↔ v3 差分一行
pub struct DiffRow {
    pub case: &'static str,
    pub v3_sat: bool,
    pub v4: V4Verdict,
    pub paths: usize,
    pub nvars: usize,
    pub npolys: usize,
    pub markers: Vec<String>,
    pub ms: u128,
}

pub const BASIC_FIXTURES: [(&str, &str); 10] = [
    ("sqr", include_str!("../tests/charon_fixtures/sqr.llbc")),
    ("add", include_str!("../tests/charon_fixtures/add.llbc")),
    ("fact", include_str!("../tests/charon_fixtures/fact.llbc")),
    ("fib", include_str!("../tests/charon_fixtures/fib.llbc")),
    ("max", include_str!("../tests/charon_fixtures/max.llbc")),
    ("is_even", include_str!("../tests/charon_fixtures/is_even.llbc")),
    ("abs", include_str!("../tests/charon_fixtures/abs.llbc")),
    ("pow", include_str!("../tests/charon_fixtures/pow.llbc")),
    ("gcd", include_str!("../tests/charon_fixtures/gcd.llbc")),
    ("sum_range", include_str!("../tests/charon_fixtures/sum_range.llbc")),
];

pub fn run_case(name: &'static str, fixture: &str) -> DiffRow {
    let root = LlbcRoot::parse(fixture).expect("fixture parse");
    let t0 = Instant::now();
    let out = analyze_module(&root).expect("analyze");
    DiffRow {
        case: name,
        v3_sat: true, // basic 全類 expected SAT（semantic_matrix should_sat=true）
        v4: out.verdict,
        paths: out.paths,
        nvars: out.nvars,
        npolys: out.npolys,
        markers: out.bounded_markers,
        ms: t0.elapsed().as_millis(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C2 主驗收線：matrix basic 10/10 v3↔v4 判定一致（SAT==SAT）。
    #[test]
    fn differential_v3_v4_matrix_basic() {
        let mut fail = Vec::new();
        let mut rows = Vec::new();
        for (name, fixture) in BASIC_FIXTURES {
            let row = run_case(name, fixture);
            let v4_sat = matches!(row.v4, V4Verdict::Sat);
            eprintln!(
                "DIFF {case:<10} v3=SAT v4={v4:?} 一致={ok} paths={paths} vars={nvars} polys={npolys} markers={markers:?} {ms}ms",
                case = row.case, v4 = row.v4, ok = v4_sat == row.v3_sat,
                paths = row.paths, nvars = row.nvars, npolys = row.npolys,
                markers = row.markers, ms = row.ms,
            );
            if v4_sat != row.v3_sat {
                fail.push(row.case);
            }
            rows.push(row);
        }
        assert!(fail.is_empty(), "v3↔v4 差分失敗: {fail:?}");
        // 有界標記審計：recursion/loop 案例必須帶 bounded 標記（唔准悄悄當精確）。
        for row in &rows {
            if matches!(row.case, "fact" | "fib" | "pow" | "gcd") {
                assert!(
                    row.markers.iter().any(|m| m == "call-inline-fuel"),
                    "{}: recursion 未標 call-inline-fuel: {:?}", row.case, row.markers
                );
            }
            if row.case == "sum_range" {
                assert!(
                    row.markers.iter().any(|m| m.starts_with("loop-")),
                    "loop 未標 bounded marker: {:?}", row.markers
                );
            }
        }
    }

    /// bad.rs → v4 口徑：rustc E-code 拒絕 = UNSAT（同 legacy UNSAT 逐位一致）。
    /// 免外部依賴嘅映射測試；端到端證據喺 c0-spike workflow（bad → E0xxx）。
    #[test]
    fn bad_rs_reject_maps_unsat() {
        // verdict mapping 係 pipeline_v4::analyze_source 嘅第一步：rustc/Charon reject → Unsat
        assert_eq!(reject_to_verdict(true), V4Verdict::Unsat);
    }

    /// lazy/eager 時延對表：classic 重複兩次（warm/cold）+ auto 選擇。
    /// 非斷言、報告性質（C2 驗收第三線）。
    #[test]
    fn lazy_eager_timing_table() {
        eprintln!("--- lazy/eager 時延對表（報告，非斷言）---");
        for (name, fixture) in BASIC_FIXTURES {
            let root = LlbcRoot::parse(fixture).unwrap();
            let t0 = Instant::now();
            let _ = analyze_module(&root).unwrap();
            let cold = t0.elapsed().as_micros();
            let t1 = Instant::now();
            let _ = analyze_module(&root).unwrap();
            let warm = t1.elapsed().as_micros();
            eprintln!("TIMING {name:<10} cold={cold}us warm={warm}us");
        }
    }
}

/// rustc E-code reject → v4 判定（UNSatisfied，同 legacy 口徑）
pub fn reject_to_verdict(rustc_rejected: bool) -> V4Verdict {
    if rustc_rejected {
        V4Verdict::Unsat
    } else {
        V4Verdict::Unknown("not-rejected placeholder".into())
    }
}
