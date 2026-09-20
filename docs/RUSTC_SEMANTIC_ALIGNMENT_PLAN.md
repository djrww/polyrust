# 語義快速對準 rustc 計劃 — RSAP v1.0（72 小時衝刺）

> **目標**：以 `rustc` 為唯一語義地真值，令 polyrust 在 72 小時內達成「語義對準」——**判定一致 + 證書可驗 + 能力邊界誠實**。  
> **狀態**：已執行 R0–R3（2026-09-20 06:40–09:00，v0.2.6 → v0.2.7-rsap，**R2/R3 本次收口**）  
> **分支**：`v0.2.6` → `rsap`（本計劃）  
> **上級**：[`CHARON_POLYIR_BLUEPRINT.md`](CHARON_POLYIR_BLUEPRINT.md) + [`CHARON_PLAN.md`](CHARON_PLAN.md) + [`DEV_PLAN_V03.md`](DEV_PLAN_V03.md)  
> **口徑版本**：`polyrust-ir/1` + `charon 0.1.265 / nightly-2026-09-17` + `rustc 1.98.1` + `Lean 4.33.1`

---

## 0. 北極星與口徑（不可妥協）

**一句話**：對任意真 Rust 程序 `P`，polyrust 的判定必須與 `rustc` 一致；`Certified` 附機器可驗證據，否則誠實 `Unknown`，絕不偽造。

| 判定 | rustc 地真值 | polyrust 允許 | polyrust 禁止 |
|------|--------------|---------------|---------------|
| `rustc 接受` | `rustc --crate-type=lib` 0 退出 | `Certified` / `Unknown` | `Unsat`（假陽性） |
| `rustc 拒絕 (E-code)` | 非 0 + `error[E...]` | `Unsat` / `Unknown` | `Certified`（假陰性 = soundness 洞） |
| `Charon has_errors` | 部份 decl 抽唔到 | `Unknown(missing_decl)` | `Certified` |

**三值口徑（審計③）**：`Certified` 係**有界判定**——參數域笛卡爾積 × 逐組合見證 × 模板終止性（`LOOP_ITERS_HARD_CAP`/`REC_DEPTH_HARD_CAP`）。超出即 `Unknown{reason}`，絕不表述為無界 SAT。

**五大原則**：
1. **rustc 係 oracle**：語義爭議一律以 `rustc` 編譯結果為準，唔靠手寫 lint 猜。
2. **Charon 係投影**：經 `frontends/charon` + `charon_llbc.rs` 讀 LLBC，core 零依賴以 mini JSON 讀 `polyrust-ir/1`；charon 升版只撞 L2，L3 永穩。
3. **模板白名單 + 誠實降級**：`polyir.rs` 白名單外一律 `Unknown{reason}`，附精確 `reason_code`（`missing_decl` / `borrow_rustc_gated` / `overflow` / `template_outer`）。
4. **見證恆等化**：比較 / `Discriminant` / `Call` 在見證模式恆等化 `dst = 模擬值`，模擬語義即 rustc 語義（i64 checked 口徑），編碼僅記錄。
5. **證書自證**：`Certified` 必經 `mir_lower::certify_mir`（域成員 + 全多項式直接求值）+ `cdcl_learnts_to_rup`（UNSAT 側），`Unknown` 永不帶假證書。

---

## 1. 現狀基線（v0.2.6，2026-09-20 實測）

| 維度 | 數值 | 來源 |
|------|------|------|
| LLBC 轉換率 | **117/152 = 77.0%**（有效 117/136=86%） | `C0_BASELINE.md` |
| 有效失敗 | 35 `rustc_reject`（16 設計 UNSAT 正確 + ~13 DSL溢出 + 3 外部依賴 + 2 語料自傷），0 `charon_err`/`timeout` | 同上 |
| lib 測試 | **161 passed**（98 C1 + 53 C2-C7 + 10 審計）→ **170 passed**（R2/R3 後） | `DEV_LOG_2026-09-20.md` + `RSAP_RESULT.md` |
| 語義 lowering 覆蓋 | 直線 + Switch 分裂 + 本地純函數 `Call` + 單調 `Loop` + 自遞迴 | `polyir.rs` 頭 doc 白名單 |
| 已知誠實缺口 | `Ref`/借用、`raw pointer`、`dyn/GAT/async/RPIT`、`enum 值傳參`、`嵌套動態 Switch`、`多出口 loop`、`mutual recursion` → 全部 `Unknown` | 同上 |
| Lean | 2 新模組 `LoopInvariant` + `Recursion` + **R3 新增 `RustcAlign` 10 定理**，零 sorry，`AuditAll=CLEAN` | `EVIDENCE.md §13e` |
| 已修 soundness 洞 ×2 | fallback guard 無條件放行 / 模擬錯誤優先級 | `f341c30` |
| RSAP 對準 | **152/152 零違規** + **三路 100 cases 6 gaps 0 divergence** | `RSAP_RESULT.md` §2 |

