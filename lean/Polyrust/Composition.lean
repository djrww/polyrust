/- # 組合性（定理 T10）——模塊化編碼的終止界：機械化核心

對應 docs/THEOREMS.md §6b（T10）。動機：過渡到真 rustc（MIR + Chalk）後，
單一系統的位元變量數急劇膨脹，T4 的 2ⁿ 界雖仍成立但已無實用意義。
解法是**改組件粒度**：跟隨編碼的天然邊界把系統切成變量集兩兩不相交的
子系統，各自獨立求解——總成本界由 2ⁿ 降為 **Σᵢ 2^{nᵢ}**（各組件界之和）。

本模組機械化此論證的四個組合環節（沿用 `Squarefree` 的抽象消耗框架）：

1. `restrictM`／`shiftM`：變量集的截斷與平移；整除性在兩者下保持
   （`dividesM_restrict_iff`、`dividesM_shift_iff`）。
2. `standard_iff_factor`：**標準單項式分解引理**——對不相交變量集上的
   首項列表，整體標準 ⟺ 兩個組件各自標準。這是「並基仍是基」的組合內核；
   代數側依據是跨組件首項互素 ⇒ S-多項式由第一準則（T5(a)）歸零，
   機械化於 `coprime_disjoint`。
3. `prodMono`／`prodMono_length`／`standard_mem_prod_universe`：
   不相交組件的標準單項式宇宙是兩組件宇宙之積，長度相乘
   ⇒ 合併運行的界 2^{n₁}·2^{n₂} = 2^{n₁+n₂}（與 T4 一致，無損失亦無增益）。
4. `composed_extensions_bound`：**分組運行的加法界**——各組件分別消耗
   ≤ 2^{n₁}、≤ 2^{n₂}，合計 ≤ 2^{n₁}+2^{n₂} ≤ 2^{n₁+n₂}（n₁,n₂ ≥ 1）。
   模塊化的真正收益在中段：界是**和**而非積的結構，讓常數上界的組件
   給出線性於程式規模的總界。

**邊界**：與 `Squarefree` 相同，這裡證明的是終止界的組合核心，不是
Buchberger 演算法本身；「各組件約化基之並 = 全系統 Gröbner 基」的
逐步計算驗證在 Rust 側（`core/src/composition.rs::verify_union_basis`）
對所有配對直接做，兩側互補。 -/

import Polyrust.Squarefree

namespace Polyrust

/-! ## 環節一：變量集截斷與平移 -/

/-- 截斷：只保留前 `n` 個變量的指數。 -/
def restrictM (m : MonoExp) (n : Nat) : MonoExp := fun j => if j < n then m j else 0

/-- 平移：把指數向量整體右移 `off` 格（用於不相交組件的變量重編號）。 -/
def shiftM (m : MonoExp) (off : Nat) : MonoExp := fun j => if off ≤ j then m (j - off) else 0

theorem restrictM_eq_of_lt (m : MonoExp) (n : Nat) (j : Nat) (h : j < n) :
    restrictM m n j = m j := by
  simp only [restrictM]
  exact if_pos h

theorem restrictM_eq_zero_of_ge (m : MonoExp) (n : Nat) (j : Nat) (h : n ≤ j) :
    restrictM m n j = 0 := by
  simp only [restrictM]
  exact if_neg (by omega)

theorem shiftM_eq_of_ge (m : MonoExp) (off : Nat) (j : Nat) (h : off ≤ j) :
    shiftM m off j = m (j - off) := by
  simp only [shiftM]
  exact if_pos h

theorem shiftM_eq_zero_of_lt (m : MonoExp) (off : Nat) (j : Nat) (h : j < off) :
    shiftM m off j = 0 := by
  simp only [shiftM]
  exact if_neg (by omega)

/-- 截斷不增加支撐。 -/
theorem supportLeM_restrict (m : MonoExp) (n : Nat) : supportLeM (restrictM m n) n := by
  intro j hj
  exact restrictM_eq_zero_of_ge m n j hj

