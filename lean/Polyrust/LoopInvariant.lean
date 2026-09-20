-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/-!
# M3 — C6 簡單 while 迴圈：單調模板之不變式健全性與閉式解

對應 Rust：`core/src/polyir.rs` 的 `lower_loop`（Lt/Le/Gt/Ge 條件＋步進
i±c 單調趨界）與 `core/src/polyir_encode.rs` 的見證恆等化編碼；CLI 實證
首例 = `m0/spike_out/loop_invariant.llbc` 之 `inv_sum`（n=5 → CERTIFIED
ret=10）。

數學內容：
1. `whileMonoF`：C6 終止模板之功能模型（cond `i < bound`、步長
   `step + 1 ≥ 1` 嚴格單調；燃料顯式——欠燃料唔聲稱結果）。
2. `whileMonoF_invariant`：**不變式健全性**——入口成立、body 單調步
   保持嘅性質，迴圈離開時仍成立。呢條係 C6 模板「認證」嘅數學內核。
3. `whileMonoF_sufficient`：**出口條件**——燃料充足（`bound - i ≤ fuel`）
   時離開值必 `≥ bound`（對應 simulator 嘅 `¬cond` 離開）。
4. `triSum`/`invSumF_spec`：真檔首例 inv_sum（Σ_{k<n} k）之端到端
   正確性——迭代模擬終值 = 三角數閉式。
-/

namespace Polyrust

/-- 帶燃料嘅 C6 單調 while 模板（cond：`i < bound`、body：
`i := i + step + 1`）。`step` 係零基偏移——實際步長 `step + 1 ≥ 1`，
保證每輪嚴格單調（模板終止性嘅來源）。 -/
def whileMonoF : Nat → Nat → Nat → Nat
  | 0, _step, _bound, i => i
  | fuel + 1, step, bound, i =>
      if i < bound then whileMonoF fuel step bound (i + step + 1) else i

/-- **不變式健全性**：入口成立、每輪單調步保持嘅性質 `P`，迴圈離開時
仍然成立。（欠燃料中途停：結果係中途值，`P` 由歸納一樣保持——本定理
對任意 fuel 都成立，唔需要充足性假設。） -/
theorem whileMonoF_invariant (bound step : Nat) (P : Nat → Prop)
    (hmono : ∀ j, j < bound → P j → P (j + step + 1)) :
    ∀ fuel i, P i → P (whileMonoF fuel step bound i) := by
  intro fuel
  induction fuel with
  | zero => intro i h; exact h
  | succ n ih =>
    intro i h
    by_cases hlt : i < bound
    · simp only [whileMonoF, if_pos hlt]
      exact ih (i + step + 1) (hmono i hlt h)
    · simp only [whileMonoF, if_neg hlt]
      exact h

/-- **出口條件**：燃料充足（`bound - i ≤ fuel`）時，離開值必 `≥ bound`
（即 `¬(離開值 < bound)`——對應 Rust simulator 以 `cond.compare` 為假
離開迴圈）。欠燃料 = 模板假設失效，Rust 側如實降級，呢度唔聲稱。 -/
theorem whileMonoF_sufficient (bound step : Nat) :
    ∀ fuel i, bound - i ≤ fuel → bound ≤ whileMonoF fuel step bound i := by
  intro fuel
  induction fuel with
  | zero =>
    intro i h
    have h1 : bound ≤ i := by omega
    simp only [whileMonoF]
    exact h1
  | succ n ih =>
    intro i h
    by_cases hlt : i < bound
    · simp only [whileMonoF, if_pos hlt]
      have h2 : bound - (i + step + 1) ≤ n := by omega
      exact ih (i + step + 1) h2
    · simp only [whileMonoF, if_neg hlt]
      omega

/-- 三角數 `T n = Σ_{k<n} k`（`T 0 = 0`、`T (k+1) = T k + k`）。 -/
def triSum : Nat → Nat
  | 0 => 0
  | k + 1 => triSum k + k

theorem triSum_mono : ∀ b a, a ≤ b → triSum a ≤ triSum b := by
  intro b
  induction b with
  | zero =>
    intro a h
    have h1 : a = 0 := by omega
    subst h1
    exact Nat.le_refl _
  | succ n ih =>
    intro a h
    by_cases h2 : a ≤ n
    · have h3 := ih a h2
      have h4 : triSum (n + 1) = triSum n + n := rfl
      omega
    · have h5 : a = n + 1 := by omega
      subst h5
      exact Nat.le_refl _

/-- C6 真檔首例 inv_sum（`loop_invariant.llbc`）：`acc=0; i=0;
while (i<n) { acc+=i; i+=1 } ret=acc`。功能模型（燃料版）。 -/
def invSumF : Nat → Nat → Nat → Nat → Nat
  | 0, _n, _i, acc => acc
  | fuel + 1, n, i, acc => if i < n then invSumF fuel n (i + 1) (acc + i) else acc

/-- **端到端正確性**：燃料充足（`n - i ≤ fuel`）時，
`invSumF fuel n i acc = acc + T n − T i`。特例 `i=0, acc=0` 給出
`invSumF fuel n 0 0 = T n`（Σ_{k<n} k 嘅閉式）——即 CLI 對 n=5 認證
ret=10 = T 5 嘅形式化版本。 -/
theorem invSumF_spec (n : Nat) : ∀ fuel i acc, n - i ≤ fuel →
    invSumF fuel n i acc = acc + triSum n - triSum i := by
  intro fuel
  induction fuel with
  | zero =>
    intro i acc h
    have h1 : i ≥ n := by omega
    simp only [invSumF]
    have h2 : triSum n ≤ triSum i := triSum_mono i n (by omega)
    omega
  | succ m ih =>
    intro i acc h
    by_cases hlt : i < n
    · simp only [invSumF, if_pos hlt]
      have h1 : n - (i + 1) ≤ m := by omega
      have h6 : triSum (i + 1) = triSum i + i := rfl
      have h2 := ih (i + 1) (acc + i) h1
      have h4 : triSum i + i ≤ triSum n := by
        have h5 := triSum_mono (i + 1) n (by omega)
        omega
      omega
    · simp only [invSumF, if_neg hlt]
      have h2 : triSum n ≤ triSum i := triSum_mono i n (by omega)
      omega

end Polyrust