**對準度量化（實測，R2/R3 後）**：

- 真檔 rustc 可編譯樣本（112 `ok`）：`Certified` 6 例（`sqr`/`pure_inner`/`inv_sum` n=5/`fact` n∈{0,1,3,5,10}）+ 其餘 `Unknown` 誠實降級 = **0 假陽性/假陰性**。
- `rustc_reject` 樣本：`ref_deref` 等 E0614 樣本經 Charon 直接無 LLBC → 入口 `rustc_reject` 口徑，不入代數（sound）。
- **R2 三路**：`v1↔v3` 1 divergence（`io_effect`）+ `v3↔v4` 0 divergence / 6 gaps（`dyn_async_rpit`×2、`template_outer`×2、`raw_ptr`×1、`unsupported_rvalue`×1）→ 全部白名單 ratchet 綠。

**缺口**：缺乏**自動化「rustc vs PolyIR 差分」量尺**——已由 `scripts/rustc_align_check.py`  + `differential:three-way` 補齊（1 條命令量化對準率）。

---

## 2. 快速對準策略（為何 72 小時足夠）

傳統路徑「手寫 parser 追 Rust 語法」永遠追唔上。RSAP 採用**脫水（desugaring）分層**：

```
真 Rust .rs ──(rustc + Charon, 夜間版 pin)──▶ LLBC (已脫水：宏展開、trait 解、借用檢查、drop)
                ──(frontends/charon, serde 允許)──▶ polyrust-ir/1 (KB 級精簡，版本化)
                    ──(core, std-only mini JSON)──▶ PolyIR lowering (模板白名單)
                        ──▶ 編碼 → T10 分解 → CDCL(T)×GB → 認證
```

**買到的東西**：
- rustc 語義地真值（typeck/borrowck/drop/trait）由工具鏈擔保，polyrust 唔再重寫。
- 全語言覆蓋（async/closure/lifetime/trait 等）天然 `Charon ok` → 可入管線。

**要付的代價**（誠實入 TCB）：
- `rustc + Charon` 入信任基（`docs/TCB.md` 明文記錄）。
- LLBC JSON 非穩定接口 → `charon.pin` 釘版 + schema 白名單 + snapshot fixtures。
- 模板外語言形態 → 誠實 `Unknown(template_outer)` + `reason_code`，唔硬撐。

**72 小時可行性**：C0–C7 已完成 70% 地基（Charon pin + LLBC parser + 4 模板 + 融合模擬 + Lean），RSAP 只需補**最薄一層「對準量尺 + rustc-gated 口徑」**即可宣稱語義對準。**R2/R3 證明 72h 足夠**：R0/R1 0.5日 + R2 0.5日 + R3 0.5日 = 1.5日內收口。

---

## 3. 架構（L1–L3 不變，新增 R 對準層）

```
L1 抽取： rustc (nightly-2026-09-17) + Charon 0.1.265 --no-dedup-serialized-ast
          │  .llbc (translated{13 keys}, has_errors 旗)
          ▼
L2 IR：   frontends/charon (本計劃補) : LLBC → polyrust-ir/1 JSON
          │  { polyrust_ir_version:1, crate, funs[{name, arg_count, body_kind, stmts…}] }
          ▼
L3 代數： core (零依賴) : charon_llbc.rs (mini JSON) → polyir.rs (模板 lowering)
          → polyir_encode.rs (消失多項式 + 見證恆等化) → certify/mir_lower
          ──┬── R 對準層（本計劃新增）
            ├── rustc_align.rs : 單檔 rustc 調用 (crate-type lib, --edition=2021)
            ├── align_verdict : Decision ↔ rustc 判定一致性檢查 (soundness 閘)
            └── differential.rs 三路：v1 / v3-legacy / v4-PolyIR

認證：   Certify (SAT: σ 見證直接求值 / UNSAT: RUP) + Lean (LoopInvariant/Recursion/RustcAlign)
```

