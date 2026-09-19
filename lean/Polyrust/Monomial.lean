-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 單項式代數（共用基礎層）

本模組提供後續所有模組共用的**指數向量單項式代數**：

- 單項式 = 指數向量 `Nat → Nat`（「n 變量」指支撐 ⊆ {0,…,n−1}）；
- 整除 `dividesM` 是逐點 `≤`；最小公倍 `lcmM` 是逐點 `max`；
  最大公因 `gcdM` 是逐點 `min`；商 `quotM a b = b / a` 是逐點減法。

被以下模組使用：
- `Polyrust.Squarefree`（T4 終止性；標準單項式 ⇒ 平方自由）
- `Polyrust.SPoly`（T5 S-多項式準則：互素 ⟹ lcm = 積 ⟹ 首項對消）
- `Polyrust.Canonical`（T7(a) 約化基唯一性：首項理想對整除封閉）
- `Polyrust.ClauseAlgebra`（T3(b)：文字因子與乘積）

**這層的邊界**：所有定理都是逐點的 `Nat` 算術（`max`/`min`/`+`/`-`），
不涉及任何域、理想或項序的公理——因此零 sorry、零公理、可完全 `decide`。
「單項式序」的公理內容另見 `Polyrust.Canonical`（在那裡以顯式假設列出）。 -/

namespace Polyrust

/-- 單項式 = 指數向量（變量索引從 0 起）。 -/
abbrev MonoExp := Nat → Nat

/-- 單項式整除：逐點指數 ≤。 -/
def dividesM (a b : MonoExp) : Prop := ∀ j, a j ≤ b j

/-- 平方自由單項式：每個指數 ≤ 1。 -/
def squarefreeM (m : MonoExp) : Prop := ∀ j, m j ≤ 1

/-- 互素：任何維度上不同時為正（gcd = 1，含「無公共變量」）。 -/
def coprimeM (a b : MonoExp) : Prop := ∀ j, a j = 0 ∨ b j = 0

/-- 單位單項式 1（全零指數向量）。 -/
def monoOne : MonoExp := fun _ => 0

/-- 單項式乘法 = 指數相加。 -/
def monoMul (a b : MonoExp) : MonoExp := fun j => a j + b j

/-- 最小公倍 lcm = 逐點 max。 -/
def lcmM (a b : MonoExp) : MonoExp := fun j => max (a j) (b j)

/-- 最大公因 gcd = 逐點 min。 -/
def gcdM (a b : MonoExp) : MonoExp := fun j => min (a j) (b j)

/-- 商：`quotM a b = b / a`（逐點減法；要求 `a ∣ b` 才有意義）。 -/
def quotM (a b : MonoExp) : MonoExp := fun j => b j - a j

/-- 變量 xᵢ 的指數向量。 -/
def x1 (i : Nat) : MonoExp := fun j => if j = i then 1 else 0

/-- xᵢ² 的指數向量（域多項式 xᵢ² − xᵢ 的首項）。 -/
def x2 (i : Nat) : MonoExp := fun j => if j = i then 2 else 0

/-! ## 整除的基本性質 -/

theorem dividesM_refl (a : MonoExp) : dividesM a a := fun _ => Nat.le_refl _

theorem dividesM_trans {a b c : MonoExp} (h1 : dividesM a b) (h2 : dividesM b c) :
    dividesM a c := fun j => Nat.le_trans (h1 j) (h2 j)

theorem dividesM_antisymm {a b : MonoExp} (h1 : dividesM a b) (h2 : dividesM b a) : a = b :=
  funext fun j => Nat.le_antisymm (h1 j) (h2 j)

theorem dividesM_monoOne (a : MonoExp) : dividesM monoOne a := fun _ => Nat.zero_le _

theorem dividesM_monoMul_left (a b : MonoExp) : dividesM a (monoMul a b) := by
  intro j; unfold monoMul; omega

theorem dividesM_monoMul_right (a b : MonoExp) : dividesM b (monoMul a b) := by
  intro j; unfold monoMul
  have := Nat.le_add_left (b j) (a j)
  omega

