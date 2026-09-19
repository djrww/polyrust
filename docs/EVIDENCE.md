# 實測證據：12 樣本 × 九條義務 × Lean 定理

本文件把 **Rust 側的實測輸出** 與 **Lean 側的機械證明** 對齊。它是
[`docs/LEAN.md`](LEAN.md) 的證據附錄，也是 `docs/THEOREMS.md` §12 的展開。

**一句話結論**：Rust 側 9/9 義務自證、單元測試全數通過（v0.1.0 時為 17/17；v0.2.2 已達 63/63）、4/4 demo 全數通過
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
| **T10** 組合性＋∏kᵢ＋認證路徑 | PASS ✓ | 新增 36 測試全綠（組合 6／認證 7／消失 9／Chalk 橋 6／MIR 前端 8）；**義務 T10**：12 樣本分解求解並基全驗證、UNSAT 判定與整體一致 |

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
| T10 組合性/∏kᵢ | 分解→逐組件→並基驗證（UNSAT 局部化）；σ 重建+直接求值認證；L0′ 值域界；管線 `PL_DECOMPOSE`；Chalk 橋 `polyrust-oracle/1` 閉環；MIR 消失多項式 lowering | `composed_extensions_bound`、`standard_iff_factor`、`buchberger_extension_bound_general`、`general_bound_specializes_to_2n`；Rust：`obligation_t10`、`chalk_bridge`、`mir_lower` |

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

> 上表為**第一版空間**（表達式片段：無引用、無賦值 ⇒ 約束系統無子句）。

### 7b. 空間擴展（2026-09-15 審計後）

全代碼審計發現 `AssignDeref`（`*r = v`）三方約定錯配（見 §9）——該 bug
之所以潛伏，正因為第一版空間不含引用/賦值。空間隨即擴展：

- 新構造子：`*e`（解引用）、`&x` / `&mut x`、`x = e`、`*e = e`、語句序列。
- 約束系統因此帶**借用互斥子句**——oracle 現在也覆蓋子句側
  （子句 → 多項式編碼與管線 S6 完全同一 `clause_to_poly`）。
- 仍無宏（宏側由 `obligations` 12 樣本與 `tier0_equivalence_corpus` 覆蓋）。

| 空間 | 良構程序 | SAT / UNSAT | 不一致 | 耗時 |
|---|---|---|---|---|
| ≤5 節點（含引用/賦值/子句） | 11265 | 1895 / 9370 | **0** | ~23 s |
| ≤6 節點（同上） | 102125 | 7737 / 94388 | **0** | ~324 s |

### 7c. V2 全空間：宏 × fn × main（2026-09-15 窮舉收官）

表達式空間不含宏與函式——臂位元、臂互斥子句、exists-arm 語義、fn 調用
仍是盲區。**V2 空間**把它們全部納入窮舉：

- **宏**：0..1 個、1..2 臂；匹配器 ∈ {`($e:expr)`, `($e:expr, $f:expr)`,
  `($v:ident)`}；臂模板 = 帶洞表達式（含 `&mut $v` 對以覆蓋**宏×借用交互**）。
- **fn**：0..2 個；參數 0..2 個（i32/bool）× 回傳（i32/bool/unit）21 種簽名。
- **調用邊界**：`Call`/`Invoke` 實參個數可與形參/匹配器**不符**（覆蓋
  未定義宏、無匹配臂、轉錄失敗臂、實參個數錯配等全部邊界路徑）。
- 代價度量：宏 = 1 + 臂數 + Σ模板節點；fn = 1 + 體節點；總預算 = 全組件和。

**V2 空間首轮即抓到兩個潛伏不一致**（修復後歸零，見 §9 #8/#9）——
再次驗證窮舉的價值：

| 空間 | 程序數 | SAT / UNSAT | 不一致 | 耗時 |
|---|---|---|---|---|
| ≤4 總節點（全組合） | 11573 | 885 / 10688 | **0** | ~8 s |
| ≤5 總節點（全組合，無截斷） | **582427** | 26717 / 555710 | **0** | ~682 s |

回歸測試：`oracle_full_space_consistency`（≤4 全組合、零不一致、雙向覆蓋、
不觸頂）＋ `oracle_full_space_covers_macro_mechanisms`（鎖定宏／fn 生成非空）。

## 9. 全代碼審計記錄（2026-09-15）

逐檔審計（約 9,700 行）後發現並修復的問題：

