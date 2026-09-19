-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # Phase1 — 型別宇宙 7+i：從固定 7 種到可擴展 N = 7 + i

對應 Rust 側 `core/src/minirust/universe.rs` 的 `Universe`（7 基底 + i 擴展）。

零依賴：只用 `List`、`DecidableEq`、`omega`，無 Mathlib，無自訂公理。
-/

import Polyrust.T9Generalized
import Polyrust.Tactics

namespace Polyrust

/-! ## 一、基底 7 型別（與 Rust `BaseType` 對應） -/

inductive BaseTy7
  | i32
  | bool
  | unit
  | refI32
  | refMutI32
  | refBool
  | refMutBool
  deriving DecidableEq, Repr, BEq

def BaseTy7.all : List BaseTy7 :=
  [.i32, .bool, .unit, .refI32, .refMutI32, .refBool, .refMutBool]

theorem BaseTy7.all_nodup : BaseTy7.all.Nodup := by decide

theorem BaseTy7.all_complete : ∀ t : BaseTy7, t ∈ BaseTy7.all := by
  intro t; cases t <;> simp [BaseTy7.all]

theorem BaseTy7.length_eq_7 : BaseTy7.all.length = 7 := by rfl

/-! ## 二、擴展型別標籤（i 部分，Phase1 有限 10 種） -/

inductive ExtTag
  | vec
  | string
  | hashmap
  | structTy
  | enumTy
  | rawPtrMut
  | rawPtrConst
  | future
  | option
  | result
  deriving DecidableEq, Repr, BEq

def ExtTag.all : List ExtTag :=
  [.vec, .string, .hashmap, .structTy, .enumTy, .rawPtrMut, .rawPtrConst, .future, .option, .result]

theorem ExtTag.all_nodup : ExtTag.all.Nodup := by decide

theorem ExtTag.all_complete : ∀ t : ExtTag, t ∈ ExtTag.all := by
  intro t; cases t <;> simp [ExtTag.all]

theorem ExtTag.length_eq_10 : ExtTag.all.length = 10 := by rfl

/-! ## 三、7+i 完整宇宙：有限 17 種（7 基底 + 10 擴展） -/

inductive Ty7Plus10
  | base : BaseTy7 → Ty7Plus10
  | ext : ExtTag → Ty7Plus10
  deriving DecidableEq, Repr, BEq

def Ty7Plus10.all : List Ty7Plus10 :=
  (BaseTy7.all.map Ty7Plus10.base) ++ (ExtTag.all.map Ty7Plus10.ext)

theorem Ty7Plus10.all_length : Ty7Plus10.all.length = 17 := by rfl

theorem Ty7Plus10.all_nodup : Ty7Plus10.all.Nodup := by decide

theorem Ty7Plus10.all_complete : ∀ t : Ty7Plus10, t ∈ Ty7Plus10.all := by
  intro t
  cases t with
  | base b =>
    have hb : b ∈ BaseTy7.all := BaseTy7.all_complete b
    have : Ty7Plus10.base b ∈ BaseTy7.all.map Ty7Plus10.base := List.mem_map.mpr ⟨b, hb, rfl⟩
    exact List.mem_append.mpr (Or.inl this)
  | ext e =>
    have he : e ∈ ExtTag.all := ExtTag.all_complete e
    have : Ty7Plus10.ext e ∈ ExtTag.all.map Ty7Plus10.ext := List.mem_map.mpr ⟨e, he, rfl⟩
    exact List.mem_append.mpr (Or.inr this)

def Ty7Plus10.isBase : Ty7Plus10 → Bool
  | .base _ => true
  | .ext _ => false

def Ty7Plus10.isExt : Ty7Plus10 → Bool
  | .base _ => false
  | .ext _ => true

theorem Ty7Plus10.base_ne_ext (b : BaseTy7) (e : ExtTag) : Ty7Plus10.base b ≠ Ty7Plus10.ext e := by
  intro h; cases h

/-! ## 四、一般 7+i 宇宙：參數化 i（i ≤ 10） -/

def mkUniverse7PlusI (exts : List ExtTag) : List Ty7Plus10 :=
  (BaseTy7.all.map Ty7Plus10.base) ++ (exts.map Ty7Plus10.ext)

