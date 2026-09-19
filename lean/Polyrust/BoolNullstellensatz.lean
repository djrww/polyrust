-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 布爾 Nullstellensatz（構造性、𝔽_p 函數層）（v0.1.4 新增）

**本模組證什麼（說死）**：

給定系統 `fs`（布爾立方 {0,1}ⁿ 上的 ℤ 值**多項式函數**層）與模數 `p`：

* `bool_nullstellensatz`（完備方向，主定理）：
  若每個立方點 `x` 都有某個 `f ∈ fs` 使 `f(x)` **模 p 可逆**（顯式逆元見證
  `iv`：`p ∣ f(x)·iv − 1`），則存在**多項式函數乘子** `mult f` 使得

      Σ_f mult_f · f ≡ 1  (mod p)     （在整個 {0,1}ⁿ 上逐點成立）

  構造是顯式的：每點選見證元素 `pf(x)` 與逆元 `piv(x)`，乘子為拉格朗日基
  `δ_a` 的線性組合（`piv(a)·δ_a` 按見證歸屬分配到各 `f`），再經布爾插值
  （`interpolation`）提升為多項式函數。這就是「無公共零點（模 p）⟹
  存在乘子把系統組合成常數 1（模 p）」——布爾情形下不需要任何
  維度/次數論證，插值基直接給出組合。
* `bool_ns_no_root_of_certificate`（可靠方向）：
  若存在乘子使 `Σ_f mult_f·f ≡ 1 (mod p)`，則系統在每點都有某個元素
  **模 p 非零**（前提 `p ∤ 1`，排除退化模數 `p = ±1`）。
  與上條合成：函數層上「可逆見證 ⟺ 常數 1 憑證」。
* `mod_p_one_in_ideal`：兌現 `T6Certificate` 檔首承諾的第 4 條——
  已證出的「處處非零」函數 `D = Σ_f mult_f·f`，給定 `D` 模 `p` 的逆元
  見證，可把乘子**縮放**成常數 1 憑證（仍是多項式函數乘子）。

**邊界（說死）**：

1. **層**：結論在 {0,1}ⁿ 上的**函數**層（模 p 逐點同餘），不是語法層
   的 `Σ gᵢfᵢ + Σ hⱼ(xⱼ²−xⱼ) = 1` 多項式恆等式。語法版需要多元除法
   （對每個 `xⱼ²−xⱼ` 逐步取餘式）——本倉庫未做，且管線不需要它：
   判定路徑是「Rust 側 Gröbner 計算 ⟺ 檢查器等價（T9）」，本模組提供
   的是**語義憑證層**（任意大小的系統、任意 `n` 的一般性陳述）。
2. **「可逆」與「非零」的差距**：完備方向的前提是**可逆**（帶逆元見證），
   不是「模 p 非零」。兩者在 `p` 為素數時等價（Bézout）；`p = 2⁶¹−1`
   的素性在 `Polyrust.Embedding`（L0）層處理，本定理不假設素性——
   構造對見證成立，見證由素性（或逐點計算）提供。
3. 可靠方向需要 `p ∤ 1`（否則任何同餘式都成立）。
4. 點集固定為 `allBits n`（`allBits_nodup`／`allBits_sound`／
   `allBits_complete` 提供 `sum_delta` 所需的全部前提）。 -/

import Polyrust.T6Certificate

namespace Polyrust

open Classical

/-! ## 一、模同餘（零依賴定義 `p ∣ a − b` 與極小 API） -/

/-- 模 `p` 同餘（整數層，零依賴：`p ∣ a − b`）。 -/
abbrev ModP (p : Int) (a b : Int) : Prop := p ∣ a - b

theorem modp_congr {p a b c d : Int} (hac : a = c) (hbd : b = d) (h : ModP p a b) :
    ModP p c d := by
  subst hac; subst hbd; exact h

theorem modp_add {p a b c d : Int} (h₁ : ModP p a b) (h₂ : ModP p c d) :
    ModP p (a + c) (b + d) := by
  obtain ⟨k, hk⟩ := h₁
  obtain ⟨l, hl⟩ := h₂
  refine ⟨k + l, ?_⟩
  calc (a + c) - (b + d) = (a - b) + (c - d) := by omega
    _ = p * k + p * l := by rw [hk, hl]
    _ = p * (k + l) := by rw [Int.mul_add]

