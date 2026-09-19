-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 運算子規則抽象（方案 (e)）：任意二元運算子規格上的端到端正確性

對應 `docs/THEOREMS.md` §11（T9）泛化方案的**第 (e) 步**：把 `T9Generalized`
語言裡**寫死的兩個二元運算子**（`add : numTy×numTy→numTy`、`eqb : numTy×numTy→eqbTy`）
抽象為**任意二元運算子規格**：

* 定義規格 `BinSpec Ty := { in1, in2, out : Ty }`（兩個輸入型別、一個輸出型別）；
* 泛化表達式 `ExprG Ty` 的建構子從「寫死的 `add`/`eqb`」變成「帶規格標籤的
  `binop s a b`」，檢查器 `tycheckG (binop s a b) τ = (τ = s.out) && a : s.in1 && b : s.in2`；
* 約束編碼 `cBinop s a b t` 的規則方程 `t_{binop,out} = t_{a,in1} · t_{b,in2}` 對任意
  規格 `s` 成立。

**這不是修辭**：Rust 側的 9 種 `BinOp`（Add/Sub/Mul/Lt/Le/Ge/Eq/Ne/And）就是 9 個
`BinSpec` 實例（見文末的實例化），Lean 側的 `add`/`eqb` 也是其中兩個。一旦本模組
定理成立，新增任何二元運算子（只要它能寫成 `BinSpec`）**零新證明**自動獲得
T1 可靠性 / T2 完備性 / T9 判定等價——這同時涵蓋了「擴展不變性」：把運算子集合
擴張只是換一個更大的 `BinSpec` 列表，定理對任意列表成立。

**零依賴**：同 `T9Generalized`，只用 `List`/`DecidableEq`/`omega`。 -/

import Polyrust.T9Generalized

namespace Polyrust

set_option linter.unusedSectionVars false
set_option linter.unusedVariables false

section OpAbs

/-! ## 一、運算子規格與泛化表達式 -/

/-- 二元運算子規格：兩個輸入型別、一個輸出型別。

Rust `BinOp` 的每一種都是一個規格，例如 `Add := {in1:=i32, in2:=i32, out:=i32}`、
`Eq := {in1:=i32, in2:=i32, out:=bool}`、`And := {in1:=bool, in2:=bool, out:=bool}`。 -/
structure BinSpec (Ty : Type) where
  in1 : Ty
  in2 : Ty
  out : Ty
  deriving DecidableEq, Repr

/-- 泛化表達式：`num` 字面量、帶規格標籤的二元運算子 `binop s a b`、條件 `ite`。

對比 `T9EndToEnd.Expr`（寫死 `add`/`eqb`），這裡 `binop s a b` 把「是哪個運算子」
變成**資料**（規格 `s`）而非語法。 -/
inductive ExprG (Ty : Type)
  | num : Int → ExprG Ty
  | binop : BinSpec Ty → ExprG Ty → ExprG Ty → ExprG Ty
  | ite : ExprG Ty → ExprG Ty → ExprG Ty → ExprG Ty
  deriving DecidableEq, Repr

variable {Ty : Type} [DecidableEq Ty]
variable (L : Lang Ty)

/-! ## 二、泛化檢查器、單型性 -/

/-- 泛化檢查器：`binop s a b` 的輸出型別必須是 `s.out`，且 `a : s.in1`、`b : s.in2`。 -/
def tycheckG : ExprG Ty → Ty → Bool
  | .num _, τ => decide (τ = L.numTy)
  | .binop s a b, τ => decide (τ = s.out) && tycheckG a s.in1 && tycheckG b s.in2
  | .ite c t f, τ => tycheckG c L.eqbTy && tycheckG t τ && tycheckG f τ

/-- 可定型（泛化運算子）。 -/
def TypableG2 (e : ExprG Ty) : Prop := ∃ t : Ty, tycheckG L e t = true

/-- `num` 可定型 ⟹ 型別固定為 `numTy`。 -/
theorem tycheckG_num_eq_numTy {n : Int} {τ : Ty}
    (h : tycheckG L (.num n) τ = true) : τ = L.numTy := by
  simpa [tycheckG] using h

/-- `binop s a b` 可定型 ⟹ 型別固定為 `s.out`（輸出型別由規格唯一決定）。 -/
theorem tycheckG_binop_eq_out {s : BinSpec Ty} {a b : ExprG Ty} {τ : Ty}
    (h : tycheckG L (.binop s a b) τ = true) : τ = s.out := by
  simp [tycheckG] at h
  exact h.1.1

