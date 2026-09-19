-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 型別宇宙泛化（定理 T9 的一般化）：任意可枚舉型別宇宙上的端到端正確性

對應 `docs/THEOREMS.md` §11（T9），但把 `T9EndToEnd.lean` 裡寫死的
2 型別（`i32`/`boolean`）泛化為任意可枚舉型別宇宙：

* 型別宇宙 = 任意型別 `Ty`，配上全枚舉 `enumAll : List Ty`（無重複、完備）；
* 語言保留 4 個建構子（`num`/`add`/`eqb`/`ite`），但把「數值型別」與
  「布爾型別」從寫死的 `i32`/`boolean` 泛化為宇宙中任意兩個互異成員
  `numTy`/`eqbTy`；
* one-hot 從 `t_{e,i32} + t_{e,bool} − 1 = 0` 泛化為
  `(Σ_{t ∈ enumAll} t_{e,t}) − 1 = 0`，求和用 `List.sum`（Lean core 內建，
  不依賴 Mathlib 的 `Finset`/`Fintype`）。

**這不是修辭**：一旦本模組的定理成立，Rust 側新增型別時無需重寫任何證明——
只要把新宇宙的 `enumAll`/`nodup`/`complete`/`numTy`/`eqbTy`/`num_ne_eqb`
六個參數代入即可（見文末的 2 型別實例化）。

**零依賴**：本模組只用 `List`（`List.sum`/`Nodup`/`Mem`）、`DecidableEq`、
`omega`，不引入任何 Mathlib 符號，也不新增自訂公理。 -/

import Polyrust.T9EndToEnd
import Polyrust.Tactics

namespace Polyrust

/-! ## 一、List 求和引理（型別宇宙求和的代數基礎）

本節引理都是「標記函數」形式 `f : α → Int`：`α` 是位置（此後是型別宇宙的
成員），`f` 給每個位置一個非負整數標記。這些引理是 one-hot 推理的根基。 -/

/-- 非負標記列表的和非負。 -/
theorem listSum_nonneg {α : Type} {f : α → Int} {l : List α}
    (hge : ∀ x ∈ l, 0 ≤ f x) : 0 ≤ (l.map f).sum := by
  induction l with
  | nil => simp
  | cons a l ih =>
    have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (List.mem_cons.mpr (Or.inr hx))
    have hfa : 0 ≤ f a := hge a (List.mem_cons.mpr (Or.inl rfl))
    have ih' := ih hge'
    simp only [List.map_cons, List.sum_cons]
    omega

/-- 某位置標記為 1、其餘非負，則和 ≥ 1。 -/
theorem listSum_ge_one_of_mem_one {α : Type} {f : α → Int} {l : List α}
    {y : α} (hy : y ∈ l) (hy1 : f y = 1) (hge : ∀ x ∈ l, 0 ≤ f x) : 1 ≤ (l.map f).sum := by
  induction l with
  | nil => simp at hy
  | cons a l ih =>
    simp only [List.map_cons, List.sum_cons, List.mem_cons] at *
    rcases hy with rfl | hy
    · rw [hy1]
      have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (Or.inr hx)
      have hn : 0 ≤ (l.map f).sum := listSum_nonneg hge'
      omega
    · have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (Or.inr hx)
      have ih' := ih hy hge'
      have hfa : 0 ≤ f a := hge a (Or.inl rfl)
      omega

/-- 兩個不同位置（`Nodup` 保證）都標記 1、其餘非負，則和 ≥ 2。 -/
theorem listSum_ge_two_of_two_ones {α : Type} {f : α → Int} {l : List α}
    (hnd : l.Nodup) {x y : α} (hx : x ∈ l) (hy : y ∈ l) (hne : x ≠ y)
    (hx1 : f x = 1) (hy1 : f y = 1) (hge : ∀ z ∈ l, 0 ≤ f z) :
    2 ≤ (l.map f).sum := by
  induction l with
  | nil => simp at hx
  | cons a l ih =>
    simp only [List.map_cons, List.sum_cons, List.mem_cons, List.nodup_cons] at *
    rcases hnd with ⟨hna, hnd'⟩
    rcases hx with rfl | hx
    · rcases hy with rfl | hy
      · exfalso; exact hne rfl
      · have hge' : ∀ z ∈ l, 0 ≤ f z := fun z hz => hge z (Or.inr hz)
        have h1 : 1 ≤ (l.map f).sum := listSum_ge_one_of_mem_one hy hy1 hge'
        rw [hx1]
        omega
    · rcases hy with rfl | hy
      · have hge' : ∀ z ∈ l, 0 ≤ f z := fun z hz => hge z (Or.inr hz)
        have h1 : 1 ≤ (l.map f).sum := listSum_ge_one_of_mem_one hx hx1 hge'
        rw [hy1]
        omega
      · have hge' : ∀ z ∈ l, 0 ≤ f z := fun z hz => hge z (Or.inr hz)
        have h2 : 2 ≤ (l.map f).sum := ih hnd' hx hy hge'
        have hfa : 0 ≤ f a := hge a (Or.inl rfl)
        omega