**不變量**：core 零第三方依賴（R 層亦 std-only，rustc 進程外調用）。

---

## 4. 里程碑（R0 → R3，每個半日可交付可回退）

| 里程碑 | 內容 | 交付 | 驗收 | 狀態 |
|--------|------|------|------|------|
| **R0 基線量尺** | `rustc_align.rs` + `align-check` CLI；定義 `RustcVerdict`/`AlignReport`；實裝單檔 `rustc` oracle（3 路徑探測 + timeout 10s） | `core/src/rustc_align.rs`、`driver::cmd_align_check` | `cargo test --lib rustc_align` 8/8 綠；`polyrust align-check --help` 可用 | ✅ 完成 |
| **R1 rustc-gated 口徑** | PolyIR 頭 doc 補 `reason_code` 枚舉；`Ref`/借用/`raw ptr`/`dyn` 等已知缺口返回 `Unknown{reason_code=borrow_rustc_gated|raw_ptr|unsupported}`；`has_errors` → `Unknown(missing_decl)` 口徑文件化 | `polyir.rs` + `charon_llbc.rs` | 5 個手工 Unknown 樣本 reason_code 精確匹配；`missing_body` → Unknown(0 Certified) | ✅ 完成 |
| **R2 差分對準閉環** | `differential.rs` 擴三路 `v1 / v3 / v4`；新增 `scripts/rustc_align_check.py`（類似 `c0_spike.py` 但比對 Decision vs rustc）；CI `rustc-align` job | `differential` + `scripts/` + `.github/workflows` | `three-way: 100 cases, v4 gaps 6 (whitelist 6), divergences 0`；152 全量 0 違規 | ✅ 完成（本批次） |
| **R3 Lean + 文件** | `lean/Polyrust/RustcAlign.lean` 10 定理；`docs/TCB.md` 信任邊界；`docs/EVIDENCE.md §13e` 指標由腳本產生 | `lean/` + `docs/` | `lake env lean RustcAlign.lean` 綠；`AuditAll=CLEAN`；`cargo test` 170 綠 | ✅ 完成（本批次） |

---

## 5. 詳細任務清單（Checklist）

### R0 — 基線量尺

- [x] `core/src/rustc_align.rs`：`RustcVerdict{Accepted, Rejected{code,stderr}, MissingToolchain}` + `rustc_oracle(src)`（`rustc --crate-type=lib --edition=2021` 三路徑探測，stderr 抓 `error[E\d+]`）
- [x] `AlignReport`：`{rustc, polyir, aligned: bool, violation?: SoundnessViolation}`，`aligned` 準則見 §0
- [x] `driver::cmd_align_check`：`polyrust align-check <file.rs> [--json]`（讀檔 → 同時跑 rustc oracle + PolyIR Charon 管線（如無 LLBC 則走 rustc 單檔）→ 對準報告）
- [x] 單測 8 項：accepted↔Certified/Unknown pass、rejected↔Unsat/Unknown pass、Certified↔rejected 判違規、Unsat↔accepted 判違規 + 2 oracle 實調

### R1 — rustc-gated 口徑

- [x] `polyir.rs` 頭 doc：`reason_code` 枚舉（10 項：`missing_decl` | `borrow_rustc_gated` | `raw_ptr` | `dyn_async_rpit` | `template_outer` | `unsupported_rvalue` | `domain_cap` | `hard_cap` | `overflow` | `external_dep`）
- [x] 落地：`Ref` rvalue（`&T` / `&mut T`）→ `Unknown{borrow_rustc_gated}`（rustc 已檢，無需代數重檢；附精確 span）
- [x] `has_errors` / `BodyKind::Error|Missing` → `Unknown(missing_decl)`（沿用 C0 口徑，防止假 SAT）
- [x] `raw pointer` place（`*const T` 解構）→ `Unknown{raw_ptr}`（誠實降級）
- [x] 測試：`unknown_reason_code_contains`（5 例）

### R2 — 差分對準閉環

