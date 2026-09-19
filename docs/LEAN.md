# Lean 4 形式化說明（機械檢查的證明）

本文件說明 `lean/` 目錄的機械化：**證了什麼、邊界在哪、錯了會怎樣**。
數學論證的來源是 [`THEOREMS.md`](THEOREMS.md)；Rust 側的實測在 §12 與
`src/` 的 `obligations` 子命令。兩者是互補的：

| | 覆蓋 | 手段 |
|---|---|---|
| Rust 側（`src/`、`cargo test`） | 具體樣本、每一步工程實現 | 樣本全枚舉、義務自證、`rustc` 實編 |
| Lean 側（`lean/`） | **任意尺寸輸入的一般性定理** | 歸納法、結構遞歸、機器檢查 |

---

## 1. 工具鏈與建置

```bash
# 最新穩定版 Lean + Lake（沿用官方 elan；本倉庫不需要 Mathlib）
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh -s -- -y
export PATH="$HOME/.elan/bin:$PATH"

cd lean
lake build            # 預設目標 Polyrust：34 個模組 + 根模組
bash ../scripts/lean-audit.sh   # 可信度審計：sorry 掃描 + #print axioms
```

實測環境：Lean **4.33.1**（commit `819816b`）、Lake 5.0.0-src、零外部依賴、
`lake build` 從零約 6–11 秒（37 jobs）。`lean-toolchain` 已固定版本，任何人
`git clone` 後 `lake build` 即得同一結果。

**無 `sorry`、無 `admit`、無自訂 `axiom`**：`Audit.lean` 逐定理
`#print axioms`，全部只列出 Lean 標準三公理（`propext`、`Classical.choice`、
`Quot.sound`）或不依賴任何公理。若有 `sorry`，清單會出現 `sorryAx`。

規模：**873 條定理/引理，10,221 行**（不含註解行另有數百行說明）。
`AuditAll.lean` 全庫審計：受檢宣告 3432、純構造 1870、零 `sorry`、零自訂公理。
**1870 條純構造宣告的逐條功用清冊見 [`docs/FORMAL_LEMMAS.md`](FORMAL_LEMMAS.md)**（由 `Enumerate3.lean` 環境掃描自動生成，判據與審計同源，`getModuleIdxFor?` 精確歸屬）。

---

## 2. 模組 ↔ 文檔定理對照（含五類分類）

五類定義（任務要求，含增量迭代）：

- **補全（雙向完備）**：把單向引理補成 ⟺，如 `typable↔root`、`borrow_sat↔clean`、`clauseSat↔polyZero`、`parse/gen` round-trip 雙向。
- **新增（鐵律核心）**：不可違反的核心命題，如 one-hot 排他、field poly `x²−x=0`、borrow 衝突互斥、lifetime 無環、unsafe 邊界、watch 移動保語義。
- **衍生（由核心推出）**：由鐵律直接推出的結論，如 pair/sum 可定型推論、pairBitSum 乘積、isMonoAt 區域化、borrow 1∈理想、P5/P6 差異。
- **次要（支撐性）**：底層支撐引理，如位元算術、MonoExp 整除、supportLe、List 求和。
- **增量迭代（第5類，迭代收斂）**：覆蓋解析/生成/型別檢查/借用/F4/F5/LoopContract 的迭代單調與收斂，如 `parseFuel_mono_succ`、`gen_length_iter`、`check_add_iff`、`borrowSystem_append_pairs_eq`、`f4_ideal_invariant_iter`、`f5_sig_iter_trans`、`fuel_iter_mono` 等，全部純構造、零 sorry、零 axiom。

