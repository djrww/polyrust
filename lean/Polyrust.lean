-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # polyrust 的 Lean 4 形式化（無 Mathlib，自包含）

`docs/THEOREMS.md` 中命題 P 證明骨幹的機械化：模組對應引理與定理如下。

| 模組 | 對應 | 內容 |
|---|---|---|
| `Polyrust.Monomial` | 基礎層 | 單項式指數向量、整除、lcm/quot、支撐、首項；純組合層（無域/理想公理） |
| `Polyrust.Tactics` | 工具 | `int_ring`：無 Mathlib 的整數多項式歸一化宏 |
| `Polyrust.ClauseDuality` | T3(a) | 子句滿足 ⟺ 子句多項式歸零；域多項式 x²−x 刻畫 {0,1}ⁿ；CNF 對偶 |
| `Polyrust.ClauseAlgebra` | T3(b) | 消解恆等式（逐點、無條件）；學習子句保留模型集/零集；UNSAT ⟺ 無零點多項式 |
| `Polyrust.WatchMove` | T3(b) 旁路：CDCL 傳播資料結構層 | 監視文字**交換**移動保持子句語義（`watch_move0/1_preserves_sat`）；全假子句不滿足（衝突偵測健全性）；**覆寫版反例**（v0.1.4 @brute 抓到的缺陷紀錄） |
| `Polyrust.UniPoly` | T8 | 求值環同態；構造性線性餘式定理；互異根 vanishing ⇒ ∏(X−tᵢ) 整除；QAP 對偶主定理 |
| `Polyrust.Squarefree` | T4 | 標準單項式 ⇒ 平方自由（域多項式入基）；平方自由 ↔ 位串雙射；恰 2ⁿ 個 ⇒ Buchberger 終止 |
| `Polyrust.Composition` | **T10**（組合性） | 變量截斷/平移下整除保持；標準單項式分解引理（`standard_iff_factor`）；不相交宇宙之積（界 2^{n₁}·2^{n₂} = 2^{n₁+n₂}）；**加法界** Σ2^{nᵢ} ≤ 2^{Σnᵢ}（模塊化：常數大小組件 ⇒ 總界線性於程式規模） |
| `Polyrust.BoundedStandard` | **∏kᵢ 一般化** | 純冪整除 ⇒ 指數逐維有界；混合進制枚舉 `allBounded` 長度 = ∏ kᵢ；`buchberger_extension_bound_general`；kᵢ ≡ 2 特例回到 T4 的 2ⁿ |
| `Polyrust.Embedding` | L0 | 𝔽_p（p=2⁶¹−1）嵌入保真：小係數多項式 0/1 點求值模零 ⟺ 整數零 |
| `Polyrust.MicroInstance` | T1/T2/T6/T7 | 加法規則、上下文矛盾、宏選臂三個微型系統的全枚舉（可靠性+完備性+判定） |
| `Polyrust.SPoly` | T5 | S-多項式：單項式乘法的性質、S-多項式 ∈ 生成理想、Buchberger 判準（鏈分解、互質首項相消） |
| `Polyrust.Canonical` | T7(a) | 既約/全簡化標準形；**若**簡化 Gröbner 基存在則唯一（CLO §2.7 Thm 5）；非空性模型 `pureNat` |
| `Polyrust.T6Certificate` | T6 | 求值同態、無公共零點證書（乘子 + 組合=1）、布爾 Lagrange 插值；多項式乘子版本 |
| `Polyrust.BoolNullstellensatz` | T6 補完：布爾 Nullstellensatz（v0.1.4 新增） | `bool_nullstellensatz`：每點有模 `p` 可逆系統元素（顯式逆元見證）⟹ **多項式函數乘子把系統組合成常數 1（模 `p`）**（拉格朗日基顯式構造）；`bool_ns_no_root_of_certificate`（可靠方向）；`mod_p_one_in_ideal`（逆元縮放，兌現 `T6Certificate` 檔首承諾）；`allBits_nodup` 補上 `sum_delta` 的 Nodup 前提 |
| `Polyrust.T9EndToEnd` | T9 | 迷你語言型別檢查器；約束編碼的可靠性/完備性/判定等價（根 ⟺ 可定型）；代碼生成 round-trip |
| `Polyrust.T9Generalized` | T9 泛化 | 型別宇宙泛化：把寫死的 2 型別抽象為任意可枚舉宇宙（`Lang`：`enumAll`/`nodup`/`complete`/`numTy`/`eqbTy`/`num_ne_eqb`），one-hot 用 `List.sum`；T1/T2/T6/T9 全部參數化，新增型別零新證明 |
| `Polyrust.ProductReduction` | T9 泛化 (b) | 積型歸約：複合型別（`pair`/積型）檢查與可定型性歸約為逐欄位檢查；pair one-hot = 分量 one-hot 之乘積（`pairBitSum_eq_mul`） |
| `Polyrust.SumReduction` | T9 泛化 (c) | 和型歸約：複合型別（`inl`/`inr`/和型）檢查與可定型性歸約為變體檢查；OR 語義（`type_typable_sum_iff`）；sum 位元 = 變體位元之和（`sumBits_sum_eq_add`） |
| `Polyrust.OpAbstraction` | T9 泛化 (e) | 運算子規則抽象：二元運算子抽象為 `BinSpec {in1,in2,out}`，`binop s a b` 帶規格標籤；T1/T2/T6/T9 對**任意**規格成立，Rust 9 種 `BinOp` 是 9 個實例（`arithSpec`/`cmpSpec`/`andSpec`），新增運算子零新證明 |
| `Polyrust.BorrowOwnership` | T1/T2/T6 借用側 | 借用存活區間與衝突（與 `analysis.rs` 同式）；`borrow_sat_iff_clean`（有根 ⟺ 無衝突）；借用子句 ↔ T3(a) 對偶；`borrow_clash_one_mem`（1 ∈ 理想，顯式組合）；所有權三規則（重疊／賦值／移動）與 P5/P6 樣本模型 |
| `Polyrust.MacroExpansion` | T7(b) | 模板語法/上下文/代入；展開是同態（正確臂可定型）；需求表回推（錯臂必被拒絕，且其約束系統無 0/1 根） |
| `Polyrust.TypeUniverse7PlusI` | 7+i 宇宙 | 7 基底 + i 擴展（17 種完整宇宙）；one-hot 分解；宇宙單調性；Lang 實例 |
| `Polyrust.ModuleFlatten` | 模組展平 | 模組樹展平為線性約束；路徑解析 |
| `Polyrust.MatchDecisionTree` | match 編譯 | `Pat`/`MatchArm`/`DecisionTree`；決策樹深度 ≤ 臂數；編譯保語義 |
| `Polyrust.ProductSum` | 積和混合 | 積型與和型的混合歸約 |
| `Polyrust.LifetimeRegion` | Lifetime | `'a`/`'static`、outlives 圖、無環鐵律、NLL 區間重疊 |
| `Polyrust.UnsafeContext` | Unsafe | 裸指針解析、unsafe 閘控、Pure/No-IO 邊界 |
| `Polyrust.AsyncStateMachine` | Async | Future 狀態機、輪詢約束、one-hot 狀態 |
| `Polyrust.StdlibEncoding` | Stdlib | Vec/String/HashMap 的多項式編碼 |
| `Polyrust.LoopContract` | Loop | 循環不變量、歸納契約 |
| `Polyrust.TraitImpl` | Trait | trait/impl 解析、方法歸約 |
| `Polyrust.F4` | **F4**（批矩陣） | 符號預處理、矩陣構建、行階梯保持理想、塊對角、稀疏、平方自由化、F4 理想不變量 |
| `Polyrust.F5` | **F5**（簽名準則） | Signature、sigLT 傳遞/反自反、F5Criterion、RewrittenCriterion、sig-safe 消元、F4F5 結合、85% 零歸約消除 |
| `Polyrust.Minor` | **次要**（支撐性） | 位元算術 `bit_*`、Lit 支撐、MonoExp 整除/lcm、supportLe、List 求和支撐、field poly 支撐、稀疏行操作、FNV 哈希單調、簽名比較傳遞、squarefree mono 支撐 |
| `Polyrust.IronLaw` | **新增**（鐵律核心） | one-hot 排他、field 多項式 x²−x=0、借用衝突互斥、lifetime 無環（static 出超所有、無自環）、unsafe 邊界、watch 移動保語義、子句對偶、F4 矩陣理想不變、F5 簽名單調、塊對角獨立 |
| `Polyrust.Derived` | **衍生**（由核心推出） | pair/sum 可定型推論（AND/OR 語義）、pairBitSum 乘積、sumBits 相加、isMonoAt 區域化、borrow 1∈理想、watch 多步守恆、P5/P6 差異、稀疏密度≤20%推論、塊數推論、F5 零歸約消除 |
| `Polyrust.Completion` | **補全**（雙向完備） | clauseSat↔polyZero 雙向拆分與 false↔1、borrow_sat↔clean 雙向、typable↔root 雙向（T9 與泛化）、watch 移動 iff、parse/gen round-trip 雙向、one-hot sum=1↔唯一真 雙向、field poly 雙向刻畫、F4 矩陣消元↔S可歸約 雙向、F5 跳過↔零歸約 雙向、Fp 嵌入雙向 |
| `Polyrust.IncrementalIteration` | **增量迭代**（第5類，迭代收斂） | parseFuel fuel 單調/穩定/收斂、gen 長度迭代與 sizeT 單調、check 分解與 Typable/IsRoot 單調、borrowSystem 子集單調與 UNSAT 單調、F4 理想不變迭代/塊獨立、F5 簽名傳遞閉包/準則單調/sig-safe 鏈、F4F5 迭代收斂、LoopContract fuel 迭代、端到端 t9+borrow+f4f5 迭代收斂、iff 補充 |

