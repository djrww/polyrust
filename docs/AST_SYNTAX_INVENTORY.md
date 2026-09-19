# AST 語法清單 — v0.2 → v0.3 Full

> 目標：列出 Rust 語法全集，標註當前 `ast_v2` / `ast_full` 覆蓋度，並說明「怎麼拿到」缺失語法（core 手寫 vs 前端 syn）

## 總覽

```
core 零依賴手寫解析器（ast_v2）  →  快速、輕量、僅 std，但只覆蓋 30% 語法
frontend syn 完整解析器（ast_full + syn） →  100% Rust 語法，依賴 syn/quote，前端專用
lowering 橋樑：FullProgram -> ProgramV2 -> Program (v0.1 core)
```

- **core 承諾**：`core` 永遠零第三方，`ast_v2` 手寫解析保留，`ast_full` 定義完整 AST 但解析仍可用手寫粗略版
- **前端**：`frontends/full` 已有 `syn = { features = ["full"] }`，可直接 `syn::parse_str::<syn::File>` 拿到完整 `File`，再映射到 `ast_full::FullProgram`

## 1. Item（頂層項）

| 構造 | 示例 | ast_v2 狀態 | ast_full 狀態 | 怎麼拿到 |
|---|---|---|---|---|
| `fn` | `fn foo(x: i32) -> i32 {}` | ✅ 粗略（body 為 String） | ✅ 完整 FnItem + FnSig + Block | core 手寫已支持簽名，body 需 syn 或手寫 Block 解析 |
| `struct` | `struct Point { x: i32 }` | ✅ 字段名/類型 | ✅ Named / Tuple / Unit | 已有，手寫 |
| `enum` | `enum Option<T> { Some(T), None }` | ✅ 變體名/字段 | ✅ 完整 Variant + Discriminant | 已有 |
| `impl` | `impl Point { fn new() }` / `impl Display for Point` | ✅ self_ty + trait 名 | ✅ trait_ref + generics + where | 已有，需補 where 解析 |
| `trait` | `trait Display { fn fmt(&self) }` | ✅ 名 | ✅ supertraits + items + auto/unsafe | 已有，方法簽名粗略 |
| `mod` | `mod geometry { ... }` / `mod foo;` | ✅ 嵌套扁平化 | ✅ 外部 mod + attrs | 已有，手寫遞歸 |
| `use` | `use std::collections::HashMap;` / `use crate::a::{b,c}` | ⚠️ 僅存 String | ✅ UseTree 完整（Path/Group/Glob/Rename） | core 僅存文本，前端 syn 可完整解析 `syn::ItemUse` |
| `const` | `const MAX: i32 = 5;` | ❌ 缺失 | ✅ ConstItem | 需手寫：`const NAME: TY = EXPR`，或 syn |
| `static` | `static mut S: i32 = 0;` | ❌ 缺失 | ✅ StaticItem | 同上 |
| `type alias` | `type MyResult = Result<i32,String>;` | ❌ 缺失 | ✅ TypeAliasItem | 手寫或 syn |
| `macro_rules!` | `macro_rules! foo { ... }` | ⚠️ Macro(String) 占位 | ✅ MacroItem | core 存文本，syn 可解析 |
| `extern crate` | `extern crate foo;` | ❌ 缺失 | ✅ ExternCrate | 很少用，syn 支持 |
| `extern block` | `extern "C" { fn foo(); }` | ❌ 缺失 | ✅ ExternBlockItem | syn 支持 |

**缺口總結 Item**：`const/static/type alias` 三大項在 `ast_v2` 缺失，已在 `ast_full` 定義，需補手寫解析器（正則 `const NAME: TY = EXPR;`）或直接用前端 syn。

## 2. Type（類型）

