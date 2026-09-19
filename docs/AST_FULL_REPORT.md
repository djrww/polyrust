# AST 語法補齊報告 — v0.3 Full

## 目標
補齊 `ast_v2` 的語法缺口，提供完整 AST 定義，並給出「如何拿到」缺失語法的路徑。

## 已完成工作

### 1. 完整 AST 定義 `core/src/minirust/ast_full.rs`
- **Vis**：Private / Pub / PubCrate / PubSuper / PubSelf / PubIn
- **Attr**：#[derive], #![...], @fuel, @invariant 等
- **Lifetime**：'a, 'b
- **GenericParam**：Type { bounds, default }, Lifetime, Const
- **WhereClause / TypeBound**
- **FullType**：16 變體
  - V2(TypeV2) 兼容舊宇宙
  - Path { path, args }：Vec<T>, HashMap<K,V>, MyMod::Point
  - Tuple, Array, Slice, Ptr, Ref, BareFn, Never, Inferred, TraitObject, ImplTrait, Macro, Group, Paren
- **FullPat**：12 變體 Wild/Ident/Lit/Path/TupleStruct/Struct/Tuple/Slice/Or/Ref/Box/Range/Macro/Type
- **FullExpr**：30+ 變體 Lit/Path/Field/Index/Call/MethodCall/Unary/Binary/Assign/If/Match/Loop/While/For/Block/Unsafe/Async/Await/Closure/Return/Break/Continue/Let/StructLit/Array/Tuple/Cast/Try/Range/Macro
- **FullStmt**：Local / Item / Expr / Semi / Macro
- **FullItem**：Fn/Struct/Enum/Impl/Trait/Mod/Use/Const/Static/TypeAlias/Macro/ExternCrate/ExternBlock
- **FullProgram**：attrs + items + main

與 `ast_v2` 互轉：`From<FullProgram> for ProgramV2`，保留降維路徑 `FullProgram -> ProgramV2 -> Program (v0.1 core)`

