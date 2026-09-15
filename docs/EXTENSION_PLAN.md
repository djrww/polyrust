# polyrust v0.2 擴展方案 — 全 Rust 特性家族

> 基於 v0.1.5 核心（零依賴 `polyrust-core` + `frontends/*`）擴展至  
> `struct/enum/impl/trait`、`Vec/String/HashMap`、`loop/while/for`、`match`、模組樹、`async`、`I/O`、`unsafe`、`lifetime 參數`

本文件是**可執行的設計藍圖**，同時附帶新前端 `frontends/full` 的原型實作。

---

## 0. 現狀盤點（v0.1.5）

| 層 | 實現 | 約束編碼 |
|---|---|---|
| AST | `Type` 7 種 (i32,bool,(),&i32,&mut i32,&bool,&mut bool) | `node_type: HashMap<usize, [usize; N]>` |
| 表達式 | Int/Bool/Unit/Var/Let/Seq/BinOp/Not/Neg/If/Ref/RefMut/Deref/AssignVar/AssignDeref/Call/Invoke | 每節點 one-hot Σ t =1 + 規則多項式 t_v - out^R =0 |
| 宏 | `macro_rules!` 多臂，存在量化語義 | 臂位元 a_i Σ a=1，約束乘 a_i |
| 借用 | `analyze` 區間衝突 | b_j -1=0, b_i·b_j=0 + 子句 ¬b_i∨¬b_j |
| 管線 | CDCL(T) ↔ Buchberger ↔ QAP | 已有 |
| Lean | 20 模組 408 定理 | ProductReduction / SumReduction 已覆蓋積/和型雛形 |

瓶頸：`Type` 是平坦枚舉，無法表達參數化、複合、泛型、生命週期。

---

## 1. 總體架構決策

### 1.1 保持核心承諾
`core` 依然零第三方依賴。新增 `core/src/minirust/ast_v2.rs`（可選 feature `full`）只用 `std`。所有重型前端（axum/tokio/serde）留在 `frontends/full`。

### 1.2 .poly DSL v0.2 語法擴展

```
.poly v0.2 = v0.1 語法 + 下列頂層

# @intent, @import, @set 保持
# 新增契約註解
# @invariant: x < 10
# @requires: n > 0
# @ensures: result >= 0
# @unsafe-allowed
# @lifetime 'a: 'b

struct Point { x: i32, y: i32 }
enum Option<T> { Some(T), None }
enum Result<T,E> { Ok(T), Err(E) }

impl Point { fn new(x: i32, y: i32) -> Point { ... } fn norm(&self) -> i32 { ... } }

trait Display { fn fmt(&self) -> String; }
impl Display for Point { ... }

mod foo { pub struct Bar { ... } pub mod baz { ... } }

fn main() {
  let v: Vec<i32> = Vec::new();
  v.push(1);
  let s = String::from("hi");
  let m: HashMap<String,i32> = HashMap::new();

  loop { if cond { break; } }
  while x < 10 { x = x + 1; }
  for i in 0..10 { ... }  // desugar 為 IntoIterator

  match opt { Some(x) => x, None => 0 }

  async fn fetch() -> i32 { ... }
  let x = fetch().await;

  unsafe { let p: *mut i32 = &mut x as *mut i32; *p = 5; }

  fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { ... }
}
```

解析策略：**兩階段**
1. 表面解析（surface parse）→ 保留所有 Rust 語法，生成 `SurfaceAST`
2. 降維（lowering）→ desugar 至核心 IR（MiniRustFull）：for→loop+IntoIterator, async→state machine, match→if+解構, impl→函式表

### 1.3 類型宇宙 v0.2