- [x] `differential.rs` 三路 ratchet（`v1_legacy` / `v3_legacy` / `v4_polyir`），白名單 `KNOWN_V4_GAPS` 6 項 + `KNOWN_V3_V4_DIVERGENCES` 0 項（見 `differential::tests::differential_v1_v3_v4_ratchet`）
- [x] `scripts/rustc_align_check.py`：`corpus()`（100 matrix + 52 examples）→ `rustc oracle` + `charon → PolyIR decide` → 聚合對準率（`results.json` + `summary.md`，152 全量 0 違規）
- [x] CI `rustc-align` job（含 `cargo test differential` 三路 + `cargo test polyir` + `lake lean RustcAlign` + 152 全量）

### R3 — Lean + 文件

- [x] `lean/Polyrust/RustcAlign.lean` 10 定理：`rustcGatedSound`（`Unknown _ ≠ Certified`）、`noFalseCertified`（`Rejected → ¬Certified`）、`missingToolchainAlwaysAligned`、`externalDepAlwaysAligned` 等（零 sorry，僅標準三公理）
- [x] `docs/TCB.md` 220 行：分層圖（mermaid）+ L0–L4 責任表 + RSAP 入賬日誌 + 重現命令
- [x] `docs/EVIDENCE.md §13e`：RSAP 對準證據（152 全量表 + 三路 6 gaps 0 div + Lean）由 `scripts/rustc_align_check.py --json` 驅動

---

## 6. 驗收閘門（Gates）

| 閘 | 命令 | 紅線 | 實測（R2/R3 後） |
|----|------|------|-----------------|
| G1 單測 | `cargo test --lib rustc_align -- --nocapture` | 8/8 綠 | ✅ 8 passed |
| G2 對準率 | `python3 scripts/rustc_align_check.py --json` | 對準率 100%（0 soundness 違規） | ✅ `{"total":152,"violations":0,"pass":true}` |
| G3 Charon + 三路 | `cargo test --lib differential -- --nocapture` | `v4 gaps 6/6 0 div` | ✅ `three-way: 100 cases, 6 gaps 0 div` |
| G4 Lean | `cd lean && lake env lean Polyrust/RustcAlign.lean && lake env lean AuditAll.lean` | `AUDIT_RESULT=CLEAN` | ✅ 10 定理零 sorry（CI `lean-action` 綠） |
| G5 全量 | `cargo test --lib` | 170 綠 | ✅ 170 passed |
| G6 文檔口徑 | `grep -r "Certified" --include="*.md"` | 口徑一致（有界三值），無「無界 SAT」 | ✅ 一致 |

---

## 7. 風險登記與回退

| 風險 | 緩解 | 回退 |
|------|------|------|
| rustc 單檔編譯誤判（缺 `extern crate` 導致假 reject） | 衛生：單檔 `crate-type=lib` + `edition=2021`；E-code 正則 `error[E\d+]` 才判 `Rejected`，其餘 `MissingToolchain` | 標 `Unknown(external_dep)`，不計入對準違規 |
| Charon nightly 耦合 | `charon.pin` 釘版，升版走顯式 PR + `scripts/c0_spike.py` 重做 fixtures | 無 Charon 環境走 `rustc_oracle` 單檔路徑，`polyrust align-check` 仍可用 |
| 模板覆蓋不足導致 Unknown 泛濫 | `reason_code` 精確分類 + 6 gaps 白名單 + P1 待辦（Ref/Loop 擴充） | 允許 Unknown，但絕不放寬為 Unsat/Certified |
| GB 複雜度（值域組合 1_000_000 上限） | `PRODUCT_CAP` + `CONTEXT_CAP=4096` + `LOOP_ITERS_HARD_CAP=4096` | 組合超限 → `Unknown(domain_cap)` 誠實降級 |

---

## 8. 執行記錄（R0–R3 已落地）

> 執行時間：2026-09-20 06:40–09:00 UTC（沙盒）  
> 執行人：Arena Agent Mode  
> 產物分支：`rsap`（基於 `v0.2.6` `1fc70b8`）

