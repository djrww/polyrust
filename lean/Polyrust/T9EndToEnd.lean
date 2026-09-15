/- # 端到端正確性（定理 T9）：迷你語言、約束編碼與代碼生成 round-trip

對應 `docs/THEOREMS.md` §11（T9）：

> 管線判定 SAT ⟺ 獨立型別檢查器接受；且 SAT 時生成的 Rust 代碼
> (a) 可被重新解析（round-trip），(b) 可被 rustc 編譯，(c) 語義保持。

Lean 側把這條「端到端」拆成**可完全機械檢查的五段**：

1. **型別檢查器** `check e τ`（Bool 值、結構遞歸）：Rust 側「獨立檢查器」的
   形式對應物。
2. **約束生成器** `genC`：每個節點給 0/1 **型別位元** `σ : Expr → Ty → Bool`，
   配上**規則方程**（多項式歸零）與 **one-hot**：
   - `num n`：`t_{e,i32} = 1`、`t_{e,bool} = 0`；
   - `add a b`：`t_{e,i32} = t_{a,i32}·t_{b,i32}`、`t_{e,bool} = 0`；
   - `eqb a b`：`t_{e,bool} = t_{a,i32}·t_{b,i32}`、`t_{e,i32} = 0`；
   - `ite c t f`：`t_{e,τ} = t_{c,bool}·t_{t,τ}·t_{f,τ}`（τ ∈ {i32, bool}）；
   - 每節點 one-hot：`t_{e,i32} + t_{e,bool} − 1 = 0`。
3. **雙向定理**：
   - `genC_sound`（T1 可靠性）：`check e τ = true` ⟹ 見證賦值 σ_D 是根；
   - `genC_complete`（T2 完備性）：任何 0/1 根逐型別等於檢查器的答案；
   - `root_implies_typable`（T6 判定）：根存在 ⟹ 可定型（one-hot 在此扮演
     「不可定型 ⟹ 1 ∈ 理想 ⟹ 無解」的代數角色）；
   - `typable_iff_root`：合併的「⟺」；`untypable_iff_no_root`：UNSAT 側。
4. **區域化推論**：`Occurs e' e` 出現關係 ⟹ 根在每個子表達式上單型
   （`isMonoAt_of_root`）——由 one-hot 直接推出。
5. **代碼生成 round-trip**（T9(a)）：`gen : Expr → List Tok`（前序線性化）與
   `parse : List Tok → Option (Expr × List Tok)`，證明 `parse (gen e ++ rest)`
   取回原 AST；以及 T7(b) 宏選臂的閘控退化律。

**邊界（說死）**：
- (b) rustc 編譯與 (c) 真實語義保持**不在 Lean 內**（依賴 Rust 編譯器與
  Rust 語義）；Lean 側證明的是「判定等價」與「生成碼可重解析」。(b)(c) 由
  Rust 側 demo/obligations 實測（`docs/THEOREMS.md` §12）。
- 位元為 Bool（0/1）；換成 𝔽_p 值並套用 L0 嵌入引理（`Polyrust.Embedding`）
  即得 𝔽_p 版本；本模組不重複該步驟。
- 求解器本身（T4 終止界、T5 S-多項式、T7(a) 規範性）見 `Polyrust.Squarefree`、
  `Polyrust.SPoly`、`Polyrust.Canonical`；本模組與之解耦。
- 定理強度：`check` 是**雙向**檢查器（沒有子型別/多型/強制轉換），所以
  「可定型」= 恰好一個型別；`check_exclusive` 把這一點也證了。 -/

import Polyrust.ClauseDuality
import Polyrust.Tactics

namespace Polyrust

/-! ## 一、迷你語言與型別檢查器 -/

/-- 型別：`i32` 與 `bool`。 -/
inductive Ty
  | i32
  | boolean
  deriving DecidableEq, Repr

/-- 迷你語言表達式。 -/
inductive Expr
  | num : Int → Expr
  | add : Expr → Expr → Expr
  | eqb : Expr → Expr → Expr
  | ite : Expr → Expr → Expr → Expr
  deriving DecidableEq, Repr

/-- 雙向型別檢查器：`check e τ` 表示「e 可被檢查為 τ」。 -/
def check : Expr → Ty → Bool
  | .num _, .i32 => true
  | .num _, _ => false
  | .add a b, .i32 => check a .i32 && check b .i32
  | .add _ _, _ => false
  | .eqb a b, .boolean => check a .i32 && check b .i32
  | .eqb _ _, _ => false
  | .ite c t f, τ => check c .boolean && check t τ && check f τ

/-- 可定型：存在 τ 使 `check e τ = true`。 -/
def Typable (e : Expr) : Prop := check e Ty.i32 = true ∨ check e Ty.boolean = true

/-- 單型性：一個節點至多有一個型別。 -/
theorem check_exclusive (e : Expr) : ¬ (check e Ty.i32 = true ∧ check e Ty.boolean = true) := by
  induction e with
  | num n => intro h; exact absurd h.2 (by simp [check])
  | add a b iha ihb => intro h; exact absurd h.2 (by simp [check])
  | eqb a b iha ihb => intro h; exact absurd h.1 (by simp [check])
  | ite c t f ihc iht ihf =>
    rintro ⟨h1, h2⟩
    simp only [check, Bool.and_eq_true_iff] at h1 h2
    exact iht ⟨h1.1.2, h2.1.2⟩

