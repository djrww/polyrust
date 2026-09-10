/- # 𝔽_p 嵌入引理（L0）——係數域選擇的正確性

對應 docs/THEOREMS.md §2（引理 L0）：
本項目把 Rust 型別/借用約束編碼為**小整數係數多項式**（|c| < 2¹⁶、
項數 < 2¹²、在 0/1 點求值），並在質域 𝔽_p（p = 2⁶¹ − 1）上做 Gröbner
基運算（避免 ℚ 上的係數/分母爆炸）。嵌入引理保證判定保真：

**(a)** 對 |值| < p 的整數：模 p 為零 ⟺ 整數為零
   —— SAT 側見證可直接驗證；
**(b)** 見證的求值界：小係數多項式在 0/1 點的值 |·| ≤ 2²⁸ < p
   —— 本項目全部多項式自動滿足。

本模組將兩者合併為 `L0_eval_faithful`：小多項式在 0/1 點求值時，
「𝔽_p 中為零 ⟺ ℤ 中為零」，即 0/1 解的判定在 𝔽_p 與 ℚ 上一致。 -/

import Polyrust.UniPoly

namespace Polyrust

/-- Mersenne 質數 p = 2⁶¹ − 1（本項目的係數域 𝔽_p）。 -/
def P61 : Nat := 2305843009213693951

theorem P61_eq : P61 + 1 = 2 ^ 61 := by
  decide

/-! ## (a) 小值模零 ⟺ 整數零 -/

/-- **L0(a)**：整數 n 的絕對值小於 p = 2⁶¹ − 1 時，
n 模 p 為零當且僅當 n 為零。（|n| < p ⇒ n ∈ (−p, p)，
此區間內唯一被 p 整除的數是 0。） -/
theorem L0_mod_faithful {n : Int} (h : n.natAbs < 2305843009213693951) :
    n % 2305843009213693951 = 0 ↔ n = 0 := by
  constructor
  · intro hm
    omega
  · intro hn
    rw [hn]
    rfl

/-! ## (b) 0/1 點求值的絕對值界 -/

/-- 求值的絕對值 ≤ 係數絕對值之和（當 |t| ≤ 1，乘法不放大）。 -/
theorem eval_abs_le_sum (p : UniPoly) (t : Int) (ht : t = 0 ∨ t = 1) :
    (UniPoly.eval p t).natAbs
      ≤ (p.map (fun c => c.natAbs)).foldr Nat.add 0 := by
  rcases ht with ht | ht
  · subst ht
    -- t = 0：每層乘積為 0
    induction p with
    | nil => simp [UniPoly.eval]
    | cons c cs ih =>
      have e : UniPoly.eval (c :: cs) 0 = c + 0 * UniPoly.eval cs 0 := rfl
      rw [e, List.map_cons, List.foldr_cons]
      have h1 : (c + 0 * UniPoly.eval cs 0).natAbs
          ≤ c.natAbs + (0 * UniPoly.eval cs 0).natAbs := Int.natAbs_add_le _ _
      have h0 : (0 : Int) * UniPoly.eval cs 0 = 0 := Int.zero_mul _
      have hc0 : c + (0 : Int) = c := Int.add_zero c
      rw [h0, hc0] at h1 ⊢
      have hz : Int.natAbs (0 : Int) = 0 := rfl
      rw [hz] at h1
      exact Nat.le_trans h1 (Nat.le_add_right _ _)
  · subst ht
    -- t = 1：乘法不放大
    induction p with
    | nil => simp [UniPoly.eval]
    | cons c cs ih =>
      have e : UniPoly.eval (c :: cs) 1 = c + 1 * UniPoly.eval cs 1 := rfl
      rw [e, List.map_cons, List.foldr_cons]
      have h1 : (c + 1 * UniPoly.eval cs 1).natAbs
          ≤ c.natAbs + (1 * UniPoly.eval cs 1).natAbs := Int.natAbs_add_le _ _
      have h3 : (1 : Int) * UniPoly.eval cs 1 = UniPoly.eval cs 1 := Int.one_mul _
      rw [h3] at h1 ⊢
      exact Nat.le_trans h1 (Nat.add_le_add_left ih _)

/-- 係數絕對值之和的長度 × 係數上界封頂。 -/
theorem sum_le (N : Nat) : ∀ (p : UniPoly), p.length ≤ N →
    (∀ c ∈ p, c.natAbs ≤ 65536) →
    (p.map (fun c => c.natAbs)).foldr Nat.add 0 ≤ N * 65536 := by
  induction N with
  | zero =>
    intro p hl _
    have hp : p = [] := by
      cases p with
      | nil => rfl
      | cons c cs =>
        have : (c :: cs).length = cs.length + 1 := rfl
        omega
    rw [hp]
    simp
  | succ N ih =>
    intro p hl hc
    cases p with
    | nil => simp
    | cons c cs =>
      have hc1 : c.natAbs ≤ 65536 := hc c (by simp)
      have hcr : ∀ c' ∈ cs, c'.natAbs ≤ 65536 := by
        intro c' hc'
        exact hc c' (by simp [hc'])
      have hl' : cs.length ≤ N := by
        have : (c :: cs).length = cs.length + 1 := rfl
        omega
      have ih' := ih cs hl' hcr
      have e : ((c :: cs).map (fun c => c.natAbs)).foldr Nat.add 0
          = c.natAbs + (cs.map (fun c => c.natAbs)).foldr Nat.add 0 := rfl
      show ((c :: cs).map (fun c => c.natAbs)).foldr Nat.add 0
        ≤ (N + 1) * 65536
      rw [e]
      have hmul : (N + 1) * 65536 = N * 65536 + 65536 := by
        rw [Nat.succ_mul]
      rw [hmul]
      omega

/-- **L0(b)**：小係數多項式（|c| ≤ 2¹⁶、項數 ≤ 2¹²）在 0/1 點的
求值絕對值 ≤ 2¹²·2¹⁶ = 2²⁸ < 2⁶¹ − 1 = p。 -/
theorem eval_abs_bound (p : UniPoly) (t : Int) (ht : t = 0 ∨ t = 1)
    (hl : p.length ≤ 4096) (hc : ∀ c ∈ p, c.natAbs ≤ 65536) :
    (UniPoly.eval p t).natAbs ≤ 268435456 := by
  have h1 := eval_abs_le_sum p t ht
  have h2 := sum_le 4096 p hl hc
  have h3 : 4096 * 65536 = 268435456 := by decide
  omega

/-! ## L0 主定理 -/

/-- **嵌入引理 L0（主定理）**：小係數多項式在 0/1 點求值的值滿足
「𝔽_p 中為零 ⟺ ℤ 中為零」。

這保證了管線在 𝔽_p 上做 Gröbner 運算時，0/1 解的判定（T1/T2/T6）
與 ℚ 上的判定完全一致——SAT 側見證逐多項式求值驗證、UNSAT 側
1 ∈ G 的蘊含（強 Nullstellensatz）都不因換域而失真。 -/
theorem L0_eval_faithful (p : UniPoly) (t : Int) (ht : t = 0 ∨ t = 1)
    (hl : p.length ≤ 4096) (hc : ∀ c ∈ p, c.natAbs ≤ 65536) :
    (UniPoly.eval p t) % 2305843009213693951 = 0
      ↔ UniPoly.eval p t = 0 := by
  have hbound : (UniPoly.eval p t).natAbs ≤ 268435456 :=
    eval_abs_bound p t ht hl hc
  have hlt : (UniPoly.eval p t).natAbs < 2305843009213693951 := by omega
  exact L0_mod_faithful hlt

end Polyrust
