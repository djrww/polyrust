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
