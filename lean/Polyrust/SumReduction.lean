-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 和型歸約（方案 (c)）：複合型別檢查歸約為變體檢查

對應 `docs/THEOREMS.md` §11（T9）泛化方案的**第 (c) 步**：在 `T9Generalized`
的「任意可枚舉型別宇宙」之上，疊加**和型**（sum / Rust 的 `enum` 變體），
證明：

1. **檢查歸約**：`check (inl a) (sum τ₁ τ₂) = check a τ₁`、`check (inr b) (sum τ₁ τ₂) = check b τ₂`
   ——和型檢查**只檢查被選中的那個變體**；
2. **可定型歸約（主定理）**：`inl a` 可定型 ⟺ `a` 可定型；`inr b` 可定型 ⟺ `b` 可定型；
   更進一步，`sum τ₁ τ₂` 可定型 ⟺ `τ₁` 可定型 **∨** `τ₂` 可定型（OR 語義，與積型的 AND 對偶）；
3. **約束編碼歸約（代數側）**：和型節點的位元空間 = 兩個變體位元空間的**不相交並**，
   故位元總和 = 變體位元總和的**和** `Σ t_a + Σ t_b`（對偶於積型的乘積），
   新增 enum 變體時，約束編碼的證明負擔**歸約為變體的證明**。

**價值**：Rust 側 7 型別中的 `bool` 本質是「兩個零載荷變體的和型」
（`true | false`），`()` 是「單變體和型」。本模組證明「和型檢查 = 被選變體
的檢查」，因此新增 enum 變體時，只需證明變體載荷，和型**零新證明**自動歸約。

**開放和型的協變性（本模組的語義註記）**：`inl a : sum τ₁ τ₂` 只檢查載荷
`a : τ₁`，故弱化分量 `τ₂` 自由——這是開放和型（ML 風 `τ₁ + τ₂`）相對於封閉
和型（Rust `enum`）的關鍵差異；Rust 的 enum 是封閉的（變體表固定），故其型別
唯一。本模組刻意用開放和型，以凸顯「檢查 = 變體檢查」的歸約本質。

**零依賴**：同 `T9Generalized`，只用 `List`/`DecidableEq`/`omega`。 -/

import Polyrust.T9Generalized

namespace Polyrust

set_option linter.unusedSectionVars false
set_option linter.unusedVariables false

/-! ## 一、和型宇宙與和型表達式 -/

/-- 和型宇宙：基本型別（`base`）或兩個和型的變體組合（`sum`，即 `τ₁ + τ₂`）。

取**歸納閉包**（可任意嵌套）；檢查器歸約不需要枚舉，故無限深度無妨。 -/
inductive SumTy (Ty : Type)
  | base : Ty → SumTy Ty
  | sum : SumTy Ty → SumTy Ty → SumTy Ty
  deriving DecidableEq, Repr

/-- 和型表達式：基本表達式（`lift`）或左變體（`inl`）/右變體（`inr`）構造。 -/
inductive SumExpr (Expr : Type)
  | lift : Expr → SumExpr Expr
  | inl : SumExpr Expr → SumExpr Expr
  | inr : SumExpr Expr → SumExpr Expr
  deriving DecidableEq, Repr

variable {Ty : Type} [DecidableEq Ty]
variable (L : Lang Ty)

/-! ## 二、和型檢查器與檢查歸約 -/

/-- 和型檢查器：`lift e` 按基本語言的 `tycheck` 檢查，`inl a` 按 `sum τ₁ τ₂`
**只檢查左變體** `a : τ₁`，`inr b` **只檢查右變體** `b : τ₂`。 -/
def tycheckSum : SumExpr Expr → SumTy Ty → Bool
  | .lift e, .base τ => tycheck L e τ
  | .lift _, .sum _ _ => false
  | .inl a, .sum τ₁ τ₂ => tycheckSum a τ₁
  | .inl _, .base _ => false
  | .inr b, .sum τ₁ τ₂ => tycheckSum b τ₂
  | .inr _, .base _ => false

/-- 和型可定型：存在某個和型使檢查通過。 -/
def TypableSum (se : SumExpr Expr) : Prop := ∃ st : SumTy Ty, tycheckSum L se st = true

/-- 型別可定型：存在某個表達式被檢查為該型別。 -/
def TypeTypable (st : SumTy Ty) : Prop := ∃ se : SumExpr Expr, tycheckSum L se st = true

/-- **檢查歸約（T-c-inl）**：`inl a` 對 `sum τ₁ τ₂` 的檢查就是 `a` 對 `τ₁` 的檢查。 -/
theorem tycheck_inl {a : SumExpr Expr} {τ₁ τ₂ : SumTy Ty} :
    tycheckSum L (.inl a) (.sum τ₁ τ₂) = tycheckSum L a τ₁ := rfl

/-- **檢查歸約（T-c-inr）**：`inr b` 對 `sum τ₁ τ₂` 的檢查就是 `b` 對 `τ₂` 的檢查。 -/
theorem tycheck_inr {b : SumExpr Expr} {τ₁ τ₂ : SumTy Ty} :
    tycheckSum L (.inr b) (.sum τ₁ τ₂) = tycheckSum L b τ₂ := rfl

