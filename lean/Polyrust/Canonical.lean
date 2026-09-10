/- # 約化 Gröbner 基的唯一性（定理 T7(a)）

對應 docs/THEOREMS.md §9（T7(a)）：**固定單項式序，理想的約化 Gröbner 基唯一。**

原文引 Cox–Little–O'Shea §2.7 定理 5。本模組把該論證分解為三個可機械證明的環節：

1. **首項理想對整除向上封閉**（`leadIdeal_upper`）：`m ∈ in(I)` ⟹ `m·m' ∈ in(I)`。
   這是因為理想對單項式倍乘封閉（`IsIdeal.smul_mem`），而首項在單項式倍乘下
   如實變化（`mulMono_isLead`）。
2. **互約化元素在固定首項下唯一**（`reduced_unique`，T7(a) 主定理）：
   若 f、g ∈ I，首項同為 μ，且兩者的非首項都**不落在 in(I) 內**，則 f = g。
   證明是純粹的「取極大項」論證：f − g ∈ I，若非零則其首項 m\* ∈ in(I)，
   而 m\* 又是 f 或 g 的非首項，與互約化矛盾。
3. **理想中沒有非零的完全約化元素**（`reduced_zero_of_mem`）：
   p ∈ I 且所有非零項都不在 in(I) 中 ⟹ p = 0。這是「正規形唯一／0 的正規形是 0」。

**邊界（說死）**：
- 本模組**不**證明「約化 Gröbner 基存在」（那需要 Buchberger 演算法與除法律）；
  它證明的是**若存在則唯一**——正是原文 §9(a) 的內容。
- 「極大項存在」（`HasMaxMono`）是**命題參數**，不是隱藏公理：本模組另外對
  具體模型（`pureNat`，即有限支撐的單變量多項式）**證明**它成立
  （`bounded_support_max`、`hmax_pureNat`），故主定理並非空洞成立。 -/

import Polyrust.SPoly

namespace Polyrust

open Classical

variable {I : MPoly → Prop}

/-! ## 首項理想 -/

/-- 理想 I 的首項理想 in(I)：m 是 I 中某元素的**首項單項式**。 -/
def leadIdeal (I : MPoly → Prop) (m : MonoExp) : Prop := ∃ p, I p ∧ IsLead m p