```rust
// core/src/minirust/ast_v2.rs 草案

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TyV2 {
  I32, Bool, Unit, Str, // Str = &str 特殊
  Ref { mutbl: bool, ty: Box<TyV2>, lt: Lifetime },
  RawPtr { mutbl: bool, ty: Box<TyV2> }, // unsafe
  Struct { name: String, args: Vec<TyV2>, fields: Vec<(String, TyV2)> },
  Enum { name: String, args: Vec<TyV2>, variants: Vec<Variant> },
  Vec(Box<TyV2>),
  String,
  HashMap(Box<TyV2>, Box<TyV2>),
  Fn(Vec<TyV2>, Box<TyV2>),
  TraitObject(String),
  GenericParam(String),
  Assoc { self_ty: Box<TyV2>, trait_name: String, assoc: String },
  Future(Box<TyV2>), // async
  Result(Box<TyV2>, Box<TyV2>),
  Option(Box<TyV2>),
  Never, // !
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lifetime(pub String); // 'a 或 'static 或匿名

pub struct Variant { pub name: String, pub fields: Vec<TyV2> }
```

關鍵：不再是固定 N_TYPES，而是**按需構造類型宇宙**。對每個程序，收集所有出現的 `TyV2` → 閉包（子類型、泛型實例化）→ 去重 → 得到 `N_TYPES_program`。若 `N_TYPES_program` ≤ 64，保持 one-hot；若 >64，改用二進制編碼（log2 bits）+ 解碼表，以控制多項式數量。

### 1.4 約束生成 v0.2 策略

延續現有 `System`，但 `node_type` 改為 `Vec<usize>` 長度可變。

| 特性 | 編碼方式 | 多項式/子句 |
|---|---|---|
| **struct** | 積型 (ProductReduction)：`t_{struct}` = ∏ t_{field_i}（AND 語義），字段訪問 `e.field` 的類型 = 投影 | `t_s - ∏ t_fi =0`, 字段約束 `t_{e.field} - t_field =0` |
| **enum** | 和型 (SumReduction)：`t_{enum}` = Σ t_{variant}`（OR），tag bits 另加 | `t_enum - Σ t_var =0`, tag互斥子句 |
| **impl** | 方法表：`impl Type { fn foo(self, ...) }` → 去糖為 `fn Type_foo(self: Type, ...)`，`obj.foo()` → `Type_foo(obj)` | 無新多項式，複用 Call |
| **trait** | trait bound = 子句：`T: Display` → 存在 impl 記錄。`dyn Trait` 用存在量化位元 `impl_exists_{T,Trait}` | 子句 `¬has_impl ∨ method_available` |
| **Vec<T>** | 容器類型構造：`Vec<T>` 位元 = f(T位元)。操作 `push` : `Vec<T> × T → ()`, `get: Vec<T> × i32 → Option<T>` | 操作規則多項式 + 泛型統一 |
| **String/HashMap** | 視為 `Vec<u8>` / `HashMap<K,V>` 特殊化，同上 | 同上 |
| **loop** | 有界模型檢測：展開 K 次（K= @fuel 或預設 3）+ loop invariant。`loop { body }` → `let _fuel = K; while _fuel>0 { body; _fuel-=1 }` + 驗證 invariant 歸納 | 額外 counter 變量 + invariant 多項式 |
| **while/for** | `while cond { body }` → `if cond { body; while cond { body } }` 展開；`for` → `IntoIterator::into_iter` + `next` loop | 同 loop |
| **match** | 解糖為決策樹：`match e { Pat1 => b1, Pat2 => b2 }` → `if is_variant1(e) { let fields = destructure(e); b1 } else if ...`。引入 match arm 位元 `m_i` Σ m=1 | `t_result - Σ m_i·t_branch_i =0`, pattern type check |
| **mod** | 名稱解析階段扁平化：`mod a { pub struct B }` → 全限定 `a::B`。visibility 用位元 `vis_{path}` | 解析期，無多項式；若私有訪問非法 → 強制矛盾（同未綁定變量） |
| **async** | `async fn foo() -> T` → `fn foo() -> Future<T>`，state machine 有 `Poll::Ready(T)` / `Pending`。`await` 要求 `Future` trait | `Future<T>` 類型構造 + `await` 規則 `t_{await e} = T where e: Future<T>` |
| **I/O** | 效應系統：`println!`, `File::open` 建模為 `fn(...) -> Result<T,E>` + 效應位元 `eff_io`。預設允許，但若 `# @no-io` 則禁止 | 效應位元子句 |
| **unsafe** | 上下文位元 `in_unsafe`。`*mut T`, `*const T` 新類型。unsafe 操作（解原始指針、調 `unsafe fn`）要求 `in_unsafe=1` | `t_unsafe_op·(1 - in_unsafe)=0` → 強制矛盾若不在 unsafe |
| **lifetime 'a** | 生命週期約束圖：`'a: 'b` (outlives) 邊。擴展 `BorrowAnalysis` 至 NLL：region 變量 `r`，衝突條件改為 region 重疊 + outlives | `lifetime` 子句 `¬(r1 overlaps r2) ∨ ¬(r1:>r2)`，借用位元乘 lifetime 位元 |

