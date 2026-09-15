/- # 積型歸約（方案 (b)）：複合型別檢查歸約為逐欄位檢查

對應 `docs/THEOREMS.md` §11（T9）泛化方案的**第 (b) 步**：在 `T9Generalized`
的「任意可枚舉型別宇宙」之上，疊加**積型**（product / Rust 的 `struct` 欄位），
證明：

1. **檢查歸約**：`check (pair a b) (prod τ₁ τ₂) = check a τ₁ && check b τ₂`
   ——積型檢查**逐欄位**進行；
2. **可定型歸約（主定理）**：`pair a b` 可定型 ⟺ 兩個分量**各自**可定型
   ——複合值的可定型性**完全由分量決定**；
3. **約束編碼歸約（代數側）**：pair 節點的 one-hot 位元總和等於分量位元
   總和的**乘積** `(Σ t_a)·(Σ t_b)`，故「分量 one-hot ⟹ pair 自動 one-hot」
   ——新增積型欄位時，約束編碼的證明負擔**歸約為欄位的證明**。

**價值**：Rust 側 7 型別中有 4 個是引用型別（`&i32`/`&mut i32`/`&bool`/`&mut bool`），
本質是「引用構造器 ∘ 基本型別」的複合。本模組證明「複合型別的檢查 = 組成部分
的檢查」，因此新增複合型別時，只需證明其組成部分，複合型別**零新證明**自動歸約。

**零依賴**：同 `T9Generalized`，只用 `List`/`DecidableEq`/`omega`。 -/

import Polyrust.T9Generalized

set_option linter.unusedSectionVars false

namespace Polyrust

/-! ## 一、積型宇宙與積型表達式 -/

/-- 積型宇宙：基本型別（`base`）或兩個積型的分量組合（`prod`）。

取**歸納閉包**（可任意嵌套），比「一層積型」更一般；檢查器歸約不需要枚舉，
故無限深度無妨。 -/
inductive ProdTy (Ty : Type)
  | base : Ty → ProdTy Ty
  | prod : ProdTy Ty → ProdTy Ty → ProdTy Ty
  deriving DecidableEq, Repr

/-- 積型表達式：基本表達式（`lift`）或一對分量的構造（`pair`）。 -/
inductive ProdExpr (Expr : Type)
  | lift : Expr → ProdExpr Expr
  | pair : ProdExpr Expr → ProdExpr Expr → ProdExpr Expr
  deriving DecidableEq, Repr

variable {Ty : Type} [DecidableEq Ty]
variable (L : Lang Ty)

/-! ## 二、積型檢查器與檢查歸約 -/

/-- 積型檢查器：`lift e` 按基本語言的 `tycheck` 檢查，`pair a b` 按
`prod τ₁ τ₂` **逐欄位**檢查分量。 -/
def tycheckProd : ProdExpr Expr → ProdTy Ty → Bool
  | .lift e, .base τ => tycheck L e τ
  | .lift _, .prod _ _ => false
  | .pair a b, .prod τ₁ τ₂ => tycheckProd a τ₁ && tycheckProd b τ₂
  | .pair _ _, .base _ => false

/-- 積型可定型：存在某個積型使檢查通過。 -/
def TypableProd (pe : ProdExpr Expr) : Prop := ∃ pt : ProdTy Ty, tycheckProd L pe pt = true

/-- **檢查歸約（T-b）**：`pair` 的檢查就是分量檢查的逐欄位合取（定義即此）。 -/
theorem tycheck_pair {a b : ProdExpr Expr} {τ₁ τ₂ : ProdTy Ty} :
    tycheckProd L (.pair a b) (.prod τ₁ τ₂) = (tycheckProd L a τ₁ && tycheckProd L b τ₂) := rfl

/-- **可定型歸約（主定理，T-b 核心）**：`pair a b` 可定型 ⟺ 存在分量型別
使 `a`、`b` **各自**可定型——複合值可定型性**完全由分量決定**。 -/
theorem typable_pair_iff {a b : ProdExpr Expr} :
    TypableProd L (.pair a b) ↔
      ∃ τ₁ τ₂ : ProdTy Ty, tycheckProd L a τ₁ = true ∧ tycheckProd L b τ₂ = true := by
  constructor
  · intro h
    rcases h with ⟨pt, h⟩
    cases pt with
    | base τ => simp [tycheckProd] at h
    | prod τ₁ τ₂ =>
      simp [tycheckProd] at h
      exact ⟨τ₁, τ₂, h.1, h.2⟩
  · intro ⟨τ₁, τ₂, h₁, h₂⟩
    exact ⟨.prod τ₁ τ₂, by simp [tycheckProd, h₁, h₂]⟩

