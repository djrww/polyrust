/-
Phase3 — Vec/String/HashMap 內建庫編碼
對應 Rust：core/src/minirust/stdlib.rs
-/

namespace Polyrust
structure VecEncoding where
  ptrVar : Nat
  lenVar : Nat
  capVar : Nat
  elemTy : String
deriving Repr

def VecEncoding.lenLeCapPoly (enc : VecEncoding) : String :=
  s!"cap{enc.capVar} - len{enc.lenVar} - slack =0"

def VecEncoding.pushPoly (enc : VecEncoding) (lenNext : Nat) : String :=
  s!"len{lenNext} - len{enc.lenVar} -1 =0"

structure StringEncoding where
  vec : VecEncoding
deriving Repr

def StringEncoding.utf8Poly (_enc : StringEncoding) : List String :=
  ["// utf8 check"]

structure HashMapEncoding where
  ptrVar : Nat
  lenVar : Nat
  capVar : Nat
  keyTy : String
  valTy : String
deriving Repr

def HashMapEncoding.uniqueKeysPoly (enc : HashMapEncoding) (k_i k_j inv : Nat) : String :=
  s!"(k{k_i} - k{k_j}) * inv{inv} -1 =0  # {enc.keyTy}"

structure StdlibRegistry where
  vecEncodings : List (String × VecEncoding)
  stringEncodings : List (String × StringEncoding)
  hashmapEncodings : List (String × HashMapEncoding)
deriving Repr

def StdlibRegistry.empty : StdlibRegistry :=
  { vecEncodings := [], stringEncodings := [], hashmapEncodings := [] }

def StdlibRegistry.registerVec (reg : StdlibRegistry) (name : String) (enc : VecEncoding) : StdlibRegistry :=
  { reg with vecEncodings := (name, enc) :: reg.vecEncodings }

def StdlibRegistry.registerString (reg : StdlibRegistry) (name : String) (enc : StringEncoding) : StdlibRegistry :=
  { reg with stringEncodings := (name, enc) :: reg.stringEncodings }

def StdlibRegistry.registerHashMap (reg : StdlibRegistry) (name : String) (enc : HashMapEncoding) : StdlibRegistry :=
  { reg with hashmapEncodings := (name, enc) :: reg.hashmapEncodings }

def StdlibRegistry.fromTypeUniverse (_typeUni : String) : StdlibRegistry :=
  StdlibRegistry.empty
    |>.registerVec "Vec<i32>" { ptrVar := 0, lenVar := 1, capVar := 2, elemTy := "i32" }
    |>.registerString "String" { vec := { ptrVar := 0, lenVar := 1, capVar := 2, elemTy := "u8" } }
    |>.registerHashMap "HashMap<String,i32>" { ptrVar := 0, lenVar := 1, capVar := 2, keyTy := "String", valTy := "i32" }

def r1csForStdlib (_typeUni : String) : List String :=
  ["// Vec len <= cap", "cap - len - slack =0", "// HashMap key unique", "(k_i - k_j)*inv -1=0"]

def testRegVecLen : Nat := (StdlibRegistry.fromTypeUniverse "Vec<i32>").vecEncodings.length
def testR1cs : List String := r1csForStdlib "Vec<i32>"

end Polyrust