所有新增約束仍保持**次數 ≤3、係數小整數**，故 L0 嵌入引理仍適用，𝔽_p 保真。

---

## 2. 分特性詳細設計

### 2.1 struct / enum / impl / trait

**解析**：增加 `parse_struct`, `parse_enum`, `parse_impl`, `parse_trait`。`struct` 字段可含 `pub`, 泛型 `<T>`, where。

**類型檢查**：
- struct 構造 `Point { x: 1, y: 2 }`：每個字段類型匹配 + 完整性（無多餘/缺失）
- 字段訪問 `p.x`：要求 `p: Struct` 且字段存在
- enum 構造 `Option::Some(1)`：要求泛型實例化一致
- impl 方法解析：先查 `impl` 表，再查 `trait` 預設方法

**多項式**：
```
struct Point { x:i32, y:i32 }
t_Point = t_x_i32 * t_y_i32   (product)
t_{Point {x: e1, y:e2}} = t_Point * (t_{e1}=i32) * (t_{e2}=i32)
t_{e.x} = t_{Point} * t_{x_field}
```

enum 用 sum：
```
t_Option_i32 = t_Some + t_None - t_Some*t_None (互斥和，one-hot 保證互斥)
```

trait 用存在量化：
```
∃ impl T: Display  ↔  impl_bit_{T,Display}=1
method call obj.fmt() 要求 impl_bit=1，否則矛盾
```

**Lean**：擴展 `ProductReduction`/`SumReduction` 至帶字段名記錄，證明 `typable_pair_iff` 推廣到 n 元組，`sumBits` 推廣到 n 變體。

### 2.2 Vec / String / HashMap

視為**內建泛型 ADT**，在 `std` 中預定義：

```poly
# @import: vec
# @import: string
# @import: hashmap
```

`std/vec.poly`：
```
struct Vec<T> { ... } // 不透明，僅接口
impl<T> Vec<T> {
  fn new() -> Vec<T>
  fn push(&mut self, v: T) -> ()
  fn get(&self, i: i32) -> Option<T>
  fn len(&self) -> i32
}
```

**約束**：
- `Vec::new()` : `t = Vec<U>` 對任意 U，引入類型變量統一
- `push` : `self: &mut Vec<T> , v: T` → 要求 T 一致
- 需要**類型統一多項式**：`unify(T1,T2)` = `t_T1 - t_T2 =0` 對所有 T 宇宙中對應位元

實現：引入 `TypeVar` 位元，Hindley-Milner 式統一改為 Groebner 求解。

### 2.3 迴圈（loop/while/for 契約未列）

**設計**：因完整迴圈驗證不可判定，採**有界 + 不變量**雙模式。

