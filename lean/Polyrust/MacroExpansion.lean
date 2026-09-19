-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # 宏展開是求值同態（定理 T7(b) 的語法層形式化）

`docs/THEOREMS.md` §9（T7(b)）：

> 展開即把臂模板 T_k 的語法變量替換為調用點節點——這正是多項式環的
> 求值同態 φ（模板變量 ↦ 節點變數）。選臂系統 = φ 逐臂實例化 + 臂位元閘控；
> a_k = 1 時其餘臂約束消失，系統同構於展開程序約束 ⇒ 可解性相同。
> 錯臂強制：型別錯配的模板在 tie 方程與規則方程聯立下產生 1。

Lean 側把「展開」形式化為**帶變量的模板語法上的代入**，並證明：

1. `subst_id`／`subst_comp`／`expand_comp`：代入的恆等律、集合律，以及
   `expand σ (subst τ e) = expand (σ ∘ τ) e`——**展開是同態**的等式形式。
2. `mem_vars_subst`：代入不引入新的自由變量（展開只能引用調用點已有的東西）。
3. `checkCtx_det`／`checkCtx_i32_false_of_bool`／`checkCtx_bool_false_of_i32`：
   模板檢查器的**型別唯一性**（一元一型）。
4. `checkCtx_expand`（**正確臂**）：模板在模板上下文 Δ 下可定型，且展開把 Δ
   宣告的每個變量送到可定型且**同型**的調用點表達式，則展開結果可定型。
5. `check_expand_reflect`（**反射性/錯臂**）：反過來，展開結果可定型 ⟹ 被用到
   的每個模板變量的代入表達式本身可定型。展開不能憑空造出可定型性。
6. `wrong_arm_untypable`（T7(b) 判定面）：把要求 i32 的臂代入 bool 型表達式
   （例如 `x == y` 的結果），展開結果**必不可定型**。
7. `wrong_arm_no_root`：配合 `typable_iff_root`，錯臂展開的約束方程組**無 0/1
   根**——這正是「錯臂系統 1 ∈ G」在前端（型別側）的等價刻畫。

**邊界（說死）**：
- 本模組處理**型別層**（哪個臂可被選中、展開是否可定型）。「1 ∈ 理想」的具體
  代數見證見 `Polyrust.MicroInstance`（`T7_wrong_arm_unsat`）與
  `Polyrust.T9EndToEnd`（`arm_gating`、`arm_gating_void`）；一般情形需要
  Nullstellensatz（本倉庫不含根式理想理論），故不在此宣稱等價。
- 模板變量的型別由上下文 Δ 宣告；調用點表達式用 T9 的 `check`（無變量版本）
  檢查。上下文是單型的（無多型、無子型別、無隱式轉換）——這是 Mini-Rust
  與本形式化的共同邊界。 -/

import Polyrust.T9EndToEnd

namespace Polyrust

/-! ## 一、模板語法、上下文、檢查器、代入 -/

/-- 帶變量的模板表達式：T9 的 `Expr` 加一個變量構造子。 -/
inductive VExpr
  | var : Nat → VExpr
  | num : Int → VExpr
  | add : VExpr → VExpr → VExpr
  | eqb : VExpr → VExpr → VExpr
  | ite : VExpr → VExpr → VExpr → VExpr
  deriving DecidableEq, Repr

/-- 模板上下文：變量 → 型別（`none` = 未宣告）。 -/
abbrev Ctx := Nat → Option Ty

/-- 模板檢查器：`checkCtx Δ e τ` = 「在 Δ 下模板 e 有型別 τ」。 -/
def checkCtx (Δ : Ctx) : VExpr → Ty → Bool
  | .var v, τ => Δ v == some τ
  | .num _, .i32 => true
  | .num _, _ => false
  | .add a b, .i32 => checkCtx Δ a .i32 && checkCtx Δ b .i32
  | .add _ _, _ => false
  | .eqb a b, .boolean => checkCtx Δ a .i32 && checkCtx Δ b .i32
  | .eqb _ _, _ => false
  | .ite c t f, τ => checkCtx Δ c .boolean && checkCtx Δ t τ && checkCtx Δ f τ

