# 重構藍圖：Charon + PolyIR 分層架構（v0.4 方向）

> 日期：2026-09-19 ｜ 狀態：提案（已獲方向性確認）｜ 上級計畫：[DEV_PLAN_V03](DEV_PLAN_V03.md)
> 外部依據：Charon（Aeneas 生態，rustc driver，將 MIR 提取為 ULLBC/LLBC），
> 2026-09 Aeneas 技術報告顯示其仍活躍擴展（多 target 提取、loop/iterator 泛化）。
> Charon 授權 Apache-2.0，作為**外部進程 + 資料格式**消費，無授權衝突。

---

## 1. 我嘅睇法（判決：強烈支持，附三個前提）

### 1.1 佢從結構上消滅「地真值漂移」

過去 48 小時我哋用差分測試 + rustc 地真值破咗嘅案，恰好證明手寫 parser 嘅結構性代價：

| 事故 | 根因 |
|---|---|
| v1 適用域 86/100 案例無法解析 | 手寫 parser 永遠追唔上 Rust |
| `mut_borrow_exclusive` 矩陣錯標 UNSAT | 無人知道 rustc 真實判定，期望係估嘅 |
| `ref_deref` 真缺口（E0614 無檢） | 手寫 borrowck lint 永遠係 rustc 嘅劣化複製 |
| Rule B（≥2 `&mut` 即衝突）一上架就被差分證偽 | 再發明 rustc 必定出假陽性 |

接入 Charon = **rustc 本體輸出**。全部「我哋以為嘅 Rust 語義」同「真實 Rust 語義」之間嘅漂移，由結構上消失。rustc 就係地真值，唔洗再仲裁。

### 1.2 MIR/LLBC 係脫水後世界——polyrust 專注返真正護城河

Charon 輸出已經完成：宏展開、方法解析、型別推導、模式降糖（`if let`/`match`→SwitchInt）、借用檢查、drop 插入。polyrust 唔使再維護三個半手寫 parser（v1/v2/v3+lint），
而係集中喺佢**冇替代品**嘅核心：CDCL(T)×Lazy GB×QAP 代數引擎、三值 UNKNOWN 語義、
LRAT 證書、效應/契約紀律。維護成本直線下降，價值密度直線上升。

### 1.3 三個必須認賬嘅代價（前提）

1. **代數複雜度唔會消失**：GB 2ⁿ 天花板研究（task-2 研究報告）仍然有效——Charon 係前端答案，唔係求解器答案。bounded unrolling + fuel + 誠實 UNKNOWN 依舊係複雜度保險絲，缺一不可。
2. **TCB 擴大要誠實入賬**：type/borrow soundness 改由 rustc 擔保 → rustc + Charon 加入信任基。要喺 TCB 白皮書明文記錄邊界改變；Charon 標記 `missing` 嘅宣告**不得給 SAT 證書**，判定降級 `UNKNOWN(missing_decl)`。
3. **Charon 工具鏈耦合**：Charon 要特定 nightly rustc 版本（pin）。無 Charon 嘅環境要優雅降級去 legacy 路徑（同 Lean 靜態嵌入嘅降級策略一致）。

---

## 2. 目標架構（分層）

```
input.rs ──外部進程──▶ Charon（nightly pinned，frontends/charon-bridge 調起）
                            │ 輸出 crate.llbc（serde JSON，LLBC 結構化 AST）
                            ▼
core/charon_llbc.rs    std-only LLBC-subset JSON parser（零依賴紅線不破）
                            │  schema 鎖 pin 版本；snapshot fixtures 入倉
                            ▼
core/polyir.rs         PolyIR —— 最小可解 IR：
                       · locals: Ty{Int{sign,bits},Bool,Tuple,Array,Ref(已消借用)}
                       · blocks: Vec<BasicBlock>，Stmt{Assign,BinOp{op,overflow_flag},
                         Call{callee,contract|inline-bound},Assert{cond,msg,span}}
                       · Term{Goto,SwitchInt{disc,arms,otherwise},CallReturn,Return}
                       · 每節點保留 Charon span（診斷精確到行列）
                            ▼
core/polyir_lower.rs   語義降糖：
                       · SSA 化 + φ 節點 → 選擇子句（ite-polys）
                       · loop → **fuel-bounded unroll**（接現有三值 UNKNOWN 基建）
                       · call → inline（depth≤2 默認）或 pre/post contract 多項式
                       · MIR checked-op 標記 → overflow 約束（rustc 保留嘅 flag 直接用）
                            ▼
┌─────────────────── 以下完全唔變 ───────────────────┐
│ gen_constraints → SystemV2 → CDCL(T)×Lazy GB →      │
│ 三值 verdict + bounded_unknown → QAP/R1CS → LRAT →  │
│ 診斷（span 直達源行列）→ JSON/CLI/frontends          │
└─────────────────────────────────────────────────────┘
```