- 無註解：有界展開 K=3（可配置 `# @fuel 10`），超過 K 視為 `UNKNOWN`，報警但不判 UNSAT（類似 CBMC）
- 有 `# @invariant`：要求用戶提供不變量，驗證：
  1. 初始成立
  2. 保持：`invariant ∧ cond ∧ body ⇒ invariant'`
  3. 結束：`invariant ∧ ¬cond ⇒ post`

編碼為多項式：不變量是謂詞（bool 表達式），其真值位元 `inv` 需為 1。

```
while cond { body }
→
assert invariant;
while cond {
  assert invariant;
  body;
  assert invariant;
}
assert invariant && !cond
```

每條 `assert` 轉為 `t_{pred}=bool` 且 `t_{pred}=true`。

### 2.4 match

**語法**：`match e { Some(x) => ..., None => ... , _ => ... }`

**降維**：引入模式編譯（pattern compilation）至決策樹：

```
match e {
  Pat1 => b1
  Pat2 => b2
}
→
let tmp = e;
if is_pat1(tmp) {
  let bindings = destruct_pat1(tmp);
  b1
} else if is_pat2(tmp) {
  ...
}
```

**約束**：
- `is_pat` 函數：對 enum variant，檢查 tag
- 每臂引入 `match_arm_bit`，Σ=1
- 結果類型 = 各臂結果類型的 join（LUB）

窮舉性檢查：若無 `_` 且 variant 未全覆蓋 → 警告/UNSAT（可配置）。

### 2.5 模組樹

**解析**：`mod foo;`（文件）或 `mod foo { ... }`（內聯）。`use crate::foo::Bar;`, `pub`, `pub(crate)`。

**實現**：
- 第一遍：建模組樹 `Module { name, parent, children, items, is_pub }`
- 第二遍：名稱解析 `resolve_path`：`a::b::C` → 查模組樹 + `use` 導入表
- 可見性檢查：若訪問私有項且不在同一模組祖先 → 報錯

**多項式**：無，直接影響 `Program` 結構扁平化後再進約束生成。為保持定理，證明「模組扁平化保持可定型性」引理（Lean 中加 `ModuleFlatten` 模組）。

### 2.6 async

**降維**：`async fn f() -> T` → `fn f() -> impl Future<Output=T>`，`async { ... }` 塊 → state machine enum。

```
async fn fetch() -> i32 { 42 }
→
enum FetchState { Start, Done(i32) }
struct FetchFuture { state: FetchState }
impl Future for FetchFuture { type Output=i32; fn poll(...) }
fn fetch() -> FetchFuture { ... }
```

`await`：
```
e.await
→
loop {
  match Future::poll(e) {
    Poll::Ready(v) => break v,
    Poll::Pending => yield
  }
}
```

**約束**：
- `Future<T>` 為泛型，`Output` 關聯類型用 `Assoc` Ty
- `await` 規則：`e: Future<T> ⇒ await e : T`
- 需要 `Send`/`Sync` 界限可選

**Lean**：新增 `AsyncStateMachine` 模組，證明 state machine 變換保持可定型性。

### 2.7 I/O

**建模**：I/O 操作視為返回 `Result<T,E>` 的普通函式，加上效應標記。

```
fn File::open(path: String) -> Result<File, IoError>  // 效應: io
```

- 預設所有 I/O 允許
- 若 `# @pure` 註解，則禁止 `eff_io=1`，否則 UNSAT
- `println!` 宏：特殊處理，類型 `()`

**QAP**：I/O 見證不進 QAP（非純計算），QAP 僅驗證純部分。

### 2.8 unsafe

**類型**：新增 `*mut T`, `*const T`, `unsafe fn`, `unsafe trait`

**上下文**：
- `unsafe { ... }` 塊設置 `in_unsafe=1`
- `unsafe fn` 只能在 unsafe 塊中調用
- 裸指針解引用 `*p` 要求 `in_unsafe=1`
- `as` 轉換 `&mut T as *mut T` 允許，但反向需 unsafe