/-- 模板可定型。 -/
def TypableCtx (Δ : Ctx) (e : VExpr) : Prop :=
  checkCtx Δ e Ty.i32 = true ∨ checkCtx Δ e Ty.boolean = true

/-- 自由變量出現（列表，可重複）。 -/
def vars : VExpr → List Nat
  | .var v => [v]
  | .num _ => []
  | .add a b => vars a ++ vars b
  | .eqb a b => vars a ++ vars b
  | .ite c t f => vars c ++ vars t ++ vars f

/-- **語法代入**（模板 → 模板）。 -/
def subst (ρ : Nat → VExpr) : VExpr → VExpr
  | .var v => ρ v
  | .num n => .num n
  | .add a b => .add (subst ρ a) (subst ρ b)
  | .eqb a b => .eqb (subst ρ a) (subst ρ b)
  | .ite c t f => .ite (subst ρ c) (subst ρ t) (subst ρ f)

/-- **展開**（模板 → 調用點程序）：把模板變量換成 T9 語言中的實際節點。 -/
def expand (σ : Nat → Expr) : VExpr → Expr
  | .var v => σ v
  | .num n => .num n
  | .add a b => .add (expand σ a) (expand σ b)
  | .eqb a b => .eqb (expand σ a) (expand σ b)
  | .ite c t f => .ite (expand σ c) (expand σ t) (expand σ f)

/-! ## 二、代入的代數律 -/

/-- 恆等語法代入不動模板。 -/
theorem subst_id (e : VExpr) : subst (fun v => .var v) e = e := by
  induction e with
  | var v => rfl
  | num n => rfl
  | add a b iha ihb => simp [subst, iha, ihb]
  | eqb a b iha ihb => simp [subst, iha, ihb]
  | ite c t f ihc iht ihf => simp [subst, ihc, iht, ihf]

/-- 語法代入的集合律。 -/
theorem subst_comp (ρ₁ ρ₂ : Nat → VExpr) (e : VExpr) :
    subst ρ₂ (subst ρ₁ e) = subst (fun v => subst ρ₂ (ρ₁ v)) e := by
  induction e with
  | var v => rfl
  | num n => rfl
  | add a b iha ihb => simp [subst, iha, ihb]
  | eqb a b iha ihb => simp [subst, iha, ihb]
  | ite c t f ihc iht ihf => simp [subst, ihc, iht, ihf]

/-- **展開是同態**：先語法代入再展開 = 把代入結果展開後再展開。 -/
theorem expand_comp (σ : Nat → Expr) (ρ : Nat → VExpr) (e : VExpr) :
    expand σ (subst ρ e) = expand (fun v => expand σ (ρ v)) e := by
  induction e with
  | var v => rfl
  | num n => rfl
  | add a b iha ihb => simp [subst, expand, iha, ihb]
  | eqb a b iha ihb => simp [subst, expand, iha, ihb]
  | ite c t f ihc iht ihf => simp [subst, expand, ihc, iht, ihf]

/-! ## 三、變量出現（展開只能引用調用點已有的東西） -/

theorem mem_vars_add {v : Nat} {a b : VExpr} (h : v ∈ vars (.add a b)) :
    v ∈ vars a ∨ v ∈ vars b := by
  rw [vars, List.mem_append] at h
  exact h

theorem mem_vars_ite {v : Nat} {c t f : VExpr} (h : v ∈ vars (.ite c t f)) :
    v ∈ vars c ∨ v ∈ vars t ∨ v ∈ vars f := by
  rw [vars, List.mem_append, List.mem_append] at h
  rcases h with h | h
  · rcases h with h | h
    · exact Or.inl h
    · exact Or.inr (Or.inl h)
  · exact Or.inr (Or.inr h)

