# v0.2 實現方案 — 7+i 全特性

> 基於 Phase1 已完成的 `Universe N=7+i` 與 Lean `TypeUniverse7PlusI`

## 0. 總覽

```
v0.1 core (7 types)  →  Phase1 (7+i Universe, AST v2, Lexer Gt)  →  Phase2 (lowering + constraints)  →  Phase3 (borrowck + QAP)
```

- **core 零依賴**：`core/src/minirust/*` 僅用 `std`，無第三方
- **前端隔離**：`axum, tokio, ureq` 僅在 `frontends/*`
- **Lean 綠**：`Polyrust.TypeUniverse7PlusI` 已構建，`lang17` 20 模組

## 1. 型別系統擴展

### 1.1 Rust 側

`core/src/minirust/universe.rs`

```rust
enum BaseType { I32, Bool, Unit, RefI32, RefMutI32, RefBool, RefMutBool } // 7
enum ExtType {
  Vec(Box<TypeV2>), String, HashMap(Box<TypeV2>,Box<TypeV2>),
  Struct{name, args}, Enum{name, args},
  RawPtr{mutbl, inner}, Future(Box<TypeV2>), Option(Box<TypeV2>), Result(Box<TypeV2>,Box<TypeV2>),
  RefExt{mutbl, inner, lifetime}, GenericParam(String), Assoc{...}, Never
}
enum TypeV2 { Base(BaseType), Ext(ExtType) }

struct Universe { types: Vec<TypeV2>, index_map, base_indices: [usize;7], ext_counts }
impl Universe {
  fn new() -> Self // N=7
  fn insert_closure(&mut self, ty: TypeV2) -> usize // 遞歸插入子類型
  fn n_types(&self) -> usize // N=7+i
  fn one_hot_poly_text(&self, node_id: usize) -> String // Σ t -1
  fn field_polys_text(&self, node_id: usize) -> Vec<String> // t^2 - t
}
```

`parse_type_v2(s: &str) -> TypeV2` 支持：
- `Vec<i32>`, `Vec<String>`, `HashMap<String,i32>`, `Option<T>`, `Result<T,E>`, `*mut T`, `*const T`, `&T`, `&mut T`, `&'a T`, `Future<T>`, `Point`, `Generic`

### 1.2 Lean 側

`lean/Polyrust/TypeUniverse7PlusI.lean`

```lean
inductive BaseTy7 | i32 | bool | unit | refI32 | refMutI32 | refBool | refMutBool
def BaseTy7.all : List BaseTy7 := 7 種
theorem BaseTy7.all_nodup : by native_decide
theorem BaseTy7.all_complete

inductive ExtTag | vec | string | hashmap | structTy | enumTy | rawPtrMut | rawPtrConst | future | option | result
def ExtTag.all : List ExtTag := 10 種

inductive Ty7Plus10 | base : BaseTy7 → Ty7Plus10 | ext : ExtTag → Ty7Plus10
def Ty7Plus10.all := (Base.all.map base) ++ (Ext.all.map ext) -- N=17
def mkUniverse7PlusI (exts : List ExtTag) : List Ty7Plus10 := (Base.all.map base) ++ (exts.map ext)
theorem mkUniverse7PlusI_length : length = 7 + exts.length
theorem mkUniverse7PlusI_nodup : Nodup 若 exts Nodup
def lang17 : Lang Ty7Plus10 := { enumAll := all, nodup, complete, numTy := base i32, eqbTy := base bool, num_ne_eqb }
theorem oneHot_decompose : Σ_{7+i} = Σ_7 + Σ_i
theorem base_one_hot_implies_ext_zero
theorem one_hot_unique_17
```

## 2. AST 擴展

`core/src/minirust/ast_v2.rs`

```rust
struct StructDefV2 { name, generics, lifetimes, fields: Vec<(String, TypeV2)>, is_pub }
struct EnumDefV2 { name, variants: Vec<VariantV2>, ... }
struct FnSigV2 { name, params: Vec<(String, TypeV2)>, ret: TypeV2, is_pub, is_unsafe, is_async, is_method }
enum ItemV2 { Struct, Enum, Fn, Impl, Trait, Mod, Use, Macro }
struct ProgramV2 { items, main, universe: Universe }
impl ProgramV2::parse_v2(src: &str) -> Result<Self, String>
```

