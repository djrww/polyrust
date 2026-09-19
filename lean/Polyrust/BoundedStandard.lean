/- # ∏kᵢ 一般化終止界——從布爾（kᵢ≡2）到任意值域

對應 docs/THEOREMS.md §6 註記與 §6b。T4 的 2ⁿ 界依賴域多項式
xᵢ² − xᵢ（每個變量次數界 2）；過渡到真 rustc 後，多值變量
（候選型別集、obligation 解空間）用**消失多項式** ∏_v (x − v) 編碼，
值域大小 k 的變量次數界為 k。一般界即 **∏ᵢ kᵢ**——T4 的 2ⁿ 是
kᵢ ≡ 2 的特例（`general_bound_specializes_to_2n` 機械化此事實）。

本模組機械化四個環節（框架沿用 `Squarefree` 的抽象消耗論證）：

1. `xp`／`xp_divides_of_ge`／`standard_implies_boundedF`：
   若首項理想含每個變量的純冪 xᵢ^{kᵢ}，則標準單項式的每個指數
   有界（m i < k i）——T4「標準 ⇒ 平方自由」的直接推廣。
2. `boundOf`／`boundedByF`／`allBounded`：**混合進制枚舉**——
   有界指數向量與各維度取值的笛卡爾積一一對應，枚舉長度 = ∏ kᵢ
   （`allBounded_length`），可靠性與完備性齊備（雙射計數）。
3. `buchberger_extension_bound_general`：標準集鏈落在 `allBounded ks` 內
   ⇒ 擴充步數 ≤ ∏ kᵢ。
4. `twos`／`listProd_twos`／`general_bound_specializes_to_2n`：
   kᵢ ≡ 2 時 ∏ kᵢ = 2ⁿ，T4 原界是其推論。

**邊界**：與 `Squarefree` 相同，這是終止界的組合核心，不是演算法本身；
「次數界 ⇒ 標準集有界」的代數入口（理想含各變量純冪）由調用方以
`hlead` 假設提供。消失多項式的構造、求值與嵌入引理 L0′ 在
Rust 側（`core/src/vanishing.rs`）實現並測試。 -/

import Polyrust.Squarefree

namespace Polyrust

/-! ## 環節一：純冪與指數有界 -/

/-- 純冪 xᵢ^k 的指數向量。 -/
def xp (i k : Nat) : MonoExp := fun j => if j = i then k else 0

/-- 指數 ≥ k ⇒ xᵢ^k 整除該單項式。 -/
theorem xp_divides_of_ge (m : MonoExp) (i k : Nat) (h : k ≤ m i) :
    dividesM (xp i k) m := by
  intro j
  by_cases hj : j = i
  · show (if j = i then k else 0) ≤ m j
    rw [if_pos hj, hj]
    exact h
  · show (if j = i then k else 0) ≤ m j
    rw [if_neg hj]
    omega

/-- **T4 關鍵引理的推廣**：若每個變量都有某基首項整除其純冪
xᵢ^{b i}（消失多項式入基 ⇒ 首項理想含該純冪），則標準單項式的
指數逐維有界。逆否即「指數越界 ⇒ 非標準」。 -/
theorem standard_implies_boundedF (L : List MonoExp) (b : Nat → Nat)
    (hpos : ∀ i, 0 < b i)
    (hlead : ∀ i, ∃ ℓ ∈ L, dividesM ℓ (xp i (b i)))
    (m : MonoExp) (hstd : ∀ ℓ ∈ L, ¬ dividesM ℓ m) :
    ∀ i, m i < b i := by
  intro i
  rcases Nat.lt_or_ge (m i) (b i) with hlt | hge
  · exact hlt
  · obtain ⟨ℓ, hℓL, hℓ⟩ := hlead i
    exact absurd (dividesM_trans hℓ (xp_divides_of_ge m i (b i) hge)) (hstd ℓ hℓL)