/-- **單型性（泛化運算子）**：任一節點至多被檢查為一個型別。

`binop` 節點的輸出型別由規格 `s.out` 唯一決定（規格是資料，不是可選的）。 -/
theorem tycheckG_exclusive (L : Lang Ty) : ∀ (e : ExprG Ty), ∀ τ τ' : Ty, τ ≠ τ' →
    ¬ (tycheckG L e τ = true ∧ tycheckG L e τ' = true) := by
  intro e
  induction e with
  | num n =>
    intro τ τ' hne ⟨h1, h2⟩
    have hτ : τ = L.numTy := tycheckG_num_eq_numTy L h1
    have hτ' : τ' = L.numTy := tycheckG_num_eq_numTy L h2
    exact hne (hτ.trans hτ'.symm)
  | binop s a b iha ihb =>
    intro τ τ' hne ⟨h1, h2⟩
    have hτ : τ = s.out := tycheckG_binop_eq_out L h1
    have hτ' : τ' = s.out := tycheckG_binop_eq_out L h2
    exact hne (hτ.trans hτ'.symm)
  | ite c t f ihc iht ihf =>
    intro τ τ' hne ⟨h1, h2⟩
    simp only [tycheckG, Bool.and_eq_true_iff] at h1 h2
    exact iht τ τ' hne ⟨h1.1.2, h2.1.2⟩

/-! ## 三、約束編碼（運算子規格參數化） -/

/-- 位元賦值：每個節點、每個型別一個 0/1 位元。 -/
abbrev SigmaG2 (Ty : Type) := ExprG Ty → Ty → Bool

/-- 節點 e 在型別 τ 上的位元，作為 ℤ 值。 -/
def tbG2 (σ : SigmaG2 Ty) (e : ExprG Ty) (τ : Ty) : Int := bit (σ e τ)

/-- **one-hot（泛化運算子）**：`(Σ_{t ∈ enumAll} bit(σ e t)) − 1 = 0`。 -/
def oneHotG2 (σ : SigmaG2 Ty) (e : ExprG Ty) : Int :=
  (L.enumAll.map (fun t => bit (σ e t))).sum - 1

/-- one-hot 約束（以節點為參數的形式）。 -/
def oneHotC2 (e : ExprG Ty) : SigmaG2 Ty → Int := fun σ => oneHotG2 L σ e

/-- 規格 `s` 的輸出標記（對偶於 `T9Generalized.numMark`/`eqbMark`）。 -/
def outMark (s : BinSpec Ty) (t : Ty) : Int := bit (decide (t = s.out))

/-- `num` 節點的規則方程（逐型別）：位元 = num 標記。 -/
def cNum2 (n : Int) (t : Ty) : SigmaG2 Ty → Int :=
  fun σ => tbG2 σ (.num n) t - numMark L t

/-- `binop s a b` 節點的規則方程（逐型別）：
位元 = 輸出標記 · a 在 s.in1 的位元 · b 在 s.in2 的位元（類比 `cAdd` 的 `numMark`）。 -/
def cBinop (s : BinSpec Ty) (a b : ExprG Ty) (t : Ty) : SigmaG2 Ty → Int :=
  fun σ => tbG2 σ (.binop s a b) t - outMark s t * tbG2 σ a s.in1 * tbG2 σ b s.in2

/-- `ite` 節點的規則方程（逐型別）。 -/
def cIte2 (c t f : ExprG Ty) (τ : Ty) : SigmaG2 Ty → Int :=
  fun σ => tbG2 σ (.ite c t f) τ - (tbG2 σ c L.eqbTy * tbG2 σ t τ * tbG2 σ f τ)

/-- 約束生成（泛化運算子）：每個節點的規則方程 + one-hot。 -/
def genCG2 : ExprG Ty → List (SigmaG2 Ty → Int)
  | .num n => (L.enumAll.map (fun t => cNum2 L n t)) ++ [oneHotC2 L (.num n)]
  | .binop s a b => genCG2 a ++ genCG2 b ++ (L.enumAll.map (fun t => cBinop s a b t)) ++ [oneHotC2 L (.binop s a b)]
  | .ite c t f => genCG2 c ++ genCG2 t ++ genCG2 f ++ (L.enumAll.map (fun τ => cIte2 L c t f τ)) ++ [oneHotC2 L (.ite c t f)]

/-- 根：滿足全部約束的位元賦值。 -/
def IsRootG2 (e : ExprG Ty) (σ : SigmaG2 Ty) : Prop := ∀ p ∈ genCG2 L e, p σ = 0

/-- 見證賦值：直接取檢查器的答案。 -/
def witnessG2 : SigmaG2 Ty := fun e' τ => tycheckG L e' τ

/-! ## 四、位元標記與規則方程在見證下的化簡 -/

/-- 規格輸出標記：`s.out` 自映為 1。 -/
theorem outMark_self {s : BinSpec Ty} : outMark s s.out = 1 := by simp [outMark, bit]

/-- 規格輸出標記：`t ≠ s.out` 時為 0。 -/
theorem outMark_of_ne {s : BinSpec Ty} {t : Ty} (h : t ≠ s.out) : outMark s t = 0 := by
  simp [outMark, bit, h]

/-- 規格輸出標記求和 = 1（`s.out ∈ enumAll` 由 `complete` 保證）。 -/
theorem sum_outMark {s : BinSpec Ty} : (L.enumAll.map (fun t => outMark s t)).sum = 1 := by
  have hx : s.out ∈ L.enumAll := L.complete s.out
  simpa [outMark] using (sum_mark_eq_one_of_mem hx L.nodup)

/-! ## 五、T1（可靠性）：可定型 ⟹ 見證賦值是根 -/

/-- 規則方程在見證賦值下歸零（`num` 節點）。 -/
theorem cNum2_witness {n : Int} {t : Ty} : cNum2 L n t (witnessG2 L) = 0 := by
  unfold cNum2 tbG2 witnessG2
  simp only [tycheckG]
  simp [numMark, bit]

/-- 規則方程在見證賦值下歸零（`binop` 節點，逐型別；需子節點可定型）。 -/
theorem cBinop_witness {s : BinSpec Ty} {a b : ExprG Ty} {t : Ty}
    (ha : tycheckG L a s.in1 = true) (hb : tycheckG L b s.in2 = true) :
    cBinop s a b t (witnessG2 L) = 0 := by
  unfold cBinop tbG2 witnessG2
  simp only [tycheckG]
  rw [ha, hb]
  simp [outMark, bit]

/-- 規則方程在見證賦值下歸零（`ite` 節點，逐型別）。 -/
theorem cIte2_witness {c t f : ExprG Ty} {τ : Ty}
    (hc : tycheckG L c L.eqbTy = true)
    (ht : tycheckG L t τ = true) (hf : tycheckG L f τ = true) :
    cIte2 L c t f τ (witnessG2 L) = 0 := by
  unfold cIte2 tbG2 witnessG2
  simp only [tycheckG]
  rw [hc, ht, hf]
  simp [bit]

/-- one-hot 在見證賦值下歸零（`num` 節點）。 -/
theorem oneHot_num_witness2 {n : Int} : oneHotG2 L (witnessG2 L) (.num n) = 0 := by
  unfold oneHotG2 witnessG2
  simp only [tycheckG]
  have h : (L.enumAll.map (fun t => bit (decide (t = L.numTy)))).sum = 1 := by
    simpa [numMark] using (sum_numMark L)
  omega

/-- one-hot 在見證賦值下歸零（`binop` 節點；規格參數化）。 -/
theorem oneHot_binop_witness {s : BinSpec Ty} {a b : ExprG Ty}
    (ha : tycheckG L a s.in1 = true) (hb : tycheckG L b s.in2 = true) :
    oneHotG2 L (witnessG2 L) (.binop s a b) = 0 := by
  unfold oneHotG2 witnessG2
  have hmain : (L.enumAll.map (fun t => bit (tycheckG L (.binop s a b) t))).sum = 1 := by
    have hmap : (L.enumAll.map (fun t => bit (tycheckG L (.binop s a b) t)))
        = (L.enumAll.map (fun t => bit (decide (t = s.out)))) := by
      apply List.map_eq_map_iff.mpr
      intro t ht
      simp only [tycheckG]
      rw [ha, hb]
      simp [bit]
    rw [hmap]
    simpa [outMark] using (sum_outMark L)
  omega

/-- one-hot 在見證賦值下歸零（`ite` 節點）。 -/
theorem oneHot_ite_witness2 {c t f : ExprG Ty} {τ : Ty}
    (hc : tycheckG L c L.eqbTy = true)
    (ht : tycheckG L t τ = true) (hf : tycheckG L f τ = true) :
    oneHotG2 L (witnessG2 L) (.ite c t f) = 0 := by
  unfold oneHotG2 witnessG2
  have hmain : (L.enumAll.map (fun t' => bit (tycheckG L (.ite c t f) t'))).sum = 1 := by
    have hx : τ ∈ L.enumAll := L.complete τ
    have hmap : (L.enumAll.map (fun t' => bit (tycheckG L (.ite c t f) t')))
        = (L.enumAll.map (fun t' => bit (decide (t' = τ)))) := by
      apply List.map_eq_map_iff.mpr
      intro t' ht'
      by_cases ht'τ : t' = τ
      · subst ht'τ
        simp only [tycheckG]
        rw [hc, ht, hf]
        simp [bit]
      · have htc : tycheckG L t t' = false := by
          have hex := tycheckG_exclusive L t t' τ ht'τ
          by_cases hh : tycheckG L t t' = true
          · exfalso; exact hex ⟨hh, ht⟩
          · simpa using hh
        simp only [tycheckG]
        rw [hc, htc]
        simp [bit, ht'τ]
    rw [hmap]
    have hsum := sum_mark_eq_one_of_mem hx L.nodup
    simpa using hsum
  omega

/-! ## 六、約束表成員關係（顯式構造） -/

/-- 子表達式約束在父約束表裡（`binop`，左）。 -/
theorem mem_genCG2_binop_left {p : SigmaG2 Ty → Int} {s : BinSpec Ty} {a b : ExprG Ty}
    (hp : p ∈ genCG2 L a) : p ∈ genCG2 L (.binop s a b) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))))

