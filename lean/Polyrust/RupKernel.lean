-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # RUP 證書核健全性（P0-C2 形式化部）

形式化 `core/src/lrat.rs` 的 **RUP（Reverse Unit Propagation）推理規則**：檢查器
要加入子句 `C` 時，假設 `¬C`（C 的全部文字為假），對「初始子句 + 已接受子句」
做單元傳播，必須導出衝突。本模組證明：凡經 RUP 鏈條逐步接納、且某步加入空
子句的證明，其基底 CNF **在任何模型下都不滿足**——UNSAT 判定可靠。

與 Rust 檢查器（`lrat.rs`）的逐項對應
--------------------------------------
* `Propagates F asms l` ↔ Rust `unit_propagate` 收斂後賦值（或假設）為真的
  全體文字之聲明式閉包。橋接論證：Rust 端每個被賦值的文字，其來源只有二——
  (i) `is_rup` 假設 `¬C` 的直接攝入（對應 `asm`）；(ii) 子句只剩一個未賦值
  文字時的强制（對應 `unit`）。故按賦值次序歸納，Rust 的賦值集 ⊆ 本閉包；
  Rust 報告衝突（`n_unassigned == 0 && !satisfied`）⟹ 本定義 `UpConflict`。
* `UpConflict` ↔ Rust `unit_propagate` 回傳 `true`（存在被全否的子句；
  空子句直接命中，與 Rust 對基底含空子句立即衝突的特判一致）。
* `IsRup` ↔ Rust `is_rup`（含越界拒絕在內的保守分支是實現細節，
  健全性定理只需其成功路徑的數學後承）。
* `Accepts` ↔ Rust `check_rup_proof` 的逐步迴圈（每一步以「基底 + 先前
  接納過的全部子句」為資料庫做 RUP 檢查）。
* `rup_kernel_sound` ↔ Rust 返回 `Ok(steps)` 的語義後承：證明中含空子句 ⟹
  基底 CNF 無解。Rust 於首個空子句即返回、忽略其後步驟——取前綴即本定理的
  `proof`。

設計約束（對齊專案文化）：無 Mathlib、自包含；謝絕 `sorry`。證明只用
Lean 核心（可能動用 `Classical`，審計以 `#print axioms` 度三公理為 CLEAN）。
-/

namespace Polyrust

namespace RupKernel

open Classical

/-- 布爾變量。 -/
abbrev Var := Nat

/-- 文字：變量與極性。對應 DIMACS 的 `±v`（Rust `lrat.rs::Clause = Vec<i32>`）。 -/
structure Lit where
  var : Var
  pos : Bool
  deriving DecidableEq

/-- 反置文字。 -/
def Lit.neg (l : Lit) : Lit := ⟨l.var, !l.pos⟩

/-- 子句 = 文字的析取；空子句為 ⊥。 -/
abbrev Clause := List Lit

/-- CNF = 子句的合取。 -/
abbrev Cnf := List Clause

/-- 總賦值：每變量一個布爾值。 -/
abbrev Model := Var → Bool

/-- 文字於模型下成立。 -/
def Lit.Holds (σ : Model) (l : Lit) : Prop := σ l.var = l.pos

/-- 子句滿足：存在成立文字。空子句永不滿足（⊥ 語義自明）。 -/
def Clause.Satisfied (σ : Model) (c : Clause) : Prop := ∃ l ∈ c, l.Holds σ

/-- CNF 滿足：所有子句滿足。 -/
def Cnf.Satisfied (σ : Model) (F : Cnf) : Prop := ∀ c ∈ F, c.Satisfied σ

/-- 文字與其反置不可能同時成立。 -/
theorem Lit.not_holds_both {σ : Model} {l : Lit} (h1 : l.Holds σ) (h2 : l.neg.Holds σ) :
    False := by
  have h1' : σ l.var = l.pos := h1
  have h2' : σ l.var = !l.pos := h2
  have e : l.pos = !l.pos := h1'.symm.trans h2'
  cases hp : l.pos <;> simp [hp] at e

/-- 文字不成立 ⟹ 其反置成立（布爾排中）。 -/
theorem Lit.holds_neg_of_not_holds {σ : Model} {l : Lit} (h : ¬ l.Holds σ) :
    l.neg.Holds σ := by
  have h' : ¬ (σ l.var = l.pos) := h
  have hb : σ l.var = !l.pos := by
    cases hσ : σ l.var <;> cases hl : l.pos <;> simp_all
  have : l.neg.Holds σ = (σ l.var = !l.pos) := rfl
  rw [this]; exact hb

/-- 單元傳播閉包（聲明式）：假設集合 `asms` 下，由 CNF `F` 强制為真的全體文字。
    為**最小**閉包——Rust `unit_propagate` 的收斂賦值集 ⊆ 此閉包（見檔首論證）。 -/
