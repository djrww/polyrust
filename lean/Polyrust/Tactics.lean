-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 小型多項式正規化策略（`int_ring`）

Lean 4 core 沒有 `ring`／`ring_nf`（那是 Mathlib 的）。本專案刻意不依賴 Mathlib，
因此提供一個**只用 core 引理**的多項式正規化策略：

`int_ring` = `simp only [分配律、交換律、結合律、單位元、負元] ; omega`

其效果是把 ℤ 上的多項式等式化為「單項式和」的正規形式後比較；
對於本專案出現的等式（次數低、變量少），足以完全判定。

**邊界**：這不是一般的一元/多元多項式環判定程式（沒有做 Gröbner 或
Hilbert 基的化簡），遇到需要真正多項式除法的目標不會成功——那類目標
在本專案中都以顯式的代數引理逐步處理。 -/

namespace Polyrust

macro "int_ring" : tactic => `(tactic|
  (simp only [Int.mul_add, Int.add_mul, Int.mul_comm, Int.mul_left_comm, Int.mul_assoc,
      Int.one_mul, Int.mul_one, Int.zero_mul, Int.mul_zero, Int.sub_eq_add_neg,
      Int.add_assoc, Int.add_comm, Int.add_left_comm, Int.add_zero, Int.zero_add,
      Int.neg_mul, Int.mul_neg, Int.neg_neg]
    <;> omega))

end Polyrust