/-- **代入不引入新的自由變量**：`subst ρ e` 的自由變量必來自某個 `ρ w`（w ∈ e）。 -/
theorem mem_vars_subst {v : Nat} {ρ : Nat → VExpr} :
    ∀ {e : VExpr}, v ∈ vars (subst ρ e) → ∃ w, w ∈ vars e ∧ v ∈ vars (ρ w) := by
  intro e
  induction e with
  | var w => intro h; exact ⟨w, by simp [vars], h⟩
  | num n => intro h; simp [subst, vars] at h
  | add a b iha ihb =>
    intro h
    rw [subst, vars, List.mem_append] at h
    rcases h with h | h
    · obtain ⟨w, hw, hv⟩ := iha h
      exact ⟨w, by rw [vars, List.mem_append]; exact Or.inl hw, hv⟩
    · obtain ⟨w, hw, hv⟩ := ihb h
      exact ⟨w, by rw [vars, List.mem_append]; exact Or.inr hw, hv⟩
  | eqb a b iha ihb =>
    intro h
    rw [subst, vars, List.mem_append] at h
    rcases h with h | h
    · obtain ⟨w, hw, hv⟩ := iha h
      exact ⟨w, by rw [vars, List.mem_append]; exact Or.inl hw, hv⟩
    · obtain ⟨w, hw, hv⟩ := ihb h
      exact ⟨w, by rw [vars, List.mem_append]; exact Or.inr hw, hv⟩
  | ite c t f ihc iht ihf =>
    intro h
    rw [subst, vars, List.mem_append, List.mem_append] at h
    rcases h with h | h
    · rcases h with h | h
      · obtain ⟨w, hw, hv⟩ := ihc h
        exact ⟨w, by rw [vars, List.mem_append, List.mem_append]; exact Or.inl (Or.inl hw), hv⟩
      · obtain ⟨w, hw, hv⟩ := iht h
        exact ⟨w, by rw [vars, List.mem_append, List.mem_append]; exact Or.inl (Or.inr hw), hv⟩
    · obtain ⟨w, hw, hv⟩ := ihf h
      exact ⟨w, by rw [vars, List.mem_append, List.mem_append]; exact Or.inr hw, hv⟩

/-! ## 四、模板檢查器的型別唯一性 -/

