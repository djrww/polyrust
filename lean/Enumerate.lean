-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
列舉腳本（臨時工具，不入 lake 目標）：
輸出 `Polyrust.*` 命名空間下全部宣告的 TSV：
    name \t kind \t axioms \t type
判據與 AuditAll.lean 同源（Lean.collectAxioms）。模組歸屬由源碼解析補上。
執行：cd lean && lake env lean Enumerate.lean > /tmp/enumerate.tsv
-/

import Lean
import Polyrust

open Lean Meta

def enumAll : MetaM Unit := do
  let env ← getEnv
  let mut n := 0
  for (nm, ci) in env.constants.toList do
    unless (`Polyrust).isPrefixOf nm do continue
    if (nm.toString.splitOn "_hyg").length > 1 then continue
    let opt : Option (String × Expr) := match ci with
      | .axiomInfo v   => some ("axiom", v.type)
      | .opaqueInfo v  => some ("opaque", v.type)
      | .thmInfo v     => some ("theorem", v.type)
      | .defnInfo v    => some ("def", v.type)
      | _              => none
    let some (kind, type) := opt | continue
    let ax ← Lean.collectAxioms nm
    let axStr := ";".intercalate (ax.toList.map (·.toString))
      let typeStr := (toString (← ppExpr type)).replace "\n" " " |>.replace "\t" " "
    IO.println s!"{nm.toString}\t{kind}\t{axStr}\t{typeStr.replace "\t" " "}"
    n := n + 1
  IO.println s!"ENUMERATED={n}"

#eval! enumAll