/-- **T7(a) 環節一**：首項理想對整除向上封閉——
`m ∈ in(I)` ⟹ `m·m' ∈ in(I)`（乘以任意單項式）。 -/
theorem leadIdeal_upper (hI : IsIdeal I) {a b : MonoExp}
    (ha : leadIdeal I a) (hab : dividesM a b) : leadIdeal I b := by
  obtain ⟨p, hpI, hlead⟩ := ha
  refine ⟨mulMono (quotM a b) p, hI.smul_mem _ _ hpI, ?_⟩
  have h := mulMono_isLead (μ := a) (μ' := quotM a b) hlead
  rw [monoMul_comm (quotM a b) a, monoMul_quotM hab] at h
  exact h

/-! ## 互約化與完全約化 -/

/-- 互約化：μ 是 p 的首項，且 p 的所有**非首項**都不落在 W 內。
（約化 Gröbner 基的條件：任一基元素的任何項都不被其他基元素的首項整除。） -/
def Interreduced (W : MonoExp → Prop) (μ : MonoExp) (p : MPoly) : Prop :=
  IsLead μ p ∧ ∀ m, p m ≠ 0 → m ≠ μ → ¬ W m

/-- 完全約化：p 的所有非零項都不落在 W 內。 -/
def FullyReduced (W : MonoExp → Prop) (p : MPoly) : Prop :=
  ∀ m, p m ≠ 0 → ¬ W m

/-- 「極大項存在」：任何非零多項式都有對整除極大的項。
對有限支撐多項式在與整除相容的項序下皆成立（見 `hmax_pureNat`）。 -/
def HasMaxMono (p : MPoly) : Prop :=
  (∃ m, p m ≠ 0) → ∃ m, p m ≠ 0 ∧ ∀ m', p m' ≠ 0 → dividesM m' m

theorem subP_apply (p q : MPoly) (m : MonoExp) : subP p q m = p m - q m := rfl

theorem int_sub_ne_zero {a b : Int} (h : a ≠ b) : a - b ≠ 0 := by
  intro hc
  exact h (by omega)

theorem int_sub_ne_zero_or {a b : Int} (h : a - b ≠ 0) : a ≠ 0 ∨ b ≠ 0 := by
  by_cases ha : a = 0
  · right
    intro hb
    exact h (by rw [ha, hb]; omega)
  · left; exact ha

/-! ## T7(a) 主定理：互約化 + 同首項 ⟹ 唯一 -/

theorem reduced_unique (hI : IsIdeal I) {μ : MonoExp} {f g : MPoly}
    (hfI : I f) (hgI : I g)
    (hf : Interreduced (leadIdeal I) μ f) (hg : Interreduced (leadIdeal I) μ g)
    (hcoeff : f μ = g μ)
    (hmax : HasMaxMono (subP f g)) :
    f = g := by
  obtain ⟨hflead, hfred⟩ := hf
  obtain ⟨hglead, hgred⟩ := hg
  funext m
  by_cases hm : f m = g m
  · exact hm
  · exfalso
    have hne : ∃ m, subP f g m ≠ 0 := ⟨m, by rw [subP_apply]; exact int_sub_ne_zero hm⟩
    obtain ⟨mstar, hmstar, hmaxstar⟩ := hmax hne
    have hmstar' : f mstar - g mstar ≠ 0 := hmstar
    have hmem : leadIdeal I mstar :=
      ⟨subP f g, hI.sub_mem f g hfI hgI,
       ⟨hmstar, fun m' hm' => hmaxstar m' hm'⟩⟩
    have hne_μ : mstar ≠ μ := by
      intro heq
      have hval : f mstar - g mstar ≠ 0 := hmstar'
      rw [heq, hcoeff] at hval
      exact hval (by omega)
    rcases int_sub_ne_zero_or hmstar' with hfne | hgne
    · exact hfred mstar hfne hne_μ hmem
    · exact hgred mstar hgne hne_μ hmem

/-- **T7(a) 環節三**：理想中沒有非零的完全約化元素——「0 的正規形是 0」。 -/
theorem reduced_zero_of_mem (_hI : IsIdeal I) {p : MPoly}
    (hpI : I p) (hred : FullyReduced (leadIdeal I) p)
    (hmax : HasMaxMono p) : p = zeroP := by
  funext m
  by_cases hm : p m = 0
  · exact hm
  · exfalso
    obtain ⟨mstar, hmstar, hmaxstar⟩ := hmax ⟨m, hm⟩
    have hmem : leadIdeal I mstar :=
      ⟨p, hpI, ⟨hmstar, fun m' hm' => hmaxstar m' hm'⟩⟩
    exact hred mstar hmstar hmem

/-- 衍生：若首項理想中的元素都互不相同地對應到基元素的首項
（即兩元素首項相同），則兩元素相等——「基元素被其首項唯一決定」。 -/
theorem lead_determines_element (hI : IsIdeal I) {μ : MonoExp} {f g : MPoly}
    (hfI : I f) (hgI : I g)
    (hf : Interreduced (leadIdeal I) μ f) (hg : Interreduced (leadIdeal I) μ g)
    (hcoeff : f μ = g μ) (hmax : HasMaxMono (subP f g)) : f = g :=
  reduced_unique hI hfI hgI hf hg hcoeff hmax

/-! ## 非空洞性：對具體模型證明「極大項存在」

模型：只有第 0 個變量可以非零的「純單項式」族
`mon n = x₀ⁿ`（即 m j = 0，∀ j ≥ 1），係數由 `c : Nat → Int` 給定。
這是有限支撐的單變量多項式，`HasMaxMono` 可由「取最大非零指數」直接證明。 -/

/-- 單變量模型的單項式 x₀ⁿ。 -/
def mon (n : Nat) : MonoExp := fun j => if j = 0 then n else 0

theorem mon_zero (n : Nat) : mon n 0 = n := by
  simp [mon]

theorem mon_of_ne {n j : Nat} (hj : j ≠ 0) : mon n j = 0 := by
  simp [mon, hj]

/-- 只含 `mon n` 型單項式的多項式（有限支撐的單變量情形）。 -/
noncomputable def pureNat (c : Nat → Int) : MPoly :=
  fun m => if (∀ j, 1 ≤ j → m j = 0) then c (m 0) else 0

theorem pureNat_mon (c : Nat → Int) (n : Nat) : pureNat c (mon n) = c n := by
  have hp : (∀ j, 1 ≤ j → mon n j = 0) := fun j hj => mon_of_ne (by omega)
  unfold pureNat
  rw [if_pos hp]
  simp [mon]

theorem pureNat_apply {c : Nat → Int} {m : MonoExp} (h : pureNat c m ≠ 0) :
    c (m 0) ≠ 0 ∧ ∀ j, 1 ≤ j → m j = 0 := by
  by_cases hp : (∀ j, 1 ≤ j → m j = 0)
  · exact ⟨by rwa [pureNat, if_pos hp] at h, hp⟩
  · rw [pureNat, if_neg hp] at h
    exact absurd rfl h

/-- 有限支撐的係數序列取最大非零指數（對 `Nat` 歸納）。 -/
theorem bounded_support_max : ∀ (N : Nat) (c : Nat → Int),
    (∀ n, N ≤ n → c n = 0) → (∃ n, c n ≠ 0) →
    ∃ n, c n ≠ 0 ∧ ∀ n', c n' ≠ 0 → n' ≤ n := by
  intro N
  induction N with
  | zero =>
    intro c hN hex
    obtain ⟨n, hn⟩ := hex
    exact absurd (hN n (Nat.zero_le n)) hn
  | succ N ih =>
    intro c hN hex
    by_cases h : c N = 0
    · have hN' : ∀ n, N ≤ n → c n = 0 := by
        intro n hn
        by_cases hlt : n = N
        · rw [hlt]; exact h
        · exact hN n (by omega)
      exact ih c hN' hex
    · refine ⟨N, h, ?_⟩
      intro n' hn'
      by_cases hlt : n' = N
      · rw [hlt]; exact Nat.le_refl N
      · by_cases hle : N + 1 ≤ n'
        · exact absurd (hN n' hle) hn'
        · omega

/-- **非空洞性**：對上述具體模型，`HasMaxMono` 成立。 -/
theorem hmax_pureNat (c : Nat → Int) (N : Nat) (hN : ∀ n, N ≤ n → c n = 0) :
    HasMaxMono (pureNat c) := by
  intro hex
  obtain ⟨m₀, hm₀⟩ := hex
  obtain ⟨hc0, _⟩ := pureNat_apply hm₀
  obtain ⟨n, hn, hmaxn⟩ := bounded_support_max N c hN ⟨m₀ 0, hc0⟩
  refine ⟨mon n, ?_, ?_⟩
  · rw [pureNat_mon]; exact hn
  · intro m' hm'
    obtain ⟨hc', hpure'⟩ := pureNat_apply hm'
    have hle : m' 0 ≤ n := hmaxn (m' 0) hc'
    intro j
    by_cases hj : j = 0
    · subst hj
      show m' 0 ≤ mon n 0
      rw [mon_zero]
      exact hle
    · have h1 : 1 ≤ j := by omega
      rw [hpure' j h1]
      exact Nat.zero_le _

/-- **T7(a) 具體實例（非空洞性）**：在單變量模型上，兩個互約化、同首項的
理想元素相等——把主定理的假設在此模型上兌現（`hmax` 由 `hmax_pureNat` 提供）。 -/
theorem reduced_unique_univariate (hI : IsIdeal I) {μ : MonoExp} {c₁ c₂ : Nat → Int}
    {N : Nat} (_hN₁ : ∀ n, N ≤ n → c₁ n = 0) (_hN₂ : ∀ n, N ≤ n → c₂ n = 0)
    (hfI : I (pureNat c₁)) (hgI : I (pureNat c₂))
    (hf : Interreduced (leadIdeal I) μ (pureNat c₁))
    (hg : Interreduced (leadIdeal I) μ (pureNat c₂))
    (hcoeff : pureNat c₁ μ = pureNat c₂ μ)
    (hmax : HasMaxMono (subP (pureNat c₁) (pureNat c₂))) :
    pureNat c₁ = pureNat c₂ :=
  reduced_unique hI hfI hgI hf hg hcoeff hmax

end Polyrust