已實現測試：
- `test_parse_struct`: `struct Point { x: i32, y: i32 }`
- `test_parse_enum`: `enum Option<T> { Some(T), None }`
- `test_parse_full`: 含 struct+enum+trait+impl+fn

## 3. 詞法擴展

`core/src/minirust/lexer.rs`

- 新增 `Tok::Gt` (`>`)，原來孤立 `>` 報錯，現返回 `Gt`，支持 `Vec<T>` 泛型閉合
- 支持 `'` 開頭生命週期 `Ident` (`'a`)
- 支持 `::` 拆為兩個 `Colon`
- 擴展關鍵字：`struct enum impl trait mod use pub unsafe async await loop while for match where dyn Self super crate self`
- `show` 處理 `Gt`

兼容性：v0.1 程序仍解析成功

`core/src/minirust/parse_v2.rs`

- `lex_v2`: 在原 `lex` 上把擴展關鍵字標為 `Kw`
- `extract_type_tokens`: 處理 `< >` 嵌套深度，`Gt` 遞減
- `build_universe_from_src`: 從源碼粗略提取類型，構造宇宙

## 4. 各特性實現方案

### 4.1 struct

**語法**：
```poly
pub struct Point { x: i32, y: i32 }
pub struct User<T> { id: i32, data: T }
```

**AST**：`StructDefV2`

**Lowering**（Phase2 `lower.rs`）：
- `struct Point { x: i32, y: i32 }` → product type
- 約束：`t_{v,Point} - t_{x}*t_{y}=0`，字段訪問 `p.x` → 投影多項式
- 構造 `Point { x: 3, y: 4 }` → `let tmp_x = 3; let tmp_y = 4; let p = mk_product(tmp_x, tmp_y)`

**多項式**：
```
# Point struct
t_Point - t_x * t_y =0
# field access
t_{p.x} - t_p * t_x_field =0
```

**Lean**：`Ty7Plus10.ext .structTy`，`product` 編碼引理

### 4.2 enum

**語法**：
```poly
enum Option<T> { Some(T), None }
enum Status { Active, Inactive, Pending(i32), WithData { code: i32, msg: String } }
```

**AST**：`EnumDefV2` + `VariantV2`

**Lowering**：
- `enum Option<T>` → sum type，`Option::Some(5)` → `inl 5`, `None` → `inr unit`
- `match` 決策樹見 §4.5

**多項式**：
```
t_Option - (t_Some + t_None)=0
t_Some - t_inner=0
```

### 4.3 impl / trait

**語法**：
```poly
trait Display { fn fmt(self) -> String; }
impl Display for Point { fn fmt(self) -> String { ... } }
impl Point { fn new(x: i32, y: i32) -> Point { Point { x, y } } }
```

**AST**：`TraitDefV2`, `ImplDefV2`

**Lowering**：
- `trait` → trait bound 子句，`impl Trait for Type` → 函數表
- `impl` 方法 → 去糖為 `fn Point_new(x: i32, y: i32) -> Point`
- `self` → 第一個參數
- trait object `dyn Display` → 存在類型 + vtable（Phase3）

**約束**：
```
# trait bound: 若 T: Display 則 t_{T,Display}=1
# impl: fn table entry
t_{Point::fmt} - t_{Point}*t_{Display}=0
```

### 4.4 Vec / String / HashMap

**語法**：
```poly
let mut v: Vec<i32> = Vec::new();
v.push(1);
let s: String = String::new();
let mut m: HashMap<String, i32> = HashMap::new();
```

**Lowering**：
- `Vec<T>` → `*mut T + len + cap` 三元組，`Vec::new()` → `null + 0 + 0`
- `push` → `len+1` + `*mut` 寫
- `String` → `Vec<u8>` 特化，`String::new()` 同 `Vec::new()`
- `HashMap<K,V>` → `Vec<(K,V)> + hash_fn`，簡化為 `Vec` + 約束 `k1 != k2`

**多項式**：
```
# Vec
t_Vec - t_ptr * t_len * t_cap =0
# len
t_len' - t_len -1 =0
```

**QAP**：`Vec` 操作度 ≤3，仍可 Gröbner

### 4.5 loops: loop / while / for

**契約**：`for`/`while` 需不變式（loop contract）

