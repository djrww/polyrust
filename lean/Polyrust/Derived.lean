/- # 衍生引理（由核心引理推出）

本模組匯集由鐵律核心命題直接推出的衍生結論：
- pair/sum/closure 可定型推論
- borrow clash 顯式 1∈理想
- isMonoAt_of_root 等區域化推論
- watchMoves 語義守恆
- clauseSat 存在刻畫

全部由核心引理推出，無 sorry。
-/

import Polyrust.T9EndToEnd
import Polyrust.T9Generalized
import Polyrust.ProductReduction
import Polyrust.SumReduction
import Polyrust.BorrowOwnership
import Polyrust.WatchMove
import Polyrust.ClauseDuality

namespace Polyrust

/-! ## 一、pair 積型衍生 -/

theorem derived_typable_pair_iff {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (a b : ProdExpr Expr) :
    TypableProd L (.pair a b) ↔
      ∃ τ₁ τ₂ : ProdTy Ty, tycheckProd L a τ₁ = true ∧ tycheckProd L b τ₂ = true :=
  typable_pair_iff L (a := a) (b := b)

theorem derived_typable_pair_fst {Ty : Type} [DecidableEq Ty]
    {L : Lang Ty} {a b : ProdExpr Expr}
    (h : TypableProd L (.pair a b)) :
    ∃ τ₁ : ProdTy Ty, tycheckProd L a τ₁ = true := by
  rcases (typable_pair_iff L).mp h with ⟨τ₁, τ₂, h₁, _⟩
  exact ⟨τ₁, h₁⟩

theorem derived_typable_pair_snd {Ty : Type} [DecidableEq Ty]
    {L : Lang Ty} {a b : ProdExpr Expr}
    (h : TypableProd L (.pair a b)) :
    ∃ τ₂ : ProdTy Ty, tycheckProd L b τ₂ = true := by
  rcases (typable_pair_iff L).mp h with ⟨τ₁, τ₂, _, h₂⟩
  exact ⟨τ₂, h₂⟩

theorem derived_pairBitSum_eq_mul {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σa σb : Ty → Bool) :
    pairBitSum L σa σb =
      (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum *
      (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum :=
  pairBitSum_eq_mul (L := L) σa σb

theorem derived_pair_oneHot_of_oneHot {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σa σb : Ty → Bool)
    (ha : (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum = 1)
    (hb : (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum = 1) :
    pairBitSum L σa σb = 1 :=
  pair_oneHot_of_oneHot (L := L) σa σb ha hb

theorem derived_pairBitSum_eq_one_imp {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σa σb : Ty → Bool)
    (h : pairBitSum L σa σb = 1) :
    (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum = 1 ∧
    (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum = 1 :=
  pairBitSum_eq_one_imp (L := L) σa σb h

/-! ## 二、sum 和型衍生 -/

theorem derived_typable_inl_iff {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (a : SumExpr Expr) :
    TypableSum L (.inl a) ↔ TypableSum L a :=
  typable_inl_iff L (a := a)

theorem derived_typable_inr_iff {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (b : SumExpr Expr) :
    TypableSum L (.inr b) ↔ TypableSum L b :=
  typable_inr_iff L (b := b)

theorem derived_type_typable_sum_iff {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (τ₁ τ₂ : SumTy Ty) :
    TypeTypable L (.sum τ₁ τ₂) ↔ TypeTypable L τ₁ ∨ TypeTypable L τ₂ :=
  type_typable_sum_iff (L := L)

theorem derived_sumBits_sum_eq_add {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σa σb : Ty → Bool) :
    (sumBits L σa σb).sum =
      (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum +
      (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum :=
  sumBits_sum_eq_add (L := L) σa σb

theorem derived_sumBits_sum_eq_two {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σa σb : Ty → Bool)
    (ha : (L.enumAll.map (fun τ₁ => bit (σa τ₁))).sum = 1)
    (hb : (L.enumAll.map (fun τ₂ => bit (σb τ₂))).sum = 1) :
    (sumBits L σa σb).sum = 2 :=
  sumBits_sum_eq_two (L := L) σa σb ha hb

theorem derived_tycheck_inl {Ty : Type} [DecidableEq Ty]
    {L : Lang Ty} {a : SumExpr Expr} {τ₁ τ₂ : SumTy Ty} :
    tycheckSum L (.inl a) (.sum τ₁ τ₂) = tycheckSum L a τ₁ :=
  tycheck_inl (L := L)

theorem derived_tycheck_inr {Ty : Type} [DecidableEq Ty]
    {L : Lang Ty} {b : SumExpr Expr} {τ₁ τ₂ : SumTy Ty} :
    tycheckSum L (.inr b) (.sum τ₁ τ₂) = tycheckSum L b τ₂ :=
  tycheck_inr (L := L)

/-! ## 三、isMonoAt 區域化衍生（由 one-hot 推出） -/

theorem derived_isMonoAt_of_root {e e' : Expr} {σ : Sigma}
    (hroot : IsRoot e σ) (h : Occurs e' e) : IsMonoAt σ e' :=
  isMonoAt_of_root hroot h

theorem derived_isMonoAt_self_of_root {e : Expr} {σ : Sigma}
    (hroot : IsRoot e σ) : IsMonoAt σ e :=
  isMonoAt_self_of_root hroot

theorem derived_isMonoAtG_of_root {Ty : Type} [DecidableEq Ty]
    {L : Lang Ty} {e : Expr} {σ : SigmaG Ty}
    (hroot : IsRootG L e σ) : ∀ τ τ' : Ty, τ ≠ τ' →
    ¬ (σ e τ = true ∧ σ e τ' = true) :=
  isMonoAtG_of_root L hroot

/-! ## 四、borrow 衍生：顯式 1∈理想 -/

theorem derived_borrow_clash_one_mem (i j : Nat) :
    BgenIdeal (clashSet i j) (fun _ => 1) :=
  borrow_clash_one_mem i j

theorem derived_borrow_assign_one_mem (n : Nat) :
    BgenIdeal (assignSet n) (fun _ => 1) :=
  borrow_assign_one_mem n

theorem derived_borrow_clash_no_root (i j : Nat) :
    ¬ ∃ β : BSign, ∀ p, clashSet i j p → p β = 0 :=
  borrow_clash_no_root i j

theorem derived_borrow_assign_no_root (n : Nat) :
    ¬ ∃ β : BSign, ∀ p, assignSet n p → p β = 0 :=
  borrow_assign_no_root n

theorem derived_use_after_move_unsat (m u : Nat) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [m, u] [(m, u)] [], c β = 0 :=
  use_after_move_unsat m u

theorem derived_assign_while_borrowed_unsat (b : Nat) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [b] [] [b], c β = 0 :=
  assign_while_borrowed_unsat b

/-! ## 五、clauseSat 衍生 -/

theorem derived_clauseSat_iff_exists (σ : Assignment) (C : List Lit) :
    clauseSat σ C = true ↔ ∃ l ∈ C, litSat σ l = true :=
  clauseSat_iff_exists σ C

theorem derived_clauseSat_all_false (σ : Assignment) (C : List Lit)
    (h : ∀ l ∈ C, litSat σ l = false) : clauseSat σ C = false :=
  clauseSat_all_false σ C h

theorem derived_clashClause_false_of_live (β : Assignment) (i j : Nat)
    (hi : β i = true) (hj : β j = true) :
    clauseSat β (clashClause i j) = false :=
  clashClause_false_of_live β i j hi hj

/-! ## 六、watch 衍生：多步守恆 -/

theorem derived_watchMove_preserves_sat (σ : Assignment) {C D : List Lit}
    (h : WatchMove C D) : clauseSat σ C = clauseSat σ D :=
  watchMove_preserves_sat σ h

theorem derived_watchMoves_preserve_sat (σ : Assignment) {C D : List Lit}
    (h : WatchMoves C D) : clauseSat σ C = clauseSat σ D :=
  watchMoves_preserve_sat σ h

theorem derived_watchMoves_sound (σ : Assignment) {C D : List Lit}
    (h : WatchMoves C D) : clauseSat σ C = true → clauseSat σ D = true :=
  watchMoves_sound σ h

/-! ## 七、T9 衍生：可定型 ↔ 根 的推論 -/

theorem derived_root_bit_determined {e : Expr} {σ : Sigma}
    (hroot : IsRoot e σ) (τ : Ty) : σ e τ = check e τ :=
  root_bit_determined hroot τ

theorem derived_typable_exists_root {e : Expr} (h : Typable e) :
    ∃ σ : Sigma, IsRoot e σ :=
  typable_exists_root h

theorem derived_untypable_iff_no_root (e : Expr) :
    ¬ Typable e ↔ ¬ ∃ σ : Sigma, IsRoot e σ :=
  untypable_iff_no_root e

theorem derived_root_implies_typableG {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) {e : Expr} {σ : SigmaG Ty}
    (hroot : IsRootG L e σ) : TypableG L e :=
  root_implies_typableG L hroot

theorem derived_root_bit_determinedG {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) {e : Expr} {σ : SigmaG Ty}
    (hroot : IsRootG L e σ) (τ : Ty) : σ e τ = tycheck L e τ :=
  root_bit_determinedG L hroot τ

/-! ## 八、P5/P6 衍生：區間差異即判定差異 -/

theorem derived_p5_conflict : conflictsWith ⟨1, 7, 1, 4⟩ ⟨2, 7, 2, 4⟩ :=
  p5_conflict

theorem derived_p6_no_conflict : ¬ conflictsWith ⟨1, 7, 1, 2⟩ ⟨2, 7, 3, 4⟩ :=
  p6_no_conflict

theorem derived_p5_unsat :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [1, 2] [(1, 2)] [], c β = 0 :=
  p5_unsat

theorem derived_p6_sat :
    ∃ β : BSign, ∀ c ∈ borrowSystem [1, 2] [] [], c β = 0 :=
  p6_sat

theorem derived_p5_p6_differ :
    (∃ b₁ ∈ p5Borrows, ∃ b₂ ∈ p5Borrows, b₁.node ≠ b₂.node ∧ conflictsWith b₁ b₂) ∧
      (∀ b₁ ∈ p6Borrows, ∀ b₂ ∈ p6Borrows, b₁.node ≠ b₂.node → ¬ conflictsWith b₁ b₂) :=
  p5_p6_differ

/-! ## 九、genC 檢查器衍生 -/

theorem derived_check_i32_false_of_bool {e : Expr}
    (h : check e Ty.boolean = true) : check e Ty.i32 = false :=
  check_i32_false_of_bool h

theorem derived_check_bool_false_of_i32 {e : Expr}
    (h : check e Ty.i32 = true) : check e Ty.boolean = false :=
  check_bool_false_of_i32 h

theorem derived_bit_and (a b : Bool) : bit (a && b) = bit a * bit b :=
  bit_and a b

end Polyrust
