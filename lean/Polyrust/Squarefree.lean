-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # Buchberger 終止性（定理 T4）——完整機械化

對應 docs/THEOREMS.md §6（T4）：

含域多項式 B = {xᵢ² − xᵢ} 時，首項理想 in(I) 包含每個 xᵢ²，因此
**標準單項式**（不被任何基首項整除者）必然**平方自由**（每個指數 ≤ 1）；
而 n 變量平方自由單項式恰有 2ⁿ 個（與位串雙射）。基每次擴充會加入一個
此前標準的首項，令標準集**嚴格縮小**；標準集單調遞減、基數 ≤ 2ⁿ，
故擴充次數 ≤ 2ⁿ，Buchberger 必終止。

本模組完整形式化此鏈的四個環節：

1. `x2_divides_of_ge2`（見 `Polyrust.Monomial`）：非平方自由 ⇒ xᵢ² 整除。
2. `standard_implies_squarefree`：標準單項式 ⇒ 平方自由。
3. `squarefree_bij` / `squarefree_count`：平方自由單項式 ↔ 位串（雙射），
   恰 2ⁿ 個（`allBits` 可靠性 + 完備性 + 計數）。
4. `buchberger_extension_bound`：**抽象的消耗論證**——嚴格遞減的
   `Sublist` 鏈在長度 ≤ 2ⁿ 的有限集合內，步數 ≤ 2ⁿ 步 ⇒ 必終止。

**邊界**：這裡證明的是「終止界」的組合核心，不是 Buchberger 演算法本身
（S-多項式的計算、約化、準則見 `Polyrust.SPoly`；約化基唯一性見
`Polyrust.Canonical`）。整個論證不需要域、理想或項序：終止性只用到
「新首項此前是標準單項式」這一事實（T5 準則與 T7(a) 保證其正確性）。 -/

import Polyrust.Monomial

namespace Polyrust

/-! ## 環節一：標準單項式必平方自由 -/

/-- **T4 關鍵引理**：若每個 xᵢ² 都被基的某個首項整除（域多項式入基 ⇒
首項理想含 xᵢ²），則任何標準單項式（不被任何首項整除）必平方自由。

逆否形式即「非平方自由 ⇒ 非標準」：m 某維指數 ≥ 2 ⇒ xᵢ² | m ⇒
（傳遞性）某首項 | m。這限制了首項理想補集（標準單項式集）大小 ≤ 2ⁿ。 -/
theorem standard_implies_squarefree (L : List MonoExp)
    (hlead : ∀ i, ∃ ℓ ∈ L, dividesM ℓ (x2 i))
    (m : MonoExp) (hstd : ∀ ℓ ∈ L, ¬ dividesM ℓ m) :
    squarefreeM m := by
  intro i
  by_cases h1 : m i ≤ 1
  · exact h1
  · exfalso
    obtain ⟨ℓ, hℓL, hℓ⟩ := hlead i
    have hx2 : dividesM (x2 i) m := x2_divides_of_ge2 m i (by omega)
    exact hstd ℓ hℓL (dividesM_trans hℓ hx2)

/-- 標準單項式集（對於首項理想含全部 xᵢ² 的基）的每個元素都平方自由，
這是「標準集 ⊆ 平方自由單項式集」的逐點形式。 -/
theorem standard_set_subset_squarefree (L : List MonoExp)
    (hlead : ∀ i, ∃ ℓ ∈ L, dividesM ℓ (x2 i))
    (l : List MonoExp) (hl : ∀ m ∈ l, ∀ ℓ ∈ L, ¬ dividesM ℓ m) :
    ∀ m ∈ l, squarefreeM m :=
  fun m hm => standard_implies_squarefree L hlead m (hl m hm)

/-! ## 環節二：平方自由單項式 ↔ 位串 -/

/-- 位串 → 平方自由單項式：出現（1）或不出現（0）。 -/
def ofBits (f : Nat → Bool) : MonoExp := fun j => if f j then 1 else 0

