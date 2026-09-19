# 測試矩陣（M0 凍結）

閘門分四級。未標「硬」的項目不得擋住 merge，但也不得從 README 消失。

## A. Kernel 硬閘門（必須綠才能稱 v1 完好）

| ID | 命令 | 期望 | 阻斷 |
|---|---|---|---|
| K1 | `cargo test --release -p polyrust-core --lib -- --skip brute --skip exhaust` | 全過 | 硬 |
| K2 | `cargo test --release -p polyrust-core --lib brute exhaust` | 全過（慢） | 硬（可 nightly） |
| K3 | `cargo build --release -p polyrust-core` 且 `test -x target/release/polyrust` | 產出 CLI | 硬 |
| K4 | `./target/release/polyrust demo` | demo A–D：SAT 路徑 QAP✓ 篡改拒絕✓ rustc✓；UNSAT 路徑 `1 ∈ G` | 硬 |
| K5 | `./target/release/polyrust obligations` | T1–T9 對 12 樣本全 PASS | 硬（約 6 min，可獨立 job） |
| K6 | `./target/release/polyrust check examples/sqr.poly` | SAT + 生成碼 | 硬 |

## B. Lean 硬閘門

| ID | 命令 | 期望 | 阻斷 |
|---|---|---|---|
| L1 | `cd lean && lake build` | 34 模組 + 根通過 | 硬 |
| L2 | `bash scripts/lean-audit.sh` | 無 `sorryAx`、無自訂公理 | 硬 |
| L3 | `lake env lean AuditAll.lean`（或 CI 等價步驟） | `AUDIT_RESULT=CLEAN` | 硬 |

對照文件：`docs/LEAN.md`（652 定理 / AuditAll 3074）。  
EVIDENCE.md 的「273 定理 / 14 模組」是 2026-09-10 快照，**不再作為閘門數字**。

## C. Surface / frontend（資訊性，直到 M1）

| ID | 命令 | 期望 | 阻斷 |
|---|---|---|---|
| S1 | `cargo test --release -p polyrust-core --lib pipeline_v2` | 現有單元測試過 | 軟（M1 前） |
| S2 | 以 `check-v2` 跑 `examples/phase3/*.poly` | 與 `docs/EXAMPLES_PHASE3.md` 標籤一致 | 軟 |
| S3 | `cargo test --release -p polyrust-full oracle::tests::test_oracle_gap` | CI 已列為硬；M0 承認這是 **syn Oracle 缺口檢測**，不是代數完備 | 維持 CI 現狀 |
| S4 | `cargo test --release`（workspace） | CI rust job 現況 | 硬（CI 已如此） |
| S5 | `cargo fmt --check` / `clippy` | 風格債 | 資訊性（CI `continue-on-error`） |

## D. 回歸金樣（v1 禁止變語義）

改 solver / 約束生成時，下列輸出形狀視為契約：

- demo A：SAT，可 rustc
- demo B：型別錯 → UNSAT，`1 ∈ G`
- demo C：雙重疊 `&mut` → UNSAT，借用子句 + `1 ∈ G`
- demo D：選臂 SAT，可 rustc
- `examples/sqr.poly`、`examples/bad.poly`

v2 / Phase3 的 `*_sat.poly` / `*_unsat.poly` **不是** v1 契約，直到 lower 進同一 `System`。

## E. 本樹靜態測試屬性分佈（core，166）

| 模組 | `#[test]` |
|---|---:|
| `dsl.rs` | 17 |
| `checker.rs` | 15 |
| `llm.rs` | 13 |
| `parse_full.rs` | 10 |
| `pipeline_v2.rs` | 10 |
| `lifetime.rs` / `ty.rs` / `universe.rs` | 各 7 |
| `effects.rs` / `lower.rs` / `trait_impl.rs` | 6 / 6 / 6 |
| `cdcl.rs` / `groebner.rs` / `exhaust.rs` / `async_qap.rs` | 各 5 |
| `borrowck.rs` / `constraints_v2.rs` / `contracts.rs` / `stdlib.rs` | 各 4 |
| 其餘 | ≤3 |
| `obligations.rs` / `pipeline.rs` / `codegen.rs` / `driver.rs` | 0（義務在 CLI） |

解讀：測試質量偏「新模組單元測試」，kernel 端到端集中在 CLI `demo` / `obligations`，不是 lib test。刪 CLI 子命令等於拆掉 T9 閘門。

## F. CI 對照（`.github/workflows/ci.yml`）

觸發：`main`、`v0.1.*`、`v0.2.*`、tag `v*`、PR。

已是硬閘門：`cargo test --release`、release 編譯、Oracle gap、`lake build`、AuditAll。  
`obligations` job 獨立、不擋 `verify` —— M0 **維持**，但發 tag 前必須人為看過 K5。
