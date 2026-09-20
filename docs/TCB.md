# TCB 白皮書 — polyrust 信任邊界（v0.2.7-rsap + WRP-R2，2026-09-20）

> **一句話**：`rustc` 是語義地真值（typeck/borrowck/drop/trait），`Charon` 是脫水投影，`PolyIR` 是可判定子語言，`CDCL(T)×GB` 是判定引擎，`Lean` 是機械化 meta 證明。信任邊界按此分層**誠實入賬**，邊界外一律 `Unknown[reason_code]` 不偽造。

**口徑版本**：`polyrust-ir/1` · `charon 0.1.265` · `nightly-2026-09-17` · `rustc 1.98.1` · `Lean 4.33.1`  
**上級**：[`RUSTC_SEMANTIC_ALIGNMENT_PLAN.md`](RUSTC_SEMANTIC_ALIGNMENT_PLAN.md) §7 + [`CHARON_POLYIR_BLUEPRINT.md`](CHARON_POLYIR_BLUEPRINT.md) §1.3  
**驗證**：`scripts/rustc_align_check.py` 152 樣本 0 違規 + `cargo test --lib differential::tests::differential_v1_v3_v4_ratchet`（0 gaps） + `cargo test --lib semantic_matrix`（100/100） + `lean/Polyrust/RustcAlign.lean` 10 定理零 sorry

---

## 1. 邊界圖（數據流 × 信任層次）

```mermaid
flowchart LR
    subgraph TCB ["TCB（必須信任）"]
        A[rustc nightly-2026-09-17<br/>typeck / borrowck / drop<br/>trait resolution]
        B[Charon 0.1.265<br/>MIR → LLBC<br/>--no-dedup-serialized-ast]
        C[PolyIR charon_llbc.rs<br/>std-only JSON parser<br/>13-key whitelist]
        D[PolyIR polyir.rs<br/>模板認形白名單<br/>reason_code 降級]
        E[PolyIR polyir_encode.rs<br/>消失多項式 + 見證恆等化<br/>PRODUCT_CAP 1e6 / HARD_CAP 4096]
        F[Solver cdcl.rs + groebner.rs<br/>CDCL(T) × Lazy GB<br/>S-多項式準則]
        G[Certify mir_lower.rs + certify.rs<br/>域成員 + 多項式直接求值<br/>RUP]
        H[Lean 4.33.1 kernel<br/>RustcAlign / LoopInvariant / Recursion]
    end
    subgraph Untrusted ["非 TCB（不信任，僅作輸入）"]
        U1[*.rs 源文件<br/>任意 Rust 2021]
        U2[*.poly DSL<br/>legacy v1/v3]
        U3[LLBC JSON<br/>外部進程產物]
    end
    U1 -->|rustc + Charon 進程外| B -->|JSON| C --> D --> E --> F --> G
    U2 -->|legacy parser| D
    U3 -.->|schema漂移即hard error| C
    A -.->|地真值 oracle| D
    H -.->|機械化不變量| D
    G -->|Certified / Unsat / Unknown| Out([CLI / JSON / Lean 原生庫])
    style TCB fill:#0b3d20,stroke:#2ecc71,stroke-width:2px,color:#fff
    style Untrusted fill:#4a1a1a,stroke:#e74c3c,color:#fff
```

**顏色**：綠框 = 進入 TCB（需信任其實作正確）；紅框 = 輸入/載體（不信任，經白名單/錯誤邊界隔離）。

---

## 2. 分層責任與假設

