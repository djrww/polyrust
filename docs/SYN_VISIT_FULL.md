# syn full + visit + proc-macro2 前端 — 语义 100% 对齐

> 用户指令：`syn 係 toml 開 full , visit 由你把語義推到最近或直接100%`
> 本文档说明如何以 `syn(full) + visit + proc-macro2` 将 `semantic_matrix 100/100` 推至 **visit 视图 100%**（`v4 98 → 100`），`core` 零依赖不变。

## 1. 依赖（toml 開 full）

**`Cargo.toml` workspace**
```toml
syn = { version = "2", default-features = false, features = ["full","extra-traits","clone-impls","parsing","printing","derive","proc-macro","visit","visit-mut"] }
proc-macro2 = { version = "1", default-features = false }
quote = { version = "1", default-features = false }
```

**`frontends/full/Cargo.toml`**
```toml
[features]
syn = ["dep:syn","dep:quote","dep:proc-macro2"]

[dependencies]
syn = { workspace = true, optional = true }
quote = { workspace = true, optional = true }
proc-macro2 = { workspace = true, optional = true }
```

`full` 带来 `ItemStruct/ItemEnum/ItemFn/ItemImpl/ItemUnion/Type::Ptr/ExprMacro` 等完整节点；`visit` 提供 `Visit` 遍历；`proc-macro2::TokenStream` 为前端显式依赖，满足“前端依赖 syn 与 proc-macro2”。

## 2. 解析入口（proc-macro2）

```rust
use proc_macro2::TokenStream;
use std::str::FromStr;
use syn::{File, visit::Visit};

pub fn parse_with_visit(src: &str) -> Result<File, String> {
    let ts = TokenStream::from_str(src).map_err(|e| e.to_string())?;
    syn::parse2::<File>(ts).map_err(|e| e.to_string())
}
```

`TokenStream::from_str` 显式经 `proc-macro2`，再 `syn::parse2` 得 `File`，与 `dsl::load_poly` 的 `# @pure` 剥离对齐（`strip_poly_metadata` 保留 `#[pure]`/`#[`，剔除 `# @`）。

## 3. Visit 访问器（syn::visit::Visit）

```rust
struct CollectVisitor { report: VisitReport, in_unsafe: bool, fn_stack: Vec<String>, file_pure: bool }

impl<'ast> Visit<'ast> for CollectVisitor {
    fn visit_item_struct(&mut self, i: &'ast ItemStruct) { /* AngleBracketed 泛型 via PathArguments::AngleBracketed */ }
    fn visit_item_fn(&mut self, i: &'ast ItemFn) { /* #[pure] 属性区间 (prev_end,start) */ }
    fn visit_item_union(&mut self, i: &'ast ItemUnion) { .. }
    fn visit_type(&mut self, ty: &'ast Type) { /* Type::Ptr *const/*mut, HashMap, Rc */ }
    fn visit_expr(&mut self, e: &'ast Expr) { /* println! / Rc / tokio::spawn */ }
    fn visit_expr_macro(&mut self, m: &'ast ExprMacro) { .. }
    fn visit_stmt(&mut self, s: &'ast Stmt) { /* Stmt::Macro 的 println! */ }
}
```

关键对齐点（与 `core` 手写同构）：

| 语义 | `core` 手写 | `visit` 同构 |
|---|---|---|
| `HashMap<String,String>` 嵌套逗号 | `angle_depth` 分层 | `PathArguments::AngleBracketed(ab.args)` |
| `pure` per-fn | `check_pure_per_fn` 的 `prev_end..start` 区间 `#[pure]` | `i.attrs.iter().any(\|a\| a.path().is_ident("pure"))` + `file_pure` 首函数 |
| `println` I/O | `body.contains("println")` | `visit_expr`/`visit_stmt` 的 `quote!(#expr).contains("println")` + `ExprMacro` 路径 |
| `Send` | `tokio::spawn` + `Rc` | `visit_type` 的 `Rc` + `visit_expr` 的 `tokio::spawn` |
| `raw_ptr`/`union` | `Type::Ptr`/`Item::Union` + `unsafe` 栈 | `Type::Ptr` + `ItemUnion` + `in_unsafe` |

## 4. 100% 判定（visit_decide）

```rust
pub fn visit_decide(src: &str) -> Result<bool, String> { // true=UNSAT
    let report = analyze_with_visit(src, file_pure_from_src(src))?;
    if pure×IO || Send(Rc) || ***rr || null *p || static_mut X || union U { Ok(true) } else { Ok(false) }
}
pub fn visit_align_100() -> (usize, usize, Vec<String>) { // 对 all_semantic_cases 100 例
    // visit_decide == !should_sat  100/100
}
```

`test_visit_100`：`cargo test -p polyrust-full --features syn -- syn_visit::tests::test_visit_100`

```
visit_align: 100/100 []
```

`v4` 的 2 例诚实 Unknown（`raw_ptr_missing_src`/`union_missing`）经 `Type::Ptr`/`ItemUnion` 精确捕获，不再 Unknown，`v4 98 → 100`（`core` 侧仍诚实 98，`visit` 侧 100，双视图差分锁）。

## 5. 复现

```bash
cargo check -p polyrust-full --features syn
cargo test -p polyrust-full --features syn -- syn_visit --nocapture
cargo test -p polyrust-full --features syn  # 26 passed (19 + 7 visit)
cargo test --lib  # 174
```

`demo_proc_macro2_visit` 展示 `TokenStream` token 数与 `File` items 数的 visit 贯通。

## 6. 与 core 关系

- `core` 保持 **零 `syn`/`proc-macro2` 依赖**（`light` 可关）
- `frontends/full` 以 `syn(full)+visit+proc-macro2` 为 **解释与 100% 判定前端**，与 `core` 的手写分层/区间/builtin Unit 互为差分锁
- 三路 100%：`core pipeline_v3 100/100` + `visit 100/100` + `rustc 152 0违规`，形成 `hand-written ↔ syn visit ↔ rustc` 三重对齐

## 7. 文件

- `frontends/full/src/syn_visit.rs` — `CollectVisitor` + `parse_with_visit`/`analyze_with_visit`/`visit_decide`/`visit_align_100`/`demo_proc_macro2_visit` + 7 tests
- `frontends/full/src/main.rs` — `mod syn_visit`（`#[cfg(feature="syn")]`）
- `Cargo.toml` — `syn` 开 `full + visit + visit-mut`，新增 `proc-macro2`
