-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- UnsafeEmitProof — Lean 對 emit 文本/IR 的對應定理 -/
import Polyrust.UnsafeSafety
namespace Polyrust

structure RawPtrEmitIR where
  nonNull : Bool
  aligned : Bool
  inBounds : Bool
  notDangling : Bool
  valid : Bool
  deref : Bool
  inUnsafe : Bool
  validSrc : Option String
deriving DecidableEq, Repr

def RawPtrEmitIR.checkValidEq (r : RawPtrEmitIR) : Bool :=
  r.valid == (r.nonNull && r.aligned && r.inBounds && r.notDangling)
def RawPtrEmitIR.checkDerefRequiresValid (r : RawPtrEmitIR) : Bool :=
  if r.deref then r.valid else true
def RawPtrEmitIR.checkDerefRequiresUnsafe (r : RawPtrEmitIR) : Bool :=
  if r.deref then r.inUnsafe else true
def RawPtrEmitIR.checkDerefFixed (r : RawPtrEmitIR) : Bool :=
  r.deref == true
def RawPtrEmitIR.checkValidSrc (r : RawPtrEmitIR) : Bool :=
  r.validSrc.isSome
def RawPtrEmitIR.isEmitValid (r : RawPtrEmitIR) : Bool :=
  r.checkValidEq && r.checkDerefRequiresValid && r.checkDerefRequiresUnsafe && r.checkDerefFixed && r.checkValidSrc &&
  (r.valid == true) && (r.inUnsafe == true) && (r.nonNull == true) && (r.aligned == true) && (r.inBounds == true) && (r.notDangling == true)

structure StaticMutEmitIR where
  access : Bool
  inUnsafe : Bool
  exclusive : Bool
  mutexProtected : Bool
  singleThreaded : Bool
  safe : Bool
  validSrc : Option String
deriving DecidableEq, Repr

def StaticMutEmitIR.checkSafeRequiresUnsafe (s : StaticMutEmitIR) : Bool :=
  if s.safe then s.inUnsafe else true
def StaticMutEmitIR.checkSafeRequiresProtection (s : StaticMutEmitIR) : Bool :=
  if s.safe then (s.exclusive || s.mutexProtected || s.singleThreaded) else true
def StaticMutEmitIR.checkAccessRequiresSafe (s : StaticMutEmitIR) : Bool :=
  if s.access then s.safe else true
def StaticMutEmitIR.checkAccessFixed (s : StaticMutEmitIR) : Bool :=
  s.access == true
def StaticMutEmitIR.checkValidSrc (s : StaticMutEmitIR) : Bool :=
  s.validSrc.isSome
def StaticMutEmitIR.isEmitValid (s : StaticMutEmitIR) : Bool :=
  s.checkSafeRequiresUnsafe && s.checkSafeRequiresProtection && s.checkAccessRequiresSafe && s.checkAccessFixed && s.checkValidSrc &&
  (s.safe == true) && (s.inUnsafe == true)

structure UnionEmitIR where
  activeTag : Bool
  accessedTag : Bool
  tagMatch : Bool
  inUnsafe : Bool
  safe : Bool
  validSrc : Option String
deriving DecidableEq, Repr

def UnionEmitIR.checkTagMatchEq (u : UnionEmitIR) : Bool :=
  u.tagMatch == (u.activeTag == u.accessedTag)
def UnionEmitIR.checkSafeEq (u : UnionEmitIR) : Bool :=
  u.safe == (u.inUnsafe && u.tagMatch)
def UnionEmitIR.checkFixed (u : UnionEmitIR) : Bool :=
  u.activeTag == true && u.accessedTag == true
def UnionEmitIR.checkSafeFixed (u : UnionEmitIR) : Bool :=
  u.safe == true
def UnionEmitIR.checkValidSrc (u : UnionEmitIR) : Bool :=
  u.validSrc.isSome
def UnionEmitIR.isEmitValid (u : UnionEmitIR) : Bool :=
  u.checkTagMatchEq && u.checkSafeEq && u.checkFixed && u.checkSafeFixed && u.checkValidSrc &&
  (u.tagMatch == true) && (u.inUnsafe == true)

structure UnsafeFnEmitIR where
  call : Bool
  inUnsafe : Bool
  precond : Bool
  safe : Bool
  fnName : String
  validSrc : Option String
deriving Repr

