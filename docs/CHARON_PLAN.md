# 工程藍圖：Mini-Rust 子集 → rustc + Charon + IR layer

> 狀態：**定案稿 v2（2026-09-20）** — §7 五題已拍板（見 §7 決議欄），本檔即正式規格。
> 前置：步驟 1–3（T10 管線接線、Chalk 判決橋、MIR lowering 雛形）已完成並入庫（`b4673fe`）。

---

## 0. 目標與不可妥協的不變量

**目標**：以 rustc（經 Charon）取代自製 Mini-Rust 前端（lexer/parser/宏/typeck/checker），
令 polyrust 檢查**真 Rust 程式**；代數層（CDCL(T)×Gröbner×QAP）與認證層原封不動重用。

**不變量（任何設計不得違反）**：
1. **core 零第三方依賴**（std-only）——第三方依賴只准喺 `frontends/*`
2. **Lean 零 sorry、零自訂公理**（`AuditAll = CLEAN` 鐵律）
3. **判定一致性鐵律**：代數判定 ⟺ rustc 判決（差分義務鎖定，見 M4）
4. **T4/T10/∏kᵢ 界不動**：新前端只改「約束由邊來」，界與終止性論證全數重用
5. **認證路徑為日常主路徑**：rustc 判決 → artifact → σ 重建 → 直接求值（多項式時間）

---

## 1. 現狀盤點：換乜、留乜

### 換走（成為 legacy，M4 之前保持後備）
| 組件 | 原職 | 由邊個接手 |
|---|---|---|
| `minirust/lexer,parse*,ast,ast_v2,ast_full` | 詞法/語法/AST | rustc 前端（經 Charon） |
| `minirust/macros` | macro_rules! 衛生展開 | rustc 展開（Charon 出嘅係展開後 MIR） |
| `minirust/ty,checker,universe` | 直接型別檢查（ground truth） | rustc typeck；polyrust 只做代數重推 |
| `minirust/constraints,constraints_v2,lower` | 位元約束生成 | **新 IR 層 + 編碼器**（§3 L3） |
| `minirust/analysis,borrowck` | 借用活性（subset 級） | **代數 borrowck 重推**（M3，研究核心） |

### 留低（原封重用）
| 組件 | 角色 |
|---|---|
| `poly,fp,frac,groebner*,cdcl,qap` | 代數引擎（零改動） |
| `composition,vanishing,certify` | T10 分解、消失編碼、認證路徑 |
| `chalk_bridge` | oracle artifact 模式（升級 v2，見 M4） |
| `minirust/mir_lower` | 由玩具升格為 IR 層嘅種子（值域抽象、L0′ 門檻全數保留） |
| Lean 側全部 45 jobs | `Squarefree/Composition/BoundedStandard/...` 不動；新增見 M5 |

---

## 2. Charon 事實核查（2026-09-20）

- Charon 係 rustc driver：喺 crate 內以 `cargo charon` 方式運行，輸出整個 crate 嘅
  **JSON**（`.llbc` / `.ullbc` 檔）：型別、函數、trait、**簡化 MIR body**、語義資訊
- **ULLBC** = 簡化 MIR（CFG）；**LLBC** = 控制流重建（loop/if-then-else 結構化）
  ——LLBC 保留 move/copy/reborrow、重建常數、淺 pattern match、checked 運算
- **依賴 nightly**：charon 用 nightly 編譯（driver 需求），nightly 版本由佢嘅
  `rust-toolchain` 釘住；**alpha 軟件，官方明言 API 有 breaking changes 計劃中**
- 讀取方式：`charon-lib` crate（serde）或 charon-ml；JSON 自描述

**關鍵推論**：Charon 嘅輸出係 JSON —— 我哋可以沿 `frontends/full`（syn Oracle）先例，
喺 `frontends/charon` 用 serde/charon-lib 讀佢，**轉成自家 `polyrust-ir/1` schema**，
core 只需用（已在 `chalk_bridge.rs` 落地嘅）mini JSON 讀取器——**core 保持零依賴**。

---

## 3. 目標架構（三層）

```
L1 抽取層（ Nightly, 依賴重）
  rustc + charon（釘版本）
      │  crate.llbc.json（第三方 schema，大）
      ▼
L2 IR 層（frontends/charon，允許 serde/charon-lib）
  charon LLBC → PolyrustIR（自家中間表示：bodies/locals/ops/terminators/
  types/aggregates/loan events）→ polyrust-ir/1 JSON（自家 schema、版本化、精簡）
      │  只含代數編碼所需欄位（~KB 級/body）
      ▼
L3 代數層（core，零依賴不變）
  IR reader（mini JSON，比照 chalk_bridge）
  → 編碼器（mir_lower 升格：位元串（型別/臂/借用）× 消失多項式（多值/算術））
  → T10 逐 body 分解 → CDCL(T)×GB → σ → QAP/codegen
  → 認證：certify + artifact（oracle v2）
```

