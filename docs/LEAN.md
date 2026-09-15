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
lake build            # 預設目標 Polyrust：20 個模組 + 根模組
bash ../scripts/lean-audit.sh   # 可信度審計：sorry 掃描 + #print axioms
```

實測環境：Lean **4.33.1**（commit `819816b`）、Lake 5.0.0-src、零外部依賴、
`lake build` 從零約 6–11 秒（22 jobs）。`lean-toolchain` 已固定版本，任何人
`git clone` 後 `lake build` 即得同一結果。

**無 `sorry`、無 `admit`、無自訂 `axiom`**：`Audit.lean` 逐定理
`#print axioms`，全部只列出 Lean 標準三公理（`propext`、`Classical.choice`、
`Quot.sound`）或不依賴任何公理。若有 `sorry`，清單會出現 `sorryAx`。

規模：**408 條定理/引理，6,502 行**（不含註解行另有數百行說明）。
`AuditAll.lean` 全庫審計：受檢宣告 1787、純構造 970、零 `sorry`、零自訂公理。

---

## 2. 模組 ↔ 文檔定理對照

| Lean 模組 | 行數 | 對應 | 核心內容 |
|---|---|---|---|
| `Polyrust/Monomial.lean` | 246 | 基礎層 | 指數向量、加/減/ℓcm/整除、支撐、首項單項式；純組合、零域公理、可 `decide` |
| `Polyrust/Tactics.lean` | 24 | 工具 | `int_ring`：無 Mathlib 的整數多項式歸一化宏 |
| `Polyrust/ClauseDuality.lean` | 125 | **T3(a)** | `clause_duality`：σ ⊨ C ⟺ P_C(σ)=0；`field_poly_bit`；`cnf_duality` |
| `Polyrust/ClauseAlgebra.lean` | 260 | **T3(b)** | `resolution_identity`（消解恆等式，逐點、無條件）；`learned_preserves_models`、`learned_preserves_polyZero`；`unsat_iff_no_polyZero` |
| `Polyrust/WatchMove.lean` | 190 | **T3(b) 旁路：CDCL 傳播資料結構層**（v0.1.4 新增） | `watch_move0/1_preserves_sat`：監視文字**交換**移動保持子句語義（任意賦值）；`clauseSat_all_false`：全假子句不滿足（衝突偵測健全性）；`watchMoves_preserve_sat`：**任意步數**移動後語義必然如初（迭代不變量）；`watch_overwrite_unsound`：覆寫版反例（零公理純計算）——v0.1.4 @brute 抓到的缺陷紀錄 |
| `Polyrust/UniPoly.lean` | 256 | **T8** | `eval_add/mul/sub`（求值環同態）；`div_linear`（Horner 除法）；`vanishing_prod_dvd`；`qap_duality` |
| `Polyrust/Squarefree.lean` | 372 | **T4** | `standard_implies_squarefree`；`toBits/ofBits` 雙射；`squarefree_count`（恰 2ⁿ）；`sublist_chain_length` + `buchberger_extension_bound`（≤ 2ⁿ）+ `no_infinite_sublist_chain` |
| `Polyrust/Embedding.lean` | 140 | **L0** | `L0_mod_faithful`（\|n\| < 2⁶¹−1 ⇒ 模零 ⟺ 為零）；`eval_abs_bound`；`L0_eval_faithful` |
| `Polyrust/MicroInstance.lean` | 104 | T1/T2/T6/T7 微實例 | 加法規則、上下文矛盾、宏選臂三系統的全枚舉 |
| `Polyrust/SPoly.lean` | 356 | **T5** | `mulMono_*`（單項式倍乘律）、`sPoly_mem_genIdeal`、`genIdeal_insert_sPoly`、`coprime_criterion`（互素準則）、`sPoly_chain_decomposition` + `chain_criterion`（鏈準則）、`sPoly_self` |
| `Polyrust/Canonical.lean` | 219 | **T7(a)** | `leadIdeal_upper`；`reduced_unique`：簡化 Gröbner 基**若存在則唯一**；`reduced_zero_of_mem`、`lead_determines_element`；非空性模型 `pureNat` |
| `Polyrust/T6Certificate.lean` | 429 | **T6** | `inIdeal_no_root`、`one_mem_no_root`（1 ∈ 理想 ⟹ 無 0/1 根）；`interpolation`（布爾 Lagrange 插值）；`no_root_certificate`、`no_root_poly_certificate`（無公共零點 ⟹ 存在乘子使組合恆等於非零）；第 4 條檔首承諾（逆元縮放為 1）由 `BoolNullstellensatz.mod_p_one_in_ideal` 兌現 |
| `Polyrust/BoolNullstellensatz.lean` | 332 | **T6 補完：布爾 Nullstellensatz**（v0.1.4 新增） | `bool_nullstellensatz`：每點有模 `p` 可逆系統元素（顯式逆元見證）⟹ 多項式函數乘子把系統組合成常數 1（模 `p`）；`bool_ns_no_root_of_certificate`（可靠方向）；`mod_p_one_in_ideal`（逆元縮放）；`allBits_nodup`（`Squarefree` 內，補上 `sum_delta` 的 Nodup 前提） |
| `Polyrust/T9EndToEnd.lean` | 764 | **T9** | `check`（獨立型別檢查器）、`genC`（約束編碼）、`genC_sound`（T1）、`genC_complete`（T2）、`root_implies_typable`、`typable_iff_root`（T6 判定等價）、`untypable_iff_no_root`、`isMonoAt_of_root`、`parseFuel_gen`/`parse_gen`（代碼生成 round-trip）、`arm_gating*`（T7(b) 閘控） |
| `Polyrust/BorrowOwnership.lean` | 511 | **T1/T2/T6 借用側** | 借用存活區間與衝突（與 `analysis.rs` 同式）；`borrow_sat_iff_clean`：**系統有 0/1 根 ⟺ 無衝突**；`clashClause_duality`／`assignClause_duality`（借用子句 ↔ T3(a) 對偶）；`borrow_clash_one_mem`／`borrow_assign_one_mem`（**1 ∈ 理想**，顯式組合）；所有權三規則（重疊／賦值／移動）＋ P5/P6 樣本模型；`t9_borrow_decision`（T9 加借用的端到端判定） |
| `Polyrust/T9Generalized.lean` | 786 | **T9 泛化 (a)** | 型別宇宙參數化：`Lang`（`enumAll`/`nodup`/`complete`/`numTy`/`eqbTy`/`num_ne_eqb`）；one-hot 用 `List.sum`；`tycheck_exclusive`、`genC_soundG`（T1）、`genC_completeG`（T2）、`typable_iff_rootG`（T6/T9）全部對任意可枚舉宇宙成立 |
| `Polyrust/ProductReduction.lean` | 208 | **T9 泛化 (b)** | 積型（引用型別 `&`/`&mut` × 基本型別）：`typable_pair_iff`（AND 語義）、`typable_pair_fst/snd`、`pairBitSum_eq_mul`（one-hot 乘積）、`pairBitSum_eq_one_imp` |
| `Polyrust/SumReduction.lean` | 166 | **T9 泛化 (c)** | 和型（`bool`=`true\|false` 變體和、`()` 單變體）：`type_typable_sum_iff`（OR 語義）、`sumBits_sum_eq_add`（位元相加）、`sumBits_sum_eq_two` |
| `Polyrust/OpAbstraction.lean` | 543 | **T9 泛化 (e)** | 運算子規則抽象：`BinSpec {in1,in2,out}` 為資料；`tycheckG_exclusive`、`genC_soundG2`（T1）、`genC_completeG2`（T2）、`typable_iff_rootG2`（T6/T9）對**任意**規格成立；Rust 9 種 `BinOp` = `arithSpec`/`cmpSpec`/`andSpec` 三個實例（§十機械驗證） |
| `Polyrust/MacroExpansion.lean` | 440 | **T7(b)** | 模板語法 + 上下文 + 代入；`subst_id/subst_comp/expand_comp`（**展開是同態**）；`mem_vars_subst`；`checkCtx_det`（型別唯一）；`checkCtx_expand`（**正確臂**）；`check_expand_reflect`、`check_expand_demands` + `arm_demand`（**需求表回推**）；`wrong_arm_untypable`、`wrong_arm_no_root`（**錯臂必被拒絕、其約束系統無 0/1 根**） |
| `Audit.lean` | 160 | 審計 | 98 條主定理的 `#print axioms`（不屬於 `lake build` 預設目標） |