**語法**：
```poly
loop { if cond { break; } }
while x > 0 { x = x -1; }
for x in v { sum = sum + x; } # @invariant sum == Σ_{i<iter} v[i]
```

**Lowering**：
- `loop` → `while true`
- `while cond { body }` → `loop { if !cond { break; } body }` + 不變式
- `for x in iter { body }` → `let mut it = iter.into_iter(); loop { match it.next() { Some(x) => body, None => break } }`

**約束**：
```
# loop 不變式 I
I_entry ∧ (I ∧ cond → I_body) ∧ (I ∧ ¬cond → post)
# 編碼為多項式
t_I_entry * t_I - t_I_body =0
```

**前端**：`frontends/full` 檢測 `loop/while/for`，要求 `@invariant`

### 4.6 match

**語法**：
```poly
let y = match x {
  Some(v) => v,
  None => 0,
  _ => 0,
};
match status {
  Active => 1,
  Inactive => 0,
  Pending(code) => code,
  WithData { code, msg } => code,
}
```

**Lowering**：`match` → if 決策樹

```rust
match x {
  Some(v) => e1,
  None => e2,
}
→
if is_Some(x) { let v = unwrap_Some(x); e1 } else { e2 }
```

**多項式**：
```
t_match - (t_Some * t_e1 + t_None * t_e2)=0
t_Some + t_None -1=0
```

**Lean**：`MatchDecisionTree` 模組，證明決策樹與原 `match` 等價

### 4.7 mod tree

**語法**：
```poly
mod utils {
  pub struct Point { x: i32, y: i32 }
  pub fn make_point() -> Point { Point { x: 0, y: 0 } }
}
mod inner;
pub use utils::Point;
```

**Lowering**：
- `mod foo { ... }` → 扁平化，`foo::Point` → `foo_Point`
- `mod foo;` → 讀 `foo.poly` 或 `foo/mod.poly`
- `use` → 路徑替換

**實現**：
```rust
fn flatten_mod(mod_def: ModDefV2, prefix: &str) -> Vec<ItemV2> {
  mod_def.items.into_iter().map(|item| item.with_prefix(prefix)).collect()
}
```

**Lean**：`ModuleFlatten` 模組，證明扁平化保持 `TypableG`

### 4.8 async

**語法**：
```poly
async fn fetch(id: i32) -> Result<User, String> { ... }
fn main() { let fut = fetch(1); let user = fut.await; }
```

**Lowering**：async → 狀態機 enum

```rust
async fn fetch(id: i32) -> i32 { 42 }
→
enum FetchState { Start(i32), Poll(i32), Done(i32) }
fn fetch(id: i32) -> Future<i32> { Future { state: Start(id) } }
fn poll(fut: Future<i32>) -> Poll<i32> { match fut.state { Start(id) => Poll::Ready(42), ... } }
```

**多項式**：狀態機轉移 `t_state' - t_state * t_poll =0`

### 4.9 I/O

**語法**：
```poly
println!("hello {}", x);
let file = File::open("data.txt");
let content = file.read_to_string();
```

**Lowering**：
- `println!` → 宏展開為 `io::stdout().write()`
- `File` → `*mut FileHandle` + 路徑
- I/O 操作在約束中為 `opaque`，僅檢查類型，不生成 QAP（`@trusted`）

**編碼**：
```
# I/O 為外部函數，標為 trusted，跳過 QAP
# @trusted
fn println(s: String) { }
```

### 4.10 unsafe

**語法**：
```poly
unsafe fn raw_access(p: *mut i32) -> i32 { *p }
fn main() {
  let mut x = 5;
  let p: *mut i32 = &mut x as *mut i32;
  unsafe { *p = 10; }
}
```

**Lowering**：
- `*mut T` → `RawPtr{mutbl:true, inner:T}`，`TypeV2::Ext(ExtType::RawPtr)`
- `unsafe { ... }` 塊內 `*p` 解引用不生成 borrow 約束
- `&mut x as *mut i32` → `raw_from_mut`

**約束**：
```
# raw ptr 無 borrow
t_raw - t_inner =0
# unsafe deref
t_deref - t_raw =0  # 無 borrow check
```

**安全**：`unsafe` 塊需 `@unsafe` 標註，前端 UI 警告

### 4.11 lifetime

**語法**：
```poly
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }
fn foo<'a, 'b: 'a>(x: &'a i32, y: &'b i32) -> &'a i32 { x }
```

