-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 子句代數：消解、學習子句與模型守恆（定理 T3(b) + CDCL 學習）

對應 docs/THEOREMS.md §5（T3）：

> (b) CDCL 學習子句 C 的多項式 P_C ∈ ⟨P_Φ ∪ B⟩。

原文的證明用「布爾理想是根式理想 + Nullstellensatz」。本模組做的是**更強的、
完全構造性的一步**：CDCL 的學習子句是由**消解（resolution）**逐步導出的，
而消解對多項式編碼是**逐點恆等式**：

  C = (l ∨ C₁),  D = (¬l ∨ C₂),  消解式 R = C₁ ∨ C₂  ⟹
  **P_R(σ) = P_C(σ)·P_{C₂}(σ) + P_{C₁}(σ)·P_D(σ)**   （∀σ，無需布爾約束）

（`resolution_identity`）。也就是說：學習子句的多項式不是「碰巧在簇上為零」，
而是**父母子句多項式的顯式線性組合**——這是 T3(b) 的構造性、可檢查版本，
比 Nullstellensatz 論證提供更多的資訊（見證係數被寫出來了）。

由此得到 CDCL 迴圈的兩個不變式：

* `learned_poly_vanishes`：學習得到的子句在 Φ 的每個模型上多項式歸零；
* `learned_preserves_models`：把被蘊涵的子句加入 Φ **不改變模型集**
  ——即學習步既不放走 SAT 解，也不引入假解（UNSAT 側的可靠性）。

**邊界**：這裡證明的是「學習子句的正確性」（CDCL 的推理規則層面），
不是 CDCL 演算法本身的終止性/完備性（那需要 DPLL 的搜索樹論證與
單位傳播的形式化，本專案未做；見 `docs/LEAN_FORMALIZATION.md` 邊界表）。 -/

import Polyrust.ClauseDuality
import Polyrust.Tactics

namespace Polyrust

/-! ## 文字的補與因子 -/

/-- 文字的否定：同一變量、相反極性。 -/
def Lit.neg (l : Lit) : Lit := ⟨l.var, !l.pos⟩

theorem Lit.neg_neg (l : Lit) : l.neg.neg = l := by
  cases l with
  | mk v p => cases p <;> rfl

theorem Lit.neg_var (l : Lit) : l.neg.var = l.var := rfl

/-- 文字與其補：恰有一個為真（布爾互補律）。 -/
theorem litSat_neg (σ : Assignment) (l : Lit) :
    litSat σ l.neg = !litSat σ l := by
  cases l with
  | mk v p => cases p <;> simp [litSat, Lit.neg]

/-- 布爾取值的「0/1 互補」：`bit b + bit (!b) = 1`。 -/
theorem bit_add_bit_not (b : Bool) : bit b + bit (!b) = 1 := by
  cases b <;> rfl

/-- 補文字的因子 = 原文字的 0/1 取值：`1 − (1 − a) = a`。 -/
theorem litFactor_neg (σ : Assignment) (l : Lit) :
    litFactor σ l.neg = bit (litSat σ l) := by
  unfold litFactor
  rw [litSat_neg]
  cases h : litSat σ l <;> simp [bit]

/-- 因子與其補因子之和為 1（消解恆等式的代數核心）。 -/
theorem litFactor_add_neg (σ : Assignment) (l : Lit) :
    litFactor σ l + litFactor σ l.neg = 1 := by
  rw [litFactor_neg]
  unfold litFactor
  have := bit_add_bit_not (litSat σ l)
  omega

/-! ## 子句多項式的乘積性與消解 -/

/-- 乘積折疊對串接的乘性（純列表/環事實）。 -/
theorem foldr_mul_append (l₁ l₂ : List Int) :
    (l₁ ++ l₂).foldr (fun a b => a * b) 1 = l₁.foldr (fun a b => a * b) 1 * l₂.foldr (fun a b => a * b) 1 := by
  induction l₁ with
  | nil => simp
  | cons a l ih =>
    rw [List.cons_append, List.foldr_cons, List.foldr_cons, ih, Int.mul_assoc]

/-- 子句多項式對子句串接是乘性的：`P_{C₁ ++ C₂} = P_{C₁} · P_{C₂}`。 -/
theorem clausePoly_append (σ : Assignment) (C₁ C₂ : List Lit) :
    clausePoly σ (C₁ ++ C₂) = clausePoly σ C₁ * clausePoly σ C₂ := by
  rw [clausePoly, clausePoly, clausePoly, List.map_append, foldr_mul_append]

theorem clausePoly_cons (σ : Assignment) (l : Lit) (C : List Lit) :
    clausePoly σ (l :: C) = litFactor σ l * clausePoly σ C := rfl

theorem litFactor_eq_one_sub (σ : Assignment) (l : Lit) :
    litFactor σ l = 1 - bit (litSat σ l) := rfl

/-- **T3(b) 消解恆等式（逐點，構造性）**：
對任意賦值 σ（甚至不需 σ 是 0/1，因為因子已是 0/1 值），
消解式的多項式等於兩個父母子句多項式的顯式組合：

  P_{C₁∨C₂} = P_{l∨C₁}·P_{C₂} + P_{C₁}·P_{¬l∨C₂}