theorem modp_mul_left {p a b : Int} (h : ModP p a b) (c : Int) :
    ModP p (c * a) (c * b) := by
  obtain ⟨k, hk⟩ := h
  refine ⟨c * k, ?_⟩
  calc c * a - c * b = c * (a - b) := by rw [Int.mul_sub]
    _ = c * (p * k) := by rw [hk]
    _ = p * (c * k) := by rw [← Int.mul_assoc, Int.mul_comm c p, Int.mul_assoc]

/-- 兩個「≡ 0」同餘式相加仍 ≡ 0。 -/
theorem modp_add0 {p a c : Int} (h₁ : ModP p a 0) (h₂ : ModP p c 0) : ModP p (a + c) 0 := by
  obtain ⟨k, hk⟩ := h₁
  obtain ⟨l, hl⟩ := h₂
  refine ⟨k + l, ?_⟩
  calc (a + c) - 0 = (a - 0) + (c - 0) := by omega
    _ = p * k + p * l := by rw [hk, hl]
    _ = p * (k + l) := by rw [Int.mul_add]

/-- 兩個同餘式消去左端：`s ≡ 1` 且 `s ≡ 0` ⟹ `0 ≡ 1`。 -/
theorem modp_cancel {p s u v : Int} (h₁ : ModP p s u) (h₂ : ModP p s v) : ModP p v u := by
  obtain ⟨k, hk⟩ := h₁
  obtain ⟨l, hl⟩ := h₂
  refine ⟨k - l, ?_⟩
  calc v - u = (s - u) - (s - v) := by omega
    _ = p * k - p * l := by rw [hk, hl]
    _ = p * (k - l) := by rw [Int.mul_sub]

/-! ## 二、求和引理（標量提取、纖維重排、單位分拆的組合） -/

/-- 標量提出求和：`Σ_f c·t(f) = c·Σ_f t(f)`。 -/
theorem sum_mul_left (l : List F) (c : Int) (t : F → Int) :
    (l.map (fun f => c * t f)).sum = c * (l.map t).sum := by
  induction l with
  | nil => simp
  | cons f rest ih =>
    rw [List.map_cons, List.sum_cons, List.map_cons, List.sum_cons, ih, ← Int.mul_add]

/-- 逐項相加的求和分配。 -/
theorem sum_map_add (l : List F) (u v : F → Int) :
    (l.map (fun f => u f + v f)).sum = (l.map u).sum + (l.map v).sum := by
  induction l with
  | nil => simp
  | cons f rest ih =>
    rw [List.map_cons, List.map_cons, List.map_cons, List.sum_cons, List.sum_cons,
        List.sum_cons, ih]
    omega

/-- 求和同餘（任意元素型別版）。 -/
theorem sum_map_congr_gen {α : Type u_1} {l : List α} {u v : α → Int}
    (h : ∀ a ∈ l, u a = v a) : (l.map u).sum = (l.map v).sum := by
  induction l with
  | nil => simp
  | cons a rest ih =>
    rw [List.map_cons, List.map_cons, List.sum_cons, List.sum_cons, h a (by simp)]
    rw [ih (fun b hb => h b (by simp [hb]))]

/-- 三因子輪換：`a·b·c = a·c·b`。 -/
theorem mul_rotate (a b c : Int) : a * b * c = a * c * b := by
  rw [Int.mul_assoc, Int.mul_comm b c, ← Int.mul_assoc]

/-- 在無重複列表中按「等於 `f₀`」挑選：唯一留下的項是 `c · f₀(x)`。 -/
theorem sum_pick (fs : List F) (hfsnd : fs.Nodup) (f₀ : F) (hf₀ : f₀ ∈ fs) (c : Int)
    (x : Nat → Bool) :
    (fs.map (fun f => (if f₀ = f then c else 0) * f x)).sum = c * f₀ x := by
  induction fs with
  | nil => exact absurd hf₀ (by simp)
  | cons f rest ih =>
    rw [List.map_cons, List.sum_cons]
    obtain ⟨hfrest, hndrest⟩ := List.nodup_cons.mp hfsnd
    by_cases heq : f₀ = f
    · have hz : ∀ t ∈ rest.map (fun g => (if f₀ = g then c else 0) * g x), t = 0 := by
        intro t ht
        obtain ⟨g, hg, heq2⟩ := List.mem_map.mp ht
        rw [← heq2]
        have hne : f₀ ≠ g := fun hfg => hfrest (heq.symm.trans hfg ▸ hg)
        simp [hne]
      have hsum0 : (rest.map (fun g => (if f₀ = g then c else 0) * g x)).sum = 0 :=
        sum_zero_of_all_zero _ hz
      rw [hsum0, Int.add_zero, heq]
      simp
    · have h0 : (if f₀ = f then c else 0) * f x = 0 := by simp [heq]
      rw [h0, Int.zero_add]
      apply ih hndrest
      rcases List.mem_cons.mp hf₀ with h | h
      · exact absurd h heq
      · exact h