| Lean 模組 | 行數 | 總宣告 | 純構造 | 四類 | 核心內容 |
|---|---|---:|---:|---|---|
| `Polyrust/Monomial.lean` | 246 | 69 | 19 | 次要 | 指數向量、加/減/ℓcm/整除、支撐、首項單項式；純組合、零域公理、可 `decide` |
| `Polyrust/Tactics.lean` | 24 | 2 | 2 | 次要 | `int_ring`：無 Mathlib 的整數多項式歸一化宏 |
| `Polyrust/ClauseDuality.lean` | 125 | 35 | 32 | 鐵律 | `clause_duality`：σ ⊨ C ⟺ P_C(σ)=0；`field_poly_bit`；`cnf_duality` |
| `Polyrust/ClauseAlgebra.lean` | 260 | 36 | 13 | 鐵律 | `resolution_identity`；`learned_preserves_models`；`unsat_iff_no_polyZero` |
| `Polyrust/WatchMove.lean` | 172 | 23 | 7 | 鐵律 | `watch_move0/1_preserves_sat`：監視文字**交換**移動保持子句語義；`watchMoves_preserve_sat`：任意步數語義如初；`watch_overwrite_unsound` 反例 |
| `Polyrust/UniPoly.lean` | 256 | 59 | 26 | 鐵律 | `eval_add/mul/sub`；`div_linear`；`vanishing_prod_dvd`；`qap_duality` |
| `Polyrust/Squarefree.lean` | 372 | 50 | 17 | 鐵律 | `standard_implies_squarefree`；`toBits/ofBits` 雙射；`buchberger_extension_bound` |
| `Polyrust/Embedding.lean` | 140 | 13 | 2 | 鐵律 | `L0_mod_faithful`；`eval_abs_bound`；`L0_eval_faithful` |
| `Polyrust/MicroInstance.lean` | 104 | 16 | 13 | 衍生 | 加法規則、上下文矛盾、宏選臂三系統的全枚舉 |
| `Polyrust/SPoly.lean` | 356 | 47 | 8 | 鐵律 | `sPoly_mem_genIdeal`、`coprime_criterion`、`chain_criterion` |
| `Polyrust/Canonical.lean` | 219 | 29 | 7 | 鐵律 | `reduced_unique`：簡化 Gröbner 基若存在則唯一 |
| `Polyrust/T6Certificate.lean` | 429 | 67 | 17 | 鐵律 | `inIdeal_no_root`、`one_mem_no_root`；`interpolation`；`no_root_certificate` |
| `Polyrust/BoolNullstellensatz.lean` | 332 | 26 | 2 | 鐵律 | `bool_nullstellensatz`：每點可逆 ⟹ 組合成 1（模 p）；`mod_p_one_in_ideal` |
| `Polyrust/T9EndToEnd.lean` | 764 | 306 | 213 | 鐵律+補全 | `check`、`genC`、`genC_sound`（T1）、`genC_complete`（T2）、`typable_iff_root`（T6）、`parse_gen` round-trip |
| `Polyrust/BorrowOwnership.lean` | 511 | 132 | 90 | 鐵律+補全 | `borrow_sat_iff_clean`：有根 ⟺ 無衝突；`borrow_clash_one_mem`（1∈理想）；P5/P6 模型 |
| `Polyrust/T9Generalized.lean` | 786 | 175 | 83 | 鐵律+補全 | `Lang` 宇宙參數化；`tycheck_exclusive`、`typable_iff_rootG` 對任意宇宙成立 |
| `Polyrust/ProductReduction.lean` | 208 | 115 | 97 | 衍生 | `typable_pair_iff`（AND）、`pairBitSum_eq_mul` |
| `Polyrust/SumReduction.lean` | 166 | 121 | 106 | 衍生 | `type_typable_sum_iff`（OR）、`sumBits_sum_eq_add` |
| `Polyrust/OpAbstraction.lean` | 543 | 163 | 107 | 衍生 | `BinSpec` 抽象；任意規格的 T1/T2/T9 |
| `Polyrust/MacroExpansion.lean` | 440 | 182 | 109 | 衍生 | `subst_id`、`expand_comp` 同態；`wrong_arm_no_root` |
| `Polyrust/TypeUniverse7PlusI.lean` | 344 | 192 | 147 | 鐵律 | 7 基底 +10 擴展（17 種）；one-hot 分解；`one_hot_unique_17` |
| `Polyrust/ModuleFlatten.lean` | 94 | 71 | 60 | 次要 | 模組樹展平為線性約束 |
| `Polyrust/MatchDecisionTree.lean` | 95 | 128 | 93 | 衍生 | `Pat`/`DecisionTree`；深度 ≤ 臂數；`decisionTreeToIfChain` |
| `Polyrust/ProductSum.lean` | 115 | 75 | 50 | 衍生 | 積型與和型的混合歸約；`ProductConstraint`/`SumConstraint` |
| `Polyrust/LifetimeRegion.lean` | 113 | 111 | 93 | 鐵律 | `'static` 出超所有、無自環、NLL 重疊；`static_outlives_all`、`outlives_refl` |
| `Polyrust/UnsafeContext.lean` | 81 | 83 | 66 | 鐵律 | 裸指針解析、unsafe 閘控、Pure/No-IO；`parseRawPtr` |
| `Polyrust/AsyncStateMachine.lean` | 83 | 85 | 65 | 鐵律 | Future 狀態機、輪詢約束、one-hot 狀態；`state_number` 無 sorry 證明 |
| `Polyrust/StdlibEncoding.lean` | 68 | 81 | 66 | 鐵律 | Vec/String/HashMap 多項式編碼 |
| `Polyrust/LoopContract.lean` | 54 | 38 | 21 | 鐵律 | 循環不變量、歸納契約、fuel 有界展開 |
| `Polyrust/TraitImpl.lean` | 77 | 116 | 100 | 衍生 | trait/impl 解析、方法表、存在量化 |
| `Polyrust/Minor.lean` | 239 | 54 | 17 | 次要 | 位元算術 `minor_bit_*`、Lit 支撐、MonoExp 整除、supportLe、List 求和支撐、field poly 支撐 |
| `Polyrust/IronLaw.lean` | 254 | 48 | 13 | 鐵律核心 | one-hot 排他、field poly `x²−x=0`、borrow 衝突互斥、lifetime 無環、unsafe 邊界、watch 保語義、子句對偶 |
| `Polyrust/Derived.lean` | 236 | 41 | 5 | 衍生 | pair/sum 可定型推論、pairBitSum 乘積、isMonoAt 區域化、borrow 1∈理想、watch 多步守恆、P5/P6 差異 |
| `Polyrust/Completion.lean` | 299 | 44 | 3 | 雙向完備 | clauseSat↔polyZero 雙向、false↔1、borrow_sat↔clean、typable↔root、watch iff、parse/gen 雙向、one-hot 唯一性雙向、field poly 雙向 |
| `Polyrust/IncrementalIteration.lean` | 636 | 85 | 85 | 增量迭代 | parseFuel 單調/穩定/收斂、gen 迭代、sizeT 單調、check 分解、Typable/IsRoot 單調、borrowSystem 單調、F4/F5 迭代收斂、fuel 迭代、端到端迭代 |
| `Polyrust/Composition.lean` | 290 | 23 | 20 | **T10 組合性** | `standard_iff_factor`；`buchberger_extension_bound_product(_pow)`；`composed_extensions_bound`：Σ2^{nᵢ} ≤ 2^{Σnᵢ}，組件常數大小 ⇒ 總界線性於規模 |
| `Polyrust/BoundedStandard.lean` | 216 | 16 | 10 | **∏kᵢ 一般化** | `standard_implies_boundedF`；`allBounded_length` = ∏kᵢ；`buchberger_extension_bound_general`；`general_bound_specializes_to_2n`（T4 即 kᵢ≡2 特例） |
| `Audit.lean` | 160 | — | — | 工具 | 主定理 `#print axioms` |
| `AuditAll.lean` | 75 | 3984 受檢 | 920 純構造定理 | 工具 | 全庫掃描，零 sorry、零自訂公理，`AUDIT_RESULT=CLEAN`（2026-09-20 重跑） |