**設計原則**：L1→L2 之間係 charon schema（會變）；L2→L3 之間係**自家穩定 schema**
（我哋控制、版本化、向後相容）。charon 上游升級只撞 L2，L3 永遠穩陣——同
chalk_bridge「rustc 判決 → artifact → 認證」同一哲學。

---

## 4. 里程碑（每個有驗收準則）

### M0 環境與 spike（先行，風險探測）
- 裝 charon（釘 release tag + 佢指定嘅 nightly）；`rust-toolchain.toml` 入 repo
- 對 P1–P12 移植版（真 Rust crate）跑 `cargo charon`，攞 `.llbc.json`
- 驗收：至少 1 個 SAT 樣本（P6 借用類）攞到 LLBC，body 大小/結構可讀
- **產出**：`scripts/setup_charon.sh`、樣本 crate `corpus/rust-samples/`、JSON 節錄

### M1 IR 層
- `frontends/charon`：LLBC JSON → PolyrustIR（struct 定義）→ `polyrust-ir/1`
- core：`ir_reader.rs`（mini JSON 讀取器）+ IR 型別（`minirust/ir.rs`）
- 驗收：P1–P12 全部 round-trip（charon→IR→core 讀入一致）；schema 版本測試
  （未知欄位跳過、缺欄位報錯）

### M2 編碼器（mir_lower 升格）
- 語句覆蓋：賦值（copy/move 語義）、二/op（**checked**：overflow ⇒ panic 分支 →
  兩域 case 編碼）、條件/switch（分支 ctx 乘法，比照 macro 選臂）、聚合/enum
  （判別式 vanishing + 欄位變數）、呼叫（逐 body 分解、簽名級耦合）
- 值域抽象沿用 mir_lower（≤64 封頂）+ L0′ 門檻
- **循環（invariant 路線，已定案）**：loop 頭 → invariant slot；自動推導簡單形
  （常數初始化循環、計數器上界）；推唔出 ⇒ 該 body `certified=false` 降級，
  絕不 unroll 硬爆。歸納契約的健全性由 `LoopContract.lean` 承接（M5 補引理掛接）
- 驗收：MIR 樣本代數判定 ⟺ 期望；每 body 組件分解報告（T10 格式）；
  含 loop 樣本至少 1 例：有 invariant 時全綠、無 invariant 時如實降級

### M3 代數 borrowck 重推（研究核心）
- 由 IR move/copy/borrow 事件推 **loan/活性**事實（Polonius-lite；
  `analysis.rs` 規則升級到 MIR 級）
- 編碼：借用衝突 ⟹ 約束（沿 P5/P6 鐵律語義）；NLL 區間重推係我哋做，
  charon **唔會**畀 region 資訊
- 驗收：借用 corpus（含 NLL 邊角：temp-borrow、two-phase、shadowing）全數
  同 rustc 一致

### M4 差分義務 + 認證閉環
- **SAT 側**：rustc 接受嘅程式 → 代數必 SAT 且 σ 認證通過
- **UNSAT 側（重要！）**：charon 抽唔到被 rustc 拒絕嘅 MIR（編譯失敗就冇 MIR）
  ⇒ 改用**突變測試**：喺 IR 層注入違規（翻轉型別選擇/激活借用衝突）⇒ 代數必 UNSAT
  ——`chalk_bridge` 測試已示範此模式
- `oracle v2` artifact：rustc 判決（pass/diagnostic）+ charon 元資料 → 認證
- 驗收：`obligation_t11`（前端忠實性）12+ 樣本全綠

### M5 Lean 與文檔
- `Polyrust.IrFaithful`（T11 陳述骨架：IR 語義 → 編碼可靠性/完備性，沿用 T1/T2 模式）
- THEOREMS.md §12、LEAN.md、EVIDENCE.md 更新；舊前端標記 legacy

---

## 5. 特別注意（坑）

1. **工具鏈 churn（最高風險）**：charon 係 alpha、API 有計劃中 breaking changes、
   nightly 釘版跟 charon release 走。對策：**釘 release tag**（唔好追 main）、
   升級係明確 chore（獨立 PR、跑全部義務）、CI 記錄 charon version hash
2. **被拒程式冇 MIR**：rustc 編譯失敗 ⇒ charon 無產出 ⇒ UNSAT 側唔可以「攞被拒源碼
   餵代數」——必須走突變測試路線（M4）。呢個同直覺相反，一開始就要定好
3. **借用語義重疊**：charon 編譯過嘅 crate 已經 borrow-clean；我哋嘅價值唔係
   「再發現 rustc 漏嘅錯」，而係**代數重推 + 機器可驗證證書**。表述唔好亂咁誇
