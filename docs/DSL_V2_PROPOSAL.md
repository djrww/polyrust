# .poly v0.2 DSL 提案 — 7+i 型別宇宙

> 對應 Phase1 已實現的 `Universe N=7+i` 與 Lean `TypeUniverse7PlusI`

## 1. 設計目標

- **保持核心零依賴**：`polyrust-core` 仍 `no_std` 兼容，無第三方依賴；擴展僅在 `core/src/minirust/*` 內用 `std` 集合
- **前端隔離**：第三方依賴（`axum`, `tokio`, `ureq`）僅在 `frontends/*`
- **向後兼容**：v0.1 程序（僅 `i32, bool, (), &i32, &mut i32, &bool, &mut bool`）無需修改仍可驗證
- **可擴展**：新增類型僅擴展 `Universe` 列表，約束生成 `Σ_N t -1 =0` 自動適配

## 2. .poly 文件結構

```poly
@intent: "demo struct+enum+vec"
@mode: full
@type-universe: auto  # auto | 7 | 7+3 | 17
@fuel: 2000

# 可選全局設置
@set N_TYPES = 7+3
@set QAP = true
@set CDCL = true

# 導入（可選，暫為註釋）
@import std::vec::Vec
@import std::collections::HashMap

# ── 模塊樹 ──
mod utils {
  pub struct Point { x: i32, y: i32 }
  pub enum Option<T> { Some(T), None }
  pub trait Display { fn fmt(self) -> String; }
  impl Display for Point { fn fmt(self) -> String { ... } }
}

# ── 頂層定義 ──
pub struct User {
  id: i32,
  name: String,
  tags: Vec<String>,
  meta: Option<HashMap<String, String>>,
}

pub enum Status {
  Active,
  Inactive,
  Pending(i32),
  WithData { code: i32, msg: String },
}

pub trait Repo {
  fn get(self, id: i32) -> Option<User>;
  fn list(self) -> Vec<User>;
}

pub struct MemRepo { data: Vec<User> }
impl Repo for MemRepo {
  fn get(self, id: i32) -> Option<User> { ... }
  fn list(self) -> Vec<User> { self.data }
}

# ── 函數 ──
pub fn make_user(id: i32) -> User {
  User { id, name: String::new(), tags: Vec::new(), meta: None }
}

pub unsafe fn raw_access(p: *mut i32) -> i32 { *p }

pub async fn fetch(id: i32) -> Result<User, String> {
  let u = make_user(id);
  Ok(u)
}

fn main() {
  let mut v: Vec<i32> = Vec::new();
  v.push(1);
  v.push(2);
  let sum = 0;
  for x in v {
    sum = sum + x;
  }
  let s = match sum {
    0 => "empty",
    _ => "nonempty",
  };
  let _ = s;
}
```

## 3. 語法擴展 BNF（增量）

