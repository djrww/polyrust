-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
# 全環境公理審計（環境掃描版）

`Audit.lean` 逐條 `#print axioms` 檢查的是**手列清單**（84 條）——若日後新增定理
而忘了加進清單，缺口不會被發現。本檔案直接掃描環境中 `Polyrust` 命名空間下的
**每一條**宣告，因此對「新增定理」也自動有效。

底層用 Lean 內建的 `Lean.collectAxioms`（即 `#print axioms` 命令背後使用的同一
個函式），因此判據與逐條審計同源、無自寫遍歷的偏差。

判據：
* 依賴 `sorryAx`                 → 證明有洞，**不允許**
* 依賴非標準公理                  → 只允許 `propext` / `Classical.choice` / `Quot.sound`
* 宣告本身是 `axiom` 或 `opaque`  → **不允許**（本倉庫所有結論都應是定理/定義）

執行：`cd lean && lake env lean AuditAll.lean`
輸出末行必為 `AUDIT_RESULT=CLEAN` 或 `AUDIT_RESULT=DIRTY`，CI 以此判定。
-/

import Lean
import Polyrust

open Lean

/-- 白名單：Lean 標準三公理（與 Mathlib 一致）。 -/
def stdAxiomList : List Name := [`propext, `Classical.choice, `Quot.sound]

/-- 掃描環境中 `Polyrust.*` 的每一條宣告，回傳
    (受檢數, 純構造性數, 有洞清單, 非標準公理/opaque 清單)。 -/
def auditAll : CoreM (Nat × Nat × Array Name × Array Name) := do
  let env ← getEnv
  let mut checked : Nat := 0
  let mut constructive : Nat := 0
  let mut sorries : Array Name := #[]
  let mut extra : Array Name := #[]
  for (n, ci) in env.constants.toList do
    -- 只看 Polyrust 命名空間，並跳過巨集生成的 `_hyg` 內部宣告
    if !(`Polyrust).isPrefixOf n then continue
    if (n.toString.splitOn "_hyg").length > 1 then continue
    checked := checked + 1
    match ci with
    | .axiomInfo _  => extra := extra.push n
    | .opaqueInfo _ => extra := extra.push n
    | .thmInfo _ | .defnInfo _ =>
      let ax ← Lean.collectAxioms n
      if ax.contains `sorryAx then
        sorries := sorries.push n
      else
        let bad := ax.filter fun a => !(stdAxiomList.contains a)
        if bad.isEmpty then
          if ax.isEmpty then constructive := constructive + 1
        else
          extra := extra.push n
    | _ => continue
  return (checked, constructive, sorries, extra)

#eval show CoreM Unit from do
  let (checked, constructive, sorries, extra) ← auditAll
  IO.println "== 全環境公理審計（Polyrust.* 命名空間，Lean.collectAxioms） =="
  IO.println s!"受檢宣告數                ：{checked}"
  IO.println s!"純構造性（零公理依賴）    ：{constructive}"
  IO.println s!"含 sorryAx（必須為 0）    ：{sorries.size}"
  IO.println s!"非標準公理/opaque（須為 0）：{extra.size}"
  unless sorries.isEmpty do
    IO.println "❌ 以下宣告的證明有洞（sorryAx）："
    for n in sorries do IO.println s!"   - {n}"
  unless extra.isEmpty do
    IO.println "❌ 以下宣告用了非標準公理，或本身是 axiom/opaque："
    for n in extra do IO.println s!"   - {n}"
  if sorries.isEmpty && extra.isEmpty then
    IO.println "✅ 乾淨：零 sorry、零自訂公理，僅使用 Lean 標準三公理"
    IO.println "AUDIT_RESULT=CLEAN"
  else
    IO.println "AUDIT_RESULT=DIRTY"