| 層 | 組件 | 負責語義 | 信任假設 | 失效率隔離 |
|---|---|---|---|---|
| **L0 rustc** | `rustc` nightly-2026-09-17 + stable 1.98.1（`rustc_align.rs` 三路徑探測） | **全部** Rust 語言語義（typeck、borrowck、drop、trait、lifetime、unsafe 合法性） | 信任 `rustc` 對「合法程序」的判定為真（oracle）；`rustc` 本身 bug 視為 TCB 風險但概率極低，經 `charon.pin` 與發版釘版可追溯 | 單檔 `crate-type=lib --edition=2021` 脫水；`error[E\d+]` 正則才判 `Rejected`，餘為 `ExternalDep` 寬鬆對準 |
| **L1 Charon** | `Charon 0.1.265`（`charon.pin`） + `frontends/charon` | MIR 脫水：宏展開、方法解析、模式降糖、借用檢查結果、drop 插入 | 信任 Charon 正確把 rustc 接受程序投影為 LLBC 且 `has_errors` 旗可信；JSON 非穩定接口視為載體，經 `charon_llbc.rs` 白名單隔離 | `unknown root key` / 未知 `BodyKind` / 缺 `translated` 鍵 → **hard error** 非 silent；`has_errors=true` → `Unknown(missing_decl)` 禁止 Certified |
| **L2 PolyIR 抽取** | `core/charon_llbc.rs`（零依賴 mini-JSON） | LLBC JSON → `PolyModule` 結構投影（14 鍵白名單、BodyKind 三態、span 保留） | 信任 parser 正確拒絕 schema 漂移（`charon --no-dedup-serialized-ast` 口徑） | 超出白名單 → `Err`；`Error`/`Missing` body → `PirVerdict::Unknown(missing_decl)` |
| **L2.5 PolyIR 模板** | `core/polyir.rs` `lower_fun`（R1 rustc-gated） | 可判定子語言：直線算術 / Switch 分裂 / 本地純 `Call` / 單調 `Loop` / 自遞迴；其餘 `Ref`/`RawPtr`/`dyn-async`/`union`/`multi-exit loop`/`mutual recursion` | 信任 `reason_code` 枚舉窮舉，且每個分支皆以 `Unknown[reason_code:xxx]` 誠實降級，永不偽造 `Certified` | 對應 `lean/Polyrust/RustcAlign.lean` 之 `rustcGatedSound`：`Unknown _ ≠ Certified`（`rfl`） |
| **L3 編碼/求解** | `polyir_encode.rs` + `cdcl.rs` + `groebner_f4/f5.rs` + `vanishing.rs` | 值軌跡→多項式系統（消失多項式 + 見證恆等化）→ CDCL(T)×GB 判定 | 信任 Buchberger 終止性（`Squarefree` 2ⁿ 界）、S-多項式入理想（`SPoly`）、QAP 同態（`UniPoly`）、`PRODUCT_CAP`/`HARD_CAP` 界限正確 | `domain_cap`/`hard_cap`/`overflow` 超限 → `Unknown`；`Certified` 必經 `certify_mir` 全多項式直接求值 + `cdcl_learnts_to_rup`（L3 已自證） |
| **L4 認證** | `core/certify.rs` + `core/mir_lower.rs` | **證書自證**：SAT 見證 σ 逐多項式求值 + UNSAT RUP | 信任 `Lean` kernel 檢查（僅 `DEPENDS ON AXIOMS` 三標準公理） | `Certified` 附 `ret`/`paths`/`excluded` 可獨立檢；`Unknown` 永不帶假證書 |
| **Meta Lean** | `lean/Polyrust/*` 41 模組（4.33.1，無 Mathlib） | 機械化不變量：`LoopInvariant.whileMonoF_invariant`、`Recursion.factF_spec`、`RustcAlign.noFalseCertified` 等 273 定理 | 信任 Lean kernel + 4.33.1 toolchain（`lean-toolchain` pin，CI `lake build` + `AuditAll=CLEAN`） | 零 `sorry`、零自訂公理（`AuditAll` 掃全環境），`lean:static`/`shared` 僅供嵌入，不入證明 TCB |

**非 TCB（明確排除）**：