/-! ## 二、約束生成（型別位元 0/1 賦值上的多項式方程組）

每條約束都是**具名多項式**。這不是修辭：Lean 不接受無法在證明中
指名道姓的目標，匿名 lambda 會讓後續 `rw`/`rcases` 無從下手。 -/

/-- 型別位元賦值：每個節點、每個型別一個 0/1 位元。 -/
abbrev Sigma := Expr → Ty → Bool

/-- 節點 e 在型別 τ 上的位元，作為 ℤ 值。 -/
def tb (σ : Sigma) (e : Expr) (τ : Ty) : Int := bit (σ e τ)

/-- one-hot 約束：`t_{e,i32} + t_{e,bool} − 1 = 0`。 -/
def oneHot (σ : Sigma) (e : Expr) : Int := tb σ e .i32 + tb σ e .boolean - 1

/-- one-hot 約束（以節點為參數的形式，便於指名其為約束表成員）。 -/
def oneHotC (e : Expr) : Sigma → Int := fun σ => oneHot σ e

/-- `num n`：`t = 1`（型別 i32）。 -/
def cNumI (n : Int) : Sigma → Int := fun σ => tb σ (.num n) .i32 - 1
/-- `num n`：`t = 0`（型別 bool 被排除）。 -/
def cNumB (n : Int) : Sigma → Int := fun σ => tb σ (.num n) .boolean

/-- `add a b`：`t_{e,i32} = t_{a,i32}·t_{b,i32}`。 -/
def cAddI (a b : Expr) : Sigma → Int :=
  fun σ => tb σ (.add a b) .i32 - tb σ a .i32 * tb σ b .i32
/-- `add a b`：`t_{e,bool} = 0`（加法不產生布爾值）。 -/
def cAddB (a b : Expr) : Sigma → Int := fun σ => tb σ (.add a b) .boolean

/-- `eqb a b`：`t_{e,bool} = t_{a,i32}·t_{b,i32}`。 -/
def cEqbB (a b : Expr) : Sigma → Int :=
  fun σ => tb σ (.eqb a b) .boolean - tb σ a .i32 * tb σ b .i32
/-- `eqb a b`：`t_{e,i32} = 0`。 -/
def cEqbI (a b : Expr) : Sigma → Int := fun σ => tb σ (.eqb a b) .i32

/-- `ite c t f`：`t_{e,i32} = t_{c,bool}·t_{t,i32}·t_{f,i32}`。 -/
def cIteI (c t f : Expr) : Sigma → Int :=
  fun σ => tb σ (.ite c t f) .i32 - tb σ c .boolean * tb σ t .i32 * tb σ f .i32
/-- `ite c t f`：`t_{e,bool} = t_{c,bool}·t_{t,bool}·t_{f,bool}`。 -/
def cIteB (c t f : Expr) : Sigma → Int :=
  fun σ => tb σ (.ite c t f) .boolean - tb σ c .boolean * tb σ t .boolean * tb σ f .boolean

/-- 各節點的 one-hot 約束（具名）。 -/
def cNumH (n : Int) : Sigma → Int := oneHotC (.num n)
def cAddH (a b : Expr) : Sigma → Int := oneHotC (.add a b)
def cEqbH (a b : Expr) : Sigma → Int := oneHotC (.eqb a b)
def cIteH (c t f : Expr) : Sigma → Int := oneHotC (.ite c t f)

/-- 約束生成：規則方程 + one-hot。 -/
def genC : Expr → List (Sigma → Int)
  | .num n => [cNumI n, cNumB n, cNumH n]
  | .add a b => genC a ++ genC b ++ [cAddI a b, cAddB a b, cAddH a b]
  | .eqb a b => genC a ++ genC b ++ [cEqbB a b, cEqbI a b, cEqbH a b]
  | .ite c t f => genC c ++ genC t ++ genC f ++ [cIteI c t f, cIteB c t f, cIteH c t f]

/-- σ 是方程組的 0/1 根。 -/
def IsRoot (e : Expr) (σ : Sigma) : Prop := ∀ p ∈ genC e, p σ = 0

/-! ## 三、約束表成員關係（顯式構造，不依賴 `simp` 的嵌套順序） -/

theorem genC_num_oneHot (n : Int) : cNumH n ∈ genC (.num n) :=
  List.mem_cons_of_mem _ (List.mem_cons_of_mem _ List.mem_cons_self)

theorem genC_add_oneHot (a b : Expr) : cAddH a b ∈ genC (.add a b) :=
  List.mem_append.mpr
    (Or.inr (List.mem_cons_of_mem _ (List.mem_cons_of_mem _ List.mem_cons_self)))

theorem genC_eqb_oneHot (a b : Expr) : cEqbH a b ∈ genC (.eqb a b) :=
  List.mem_append.mpr
    (Or.inr (List.mem_cons_of_mem _ (List.mem_cons_of_mem _ List.mem_cons_self)))

theorem genC_ite_oneHot (c t f : Expr) : cIteH c t f ∈ genC (.ite c t f) :=
  List.mem_append.mpr
    (Or.inr (List.mem_cons_of_mem _ (List.mem_cons_of_mem _ List.mem_cons_self)))

/-- 子表達式的約束都在父節點的約束表裡（加法，左）。 -/
theorem mem_genC_of_left {p : Sigma → Int} {a b : Expr} (hp : p ∈ genC a) :
    p ∈ genC (.add a b) :=
  List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))

