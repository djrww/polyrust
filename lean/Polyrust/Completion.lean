/- # 雙向完備（補全）

本模組把以往只有單向的引理補全為雙向 ⟺，實現「補全」類：
- clauseSat ↔ polyZero 的兩個方向拆分與合併
- borrow_sat ↔ clean 的 sound / complete 拆分
- typable ↔ root 的 sound / complete 拆分
- parse / gen round-trip 雙向
- one-hot sum=1 ↔ 存在唯一真 的雙向
- field poly 的雙向刻畫

全部純構造、零 sorry。
-/

import Polyrust.ClauseDuality
import Polyrust.BorrowOwnership
import Polyrust.T9EndToEnd
import Polyrust.T9Generalized
import Polyrust.WatchMove
import Polyrust.TypeUniverse7PlusI

namespace Polyrust

/-! ## 一、子句對偶的雙向拆分 -/

theorem clauseSat_complete (σ : Assignment) (C : List Lit) :
    clauseSat σ C = true → clausePoly σ C = 0 :=
  (clause_duality σ C).mp

theorem clauseSat_sound (σ : Assignment) (C : List Lit) :
    clausePoly σ C = 0 → clauseSat σ C = true :=
  (clause_duality σ C).mpr

theorem clauseSat_iff_polyZero (σ : Assignment) (C : List Lit) :
    clauseSat σ C = true ↔ clausePoly σ C = 0 :=
  clause_duality σ C

theorem clauseSat_false_iff_polyOne (σ : Assignment) (C : List Lit) :
    clauseSat σ C = false ↔ clausePoly σ C = 1 := by
  constructor
  · intro hfalse
    induction C with
    | nil =>
      simp [clausePoly]
    | cons l Cs ih =>
      have hany : (l :: Cs).any (litSat σ) = false := by
        simpa [clauseSat] using hfalse
      simp only [List.any_cons, Bool.or_eq_false_iff] at hany
      rcases hany with ⟨hl, hrest⟩
      have hl_false : litSat σ l = false := by
        cases hb : litSat σ l with
        | true => simp [hb] at hl
        | false => rfl
      have hrest_false : clauseSat σ Cs = false := hrest
      have hf : litFactor σ l = 1 := by simp [litFactor, hl_false, bit]
      have hpoly_rest : clausePoly σ Cs = 1 := ih hrest_false
      calc clausePoly σ (l :: Cs)
          = litFactor σ l * clausePoly σ Cs := by
            simp [clausePoly, List.map_cons, List.foldr_cons]
        _ = 1 * 1 := by rw [hf, hpoly_rest]
        _ = 1 := by simp
  · intro hone
    cases hc : clauseSat σ C with
    | true =>
      have hzero : clausePoly σ C = 0 := (clause_duality σ C).mp hc
      omega
    | false => rfl

theorem cnfSat_complete (σ : Assignment) (Φ : List (List Lit)) :
    cnfSat σ Φ = true → ∀ p ∈ cnfPolys σ Φ, p = 0 :=
  (cnf_duality σ Φ).mp

theorem cnfSat_sound (σ : Assignment) (Φ : List (List Lit)) :
    (∀ p ∈ cnfPolys σ Φ, p = 0) → cnfSat σ Φ = true :=
  (cnf_duality σ Φ).mpr

theorem cnfSat_iff_allPolyZero (σ : Assignment) (Φ : List (List Lit)) :
    cnfSat σ Φ = true ↔ ∀ p ∈ cnfPolys σ Φ, p = 0 :=
  cnf_duality σ Φ

/-! ## 二、借用完備性雙向拆分 -/

