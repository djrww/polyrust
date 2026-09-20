-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-
C5.2 — Polyir.UnrollSound：bounded unroll 編碼忠實性 + 抽象層 UNSAT 反映。

對應 Rust：`core/src/llbc_lower.rs`（C3 bounded unroll `@fuel` K 輪、
C2/C4 abstraction markers 語義表抽象化）。

主張三個（都係核心引擎依賴嘅硬保證）：

1. `steps_add`：`k₁ + k₂` 步到達 ⟺ 可拆成兩段。
   bounded unroll 所捕捉嘅關係**就係**步數語義本身——編碼忠實，
   唔係過近似（over-approx）亦唔係欠近似（under-approx）。
   呢個係「unroll 之後仲可以繼續拆段 reasoning」（fuel 語言）嘅根。

2. `steps_mono`：轉移關係放大（抽象層 `step_abs ⊇ step`）⇒ 步數到達關係放大。
   即係：原語義每條執行軌跡，喺合法抽象層都仲存在（抽象只會加唔會減）。

3. `unroll_unsat_reflects`：將 (2) 包成引擎語言——
   **抽象層（GB）判 UNSAT ⟹ 原語義無 K 步執行**。
   呢個先係 abstraction markers 可以「反映判決」嘅邏輯根，
   亦係 C3 `loop-fuel-exhausted → UNKNOWN` 口徑嘅健全性基石：
   UNSAT 口徑至上，任何標記抽象唔會製造假 UNSAT。

唔用 Mathlib，Lean 4 core 自包含；三個定理都要 `#print axioms` 零 sorry。
-/

namespace Polyrust
namespace UnrollSound

universe u

/-- `Steps step k a c`：由 `a` 經 `step` 恰行 `k` 步到 `c`。
    原始（未抽象）語義同抽象語義都係呢個型——平行定義唔需要。 -/
inductive Steps {α : Type u} (step : α → α → Prop) : Nat → α → α → Prop where
  | refl : Steps step 0 a a
  | step {k a b c} : step a b → Steps step k b c → Steps step (k + 1) a c

/-- (1) 步數加法：k₁+k₂ 步到達 ⟺ 中段存在。兩個方向都證——
    正方向（拆段）係 continuation 融合嘅根，反方向（拼接）係多段 unroll 串合嘅根。 -/
theorem steps_add {α : Type u} {step : α → α → Prop} :
    ∀ (k1 : Nat) (a c : α) (k2 : Nat),
      Steps step (k1 + k2) a c ↔ ∃ b, Steps step k1 a b ∧ Steps step k2 b c := by
  intro k1
  induction k1 with
  | zero =>
    intro a c k2
    constructor
    · intro h
      -- 0 + k2 唔會自行 definitional 歸約（Nat.add 遞歸喺右參數），要 rewrite
      rw [Nat.zero_add] at h
      exact ⟨a, Steps.refl, h⟩
    · rintro ⟨b, hb, h⟩
      cases hb
      rw [Nat.zero_add]
      exact h
  | succ k1 ih =>
    intro a c k2
    constructor
    · intro h
      -- (k1+1)+k2 = (k1+k2)+1，對返下一步 case 先至拆得郁
      rw [Nat.succ_add] at h
      cases h with
      | step hab hrest =>
        obtain ⟨b, hb1, hb2⟩ := ((ih _ _ k2).mp) hrest
        exact ⟨b, Steps.step hab hb1, hb2⟩
    · rintro ⟨b, hb, h2⟩
      cases hb with
      | step hab hrest =>
        have ih' := ((ih _ _ k2).mpr) ⟨b, hrest, h2⟩
        rw [Nat.succ_add]
        exact Steps.step hab ih'

/-- (2) 抽象化單調：轉移關係放大 ⇒ K 步到達關係放大。
    「合法抽象層唔會刪走真執行」嘅形式化。 -/
theorem steps_mono {α : Type u} {step sub : α → α → Prop}
    (h : ∀ a b, step a b → sub a b) :
    ∀ {k : Nat} {a c : α}, Steps step k a c → Steps sub k a c := by
  intro k
  induction k with
  | zero =>
    intro _ _ h
    cases h
    exact Steps.refl
  | succ k ih =>
    intro _ _ h
    cases h with
    | step h1 hrest =>
      exact Steps.step (h _ _ h1) (ih hrest)

/-- (3) 引擎語言版：抽象層無 K 步執行 ⟹ 原語義無 K 步執行。
    配合 `steps_add`（編碼忠實）：bounded unroll + abstraction 嘅
    UNSAT 判決對原語義健全——假 UNSAT 唔可能存在。 -/
theorem unroll_unsat_reflects {α : Type u} {step_abs step : α → α → Prop}
    (hsub : ∀ a b, step a b → step_abs a b) {K : Nat} :
    (∀ a c, ¬ Steps step_abs K a c) → (∀ a c, ¬ Steps step K a c) := by
  intro hnosat a c h
  exact hnosat a c (steps_mono (α := α) hsub h)

end UnrollSound
end Polyrust
