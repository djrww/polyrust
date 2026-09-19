-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 監視文字移動的健全性（雙監視文字方案，v0.1.4 新增）

形式化 `src/cdcl.rs` `propagate` 的**雙監視文字（two-watched-literal）
方案**的核心不變量。子句 `C` 只監視兩個文字（位置 0、1）；當某個被監視
文字 `f` 被當前賦值否證時，求解器在位置 ≥ 2 找一個「非假」文字 `l`，
把監視從 `f` **交換**到 `l`：

    C  = f :: B :: pre ++ l :: post
    C' = l :: B :: pre ++ f :: post      （f 與 l 對調，子句多重集不變）

本模組證明：

* `watch_move0_preserves_sat` / `watch_move1_preserves_sat`——
  **交換版移動不改變子句語義**（任意賦值下滿足性等價）；
* `clauseSat_all_false`——子句全部文字為假 ⟹ 不滿足
  （衝突偵測的健全性：回報衝突時子句確實在當前賦值下為假）；
* `watch_overwrite_unsound`——**覆寫版**（丟失被否證文字、`l` 留存原位）
  有顯式反例：子句語義被改變。這正是 @brute 對照常態化在純 3-SAT
  硬樣本上抓到的 v0.1.4 缺陷的數學紀錄（`docs/EVIDENCE.md` §10b）：
  覆寫後子句變強，回溯到使 `f` 為真的賦值時造成不可靠剪枝，
  SAT 誤報 UNSAT。交換版（上兩條定理）是唯一健全的寫法。

與 `Polyrust.ClauseAlgebra`（消解/學習子句的**推理規則**健全性）互補：
本模組管的是**資料結構層**（監視指針移動不改子句內容），
`ClauseAlgebra` 管的是**邏輯層**（導出的子句被原子句集蘊含）。
兩者合成：CDCL 傳播迴圈裡「子句表語義守恆」的完整論證骨幹。 -/

import Polyrust.ClauseDuality

namespace Polyrust

/-! ## 子句滿足的成员刻畫 -/

/-- 子句滿足的存在量詞刻畫：`clauseSat`（`List.any` 的布爾編碼）
等價於「存在屬於子句的真文字」。 -/
theorem clauseSat_iff_exists (σ : Assignment) (C : List Lit) :
    clauseSat σ C = true ↔ ∃ l ∈ C, litSat σ l = true := by
  induction C with
  | nil => simp [clauseSat]
  | cons l C' ih =>
    simp only [clauseSat, List.any_cons, Bool.or_eq_true, List.mem_cons]
    grind

/-- 成員集相同的兩個子句滿足性相同（`clauseSat` 只看成員、不看順序與重複）。 -/
theorem clauseSat_congr_mem (σ : Assignment) {C D : List Lit}
    (h : ∀ x, x ∈ C ↔ x ∈ D) : clauseSat σ C = clauseSat σ D := by
  cases hC : clauseSat σ C <;> cases hD : clauseSat σ D
  · rfl
  · -- C 假、D 真：D 的某個真文字也在 C 中 ⟹ C 真，矛盾
    exfalso
    rcases (clauseSat_iff_exists σ D).mp hD with ⟨l, hm, hs⟩
    have hc := (clauseSat_iff_exists σ C).2 ⟨l, (h l).2 hm, hs⟩
    rw [hC] at hc
    exact Bool.noConfusion hc
  · -- C 真、D 假：對稱
    exfalso
    rcases (clauseSat_iff_exists σ C).mp hC with ⟨l, hm, hs⟩
    have hd := (clauseSat_iff_exists σ D).2 ⟨l, (h l).1 hm, hs⟩
    rw [hD] at hd
    exact Bool.noConfusion hd
  · rfl

/-! ## 監視移動（交換版）健全性 -/

/-- **監視移動健全性（監視位 0）**：把被否證的監視文字 `f`（位置 0）
與位置 ≥ 2 的文字 `l` **交換**，子句語義不變——
對應 `src/cdcl.rs` `propagate` 中 `c2[0] = l; c2[k] = falsified`。 -/
theorem watch_move0_preserves_sat (σ : Assignment) (f B l : Lit)
    (pre post : List Lit) :
    clauseSat σ (f :: B :: pre ++ l :: post) =
    clauseSat σ (l :: B :: pre ++ f :: post) := by
  apply clauseSat_congr_mem
  intro x
  simp [List.mem_append]
  grind

/-- **監視移動健全性（監視位 1）**：同 `watch_move0_preserves_sat`，
監視文字在位置 1 的情形（`c2[1] = l; c2[k] = falsified`）。 -/
theorem watch_move1_preserves_sat (σ : Assignment) (B f l : Lit)
    (pre post : List Lit) :
    clauseSat σ (B :: f :: pre ++ l :: post) =
    clauseSat σ (B :: l :: pre ++ f :: post) := by
  apply clauseSat_congr_mem
  intro x
  simp [List.mem_append]
  grind

