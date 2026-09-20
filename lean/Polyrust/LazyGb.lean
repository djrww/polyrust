-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # LazyGb：lazy 判定 ≡ eager 判定（P0-C4 管線層精化）

`pipeline.rs` 的 `GbBasisMode`：

* **Eager**：S6 對 merged 系統（原多項式 + field 多項式 `x²−x` + 全部子句
  多項式）跑全量既約 Gröbner 基；`is_unsat ⟺ G = {1}`。
* **Lazy**：跳過 S6 全基，`is_unsat = cdcl_failed`（CDCL(T) 迴圈的最終判定）。

數學上（T3/T6 已具體系於 `ClauseAlgebra`/`BoolNullstellensatz`/`T6Certificate`）：
布爾根之否定 ⟺ 無公共零點 ⟺ 1 ∈ 理想 ⟺ 既約基為 {1}。故 **eager 判定 =
「不存在滿足骨架子句 ∧ 理論約束的布爾模型」**——本模組以此為 eager 的規格。

lazy 迴圈的每一輪：CDCL 對當前骨架求布爾模型 `m*`；理論端（GB 對
pinning 後系統）判定 `m*` 的立方體下有無布爾根——無則把立方體否句
（`lc`）補入骨架再迴圈。本模組證明**管線層精化**：

1. `negCube_satisfied_of_not_agree`／`negCube_none_holds_iff_agree`——
   立方體否句的滿足語義（補理形式正確）；
2. `negCube_step_valid`——Rust 每輪 `1 ∈ G(with_field)` 給出的 pinning
   無根前提（對應 `T6Certificate` 的證書消去，Rust 端由 GB 引擎承擔）
   ⟹ 該輪補理係理論邏輯後承（鏈步合法）；
3. `ValidChain`／`validChain_strengthens`——逐步合法補理鏈組合為全域強化：
   任何「初始骨架 ∧ 理論」模型都滿足終局骨架；
4. **判決一致（主線）**：`lazy_unsat_sound`（lazy UNSAT ⟹ eager 無根）與
   `eager_sat_of_lazy_witness`（lazy SAT 見證 ⟹ eager 有根，兩出口互斥
   且覆蓋）——只要迴圈終止且每輪理論前提成立，lazy 判決與 eager 判決
   **恰一致**。

信任邊界如實申報：鏈步前提（每輪 pinning 系統無布爾根）喺 Rust 端由
reduced-Groebner 引擎承擔，與 SMT 證書中理論補理作為可信推論一致；
本模組證明嘅係**精化**——喺該前提下兩種判定數學上重合。
-/

import Polyrust.RupKernel

namespace Polyrust

namespace LazyGb

open Classical
open Polyrust.RupKernel

/-- 立方體：被 pinning 的變量清單（Rust 迴圈中現行布爾模型的全域賦值段）。 -/
abbrev Cube := List (Var × Bool)

/-- 模型同意立方體：全部 pinning 變量取值如約。 -/
def Agree (σ : Model) (cube : Cube) : Prop := ∀ p ∈ cube, σ p.1 = p.2

/-- 立方體否句：每對 `(v, b)` 出文字 `(v, ¬b)`。Rust `lc` 的數學形。 -/
def negCube (cube : Cube) : Clause := cube.map (fun p => ⟨p.1, !p.2⟩)

/-- 否句全部文字皆假 ⟺ 模型同意立方體。 -/
theorem negCube_none_holds_iff_agree {σ : Model} {cube : Cube} :
    (∀ l ∈ negCube cube, ¬ l.Holds σ) ↔ Agree σ cube := by
  constructor
  · intro hH p hp
    have hl : (⟨p.1, !p.2⟩ : Lit) ∈ negCube cube :=
      List.mem_map.mpr ⟨p, hp, rfl⟩
    have hnn := hH _ hl
    cases hv : σ p.1 <;> cases hb : p.2 <;> simp_all [Lit.Holds]
  · intro hag l hl hh
    obtain ⟨p, hp, rfl⟩ := List.mem_map.mp hl
    have hpin : σ p.1 = p.2 := hag p hp
    have hE : σ p.1 = !p.2 := hh
    rw [hpin] at hE
    cases hb : p.2 <;> simp [hb] at hE

/-- 否句可滿足 ⟸ 模型不同意立方體（比較方向需要布爾排中）。 -/
theorem negCube_satisfied_of_not_agree {σ : Model} {cube : Cube}
    (h : ¬ Agree σ cube) : (negCube cube).Satisfied σ := by
  apply Classical.byContradiction
  intro hnsat
  apply h
  apply (negCube_none_holds_iff_agree (σ := σ) (cube := cube)).mp
  intro l hl hh
  exact hnsat ⟨l, hl, hh⟩

/-- **迴圈步合法**（Rust 每輪 `1 ∈ G(with_field)` 的精化後承）：
    理論在立方體 pinning 下無模型 ⟹ 加入立方體否句保持「骨架 ∧ 理論」模型集。 -/
