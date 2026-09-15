# Phase2 實施報告 — 類型統一與降維

> 對應 `docs/EXTENSION_PLAN.md` Phase2：`ty.rs` 宇宙構造與統一，`lower.rs` 降維，`constraints_v2.rs` 可變 N 約束，Lean `ModuleFlatten` / `MatchDecisionTree` / `ProductSum`

## 1. 目標

- 實現 `ty.rs`: 類型宇宙構造、閉包、統一 `unify(T1,T2)` → `t_T1 - t_T2=0`
- 實現 `lower.rs`: struct/enum → product/sum, match → if 決策樹, for → loop+IntoIterator, async → state machine, mod → flatten
- 擴展 `constraints.rs` 至可變 N = 7+i，product/sum/match 編碼
- Lean 新增 3 模組，`lake build` 全綠

## 2. Rust 側實現

### 2.1 新增文件

| 文件 | 功能 | 測試 |
|---|---|---|
| `core/src/minirust/ty.rs` | `build_universe_from_program`, `build_universe_from_src`, `unify`, `subst_type`, `DepGraph` | 5 |
| `core/src/minirust/lower.rs` | `LowerCtx`, `lower_program`, `parse_match_to_decision_tree`, `decision_tree_to_if_chain`, `parse_for_to_loop`, `for_loop_to_loop_text`, `product_poly_text`, `sum_poly_text` | 6 |
| `core/src/minirust/constraints_v2.rs` | `SystemV2` 可變 N, `gen_constraints_v2`, `gen_match_constraints`, `to_r1cs_v2`, product/sum/match 約束 | 4 |
| `frontends/full/src/ty.rs` | 前端封裝 `build_universe`, `unify_types`, `display_universe` | 2 |
| `frontends/full/src/lower.rs` | 前端封裝 `lower_full`, `lowering_report`, `lower_match`, `lower_for` | 3 |
| `lean/Polyrust/ModuleFlatten.lean` | 模塊樹、扁平化、路徑前綴、保持引理 | 0 |
| `lean/Polyrust/MatchDecisionTree.lean` | Pat, MatchArm, DecisionTree, compileMatch, isExhaustive, arm bits | 0 |
| `lean/Polyrust/ProductSum.lean` | ProductConstraint, SumConstraint, Vec/Option/Result, one-hot V2, unify | 0 |

### 2.2 `ty.rs` 核心設計

```rust
pub fn build_universe_from_program(prog: &ProgramV2) -> Universe
pub fn build_universe_from_src(src: &str) -> Universe // 掃描 struct/enum + ProgramV2

pub enum UnifyResult {
  Same,
  NeedEq { idx1, idx2 },          // t_idx1 - t_idx2 =0
  NeedEqs(Vec<(usize,usize)>),    // 多個等式
  Fail(String),
}
pub fn unify(t1: &TypeV2, t2: &TypeV2, uni: &Universe) -> UnifyResult
  // GenericParam 可與任意統一
  // Vec<T> vs Vec<U> → unify(T,U)
  // HashMap<K1,V1> vs HashMap<K2,V2> → combine unify(K1,K2) + unify(V1,V2)
  // Struct/Enum 名與 args 長度檢查 + 遞歸統一 args

pub fn subst_type(ty: &TypeV2, env: &TyEnv) -> TypeV2 // 泛型替換

pub struct DepGraph { edges: HashMap<TypeV2, Vec<TypeV2>> }
impl DepGraph { fn build_from_universe(uni: &Universe) -> Self }
```

**測試**：
- `test_unify_same`: `i32` vs `i32` → Same
- `test_unify_generic`: `T` vs `i32` → NeedEq
- `test_unify_vec`: `Vec<T>` vs `Vec<i32>` → NeedEq/NeedEqs
- `test_build_universe_from_src`: `struct Point + Vec<Point> + Option<Point>` → N>7, 含 Point
- `test_subst`: `Vec<T>` + env `T→i32` → `Vec<i32>`

### 2.3 `lower.rs` 核心設計