---

## 3. 九定理覆蓋表（含邊界）

「邊界」一欄說明：**Lean 內證到哪裡為止、剩下什麼依賴 Rust 側或未被形式化**。

| 定理 | Lean 內證明的內容 | 邊界（不在 Lean 內） |
|---|---|---|
| **L0** | 小係數多項式在 0/1 點求值：模 p 為零 ⟺ 整數為零；求值界 ≤ 2²⁸ | 「小係數」的上界是假設；p = 2⁶¹−1 為素數用事實性引理 `P61_eq` |
| **T1** | `genC_sound`：檢查器接受 ⟹ 見證賦值 σ_D 是約束系統的 0/1 根；**借用側** `borrow_sat_of_clean` | σ_D 的「未覆蓋節點補全」在 Rust 側；借用「活躍」由 `b − 1 = 0` 方程表示 |
| **T2** | `genC_complete`：任一 0/1 根逐型別解碼為檢查器答案；**借用側** `borrow_sat_iff_clean` | 同上 |
| **T3(a)** | `clause_duality`、`cnf_duality`、`field_poly_bit` | 係數域的作用由 `bit` 的 0/1 編碼承擔；𝔽_p 版本經 L0 嵌入 |
| **T3(b)** | `resolution_identity`；`learned_preserves_models`；**WatchMove**：`watch_move0/1_preserves_sat`、`watchMoves_preserve_sat` | 完整的 P_C ∈ ⟨P_Φ ∪ B⟩ 需要 Nullstellensatz；傳播演算法終止性未形式化 |
| **T4** | `buchberger_extension_bound`：含域多項式時基擴充 ≤ 2ⁿ | 抽象化為 Sublist 鏈 |
| **T5** | `coprime_criterion`、`chain_criterion` | 準則「不改變約化基」的完整陳述依賴 T7(a) |
| **T6** | `one_mem_no_root`；`no_root_certificate`；**布爾 Nullstellensatz**；**借用側** `borrow_one_mem_no_root`＋`borrow_clash_one_mem` | 函數層證書；語法層恆等式未做 |
| **T7(a)** | `reduced_unique`：固定項序下簡化 Gröbner 基若存在則唯一 | 存在性不在本倉庫內 |
| **T7(b)** | `checkCtx_expand`、`wrong_arm_no_root`、`arm_gating*` | 錯臂系統 1∈G 的代數見證在微實例中 |
| **T8** | `qap_duality` | deg 層面以列表次數陳述 |
| **T9** | `typable_iff_root`、`untypable_iff_no_root`、`parse_gen`、`gen_length` | (b) rustc 可編譯與 (c) 語義保持不在 Lean 內，由 Rust 側實測 |