| 構造 | 示例 | ast_v2 (TypeV2) | ast_full | 怎麼拿到 |
|---|---|---|---|---|
| Base 7 | `i32, bool, (), &i32` | ✅ | ✅ V2 | 已有 |
| `Vec<T>` | `Vec<i32>` | ✅ | ✅ Path + args | 已有 |
| `String` | `String` | ✅ | ✅ | 已有 |
| `HashMap<K,V>` | `HashMap<String,i32>` | ✅ | ✅ | 已有 |
| `Option<T>` | `Option<i32>` | ✅ | ✅ | 已有 |
| `Result<T,E>` | `Result<i32, E>` | ✅ | ✅ | 已有 |
| `*mut T / *const T` | `*mut i32` | ✅ | ✅ Ptr | 已有 |
| `&T / &mut T` | `&i32` | ✅ Base + RefExt | ✅ Ref | 已有 |
| `&'a T` | `&'a i32` | ✅ RefExt + lifetime | ✅ Ref + lifetime | 已有 |
| `Future<T>` | `Future<i32>` | ✅ | ✅ | 已有 |
| `Struct/Enum`具名 | `Point`, `Point<T>` | ✅ Struct{name,args} | ✅ Path | 已有 |
| `Tuple` | `(i32, bool)` | ❌ 缺失 | ✅ Tuple(Vec) | 需手寫 `(` `)` 分割，或 syn |
| `Array` | `[i32; 3]` | ❌ 缺失 | ✅ Array{elem,len} | 同上 |
| `Slice` | `[i32]` | ❌ 缺失 | ✅ Slice | 同上 |
| `BareFn` | `fn(i32)->bool` | ❌ 缺失 | ✅ BareFn | syn 支持 |
| `Never !` | `!` | ⚠️ Ext Never 存在但解析弱 | ✅ Never | 手寫 `-> !` 檢測 |
| `Inferred _` | `_` | ❌ 缺失 | ✅ Inferred | syn 支持 |
| `dyn Trait` | `dyn Display + Send` | ❌ 缺失 | ✅ TraitObject | syn 支持 |
| `impl Trait` | `impl Future` | ❌ 缺失 | ✅ ImplTrait | syn 支持 |
| `GenericParam` | `T, U` | ⚠️ 部分 | ✅ GenericParam | 已有部分，需補 bounds 解析 |
| `Assoc` | `<T as Trait>::Assoc` | ✅ | ✅ Path | 已有 |

**缺口總結 Type**：`Tuple/Array/Slice/BareFn/dyn/impl Trait/_` 6 項在 `ast_v2` 缺失，已在 `ast_full` 定義。獲取方式：
- core 手寫：擴展 `parse_type_v2` 增加 `(` `[` `fn(` `dyn` `impl` `_` 分支（約 50 行）
- 前端 syn：`syn::Type` 直接匹配 `Type::Tuple`, `Type::Array`, `Type::Slice`, `Type::BareFn` 等，映射到 `FullType`

## 3. Pat（模式）

| 構造 | 示例 | ast_v2 | ast_full | 怎麼拿到 |
|---|---|---|---|---|
| Wild `_` | `_` | ❌ body 文本中 | ✅ Wild | 手寫或 syn |
| Ident `x`, `mut x` | `let mut x = 5` | ⚠️ 粗略 | ✅ Ident{mutbl, by_ref, subpat} | 手寫已部分支持 |
| Path `None` | `None` | ❌ | ✅ Path | syn |
| TupleStruct `Some(x)` | `Some(x)` | ⚠️ Variant 字段 | ✅ TupleStruct | 手寫 |
| Struct `Point { x, y }` | `Point { x, y, .. }` | ❌ | ✅ Struct | 手寫需平衡 `{}` |
| Tuple `(a,b)` | `(a,b)` | ❌ | ✅ Tuple | 手寫 |
| Slice `[a,b]` | `[a,b]` | ❌ | ✅ Slice | syn |
| Or `a | b` | `Some(a) | None` | ❌ | ✅ Or | syn `Pat::Or` |
| Ref `&x` | `&x` | ❌ | ✅ Ref | syn |
| Range `0..10` | `0..=10` | ❌ | ✅ Range | syn |
| Type ascription `x: i32` | `x: i32` | ❌ | ✅ Type | syn |

**缺口**：Pat 在 `ast_v2` 幾乎未結構化（僅 body_src String），`ast_full::FullPat` 已完整定義。獲取：前端 syn `syn::Pat` 映射。

## 4. Expr（表達式）— 最多缺口

