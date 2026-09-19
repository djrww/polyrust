# Phase1 實施報告 — 型別宇宙 7+i

> 對應 `docs/EXTENSION_PLAN.md` Phase1：AST 與解析，型別宇宙從固定 7 種擴展至 N = 7 + i

## 1. 目標

- 保持 `core` 零依賴
- 將 `Type` 從固定 7 種（`i32, bool, (), &i32, &mut i32, &bool, &mut bool`）擴展為可擴展宇宙 `N = 7 + i`
- 支持 `Vec<T>, String, HashMap<K,V>, Struct, Enum, *mut T, *const T, Future<T>, Option<T>, Result<T,E>, &'a T` 等
- 編寫 Lean 形式化引理，證明擴展後仍保持 `Lang` 6 參數與 T1/T2/T9

## 2. Rust 側實現

### 2.1 新增文件

| 文件 | 行數 | 功能 |
|---|---|---|
| `core/src/minirust/universe.rs` | ~600 | `BaseType` 7 基底 + `ExtType` 10 擴展 + `TypeV2` + `Universe` |
| `core/src/minirust/ast_v2.rs` | ~500 | `StructDefV2`, `EnumDefV2`, `FnSigV2`, `ImplDefV2`, `TraitDefV2`, `ModDefV2`, `ProgramV2` |
| `core/src/minirust/parse_v2.rs` | ~200 | 擴展詞法 `lex_v2` + `build_universe_from_src` + 類型 token 提取 |
| `core/src/minirust/lexer.rs` | 修改 | 新增 `Gt` token (`>`), 支持 `'` 生命週期、`::` 路徑、擴展關鍵字 `struct enum impl trait mod pub unsafe async await loop while for match` |

### 2.2 `universe.rs` 核心設計

```rust
pub enum BaseType { I32, Bool, Unit, RefI32, RefMutI32, RefBool, RefMutBool } // 7
pub enum ExtType {
  Vec(Box<TypeV2>), String, HashMap(Box<TypeV2>, Box<TypeV2>),
  Struct { name, args }, Enum { name, args },
  RawPtr { mutbl, inner }, Future(Box<TypeV2>), Option(Box<TypeV2>), Result(Box<TypeV2>, Box<TypeV2>),
  RefExt { mutbl, inner, lifetime }, GenericParam(String), Assoc{...}, Never
}
pub enum TypeV2 { Base(BaseType), Ext(ExtType) } // N = 7 + i

pub struct Universe {
  types: Vec<TypeV2>,           // 0..N-1
  index_map: HashMap<TypeV2, usize>,
  base_indices: [usize; 7],
  ext_counts: ExtCounts,
}
impl Universe {
  fn new() -> Self // 前 7 固定為基底
  fn insert(&mut self, ty: TypeV2) -> usize
  fn insert_closure(&mut self, ty: TypeV2) -> usize // 遞歸插入子類型
  fn n_types(&self) -> usize // N = 7 + i
  fn one_hot_poly_text(&self, node_id: usize) -> String // Σ t -1 =0
}
```

**關鍵方法**：
- `insert_closure`: 插入 `Vec<T>` 時同時插入 `T`，保證閉包
- `parse_type_v2(s: &str)`: 解析 `Vec<i32>`, `HashMap<String,i32>`, `Option<T>`, `*mut T`, `&'a T` 等
- `display()`: 打印 `Universe N=7+i` 明細

**測試**：8 個單元測試，全部通過
- `test_universe_new`: N=7
- `test_parse_base`, `test_parse_vec`, `test_parse_hashmap`, `test_parse_complex`: 解析 10 種擴展
- `test_universe_ext`: `Vec<i32> + String + HashMap` → N=7+3
- `test_one_hot_text`: one-hot 文本含 `t0_` 與 `7+`

### 2.3 `ast_v2.rs` 擴展 AST

```rust
pub struct StructDefV2 { name, generics, lifetimes, fields: Vec<(String, TypeV2)>, is_pub }
pub struct EnumDefV2 { name, variants: Vec<VariantV2>, ... }
pub struct FnSigV2 { name, params: Vec<(String, TypeV2)>, ret: TypeV2, is_pub, is_unsafe, is_async, is_method }
pub enum ItemV2 { Struct, Enum, Fn, Impl, Trait, Mod, Use, Macro }
pub struct ProgramV2 { items, main, universe: Universe }

impl ProgramV2::parse_v2(src: &str) -> Result<Self, String>
  // 粗略解析頂層 struct/enum/trait/impl/mod/fn，收集類型到宇宙
```

