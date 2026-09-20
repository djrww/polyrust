-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-!
# M3-C7 — 自遞迴模板：階乘之燃料模型與端到端正確性

對應 Rust：`core/src/polyir.rs` 的 `self_recursion_step`（單一 int 參數、
單一 self-call、實參 = 參數別名 ± c 單調步進）與 `rec_depth_cap`
（模擬深度上限 = |p₀|/|Δ| + 8，硬上限 4096；欠深度如實降級）。CLI 實證：
真檔 `fact` n∈{0,1,3,5,10} 全 CERTIFIED（1/1/6/120/3628800）；n=100
checked 溢出如實排除；n=100000 深限如實降級。

數學內容：
1. `fact`：自然數階乘（結構遞迴——Rust 側模板嘅功能規格）。
2. `factF`：燃料版（fuel 显式——欠燃料唔聲稱結果，對應 simulator
   深度上限的「誠實降級」原則）。
3. `factF_spec`：**燃料充足（n ≤ fuel）時 `factF fuel n = fact n`**——
   遞迴模板認證嘅數學內核。
4. `fact_pos`：階乘恆正（認證宣稱值嘅非平凡性見證）。
-/

namespace Polyrust

/-- 階乘（C7 遞迴模板嘅功能規格）。 -/
def fact : Nat → Nat
  | 0 => 1
  | n + 1 => (n + 1) * fact n

/-- 燃料版階乘（對應 Rust simulate 深度上限；欠燃料唔聲稱結果）。 -/
def factF : Nat → Nat → Nat
  | 0, _ => 1
  | _ + 1, 0 => 1
  | fuel + 1, n + 1 => (n + 1) * factF fuel n

/-- **燃料正確性**：`n ≤ fuel` 時，燃料版 = 結構定義。呢條係 C7 遞迴
模板認證嘅數學內核（模擬展開 = 真實遞迴語義）。 -/
theorem factF_spec : ∀ fuel n, n ≤ fuel → factF fuel n = fact n := by
  intro fuel
  induction fuel with
  | zero =>
    intro n h
    have h1 : n = 0 := by omega
    subst h1
    rfl
  | succ m ih =>
    intro n h
    cases n with
    | zero => rfl
    | succ k =>
      have h1 : k ≤ m := by omega
      show (k + 1) * factF m k = fact (k + 1)
      rw [ih k h1]
      rfl

/-- 階乘恆正（認證宣稱值嘅非平凡性見證）。 -/
theorem fact_pos : ∀ n, 0 < fact n := by
  intro n
  induction n with
  | zero => exact Nat.zero_lt_one
  | succ k ih =>
    show 0 < (k + 1) * fact k
    exact Nat.mul_pos (Nat.succ_pos k) ih

end Polyrust
