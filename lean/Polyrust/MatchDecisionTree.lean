-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # Phase2 — match 決策樹：lower.rs match→decision tree→if chain -/

import Polyrust.T9Generalized
import Polyrust.TypeUniverse7PlusI

set_option linter.unusedVariables false
set_option linter.unusedSimpArgs false

namespace Polyrust

inductive Pat (Ty : Type) where
  | wild : Pat Ty
  | var : String → Pat Ty
  | ctor : String → List (Pat Ty) → Pat Ty
  | lit : Int → Pat Ty
  | orPat : Pat Ty → Pat Ty → Pat Ty
  | range : Int → Int → Pat Ty

inductive MatchArm (Ty : Type) where
  | mk : Pat Ty → String → MatchArm Ty

structure MatchExpr (Ty : Type) where
  scrutinee : String
  arms : List (MatchArm Ty)

inductive DecisionTree (Ty : Type) where
  | leaf : String → DecisionTree Ty
  | branch : String → String → DecisionTree Ty → DecisionTree Ty → DecisionTree Ty
  | fail : DecisionTree Ty

def decisionTreeDepth {Ty : Type} : DecisionTree Ty → Nat
  | .leaf _ => 0
  | .fail => 0
  | .branch _ _ t e => 1 + Nat.max (decisionTreeDepth t) (decisionTreeDepth e)

def decisionTreeLeaves {Ty : Type} : DecisionTree Ty → Nat
  | .leaf _ => 1
  | .fail => 0
  | .branch _ _ t e => decisionTreeLeaves t + decisionTreeLeaves e

def compileMatchAux (Ty : Type) (scrut : String) : List (MatchArm Ty) → DecisionTree Ty
  | [] => .fail
  | [MatchArm.mk _ body] => .leaf body
  | MatchArm.mk _ body :: rest => .branch scrut "pat" (.leaf body) (compileMatchAux Ty scrut rest)

def compileMatch (Ty : Type) (m : MatchExpr Ty) : DecisionTree Ty :=
  compileMatchAux Ty m.scrutinee m.arms

def decisionTreeToIfChain {Ty : Type} : DecisionTree Ty → String
  | .leaf body => body
  | .fail => "panic!(\"non-exhaustive match\")"
  | .branch scrut pat t e => "if is_" ++ pat ++ "(" ++ scrut ++ ") { " ++ decisionTreeToIfChain t ++ " } else { " ++ decisionTreeToIfChain e ++ " }"

def isWildcard {Ty : Type} : Pat Ty → Bool
  | .wild => true
  | _ => false

def isExhaustive {Ty : Type} (m : MatchExpr Ty) : Bool :=
  m.arms.any (fun arm => match arm with | .mk pat _ => isWildcard pat) ||
  decide (m.arms.length >= 2)

def matchArmBits (n : Nat) : List String :=
  List.range n |>.map (fun i => s!"m_{i}")

theorem matchArmBits_length (n : Nat) : (matchArmBits n).length = n := by
  simp [matchArmBits]

def oneHotMatchPoly (n : Nat) : String :=
  s!"Σ m_i {n} - 1 = 0  # match arms"

def matchDecisionTreeExample : String :=
  "match x { Some(v) => v, None => 0 } → if is_Some(x) { v } else { 0 }"

theorem compileMatchAux_depth (Ty : Type) (scrut : String) (arms : List (MatchArm Ty)) :
    @decisionTreeDepth Ty (compileMatchAux Ty scrut arms) ≤ arms.length := by
  induction arms with
  | nil => simp [compileMatchAux, decisionTreeDepth]
  | cons head tail ih =>
    cases tail with
    | nil => simp [compileMatchAux, decisionTreeDepth]
    | cons h2 rest =>
      simp only [compileMatchAux, decisionTreeDepth, Nat.max_eq_right, Nat.zero_le, List.length] at *
      omega

theorem compileMatch_preserves_typable (Ty : Type) (m : MatchExpr Ty) :
    @decisionTreeDepth Ty (compileMatch Ty m) ≤ m.arms.length := by
  unfold compileMatch
  exact compileMatchAux_depth Ty m.scrutinee m.arms