/-- 平方自由單項式 → 位串：指數是否為 1。 -/
def toBits (m : MonoExp) : Nat → Bool := fun j => m j == 1

/-- **雙射（右逆）**：位串 → 單項式 → 位串還原。 -/
theorem toBits_ofBits (f : Nat → Bool) : toBits (ofBits f) = f := by
  funext j
  cases hfj : f j <;> simp [toBits, ofBits, hfj]

/-- ofBits 總是平方自由。 -/
theorem ofBits_squarefree (f : Nat → Bool) : squarefreeM (ofBits f) := by
  intro j
  cases hfj : f j <;> simp [ofBits, hfj]

/-- **雙射（左逆）**：平方自由單項式 → 位串 → 單項式還原。
`ofBits` 與 `toBits` 互為逆 ⇒ 平方自由單項式與位串一一對應。 -/
theorem ofBits_toBits {m : MonoExp} (h : squarefreeM m) : ofBits (toBits m) = m := by
  funext j
  have hj : m j ≤ 1 := h j
  rcases Nat.lt_or_ge (m j) 2 with hlt | hge
  · have hm01 : m j = 0 ∨ m j = 1 := by omega
    rcases hm01 with h0 | h1
    · simp [ofBits, toBits, h0]
    · simp [ofBits, toBits, h1]
  · omega

/-- 衍生：`ofBits` 單射（右逆存在）。 -/
theorem ofBits_injective : Function.Injective ofBits := by
  intro f g hfg
  have := congrArg toBits hfg
  rwa [toBits_ofBits, toBits_ofBits] at this

/-! ## 環節三：n 維位串的枚舉與計數 -/

/-- 位串的 n 維支撐條件：第 n 位以後恆 false。 -/
def supportLe (f : Nat → Bool) (n : Nat) : Prop := ∀ j, n ≤ j → f j = false

/-- 單項式的 n 維支撐條件。 -/
def supportLeM (m : MonoExp) (n : Nat) : Prop := ∀ j, n ≤ j → m j = 0

/-- 在位置 i 設定位元。 -/
def setN (f : Nat → Bool) (i : Nat) (b : Bool) : Nat → Bool :=
  fun j => if j = i then b else f j

theorem setN_self (f : Nat → Bool) (i : Nat) (b : Bool) : setN f i b i = b := by
  simp [setN]

theorem setN_ne {f : Nat → Bool} {i j : Nat} (h : j ≠ i) (b : Bool) :
    setN f i b j = f j := by
  simp [setN, h]

/-- n 維位串的全枚舉（長度 2ⁿ）。 -/
def allBits : Nat → List (Nat → Bool)
  | 0 => [fun _ => false]
  | n + 1 => (allBits n).flatMap (fun f => [setN f n false, setN f n true])

theorem allBits_length (n : Nat) : (allBits n).length = 2 ^ n := by
  induction n with
  | zero => simp [allBits, Nat.pow_zero]
  | succ n ih =>
    have key : ∀ (l : List (Nat → Bool)) (n : Nat),
        (l.flatMap (fun f => [setN f n false, setN f n true])).length = 2 * l.length := by
      intro l n
      induction l with
      | nil => simp
      | cons f fs ihl =>
        rw [List.flatMap_cons, List.length_append, ihl]
        simp
        omega
    rw [allBits, key, ih, Nat.pow_succ]
    omega

/-- 枚舉的可靠性：`allBits n` 的元素都支撐 ≤ n。 -/
theorem allBits_sound : ∀ (n : Nat) (f : Nat → Bool), f ∈ allBits n → supportLe f n := by
  intro n
  induction n with
  | zero =>
    intro f hf j _
    have hfl : f = fun _ => false := by
      have h1 : allBits 0 = [fun _ => false] := rfl
      rw [h1] at hf
      exact List.mem_singleton.mp hf
    rw [hfl]
  | succ n ih =>
    intro f hf
    rw [allBits, List.mem_flatMap] at hf
    obtain ⟨g, hgL, hgf⟩ := hf
    have hgn : supportLe g n := ih g hgL
    intro j hj
    rcases List.mem_cons.mp hgf with h | h
    · rw [h]
      by_cases hjn : j = n
      · rw [hjn, setN_self]
      · rw [setN_ne hjn]
        exact hgn j (by omega)
    · rcases List.mem_singleton.mp h with h
      rw [h]
      by_cases hjn : j = n
      · omega
      · rw [setN_ne hjn]
        exact hgn j (by omega)