/-- 平移 n 維單項式 ⇒ 支撐落在 [off, off+n)。 -/
theorem supportLeM_shift (m : MonoExp) (off n : Nat) (h : supportLeM m n) :
    supportLeM (shiftM m off) (off + n) := by
  intro j hj
  by_cases hlt : off ≤ j
  · rw [shiftM_eq_of_ge m off j hlt]
    exact h (j - off) (by omega)
  · rw [shiftM_eq_zero_of_lt m off j (by omega)]

/-- 截斷下的整除等價：支撐 ⊆ 前 n 維的首項「整除 m」⟺「整除 m 的截斷」。 -/
theorem dividesM_restrict_iff (ℓ m : MonoExp) (n : Nat) (hsupp : supportLeM ℓ n) :
    dividesM ℓ m ↔ dividesM ℓ (restrictM m n) := by
  constructor
  · intro hd j
    by_cases hj : j < n
    · rw [restrictM_eq_of_lt m n j hj]
      exact hd j
    · have hl : ℓ j = 0 := hsupp j (by omega)
      rw [restrictM_eq_zero_of_ge m n j (by omega)]
      omega
  · intro hd j
    by_cases hj : j < n
    · have := hd j
      rwa [restrictM_eq_of_lt m n j hj] at this
    · have hl : ℓ j = 0 := hsupp j (by omega)
      omega

/-- 平移下的整除等價：平移首項整除整體單項式 ⟺ 原首項整除「右移視角」。 -/
theorem dividesM_shift_iff (ℓ m : MonoExp) (off n : Nat) (hsupp : supportLeM ℓ n) :
    dividesM (shiftM ℓ off) m ↔ dividesM ℓ (fun k => m (off + k)) := by
  constructor
  · intro hd k
    have := hd (off + k)
    rw [shiftM_eq_of_ge ℓ off (off + k) (by omega)] at this
    have hsub : off + k - off = k := by omega
    rwa [hsub] at this
  · intro hd j
    by_cases hj : off ≤ j
    · rw [shiftM_eq_of_ge ℓ off j hj]
      have hthis : ℓ (j - off) ≤ m (off + (j - off)) := hd (j - off)
      have hadd : off + (j - off) = j := by omega
      rwa [hadd] at hthis
    · rw [shiftM_eq_zero_of_lt ℓ off j (by omega)]
      omega

/-! ## 環節二：標準單項式分解引理 -/

/-- **T10 核心引理（標準分解）**：首項列表 `L₁`（變量 0..n₁）與
`L₂`（平移後變量 n₁..n₁+n₂）變量不相交時，單項式對 `L₁ ++ shift(L₂)`
標準 ⟺ 截斷對 `L₁` 標準 ∧ 右移視角對 `L₂` 標準。 -/
theorem standard_iff_factor (L₁ L₂ : List MonoExp) (n₁ n₂ : Nat) (m : MonoExp)
    (hsupp1 : ∀ ℓ ∈ L₁, supportLeM ℓ n₁)
    (hsupp2 : ∀ ℓ ∈ L₂, supportLeM ℓ n₂) :
    (∀ ℓ ∈ L₁ ++ L₂.map (shiftM · n₁), ¬ dividesM ℓ m) ↔
    (∀ ℓ ∈ L₁, ¬ dividesM ℓ (restrictM m n₁)) ∧
    (∀ ℓ ∈ L₂, ¬ dividesM ℓ fun k => m (n₁ + k)) := by
  constructor
  · intro h
    constructor
    · intro ℓ hℓ hd
      exact h ℓ (List.mem_append.mpr (Or.inl hℓ))
        (by rwa [dividesM_restrict_iff ℓ m n₁ (hsupp1 ℓ hℓ)])
    · intro ℓ hℓ hd
      have hmem : shiftM ℓ n₁ ∈ L₁ ++ L₂.map (shiftM · n₁) :=
        List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨ℓ, hℓ, rfl⟩))
      exact h (shiftM ℓ n₁) hmem
        (by rwa [dividesM_shift_iff ℓ m n₁ n₂ (hsupp2 ℓ hℓ)])
  · rintro ⟨h1, h2⟩ ℓ hℓ hd
    rcases List.mem_append.mp hℓ with hA | hB
    · exact h1 ℓ hA (by rwa [← dividesM_restrict_iff ℓ m n₁ (hsupp1 ℓ hA)])
    · obtain ⟨ℓ₂, hℓ₂, hEq⟩ := List.mem_map.mp hB
      rw [← hEq] at hd
      exact h2 ℓ₂ hℓ₂ (by rwa [← dividesM_shift_iff ℓ₂ m n₁ n₂ (hsupp2 ℓ₂ hℓ₂)])