**語義職責重新劃分（商業賣點區域）**：

| 層 | 負責方 |
|---|---|
| typecheck / borrowck / drop / trait resolution | rustc（TCB，明文入賬） |
| 算術 overflow、array bounds、panic-freedom | **polyrust（新增值面）** |
| 契約 `@require/@ensure`、loop fuel/有界終止 | **polyrust** |
| 效應紀律（io/fd/net/prng…） | **polyrust** |
| UNSAT 證書（LRAT）、SAT 見證（QAP）、三值誠實 | **polyrust** |

現有 `check_borrow_conflicts` / `check_deref_depth` 喺 **Charon 路徑係冗餘**（rustc 已拒收），
保留喺 legacy `.poly` 路徑並標「rustc-gated on Charon path」。

---

## 3. 遷移策略（兼容優先，唔做大爆炸重寫）

1. **輸入分流**：`*.rs` → 新鏈（Charon→PolyIR）；`*.poly` DSL → legacy v3 鏈（保留作 regression corpus，不即刪）。
2. **無 Charon 環境**：frontends/charon-bridge 偵測唔到 `charon` 二進制 → 優雅降級到 legacy v3 parser + 明確 warning（同 Lean 嵌入降級策略同款）。
3. **遷移安全網＝reuse 差分基建**（岩岩起好嗰套）：`differential.rs` 擴三路（v1/v3-legacy/v4-polyir），
   matrix 100 全部變 v4 驗收集；再加 rustc-行為差分（root-case set：每個 semantic case 生成 `main` 執行版，rustc 跑出真實結果對照）。**每個里程碑都係綠燈先可以行下一步**。

## 4. 工程藍圖（里程碑 × 驗收 × 工數粗估）

| 里程碑 | 內容 | 驗收（全部要 CI 可斷言） | 粗估 |
|---|---|---|---|
| **C0 Spike ✅（2026-09-20 轉綠）** | pin Charon（`charon.pin` = `ca501af6`）；152 例（100 matrix + 52 examples/phase3）全過 `charon rustc --edition=2021`；missing rate + E-code 分解有報告；LLBC schema snapshot v1（`{charon_version, translated{…}, has_errors}`） | **117/152 = 77.0%**（≥70% PASS）；有效轉換率（除 16 設計 UNSAT）**86.0%**；charon_err/timeout = 0/0；基線報告 `docs/C0_BASELINE.md` | 已完成 |