```bnf
File ::= (Attr | Item)* FnMain
Attr ::= "@" Ident ":" StringLit | "@set" Ident "=" Expr | "@import" Path

Item ::= StructDef | EnumDef | TraitDef | ImplDef | ModDef | FnDef | Use | Macro

StructDef ::= Vis? "struct" Ident Generics? "{" (Field ",")* "}"
Field ::= Vis? Ident ":" Type

EnumDef ::= Vis? "enum" Ident Generics? "{" Variant ("," Variant)* "}"
Variant ::= Ident | Ident "(" TypeList ")" | Ident "{" FieldList "}"

TraitDef ::= Vis? "trait" Ident Generics? "{" (FnSig ";")* "}"
ImplDef ::= "impl" Generics? TraitRef? "for" Type "{" (FnDef)* "}"
  TraitRef ::= Path | Path "<" TypeList ">"

ModDef ::= Vis? "mod" Ident "{" Item* "}" | Vis? "mod" Ident ";"  # 後者為文件模塊

FnDef ::= Vis? "fn" Ident Generics? "(" Params? ")" ("->" Type)? Block
FnSig ::= Vis? "fn" Ident Generics? "(" Params? ")" ("->" Type)? ";"
Params ::= (Mut? Ident ":" Type) ("," ...)*

Type ::= BaseTy | ExtTy | GenericParam | AssocTy
BaseTy ::= "i32" | "bool" | "()" | "&" Mut? Type | "&" Lifetime Mut? Type
ExtTy ::= "Vec" "<" Type ">"
        | "String"
        | "HashMap" "<" Type "," Type ">"
        | "Option" "<" Type ">"
        | "Result" "<" Type "," Type ">"
        | "Future" "<" Type ">" | "impl" "Future" "<" Type ">"
        | "*" "mut" Type | "*" "const" Type
        | Ident ("::" Ident)* ("<" TypeList ">")?  # struct/enum 名
        | "(" TypeList ")" | "[" Type ";" Expr "]" | "[" Type "]"

Generics ::= "<" (Lifetime | Ident (":" TraitBound)?) ("," ...)* ">"
Lifetime ::= "'" Ident | "'static" | "'_"
TraitBound ::= Path ("+" Path)*

Vis ::= "pub" | "pub" "(" "crate" ")" | "pub" "(" "super" ")" | ...

Block ::= "{" Stmt* "}"
Stmt ::= Let | Assign | ExprStmt | Loop | While | For | Match | Return | UnsafeBlock | AsyncBlock
Loop ::= "loop" Block
While ::= "while" Expr Block
For ::= "for" Pat "in" Expr Block
Match ::= "match" Expr "{" (Pat "=>" (Expr | Block) ",")* "}"
Pat ::= "_" | Ident | Literal | StructPat | EnumPat | TuplePat | RefPat
StructPat ::= Path "{" (Ident ":" Pat) ("," ...)* "}"
EnumPat ::= Path ("(" PatList ")" | "{" FieldPatList "}")?

Expr ::= Literal | Path | Call | MethodCall | FieldAccess | Index | Ref | Deref | Binary | Unary | If | Match | Loop | While | For | Block | UnsafeBlock | Await | AsyncBlock | VecLit | StructLit

UnsafeBlock ::= "unsafe" Block
AsyncBlock ::= "async" Block
Await ::= Expr "." "await"

# 借用
Borrow ::= "&" Lifetime? Mut? Expr
Mut ::= "mut"
```

## 4. 型別宇宙 7+i

### 4.1 基底 7

```
0: i32
1: bool
2: ()
3: &i32
4: &mut i32
5: &bool
6: &mut bool
```

Lean: `BaseTy7.all = [i32, bool, unit, refI32, refMutI32, refBool, refMutBool]`

### 4.2 擴展標籤 10 (Phase1 已實現)

```
vec, string, hashmap, structTy, enumTy, rawPtrMut, rawPtrConst, future, option, result
```

Rust: `ExtType::{Vec, String, HashMap, Struct, Enum, RawPtr, Future, Option, Result, RefExt, GenericParam, Assoc, Never}`

Lean: `ExtTag.all = 10`

### 4.3 動態 N = 7 + i

- `Universe::new()` 起始 N=7
- `insert_closure(ty)` 遞歸插入子類型，閉包
- 例如 `Vec<i32> + String + HashMap<String,i32>` → N=7+5 (Vec<i32>, String, HashMap, String 復用, i32 復用 → 實際 7+3)
- Lean: `mkUniverse7PlusI exts` 長度 `7 + exts.length`, `nodup` 若 `exts` nodup

### 4.4 約束生成

對每個節點 `v`:

```
one-hot: Σ_{k=0}^{N-1} t_{v,k} -1 =0
domain: t_{v,k}^2 - t_{v,k}=0
```

L0 嵌入：`embedBase7` 將基底 one-hot 映射為擴展宇宙 one-hot，保留 `Σ=1`

Product (struct): `t_{v,Struct} - t_{field1}*...*t_{fieldN}=0` (積型)

Sum (enum): `t_{v,Enum} - Σ_{variant} t_{variant}=0`

## 5. 借用分析擴展

Phase2 計劃：

```
LifetimeParam: 'a, 'b, 'static
Region: Set<Lifetime>

Borrowck V2:
  - RefExt { mutbl, inner, lifetime }
  - 約束: 若 t_{v, &mut T} =1 則 borrow 活，生成 borrow-polys
  - lifetime 約束: 'a: 'b  →  region inclusion 多項式
  - 編碼: borrow_graph 為 SAT 子句，仍可 CDCL
```

## 6. QAP 處理