因此 CDCL 學習子句的多項式**逐點**落在 ⟨P_{l∨C₁}, P_{¬l∨C₂}⟩ 中——
這是 T3(b) 的無條件版本（無須 Nullstellensatz）。 -/
theorem resolution_identity (σ : Assignment) (l : Lit) (C₁ C₂ : List Lit) :
    clausePoly σ (C₁ ++ C₂)
      = clausePoly σ (l :: C₁) * clausePoly σ C₂
        + clausePoly σ C₁ * clausePoly σ (l.neg :: C₂) := by
  rw [clausePoly_append, clausePoly_cons, clausePoly_cons, litFactor_eq_one_sub,
      litFactor_neg]
  int_ring

/-- 消解式的布爾側：σ 同時滿足兩個父母子句 ⟹ σ 滿足消解式。 -/
theorem resolvent_sat {σ : Assignment} {l : Lit} {C₁ C₂ : List Lit}
    (h₁ : clauseSat σ (l :: C₁) = true) (h₂ : clauseSat σ (l.neg :: C₂) = true) :
    clauseSat σ (C₁ ++ C₂) = true := by
  refine (clause_duality σ (C₁ ++ C₂)).mpr ?_
  have hz₁ : clausePoly σ (l :: C₁) = 0 := (clause_duality σ (l :: C₁)).mp h₁
  have hz₂ : clausePoly σ (l.neg :: C₂) = 0 := (clause_duality σ (l.neg :: C₂)).mp h₂
  rw [resolution_identity σ l C₁ C₂, hz₁, hz₂]
  simp

/-- **T3(b) 多項式版**：若 σ 是 Φ 中兩個父母子句的「多項式零點」，
則 σ 也是消解式的零點——即學習步不會破壞既有解的歸零性質。 -/
theorem resolution_poly_zero {σ : Assignment} {l : Lit} {C₁ C₂ : List Lit}
    (h₁ : clausePoly σ (l :: C₁) = 0) (h₂ : clausePoly σ (l.neg :: C₂) = 0) :
    clausePoly σ (C₁ ++ C₂) = 0 := by
  rw [resolution_identity σ l C₁ C₂, h₁, h₂]
  simp

/-! ## T3(a) 的衍生：因子結構與特殊子句 -/

/-- 因子是 0 或 1（子句多項式因此是 0/1 值函數）。 -/
theorem litFactor_eq_zero_or_one (σ : Assignment) (l : Lit) :
    litFactor σ l = 0 ∨ litFactor σ l = 1 := by
  unfold litFactor
  cases h : litSat σ l <;> simp [bit]

