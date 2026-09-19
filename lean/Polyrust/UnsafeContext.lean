/-
Phase3 — Unsafe 上下文位元、I/O 效應、Pure、No-IO
對應 Rust：core/src/minirust/effects.rs, borrowck.rs
-/

namespace Polyrust
inductive RawPtrKind where
| const
| mut
deriving DecidableEq, Repr

structure RawPtrTy where
  kind : RawPtrKind
  inner : String
deriving DecidableEq, Repr

structure EffectContext where
  inUnsafe : Bool
  unsafeAllowed : Bool
  hasIO : Bool
  pure : Option Bool
  noIO : Bool
  unsafeUsages : List Nat
  ioUsages : List Nat
  rawPtrOps : List Nat
deriving Repr

def EffectContext.empty : EffectContext :=
  { inUnsafe := false, unsafeAllowed := false, hasIO := false,
    pure := none, noIO := false,
    unsafeUsages := [], ioUsages := [], rawPtrOps := [] }

def EffectContext.checkUnsafeGate (ctx : EffectContext) : Except String Unit :=
  if !ctx.unsafeUsages.isEmpty && !ctx.unsafeAllowed && !ctx.inUnsafe then
    .error "unsafe usage but not allowed"
  else .ok ()

def EffectContext.checkNoIO (ctx : EffectContext) : Except String Unit :=
  if ctx.noIO && ctx.hasIO then
    .error "I/O but no-io set"
  else .ok ()

def EffectContext.checkPure (ctx : EffectContext) : Except String Unit :=
  match ctx.pure with
  | some true =>
    if ctx.hasIO then .error "pure function has I/O"
    else .ok ()
  | _ => .ok ()

def EffectContext.checkAll (ctx : EffectContext) : Except String Unit := do
  ctx.checkUnsafeGate
  ctx.checkNoIO
  ctx.checkPure

def isIOCall (name : String) : Bool :=
  name == "print" || name == "println" || name == "read" || name == "write" || name == "open"

def parseRawPtr (s : String) : Option RawPtrTy :=
  if s.startsWith "*const " then
    some { kind := .const, inner := (s.drop 7).toString }
  else if s.startsWith "*mut " then
    some { kind := .mut, inner := (s.drop 5).toString }
  else none

def checkRawPtrDeref (_ty : RawPtrTy) (ctx : EffectContext) (nodeId : Nat) : Except String Unit :=
  if !ctx.inUnsafe && !ctx.unsafeAllowed then
    .error s!"raw ptr deref requires unsafe at {nodeId}"
  else .ok ()

theorem empty_checks_ok : EffectContext.empty.checkAll.isOk = true := by
  rfl

def testCtx : EffectContext :=
  { EffectContext.empty with unsafeUsages := [1], unsafeAllowed := false }

def testCtxCheck : Except String Unit := testCtx.checkUnsafeGate
def testEmptyCheck : Except String Unit := EffectContext.empty.checkAll
def testParseConst : Option RawPtrTy := parseRawPtr "*const i32"
def testParseMut : Option RawPtrTy := parseRawPtr "*mut u8"

end Polyrust