---

## 3. 九定理覆蓋表（含邊界）

「邊界」一欄說明：**Lean 內證到哪裡為止、剩下什麼依賴 Rust 側或未被形式化**。

| 定理 | Lean 內證明的內容 | 邊界（不在 Lean 內） |
|---|---|---|
| **L0** | 小係數多項式在 0/1 點求值：模 p 為零 ⟺ 整數為零；求值界 ≤ 2²⁸ | 「小係數」的上界是假設；p = 2⁶¹−1 為素數用事實性引理 `P61_eq` |
| **T1** | `genC_sound`：檢查器接受 ⟹ 見證賦值 σ_D 是約束系統的 0/1 根（一般語言，非樣本）；**借用側** `borrow_sat_of_clean`（乾淨 ⟹ 有根）＋ `clashClause_false_of_live`（活躍衝突子句為假） | σ_D 的「未覆蓋節點補全」在 Rust 側；借用「活躍」由 `b − 1 = 0` 方程表示（與實作一致），未形式化生命週期／重借用／分支合流 |
| **T2** | `genC_complete`：任一 0/1 根逐型別解碼為檢查器答案（⟺ 而後 `typable_iff_root`）；**借用側** `borrow_sat_iff_clean`（有根 ⟹ 無衝突，前提：衝突節點皆活躍） | 同上；「活躍」的前提必須由程式結構提供（子句集單獨看是平凡可滿足的） |
| **T3(a)** | `clause_duality`、`cnf_duality`、`field_poly_bit`：滿足 ⟺ 子句多項式歸零 | 係數域的作用由 `bit` 的 0/1 編碼承擔；𝔽_p 版本經 L0 嵌入 |
| **T3(b)** | `resolution_identity`：P_{C∨D} = P_{l∨C}·P_D + P_C·P_{¬l∨D}（**逐點、無條件**）；`learned_preserves_models`/`learned_preserves_polyZero`：被蘊涵的學習子句不改變模型集與零集；**資料結構層**（`WatchMove`）：監視文字交換移動保持子句語義（`watch_move0/1_preserves_sat`）、衝突偵測健全性（`clauseSat_all_false`）、覆寫版反例（`watch_overwrite_unsound`） | 完整的 P_C ∈ ⟨P_Φ ∪ B⟩ 需要 Nullstellensatz／根式理想理論（本倉庫不含）；Lean 給的是**構造性核心**（消解組合 + 值層守恆）。監視移動的單步與**任意步數迭代**語義守恆已形式化（`watchMoves_preserve_sat`）；傳播演算法本身的終止性／完備性未形式化 |
| **T4** | `buchberger_extension_bound`：含域多項式時基擴充 ≤ 2ⁿ；`no_infinite_sublist_chain` | 抽象化為「標準單項式集合的嚴格遞減 Sublist 鏈」；不形式化演算法本身的實作 |
| **T5** | `coprime_criterion`（互素首項 ⟹ S → 0）、`sPoly_chain_decomposition` + `chain_criterion`（鏈準則）、`genIdeal_insert_sPoly`（不改變理想）、`sPoly_self` | 準則「不改變約化基」的完整陳述依賴 T7(a) 的唯一性；此處證的是理想層不變性 |
| **T6** | `one_mem_no_root`：1 ∈ 理想 ⟹ 無 0/1 根（求值同態論證）；`no_root_certificate`、`no_root_poly_certificate`：無公共零點 ⟹ 存在乘子使線性組合處處非零；**布爾 Nullstellensatz**（`BoolNullstellensatz`，v0.1.4）：每點有模 `p` 可逆系統元素（顯式逆元見證）⟹ 多項式函數乘子把系統組合成**常數 1（模 `p`）**（`bool_nullstellensatz`）＋反向 `bool_ns_no_root_of_certificate`（前提 `p ∤ 1`）；`mod_p_one_in_ideal`：處處非零證書＋逆元見證 ⟹ 常數 1 證書；**借用側** `borrow_one_mem_no_root`＋`borrow_clash_one_mem`／`borrow_assign_one_mem`（衝突系統的 1 有**顯式可讀見證**：`x_i x_j − x_j(x_i−1) − (x_j−1) = 1`） | 布爾 Nullstellensatz 證在**函數層**（{0,1}ⁿ 上逐點模 `p` 同餘，乘子為多項式函數）；**語法層**的 `Σ gᵢfᵢ + Σⱼ hⱼ(xⱼ²−xⱼ) = 1` 多項式恆等式未做（需多元除法），管線亦不需要。「可逆」前提帶顯式逆元見證；素數 `p` 下「可逆 ⟺ 非零」的 Bézout 等價不在本定理內（由 L0／Rust 側提供） |
| **T7(a)** | `reduced_unique`：固定項序下簡化 Gröbner 基**若存在則唯一**（含 `reduced_zero_of_mem`、`lead_determines_element`）；`reduced_unique_univariate` 在 `pureNat` 模型上的非空性 | **存在性不在本倉庫內**（Buchberger 收斂）；`HasMaxMono` 作為顯式前提（對 `pureNat` 已證） |
| **T7(b)** | `checkCtx_expand`（展開是同態：模板良型 + 同型代入 ⟹ 展開可定型）、`check_expand_demands`（需求表回推）、`wrong_arm_untypable`、`wrong_arm_no_root`（**錯臂 ⟹ 不可定型 ⟺ 約束系統無 0/1 根**）、`arm_gating*`（閘控退化律）、`MicroInstance.T7_wrong_arm_unsat` | 「錯臂系統 1 ∈ G」的代數見證在微實例中給出；一般情形依賴 Nullstellensatz |
| **T8** | `qap_duality`：Z ∣ A·B − C ⟺ z 滿足全部 R1CS 約束；`div_linear`（導線多項式的構造性除法）；`vanishing_prod_dvd` | deg ≤ m−1 的層面在 `UniPoly` 中以列表次數陳述；t_i 互異是前提 |
| **T9** | `typable_iff_root`（管線判定 ⟺ 獨立檢查器接受）、`untypable_iff_no_root`（UNSAT 側）、`parse_gen`（生成碼**必定可重解析**，含 `parseFuel_gen` 的燃料終止性）、`gen_length`/`sizeT` | **(b) rustc 可編譯**與 **(c) 真實語義保持**不在 Lean 內（依賴 Rust 編譯器與其語義），由 Rust 側 demo/obligations 實測；Lean 證的是「判定等價 + 生成碼可重解析 + 展開同態」 |