/-- 子表達式的約束都在父節點的約束表裡（加法，右）。 -/
theorem mem_genC_of_right {p : Sigma → Int} {a b : Expr} (hp : p ∈ genC b) :
    p ∈ genC (.add a b) :=
  List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))

theorem mem_genC_eqb_left {p : Sigma → Int} {a b : Expr} (hp : p ∈ genC a) :
    p ∈ genC (.eqb a b) :=
  List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))

theorem mem_genC_eqb_right {p : Sigma → Int} {a b : Expr} (hp : p ∈ genC b) :
    p ∈ genC (.eqb a b) :=
  List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))

theorem mem_genC_ite_first {p : Sigma → Int} {c t f : Expr} (hp : p ∈ genC c) :
    p ∈ genC (.ite c t f) :=
  List.mem_append.mpr
    (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl hp)))))

theorem mem_genC_ite_second {p : Sigma → Int} {c t f : Expr} (hp : p ∈ genC t) :
    p ∈ genC (.ite c t f) :=
  List.mem_append.mpr
    (Or.inl (List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inr hp)))))

theorem mem_genC_ite_third {p : Sigma → Int} {c t f : Expr} (hp : p ∈ genC f) :
    p ∈ genC (.ite c t f) :=
  List.mem_append.mpr
    (Or.inl (List.mem_append.mpr (Or.inr hp)))

/-! ## 四、約束表的成員分解（用於把「∀ p ∈ 表」拆成逐條規則） -/

theorem mem_genC_num {p : Sigma → Int} {n : Int} (hp : p ∈ genC (.num n)) :
    p = cNumI n ∨ p = cNumB n ∨ p = cNumH n := by
  rcases List.mem_cons.mp hp with h | h
  · exact Or.inl h
  · rcases List.mem_cons.mp h with h | h
    · exact Or.inr (Or.inl h)
    · rw [List.mem_singleton] at h
      exact Or.inr (Or.inr h)

theorem mem_genC_add {p : Sigma → Int} {a b : Expr} (hp : p ∈ genC (.add a b)) :
    p ∈ genC a ∨ p ∈ genC b ∨ p = cAddI a b ∨ p = cAddB a b ∨ p = cAddH a b := by
  rcases List.mem_append.mp hp with h | h
  · rcases List.mem_append.mp h with h | h
    · exact Or.inl h
    · exact Or.inr (Or.inl h)
  · rcases List.mem_cons.mp h with h | h
    · exact Or.inr (Or.inr (Or.inl h))
    · rcases List.mem_cons.mp h with h | h
      · exact Or.inr (Or.inr (Or.inr (Or.inl h)))
      · rw [List.mem_singleton] at h
        exact Or.inr (Or.inr (Or.inr (Or.inr h)))

theorem mem_genC_eqb {p : Sigma → Int} {a b : Expr} (hp : p ∈ genC (.eqb a b)) :
    p ∈ genC a ∨ p ∈ genC b ∨ p = cEqbB a b ∨ p = cEqbI a b ∨ p = cEqbH a b := by
  rcases List.mem_append.mp hp with h | h
  · rcases List.mem_append.mp h with h | h
    · exact Or.inl h
    · exact Or.inr (Or.inl h)
  · rcases List.mem_cons.mp h with h | h
    · exact Or.inr (Or.inr (Or.inl h))
    · rcases List.mem_cons.mp h with h | h
      · exact Or.inr (Or.inr (Or.inr (Or.inl h)))
      · rw [List.mem_singleton] at h
        exact Or.inr (Or.inr (Or.inr (Or.inr h)))

theorem mem_genC_ite {p : Sigma → Int} {c t f : Expr} (hp : p ∈ genC (.ite c t f)) :
    p ∈ genC c ∨ p ∈ genC t ∨ p ∈ genC f ∨
      p = cIteI c t f ∨ p = cIteB c t f ∨ p = cIteH c t f := by
  rcases List.mem_append.mp hp with h | h
  · rcases List.mem_append.mp h with h | h
    · rcases List.mem_append.mp h with h | h
      · exact Or.inl h
      · exact Or.inr (Or.inl h)
    · exact Or.inr (Or.inr (Or.inl h))
  · rcases List.mem_cons.mp h with h | h
    · exact Or.inr (Or.inr (Or.inr (Or.inl h)))
    · rcases List.mem_cons.mp h with h | h
      · exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inl h))))
      · rw [List.mem_singleton] at h
        exact Or.inr (Or.inr (Or.inr (Or.inr (Or.inr h))))

/-! ## 五、出現關係與「根在子表達式上單型」 -/

/-- 出現關係：`Occurs e' e` 表示 e' 是 e（的語法）子表達式，含自身。 -/
inductive Occurs : Expr → Expr → Prop
  | refl (e : Expr) : Occurs e e
  | add_l {e a b : Expr} : Occurs e a → Occurs e (.add a b)
  | add_r {e a b : Expr} : Occurs e b → Occurs e (.add a b)
  | eqb_l {e a b : Expr} : Occurs e a → Occurs e (.eqb a b)
  | eqb_r {e a b : Expr} : Occurs e b → Occurs e (.eqb a b)
  | ite_c {e c t f : Expr} : Occurs e c → Occurs e (.ite c t f)
  | ite_t {e c t f : Expr} : Occurs e t → Occurs e (.ite c t f)
  | ite_f {e c t f : Expr} : Occurs e f → Occurs e (.ite c t f)

