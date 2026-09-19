-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
namespace Polyrust

structure RawPtrSafety where
  ptrNonNull : Bool
  ptrAligned : Bool
  ptrInBounds : Bool
  ptrNotDangling : Bool
  ptrValid : Bool
  ptrDeref : Bool
  inUnsafe : Bool
deriving DecidableEq, Repr

def RawPtrSafety.checkValid (s : RawPtrSafety) : Bool :=
  s.ptrValid == (s.ptrNonNull && s.ptrAligned && s.ptrInBounds && s.ptrNotDangling)

def RawPtrSafety.checkDerefSafe (s : RawPtrSafety) : Bool :=
  if s.ptrDeref then s.ptrValid && s.inUnsafe else true

def RawPtrSafety.isSafe (s : RawPtrSafety) : Bool :=
  s.checkValid && s.checkDerefSafe

theorem raw_ptr_valid_ok : (RawPtrSafety.mk true true true true true false false).checkValid = true := by rfl
theorem raw_ptr_isSafe_ok : (RawPtrSafety.mk true true true true true true true).isSafe = true := by rfl

theorem raw_ptr_deref_requires_valid_and_unsafe (s : RawPtrSafety) (hDeref : s.ptrDeref) (hSafe : s.checkDerefSafe) : s.ptrValid = true ∧ s.inUnsafe = true := by
  simp [RawPtrSafety.checkDerefSafe, hDeref] at hSafe
  exact hSafe

structure StaticMutSafety where
  access : Bool
  inUnsafe : Bool
  exclusive : Bool
  mutexProtected : Bool
  singleThreaded : Bool
  safe : Bool
deriving DecidableEq, Repr

def StaticMutSafety.checkSafe (s : StaticMutSafety) : Bool :=
  s.safe == (s.inUnsafe && (s.exclusive || s.mutexProtected || s.singleThreaded))

def StaticMutSafety.checkAccess (s : StaticMutSafety) : Bool :=
  if s.access then s.safe else true

def StaticMutSafety.isSafe (s : StaticMutSafety) : Bool :=
  s.checkSafe && s.checkAccess

theorem static_mut_isSafe_ok : (StaticMutSafety.mk true true true false false true).isSafe = true := by rfl

theorem static_mut_access_requires_safe (s : StaticMutSafety) (hAcc : s.access) (hCheck : s.checkAccess) : s.safe := by
  simp [StaticMutSafety.checkAccess, hAcc] at hCheck
  exact hCheck

theorem static_mut_safe_protection (s : StaticMutSafety) (hSafe : s.safe) (hCheck : s.checkSafe) : s.inUnsafe = true := by
  unfold StaticMutSafety.checkSafe at hCheck
  simp at hCheck
  rw [hCheck] at hSafe
  simp at hSafe
  exact hSafe.left

structure UnionSafety where
  activeTag : Bool
  accessedTag : Bool
  tagMatch : Bool
  inUnsafe : Bool
  safe : Bool
deriving DecidableEq, Repr

def UnionSafety.checkTagMatch (s : UnionSafety) : Bool :=
  if s.tagMatch then s.activeTag == s.accessedTag else true

def UnionSafety.checkSafe (s : UnionSafety) : Bool :=
  s.safe == (s.inUnsafe && s.tagMatch)

def UnionSafety.isSafe (s : UnionSafety) : Bool :=
  s.checkTagMatch && s.checkSafe

theorem union_isSafe_ok : (UnionSafety.mk true true true true true).isSafe = true := by rfl

theorem union_safe_requires_tag_match (s : UnionSafety) (hSafe : s.safe) (hCheck : s.checkSafe) : s.tagMatch = true := by
  unfold UnionSafety.checkSafe at hCheck
  simp at hCheck
  rw [hCheck] at hSafe
  simp at hSafe
  exact hSafe.right

structure UnsafeFnSafety where
  fnName : String
  call : Bool
  inUnsafe : Bool
  precond : Bool
  safe : Bool
deriving DecidableEq, Repr