> **C0 現況（2026-09-19）**：基建三件套已完成（`charon.pin`、`scripts/c0_spike.py` 全量 harness、
> `scripts/c0_env_check.sh` 一鍵環境）；全量執行**受執行沙箱 2GB RAM 限制**（Charon 構建硬下限 ≈4GB，
> 法證見 `docs/C0_CHARON_SPIKE.md`）；重跑 = `bash scripts/c0_env_check.sh`（≥4GB 機器 15–30 分鐘）。
> 附帶確認：pin 版本已含 multi-target translate+merge（C4 需要嘅 modularity 基礎 ✓）；
> 三態入口（rustc_reject／ok_with_missing→UNKNOWN(missing_decl)／ok）口徑已喺 harness 實作。
| **C1 PolyIR + LLBC parser ✅（2026-09-20）** | `core/charon_llbc.rs`（std-only JSON parser，offset 級錯誤）→ `core/polyir.rs`（結構 lift：lowerable/missing 投影）；10 個 `.llbc` fixtures 入倉（`core/tests/charon_fixtures/`） | 10 fixtures parse+round-trip 全過；unknown root key/未知 body variant/缺 key 三線 hard error 測試覆蓋；pinned schema 13 keys 白名單 + body {Structured|Error|Missing} 白名單；**98/98 lib 測試綠**。殘尾：C2 lowering 接入先開 body 內容類型化 | 1 日完成 |
| **C2 直線 + 分支 lowering** | int/bool 算術、checked-op overflow、if/match（SwitchInt→ite-clause）；接入下半段 | matrix `basic` 全類 v3↔v4 差分一致；sqr/bad `.rs` 版端到端 SAT/UNSAT 同 legacy 逐位一致；lazy/eager 時延對表 | ~1 週 |
| **C3 loop + fuel + UNKNOWN** | LLBC 結構化 loop bounded unroll；`@fuel`/`@invariant` 口徑不變（span→源行 DSL comment 或 `#[polyrust::fuel(N)]` attr 雙軌） | loop_unknown 端到端 UNKNOWN + reason 不變；for/while/match 15 案例差分一致 | ~1 週 |
| **C4 calls / contracts** | inline depth 默認 2 + contract 模式（`@require/@ensure` → pre/post polys）；panic-freedom 檢查面 | struct/impl/trait 案例差分推進；commercial 10 案例至少 6 案例新鏈可判（含 enterprise_ide 目標修復 false-UNSAT） | ~1 週 |
| **C5 預設切換 + Lean 補強** | `check *.rs` 走 v4；v1 parser 正式 deprecated（保留 v1.0 tag 可回退）；Lean：`Polyir.UnrollSound`（bounded unroll ⊂ 原語義）定理；TCB 白皮書補 Charon/rustc 章節；README/EVIDENCE 更新 | 全部 CI 綠；obligations/audit 鏈新輸出 sanity 過；白皮書 TCB 邊界圖更新 | 1–2 週 |

**並行 CI job**（每個 milestone 嗰刻起生效）：`charon-pin`（工具鏈一致性）、`llbc-snapshot`（schema 漂移檢測）、`differential-3way`（v1/v3/v4 ratchet）。

**總工數粗估 4–6 週一人全職**；每里程碑獨立可交付、獨立可回退（新鏈唔穩就退回 legacy，輸入分流保證無縫）。

## 5. 風險登記

| 風險 | 緩解 |
|---|---|
| Charon × nightly 版本耦合 | `charon.pin` 鎖 commit + CI 每日 canary job；升版本係顯式 PR（schema snapshot 會自動紅） |
| LLBC schema 跨版本漂移（JSON 非穩定接口） | exhaustive parser + snapshot fixtures + 升級 PR 強制帶新 fixtures |
| Charon 不支援面（dyn Trait、GAT、async、RPIT——按 Charon 現有文檔） | missing-decl 口徑：該宣告唔畀 SAT，判定降 `UNKNOWN(missing_decl)`；frontends 降級 legacy |
| 複雜度爆發（full-Rust 程序落 GB） | fuel/bound 上限 + UNKNOWN 誠實降級；FPT/caching 研究（P1-D2/D3）原路接入 PolyIR |
| 團隊規模 / 中途爛尾 | 每里程碑綠燈先行、輸入分流可隨時回退、差分 ratchet 全程守底 |

## 6. 對現有計畫嘅 supersede/accelerate 清單

| DEV_PLAN_V03 項 | 影響 |
|---|---|
| P0-C4 Lazy GB Lean 精化 | **保留**（後端不變） |
| P0-C5 bounded-SAT 標記全鏈路 | **加速**——直接變 fuel 機制嘅通用層，新鏈原生三值 |
| P1-D2 FPT 前置消解 / P1-D3 切塊 | **不變**，輸入改由 PolyIR 餵 |
| v1 parser 擴闊（P2 類 86-case 適用域下沉） | **作廢**——最大收益：parser 維護成本歸零 |
| P0-C1 上線嘅 DerefLint / 借用 lint | 新鏈冗餘；legacy 保留標「rustc-gated」 |
| Eurydice（Charon→C）旁路 | **留意**——之後可以做「證書版 C 輸出」，商業故事更大 |

## 7. 一句話總結

Charon 買嚟嘅嘢：**rustc 語義地真值 + 全語言覆蓋 + parser 維護成本歸零**。
我哋要付出嘅嘢：**TCB 擴大要誠實、schema/工具鏈要 pin 死、GB 複雜度照舊要 fuel 保險絲**。
淨收益明確——polyrust 從「一個 Rust 子集嘅實驗驗證器」升格做「rustc 語義錨定嘅代數驗證前端」。
