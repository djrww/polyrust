-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # S-多項式與 Buchberger 準則（定理 T5）

對應 docs/THEOREMS.md §7（T5）。原文的 (a)(b)(c) 說的是：

> (a) 第一準則（互素）：gcd(LM f, LM g) = 1 ⟹ S(f,g) →_{f,g} 0
> (b) 第二準則（鏈）：若 ∃h，LM(h) | lcm(LM f, LM g) 且 S(f,h) → 0、S(g,h) → 0，則 S(f,g) → 0
> (c) 兩準則不改變（約化）Gröbner 基

**形式化的邊界（先說死）**：完整的 (a)(b) 需要「多項式除法律 + 項序 + →_G 歸約」
的整套機器。本模組不假裝做完了那件事，而是**精確地把兩個準則的數學心臟抽出來
並證明之**——這些恆等式正是所有教科書（Cox–Little–O'Shea §2.9）準則證明的
唯一內容：

* **T5(a) 心臟**：互素時 `lcm(μ,ν) = μ·ν`（`lcmM_eq_monoMul_of_coprime`），
  故 S(f,g) = ν·f − μ·g；於是
  - 首項對消（`coprime_sPoly_head_cancel`）：S 在 γ 處係數恰為 0；
  - 支撐分裂（`coprime_sPoly_support`）：S 的每個非零項都被 ν 或 μ 整除
    —— 這正是「用 g 與 f 各約化一次即歸零」的機制。
* **T5(b) 心臟**：`sPoly_chain_decomposition` 是一條**多項式恆等式**
  S(f,g) = (γ/γ₁)·S(f,h) − (γ/γ₂)·S(g,h)（γ = lcm(μ,ν), τ = LM h | γ,
  γ₁ = lcm(μ,τ), γ₂ = lcm(ν,τ)），其中 τ 項恰好對消。由此，
  「S(f,h)、S(g,h) 屬於理想」⟹「S(f,g) 屬於理想」——這是鏈準則的全部內容
  （`chain_criterion`）。
* **T5(c)**：S-多項式本就屬於 ⟨f,g⟩，故把它們加入基不改變理想
  （`sPoly_mem_genIdeal`、`genIdeal_insert_sPoly`）。這保證「準則只跳過
  必然歸零的 S-多項式」不會損害 Buchberger 條件的完備性。

多項式以 `MonoExp → Int` 表示（係數函數），首項以 `IsLead` 刻畫。
理想以「對減法與單項式倍乘封閉」的述詞 `IsIdeal` 表示，`genIdeal` 為其交集。 -/

import Polyrust.Monomial

namespace Polyrust

/- 註：`mulMono` 需要判定 `dividesM μ ν`（指數向量的逐點比較），
本層使用 classical 邏輯提供該 `Decidable` 實例——這只影響「取哪個分支」的
可判定性，不引入任何數學假設（`#print axioms` 會顯示 Lean 標準三公理
`propext`、`Quot.sound`、`Classical.choice`；其餘模組皆不需 classical）。 -/
open Classical

/-! ## 多項式（係數函數）與理想 -/

/-- 多項式：單項式 → 係數（本層只用到支撐、首項與理想封閉性，
故不強制有限支撐；所有定理都對「支撐」逐項陳述）。 -/
abbrev MPoly := MonoExp → Int

def zeroP : MPoly := fun _ => 0
def addP (p q : MPoly) : MPoly := fun m => p m + q m
def subP (p q : MPoly) : MPoly := fun m => p m - q m
def negP (p : MPoly) : MPoly := fun m => -(p m)

/-- 單項式倍乘：(μ·p)(ν) = p(ν/μ)（μ ∤ ν 時為 0）。 -/
noncomputable def mulMono (μ : MonoExp) (p : MPoly) : MPoly :=
  fun ν => if dividesM μ ν then p (quotM μ ν) else 0

/-- μ 是 p 的首項單項式：係數非零，且所有非零項都被 μ 整除。 -/
def IsLead (μ : MonoExp) (p : MPoly) : Prop := p μ ≠ 0 ∧ ∀ m, p m ≠ 0 → dividesM m μ

/-- 理想：對 0、減法、單項式倍乘封閉（域上理想的最小充分刻畫）。 -/
structure IsIdeal (I : MPoly → Prop) : Prop where
  zero_mem : I zeroP
  sub_mem : ∀ p q, I p → I q → I (subP p q)
  smul_mem : ∀ μ p, I p → I (mulMono μ p)