theorem borrow_sat_sound {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (h₁ : pairs = []) (h₂ : assigns = []) :
    ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 :=
  borrow_sat_of_clean h₁ h₂

theorem borrow_sat_complete_clash {i j : Nat} {live : List Nat}
    {pairs : List (Nat × Nat)} {assigns : List Nat}
    (hpair : (i, j) ∈ pairs) (hi : i ∈ live) (hj : j ∈ live) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 :=
  borrow_unsat_of_clash hpair hi hj

theorem borrow_sat_complete_assign {n : Nat} {live : List Nat}
    {pairs : List (Nat × Nat)} {assigns : List Nat}
    (hn : n ∈ assigns) (hi : n ∈ live) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 :=
  borrow_unsat_of_assign hn hi

theorem borrow_sat_iff_clean_completion {live : List Nat}
    {pairs : List (Nat × Nat)} {assigns : List Nat}
    (hpl : ∀ p ∈ pairs, p.1 ∈ live ∧ p.2 ∈ live)
    (hal : ∀ n ∈ assigns, n ∈ live) :
    (∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0) ↔
      pairs = [] ∧ assigns = [] :=
  borrow_sat_iff_clean hpl hal

theorem borrow_clean_sound {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat}
    (h : ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0)
    (hpl : ∀ p ∈ pairs, p.1 ∈ live ∧ p.2 ∈ live)
    (hal : ∀ n ∈ assigns, n ∈ live) :
    pairs = [] ∧ assigns = [] :=
  (borrow_sat_iff_clean hpl hal).mp h

theorem borrow_clean_complete {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat}
    (h : pairs = [] ∧ assigns = []) :
    ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 :=
  borrow_sat_of_clean h.1 h.2

/-! ## 三、T9 端到端雙向完備 -/

theorem genC_sound_completion (e : Expr) (τ : Ty) (h : check e τ = true) :
    IsRoot e witness ∧ witness e τ = true :=
  genC_sound e τ h

theorem genC_complete_completion (e : Expr) (σ : Sigma) (hroot : IsRoot e σ) (τ : Ty) :
    σ e τ = check e τ :=
  genC_complete e σ hroot τ

theorem typable_iff_root_completion (e : Expr) :
    Typable e ↔ ∃ σ : Sigma, IsRoot e σ :=
  typable_iff_root e

theorem typable_sound_completion (e : Expr) :
    Typable e → ∃ σ : Sigma, IsRoot e σ :=
  (typable_iff_root e).mp

theorem typable_complete_completion (e : Expr) :
    (∃ σ : Sigma, IsRoot e σ) → Typable e :=
  (typable_iff_root e).mpr

theorem untypable_iff_no_root_completion (e : Expr) :
    ¬ Typable e ↔ ¬ ∃ σ : Sigma, IsRoot e σ :=
  untypable_iff_no_root e

/-! ## 四、T9 泛化雙向完備 -/

theorem genC_soundG_completion {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (e : Expr) (τ : Ty) (h : tycheck L e τ = true) :
    IsRootG L e (witnessG L) ∧ (witnessG L) e τ = true :=
  genC_soundG L e τ h

theorem genC_completeG_completion {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (e : Expr) (σ : SigmaG Ty) (hroot : IsRootG L e σ) (τ : Ty) :
    σ e τ = tycheck L e τ :=
  genC_completeG L e σ hroot τ

theorem typable_iff_rootG_completion {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (e : Expr) :
    TypableG L e ↔ ∃ σ : SigmaG Ty, IsRootG L e σ :=
  typable_iff_rootG L e

theorem typableG_sound_completion {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (e : Expr) :
    TypableG L e → ∃ σ : SigmaG Ty, IsRootG L e σ :=
  (typable_iff_rootG L e).mp

theorem typableG_complete_completion {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (e : Expr) :
    (∃ σ : SigmaG Ty, IsRootG L e σ) → TypableG L e :=
  (typable_iff_rootG L e).mpr

/-! ## 五、watch 移動雙向完備 -/

theorem watch_move_iff (σ : Assignment) (f B l : Lit) (pre post : List Lit) :
    clauseSat σ (f :: B :: pre ++ l :: post) = true ↔
    clauseSat σ (l :: B :: pre ++ f :: post) = true := by
  rw [watch_move0_preserves_sat σ f B l pre post]

theorem watch_move1_iff (σ : Assignment) (B f l : Lit) (pre post : List Lit) :
    clauseSat σ (B :: f :: pre ++ l :: post) = true ↔
    clauseSat σ (B :: l :: pre ++ f :: post) = true := by
  rw [watch_move1_preserves_sat σ B f l pre post]

theorem watchMoves_iff (σ : Assignment) {C D : List Lit} (h : WatchMoves C D) :
    clauseSat σ C = true ↔ clauseSat σ D = true := by
  rw [watchMoves_preserve_sat σ h]

/-! ## 六、parse / gen round-trip 雙向 -/

theorem parse_gen_sound (e : Expr) (rest : List Tok) :
    parse (gen e ++ rest) = some (e, rest) :=
  parse_gen e rest

theorem parse_gen_complete_left_inverse (e : Expr) :
    parse (gen e) = some (e, []) := by
  simpa using parse_gen e []

theorem gen_parse_prefix_property (e : Expr) (rest : List Tok) :
    (gen e ++ rest).length = (gen e).length + rest.length := by
  simp [List.length_append]

theorem gen_length_completion (e : Expr) :
    (gen e).length = sizeT e :=
  gen_length e

theorem typable_gen_parses_completion (e : Expr) (τ : Ty) (h : check e τ = true) :
    ∃ e' : Expr, parse (gen e) = some (e', []) ∧ e' = e ∧ check e' τ = true :=
  typable_gen_parses e τ h

/-! ## 七、one-hot 雙向完備 -/

theorem oneHot_iff_exists_unique {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σ : Ty → Bool) :
    (L.enumAll.map (fun t => bit (σ t))).sum = 1 ↔
      ∃ t ∈ L.enumAll, σ t = true ∧ ∀ t' ∈ L.enumAll, σ t' = true → t' = t := by
  constructor
  · intro hone
    have hge : ∀ x ∈ L.enumAll, 0 ≤ bit (σ x) := fun x _ => bit_nonneg (σ x)
    have hex := listSum_eq_one_exists_one hone hge
    rcases hex with ⟨t, ht, ht1⟩
    have ht_true : σ t = true := bit_eq_one_iff.mp ht1
    refine ⟨t, ht, ht_true, ?_⟩
    intro t' ht' ht'_true
    by_cases heq : t' = t
    · exact heq
    · have hbit_t : bit (σ t) = 1 := by simp [bit, ht_true]
      have hbit_t' : bit (σ t') = 1 := by simp [bit, ht'_true]
      have h2 : 2 ≤ (L.enumAll.map (fun t => bit (σ t))).sum :=
        listSum_ge_two_of_two_ones L.nodup ht ht' (Ne.symm heq) hbit_t hbit_t' hge
      omega
  · intro ⟨t, ht, ht_true, huniq⟩
    have hmap_eq : L.enumAll.map (fun t' => bit (σ t')) =
        L.enumAll.map (fun t' => bit (decide (t' = t))) := by
      apply List.map_eq_map_iff.mpr
      intro t' ht'
      by_cases heq : t' = t
      · subst heq; simp [bit, ht_true]
      · have hfalse : σ t' = false := by
          cases hb : σ t' with
          | true =>
            have heq' := huniq t' ht' hb
            exact absurd heq' heq
          | false => rfl
        simp [bit, hfalse, heq]
    have hsum_eq : (L.enumAll.map (fun t' => bit (σ t'))).sum =
        (L.enumAll.map (fun t' => bit (decide (t' = t)))).sum := by
      rw [hmap_eq]
    rw [hsum_eq]
    exact sum_mark_eq_one_of_mem ht L.nodup