theorem monoMul_comm (a b : MonoExp) : monoMul a b = monoMul b a := by
  funext j; unfold monoMul; omega

theorem monoMul_assoc (a b c : MonoExp) :
    monoMul (monoMul a b) c = monoMul a (monoMul b c) := by
  funext j; unfold monoMul; omega

theorem monoMul_monoOne (a : MonoExp) : monoMul a monoOne = a := by
  funext j; unfold monoMul monoOne; omega

theorem monoOne_monoMul (a : MonoExp) : monoMul monoOne a = a := by
  funext j; unfold monoMul monoOne; omega

/-! ## lcm / gcd 的普遍性質 -/

theorem dividesM_lcmM_left (a b : MonoExp) : dividesM a (lcmM a b) := by
  intro j; unfold lcmM; exact Nat.le_max_left _ _

theorem dividesM_lcmM_right (a b : MonoExp) : dividesM b (lcmM a b) := by
  intro j; unfold lcmM; exact Nat.le_max_right _ _

/-- lcm 是最小上界：`lcm a b ∣ c ⟺ a ∣ c ∧ b ∣ c`。 -/
theorem dividesM_lcmM_iff (a b c : MonoExp) :
    dividesM (lcmM a b) c ↔ dividesM a c ∧ dividesM b c := by
  constructor
  · intro h
    exact ⟨fun j => Nat.le_trans (Nat.le_max_left _ _) (h j),
           fun j => Nat.le_trans (Nat.le_max_right _ _) (h j)⟩
  · rintro ⟨h1, h2⟩ j
    unfold lcmM
    exact Nat.max_le.mpr ⟨h1 j, h2 j⟩

theorem dividesM_gcdM_left (a b : MonoExp) : dividesM (gcdM a b) a := by
  intro j; unfold gcdM; exact Nat.min_le_left _ _

theorem dividesM_gcdM_right (a b : MonoExp) : dividesM (gcdM a b) b := by
  intro j; unfold gcdM; exact Nat.min_le_right _ _

/-- gcd 是最大下界：`c ∣ gcd a b ⟺ c ∣ a ∧ c ∣ b`。 -/
theorem dividesM_gcdM_iff (a b c : MonoExp) :
    dividesM c (gcdM a b) ↔ dividesM c a ∧ dividesM c b := by
  constructor
  · intro h
    exact ⟨fun j => Nat.le_trans (h j) (Nat.min_le_left _ _),
           fun j => Nat.le_trans (h j) (Nat.min_le_right _ _)⟩
  · rintro ⟨h1, h2⟩ j
    unfold gcdM
    exact Nat.le_min.mpr ⟨h1 j, h2 j⟩

/-! ## 商 -/

/-- `b · (a / b) = a`（當 `b ∣ a`）。 -/
theorem monoMul_quotM {a b : MonoExp} (h : dividesM b a) :
    monoMul b (quotM b a) = a := by
  funext j
  have hj : b j ≤ a j := h j
  unfold monoMul quotM
  omega

/-- `(a / a) = 1`。 -/
theorem quotM_self (a : MonoExp) : quotM a a = monoOne := by
  funext j; unfold quotM monoOne; omega

theorem quotM_monoMul_left (a b : MonoExp) : quotM a (monoMul a b) = b := by
  funext j; unfold quotM monoMul; omega

theorem quotM_monoMul_right (a b : MonoExp) : quotM b (monoMul a b) = a := by
  funext j; unfold quotM monoMul; omega

/-- 商對 lcm 的分解：`a ∣ b` 且 `b ∣ γ` 時 `γ / a = (b/a)·(γ/b)`。
這正是 S-多項式係數化簡（T5(b) 鏈準則）用到的**單項式記帳恆等式**。 -/
theorem quotM_factor {a b γ : MonoExp} (h1 : dividesM a b) (h2 : dividesM b γ) :
    quotM a γ = monoMul (quotM a b) (quotM b γ) := by
  funext j
  have h1j : a j ≤ b j := h1 j
  have h2j : b j ≤ γ j := h2 j
  show γ j - a j = (b j - a j) + (γ j - b j)
  omega

/-! ## 互素：lcm = 積 -/

