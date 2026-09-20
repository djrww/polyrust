# polyrust 後續開發計畫（統一重排版 v3）

> 版本：v0.3-plan ｜ 日期：2026-09-19 ｜ 維護：polyrust dev
> 來源：合併重排兩份研究——《v0.2.2 落地審查》（工程/商業缺口 P0–P2）與
> 《突破 GB 2ⁿ 天花板研究》（方法論路線 S0–S5）。本文件為**唯一當前計畫**，
> 舊計畫檔（COMMERCIAL_LANDING_PLAN 等）以歷史文件看待。
>
> 圖例：✅ 已完成（v0.3-hardening 批次，2026-09-19）｜🟡 部分完成｜⬜ 未開始｜🔒 紅線

---

## 0. 北極星（一句話）

**對展開後的完整 Rust 子集，給出 SAT/UNSAT/UNKNOWN 三值判定，每個判定附多項式時間可驗、
形式化背書的證書；代價 = poly(n)·2^{O(c)}，c 為真歧義選擇點——先做到「判定快、證書真、口徑誠實」，
再擴子集與性能。**

> **v0.4 戰略決議（2026-09-19，方向已確認）**：前端接入 **Charon + PolyIR** 分層
> （rustc 語義地真值、parser 維護成本歸零、TCB 擴大誠實入賬、GB 複雜度靠 fuel 保險絲不變）——
> 完整論證同藍圖見 [CHARON_POLYIR_BLUEPRINT.md](CHARON_POLYIR_BLUEPRINT.md)。
> 受影響項：v1 parser 擴闊作廢；P1-D2/D3 輸入改餵 PolyIR；差分基建擴三路（v1/v3/v4）。

三條戰備紅線（🔒，寫入開發規約）：
1. 任何判定不得偽造證書或偽造判定（verified=false 如實上報；UNKNOWN 不得降格為 SAT/UNSAT）。
2. 全量規約 Gröbner 基按_Eager（審計/證據）_ 與 _Lazy（默認判定）_ 分層；不得讓急於全基回流默認路徑（PR 需附 benchmark）。
3. 文檔口徑單一化：測試數/定理數/性能數字由腳本或 CI 產生；README 不手寫易漂移數字。

---

## 1. 本批已完成（v0.3-hardening @ dev/v0.3-hardening，2026-09-19）

| 項 | 內容 | 驗收（實測） |
|---|---|---|
| ✅ H1 倉庫衛生 | 刪 `--help/`、`core/core/` 事故目錄；.gitignore 補齊 | 樹內無生成物 |
| ✅ P1 Lazy GB | `GbBasisMode::{Eager,Lazy}`；默認 Lazy 跳過最終全基（T3/T6 保證）；`run_pipeline_eager` 保留給 obligations/demo/exhaust/engine；`--eager-gb`/`POLY_EAGER_GB=1` 開關；JSON `gb_mode` | sqr `check` **23.4s→1.89s（-92%）**；SAT/QAP/rustc 逐項一致；bad UNSAT 0.03s；eager 對照逐位一致（basis 177） |
| ✅ C2a 移除偽造 QAP | 刪除 pipeline_v2「verified=false 強制改 true」邏輯，註釋存檔 | 69/69 測試綠（含 commercial_pipeline QAP 斷言，真實驗證自然通過） |
| ✅ U1 UNKNOWN 首步 | 顯式 `@fuel` 不足 + 無 `@invariant` → `verdict=UNKNOWN` + `unknown_reason`（保守：不誤報） | loop_unknown.poly `SAT(誤標)`→**UNKNOWN**；loop_sat 維持 SAT 無回歸 |
| ✅ L1 LRAT 核 | `core/src/lrat.rs`：RUP-only 逐步複核器（std-only、無 unsafe）+ drat/DIMACS 導出 + 6 測試 | 正/反/邊界用例全過 |
| ✅ T1 矩陣 ratchet | 語義矩陣門檻 70%→**94%**（P0-C1 後再收緊至 **96%**）+ 白名單 ratchet；新失敗=回歸紅 | 96/100 鎖定，白名單 4 項（async×2, io_with_pure_call, enterprise_ide） |
| ✅ G1 CI 閘門縫 | `grep passed`（恆真）→ 斷言 `test result: ok` 且無 `FAILED` | workflow 已修 |
| ✅ D1 口徑修正 | README/EVIDENCE 測試數 17/67 → **63**（現 69，含 lrat） | grep 無舊數字 |
| 🟡 W1 警告清理 | `cargo fix` 套用 21 項：lib 警告 35→14 | 餘 14（多為 dead_code，列 P2） |

---

## 2. P0 — 正確性與信任（下批最優先）