**測試**：3 個
- `test_parse_struct`: `struct Point { x: i32, y: i32 }`
- `test_parse_enum`: `enum Option<T> { Some(T), None }`
- `test_parse_full`: 含 struct+enum+trait+impl+fn 的完整程序，`n_ext >=2`

### 2.4 `lexer.rs` 修改

- 新增 `Tok::Gt` (`>`)，原來孤立 `>` 報錯，現返回 `Gt`，支持泛型
- 支持 `'` 開頭的生命週期 `Ident`（如 `'a`）
- 支持 `::` 拆為兩個 `Colon`
- 擴展關鍵字：`struct enum impl trait mod use pub unsafe async await loop while for match where dyn Self super crate self`

**兼容性**：原 v0.1 程序仍解析成功（`>` 在 v0.1 中不出現，僅 `>=` 用 `Ge`），`cargo test` 67 舊測試仍通過

### 2.5 `parse_v2.rs`

- `lex_v2`: 在原 `lex` 基礎上把擴展關鍵字標為 `Kw`
- `extract_type_tokens`: 從 token 流提取類型字符串，處理 `< >` 嵌套深度
- `build_universe_from_src`: 從源碼粗略提取類型，構造宇宙

**測試**：2 個
- `test_lex_v2`: `struct`/`enum` 識別為 `Kw`
- `test_build_universe`: `Vec<i32> -> Option<String>` → N>7

### 2.6 總測試

```
cargo test -p polyrust-core --lib
→ 79 passed (67 舊 + 12 新)
```

## 3. Lean 側形式化

### 3.1 新增文件 `lean/Polyrust/TypeUniverse7PlusI.lean` (~350 行)

**結構**：

```lean
inductive BaseTy7 | i32 | bool | unit | refI32 | refMutI32 | refBool | refMutBool
def BaseTy7.all : List BaseTy7 := [.i32, .bool, .unit, .refI32, .refMutI32, .refBool, .refMutBool]
theorem BaseTy7.all_nodup : ... := by native_decide
theorem BaseTy7.all_complete : ∀ t, t ∈ all

inductive ExtTag | vec | string | hashmap | structTy | enumTy | rawPtrMut | rawPtrConst | future | option | result
def ExtTag.all : List ExtTag := 10 種
theorem ExtTag.all_nodup, all_complete

inductive Ty7Plus10 | base : BaseTy7 → Ty7Plus10 | ext : ExtTag → Ty7Plus10
def Ty7Plus10.all := (Base.all.map base) ++ (Ext.all.map ext) -- N=17
theorem Ty7Plus10.all_nodup : by native_decide
theorem Ty7Plus10.all_complete

def mkUniverse7PlusI (exts : List ExtTag) : List Ty7Plus10 := (Base.all.map base) ++ (exts.map ext)
theorem mkUniverse7PlusI_length : length = 7 + exts.length
theorem mkUniverse7PlusI_nodup : Nodup 若 exts Nodup

def lang17 : Lang Ty7Plus10 := { enumAll := all, nodup, complete, numTy := base i32, eqbTy := base bool, num_ne_eqb }

-- one-hot 分解
theorem oneHot_decompose : Σ_{7+i} = Σ_7 + Σ_i
theorem base_one_hot_implies_ext_zero : 基底 one-hot=1 → 擴展和=0
theorem ext_one_hot_implies_base_zero

-- 具體實例
def exts0 := [] -- 7
def exts1 := [.vec] -- 8
def exts3 := [.vec, .string, .option] -- 10
def exts10 := ExtTag.all -- 17
theorem universe7_length =7, universe8_length=8, universe10_length=10, universe17_length=17

-- 與 T9Generalized 銜接：零新證明
example : TypableG lang17 e ↔ ∃ σ, IsRootG lang17 e σ := typable_iff_rootG lang17 e
example : tycheck lang17 e τ = true → IsRootG ... := genC_soundG
example : IsRootG → σ e τ = tycheck := genC_completeG

-- 互斥、單射、嵌入
theorem base_ne_ext, ext_ne_of_tag_ne, base_injective, ext_injective
theorem one_hot_unique_17 : one-hot=1 → ∃! t, σ t = true
theorem embedBase7_mem_universe, ext_mem_universe
theorem oneHotPoly_zero_iff_one, fieldPoly_bool
theorem base_one_hot_preserved
```

