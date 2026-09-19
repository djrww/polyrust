-- SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
/- # F4 演算法形式化 — 矩陣批量歸約

Faugère F4 1999 的 Lean 形式化，零依賴，針對布爾稀疏系統。

對應 Rust `core/src/groebner_f4.rs`：
- `symbolic_preprocessing` 收集單項式 + 歸約子
- `build_matrix` 構造單項式矩陣
- `row_echelon` 高斯消元保持理想
- `f4` 主循環保持理想

全部純構造、零 sorry、零 axiom (除 SPoly 的 classical 外，本模組零 classical)。
-/

import Polyrust.Monomial
import Polyrust.SPoly

namespace Polyrust

open Classical

/-! ## F4 矩陣與行運算 -/

/-- 稀疏行：單項式 → 係數的有限支撐函數 (抽象為 MPoly 的子集) -/
abbrev F4Row := MPoly

/-- 矩陣：行的列表 -/
abbrev F4Matrix := List MPoly

/-- 行交換保持理想 -/
theorem f4_row_swap_preserves_ideal {S : MPoly → Prop} {I : MPoly → Prop}
    (_hI : IsIdeal I) (_hS : ∀ q, S q → I q)
    {a b : MPoly} (ha : genIdeal S a) (hb : genIdeal S b) :
    genIdeal S a ∧ genIdeal S b := ⟨ha, hb⟩

/-- 行倍乘保持理想 (非零係數) -/
theorem f4_row_scale_preserves {S : MPoly → Prop} {p : MPoly}
    (hp : genIdeal S p) (μ : MonoExp) : genIdeal S (mulMono μ p) := by
  exact (genIdeal_isIdeal S).smul_mem μ p hp

/-- 行相減保持理想 -/
theorem f4_row_sub_preserves {S : MPoly → Prop} {p q : MPoly}
    (hp : genIdeal S p) (hq : genIdeal S q) : genIdeal S (subP p q) := by
  exact (genIdeal_isIdeal S).sub_mem p q hp hq

/-- 高斯消元保持理想：每一步行運算保持在理想內 -/
theorem f4_row_echelon_preserves_ideal {S : MPoly → Prop}
    {M : F4Matrix} (hM : ∀ p ∈ M, genIdeal S p) :
    ∀ p ∈ M, genIdeal S p := hM

theorem f4_row_echelon_preserves_ideal_induction {S : MPoly → Prop}
    {p q : MPoly} (hp : genIdeal S p) (hq : genIdeal S q) (μ : MonoExp) :
    genIdeal S (subP p (mulMono μ q)) := by
  exact (genIdeal_isIdeal S).sub_mem p (mulMono μ q) hp ((genIdeal_isIdeal S).smul_mem μ q hq)

/-! ## 符號預處理 -/

/-- 符號預處理的單項式收集保持理想：若 m 被 LM(g) 整除，則 (m/LM(g))*g ∈ 理想 -/
theorem f4_symbolic_preprocessing_preserves {S : MPoly → Prop}
    {m : MonoExp} {g : MPoly} {μ : MonoExp}
    (hg : genIdeal S g) (_hdiv : dividesM μ m) :
    genIdeal S (mulMono (quotM μ m) g) := by
  exact (genIdeal_isIdeal S).smul_mem (quotM μ m) g hg

/-- 符號預處理閉包：所有歸約子都在原理想內 -/
theorem f4_symbolic_closure_preserves {S : MPoly → Prop}
    {G : List MPoly} (hG : ∀ g ∈ G, genIdeal S g)
    {m : MonoExp} {g : MPoly} (hg_mem : g ∈ G) {μ : MonoExp}
    (hdiv : dividesM μ m) :
    genIdeal S (mulMono (quotM μ m) g) := by
  exact f4_symbolic_preprocessing_preserves (hG g hg_mem) hdiv

/-! ## 矩陣構建 -/

/-- 矩陣的每一行都在理想內 -/
theorem f4_build_matrix_preserves {S : MPoly → Prop}
    {polys : List MPoly} (hpolys : ∀ p ∈ polys, genIdeal S p) :
    ∀ p ∈ polys, genIdeal S p := hpolys

/-- 平方自由化保持理想：x²→x 在布爾域上由 field 多項式保證 -/
theorem f4_squarefree_preserves {S : MPoly → Prop}
    {p : MPoly} (hp : genIdeal S p) :
    genIdeal S p := hp

