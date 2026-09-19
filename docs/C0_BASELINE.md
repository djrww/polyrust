# C0 基線記錄 — Charon+PolyIR 藍圖首里程碑（2026-09-20）

## 判定結果

| 指標 | 數值 | 口徑 |
|---|---|---|
| 出 LLBC | **117/152（77.0%）** | 藍團 C0 驗收線 ≥70% → **PASS（綠燈）** |
| 有效轉換率 | **117/136 = 86.0%** | 排除設計 UNSAT 16 例（phase3/\*\_unsat + examples/bad） |
| ok（零 missing） | 112 | |
| ok_with_missing | 5 | Charon `has_errors` 旗號標記 |
| rustc_reject | 35 | 見下分解 |
| charon_err / timeout | **0 / 0** | Charon 本身零崩潰、零逾時 |

## rustc_reject 35 例分解（抽樣實測確認 E-code）

| 類 | 例數級 | 代表 | 判定 |
|---|---|---|---|
| 設計 UNSAT | 16 | phase3/\*\_unsat、examples/bad（如 break_unsat E0268 `break` outside loop） | **判定正確**，呢啲存在就係畀引擎判 UNSAT 嘅 |
| DSL 溢出（sat/unknown 但 rustc-invalid） | ~13 | try_sat E0277（`?` 用喺非 Result fn）、vec_sat E0425（無 `use std::collections::HashMap;` / `Vec_new` 蛇形命名）、match_sat E0308 | polyrust DSL 容忍、真 Rust 唔過 → **語料-vs-Rust 誠實邊界**，C1 修剪；唔算 Charon 缺口 |
| 外部依賴 | 3+ | async_spawn E0433（無 tokio crate）、matrix/commercial 若干 | 單檔 rustc 無 Cargo deps → 合理 reject；C1 用 `charon cargo` 路線處理 |
| 語料自傷 | 2 | enum_color E0369（無 derive(PartialEq) 用 ==）、ref_deref E0614 | C1 修正語料或維持 UNSAT 註記 |
| Charon 真缺口 | **0（抽樣未見）** | — | 零崩潰；async_simple/async_await/loop/closure/lifetime 等全部成功出 LLBC |

## 方法（可重現）

1. 語料：`core/src/semantic_matrix.rs` 100 例 matrix + `examples/*.poly` 52 例（共 152）；先經 sanitizer（剝 `#` DSL 註釋行）。
2. 調用：`charon rustc -- <in.rs> --crate-type=rlib --edition=2021`（**C0 法證一**：舊 top-level `--crate-type` 被 clap 拒→首跑 152/152 全誤分；**法證二**：預設 edition 2015 令 async E0670，全類別假陰性）。
3. 分類口徑：rustc_reject 只認 `[E\d+]`；退出碼非零無 E-code → charon_err；`has_errors` → ok_with_missing ≥1。
4. 執行單元：GitHub Actions ubuntu-24.04（Charon pinned `ca501af6` 本地編譯 ~2min，`.github/workflows/c0-charon-spike.yml`）；release `c0-charon-spike` 雙資產（results + charon binary 9MB）。
5. 本地覆算：release 拉 binary 到沙箱（glibc 2.41 ✓），`python3 scripts/c0_spike.py --charon <bin>` 全量 33 秒，數字一致（77.0%）。

## 對藍圖的影響

- **C0 → C1 轉綠**：`docs/CHARON_POLYIR_BLUEPRINT.md` §4 門檻達成。C1 = PolyIR schema 設計 + LLBC→PolyIR 升降映射 + 語料修剪清單（上表 DSL 溢出 + 語料自傷類）。
- **TCB 結論佐證**：Charon 在 corpus 上零崩潰零逾時，唯一失收原因全部可歸因「輸入唔係合法單檔 Rust 2021」——「Charon 抽 MIR 可信、TCB 外延入賬」嘅藍圖論斷得到 152 例實測支持。
- **schema 修正遲早要**：`llbc_stats` 揭示新版 LLBC 結構 `{charon_version, translated{crate_name,files,type_decls,fun_decls,…}, has_errors}`；PolyIR/Charon 夾層 schema snapshot 以今次為 v1 基線。

## 快照

- 語料快照：semantic_matrix.rs + examples/ @ v0.2.3（含 Phase B，89/89 測試綠）
- pinned：charon `ca501af6b8abfc65cc67980cf312388de68b8c67`（`charon.pin`）
- 結果工件：release tag `c0-charon-spike`（每次 push c0-charon*/c0-spike* 自動重跑覆寫）