def UnsafeFnSafety.checkSafe (s : UnsafeFnSafety) : Bool :=
  s.safe == (s.inUnsafe && s.precond)

def UnsafeFnSafety.checkCall (s : UnsafeFnSafety) : Bool :=
  if s.call then s.safe else true

def UnsafeFnSafety.isSafe (s : UnsafeFnSafety) : Bool :=
  s.checkSafe && s.checkCall

theorem unsafe_fn_isSafe_ok : (UnsafeFnSafety.mk "foo" true true true true).isSafe = true := by rfl

theorem unsafe_fn_call_requires_safe (s : UnsafeFnSafety) (hCall : s.call) (hCheck : s.checkCall) : s.safe := by
  simp [UnsafeFnSafety.checkCall, hCall] at hCheck
  exact hCheck

theorem unsafe_fn_safe_requires_precond (s : UnsafeFnSafety) (hSafe : s.safe) (hCheck : s.checkSafe) : s.inUnsafe = true ∧ s.precond = true := by
  unfold UnsafeFnSafety.checkSafe at hCheck
  simp at hCheck
  rw [hCheck] at hSafe
  simp at hSafe
  exact hSafe

structure UnsafeTraitSafety where
  traitName : String
  implExists : Bool
  isUnsafeImpl : Bool
  invariant : Bool
  safe : Bool
deriving DecidableEq, Repr

def UnsafeTraitSafety.checkSafe (s : UnsafeTraitSafety) : Bool :=
  s.safe == (s.isUnsafeImpl && s.invariant)

def UnsafeTraitSafety.checkImpl (s : UnsafeTraitSafety) : Bool :=
  if s.implExists then s.safe else true

def UnsafeTraitSafety.isSafe (s : UnsafeTraitSafety) : Bool :=
  s.checkSafe && s.checkImpl

theorem unsafe_trait_isSafe_ok : (UnsafeTraitSafety.mk "Send" true true true true).isSafe = true := by rfl

theorem unsafe_trait_impl_requires_safe (s : UnsafeTraitSafety) (hImpl : s.implExists) (hCheck : s.checkImpl) : s.safe := by
  simp [UnsafeTraitSafety.checkImpl, hImpl] at hCheck
  exact hCheck

theorem unsafe_trait_safe_requires_invariant (s : UnsafeTraitSafety) (hSafe : s.safe) (hCheck : s.checkSafe) : s.isUnsafeImpl = true ∧ s.invariant = true := by
  unfold UnsafeTraitSafety.checkSafe at hCheck
  simp at hCheck
  rw [hCheck] at hSafe
  simp at hSafe
  exact hSafe

structure AllUnsafeSafe where
  rawPtr : RawPtrSafety
  staticMut : StaticMutSafety
  unionSafe : UnionSafety
  unsafeFn : UnsafeFnSafety
  unsafeTrait : UnsafeTraitSafety
deriving Repr

def AllUnsafeSafe.isFullySafe (a : AllUnsafeSafe) : Bool :=
  a.rawPtr.isSafe && a.staticMut.isSafe && a.unionSafe.isSafe && a.unsafeFn.isSafe && a.unsafeTrait.isSafe

theorem all_unsafe_safe_example :
  let raw := RawPtrSafety.mk true true true true true true true
  let sm := StaticMutSafety.mk true true true false false true
  let us := UnionSafety.mk true true true true true
  let uf := UnsafeFnSafety.mk "my_unsafe" true true true true
  let ut := UnsafeTraitSafety.mk "Send" true true true true
  (AllUnsafeSafe.mk raw sm us uf ut).isFullySafe = true := by rfl

def RawPtrSafety.fromEffect (inUnsafe : Bool) (valid : Bool) (deref : Bool) : RawPtrSafety :=
  { ptrNonNull := valid, ptrAligned := valid, ptrInBounds := valid, ptrNotDangling := valid,
    ptrValid := valid, ptrDeref := deref, inUnsafe := inUnsafe }

theorem fromEffect_safe_when_valid_and_unsafe :
  (RawPtrSafety.fromEffect true true true).isSafe = true := by rfl

end Polyrust