/-- 跨組件首項互素：支撐不相交的兩個首項，任何維度上不同時為正。
這是「並基仍是 Gröbner 基」的代數依據（第一準則，T5(a)）的組合內容。 -/
theorem coprime_disjoint (a b : MonoExp) (n : Nat)
    (ha : supportLeM a n) (hb : ∀ j, j < n → b j = 0) : coprimeM a b := by
  intro j
  by_cases hj : j < n
  · right; exact hb j hj
  · left; exact ha j (by omega)

/-! ## 環節三：合併運行的乘法界（2^{n₁}·2^{n₂} = 2^{n₁+n₂}） -/

/-- 不相交宇宙之積：每對 (m₁, m₂) 合成單項式乘法（支撐不相交 ⇒ 無衝突）。 -/
def prodMono (U₁ U₂ : List MonoExp) : List MonoExp :=
  U₁.flatMap fun m₁ => U₂.map fun m₂ => monoMul m₁ m₂

theorem prodMono_length (U₁ U₂ : List MonoExp) :
    (prodMono U₁ U₂).length = U₁.length * U₂.length := by
  induction U₁ with
  | nil => simp [prodMono]
  | cons a as ih =>
    have hsplit : (prodMono (a :: as) U₂).length
        = (U₂.map fun m₂ => monoMul a m₂).length + (prodMono as U₂).length := by
      show ((a :: as).flatMap fun m₁ => U₂.map fun m₂ => monoMul m₁ m₂).length
          = (U₂.map fun m₂ => monoMul a m₂).length
            + (as.flatMap fun m₁ => U₂.map fun m₂ => monoMul m₁ m₂).length
      rw [List.flatMap_cons, List.length_append]
    rw [hsplit, List.length_map, List.length_cons, ih, Nat.add_mul, Nat.one_mul]
    omega

theorem mem_prodMono {U₁ U₂ : List MonoExp} {m : MonoExp} :
    m ∈ prodMono U₁ U₂ ↔ ∃ m₁ ∈ U₁, ∃ m₂ ∈ U₂, m = monoMul m₁ m₂ := by
  constructor
  · intro h
    obtain ⟨m₁, hm₁, hm⟩ := List.mem_flatMap.mp h
    obtain ⟨m₂, hm₂, heq⟩ := List.mem_map.mp hm
    exact ⟨m₁, hm₁, m₂, hm₂, heq.symm⟩
  · rintro ⟨m₁, hm₁, m₂, hm₂, rfl⟩
    exact List.mem_flatMap.mpr ⟨m₁, hm₁, List.mem_map.mpr ⟨m₂, hm₂, rfl⟩⟩

private theorem restrictM_squarefree (m : MonoExp) (n : Nat) (hsq : squarefreeM m) :
    squarefreeM (restrictM m n) := by
  intro j
  by_cases hj : j < n
  · rw [restrictM_eq_of_lt m n j hj]
    exact hsq j
  · rw [restrictM_eq_zero_of_ge m n j (by omega)]
    omega