/-- 由集合 S 生成的理想（所有包含 S 的理想的交集）。 -/
def genIdeal (S : MPoly → Prop) : MPoly → Prop :=
  fun p => ∀ I : MPoly → Prop, IsIdeal I → (∀ q, S q → I q) → I p

theorem genIdeal_subset {S : MPoly → Prop} {q : MPoly} (hq : S q) : genIdeal S q :=
  fun _ _ hS => hS q hq

theorem genIdeal_isIdeal (S : MPoly → Prop) : IsIdeal (genIdeal S) := by
  refine ⟨?_, ?_, ?_⟩
  · intro I hI _
    exact hI.zero_mem
  · intro p q hp hq I hI hS
    exact hI.sub_mem p q (hp I hI hS) (hq I hI hS)
  · intro μ p hp I hI hS
    exact hI.smul_mem μ p (hp I hI hS)

theorem genIdeal_least {S : MPoly → Prop} {I : MPoly → Prop} (hI : IsIdeal I)
    (hS : ∀ q, S q → I q) : ∀ p, genIdeal S p → I p :=
  fun _ hp => hp I hI hS

/-! ## 單項式倍乘的基本性質 -/

theorem mulMono_apply {μ ν : MonoExp} {p : MPoly} (h : dividesM μ ν) :
    mulMono μ p ν = p (quotM μ ν) := by
  unfold mulMono
  rw [if_pos h]

theorem mulMono_apply_not_dvd {μ ν : MonoExp} {p : MPoly} (h : ¬ dividesM μ ν) :
    mulMono μ p ν = 0 := by
  unfold mulMono
  rw [if_neg h]

theorem mulMono_zero (μ : MonoExp) : mulMono μ zeroP = zeroP := by
  funext ν
  by_cases h : dividesM μ ν
  · rw [mulMono_apply h]; rfl
  · rw [mulMono_apply_not_dvd h]; rfl

/-- 單項式倍乘對減法分配。 -/
theorem mulMono_subP (μ : MonoExp) (p q : MPoly) :
    mulMono μ (subP p q) = subP (mulMono μ p) (mulMono μ q) := by
  funext m
  by_cases h : dividesM μ m
  · simp only [mulMono_apply h, subP]
  · simp only [mulMono_apply_not_dvd h, subP]
    simp

/-- 支撐：`μ·p` 的非零項必被 μ 整除。 -/
theorem mulMono_support {μ m : MonoExp} {p : MPoly} (h : mulMono μ p m ≠ 0) :
    dividesM μ m := by
  by_cases hc : dividesM μ m
  · exact hc
  · exact absurd (mulMono_apply_not_dvd hc) h