/-- **互素準則的算術核心（T5(a)）**：`gcd a b = 1` ⟹ `lcm a b = a·b`。 -/
theorem lcmM_eq_monoMul_of_coprime {a b : MonoExp} (h : coprimeM a b) :
    lcmM a b = monoMul a b := by
  funext j
  have hj : a j = 0 ∨ b j = 0 := h j
  unfold lcmM monoMul
  rcases hj with h0 | h0 <;> omega

/-- 互素時 `lcm / a = b`——S-多項式兩側乘子恰好是對方的首項。 -/
theorem quotM_lcmM_left_of_coprime {a b : MonoExp} (h : coprimeM a b) :
    quotM a (lcmM a b) = b := by
  rw [lcmM_eq_monoMul_of_coprime h, quotM_monoMul_left]

theorem quotM_lcmM_right_of_coprime {a b : MonoExp} (h : coprimeM a b) :
    quotM b (lcmM a b) = a := by
  rw [lcmM_eq_monoMul_of_coprime h, quotM_monoMul_right]

theorem gcdM_eq_monoOne_of_coprime {a b : MonoExp} (h : coprimeM a b) :
    gcdM a b = monoOne := by
  funext j
  have hj : a j = 0 ∨ b j = 0 := h j
  unfold gcdM monoOne
  rcases hj with h0 | h0 <;> omega

theorem coprimeM_comm {a b : MonoExp} (h : coprimeM a b) : coprimeM b a :=
  fun j => (h j).symm

/-! ## xᵢ²、平方自由與標準單項式的關係 -/

theorem x2_self (i : Nat) : x2 i i = 2 := by
  simp [x2]

theorem x2_ne {i j : Nat} (h : j ≠ i) : x2 i j = 0 := by
  simp [x2, h]

theorem x1_le_x2 (i : Nat) : dividesM (x1 i) (x2 i) := by
  intro j
  by_cases hj : j = i
  · rw [hj]; simp [x1, x2]
  · simp [x1, x2, hj]

/-- 非平方自由（某維指數 ≥ 2）⟹ xᵢ² 整除之。 -/
theorem x2_divides_of_ge2 (m : MonoExp) (i : Nat) (h : 2 ≤ m i) : dividesM (x2 i) m := by
  intro j
  by_cases hj : j = i
  · rw [hj]
    simp [x2]
    exact h
  · have hx : x2 i j = 0 := x2_ne hj
    rw [hx]
    exact Nat.zero_le _

/-- 平方自由單項式 = 不被任何 xᵢ² 整除。 -/
theorem not_x2_dvd_of_squarefree {m : MonoExp} (h : squarefreeM m) (i : Nat) :
    ¬ dividesM (x2 i) m := by
  intro hd
  have h2 : 2 ≤ m i := by
    have := hd i
    rwa [x2_self] at this
  have h1 : m i ≤ 1 := h i
  omega

/-- 平方自由：任何維度指數 0 或 1。 -/
theorem squarefreeM_iff_le_one (m : MonoExp) :
    squarefreeM m ↔ ∀ j, m j = 0 ∨ m j = 1 := by
  constructor
  · intro h j
    have := h j
    omega
  · intro h j
    rcases h j with h0 | h1 <;> omega

/-- 平方自由且整除 b ⟹ 整除 b 的「平方自由部分」。 -/
theorem squarefree_of_dvd {a b : MonoExp} (h : dividesM a b) (hb : squarefreeM b) :
    squarefreeM a := by
  intro j
  have h1 : a j ≤ b j := h j
  have h2 : b j ≤ 1 := hb j
  omega

theorem monoOne_squarefree : squarefreeM monoOne := fun _ => Nat.zero_le _

theorem monoMul_squarefree {a b : MonoExp} (ha : squarefreeM a) (hb : squarefreeM b)
    (hc : coprimeM a b) : squarefreeM (monoMul a b) := by
  intro j
  have haj : a j ≤ 1 := ha j
  have hbj : b j ≤ 1 := hb j
  have hcj : a j = 0 ∨ b j = 0 := hc j
  unfold monoMul
  rcases hcj with h0 | h0 <;> omega

end Polyrust