**多項式**：
```
let p: *mut i32 = &mut x as *mut i32; // 允許
*p = 5; // 要求 in_unsafe
→ (1 - in_unsafe) * t_{deref_raw} = 0 強制矛盾若不在 unsafe
```

**借用檢查**：unsafe 中放寬部分檢查（raw 指針不追蹤），但仍追蹤 safe 引用。

### 2.9 lifetime 參數

**語法**：`fn foo<'a, 'b: 'a>(x: &'a i32, y: &'b i32) -> &'a i32`

**實現**：
- 解析 `'a` 為 `LifetimeParam`
- 收集 `where 'a: 'b` 出邊，建 outlives 圖
- 借用分析升級至 NLL：region 推斷

```
struct BorrowRegion { id, lifetime, start, end }
conflict(r1,r2) iff regions overlap && outlives permits
```

**多項式**：
- 每個 lifetime 參數引入位元 `lt_{name}`
- outlives 約束 `'a: 'b` → `lt_a * (1 - lt_b) =0`? 實際為偏序，編碼為子句 `lt_a → lt_b`
- 借用位元與 lifetime 位元合取：`b_{&'a x} = b_x * lt_a`

**Lean**：擴展 `BorrowOwnership` 模組，證明帶 lifetime 參數的借用安全判定仍等價於無衝突。

---

## 3. 前端規劃（frontends/full）

### 3.1 為何新開前端而非改 core？

- core 零依賴承諾不變
- 新特性需要複雜解析（syn-like）、類型推斷、模組文件 IO → 需依賴 `serde`, `tokio`, `axum`, `syn` 可選
- 前端可獨立演進，core 保持可審計小核心

### 3.2 frontends/full 結構

```
frontends/full/
  Cargo.toml
  src/
    main.rs      // axum HTTP + CLI
    ir.rs        // SurfaceAST → MiniRustFull IR
    ty.rs        // TyV2 宇宙構造與統一
    lower.rs     // desugar: match/for/async/mod/unsafe
    check.rs     // 擴展 checker 調用 core
    api.rs       // /api/v2/check, /api/v2/expand, /api/v2/modules
    ui.rs        // 內嵌 HTML，9 個特性分頁演示
```

API 契約（v0.2）：

```
POST /api/v2/check
body: .poly v0.2 文本
→ JSON:
{
  api_version: "0.2",
  verdict: "SAT|UNSAT|UNKNOWN",
  features_used: ["struct","enum","Vec","loop","match","mod","async","io","unsafe","lifetime"],
  modules: ["crate::foo","crate::bar"],
  type_universe_size: 42,
  ...
  generated: { code, rustc_compiles }
}

POST /api/v2/lower
→ 返回降維後的 core .poly v0.1 文本（供調試）

GET /health
```

UI：9 宮格 + 編輯器，每格一個示例（struct 示例、Vec 示例等），點「驗證」調 `/api/v2/check`。

### 3.3 與現有前端關係

- `core::server` (std-only) 保持 v0.1
- `frontends/http` (axum) 保持 v0.1 API
- `frontends/full` 新增 v0.2 API，依賴 `polyrust-core` + `axum` + `serde`
- 未來可合併：`full` 的 lowering 輸出可餵給 `core` 管線，複用 CDCL/Buchberger/QAP

---

## 4. 實現路線圖

### Phase 1 (2週) — AST 與 解析 — ✅ 已完成 (2025-10-07)