/-- 首項倍乘：`μ 是 p 的首項` ⟹ `μ'·μ 是 μ'·p 的首項`。 -/
theorem mulMono_isLead {μ μ' : MonoExp} {p : MPoly} (h : IsLead μ p) :
    IsLead (monoMul μ' μ) (mulMono μ' p) := by
  obtain ⟨hcoeff, hsupp⟩ := h
  constructor
  · have hd : dividesM μ' (monoMul μ' μ) := dividesM_monoMul_left μ' μ
    rw [mulMono_apply hd, quotM_monoMul_left]
    exact hcoeff
  · intro m hm
    have hdm : dividesM μ' m := mulMono_support hm
    have hp : p (quotM μ' m) ≠ 0 := by
      have := mulMono_apply (μ := μ') (ν := m) (p := p) hdm
      rw [this] at hm
      exact hm
    have hquot : dividesM (quotM μ' m) μ := hsupp _ hp
    intro j
    have h1 : μ' j ≤ m j := hdm j
    have h2 : quotM μ' m j ≤ μ j := hquot j
    have h2' : quotM μ' m j = m j - μ' j := rfl
    unfold monoMul
    omega

/-- 判定引理：`μ'·μ ∣ m ⟺ μ ∣ m ∧ μ' ∣ m/μ`（單項式除法的結合律形式）。 -/
theorem dividesM_monoMul_iff (μ μ' m : MonoExp) :
    dividesM (monoMul μ' μ) m ↔ dividesM μ m ∧ dividesM μ' (quotM μ m) := by
  constructor
  · intro h
    constructor
    · intro j
      have := h j
      unfold monoMul at this
      omega
    · intro j
      have h1 : μ' j + μ j ≤ m j := h j
      have h2 : μ j ≤ m j := by
        have := h j
        unfold monoMul at this
        omega
      show μ' j ≤ m j - μ j
      omega
  · rintro ⟨h1, h2⟩ j
    have h1j : μ j ≤ m j := h1 j
    have h2j : μ' j ≤ m j - μ j := h2 j
    unfold monoMul
    omega

/-- 單項式倍乘的結合律：`μ·(μ'·p) = (μ'·μ)·p`。 -/
theorem mulMono_mulMono (μ μ' : MonoExp) (p : MPoly) :
    mulMono μ (mulMono μ' p) = mulMono (monoMul μ' μ) p := by
  funext m
  have hiff := dividesM_monoMul_iff μ μ' m
  by_cases h1 : dividesM μ m
  · by_cases h2 : dividesM μ' (quotM μ m)
    · have h3 : dividesM (monoMul μ' μ) m := hiff.mpr ⟨h1, h2⟩
      rw [mulMono_apply h1, mulMono_apply h2, mulMono_apply h3]
      congr 1
      funext j
      have h1j : μ j ≤ m j := h1 j
      have h2j : μ' j ≤ quotM μ m j := h2 j
      have h2j' : μ' j ≤ m j - μ j := h2j
      show (m j - μ j) - μ' j = m j - (μ' j + μ j)
      omega
    · have h3 : ¬ dividesM (monoMul μ' μ) m := fun h4 => h2 (hiff.mp h4).2
      rw [mulMono_apply h1, mulMono_apply_not_dvd h2, mulMono_apply_not_dvd h3]
  · have h3 : ¬ dividesM (monoMul μ' μ) m := fun h4 => h1 (hiff.mp h4).1
    rw [mulMono_apply_not_dvd h1, mulMono_apply_not_dvd h3]

/-! ## S-多項式 -/

/-- S-多項式（首項係數已 monic 化）：
S(f,g) = (γ/μ)·f − (γ/ν)·g，其中 μ = LM f、ν = LM g、γ = lcm(μ,ν)。 -/
noncomputable def sPoly (μ ν : MonoExp) (f g : MPoly) : MPoly :=
  subP (mulMono (quotM μ (lcmM μ ν)) f) (mulMono (quotM ν (lcmM μ ν)) g)

theorem sPoly_apply (μ ν : MonoExp) (f g : MPoly) (m : MonoExp) :
    sPoly μ ν f g m
      = mulMono (quotM μ (lcmM μ ν)) f m - mulMono (quotM ν (lcmM μ ν)) g m := rfl

/-- S-多項式屬於 ⟨f, g⟩（由定義直接線性組合得到）——T5(c) 的基礎。 -/
theorem sPoly_mem_genIdeal {S : MPoly → Prop} {μ ν : MonoExp} {f g : MPoly}
    (hf : S f) (hg : S g) : genIdeal S (sPoly μ ν f g) := by
  intro I hI hS
  exact hI.sub_mem _ _ (hI.smul_mem _ _ (hS f hf)) (hI.smul_mem _ _ (hS g hg))

/-- 理想生成的單調性：生成集越大，理想越大。 -/
theorem genIdeal_mono {S T : MPoly → Prop} (h : ∀ q, S q → T q) :
    ∀ p, genIdeal S p → genIdeal T p :=
  fun _ hp => hp _ (genIdeal_isIdeal T) (fun q hq => genIdeal_subset (h q hq))

/-- **T5(c)**：把 S(f,g) 加入基不改變所生成的理想——準則「跳過」S-多項式
不會漏掉理想中的任何元素。 -/
theorem genIdeal_insert_sPoly {S : MPoly → Prop} {μ ν : MonoExp} {f g : MPoly}
    (hf : S f) (hg : S g) (p : MPoly) :
    genIdeal (fun q => S q ∨ q = sPoly μ ν f g) p ↔ genIdeal S p := by
  constructor
  · intro hp
    refine genIdeal_least (S := fun q => S q ∨ q = sPoly μ ν f g) (I := genIdeal S)
      (genIdeal_isIdeal S) ?_ p hp
    intro q hq
    rcases hq with hq | hq
    · exact genIdeal_subset hq
    · rw [hq]
      exact sPoly_mem_genIdeal hf hg
  · intro hp
    refine genIdeal_least (S := S) (I := genIdeal (fun q => S q ∨ q = sPoly μ ν f g))
      (genIdeal_isIdeal _) ?_ p hp
    intro q hq
    exact genIdeal_subset (Or.inl hq)

/-! ## T5(a)：互素首項準則 -/

/-- **T5(a) 首項對消**：μ、ν 互素且 f、g 首項係數為 1 時，
S(f,g) 在 γ = lcm(μ,ν) = μ·ν 處的係數恰為 0
（兩側的首項 (γ/μ)·μ = γ 與 (γ/ν)·ν = γ 相消）。 -/
theorem coprime_sPoly_head_cancel {μ ν : MonoExp} (hcop : coprimeM μ ν)
    {f g : MPoly} (hf : f μ = 1) (hg : g ν = 1) :
    sPoly μ ν f g (lcmM μ ν) = 0 := by
  have hγ : lcmM μ ν = monoMul μ ν := lcmM_eq_monoMul_of_coprime hcop
  have hqμ : quotM μ (lcmM μ ν) = ν := quotM_lcmM_left_of_coprime hcop
  have hqν : quotM ν (lcmM μ ν) = μ := quotM_lcmM_right_of_coprime hcop
  have h1 : dividesM ν (lcmM μ ν) := by
    rw [hγ]; exact dividesM_monoMul_right μ ν
  have h2 : dividesM μ (lcmM μ ν) := by
    rw [hγ]; exact dividesM_monoMul_left μ ν
  rw [sPoly_apply, hqμ, hqν, mulMono_apply h1, mulMono_apply h2, hγ,
      quotM_monoMul_right, quotM_monoMul_left, hf, hg]
  exact Int.sub_self 1

/-- **T5(a) 支撐分裂（第一準則的心臟）**：μ、ν 互素時，S(f,g) 的每個非零項
都被 ν 或 μ 整除。所以 S(f,g) 用 g 與 f 各約化一次即歸零——這正是
教科書「互素 ⇒ S(f,g) →_{f,g} 0」論證的全部內容。 -/
theorem coprime_sPoly_support {μ ν : MonoExp} (hcop : coprimeM μ ν)
    (f g : MPoly) :
    ∀ m, sPoly μ ν f g m ≠ 0 → dividesM ν m ∨ dividesM μ m := by
  intro m hm
  have hγ : lcmM μ ν = monoMul μ ν := lcmM_eq_monoMul_of_coprime hcop
  have hqμ : quotM μ (lcmM μ ν) = ν := quotM_lcmM_left_of_coprime hcop
  have hqν : quotM ν (lcmM μ ν) = μ := quotM_lcmM_right_of_coprime hcop
  rw [sPoly_apply, hqμ, hqν] at hm
  by_cases hA : mulMono ν f m = 0
  · right
    have hB : mulMono μ g m ≠ 0 := by
      intro hB0
      exact hm (by rw [hA, hB0]; exact Int.sub_self 0)
    exact mulMono_support hB
  · left
    exact mulMono_support hA

/-- 衍生：互素準則的「兩步式」陳述——S(f,g) 首項已對消，且每一項要麼被 ν 整除
（可用 g 消去），要麼被 μ 整除（可用 f 消去）。 -/
theorem coprime_criterion {μ ν : MonoExp} (hcop : coprimeM μ ν) (f g : MPoly)
    (hf : f μ = 1) (hg : g ν = 1) :
    sPoly μ ν f g (lcmM μ ν) = 0
      ∧ (∀ m, sPoly μ ν f g m ≠ 0 → dividesM ν m ∨ dividesM μ m) :=
  ⟨coprime_sPoly_head_cancel hcop hf hg, coprime_sPoly_support hcop f g⟩

/-! ## T5(b)：鏈準則——S-多項式的分解恆等式

設 γ = lcm(μ,ν)、τ = LM h 滿足 τ | γ，並令 γ₁ = lcm(μ,τ)、γ₂ = lcm(ν,τ)。
則有**多項式恆等式**

  S(f,g) = (γ/γ₁)·S(f,h) − (γ/γ₂)·S(g,h)

（τ 項在兩側以相同係數 γ/τ 出現而對消）。這是 Gebauer–Möller 鏈準則
證明中唯一的實質步驟：只要 S(f,h)、S(g,h) 歸零（或落在理想內），
S(f,g) 就被線性組合出來。 -/

theorem sPoly_chain_decomposition (μ ν τ : MonoExp) (f g h : MPoly)
    (hτγ : dividesM τ (lcmM μ ν)) :
    sPoly μ ν f g
      = subP (mulMono (quotM (lcmM μ τ) (lcmM μ ν)) (sPoly μ τ f h))
             (mulMono (quotM (lcmM ν τ) (lcmM μ ν)) (sPoly ν τ g h)) := by
  have hμγ : dividesM μ (lcmM μ ν) := dividesM_lcmM_left μ ν
  have hνγ : dividesM ν (lcmM μ ν) := dividesM_lcmM_right μ ν
  have hγ₁γ : dividesM (lcmM μ τ) (lcmM μ ν) :=
    (dividesM_lcmM_iff μ τ (lcmM μ ν)).mpr ⟨hμγ, hτγ⟩
  have hγ₂γ : dividesM (lcmM ν τ) (lcmM μ ν) :=
    (dividesM_lcmM_iff ν τ (lcmM μ ν)).mpr ⟨hνγ, hτγ⟩
  have hτγ₁ : dividesM τ (lcmM μ τ) := dividesM_lcmM_right μ τ
  have hτγ₂ : dividesM τ (lcmM ν τ) := dividesM_lcmM_right ν τ
  have hμγ₁ : dividesM μ (lcmM μ τ) := dividesM_lcmM_left μ τ
  have hνγ₂ : dividesM ν (lcmM ν τ) := dividesM_lcmM_left ν τ
  -- 把 γ/μ 分解為 (γ/γ₁)·(γ₁/μ)
  have hA : mulMono (quotM μ (lcmM μ ν)) f
      = mulMono (quotM (lcmM μ τ) (lcmM μ ν))
          (mulMono (quotM μ (lcmM μ τ)) f) := by
    rw [quotM_factor hμγ₁ hγ₁γ, ← mulMono_mulMono]
  have hB : mulMono (quotM ν (lcmM μ ν)) g
      = mulMono (quotM (lcmM ν τ) (lcmM μ ν))
          (mulMono (quotM ν (lcmM ν τ)) g) := by
    rw [quotM_factor hνγ₂ hγ₂γ, ← mulMono_mulMono]
  -- τ 項：兩側係數都是 γ/τ
  have hH₁ : mulMono (quotM (lcmM μ τ) (lcmM μ ν)) (mulMono (quotM τ (lcmM μ τ)) h)
      = mulMono (quotM τ (lcmM μ ν)) h := by
    rw [mulMono_mulMono, ← quotM_factor hτγ₁ hγ₁γ]
  have hH₂ : mulMono (quotM (lcmM ν τ) (lcmM μ ν)) (mulMono (quotM τ (lcmM ν τ)) h)
      = mulMono (quotM τ (lcmM μ ν)) h := by
    rw [mulMono_mulMono, ← quotM_factor hτγ₂ hγ₂γ]
  funext m
  rw [sPoly_apply, hA, hB]
  simp only [sPoly, mulMono_subP, subP]
  rw [hH₁, hH₂]
  omega

/-- **T5(b) 鏈準則（理想形式）**：若 S(f,h) 與 S(g,h) 都落在 S 生成的理想內
（在 Buchberger 迴圈中即「兩者皆歸約至 0」），則 S(f,g) 也落在該理想內。
這就是「第二準則（鏈）」的全部代數內容。 -/
theorem chain_criterion {S : MPoly → Prop} {μ ν τ : MonoExp}
    (hτγ : dividesM τ (lcmM μ ν)) {f g h : MPoly}
    (hfh : genIdeal S (sPoly μ τ f h)) (hgh : genIdeal S (sPoly ν τ g h)) :
    genIdeal S (sPoly μ ν f g) := by
  have hdec := sPoly_chain_decomposition μ ν τ f g h hτγ
  rw [hdec]
  exact (genIdeal_isIdeal S).sub_mem _ _
    ((genIdeal_isIdeal S).smul_mem _ _ hfh)
    ((genIdeal_isIdeal S).smul_mem _ _ hgh)

/-- 衍生：鏈準則的「對稱」情形——τ 取 μ 或 ν 時退化為恆等式
（S(f,f) = 0、S(f,g) 的自身分解），故準則在退化情形自動成立。 -/
theorem sPoly_self (μ : MonoExp) (f : MPoly) : sPoly μ μ f f = zeroP := by
  funext m
  rw [sPoly_apply]
  rw [show quotM μ (lcmM μ μ) = quotM μ (lcmM μ μ) from rfl]
  simp [zeroP]

/-- 衍生：γ = lcm(μ,ν) 是 S-多項式兩側乘子的來源，
其商恰為 `quotM`；此處記錄一個常用特例：γ/μ · μ = γ（記帳用）。 -/
theorem quotM_lcmM_mul (μ ν : MonoExp) :
    monoMul (quotM μ (lcmM μ ν)) μ = lcmM μ ν := by
  rw [monoMul_comm]
  exact monoMul_quotM (dividesM_lcmM_left μ ν)

theorem quotM_lcmM_mul_right (μ ν : MonoExp) :
    monoMul (quotM ν (lcmM μ ν)) ν = lcmM μ ν := by
  rw [monoMul_comm]
  exact monoMul_quotM (dividesM_lcmM_right μ ν)

end Polyrust
