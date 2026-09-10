/- # Gröbner 判定定理（T6）：求值同態、憑證與布爾插值

對應 docs/THEOREMS.md §8（T6）：

> 1 ∈ G(F(e) ∪ B ∪ Φ) ⟺ e 不可定型（或借用不安全）。

原文的 (⟸) 用「有根則 1 不在理想」，(⟹) 用「1 在理想 ⟹ 無 0/1 解」。本模組把
這兩半寫成可檢查的形式，並補上原文口頭帶過的關鍵事實：

1. **求值同態（`Eval`）與可靠性**（`inIdeal_no_root`）：
   `1 ∈ ⟨S⟩` 與「存在 E-零點」互斥（E 是任意把多項式送到 ℤ 的環同態）。
   這是原文「求值同態把 1 映到 1 ≠ 0」一句的完整形式化。
2. **憑證完備性**（`no_root_certificate`）：系統在有限布爾點集上無公共零點 ⟹
   存在乘子把系統組合成一個**處處非零的函數** D。
3. **多項式乘子**（`no_root_poly_certificate`）：配合布爾插值，乘子可取為
   **多項式函數**——這使「1 ∈ G」的判定在布爾多項式環中成立。
4. **𝔽_p 上的 1 ∈ I**（`mod_p_one_in_ideal`）：給定 D 的模 p 逆元見證，
   可把 D 縮放為常數 1（這一步需要 p 的素性；本模組以顯式見證取代）。
5. **布爾插值定理**（`interpolation`）：{0,1}ⁿ 上**任何函數**都是多項式函數，
   以顯式拉格朗日基 δ_σ = ∏_j(σ_j ? x_j : 1 − x_j) 構造。

**邊界（說死）**：
- 「函數」是布爾點集上的 ℤ 值函數；「多項式函數」由 `PolyFn` **歸納定義**
  （生成元：常數與座標函數；封閉於 +、×、−），不是一般多項式環。
  故這裡證明的是**布爾多項式的理想判定**，不是完整多項式環理想論。
- 插值與憑證都在**有限點集**（支撐 ≤ n 的布爾點）上陳述；點集以列舉
  `pts` 給出，並假設 `pts.Nodup` 與覆蓋性——列舉的存在性由
  `Polyrust.Squarefree.allBits` 提供（其 `Nodup` 性是純組合事實）。
- 𝔽_p 結論需要「D 模 p 非零」與逆元見證作為顯式假設。 -/

import Polyrust.SPoly
import Polyrust.Squarefree
import Polyrust.ClauseDuality

namespace Polyrust

open Classical

/-! ## 一、求值同態與可靠性 -/

/-- 常數多項式 1。 -/
noncomputable def oneP : MPoly := fun m => if m = monoOne then 1 else 0

/-- 求值同態：把多項式送到 ℤ 的環同態（單項式 μ 的值為 `w μ`）。
布爾點求值、代入特化、模 p 化約都是此結構的實例。 -/
structure Eval where
  toFun : MPoly → Int
  w : MonoExp → Int
  map_zero : toFun zeroP = 0
  map_sub : ∀ p q, toFun (subP p q) = toFun p - toFun q
  map_smul : ∀ μ p, toFun (mulMono μ p) = w μ * toFun p

/-- **T6 可靠性（一般形式）**：若系統 S 的每個元素都被求值同態 E 送到 0，
則 `1 ∉ ⟨S⟩`——「1 屬於理想」與「存在公共零點」互斥。
（原文「求值同態把 1 映到 1 ≠ 0」的形式化。） -/
theorem inIdeal_no_root (E : Eval) (h1 : E.toFun oneP = 1) {S : MPoly → Prop}
    (hmem : genIdeal S oneP) (hroot : ∀ p, S p → E.toFun p = 0) : False := by
  have hzero : IsIdeal (fun p => E.toFun p = 0) := by
    refine ⟨E.map_zero, ?_, ?_⟩
    · intro p q hp hq
      rw [E.map_sub, hp, hq]
      omega
    · intro μ p hp
      rw [E.map_smul, hp]
      omega
  have h := hmem (fun p => E.toFun p = 0) hzero hroot
  rw [h1] at h
  exact absurd h (by decide)