### 「錯了會怎樣」：若這些定理為假

* **T1/T2 為假** → 管線的 SAT/UNSAT 判定與真實型別系統脫鉤：會出現「管線說可解、檢查器拒絕」的腳本，`polyrust` 的結論失去意義（假陽性/假陰性皆可能）。
* **T3(a)/T3(b) 為假** → CDCL 學習的子句不在理想內 ⇒ 學習會**改變**解集：UNSAT 可能被「學成」SAT（漏報），或反之。
* **監視移動不健全（v0.1.4 實例）** → 傳播過程中子句被靜默修改（丟失文字）⇒ 回溯後以「變強」的子句剪枝 ⇒ **SAT 誤報 UNSAT**。此類缺陷與學習子句錯誤互為表裏：前者改子句內容、後者加錯誤子句，兩者都破壞解集守恆。@brute 對照常態化（純 3-SAT 硬樣本）已把這條路徑列為駐場測試。
* **T4 為假** → Buchberger 不終止：`groebner` 子命令可能永不返回（不可判定性障壁）。
* **T5 為假** → 準則跳過的 S-多項式其實非零 ⇒ 產出的集合不再是 Gröbner 基，T6 的判定失去完備性。
* **T6 為假** → 「1 ∈ G」不再是「不可定型」的判據：錯臂可能被判為可解（漏報不安全程序），這是最嚴重的一類錯誤。
* **T7(a) 為假** → 同一理想的約化基不唯一 ⇒ 判定結果依賴策略（`Normal` vs `FIFO`），無法對外承諾規範形式。
* **T7(b) 為假** → 宏展開改變可解性 ⇒ 宏是「不安全」的語法糖，錯臂可能被靜默接受。
* **T8 為假** → QAP 不再忠實於 R1CS ⇒ 算術化（SNARK 側）可能接受不滿足約束的見證。
* **T9 為假** → 生成的 Rust 代碼可能無法重解析或語義不同 ⇒ 端到端保證斷裂（(b)(c) 由實編實測覆蓋）。

