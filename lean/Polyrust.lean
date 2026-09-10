/- # polyrust 的 Lean 4 形式化（無 Mathlib，自包含）

docs/THEOREMS.md 中命題 P 證明骨幹的機械化：五個模組對應引理與定理如下。

| 模組 | 對應 | 內容 |
|---|---|---|
| `Polyrust.ClauseDuality` | T3(a) | 子句滿足 ⟺ 子句多項式歸零；域多項式 x²−x 刻畫 {0,1}ⁿ；CNF 對偶 |
| `Polyrust.UniPoly` | T8 | 求值環同態；構造性線性餘式定理；互異根 vanishing ⇒ ∏(X−tᵢ) 整除；QAP 對偶主定理 |
| `Polyrust.Squarefree` | T4 | 標準單項式 ⇒ 平方自由（域多項式入基）；平方自由 ↔ 位串雙射；恰 2ⁿ 個 ⇒ Buchberger 終止 |
| `Polyrust.Embedding` | L0 | 𝔽_p（p=2⁶¹−1）嵌入保真：小係數多項式 0/1 點求值模零 ⟺ 整數零 |
| `Polyrust.MicroInstance` | T1/T2/T6/T7 | 加法規則、上下文矛盾、宏選臂三個微型系統的全枚舉（可靠性+完備性+判定） |

構建：`lake build`（僅需 Lean 4.33+，無外部依賴）。 -/

import Polyrust.ClauseDuality
import Polyrust.UniPoly
import Polyrust.Squarefree
import Polyrust.Embedding
import Polyrust.MicroInstance
