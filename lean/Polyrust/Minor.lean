/- # 次要引理（支撐性）

對應任務四類中的「次要」：為鐵律與完備性提供底層支撐的純組合/位元引理。

本模組零依賴、純構造、零 sorry、零自定義 axiom。
避免與既有命名衝突，所有新引理以 minor_ 前綴或 _minor 後綴命名。
-/

import Polyrust.Monomial
import Polyrust.ClauseDuality
import Polyrust.ClauseAlgebra
import Polyrust.Squarefree
import Polyrust.T9EndToEnd
import Polyrust.T9Generalized

namespace Polyrust

/-! ## 一、位元算術支撐 -/

theorem minor_bit_mul_self (b : Bool) : bit b * bit b = bit b := by
  cases b <;> simp [bit]

theorem minor_bit_mul_not (b : Bool) : bit b * bit (!b) = 0 := by
  cases b <;> simp [bit]

theorem minor_bit_eq_zero_or_one (b : Bool) : bit b = 0 ∨ bit b = 1 := by
  cases b
  · left; rfl
  · right; rfl

theorem minor_bit_le_one (b : Bool) : bit b ≤ 1 := by
  cases b <;> simp [bit]

theorem minor_bit_and_comm (a b : Bool) : bit (a && b) = bit (b && a) := by
  cases a <;> cases b <;> rfl

theorem minor_bit_or_eq_add_sub_mul (a b : Bool) :
    bit (a || b) = bit a + bit b - bit a * bit b := by
  cases a <;> cases b <;> simp [bit]

theorem minor_bit_not_eq_one_sub (b : Bool) : bit (!b) = 1 - bit b := by
  cases b <;> simp [bit]

theorem minor_bit_eq_zero_iff {b : Bool} : bit b = 0 ↔ b = false :=
  bit_eq_zero_iff

theorem minor_bit_eq_one_iff {b : Bool} : bit b = 1 ↔ b = true :=
  bit_eq_one_iff

theorem minor_bit_nonneg (b : Bool) : 0 ≤ bit b :=
  bit_nonneg b

theorem minor_bit_injective {a b : Bool} (h : bit a = bit b) : a = b :=
  bit_injective h

theorem minor_bit_and (a b : Bool) : bit (a && b) = bit a * bit b :=
  bit_and a b

theorem minor_bit_add_bit_not (b : Bool) : bit b + bit (!b) = 1 :=
  bit_add_bit_not b

/-! ## 二、Lit 支撐 -/

theorem minor_lit_neg_neg (l : Lit) : l.neg.neg = l :=
  Lit.neg_neg l

theorem minor_lit_neg_var (l : Lit) : l.neg.var = l.var :=
  Lit.neg_var l

theorem minor_litSat_neg (σ : Assignment) (l : Lit) :
    litSat σ l.neg = !litSat σ l :=
  litSat_neg σ l

theorem minor_litFactor_zero_or_one (σ : Assignment) (l : Lit) :
    litFactor σ l = 0 ∨ litFactor σ l = 1 := by
  unfold litFactor
  cases hb : litSat σ l with
  | true => left; simp [bit, hb]
  | false => right; simp [bit, hb]

theorem minor_litFactor_negLit (β : Assignment) (n : Nat) :
    litFactor β ⟨n, false⟩ = bit (β n) := by
  simp only [litFactor, litSat, bit]
  cases β n <;> rfl

/-! ## 三、MonoExp 支撐 -/

theorem minor_dividesM_refl (a : MonoExp) : dividesM a a :=
  dividesM_refl a

theorem minor_dividesM_trans {a b c : MonoExp}
    (h1 : dividesM a b) (h2 : dividesM b c) : dividesM a c :=
  dividesM_trans h1 h2

theorem minor_dividesM_antisymm {a b : MonoExp}
    (h1 : dividesM a b) (h2 : dividesM b a) : a = b :=
  dividesM_antisymm h1 h2

theorem minor_dividesM_monoOne (a : MonoExp) : dividesM monoOne a :=
  dividesM_monoOne a

theorem minor_monoMul_comm (a b : MonoExp) : monoMul a b = monoMul b a :=
  monoMul_comm a b

theorem minor_monoMul_assoc (a b c : MonoExp) :
    monoMul (monoMul a b) c = monoMul a (monoMul b c) :=
  monoMul_assoc a b c

theorem minor_coprimeM_comm {a b : MonoExp} (h : coprimeM a b) : coprimeM b a :=
  coprimeM_comm h

theorem minor_monoOne_squarefree : squarefreeM monoOne :=
  monoOne_squarefree

theorem minor_squarefree_of_dvd {a b : MonoExp}
    (h : dividesM a b) (hb : squarefreeM b) : squarefreeM a :=
  squarefree_of_dvd h hb

theorem minor_x1_le_x2 (i : Nat) : dividesM (x1 i) (x2 i) :=
  x1_le_x2 i

theorem minor_x2_divides_of_ge2 (m : MonoExp) (i : Nat) (h : 2 ≤ m i) :
    dividesM (x2 i) m :=
  x2_divides_of_ge2 m i h

/-! ## 四、supportLe 支撐 -/

theorem minor_supportLe_nth_false {f : Nat → Bool} {n : Nat}
    (h : supportLe f n) : f n = false :=
  supportLe_nth_false h