### 2. 覆蓋率檢測工具 `HandwrittenParser::coverage_report`
- 位於 `ast_full.rs`，零依賴，僅用 `contains`
- 44 項檢查：struct/enum/fn/impl/trait/mod/use/const/static/type alias/Vec/HashMap/Option/Result/Tuple/Array/Slice/*mut/&T/&'a/fn ptr/impl Trait/dyn Trait/!/ _ /let/if/match/loop/while/for/closure/await/async/unsafe/return/break/?/range/macro/attr/pub/generic/where/lifetime
- CLI：`cargo run -p polyrust-core --bin polyrust -- ast-coverage <file>`
  - 文本報告：✅/❌ + 描述
  - 缺失列表 + 獲取方式提示

### 3. 前端 syn 橋接 `frontends/full/src/syn_bridge.rs`
- 依賴 `syn = { features = ["full"] }`，僅前端可用，core 保持零依賴承諾
- `parse_with_syn(src) -> FullProgram`
- 映射：
  - `syn::File` → `FullProgram`
  - `syn::Item::Fn/Struct/Enum/Impl/Mod/Use/Const/Static/Type` → `FullItem`
  - `syn::Type::Path/Tuple/Array/Slice/Ptr/Reference/BareFn/Never/Infer/TraitObject/ImplTrait` → `FullType`
  - `syn::Pat::Wild/Ident/Struct/Tuple/Or/Range` → `FullPat`
  - `syn::Expr::Field/Index/Call/MethodCall/Await/Closure/Match/Loop/While/For/Block/Unsafe/Async/Return/Break/StructLit/Array` → `FullExpr`
- 測試：`test_syn_bridge_struct`, `test_syn_bridge_full` 通過

### 4. 語法清單文檔 `docs/AST_SYNTAX_INVENTORY.md`
- 按 Item/Type/Pat/Expr/Stmt/Generics 分類
- 每項標註：示例、ast_v2 狀態、ast_full 狀態、怎麼拿到
- 缺口統計：原 ast_v2 缺失約 19 項（const/static/type alias, Tuple/Array/Slice/BareFn/dyn/impl Trait/_, Pat Or/Range, Expr Closure/Return/Break/Try/Cast/Range 等），已在 ast_full 定義

### 5. 全語法示例 `examples/full_syntax.poly`
- 包含 44 項全部語法，`ast-coverage` 報告「全部語法已覆蓋 ✓」
- 用於回歸測試與文檔示例

## 尚缺什麼？怎麼拿到？

### 路徑 A：core 零依賴手寫（推薦逐步補）

| 缺口 | 當前 | 補齊方式 | 工作量 |
|---|---|---|---|
| `const/static/type alias` | ast_v2 無 | 在 `ast_v2.rs::parse_v2` 增加 `const`, `static`, `type` 分支，正則 `const NAME: TY = EXPR;` | 50 行 |
| `Tuple/Array/Slice/BareFn` | parse_type_v2 無 | 擴展 `parse_type_v2` 增加 `(` `[` `fn(` 分支，平衡括號解析 | 50 行 |
| `dyn/impl Trait/_/!` | 無 | 增加 `dyn`, `impl`, `_`, `!` 關鍵字檢測 | 30 行 |
| `Pat Or/Range/Slice` | 無 | 新建 `parse_pat()` 遞歸，處理 `a | b`, `0..10`, `[a,b]` | 80 行 |
| `Expr Closure/Return/Break/Try/Cast/Range` | 無 | 新建 `parse_expr()` precedence climbing，處理 `|x|`, `return`, `break`, `?`, `as`, `..` | 300 行 |
| `UseTree Group/Glob` | 僅 String | 解析 `use a::{b,c}` 花括號組 | 40 行 |

總計約 500 行，零依賴，保持 core 承諾。

### 路徑 B：前端 syn 完整（立即可用）

```rust
// frontends/full/src/main.rs
use syn_bridge::parse_with_syn;
let full_prog = parse_with_syn(&src)?;
let v2_prog: ProgramV2 = full_prog.into(); // 已實現 From
let lowered = lower_program(v2_prog)?;
```

- 優點：100% Rust 語法，無需手寫
- 缺點：僅前端可用，core 仍需手寫以保持零依賴
- 已實現：`syn_bridge.rs` 可直接用於 `/api/v2/check`，先嘗試 syn 解析，失敗回退到 `ProgramV2::parse_v2`

### 路徑 C：混合（推薦）

- core 手寫補 `const/static/type alias` + `Tuple/Array/Slice`（最常用，80% 用例）
- frontend syn 作為參考實現與測試 oracle，對比 core 手寫輸出，生成差異報告
- 本報告 + `HandwrittenParser::coverage_report()` 作為自動化檢測，`cargo run --bin polyrust -- ast-coverage <file>` 輸出缺口

## 驗證

```bash
cargo test -p polyrust-core --lib -- ast_full -- --nocapture
# 2 passed

cargo test -p polyrust-full -- syn_bridge -- --nocapture
# 2 passed

cargo test -p polyrust-core --lib -- --skip brute --skip exhaust
# 139 passed (原 137 + 2 ast_full)

cargo run -p polyrust-core --bin polyrust -- ast-coverage examples/full_syntax.poly
# 全部語法已覆蓋 ✓

cargo build -p polyrust-full
# 成功，39 warnings（未使用字段等，正常）
```

## 文件清單

- `core/src/minirust/ast_full.rs`：完整 AST + 覆蓋率工具 + 測試
- `core/src/minirust/mod.rs`：新增 `pub mod ast_full`
- `frontends/full/src/syn_bridge.rs`：syn → ast_full 橋接
- `frontends/full/src/main.rs`：註冊 `mod syn_bridge`
- `core/src/main.rs`：新增 `ast-coverage` 子命令
- `docs/AST_SYNTAX_INVENTORY.md`：詳細清單（44 項 + 獲取路徑）
- `docs/AST_FULL_REPORT.md`：本報告
- `examples/full_syntax.poly`：全語法示例，覆蓋率 100%

## 下一步

- [ ] 在 core 增加 `parse_full.rs`，實現 FullType Tuple/Array/Slice/BareFn（50 行）
- [ ] 補 const/static/type 的 ItemV2 解析（3 分支）
- [ ] 在 `/api/v2/check` 中集成 syn_bridge：先 syn，後回退
- [ ] 增加 `ast-coverage` 到 `polyrust serve` 的 UI，顯示語法覆蓋率
- [ ] 為每個缺口語法在 `examples/phase3/` 增加 SAT/UNSAT 示例