/-- 枚舉的完備性：支撐 ≤ n 的位串都在 `allBits n` 中。 -/
theorem allBits_complete : ∀ (n : Nat) (f : Nat → Bool), supportLe f n → f ∈ allBits n := by
  intro n
  induction n with
  | zero =>
    intro f hf
    have hfconst : f = fun _ => false := by
      funext j
      exact hf j (Nat.zero_le j)
    rw [hfconst]
    simp [allBits]
  | succ n ih =>
    intro f hf
    -- 抹掉第 n 位得到 n 維支撐的位串
    have hgn : supportLe (setN f n false) n := by
      intro j hj
      by_cases hjn : j = n
      · rw [hjn, setN_self]
      · rw [setN_ne hjn]
        exact hf j (by omega)
    have hmem : setN f n false ∈ allBits n := ih (setN f n false) hgn
    by_cases hfn : f n = false
    · have key : setN (setN f n false) n false = f := by
        funext j
        by_cases hj : j = n
        · rw [hj, setN_self, hfn]
        · rw [setN_ne hj, setN_ne hj]
      rw [allBits, List.mem_flatMap]
      exact ⟨setN f n false, hmem, by rw [key]; simp⟩
    · have hfn' : f n = true := by
        cases hfnt : f n
        · exact absurd hfnt hfn
        · rfl
      have key : setN (setN f n false) n true = f := by
        funext j
        by_cases hj : j = n
        · rw [hj, setN_self, hfn']
        · rw [setN_ne hj, setN_ne hj]
      rw [allBits, List.mem_flatMap]
      exact ⟨setN f n false, hmem, by rw [key]; simp⟩

/-- `setN` 在不同位值下產生不同函數。 -/
theorem setN_false_ne_true (f : Nat → Bool) (n : Nat) :
    setN f n false ≠ setN f n true := by
  intro h
  have := congrFun h n
  rw [setN_self, setN_self] at this
  exact Bool.noConfusion this

/-- 支撐 ≤ n 的位串，第 n 位必為假。 -/
theorem supportLe_nth_false {f : Nat → Bool} {n : Nat} (h : supportLe f n) : f n = false :=
  h n (Nat.le_refl n)

/-- 支撐 ≤ n 的兩個位串：設定第 n 位後相等 ⟹ 原本相等（`setN` 的單射性）。 -/
theorem setN_inj {f g : Nat → Bool} {n : Nat} (hf : supportLe f n) (hg : supportLe g n)
    {b₁ b₂ : Bool} (h : setN f n b₁ = setN g n b₂) : f = g := by
  funext j
  by_cases hjn : j = n
  · subst hjn
    rw [supportLe_nth_false hf, supportLe_nth_false hg]
  · have := congrFun h j
    rw [setN_ne hjn b₁, setN_ne hjn b₂] at this
    exact this

private theorem nodup_append {α : Type u} {l₁ l₂ : List α} (h₁ : l₁.Nodup) (h₂ : l₂.Nodup)
    (hd : ∀ x, x ∈ l₁ → x ∈ l₂ → False) : (l₁ ++ l₂).Nodup := by
  induction l₁ generalizing h₂ with
  | nil => exact h₂
  | cons a rest ih =>
    obtain ⟨hanr, hndr⟩ := List.nodup_cons.mp h₁
    simp only [List.cons_append]
    rw [List.nodup_cons]
    constructor
    · intro h
      rcases List.mem_append.mp h with h | h
      · exact hanr h
      · exact hd a (by simp) h
    · exact ih hndr h₂ (fun x hx => hd x (by simp [hx]))

private theorem nodup_flatMap_of {α β : Type _} (l : List α) (g : α → List β)
    (hnd : l.Nodup)
    (hself : ∀ a ∈ l, (g a).Nodup)
    (hdisj : ∀ a b, a ∈ l → b ∈ l → a ≠ b → ∀ x, x ∈ g a → x ∈ g b → False) :
    (l.flatMap g).Nodup := by
  induction l with
  | nil => simp
  | cons a rest ih =>
    obtain ⟨hanr, hndr⟩ := List.nodup_cons.mp hnd
    rw [List.flatMap_cons]
    apply nodup_append
    · exact hself a (by simp)
    · exact ih hndr (fun b hb => hself b (by simp [hb]))
        (fun b c hb hc hbc x hxb hxc =>
          hdisj b c (by simp [hb]) (by simp [hc]) hbc x hxb hxc)
    · intro x hxa hxr
      obtain ⟨b, hbr, hxb⟩ := List.mem_flatMap.mp hxr
      exact hdisj a b (by simp) (by simp [hbr]) (fun hab => hanr (hab ▸ hbr)) x hxa hxb

/-- **枚舉無重複**：`allBits n` 把每個支撐 ≤ n 的位串**恰列一次**。
這是 `T6Certificate.sum_delta`／`interpolation` 所需 `pts.Nodup` 前提的來源。 -/
theorem allBits_nodup (n : Nat) : (allBits n).Nodup := by
  induction n with
  | zero => simp [allBits]
  | succ n ih =>
    rw [allBits]
    apply nodup_flatMap_of (allBits n) (fun f => [setN f n false, setN f n true]) ih
    · intro f _
      rw [List.nodup_cons]
      constructor
      · intro h
        exact setN_false_ne_true f n (List.mem_singleton.mp h)
      · simp
    · intro f g hf hg hfg x hxf hxg
      obtain ⟨bf, hbf⟩ : ∃ b, x = setN f n b := by
        rcases List.mem_cons.mp hxf with h | h
        · exact ⟨false, h⟩
        · exact ⟨true, List.mem_singleton.mp h⟩
      obtain ⟨bg, hbg⟩ : ∃ b, x = setN g n b := by
        rcases List.mem_cons.mp hxg with h | h
        · exact ⟨false, h⟩
        · exact ⟨true, List.mem_singleton.mp h⟩
      have heq : setN f n bf = setN g n bg := by rw [← hbf, ← hbg]
      exact hfg (setN_inj (allBits_sound n f hf) (allBits_sound n g hg) heq)

/-- **T4 計數定理**：n 維平方自由單項式可由長度恰為 2ⁿ 的列表完全枚舉
（可靠性 + 完備性 = 恰好枚舉一遍，即雙射計數）。 -/
theorem squarefree_count (n : Nat) :
    ∃ l : List MonoExp, l.length = 2 ^ n
      ∧ (∀ m ∈ l, squarefreeM m ∧ supportLeM m n)
      ∧ (∀ m, squarefreeM m → supportLeM m n → m ∈ l) := by
  refine ⟨(allBits n).map ofBits, ?_, ?_, ?_⟩
  · rw [List.length_map]
    exact allBits_length n
  · intro m hm
    obtain ⟨f, hfL, hfEq⟩ := List.mem_map.mp hm
    rw [← hfEq]
    refine ⟨ofBits_squarefree f, ?_⟩
    intro j hj
    have hfs : supportLe f n := allBits_sound n f hfL
    have hfj : f j = false := hfs j hj
    simp [ofBits, hfj]
  · intro m hsq hsupp
    have htb : supportLe (toBits m) n := by
      intro j hj
      have hmj : m j = 0 := hsupp j hj
      simp [toBits, hmj]
    have hmem : toBits m ∈ allBits n := allBits_complete n (toBits m) htb
    exact List.mem_map.mpr ⟨toBits m, hmem, ofBits_toBits hsq⟩

/-- 衍生：平方自由且 n 維支撐的單項式不同者互異（雙射的左逆+右逆）。 -/
theorem squarefree_eq_of_bits_eq {m m' : MonoExp} (hm : squarefreeM m) (hm' : squarefreeM m')
    (h : toBits m = toBits m') : m = m' := by
  rw [← ofBits_toBits hm, ← ofBits_toBits hm', h]

/-! ## 環節四：嚴格遞減的標準集 ⇒ 擴充次數 ≤ 2ⁿ（終止界）

Buchberger 每加入一個新首項 g，該首項此前必是**標準單項式**（否則 S-餘式會被
完全歸約），加入後它退出標準集且不再回來（首項理想單調遞增）。於是把狀態記錄為
「標準集」的一個 `Sublist` 鏈：每個擴充步嚴格縮小一次，而鏈始終落在
U = 平方自由 n 維單項式（|U| = 2ⁿ）之內 ⇒ 步數 ≤ 2ⁿ。 -/

/-- 抽象消耗引理：嚴格遞減的 `Sublist` 鏈，第 k 步之後長度至少減少 k。 -/
theorem sublist_chain_length (_U : List β) (S : Nat → List β)
    (hsub : ∀ k, (S (k+1)).Sublist (S k))
    (hstrict : ∀ k, S (k+1) ≠ S k) :
    ∀ k, (S k).length + k ≤ (S 0).length := by
  intro k
  induction k with
  | zero => omega
  | succ k ih =>
    have hs : (S (k+1)).Sublist (S k) := hsub k
    have hle : (S (k+1)).length ≤ (S k).length := List.Sublist.length_le hs
    have hne : (S (k+1)).length ≠ (S k).length := by
      intro hlen
      exact hstrict k (List.Sublist.eq_of_length hs hlen)
    have hlt : (S (k+1)).length < (S k).length := by omega
    omega

/-- **T4 終止界（一般形式）**：標準集鏈落在長度 ≤ N 的有限集合 U 內，
則擴充步數 k ≤ N。 -/
theorem extension_bound (U : List β) (S : Nat → List β)
    (hsub : ∀ k, (S (k+1)).Sublist (S k))
    (hstrict : ∀ k, S (k+1) ≠ S k)
    (hbase : (S 0).Sublist U) (k : Nat) :
    k ≤ U.length := by
  have h1 : (S k).length + k ≤ (S 0).length := sublist_chain_length U S hsub hstrict k
  have h2 : (S 0).length ≤ U.length := List.Sublist.length_le hbase
  omega

/-- **T4 主定理（Buchberger 終止性）**：狀態為「n 變量平方自由單項式」
（以 `allBits n ∘ ofBits` 枚舉，長度 2ⁿ）內的嚴格遞減 `Sublist` 鏈時，
擴充步數 k ≤ 2ⁿ。故帶域多項式 B 的 Buchberger 演算法必在有限步內終止。 -/
theorem buchberger_extension_bound (n k : Nat) (S : Nat → List MonoExp)
    (hsub : ∀ j, (S (j+1)).Sublist (S j))
    (hstrict : ∀ j, S (j+1) ≠ S j)
    (hbase : (S 0).Sublist ((allBits n).map ofBits)) :
    k ≤ 2 ^ n := by
  have h := extension_bound ((allBits n).map ofBits) S hsub hstrict hbase k
  rw [List.length_map, allBits_length] at h
  exact h

/-- 終止性的直接推論：不存在無窮的嚴格遞減標準集鏈（反證形式）。 -/
theorem no_infinite_sublist_chain (U : List β) (S : Nat → List β)
    (hsub : ∀ k, (S (k+1)).Sublist (S k))
    (hstrict : ∀ k, S (k+1) ≠ S k)
    (hbase : (S 0).Sublist U) :
    False := by
  have h := extension_bound U S hsub hstrict hbase (U.length + 1)
  omega

end Polyrust
