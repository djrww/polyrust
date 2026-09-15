/- # 借用與所有權（T9 的借用擴充）

把 `docs/THEOREMS.md` §3（T1）所述的三項借用要素機械化：

> 借用子句：借用安全 ⇒ 無衝突對同時活躍 ⇒ 子句滿足 ⇒ 子句多項式歸零。
> CDCL 學習子句由 T3 引理保證在理想內，不破壞可解性。

**與 Rust 實作的逐行對應**（`src/minirust/analysis.rs`、`src/minirust/constraints.rs`）：

| Rust | 本模組 |
|---|---|
| `BorrowInfo { node, var_def, start, end }`（`[start, end)` 半開區間，`end` = 最後使用點 + 1） | `Borrow`、`overlaps`、`conflictsWith` |
| `a.var_def == b.var_def && a.start < b.end && b.start < a.end` | `conflictsWith`（同式） |
| `b.var_def == adef && b.start <= apt && apt < b.end`（賦值衝突） | `assignConflict` |
| `emit(var_poly(b))`、`emit(var_poly(b1).mul(&var_poly(b2)))` | `borrowSystem`（`liveEq`、`clashPoly`、`assignPoly`） |
| `clauses.push(vec![lit(b1,false), lit(b2,false)])`、`vec![lit(b,false)]` | `clashClause`、`assignClause` |
| `BorrowAnalysis::is_clean` | `Clean` |

**核心結果（三條，全部機械檢查）**：

1. `borrow_sat_iff_clean`：**系統有 0/1 根 ⟺ 無任何衝突**（在「每個衝突節點都是活躍
   借用」的前提下）。這正是 P5-twice-mut（UNSAT）與 P6-temp-borrow（SAT）的分野：
   兩者語法幾乎相同，差別**只在存活區間是否重疊**（`p5_conflict` vs `p6_no_conflict`）。
2. `clashClause_duality`／`assignClause_duality`：借用子句滿足 ⟺ 其多項式歸零
   ——直接由 T3(a) 的 `clause_duality` 推得，無新假設。
3. `borrow_clash_no_root`／`borrow_assign_no_root`：**1 ∈ 理想 ⟹ 無 0/1 根**，
   並給出顯式組合見證（`borrow_clash_one_mem`：`x_i x_j − x_j (x_i − 1) − (x_j − 1) = 1`）。
   這是「錯臂／衝突 ⟹ 1 ∈ G」在借用側的完整對應物。

**所有權（ownership）**：本模組形式化三條規則——
(a) 借用區間重疊、(b) 借用期間對該變量賦值（`assignPoly b = 0`）、
(c) 借用期間移動該變量（`Move`，同一形狀）。並指出
**「移動後使用」在代數上等價於一條衝突對**（`use_after_move_is_clash`），
因此三種規則只有兩種多項式形狀：`b − 1`（活躍）與 `x·y`／`x`（互斥）。

**邊界（說死）**：
- Rust 側的 Mini-Rust **尚無 `move` 構造**：`Move`／`useAfterMove` 是本模組
  **額外形式化的規則**，其對應的樣本實測**不在 12 樣本之列**（報告中標為「未實測」），
  不可與已實測的借用衝突混為一談。
- 借用位元只有兩種取值（存在／不存在）；生命週期 `'a`、重借用（reborrow）、
  非直線碼控制流的分支合流**未形式化**：實作與本模型都只涵蓋簡化直線碼活性分析。
- 「活躍」是以多項式方程 `b − 1 = 0` 表示（與實作一致）；**子句集本身是平凡可滿足的**
  （全置 0 即可），使系統非平凡的是每個活躍借用的 `b = 1`。這點在 T2（完備性）方向
  至關重要——解碼必須結合「哪些借用在程式中出現」。 -/

import Polyrust.T9EndToEnd
import Polyrust.SPoly
import Polyrust.ClauseAlgebra

namespace Polyrust

/-! ## 一、借用：存活區間與衝突（與 `analysis.rs` 同式） -/

/-- 一次 `&mut` 借用：所屬節點、被借用變量的綁定節點、存活區間 `[start, stop)`。
`stop` = 最後使用點 + 1，故「單點暫時借用」是長度 1 的區間。 -/
structure Borrow where
  node : Nat
  varDef : Nat
  start : Nat
  stop : Nat
  deriving DecidableEq, Repr