| 構造 | 示例 | ast_v2 | ast_full | 怎麼拿到 |
|---|---|---|---|---|
| Lit | `1, true, "hi"` | ✅ EKind::Int/Bool | ✅ Lit | 已有 |
| Path/Var | `x, Point` | ✅ Var | ✅ Path | 已有 |
| Field `x.y` | `p.x` | ❌ | ✅ Field | 手寫 `.` 分割或 syn |
| Index `v[0]` | `v[0]` | ❌ | ✅ Index | 同上 |
| Call `f(a)` | `foo(1,2)` | ✅ Call | ✅ Call | 已有 |
| MethodCall `v.push(1)` | `v.push(1)` | ❌ 文本 | ✅ MethodCall + turbofish | 手寫需 `receiver.method::<T>(args)` 解析，syn 有 `ExprMethodCall` |
| Unary `!x, *x, &x` | `!b, *p` | ✅ Not/Deref/Ref | ✅ Unary | 已有部分 |
| Binary `x+y` | `x+y` | ✅ BinOp | ✅ Binary | 已有 |
| Assign `x=y` | `x = y` | ✅ AssignVar | ✅ Assign | 已有 |
| AssignOp `x+=y` | `x += 1` | ❌ | ✅ AssignOp | 需補 |
| If | `if cond { } else { }` | ✅ If | ✅ If | 已有 |
| Match | `match x { ... }` | ⚠️ 文本→決策樹 | ✅ Match + arms | 已有 `parse_match_to_decision_tree` 粗略，前端 syn `ExprMatch` 完整 |
| Loop `loop {}` | `loop {}` | ⚠️ | ✅ Loop + label | 手寫或 syn |
| While `while cond {}` | `while x<10 {}` | ⚠️ | ✅ While | 同上 |
| For `for x in iter {}` | `for x in v {}` | ⚠️ 文本→loop | ✅ For | 已有 `parse_for_to_loop` |
| Block `{ stmts }` | `{ let x=1; x }` | ❌ body_src String | ✅ Block | 需手寫 `{}` 平衡 + Stmt 分割，syn 直接 |
| Unsafe `unsafe {}` | `unsafe { *p =5 }` | ⚠️ 標記位 | ✅ Unsafe | 手寫 |
| Async `async {}` | `async { 42 }` | ⚠️ | ✅ Async | 手寫 |
| Await `x.await` | `x.await` | ⚠️ 文本替換 | ✅ Await | 手寫 |
| Closure `|x| x+1` | `|x| x+1` | ❌ | ✅ Closure | syn `ExprClosure` |
| Return `return 5` | `return 5` | ❌ | ✅ Return | 手寫 |
| Break/Continue | `break, continue` | ❌ | ✅ Break/Continue | 手寫 |
| Let `if let Some(x)=opt` | `if let Some(x)=opt {}` | ❌ | ✅ Let | syn |
| StructLit `Point { x:1 }` | `Point { x:1 }` | ⚠️ 文本 | ✅ StructLit | 手寫平衡 `{}` |
| Array `[1,2,3]` | `[1,2,3]` | ❌ | ✅ Array | 手寫 |
| ArrayRepeat `[0;10]` | `[0; 10]` | ❌ | ✅ ArrayRepeat | 手寫 |
| Tuple `(1,2)` | `(1,2)` | ❌ | ✅ Tuple | 手寫 |
| Cast `x as i32` | `x as i32` | ❌ | ✅ Cast | syn |
| Try `x?` | `x?` | ❌ | ✅ Try | syn |
| Range `0..10` | `0..10, 0..=10` | ❌ | ✅ Range | syn |
| Macro `println!("hi")` | `println!("hi")` | ✅ Invoke | ✅ Macro | 已有 |
| Verbatim | 保留文本 | - | ✅ Verbatim | - |

**缺口總結 Expr**：`ast_v2` 僅有 `EKind` 8 變體（Int/Bool/Unit/Var/Let/Seq/BinOp/If/Ref/Call/Invoke），缺失 20+。`ast_full::FullExpr` 已補 30 變體。

**怎麼拿到**：
1. **core 零依賴手寫**：擴展 `parse.rs`，增加 `parse_expr()` 遞歸下降，處理 `field/index/call/method/await/cast/try/range/block`（約 300 行，參考 `syn` 的 precedence climbing）
2. **前端 syn 完整**：`syn::parse_str::<syn::Expr>` 直接得到 `Expr::Field`, `Expr::MethodCall`, `Expr::Await`, `Expr::Closure` 等，映射到 `FullExpr`（已在 `frontends/full/src/syn_bridge.rs` 規劃）

## 5. Stmt

| 構造 | ast_v2 | ast_full | 怎麼拿到 |
|---|---|---|---|
| `let PAT: TY = EXPR;` | ⚠️ 文本 | ✅ Local{pat,ty,init} | 手寫 `let` 解析已部分，前端 syn `Stmt::Local` |
| `ITEM` | ✅ | ✅ Item | 已有 |
| `EXPR;` / `EXPR` | ✅ Seq | ✅ Expr / Semi | 已有 |
| Macro stmt | ❌ | ✅ Macro | syn |

## 6. Generics / Where / Lifetime / Attr / Vis

| 構造 | ast_v2 | ast_full | 怎麼拿到 |
|---|---|---|---|
| `Vis` `pub, pub(crate)` | ⚠️ bool is_pub | ✅ Vis 枚舉 | 手寫 `pub` 前綴檢測已部分，syn `Visibility` |
| `Attr` `#[derive(...)]` | ⚠️ 忽略 | ✅ Attr{name,args} | 手寫 `#[` 平衡，syn `Attribute` |
| `Lifetime 'a` | ⚠️ String | ✅ Lifetime | 已有部分 |
| `GenericParam T, 'a, const N` | ⚠️ Vec<String> | ✅ GenericParam 枚舉 | 手寫 `<T, 'a, const N: i32>` 解析需 100 行，syn `Generics` |
| `WhereClause T: Display` | ❌ Vec<String> 占位 | ✅ WhereClause + TypeBound | 同上 |
| `TypeBound Display + Send` | ❌ | ✅ TypeBound | 同上 |