---

## 3.5 借用與所有權（T9 的借用擴充）

`Polyrust/BorrowOwnership.lean` 把 Rust 側 `analysis.rs`／`constraints.rs` 的借用層
逐式搬進 Lean，並證三件「說死」的事：

**(1) 判定等價（借用側的 T6）**

```
borrow_sat_iff_clean : (∃ β, ∀ c ∈ borrowSystem live pairs assigns, c β = 0) ↔ pairs = [] ∧ assigns = []
```
前提是「每個衝突節點都是活躍借用」。這條定理把三個概念釘在一起：
* **代數**：約束系統（活躍方程 `b − 1`、衝突多項式 `b_i b_j`、排除方程 `b`）有 0/1 根；
* **分析**：衝突對集合與被排除集合皆空（`BorrowAnalysis::is_clean`）；
* **語義**：無重疊的 `&mut`、無「借用期間賦值」。

**(2) 子句 ⟷ 多項式（接 T3(a)）**

```
clashClause_poly     : clausePoly β [¬b_i, ¬b_j] = b_i · b_j
clashClause_duality  : clauseSat β [¬b_i, ¬b_j] = true ↔ b_i · b_j = 0
assignClause_duality : clauseSat β [¬b]        = true ↔ b = 0
```
也就是 CDCL 用的子句與 Buchberger 用的多項式是同一件事的兩種編碼——這正是
文檔 T1 所說「借用子句 ⇒ 子句多項式歸零」的機械版本。

