-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 編碼可靠性/完備性與判定的微型實例（T1/T2/T6/T7 的全枚舉見證）

對應 docs/THEOREMS.md §3（T1）、§4（T2）、§8（T6）、§9（T7）：
這裡把「型別規則 → 多項式方程」的編碼在三個微型系統上**完全機械化**——
型別位元直接取 Bool，規則方程化為 Int 係數多項式，全部 2^k 個 0/1
賦值逐一枚舉驗證（Lean 的 cases + simp 即決定性窮舉）：

1. **加法規則**（T1+T2）：e₁ + e₂ : i32 ⟺ t_x ∧ t_y ∧ t_r 全真。
   編碼 `addPoly1 = t_x + t_y − t_r − 1`、`addPoly2 = t_x − t_y`。
   `add_sat_iff`：根 ⟺ 良構（可靠性 σ_D 是根 + 完備性根解碼推導）。
2. **上下文矛盾**（T6 UNSAT 側）：結果被上下文強制為 bool
   （子句 1 − t_r = 0）與加法規則聯立 ⇒ 無 0/1 解 ⇒ 不可定型。
3. **宏選臂**（T7 exists-arm）：m!(x) 兩臂，臂位元 a 閘控
   （a·(模板約束)）。∀x 總存在可行臂；強制錯臂 ⇒ 無解。 -/

import Polyrust.ClauseDuality

namespace Polyrust

/-! ## 系統一：加法規則 e₁ + e₂ : i32

規則方程（位元取 {0,1}，one-hot 由 Bool 內建，域多項式自動滿足）：
- t_x + t_y − t_r − 1 = 0（兩操作數皆 i32 ⇒ 結果 i32）
- t_x − t_y = 0（操作數類型一致） -/

/-- 規則方程 1：t_x + t_y − t_r − 1。 -/
def addPoly1 (tx ty tr : Bool) : Int := bit tx + bit ty - bit tr - 1

/-- 規則方程 2：t_x − t_y。 -/
def addPoly2 (tx ty _tr : Bool) : Int := bit tx - bit ty

/-- 良構性：推導存在（兩操作數 i32、結果 i32）。 -/
def wellTypedAdd (tx ty tr : Bool) : Prop := tx = true ∧ ty = true ∧ tr = true

/-- **T1 微實例（可靠性）**：推導 D 的位元賦值 σ_D 是方程組的根。 -/
theorem T1_micro :
    addPoly1 true true true = 0 ∧ addPoly2 true true true = 0 :=
  ⟨rfl, rfl⟩

/-- **T1+T2 微實例（根 ⟺ 良構）**：
方程組的 0/1 栤恰是良構賦值——可靠性（⇐ 方向）與完備性（⇒ 方向：
任意根都解碼出推導）同時成立。 -/
theorem add_sat_iff (tx ty tr : Bool) :
    (addPoly1 tx ty tr = 0 ∧ addPoly2 tx ty tr = 0) ↔ wellTypedAdd tx ty tr := by
  cases tx <;> cases ty <;> cases tr <;>
    simp [addPoly1, addPoly2, bit, wellTypedAdd]

/-- **T6 微實例（UNSAT 側）**：上下文強制結果為 bool
（型別環境的子句 t_r = 0）與加法規則（要求 t_r = 1）矛盾——
聯立系統在 {0,1}³ 上無解 ⇒（由 T2 逆否）無推導 ⇒ 不可定型，
對應 Gröbner 側 1 ∈ G。 -/
theorem T6_micro_unsat :
    ∀ tx ty tr : Bool,
      ¬ (addPoly1 tx ty tr = 0 ∧ addPoly2 tx ty tr = 0 ∧ bit tr = 0) := by
  intro tx ty tr
  cases tx <;> cases ty <;> cases tr <;>
    simp [addPoly1, addPoly2, bit]

/-- **T6 微實例（SAT 側）**：無上下文衝突時系統可解（σ = 全真）。 -/
theorem T6_micro_sat :
    ∃ tx ty tr : Bool, addPoly1 tx ty tr = 0 ∧ addPoly2 tx ty tr = 0 :=
  ⟨true, true, true, rfl, rfl⟩

/-! ## 系統二：宏選臂 m!(x)（exists-arm 語義）

兩臂：臂 1 模板 = x（輸出類型隨 x）；臂 2 模板 = !x（要求 x : bool，
輸出 bool）。臂位元 a（true = 臂 1）閘控全部模板約束；上下文強制
結果為 i32（t_r = true）。臂 k 的約束乘以臂位元因子：
- 臂 1：a·(t_x − t_r) = 0（輸出類型 = x 的類型）
- 臂 2：(1−a)·t_x = 0（x : bool）∧ (1−a)·(1−t_r) = 0（輸出 bool） -/

/-- 臂 1 約束：a·(t_x − t_r)。 -/
def armPoly (a tx tr : Bool) : Int := bit a * (bit tx - bit tr)

/-- 臂 2 約束 A：(1−a)·t_x。 -/
def arm2PolyA (a tx _tr : Bool) : Int := (1 - bit a) * bit tx

/-- 臂 2 約束 B：(1−a)·(1−t_r)。 -/
def arm2PolyB (a _tx tr : Bool) : Int := (1 - bit a) * (1 - bit tr)

/-- **T7 微實例（exists-arm 可解）**：對任意 x 的類型，總存在臂選擇
使系統可解（宏總能定型）——臂選擇的存在量化語義。 -/
theorem T7_arm_sat :
    ∀ tx : Bool, ∃ a : Bool,
      armPoly a tx true = 0 ∧ arm2PolyA a tx true = 0 ∧ arm2PolyB a tx true = 0 := by
  intro tx
  cases tx
  · -- x : bool ⇒ 選臂 2（a = false）
    exact ⟨false, rfl, rfl, rfl⟩
  · -- x : i32 ⇒ 選臂 1（a = true）
    exact ⟨true, rfl, rfl, rfl⟩

/-- **T7 微實例（錯臂矛盾）**：強制臂 1（a = true）且 x : bool
（t_x = false）且結果 i32（t_r = true）⇒ 臂 1 的類約方程
a·(t_x − t_r) = −1 ≠ 0 ⇒ 無解——「錯臂 1 ∈ G」的微型化。 -/
theorem T7_wrong_arm_unsat :
    ∀ tx a : Bool, tx = false → a = true →
      ¬ (armPoly a tx true = 0 ∧ arm2PolyA a tx true = 0
           ∧ arm2PolyB a tx true = 0) := by
  intro tx a htx ha
  rw [htx, ha]
  simp [armPoly, bit]

end Polyrust