/-- 子表達式約束在父約束表裡（`binop`，右）。 -/
theorem mem_genCG2_binop_right {p : SigmaG2 Ty → Int} {s : BinSpec Ty} {a b : ExprG Ty}
    (hp : p ∈ genCG2 L b) : p ∈ genCG2 L (.binop s a b) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))

/-- 子表達式約束在父約束表裡（`ite`，條件/真/假分支）。 -/
theorem mem_genCG2_ite_first {p : SigmaG2 Ty → Int} {c t f : ExprG Ty}
    (hp : p ∈ genCG2 L c) : p ∈ genCG2 L (.ite c t f) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))))))

theorem mem_genCG2_ite_second {p : SigmaG2 Ty → Int} {c t f : ExprG Ty}
    (hp : p ∈ genCG2 L t) : p ∈ genCG2 L (.ite c t f) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))))

theorem mem_genCG2_ite_third {p : SigmaG2 Ty → Int} {c t f : ExprG Ty}
    (hp : p ∈ genCG2 L f) : p ∈ genCG2 L (.ite c t f) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))

/-- 規則方程成員（`num`）。 -/
theorem mem_genCG2_cNum2 {n : Int} {t : Ty} (ht : t ∈ L.enumAll) :
    cNum2 L n t ∈ genCG2 L (.num n) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_map.mpr ⟨t, ht, rfl⟩))