/-- **型別唯一性**：同一模板在同一上下文下不可能有兩個型別。 -/
theorem checkCtx_det {Δ : Ctx} : ∀ (e : VExpr) {τ τ' : Ty},
    checkCtx Δ e τ = true → checkCtx Δ e τ' = true → τ = τ' := by
  intro e
  induction e with
  | var v =>
    intro τ τ' h h'
    simp only [checkCtx, beq_iff_eq] at h h'
    rw [h] at h'
    injection h' with h''
  | num n => intro τ τ' h h'; cases τ <;> cases τ' <;> simp_all [checkCtx]
  | add a b iha ihb => intro τ τ' h h'; cases τ <;> cases τ' <;> simp_all [checkCtx]
  | eqb a b iha ihb => intro τ τ' h h'; cases τ <;> cases τ' <;> simp_all [checkCtx]
  | ite c t f ihc iht ihf =>
    intro τ τ' h h'
    simp only [checkCtx, Bool.and_eq_true_iff] at h h'
    obtain ⟨⟨-, ht⟩, -⟩ := h
    obtain ⟨⟨-, ht'⟩, -⟩ := h'
    cases τ <;> cases τ'
    · rfl
    · exact iht ht ht'
    · exact iht ht ht'
    · rfl

/-- 可為 bool ⇒ 不可為 i32。 -/
theorem checkCtx_i32_false_of_bool {Δ : Ctx} {e : VExpr}
    (h : checkCtx Δ e Ty.boolean = true) : checkCtx Δ e Ty.i32 = false := by
  cases hh : checkCtx Δ e Ty.i32 with
  | false => rfl
  | true => exact absurd (checkCtx_det e hh h) (by decide)

/-- 可為 i32 ⇒ 不可為 bool。 -/
theorem checkCtx_bool_false_of_i32 {Δ : Ctx} {e : VExpr}
    (h : checkCtx Δ e Ty.i32 = true) : checkCtx Δ e Ty.boolean = false := by
  cases hh : checkCtx Δ e Ty.boolean with
  | false => rfl
  | true => exact absurd (checkCtx_det e h hh) (by decide)

/-! ## 五、正確臂：展開是同態（可定型性向前傳遞） -/

/-- **T7(b) 同態性（正確臂）**：模板在 Δ 下可定型；展開把 Δ 宣告的每個變量
送到**同型且可定型**的調用點表達式；則展開結果可定型。 -/
theorem checkCtx_expand {Δ : Ctx} {σ : Nat → Expr}
    (hσ : ∀ v τ, Δ v = some τ → check (σ v) τ = true) :
    ∀ (e : VExpr) (τ : Ty), checkCtx Δ e τ = true → check (expand σ e) τ = true := by
  intro e
  induction e with
  | var v =>
    intro τ h
    simp only [checkCtx, beq_iff_eq] at h
    exact hσ v τ h
  | num n => intro τ h; cases τ <;> simp [checkCtx, expand, check] at h ⊢
  | add a b iha ihb =>
    intro τ h
    cases τ
    · simp only [checkCtx, Bool.and_eq_true_iff] at h
      simp only [expand, check, Bool.and_eq_true_iff]
      exact ⟨iha Ty.i32 h.1, ihb Ty.i32 h.2⟩
    · simp [checkCtx] at h
  | eqb a b iha ihb =>
    intro τ h
    cases τ
    · simp [checkCtx] at h
    · simp only [checkCtx, Bool.and_eq_true_iff] at h
      simp only [expand, check, Bool.and_eq_true_iff]
      exact ⟨iha Ty.i32 h.1, ihb Ty.i32 h.2⟩
  | ite c t f ihc iht ihf =>
    intro τ h
    simp only [checkCtx, Bool.and_eq_true_iff] at h
    have hc : checkCtx Δ c Ty.boolean = true := h.1.1
    have ht : checkCtx Δ t τ = true := h.1.2
    have hf : checkCtx Δ f τ = true := h.2
    simp only [expand, check, Bool.and_eq_true_iff]
    exact ⟨⟨ihc Ty.boolean hc, iht τ ht⟩, ihf τ hf⟩

/-- 衍生：模板可定型 ⟹ 展開可定型。 -/
theorem checkCtx_expand_typable {Δ : Ctx} {σ : Nat → Expr}
    (hσ : ∀ v τ, Δ v = some τ → check (σ v) τ = true) {e : VExpr}
    (h : TypableCtx Δ e) : Typable (expand σ e) := by
  rcases h with h | h
  · exact Or.inl (checkCtx_expand hσ e Ty.i32 h)
  · exact Or.inr (checkCtx_expand hσ e Ty.boolean h)

/-! ## 六、錯臂：可定型性被展開反射（不可定型的代入必然失敗） -/

/-- **T7(b) 反射性**：若展開結果可定型 τ，則每個被用到的模板變量，其代入的
表達式本身可定型。「展開不能憑空造出可定型性」。 -/
theorem check_expand_reflect {σ : Nat → Expr} :
    ∀ (e : VExpr) (τ : Ty), check (expand σ e) τ = true →
      ∀ v, v ∈ vars e → ∃ τ', check (σ v) τ' = true := by
  intro e
  induction e with
  | var w =>
    intro τ h v hv
    have hv' : v = w := by simpa [vars] using hv
    subst hv'
    exact ⟨τ, h⟩
  | num n => intro τ h v hv; simp [vars] at hv
  | add a b iha ihb =>
    intro τ h v hv
    cases τ
    · simp only [expand, check, Bool.and_eq_true_iff] at h
      rcases mem_vars_add hv with hv | hv
      · exact iha Ty.i32 h.1 v hv
      · exact ihb Ty.i32 h.2 v hv
    · simp only [expand, check, Bool.false_eq_true] at h
  | eqb a b iha ihb =>
    intro τ h v hv
    cases τ
    · simp only [expand, check, Bool.false_eq_true] at h
    · simp only [expand, check, Bool.and_eq_true_iff] at h
      rcases mem_vars_add hv with hv | hv
      · exact iha Ty.i32 h.1 v hv
      · exact ihb Ty.i32 h.2 v hv
  | ite c t f ihc iht ihf =>
    intro τ h v hv
    simp only [expand, check, Bool.and_eq_true_iff] at h
    rcases mem_vars_ite hv with hv | hv | hv
    · exact ihc Ty.boolean h.1.1 v hv
    · exact iht τ h.1.2 v hv
    · exact ihf τ h.2 v hv

/-! ## 七、T7(b) 判定面：錯臂必被拒絕 -/

/-- 要求 i32 參數的模板臂：`armTemplate v = (v + 1)`。 -/
def armTemplate (v : Nat) : VExpr := .add (.var v) (.num 1)

/-- 模板臂的宣告上下文：`v` 被宣告為 i32，其餘未宣告。 -/
def armCtx (v : Nat) : Ctx := fun w => if w = v then some Ty.i32 else none

/-- 模板臂在 `armCtx v` 下可定型（模板本身是良型的）。 -/
theorem armTemplate_typable (v : Nat) : checkCtx (armCtx v) (armTemplate v) Ty.i32 = true := by
  show ((armCtx v v == some Ty.i32) && true) = true
  rw [show armCtx v v = some Ty.i32 from if_pos rfl]
  decide

/-- **正確臂（同態路徑）**：由 `checkCtx_expand` 直接得到——模板良型 + 代入同型
⇒ 展開可定型。這是「宏展開保持可解性」的正面一半。 -/
theorem right_arm_typable_via_hom {σ : Nat → Expr} {v : Nat}
    (hσ : check (σ v) Ty.i32 = true) :
    check (expand σ (armTemplate v)) Ty.i32 = true := by
  refine checkCtx_expand (Δ := armCtx v) (σ := σ) ?_ (armTemplate v) Ty.i32
    (armTemplate_typable v)
  intro w τ hw
  simp only [armCtx] at hw
  split at hw
  · rename_i hwv
    injection hw with hwτ
    subst hwτ
    rw [hwv]
    exact hσ
  · exact absurd hw (by simp)

/-- 正確臂（直接展開）：模板要求 `v : i32`、代入確為 i32 ⟹ 展開可定型。 -/
theorem right_arm_typable {σ : Nat → Expr} {v : Nat}
    (hσ : check (σ v) Ty.i32 = true) :
    check (expand σ (armTemplate v)) Ty.i32 = true := by
  simp only [expand, armTemplate, check, Bool.and_eq_true_iff]
  exact ⟨hσ, trivial⟩

/-- **型別需求表**：`demands e τ` 列出「若 e 要具型別 τ，則哪些變量必須具哪些
型別」。這與 T9 的 `genC` 是同一件事的語法側對偶（約束表 vs 需求表）。 -/
def demands : VExpr → Ty → List (Nat × Ty)
  | .var v, τ => [(v, τ)]
  | .num _, _ => []
  | .add a b, .i32 => demands a .i32 ++ demands b .i32
  | .add _ _, _ => []
  | .eqb a b, .boolean => demands a .i32 ++ demands b .i32
  | .eqb _ _, _ => []
  | .ite c t f, τ => demands c .boolean ++ demands t τ ++ demands f τ

/-- **展開沿需求表回推型別**：若展開結果具型別 τ，則需求表中的每一項
（變量 v 需具型別 σ'）都被滿足。這是錯臂判定的核心。 -/
theorem check_expand_demands {σ : Nat → Expr} :
    ∀ (e : VExpr) (τ : Ty), check (expand σ e) τ = true →
      ∀ p, p ∈ demands e τ → check (σ p.1) p.2 = true := by
  intro e
  induction e with
  | var v =>
    intro τ h p hp
    simp only [demands, List.mem_cons, List.not_mem_nil, or_false] at hp
    rw [hp]
    exact h
  | num n => intro τ h p hp; simp [demands] at hp
  | add a b iha ihb =>
    intro τ h p hp
    cases τ
    · simp only [expand, check, Bool.and_eq_true_iff] at h
      simp only [demands, List.mem_append] at hp
      rcases hp with hp | hp
      · exact iha Ty.i32 h.1 p hp
      · exact ihb Ty.i32 h.2 p hp
    · have hfalse : check (expand σ (a.add b)) Ty.boolean = false := rfl
      rw [hfalse] at h
      exact absurd h (by decide)
  | eqb a b iha ihb =>
    intro τ h p hp
    cases τ
    · have hfalse : check (expand σ (a.eqb b)) Ty.i32 = false := rfl
      rw [hfalse] at h
      exact absurd h (by decide)
    · simp only [expand, check, Bool.and_eq_true_iff] at h
      simp only [demands, List.mem_append] at hp
      rcases hp with hp | hp
      · exact iha Ty.i32 h.1 p hp
      · exact ihb Ty.i32 h.2 p hp
  | ite c t f ihc iht ihf =>
    intro τ h p hp
    simp only [expand, check, Bool.and_eq_true_iff] at h
    simp only [demands, List.mem_append] at hp
    rcases hp with hp | hp
    · rcases hp with hp | hp
      · exact ihc Ty.boolean h.1.1 p hp
      · exact iht τ h.1.2 p hp
    · exact ihf τ h.2 p hp

/-- 臂模板的型別需求：`v` 必須是 i32。 -/
theorem arm_demand (v : Nat) : (v, Ty.i32) ∈ demands (armTemplate v) Ty.i32 := by
  show (v, Ty.i32) ∈ demands (.var v) Ty.i32 ++ demands (.num 1) Ty.i32
  exact List.mem_append.mpr (Or.inl (List.mem_singleton.mpr rfl))

/-- **反射的具體化**：展開 `armTemplate v` 若可定型（任何型別），則 `σ v` 必為 i32
——臂的型別要求沿展開回推。 -/
theorem expand_arm_forces_i32 {σ : Nat → Expr} {v : Nat} {τ : Ty}
    (h : check (expand σ (armTemplate v)) τ = true) :
    check (σ v) Ty.i32 = true := by
  have hτ : τ = Ty.i32 := by
    cases τ with
    | i32 => rfl
    | boolean =>
      exfalso
      have hfalse : check (expand σ (armTemplate v)) Ty.boolean = false := rfl
      rw [hfalse] at h
      exact absurd h (by decide)
  subst hτ
  exact check_expand_demands (armTemplate v) Ty.i32 h (v, Ty.i32) (arm_demand v)

/-- **T7(b) 錯臂**：把要求 i32 的臂代入 bool 型表達式（例如 `x == y` 的結果），
展開結果在任何情況下都不可定型——型別錯配的宏展開必被拒絕。 -/
theorem wrong_arm_untypable {σ : Nat → Expr} {v : Nat}
    (hbool : check (σ v) Ty.boolean = true) :
    ¬ Typable (expand σ (armTemplate v)) := by
  rintro (h | h)
  · have h32 : check (σ v) Ty.i32 = true := expand_arm_forces_i32 h
    rw [check_bool_false_of_i32 h32] at hbool
    exact absurd hbool (by decide)
  · exfalso
    have hfalse : check (expand σ (armTemplate v)) Ty.boolean = false := rfl
    rw [hfalse] at h
    exact absurd h (by decide)

/-- **衍生（正確臂的代數面）**：正確臂展開可定型 ⟹ 其約束方程組有 0/1 根。 -/
theorem right_arm_solvable {σ : Nat → Expr} {v : Nat}
    (hσ : check (σ v) Ty.i32 = true) :
    ∃ ρ : Sigma, IsRoot (expand σ (armTemplate v)) ρ :=
  (typable_iff_root _).mp (Or.inl (right_arm_typable hσ))

/-- **衍生（錯臂的代數面）**：錯臂展開的約束方程組**無** 0/1 根——這是
「錯臂系統 1 ∈ G」在前端（型別側）的等價刻畫。 -/
theorem wrong_arm_no_root {σ : Nat → Expr} {v : Nat}
    (hbool : check (σ v) Ty.boolean = true) :
    ¬ ∃ ρ : Sigma, IsRoot (expand σ (armTemplate v)) ρ :=
  (untypable_iff_no_root _).mp (wrong_arm_untypable hbool)

end Polyrust