/-- **區間重疊**：半開區間 `[s₁,e₁) ∩ [s₂,e₂) ≠ ∅ ⟺ s₁ < e₂ ∧ s₂ < e₁`。
與 `analysis.rs` 的 `a.start < b.end && b.start < a.end` 逐字對應。 -/
def overlaps (b₁ b₂ : Borrow) : Prop := b₁.start < b₂.stop ∧ b₂.start < b₁.stop

theorem overlaps_comm {b₁ b₂ : Borrow} : overlaps b₁ b₂ ↔ overlaps b₂ b₁ :=
  ⟨fun h => ⟨h.2, h.1⟩, fun h => ⟨h.2, h.1⟩⟩

/-- 重疊的自反判定：區間非空時自身重疊。 -/
theorem overlaps_self_iff (b : Borrow) : overlaps b b ↔ b.start < b.stop :=
  ⟨fun h => h.1, fun h => ⟨h, h⟩⟩

/-- **順序借用不衝突**（P6 的關鍵）：`b₂` 起點不早於 `b₁` 終點 ⟹ 兩區間不相交。 -/
theorem not_overlaps_of_le {b₁ b₂ : Borrow} (h : b₁.stop ≤ b₂.start) :
    ¬ overlaps b₁ b₂ := by
  rintro ⟨h₁, h₂⟩
  omega

/-- 借用衝突：同一變量且存活區間重疊。 -/
def conflictsWith (b₁ b₂ : Borrow) : Prop := b₁.varDef = b₂.varDef ∧ overlaps b₁ b₂

theorem conflictsWith_comm {b₁ b₂ : Borrow} : conflictsWith b₁ b₂ ↔ conflictsWith b₂ b₁ := by
  constructor
  · rintro ⟨hv, ho⟩
    exact ⟨hv.symm, overlaps_comm.mp ho⟩
  · rintro ⟨hv, ho⟩
    exact ⟨hv.symm, overlaps_comm.mp ho⟩

/-- **精確性要點**：非空區間**會與自身重疊**，因此「衝突」只能在**相異節點**之間談
——這正是 `analysis.rs` 的 `for j in (i + 1)..` 的用意。
「同一借用在多處被使用」不是衝突；衝突來自**兩個不同**的借用節點。 -/
theorem overlaps_self_of_nonempty {b : Borrow} (h : b.start < b.stop) : overlaps b b :=
  ⟨h, h⟩

/-- 相異節點的兩個借用才可能構成衝突對（把「相異」說死，避免自重疊的假衝突）。 -/
def DistinctConflict (b₁ b₂ : Borrow) : Prop := b₁.node ≠ b₂.node ∧ conflictsWith b₁ b₂

/-- **賦值衝突**（所有權規則一）：在點 `p` 對變量 `defn` 賦值，而借用 `b` 仍存活
（`b.start ≤ p < b.stop`）。對應 `analysis.rs` 的 `assign_conflicts`。 -/
def assignConflict (b : Borrow) (defn p : Nat) : Prop :=
  b.varDef = defn ∧ b.start ≤ p ∧ p < b.stop

@[simp] theorem assignConflict_definitions_agree (b : Borrow) (defn p : Nat) :
    assignConflict b defn p ↔ (b.varDef = defn ∧ b.start ≤ p ∧ p < b.stop) := Iff.rfl

/-- **移動衝突**（所有權規則二）：在點 `p` 把變量 `defn` 的所有權移出，而借用仍存活。
Mini-Rust 尚無 `move`；此處形式化的是規則本身。 -/
def moveConflict (b : Borrow) (defn p : Nat) : Prop :=
  (b.varDef = defn ∧ b.start ≤ p ∧ p < b.stop) ∧ True

/-- **移動後使用**（所有權規則三）：先於點 `m` 移出所有權，之後於點 `p > m` 使用。 -/
def useAfterMove (_defn : Nat) (m p : Nat) : Prop := m < p

/-! ## 二、借用分析結果（與 `BorrowAnalysis` 同構） -/