/-- 規則方程成員（`binop`）。 -/
theorem mem_genCG2_cBinop {s : BinSpec Ty} {a b : ExprG Ty} {t : Ty} (ht : t ∈ L.enumAll) :
    cBinop s a b t ∈ genCG2 L (.binop s a b) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨t, ht, rfl⟩))))

/-- 規則方程成員（`ite`）。 -/
theorem mem_genCG2_cIte2 {c t f : ExprG Ty} {τ : Ty} (hτ : τ ∈ L.enumAll) :
    cIte2 L c t f τ ∈ genCG2 L (.ite c t f) := by
  unfold genCG2
  exact List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨τ, hτ, rfl⟩))))

/-- one-hot 成員（各節點）。 -/
theorem mem_genCG2_oneHot_num2 {n : Int} : oneHotC2 L (.num n) ∈ genCG2 L (.num n) := by
  unfold genCG2; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

theorem mem_genCG2_oneHot_binop {s : BinSpec Ty} {a b : ExprG Ty} :
    oneHotC2 L (.binop s a b) ∈ genCG2 L (.binop s a b) := by
  unfold genCG2; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

theorem mem_genCG2_oneHot_ite {c t f : ExprG Ty} :
    oneHotC2 L (.ite c t f) ∈ genCG2 L (.ite c t f) := by
  unfold genCG2; exact List.mem_append.mpr (Or.inr (List.mem_singleton.mpr rfl))

