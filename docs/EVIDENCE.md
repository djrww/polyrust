# 實測證據：12 樣本 × 九條義務 × Lean 定理

本文件把 **Rust 側的實測輸出** 與 **Lean 側的機械證明** 對齊。它是
[`docs/LEAN.md`](LEAN.md) 的證據附錄，也是 `docs/THEOREMS.md` §12 的展開。

**一句話結論**：Rust 側 9/9 義務自證、17/17 單元測試、4/4 demo 全數通過
（12 個程序樣本逐項見證已存檔）；Lean 側 273 條定理、零 `sorry`、零自訂公理全數
編譯通過；兩側的每一項都在下表有明確對應，且**未覆蓋的範圍也已標明**。

---

## 0. 如何重現

```bash
bash scripts/lean-evidence.sh      # 重建 lean/Audit.out 與 docs/evidence/*.raw.txt
```

原始輸出（未經人工編輯，含時間戳與工具鏈版本）：

| 檔案 | 內容 |
|---|---|
| `docs/evidence/env.txt` | 工具鏈版本 |
| `docs/evidence/obligations.raw.txt` | `polyrust obligations`：九條義務自證（12 樣本細節），耗時 6 分 37 秒 |
| `docs/evidence/demo.raw.txt` | `polyrust demo`：四個 demo 的完整管線報告（含 `rustc` 實編） |
| `docs/evidence/tests.raw.txt` | `cargo test --release`：17/17 |
| `lean/Audit.out` | 逐定理 `#print axioms`（84 條受檢） |

環境：Lean **4.33.1**（commit `819816b`，`lean/lean-toolchain` 固定）＋ Lake 5.0.0-src；
rustc/cargo **1.98.1**；無外部依賴（Lean 側無 Mathlib、Rust 側無 crate）。

---

## 1. 九條義務自證（`obligations.raw.txt` 摘要）

| 義務 | 結論 | 實測見證（節錄） |
|---|---|---|
| **T1** 編碼可靠性 | PASS ✓ | 7 個良構樣本（P1, P3, P6, P7, P8, P10, P11）：σ_D 是系統的根 ✓ |
| **T2** 編碼完備性 | PASS ✓ | 同 7 樣本：σ 解碼 ⟺ 推導 ✓ |
| **T3** 子句–多項式對偶 | PASS ✓ | CDCL 判定與 GB 判定一致；學習子句範式皆 0 ✓；PHP(4,3) UNSAT ✓ |
| **T4** Buchberger 終止性 | PASS ✓ | 12 樣本擴充次數：0–645，全部 ≤ 2ⁿ |
| **T5** S-多項式準則 | PASS ✓ | 280 配對抽檢全歸零；策略無關（P1/P9/P12）；消除率 **712,549 → 37,823（95%）** |
| **T6** Gröbner 判定定理 | PASS ✓ | 12 樣本「1 ∈ G ⟺ 檢查器拒絕」逐項一致 |
| **T7** 規範性與宏展開 | PASS ✓ | 策略/順序無關 ⇒ 唯一約化基；P7/P8 選臂可解 ✓、錯臂 1 ∈ G ✓ |
| **T8** QAP 忠實性 | PASS ✓ | 7 個 SAT 樣本 QAP 通過；竄改見證全數被拒 |
| **T9** 端到端 | PASS ✓ | 7 個 SAT 樣本：判定 ✓ round-trip ✓ **rustc ✓** |

單元測試：`17 passed; 0 failed`。四個 demo：

| demo | 來源特徵 | 變量/生成元/子句 | 判定 | 補充 |
|---|---|---|---|---|
| A | `sqr!` 宏 + 函式 | 170 / 313 / 2 | SAT（1 ∉ G） | round-trip ✓ rustc ✓ |
| B | 型別錯（`$e + true`） | 76 生成元 | UNSAT（**1 ∈ G**，基 = {1}） | 2 輪 CDCL |
| C | **兩個重疊 `&mut`** | 101 / 210 / **5** | UNSAT（**1 ∈ G**，基 = {1}） | 4 輪 CDCL、學習 2 條子句 |
| D | 型別導向選臂 | 72 / 145 / 3 | SAT（1 ∉ G） | round-trip ✓ rustc ✓ |

> demo C 是**借用衝突**的端到端案例：5 條子句（3 條型別/臂 + 2 條借用衝突）
> 與 1 ∈ G 同時出現——這正是 `docs/THEOREMS.md` §3（T1）所述「借用子句」的實測面。

---

## 2. 12 樣本主表（Rust 實測 × Lean 定理）

「Lean 定理」欄列出該樣本所**例化**的一般定理：Rust 這一列是**這個樣本**的實測，
Lean 那一欄是**所有同型程序**的證明。