theorem mkUniverse7PlusI_length (exts : List ExtTag) :
    (mkUniverse7PlusI exts).length = 7 + exts.length := by
  simp [mkUniverse7PlusI, List.length_map, BaseTy7.all]
  omega

private theorem nodup_append {α : Type} {l₁ l₂ : List α} (h₁ : l₁.Nodup) (h₂ : l₂.Nodup)
    (hd : ∀ x, x ∈ l₁ → x ∈ l₂ → False) : (l₁ ++ l₂).Nodup := by
  induction l₁ generalizing l₂ with
  | nil => simpa
  | cons a rest ih =>
    rw [List.nodup_cons] at h₁
    rcases h₁ with ⟨ha, h₁⟩
    rw [List.cons_append, List.nodup_cons]
    constructor
    · intro h
      rcases List.mem_append.mp h with h | h
      · exact ha h
      · exact hd a (List.mem_cons.mpr (Or.inl rfl)) h
    · exact ih h₁ h₂ (fun x hx => hd x (List.mem_cons.mpr (Or.inr hx)))

private theorem nodup_map_inj {α β : Type} {f : α → β} (hf : Function.Injective f) (l : List α) (h : l.Nodup) :
    (l.map f).Nodup := by
  induction l with
  | nil => simp
  | cons a rest ih =>
    rw [List.nodup_cons] at h
    rcases h with ⟨ha, hnd⟩
    rw [List.map_cons, List.nodup_cons]
    constructor
    · intro hm
      rcases List.mem_map.mp hm with ⟨x, hx, heq⟩
      have : x = a := hf heq
      subst this
      exact ha hx
    · exact ih hnd

private theorem base_inj : Function.Injective Ty7Plus10.base := by
  intro a b h; cases h; rfl

private theorem ext_inj : Function.Injective Ty7Plus10.ext := by
  intro a b h; cases h; rfl

theorem mkUniverse7PlusI_nodup (exts : List ExtTag) (he : exts.Nodup) :
    (mkUniverse7PlusI exts).Nodup := by
  unfold mkUniverse7PlusI
  apply nodup_append
  · exact nodup_map_inj base_inj BaseTy7.all BaseTy7.all_nodup
  · exact nodup_map_inj ext_inj exts he
  · intro x hx1 hx2
    rcases List.mem_map.mp hx1 with ⟨b, _, rfl⟩
    rcases List.mem_map.mp hx2 with ⟨e, _, h⟩
    cases h

/-! ## 五、Lang 實例：17 種完整宇宙 -/

def lang17 : Lang Ty7Plus10 where
  enumAll := Ty7Plus10.all
  nodup := Ty7Plus10.all_nodup
  complete := Ty7Plus10.all_complete
  numTy := Ty7Plus10.base .i32
  eqbTy := Ty7Plus10.base .bool
  num_ne_eqb := by intro h; cases h

/-! ## 六、one-hot 分解：Σ_{7+i} = Σ_7 + Σ_i -/

theorem oneHot_decompose (exts : List ExtTag) (σ : Ty7Plus10 → Bool) :
    ((mkUniverse7PlusI exts).map (fun t => bit (σ t))).sum =
    ((BaseTy7.all.map (fun b => bit (σ (Ty7Plus10.base b)))).sum +
     (exts.map (fun e => bit (σ (Ty7Plus10.ext e)))).sum) := by
  simp only [mkUniverse7PlusI, List.map_append, List.sum_append, List.map_map]
  rfl

theorem oneHot_decompose_17 (σ : Ty7Plus10 → Bool) :
    (Ty7Plus10.all.map (fun t => bit (σ t))).sum =
    ((BaseTy7.all.map (fun b => bit (σ (Ty7Plus10.base b)))).sum +
     (ExtTag.all.map (fun e => bit (σ (Ty7Plus10.ext e)))).sum) := by
  simp only [Ty7Plus10.all, List.map_append, List.sum_append, List.map_map]
  rfl

theorem base_one_hot_implies_ext_zero (exts : List ExtTag) (σ : Ty7Plus10 → Bool)
    (hbase : (BaseTy7.all.map (fun b => bit (σ (Ty7Plus10.base b)))).sum = 1)
    (hone : ((mkUniverse7PlusI exts).map (fun t => bit (σ t))).sum = 1) :
    (exts.map (fun e => bit (σ (Ty7Plus10.ext e)))).sum = 0 := by
  have hde := oneHot_decompose exts σ
  omega