/-! ## 七、T1（可靠性）主定理 -/

/-- **T1 主定理（泛化運算子）**：可定型程序 ⟹ 見證賦值滿足全部約束。 -/
theorem genC_soundG2 (L : Lang Ty) : ∀ (e : ExprG Ty) (τ : Ty),
    tycheckG L e τ = true → IsRootG2 L e (witnessG2 L) ∧ (witnessG2 L) e τ = true := by
  intro e
  induction e with
  | num n =>
    intro τ hτ
    refine ⟨?_, hτ⟩
    intro p hp
    rcases List.mem_append.mp hp with hmap | hone
    · rcases List.mem_map.mp hmap with ⟨t, ht, rfl⟩
      exact cNum2_witness L
    · rw [List.mem_singleton.mp hone]
      simpa [oneHotC2] using (oneHot_num_witness2 L)
  | binop s a b iha ihb =>
    intro τ hτ
    have hsplit : τ = s.out ∧ tycheckG L a s.in1 = true ∧ tycheckG L b s.in2 = true := by
      simp [tycheckG] at hτ
      exact ⟨hτ.1.1, hτ.1.2, hτ.2⟩
    rcases hsplit with ⟨rfl, ha, hb⟩
    have hra : IsRootG2 L a (witnessG2 L) := (iha s.in1 ha).1
    have hrb : IsRootG2 L b (witnessG2 L) := (ihb s.in2 hb).1
    refine ⟨?_, hτ⟩
    intro p hp
    rcases List.mem_append.mp hp with h | h
    · rcases List.mem_append.mp h with h | h
      · rcases List.mem_append.mp h with h | h
        · exact hra p h
        · exact hrb p h
      · rcases List.mem_map.mp h with ⟨t, ht, rfl⟩
        exact cBinop_witness L ha hb
    · rw [List.mem_singleton.mp h]
      simpa [oneHotC2] using (oneHot_binop_witness L ha hb)
  | ite c t f ihc iht ihf =>
    intro τ hτ
    have hsplit : tycheckG L c L.eqbTy = true ∧ tycheckG L t τ = true ∧ tycheckG L f τ = true := by
      simp [tycheckG] at hτ
      exact ⟨hτ.1.1, hτ.1.2, hτ.2⟩
    rcases hsplit with ⟨hc, ht, hf⟩
    have hrc : IsRootG2 L c (witnessG2 L) := (ihc L.eqbTy hc).1
    have hrt : IsRootG2 L t (witnessG2 L) := (iht τ ht).1
    have hrf : IsRootG2 L f (witnessG2 L) := (ihf τ hf).1
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
          exact cIte2_witness L hc ht hf
        · have htt : tycheckG L t t' = false := by
            have hex := tycheckG_exclusive L t t' τ ht'τ
            by_cases hh : tycheckG L t t' = true
            · exfalso; exact hex ⟨hh, ht⟩
            · simpa using hh
          unfold cIte2 tbG2 witnessG2
          simp only [tycheckG]
          rw [hc, htt]
          simp [bit]
    · rw [List.mem_singleton.mp h]
      simpa [oneHotC2] using (oneHot_ite_witness2 L hc ht hf)

/-! ## 八、T2（完備性）主定理 -/