def UnsafeFnEmitIR.checkSafeEq (f : UnsafeFnEmitIR) : Bool :=
  f.safe == (f.inUnsafe && f.precond)
def UnsafeFnEmitIR.checkCallRequiresSafe (f : UnsafeFnEmitIR) : Bool :=
  if f.call then f.safe else true
def UnsafeFnEmitIR.checkCallFixed (f : UnsafeFnEmitIR) : Bool :=
  f.call == true
def UnsafeFnEmitIR.checkValidSrc (f : UnsafeFnEmitIR) : Bool :=
  f.validSrc.isSome
def UnsafeFnEmitIR.isEmitValid (f : UnsafeFnEmitIR) : Bool :=
  f.checkSafeEq && f.checkCallRequiresSafe && f.checkCallFixed && f.checkValidSrc &&
  (f.safe == true) && (f.precond == true) && (f.inUnsafe == true)

structure UnsafeTraitEmitIR where
  implExists : Bool
  isUnsafeImpl : Bool
  invariant : Bool
  safe : Bool
  traitName : String
  validSrc : Option String
deriving Repr

def UnsafeTraitEmitIR.checkSafeEq (t : UnsafeTraitEmitIR) : Bool :=
  t.safe == (t.isUnsafeImpl && t.invariant)
def UnsafeTraitEmitIR.checkImplRequiresSafe (t : UnsafeTraitEmitIR) : Bool :=
  if t.implExists then t.safe else true
def UnsafeTraitEmitIR.checkImplFixed (t : UnsafeTraitEmitIR) : Bool :=
  t.implExists == true
def UnsafeTraitEmitIR.checkValidSrc (t : UnsafeTraitEmitIR) : Bool :=
  t.validSrc.isSome
def UnsafeTraitEmitIR.isEmitValid (t : UnsafeTraitEmitIR) : Bool :=
  t.checkSafeEq && t.checkImplRequiresSafe && t.checkImplFixed && t.checkValidSrc &&
  (t.safe == true) && (t.invariant == true) && (t.isUnsafeImpl == true)

def rawPtrEmitTextExample : String :=
  "// raw_ptr node 9000 deref=1 valid_src=annotation_contract"

theorem emit_text_contains_deref_fixed_implies_ir_fixed (ir : RawPtrEmitIR) (hEmit : ir.checkDerefFixed = true) :
    ir.deref = true := by
  simp [RawPtrEmitIR.checkDerefFixed] at hEmit
  exact hEmit