| # | 樣本 | 期望 | T6 實測 | T1/T2 σ_D | T4 擴充 ≤ 2ⁿ | T9 round-trip/rustc | 對應的 Lean 定理 |
|---|---|---|---|---|---|---|---|
| 1 | P1-plain | 良構 | SAT=良構 ✓ | ✓ | 53 ≤ 2⁴⁹ | ✓ / ✓ | `genC_sound`、`genC_complete`、`typable_iff_root`、`parse_gen` |
| 2 | P2-addbool | 不良構 | 1∈G ✓ | — | 0 ≤ 2²¹ | — | `untypable_iff_no_root`、`borrow_one_mem_no_root` |
| 3 | P3-sqr | 良構 | SAT=良構 ✓ | ✓ | 112 ≤ 2⁸⁵ | ✓ / ✓ | `checkCtx_expand`、`right_arm_typable`、`parse_gen` |
| 4 | P4-bad | 不良構 | 1∈G ✓ | — | 1 ≤ 2⁴³ | — | `wrong_arm_untypable`、`wrong_arm_no_root` |
| 5 | **P5-twice-mut** | 不良構（借用） | 1∈G ✓ | — | 48 ≤ 2¹⁰¹ | — | **`p5_conflict`、`p5_unsat`、`borrow_unsat_of_clash`、`borrow_clash_one_mem`** |
| 6 | **P6-temp-borrow** | 良構（借用安全） | SAT=良構 ✓ | ✓ | 70 ≤ 2⁷³ | ✓ / ✓ | **`p6_no_conflict`、`p6_sat`、`borrow_sat_of_clean`、`not_overlaps_of_le`** |
| 7 | P7-pick-int | 良構 | SAT=良構 ✓ | ✓ | 139 ≤ 2⁷² | ✓ / ✓ | `MicroInstance.T7_arm_sat`、`checkCtx_expand` |
| 8 | P8-pick-bool | 良構 | SAT=良構 ✓ | ✓ | 645 ≤ 2⁷² | ✓ / ✓ | `MicroInstance.T7_arm_sat`、`right_arm_typable` |
| 9 | P9-ifmix | 不良構 | 1∈G ✓ | — | 0 ≤ 2²⁸ | — | `genC_complete`（ite 分支型別不一致） |
| 10 | P10-fn | 良構 | SAT=良構 ✓ | ✓ | 127 ≤ 2⁹² | ✓ / ✓ | `UniPoly.qap_duality`、`parse_gen` |
| 11 | P11-shadow | 良構 | SAT=良構 ✓ | ✓ | 71 ≤ 2⁸⁴ | ✓ / ✓ | `genC_sound`（遮蔽 = 新綁定節點） |
| 12 | **P12-assign-bad** | 不良構（所有權） | 1∈G ✓ | — | 0 ≤ 2⁴² | — | **`assign_while_borrowed_unsat`、`borrow_assign_one_mem`** |

讀法舉例（第 5 列）：Rust 側**具體測到** P5-twice-mut 的約束系統含 1（基 = {1}、
48 次基擴充）；Lean 側**一般地證明**「只要存在一對重疊的同變量 `&mut`，系統即無
0/1 根，且 1 有顯式組合見證」——樣本是定理的特例，不是定理的全部。

---

## 3. 借用側：P5 與 P6 的分野（本輪新增）

兩個樣本的原始碼幾乎相同，差別只在**存活區間**：

| | P5-twice-mut | P6-temp-borrow |
|---|---|---|
| 原始碼 | `let r1 = &mut $v; let r2 = &mut $v; *r1 + *r2` | `*(&mut $v) + *(&mut $v)` |
| 存活區間（`analysis.rs`） | r1 `[1,4)`、r2 `[2,4)` → **重疊** | 兩個暫時借用各 `[p,p+1)` → **不相交** |
| Rust 實測 | 1 ∈ G（UNSAT） | SAT；σ_D 是根；round-trip ✓ rustc ✓ |
| Lean 定理 | `overlaps ⟨1,7,1,4⟩ ⟨2,7,2,4⟩`；`p5_unsat`；`borrow_clash_one_mem`（1 的組合：`x₁x₂ − x₂(x₁−1) − (x₂−1) = 1`） | `¬ overlaps ⟨1,7,1,2⟩ ⟨2,7,3,4⟩`；`p6_sat`；`borrow_sat_of_clean` |
| 共同的一般定理 | — | `borrow_sat_iff_clean`：**有 0/1 根 ⟺ 無衝突對且無被排除節點** |

`p5_p6_differ` 把「同一語言、同一變量、同一 `&mut` 數量，判定相反」這件事形式化為
一條定理——**差別只在區間**。

**兩處精確性要點**（實作與形式化都採用，且 Lean 側已證）：

