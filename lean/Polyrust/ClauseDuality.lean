/- # 子句–多項式對偶（定理 T3(a) 的核心）

對應 docs/THEOREMS.md §5（T3）：
子句 C = (ℓ₁ ∨ … ∨ ℓ_k) 編碼為多項式 P_C = ∏ᵢ (1 − x̃ᵢ)，
其中 x̃ᵢ = xᵢ（正文字）或 1 − xᵢ（負文字）。

**對偶引理**：賦值 σ 滿足 C ⟺ P_C(σ) = 0（在 0/1 賦值下）。
由此（配合域多項式 x² − x 將解空間限制於 {0,1}ⁿ）：
Φ 可滿足 ⟺ V₀/₁(P_Φ ∪ B) ≠ ∅。

證明方法：對子句長度歸納，按首文字在 σ 下的真假分況——
文字為真 ⟺ 對應因子為 0 ⟺ 乘積歸零。 -/

namespace Polyrust

/-- 文字：變量編號 + 極性（`true` = 正文字 xᵢ，`false` = 負文字 ¬xᵢ）。 -/
structure Lit where
  var : Nat
  pos : Bool
  deriving Repr

/-- 布爾賦值 σ : 變量 → {0,1}。 -/
abbrev Assignment := Nat → Bool

/-- σ 滿足文字 ℓ：正文字看 σᵢ，負文字看 ¬σᵢ。 -/
def litSat (σ : Assignment) (l : Lit) : Bool :=
  if l.pos then σ l.var else !σ l.var

/-- σ 滿足子句 C ⟺ 某文字為真（存在量詞的布爾編碼）。 -/
def clauseSat (σ : Assignment) (C : List Lit) : Bool :=
  C.any (litSat σ)

/-- 布爾值 → 整數（{0,1} 嵌入 ℤ；係數域 𝔽_p 的代表元）。 -/
def bit (b : Bool) : Int :=
  if b then 1 else 0

/-- 文字 ℓ 的多項式因子 (1 − x̃ℓ) 在賦值 σ 處的值。 -/
def litFactor (σ : Assignment) (l : Lit) : Int :=
  1 - bit (litSat σ l)

/-- 子句多項式 P_C = ∏ (1 − x̃ᵢ) 在賦值 σ 處的值。 -/
def clausePoly (σ : Assignment) (C : List Lit) : Int :=
  (C.map (litFactor σ)).foldr (fun a b => a * b) 1

theorem bit_eq_one (b : Bool) (h : b = true) : bit b = 1 := by
  rw [h]; rfl

theorem bit_eq_zero (b : Bool) (h : b = false) : bit b = 0 := by
  rw [h]; rfl

/-- **域多項式引理**：x² − x 在 0/1 賦值下恆為零——
這正是布爾解空間 {0,1}ⁿ 的代數刻畫（理想 B 的作用）。 -/
theorem field_poly_bit (b : Bool) : bit b * (bit b - 1) = 0 := by
  cases b <;> rfl

/-- **對偶引理（T3(a) 核心）**：
σ 滿足子句 C ⟺ 子句多項式 P_C 在 σ 處歸零。 -/
theorem clause_duality (σ : Assignment) (C : List Lit) :
    clauseSat σ C = true ↔ clausePoly σ C = 0 := by
  induction C with
  | nil =>
    -- 空子句不可滿足；P = 1 ≠ 0。兩側皆假。
    constructor
    · intro h
      rw [clauseSat, List.any_nil] at h
      exact absurd h (by simp)
    · intro h
      rw [clausePoly, List.map_nil, List.foldr_nil] at h
      exact absurd h (by simp)
  | cons l C' ih =>
    by_cases h : litSat σ l = true
    · -- 首文字滿足：因子 = 1 − 1 = 0，乘積歸零；子句滿足。兩側皆真。
      have hf : litFactor σ l = 0 := by simp [litFactor, h, bit]
      constructor
      · intro _
        rw [clausePoly, List.map_cons, List.foldr_cons, hf]
        simp
      · intro _
        rw [clauseSat, List.any_cons, h]
        simp
    · -- 首文字不滿足：因子 = 1 − 0 = 1，乘積 = 餘子句多項式；滿足性遞移。
      have hl : litSat σ l = false := by
        cases hb : litSat σ l with
        | false => rfl
        | true => exact absurd hb h
      have hf : litFactor σ l = 1 := by simp [litFactor, hl, bit]
      constructor
      · intro hsat
        rw [clauseSat, List.any_cons, hl, Bool.false_or] at hsat
        rw [clausePoly, List.map_cons, List.foldr_cons, hf, Int.one_mul]
        exact ih.mp hsat
      · intro hpoly
        rw [clausePoly, List.map_cons, List.foldr_cons, hf, Int.one_mul] at hpoly
        rw [clauseSat, List.any_cons, hl, Bool.false_or]
        exact ih.mpr hpoly

/-- CNF 公式 Φ 的滿足性。 -/
def cnfSat (σ : Assignment) (Φ : List (List Lit)) : Bool :=
  Φ.all (clauseSat σ)

/-- CNF 公式的多項式組 P_Φ 在 σ 處的值列表。 -/
def cnfPolys (σ : Assignment) (Φ : List (List Lit)) : List Int :=
  Φ.map (clausePoly σ)

/-- **CNF 對偶（T3(a) 全式）**：
Φ 在 σ 下可滿足 ⟺ P_Φ 的全部多項式在 σ 處歸零
（配合 `field_poly_bit`：解空間恰為 {0,1}ⁿ 上的模型集）。 -/
theorem cnf_duality (σ : Assignment) (Φ : List (List Lit)) :
    cnfSat σ Φ = true ↔ ∀ p ∈ cnfPolys σ Φ, p = 0 := by
  constructor
  · intro h p hp
    rw [cnfPolys] at hp
    obtain ⟨C, hC, hCp⟩ := List.mem_map.mp hp
    rw [← hCp]
    rw [cnfSat, List.all_eq_true] at h
    exact (clause_duality σ C).mp (h C hC)
  · intro h
    rw [cnfSat, List.all_eq_true]
    intro C hC
    have hmem : clausePoly σ C ∈ cnfPolys σ Φ := by
      rw [cnfPolys]
      exact List.mem_map.mpr ⟨C, hC, rfl⟩
    exact (clause_duality σ C).mpr (h _ hmem)

end Polyrust