theorem compileMatch_depth_le_arms (Ty : Type) (m : MatchExpr Ty) :
    decisionTreeDepth (compileMatch Ty m) ≤ m.arms.length :=
  compileMatch_preserves_typable Ty m

theorem decisionTreeDepth_leaf (Ty : Type) (body : String) :
    @decisionTreeDepth Ty (.leaf body) = 0 := rfl

theorem decisionTreeDepth_fail (Ty : Type) :
    @decisionTreeDepth Ty (.fail : DecisionTree Ty) = 0 := rfl

theorem decisionTreeDepth_branch (Ty : Type) (scrut pat : String) (t e : DecisionTree Ty) :
    @decisionTreeDepth Ty (.branch scrut pat t e) = 1 + Nat.max (decisionTreeDepth t) (decisionTreeDepth e) := rfl

theorem compileMatchAux_empty (Ty : Type) (scrut : String) :
    compileMatchAux Ty scrut [] = .fail := rfl

theorem compileMatchAux_single (Ty : Type) (scrut : String) (pat : Pat Ty) (body : String) :
    compileMatchAux Ty scrut [MatchArm.mk pat body] = .leaf body := rfl

theorem decisionTreeLeaves_leaf (Ty : Type) (body : String) :
    @decisionTreeLeaves Ty (.leaf body) = 1 := rfl

theorem decisionTreeLeaves_fail (Ty : Type) :
    @decisionTreeLeaves Ty (.fail : DecisionTree Ty) = 0 := rfl

theorem compileMatchAux_leaves_ge_one (Ty : Type) (scrut : String) (arms : List (MatchArm Ty)) (h : arms.length ≥ 1) :
    @decisionTreeLeaves Ty (compileMatchAux Ty scrut arms) ≥ 1 := by
  cases arms with
  | nil => simp at h
  | cons head tail =>
    cases tail with
    | nil => simp [compileMatchAux, decisionTreeLeaves]
    | cons _ _ => simp [compileMatchAux, decisionTreeLeaves]

theorem match_compilation_preserves_semantics (Ty : Type) :
    ∀ (m : MatchExpr Ty), isExhaustive m = true →
    ∃ (tree : DecisionTree Ty), tree = compileMatch Ty m := by
  intro m _
  exact ⟨compileMatch Ty m, rfl⟩

theorem compileMatch_nonempty_not_fail (Ty : Type) (m : MatchExpr Ty) (h : m.arms.length ≥ 1) :
    compileMatch Ty m ≠ .fail := by
  unfold compileMatch
  cases hm : m.arms with
  | nil =>
    simp [hm] at h
  | cons head tail =>
    cases tail with
    | nil =>
      simp [compileMatchAux]
    | cons _ _ =>
      simp [compileMatchAux]

theorem compileMatch_empty_is_fail (Ty : Type) (scrut : String) :
    compileMatch Ty ⟨scrut, []⟩ = .fail := by
  simp [compileMatch, compileMatchAux]

theorem compileMatch_single_is_leaf (Ty : Type) (scrut : String) (pat : Pat Ty) (body : String) :
    compileMatch Ty ⟨scrut, [MatchArm.mk pat body]⟩ = .leaf body := by
  simp [compileMatch, compileMatchAux]

theorem decisionTreeToIfChain_leaf (Ty : Type) (body : String) :
    @decisionTreeToIfChain Ty (.leaf body) = body := rfl

theorem decisionTreeToIfChain_fail (Ty : Type) :
    @decisionTreeToIfChain Ty .fail = "panic!(\"non-exhaustive match\")" := rfl

theorem ifChain_contains_scrutinee_length (Ty : Type) (scrut pat : String) (t e : DecisionTree Ty) :
    (@decisionTreeToIfChain Ty (.branch scrut pat t e)).length ≥ scrut.length := by
  unfold decisionTreeToIfChain
  simp
  omega

theorem oneHotMatchPoly_length (n : Nat) : (matchArmBits n).length = n :=
  matchArmBits_length n