- `v1`/`v2`/`v3` legacy parsers（`pipeline.rs`/`pipeline_v3.rs`）：**已 deprecated**，僅保留作 `differential.rs` 差分回歸參照（86/100 適用域外歸為 `None`，不入判定 TCB）。
- `frontends/*`（TS/IDE）：屬傳輸層，診斷直達 `span` 由 PolyIR 產生，前端僅渲染。
- `cargo deny` / `clippy` / `fmt`：屬工程門檻，不入語義 TCB。

---

## 3. RSAP 對準層（R0–R1 已落地，R2–R3 本版完成）

**判定一致性鐵律**（`core/src/rustc_align.rs` `AlignReport::check`）：

```
Accepted   ↔ Certified / Unknown ✓ | Unsat ✗ (false_positive)
Rejected(E) ↔ Unsat / Unknown ✓ | Certified ✗ (soundness_violation = CI 紅)
MissingToolchain / ExternalDep → 一律 aligned（語料邊界，Unknown 誠實即對準）
```

對應 Lean（`Polyrust/RustcAlign.lean`）：

- `rustcGatedSound`：`∀ r, Unknown r ≠ Certified`
- `noFalseCertified`：`aligned (Rejected code) Certified = false`
- `missingToolchainAlwaysAligned` / `externalDepAlwaysAligned`：語義邊界一律對準

**reason_code 窮舉**（`polyir.rs` + `rustc_align.rs` 共用白名單，10 項）：

`missing_decl` | `borrow_rustc_gated` | `raw_ptr` | `dyn_async_rpit` | `template_outer` | `unsupported_rvalue` | `domain_cap` | `hard_cap` | `overflow` | `external_dep`

**實測**（2026-09-20 沙盒，WRP-R2 後，`semantic_matrix` 100/100）：

- `scripts/rustc_align_check.py` **152/152**（100 matrix + 52 examples）→ `Accepted 117 | Rejected 35 | 違規 0 → PASS ✓`（`differential` 三路 `v4 gaps 0/100`，decidable 98/100，WRP-R2 3→0 收窄 100%）
- `cargo test --lib differential::tests::differential_v1_v3_v4_ratchet` → `three-way: 100 cases, v4 gaps 0 (whitelist 0), v3↔v4 divergences 0, v4_decidable 98/100`
- `cargo test --lib rustc_align` 8/8 + `cargo test --lib rustc_syntax` 4/4 + `cargo test --lib semantic_matrix` 100/100 + `cargo test --lib` **174/174**

詳見 [`RSAP_RESULT.md`](RSAP_RESULT.md) §2 與 [`RUSTC_SEMANTIC_ALIGNMENT_PLAN.md`](RUSTC_SEMANTIC_ALIGNMENT_PLAN.md) §8 及 [`RUSTC_SYNTAX_STUDY.md`](RUSTC_SYNTAX_STUDY.md)。

---

## 4. 信任邊界變更日誌（RSAP）

| 版本 | 變更 | 入 TCB | 出 TCB | 證據 |
|---|---|---|---|---|
| v0.2.6 → v0.2.7-rsap | 新增 `rustc` + `Charon` 為地真值/投影 | `rustc 1.98.1` + `Charon 0.1.265`（`charon.pin`） | — | `C0_BASELINE.md` 117/152、零 charon_err |
| R1 | `polyir.rs` 頭 doc 白名單 + `borrow_rustc_gated` / `raw_ptr` 精確降級 | — | `check_deref_depth` 等自製 lint 降為 legacy 冗餘（保留不刪除） | `cargo test polyir` 5 例 reason_code 精確 |
| R2 | `differential.rs` 擴三路 `v1/v3/v4` ratchet，白名單 6→0（WRP-R2 3→0）| — | v1 parser 擴闊作廢（`CHARON_POLYIR_BLUEPRINT.md` §6） | three-way 0 gaps 0 divergence, 98/100 decidable, `semantic_matrix` 100/100 |
| R3 | `lean/Polyrust/RustcAlign.lean` + 本文件 + `EVIDENCE.md §13e` | `Lean 4.33.1 kernel`（`RustcAlign` 10 定理） | — | `lake build` 綠、`AuditAll=CLEAN`（CI） |
| WRP-R2 | `pipeline_v2.rs` 3 修复 + `rustc_syntax.rs` + `RUSTC_SYNTAX_STUDY.md` | `rustc_syntax`（`split_fields_top_level`） | — | `semantic_matrix` 100/100, `differential` 0 gaps, `rustc_align` 152 0违规, `174/174` |