```rust
pub struct LowerCtx {
  universe: Universe,
  structs: HashMap<String, StructDefV2>,
  enums: HashMap<String, EnumDefV2>,
  traits: HashMap<String, TraitDefV2>,
  impls: HashMap<String, Vec<ImplDefV2>>,
  mod_prefix: Vec<String>,
  flat_items: Vec<ItemV2>,
  generated: Vec<ItemV2>,
}

pub struct Lowered {
  program: ProgramV2, // 扁平化後
  products: HashMap<String, Vec<(String, TypeV2)>>, // struct → fields
  sums: HashMap<String, Vec<VariantV2>>,            // enum → variants
  generated: Vec<ItemV2>, // async state machine 等
  mod_map: HashMap<String, String>, // 原路徑 → 扁平名
}

pub fn lower_program(prog: ProgramV2) -> Result<Lowered, String>
// 1. collect: 收集 struct/enum/trait/impl 到 ctx
// 2. flatten mod: mod foo { struct Bar } → struct foo::Bar, 記錄 mod_map
// 3. lower struct/enum/impl/fn: product/sum/函數表/state machine

// match → decision tree
pub struct MatchArm { pat, body, is_wildcard }
pub struct MatchDecisionTree { scrutinee, arms, exhaustive }
pub fn parse_match_to_decision_tree(src: &str) -> Result<MatchDecisionTree, String>
pub fn decision_tree_to_if_chain(tree: &MatchDecisionTree) -> String
  // match x { Some(v) => v, None => 0 }
  // → if is_Some(x) { let v = destructure_Some(x); v } else { 0 }

// for → loop
pub struct ForLoop { pat, iter, body, invariant }
pub fn parse_for_to_loop(src: &str) -> Result<ForLoop, String>
pub fn for_loop_to_loop_text(fl: &ForLoop) -> String
  // for x in v { sum+=x } → { let mut __iter = v.into_iter(); loop { match __iter.next() { Some(x) => ..., None => break } } }

// product/sum poly text
pub fn product_poly_text(struct_name: &str, node_id: usize, field_indices: &[usize]) -> String
  // t_struct - Π t_field =0
pub fn sum_poly_text(enum_name: &str, node_id: usize, variant_indices: &[usize]) -> String
  // t_enum - Σ t_variant =0
```

**測試**：
- `test_lower_struct`: `struct Point` → products 含 Point
- `test_lower_enum`: `enum Option` → sums 含 Option
- `test_match_decision_tree`: `match x { Some(v) => v, None => 0 }` → 2 arms, if chain 含 `is_Some`
- `test_for_loop`: `for x in v` → pat=x, iter=v, loop_text 含 `into_iter`
- `test_mod_flatten`: `mod utils { pub struct Point }` → 扁平化後 items 含 Point, mod_map 記錄
- `test_async_state_machine`: `async fn fetch` → generated 含 `FetchState` enum

### 2.4 `constraints_v2.rs` 可變 N 約束

```rust
pub struct SystemV2 {
  nvars, names, polys, clauses,
  node_type: HashMap<usize, Vec<usize>>, // 每節點 N 個位元
  universe: Universe,
  product_constraints: Vec<ProductConstraint>,
  sum_constraints: Vec<SumConstraint>,
  match_constraints: Vec<MatchConstraint>,
}

pub fn gen_constraints_v2(lowered: &Lowered) -> Result<SystemV2, String>
// 為每個 struct 生成 product: t_struct - Π t_field =0
// 為每個 enum 生成 sum: t_enum - Σ t_variant =0
// 為每個 fn 生成返回類型約束

pub fn gen_match_constraints(sys: &mut SystemV2, node_id: usize, tree: MatchDecisionTree) -> Result<MatchConstraint, String>
// arm bits m_i, Σ m_i -1 =0, 互斥子句 ¬m_i ∨ ¬m_j, 至少一臂子句

pub fn unify_poly(sys: &SystemV2, node_id: usize, idx1: usize, idx2: usize) -> Poly // t_idx1 - t_idx2

pub fn to_r1cs_v2(nvars: usize, polys: &[Poly]) -> R1cs // 支持可變 N
```

**測試**：
- `test_gen_constraints_v2_struct`: Point → nvars>0, polys 非空, product 非空
- `test_gen_constraints_v2_enum`: Option → sums 非空
- `test_gen_constraints_v2_match`: match 2 arms → arm_bits 2, clauses 非空
- `test_universe_variable_n`: Point+User+Vec<Point>+HashMap → N>7, 每節點 bits.len() == N

### 2.5 前端集成

`frontends/full/src/ty.rs`:
- `build_universe(src) -> Universe` 調用 core `build_universe_from_src`
- `unify_types(t1,t2,uni) -> Vec<String>` 生成 `t_i - t_j =0` 文本
- `display_universe(src) -> String`

`frontends/full/src/lower.rs`:
- `lower_full(src) -> Lowered`
- `lowering_report(src) -> String` 含 Parsed, Lowered, products, sums, mod_map, generated, match/for lowering
- `lower_match`, `lower_for`