/-- 衍生：pair 可定型 ⟹ 左分量在某型別可定型。 -/
theorem typable_pair_fst {a b : ProdExpr Expr} (h : TypableProd L (.pair a b)) :
    ∃ τ₁ : ProdTy Ty, tycheckProd L a τ₁ = true := by
  rcases (typable_pair_iff L).mp h with ⟨τ₁, τ₂, h₁, _⟩
  exact ⟨τ₁, h₁⟩

/-- 衍生：pair 可定型 ⟹ 右分量在某型別可定型。 -/
theorem typable_pair_snd {a b : ProdExpr Expr} (h : TypableProd L (.pair a b)) :
    ∃ τ₂ : ProdTy Ty, tycheckProd L b τ₂ = true := by
  rcases (typable_pair_iff L).mp h with ⟨τ₁, τ₂, _, h₂⟩
  exact ⟨τ₂, h₂⟩

/-- **積型單型性**：任一積型表達式至多被檢查為一個型別（由基本語言的
單型性 `tycheck_exclusive` 與 `prod`/`base` 構造子互異推出）。 -/
theorem tycheckProd_exclusive : ∀ (pe : ProdExpr Expr), ∀ τ τ' : ProdTy Ty, τ ≠ τ' →
    ¬ (tycheckProd L pe τ = true ∧ tycheckProd L pe τ' = true) := by
  intro pe
  induction pe with
  | lift e =>
    intro τ τ' hne ⟨h₁, h₂⟩
    cases τ with
    | base t1 =>
      cases τ' with
      | base t2 =>
        have he : t1 ≠ t2 := by intro h; exact hne (by simp [h])
        have hx := tycheck_exclusive L e t1 t2 he
        exact hx ⟨by simpa [tycheckProd] using h₁, by simpa [tycheckProd] using h₂⟩
      | prod _ _ => simp [tycheckProd] at h₂
    | prod _ _ =>
      simp [tycheckProd] at h₁
  | pair a b iha ihb =>
    intro τ τ' hne ⟨h₁, h₂⟩
    cases τ with
    | base _ => simp [tycheckProd] at h₁
    | prod τ₁ τ₂ =>
      cases τ' with
      | base _ => simp [tycheckProd] at h₂
      | prod τ₁' τ₂' =>
        simp only [tycheckProd, Bool.and_eq_true_iff] at h₁ h₂
        -- τ₁ ≠ τ₁' 或 τ₂ ≠ τ₂'（由 prod 構造子互異）
        have hcomp : τ₁ ≠ τ₁' ∨ τ₂ ≠ τ₂' := by
          by_cases h1 : τ₁ = τ₁'
          · right
            intro h2
            apply hne
            rw [h1, h2]
          · left
            exact h1
        cases hcomp with
        | inl h1 => exact iha τ₁ τ₁' h1 ⟨h₁.1, h₂.1⟩
        | inr h2 => exact ihb τ₂ τ₂' h2 ⟨h₁.2, h₂.2⟩

/-! ## 三、約束編碼歸約：pair 的 one-hot = 分量 one-hot 的乘積 -/

/-- 純代數分配律：`(l.map (fun y => a * f y)).sum = a * (l.map f).sum`。 -/
theorem sum_map_mul_left {α : Type} {f : α → Int} {l : List α} (a : Int) :
    (l.map (fun y => a * f y)).sum = a * (l.map f).sum := by
  induction l with
  | nil => simp
  | cons b l ih =>
    simp only [List.map_cons, List.sum_cons]
    rw [ih]
    exact (Int.mul_add a (f b) (l.map f).sum).symm

/-- **雙重求和 = 求和乘積**：笛卡爾積（`flatMap`）上的元素乘積求和，
等於兩個分量求和的乘積——積型 one-hot 歸約的代數核心。 -/
theorem sum_mul_sum {α β : Type} {f : α → Int} {g : β → Int} {l₁ : List α} {l₂ : List β} :
    (l₁.flatMap (fun x => l₂.map (fun y => f x * g y))).sum = (l₁.map f).sum * (l₂.map g).sum := by
  induction l₁ with
  | nil => simp [List.flatMap]
  | cons a l₁ ih =>
    simp only [List.flatMap_cons, List.sum_append, ih]
    rw [sum_map_mul_left]
    exact (Int.add_mul (f a) (l₁.map f).sum (l₂.map g).sum).symm

/-- pair 節點對分量型別 `(τ₁, τ₂)` 的位元，由分量位元 `σa τ₁ · σb τ₂` 給出
（積型規則方程 `t_{pair,τ₁τ₂} = t_{a,τ₁} · t_{b,τ₂}` 的位元形式）。 -/
def pairBit (σa σb : Ty → Bool) (τ₁ τ₂ : Ty) : Int := bit (σa τ₁) * bit (σb τ₂)

/-- pair 節點的**位元總和**（對所有 `(τ₁, τ₂) ∈ enumAll × enumAll` 求和）。 -/
def pairBitSum (σa σb : Ty → Bool) : Int :=
  (L.enumAll.flatMap (fun τ₁ => L.enumAll.map (fun τ₂ => pairBit σa σb τ₁ τ₂))).sum

/-- **one-hot 乘積歸約（主定理，約束側）**：pair 的位元總和等於分量位元
總和的**乘積** `(Σ t_a)·(Σ t_b)`。 -/
theorem pairBitSum_eq_mul (σa σb : Ty → Bool) :
    pairBitSum L σa σb =
      (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum * (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum := by
  unfold pairBitSum pairBit
  exact sum_mul_sum

/-- **分量 one-hot ⟹ pair 自動 one-hot**：若 a、b 的位元總和各為 1，
則 pair 的位元總和為 1——新增積型欄位時，one-hot 約束**零新證明**自動歸約。 -/
theorem pair_oneHot_of_oneHot (σa σb : Ty → Bool)
    (ha : (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum = 1)
    (hb : (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum = 1) :
    pairBitSum L σa σb = 1 := by
  rw [pairBitSum_eq_mul, ha, hb]
  omega

/-- 衍生：pair 位元總和非負（分量位元非負的乘積）。 -/
theorem pairBitSum_nonneg (σa σb : Ty → Bool) : 0 ≤ pairBitSum L σa σb := by
  rw [pairBitSum_eq_mul]
  exact Int.mul_nonneg
    (listSum_nonneg (fun x hx => bit_nonneg (σa x)))
    (listSum_nonneg (fun x hx => bit_nonneg (σb x)))

/-- 衍生（反向）：pair 位元總和為 1 ⟹ 分量位元總和**都是** 1
（非負整數乘積 = 1 ⟹ 兩因子 = 1）。 -/
theorem pairBitSum_eq_one_imp (σa σb : Ty → Bool) (h : pairBitSum L σa σb = 1) :
    (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum = 1 ∧
    (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum = 1 := by
  rw [pairBitSum_eq_mul] at h
  have hA : 0 ≤ (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum :=
    listSum_nonneg (fun x hx => bit_nonneg (σa x))
  have hB : 0 ≤ (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum :=
    listSum_nonneg (fun x hx => bit_nonneg (σb x))
  exact ⟨Int.eq_one_of_mul_eq_one_right hA h, Int.eq_one_of_mul_eq_one_left hB h⟩

/-- **積型判定歸約**：pair 語句的「可定型」可由分量「可定型」**合成**，
且反向亦然——把積型語言的判定完全歸約到基本語言（配合 `typable_pair_iff`
與基本語言的 `typable_iff_rootG`）。 -/
theorem typable_pair_from_components {a b : ProdExpr Expr}
    (ha : TypableProd L a) (hb : TypableProd L b) : TypableProd L (.pair a b) := by
  rcases ha with ⟨τ₁, h₁⟩
  rcases hb with ⟨τ₂, h₂⟩
  exact (typable_pair_iff L).mpr ⟨τ₁, τ₂, h₁, h₂⟩

end Polyrust