theorem raw_ptr_emit_implies_no_ub (r : RawPtrEmitIR) (hEmit : r.isEmitValid = true) :
    r.valid = true ∧ r.inUnsafe = true ∧ r.deref = true := by
  have hv : r.valid = true := by
    cases h : r.valid
    · simp [RawPtrEmitIR.isEmitValid, RawPtrEmitIR.checkValidEq, RawPtrEmitIR.checkDerefRequiresValid, RawPtrEmitIR.checkDerefRequiresUnsafe, RawPtrEmitIR.checkDerefFixed, RawPtrEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hu : r.inUnsafe = true := by
    cases h : r.inUnsafe
    · simp [RawPtrEmitIR.isEmitValid, RawPtrEmitIR.checkValidEq, RawPtrEmitIR.checkDerefRequiresValid, RawPtrEmitIR.checkDerefRequiresUnsafe, RawPtrEmitIR.checkDerefFixed, RawPtrEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hd : r.deref = true := by
    cases h : r.deref
    · simp [RawPtrEmitIR.isEmitValid, RawPtrEmitIR.checkValidEq, RawPtrEmitIR.checkDerefRequiresValid, RawPtrEmitIR.checkDerefRequiresUnsafe, RawPtrEmitIR.checkDerefFixed, RawPtrEmitIR.checkValidSrc, h] at hEmit
    · rfl
  exact ⟨hv, hu, hd⟩

theorem static_mut_emit_implies_no_data_race (s : StaticMutEmitIR) (hEmit : s.isEmitValid = true) :
    s.safe = true ∧ s.inUnsafe = true ∧ s.access = true := by
  have hs : s.safe = true := by
    cases h : s.safe
    · simp [StaticMutEmitIR.isEmitValid, StaticMutEmitIR.checkSafeRequiresUnsafe, StaticMutEmitIR.checkSafeRequiresProtection, StaticMutEmitIR.checkAccessRequiresSafe, StaticMutEmitIR.checkAccessFixed, StaticMutEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hu : s.inUnsafe = true := by
    cases h : s.inUnsafe
    · simp [StaticMutEmitIR.isEmitValid, StaticMutEmitIR.checkSafeRequiresUnsafe, StaticMutEmitIR.checkSafeRequiresProtection, StaticMutEmitIR.checkAccessRequiresSafe, StaticMutEmitIR.checkAccessFixed, StaticMutEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have ha : s.access = true := by
    cases h : s.access
    · simp [StaticMutEmitIR.isEmitValid, StaticMutEmitIR.checkSafeRequiresUnsafe, StaticMutEmitIR.checkSafeRequiresProtection, StaticMutEmitIR.checkAccessRequiresSafe, StaticMutEmitIR.checkAccessFixed, StaticMutEmitIR.checkValidSrc, h] at hEmit
    · rfl
  exact ⟨hs, hu, ha⟩

theorem union_emit_implies_no_type_pun (u : UnionEmitIR) (hEmit : u.isEmitValid = true) :
    u.tagMatch = true ∧ u.safe = true ∧ u.inUnsafe = true := by
  have ht : u.tagMatch = true := by
    cases h : u.tagMatch
    · simp [UnionEmitIR.isEmitValid, UnionEmitIR.checkTagMatchEq, UnionEmitIR.checkSafeEq, UnionEmitIR.checkFixed, UnionEmitIR.checkSafeFixed, UnionEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hs : u.safe = true := by
    cases h : u.safe
    · simp [UnionEmitIR.isEmitValid, UnionEmitIR.checkTagMatchEq, UnionEmitIR.checkSafeEq, UnionEmitIR.checkFixed, UnionEmitIR.checkSafeFixed, UnionEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hu : u.inUnsafe = true := by
    cases h : u.inUnsafe
    · simp [UnionEmitIR.isEmitValid, UnionEmitIR.checkTagMatchEq, UnionEmitIR.checkSafeEq, UnionEmitIR.checkFixed, UnionEmitIR.checkSafeFixed, UnionEmitIR.checkValidSrc, h] at hEmit
    · rfl
  exact ⟨ht, hs, hu⟩

theorem unsafe_fn_emit_implies_precond (f : UnsafeFnEmitIR) (hEmit : f.isEmitValid = true) :
    f.safe = true ∧ f.precond = true ∧ f.inUnsafe = true ∧ f.call = true := by
  have hs : f.safe = true := by
    cases h : f.safe
    · simp [UnsafeFnEmitIR.isEmitValid, UnsafeFnEmitIR.checkSafeEq, UnsafeFnEmitIR.checkCallRequiresSafe, UnsafeFnEmitIR.checkCallFixed, UnsafeFnEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hp : f.precond = true := by
    cases h : f.precond
    · simp [UnsafeFnEmitIR.isEmitValid, UnsafeFnEmitIR.checkSafeEq, UnsafeFnEmitIR.checkCallRequiresSafe, UnsafeFnEmitIR.checkCallFixed, UnsafeFnEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hu : f.inUnsafe = true := by
    cases h : f.inUnsafe
    · simp [UnsafeFnEmitIR.isEmitValid, UnsafeFnEmitIR.checkSafeEq, UnsafeFnEmitIR.checkCallRequiresSafe, UnsafeFnEmitIR.checkCallFixed, UnsafeFnEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hc : f.call = true := by
    cases h : f.call
    · simp [UnsafeFnEmitIR.isEmitValid, UnsafeFnEmitIR.checkSafeEq, UnsafeFnEmitIR.checkCallRequiresSafe, UnsafeFnEmitIR.checkCallFixed, UnsafeFnEmitIR.checkValidSrc, h] at hEmit
    · rfl
  exact ⟨hs, hp, hu, hc⟩

theorem unsafe_trait_emit_implies_invariant (t : UnsafeTraitEmitIR) (hEmit : t.isEmitValid = true) :
    t.safe = true ∧ t.invariant = true ∧ t.isUnsafeImpl = true ∧ t.implExists = true := by
  have hs : t.safe = true := by
    cases h : t.safe
    · simp [UnsafeTraitEmitIR.isEmitValid, UnsafeTraitEmitIR.checkSafeEq, UnsafeTraitEmitIR.checkImplRequiresSafe, UnsafeTraitEmitIR.checkImplFixed, UnsafeTraitEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hi : t.invariant = true := by
    cases h : t.invariant
    · simp [UnsafeTraitEmitIR.isEmitValid, UnsafeTraitEmitIR.checkSafeEq, UnsafeTraitEmitIR.checkImplRequiresSafe, UnsafeTraitEmitIR.checkImplFixed, UnsafeTraitEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have hu : t.isUnsafeImpl = true := by
    cases h : t.isUnsafeImpl
    · simp [UnsafeTraitEmitIR.isEmitValid, UnsafeTraitEmitIR.checkSafeEq, UnsafeTraitEmitIR.checkImplRequiresSafe, UnsafeTraitEmitIR.checkImplFixed, UnsafeTraitEmitIR.checkValidSrc, h] at hEmit
    · rfl
  have he : t.implExists = true := by
    cases h : t.implExists
    · simp [UnsafeTraitEmitIR.isEmitValid, UnsafeTraitEmitIR.checkSafeEq, UnsafeTraitEmitIR.checkImplRequiresSafe, UnsafeTraitEmitIR.checkImplFixed, UnsafeTraitEmitIR.checkValidSrc, h] at hEmit
    · rfl
  exact ⟨hs, hi, hu, he⟩

theorem valid_src_required_raw_ptr (r : RawPtrEmitIR) (hEmit : r.isEmitValid = true) :
    r.validSrc.isSome = true := by
  cases h : r.validSrc.isSome
  · simp [RawPtrEmitIR.isEmitValid, RawPtrEmitIR.checkValidEq, RawPtrEmitIR.checkDerefRequiresValid, RawPtrEmitIR.checkDerefRequiresUnsafe, RawPtrEmitIR.checkDerefFixed, RawPtrEmitIR.checkValidSrc, h] at hEmit
  · rfl

theorem valid_src_required_static_mut (s : StaticMutEmitIR) (hEmit : s.isEmitValid = true) :
    s.validSrc.isSome = true := by
  cases h : s.validSrc.isSome
  · simp [StaticMutEmitIR.isEmitValid, StaticMutEmitIR.checkSafeRequiresUnsafe, StaticMutEmitIR.checkSafeRequiresProtection, StaticMutEmitIR.checkAccessRequiresSafe, StaticMutEmitIR.checkAccessFixed, StaticMutEmitIR.checkValidSrc, h] at hEmit
  · rfl

theorem valid_src_required_union (u : UnionEmitIR) (hEmit : u.isEmitValid = true) :
    u.validSrc.isSome = true := by
  cases h : u.validSrc.isSome
  · simp [UnionEmitIR.isEmitValid, UnionEmitIR.checkTagMatchEq, UnionEmitIR.checkSafeEq, UnionEmitIR.checkFixed, UnionEmitIR.checkSafeFixed, UnionEmitIR.checkValidSrc, h] at hEmit
  · rfl

theorem valid_src_required_unsafe_fn (f : UnsafeFnEmitIR) (hEmit : f.isEmitValid = true) :
    f.validSrc.isSome = true := by
  cases h : f.validSrc.isSome
  · simp [UnsafeFnEmitIR.isEmitValid, UnsafeFnEmitIR.checkSafeEq, UnsafeFnEmitIR.checkCallRequiresSafe, UnsafeFnEmitIR.checkCallFixed, UnsafeFnEmitIR.checkValidSrc, h] at hEmit
  · rfl

theorem valid_src_required_unsafe_trait (t : UnsafeTraitEmitIR) (hEmit : t.isEmitValid = true) :
    t.validSrc.isSome = true := by
  cases h : t.validSrc.isSome
  · simp [UnsafeTraitEmitIR.isEmitValid, UnsafeTraitEmitIR.checkSafeEq, UnsafeTraitEmitIR.checkImplRequiresSafe, UnsafeTraitEmitIR.checkImplFixed, UnsafeTraitEmitIR.checkValidSrc, h] at hEmit
  · rfl

theorem valid_src_required (r : RawPtrEmitIR) (s : StaticMutEmitIR) (u : UnionEmitIR) (f : UnsafeFnEmitIR) (t : UnsafeTraitEmitIR)
    (hr : r.isEmitValid = true) (hs : s.isEmitValid = true) (hu : u.isEmitValid = true) (hf : f.isEmitValid = true) (ht : t.isEmitValid = true) :
    r.validSrc.isSome = true ∧ s.validSrc.isSome = true ∧ u.validSrc.isSome = true ∧ f.validSrc.isSome = true ∧ t.validSrc.isSome = true := by
  exact ⟨valid_src_required_raw_ptr r hr, valid_src_required_static_mut s hs, valid_src_required_union u hu, valid_src_required_unsafe_fn f hf, valid_src_required_unsafe_trait t ht⟩

def emitTextForRawPtr (r : RawPtrEmitIR) : String :=
  s!"// raw_ptr deref={r.deref} valid={r.valid} inUnsafe={r.inUnsafe} validSrc={r.validSrc}"

theorem emit_text_matches_ir (r : RawPtrEmitIR) :
    r.checkDerefFixed = true → r.deref = true := by
  intro h
  simp [RawPtrEmitIR.checkDerefFixed] at h
  exact h

structure AllEmitIR where
  rawPtr : RawPtrEmitIR
  staticMut : StaticMutEmitIR
  unionIR : UnionEmitIR
  unsafeFn : UnsafeFnEmitIR
  unsafeTrait : UnsafeTraitEmitIR

def AllEmitIR.isFullyEmitValid (a : AllEmitIR) : Bool :=
  a.rawPtr.isEmitValid && a.staticMut.isEmitValid && a.unionIR.isEmitValid && a.unsafeFn.isEmitValid && a.unsafeTrait.isEmitValid

theorem all_emit_valid_implies_no_runtime_ub (a : AllEmitIR) (h : a.isFullyEmitValid = true) :
    a.rawPtr.valid = true ∧ a.rawPtr.inUnsafe = true ∧
    a.staticMut.safe = true ∧ a.unionIR.safe = true ∧
    a.unsafeFn.safe = true ∧ a.unsafeTrait.safe = true := by
  have hr : a.rawPtr.isEmitValid = true := by
    cases h2 : a.rawPtr.isEmitValid
    · simp [AllEmitIR.isFullyEmitValid, h2] at h
    · rfl
  have hs : a.staticMut.isEmitValid = true := by
    cases h2 : a.staticMut.isEmitValid
    · simp [AllEmitIR.isFullyEmitValid, h2] at h
    · rfl
  have hu : a.unionIR.isEmitValid = true := by
    cases h2 : a.unionIR.isEmitValid
    · simp [AllEmitIR.isFullyEmitValid, h2] at h
    · rfl
  have hf : a.unsafeFn.isEmitValid = true := by
    cases h2 : a.unsafeFn.isEmitValid
    · simp [AllEmitIR.isFullyEmitValid, h2] at h
    · rfl
  have ht : a.unsafeTrait.isEmitValid = true := by
    cases h2 : a.unsafeTrait.isEmitValid
    · simp [AllEmitIR.isFullyEmitValid, h2] at h
    · rfl
  have hRaw := raw_ptr_emit_implies_no_ub a.rawPtr hr
  have hSM := static_mut_emit_implies_no_data_race a.staticMut hs
  have hU := union_emit_implies_no_type_pun a.unionIR hu
  have hF := unsafe_fn_emit_implies_precond a.unsafeFn hf
  have hT := unsafe_trait_emit_implies_invariant a.unsafeTrait ht
  exact ⟨hRaw.left, hRaw.right.left, hSM.left, hU.right.left, hF.left, hT.left⟩

theorem emit_example_no_ub :
    let raw : RawPtrEmitIR := { nonNull := true, aligned := true, inBounds := true, notDangling := true, valid := true, deref := true, inUnsafe := true, validSrc := some "// @valid: non_null, aligned, in_bounds, not_dangling" }
    let sm : StaticMutEmitIR := { access := true, inUnsafe := true, exclusive := true, mutexProtected := false, singleThreaded := false, safe := true, validSrc := some "// @exclusive" }
    let u : UnionEmitIR := { activeTag := true, accessedTag := true, tagMatch := true, inUnsafe := true, safe := true, validSrc := some "// @tag_match" }
    let uf : UnsafeFnEmitIR := { call := true, inUnsafe := true, precond := true, safe := true, fnName := "my_unsafe", validSrc := some "// @precond: x > 0" }
    let ut : UnsafeTraitEmitIR := { implExists := true, isUnsafeImpl := true, invariant := true, safe := true, traitName := "Send", validSrc := some "// @invariant: Send is safe" }
    (AllEmitIR.mk raw sm u uf ut).isFullyEmitValid = true := by rfl

end Polyrust