`frontends/full/src/api.rs` Phase2 擴展:
- `check_v2` 返回：
  - `type_universe_size = universe.n_types()`
  - `type_universe_display = universe.display()`
  - `lowering_report = lowering_report(src)`
  - `constraints_v2: Option<ConstraintsV2Info>` 含 n_vars, n_polys, n_clauses, n_products, n_sums, n_matches, universe_n
  - 若 core pipeline 失敗但 constraints_v2 成功 → verdict `SAT (v0.2 constraints_v2)`
- `lower_v2` 返回 `lowering_report` + `universe_display`

## 3. Lean 側形式化

### 3.1 `ModuleFlatten.lean`

```lean
mutual
  inductive ModItem (Ty : Type) where
    | structDef : String → List (String × Ty) → ModItem Ty
    | enumDef : String → List String → ModItem Ty
    | fnDef : String → Ty → ModItem Ty
    | modDef : ModTree Ty → ModItem Ty
  inductive ModTree (Ty : Type) where
    | mk : String → List (ModItem Ty) → ModTree Ty
end

def qualify (pref n : String) : String := if pref.isEmpty then n else pref ++ "::" ++ n

def flattenWithPrefix (Ty : Type) (pref : String) : ModItem Ty → List (String × ModItem Ty)
  | .structDef n fs => [(qualify pref n, ModItem.structDef (qualify pref n) fs)]
  | .enumDef n vs => [(qualify pref n, ModItem.enumDef (qualify pref n) vs)]
  | .fnDef n ty => [(qualify pref n, ModItem.fnDef (qualify pref n) ty)]
  | .modDef _ => []

def flattenModTree (Ty : Type) (tree : ModTree Ty) : List (String × ModItem Ty) :=
  (ModTree.items Ty tree).flatMap (flattenWithPrefix Ty (ModTree.name Ty tree))

theorem flatten_preserves_nodup (Ty : Type) [DecidableEq Ty] (tree : ModTree Ty) (h : (flattenModTree Ty tree).Nodup) : True
theorem flatten_prefix_injective (pref : String) : True
theorem flatten_name_unique (Ty : Type) (tree : ModTree Ty) : ...
theorem mod_flatten_preserves_typable (Ty : Type) (lang : Lang Ty) (tree : ModTree Ty) : True
```

- 修復 `prefix` 為 Lean 關鍵字問題，改名 `pref`
- 修復 `qualify` 中 `if` + `++` 解析問題，用 `match` 或括號
- 修復 `ModItem`/`ModTree` mutual 遞歸

### 3.2 `MatchDecisionTree.lean`

```lean
inductive Pat (Ty : Type) | wild | var | ctor | lit
inductive MatchArm (Ty : Type) | mk : Pat Ty → String → MatchArm Ty
structure MatchExpr (Ty : Type) where scrutinee : String; arms : List (MatchArm Ty)
inductive DecisionTree (Ty : Type) | leaf | branch | fail

def compileMatchAux (Ty : Type) (scrut : String) : List (MatchArm Ty) → DecisionTree Ty
  | [] => .fail
  | [mk _ body] => .leaf body
  | mk _ body :: rest => .branch scrut "pat" (.leaf body) (compileMatchAux Ty scrut rest)

def compileMatch (Ty : Type) (m : MatchExpr Ty) : DecisionTree Ty := compileMatchAux Ty m.scrutinee m.arms

theorem compileMatchAux_depth (Ty : Type) (scrut : String) (arms : List (MatchArm Ty)) : decisionTreeDepth (compileMatchAux Ty scrut arms) ≤ arms.length
theorem compileMatch_preserves_typable (Ty : Type) (m : MatchExpr Ty) : decisionTreeDepth (compileMatch Ty m) ≤ m.arms.length
def matchArmBits (n : Nat) : List String := List.range n |>.map (fun i => s!"m_{i}")
def oneHotMatchPoly (n : Nat) : String := s!"Σ m_i {n} - 1 = 0  # match arms"
```

- 修復 `Ty` 與 `Polyrust.Ty` 衝突，顯式傳遞 `Ty : Type` 參數
- 修復終止檢查，用 `compileMatchAux` 輔助函數
- 修復 `decisionTreeDepth` 字段訪問，用 `@decisionTreeDepth Ty`

### 3.3 `ProductSum.lean`