/-- **纖維重排**：每個點 `a ∈ pts` 歸屬於某個 `pf a ∈ fs`；
「先按元素 `f` 求和、再對歸屬於 `f` 的點求和」等於「直接按點求和」。
這是把逐點構造的乘子拆回「每個系統元素一個乘子」的關鍵。 -/
theorem sum_fiber (fs : List F) (hfsnd : fs.Nodup) (pts : List (Nat → Bool))
    (pf : (Nat → Bool) → F) (hpf : ∀ a ∈ pts, pf a ∈ fs)
    (piv : (Nat → Bool) → Int) (n : Nat) (x : Nat → Bool) :
    (fs.map (fun f =>
        (pts.map (fun a => if pf a = f then piv a * delta n a x else 0)).sum * f x)).sum
    = (pts.map (fun a => piv a * delta n a x * pf a x)).sum := by
  induction pts with
  | nil =>
    have hz : ∀ t ∈ fs.map (fun f =>
        (List.map (fun a => if pf a = f then piv a * delta n a x else 0) []).sum * f x),
        t = 0 := by
      intro t ht
      obtain ⟨f, _, heq⟩ := List.mem_map.mp ht
      rw [← heq]
      simp
    rw [sum_zero_of_all_zero _ hz]
    simp
  | cons a rest ih =>
    rw [List.map_cons, List.sum_cons]
    have hin : ∀ f,
        ((a :: rest).map (fun b => if pf b = f then piv b * delta n b x else 0)).sum
        = (if pf a = f then piv a * delta n a x else 0)
          + (rest.map (fun b => if pf b = f then piv b * delta n b x else 0)).sum := by
      intro f
      rw [List.map_cons, List.sum_cons]
    rw [sum_map_congr (l := fs)
      (u := fun f => ((a :: rest).map (fun b => if pf b = f then piv b * delta n b x else 0)).sum * f x)
      (v := fun f => (if pf a = f then piv a * delta n a x else 0) * f x
          + (rest.map (fun b => if pf b = f then piv b * delta n b x else 0)).sum * f x)
      (fun f _ => by rw [hin f, Int.add_mul])]
    rw [sum_map_add]
    rw [sum_pick fs hfsnd (pf a) (hpf a (by simp)) (piv a * delta n a x) x]
    rw [ih (fun b hb => hpf b (by simp [hb]))]

/-! ## 三、布爾 Nullstellensatz（完備方向：顯式乘子 ⟹ 組合成 1） -/

/-- 逐點選出的見證元素（無公共零點 ⟹ 每點一個 `f ∈ fs` 可逆）。 -/
noncomputable def bnzPf (fs : List F) (p : Int) (x : Nat → Bool) : F :=
  if h : ∃ f ∈ fs, ∃ iv : Int, ModP p (f x * iv) 1 then h.choose else fs.headD (fun _ => 0)

/-- 對應的模 `p` 逆元。 -/
noncomputable def bnzPiv (fs : List F) (p : Int) (x : Nat → Bool) : Int :=
  if h : ∃ f ∈ fs, ∃ iv : Int, ModP p (f x * iv) 1 then h.choose_spec.2.choose else 0

theorem bnzPf_spec (fs : List F) (p : Int) (x : Nat → Bool)
    (h : ∃ f ∈ fs, ∃ iv : Int, ModP p (f x * iv) 1) :
    bnzPf fs p x ∈ fs ∧ ModP p (bnzPf fs p x x * bnzPiv fs p x) 1 := by
  have h1 : bnzPf fs p x = h.choose := dif_pos h
  have h2 : bnzPiv fs p x = h.choose_spec.2.choose := dif_pos h
  rw [h1, h2]
  exact ⟨h.choose_spec.1, h.choose_spec.2.choose_spec⟩