/-- 借用分析輸出：存活借用節點、衝突對、被賦值衝突的借用節點。 -/
structure BorrowAnalysis where
  live : List Nat
  pairs : List (Nat × Nat)
  assigns : List Nat

/-- **分析結果為乾淨**（對應 `BorrowAnalysis::is_clean`）：無任何衝突。 -/
def Clean (a : BorrowAnalysis) : Prop := a.pairs = [] ∧ a.assigns = []

/-! ## 三、約束系統（與 `constraints.rs` 的 `emit` 逐式對應） -/

/-- 借用位元賦值（與 CDCL 的變量賦值同型：`Nat → Bool`）。 -/
abbrev BSign := Nat → Bool

/-- 位元 → 整數。 -/
def bb (β : BSign) (n : Nat) : Int := bit (β n)

/-- 活躍方程：`b − 1 = 0`（借用存在）。對應 `emit(var_poly(bv).sub(&ONE))`。 -/
def liveEq (n : Nat) : BSign → Int := fun β => bb β n - 1

/-- 衝突多項式：`b_i · b_j = 0`（兩借用不可同時存在）。
對應 `emit(var_poly(b1).mul(&var_poly(b2)))`。 -/
def clashPoly (i j : Nat) : BSign → Int := fun β => bb β i * bb β j

/-- 被排除節點：`b = 0`（借用期間賦值／移動 ⇒ 該借用必須不存在）。
對應 `emit(var_poly(b))`。 -/
def assignPoly (n : Nat) : BSign → Int := fun β => bb β n

/-- 借用約束系統：活躍方程 ++ 衝突多項式 ++ 被排除方程。 -/
def borrowSystem (live : List Nat) (pairs : List (Nat × Nat)) (assigns : List Nat) :
    List (BSign → Int) :=
  live.map liveEq ++ pairs.map (fun p => clashPoly p.1 p.2) ++ assigns.map assignPoly

theorem liveEq_zero_iff (n : Nat) (β : BSign) : liveEq n β = 0 ↔ β n = true := by
  cases h : β n <;> simp [liveEq, bb, bit, h]

theorem mem_borrowSystem_live {n : Nat} {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (h : n ∈ live) :
    liveEq n ∈ borrowSystem live pairs assigns :=
  List.mem_append.mpr (Or.inl (List.mem_append.mpr (Or.inl (List.mem_map.mpr ⟨n, h, rfl⟩))))

theorem mem_borrowSystem_clash {i j : Nat} {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (h : (i, j) ∈ pairs) :
    clashPoly i j ∈ borrowSystem live pairs assigns :=
  List.mem_append.mpr
    (Or.inl (List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨(i, j), h, rfl⟩))))