/-! ## 環節二：混合進制枚舉（有界指數向量 ↔ ∏ kᵢ） -/

/-- 值域界函數：第 `j` 個變量的指數界；列表用盡後界為 1（指數必 0）。
遞迴定義避免了列表索引，與 `boundedByF`／`allBounded` 的結構歸納對齊。 -/
def boundOf : List Nat → Nat → Nat
  | [], _j => 1
  | k :: ks, j => if j = 0 then k else boundOf ks (j - 1)

/-- 有界單項式：每個指數嚴格小於其值域界。 -/
def boundedByF (ks : List Nat) (m : MonoExp) : Prop := ∀ j, m j < boundOf ks j

/-- 有界指數向量的全枚舉（混合進制；長度 = ∏ kᵢ）。 -/
def allBounded : List Nat → List MonoExp
  | [] => [fun _ => 0]
  | k :: ks => (allBounded ks).flatMap fun f => (List.range k).map fun r =>
      fun j => if j = 0 then r else f (j - 1)

/-- 自定義列表乘積（零依賴；避免對 Std 引理版本的依賴）。 -/
def listProd : List Nat → Nat
  | [] => 1
  | k :: ks => k * listProd ks

private theorem flatMap_range_length (k : Nat) (l : List MonoExp) :
    (l.flatMap fun f => (List.range k).map fun r =>
      fun j => if j = 0 then r else f (j - 1)).length = l.length * k := by
  induction l with
  | nil => simp
  | cons f fs ih =>
    rw [List.flatMap_cons, List.length_append, List.length_map, List.length_range,
      List.length_cons, ih, Nat.add_mul, Nat.one_mul]
    omega

/-- **∏kᵢ 計數定理**：枚舉長度 = 列表乘積。 -/
theorem allBounded_length (ks : List Nat) : (allBounded ks).length = listProd ks := by
  induction ks with
  | nil => simp [allBounded, listProd]
  | cons k ks ih =>
    rw [allBounded, flatMap_range_length, ih, listProd, Nat.mul_comm]

/-- 枚舉的可靠性：`allBounded ks` 的元素都有界。 -/
theorem allBounded_sound : ∀ (ks : List Nat) (m : MonoExp),
    m ∈ allBounded ks → boundedByF ks m := by
  intro ks
  induction ks with
  | nil =>
    intro m hm
    have hz : m = fun _ => 0 := List.mem_singleton.mp hm
    rw [hz]
    intro j
    have h1 : boundOf [] j = 1 := rfl
    show 0 < boundOf [] j
    omega
  | cons k ks ih =>
    intro m hm
    obtain ⟨f, hfL, hf⟩ := List.mem_flatMap.mp hm
    obtain ⟨r, hr, hrEq⟩ := List.mem_map.mp hf
    have hrk : r < k := List.mem_range.mp hr
    subst hrEq
    intro j
    show (if j = 0 then r else f (j - 1)) < boundOf (k :: ks) j
    have hbnd : boundOf (k :: ks) j = if j = 0 then k else boundOf ks (j - 1) := by
      simp only [boundOf]
    rw [hbnd]
    by_cases hj : j = 0
    · rw [if_pos hj, if_pos hj]
      exact hrk
    · rw [if_neg hj, if_neg hj]
      exact ih f hfL (j - 1)