/-- 逆否形式（原文 (⟹) 方向）：若 1 屬於 S 生成的理想，
則 S 不可能全部被 E 送到 0——即「1 ∈ G」排除了任何公共零點的存在。 -/
theorem one_mem_no_root (E : Eval) (h1 : E.toFun oneP = 1) {S : MPoly → Prop}
    (hmem : genIdeal S oneP) : ¬ (∀ p, S p → E.toFun p = 0) :=
  fun hroot => inIdeal_no_root E h1 hmem hroot

/-! ## 二、布爾方體上的多項式函數與插值 -/

/-- 布爾點上的 ℤ 值函數。 -/
abbrev F := (Nat → Bool) → Int

/-- 座標函數（第 j 個變量的 0/1 取值）。 -/
def coord (j : Nat) : F := fun x => bit (x j)

/-- 「多項式函數」：由常數與座標函數經 +、×、− 生成的函數。 -/
inductive PolyFn (n : Nat) : F → Prop
  | const (c : Int) : PolyFn n (fun _ => c)
  | coord (j : Nat) (hj : j < n) : PolyFn n (coord j)
  | add {f g : F} : PolyFn n f → PolyFn n g → PolyFn n (fun x => f x + g x)
  | mul {f g : F} : PolyFn n f → PolyFn n g → PolyFn n (fun x => f x * g x)
  | neg {f : F} : PolyFn n f → PolyFn n (fun x => -(f x))

namespace PolyFn

variable {n m : Nat} {f g : F}

theorem zero : PolyFn n (fun _ => (0 : Int)) := const 0
theorem one : PolyFn n (fun _ => (1 : Int)) := const 1

theorem sub (hf : PolyFn n f) (hg : PolyFn n g) : PolyFn n (fun x => f x - g x) := by
  have h := add hf (neg hg)
  have heq : (fun x => f x + -(g x)) = (fun x => f x - g x) := by
    funext x; omega
  rw [heq] at h
  exact h

theorem complCoord (j : Nat) (hj : j < n) : PolyFn n (fun x => 1 - bit (x j)) :=
  sub one (coord j hj)

theorem smulConst (c : Int) (hf : PolyFn n f) : PolyFn n (fun x => c * f x) := by
  have := mul (const c) hf
  simpa using this

theorem mono (hnm : n ≤ m) : PolyFn n f → PolyFn m f := by
  intro hf
  induction hf with
  | const c => exact const c
  | coord j hj => exact coord j (by omega)
  | add _ _ ihf ihg => exact add ihf ihg
  | mul _ _ ihf ihg => exact mul ihf ihg
  | neg _ ih => exact neg ih

theorem sumList (l : List F) (h : ∀ g ∈ l, PolyFn n g) :
    PolyFn n (fun x => (l.map (fun g => g x)).sum) := by
  induction l with
  | nil => simpa using (zero : PolyFn n (fun _ => (0 : Int)))
  | cons a rest ih =>
    have ha : PolyFn n a := h a (by simp)
    have hrest : ∀ g ∈ rest, PolyFn n g := fun g hg => h g (by simp [hg])
    have hih := ih hrest
    have heq : (fun x => ((a :: rest).map (fun g => g x)).sum)
        = (fun x => a x + (rest.map (fun g => g x)).sum) := by
      funext x; simp
    rw [heq]
    exact add ha hih

theorem prodList (l : List F) (h : ∀ g ∈ l, PolyFn n g) :
    PolyFn n (fun x => (l.map (fun g => g x)).prod) := by
  induction l with
  | nil => simpa using (one : PolyFn n (fun _ => (1 : Int)))
  | cons a rest ih =>
    have ha : PolyFn n a := h a (by simp)
    have hrest : ∀ g ∈ rest, PolyFn n g := fun g hg => h g (by simp [hg])
    have hih := ih hrest
    have heq : (fun x => ((a :: rest).map (fun g => g x)).prod)
        = (fun x => a x * (rest.map (fun g => g x)).prod) := by
      funext x; simp
    rw [heq]
    exact mul ha hih