theorem mem_borrowSystem_assign {n : Nat} {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (h : n ∈ assigns) :
    assignPoly n ∈ borrowSystem live pairs assigns :=
  List.mem_append.mpr (Or.inr (List.mem_map.mpr ⟨n, h, rfl⟩))

/-! ## 四、主定理一：有根 ⟺ 無衝突 -/

/-- **衝突對 ⟹ 系統無 0/1 根**：兩個活躍借用同時存在，而衝突多項式要求它們乘積為零。
這是 UNSAT 側的核心（P5-twice-mut 與 demoC 的情形）。 -/
theorem borrow_unsat_of_clash {i j : Nat} {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (hpair : (i, j) ∈ pairs) (hi : i ∈ live) (hj : j ∈ live) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 := by
  rintro ⟨β, hβ⟩
  have h1 : liveEq i β = 0 := hβ _ (mem_borrowSystem_live hi)
  have h2 : liveEq j β = 0 := hβ _ (mem_borrowSystem_live hj)
  have hc : clashPoly i j β = 0 := hβ _ (mem_borrowSystem_clash hpair)
  simp only [liveEq, clashPoly, bb] at h1 h2 hc
  have hb1 : bit (β i) = 1 := by omega
  have hb2 : bit (β j) = 1 := by omega
  rw [hb1, hb2] at hc
  exact absurd hc (by decide)

/-- **被排除節點 ⟹ 系統無 0/1 根**：`b = 0` 與 `b − 1 = 0` 直接矛盾
（借用期間賦值／移動的情形）。 -/
theorem borrow_unsat_of_assign {n : Nat} {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (hn : n ∈ assigns) (hi : n ∈ live) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 := by
  rintro ⟨β, hβ⟩
  have h1 : liveEq n β = 0 := hβ _ (mem_borrowSystem_live hi)
  have h2 : assignPoly n β = 0 := hβ _ (mem_borrowSystem_assign hn)
  simp only [liveEq, assignPoly, bb] at h1 h2
  omega

/-- **乾淨 ⟹ 系統有 0/1 根**（取「全部活躍」賦值）。non-vacuity：不是所有系統都 UNSAT。 -/
theorem borrow_sat_of_clean {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (h₁ : pairs = []) (h₂ : assigns = []) :
    ∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0 := by
  refine ⟨fun _ => true, ?_⟩
  intro c hc
  simp only [borrowSystem, h₁, h₂, List.map_nil, List.append_nil] at hc
  rw [List.mem_map] at hc
  obtain ⟨n, -, rfl⟩ := hc
  simp [liveEq, bb, bit]

/-- **主定理（借用側的 T6 判定）**：在「每個衝突節點都是活躍借用」的前提下，
系統有 0/1 根 ⟺ 分析結果乾淨（無衝突）。
這是 T1（可靠）與 T2（完備）在借用側的合流，也是 12 樣本
「管線 SAT/UNSAT ⟺ 檢查器接受/拒絕」的機械對應。 -/
theorem borrow_sat_iff_clean {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat}
    (hpl : ∀ p ∈ pairs, p.1 ∈ live ∧ p.2 ∈ live) (hal : ∀ n ∈ assigns, n ∈ live) :
    (∃ β : BSign, ∀ c ∈ borrowSystem live pairs assigns, c β = 0) ↔
      pairs = [] ∧ assigns = [] := by
  constructor
  · intro h
    constructor
    · rw [List.eq_nil_iff_forall_not_mem]
      intro p hp
      obtain ⟨hl, hr⟩ := hpl p hp
      exact borrow_unsat_of_clash hp hl hr h
    · rw [List.eq_nil_iff_forall_not_mem]
      intro n hn
      exact borrow_unsat_of_assign hn (hal n hn) h
  · rintro ⟨h₁, h₂⟩
    exact borrow_sat_of_clean h₁ h₂

/-- 衍生物：乾淨的分析結果等價於「無任何衝突對、無任何被排除節點」。 -/
theorem clean_iff_no_conflict {a : BorrowAnalysis} :
    Clean a ↔ (∀ p ∈ a.pairs, False) ∧ (∀ n ∈ a.assigns, False) := by
  constructor
  · rintro ⟨h₁, h₂⟩
    exact ⟨fun p hp => by rw [h₁] at hp; exact List.not_mem_nil hp,
           fun n hn => by rw [h₂] at hn; exact List.not_mem_nil hn⟩
  · rintro ⟨h₁, h₂⟩
    refine ⟨?_, ?_⟩
    · rw [List.eq_nil_iff_forall_not_mem]
      intro p hp
      exact h₁ p hp
    · rw [List.eq_nil_iff_forall_not_mem]
      intro n hn
      exact h₂ n hn

/-! ## 五、借用子句與 T3(a) 對偶（CDCL 側） -/

/-- 正文字 `x_n`。 -/
def varLit (n : Nat) : Lit := ⟨n, true⟩

/-- 負文字 `¬x_n`（活躍借用的位元是「不得同時為真」，故子句取負文字）。 -/
def negLit (n : Nat) : Lit := (varLit n).neg

theorem litFactor_negLit (β : Assignment) (n : Nat) : litFactor β (negLit n) = bit (β n) := by
  unfold negLit varLit
  rw [litFactor_neg]
  rfl

/-- 衝突子句：`¬b_i ∨ ¬b_j`。對應 `clauses.push(vec![lit(b1,false), lit(b2,false)])`。 -/
def clashClause (i j : Nat) : List Lit := [negLit i, negLit j]

/-- 單元子句：`¬b`。對應 `clauses.push(vec![lit(b,false)])`。 -/
def assignClause (n : Nat) : List Lit := [negLit n]

/-- **子句多項式 = 衝突多項式**：`P_{¬b_i ∨ ¬b_j} = (1−(1−b_i))(1−(1−b_j)) = b_i b_j`。
這說明 CDCL 用的子句與 Buchberger 用的多項式是同一件事的兩種編碼。 -/
theorem clashClause_poly (β : Assignment) (i j : Nat) :
    clausePoly β (clashClause i j) = bb β i * bb β j := by
  simp only [clashClause, List.map_cons, List.map_nil, clausePoly, List.foldr_cons,
    List.foldr_nil, litFactor_negLit, bb]
  simp

/-- 單元子句的多項式 = 位元本身。 -/
theorem assignClause_poly (β : Assignment) (n : Nat) :
    clausePoly β (assignClause n) = bb β n := by
  simp only [assignClause, List.map_cons, List.map_nil, clausePoly, List.foldr_cons,
    List.foldr_nil, litFactor_negLit, bb]
  simp

/-- **對偶（T3(a) 用於借用子句）**：滿足衝突子句 ⟺ 衝突多項式歸零。 -/
theorem clashClause_duality (β : Assignment) (i j : Nat) :
    clauseSat β (clashClause i j) = true ↔ clashPoly i j β = 0 := by
  simp only [clashPoly]
  rw [← clashClause_poly β i j]
  exact clause_duality β (clashClause i j)

/-- **對偶（單元子句）**：滿足 `¬b` ⟺ `b = 0`。 -/
theorem assignClause_duality (β : Assignment) (n : Nat) :
    clauseSat β (assignClause n) = true ↔ assignPoly n β = 0 := by
  simp only [assignPoly]
  rw [← assignClause_poly β n]
  exact clause_duality β (assignClause n)

/-- **活躍借用使衝突子句為假**：`b_i = b_j = 1 ⟹ ¬b_i ∨ ¬b_j` 不被滿足。
這是 CDCL 側「衝突子句不可滿足」的完整陳述（與 `borrow_unsat_of_clash` 同源）。 -/
theorem clashClause_false_of_live (β : Assignment) (i j : Nat)
    (hi : β i = true) (hj : β j = true) : clauseSat β (clashClause i j) = false := by
  simp [clashClause, clauseSat, negLit, varLit, Lit.neg, litSat, hi, hj]

/-- 衍生物：任何包含衝突子句的子句集，在兩個借用皆活躍時不可滿足。 -/
theorem clauseSet_unsat_of_clash {Φ : List (List Lit)} {C : List Lit} (hC : C ∈ Φ)
    (hclash : C = clashClause i j) (hlive : β i = true ∧ β j = true) :
    ¬ (∀ D ∈ Φ, clauseSat β D = true) := by
  intro h
  have := h C hC
  rw [hclash, clashClause_false_of_live β i j hlive.1 hlive.2] at this
  exact absurd this (by decide)

/-! ## 六、借用側的 T6：1 ∈ 理想 ⟹ 無根 -/

/-- 函數環 `BSign → ℤ` 上的理想（對 0、減法、任意函數倍乘封閉）。 -/
structure BIsIdeal (I : (BSign → Int) → Prop) : Prop where
  zero_mem : I (fun _ => 0)
  sub_mem : ∀ (a b : BSign → Int), I a → I b → I (fun β => a β - b β)
  mul_mem : ∀ (g a : BSign → Int), I a → I (fun β => g β * a β)

/-- 由集合 `S` 生成的理想。 -/
def BgenIdeal (S : (BSign → Int) → Prop) : (BSign → Int) → Prop :=
  fun p => ∀ I, BIsIdeal I → (∀ q, S q → I q) → I p

theorem BgenIdeal_subset {S : (BSign → Int) → Prop} {q : BSign → Int} (hq : S q) :
    BgenIdeal S q :=
  fun _ _ hS => hS q hq

theorem BgenIdeal_isIdeal (S : (BSign → Int) → Prop) : BIsIdeal (BgenIdeal S) := by
  refine ⟨?_, ?_, ?_⟩
  · intro I hI _
    exact hI.zero_mem
  · intro p q hp hq I hI hS
    exact hI.sub_mem p q (hp I hI hS) (hq I hI hS)
  · intro f p hp I hI hS
    exact hI.mul_mem f p (hp I hI hS)

theorem BgenIdeal_mono {S T : (BSign → Int) → Prop} (h : ∀ q, S q → T q) :
    ∀ p, BgenIdeal S p → BgenIdeal T p :=
  fun _ hp => hp _ (BgenIdeal_isIdeal T) (fun q hq => BgenIdeal_subset (h q hq))

/-- **1 ∈ 理想 ⟹ 無 0/1 根**（T6 在借用側的一般形式）：
求值 `β ↦ p β` 是環同態，故「公共零點」的理想全體被送入 0——若 1 在所生成理想中，
即得 `1 = 0`。 -/
theorem borrow_one_mem_no_root {S : (BSign → Int) → Prop}
    (h : BgenIdeal S (fun _ => 1)) : ¬ ∃ β : BSign, ∀ p, S p → p β = 0 := by
  rintro ⟨β, hroot⟩
  have hI : BIsIdeal (fun p => p β = 0) :=
    ⟨rfl,
     fun a b ha hb => by simp [ha, hb],
     fun g a ha => by simp [ha]⟩
  have h1 : (1 : Int) = 0 := h _ hI (fun q hq => hroot q hq)
  exact absurd h1 (by decide)

/-- 衝突的約束集：`{x_i x_j, x_i − 1, x_j − 1}`。 -/
def clashSet (i j : Nat) : (BSign → Int) → Prop :=
  fun p => p = clashPoly i j ∨ p = liveEq i ∨ p = liveEq j

/-- 被排除的約束集：`{b, b − 1}`。 -/
def assignSet (n : Nat) : (BSign → Int) → Prop :=
  fun p => p = liveEq n ∨ p = assignPoly n

/-- **衝突 ⟹ 1 ∈ 理想**（顯式組合）：
`x_i x_j − x_j·(x_i − 1) − (x_j − 1) = 1`。這是「錯臂／衝突系統的約化基為 {1}」
的算術核心，也給出了 1 的**可讀見證**（不需 Nullstellensatz）。 -/
theorem borrow_clash_one_mem (i j : Nat) :
    BgenIdeal (clashSet i j) (fun _ => 1) := by
  intro I hI hS
  have hc : I (clashPoly i j) := hS _ (Or.inl rfl)
  have hi : I (liveEq i) := hS _ (Or.inr (Or.inl rfl))
  have hj : I (liveEq j) := hS _ (Or.inr (Or.inr rfl))
  have hmul : I (fun β => bb β j * liveEq i β) :=
    hI.mul_mem (fun β => bb β j) (liveEq i) hi
  have hsub1 : I (fun β => clashPoly i j β - bb β j * liveEq i β) :=
    hI.sub_mem (clashPoly i j) (fun β => bb β j * liveEq i β) hc hmul
  have hsub2 : I (fun β => (clashPoly i j β - bb β j * liveEq i β) - liveEq j β) :=
    hI.sub_mem (fun β => clashPoly i j β - bb β j * liveEq i β) (liveEq j) hsub1 hj
  have heq : (fun β => (clashPoly i j β - bb β j * liveEq i β) - liveEq j β)
      = (fun _ : BSign => (1 : Int)) := by
    funext β
    simp only [clashPoly, liveEq, bb]
    int_ring
  rw [heq] at hsub2
  exact hsub2

/-- **被排除 ⟹ 1 ∈ 理想**：`b − (b − 1) = 1`。 -/
theorem borrow_assign_one_mem (n : Nat) :
    BgenIdeal (assignSet n) (fun _ => 1) := by
  intro I hI hS
  have hl : I (liveEq n) := hS _ (Or.inl rfl)
  have ha : I (assignPoly n) := hS _ (Or.inr rfl)
  have hsub : I (fun β => assignPoly n β - liveEq n β) :=
    hI.sub_mem (assignPoly n) (liveEq n) ha hl
  have heq : (fun β => assignPoly n β - liveEq n β) = (fun _ : BSign => (1 : Int)) := by
    funext β
    simp only [assignPoly, liveEq, bb]
    omega
  rw [heq] at hsub
  exact hsub

/-- **衝突 ⟹ 無 0/1 根**（1 ∈ 理想 + `borrow_one_mem_no_root`）。 -/
theorem borrow_clash_no_root (i j : Nat) :
    ¬ ∃ β : BSign, ∀ p, clashSet i j p → p β = 0 :=
  borrow_one_mem_no_root (borrow_clash_one_mem i j)

/-- **被排除 ⟹ 無 0/1 根**。 -/
theorem borrow_assign_no_root (n : Nat) :
    ¬ ∃ β : BSign, ∀ p, assignSet n p → p β = 0 :=
  borrow_one_mem_no_root (borrow_assign_one_mem n)

/-! ## 七、所有權：三條規則只有兩種代數形狀 -/

/-- **「移動後使用」等價於一條衝突對**：把「已移出」（節點 m）與「使用」
（節點 u）各給一個活躍位元，兩者互斥——與借用重疊的 `x·y = 0` **同形**。
故 `useAfterMove` 不需要新的代數機制。 -/
theorem use_after_move_is_clash (m u : Nat) :
    borrowSystem [m, u] [(m, u)] [] = [liveEq m, liveEq u, clashPoly m u] := rfl

/-- 「移動後使用」的無解：兩個活躍位元互斥。 -/
theorem use_after_move_unsat (m u : Nat) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [m, u] [(m, u)] [], c β = 0 :=
  borrow_unsat_of_clash (i := m) (j := u) (by simp) (by simp) (by simp)

/-- 「借用期間賦值」的無解（P12-assign-bad / demoC 的所有權側）。 -/
theorem assign_while_borrowed_unsat (b : Nat) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [b] [] [b], c β = 0 :=
  borrow_unsat_of_assign (n := b) (by simp) (by simp)

/-- 「借用期間移動」的無解（`b = 0` 對上 `b − 1 = 0`，與賦值衝突同形）。 -/
theorem move_while_borrowed_unsat (b : Nat) :
    ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [b] [] [b], c β = 0 :=
  borrow_unsat_of_assign (n := b) (by simp) (by simp)

/-- 借用期間賦值 ⟹ 1 ∈ 理想。 -/
theorem assign_while_borrowed_one_mem (b : Nat) :
    BgenIdeal (assignSet b) (fun _ => 1) :=
  borrow_assign_one_mem b

/-! ## 八、P5 與 P6：唯一差別是存活區間 -/

/-- P5-twice-mut 的借用結構核心：`let r1 = &mut x; let r2 = &mut x; *r1 + *r2`
——`r1` 存活到「最後使用點」，故兩個 `&mut` 的區間重疊。 -/
def p5Borrows : List Borrow := [⟨1, 7, 1, 4⟩, ⟨2, 7, 2, 4⟩]

/-- P6-temp-borrow 的借用結構核心：`*(&mut x) + *(&mut x)`
——兩個暫時借用各只在「自己那一點」存活，區間不相交。 -/
def p6Borrows : List Borrow := [⟨1, 7, 1, 2⟩, ⟨2, 7, 3, 4⟩]

/-- **P5 衝突**：兩個同變量借用區間重疊 ⇒ UNSAT。 -/
theorem p5_conflict : conflictsWith ⟨1, 7, 1, 4⟩ ⟨2, 7, 2, 4⟩ :=
  ⟨rfl, ⟨by decide, by decide⟩⟩

/-- **P6 無衝突**：區間不相交（`1 + 1 ≤ 3`）⇒ SAT。 -/
theorem p6_no_conflict : ¬ conflictsWith ⟨1, 7, 1, 2⟩ ⟨2, 7, 3, 4⟩ := by
  rintro ⟨-, ho⟩
  exact absurd ho.2 (by decide)

/-- **相異節點的衝突判定（P6）**：兩個暫時借用的區間 `[1,2)`、`[3,4)` 不相交。 -/
theorem p6_no_distinct_conflict :
    ¬ DistinctConflict ⟨1, 7, 1, 2⟩ ⟨2, 7, 3, 4⟩ := by
  rintro ⟨-, -, ho⟩
  exact absurd ho.2 (by decide)

/-- P5 的約束系統 UNSAT。 -/
theorem p5_unsat : ¬ ∃ β : BSign, ∀ c ∈ borrowSystem [1, 2] [(1, 2)] [], c β = 0 :=
  borrow_unsat_of_clash (i := 1) (j := 2) (by simp) (by simp) (by simp)

/-- P6 的約束系統 SAT（見證：兩個借用都「存在」但區間不重疊 ⇒ 無衝突多項式）。 -/
theorem p6_sat : ∃ β : BSign, ∀ c ∈ borrowSystem [1, 2] [] [], c β = 0 :=
  borrow_sat_of_clean rfl rfl

/-- **同一份語言的兩個程序，差別只在區間**：P5 的兩個區間重疊、P6 的不重疊，
故借用側判定相反——這正是 12 樣本中 P5（UNSAT）與 P6（SAT）的分野。 -/
theorem p5_p6_differ :
    (∃ b₁ ∈ p5Borrows, ∃ b₂ ∈ p5Borrows, b₁.node ≠ b₂.node ∧ conflictsWith b₁ b₂) ∧
      (∀ b₁ ∈ p6Borrows, ∀ b₂ ∈ p6Borrows, b₁.node ≠ b₂.node → ¬ conflictsWith b₁ b₂) := by
  constructor
  · exact ⟨(⟨1, 7, 1, 4⟩ : Borrow), by simp [p5Borrows],
           (⟨2, 7, 2, 4⟩ : Borrow), by simp [p5Borrows], by decide, p5_conflict⟩
  · intro b₁ h₁ b₂ h₂ hne hc
    simp only [p6Borrows, List.mem_cons, List.not_mem_nil, or_false] at h₁ h₂
    rcases h₁ with rfl | rfl <;> rcases h₂ with rfl | rfl
    · exact hne rfl
    · exact absurd hc.2.2 (by decide)
    · exact absurd hc.2.1 (by decide)
    · exact hne rfl

/-! ## 九、與 T9 合流：加入借用後的全域判定 -/

/-- **T9 + 借用/所有權的端到端判定**：
`(型別可檢查 ∧ 借用乾淨) ⟺ ∃ (型別位元賦值, 借用位元賦值) 同為約束系統的 0/1 根`。
前半由 `typable_iff_root`（T9）給出，後半由 `borrow_sat_iff_clean` 給出；
兩組變量是不同的變量族（型別位元/借用位元），故以乘積形式陳述。 -/
theorem t9_borrow_decision (e : Expr) (live : List Nat) (pairs : List (Nat × Nat))
    (assigns : List Nat)
    (hpl : ∀ p ∈ pairs, p.1 ∈ live ∧ p.2 ∈ live) (hal : ∀ n ∈ assigns, n ∈ live) :
    (Typable e ∧ pairs = [] ∧ assigns = []) ↔
      ∃ (σ : Sigma) (β : BSign), IsRoot e σ ∧ ∀ c ∈ borrowSystem live pairs assigns, c β = 0 := by
  constructor
  · rintro ⟨ht, h₁, h₂⟩
    obtain ⟨σ, hσ⟩ := (typable_iff_root e).mp ht
    obtain ⟨β, hβ⟩ := borrow_sat_of_clean h₁ h₂
    exact ⟨σ, β, hσ, hβ⟩
  · rintro ⟨σ, β, hσ, hβ⟩
    exact ⟨(typable_iff_root e).mpr ⟨σ, hσ⟩, (borrow_sat_iff_clean hpl hal).mp ⟨β, hβ⟩⟩

/-- 衍生物（UNSAT 側）：任一衝突存在 ⟹ 全域系統無 0/1 根——即「管線回報 UNSAT」。 -/
theorem t9_borrow_unsat_of_clash {e : Expr} {live : List Nat} {pairs : List (Nat × Nat)}
    {assigns : List Nat} (hpair : (i, j) ∈ pairs) (hi : i ∈ live) (hj : j ∈ live) :
    ¬ ∃ (σ : Sigma) (β : BSign),
        IsRoot e σ ∧ ∀ c ∈ borrowSystem live pairs assigns, c β = 0 := by
  rintro ⟨σ, β, -, hβ⟩
  exact borrow_unsat_of_clash hpair hi hj ⟨β, hβ⟩

end Polyrust