---

## 5. 未覆蓋與顯式非擔保

- **複雜度天花板**：GB 2ⁿ（`Squarefree`）+ `PRODUCT_CAP=1_000_000` + `CONTEXT_CAP=4096` + `LOOP_ITERS_HARD_CAP=4096`；超限 → `Unknown(domain_cap/hard_cap)`，**不聲稱**無界 SAT/UNSAT（`RSAP` 三值口徑）。
- **語言子集**：當前 `Certified` 僅覆蓋 L2.5 白名單；`Ref` 借用語義（NLL）、`raw pointer` 解引用合法性、`dyn Trait`/`GAT`/`async`/`RPIT`、`union` 標籤缺失、`mutual recursion`、`multi-exit loop` 等皆 `Unknown(template_outer|…)`——**由 rustc 擔保，polyrust 不重檢**（見 `borrow_rustc_gated` 口徑）。
- **反例**：`differential.rs` `KNOWN_DIVERGENCES` 2 項（`io_effect` + `io_with_pure_call`：v1 保守 UNSAT vs v3/rustc SAT，後者為 WRP-R2 修 v3 後暴露）屬 v1 過嚴，修 v1 屬 P2；v4 已 **0 gaps**（§3，剩余 2 Rejected Unknown 为诚实边界，不入 gap）；`rustc_syntax` 已确立“语法→类型→效应→判定”四层路径，P1 将以 syn 级泛型与按函数效应清零剩余 honest 边界。
- **供給側**：`charon` noctool 環境 → `polyrust align-check` 走 `rustc_oracle` 單檔路徑（`MissingToolchain` 永對準），`check *.rs` 退化為 legacy，但不誤判。

---

## 6. 重現與審計命令

```bash
# 1. 對準率（152 全量，違規必須 0）
python3 scripts/rustc_align_check.py --out /tmp/rsap --json
cat /tmp/rsap/summary.md   # 總樣本 152；Accepted 117 | Rejected 35 | 違規 0 → PASS

# 2. 三路差分（100 matrix，0 gaps 0 divergence, 98/100 decidable）
cargo test --lib differential::tests::differential_v1_v3_v4_ratchet -- --nocapture
cargo test --lib semantic_matrix -- --nocapture   # 100/100
cargo test --lib rustc_syntax -- --nocapture      # 4 passed

# 3. rustc-gated 單測
cargo test --lib rustc_align -- --nocapture   # 8 passed
cargo test --lib polyir -- --nocapture        # reason_code 5 例 + 結構 lift

# 4. Lean（需 elan；無工具鏈亦不紅，CI 另跑）
cd lean && lake env lean Polyrust/RustcAlign.lean
lake env lean AuditAll.lean | grep AUDIT_RESULT   # CLEAN

# 5. TCB 邊界自檢
grep -r "reason_code" core/src/polyir.rs | wc -l          # 10 白名單關鍵字
grep -r "MissingToolchain\|ExternalDep" core/src/rustc_align.rs | wc -l
```

---

*本文件與 `EVIDENCE.md §13e` 同步由 `scripts/rustc_align_check.py --json` 產生之度量驅動；任何 TCB 擴大/縮小皆需本文件 PR 顯式記錄並經 `differential` + `rustc_align` + `AuditAll` 三閘門。*