/-- 非負標記列表和為 1，則存在某位置標記為 1（「和=1 ⟹ 有解」）。 -/
theorem listSum_eq_one_exists_one {α : Type} {f : α → Int} {l : List α}
    (hsum : (l.map f).sum = 1) (hge : ∀ x ∈ l, 0 ≤ f x) : ∃ x, x ∈ l ∧ f x = 1 := by
  induction l with
  | nil => simp at hsum
  | cons a l ih =>
    simp only [List.map_cons, List.sum_cons, List.mem_cons] at *
    have hge' : ∀ x ∈ l, 0 ≤ f x := fun x hx => hge x (Or.inr hx)
    have hfa : 0 ≤ f a := hge a (Or.inl rfl)
    have hn : 0 ≤ (l.map f).sum := listSum_nonneg hge'
    by_cases ha1 : f a = 1
    · exact ⟨a, Or.inl rfl, ha1⟩
    · have ha0 : f a = 0 := by omega
      have hs' : (l.map f).sum = 1 := by omega
      rcases ih hs' hge' with ⟨x, hx, hx1⟩
      exact ⟨x, Or.inr hx, hx1⟩

/-- 「無重複列表上，x 不在列表中」時，標記函數的和為 0。 -/
theorem sum_mark_zero_of_not_mem {Ty : Type} [DecidableEq Ty] {l : List Ty} {x : Ty}
    (hx : x ∉ l) : (l.map (fun t => bit (decide (t = x)))).sum = 0 := by
  induction l with
  | nil => simp
  | cons a l ih =>
    simp only [List.mem_cons] at hx
    have hne : a ≠ x := fun h => hx (Or.inl h.symm)
    have ha : bit (decide (a = x)) = 0 := by
      simp [bit, hne]
    have hx' : x ∉ l := fun h => hx (Or.inr h)
    have ih' := ih hx'
    simp only [List.map_cons, List.sum_cons]
    rw [ha, ih']
    omega

/-- 「無重複列表上，唯一標記位置」：x ∈ l 且 Nodup，則標記函數
`fun t => bit (decide (t = x))` 的和恰為 1。 -/
theorem sum_mark_eq_one_of_mem {Ty : Type} [DecidableEq Ty] {l : List Ty} {x : Ty}
    (hx : x ∈ l) (hnd : l.Nodup) :
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
        sum_mark_zero_of_not_mem hx_not
      rw [ha, hsl]
      omega
    · have hne : a ≠ x := fun h => hna (h ▸ hx)
      have ha : bit (decide (a = x)) = 0 := by simp [bit, hne]
      have ih' := ih hx hnd'
      rw [ha, ih']
      omega


/-! ## 二、泛化語言：型別宇宙、檢查器、約束生成 -/

section Generalized

/-- 型別宇宙：任意型別 `Ty` 的全枚舉 + 兩個互異基元型別 + 完備性/無重複證明。

`Ty` 是**參數**（非欄位），使 `DecidableEq Ty` 成為獨立 typeclass 參數——
這是刻意設計：把 `DecidableEq` 放進 dependent 欄位會讓 `decide` 無法合成實例。 -/
structure Lang (Ty : Type) where
  enumAll : List Ty
  nodup : enumAll.Nodup
  complete : ∀ t : Ty, t ∈ enumAll
  numTy : Ty
  eqbTy : Ty
  num_ne_eqb : numTy ≠ eqbTy

variable {Ty : Type} [DecidableEq Ty]
variable (L : Lang Ty)

/-- 泛化雙向型別檢查器：`tycheck e τ` 表示「e 可被檢查為 τ」。

`num`/`add` 的型別是 `L.numTy`，`eqb` 的輸出型別是 `L.eqbTy`（輸入仍是
`L.numTy`），`ite` 的條件是 `L.eqbTy`、輸出是分支型別。 -/
def tycheck : Expr → Ty → Bool
  | .num _, τ => decide (τ = L.numTy)
  | .add a b, τ => decide (τ = L.numTy) && tycheck a L.numTy && tycheck b L.numTy
  | .eqb a b, τ => decide (τ = L.eqbTy) && tycheck a L.numTy && tycheck b L.numTy
  | .ite c t f, τ => tycheck c L.eqbTy && tycheck t τ && tycheck f τ

/-- 可定型（泛化）：存在宇宙中某個型別 τ 使 `tycheck e τ = true`。 -/
def TypableG (e : Expr) : Prop := ∃ t : Ty, tycheck L e t = true

/-- 各建構子可定型 ⟹ 其「基元型別」固定。 -/
theorem tycheck_num_eq_numTy {n : Int} {τ : Ty}
    (h : tycheck L (.num n) τ = true) : τ = L.numTy := by
  simpa [tycheck] using h

theorem tycheck_add_eq_numTy {a b : Expr} {τ : Ty}
    (h : tycheck L (.add a b) τ = true) : τ = L.numTy := by
  simp [tycheck] at h
  exact h.1.1

theorem tycheck_eqb_eq_eqbTy {a b : Expr} {τ : Ty}
    (h : tycheck L (.eqb a b) τ = true) : τ = L.eqbTy := by
  simp [tycheck] at h
  exact h.1.1

/-- **單型性（泛化）**：任一節點至多被檢查為一個型別。

對比 2 型別版本（`T9EndToEnd.check_exclusive`）的 `cases τ`：這裡改由
`L.num_ne_eqb` 保證兩個基元型別互異，`ite` 分支由歸納假設傳遞。 -/
theorem tycheck_exclusive (L : Lang Ty) : ∀ (e : Expr), ∀ τ τ' : Ty, τ ≠ τ' →
    ¬ (tycheck L e τ = true ∧ tycheck L e τ' = true) := by
  intro e
  induction e with
  | num n =>
    intro τ τ' hne ⟨h1, h2⟩
    have hτ : τ = L.numTy := tycheck_num_eq_numTy L h1
    have hτ' : τ' = L.numTy := tycheck_num_eq_numTy L h2
    exact hne (hτ.trans hτ'.symm)
  | add a b iha ihb =>
    intro τ τ' hne ⟨h1, h2⟩
    have hτ : τ = L.numTy := tycheck_add_eq_numTy L h1
    have hτ' : τ' = L.numTy := tycheck_add_eq_numTy L h2
    exact hne (hτ.trans hτ'.symm)
  | eqb a b iha ihb =>
    intro τ τ' hne ⟨h1, h2⟩
    have hτ : τ = L.eqbTy := tycheck_eqb_eq_eqbTy L h1
    have hτ' : τ' = L.eqbTy := tycheck_eqb_eq_eqbTy L h2
    exact hne (hτ.trans hτ'.symm)
  | ite c t f ihc iht ihf =>
    intro τ τ' hne ⟨h1, h2⟩
    simp only [tycheck, Bool.and_eq_true_iff] at h1 h2
    exact iht τ τ' hne ⟨h1.1.2, h2.1.2⟩

/-- 位元賦值：每個節點、每個型別一個 0/1 位元。 -/
abbrev SigmaG (Ty : Type) := Expr → Ty → Bool

/-- 節點 e 在型別 τ 上的位元，作為 ℤ 值。 -/
def tbG (σ : SigmaG Ty) (e : Expr) (τ : Ty) : Int := bit (σ e τ)

/-- **one-hot（泛化）**：`(Σ_{t ∈ enumAll} bit(σ e t)) − 1 = 0`。 -/
def oneHotG (σ : SigmaG Ty) (e : Expr) : Int :=
  (L.enumAll.map (fun t => bit (σ e t))).sum - 1

/-- one-hot 約束（以節點為參數的形式）。 -/
def oneHotCG (e : Expr) : SigmaG Ty → Int := fun σ => oneHotG L σ e

/-- 型別 t 的「基元標記」：t = numTy 時為 1，否則 0（作為 ℤ）。 -/
def numMark (t : Ty) : Int := bit (decide (t = L.numTy))
def eqbMark (t : Ty) : Int := bit (decide (t = L.eqbTy))

/-- `num n` 的規則方程（對每個 t ∈ enumAll）：`t_{e,t} = numMark t`。 -/
def cNum (n : Int) (t : Ty) : SigmaG Ty → Int :=
  fun σ => tbG σ (.num n) t - numMark L t

/-- `add a b` 的規則方程（對每個 t）：`t_{e,t} = numMark t · t_a,numTy · t_b,numTy`。 -/
def cAdd (a b : Expr) (t : Ty) : SigmaG Ty → Int :=
  fun σ => tbG σ (.add a b) t - numMark L t * tbG σ a L.numTy * tbG σ b L.numTy

/-- `eqb a b` 的規則方程（對每個 t）：`t_{e,t} = eqbMark t · t_a,numTy · t_b,numTy`。 -/
def cEqb (a b : Expr) (t : Ty) : SigmaG Ty → Int :=
  fun σ => tbG σ (.eqb a b) t - eqbMark L t * tbG σ a L.numTy * tbG σ b L.numTy

/-- `ite c t f` 的規則方程（對每個 τ）：`t_{e,τ} = t_c,eqbTy · t_t,τ · t_f,τ`。 -/
def cIte (c t f : Expr) (τ : Ty) : SigmaG Ty → Int :=
  fun σ => tbG σ (.ite c t f) τ - tbG σ c L.eqbTy * tbG σ t τ * tbG σ f τ

/-- 約束生成（泛化）：對每個子節點、每個型別生成規則方程，加 one-hot。 -/
def genCG : Expr → List (SigmaG Ty → Int)
  | .num n => (L.enumAll.map (fun t => cNum L n t)) ++ [oneHotCG L (.num n)]
  | .add a b => genCG a ++ genCG b ++ (L.enumAll.map (fun t => cAdd L a b t)) ++ [oneHotCG L (.add a b)]
  | .eqb a b => genCG a ++ genCG b ++ (L.enumAll.map (fun t => cEqb L a b t)) ++ [oneHotCG L (.eqb a b)]
  | .ite c t f => genCG c ++ genCG t ++ genCG f ++ (L.enumAll.map (fun τ => cIte L c t f τ)) ++ [oneHotCG L (.ite c t f)]

/-- σ 是方程組的 0/1 根。 -/
def IsRootG (e : Expr) (σ : SigmaG Ty) : Prop := ∀ p ∈ genCG L e, p σ = 0

/-- 見證賦值 σ_D：節點 e' 在位元 τ 上的值就是檢查器的答案 `tycheck e' τ`。 -/
def witnessG : SigmaG Ty := fun e' τ => tycheck L e' τ

/-! ## 三、位元算術與標記化簡 -/

/-- 位元非負：`bit b ∈ {0,1}`。 -/
theorem bit_nonneg (b : Bool) : 0 ≤ bit b := by cases b <;> simp [bit]

/-- `numMark` 在其本元處為 1。 -/
theorem numMark_self : numMark L L.numTy = 1 := by simp [numMark, bit]

/-- `numMark` 在非本元處為 0。 -/
theorem numMark_of_ne {t : Ty} (h : t ≠ L.numTy) : numMark L t = 0 := by
  simp [numMark, bit, h]

/-- `eqbMark` 在其本元處為 1。 -/
theorem eqbMark_self : eqbMark L L.eqbTy = 1 := by simp [eqbMark, bit]

/-- `eqbMark` 在非本元處為 0。 -/
theorem eqbMark_of_ne {t : Ty} (h : t ≠ L.eqbTy) : eqbMark L t = 0 := by
  simp [eqbMark, bit, h]

/-- `numMark` 的值就是 `bit (decide (t = numTy))`（定義展開）。 -/
theorem numMark_eq {t : Ty} : numMark L t = bit (decide (t = L.numTy)) := rfl

/-- `numTy` 在宇宙枚舉中，且無重複 —— one-hot 的「和=1」前提。 -/
theorem sum_numMark : (L.enumAll.map (fun t => numMark L t)).sum = 1 := by
  have hx : L.numTy ∈ L.enumAll := L.complete L.numTy
  have h := sum_mark_eq_one_of_mem hx L.nodup
  -- h : (L.enumAll.map (fun t => bit (decide (t = L.numTy)))).sum = 1
  simpa [numMark] using h

/-- `eqbTy` 的標記和也為 1。 -/
theorem sum_eqbMark : (L.enumAll.map (fun t => eqbMark L t)).sum = 1 := by
  have hx : L.eqbTy ∈ L.enumAll := L.complete L.eqbTy
  have h := sum_mark_eq_one_of_mem hx L.nodup
  simpa [eqbMark] using h

/-! ## 四、T1（可靠性）：可定型 ⟹ 見證賦值是根 -/

/-- 規則方程在見證賦值下歸零（`num` 節點，逐型別）。 -/
theorem cNum_witness {n : Int} {t : Ty} : cNum L n t (witnessG L) = 0 := by
  unfold cNum tbG witnessG
  simp only [tycheck]
  simp [numMark, bit]

/-- 規則方程在見證賦值下歸零（`add` 節點，逐型別；需子節點可定型）。 -/
theorem cAdd_witness {a b : Expr} {t : Ty}
    (ha : tycheck L a L.numTy = true) (hb : tycheck L b L.numTy = true) :
    cAdd L a b t (witnessG L) = 0 := by
  unfold cAdd tbG witnessG
  simp only [tycheck]
  rw [ha, hb]
  simp [numMark, bit]

/-- 規則方程在見證賦值下歸零（`eqb` 節點，逐型別）。 -/
theorem cEqb_witness {a b : Expr} {t : Ty}
    (ha : tycheck L a L.numTy = true) (hb : tycheck L b L.numTy = true) :
    cEqb L a b t (witnessG L) = 0 := by
  unfold cEqb tbG witnessG
  simp only [tycheck]
  rw [ha, hb]
  simp [eqbMark, bit]

/-- 規則方程在見證賦值下歸零（`ite` 節點，逐型別）。 -/
theorem cIte_witness {c t f : Expr} {τ : Ty}
    (hc : tycheck L c L.eqbTy = true)
    (ht : tycheck L t τ = true) (hf : tycheck L f τ = true) :
    cIte L c t f τ (witnessG L) = 0 := by
  unfold cIte tbG witnessG
  simp only [tycheck]
  rw [hc, ht, hf]
  simp [bit]

/-- one-hot 在見證賦值下歸零（`num` 節點）。 -/
theorem oneHot_num_witness {n : Int} : oneHotG L (witnessG L) (.num n) = 0 := by
  unfold oneHotG witnessG
  simp only [tycheck]
  have h : (L.enumAll.map (fun t => bit (decide (t = L.numTy)))).sum = 1 := by
    simpa [numMark] using (sum_numMark L)
  omega

/-- one-hot 在見證賦值下歸零（`add` 節點）。 -/
theorem oneHot_add_witness {a b : Expr}
    (ha : tycheck L a L.numTy = true) (hb : tycheck L b L.numTy = true) :
    oneHotG L (witnessG L) (.add a b) = 0 := by
  unfold oneHotG witnessG
  have hmain : (L.enumAll.map (fun t => bit (tycheck L (.add a b) t))).sum = 1 := by
    have hmap : (L.enumAll.map (fun t => bit (tycheck L (.add a b) t)))
        = (L.enumAll.map (fun t => bit (decide (t = L.numTy)))) := by
      apply List.map_eq_map_iff.mpr
      intro t ht
      simp only [tycheck]
      rw [ha, hb]
      simp [bit]
    rw [hmap]
    simpa [numMark] using (sum_numMark L)
  omega

/-- one-hot 在見證賦值下歸零（`eqb` 節點）。 -/
theorem oneHot_eqb_witness {a b : Expr}
    (ha : tycheck L a L.numTy = true) (hb : tycheck L b L.numTy = true) :
    oneHotG L (witnessG L) (.eqb a b) = 0 := by
  unfold oneHotG witnessG
  have hmain : (L.enumAll.map (fun t => bit (tycheck L (.eqb a b) t))).sum = 1 := by
    have hmap : (L.enumAll.map (fun t => bit (tycheck L (.eqb a b) t)))
        = (L.enumAll.map (fun t => bit (decide (t = L.eqbTy)))) := by
      apply List.map_eq_map_iff.mpr
      intro t ht
      simp only [tycheck]
      rw [ha, hb]
      simp [bit]
    rw [hmap]
    simpa [eqbMark] using (sum_eqbMark L)
  omega

/-- one-hot 在見證賦值下歸零（`ite` 節點）。 -/
theorem oneHot_ite_witness {c t f : Expr} {τ : Ty}
    (hc : tycheck L c L.eqbTy = true)
    (ht : tycheck L t τ = true) (hf : tycheck L f τ = true) :
    oneHotG L (witnessG L) (.ite c t f) = 0 := by
  unfold oneHotG witnessG
  have hmain : (L.enumAll.map (fun t' => bit (tycheck L (.ite c t f) t'))).sum = 1 := by
    have hx : τ ∈ L.enumAll := L.complete τ
    have hmap : (L.enumAll.map (fun t' => bit (tycheck L (.ite c t f) t')))
        = (L.enumAll.map (fun t' => bit (decide (t' = τ)))) := by
      apply List.map_eq_map_iff.mpr
      intro t' ht'
      by_cases ht'τ : t' = τ
      · subst ht'τ
        simp only [tycheck]
        rw [hc, ht, hf]
        simp [bit]
      · have htc : tycheck L t t' = false := by
          have hex := tycheck_exclusive L t t' τ ht'τ
          by_cases hh : tycheck L t t' = true
          · exfalso; exact hex ⟨hh, ht⟩
          · simpa using hh
        simp only [tycheck]
        rw [hc, htc]
        simp [bit, ht'τ]
    rw [hmap]
    have hsum := sum_mark_eq_one_of_mem hx L.nodup
    simpa using hsum
  omega

/-- **T1 主定理（泛化）**：可定型程序 ⟹ 見證賦值滿足全部約束。 -/
theorem genC_soundG (L : Lang Ty) : ∀ (e : Expr) (τ : Ty),
    tycheck L e τ = true → IsRootG L e (witnessG L) ∧ (witnessG L) e τ = true := by
  intro e
  induction e with
  | num n =>
    intro τ hτ
    refine ⟨?_, hτ⟩
    intro p hp
    rcases List.mem_append.mp hp with hmap | hone
    · rcases List.mem_map.mp hmap with ⟨t, ht, rfl⟩
      exact cNum_witness L
    · rw [List.mem_singleton.mp hone]
      simpa [oneHotCG] using (oneHot_num_witness L)
  | add a b iha ihb =>
    intro τ hτ
    have hsplit : τ = L.numTy ∧ tycheck L a L.numTy = true ∧ tycheck L b L.numTy = true := by
      simp [tycheck] at hτ
      exact ⟨hτ.1.1, hτ.1.2, hτ.2⟩
    rcases hsplit with ⟨rfl, ha, hb⟩
    have hra : IsRootG L a (witnessG L) := (iha L.numTy ha).1
    have hrb : IsRootG L b (witnessG L) := (ihb L.numTy hb).1
    refine ⟨?_, hτ⟩
    intro p hp
    rcases List.mem_append.mp hp with h | h
    · rcases List.mem_append.mp h with h | h
      · rcases List.mem_append.mp h with h | h
        · exact hra p h
        · exact hrb p h
      · rcases List.mem_map.mp h with ⟨t, ht, rfl⟩
        exact cAdd_witness L ha hb
    · rw [List.mem_singleton.mp h]
      simpa [oneHotCG] using (oneHot_add_witness L ha hb)
  | eqb a b iha ihb =>
    intro τ hτ
    have hsplit : τ = L.eqbTy ∧ tycheck L a L.numTy = true ∧ tycheck L b L.numTy = true := by
      simp [tycheck] at hτ
      exact ⟨hτ.1.1, hτ.1.2, hτ.2⟩
    rcases hsplit with ⟨rfl, ha, hb⟩
    have hra : IsRootG L a (witnessG L) := (iha L.numTy ha).1
    have hrb : IsRootG L b (witnessG L) := (ihb L.numTy hb).1
    refine ⟨?_, hτ⟩
    intro p hp
    rcases List.mem_append.mp hp with h | h
    · rcases List.mem_append.mp h with h | h
      · rcases List.mem_append.mp h with h | h
        · exact hra p h
        · exact hrb p h
      · rcases List.mem_map.mp h with ⟨t, ht, rfl⟩
        exact cEqb_witness L ha hb
    · rw [List.mem_singleton.mp h]
      simpa [oneHotCG] using (oneHot_eqb_witness L ha hb)
  | ite c t f ihc iht ihf =>
    intro τ hτ
    have hsplit : tycheck L c L.eqbTy = true ∧ tycheck L t τ = true ∧ tycheck L f τ = true := by
      simp [tycheck] at hτ
      exact ⟨hτ.1.1, hτ.1.2, hτ.2⟩
    rcases hsplit with ⟨hc, ht, hf⟩
    have hrc : IsRootG L c (witnessG L) := (ihc L.eqbTy hc).1
    have hrt : IsRootG L t (witnessG L) := (iht τ ht).1
    have hrf : IsRootG L f (witnessG L) := (ihf τ hf).1
    refine ⟨?_, hτ⟩
    intro p hp
    rcases List.mem_append.mp hp with h | h
    · rcases List.mem_append.mp h with h | h
      · rcases List.mem_append.mp h with h | h
        · rcases List.mem_append.mp h with h | h
          · exact hrc p h
          · exact hrt p h
        · exact hrf p h
      · rcases List.mem_map.mp h with ⟨t', ht', rfl⟩
        by_cases ht'τ : t' = τ
        · subst ht'τ
          exact cIte_witness L hc ht hf
        · have htt : tycheck L t t' = false := by
            have hex := tycheck_exclusive L t t' τ ht'τ
            by_cases hh : tycheck L t t' = true
            · exfalso; exact hex ⟨hh, ht⟩
            · simpa using hh
          unfold cIte tbG witnessG
          simp only [tycheck]
          rw [hc, htt]
          simp [bit]
    · rw [List.mem_singleton.mp h]
      simpa [oneHotCG] using (oneHot_ite_witness L hc ht hf)

/-! ## 五、約束表成員關係（顯式構造） -/

/-- 子表達式約束在父約束表裡（加法，左）。 -/
theorem mem_genCG_add_left {p : SigmaG Ty → Int} {a b : Expr} (hp : p ∈ genCG L a) :
    p ∈ genCG L (.add a b) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))))

/-- 子表達式約束在父約束表裡（加法，右）。 -/
theorem mem_genCG_add_right {p : SigmaG Ty → Int} {a b : Expr} (hp : p ∈ genCG L b) :
    p ∈ genCG L (.add a b) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))

/-- 子表達式約束在父約束表裡（`eqb`，左）。 -/
theorem mem_genCG_eqb_left {p : SigmaG Ty → Int} {a b : Expr} (hp : p ∈ genCG L a) :
    p ∈ genCG L (.eqb a b) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))))

/-- 子表達式約束在父約束表裡（`eqb`，右）。 -/
theorem mem_genCG_eqb_right {p : SigmaG Ty → Int} {a b : Expr} (hp : p ∈ genCG L b) :
    p ∈ genCG L (.eqb a b) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))

/-- 子表達式約束在父約束表裡（`ite`，條件）。 -/
theorem mem_genCG_ite_first {p : SigmaG Ty → Int} {c t f : Expr} (hp : p ∈ genCG L c) :
    p ∈ genCG L (.ite c t f) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))))))

/-- 子表達式約束在父約束表裡（`ite`，真分支）。 -/
theorem mem_genCG_ite_second {p : SigmaG Ty → Int} {c t f : Expr} (hp : p ∈ genCG L t) :
    p ∈ genCG L (.ite c t f) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))))

/-- 子表達式約束在父約束表裡（`ite`，假分支）。 -/
theorem mem_genCG_ite_third {p : SigmaG Ty → Int} {c t f : Expr} (hp : p ∈ genCG L f) :
    p ∈ genCG L (.ite c t f) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))

/-- 規則方程成員（`num`）。 -/
theorem mem_genCG_cNum {n : Int} {t : Ty} (ht : t ∈ L.enumAll) :
    cNum L n t ∈ genCG L (.num n) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_map.mpr ⟨t, ht, rfl⟩))

/-- 規則方程成員（`add`）。 -/
theorem mem_genCG_cAdd {a b : Expr} {t : Ty} (ht : t ∈ L.enumAll) :
    cAdd L a b t ∈ genCG L (.add a b) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨t, ht, rfl⟩))))

/-- 規則方程成員（`eqb`）。 -/
theorem mem_genCG_cEqb {a b : Expr} {t : Ty} (ht : t ∈ L.enumAll) :
    cEqb L a b t ∈ genCG L (.eqb a b) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨t, ht, rfl⟩))))

/-- 規則方程成員（`ite`）。 -/
theorem mem_genCG_cIte {c t f : Expr} {τ : Ty} (hτ : τ ∈ L.enumAll) :
    cIte L c t f τ ∈ genCG L (.ite c t f) := by
  unfold genCG
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨τ, hτ, rfl⟩))))

/-- one-hot 成員（各節點）。 -/
theorem mem_genCG_oneHot_num {n : Int} : oneHotCG L (.num n) ∈ genCG L (.num n) := by
  unfold genCG; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

theorem mem_genCG_oneHot_add {a b : Expr} : oneHotCG L (.add a b) ∈ genCG L (.add a b) := by
  unfold genCG; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

theorem mem_genCG_oneHot_eqb {a b : Expr} : oneHotCG L (.eqb a b) ∈ genCG L (.eqb a b) := by
  unfold genCG; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

theorem mem_genCG_oneHot_ite {c t f : Expr} : oneHotCG L (.ite c t f) ∈ genCG L (.ite c t f) := by
  unfold genCG; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

/-! ## 六、T2（完備性）：任何根都逐型別解碼為檢查器的答案 -/

/-- **T2 主定理（泛化）**：方程組的任一 0/1 根，其位元與檢查器判定逐型別一致
（`σ e τ = tycheck L e τ`）。 -/
theorem genC_completeG (L : Lang Ty) : ∀ (e : Expr) (σ : SigmaG Ty),
    IsRootG L e σ → ∀ τ, σ e τ = tycheck L e τ := by
  intro e
  induction e with
  | num n =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hc := hroot (cNum L n τ) (mem_genCG_cNum L hmemτ)
    have hbit : bit (σ (.num n) τ) = numMark L τ := by
      unfold cNum tbG at hc
      omega
    have hbit' : bit (σ (.num n) τ) = bit (decide (τ = L.numTy)) := by
      simpa [numMark] using hbit
    have hinj := bit_injective hbit'
    simpa [tycheck] using hinj
  | add a b iha ihb =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hroota : IsRootG L a σ := fun p hp => hroot p (mem_genCG_add_left L hp)
    have hrootb : IsRootG L b σ := fun p hp => hroot p (mem_genCG_add_right L hp)
    have ha := iha σ hroota L.numTy
    have hb := ihb σ hrootb L.numTy
    have hc := hroot (cAdd L a b τ) (mem_genCG_cAdd L hmemτ)
    have hrule : bit (σ (.add a b) τ) =
        numMark L τ * bit (σ a L.numTy) * bit (σ b L.numTy) := by
      unfold cAdd tbG at hc
      omega
    rw [ha, hb] at hrule
    apply bit_injective
    rw [hrule]
    simp only [tycheck, bit_and, numMark]
  | eqb a b iha ihb =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hroota : IsRootG L a σ := fun p hp => hroot p (mem_genCG_eqb_left L hp)
    have hrootb : IsRootG L b σ := fun p hp => hroot p (mem_genCG_eqb_right L hp)
    have ha := iha σ hroota L.numTy
    have hb := ihb σ hrootb L.numTy
    have hc := hroot (cEqb L a b τ) (mem_genCG_cEqb L hmemτ)
    have hrule : bit (σ (.eqb a b) τ) =
        eqbMark L τ * bit (σ a L.numTy) * bit (σ b L.numTy) := by
      unfold cEqb tbG at hc
      omega
    rw [ha, hb] at hrule
    apply bit_injective
    rw [hrule]
    simp only [tycheck, bit_and, eqbMark]
  | ite c t f ihc iht ihf =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hrootc : IsRootG L c σ := fun p hp => hroot p (mem_genCG_ite_first L hp)
    have hroott : IsRootG L t σ := fun p hp => hroot p (mem_genCG_ite_second L hp)
    have hrootf : IsRootG L f σ := fun p hp => hroot p (mem_genCG_ite_third L hp)
    have hc := ihc σ hrootc L.eqbTy
    have ht := iht σ hroott τ
    have hf := ihf σ hrootf τ
    have hci := hroot (cIte L c t f τ) (mem_genCG_cIte L hmemτ)
    have hrule : bit (σ (.ite c t f) τ) =
        bit (σ c L.eqbTy) * bit (σ t τ) * bit (σ f τ) := by
      unfold cIte tbG at hci
      omega
    rw [hc, ht, hf] at hrule
    apply bit_injective
    rw [hrule]
    simp only [tycheck, bit_and]

/-! ## 七、T6（判定）與 T9（判定等價） -/

/-- one-hot 約束在任何節點的約束表裡（通用形式）。 -/
theorem mem_genCG_oneHot (e : Expr) : oneHotCG L e ∈ genCG L e := by
  cases e with
  | num n => exact mem_genCG_oneHot_num L
  | add a b => exact mem_genCG_oneHot_add L
  | eqb a b => exact mem_genCG_oneHot_eqb L
  | ite c t f => exact mem_genCG_oneHot_ite L

/-- **T6 主定理（泛化）**：方程組有 0/1 根 ⟹ 程序可定型。

one-hot 約束在此扮演「不可定型 ⟹ 系統無解」的代數角色：根使
`Σ_t bit(σ e t) = 1`，由非負性知恰有一個位元為 1，再由完備性（T2）
解碼為 `tycheck` 的答案。 -/
theorem root_implies_typableG (L : Lang Ty) {e : Expr} {σ : SigmaG Ty}
    (hroot : IsRootG L e σ) : TypableG L e := by
  have hone : oneHotG L σ e = 0 := by
    have h := hroot (oneHotCG L e) (mem_genCG_oneHot L e)
    simpa [oneHotCG] using h
  have hsum : (L.enumAll.map (fun t => bit (σ e t))).sum = 1 := by
    unfold oneHotG at hone
    omega
  have hge : ∀ x ∈ L.enumAll, 0 ≤ bit (σ e x) := fun x hx => bit_nonneg (σ e x)
  have hex := listSum_eq_one_exists_one hsum hge
  rcases hex with ⟨t, ht, ht1⟩
  have hst : σ e t = true := bit_eq_one_iff.mp ht1
  have hcomp := genC_completeG L e σ hroot t
  refine ⟨t, ?_⟩
  rw [← hcomp, hst]

/-- **T9 判定等價（泛化）**：管線「代數可解」⟺ 檢查器「接受」。 -/
theorem typable_iff_rootG (L : Lang Ty) (e : Expr) :
    TypableG L e ↔ ∃ σ : SigmaG Ty, IsRootG L e σ := by
  constructor
  · intro ht
    rcases ht with ⟨t, ht⟩
    exact ⟨witnessG L, (genC_soundG L e t ht).1⟩
  · intro ⟨σ, hroot⟩
    exact root_implies_typableG L hroot

/-- 衍生：不可定型 ⟺ 方程組無 0/1 根（UNSAT 側）。 -/
theorem untypable_iff_no_rootG (L : Lang Ty) (e : Expr) :
    ¬ TypableG L e ↔ ¬ ∃ σ : SigmaG Ty, IsRootG L e σ := by
  constructor
  · intro h ⟨σ, hroot⟩
    exact h (root_implies_typableG L hroot)
  · intro h ht
    exact h ((typable_iff_rootG L e).mp ht)

/-- 衍生：根的型別位元由檢查器唯一決定。 -/
theorem root_bit_determinedG (L : Lang Ty) {e : Expr} {σ : SigmaG Ty}
    (hroot : IsRootG L e σ) (τ : Ty) : σ e τ = tycheck L e τ :=
  genC_completeG L e σ hroot τ

/-- 衍生：可定型 ⟹ 存在滿足約束的賦值。 -/
theorem typable_exists_rootG (L : Lang Ty) {e : Expr} (h : TypableG L e) :
    ∃ σ : SigmaG Ty, IsRootG L e σ :=
  (typable_iff_rootG L e).mp h

/-- **根在子表達式上單型**（one-hot 的直接推論，泛化版）。

由 `oneHotG L σ e = 0`（即 `Σ_t bit(σ e t) = 1`）與非負性，推出至多一個
型別位元為 1。 -/
theorem isMonoAtG_of_root (L : Lang Ty) {e : Expr} {σ : SigmaG Ty}
    (hroot : IsRootG L e σ) : ∀ τ τ' : Ty, τ ≠ τ' →
    ¬ (σ e τ = true ∧ σ e τ' = true) := by
  have hone : oneHotG L σ e = 0 := by
    have h := hroot (oneHotCG L e) (mem_genCG_oneHot L e)
    simpa [oneHotCG] using h
  have hsum : (L.enumAll.map (fun t => bit (σ e t))).sum = 1 := by
    unfold oneHotG at hone
    omega
  intro τ τ' hne ⟨h1, h2⟩
  have hbitτ : bit (σ e τ) = 1 := by simp [bit, h1]
  have hbitτ' : bit (σ e τ') = 1 := by simp [bit, h2]
  have hmemτ : τ ∈ L.enumAll := L.complete τ
  have hmemτ' : τ' ∈ L.enumAll := L.complete τ'
  have hge : ∀ z ∈ L.enumAll, 0 ≤ bit (σ e z) := fun z hz => bit_nonneg (σ e z)
  have h2sum : 2 ≤ (L.enumAll.map (fun t => bit (σ e t))).sum :=
    listSum_ge_two_of_two_ones L.nodup hmemτ hmemτ' hne hbitτ hbitτ' hge
  omega

end Generalized

/-! ## 八、實例化：泛化定理還原 2 型別版，並零成本擴展到新宇宙 -/

section Instantiation

/-- 原 2 型別宇宙（`i32`/`boolean`）作為 `Lang` 實例——證明負擔僅在
**構造這 6 個參數**（`enumAll`/`nodup`/`complete`/`numTy`/`eqbTy`/`num_ne_eqb`）。 -/
def twoTypeLang : Lang Ty where
  enumAll := [Ty.i32, Ty.boolean]
  nodup := by simp
  complete := by intro t; cases t <;> simp
  numTy := Ty.i32
  eqbTy := Ty.boolean
  num_ne_eqb := by intro h; cases h

/-- 泛化判定等價（T9）在 2 型別宇宙上自動成立——無需重寫 `T9EndToEnd` 的證明。 -/
example (e : Expr) :
    TypableG twoTypeLang e ↔ ∃ σ : SigmaG Ty, IsRootG twoTypeLang e σ :=
  typable_iff_rootG twoTypeLang e

/-- 泛化可靠性（T1）在 2 型別宇宙上自動成立。 -/
example (e : Expr) (τ : Ty) :
    tycheck twoTypeLang e τ = true →
    IsRootG twoTypeLang e (witnessG twoTypeLang) ∧ (witnessG twoTypeLang) e τ = true :=
  genC_soundG twoTypeLang e τ

/-- 泛化完備性（T2）在 2 型別宇宙上自動成立。 -/
example (e : Expr) (σ : SigmaG Ty) (h : IsRootG twoTypeLang e σ) (τ : Ty) :
    σ e τ = tycheck twoTypeLang e τ :=
  genC_completeG twoTypeLang e σ h τ

/-- **擴展性展示**：3 型別宇宙（`i32`/`i64`/`boolean`）。 -/
inductive Ty3 | i32 | i64 | boolean
  deriving DecidableEq, Repr

/-- 3 型別宇宙的 `Lang` 實例：**新增 `i64` 的唯一證明負擔是寫下這 6 個參數**。 -/
def threeTypeLang : Lang Ty3 where
  enumAll := [Ty3.i32, Ty3.i64, Ty3.boolean]
  nodup := by simp
  complete := by intro t; cases t <;> simp
  numTy := Ty3.i32
  eqbTy := Ty3.boolean
  num_ne_eqb := by intro h; cases h

/-- 判定等價（T9）在 3 型別宇宙上**零新證明**成立。 -/
example (e : Expr) :
    TypableG threeTypeLang e ↔ ∃ σ : SigmaG Ty3, IsRootG threeTypeLang e σ :=
  typable_iff_rootG threeTypeLang e

/-- 可靠性（T1）在 3 型別宇宙上**零新證明**成立。 -/
example (e : Expr) (τ : Ty3) :
    tycheck threeTypeLang e τ = true →
    IsRootG threeTypeLang e (witnessG threeTypeLang) ∧ (witnessG threeTypeLang) e τ = true :=
  genC_soundG threeTypeLang e τ

end Instantiation

end Polyrust