**AST**：`RefExt { lifetime: Option<String>, mutbl, inner }`

**Lowering**（Phase3）：
- 生命週期參數 → region 變量
- `'a: 'b` → region inclusion `region_a ⊇ region_b`
- borrowck V2：`&'a mut T` 活時，`T` 不可別名

**約束**：
```
# region inclusion
t_region_a - t_region_b * t_incl =0
# borrow live
t_borrow_live - t_ref * t_region =0
```

**Lean**：`LifetimeRegion` 模組，證明 region 包含傳遞性

## 5. 約束生成擴展

### 5.1 可變 N

```rust
// 舊
const N_TYPES: usize = 7;
// 新
let n_types = universe.n_types(); // 7+i
```

`constraints.rs`：

```rust
fn one_hot_poly(node_id: usize, n: usize) -> Poly {
  let mut poly = Poly::zero();
  for k in 0..n { poly += var(node_id, k); }
  poly - 1
}
fn field_polys(node_id: usize, n: usize) -> Vec<Poly> {
  (0..n).map(|k| var(node_id,k)*var(node_id,k) - var(node_id,k)).collect()
}
```

### 5.2 Product / Sum

```rust
fn product_poly(struct_ty: &StructDefV2, node_id: usize) -> Poly {
  // t_struct - Π t_field =0
}
fn sum_poly(enum_ty: &EnumDefV2, node_id: usize) -> Poly {
  // t_enum - Σ t_variant =0
}
```

### 5.3 Borrow / Lifetime

```rust
fn borrow_poly(ref_ty: TypeV2, node_id: usize) -> Poly {
  // t_ref - t_inner * t_region =0
}
```

## 6. QAP 處理

- `n_vars = Σ_nodes N + aux`
- `bits_per_node = N`
- 對 `N=17`，每節點 17 變量，17 域多項式 +1 one-hot，L0 嵌入保證度 ≤2
- `Vec<T>` 操作度 ≤3，`HashMap` 度 ≤4，仍可 Gröbner 基化簡（Buchberger 參數 `fuel` 控制）
- `async` 狀態機度 ≤2，`unsafe` 度 ≤1

## 7. 前端隔離

```
core/                 -- 零依賴
  src/minirust/
    universe.rs       -- TypeV2, Universe
    ast_v2.rs         -- Struct/Enum/Impl/Trait/Mod
    parse_v2.rs       -- 擴展解析
    lexer.rs          -- Gt, Lifetime

frontends/
  full/               -- axum/tokio，調用 core Universe，/api/v2/check 返回 type_universe_size
    build.rs          -- emit Lean links
    src/api.rs        -- check_v2, lower_v2
    src/ir.rs         -- detect_features, estimate_type_universe 用 Universe
    src/encoding.rs   -- 各特性編碼示例
  http/               -- axum
  llm/                -- ureq
```

## 8. Lean 形式化

- `TypeUniverse7PlusI.lean`: `BaseTy7` 7, `ExtTag` 10, `Ty7Plus10` 17, `mkUniverse7PlusI`, `lang17`, `oneHot_decompose`, `one_hot_unique_17`
- 與 `T9Generalized.lean` 銜接：`typable_iff_rootG`, `genC_soundG`, `genC_completeG` 直接可用
- Phase2 計劃：`ModuleFlatten`, `MatchDecisionTree`, `LifetimeRegion`, `VecEncoding`

## 9. 測試

```
cargo test -p polyrust-core --lib -- universe ast_v2 parse_v2 -- 12 passed
cargo test -p polyrust-core --lib -- --skip brute --skip exhaust -- 69 passed
lake build Polyrust.TypeUniverse7PlusI -- ok
lake build -- 24 jobs ok
```

## 10. 示例 .poly

見 `examples/demo_7plusI.poly`，含 struct, enum, Vec, String, HashMap, Option, raw ptr, lifetime, async, match, for

## 11. 下一步

- [x] Phase1: AST + Universe + Lexer
- [ ] Phase2: ty.rs unify, lower.rs struct/enum/match/loop/mod flatten, constraints.rs N 可變
- [ ] Phase3: borrowck lifetime, unsafe raw ptr, async state machine, QAP 集成
- [ ] Phase4: 前端 UI 展示 N, Lean 完整 ModuleFlatten + MatchDecisionTree
```