**(3) 1 ∈ 理想（借用側的 T6 判定，含顯式見證）**

```
borrow_clash_one_mem  : 1 ∈ ⟨x_i x_j, x_i − 1, x_j − 1⟩   -- 組合：x_i x_j − x_j(x_i−1) − (x_j−1) = 1
borrow_assign_one_mem : 1 ∈ ⟨b, b − 1⟩                    -- 組合：b − (b−1) = 1
borrow_clash_no_root  : 上述集合無公共 0/1 零點
```
這給出「衝突系統的約化基 = {1}」的**算術核心**，且 1 的組合是**可讀的**——
不必引用 Nullstellensatz。

**所有權**：形式化三條規則，並指出它們在代數上只有兩種形狀：

| 規則 | 條件（`analysis.rs` 同式） | 多項式形狀 |
|---|---|---|
| 借用重疊 | 同 `var_def` ∧ `a.start < b.end ∧ b.start < a.end` | `x_i x_j = 0`（互斥） |
| 借用期間賦值 | `b.start ≤ p < b.stop` | `b = 0`（與活躍方程矛盾） |
| 借用期間移動 | 同上（`Move`） | `b = 0` |
| 移動後使用 | `m < p`，m = 移出點、p = 使用點 | `x_m x_p = 0`（**等價於一條衝突對**，`use_after_move_is_clash`） |

**精確性要點（兩處容易錯的地方，已寫死在程式裡）**

1. **非空區間會與自身重疊**（`overlaps_self_of_nonempty`），故「衝突」只能在
   **相異節點**之間談——對應 `analysis.rs` 的 `for j in (i+1)..`。把「同一借用被
   使用多次」當成衝突是**錯的**。
2. **子句集單獨看是平凡可滿足的**（全置 0 即可）。使系統非平凡的是每個活躍借用的
   `b − 1 = 0`。故 `borrow_sat_iff_clean` 的完備性方向**必須**假設衝突節點活躍——
   這與 Rust 側 `obligation_t1` 把 `borrow_vars` 全置 `Frac::ONE` 的做法一致。

**樣本對應**：`p5_conflict`／`p5_unsat`（`let r1 = &mut x; let r2 = &mut x; *r1 + *r2`，
區間重疊 ⇒ UNSAT）對 `p6_no_conflict`／`p6_sat`（`*(&mut x) + *(&mut x)`，區間不相交
⇒ SAT）。兩個程序語法幾乎相同，**唯一差別是存活區間是否重疊**——`p5_p6_differ`
把這一點形式化。

**未形式化（明確排除）**：生命週期參數 `'a`、重借用（reborrow）、非直線碼的分支合流、
`Rc`/`RefCell` 之類的內部可變性；Mini-Rust 亦無 `move` 構造，故「移動」兩條規則是
**規則層的形式化**，其樣本實測不在 12 樣本之列（見 `docs/EVIDENCE.md` 的標註）。

## 4. 衍生與次要引理（Lean 內可推得者一併形式化）

使用者要求「除九定理外，所有可邏輯推得的衍生/次要引理與功能也要寫進 Lean」。
本倉庫的做法是：**每個模組都把主定理所需的輔助結構一併入庫**，包含

* `Monomial`：單項式代數全套（加/減/ℓcm/整除/支撐/首項）——T4/T5/T7 的共同基礎；
* `ClauseAlgebra`：`clausePoly_append`、`clausePoly_nil_ne_zero`、`clausePoly_tautology`、
  `clausePoly_eq_zero_or_one`、`Lit.neg`/`litSat_neg`/`litFactor_neg`、`mod_iff_polyZero`、
  `entails_poly_vanishes`、`entails_resolvent`、`models_mono`、`unsat_of_nil_mem`；