| # | 缺口 | 具體任務 | 驗收標準 |
|---|---|---|---|
| P0-C1 ✅ | ~~v3 借用檢查弱化~~（2026-09-19 結案） | v1↔v3 **差分測試**（`core/src/differential.rs`）已納 CI-ratchet：100 案例，白名單 1（io_effect=v1 過嚴）+ v1 適用域 86；rustc 地真值查明：mut_borrow_exclusive 舊期望屬錯標（rustc 合法、v1 SAT）→ 矩陣改 SAT；ref_deref 為真缺口 → `check_deref_depth`（E0614 語義）補強。曾試行 Rule B 經差分**證偽回退** | 差分綠、矩陣 94→**96/100**、端到端 ref_deref=UNSAT+診斷、exclusive=SAT ✓ |
| P0-C2 ✅ | **LRAT 接入 CDCL**（S1 後半）（2026-09-20 結案） | cdcl.rs 學習子勺記錄為證明軌；UNSAT 時輸出 `proof.drat`（`POLY_LRAT_OUT`，配 `POLY_LRAT_CNF` 公式）並以 lrat.rs 自檢；Lean `Polyrust.RupKernel` 可靠性定理（接受⇒CNF 無解） | UNSAT 語料（7 合成 + matrix 全 should_sat=false 類）全產證自檢過（`lrat_unsat_certificates_across_unsat_corpus`）；AuditAll CLEAN（4269 受檢、0 sorry、0 非標準公理）；`1∈G` 證書退為 `--audit` 模式（=`--eager-gb` 別名，JSON 路線同生效）。修復一宗設計缺陷：證書基底原取 `clauses[..n_orig]`（剔除單元/空/恆真子句嘅入庫表）⟹ 基底≠實際輸入、輸入含空子句時偽失敗；改為構造輸入逐字鏡像 `cnf_base`（理論補理作可信公理入基底，對齊 SMT 實踐） |
| P0-C3 ✅ | ~~授權不統一~~（2026-09-19 結案） | **決策：全產品 AGPL-3.0 + 商業雙授權**——唔做 permissive core。「core 咪最值錢既地方，core 畀人任用，其他嘢唔值錢」— 用戶原話；全部 142 源檔 SPDX `(AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)`、`license-scan.sh` CI 硬閘門、LICENSE.MIT 刪除、LICENSE.COMMERCIAL 轉 v1.0（聯絡 TBA）、Cargo.toml `license="AGPL-3.0-only"`、README 授權段 | license-scan 綠；無 *.example 殘留 ✓ **手尾：首次商業發佈前補真實聯絡（LICENSE.COMMERCIAL/COMMERCIAL_NOTICE.md 兩處 TBA）** |
| P0-C4 ✅ | **Lazy 模式的 Lean 精化證明**（2026-09-20 結案） | `Polyrust.LazyGb.lean`：立方體否句語義（`negCube_none_holds_iff_agree` 等）、迴圈步合法（`negCube_step_valid`）、合法補理鏈組合（`ValidChain`/`validChain_strengthens`）、主線 `lazy_unsat_sound` + `eager_sat_of_lazy_witness` ⟹ `lazy_refines_eager`（兩判定數學重合；鏈步理論前提如實申報由 GB 引擎承擔） | 定理入庫（7+1）；`eager_sat_of_lazy_witness` 零公理依賴，其餘全喺標準三公理內；AuditAll CLEAN |
| P0-C5 ✅ | **bounded-SAT 標記全鏈路**（2026-09-20 結案） | v2/v3 JSON 新增 `bounded:{kind,fuel,insufficient}`（觸發條件同約束生成端一致：顯式 @fuel 或源含 loop/while/for ⟹ fuel 顯式值/3）；**默認 fuel=3 路徑納入 UNKNOWN 誠實三值**（兌現 P1-U2 承諾）；v2 `api_version` 0.2→0.3、v3 三值對齊穿透；CI fail-closed 語義文件化 | 契約文檔 `docs/JSON_CONTRACT_V03.md`；測試 4 新增全綠（純函數 2 + JSON 契約 2），核心 lib 131/131 綠零回歸 |

## 3. P1 — 性能與可用性（FPT 路線落地）

| # | 任務 | 驗收 |
|---|---|---|
| P1-D1 | CDCL 現代化（VSIDS/相位保存/重啟/LBD 裁减；現 468 行為基礎版） | 硬實例基準提升 ≥10×；obligations 6min→<2min |
| P1-D2 | FPT 前置消解：unify/region 閉包在約束生成前消去確定變量；JSON 暴露自由度殘量 `ambiguity_c` | 12 樣本 c≪n 有報告；1000 行級 .poly 近線性擴展 |
| P1-D3 | 函數級子系統切塊（per-body 系統 + 邊界變量） + treewidth min-fill 排序 | 大例 GB 階成本按塊縮放；treewidth 報告入 stats |
| P1-D4 | 診斷訊息：`Diagnostic{span,code,help}`；UNSAT 衝突核映射回 .poly 行號 | check JSON 新增 diagnostics；IDE/前端可高亮 |
| P1-D5 | v2 QAP 真見證：sigma_f 由手填 first-bit 改為真求解（對齊 Lazy+T8）；product/sum 跨節點約束入 R1CS | v2 QAP 覆蓋不再僅 field polys；「僅展示流程」註釋可刪 |
| P1-D6 | 𝔽₂ 商環 + ZDD 多項式表示實驗（PolyBoRi 路線，std-only 新模組 zdd.rs）＋F4 並行稀疏消元（std::thread 分片） | PolyBoRi 類基準（乘法器實例）可解規模 ≥4×；audit 模式同答案 |
| P1-D7 ✅ | ~~測試補齊：frontends/http、llm、ide 目前 0 測試~~（2026-09-20 結案） | http 6（health 契約／nl 400×2／nl mock 離線回路／funnel 404·422·200）、llm 5（mock 回落核心／名稱契約／自訂名 OpenAI 相容／環境敏感路徑／flag 契約，全零網絡）、ide 7（health×2／open memory·404／save 版本推進／檔案樹遍歷／compile 迷你源回路）。全部 handler 直調，無 tower/瀏覽器需求；workspace 測試 173/173 綠 |