theorem minor_supportLe_mono {f : Nat → Bool} {n m : Nat}
    (h : supportLe f n) (hle : n ≤ m) : supportLe f m := by
  intro j hj
  exact h j (Nat.le_trans hle hj)

theorem minor_supportLe_empty : supportLe (fun _ => false) 0 := by
  intro j _; rfl

theorem minor_supportLeM_monoOne : supportLeM monoOne 0 := by
  intro j _; simp [monoOne]

/-! ## 五、List 求和支撐（one-hot 基礎） -/

theorem minor_listSum_nonneg {α : Type} {f : α → Int} {l : List α}
    (hge : ∀ x ∈ l, 0 ≤ f x) : 0 ≤ (l.map f).sum := by
  induction l with
  | nil => simp
  | cons a l ih =>
    have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (List.mem_cons.mpr (Or.inr hx))
    have hfa : 0 ≤ f a := hge a (List.mem_cons.mpr (Or.inl rfl))
    have ih' := ih hge'
    simp only [List.map_cons, List.sum_cons]
    omega

theorem minor_listSum_ge_one_of_mem_one {α : Type} {f : α → Int} {l : List α}
    {y : α} (hy : y ∈ l) (hy1 : f y = 1) (hge : ∀ x ∈ l, 0 ≤ f x) :
    1 ≤ (l.map f).sum := by
  induction l with
  | nil => simp at hy
  | cons a l ih =>
    simp only [List.map_cons, List.sum_cons, List.mem_cons] at *
    rcases hy with rfl | hy
    · rw [hy1]
      have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (Or.inr hx)
      have hn : 0 ≤ (l.map f).sum := minor_listSum_nonneg hge'
      omega
    · have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (Or.inr hx)
      have ih' := ih hy hge'
      have hfa : 0 ≤ f a := hge a (Or.inl rfl)
      omega

theorem minor_sum_mark_zero_of_not_mem {Ty : Type} [DecidableEq Ty]
    {l : List Ty} {x : Ty} (hx : x ∉ l) :
    (l.map (fun t => bit (decide (t = x)))).sum = 0 := by
  induction l with
  | nil => simp
  | cons a l ih =>
    simp only [List.mem_cons] at hx
    have hne : a ≠ x := fun h => hx (Or.inl h.symm)
    have ha : bit (decide (a = x)) = 0 := by simp [bit, hne]
    have hx' : x ∉ l := fun h => hx (Or.inr h)
    have ih' := ih hx'
    simp only [List.map_cons, List.sum_cons]
    rw [ha, ih']
    omega

theorem minor_sum_mark_eq_one_of_mem {Ty : Type} [DecidableEq Ty]
    {l : List Ty} {x : Ty} (hx : x ∈ l) (hnd : l.Nodup) :
    (l.map (fun t => bit (decide (t = x)))).sum = 1 := by
  induction l with
  | nil => simp at hx
  | cons a l ih =>
    simp only [List.map_cons, List.sum_cons, List.mem_cons, List.nodup_cons] at *
    rcases hnd with ⟨hna, hnd'⟩
    rcases hx with rfl | hx
    · have ha : bit (decide (x = x)) = 1 := by simp [bit]
      have hx_not : x ∉ l := fun h => hna h
      have hsl : (l.map (fun t => bit (decide (t = x)))).sum = 0 :=
        minor_sum_mark_zero_of_not_mem hx_not
      rw [ha, hsl]
      omega
    · have hne : a ≠ x := fun h => hna (h ▸ hx)
      have ha : bit (decide (a = x)) = 0 := by simp [bit, hne]
      have ih' := ih hx hnd'
      rw [ha, ih']
      omega

/-! ## 六、field poly 支撐 -/

theorem minor_field_poly_bit (b : Bool) : bit b * (bit b - 1) = 0 :=
  field_poly_bit b

theorem minor_fieldPoly_bool (b : Bool) : bit b * bit b - bit b = 0 := by
  cases b <;> simp [bit]

theorem minor_bit_mul_sub_one_eq_zero (b : Bool) : bit b * bit b - bit b = 0 :=
  minor_fieldPoly_bool b

/-! ## 七、one-hot 基礎支撐 -/

theorem minor_oneHot_exists_one {α : Type} {f : α → Int} {l : List α}
    (hge : ∀ x ∈ l, 0 ≤ f x) (hsum : (l.map f).sum = 1) :
    ∃ x ∈ l, f x = 1 := by
  induction l with
  | nil => simp at hsum
  | cons a l ih =>
    simp only [List.map_cons, List.sum_cons] at hsum
    have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (List.mem_cons.mpr (Or.inr hx))
    have hfa : 0 ≤ f a := hge a (List.mem_cons.mpr (Or.inl rfl))
    have hn : 0 ≤ (l.map f).sum := minor_listSum_nonneg hge'
    by_cases ha1 : f a = 1
    · exact ⟨a, List.mem_cons.mpr (Or.inl rfl), ha1⟩
    · have ha0 : f a = 0 := by omega
      have hs' : (l.map f).sum = 1 := by omega
      rcases ih hge' hs' with ⟨x, hx, hx1⟩
      exact ⟨x, List.mem_cons.mpr (Or.inr hx), hx1⟩

end Polyrust