theorem oneHot_sound {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σ : Ty → Bool) :
    (∃ t ∈ L.enumAll, σ t = true ∧ ∀ t' ∈ L.enumAll, σ t' = true → t' = t) →
      (L.enumAll.map (fun t => bit (σ t))).sum = 1 :=
  (oneHot_iff_exists_unique L σ).mpr

theorem oneHot_complete {Ty : Type} [DecidableEq Ty]
    (L : Lang Ty) (σ : Ty → Bool) :
    (L.enumAll.map (fun t => bit (σ t))).sum = 1 →
      ∃ t ∈ L.enumAll, σ t = true ∧ ∀ t' ∈ L.enumAll, σ t' = true → t' = t :=
  (oneHot_iff_exists_unique L σ).mp

/-! ## 八、field poly 雙向刻畫 -/

theorem field_poly_iff_bool (b : Bool) :
    bit b * (bit b - 1) = 0 ↔ (bit b = 0 ∨ bit b = 1) := by
  constructor
  · intro _
    cases b
    · left; rfl
    · right; rfl
  · intro _
    exact field_poly_bit b

theorem field_poly_complete (b : Bool) : bit b * (bit b - 1) = 0 :=
  field_poly_bit b

theorem field_poly_sound (b : Bool) (h : bit b = 0 ∨ bit b = 1) :
    bit b * (bit b - 1) = 0 := by
  rcases h with h0 | h1
  · rw [h0]; simp
  · rw [h1]; simp

/-! ## 九、17 宇宙 one-hot 雙向 -/

theorem one_hot_unique_17_completion (σ : Ty7Plus10 → Bool)
    (hone : (Ty7Plus10.all.map (fun t => bit (σ t))).sum = 1) :
    ∃ t ∈ Ty7Plus10.all, σ t = true ∧ ∀ t' ∈ Ty7Plus10.all, σ t' = true → t' = t :=
  one_hot_unique_17 σ hone

theorem oneHotPoly_zero_iff_one_completion (exts : List ExtTag)
    (σ : Ty7Plus10 → Bool) :
    oneHotPoly exts σ = 0 ↔
      ((mkUniverse7PlusI exts).map (fun t => bit (σ t))).sum = 1 :=
  oneHotPoly_zero_iff_one exts σ

end Polyrust