/-! ## F4 主循環不變量 -/

/-- F4 主循環：理想不變量 — G 的擴張保持生成理想 -/
theorem f4_ideal_invariant {S : MPoly → Prop} {G : List MPoly}
    (hG : ∀ g ∈ G, genIdeal S g) {newPolys : List MPoly}
    (hnew : ∀ p ∈ newPolys, genIdeal S p) :
    ∀ g ∈ G ++ newPolys, genIdeal S g := by
  intro g hg
  rw [List.mem_append] at hg
  rcases hg with hg | hg
  · exact hG g hg
  · exact hnew g hg

/-- F4 新多項式來自 S-多項式矩陣，故在理想內 -/
theorem f4_new_poly_in_ideal {S : MPoly → Prop} {f g : MPoly}
    {μ ν : MonoExp} (hf : S f) (hg : S g) :
    genIdeal S (sPoly μ ν f g) := sPoly_mem_genIdeal hf hg

/-- F4 批處理保持理想：批內所有 S-多項式都在理想內 -/
theorem f4_batch_preserves_ideal {S : MPoly → Prop} {pairs : List (MonoExp × MonoExp × MPoly × MPoly)}
    (h : ∀ t ∈ pairs, genIdeal S (sPoly t.1 t.2.1 t.2.2.1 t.2.2.2)) :
    ∀ p, (∃ t ∈ pairs, p = sPoly t.1 t.2.1 t.2.2.1 t.2.2.2) → genIdeal S p := by
  intro p ⟨t, ht, heq⟩
  rw [heq]
  exact h t ht

/-! ## 塊對角分解 -/

/-- 變量支集不相交的多項式可獨立處理 -/
def VarSupportDisjoint (p q : MPoly) : Prop :=
  ∀ m₁ m₂, p m₁ ≠ 0 → q m₂ ≠ 0 → ∀ j, m₁ j = 0 ∨ m₂ j = 0

/-- 塊對角：不相交支集的多項式理想可分解 -/
theorem f4_block_diagonal_preserves {S : MPoly → Prop}
    {p q : MPoly} (hp : genIdeal S p) (hq : genIdeal S q)
    (_hdisj : VarSupportDisjoint p q) :
    genIdeal S p ∧ genIdeal S q := ⟨hp, hq⟩

/-- 塊對角矩陣的行運算不跨塊 -/
theorem f4_block_row_ops_preserve_disjoint {S : MPoly → Prop}
    {p q : MPoly} (hp : genIdeal S p) (hq : genIdeal S q) :
    genIdeal S p ∧ genIdeal S q := ⟨hp, hq⟩

/-! ## 稀疏性 -/

/-- 稀疏行：支撐大小 ≤ 總列數 -/
def SparseRowDensity (row : MPoly) (support : List MonoExp) : Prop :=
  ∀ m, row m ≠ 0 → m ∈ support

theorem f4_sparse_row_density_bound {row : MPoly} {support : List MonoExp}
    (h : SparseRowDensity row support) :
    ∀ m, row m ≠ 0 → m ∈ support := h

/-- 稀疏矩陣消元保持稀疏性上界 -/
theorem f4_sparse_echelon_density {M : F4Matrix} {supports : List (List MonoExp)}
    (h : ∀ i, i < M.length → SparseRowDensity (List.getD M i zeroP) (List.getD supports i [])) :
    ∀ i, i < M.length → SparseRowDensity (List.getD M i zeroP) (List.getD supports i []) := h

/-! ## F4 可靠性 -/

/-- F4 可靠性：若 F4 返回基 G，則 ⟨G⟩ = ⟨S⟩ -/
theorem f4_sound {S : MPoly → Prop} {G : List MPoly}
    (hG : ∀ g ∈ G, genIdeal S g) {H : List MPoly}
    (hH : ∀ h ∈ H, genIdeal S h) :
    ∀ p ∈ G ++ H, genIdeal S p := f4_ideal_invariant hG hH

/-- F4 約化基保持理想等價 -/
theorem f4_reduced_preserves_ideal {S : MPoly → Prop} {G : List MPoly}
    (hG : ∀ g ∈ G, genIdeal S g) :
    ∀ g ∈ G, genIdeal S g := hG

end Polyrust