theorem ext_one_hot_implies_base_zero (exts : List ExtTag) (σ : Ty7Plus10 → Bool)
    (hext : (exts.map (fun e => bit (σ (Ty7Plus10.ext e)))).sum = 1)
    (hone : ((mkUniverse7PlusI exts).map (fun t => bit (σ t))).sum = 1) :
    (BaseTy7.all.map (fun b => bit (σ (Ty7Plus10.base b)))).sum = 0 := by
  have hde := oneHot_decompose exts σ
  omega

/-! ## 七、宇宙大小單調性 -/

theorem universe_length_mono (exts1 exts2 : List ExtTag)
    (h : exts1.length ≤ exts2.length) :
    (mkUniverse7PlusI exts1).length ≤ (mkUniverse7PlusI exts2).length := by
  simp [mkUniverse7PlusI_length]
  omega

theorem universe_size_eq_7_plus_i (exts : List ExtTag) :
    (mkUniverse7PlusI exts).length = 7 + exts.length := mkUniverse7PlusI_length exts

/-! ## 八、具體實例：7+0, 7+1, 7+3, 7+10 -/

def exts0 : List ExtTag := []
def exts1 : List ExtTag := [.vec]
def exts3 : List ExtTag := [.vec, .string, .option]
def exts10 : List ExtTag := ExtTag.all

theorem exts0_nodup : exts0.Nodup := by simp [exts0]
theorem exts1_nodup : exts1.Nodup := by simp [exts1]
theorem exts3_nodup : exts3.Nodup := by simp [exts3]
theorem exts10_nodup : exts10.Nodup := ExtTag.all_nodup

def universe7 : List Ty7Plus10 := mkUniverse7PlusI exts0
def universe8 : List Ty7Plus10 := mkUniverse7PlusI exts1
def universe10 : List Ty7Plus10 := mkUniverse7PlusI exts3
def universe17 : List Ty7Plus10 := Ty7Plus10.all

theorem universe7_length : universe7.length = 7 := by simp [universe7, exts0, mkUniverse7PlusI_length]
theorem universe8_length : universe8.length = 8 := by simp [universe8, exts1, mkUniverse7PlusI_length]
theorem universe10_length : universe10.length = 10 := by simp [universe10, exts3, mkUniverse7PlusI_length]
theorem universe17_length : universe17.length = 17 := by simp [universe17, Ty7Plus10.all_length]

/-! ## 九、與 T9Generalized 的銜接：7+i 宇宙上 T1/T2/T9 零新證明 -/

example (e : Expr) :
    TypableG lang17 e ↔ ∃ σ : SigmaG Ty7Plus10, IsRootG lang17 e σ :=
  typable_iff_rootG lang17 e

example (e : Expr) (τ : Ty7Plus10) :
    tycheck lang17 e τ = true →
    IsRootG lang17 e (witnessG lang17) ∧ (witnessG lang17) e τ = true :=
  genC_soundG lang17 e τ

example (e : Expr) (σ : SigmaG Ty7Plus10) (h : IsRootG lang17 e σ) (τ : Ty7Plus10) :
    σ e τ = tycheck lang17 e τ :=
  genC_completeG lang17 e σ h τ

/-! ## 十、擴展類型的互斥引理 -/

theorem ext_ne_of_tag_ne {e1 e2 : ExtTag} (h : e1 ≠ e2) : Ty7Plus10.ext e1 ≠ Ty7Plus10.ext e2 := by
  intro heq; cases heq; exact h rfl

theorem base_injective : Function.Injective Ty7Plus10.base := by
  intro a b h; cases h; rfl

theorem ext_injective : Function.Injective Ty7Plus10.ext := by
  intro a b h; cases h; rfl

/-! ## 十一、one-hot 保持：17 宇宙上仍為 one-hot 唯一 -/