/-- 枚舉的完備性：有界單項式都在 `allBounded ks` 內。 -/
theorem allBounded_complete : ∀ (ks : List Nat) (m : MonoExp),
    boundedByF ks m → m ∈ allBounded ks := by
  intro ks
  induction ks with
  | nil =>
    intro m hb
    have hz : m = fun _ => 0 := by
      funext j
      have hj : m j < boundOf [] j := hb j
      have h1 : boundOf [] j = 1 := rfl
      omega
    rw [hz]
    simp [allBounded]
  | cons k ks ih =>
    intro m hb
    have h0 : m 0 < k := by
      have hb0 : m 0 < boundOf (k :: ks) 0 := hb 0
      have hbnd : boundOf (k :: ks) 0 = k := rfl
      omega
    -- 尾部：右移一位仍是有界的
    have htail : boundedByF ks (fun j => m (j + 1)) := by
      intro j
      have hthis := hb (j + 1)
      have hbnd : boundOf (k :: ks) (j + 1) = boundOf ks j := by
        simp only [boundOf]
        exact if_neg (by omega)
      rw [hbnd] at hthis
      exact hthis
    have hmem : (fun j => m (j + 1)) ∈ allBounded ks := ih (fun j => m (j + 1)) htail
    apply List.mem_flatMap.mpr
    refine ⟨(fun j => m (j + 1)), hmem, ?_⟩
    apply List.mem_map.mpr
    refine ⟨m 0, List.mem_range.mpr h0, ?_⟩
    funext j
    show (if j = 0 then m 0 else m (j - 1 + 1)) = m j
    by_cases hj : j = 0
    · rw [if_pos hj, hj]
    · rw [if_neg hj]
      have hsub : j - 1 + 1 = j := by omega
      rw [hsub]

/-! ## 環節三：一般化終止界 -/

/-- **∏kᵢ 終止界（主定理）**：標準集鏈落在長度 ∏ kᵢ 的枚舉內 ⇒
擴充步數 ≤ ∏ kᵢ。T4（kᵢ ≡ 2）由此推出。 -/
theorem buchberger_extension_bound_general (ks : List Nat) (k : Nat) (S : Nat → List MonoExp)
    (hsub : ∀ j, (S (j + 1)).Sublist (S j))
    (hstrict : ∀ j, S (j + 1) ≠ S j)
    (hbase : (S 0).Sublist (allBounded ks)) :
    k ≤ listProd ks := by
  have h := extension_bound (allBounded ks) S hsub hstrict hbase k
  rwa [allBounded_length] at h

/-- 「標準 ⇒ 有界 ⇒ 在枚舉內」的一步到位推論（供上游以 `hlead` 直接接入）。 -/
theorem standard_mem_allBounded (ks : List Nat) (L : List MonoExp) (m : MonoExp)
    (hpos : ∀ j, 0 < boundOf ks j)
    (hlead : ∀ j, ∃ ℓ ∈ L, dividesM ℓ (xp j (boundOf ks j)))
    (hstd : ∀ ℓ ∈ L, ¬ dividesM ℓ m) :
    m ∈ allBounded ks :=
  allBounded_complete ks m (standard_implies_boundedF L (boundOf ks) hpos hlead m hstd)

/-! ## 環節四：T4 是特例（kᵢ ≡ 2 ⇒ ∏ kᵢ = 2ⁿ） -/

/-- n 個值域界全為 2 的列表。 -/
def twos : Nat → List Nat
  | 0 => []
  | n + 1 => 2 :: twos n

/-- ∏ (2,…,2) = 2ⁿ：一般界在布爾情形回到 T4。 -/
theorem listProd_twos (n : Nat) : listProd (twos n) = 2 ^ n := by
  induction n with
  | zero => simp [twos, listProd]
  | succ n ih =>
    rw [twos, listProd, ih, Nat.pow_succ, Nat.mul_comm]

/-- **T4 ⊂ ∏kᵢ**：值域界全 2 時，一般化終止界即原布爾界 2ⁿ。 -/
theorem general_bound_specializes_to_2n (n k : Nat) (S : Nat → List MonoExp)
    (hsub : ∀ j, (S (j + 1)).Sublist (S j))
    (hstrict : ∀ j, S (j + 1) ≠ S j)
    (hbase : (S 0).Sublist (allBounded (twos n))) :
    k ≤ 2 ^ n := by
  have h := buchberger_extension_bound_general (twos n) k S hsub hstrict hbase
  rwa [listProd_twos] at h

end Polyrust