/-- 標準單項式（平方自由 + 支撐 ≤ n₁+n₂）落在兩個組件宇宙的積內：
這是「合併運行的初始標準集 ⊆ 積宇宙」的成員資格論證。 -/
theorem standard_mem_prod_universe (n₁ n₂ : Nat) (m : MonoExp)
    (hsq : squarefreeM m) (hsupp : supportLeM m (n₁ + n₂)) :
    m ∈ prodMono ((allBits n₁).map ofBits)
        ((allBits n₂).map fun f => shiftM (ofBits f) n₁) := by
  have hm1sq : squarefreeM (restrictM m n₁) := restrictM_squarefree m n₁ hsq
  have hm2sq : squarefreeM (fun k => m (n₁ + k)) := fun k => hsq (n₁ + k)
  -- 兩半各自的位串在 allBits 內（完備性）
  have h1 : toBits (restrictM m n₁) ∈ allBits n₁ := by
    apply allBits_complete n₁ (toBits (restrictM m n₁))
    intro j hj
    have hz : restrictM m n₁ j = 0 := supportLeM_restrict m n₁ j hj
    simp [toBits, hz]
  have h2 : toBits (fun k => m (n₁ + k)) ∈ allBits n₂ := by
    apply allBits_complete n₂ (toBits (fun k => m (n₁ + k)))
    intro j hj
    have hz : m (n₁ + j) = 0 := hsupp (n₁ + j) (by omega)
    simp [toBits, hz]
  -- 逐點重組：m = restrict m · shift(右移視角)
  have hdec : m = monoMul (restrictM m n₁) (shiftM (fun k => m (n₁ + k)) n₁) := by
    funext j
    show m j = (restrictM m n₁) j + (shiftM (fun k => m (n₁ + k)) n₁) j
    by_cases hj : j < n₁
    · have e1 : restrictM m n₁ j = m j := restrictM_eq_of_lt m n₁ j hj
      have e2 : (shiftM (fun k => m (n₁ + k)) n₁) j = 0 :=
        shiftM_eq_zero_of_lt (fun k => m (n₁ + k)) n₁ j hj
      omega
    · have e1 : restrictM m n₁ j = 0 := restrictM_eq_zero_of_ge m n₁ j (by omega)
      have e2 : (shiftM (fun k => m (n₁ + k)) n₁) j = m (n₁ + (j - n₁)) :=
        shiftM_eq_of_ge (fun k => m (n₁ + k)) n₁ j (by omega)
      have e3 : m (n₁ + (j - n₁)) = m j := by
        congr 1
        omega
      omega
  rw [mem_prodMono]
  refine ⟨restrictM m n₁,
    List.mem_map.mpr ⟨toBits (restrictM m n₁), h1, ofBits_toBits hm1sq⟩,
    shiftM (fun k => m (n₁ + k)) n₁,
    List.mem_map.mpr ⟨toBits (fun k => m (n₁ + k)), h2, ?_⟩, hdec⟩
  show shiftM (ofBits (toBits (fun k => m (n₁ + k)))) n₁ = shiftM (fun k => m (n₁ + k)) n₁
  rw [ofBits_toBits hm2sq]

/-- **T10 乘法界**：合併運行的標準集鏈落在積宇宙內 ⇒ 步數 ≤ 2^{n₁}·2^{n₂}。 -/
theorem buchberger_extension_bound_product (n₁ n₂ k : Nat) (S : Nat → List MonoExp)
    (hsub : ∀ j, (S (j + 1)).Sublist (S j))
    (hstrict : ∀ j, S (j + 1) ≠ S j)
    (hbase : (S 0).Sublist (prodMono ((allBits n₁).map ofBits)
        ((allBits n₂).map fun f => shiftM (ofBits f) n₁))) :
    k ≤ 2 ^ n₁ * 2 ^ n₂ := by
  have h := extension_bound (prodMono ((allBits n₁).map ofBits)
      ((allBits n₂).map fun f => shiftM (ofBits f) n₁)) S hsub hstrict hbase k
  rw [prodMono_length, List.length_map, List.length_map, allBits_length,
      allBits_length] at h
  exact h