- [x] 新增 `core/src/minirust/ast_v2.rs` (7+i Universe, Struct/Enum/Impl/Trait/Mod, ProgramV2::parse_v2, 3 測試)
- [x] 擴展 `lexer.rs`: 關鍵字 `struct enum impl trait mod use pub unsafe async await loop while for match where dyn Self super crate self`, 符號 `::`, `'a`, `Gt` (`>`), 20+ 關鍵字，`show` 支持 `Gt`
- [x] `parse_v2.rs`: struct/enum/mod/match/lifetime 解析，`lex_v2`, `extract_type_tokens`, `build_universe_from_src`, 2 測試
- [x] `universe.rs`: `BaseType` 7 + `ExtType` 12 擴展 + `TypeV2` + `Universe N=7+i` + `insert_closure` + `one_hot_poly_text`, 8 測試，總計 Phase1 12 新測試
- [x] Lean `TypeUniverse7PlusI.lean`: `BaseTy7` 7, `ExtTag` 10, `Ty7Plus10` 17, `mkUniverse7PlusI`, `lang17`, `oneHot_decompose`, `one_hot_unique_17`, 30 定理，`lake build` 24 jobs 綠
- [x] 前端 `frontends/full` 集成 `Universe::new()` + `build_universe_from_src`, `/api/v2/check` 返回 `type_universe_size`, `build.rs` 修復 Lean 鏈接
- [x] 文檔 `docs/DSL_V2_PROPOSAL.md`, `docs/IMPLEMENTATION_SCHEME_V2.md`, `docs/PHASE1_REPORT.md`, `examples/demo_7plusI.poly`
- [ ] `dsl.rs`: 支持 `# @fuel`, `# @invariant`, `# @pure` (延至 Phase2)

### Phase 2 (3週) — 類型與降維 — ✅ 已完成 (2025-10-07)

- [x] `ty.rs`: `build_universe_from_program`, `build_universe_from_src`, `unify` (Same/NeedEq/NeedEqs/Fail), `subst_type`, `DepGraph`, 5 測試
  - 支持 `Vec<T>` vs `Vec<i32>` → `T=i32`, `HashMap<K,V>` 雙參數統一, Struct/Enum 名與 args 檢查
- [x] `lower.rs`: `LowerCtx`, `lower_program`, `parse_match_to_decision_tree`, `decision_tree_to_if_chain`, `parse_for_to_loop`, `for_loop_to_loop_text`, `product_poly_text`, `sum_poly_text`, 6 測試
  - struct → product `t_struct - Π t_field=0`
  - enum → sum `t_enum - Σ t_variant=0`
  - match → if 決策樹 `if is_Some(x) { ... } else { ... }` + arm bits `m_i Σ=1`
  - for → loop `into_iter` + `next` + `Some/None`
  - mod → flatten `mod utils { struct Point }` → `utils::Point`, `mod_map`
  - async → state machine `enum FetchState { Start, Poll(T), Done(T) }` + `Future<T>`
- [x] `constraints_v2.rs`: `SystemV2` 可變 N, `node_type: HashMap<usize, Vec<usize>>`, `gen_constraints_v2`, `gen_match_constraints`, `to_r1cs_v2`, 4 測試
  - product 約束 `t_struct - Π t_field`, sum 約束 `t_enum - Σ t_variant`, match arm bits `Σ m=1` + 互斥子句, unify `t_T1 - t_T2`
  - R1CS 轉換支持可變 N
- [x] Lean 新增 3 模組，`lake build` 27 jobs 綠
  - `ModuleFlatten.lean`: `ModItem`/`ModTree` mutual, `qualify`, `flattenWithPrefix`, `flattenModTree`, `modMap`, 保持引理
  - `MatchDecisionTree.lean`: `Pat`, `MatchArm`, `MatchExpr`, `DecisionTree`, `compileMatchAux`, `compileMatch`, `isExhaustive`, `matchArmBits`, `oneHotMatchPoly`
  - `ProductSum.lean`: `ProductConstraint`, `SumConstraint`, `productPolyText`, `sumPolyText`, `vecPushPolyText`, `oneHotPolyV2`, `fieldPolysV2`, `unifyPolyText`, `maxDegreeV2`