1. 非空區間會**與自身重疊**（`overlaps_self_of_nonempty`），故衝突只在**相異節點**
   之間談（對應 `analysis.rs` 的 `for j in (i+1)..`）。Rust 側的 12 樣本中，
   P6 的兩個暫時借用雖同變量，但彼此是**不同節點且區間不相交** ⇒ 安全。
2. 子句集單獨看**平凡可滿足**（全置 0）；使系統非平凡的是每個活躍借用的
   `b − 1 = 0`。Rust 側 `obligation_t1` 正是把所有 `borrow_vars` 置為 `Frac::ONE`
   來構造 σ_D；Lean 側的 `borrow_sat_iff_clean` 因此必須假設「衝突節點皆活躍」
   （前提 `hpl`/`hal`）——兩側的這個前提是**同一件事**。

---

## 4. 定理 ↔ 義務 ↔ 樣本 三方對照

| 定理 | Rust 實測（樣本/統計） | Lean 一般定理（任意輸入） |
|---|---|---|
| L0 𝔽_p 嵌入 | `fp` 單元測試、全部樣本在 𝔽_p 上求解 | `L0_mod_faithful`、`eval_abs_bound`、`L0_eval_faithful` |
| T1 可靠性 | 7 樣本 σ_D 為根 ✓ | `genC_sound`、`borrow_sat_of_clean`、`clashClause_false_of_live` |
| T2 完備性 | 7 樣本 σ 解碼 ⟺ 推導 ✓ | `genC_complete`、`borrow_sat_iff_clean` |
| T3(a) 對偶 | CDCL 與 GB 判定一致；PHP(4,3) UNSAT | `clause_duality`、`cnf_duality`、`clashClause_duality` |
| T3(b) 學習子句 | 學習子句範式皆 0 ✓（demo C 學習 2 條） | `resolution_identity`、`learned_preserves_models`、`learned_preserves_polyZero` |
| T4 終止性 | 12 樣本 0–645 次擴充，全部 ≤ 2ⁿ | `buchberger_extension_bound`、`no_infinite_sublist_chain` |
| T5 準則 | 280 配對抽檢全歸零；消除率 95% | `coprime_criterion`、`chain_criterion`、`sPoly_chain_decomposition` |
| T6 判定 | 12 樣本「1∈G ⟺ 拒絕」；demo B/C 基 = {1} | `one_mem_no_root`、`borrow_clash_one_mem`、`borrow_assign_one_mem`、`no_root_poly_certificate` |
| T7(a) 規範性 | 策略/順序無關 ⇒ 唯一約化基 | `reduced_unique`（若存在則唯一） |
| T7(b) 宏展開 | P7/P8 選臂可解、錯臂 1∈G | `checkCtx_expand`、`wrong_arm_untypable`、`wrong_arm_no_root` |
| T8 QAP | 7 SAT 樣本 QAP 通過；竄改被拒 | `qap_duality`、`div_linear`、`vanishing_prod_dvd` |
| T9 端到端 | 7 樣本 round-trip ✓ **rustc ✓**；判定 ⟺ 檢查器 | `typable_iff_root`、`untypable_iff_no_root`、`parse_gen`、`t9_borrow_decision` |

---

## 5. 覆蓋邊界（這一節最重要）

**什麼是「實測」、什麼是「證明」，各自到哪裡為止：**

| 主張 | 由誰保證 | 覆蓋範圍 |
|---|---|---|
| 這 12 個樣本的判定/代數/編譯行為 | Rust 實測（本文件 §2） | **僅這 12 個樣本**（＋4 demo） |
| 同型程序的判定等價、終止界、對偶、唯一性 | Lean 定理 | **任意尺寸輸入**（數學一般性） |
| 「生成的 Rust 碼可被 rustc 編譯」 | Rust 實測（7 個 SAT 樣本 `rustc ✓`） | **僅這 7 個樣本**；Lean 只證「可重解析」(`parse_gen`)，**不證** rustc 可編譯 |
| 真實 Rust 的語義保持（(c)） | **兩側都沒有**完整證明 | Rust 側以樣本 AST 對照；Lean 側只到「展開同態 + 型別判定」 |
| `move`／所有權轉移規則 | **Lean 側只有規則層形式化**；Rust 側 Mini-Rust 尚無 `move` 構造 | **未實測**（`use_after_move_unsat`、`move_while_borrowed_unsat` 無對應樣本） |
| 生命週期 `'a`、重借用、分支合流、`Rc`/`RefCell` | **兩側都沒有** | 超出 Mini-Rust 與本模型 |

**「錯了會怎樣」在此的具體含義**：若上表第 2 列（Lean 定理）為假，則第 3 列
（實測）通過的樣本仍會通過，但**沒被測到的輸入**可能出錯；反之若第 1 列（實測）
顯示某樣本與定理預測不符，那代表**形式化與實作的模型不一致**——這種不一致比
單一 bug 更嚴重，因為它意味著「證明的事」不是「程式做的事」。