* `Squarefree`：位串雙射（`ofBits_injective`、`squarefree_eq_of_bits_eq`）、
  `allBits_sound/complete`、`standard_set_subset_squarefree`；
* `SPoly`：`mulMono` 的全部代數律（`mulMono_mulMono`、`mulMono_subP`、`mulMono_isLead`、
  `dividesM_monoMul_iff`）、`quotM_lcmM_mul(_right)`、`genIdeal_mono`、`sPoly_apply`；
* `Canonical`：`subP_apply`、`int_sub_ne_zero`、`mon_zero`/`mon_of_ne`、`bounded_support_max`、
  `hmax_pureNat`；
* `T6Certificate`：`delta_*`（δ 函數全套）、`sum_*`、`interp_polyFn`、
  `exists_selection`、`selMult`、`sum_map_selMult`；
* `T9EndToEnd`：位元算術（`bit_and`、`bit_injective`、`bit_eq_zero/one_iff`）、
  單型性（`check_exclusive` 與兩個方向）、約束表成員分解（`mem_genC_*`）、
  出現關係（`Occurs`、`occurs_oneHot_mem_genC`、`isMonoAt_of_root`）、
  代碼生成長度（`gen_length`）、解析器燃料單調性；
* `MacroExpansion`：`subst_id`/`subst_comp`/`expand_comp`、`mem_vars_subst`、
  `checkCtx_det`、`demands` 需求表、`arm_demand`；
* `BorrowOwnership`：`overlaps_comm`、`overlaps_self_iff`、`not_overlaps_of_le`、
  `conflictsWith_comm`、`liveEq_zero_iff`、`mem_borrowSystem_*`（三條成員律）、
  `clean_iff_no_conflict`、`clauseSet_unsat_of_clash`、`BgenIdeal_*`（理想封閉性）、
  `use_after_move_is_clash`、`p5_one_mem_ideal`、`t9_borrow_unsat_of_clash`。

也就是：**凡是主定理證明中出現的結構，都以具名引理固化**，而不是塞在單一
大證明的隱形步驟裡——這是 Lean 的要求，也是可維護性的要求。

---

## 5. 三類「必須說死」的邊界

1. **邏輯邊界**：`propext`/`Classical.choice`/`Quot.sound` 是 Lean 標準三公理；
   `SPoly`/`Canonical`/`T6Certificate` 因整除/理想的判定性用了 `Classical.choice`
   （`decide` 化需要 `open Classical`）；`Monomial`、微實例、T9 的核心引理
   完全不依賴任何公理（審計輸出可見）。
2. **語義邊界**：Lean 的 `Expr` 是 Mini-Rust 的**核心子集**（i32/bool、加、比較、
   if-then-else、宏臂）。借用/所有權的完整語義、生命週期、`&mut` 別名規則
   不在 Lean 內，而在 Rust 側以子句 + 樣本全枚舉覆蓋。
3. **計算邊界**：Lean 證的是**數學正確性**（終止界、判定等價、嵌入保真），
   不是 Rust 實作的性能或記憶體行為；`rustc` 的可編譯性由實編實測。

---

## 6. 與 Rust 側的對接點

| Lean 定理 | Rust 側對應的實測 | 位置 |
|---|---|---|
| `typable_iff_root`、`untypable_iff_no_root` | 12 個樣本「管線 SAT/UNSAT ⟺ 檢查器接受/拒絕」 | `src/bin` / `cargo test` |
| `buchberger_extension_bound` | 樣本擴充次數 0–645 ≤ 2ⁿ | `groebner` 統計 |
| `coprime_criterion`/`chain_criterion` | 消除率 95%（712,549 → 37,823 對） | `groebner` 統計 |
| `reduced_unique` | 反序生成元 + FIFO 與原基逐元素相同 | T7(a) 義務 |
| `wrong_arm_*` | demo P7/P8 錯臂系統約化基 = {1} | demo |
| `borrow_sat_iff_clean`、`p5_unsat`／`p6_sat` | P5-twice-mut UNSAT、P6-temp-borrow SAT；借用位元 σ_D = 1（`obligation_t1`） | obligations |
| `qap_duality` | 7 個 SAT 樣本 QAP 驗證通過；竄改見證全被拒 | `qap` |
| `parse_gen` | 生成 12 個樣本代碼全部 `rustc --edition 2021` 編譯成功 | `codegen`/`minirust` |