/-- 乘法界與 T4 一致：2^{n₁}·2^{n₂} = 2^{n₁+n₂}（合併不會更好也不會更壞）。 -/
theorem buchberger_extension_bound_product_pow (n₁ n₂ k : Nat) (S : Nat → List MonoExp)
    (hsub : ∀ j, (S (j + 1)).Sublist (S j))
    (hstrict : ∀ j, S (j + 1) ≠ S j)
    (hbase : (S 0).Sublist (prodMono ((allBits n₁).map ofBits)
        ((allBits n₂).map fun f => shiftM (ofBits f) n₁))) :
    k ≤ 2 ^ (n₁ + n₂) := by
  have h := buchberger_extension_bound_product n₁ n₂ k S hsub hstrict hbase
  rw [← Nat.pow_add] at h
  exact h

/-! ## 環節四：分組運行的加法界（模塊化的真正收益） -/

/-- n ≥ 1 ⇒ 2 ≤ 2ⁿ。 -/
theorem two_le_pow_of_pos (n : Nat) (h : 1 ≤ n) : 2 ≤ 2 ^ n := by
  have h1 : 2 = 2 ^ 1 := (Nat.pow_one 2).symm
  rw [h1]
  exact Nat.pow_le_pow_right (by omega) h

/-- a,b ≥ 2 ⇒ a + b ≤ a·b（和 ≤ 積——加法界收進乘法界的算術核心）。 -/
theorem add_le_mul_of_two_le (a b : Nat) (ha : 2 ≤ a) (hb : 2 ≤ b) : a + b ≤ a * b := by
  obtain ⟨c, rfl⟩ : ∃ c, b = c + 1 := ⟨b - 1, by omega⟩
  -- 目標：a + (c + 1) ≤ a * c + a；核心：c + 1 ≤ 2c ≤ a·c（a ≥ 2, c ≥ 1）
  rw [Nat.mul_add, Nat.mul_one]
  have h3 : 2 * c ≤ a * c := Nat.mul_le_mul_right c ha
  omega

/-- **T10 加法界（主定理）**：兩個非空組件各自跑 Buchberger，
擴充次數分別 ≤ 2^{n₁}、≤ 2^{n₂}，則合計 ≤ 2^{n₁+n₂}。
與合併運行同階但結構是**和**：推廣到 m 個常數大小組件 ⇒ 總界線性於程式規模。 -/
theorem composed_extensions_bound (k₁ k₂ n₁ n₂ : Nat)
    (h₁ : k₁ ≤ 2 ^ n₁) (h₂ : k₂ ≤ 2 ^ n₂) (hn₁ : 1 ≤ n₁) (hn₂ : 1 ≤ n₂) :
    k₁ + k₂ ≤ 2 ^ (n₁ + n₂) := by
  calc
    k₁ + k₂ ≤ 2 ^ n₁ + 2 ^ n₂ := Nat.add_le_add h₁ h₂
    _ ≤ 2 ^ n₁ * 2 ^ n₂ :=
      add_le_mul_of_two_le _ _ (two_le_pow_of_pos n₁ hn₁) (two_le_pow_of_pos n₂ hn₂)
    _ = 2 ^ (n₁ + n₂) := (Nat.pow_add 2 n₁ n₂).symm

/-- 加法界的直接形式：Σ 界 ≤ Σ 2^{nᵢ}（兩組件情形），不折進 2^{Σnᵢ}——
這是報表裡「組合界」一欄的數學內容。 -/
theorem sum_component_bounds (k₁ k₂ n₁ n₂ : Nat)
    (h₁ : k₁ ≤ 2 ^ n₁) (h₂ : k₂ ≤ 2 ^ n₂) :
    k₁ + k₂ ≤ 2 ^ n₁ + 2 ^ n₂ :=
  Nat.add_le_add h₁ h₂

end Polyrust