inductive Propagates (F : Cnf) (asms : List Lit) : Lit → Prop where
  /-- 假設攝入：RUP 步中 `¬C` 的每個文字直接成立。 -/
  | asm {l : Lit} : l ∈ asms → Propagates F asms l
  /-- 單元規則：子句 `c` 中除 `l` 外所有文字的反置皆可導出，則 `l` 可導出。
      （單位子句情形：全稱前件空真，`l` 直接可導出。） -/
  | unit {c : Clause} {l : Lit} :
        c ∈ F → l ∈ c →
        (∀ l' ∈ c, l' ≠ l → Propagates F asms l'.neg) →
        Propagates F asms l

/-- UP 衝突：存在一條子句，其全部文字的反置皆可導出。
    對應 Rust `unit_propagate` 的 `n_unassigned == 0 && !satisfied → true`。 -/
def UpConflict (F : Cnf) (asms : List Lit) : Prop :=
  ∃ c ∈ F, ∀ l ∈ c, Propagates F asms l.neg

/-- RUP 步：假設 `¬C` 後於 `F` 上經單元傳播導出衝突。對應 Rust `is_rup`。 -/
def IsRup (F : Cnf) (c : Clause) : Prop :=
  UpConflict F (c.map Lit.neg)

/-- 閉包可靠：凡可導出的文字，在任何滿足 `F` 且滿足假設的模型下必然成立。 -/
theorem propagates_sound {F : Cnf} {asms : List Lit} {σ : Model}
    (hF : F.Satisfied σ) (hasm : ∀ a ∈ asms, a.Holds σ) {l : Lit}
    (h : Propagates F asms l) : l.Holds σ := by
  induction h with
  | asm hin => exact hasm _ hin
  | @unit c l hc _hmem hsub ih =>
      -- c ∈ F 被 σ 滿足：存在成立文字 l₀；其餘文字皆被 σ 否證（歸納假設），
      -- 故 l₀ 只能是被强制的 l。
      obtain ⟨l₀, hl₀, hh₀⟩ := hF c hc
      by_cases heq : l₀ = l
      · subst heq; exact hh₀
      · have hneg : l₀.neg.Holds σ := ih l₀ hl₀ heq
        exact (Lit.not_holds_both hh₀ hneg).elim

/-- RUP 步可靠（課程引理）：若 `C` 是 `F` 的 RUP，則 `F` 的任何模型都滿足 `C`。
    即 RUP 步**保模型**——把 `C` 加入資料庫不動滿足性。 -/
theorem is_rup_sound {F : Cnf} {σ : Model} {c : Clause}
    (hF : F.Satisfied σ) (hr : IsRup F c) : c.Satisfied σ := by
  obtain ⟨d, hd, hconf⟩ := hr
  apply Classical.byContradiction
  intro hnsat
  -- σ 不滿足 c ⟹ c 的每個文字皆假 ⟹ 假設集 ¬c 全部成立。
  have hf : ∀ l ∈ c, ¬ l.Holds σ := fun l hl hH => hnsat ⟨l, hl, hH⟩
  have hasm : ∀ a ∈ c.map Lit.neg, a.Holds σ := by
    intro a ha
    obtain ⟨l, hl, rfl⟩ := List.mem_map.mp ha
    exact Lit.holds_neg_of_not_holds (hf l hl)
  -- 於是衝突子句 d 的每個文字都被 σ 否證（反置可導出 ⟹ 反置成立 ⟹ 本文字不成立）。
  have hdall : ∀ l ∈ d, ¬ l.Holds σ := by
    intro l hl hH
    exact Lit.not_holds_both hH (propagates_sound hF hasm (hconf l hl))
  -- 但 d ∈ F 必被 σ 滿足——矛盾。
  obtain ⟨l₀, hl₀, hh₀⟩ := hF d hd
  exact hdall l₀ hl₀ hh₀

/-- RUP 證明鏈條：每步以「基底 + 已接納步」為資料庫。對應 Rust `check_rup_proof`
    的累積語義（其 db := 初始子句 ++ 已接受子句）。
    `F` 為**索引**而非參數（`cons` 遞歸的資料庫增長，參數定位唔啱）。 -/
inductive Accepts : Cnf → List Clause → Prop where
  | nil {F : Cnf} : Accepts F []
  | cons {F : Cnf} {c : Clause} {tl : List Clause} :
      IsRup F c → Accepts (c :: F) tl → Accepts F (c :: tl)

/-- 鏈條不變量：基底公式的任何模型都滿足鏈條中的每條子句。 -/
theorem accepts_sound {F : Cnf} {proof : List Clause} (h : Accepts F proof) :
    ∀ {σ : Model}, F.Satisfied σ → ∀ c ∈ proof, c.Satisfied σ := by
  induction h with
  | nil => intro _ _ c hc; exact absurd hc List.not_mem_nil
  | @cons F c tl hr _sub ih =>
      intro σ hF c' hc'
      have hc_sat : c.Satisfied σ := is_rup_sound hF hr
      rcases List.mem_cons.mp hc' with rfl | hmem
      · exact hc_sat
      · have hF' : Cnf.Satisfied σ (c :: F) := by
          intro d hd
          rcases List.mem_cons.mp hd with rfl | hdF
          · exact hc_sat
          · exact hF d hdF
        exact ih hF' c' hmem

/-- **主定理（RUP 核可靠）**：RUP 鏈條接納且其中加入過空子句 ⟹ 基底 CNF 無解。
    對應 Rust `check_rup_proof` 返回 `Ok(_)` 的語義後承——UNSAT 判決可信。 -/
theorem rup_kernel_sound {F : Cnf} {proof : List Clause}
    (hacc : Accepts F proof) (hbot : [] ∈ proof) :
    ∀ σ : Model, ¬ F.Satisfied σ := by
  intro σ hF
  have h := accepts_sound hacc hF [] hbot
  obtain ⟨l, hl, -⟩ := h
  exact List.not_mem_nil hl

/-- 推論（空子勺 RUP 步的內容）：末端資料庫若 UP 不一致，直接無解。
    對應 `finish_unsat` 補空子句後檢查器的「`[]` 為 RUP」驗證路徑：
    空子句的 RUP 檢查（假設集為空）成立 ⟹ 資料庫本身無解。 -/
theorem rup_empty_sound {F : Cnf} (h : IsRup F []) : ∀ σ : Model, ¬ F.Satisfied σ :=
  fun σ hF => by
    have := is_rup_sound hF h
    rcases this with ⟨l, hl, -⟩
    exact List.not_mem_nil hl

end RupKernel

end Polyrust