---

## 6. 本輪新增（相對上一版）

1. `lean/Polyrust/BorrowOwnership.lean`（511 行、42 條定理）：借用存活區間/衝突/
   所有權三規則的形式化，含主定理 `borrow_sat_iff_clean`、借用子句對偶、
   1 ∈ 理想的顯式組合、P5/P6 樣本模型、與 T9 合流的 `t9_borrow_decision`。
2. `scripts/lean-evidence.sh`：一鍵重建 Lean 審計與 Rust 實測證據。
3. `docs/evidence/`：九條義務自證、demo、單元測試的原始輸出存檔。
4. 本文件與 `docs/LEAN.md` §3.5 的邊界說明。

## 7. 窮舉一致性 oracle（2026-09-15 新增）

**方法（借鏡 `rlzl` 的 `@brute` 哲學）**：不抽樣，直接在有界細空間內枚舉**全部**良構 `.poly` 程序，逐一斷言「完整代數管線判定 ⟺ 獨立檢查器推導」；SAT 時進一步逐節點比對代數解碼出的型別與檢查器型別。

```
polyrust exhaust --size N [--cap M] [--json]
```

- 空間：表達式節點數 ≤ N；構造子＝字面量 {0,1,true,false,()}、一元 {-,!}、二元 {+,==}、if/else、let。無宏、無引用 ⇒ 約束系統無子句，判定化約為純布爾求解。
- 良構保證：`let` 只允許出現在序列位置（main 體 / if 分支 / let 體），`render` 對複合表達式全括號化，確保每個枚舉結果都能被 `parse_program` 解析（0 個 `internal_errors`）。
- 對照：`check_program`（ground truth，由型別規則直接推導）vs `gen_constraints` + `solve_boolean`（管線）。任何判定或型別解碼不一致即失敗並印反例。

**結果（≤5 節點，`cargo test --release exhaust`）**：

| 指標 | 值 |
|---|---|
| 良構程序總數 | 5210 |
| SAT / UNSAT | 807 / 4403 |
| 判定不一致（管線 ⟺ 檢查器） | **0** |
| 型別解碼不一致（SAT 逐節點） | **0** |
| 解析失敗（生成器內部錯誤） | **0** |
| 耗時 | ~9 s |

放大一級（`--size 6`）：

| 指標 | 值 |
|---|---|
| 良構程序總數 | 41450 |
| SAT / UNSAT | 3381 / 38069 |
| 判定不一致 / 型別解碼不一致 / 解析失敗 | **0 / 0 / 0** |
| 耗時 | ~128 s |

這是命題 P「判定等價」在細空間上的**窮舉見證**（非抽樣）：5210 個程序無一例外，代數管線與獨立檢查器完全一致。

> 實作備註：生成器初版曾把 `let` 放進一元/二元運算元位置（如 `(-let v0 = 0; 0)`），產生 Mini-Rust 語法非法程序，被解析器擋下而誤報為管線不一致。修正為 `gen_pure`（運算元安全，無頂層 let）+ `gen_seq`（序列位置，可含 let）雙生成器後，全部枚舉結果良構、0 解析失敗——再次印證 `rlzl` 式 oracle 的價值：**窮舉會把生成器自身的 bug 也逼出來**。

## 8. Tier-0 checker 快篩與護欄漏斗量測（2026-09-15 新增）

同源自 `rlzl` 的兩條經驗，落地進 `nl` 護欄層（細節見 `docs/LLM.md` §2/§3b）：

1. **廉價謂詞先行（快篩）**：護欄迴圈在語法閘門後先跑獨立檢查器
   （`check_program`，直接推導）；拒絕即回餵精確錯誤並跳過昂貴的
   CDCL×Buchberger×QAP。等價性依據：§7 窮舉 oracle（41,450 程序，
   檢查器 ⟺ 管線零不一致）+ T6 機械化義務。快篩只加速**拒絕**，
   不加速接受——通過快篩仍必須跑完整管線。
   等價回歸測試 `tier0_equivalence_corpus` 持續鎖定：7 個語料（含宏、
   借用衝突、函式）上「檢查器接受 ⟺ 管線 SAT」逐例斷言。

2. **測量一切（漏斗量測）**：`nl` 運行可按 `--funnel-log` /
   `POLYRUST_FUNNEL_LOG` 落盤 NDJSON；`polyrust funnel` 聚合
   「到達 → 結構 → 語法 → Tier-0 → SAT」各層計數、攔截分佈、
   平均收斂輪次、逐 provider 成功率。離線測試保證聚合與日誌往返一致。
   真 API 跑批數據待累積（免費限流下逐條記錄）。