構建：`lake build`（僅需 Lean 4.33+，無外部依賴）。 -/

import Polyrust.Monomial
import Polyrust.Tactics
import Polyrust.ClauseDuality
import Polyrust.ClauseAlgebra
import Polyrust.WatchMove
import Polyrust.UniPoly
import Polyrust.Squarefree
import Polyrust.Embedding
import Polyrust.MicroInstance
import Polyrust.SPoly
import Polyrust.Canonical
import Polyrust.T6Certificate
import Polyrust.BoolNullstellensatz
import Polyrust.T9EndToEnd
import Polyrust.T9Generalized
import Polyrust.ProductReduction
import Polyrust.SumReduction
import Polyrust.OpAbstraction
import Polyrust.MacroExpansion
import Polyrust.BorrowOwnership
import Polyrust.TypeUniverse7PlusI
import Polyrust.ModuleFlatten
import Polyrust.MatchDecisionTree
import Polyrust.ProductSum
import Polyrust.LifetimeRegion
import Polyrust.UnsafeContext
import Polyrust.AsyncStateMachine
import Polyrust.StdlibEncoding
import Polyrust.LoopContract
import Polyrust.TraitImpl
import Polyrust.F4
import Polyrust.F5
import Polyrust.Minor
import Polyrust.IronLaw
import Polyrust.Derived
import Polyrust.Completion
import Polyrust.IncrementalIteration
import Polyrust.Bidirectional7Files
import Polyrust.UnsafeSafety
import Polyrust.Composition
import Polyrust.BoundedStandard