### 五類新增覆蓋（本次補全，含增量迭代）

| 五類 | Lean 模組 | 代表定理 | 說明 |
|---|---|---|---|
| **補全** | `Completion` | `clauseSat_false_iff_polyOne`、`borrow_sat_iff_clean_completion`、`typable_iff_root_completion`、`watch_move_iff`、`parse_gen_sound`/`parse_gen_complete_left_inverse`、`oneHot_iff_exists_unique`、`field_poly_iff_bool` | 把以往單向引理補成 ⟺，實現判定等價的雙向完備；`clauseSat=false ↔ poly=1` 補上空子句與全假子句的對偶；`oneHot` 雙向把「和=1」與「存在唯一真」釘在一起 |
| **新增** | `IronLaw` | `iron_one_hot_unique`、`iron_field_poly_bit`、`iron_borrow_clash_unsat`、`iron_overlaps_self_of_nonempty`、`iron_static_outlives_all`、`iron_unsafe_gate_fails_when_unsafe_not_allowed`、`iron_watch_move0_preserves`、`iron_clause_duality`、`iron_base_ne_ext` | 不可違反的鐵律：型別唯一、位元 0/1、借用互斥、區間自重疊、static 出超所有、unsafe 閘控、watch 保語義、子句對偶、基底≠擴展 |
| **衍生** | `Derived` | `derived_typable_pair_iff`、`derived_type_typable_sum_iff`、`derived_pairBitSum_eq_mul`、`derived_isMonoAt_of_root`、`derived_borrow_clash_one_mem`、`derived_watchMoves_preserve_sat`、`derived_p5_p6_differ` | 由鐵律直接推出的結論：積型 AND、和型 OR、one-hot 乘積/相加、區域化單型、1∈理想顯式見證、多步守恆、P5/P6 僅差區間 |
| **次要** | `Minor` | `minor_bit_mul_self`、`minor_bit_eq_zero_or_one`、`minor_litFactor_zero_or_one`、`minor_dividesM_trans`、`minor_supportLe_nth_false`、`minor_listSum_nonneg`、`minor_field_poly_bit` | 底層支撐：位元算術、Lit、MonoExp、supportLe、List 求和、field poly；全部純構造、零公理、供鐵律與完備性引用 |
| **增量迭代** | `IncrementalIteration` | `parseFuel_mono_succ`/`parseFuel_mono`、`gen_length_iter`/`sizeT_pos`、`check_add_iff`、`typable_add_imp_left`/`isRoot_add_imp_left`、`borrowSystem_append_pairs_eq`/`borrow_sat_mono_pairs`/`borrow_unsat_mono_pairs`、`f4_ideal_invariant_iter`/`f4_ideal_invariant_iter3`、`f5_sig_iter_trans`/`f5_criterion_mono`/`f4f5_iter_preserves`、`fuel_iter_mono`/`fuel_iter_default_le`/`incremental_iteration_complete` | 迭代收斂：第5類覆蓋解析/生成/型別檢查/借用/F4/F5/LoopContract 的單調與迭代穩定，fuel 增加保持 some、gen 追加結合、check 分解、Typable/IsRoot 子表達式單調、borrowSystem 子集單調與 UNSAT 單調、F4 理想不變迭代、F5 簽名傳遞閉包與準則單調、F4F5 迭代收斂、端到端 t9+borrow 迭代 |