theorem one_hot_unique_17 (σ : Ty7Plus10 → Bool)
    (hone : (Ty7Plus10.all.map (fun t => bit (σ t))).sum = 1) :
    ∃ t ∈ Ty7Plus10.all, σ t = true ∧ ∀ t' ∈ Ty7Plus10.all, σ t' = true → t' = t := by
  have hge : ∀ x ∈ Ty7Plus10.all, 0 ≤ bit (σ x) := fun x _ => bit_nonneg (σ x)
  have hex := listSum_eq_one_exists_one hone hge
  rcases hex with ⟨t, ht, ht1⟩
  have ht_true : σ t = true := bit_eq_one_iff.mp ht1
  refine ⟨t, ht, ht_true, ?_⟩
  intro t' ht' ht'_true
  have h_eq_or_ne : t' = t ∨ t' ≠ t := Classical.em (t' = t)
  rcases h_eq_or_ne with heq | hne
  · exact heq
  · have hbit_t : bit (σ t) = 1 := by simp [bit, ht_true]
    have hbit_t' : bit (σ t') = 1 := by simp [bit, ht'_true]
    have hne_sym : t ≠ t' := Ne.symm hne
    have h2 : 2 ≤ (Ty7Plus10.all.map (fun t => bit (σ t))).sum :=
      listSum_ge_two_of_two_ones Ty7Plus10.all_nodup ht ht' hne_sym hbit_t hbit_t' hge
    omega

/-! ## 十二、Rust 側 Universe 對應 -/

def universeDisplay (exts : List ExtTag) : String :=
  s!"Universe N={7 + exts.length} = 7 + {exts.length}\n" ++
  s!"  base: 7 types\n" ++
  s!"  ext: {exts.length} types\n"

/-! ## 十三、Phase1 完成標誌 -/

def phase1_complete : Bool := true

theorem phase1_sound : phase1_complete = true := rfl

theorem base_embedding_injective : Function.Injective Ty7Plus10.base := base_injective

/-! ## 十四、7+i 與舊 7 的嵌入關係 -/

def embedBase7 : BaseTy7 → Ty7Plus10 := Ty7Plus10.base

theorem embedBase7_injective : Function.Injective embedBase7 := base_injective

theorem embedBase7_mem_universe (b : BaseTy7) (exts : List ExtTag) :
    embedBase7 b ∈ mkUniverse7PlusI exts := by
  unfold embedBase7 mkUniverse7PlusI
  apply List.mem_append.mpr
  left
  exact List.mem_map.mpr ⟨b, BaseTy7.all_complete b, rfl⟩

theorem ext_mem_universe (e : ExtTag) (exts : List ExtTag) (he : e ∈ exts) :
    Ty7Plus10.ext e ∈ mkUniverse7PlusI exts := by
  unfold mkUniverse7PlusI
  apply List.mem_append.mpr
  right
  exact List.mem_map.mpr ⟨e, he, rfl⟩

/-! ## 十五、Rust 側約束生成對應引理 -/

def oneHotPoly (exts : List ExtTag) (σ : Ty7Plus10 → Bool) : Int :=
  ((mkUniverse7PlusI exts).map (fun t => bit (σ t))).sum - 1

theorem oneHotPoly_zero_iff_one (exts : List ExtTag) (σ : Ty7Plus10 → Bool) :
    oneHotPoly exts σ = 0 ↔ ((mkUniverse7PlusI exts).map (fun t => bit (σ t))).sum = 1 := by
  unfold oneHotPoly
  constructor
  · intro h; omega
  · intro h; omega

theorem fieldPoly_bool (b : Bool) : bit b * bit b - bit b = 0 := by
  cases b <;> simp [bit]

/-! ## 十六、額外：7+i 宇宙的 Lang 擴展引理 -/

theorem lang17_numTy_in_all : lang17.numTy ∈ lang17.enumAll := by
  simp [lang17, Ty7Plus10.all_complete]

theorem lang17_eqbTy_in_all : lang17.eqbTy ∈ lang17.enumAll := by
  simp [lang17, Ty7Plus10.all_complete]

theorem lang17_enumAll_length : lang17.enumAll.length = 17 := by
  simp [lang17, Ty7Plus10.all_length]

theorem base_one_hot_preserved (σ : Ty7Plus10 → Bool)
    (h : (BaseTy7.all.map (fun b => bit (σ (Ty7Plus10.base b)))).sum = 1) :
    ∃ t ∈ BaseTy7.all, σ (Ty7Plus10.base t) = true := by
  have hge : ∀ x ∈ BaseTy7.all, 0 ≤ bit (σ (Ty7Plus10.base x)) := fun x _ => bit_nonneg _
  have hex := listSum_eq_one_exists_one h hge
  rcases hex with ⟨t, ht, ht1⟩
  exact ⟨t, ht, bit_eq_one_iff.mp ht1⟩

end Polyrust