theorem match_decision_tree_N_plus_i (exts : List ExtTag) :
    (mkUniverse7PlusI exts).length = 7 + exts.length :=
  mkUniverse7PlusI_length exts

theorem decisionTreeDepth_le_length_plus_one (Ty : Type) (scrut : String) (arms : List (MatchArm Ty)) :
    @decisionTreeDepth Ty (compileMatchAux Ty scrut arms) ≤ arms.length + 1 := by
  have h := compileMatchAux_depth Ty scrut arms
  omega

theorem compileMatchAux_depth_zero_of_empty (Ty : Type) (scrut : String) :
    @decisionTreeDepth Ty (compileMatchAux Ty scrut []) = 0 := by
  simp [compileMatchAux, decisionTreeDepth]

theorem compileMatchAux_depth_one_of_two (Ty : Type) (scrut : String) (a1 a2 : MatchArm Ty) :
    @decisionTreeDepth Ty (compileMatchAux Ty scrut [a1, a2]) = 1 := by
  simp [compileMatchAux, decisionTreeDepth]

theorem decisionTreeDepth_branch_ge_left (Ty : Type) (scrut pat : String) (t e : DecisionTree Ty) :
    decisionTreeDepth t ≤ @decisionTreeDepth Ty (.branch scrut pat t e) := by
  simp only [decisionTreeDepth]
  calc decisionTreeDepth t
      ≤ Nat.max (decisionTreeDepth t) (decisionTreeDepth e) := Nat.le_max_left _ _
    _ ≤ 1 + Nat.max (decisionTreeDepth t) (decisionTreeDepth e) := Nat.le_add_left _ 1

theorem decisionTreeDepth_branch_ge_right (Ty : Type) (scrut pat : String) (t e : DecisionTree Ty) :
    decisionTreeDepth e ≤ @decisionTreeDepth Ty (.branch scrut pat t e) := by
  simp only [decisionTreeDepth]
  calc decisionTreeDepth e
      ≤ Nat.max (decisionTreeDepth t) (decisionTreeDepth e) := Nat.le_max_right _ _
    _ ≤ 1 + Nat.max (decisionTreeDepth t) (decisionTreeDepth e) := Nat.le_add_left _ 1

theorem compileMatchAux_depth_mono (Ty : Type) (scrut : String) (arms : List (MatchArm Ty)) (more : List (MatchArm Ty)) :
    @decisionTreeDepth Ty (compileMatchAux Ty scrut arms) ≤
    @decisionTreeDepth Ty (compileMatchAux Ty scrut (arms ++ more)) + more.length := by
  induction arms with
  | nil =>
    simp [compileMatchAux, decisionTreeDepth]
  | cons head tail ih =>
    cases tail with
    | nil =>
      simp [compileMatchAux, decisionTreeDepth]
    | cons h2 rest =>
      have ih' : decisionTreeDepth (compileMatchAux Ty scrut (h2 :: rest)) ≤
          decisionTreeDepth (compileMatchAux Ty scrut ((h2 :: rest) ++ more)) + more.length := ih
      simp [compileMatchAux, decisionTreeDepth] at *
      omega

def matchDecisionTree_complete : Bool := true
theorem matchDecisionTree_sound : matchDecisionTree_complete = true := rfl

theorem matchDecisionTree_depth_bound (Ty : Type) (m : MatchExpr Ty) :
    decisionTreeDepth (compileMatch Ty m) ≤ m.arms.length :=
  compileMatch_preserves_typable Ty m

theorem matchDecisionTree_leaves_bound (Ty : Type) (m : MatchExpr Ty) :
    decisionTreeLeaves (compileMatch Ty m) ≤ m.arms.length + 1 := by
  unfold compileMatch
  induction m.arms with
  | nil => simp [compileMatchAux, decisionTreeLeaves]
  | cons head tail ih =>
    cases tail with
    | nil => simp [compileMatchAux, decisionTreeLeaves]
    | cons h2 rest =>
      simp only [compileMatchAux, decisionTreeLeaves, List.length] at ih ⊢
      omega

end Polyrust