/-- 原始乘子（插值前）：把各點的 `piv(a)·δ_a` 按見證歸屬分配給元素 `f`。 -/
noncomputable def bnzMultRaw (n : Nat) (pts : List (Nat → Bool)) (fs : List F)
    (p : Int) (f : F) : F :=
  fun x => (pts.map (fun a =>
    if bnzPf fs p a = f then bnzPiv fs p a * delta n a x else 0)).sum

/-- **布爾 Nullstellensatz（構造性完備方向）**：
每個立方點都有系統元素模 `p` 可逆（顯式逆元見證）⟹ 存在
**多項式函數乘子**把系統組合成常數 `1`（模 `p`，逐點於 {0,1}ⁿ）。

這就是「無公共零點 ⟹ 1 在系統生成的理想中」在布爾立方函數環上的
構造性版本——乘子被顯式寫出（拉格朗日基線性組合 + 布爾插值）。 -/
theorem bool_nullstellensatz (n : Nat) (p : Int) (fs : List F) (hfsnd : fs.Nodup)
    (hwit : ∀ x, supportLe x n → ∃ f ∈ fs, ∃ iv : Int, ModP p (f x * iv) 1) :
    ∃ mult : F → F,
      (∀ f ∈ fs, PolyFn n (mult f)) ∧
      ∀ x, supportLe x n → ModP p ((fs.map (fun f => mult f x * f x)).sum) 1 := by
  let pts := allBits n
  let raw : F → F := bnzMultRaw n pts fs p
  let mult : F → F := fun f => interp n pts (raw f)
  refine ⟨mult, fun f _ => interp_polyFn pts (raw f), ?_⟩
  intro x hx
  have hpts_nd : pts.Nodup := allBits_nodup n
  have hpts_sound : ∀ σ ∈ pts, supportLe σ n := allBits_sound n
  have hpts_cover : ∀ y, supportLe y n → y ∈ pts := allBits_complete n
  -- 第一步：插值在立方上與原始乘子一致
  have hinterp : ∀ f ∈ fs, mult f x = raw f x := by
    intro f _
    exact (interpolation pts hpts_nd hpts_sound hpts_cover (raw f)).2 x hx
  -- 第二步：纖維重排 ⇒ 逐點的 δ 基線性組合
  have hfiber :
      (fs.map (fun f => raw f x * f x)).sum
      = (pts.map (fun a => bnzPiv fs p a * delta n a x * bnzPf fs p a x)).sum := by
    exact sum_fiber fs hfsnd pts (bnzPf fs p) (fun a ha => (bnzPf_spec fs p a (hwit a (hpts_sound a ha))).1)
      (bnzPiv fs p) n x
  -- 第三步：輪換因子，套單位分拆（sum_delta）
  have hrotate :
      (pts.map (fun a => bnzPiv fs p a * delta n a x * bnzPf fs p a x)).sum
      = (pts.map (fun a => (bnzPiv fs p a * bnzPf fs p a x) * delta n a x)).sum :=
    sum_map_congr_gen
      (u := fun a => bnzPiv fs p a * delta n a x * bnzPf fs p a x)
      (v := fun a => (bnzPiv fs p a * bnzPf fs p a x) * delta n a x)
      (fun a _ => mul_rotate _ _ _)
  have hdelta :
      (pts.map (fun a => (bnzPiv fs p a * bnzPf fs p a x) * delta n a x)).sum
      = bnzPiv fs p x * bnzPf fs p x x :=
    sum_delta pts hpts_nd hpts_sound (fun a => bnzPiv fs p a * bnzPf fs p a x) x
      (hpts_cover x hx) hx
  -- 第四步：見證給出 ≡ 1（mod p），見結尾 `modp_congr`。
  have hcong : (fs.map (fun f => mult f x * f x)).sum
      = bnzPiv fs p x * bnzPf fs p x x := by
    rw [sum_map_congr (l := fs) (fun f hf => by rw [hinterp f hf])]
    rw [hfiber, hrotate, hdelta]
  rw [hcong]
  exact modp_congr (Int.mul_comm _ _) rfl (bnzPf_spec fs p x (hwit x hx)).2

/-! ## 四、可靠方向：常數 1 憑證 ⟹ 無公共零點 -/