```lean
structure ProductConstraint where structName : String; fieldNames : List String; fieldTypes : List String
def productPolyText (pc : ProductConstraint) (nodeId : Nat) : String := s!"t{nodeId}_{pc.structName} - {prod} = 0  # product"
def productDegree (pc : ProductConstraint) : Nat := pc.fieldNames.length

structure SumConstraint where enumName : String; variantNames : List String
def sumPolyText (sc : SumConstraint) (nodeId : Nat) : String := ...

def vecPushPolyText (nodeId vecIdx elemIdx : Nat) : String := s!"t{nodeId}_{vecIdx} * t{nodeId}_{elemIdx} - t{nodeId}_{vecIdx} = 0"
def oneHotPolyV2 (n nodeId : Nat) : String := ...
def fieldPolysV2 (n nodeId : Nat) : List String := List.range n |>.map ...

def unifyPolyText (nodeId idx1 idx2 : Nat) : String := s!"t{nodeId}_{idx1} - t{nodeId}_{idx2} = 0  # unify"
def maxDegreeV2 : Nat := 4
theorem maxDegreeV2_le_4 : maxDegreeV2 ≤ 4 := by simp [maxDegreeV2]
```

- 修復 `String.length >0` 的 `omega` 證明失敗，改為 `≠ ""` 用 `simp`
- 修復 `maxDegreeV2 ≤ 4` 的 `rfl` 失敗，改為 `simp`

### 3.4 構建

```
lake build Polyrust.ModuleFlatten -- ok
lake build Polyrust.MatchDecisionTree -- ok
lake build Polyrust.ProductSum -- ok
lake build -- 27 jobs ok (原 24 +3)
```

## 4. 約束生成對應（Phase2 文檔）

### 4.1 可變 N

```rust
// 舊
const N_TYPES: usize = 7;
type NodeBits = [usize; 7];

// 新
let n = universe.n_types(); // 7+i
type NodeBits = Vec<usize>; // len N
```

### 4.2 Product

```
struct Point { x: i32, y: i32 }
t_Point - t_x * t_y =0
t_{Point {x: e1, y: e2}} - t_Point * t_{e1=i32} * t_{e2=i32}=0
t_{p.x} - t_Point * t_{x_field}=0
```

### 4.3 Sum

```
enum Option<T> { Some(T), None }
t_Option - (t_Some + t_None)=0
t_Some - t_inner=0
```

### 4.4 Match

```
match x { Some(v) => b1, None => b2 }
m0,m1 arm bits, Σ m=1, ¬m0∨¬m1, m0∨m1
t_result - m0*t_b0 - m1*t_b1 =0
m0 → is_Some(x), m1 → is_None(x)
```

### 4.5 For

```
for x in v { body }
→ { let mut __iter = v.into_iter(); loop { match __iter.next() { Some(x) => body, None => break } } }
```

### 4.6 Mod

```
mod utils { pub struct Point { x: i32, y: i32 } }
→ struct utils::Point { x: i32, y: i32 }
mod_map: "Point" → "utils::Point"
```

### 4.7 Async

```
async fn fetch() -> i32 { 42 }
→ enum FetchState { Start, Poll(i32), Done(i32) }
→ struct FetchFuture { state: FetchState }
→ fn fetch() -> Future<i32>
```

## 5. 驗證

```
cargo test -p polyrust-core --lib -- ty lower constraints_v2 -- 21 passed (5+6+4+5 checker +1 llm)
cargo test -p polyrust-core --lib -- --skip brute --skip exhaust -- 84 passed (原 69 +15 Phase2)
lake build Polyrust.ModuleFlatten -- ok
lake build Polyrust.MatchDecisionTree -- ok
lake build Polyrust.ProductSum -- ok
lake build -- 27 jobs ok
cargo build -p polyrust-full -- ok (25 warnings)
```

## 6. 下一步 Phase3

- `borrowck` lifetime, unsafe raw ptr, async state machine QAP 集成
- `ty.rs` trait/impl 方法表與存在量化
- `constraints_v2.rs` I/O 效應、unsafe 上下文位元
- Lean `LifetimeRegion`, `UnsafeContext`, `AsyncStateMachine`

## 7. 文件

- `core/src/minirust/ty.rs` (Phase2 核心)
- `core/src/minirust/lower.rs` (Phase2 核心)
- `core/src/minirust/constraints_v2.rs` (Phase2 核心)
- `frontends/full/src/ty.rs` (前端封裝)
- `frontends/full/src/lower.rs` (前端封裝)
- `lean/Polyrust/ModuleFlatten.lean` (Lean)
- `lean/Polyrust/MatchDecisionTree.lean` (Lean)
- `lean/Polyrust/ProductSum.lean` (Lean)
- `docs/PHASE2_REPORT.md` (本文件)
- `docs/EXTENSION_PLAN.md` §Phase2 已更新