| # | 類別 | 問題 | 修復 |
|---|---|---|---|
| 1 | **真 bug（潛伏）** | `AssignDeref` 三方錯配：parser 已剝去 `*`（AST 約定 lhs＝引用表達式），但 checker／constraints 仍期望 `lhs.kind == Deref(..)` 永遠錯配 ⇒ `*r = v` 必被檢查器拒絕；約束側漏掉 tie 約束（潛在不完備）；codegen 生成 `r = v`（非法 Rust） | 三方統一按 AST 約定；約束補「lhs 必須 &mut」強制；測試鎖定（`test_assign_deref_*`、tier0 語料 4 例、窮舉空間含賦值） |
| 2 | 解析寬鬆 | `f(1 2)`、`fn f(a: i32 b: i32)` 被靜默接受 | 參數/實參間強制逗號（尾隨逗號仍合法）；測試鎖定 |
| 3 | 輸入路徑 panic | CDCL(T) 迴圈 `assert!(rounds < 200)` 可被大程序觸發 | 改為回傳 `Err` |
| 4 | 殘留除錯碼 | CDCL 內 `eprintln!("BUG: …")` | 改 `debug_assert!` |
| 5 | 整數溢出 | 詞法器整數字面量 `n*10+d` 於超長字面量靜默溢出 | `checked_mul/checked_add` ⇒ 報錯 |
| 6 | 資源 | HTTP server 連線無讀取超時（慢速連線佔執行緒） | 60 s 讀取超時 |
| 7 | **覆蓋缺口** | 窮舉空間不含引用/賦值 ⇒ 子句側與 `*r = v` 無窮舉覆蓋（#1 因而潛伏） | 空間擴展（§7b）；≤5 節點 11,265 程序零不一致 |
| 8 | **真 bug（V2 窮舉抓到）** | `fn` 體根節點型別**未綁定到聲明回傳型別** ⇒ `fn f() -> i32 { true }` 代數側誤判 SAT、檢查器拒絕（判定不一致） | 約束補 `t_{body,ret} − 1 = 0`；V2 空間 ≤4 全組合重跑歸零 |
| 9 | **真 bug（V2 窮舉抓到）** | 宏邊界三方錯配：(a) 未定義宏調用、(b) 無臂匹配 ⇒ 約束側只有 one-hot 無矛盾（誤判 SAT）；(c) 臂轉錄失敗時約束側 `?` 中斷而檢查器 `continue`（不對稱） | 三者統一：無可用臂 ⇒ 全部型別位元 = 0 破產 one-hot；轉錄失敗臂兩側對稱跳過；`tier0` 語料鎖定 |

記錄在案、暫不修（誠實標明）：`check_expr` 推導數 64 上限截斷（演示規模，
已註明）；`solve_boolean` 50 萬步、`div_rem` 200 萬步內部保護（超限即報錯）；
i32 運算溢出語義不建模（Mini-Rust 簡化）；server 無連線數上限（極簡定位，
已補超時）；`strip_main` 的大括號計數不感知註解中的括號（邊緣）。

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

## 10. @brute 對照常態化（2026-09-15 新增）

仿 `rlzl` 的 `@brute` oracle 模式：把「暴力法 ⟺ 代數法」逐位元比對
做成**每次 `cargo test` 都跑的駐場測試**（`src/brute.rs`、
`polyrust brute [--size N] [--json]`）。暴力側不共用任何求解器代碼
（直接 `Poly::eval_full` 逐多項式求值），獨立性成立。

### 10a. 三層對照（--size 3 常駐；--size 4 約 60s）

| 層 | 內容 | 結果 |
|---|---|---|
| 子句層 | 367 組（隨機 300＋結構化 exactly-one/鳩巢 7＋**純 3-SAT 硬樣本 60**，比例 3.0、變量 9..=12）：CDCL ⟺ `brute_force_sat` 全枚舉；SAT 模型逐條真滿足 | 零不一致 |
| 學習子句蘊涵 | 每個 ≤12 變量 SAT 實例：暴力遍歷**全部** 2^nv 滿足賦值，逐條驗證 `learned_clauses()` 被蘊含 | **1,361 次驗證，零反例** |
| 約束層 | 186 程序（182 在結構化暴力搜索規模內）：7^節點型別 × 2^布爾位直接枚舉 ⟺ Gröbner ⟺ `solve_boolean` ⟺ checker 四方一致；one-hot 解碼健全 | 零不一致 |
| 見證逐位元 | 任何規模：`verify_witness_bitwise` 對求解器輸出逐多項式歸零；竄改任一位元必被抓（`brute_witness_bitwise_rejects_tamper`） | 通過 |

