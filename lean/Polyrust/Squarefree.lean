/- # Buchberger 終止性（定理 T4）的核心組合鏈

對應 docs/THEOREMS.md §6（T4）：
含域多項式 B = {xᵢ² − xᵢ} 時，首項理想包含每個 xᵢ²，因此
**標準單項式**（不被任何基首項整除者）必然**平方自由**（每個指數 ≤ 1），
而 n 變量的平方自由單項式恰有 2ⁿ 個。基每次擴充消耗一個標準單項式
（新首項此前必是標準單項式），故擴充次數 ≤ 2ⁿ，Buchberger 必終止。

本模組完整形式化此鏈的三個環節：
1. `standard_implies_squarefree`：標準 ⇒ 平方自由（逆否：非平方自由的
   單項式被 xᵢ² | m ⪯ 某基首項整除，故非標準）。
2. `squarefree_bij`：平方自由單項式 ↔ 位串（ofBits/toBits 互逆雙射）。
3. `squarefree_count`：n 維平方自由單項式恰可由 2ⁿ 個元素的列表枚舉
   （`allBits` 完備 + 計數）。 -/

namespace Polyrust

/-! ## 單項式的指數向量表示 -/

/-- n 變量多項式的單項式 = 指數向量（ℕ → ℕ；「n 維」指支撐 ⊆ {0,…,n−1}）。 -/
abbrev MonoExp := Nat → Nat

/-- 單項式整除：逐點指數 ≤。 -/
def dividesM (a b : MonoExp) : Prop := ∀ j, a j ≤ b j

/-- 平方自由單項式：每個指數 ≤ 1（即不被任何 xᵢ² 整除的必要形式）。 -/
def squarefreeM (m : MonoExp) : Prop := ∀ j, m j ≤ 1

/-- xᵢ² 的指數向量（域多項式 xᵢ² − xᵢ 的首項）。 -/
def x2 (i : Nat) : MonoExp := fun j => if j = i then 2 else 0

theorem x2_self (i : Nat) : x2 i i = 2 := by
  simp [x2]

theorem x2_ne {i j : Nat} (h : j ≠ i) : x2 i j = 0 := by
  simp [x2, h]

theorem dividesM_refl (a : MonoExp) : dividesM a a := fun _ => Nat.le_refl _

theorem dividesM_trans {a b c : MonoExp} (h1 : dividesM a b) (h2 : dividesM b c) :
    dividesM a c := fun j => Nat.le_trans (h1 j) (h2 j)

/-- 指數 ≥ 2 的維度上，xᵢ² 整除 m。 -/
theorem x2_divides_of_ge2 (m : MonoExp) (i : Nat) (h : 2 ≤ m i) : dividesM (x2 i) m := by
  intro j
  by_cases hj : j = i
  · rw [hj, x2_self]
    exact h
  · rw [x2_ne hj]
    exact Nat.zero_le _

/-! ## 環節一：標準單項式必平方自由 -/

/-- **T4 關鍵引理**：若每個 xᵢ² 都被基的某個首項整除（域多項式入基 ⇒
首項理想含 xᵢ²），則任何標準單項式（不被任何首項整除）必平方自由。

逆否形式即「非平方自由 ⇒ 非標準」：m 某維指數 ≥ 2 ⇒ xᵢ² | m ⇒
（傳遞性）某首項 | m。這限制了首項理想的補集大小 ≤ 2ⁿ。 -/
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

/-- **雙射（左逆）**：平方自由單項式 → 位串 → 單項式還原。 -/
theorem ofBits_toBits {m : MonoExp} (h : squarefreeM m) : ofBits (toBits m) = m := by
  funext j
  have hj : m j ≤ 1 := h j
  rcases Nat.lt_or_ge (m j) 2 with hlt | hge
  · have hm01 : m j = 0 ∨ m j = 1 := by omega
    rcases hm01 with h0 | h1
    · simp [ofBits, toBits, h0]
    · simp [ofBits, toBits, h1]
  · omega

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

/-! ## 總結：平方自由單項式恰 2ⁿ 個 -/

/-- **T4 計數定理**：n 維平方自由單項式可由長度恰為 2ⁿ 的列表完全枚舉
（可靠性 + 完備性 = 恰好枚舉一遍，即雙射計數）。
配合 `standard_implies_squarefree`（標準單項式 ⊆ 平方自由單項式），
基擴充可消耗的標準單項式 ≤ 2ⁿ ⇒ Buchberger 擴充次數 ≤ 2ⁿ ⇒ 必終止。 -/
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

end Polyrust
