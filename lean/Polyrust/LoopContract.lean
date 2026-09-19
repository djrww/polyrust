-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
Phase3 — 迴圈契約與有界展開：@fuel, @invariant, @requires, @ensures
對應 Rust：core/src/minirust/contracts.rs
-/

namespace Polyrust
structure LoopContract where
  fuel : Option Nat
  invariants : List String
  requires : List String
  ensures : List String
deriving Repr

def LoopContract.empty : LoopContract :=
  { fuel := none, invariants := [], requires := [], ensures := [] }

def LoopContract.fuelOrDefault (c : LoopContract) : Nat :=
  c.fuel.getD 3

def LoopContract.invariantPolyText (c : LoopContract) (nodeId : Nat) : List String :=
  let rec go : List String → Nat → List String → List String
  | [], _, acc => acc.reverse
  | inv :: rest, i, acc => go rest (i+1) ((s!"t{nodeId}_inv{i} -1 =0  # invariant {i}: {inv}") :: acc)
  go c.invariants 0 []

def LoopContract.fuelPolyText (c : LoopContract) (nodeId : Nat) : List String :=
  let k := c.fuelOrDefault
  (List.range k).map (fun i =>
    s!"c{nodeId}_{i+1} - c{nodeId}_{i} -1 =0  # fuel {i}->{i+1}")

def unrollWhile (cond body : String) (fuel : Nat) (_invariants : List String) : String :=
  s!"// while {cond} with fuel={fuel} body={body}"

def unrollFor (pat iter body : String) (fuel : Nat) (_invariants : List String) : String :=
  s!"// for {pat} in {iter} with fuel={fuel} body={body}"

def invariantPoly (nvars invVar : Nat) : String :=
  s!"x{invVar} -1 =0  # invariant bool (nvars={nvars})"

def fuelCounterPoly (nvars c_i c_next : Nat) : String :=
  s!"x{c_next} - x{c_i} +1 =0  # fuel counter (nvars={nvars})"

theorem fuel_default : LoopContract.empty.fuelOrDefault = 3 := by rfl

theorem fuel_some (n : Nat) :
  ({ LoopContract.empty with fuel := some n }).fuelOrDefault = n := by
  simp [LoopContract.fuelOrDefault]

def testUnrollWhile : String := unrollWhile "x < 10" "x = x + 1;" 5 ["x >=0"]
def testUnrollFor : String := unrollFor "x" "v" "sum = sum + x;" 3 []
def testFuelDefault : Nat := LoopContract.empty.fuelOrDefault
def testInvariantText : List String := ({ LoopContract.empty with fuel := some 10, invariants := ["x >=0"] }).invariantPolyText 1

end Polyrust