/-- **可定型歸約（主定理，左變體）**：`inl a` 可定型 ⟺ `a` 可定型。 -/
theorem typable_inl_iff {a : SumExpr Expr} :
    TypableSum L (.inl a) ↔ TypableSum L a := by
  constructor
  · intro h
    rcases h with ⟨st, h⟩
    cases st with
    | base _ => simp [tycheckSum] at h
    | sum τ₁ τ₂ => exact ⟨τ₁, by simpa [tycheckSum] using h⟩
  · intro h
    rcases h with ⟨st, h⟩
    exact ⟨.sum st st, by simpa [tycheckSum] using h⟩

/-- **可定型歸約（主定理，右變體）**：`inr b` 可定型 ⟺ `b` 可定型。 -/
theorem typable_inr_iff {b : SumExpr Expr} :
    TypableSum L (.inr b) ↔ TypableSum L b := by
  constructor
  · intro h
    rcases h with ⟨st, h⟩
    cases st with
    | base _ => simp [tycheckSum] at h
    | sum τ₁ τ₂ => exact ⟨τ₂, by simpa [tycheckSum] using h⟩
  · intro h
    rcases h with ⟨st, h⟩
    exact ⟨.sum st st, by simpa [tycheckSum] using h⟩

/-- 合成（左）：變體載荷可定型 ⟹ 和型可定型。 -/
theorem typable_sum_of_inl {a : SumExpr Expr} (h : TypableSum L a) : TypableSum L (.inl a) :=
  (typable_inl_iff L).mpr h

/-- 合成（右）：變體載荷可定型 ⟹ 和型可定型。 -/
theorem typable_sum_of_inr {b : SumExpr Expr} (h : TypableSum L b) : TypableSum L (.inr b) :=
  (typable_inr_iff L).mpr h

/-- **可定型的 OR 語義（主定理）**：`sum τ₁ τ₂` 可定型 ⟺ `τ₁` 可定型 **∨** `τ₂`
可定型。這與積型的「AND 語義」（`typable_pair_iff`）形成精確對偶：
積型要**兩個**分量都可定型，和型只需**某一個**變體可定型。 -/
theorem type_typable_sum_iff {τ₁ τ₂ : SumTy Ty} :
    TypeTypable L (.sum τ₁ τ₂) ↔ TypeTypable L τ₁ ∨ TypeTypable L τ₂ := by
  constructor
  · intro h
    rcases h with ⟨se, h⟩
    cases se with
    | lift e => simp [tycheckSum] at h
    | inl a => left; exact ⟨a, by simpa [tycheckSum] using h⟩
    | inr b => right; exact ⟨b, by simpa [tycheckSum] using h⟩
  · intro h
    rcases h with h₁ | h₂
    · rcases h₁ with ⟨a, ha⟩
      exact ⟨.inl a, by simpa [tycheckSum] using ha⟩
    · rcases h₂ with ⟨b, hb⟩
      exact ⟨.inr b, by simpa [tycheckSum] using hb⟩

/-! ## 三、約束編碼歸約：和型 one-hot = 變體 one-hot 之和 -/

/-- 和型節點的位元列表：左變體位元空間 `++` 右變體位元空間（**不相交並**）。

對偶於積型（`ProductReduction.pairBitSum` 用 `flatMap` 做笛卡爾**積**），
和型用 `++` 做不相交**和**。 -/
def sumBits (σa σb : Ty → Bool) : List Int :=
  (L.enumAll.map (fun τ₁ => bit (σa τ₁))) ++ (L.enumAll.map (fun τ₂ => bit (σb τ₂)))

/-- **代數歸約（主定理）**：和型位元總和 = 變體位元總和的**和**
`Σ t_a + Σ t_b`（`List.sum_append`）。 -/
theorem sumBits_sum_eq_add (σa σb : Ty → Bool) :
    (sumBits L σa σb).sum =
      (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum + (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum := by
  unfold sumBits
  exact List.sum_append

/-- **變體 one-hot ⟹ 和型位元總和 = 變體數**：若 a、b 的位元總和各為 1，
則和型（二元）的位元總和為 2——新增 enum 變體時，one-hot 約束**零新證明**
自動歸約（只需 `List.sum_append`）。 -/
theorem sumBits_sum_eq_two (σa σb : Ty → Bool)
    (ha : (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum = 1)
    (hb : (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum = 1) :
    (sumBits L σa σb).sum = 2 := by
  rw [sumBits_sum_eq_add, ha, hb]
  omega

/-- 和型位元總和非負（兩個非負分量之和）。 -/
theorem sumBits_nonneg (σa σb : Ty → Bool) : 0 ≤ (sumBits L σa σb).sum := by
  rw [sumBits_sum_eq_add]
  exact Int.add_nonneg
    (listSum_nonneg (fun x hx => bit_nonneg (σa x)))
    (listSum_nonneg (fun x hx => bit_nonneg (σb x)))

end Polyrust