### 「錯了會怎樣」：若這些定理為假

* **T1/T2 為假** → 管線的 SAT/UNSAT 判定與真實型別系統脫鉤。
* **T3(a)/T3(b) 為假** → CDCL 學習的子句不在理想內 ⇒ 解集被改變。
* **監視移動不健全** → SAT 誤報 UNSAT（v0.1.4 實例）。
* **T4 為假** → Buchberger 不終止。
* **T5 為假** → Gröbner 基判定失去完備性。
* **T6 為假** → 「1 ∈ G」不再是不可定型判據。
* **T7(a) 為假** → 約化基不唯一。
* **T7(b) 為假** → 宏展開改變可解性。
* **T8 為假** → QAP 不忠實於 R1CS。
* **T9 為假** → 生成碼無法重解析或語義不同。
* **四類補全為假** → 雙向完備斷裂：可能出現「可定型但無根」或「有根但不可定型」的反例，鐵律被違反則系統出現重疊借用、unsafe 逃逸、lifetime 環等不安全行為。

---

## 3.5 借用與所有權（T9 的借用擴充）

`Polyrust/BorrowOwnership.lean` 把 Rust 側 `analysis.rs`／`constraints.rs` 的借用層
逐式搬進 Lean，並證三件「說死」的事：

**(1) 判定等價（借用側的 T6）**

```
borrow_sat_iff_clean : (∃ β, ∀ c ∈ borrowSystem live pairs assigns, c β = 0) ↔ pairs = [] ∧ assigns = []
```

**(2) 子句 ⟷ 多項式（接 T3(a)）**

```
clashClause_duality  : clauseSat β [¬b_i, ¬b_j] = true ↔ b_i · b_j = 0
```

