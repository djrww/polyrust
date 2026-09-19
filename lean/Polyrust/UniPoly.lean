-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 一元多項式與 QAP 忠實性核心（定理 T8）

對應 docs/THEOREMS.md §10（T8）：
取互異求值點 t₁,…,t_m，Z(t) = ∏ᵢ (t − tᵢ)（vanishing 多項式），
A、B、C 為導線多項式（Lagrange 裝配）。忠實性核心是：

  **Z | A·B − C ⟺ z 滿足全部 R1CS 約束**（∀j: a_j·b_j = c_j）

本模組在 ℤ 上（求值層面）形式化此對偶的兩根支柱：
1. `div_linear`：monic 線性因式的餘式定理（p = q·(X−a) + p(a)）——
   證明是構造性的（Horner 除法），對應 Rust 版 `qap.rs`。
2. `vanishing_prod_dvd`：p 在 m 個互異點全部歸零 ⇒ Z = ∏(X−tᵢ) 整除 p
   （歸納 + ℤ 無零因子）。
3. `dvd_prod_vanishing`：Z 整除 ⇒ 各點歸零（求值同態）。
三者組合成 `qap_duality`：約束滿足 ⟺ Z | A·B − C。 -/

namespace Polyrust

/-- 升冪係數的一元整係數多項式（與 Rust 版 `qap.rs` 的 `UniPoly` 同構設計）。 -/
abbrev UniPoly := List Int

namespace UniPoly

/-- 在點 t 處求值（Horner 形式）。 -/
def eval : UniPoly → Int → Int
  | [], _ => 0
  | c :: cs, t => c + t * eval cs t

def add : UniPoly → UniPoly → UniPoly
  | [], q => q
  | p, [] => p
  | a :: as, b :: bs => (a + b) :: add as bs

def smulC (c : Int) (p : UniPoly) : UniPoly := p.map (c * ·)

def shift (p : UniPoly) : UniPoly := 0 :: p

def mul : UniPoly → UniPoly → UniPoly
  | [], _ => []
  | _, [] => []
  | c :: cs, q => add (smulC c q) (shift (mul cs q))

def negP (p : UniPoly) : UniPoly := p.map (fun c => -c)

def sub (p q : UniPoly) : UniPoly := add p (negP q)

/-- X − a：monic 線性因式。 -/
def XsubC (a : Int) : UniPoly := [-a, 1]

/-- Z(t) = ∏ᵢ (t − tᵢ)：互異求值點的 vanishing 多項式。 -/
def prodXs (ts : List Int) : UniPoly :=
  ts.foldr (fun t acc => mul (XsubC t) acc) [1]

/-! ## 求值是環同態 -/

theorem eval_add (p q : UniPoly) (t : Int) : (add p q).eval t = eval p t + eval q t := by
  induction p generalizing q with
  | nil => cases q <;> simp [add, eval]
  | cons c cs ih =>
    cases q with
    | nil => simp [add, eval]
    | cons d ds =>
      have e1 : (add (c :: cs) (d :: ds)).eval t = (c + d) + t * eval (add cs ds) t := rfl
      have e2 : eval (c :: cs) t = c + t * eval cs t := rfl
      have e3 : eval (d :: ds) t = d + t * eval ds t := rfl
      rw [e1, e2, e3, ih, Int.mul_add]
      omega

theorem eval_smulC (c : Int) (p : UniPoly) (t : Int) :
    (smulC c p).eval t = c * eval p t := by
  induction p with
  | nil => simp [smulC, eval]
  | cons d ds ih =>
    have e1 : (smulC c (d :: ds)).eval t = c * d + t * eval (smulC c ds) t := rfl
    have e2 : eval (d :: ds) t = d + t * eval ds t := rfl
    have key : t * (c * eval ds t) = c * (t * eval ds t) := by
      rw [← Int.mul_assoc, ← Int.mul_assoc, Int.mul_comm t c]
    rw [e1, ih, e2, Int.mul_add, key]

theorem eval_shift (p : UniPoly) (t : Int) : (shift p).eval t = t * eval p t := by
  have e : (shift p).eval t = 0 + t * eval p t := rfl
  rw [e, Int.zero_add]

theorem eval_negP (p : UniPoly) (t : Int) : (negP p).eval t = -(eval p t) := by
  induction p with
  | nil => simp [negP, eval]
  | cons d ds ih =>
    have e1 : (negP (d :: ds)).eval t = -d + t * eval (negP ds) t := rfl
    have e2 : eval (d :: ds) t = d + t * eval ds t := rfl
    rw [e1, ih, e2, Int.mul_neg]
    omega

theorem eval_sub (p q : UniPoly) (t : Int) : (sub p q).eval t = eval p t - eval q t := by
  rw [sub, eval_add, eval_negP]
  omega