/-- **出現者的 one-hot 在約束表裡**。 -/
theorem occurs_oneHot_mem_genC : ∀ {e' e : Expr}, Occurs e' e → oneHotC e' ∈ genC e
  | _, _, .refl e => by
    cases e with
    | num n => exact genC_num_oneHot n
    | add a b => exact genC_add_oneHot a b
    | eqb a b => exact genC_eqb_oneHot a b
    | ite c t f => exact genC_ite_oneHot c t f
  | _, _, .add_l h => mem_genC_of_left (occurs_oneHot_mem_genC h)
  | _, _, .add_r h => mem_genC_of_right (occurs_oneHot_mem_genC h)
  | _, _, .eqb_l h => mem_genC_eqb_left (occurs_oneHot_mem_genC h)
  | _, _, .eqb_r h => mem_genC_eqb_right (occurs_oneHot_mem_genC h)
  | _, _, .ite_c h => mem_genC_ite_first (occurs_oneHot_mem_genC h)
  | _, _, .ite_t h => mem_genC_ite_second (occurs_oneHot_mem_genC h)
  | _, _, .ite_f h => mem_genC_ite_third (occurs_oneHot_mem_genC h)

/-- 節點層級的單型性。 -/
def IsMonoAt (σ : Sigma) (e : Expr) : Prop :=
  ∀ τ τ', τ ≠ τ' → ¬ (σ e τ = true ∧ σ e τ' = true)

/-- **根在每個子表達式上單型**：one-hot 的直接推論（型別唯一性的區域化版）。 -/
theorem isMonoAt_of_root {e e' : Expr} {σ : Sigma} (hroot : IsRoot e σ)
    (h : Occurs e' e) : IsMonoAt σ e' := by
  intro τ τ' hne ⟨h1, h2⟩
  have hmem := occurs_oneHot_mem_genC h
  have hz : oneHot σ e' = 0 := hroot (oneHotC e') hmem
  cases τ <;> cases τ' <;> simp_all [oneHot, tb, bit]

/-- 衍生：根在其自身節點上單型。 -/
theorem isMonoAt_self_of_root {e : Expr} {σ : Sigma} (hroot : IsRoot e σ) :
    IsMonoAt σ e :=
  isMonoAt_of_root hroot (.refl e)

/-! ## 六、單型性的兩個方向（`ite` 規則在錯型別分支上要靠它） -/

/-- 由「可為 bool」推出「不可為 i32」。 -/
theorem check_i32_false_of_bool {e : Expr} (h : check e Ty.boolean = true) :
    check e Ty.i32 = false := by
  cases hh : check e Ty.i32 with
  | false => rfl
  | true => exact absurd ⟨hh, h⟩ (check_exclusive e)

/-- 由「可為 i32」推出「不可為 bool」。 -/
theorem check_bool_false_of_i32 {e : Expr} (h : check e Ty.i32 = true) :
    check e Ty.boolean = false := by
  cases hh : check e Ty.boolean with
  | false => rfl
  | true => exact absurd ⟨h, hh⟩ (check_exclusive e)

/-! ## 七、位元算術引理 -/

theorem bit_and (a b : Bool) : bit (a && b) = bit a * bit b := by
  cases a <;> cases b <;> rfl

theorem bit_injective {a b : Bool} (h : bit a = bit b) : a = b := by
  cases a <;> cases b <;> simp [bit] at h ⊢

theorem bit_eq_zero_iff {b : Bool} : bit b = 0 ↔ b = false := by
  cases b <;> simp [bit]

theorem bit_eq_one_iff {b : Bool} : bit b = 1 ↔ b = true := by
  cases b <;> simp [bit]

/-! ## 八、T1（可靠性）：可定型 ⟹ 見證賦值是根 -/

/-- 見證賦值 σ_D：節點 e' 在位元 τ' 上的值就是檢查器的答案 `check e' τ'`。 -/
def witness : Sigma := fun e' τ => check e' τ

/-- **T1 主定理**：若 `check e τ = true`，則見證賦值 σ_D 滿足全部約束，
且在 (e, τ) 上取值為真——編碼不拒絕任何可定型程序。 -/
theorem genC_sound : ∀ (e : Expr) (τ : Ty), check e τ = true → IsRoot e witness ∧ witness e τ = true := by
  intro e
  induction e with
  | num n =>
    intro τ hτ
    refine ⟨?_, hτ⟩
    intro p hp
    rcases mem_genC_num hp with h | h | h
    · rw [h]; simp [cNumI, tb, witness, check, bit]
    · rw [h]; simp [cNumB, tb, witness, check, bit]
    · rw [h]; simp [cNumH, oneHotC, oneHot, tb, witness, check, bit]
  | add a b iha ihb =>
    intro τ hτ
    have hsplit : τ = Ty.i32 ∧ check a Ty.i32 = true ∧ check b Ty.i32 = true := by
      cases τ
      · rw [check, Bool.and_eq_true_iff] at hτ
        exact ⟨rfl, hτ.1, hτ.2⟩
      · simp [check] at hτ
    obtain ⟨rfl, ha, hb⟩ := hsplit
    obtain ⟨hra, -⟩ := iha Ty.i32 ha
    obtain ⟨hrb, -⟩ := ihb Ty.i32 hb
    refine ⟨?_, hτ⟩
    intro p hp
    rcases mem_genC_add hp with h | h | h | h | h
    · exact hra p h
    · exact hrb p h
    · rw [h]; simp [cAddI, tb, witness, check, ha, hb, bit]
    · rw [h]; simp [cAddB, tb, witness, check, bit]
    · rw [h]; simp [cAddH, oneHotC, oneHot, tb, witness, check, ha, hb, bit]
  | eqb a b iha ihb =>
    intro τ hτ
    have hsplit : τ = Ty.boolean ∧ check a Ty.i32 = true ∧ check b Ty.i32 = true := by
      cases τ
      · simp [check] at hτ
      · rw [check, Bool.and_eq_true_iff] at hτ
        exact ⟨rfl, hτ.1, hτ.2⟩
    obtain ⟨rfl, ha, hb⟩ := hsplit
    obtain ⟨hra, -⟩ := iha Ty.i32 ha
    obtain ⟨hrb, -⟩ := ihb Ty.i32 hb
    refine ⟨?_, hτ⟩
    intro p hp
    rcases mem_genC_eqb hp with h | h | h | h | h
    · exact hra p h
    · exact hrb p h
    · rw [h]; simp [cEqbB, tb, witness, check, ha, hb, bit]
    · rw [h]; simp [cEqbI, tb, witness, check, bit]
    · rw [h]; simp [cEqbH, oneHotC, oneHot, tb, witness, check, ha, hb, bit]
  | ite c t f ihc iht ihf =>
    intro τ hτ
    have hsplit : check c Ty.boolean = true ∧ check t τ = true ∧ check f τ = true := by
      simp only [check, Bool.and_eq_true_iff] at hτ
      exact ⟨hτ.1.1, hτ.1.2, hτ.2⟩
    obtain ⟨hc, ht, hf⟩ := hsplit
    obtain ⟨hrc, -⟩ := ihc Ty.boolean hc
    obtain ⟨hrt, -⟩ := iht τ ht
    obtain ⟨hrf, -⟩ := ihf τ hf
    refine ⟨?_, hτ⟩
    intro p hp
    rcases mem_genC_ite hp with h | h | h | h | h | h
    · exact hrc p h
    · exact hrt p h
    · exact hrf p h
    · rw [h]
      cases τ
      · simp [cIteI, tb, witness, check, hc, ht, hf, bit]
      · have ht' : check t Ty.i32 = false := check_i32_false_of_bool ht
        have hf' : check f Ty.i32 = false := check_i32_false_of_bool hf
        simp [cIteI, tb, witness, check, hc, ht', hf', bit]
    · rw [h]
      cases τ
      · have ht' : check t Ty.boolean = false := check_bool_false_of_i32 ht
        have hf' : check f Ty.boolean = false := check_bool_false_of_i32 hf
        simp [cIteB, tb, witness, check, hc, ht', hf', bit]
      · simp [cIteB, tb, witness, check, hc, ht, hf, bit]
    · rw [h]
      cases τ
      · have ht' : check t Ty.boolean = false := check_bool_false_of_i32 ht
        have hf' : check f Ty.boolean = false := check_bool_false_of_i32 hf
        simp [cIteH, oneHotC, oneHot, tb, witness, check, hc, ht, hf, ht', hf', bit]
      · have ht' : check t Ty.i32 = false := check_i32_false_of_bool ht
        have hf' : check f Ty.i32 = false := check_i32_false_of_bool hf
        simp [cIteH, oneHotC, oneHot, tb, witness, check, hc, ht, hf, ht', hf', bit]

/-! ## 九、T2（完備性）：任何根都逐型別解碼為檢查器的答案 -/

/-- **T2 主定理**：方程組的任一 0/1 根，其位元與檢查器判定逐型別一致
（`σ e τ = check e τ`）：編碼既不遺漏也不誇大推導能力。 -/
theorem genC_complete : ∀ (e : Expr) (σ : Sigma), IsRoot e σ → ∀ τ, σ e τ = check e τ := by
  intro e
  induction e with
  | num n =>
    intro σ hroot τ
    have h1 : bit (σ (.num n) .i32) = 1 := by
      have h := hroot (cNumI n) (by simp [genC])
      simp only [cNumI, tb] at h
      omega
    have h2 : bit (σ (.num n) .boolean) = 0 := by
      have h := hroot (cNumB n) (by simp [genC])
      simp only [cNumB, tb] at h
      exact h
    have hi : σ (.num n) Ty.i32 = true := bit_eq_one_iff.mp h1
    have hb : σ (.num n) Ty.boolean = false := bit_eq_zero_iff.mp h2
    cases τ <;> simp [check, hi, hb]
  | add a b iha ihb =>
    intro σ hroot τ
    have hmem : ∀ p ∈ genC a, p σ = 0 := fun p hp => hroot p (mem_genC_of_left hp)
    have hmemb : ∀ p ∈ genC b, p σ = 0 := fun p hp => hroot p (mem_genC_of_right hp)
    have hrule : bit (σ (.add a b) .i32) = bit (σ a .i32) * bit (σ b .i32) := by
      have h := hroot (cAddI a b) (by simp [genC])
      simp only [cAddI, tb] at h
      omega
    have hb0 : σ (.add a b) Ty.boolean = false := by
      have h := hroot (cAddB a b) (by simp [genC])
      simp only [cAddB, tb] at h
      exact bit_eq_zero_iff.mp h
    have ha := iha σ hmem Ty.i32
    have hbb := ihb σ hmemb Ty.i32
    cases τ
    · refine bit_injective ?_
      simp only [check, bit_and]
      rw [hrule, ha, hbb]
    · rw [hb0]; rfl
  | eqb a b iha ihb =>
    intro σ hroot τ
    have hmem : ∀ p ∈ genC a, p σ = 0 := fun p hp => hroot p (mem_genC_eqb_left hp)
    have hmemb : ∀ p ∈ genC b, p σ = 0 := fun p hp => hroot p (mem_genC_eqb_right hp)
    have hrule : bit (σ (.eqb a b) .boolean) = bit (σ a .i32) * bit (σ b .i32) := by
      have h := hroot (cEqbB a b) (by simp [genC])
      simp only [cEqbB, tb] at h
      omega
    have hi0 : σ (.eqb a b) Ty.i32 = false := by
      have h := hroot (cEqbI a b) (by simp [genC])
      simp only [cEqbI, tb] at h
      exact bit_eq_zero_iff.mp h
    have ha := iha σ hmem Ty.i32
    have hbb := ihb σ hmemb Ty.i32
    cases τ
    · rw [hi0]; rfl
    · refine bit_injective ?_
      simp only [check, bit_and]
      rw [hrule, ha, hbb]
  | ite c t f ihc iht ihf =>
    intro σ hroot τ
    have hmemc : ∀ p ∈ genC c, p σ = 0 := fun p hp => hroot p (mem_genC_ite_first hp)
    have hmemt : ∀ p ∈ genC t, p σ = 0 := fun p hp => hroot p (mem_genC_ite_second hp)
    have hmemf : ∀ p ∈ genC f, p σ = 0 := fun p hp => hroot p (mem_genC_ite_third hp)
    have hc : ∀ τ', σ c τ' = check c τ' := ihc σ hmemc
    have ht : ∀ τ', σ t τ' = check t τ' := iht σ hmemt
    have hf : ∀ τ', σ f τ' = check f τ' := ihf σ hmemf
    cases τ
    · have hrule : bit (σ (.ite c t f) .i32)
          = bit (σ c .boolean) * bit (σ t .i32) * bit (σ f .i32) := by
        have h := hroot (cIteI c t f) (by simp [genC])
        simp only [cIteI, tb] at h
        omega
      refine bit_injective ?_
      simp only [check, bit_and]
      rw [hrule, hc Ty.boolean, ht Ty.i32, hf Ty.i32, Int.mul_assoc]
    · have hrule : bit (σ (.ite c t f) .boolean)
          = bit (σ c .boolean) * bit (σ t .boolean) * bit (σ f .boolean) := by
        have h := hroot (cIteB c t f) (by simp [genC])
        simp only [cIteB, tb] at h
        omega
      refine bit_injective ?_
      simp only [check, bit_and]
      rw [hrule, hc Ty.boolean, ht Ty.boolean, hf Ty.boolean, Int.mul_assoc]

/-! ## 十、T6（判定）：根存在 ⟺ 可定型 -/

/-- **T6 主定理**：方程組有 0/1 根 ⟹ 程序可定型。
one-hot 約束在此扮演「不可定型 ⟹ 系統無解（代數側的 1 ∈ 理想）」的角色。 -/
theorem root_implies_typable {e : Expr} {σ : Sigma} (hroot : IsRoot e σ) : Typable e := by
  have h := genC_complete e σ hroot
  have hone : bit (σ e Ty.i32) + bit (σ e Ty.boolean) - 1 = 0 := by
    match e with
    | .num n =>
      have hh := hroot (cNumH n) (by simp [genC]); simpa [cNumH, oneHotC, oneHot, tb] using hh
    | .add a b =>
      have hh := hroot (cAddH a b) (by simp [genC]); simpa [cAddH, oneHotC, oneHot, tb] using hh
    | .eqb a b =>
      have hh := hroot (cEqbH a b) (by simp [genC]); simpa [cEqbH, oneHotC, oneHot, tb] using hh
    | .ite c t f =>
      have hh := hroot (cIteH c t f) (by simp [genC]); simpa [cIteH, oneHotC, oneHot, tb] using hh
  rw [h Ty.i32, h Ty.boolean] at hone
  by_cases hi : check e Ty.i32 = true
  · exact Or.inl hi
  · by_cases hb : check e Ty.boolean = true
    · exact Or.inr hb
    · exfalso
      have hi' : check e Ty.i32 = false := by cases hh : check e Ty.i32 <;> simp_all
      have hb' : check e Ty.boolean = false := by cases hh : check e Ty.boolean <;> simp_all
      rw [hi', hb'] at hone
      simp [bit] at hone

/-- **T9 判定等價**：管線的「代數可解」⟺ 獨立型別檢查器的「接受」。 -/
theorem typable_iff_root (e : Expr) : Typable e ↔ ∃ σ : Sigma, IsRoot e σ := by
  constructor
  · rintro (h | h)
    · exact ⟨witness, (genC_sound e Ty.i32 h).1⟩
    · exact ⟨witness, (genC_sound e Ty.boolean h).1⟩
  · rintro ⟨σ, hroot⟩
    exact root_implies_typable hroot

/-- 衍生：不可定型 ⟺ 方程組無 0/1 根（UNSAT 側的完整陳述）。 -/
theorem untypable_iff_no_root (e : Expr) : ¬ Typable e ↔ ¬ ∃ σ : Sigma, IsRoot e σ := by
  constructor
  · intro h ⟨σ, hroot⟩
    exact h (root_implies_typable hroot)
  · intro h ht
    exact h ((typable_iff_root e).mp ht)

/-- 衍生：根的型別位元由檢查器唯一決定。 -/
theorem root_bit_determined {e : Expr} {σ : Sigma} (hroot : IsRoot e σ) (τ : Ty) :
    σ e τ = check e τ :=
  genC_complete e σ hroot τ

/-- 衍生：型別檢查通過 ⟹ 存在滿足約束的賦值（未定型不收斂的反面）。 -/
theorem typable_exists_root {e : Expr} (h : Typable e) : ∃ σ : Sigma, IsRoot e σ :=
  (typable_iff_root e).mp h

/-! ## 十一、T9(a)：代碼生成與重新解析（round-trip） -/

/-- 線性化指令（前序）。 -/
inductive Tok
  | unit
  | num : Int → Tok
  | add
  | eqb
  | ite
  deriving DecidableEq, Repr

/-- 代碼生成：前序線性化。 -/
def gen : Expr → List Tok
  | .num n => [.num n]
  | .add a b => .add :: (gen a ++ gen b)
  | .eqb a b => .eqb :: (gen a ++ gen b)
  | .ite c t f => .ite :: (gen c ++ gen t ++ gen f)

/-- 解析器（帶燃料）：由前序指令串還原表達式（與剩餘輸入）。
燃料使結構遞歸成為可能——**這不是裝飾**：無燃料的相互遞歸無法通過 Lean 的
終止檢查，而「跳過一個子樹」本來就不是結構遞歸。 -/
def parseFuel : Nat → List Tok → Option (Expr × List Tok)
  | 0, _ => none
  | _ + 1, [] => none
  | _ + 1, .unit :: rest => some (.num 0, rest)
  | _ + 1, .num n :: rest => some (.num n, rest)
  | n + 1, .add :: rest =>
    match parseFuel n rest with
    | none => none
    | some (a, rest₁) =>
      match parseFuel n rest₁ with
      | none => none
      | some (b, rest₂) => some (.add a b, rest₂)
  | n + 1, .eqb :: rest =>
    match parseFuel n rest with
    | none => none
    | some (a, rest₁) =>
      match parseFuel n rest₁ with
      | none => none
      | some (b, rest₂) => some (.eqb a b, rest₂)
  | n + 1, .ite :: rest =>
    match parseFuel n rest with
    | none => none
    | some (c, rest₁) =>
      match parseFuel n rest₁ with
      | none => none
      | some (t, rest₂) =>
        match parseFuel n rest₂ with
        | none => none
        | some (f, rest₃) => some (.ite c t f, rest₃)

/-- 公開解析器：燃料 = 輸入長度 + 1。 -/
def parse (l : List Tok) : Option (Expr × List Tok) := parseFuel (l.length + 1) l

/-- 生成碼長度（結構遞歸，供「燃料夠多」引理使用）。 -/
def sizeT : Expr → Nat
  | .num _ => 1
  | .add a b => 1 + (sizeT a + sizeT b)
  | .eqb a b => 1 + (sizeT a + sizeT b)
  | .ite c t f => 1 + (sizeT c + sizeT t + sizeT f)

/-- `gen` 的長度就是 `sizeT`（線性化的正確性前提）。 -/
theorem gen_length (e : Expr) : (gen e).length = sizeT e := by
  induction e with
  | num n => rfl
  | add a b iha ihb =>
    simp only [gen, List.length_cons, List.length_append, sizeT, iha, ihb]
    omega
  | eqb a b iha ihb =>
    simp only [gen, List.length_cons, List.length_append, sizeT, iha, ihb]
    omega
  | ite c t f ihc iht ihf =>
    simp only [gen, List.length_cons, List.length_append, sizeT, ihc, iht, ihf]
    omega

/-- **燃料夠多時，生成碼必定解析回原表達式**（round-trip 的核心）。 -/
theorem parseFuel_gen : ∀ (e : Expr) (rest : List Tok) (n : Nat),
    (gen e ++ rest).length ≤ n → parseFuel n (gen e ++ rest) = some (e, rest) := by
  intro e rest n
  induction n generalizing e rest with
  | zero =>
    intro h
    exfalso
    have hpos : 1 ≤ (gen e).length := by cases e <;> simp [gen]
    simp only [List.length_append] at h
    omega
  | succ m ih =>
    intro h
    have hlen : sizeT e + rest.length ≤ m + 1 := by
      have h' : (gen e).length + rest.length ≤ m + 1 := by
        simpa only [List.length_append] using h
      simpa only [gen_length] using h'
    cases e with
    | num k =>
      show parseFuel (Nat.succ m) (Tok.num k :: rest) = some (Expr.num k, rest)
      rfl
    | add a b =>
      have h1 : sizeT a + (sizeT b + rest.length) ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h2 : sizeT b + rest.length ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h1' : (gen a ++ (gen b ++ rest)).length ≤ m := by
        have heq : (gen a ++ (gen b ++ rest)).length = sizeT a + (sizeT b + rest.length) := by
          simp only [List.length_append, gen_length]
        omega
      have h2' : (gen b ++ rest).length ≤ m := by
        have heq : (gen b ++ rest).length = sizeT b + rest.length := by
          simp only [List.length_append, gen_length]
        omega
      have ha := ih a (gen b ++ rest) h1'
      have hb := ih b rest h2'
      simp only [gen, List.cons_append, List.append_assoc]
      show parseFuel (Nat.succ m) (Tok.add :: (gen a ++ (gen b ++ rest))) =
        some (Expr.add a b, rest)
      simp only [parseFuel]
      rw [ha]
      dsimp only
      rw [hb]
    | eqb a b =>
      have h1 : sizeT a + (sizeT b + rest.length) ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h2 : sizeT b + rest.length ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h1' : (gen a ++ (gen b ++ rest)).length ≤ m := by
        have heq : (gen a ++ (gen b ++ rest)).length = sizeT a + (sizeT b + rest.length) := by
          simp only [List.length_append, gen_length]
        omega
      have h2' : (gen b ++ rest).length ≤ m := by
        have heq : (gen b ++ rest).length = sizeT b + rest.length := by
          simp only [List.length_append, gen_length]
        omega
      have ha := ih a (gen b ++ rest) h1'
      have hb := ih b rest h2'
      simp only [gen, List.cons_append, List.append_assoc]
      show parseFuel (Nat.succ m) (Tok.eqb :: (gen a ++ (gen b ++ rest))) =
        some (Expr.eqb a b, rest)
      simp only [parseFuel]
      rw [ha]
      dsimp only
      rw [hb]
    | ite c t f =>
      have h1 : sizeT c + (sizeT t + (sizeT f + rest.length)) ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h2 : sizeT t + (sizeT f + rest.length) ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h3 : sizeT f + rest.length ≤ m := by
        simp only [sizeT] at hlen
        omega
      have h1' : (gen c ++ (gen t ++ (gen f ++ rest))).length ≤ m := by
        have heq : (gen c ++ (gen t ++ (gen f ++ rest))).length
            = sizeT c + (sizeT t + (sizeT f + rest.length)) := by
          simp only [List.length_append, gen_length]
        omega
      have h2' : (gen t ++ (gen f ++ rest)).length ≤ m := by
        have heq : (gen t ++ (gen f ++ rest)).length = sizeT t + (sizeT f + rest.length) := by
          simp only [List.length_append, gen_length]
        omega
      have h3' : (gen f ++ rest).length ≤ m := by
        have heq : (gen f ++ rest).length = sizeT f + rest.length := by
          simp only [List.length_append, gen_length]
        omega
      have hc := ih c (gen t ++ (gen f ++ rest)) h1'
      have ht := ih t (gen f ++ rest) h2'
      have hf := ih f rest h3'
      simp only [gen, List.cons_append, List.append_assoc]
      show parseFuel (Nat.succ m) (Tok.ite :: (gen c ++ (gen t ++ (gen f ++ rest)))) =
        some (Expr.ite c t f, rest)
      simp only [parseFuel]
      rw [hc]
      dsimp only
      rw [ht]
      dsimp only
      rw [hf]

/-- **T9(a) round-trip**：生成碼接在任何輸入之前，解析必定取回原表達式，
且剩餘輸入不變——生成的代碼**必定可被重新解析**。 -/
theorem parse_gen (e : Expr) (rest : List Tok) : parse (gen e ++ rest) = some (e, rest) :=
  parseFuel_gen e rest ((gen e ++ rest).length + 1) (by omega)

/-- 衍生：可定型程序的生成碼可完整解析為同一 AST（端到端版本）。 -/
theorem typable_gen_parses (e : Expr) (τ : Ty) (h : check e τ = true) :
    ∃ e' : Expr, parse (gen e) = some (e', []) ∧ e' = e ∧ check e' τ = true :=
  ⟨e, by simpa using parse_gen e [], rfl, h⟩

/-- 衍生：round-trip 的「型別保持」——生成碼解析回的 AST 與原 AST 同型。 -/
theorem parse_gen_check (e : Expr) (τ : Ty) :
    check e τ = true → ∃ e' : Expr, parse (gen e) = some (e', []) ∧ check e' τ = true := by
  intro h
  exact ⟨e, by simpa using parse_gen e [], h⟩

/-! ## 十二、T7(b)：宏選臂的閘控退化律 -/

/-- 臂位元閘控：未選臂的約束乘 0 而消失、選中的臂乘 1 而保留。 -/
theorem arm_gating (a : Bool) (p : Sigma → Int) (σ : Sigma) :
    bit a * p σ = if a then p σ else 0 := by
  cases a <;> simp [bit]

/-- 衍生：以臂位元 a 閘控的系統，在 a = true 時與原系統同解。 -/
theorem arm_gating_root (a : Bool) (h : a = true) (p : Sigma → Int) (σ : Sigma) :
    bit a * p σ = 0 ↔ p σ = 0 := by
  rw [arm_gating, h]
  simp

/-- 衍生：a = false 時所有被閘控的約束自動滿足（未選臂不約束）。 -/
theorem arm_gating_void (p : Sigma → Int) (σ : Sigma) :
    bit (false : Bool) * p σ = 0 := by
  simp [bit]

end Polyrust