**(3) 1 ∈ 理想（借用側的 T6 判定，含顯式見證）**

```
borrow_clash_one_mem  : 1 ∈ ⟨x_i x_j, x_i − 1, x_j − 1⟩   -- x_i x_j − x_j(x_i−1) − (x_j−1) = 1
```

---

## 4. 衍生與次要引理

**四類分類**已在 §2 表格中以「四類」欄標註。本倉庫的做法是：**每個模組都把主定理所需的輔助結構一併入庫**，包含

* `Monomial`：單項式代數全套——T4/T5/T7 的共同基礎；
* `ClauseAlgebra`：`clausePoly_append`、`Lit.neg`、`litSat_neg` 等；
* `Squarefree`：位串雙射、`allBits_sound/complete`；
* `SPoly`：`mulMono` 代數律；
* `T9EndToEnd`：位元算術、單型性、約束表成員分解、`Occurs`、`isMonoAt_of_root`、`gen_length`；
* `BorrowOwnership`：`overlaps_comm`、`conflictsWith_comm`、`BgenIdeal_*` 等；
* **Minor**：`minor_bit_*`、`minor_litFactor_*`、`minor_dividesM_*`、`minor_supportLe_*`、`minor_listSum_*`、`minor_field_poly_*` 等支撐性引理；
* **IronLaw**：`iron_*` 鐵律核心；
* **Derived**：`derived_*` 由核心推出的衍生；
* **Completion**：`*_completion`、`*_iff` 雙向完備。

也就是：**凡是主定理證明中出現的結構，都以具名引理固化**，而不是塞在單一大證明的隱形步驟裡。

---

## 5. 三類「必須說死」的邊界

1. **邏輯邊界**：`propext`/`Classical.choice`/`Quot.sound` 是 Lean 標準三公理；
   `SPoly`/`Canonical`/`T6Certificate` 因整除/理想的判定性用了 `Classical.choice`；
   `Monomial`、微實例、T9 的核心引理完全不依賴任何公理（審計輸出可見）。
2. **語義邊界**：Lean 的 `Expr` 是 Mini-Rust 的核心子集。借用/所有權的完整語義、生命週期、`&mut` 別名規則不在 Lean 內，而在 Rust 側以子句 + 樣本全枚舉覆蓋。
3. **計算邊界**：Lean 證的是數學正確性，不是 Rust 實作的性能或記憶體行為。

---

## 6. 與 Rust 側的對接點

| Lean 定理 | Rust 側對應的實測 | 位置 |
|---|---|---|
| `typable_iff_root`、`untypable_iff_no_root` | 12 個樣本「管線 SAT/UNSAT ⟺ 檢查器接受/拒絕」 | `core/src` / `cargo test` |
| `buchberger_extension_bound` | 樣本擴充次數 0–645 ≤ 2ⁿ | `groebner` 統計 |
| `coprime_criterion`/`chain_criterion` | 消除率 95% | `groebner` 統計 |
| `reduced_unique` | 反序生成元 + FIFO 與原基逐元素相同 | T7(a) 義務 |
| `wrong_arm_*` | demo P7/P8 錯臂系統約化基 = {1} | demo |
| `borrow_sat_iff_clean`、`p5_unsat`／`p6_sat` | P5-twice-mut UNSAT、P6-temp-borrow SAT | obligations |
| `qap_duality` | 7 個 SAT 樣本 QAP 驗證通過 | `qap` |
| `parse_gen` | 生成 12 個樣本代碼全部 `rustc` 編譯成功 | `codegen` |
| `oneHot_iff_exists_unique` | one-hot 排他鐵律的雙向完備 | Completion |
| `iron_borrow_clash_unsat` | 借用衝突互斥鐵律 | IronLaw |
| `derived_p5_p6_differ` | P5/P6 僅差區間的衍生 | Derived |
| `minor_bit_eq_zero_or_one` | 位元 0/1 支撐 | Minor |
