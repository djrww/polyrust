-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
Phase3 — Async QAP：Future 輪詢約束、狀態機到 R1CS
對應 Rust：core/src/minirust/async_qap.rs
-/

namespace Polyrust
inductive FutureState where
| pending
| ready
| polling : Nat → FutureState
deriving DecidableEq, Repr

structure AsyncStateMachine where
  fnName : String
  numAwaitPoints : Nat
  states : List FutureState
  transitions : List (Nat × Nat × String)
  awaitTys : List String
deriving Repr

def mkStatesAux : Nat → List FutureState → List FutureState
  | 0, acc => acc
  | k+1, acc => mkStatesAux k (acc ++ [FutureState.polling k])

def mkStates (n : Nat) : List FutureState :=
  (mkStatesAux n [FutureState.pending]) ++ [FutureState.ready]

theorem mkStatesAux_length (k : Nat) (acc : List FutureState) :
    (mkStatesAux k acc).length = acc.length + k := by
  induction k generalizing acc with
  | zero => simp [mkStatesAux]
  | succ k ih =>
    simp only [mkStatesAux, List.length_append]
    calc (mkStatesAux k (acc ++ [FutureState.polling k])).length
        = (acc ++ [FutureState.polling k]).length + k := ih _
      _ = acc.length + 1 + k := by simp [List.length_append]
      _ = acc.length + (k + 1) := by omega

def AsyncStateMachine.new (fnName : String) (numAwait : Nat) : AsyncStateMachine :=
  { fnName := fnName, numAwaitPoints := numAwait, states := mkStates numAwait,
    transitions := [], awaitTys := List.replicate numAwait "" }

def AsyncStateMachine.addTransition (sm : AsyncStateMachine) (src dst : Nat) (c : String) : AsyncStateMachine :=
  { sm with transitions := (src, dst, c) :: sm.transitions }

def AsyncStateMachine.polyText (sm : AsyncStateMachine) : List String :=
  let header := [
    s!"// Async state machine for {sm.fnName}",
    s!"// {sm.numAwaitPoints} await points, {sm.states.length} states",
    "// one-hot: sum s_i =1"
  ]
  let trans := sm.transitions.map (fun t => match t with | (s, d, c) => s!"s{s} * poll_{s} - s{d} =0  # {c}")
  let bools := List.range sm.states.length |>.map (fun i => s!"s{i} * (s{i} -1) =0")
  header ++ trans ++ ["// boolean"] ++ bools

def AsyncStateMachine.pollingConstraints (sm : AsyncStateMachine) : List String :=
  let rec go : Nat → List String → List String
  | 0, acc => acc
  | n+1, acc =>
    let i := n
    go n (acc ++ [s!"poll_{i} * (poll_{i} -1) =0  # boolean", s!"poll_{i} * ready_{i} - ready_{i} =0"])
  [s!"// Polling for {sm.fnName}"] ++ go sm.numAwaitPoints []

def lowerAsyncFn (fnName : String) (_body : String) : AsyncStateMachine :=
  let numAwait := 1
  let sm0 := AsyncStateMachine.new fnName numAwait
  let rec addTrans : Nat → AsyncStateMachine → AsyncStateMachine
  | 0, sm => sm
  | n+1, sm => addTrans n (sm.addTransition n (n+1) s!"poll_{n}")
  addTrans (sm0.states.length - 1) sm0

theorem state_number (fnName : String) (n : Nat) :
  (AsyncStateMachine.new fnName n).states.length = n + 2 := by
  simp only [AsyncStateMachine.new, mkStates, List.length_append]
  rw [mkStatesAux_length]
  simp
  omega

def testSM : AsyncStateMachine := AsyncStateMachine.new "my_async" 2
def testSMStatesLen : Nat := testSM.states.length
def testPolyText : List String := (AsyncStateMachine.new "f" 1).polyText

end Polyrust