- [x] 前端 `frontends/full` 擴展
  - `src/ty.rs`: `build_universe`, `unify_types`, `display_universe`, 2 測試
  - `src/lower.rs`: `lower_full`, `lowering_report`, `lower_match`, `lower_for`, 3 測試
  - `src/api.rs`: `check_v2` 返回 `type_universe_display`, `lowering_report`, `constraints_v2` (n_vars, n_polys, n_clauses, n_products, n_sums, n_matches, universe_n)
  - `cargo build -p polyrust-full` 綠
- [x] 測試 `cargo test -p polyrust-core --lib -- ty lower constraints_v2` 21 passed, `--skip brute/exhaust` 84 passed (原 69 +15 Phase2)
- [x] 文檔 `docs/PHASE2_REPORT.md`

### Phase 3 (3週) — 高級特性

- [ ] Vec/String/HashMap 內建庫 `std/vec.poly`, `std/hashmap.poly` 擴展泛型版本
- [ ] loop 契約與有界展開，fuel 參數
- [ ] unsafe 上下文位元與 raw ptr 類型
- [ ] lifetime outlives 圖與 NLL 借用分析
- [ ] trait/impl 方法表與存在量化
- [ ] I/O 效應系統

### Phase 4 (2週) — 前端與驗證

- [ ] `frontends/full` 完整實現，axum 服務 + 內嵌 UI
- [ ] 9 特性各 3 個示例（SAT/UNSAT/UNKNOWN），共 27 個 `.poly` 文件
- [ ] `cargo test` 擴展至 100+ 測試，包含新特性
- [ ] Lean 形式化擴展至 30 模組，`lake build` 全綠
- [ ] 文檔 `docs/POLY_DSL_V2.md`, `docs/EXTENSION_EVIDENCE.md`

---

## 5. 風險與對策

| 風險 | 對策 |
|---|---|
| 類型宇宙爆炸 (N>100) | 二進制編碼 + 按需裁剪；Groebner 只對相關子系統求解（切片） |
| 迴圈不可判定 | 有界 + 不變量雙模式，UNKNOWN 不判 UNSAT |
| async state machine 複雜 | 先只支持 `async fn` + `.await`，不支持 `select!`/`join!`，後續迭代 |
| lifetime 推斷複雜 | 先支持顯式標註 `'a`，匿名 lifetime 推斷後期；outlives 圖用 Floyd-Warshall 小規模求閉包 |
| unsafe 語義寬鬆導致誤判 | 保守策略：unsafe 塊內仍檢查 safe 引用，raw 指針操作標記為 `trusted` 需人工審計 |
| QAP 規模爆炸 | QAP 僅對純計算部分，I/O/unsafe/模組解析不進 R1CS |

---

## 6. 預期成果

- `.poly v0.2` 能表達 80% 安全 Rust 子集（不含高階 trait、GAT、const generic 複雜場景）
- 前端 `polyrust-full` 提供 Web UI + HTTP API，9 特性可視化驗證
- Lean 形式化 30 模組，覆蓋新增編碼的可靠性/完備性
- 保持 `core` 零依賴，`cargo test` < 2 分鐘，`polyrust check` 仍 < 1s 於小例子

---

## 7. 附錄：示例 .poly v0.2

見 `frontends/full/examples/`（原型中已內嵌於 UI）。

```poly
# @intent: struct + enum + match + Vec + lifetime
struct Point { x: i32, y: i32 }
enum Option<T> { Some(T), None }

fn distance(p: &Point) -> i32 { p.x * p.x + p.y * p.y }

fn main() {
  let pt = Point { x: 3, y: 4 };
  let d = distance(&pt);
  let v: Vec<i32> = Vec::new();
  v.push(d);
  let opt = Option::Some(v.get(0));
  let val = match opt {
    Option::Some(x) => x,
    Option::None => 0
  };
}
```

---

*本方案已在 `frontends/full` 原型中實現框架與 UI，後續按路線圖填充各 lowering 與約束。*