- `N` 可變，但 `bits_per_node = N`（one-hot 位寬）
- `n_vars = Σ_nodes N + aux`
- `field_polys_text` 仍為 `t^2 - t`
- `one_hot_poly_text` 生成 `Σ t -1`
- 對 `N=17`，每節點 17 變量，17 個域多項式 +1 個 one-hot，L0 嵌入保證度 ≤2

## 7. 前端隔離

```
core/                 -- 零依賴 (除 std 集合)
  src/minirust/
    universe.rs       -- TypeV2, Universe
    ast_v2.rs         -- Struct/Enum/Impl/Trait/Mod
    parse_v2.rs       -- 擴展解析
    lexer.rs          -- Gt, Lifetime, 擴展關鍵字

frontends/
  full/               -- axum/tokio 依賴，調用 core Universe
    src/api.rs        -- /api/v2/check 返回 type_universe_size = Universe.n_types()
    src/ir.rs         -- detect_features, estimate_type_universe 用 Universe
  http/               -- axum 依賴
  llm/                -- ureq 依賴
```

## 8. 示例

### 8.1 Struct

```poly
struct Point { x: i32, y: i32 }
fn main() {
  let p = Point { x: 3, y: 4 };
  let _ = p.x + p.y;
}
```

宇宙：N=8 (Point)

one-hot:
```
t0_0+...+t0_7 -1=0
```

### 8.2 Enum + Match

```poly
enum Option<T> { Some(T), None }
fn main() {
  let x: Option<i32> = Option::Some(5);
  let y = match x { Some(v) => v, None => 0 };
}
```

降維：`Option<T>` → sum type，`match` → if 決策樹

```
t_{match} = t_{Some}*t_{v} + t_{None}*0
```

### 8.3 Vec + Loop

```poly
fn main() {
  let mut v: Vec<i32> = Vec::new();
  v.push(1);
  let mut sum = 0;
  for x in v { sum = sum + x; }
}
```

降維：`Vec<T>` → `*mut T + len`，`for` → `loop` + `next()`

### 8.4 Lifetime

```poly
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  if x.len() > y.len() { x } else { y }
}
fn main() {
  let s1 = "hello";
  let s2 = "world";
  let _ = longest(s1, s2);
}
```

宇宙：`RefExt { lifetime: 'a, mutbl: false, inner: String }`

約束：`'a` region 包含 `s1`, `s2` 區域

### 8.5 Unsafe

```poly
fn main() {
  let mut x = 5;
  let p: *mut i32 = &mut x as *mut i32;
  unsafe { *p = 10; }
}
```

編碼：`*mut T` 為 `RawPtr{mutbl:true, inner:T}`，`unsafe` 塊內 `*p` 解引用不生成 borrow 約束，但生成 `raw_deref` 多項式

### 8.6 Async

```poly
async fn fetch() -> i32 { 42 }
fn main() {
  let fut = fetch();
  let _ = fut.await;
}
```

降維：`async fn -> i32` → `fn -> Future<i32>`，`Future` 為狀態機 enum

```
enum FetchState { Start, Poll(i32), Done(i32) }
```

## 9. 實現路線

- [x] Phase1: AST + Universe + Lexer (本報告)
- [ ] Phase2: ty.rs unify, lower.rs struct/enum/match/loop/mod flatten, constraints.rs N 可變
- [ ] Phase3: borrowck lifetime, unsafe raw ptr, async state machine, QAP 集成
- [ ] Phase4: 前端 UI 展示 N, Lean 完整 ModuleFlatten + MatchDecisionTree 證明

## 10. Lean 對應

- `TypeUniverse7PlusI.lean`: `BaseTy7` 7, `ExtTag` 10, `Ty7Plus10` 17, `mkUniverse7PlusI`
- `Lang Ty7Plus10` 仍 6 參數，`numTy=i32`, `eqbTy=bool`
- `oneHot_decompose`, `base_one_hot_implies_ext_zero`, `one_hot_unique_17`
- 與 `T9Generalized.lean` 銜接：`typable_iff_rootG`, `genC_soundG`, `genC_completeG` 直接可用

## 11. 驗證

```
cargo test -p polyrust-core --lib -- 79 passed
lake build Polyrust.TypeUniverse7PlusI -- ok
lake build -- 24 jobs ok
polyrust-full 前端 /api/v2/check 返回 type_universe_size = Universe.n_types()
```