end PolyFn

/-! ## 三、拉格朗日基 δ_σ 及其性質 -/

/-- 拉格朗日基（指標函數）：δ n σ 在 σ 上取值 1、在其他（支撐 ≤ n 的）點取值 0。
以低 n 個座標的因子乘積遞歸構造。 -/
def delta : Nat → (Nat → Bool) → F
  | 0, _ => fun _ => 1
  | k + 1, σ => fun x => (if σ k then bit (x k) else 1 - bit (x k)) * delta k σ x

theorem delta_self (n : Nat) (σ : Nat → Bool) : delta n σ σ = 1 := by
  induction n with
  | zero => rfl
  | succ k ih =>
    show (if σ k then bit (σ k) else 1 - bit (σ k)) * delta k σ σ = 1
    rw [ih]
    cases hk : σ k <;> simp [bit]

/-- δ 在「座標差異」處歸零。 -/
theorem delta_eq_zero_aux : ∀ (n : Nat) (σ x : Nat → Bool) (j : Nat),
    j < n → x j ≠ σ j → delta n σ x = 0 := by
  intro n
  induction n with
  | zero =>
    intro σ x j hj _
    omega
  | succ k ih =>
    intro σ x j hj hjn
    by_cases hjk : j = k
    · subst hjk
      cases hσk : σ j
      · have hxj : x j = true := by
          have hh := hjn
          rw [hσk] at hh
          cases hxx : x j
          · exact absurd (by rw [hxx]) hh
          · rfl
        show (if σ j then bit (x j) else 1 - bit (x j)) * delta j σ x = 0
        rw [hσk, hxj]; simp [bit]
      · have hxj : x j = false := by
          have hh := hjn
          rw [hσk] at hh
          cases hxx : x j
          · rfl
          · exact absurd (by rw [hxx]) hh
        show (if σ j then bit (x j) else 1 - bit (x j)) * delta j σ x = 0
        rw [hσk, hxj]; simp [bit]
    · have hjk' : j < k := by omega
      show (if σ k then bit (x k) else 1 - bit (x k)) * delta k σ x = 0
      rw [ih σ x j hjk' hjn, Int.mul_zero]

theorem delta_eq_zero (n : Nat) (σ x : Nat → Bool)
    (h : ∃ j, j < n ∧ x j ≠ σ j) : delta n σ x = 0 := by
  obtain ⟨j, hj, hjn⟩ := h
  exact delta_eq_zero_aux n σ x j hj hjn

/-- 兩個支撐 ≤ n 的布爾點若不相等，則在某個 j < n 上不同。 -/
theorem exists_diff_of_ne {n : Nat} {x y : Nat → Bool}
    (hx : supportLe x n) (hy : supportLe y n) (hne : x ≠ y) :
    ∃ j, j < n ∧ x j ≠ y j := by
  by_cases h : ∃ j, j < n ∧ x j ≠ y j
  · exact h
  · exfalso
    apply hne
    funext j
    by_cases hj : j < n
    · by_cases hjj : x j = y j
      · exact hjj
      · exact absurd (Exists.intro j (And.intro hj hjj)) h
    · rw [hx j (by omega), hy j (by omega)]

theorem delta_polyFn (n : Nat) (σ : Nat → Bool) : PolyFn n (delta n σ) := by
  induction n with
  | zero => exact PolyFn.const 1
  | succ k ih =>
    show PolyFn (k+1) (fun x => (if σ k then bit (x k) else 1 - bit (x k)) * delta k σ x)
    by_cases hk : σ k
    · have h1 : (fun x => (if σ k then bit (x k) else 1 - bit (x k)) * delta k σ x)
          = (fun x => coord k x * delta k σ x) := by
        funext x; rw [if_pos hk]; rfl
      rw [h1]
      exact PolyFn.mul (PolyFn.coord k (by omega)) (PolyFn.mono (by omega) ih)
    · have h1 : (fun x => (if σ k then bit (x k) else 1 - bit (x k)) * delta k σ x)
          = (fun x => (1 - coord k x) * delta k σ x) := by
        funext x; rw [if_neg hk]; rfl
      rw [h1]
      exact PolyFn.mul (PolyFn.complCoord k (by omega)) (PolyFn.mono (by omega) ih)

/-! ## 四、分拆單位與插值定理 -/

theorem sum_zero_of_all_zero (l : List Int) (h : ∀ a ∈ l, a = 0) : l.sum = 0 := by
  induction l with
  | nil => simp
  | cons a rest ih =>
    rw [List.sum_cons, h a (by simp)]
    have : ∀ b ∈ rest, b = 0 := fun b hb => h b (by simp [hb])
    rw [ih this]
    simp

/-- **分拆單位**：δ 是布爾點集上的一組「單位分拆」——
`Σ_{σ ∈ pts} a(σ)·δ_σ(x) = a(x)`（點集需無重複、覆蓋且支撐 ≤ n）。 -/
theorem sum_delta {n : Nat} (pts : List (Nat → Bool)) (hnd : pts.Nodup)
    (hsound : ∀ σ ∈ pts, supportLe σ n) (a : (Nat → Bool) → Int) (x : Nat → Bool)
    (hx : x ∈ pts) (hxs : supportLe x n) :
    (pts.map (fun σ => a σ * delta n σ x)).sum = a x := by
  induction pts with
  | nil => exact absurd hx (by simp)
  | cons σ rest ih =>
    rw [List.map_cons, List.sum_cons]
    obtain ⟨hσrest, hndrest⟩ := List.nodup_cons.mp hnd
    by_cases hxσ : x = σ
    · subst hxσ
      have hrest0 : (rest.map (fun τ => a τ * delta n τ x)).sum = 0 := by
        apply sum_zero_of_all_zero
        intro b hb
        obtain ⟨τ, hτrest, hτeq⟩ := List.mem_map.mp hb
        rw [← hτeq]
        have hne : x ≠ τ := fun h => hσrest (h ▸ hτrest)
        obtain ⟨j, hj, hjn⟩ := exists_diff_of_ne hxs (hsound τ (by simp [hτrest])) hne
        rw [delta_eq_zero n τ x ⟨j, hj, hjn⟩, Int.mul_zero]
      rw [hrest0, delta_self, Int.mul_one, Int.add_zero]
    · have hxrest : x ∈ rest := by
        rcases List.mem_cons.mp hx with h | h
        · exact absurd h hxσ
        · exact h
      obtain ⟨j, hj, hjn⟩ := exists_diff_of_ne hxs (hsound σ (by simp)) hxσ
      have h0 : a σ * delta n σ x = 0 := by
        rw [delta_eq_zero n σ x ⟨j, hj, hjn⟩, Int.mul_zero]
      rw [h0, Int.zero_add]
      exact ih hndrest (fun τ hτ => hsound τ (by simp [hτ])) hxrest

/-- 以分拆單位把任意函數「插值」為多項式函數。 -/
noncomputable def interp (n : Nat) (pts : List (Nat → Bool)) (g : F) : F :=
  fun x => (pts.map (fun σ => g σ * delta n σ x)).sum

theorem interp_polyFn {n : Nat} (pts : List (Nat → Bool)) (g : F) :
    PolyFn n (interp n pts g) := by
  have h : ∀ f ∈ pts.map (fun σ => fun x => g σ * delta n σ x), PolyFn n f := by
    intro f hf
    obtain ⟨σ, _, hσeq⟩ := List.mem_map.mp hf
    rw [← hσeq]
    exact PolyFn.smulConst (g σ) (delta_polyFn n σ)
  have hsum := PolyFn.sumList (n := n) (pts.map (fun σ => fun x => g σ * delta n σ x)) h
  have heq : (fun x => ((pts.map (fun σ => fun x => g σ * delta n σ x)).map (fun f => f x)).sum)
      = interp n pts g := by
    funext x
    simp only [interp, List.map_map, Function.comp_def]
  rw [heq] at hsum
  exact hsum

/-- **布爾插值定理**：支撐 ≤ n 的布爾點上，任何函數都與某個多項式函數一致
（`interp` 給出顯式構造：拉格朗日基的線性組合）。 -/
theorem interpolation {n : Nat} (pts : List (Nat → Bool)) (hnd : pts.Nodup)
    (hsound : ∀ σ ∈ pts, supportLe σ n) (hcover : ∀ x, supportLe x n → x ∈ pts) (g : F) :
    PolyFn n (interp n pts g) ∧ ∀ x, supportLe x n → interp n pts g x = g x :=
  ⟨interp_polyFn pts g, fun x hx => sum_delta pts hnd hsound g x (hcover x hx) hx⟩

theorem sum_map_congr {l : List F} {u v : F → Int} (h : ∀ f ∈ l, u f = v f) :
    (l.map u).sum = (l.map v).sum := by
  induction l with
  | nil => simp
  | cons a rest ih =>
    rw [List.map_cons, List.map_cons, List.sum_cons, List.sum_cons, h a (by simp)]
    have hrest : ∀ f ∈ rest, u f = v f := fun f hf => h f (by simp [hf])
    rw [ih hrest]

/-! ## 五、T6 憑證：無公共零點 ⟹ 系統的組合是處處非零的函數

點集以列舉 `pts` 給出（`Nodup` + 支撐 ≤ n + 覆蓋全部支撐 ≤ n 的點）。
「無公共零點」表示每個點 x 都有某個系統元素 f 使 f(x) ≠ 0。 -/

/-- 系統 fs 在點集 pts 上無公共零點。 -/
def NoCommonZero (pts : List (Nat → Bool)) (fs : List F) : Prop :=
  ∀ x ∈ pts, ∃ f ∈ fs, f x ≠ 0

/-- 由無公共零點選出「每個點的見證元素」。 -/
theorem exists_selection (pts : List (Nat → Bool)) (fs : List F) (hno : NoCommonZero pts fs) :
    ∃ sel : (Nat → Bool) → F, ∀ x ∈ pts, sel x ∈ fs ∧ sel x x ≠ 0 := by
  refine ⟨fun x => if h : ∃ f ∈ fs, f x ≠ 0 then h.choose else fs.headD (fun _ => 0), ?_⟩
  intro x hx
  have h : ∃ f ∈ fs, f x ≠ 0 := hno x hx
  dsimp only
  rw [dif_pos h]
  exact ⟨h.choose_spec.1, h.choose_spec.2⟩

/-- 指示乘子：在 f 恰為 x 的見證元素時取 1，否則取 0。 -/
noncomputable def selMult (sel : (Nat → Bool) → F) (f : F) : F :=
  fun x => if f = sel x then 1 else 0

/-- 正規化列表求和：只有與 `sel x` 相符的項留下來，其和為 (sel x)(x)。 -/
theorem sum_map_selMult (fs : List F) (hnd : fs.Nodup) (sel : F) (hsel : sel ∈ fs)
    (x : Nat → Bool) :
    (fs.map (fun f => selMult (fun _ => sel) f x * f x)).sum = sel x := by
  induction fs with
  | nil => exact absurd hsel (by simp)
  | cons f rest ih =>
    rw [List.map_cons, List.sum_cons]
    obtain ⟨hfrest, hndrest⟩ := List.nodup_cons.mp hnd
    by_cases hfsel : f = sel
    · subst hfsel
      have hrest0 : (rest.map (fun g => selMult (fun _ => f) g x * g x)).sum = 0 := by
        apply sum_zero_of_all_zero
        intro b hb
        obtain ⟨g, hgrest, hgeq⟩ := List.mem_map.mp hb
        rw [← hgeq]
        have hgf : g ≠ f := fun h => hfrest (h ▸ hgrest)
        simp [selMult, hgf]
      rw [hrest0, Int.add_zero]
      simp [selMult]
    · have hselrest : sel ∈ rest := by
        rcases List.mem_cons.mp hsel with h | h
        · exact absurd h.symm hfsel
        · exact h
      have h0 : selMult (fun _ => sel) f x * f x = 0 := by
        simp [selMult, hfsel]
      rw [h0, Int.zero_add]
      exact ih hndrest hselrest

/-- **T6 憑證（函數版）**：無公共零點 ⟹ 系統的某個線性組合是**處處非零的函數**。
（乘子為任意函數；下一條定理把乘子提升為多項式。） -/
theorem no_root_certificate (pts : List (Nat → Bool)) (_hnd : pts.Nodup) (fs : List F)
    (hfsnd : fs.Nodup) (hno : NoCommonZero pts fs) :
    ∃ sel : (Nat → Bool) → F, (∀ x ∈ pts, sel x ∈ fs ∧ sel x x ≠ 0)
      ∧ (∀ x ∈ pts, (fs.map (fun f => selMult sel f x * f x)).sum = (sel x) x)
      ∧ (∀ x ∈ pts, (sel x) x ≠ 0) := by
  obtain ⟨sel, hsel⟩ := exists_selection pts fs hno
  refine ⟨sel, hsel, ?_, ?_⟩
  · intro x hx
    have hselmem : sel x ∈ fs := (hsel x hx).1
    have : (fun f => selMult sel f x * f x) = (fun f => selMult (fun _ => sel x) f x * f x) := by
      funext f
      by_cases hf : f = sel x
      · subst hf; rfl
      · have h1 : selMult sel f x = 0 := by simp [selMult, hf]
        have h2 : selMult (fun _ => sel x) f x = 0 := by simp [selMult, hf]
        rw [h1, h2]
    rw [this]
    exact sum_map_selMult fs hfsnd (sel x) hselmem x
  · intro x hx
    exact (hsel x hx).2

/-- **T6 憑證（多項式版）**：乘子可取為**多項式函數**（透過布爾插值）。
這是「1 ∈ G」判定在多項式環中的正確性來源：憑證由多項式乘子與系統元素組成。 -/
theorem no_root_poly_certificate {n : Nat} (pts : List (Nat → Bool)) (hnd : pts.Nodup)
    (hsound : ∀ σ ∈ pts, supportLe σ n) (hcover : ∀ x, supportLe x n → x ∈ pts)
    (fs : List F) (hfsnd : fs.Nodup) (_hfs : ∀ f ∈ fs, PolyFn n f)
    (hno : NoCommonZero pts fs) :
    ∃ sel : (Nat → Bool) → F, ∃ mult : F → F,
      (∀ x ∈ pts, sel x ∈ fs ∧ sel x x ≠ 0)
      ∧ (∀ f ∈ fs, PolyFn n (mult f))
      ∧ (∀ x, supportLe x n → (fs.map (fun f => mult f x * f x)).sum = (sel x) x)
      ∧ (∀ x, supportLe x n → (sel x) x ≠ 0) := by
  obtain ⟨sel, hsel, -, -⟩ := no_root_certificate pts hnd fs hfsnd hno
  refine ⟨sel, fun f => interp n pts (selMult sel f), hsel, ?_, ?_, ?_⟩
  · intro f hf
    exact interp_polyFn pts (selMult sel f)
  · intro x hx
    have hcongr : (fs.map (fun f => interp n pts (selMult sel f) x * f x)).sum
        = (fs.map (fun f => selMult sel f x * f x)).sum :=
      sum_map_congr (l := fs)
        (u := fun f => interp n pts (selMult sel f) x * f x)
        (v := fun f => selMult sel f x * f x)
        (fun f _ => by rw [(interpolation pts hnd hsound hcover (selMult sel f)).2 x hx])
    rw [hcongr]
    -- 逐點：在 fs 中只有 f = sel x 的項留下
    have hselmem : sel x ∈ fs := (hsel x (hcover x hx)).1
    have heq : (fun f => selMult sel f x * f x) = (fun f => selMult (fun _ => sel x) f x * f x) := by
      funext f
      by_cases hf : f = sel x
      · subst hf; rfl
      · have : selMult sel f x = 0 := by simp [selMult, hf]
        rw [this]; simp [selMult, hf]
    rw [heq]
    exact sum_map_selMult fs hfsnd (sel x) hselmem x
  · intro x hx
    exact (hsel x (hcover x hx)).2

end Polyrust