/-- 工程不變量形式：移動前子句被滿足 ⟹ 移動後仍被滿足
（`watch_move0_preserves_sat` 的單向推論；學習到的子句經任意多次
監視移動後仍是原式的有效後承）。 -/
theorem watch_move_sound (σ : Assignment) (f B l : Lit) (pre post : List Lit) :
    clauseSat σ (f :: B :: pre ++ l :: post) = true →
    clauseSat σ (l :: B :: pre ++ f :: post) = true := by
  intro h
  rw [← watch_move0_preserves_sat]
  exact h

/-- 具體實例（同 `watch_overwrite_unsound` 的子句）：交換版對**任意**賦值
保持滿足性——與覆寫版的反例形成對照。 -/
theorem watch_move0_example_sound (σ : Assignment) :
    clauseSat σ ([⟨0, true⟩, ⟨1, true⟩, ⟨2, true⟩] : List Lit) =
    clauseSat σ ([⟨2, true⟩, ⟨1, true⟩, ⟨0, true⟩] : List Lit) :=
  watch_move0_preserves_sat σ ⟨0, true⟩ ⟨1, true⟩ ⟨2, true⟩ [] []

/-! ## 衝突偵測的健全性 -/

/-- 子句的每個文字都被賦值否證 ⟹ 子句不滿足。
這是 `propagate` 回報衝突的前提：只有當監視文字與另一監視文字皆假、
且位置 ≥ 2 找不到非假文字時才回報——此時子句**確實**全假。 -/
theorem clauseSat_all_false (σ : Assignment) (C : List Lit)
    (h : ∀ l ∈ C, litSat σ l = false) : clauseSat σ C = false := by
  cases hc : clauseSat σ C with
  | false => rfl
  | true =>
    exfalso
    rcases (clauseSat_iff_exists σ C).mp hc with ⟨l, hm, hl⟩
    rw [h l hm] at hl
    exact Bool.noConfusion hl

/-! ## 覆寫版的反例（v0.1.4 缺陷紀錄） -/

/-- **覆寫版監視移動不健全**：若移動寫成 `c2[0] = l` 而不把被否證文字
換回 `l` 的原位（子句從 `[x₀, x₁, x₂]` 變成 `[x₂, x₁, x₂]`，`x₀` 丟失），
則存在賦值滿足原式而不滿足覆寫後的子句（σ：x₀ 真、x₁ x₂ 假）。
回溯後這類「變強」的子句繼續生效 ⇒ 不可靠剪枝 ⇒ SAT 誤報 UNSAT。
（純 3-SAT @brute 對照常態化實測抓到；修復見 `src/cdcl.rs`。） -/
theorem watch_overwrite_unsound :
    ∃ (σ : Assignment),
      clauseSat σ ([⟨0, true⟩, ⟨1, true⟩, ⟨2, true⟩] : List Lit) = true ∧
      clauseSat σ ([⟨2, true⟩, ⟨1, true⟩, ⟨2, true⟩] : List Lit) = false := by
  refine ⟨fun v => v == 0, ?_⟩
  constructor <;> decide

/-! ## 任意步數的語義守恆（「必然如初」的機械化） -/

/-- 一步合法監視移動（交換版）的歸納關係：位置 0 或位置 1 的監視文字
與位置 ≥ 2 的文字對調。 -/
inductive WatchMove : List Lit → List Lit → Prop where
  | move0 (f B l : Lit) (pre post : List Lit) :
      WatchMove (f :: B :: pre ++ l :: post) (l :: B :: pre ++ f :: post)
  | move1 (B f l : Lit) (pre post : List Lit) :
      WatchMove (B :: f :: pre ++ l :: post) (B :: l :: pre ++ f :: post)

/-- 單步移動保持滿足性（由 `watch_move0/1_preserves_sat` 直接合成）。 -/
theorem watchMove_preserves_sat (σ : Assignment) {C D : List Lit}
    (h : WatchMove C D) : clauseSat σ C = clauseSat σ D := by
  cases h with
  | move0 f B l pre post => exact watch_move0_preserves_sat σ f B l pre post
  | move1 B f l pre post => exact watch_move1_preserves_sat σ B f l pre post

/-- 任意有限步監視移動的複合關係。 -/
inductive WatchMoves : List Lit → List Lit → Prop where
  | refl (C : List Lit) : WatchMoves C C
  | step {C D E : List Lit} : WatchMove C D → WatchMoves D E → WatchMoves C E

/-- **必然如初（語義版）**：子句經**任意步數**的合法監視移動後，
在任何賦值下的滿足性與原式完全相同——不需要「恢復」子句，
因為每一步都保持語義，歸納即得全程守恆。 -/
theorem watchMoves_preserve_sat (σ : Assignment) {C D : List Lit}
    (h : WatchMoves C D) : clauseSat σ C = clauseSat σ D := by
  induction h with
  | refl C => rfl
  | step h1 h2 ih => rw [watchMove_preserves_sat σ h1, ih]

/-- 推論：移動後的子句在模型下仍被滿足（工程不變量的迭代形式）。 -/
theorem watchMoves_sound (σ : Assignment) {C D : List Lit}
    (h : WatchMoves C D) : clauseSat σ C = true → clauseSat σ D = true := by
  intro hs
  rw [← watchMoves_preserve_sat σ h]
  exact hs

end Polyrust
