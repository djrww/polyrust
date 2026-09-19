-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
Phase3 — trait/impl 方法表與存在量化
對應 Rust：core/src/minirust/trait_impl.rs
-/

namespace Polyrust
structure TraitMethod where
  name : String
  retTy : String
  hasDefault : Bool
deriving Repr, DecidableEq

structure TraitDef where
  name : String
  typeParams : List String
  lifetimeParams : List String
  methods : List TraitMethod
  supertraits : List String
deriving Repr

structure ImplMethod where
  name : String
  retTy : String
  body : String
deriving Repr, DecidableEq

structure ImplDef where
  traitName : Option String
  forTy : String
  typeParams : List String
  lifetimeParams : List String
  methods : List ImplMethod
deriving Repr

structure MethodTable where
  traits : List TraitDef
  impls : List ImplDef
deriving Repr

def MethodTable.empty : MethodTable := { traits := [], impls := [] }

def MethodTable.addTrait (mt : MethodTable) (tr : TraitDef) : MethodTable :=
  { mt with traits := tr :: mt.traits }

def MethodTable.addImpl (mt : MethodTable) (imp : ImplDef) : MethodTable :=
  { mt with impls := imp :: mt.impls }

def MethodTable.resolveMethod (mt : MethodTable) (recvTy method : String) : Option ImplMethod :=
  mt.impls.find? (fun imp => imp.forTy == recvTy) |>.bind (fun imp =>
    imp.methods.find? (fun m => m.name == method))

def MethodTable.resolveTraitMethod (mt : MethodTable) (traitName method : String) : Option TraitMethod :=
  mt.traits.find? (fun tr => tr.name == traitName) |>.bind (fun tr =>
    tr.methods.find? (fun m => m.name == method))

def MethodTable.existentialPoly (_mt : MethodTable) (varName boundTrait body : String) : String :=
  s!"exists {varName}: {boundTrait} body={body}"

def MethodTable.typesImplementing (mt : MethodTable) (traitName : String) : List String :=
  mt.impls.filterMap (fun imp =>
    match imp.traitName with
    | some t => if t == traitName then some imp.forTy else none
    | none => none)

def traitMethodBar : TraitMethod := { name := "bar", retTy := "i32", hasDefault := false }
def implMethodBar : ImplMethod := { name := "bar", retTy := "i32", body := "42" }

def testTrait : TraitDef :=
  { name := "Foo", typeParams := [], lifetimeParams := [], methods := [traitMethodBar], supertraits := [] }

def testImpl : ImplDef :=
  { traitName := some "Foo", forTy := "MyType", typeParams := [], lifetimeParams := [], methods := [implMethodBar] }

def testResolve : Option ImplMethod := (MethodTable.empty.addTrait testTrait |>.addImpl testImpl).resolveMethod "MyType" "bar"
def testTypesImpl : List String := (MethodTable.empty.addTrait testTrait |>.addImpl testImpl).typesImplementing "Foo"

end Polyrust