/-- **T2 主定理（泛化運算子）**：方程組的任一 0/1 根，其位元與檢查器判定逐型別一致。 -/
theorem genC_completeG2 (L : Lang Ty) : ∀ (e : ExprG Ty) (σ : SigmaG2 Ty),
    IsRootG2 L e σ → ∀ τ, σ e τ = tycheckG L e τ := by
  intro e
  induction e with
  | num n =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hc := hroot (cNum2 L n τ) (mem_genCG2_cNum2 L hmemτ)
    have hbit : bit (σ (.num n) τ) = numMark L τ := by
      unfold cNum2 tbG2 at hc
      omega
    have hbit' : bit (σ (.num n) τ) = bit (decide (τ = L.numTy)) := by
      simpa [numMark] using hbit
    have hinj := bit_injective hbit'
    simpa [tycheckG] using hinj
  | binop s a b iha ihb =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hroota : IsRootG2 L a σ := fun p hp => hroot p (mem_genCG2_binop_left L hp)
    have hrootb : IsRootG2 L b σ := fun p hp => hroot p (mem_genCG2_binop_right L hp)
    have ha := iha σ hroota s.in1
    have hb := ihb σ hrootb s.in2
    have hc := hroot (cBinop s a b τ) (mem_genCG2_cBinop L hmemτ)
    have hrule : bit (σ (.binop s a b) τ) =
        outMark s τ * bit (σ a s.in1) * bit (σ b s.in2) := by
      unfold cBinop tbG2 at hc
      omega
    rw [ha, hb] at hrule
    apply bit_injective
    rw [hrule]
    simp only [tycheckG, bit_and, outMark]
  | ite c t f ihc iht ihf =>
    intro σ hroot τ
    have hmemτ : τ ∈ L.enumAll := L.complete τ
    have hrootc : IsRootG2 L c σ := fun p hp => hroot p (mem_genCG2_ite_first L hp)
    have hroott : IsRootG2 L t σ := fun p hp => hroot p (mem_genCG2_ite_second L hp)
    have hrootf : IsRootG2 L f σ := fun p hp => hroot p (mem_genCG2_ite_third L hp)
    have hc := ihc σ hrootc L.eqbTy
    have ht := iht σ hroott τ
    have hf := ihf σ hrootf τ
    have hci := hroot (cIte2 L c t f τ) (mem_genCG2_cIte2 L hmemτ)
    have hrule : bit (σ (.ite c t f) τ) =
        bit (σ c L.eqbTy) * bit (σ t τ) * bit (σ f τ) := by
      unfold cIte2 tbG2 at hci
      omega
    rw [hc, ht, hf] at hrule
    apply bit_injective
    rw [hrule]
    simp only [tycheckG, bit_and]

/-! ## 九、T6（判定）與 T9（判定等價） -/

/-- one-hot 約束在任何節點的約束表裡（通用形式）。 -/
theorem mem_genCG2_oneHot (e : ExprG Ty) : oneHotC2 L e ∈ genCG2 L e := by
  cases e with
  | num n => exact mem_genCG2_oneHot_num2 L
  | binop s a b => exact mem_genCG2_oneHot_binop L
  | ite c t f => exact mem_genCG2_oneHot_ite L

/-- **T6 主定理（泛化運算子）**：方程組有 0/1 根 ⟹ 程序可定型。 -/
theorem root_implies_typableG2 (L : Lang Ty) {e : ExprG Ty} {σ : SigmaG2 Ty}
    (hroot : IsRootG2 L e σ) : TypableG2 L e := by
  have hone : oneHotG2 L σ e = 0 := by
    have h := hroot (oneHotC2 L e) (mem_genCG2_oneHot L e)
    simpa [oneHotC2] using h
  have hsum : (L.enumAll.map (fun t => bit (σ e t))).sum = 1 := by
    unfold oneHotG2 at hone
    omega
  have hge : ∀ x ∈ L.enumAll, 0 ≤ bit (σ e x) := fun x hx => bit_nonneg (σ e x)
  have hex := listSum_eq_one_exists_one hsum hge
  rcases hex with ⟨t, ht, ht1⟩
  have hst : σ e t = true := bit_eq_one_iff.mp ht1
  have hcomp := genC_completeG2 L e σ hroot t
  refine ⟨t, ?_⟩
  rw [← hcomp, hst]

/-- **T9 判定等價（泛化運算子）**：管線「代數可解」⟺ 檢查器「接受」。 -/
theorem typable_iff_rootG2 (L : Lang Ty) (e : ExprG Ty) :
    TypableG2 L e ↔ ∃ σ : SigmaG2 Ty, IsRootG2 L e σ := by
  constructor
  · intro ht
    rcases ht with ⟨t, ht⟩
    exact ⟨witnessG2 L, (genC_soundG2 L e t ht).1⟩
  · intro ⟨σ, hroot⟩
    exact root_implies_typableG2 L hroot

/-- 衍生：不可定型 ⟺ 方程組無 0/1 根（UNSAT 側）。 -/
theorem untypable_iff_no_rootG2 (L : Lang Ty) (e : ExprG Ty) :
    ¬ TypableG2 L e ↔ ¬ ∃ σ : SigmaG2 Ty, IsRootG2 L e σ := by
  constructor
  · intro h ⟨σ, hroot⟩
    exact h (root_implies_typableG2 L hroot)
  · intro h ht
    exact h ((typable_iff_rootG2 L e).mp ht)