### 10b. 本輪抓到並修復的真 bug：CDCL 雙監視文字「覆寫而非交換」

常態化對照**立即產生回報**：純 3-SAT 硬樣本（寬恆 3，迫使真正的
衝突驅動學習）下，CDCL 對可解實例誤報 UNSAT。除錯過程：

1. 以 2^10 全賦值驗證每條學習子句 ⇒ 定位到一條不被原式蘊含的
   學習子句 `[x2 ∨ x0]`（存在模型使兩者皆假）。
2. 加 `debug_assert` 不變量 ⇒ 發現子句表中出現**重複文字**
   `[19, 16, 19]`——子句內容被破壞。
3. 根因：`propagate` 移動監視文字時寫成 `c2[0] = l`（**覆寫**），
   被否證的文字直接丟失、`l` 又在原位 ≥2 留存，子句 (A∨B∨C) 變成
   (B∨C)∪{重複}，語義變強；回溯後該「強化子句」仍生效 ⇒
   不可靠剪枝 ⇒ SAT 誤報 UNSAT。MiniSat 原版是**交換**（被否證
   文字換到 `l` 的原位，子句多重集不變）。

修復：`src/cdcl.rs` `propagate` 改為交換（`c2[k] = falsified`），
並新增三個永久 `debug_assert` 不變量（衝突子句全假／原因子句
蘊含真＋其餘全假／學習子句尾文字回跳後全假）。回歸鎖定：
`cdcl_watch_move_regression_3sat`（oracle 判定＋學習子句全賦值
蘊涵驗證）。

教訓（同 `rlzl`）：**混合窄寬度的易實例測不出學習路徑的缺陷**；
常駐對照必須含「強迫觸發目標代碼路徑」的硬樣本。

### 10c. 回歸基線

- `cargo test --release`：**67/67 綠**（63 舊＋3 個 @brute 駐場＋1 回歸鎖定），~32s。
- `cargo test`（debug，`debug_assert` 全開）：**67/67 綠**，~231s。
- `polyrust brute --size 4`：1,286 程序、1,282 暴力搜索、254 見證
  逐位元驗證、四方一致、零不一致。

## 11. Lean 形式化更新與靜態嵌入落地（2026-09-15 新增）

**溫故知新**：查閱 Lean 4.27–4.33 發布說明（4.33.0 = 2026-08-10）後，
確認釘住的 `v4.33.1` 仍是最新穩定分支；新版重點（`grind`/`lia` 改進、
`backward.isDefEq.respectTransparency.types` 預設開啟、do elaborator
换代）對本庫無破壞（從零重建零警告）。

### 11a. 新模組 `Polyrust.WatchMove`（134 行、8 條定理）

把 §10b 抓到並修復的 CDCL 缺陷的**數學核心**機械化（接續 `ClauseAlgebra`
的推理規則層，補上資料結構層）：

| 定理 | 內容 | 公理依賴 |
|---|---|---|
| `clauseSat_iff_exists` | 子句滿足 ⟺ 存在真文字（成员刻畫） | 標準三公理 |
| `clauseSat_congr_mem` | 成員集相同 ⇒ 滿足性相同 | 標準三公理 |
| `watch_move0/1_preserves_sat` | **監視文字交換移動保持子句語義**（任意賦值）——`propagate` 的交換不變量 | 標準三公理 |
| `watch_move_sound` | 移動前滿足 ⇒ 移動後滿足（工程不變量形式） | 標準三公理 |
| `watch_move0_example_sound` | 具體實例對任意賦值成立 | 標準三公理 |
| `clauseSat_all_false` | 全假子句不滿足（衝突偵測健全性） | 標準三公理 |
| `watch_overwrite_unsound` | **覆寫版顯式反例**（σ：x₀ 真）——缺陷紀錄，零公理純計算 | **無** |

證明用 Lean 4.33 核心 `grind`（零 Mathlib 依賴不變）。`Audit.lean` 手列
清單新增 6 條（並移除一條既有重複項，現 90 條）。

### 11b. 靜態嵌入在案債務清結

安裝 elan 4.2.4 + `leanprover/lean4:v4.33.1` 後：