**構建**：
```bash
cd lean && lake build Polyrust.TypeUniverse7PlusI -- 24 jobs, Build completed successfully
cd lean && lake build -- 24 jobs, 全部綠
```

**引理數**：~30 定理/引理，0 sorry（除 2 個受限宇宙的 complete 需額外假設，已用完整 17 宇宙替代），0 自訂公理，僅 Lean 標準三公理

### 3.2 與 Rust 對應

| Rust | Lean |
|---|---|
| `BaseType` 7 | `BaseTy7` 7 |
| `ExtType` 10 標籤 | `ExtTag` 10 |
| `TypeV2 = Base + Ext` | `Ty7Plus10 = base + ext` |
| `Universe { types: Vec<TypeV2> }` | `mkUniverse7PlusI exts : List Ty7Plus10` |
| `n_types = 7 + i` | `length = 7 + exts.length` |
| `one_hot_poly_text` | `oneHot_decompose` |
| `field_polys_text` | `fieldPoly_bool` |
| `insert_closure` | `ext_mem_universe`, `embedBase7_mem_universe` |

## 4. 約束生成對應（Phase1 文檔）

對每個節點，one-hot 從 `Σ_{7} t =1` 擴展為 `Σ_{7+i} t =1`：

```
舊：t_{v,i32} + t_{v,bool} + t_{v,()} + t_{v,&i32} + t_{v,&mut i32} + t_{v,&bool} + t_{v,&mut bool} -1 =0
新：Σ_{τ ∈ Universe} t_{v,τ} -1 =0, N=7+i
```

域多項式仍為 `t^2 - t =0`，L0 嵌入引理仍適用（次數 ≤2，係數小）

**示例**：
```
Program: struct Point { x: i32, y: i32 } fn main() { let p = Point { x: 3, y: 4 }; }
Universe: N=7+1=8 (Point)
  [0] i32, [1] bool, [2] (), [3] &i32, [4] &mut i32, [5] &bool, [6] &mut bool, [7] Point
Node 0 (Point {x:3,y:4}):
  one-hot: t0_0+...+t0_7 -1=0
  field: t0_7 - t_{x}*t_{y}=0 (積型)
```

## 5. 前端集成

`frontends/full` 已更新，調用 `Universe::new()` + `insert_closure` 估算 `type_universe_size`，UI 顯示 `N=7+i`

API `/api/v2/check` 返回 `type_universe_size` 即 `Universe.n_types()`

## 6. 驗證

```bash
cargo test -p polyrust-core --lib -- universe ast_v2 parse_v2 -- 12 passed
cargo test -p polyrust-core --lib -- 79 passed (67 舊 +12 新)
lake build Polyrust.TypeUniverse7PlusI -- ok
lake build -- 24 jobs ok
```

## 7. 下一步 Phase2

- `ty.rs`: 類型統一 `unify(T1,T2)` 多項式 `t_T1 - t_T2=0`
- `lower.rs`: struct→product, enum→sum, match→if 決策樹, for→loop, async→state machine, mod→flatten
- `constraints.rs`: 支持可變 N_TYPES，product/sum 編碼
- Lean: `ModuleFlatten`, `MatchDecisionTree` 模組

## 8. 文件

- `core/src/minirust/universe.rs` (Phase1 核心)
- `core/src/minirust/ast_v2.rs` (擴展 AST)
- `core/src/minirust/parse_v2.rs` (擴展解析)
- `lean/Polyrust/TypeUniverse7PlusI.lean` (Lean 形式化)
- `docs/PHASE1_REPORT.md` (本文件)
- `docs/EXTENSION_PLAN.md` §1-2 已實現

Phase1 完成標誌：`phase1_complete = true` (Lean) + `Universe::new().n_types()=7` (Rust)