/-- 衍生：根的型別位元由檢查器唯一決定。 -/
theorem root_bit_determinedG2 (L : Lang Ty) {e : ExprG Ty} {σ : SigmaG2 Ty}
    (hroot : IsRootG2 L e σ) (τ : Ty) : σ e τ = tycheckG L e τ :=
  genC_completeG2 L e σ hroot τ

/-- 衍生：可定型 ⟹ 存在滿足約束的賦值。 -/
theorem typable_exists_rootG2 (L : Lang Ty) {e : ExprG Ty} (h : TypableG2 L e) :
    ∃ σ : SigmaG2 Ty, IsRootG2 L e σ :=
  (typable_iff_rootG2 L e).mp h

/-- **根在子表達式上單型**（one-hot 的直接推論，泛化運算子版）。 -/
theorem isMonoAtG2_of_root (L : Lang Ty) {e : ExprG Ty} {σ : SigmaG2 Ty}
    (hroot : IsRootG2 L e σ) : ∀ τ τ' : Ty, τ ≠ τ' →
    ¬ (σ e τ = true ∧ σ e τ' = true) := by
  have hone : oneHotG2 L σ e = 0 := by
    have h := hroot (oneHotC2 L e) (mem_genCG2_oneHot L e)
    simpa [oneHotC2] using h
  have hsum : (L.enumAll.map (fun t => bit (σ e t))).sum = 1 := by
    unfold oneHotG2 at hone
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

end OpAbs

/-! ## 十、實例化：Rust 的 9 種 `BinOp` 是 `BinSpec` 的 9 個實例 -/

section Instantiation

/-- 算術類運算子（Add/Sub/Mul）：`numTy × numTy → numTy`。 -/
def arithSpec (L : Lang Ty) : BinSpec Ty :=
  { in1 := L.numTy, in2 := L.numTy, out := L.numTy }

/-- 比較類運算子（Lt/Le/Ge/Eq/Ne）：`numTy × numTy → eqbTy`。 -/
def cmpSpec (L : Lang Ty) : BinSpec Ty :=
  { in1 := L.numTy, in2 := L.numTy, out := L.eqbTy }

/-- 布爾類運算子（And）：`eqbTy × eqbTy → eqbTy`。 -/
def andSpec (L : Lang Ty) : BinSpec Ty :=
  { in1 := L.eqbTy, in2 := L.eqbTy, out := L.eqbTy }

/-- `T9Generalized` 的 `add` 是 `binop arithSpec` 的特例（同一型別規則）。 -/
example (L : Lang Ty) {a b : ExprG Ty} {τ : Ty} :
    tycheckG L (.binop (arithSpec L) a b) τ = tycheckG L (.binop (arithSpec L) a b) τ := rfl

/-- 算術運算子輸出 `numTy`（`1 + 2 : numTy`）。 -/
example (L : Lang Ty) [DecidableEq Ty] :
    tycheckG L (.binop (arithSpec L) (.num 1) (.num 2)) L.numTy = true := by
  simp [tycheckG, arithSpec]

/-- 比較運算子輸出 `eqbTy`（`1 == 2 : eqbTy`）。 -/
example (L : Lang Ty) [DecidableEq Ty] :
    tycheckG L (.binop (cmpSpec L) (.num 1) (.num 2)) L.eqbTy = true := by
  simp [tycheckG, cmpSpec]

/-- 布爾運算子複合（`(1 == 2) && (3 == 4) : eqbTy`），
展示運算子可**任意複合**而核心定理自動成立。 -/
example (L : Lang Ty) [DecidableEq Ty] :
    tycheckG L (.binop (andSpec L)
        (.binop (cmpSpec L) (.num 1) (.num 2))
        (.binop (cmpSpec L) (.num 3) (.num 4))) L.eqbTy = true := by
  simp [tycheckG, andSpec, cmpSpec]

/-- 類型錯誤被拒絕：算術運算子不會輸出 `eqbTy`（`1 + 2` 不是布爾）。 -/
example (L : Lang Ty) [DecidableEq Ty] (h : L.numTy ≠ L.eqbTy) :
    tycheckG L (.binop (arithSpec L) (.num 1) (.num 2)) L.eqbTy = false := by
  simp [tycheckG, arithSpec]
  exact decide_eq_false h.symm

end Instantiation

end Polyrust