theorem negCube_step_valid {T : Model → Prop} {db : Cnf} {cube : Cube}
    (hpin : ∀ m, Agree m cube → ¬ T m) :
    ∀ m, db.Satisfied m → T m → (negCube cube).Satisfied m := by
  intro m _ hT
  exact negCube_satisfied_of_not_agree (fun hag => hpin m hag hT)

/-- **依序合法補理鏈**（CDCL(T) 迴圈演化的抽象）：
    每步加入的子句係「已強化骨架 ∧ 理論」的邏輯後承。 -/
inductive ValidChain (T : Model → Prop) (initial : Cnf) : Cnf → Prop where
  /-- 空鏈：初始骨架自身。 -/
  | nil : ValidChain T initial initial
  /-- 一步：`L` 喺現行骨架 `db` 與理論 `T` 下係後承。 -/
  | cons {db : Cnf} {L : Clause} :
      ValidChain T initial db →
      (∀ m, db.Satisfied m → T m → L.Satisfied m) →
      ValidChain T initial (L :: db)

/-- **逐步合法 ⟹ 全域強化**：任何「初始骨架 ∧ 理論」模型都滿足終局骨架。
    迴圈單調性的管線層精化不變量。 -/
theorem validChain_strengthens {T : Model → Prop} {initial final : Cnf}
    (h : ValidChain T initial final) :
    ∀ m, initial.Satisfied m → T m → final.Satisfied m := by
  induction h with
  | nil => intro m hI _ c hc; exact hI c hc
  | @cons db L _ hL ih =>
      intro m hI hT c hc
      have hdbm : db.Satisfied m := ih m hI hT
      rcases List.mem_cons.mp hc with rfl | hmem
      · exact hL m hdbm hT
      · exact hdbm c hmem

/-- 強化關係嘅別名（文檔化用）：`final` 係 `initial` 嘅理論強化。 -/
def Strengthens (T : Model → Prop) (initial final : Cnf) : Prop :=
  ∀ m, initial.Satisfied m → T m → final.Satisfied m

/-- **eager 判定規格**：merged 系統無布爾根
    （佈爾模型滿足初始骨架且滿足理論者不存在；`is_unsat ⟺ G = {1}` 的語義面）。 -/
def EagerUnsat (T : Model → Prop) (initial : Cnf) : Prop :=
  ∀ m, ¬ (initial.Satisfied m ∧ T m)

/-- **主線一：lazy UNSAT 健全**。合法鏈終局骨架布爾無解 ⟹ eager 無根。 -/
theorem lazy_unsat_sound {T : Model → Prop} {initial final : Cnf}
    (hc : ValidChain T initial final) (hfin : ∀ m, ¬ final.Satisfied m) :
    EagerUnsat T initial := by
  intro m ⟨hI, hT⟩
  exact hfin m (validChain_strengthens hc m hI hT)

/-- **合法鏈超集性**：鏈只加唔減——初始骨架每條子句都留存於終局骨架
    （對應 Rust `clauses_sys.push(lc)` 嘅 append-only 性質）。 -/
theorem validChain_superset {T : Model → Prop} {initial final : Cnf}
    (h : ValidChain T initial final) : ∀ c ∈ initial, c ∈ final := by
  induction h with
  | nil => intro c hc_mem; exact hc_mem
  | @cons db L _ _ ih => intro c hc_mem; exact List.mem_cons_of_mem L (ih c hc_mem)

/-- **主線二：lazy SAT 緊緻**。迴圈產出「終局骨架 ∧ 理論」模型見證 ⟹
    eager 有根（初始 ⊆ 終局，見證直接係 merged 布爾根）。兩出口互斥。 -/
theorem eager_sat_of_lazy_witness {T : Model → Prop} {initial final : Cnf}
    (hsub : ∀ c ∈ initial, c ∈ final)
    (h : ∃ m, final.Satisfied m ∧ T m) : ¬ EagerUnsat T initial := by
  intro hU
  obtain ⟨m, hF, hT⟩ := h
  have hI : initial.Satisfied m := fun c hc_mem => hF c (hsub c hc_mem)
  exact hU m ⟨hI, hT⟩

/-- **判決一致（P0-C4 終點）：lazy ≡ eager**。
    合法補理鏈下，lazy 兩個出口各自精化為 eager 判決的同名結論：
    UNSAT 出口健全（`lazy_unsat_sound`）、SAT 出口緊緻（`eager_sat_of_lazy_witness`）。
    由布爾排他（「存在 merged 布爾根」與其否定窮盡全部情形），
    只要迴圈終止且每輪理論前提成立，兩種判定**數學上重合**。 -/
theorem lazy_refines_eager {T : Model → Prop} {initial final : Cnf}
    (hc : ValidChain T initial final) :
    ((∀ m, ¬ final.Satisfied m) → EagerUnsat T initial) ∧
    ((∃ m, final.Satisfied m ∧ T m) → ¬ EagerUnsat T initial) :=
  ⟨lazy_unsat_sound hc, eager_sat_of_lazy_witness (validChain_superset hc)⟩

end LazyGb

end Polyrust