| 步驟 | 動作 | 提交/文件 |
|------|------|-----------|
| 1 | 克隆 `v0.2.6` → `/home/user/polyrust`，審計基線（161 tests、C0 77%、10 fixtures） | `C0_BASELINE.md` / `DEV_LOG_2026-09-20.md` |
| 2 | 新建 `core/src/rustc_align.rs`（342 行，std-only）：`rustc_oracle` + `align` + `reason_code` + 8 單測 | `rustc_align.rs` |
| 3 | `core/src/lib.rs` 註冊 `pub mod rustc_align` | `lib.rs` |
| 4 | `core/src/polyir.rs` 頭 doc 補 `reason_code` 枚舉 + `Ref`→`borrow_rustc_gated`、`raw_ptr`→`raw_ptr` 精確降級（3 處） | `polyir.rs` |
| 5 | `core/src/driver.rs` 新增 `cmd_align_check`（`align-check` / `rustc-align` 子命令，`--json` 機讀） | `driver.rs` |
| 6 | `core/src/main.rs` 接線 `align-check` → `driver::cmd_align_check` | `main.rs` |
| 7 | `lean/Polyrust/RustcAlign.lean` 骨架（`rustcGatedSound` / `noFalseCertified`，零 sorry） | `RustcAlign.lean` 4 定理 |
| 8 | `scripts/rustc_align_check.py` harness（`corpus()` → rustc + Charon 可選 → 對準聚合） | `scripts/` |
| 9 | `.github/workflows/ci.yml` 補 `rustc-align` job（無 Charon 亦可綠） | `ci.yml` R0/R1 版 |
| 10 | `cargo test --lib rustc_align` 8/8 綠；`cargo test --lib` 169/169 綠；`polyrust align-check` 手工驗證 | R0/R1 度量 0 違規 |
| 11 | **R2**：`differential.rs` 擴三路（`ThreeWayRow` + `polyir_v4_gap_reason` + `run_three_way`，`KNOWN_V4_GAPS` 6 項） | `differential.rs` +140 行 |
| 12 | **R2**：CI 補 `differential 三路` 硬閘門 + 上傳擴列 | `ci.yml` R2 版 |
| 13 | **R3**：`RustcAlign.lean` 增至 10 定理（`missingToolchainAlwaysAligned` 等，`cases <;> rfl`） | `RustcAlign.lean` 95 行 |
| 14 | **R3**：新增 `docs/TCB.md` 220 行（mermaid 邊界圖 + L0–L4 表 + 入賬日誌）+ `docs/EVIDENCE.md §13e`（152 表 + 三路）+ `docs/RSAP_RESULT.md` 更新至 170 | `docs/` |
| 15 | **驗收**：`cargo test --lib differential` 2/2 綠（`three-way: 100 cases, 6 gaps 0 div`）+ `cargo test --lib` 170/170 + `scripts/rustc_align_check.py` 152 PASS | 本節下方實測 |

**實測（沙盒，R2/R3 後，`--release`）：**

```
cargo test --lib rustc_align          → 8 passed
cargo test --lib differential         → 2 passed (differential: 1 div / three-way: 6 gaps 0 div)
cargo test --lib                      → 170 passed (161 原有 + 9 新增)
python3 scripts/rustc_align_check.py --json → {"total":152,"violations":0,"pass":true}
./target/debug/polyrust align-check --json  → 3 抽樣 aligned:true
cargo test --lib polyir               → reason_code 5 例全綠
```

**度量**：對準違規 = **0**；`Certified` 僅在 `rustc Accepted` 分支內出現；三路 6 gaps 0 divergence（R2 ratchet 綠）。

---

## 9. 下一步（P1 連接）

- **C2–C4 模板收窄**：`C2 直線+分支` 已穩；`C3 loop+fuel` / `C4 calls/contracts` 按 6 gaps 逐項下沉（每項修好即刪白名單行，ratchet 自動紅→綠）。
- **Ref/借用語義擴充**：從 `Unknown(borrow_rustc_gated)` 升級為 NLL 區間小重推（Polonius-lite），仍以 rustc 為 oracle 差分。
- **Charon L2 前端**：`frontends/charon` 實裝 `LLBC → polyrust-ir/1` serde 轉譯（本計劃已留接口，ci 可選）。
- **Lean 完整化**：`RustcAlign` 10 定理已足 R3；後續 `UnrollSound` / `ContractSound` 接 `C3/C4`。

---

*RSAP v1.0（R0–R3）執行完畢。`polyrust` 的語義以 `rustc` 為地真值：任何 `Certified` 必經 rustc 接受 + 獨立證書自證；任何 rustc 拒絕必不出現 `Certified`。能力邊界以 `Unknown{reason_code}` 誠實申報；三路差分保證 legacy 與新鏈不漂移——**快速對準，絕不假對準**。*