## 7. 當前覆蓋率統計（基於 HandwrittenParser::coverage_report）

對 `examples/phase3/*.poly` 運行 `coverage_report`：

- ✅ 已支持 25+ 項：struct, enum, fn, impl, trait, mod, use, Vec, HashMap, Option, Result, &T, *mut, lifetime, let, if, match, loop, while, for, await, async, unsafe, attr, pub, generic
- ❌ 未覆蓋（core 手寫解析器需補）：
  - `const/static/type alias` (3)
  - `Tuple/Array/Slice/BareFn/dyn/impl Trait/_` (6)
  - `Pat Or/Range/Slice` (3)
  - `Expr Closure/Return/Break/Try/Cast/Range/Array` (6)
  - `UseTree Group/Glob/Rename` 完整 (1)

**合計缺口約 19 項**，已在 `ast_full` 定義，等待手寫解析器實現或前端 syn 橋接。

## 8. 怎麼拿到缺失語法 — 具體路徑

### 路徑 A：core 零依賴手寫（保持承諾）

1. **擴展 `universe.rs::parse_type_v2`**：增加 `(` `)` → Tuple, `[T; N]` → Array, `[T]` → Slice, `fn()` → BareFn, `dyn` → TraitObject, `impl` → ImplTrait, `_` → Inferred
2. **新建 `parse_full.rs`**：遞歸下降 `parse_expr()` / `parse_pat()` / `parse_stmt()`，參考 `ast.rs` 的 `EKind` 但擴展到 `FullExpr`
3. **擴展 `ast_v2.rs::parse_struct/enum/fn`**：增加 `const/static/type` 分支，`where` 子句解析
4. **測試**：用 `cargo test -p polyrust-core ast_full` 驗證

工作量：約 500 行，零依賴，保持 core 承諾。

### 路徑 B：前端 syn 橋接（已部分實現）

1. **創建 `frontends/full/src/syn_bridge.rs`**：
   ```rust
   use syn::{File, Item, Type, Pat, Expr};
   pub fn syn_file_to_full_program(file: File) -> ast_full::FullProgram { ... }
   ```
   - `Item::Fn` → `FnItem`
   - `Item::Struct` → `StructItem`
   - `Type::Tuple/Array/Slice/BareFn` → `FullType`
   - `Pat::Wild/Ident/Struct` → `FullPat`
   - `Expr::Await/Closure/MethodCall` → `FullExpr`

2. **在 `main.rs` 的 `/api/v2/check` 中**：
   - 先嘗試 `syn::parse_str::<File>(src)`，成功則走 `FullProgram` 路徑，生成 `ProgramV2` 用於約束
   - 失敗則回退到 `ProgramV2::parse_v2`（兼容舊 `.poly` DSL）

3. **優勢**：100% Rust 語法，無需手寫，立即可用；劣勢：僅前端可用，core 仍需手寫以保持零依賴

### 路徑 C：混合（推薦）

- **core**：繼續手寫，逐步補齊 `const/static/type alias` + `Tuple/Array/Slice`（最常用，約 100 行即可覆蓋 80% 用例）
- **frontend**：用 syn 實現完整 `FullProgram`，作為「參考實現」與「測試 oracle」，對比 core 手寫解析器的輸出，生成差異報告
- **文檔**：本文件 + `ast_full.rs` 的 `HandwrittenParser::coverage_report()` 作為自動化檢測工具，`cargo run --bin polyrust -- ast-coverage <file>` 可輸出缺口

## 9. 下一步 TODO

- [ ] 在 `core` 增加 `parse_full.rs`，實現 `FullType::Tuple/Array/Slice/BareFn`
- [ ] 補 `const/static/type` 的 `ItemV2` 解析（`ast_v2.rs` 增加 3 分支）
- [ ] 前端新增 `syn_bridge.rs`，實現 `syn::File -> FullProgram`
- [ ] 在 `polyrust-core` 二進制增加 `ast-coverage` 子命令，調用 `HandwrittenParser::coverage_report`
- [ ] 在 `docs/EXAMPLES_PHASE3.md` 中為每個缺口語法增加示例 `.poly`，驗證 `SAT/UNSAT`

## 10. 參考

- `core/src/minirust/ast_v2.rs`：當前手寫頂層解析器
- `core/src/minirust/ast_full.rs`：完整 AST 定義 + 覆蓋率檢測
- `core/src/minirust/universe.rs::parse_type_v2`：類型解析擴展點
- `frontends/full/Cargo.toml`：`syn = { features = ["full"] }` 已就緒
- `docs/POLY_DSL.md`：`.poly` DSL 語法