/-- 系統元素在 `x` 處逐個 ≡ 0（模 `p`）⟹ 任何線性組合的和也 ≡ 0。 -/
theorem modp_sum_zero (fs : List F) (p : Int) (mult : F → F) (x : Nat → Bool)
    (h : ∀ f ∈ fs, ModP p (f x) 0) :
    ModP p ((fs.map (fun f => mult f x * f x)).sum) 0 := by
  induction fs with
  | nil =>
    change ModP p 0 0
    exact ⟨0, by omega⟩
  | cons f rest ih =>
    rw [List.map_cons, List.sum_cons]
    apply modp_add0
    · exact modp_congr rfl (Int.mul_zero (mult f x)) (modp_mul_left (h f (by simp)) (mult f x))
    · exact ih (fun g hg => h g (by simp [hg]))

/-- **憑證的可靠性**：存在乘子把系統組合成常數 `1`（模 `p`）⟹
每個立方點都有某個系統元素模 `p` 非零。
（前提 `p ∤ 1`：排除退化模數 `p = ±1`——此時任何同餘式都平凡成立。） -/
theorem bool_ns_no_root_of_certificate (n : Nat) (p : Int) (fs : List F)
    (hp : ¬ ModP p 0 1) (mult : F → F)
    (hcert : ∀ x, supportLe x n → ModP p ((fs.map (fun f => mult f x * f x)).sum) 1) :
    ∀ x, supportLe x n → ∃ f ∈ fs, ¬ ModP p (f x) 0 := by
  intro x hx
  apply Classical.byContradiction
  intro hall
  have hall' : ∀ f ∈ fs, ModP p (f x) 0 := by
    intro f hf
    by_cases h : ModP p (f x) 0
    · exact h
    · exact absurd ⟨f, hf, h⟩ hall
  -- 每項 ≡ 0 ⟹ 總和 ≡ 0
  exact hp (modp_cancel (hcert x hx) (modp_sum_zero fs p mult x hall'))

/-! ## 五、逆元縮放：處處非零的憑證 ⟹ 常數 1 憑證（兌現檔首承諾） -/

/-- **模 `p` 縮放為 1**（`T6Certificate` 檔首第 4 條承諾的實現）：
已知 `D = Σ_f mult_f·f` 在立方上處處非零，且每點的 `D(x)` 有模 `p`
逆元見證 ⟹ 存在（仍是多項式函數的）乘子把系統組合成常數 `1`（模 `p`）。
構造：逐點乘上 `D(x)` 的逆元 `iv(x)`，再經布爾插值恢復多項式性。 -/
theorem mod_p_one_in_ideal (n : Nat) (p : Int) (fs : List F) (D : F) (mult : F → F)
    (hcomb : ∀ x, supportLe x n → (fs.map (fun f => mult f x * f x)).sum = D x)
    (hinv : ∀ x, supportLe x n → ∃ iv : Int, ModP p (D x * iv) 1) :
    ∃ mult1 : F → F,
      (∀ f ∈ fs, PolyFn n (mult1 f)) ∧
      ∀ x, supportLe x n → ModP p ((fs.map (fun f => mult1 f x * f x)).sum) 1 := by
  let iv : (Nat → Bool) → Int :=
    fun x => if h : ∃ i : Int, ModP p (D x * i) 1 then h.choose else 0
  have hiv : ∀ x, supportLe x n → ModP p (D x * iv x) 1 := by
    intro x hx
    dsimp [iv]
    rw [dif_pos (hinv x hx)]
    exact (hinv x hx).choose_spec
  let pts := allBits n
  let raw : F → F := fun f x => iv x * mult f x
  refine ⟨fun f => interp n pts (raw f), fun f _ => interp_polyFn pts (raw f), ?_⟩
  intro x hx
  have hinterp : ∀ f ∈ fs, interp n pts (raw f) x = raw f x := by
    intro f _
    exact (interpolation pts (allBits_nodup n) (allBits_sound n) (allBits_complete n)
      (raw f)).2 x hx
  have h1 : (fs.map (fun f => interp n pts (raw f) x * f x)).sum
      = iv x * (fs.map (fun f => mult f x * f x)).sum := by
    rw [sum_map_congr (l := fs) (fun f hf => by rw [hinterp f hf])]
    dsimp only [raw]
    rw [sum_map_congr (l := fs) (fun f _ => Int.mul_assoc (iv x) (mult f x) (f x))]
    exact sum_mul_left fs (iv x) (fun f => mult f x * f x)
  rw [h1, hcomb x hx]
  exact modp_congr (Int.mul_comm _ _) rfl (hiv x hx)

end Polyrust