/-- 衍生：子句多項式的值恰為 0 或 1。 -/
theorem clausePoly_eq_zero_or_one (σ : Assignment) (C : List Lit) :
    clausePoly σ C = 0 ∨ clausePoly σ C = 1 := by
  induction C with
  | nil => right; rfl
  | cons l C ih =>
    rw [clausePoly_cons]
    rcases litFactor_eq_zero_or_one σ l with h | h <;>
      rcases ih with h' | h' <;> rw [h, h'] <;> simp

/-- 衍生：子句多項式歸零 ⟺ 某文字滿足（重述 `clause_duality`，
但以「因子為零」的形式給出，便於後續模組引用）。 -/
theorem clausePoly_zero_iff_exists (σ : Assignment) (C : List Lit) :
    clausePoly σ C = 0 ↔ ∃ l ∈ C, litSat σ l = true := by
  constructor
  · intro h
    have hsat : clauseSat σ C = true := (clause_duality σ C).mpr h
    rw [clauseSat] at hsat
    exact List.any_eq_true.mp hsat
  · rintro ⟨l, hl, hl'⟩
    exact (clause_duality σ C).mp (List.any_eq_true.mpr ⟨l, hl, hl'⟩)

/-- 空子句的多項式是常數 1：它「永不歸零」——U N S A T 的算術面貌。 -/
theorem clausePoly_nil : clausePoly (σ : Assignment) [] = 1 := rfl

theorem clausePoly_nil_ne_zero (σ : Assignment) : clausePoly σ ([] : List Lit) ≠ 0 := by
  rw [clausePoly_nil]; decide

/-- 重言子句（同時含 l 與 ¬l）的多項式恆為 0，即它不施加任何約束。 -/
theorem clausePoly_tautology (σ : Assignment) (l : Lit) (C D : List Lit) :
    clausePoly σ (l :: (C ++ l.neg :: D)) = 0 := by
  refine (clausePoly_zero_iff_exists σ _).mpr ?_
  cases h : litSat σ l with
  | false =>
    refine ⟨l.neg, ?_, ?_⟩
    · simp
    · rw [litSat_neg, h]; rfl
  | true => exact ⟨l, by simp, h⟩

/-! ## CNF 層：模型集與學習步的守恆 -/

/-- CNF 模型集：σ 滿足 Φ 的所有子句。 -/
def Mod (σ : Assignment) (Φ : List (List Lit)) : Prop := cnfSat σ Φ = true

/-- CNF 的多項式零點集：Φ 的全部子句多項式在 σ 處歸零。 -/
def PolyZero (σ : Assignment) (Φ : List (List Lit)) : Prop :=
  ∀ p ∈ cnfPolys σ Φ, p = 0

/-- **T3(a) 全式的全域形式（重述 + 命名）**：模型集 = 多項式零點集。 -/
theorem mod_iff_polyZero (σ : Assignment) (Φ : List (List Lit)) :
    Mod σ Φ ↔ PolyZero σ Φ :=
  cnf_duality σ Φ

/-- 子句被 CNF 蘊涵：Φ 的每個模型都滿足 C。 -/
def Entails (Φ : List (List Lit)) (C : List Lit) : Prop :=
  ∀ σ, cnfSat σ Φ = true → clauseSat σ C = true

/-- **T3(b) 值層版本**：被 Φ 蘊涵的子句，在 Φ 的每個多項式零點上多項式歸零。 -/
theorem entails_poly_vanishes {Φ : List (List Lit)} {C : List Lit} (h : Entails Φ C) :
    ∀ σ, PolyZero σ Φ → clausePoly σ C = 0 := by
  intro σ hσ
  have hmod : cnfSat σ Φ = true := (mod_iff_polyZero σ Φ).mpr hσ
  exact (clause_duality σ C).mp (h σ hmod)

/-- 消解式被父母子句蘊涵（布爾側的單步推理）。 -/
theorem entails_resolvent (Φ : List (List Lit)) {l : Lit} {C₁ C₂ : List Lit}
    (h₁ : (l :: C₁) ∈ Φ) (h₂ : (l.neg :: C₂) ∈ Φ) : Entails Φ (C₁ ++ C₂) := by
  intro σ hmod
  have h1 : clauseSat σ (l :: C₁) = true := List.all_eq_true.mp hmod _ h₁
  have h2 : clauseSat σ (l.neg :: C₂) = true := List.all_eq_true.mp hmod _ h₂
  exact resolvent_sat h1 h2

/-- **CDCL 學習步的模型守恆（T3(b) 的演算法後果）**：
若 C 被 Φ 蘊涵，則把 C 加入 Φ 不改變模型集。 -/
theorem learned_preserves_models {Φ : List (List Lit)} {C : List Lit}
    (h : Entails Φ C) (σ : Assignment) :
    (cnfSat σ (C :: Φ) = true) ↔ (cnfSat σ Φ = true) := by
  constructor
  · intro h'
    rw [cnfSat, List.all_eq_true]
    intro D hD
    exact List.all_eq_true.mp h' D (List.mem_cons_of_mem _ hD)
  · intro h'
    rw [cnfSat, List.all_eq_true]
    intro D hD
    simp only [List.mem_cons] at hD
    rcases hD with hDC | hDΦ
    · rw [hDC]; exact h σ h'
    · exact List.all_eq_true.mp h' D hDΦ

/-- 衍生：學習步同時守恆多項式零點集（代數側的同一事實）。 -/
theorem learned_preserves_polyZero {Φ : List (List Lit)} {C : List Lit}
    (h : Entails Φ C) (σ : Assignment) :
    PolyZero σ (C :: Φ) ↔ PolyZero σ Φ :=
  (mod_iff_polyZero σ (C :: Φ)).symm.trans
    ((learned_preserves_models h σ).trans (mod_iff_polyZero σ Φ))

/-- 衍生（單調性）：加入任意子句只會**縮小**模型集。 -/
theorem models_mono (σ : Assignment) {Φ : List (List Lit)} {C : List Lit}
    (h : cnfSat σ (C :: Φ) = true) : cnfSat σ Φ = true := by
  rw [cnfSat, List.all_eq_true]
  intro D hD
  exact List.all_eq_true.mp h D (List.mem_cons_of_mem _ hD)

/-- 衍生：若 Φ 含空子句，則 Φ 不可滿足（多項式側：常數 1 出現 ⇒ 零點集空）。 -/
theorem unsat_of_nil_mem {Φ : List (List Lit)} (h : ([] : List Lit) ∈ Φ) :
    ∀ σ, cnfSat σ Φ = false := by
  intro σ
  cases hval : cnfSat σ Φ with
  | false => rfl
  | true =>
    have hnil := List.all_eq_true.mp hval _ h
    rw [clauseSat, List.any_nil] at hnil
    exact absurd hnil (by simp)

/-- 衍生：Φ 不可滿足 ⟺ 不存在多項式零點（T3(a) 的逆否形式）。 -/
theorem unsat_iff_no_polyZero (Φ : List (List Lit)) :
    (∀ σ, cnfSat σ Φ = false) ↔ (∀ σ, ¬ PolyZero σ Φ) := by
  constructor
  · intro h σ hz
    have : cnfSat σ Φ = true := (mod_iff_polyZero σ Φ).mpr hz
    rw [h σ] at this
    exact absurd this (by simp)
  · intro h σ
    cases hval : cnfSat σ Φ
    · rfl
    · exact absurd ((mod_iff_polyZero σ Φ).mp hval) (h σ)

end Polyrust