theorem eval_mul (p q : UniPoly) (t : Int) : (mul p q).eval t = eval p t * eval q t := by
  induction p with
  | nil => simp [mul, eval]
  | cons c cs ih =>
    cases q with
    | nil => simp [mul, eval]
    | cons d ds =>
      have m : mul (c :: cs) (d :: ds)
          = add (smulC c (d :: ds)) (shift (mul cs (d :: ds))) := rfl
      rw [m, eval_add, eval_smulC, eval_shift, ih]
      have e2 : eval (d :: ds) t = d + t * eval ds t := rfl
      have e3 : eval (c :: cs) t = c + t * eval cs t := rfl
      rw [e2, e3, ← Int.mul_assoc, Int.add_mul]

theorem eval_XsubC (a t : Int) : (XsubC a).eval t = t - a := by
  have e : (XsubC a).eval t = -a + t * (1 + t * 0) := rfl
  rw [e, Int.mul_zero, Int.add_zero, Int.mul_one]
  omega

theorem eval_prodXs (ts : List Int) (t : Int) :
    (prodXs ts).eval t = (ts.map (fun ti => t - ti)).foldr (fun a b => a * b) 1 := by
  induction ts with
  | nil =>
    have e : (prodXs []).eval t = 1 + t * 0 := rfl
    rw [e, Int.mul_zero, Int.add_zero, List.map_nil, List.foldr_nil]
  | cons t0 rest ih =>
    have m : prodXs (t0 :: rest) = mul (XsubC t0) (prodXs rest) := rfl
    rw [m, eval_mul, eval_XsubC, ih, List.map_cons, List.foldr_cons]

/-! ## 支柱一：monic 線性餘式定理（構造性 Horner 除法） -/

/-- **線性餘式定理**：∀p a, ∃q, ∀t, p(t) = q(t)·(t − a) + p(a)。
構造：q 由 Horner 除法逐係數遞歸生成（見證存在性）。 -/
theorem div_linear (p : UniPoly) (a : Int) :
    ∃ q : UniPoly, ∀ t : Int, eval p t = eval q t * (t - a) + eval p a := by
  induction p with
  | nil =>
    refine ⟨[], ?_⟩
    intro t
    simp [eval]
  | cons c cs ih =>
    obtain ⟨qcs, hcs⟩ := ih
    refine ⟨add (shift qcs) [eval cs a], ?_⟩
    intro t
    -- q 的求值：eval (add (shift qcs) [D]) t = t * eval qcs t + D
    have eD : eval [eval cs a] t = eval cs a + t * 0 := rfl
    have h1 : eval (add (shift qcs) [eval cs a]) t
        = t * eval qcs t + eval cs a := by
      rw [eval_add, eval_shift, eD]
      have hz : t * 0 = 0 := Int.mul_zero t
      rw [hz, Int.add_zero]
    have h2 : eval (c :: cs) t = c + t * eval cs t := rfl
    have h3 : eval (c :: cs) a = c + a * eval cs a := rfl
    rw [h1, h2, h3]
    have hq := hcs t
    -- 核心環等式，逐步歸一後交給 omega：
    -- c + t*(S*(t−a) + D) = (t*S + D)*(t−a) + c + a*D，其中 S = eval qcs t，D = eval cs a
    have key1 : t * (eval qcs t * (t - a) + eval cs a)
        = t * (eval qcs t * (t - a)) + t * eval cs a := Int.mul_add _ _ _
    have key2 : t * (eval qcs t * (t - a)) = t * eval qcs t * (t - a) :=
      (Int.mul_assoc _ _ _).symm
    have key3 : (t * eval qcs t + eval cs a) * (t - a)
        = t * eval qcs t * (t - a) + eval cs a * (t - a) := Int.add_mul _ _ _
    have key4 : eval cs a * (t - a) = eval cs a * t - eval cs a * a := Int.mul_sub _ _ _
    have key5 : eval cs a * t = t * eval cs a := Int.mul_comm _ _
    have key6 : eval cs a * a = a * eval cs a := Int.mul_comm _ _
    rw [hq, key1, key2, key3, key4, key5, key6]
    omega

/-! ## 支柱二：互異根 ⇒ vanishing 多項式整除 -/