4. **checked 運算與浮點**：`+` 喺 MIR 係 checked（overflow ⇒ panic 分支，要編碼）；
   浮點**唔係**𝔽_p 代數語義——MVP 直接拒（報「不支援浮點」），後續先諗位向量編碼
5. **L0′ 係硬門檻**：真 MIR 算術鏈（已實測 2.6e18 平方鏈被拒）要政策：值域封頂、
   超限拒絕、或該 body 降級「以 rustc 判決為準、certified=false」——唔可以靜默過
6. **循環（已定：invariant 路線）**：LLBC 有結構化 loop 但代數上係不動點。
   首選**歸納契約**：loop 頭帶 invariant slot——invariant 可自動推導（常數歸納/
   單調計數器等簡單形）或由 `.poly` 註解提供；無 invariant 又推唔出 ⇒ 該 body
   **降級** `certified=false`（以 rustc 判決為準），唔好用 unroll 硬爆。
   `LoopContract.lean`（歸納契約）+ `contracts.rs` 已有理論與雛形，M2 落地
7. **泛型/trait**：charon 輸出保留泛型結構（trait 解已附）；代數需要單態實例。
   **MVP 只收單態 corpus**；泛型延後（可行路：經 rustc monomorphize 輸出）
8. **JSON 體積**：真 crate LLBC 以 MB 計。對策：L2 只抽取需要的 bodies、
   `polyrust-ir/1` 精簡 schema（KB 級）、core 串流讀取、PolyCache 用 body hash 做 key
9. **環境可復現**：nightly ~GB 級下載；sandbox/CI 要 `scripts/setup_charon.sh`；
   工具鏈唔入 workspace snapshot——文檔寫明「跑 M0 前先執行 setup」
10. **授權**：charon Apache/MIT；經 JSON 接口消費，core 無連結 ⇒ 無授權污染
    （AGPL 議題不受影響；frontends/charon 嘅依賴合規照現有 frontends 慣例審計）
11. **舊前端退役**：Mini-Rust 唔好即刻刪——佢係而家 99 個測試同 10 條義務嘅地基。
    新路徑 `PL_FRONTEND=charon` 旗標並行，等差分義務連續 N 次全綠先轉正
12. **IR 命名空間**：`mir_lower.rs` 嘅 `MirBody` 會升格；避免同 charon 嘅 "MIR" 概念
    混淆——自家 IR 命名建議 `PolyrustIR`（body/stmt/op 前綴 `pir_`）

---

## 6. 風險表

| 風險 | 概率 | 影響 | 緩解 |
|---|---|---|---|
| charon 升級破壞 L2 | 高 | 中 | 釘版 + schema 版本協商 + 升級 chore 流程 |
| borrowck 重推同 rustc 唔一致（NLL 邊角） | 中 | 高 | 差分 corpus 逐案對齊；不一致樣本入 corpus 永久鎖定 |
| L0′ 大面積拒絕算術 body | 中 | 中 | 值域封頂政策 + certified=false 降級路徑 |
| nightly 環境喺 CI/sandbox 不可復現 | 中 | 中 | setup 腳本 + 版本鎖 + 文檔 |
| JSON 解析效能（MB 級） | 低 | 低 | L2 抽取精簡、串流、body-level hash |
| 範圍蔓延（想做晒 typeck+borrowck+浮點） | 高 | 高 | MVP 範圍由 §7 Q2 決定，里程碑唔越界 |

---

## 7. 決議（2026-09-20 已拍板）

| # | 問題 | **決定** | 執行要點 |
|---|---|---|---|
| Q1 | 接線形態 | **(a) `frontends/charon`** | serde/charon-lib 只入此 crate；core 讀 `polyrust-ir/1`（mini JSON，比照 chalk_bridge） |
| Q2 | MVP 語義範圍 | **(c) 分期全做** | M1 IR → M2 編碼 → M3 代數 borrowck（主菜）；typeck 一路經 rustc 判決 + artifact |
| Q3 | UNSAT 側差分 | **(a) 突變測試**（跟建議） | IR 層注入違規 ⇒ 代數必 UNSAT；被拒源碼不可經 charon 抽 MIR，此路不通 |
| Q4 | 循環政策 | **(c) invariant 路線**（超出建議——採研究題為 MVP） | loop 頭 invariant slot：自動推導簡單形 + `.poly` 註解；推唔出 ⇒ `certified=false` 降級，**唔做 unroll 硬爆**。M5 要補 `LoopContract.lean` 掛接引理 |
| Q5 | 舊前端命運 | **(b) legacy 並行一個版本週期** | `PL_FRONTEND=charon` 旗標；差分義務連續全綠一個週期後再議刪除 |

**Q4 決定的連鎖調整**：M2 的 loop 編碼改為 invariant slot + 自動推導 + 降級路徑
（見 §4 M2）；里程碑總量不變，但 M2 工程量↑、M5 增加 `IrLoop` 歸納契約引理。