## 4. P2 — 產品化與生態

| # | 任務 |
|---|---|
| P2-E1 | serve/HTTP 硬化：auth token、限流、`--bind` 參數（現 0.0.0.0 裸奔）、JSON 契約 semver、OpenAPI |
| P2-E2 | 真 LSP（tower-lsp；依賴 lsp-types/ropey 已在樹內未用）+ VSCode extension MVP（診斷、N badge、quick fix） |
| P2-E3 | Solana 去水分：改述「生成可部署驗證合約骨架」或 devnet 實部署一次存證；證書哈希從 cert_account 讀、移除硬編碼 `[0xAB;32]` |
| P2-E4 | PolyCache 真增量（存真 basis/節點級失效；現僅計數）；IDE/LSP 實時基礎 |
| P2-E5 | 發佈物：棄「分支即版本」、tag+Release、crates.io 發 core、Docker 鏡像、可攜 `-march=x86-64-v3` 變體 |
| P2-E6 | 全 Rust 有界角落：d 界 trait 求解、const/macro 燃料化、三值全鏈路；Lean 三定理（DeterministicPropagation/TreewidthBound/ThreeValuedSound/BoundedTrait） |
| P2-E7 ⏭ | 對外基準：與 rustc/clippy/Kani/Prusti 判定一致率報告；~~TCB 白皮書~~ | **TCB 白皮書已由 C5.3 達成**（`docs/TCB_WHITEPAPER.md` 四層信任圖，2026-09-20）；一致率基準屬大型對外活動，待獨立排期 |
| P2-E8 ⏭ | ~~warnings → 0~~；`cargo fmt` 一次性清倉 + clippy 轉硬閘門（在獨立 PR 做，避免污染功能 diff） | **warnings→0(lib) 已由 T5 審計達成**（style 級 ~87 項已認領於審計文檔 §4）；fmt 清倉與 CI 硬閘門按原註仍留獨立 PR |
| P2-E9 | Lean 原生庫可攜化與 `-march=native` 分離（README 已自知不可攜） |

## 5. 指標看板（每 release 由 CI 產生）

- 判定：單元測試數（現 69）、語義矩陣通過率（基線 94%→目標 100%）、v1↔v3 差分一致率（目標 100%）
- 性能：sqr check 延遲（現 lazy 1.9s / eager 25s）、obligations 時長（現 ~6min）、可解實例規模（ZDD 後重測）
- 信任：Lean 模組/定理數（AuditAll 宣告數）、LRAT 產證覆蓋率（UNSAT 案例附證書比例）
- 倉庫：warnings（35→14→0）、文檔漂移檢查（腳本對賬 README 數字）

## 6. 風險登記（節選）

| 風險 | 緩解 |
|---|---|
| Lazy 判定與 Eager 出現分歧（若 T3/T6 覆蓋有洞） | CI 同跑兩模式差分（S0 已留 --eager-gb）；P0-C4 Lean 精化 |
| UNKNOWN 泛濫降低可用性 | 保守觸發原則（寧漏勿濫）；fuel/i nvariant 模板庫；度量 UNKNOWN-rate |
| v3 差分修復牽動商用管線行為 | 白名單 ratchet 控制節奏；每修好一例即鎖一行 |
| 授權切換的法律抖動 | 全檔頭掃描先於切換；舊 tag 保留原授權不動 |

---

### 附：本批代碼證據（2026-09-19，dev/v0.3-hardening）

```
[1] sqr lazy   : verdict SAT, QAP✓, rustc✓      1.89s   （基線 23.4s，-92%）
[2] sqr eager  : verdict SAT, basis 177, QAP✓   25.35s  （行為凍結對照）
[3] bad lazy   : verdict UNSAT, QAP None        0.031s
[4] check-v2 loop_unknown.poly  → verdict UNKNOWN（unknown_reason: loop fuel 不足：while x < 100 約需 100 次迭代）
[5] check-v2 loop_sat.poly      → verdict SAT（無回歸）
測試           : 69/69 綠（63 舊 + 6 lrat）；semantic matrix 94/100 鎖定白名單 6 項
PL_DBG 計時定位 : S6 全量規約基 21.52s/23.4s = 92%（F4，170 vars/314 polys）
```