/-- p 在互異點集 ts 上全部歸零 ⇒ ∏(X − tᵢ) | p（求值層面）。
歸納論證：p(t₀) = 0 ⇒ p = q₁·(X−t₀)；對其餘根用 ℤ 無零因子剝離 (t−t₀)；遞歸。 -/
theorem vanishing_prod_dvd (ts : List Int) (hnd : ts.Nodup) (p : UniPoly)
    (h : ∀ t ∈ ts, eval p t = 0) :
    ∃ q : UniPoly, ∀ t : Int, eval p t = eval q t * eval (prodXs ts) t := by
  induction ts generalizing p with
  | nil =>
    refine ⟨p, ?_⟩
    intro t
    have e : (prodXs []).eval t = 1 + t * 0 := rfl
    rw [e, Int.mul_zero, Int.add_zero, Int.mul_one]
  | cons t0 rest ih =>
    obtain ⟨h0nr, hndr⟩ := List.nodup_cons.mp hnd
    have h0 : eval p t0 = 0 := h t0 List.mem_cons_self
    obtain ⟨q1, hq1⟩ := div_linear p t0
    have hq1' : ∀ t : Int, eval p t = eval q1 t * (t - t0) := by
      intro t
      rw [hq1 t, h0, Int.add_zero]
    have hrest : ∀ t ∈ rest, eval q1 t = 0 := by
      intro t ht
      have hval := hq1' t
      rw [h t (List.mem_cons_of_mem _ ht)] at hval
      rcases Int.mul_eq_zero.mp hval.symm with hz | hz
      · exact hz
      · exfalso
        have hte : t = t0 := by omega
        subst hte
        exact h0nr ht
    obtain ⟨q2, hq2⟩ := ih hndr q1 hrest
    refine ⟨q2, ?_⟩
    intro t
    have hprod : eval (prodXs (t0 :: rest)) t = (t - t0) * eval (prodXs rest) t := by
      have m : prodXs (t0 :: rest) = mul (XsubC t0) (prodXs rest) := rfl
      rw [m, eval_mul, eval_XsubC]
    rw [hq1' t, hq2 t, hprod, Int.mul_assoc, Int.mul_comm (eval (prodXs rest) t) (t - t0)]

/-! ## 支柱三：整除 ⇒ 各點歸零 -/

theorem prod_zero_of_mem (ts : List Int) (t : Int) (h : t ∈ ts) :
    (ts.map (fun ti => t - ti)).foldr (fun a b => a * b) 1 = 0 := by
  induction ts with
  | nil => cases h
  | cons t0 rest ih =>
    rcases List.mem_cons.mp h with heq | hmem
    · rw [List.map_cons, List.foldr_cons, heq, Int.sub_self, Int.zero_mul]
    · rw [List.map_cons, List.foldr_cons, ih hmem, Int.mul_zero]

theorem dvd_prod_vanishing (ts : List Int) (q p : UniPoly)
    (h : ∀ t : Int, eval p t = eval q t * eval (prodXs ts) t)
    (t : Int) (ht : t ∈ ts) : eval p t = 0 := by
  have hz : eval (prodXs ts) t = 0 := by
    rw [eval_prodXs, prod_zero_of_mem ts t ht]
  rw [h t, hz, Int.mul_zero]

/-! ## T8 主定理：QAP 對偶 -/

/-- **QAP 忠實性（T8(b) 核心）**：
設求值點互異，A、B、C 為導線多項式，α、β、γ 為各點的約束兩側值
（即 R1CS：⟨a_j, z⟩·⟨b_j, z⟩ = ⟨c_j, z⟩ 經 Lagrange 裝配後的點值）。
則「全部約束成立」⟺「Z = ∏(X−tⱼ) 整除 A·B − C」。

這正是 QAP 驗證方程 Z | a·b − c 的數學內容：
一次多項式整除檢驗（O(1) 配對運算）完全等價於 m 條約束逐一檢查。 -/
theorem qap_duality (ts : List Int) (hnd : ts.Nodup)
    (A B C : UniPoly) (α β γ : Int → Int)
    (hA : ∀ t ∈ ts, eval A t = α t) (hB : ∀ t ∈ ts, eval B t = β t)
    (hC : ∀ t ∈ ts, eval C t = γ t) :
    (∀ t ∈ ts, α t * β t = γ t) ↔
    ∃ Q : UniPoly, ∀ t : Int,
      eval (sub (mul A B) C) t = eval Q t * eval (prodXs ts) t := by
  constructor
  · intro hall
    refine vanishing_prod_dvd ts hnd (sub (mul A B) C) ?_
    intro t ht
    rw [eval_sub, eval_mul, hA t ht, hB t ht, hC t ht, hall t ht]
    omega
  · intro hQ
    obtain ⟨Q, hQ⟩ := hQ
    intro t ht
    have hz : eval (prodXs ts) t = 0 := by
      rw [eval_prodXs, prod_zero_of_mem ts t ht]
    have hval := hQ t
    rw [hz, Int.mul_zero] at hval
    rw [eval_sub, eval_mul, hA t ht, hB t ht, hC t ht] at hval
    omega

end UniPoly

end Polyrust