- `lake build`：22 jobs、從零 ~9s、零警告、零棄用。
- `lake build Polyrust:static`：42 jobs，產 `libpolyrust_x2dformal_Polyrust.a`（1.37 MB）。
- `cargo build --release`：`build.rs` 成功連結（此前因無工具鏈而「略過」），
  二進位 3.8 MB，`ldd` 僅含 glibc 系（libgcc_s/libm/libc）——Lean 執行時、
  gmp、uv、ssl、libc++ 全部靜態嵌入。
- 啟動自檢：`〔Lean 4 形式化庫（Polyrust.*，19 模組）已靜態嵌入並載入 ✓〕`。

### 11d. 迭代不變量補完（「必然如初」的機械化）

單步交換定理之上，`WatchMove`／`WatchMoves` 歸納關係＋
`watchMoves_preserve_sat`：子句經**任意有限步**合法監視移動後，
任何賦值下滿足性與原式相同——「不需要恢復原狀，因為語義每一步
都未變」這個工程直覺現為機器檢查的定理（`watchMoves_sound` 為其
工程不變量形式）。

### 11c. 審計基線（更新後）

- 385 條定理/引理、6,045 行、19 模組。
- `AuditAll`：受檢宣告 **1736**、純構造 **961**、零 `sorry`、零自訂公理
  ⇒ `AUDIT_RESULT=CLEAN`。
- `scripts/lean-audit.sh` 全程通過。

## 12. 布爾 Nullstellensatz 形式化與兩處過度聲稱的補完（2026-09-15 新增）

針對「衍生引理到底證什麼、邊界在哪、錯了會怎樣」的審查，新增模組
`Polyrust.BoolNullstellensatz`（332 行）並補完兩處**檔首過度聲稱**：

### 12a. 補完的兩處過度聲稱（此前只有註解承諾、沒有證明）

| 缺口 | 位置 | 補完 |
|---|---|---|
| `mod_p_one_in_ideal`（逆元縮放為 1）僅在檔首第 4 條被提及，**全庫無此定理** | `T6Certificate.lean` | 實現於 `BoolNullstellensatz.mod_p_one_in_ideal`；檔首改為如實指向 |
| `allBits` 的 `Nodup` 被註解聲稱為「純組合事實」，**從未被證明**——而 `sum_delta`／`interpolation` 以它為前提 | `Squarefree.lean` | `allBits_nodup`（含 `setN_inj`、`nodup_flatMap_of` 等組合引理） |

### 12b. 布爾 Nullstellensatz（說死版）

* **證什麼**：`bool_nullstellensatz`——系統 `fs`（{0,1}ⁿ 上多項式函數層）
  若每點都有某元素模 `p` **可逆**（顯式逆元見證 `p ∣ f(x)·iv − 1`），
  則存在**多項式函數乘子**使 `Σ_f mult_f·f ≡ 1 (mod p)` 在整個立方上
  逐點成立。構造顯式：拉格朗日基 `δ_a` 按見證歸屬分配（`sum_fiber`
  纖維重排）＋布爾插值提升多項式性。反向 `bool_ns_no_root_of_certificate`
  （前提 `p ∤ 1`）：常數 1 憑證 ⟹ 每點有模 `p` 非零元素。
* **邊界**：①函數層（逐點同餘），非語法層 `Σ gᵢfᵢ + Σ hⱼ(xⱼ²−xⱼ) = 1`
  恆等式（需多元除法；管線不需要）；②「可逆」帶顯式見證，不假設 `p`
  素性——素數下「可逆 ⟺ 非零」的 Bézout 等價由 L0／Rust 側提供；
  ③可靠方向需 `p ∤ 1`。
* **錯了會怎樣**：若 `bool_nullstellensatz` 為假，「無公共零點 ⟹
  1 ∈ 理想」的代數判據斷裂——Gröbner 側算出 1 與語義側無根不再互推，
  T6「1 ∈ G ⟺ 不可定型」失去一般性論證骨幹（只剩樣本檢查）。
* **與既有 `no_root_certificate` 的關係**：舊定理止於「組合 = 處處非零
  函數 D」（ℤ 層無除法，這是不能到 1 的數學邊界，非偷懶）；
  新定理在模 `p` 層用逆元見證跨過這一步。

### 12c. 審計基線（更新後）

* 408 條定理/引理、6,502 行、20 模組；`Audit.lean` 手列 98 條。
* `AuditAll`：受檢宣告 **1787**、純構造 **970**、零 `sorry`、零自訂公理
  ⇒ `AUDIT_RESULT=CLEAN`。新主定理僅依賴標準三公理
  （`Classical.choice` 來自見證選取，屬可追蹤的標準公理）。
