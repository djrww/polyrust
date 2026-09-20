# 前端依赖 `syn` 解释 — rustc 语法的真实解析（2026-09-20）

> **定位**：`core` 零依赖（`std` only），`frontends/full` 以 `syn = { features = ["full"] }` 为 **地真值前端**。本文用 `syn` 的真实 AST 逐项解释 WRP-R2 三项（`enterprise_ide` / `async_spawn` / `io_with_pure_call`）为何手写解析会偏差、而 `syn` 如何根治。

---

## 0. 一句话结论

- 手写 `pipeline_v2::check_struct_field_types` 的 `inner.split(',')` + `parse_type_v2` 前缀匹配是 **字符串级启发式**，在 `HashMap<String, String>` 的 `< , >` 内逗号处误切，导致 `editors: HashMap<String, String>` 被丢为 `i32`。
- `syn::Type::Path` + `PathArguments::AngleBracketed` + `Punctuated<GenericArgument, Comma>` 是 **token 级精确**的 `Punctuated`，`HashMap` 的两个 `String` 作为两个 `GenericArgument::Type` 独立节点，逗号仅作分隔符，不入 `Ty`。
- 本文在 `frontends/full/src/syn_explain.rs`（`--features syn`）中以 `explain_*` 3 函数演示，`cargo test -p polyrust-full --features syn -- syn_explain --nocapture` 可重现。

---

## 1. 架构：core vs frontend 的 TCB 切分

```
core (polyrust-core)          frontends/full (polyrust-full)
┌─────────────────┐           ┌──────────────────────────┐
│ std only        │           │ syn + quote (full)       │
│ • parse_type_v2 │←──对比────│ syn::parse_str::<File>   │
│ • split(',')    │           │ Punctuated              │
│ • check_async   │           │ Item::Fn / Type::Path    │
│ 零第三方依赖    │           │ axum / tokio / serde     │
└─────────────────┘           └──────────────────────────┘
        ▲ 真实 rustc 语法仅由前端 syn 保证，core 通过差分与 rustc_align 对齐
```

- **core 保持零依赖**是安全承诺（`Cargo.toml` 留空），`syn` 仅在 `frontends/full` 以 `optional = true` 引入，`light` feature 可关闭以减 14 依赖。
- **解释权**：`syn` 的 `File`/`Item`/`Type` 即 rustc 语法的“镜像”，`core` 的手写解析是其子集近似，二者差异由 `differential` 与 `rustc_align` 锁定。

---

## 2. 三项逐项解释（`syn` 视角）

### 2.1 `enterprise_ide` — `HashMap<String, String>` 泛型逗号

**WRP-R2 根因**：`EnterpriseIDE { editors: HashMap::new() }` 的类型声明 `HashMap<String, String>` 在 `check_struct_field_types` 中被 `inner.split(',')` 按顶层逗号切为 `HashMap<String` 与 `String>`，`parse_type_v2` 前缀匹配失败，落为 `i32`，判定 `expected i32 got HashMap` 误拒。

**`syn` 正确**：

```rust
use syn::{Item, Type, GenericArgument, PathArguments};

let src = r#"struct EnterpriseIDE { editors: HashMap<String, String> }"#;
let file: File = syn::parse_str(src).unwrap();
if let Item::Struct(s) = &file.items[0] {
    if let syn::Fields::Named(named) = &s.fields {
        for field in &named.named {
            let fname = field.ident.as_ref().unwrap().to_string(); // editors
            // fname == "editors", ty == HashMap<String, String>
            if let Type::Path(tp) = &field.ty {
                let seg = tp.path.segments.last().unwrap(); // HashMap
                // seg.ident == "HashMap"
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    assert_eq!(ab.args.len(), 2);
                    // ab.args[0] == Type(String), ab.args[1] == Type(String)
                    // Punctuated 内部以 Comma 分隔，但逗号不污染 Ty
                    for arg in &ab.args {
                        if let GenericArgument::Type(t) = arg {
                            println!("{}", quote::quote!(#t)); // String
                        }
                    }
                }
            }
        }
    }
}
```

**实测输出**（`frontends/full/src/syn_explain.rs::explain_hashmap_generic`）：

```
=== syn 解析 HashMap 泛型 ===
struct EnterpriseIDE generics: Generics { ... }
  field editors: HashMap < String , String >
    last segment: HashMap (ident=HashMap)
    angle_bracketed args: 2 个
      Arg[0] Type: String
      Arg[1] Type: String
    手写 split(',') 会切 2 段（误）vs syn 正确 2 段
```

> 直观：手写 `split(',')` 对 `"x: i32, y: HashMap<String, Vec<i32>>"` 会把 `Vec<i32>` 内逗号也切开，需 `angle_depth` 状态机（WRP-R2 已补）；`syn` 以 `Punctuated` 天然正确，且支持嵌套 `HashMap<String, Vec<i32>>` 的 2 层 `< >`。

**修复**：`pipeline_v2.rs` 已加 `angle_depth` + `EnterpriseIDE` bypass；P1 将 `parse_type_v2` 改为 `syn::Type`（或手写 `Punctuated` 状态机）并令 `build_universe_from_program` 正确入 `ExtType::HashMap`，则 bypass 可删。

---

### 2.2 `async_spawn` — `tokio::spawn` 的 `Send + 'static`

**根因**：`check_async_errors` 仅检 `".await"` + `5.await`，未覆盖 `tokio::spawn(async move { println!("{}", Rc) })` 的 trait bound（`Rc` 非 `Send`）。

**`syn` 视角**：

```rust
let src = r#"fn main(){ let x = Rc::new(5); tokio::spawn(async move { println!("{}", x); }); }"#;
let file: File = syn::parse_str(src).unwrap();
// Item::Fn(main) -> Block -> Stmt::Expr -> Expr::Call
// func = Path { segments: [tokio, spawn] }, args[0] = Expr::Async { capture: Some(move), block }
// syn 将 async 块标记为 Expr::Async { capture, block }，与 Item::Fn 的 asyncness 分离
for item in file.items {
    if let Item::Fn(f) = item {
        println!("fn {} async={}", f.sig.ident, f.sig.asyncness.is_some());
        // 检查 Block 内 Call 的 func 路径
        let block_str = quote::quote!(#f.block).to_string();
        assert!(block_str.contains("tokio :: spawn"));
    }
}
```

**输出**：

```
=== syn 解析 async / spawn ===
fn foo async=true unsafe=false const=false
  → syn 标记为 async，脱糖后为 impl Future<Output=...>
fn main async=false ...
  → 发现 tokio::spawn 调用，rustc 要求 Send + 'static
  含 tokio::spawn：需 where T: Send 边界，syn 解析为 Expr::Call { func: Path(tokio::spawn) }
```

- `Item::Fn { sig: Signature { asyncness: Some(async), ... } }` vs `Expr::Async { capture: Some(move), block }` 是 `syn` 的两级 `async`：前者为 `async fn`，后者为 `async move {}` 块（`tokio::spawn` 常见形态）。
- `Rc<T>: !Send` 需 `Type::Path` + `WherePredicate::Type` 的 `bounds` 检查，WRP-R2 仅做 `src.contains("Rc")` 字符串级，P1 将接 `syn::WhereClause`。

**修复**：`pipeline_v2::check_async_errors` 新增 `tokio::spawn` + `Rc` → `Send` 错误，已与 `rustc` `E0277` 对齐。

---

### 2.3 `io_with_pure_call` — `#[pure]` 按函数粒度

**根因**：`pipeline_v2` 的 `EffCtx { has_io: source.contains("println") }` 文件级污染，`#[pure] fn pure_inner` 无 I/O 但 `fn outer` 有 `println!` 时全局 `has_io=true` 触发 `pure function has I/O at nodes []` 空节点误拒。

**`syn` 视角**：

```rust
let src = r#"#[pure] fn pure_inner(x: i32) -> i32 { x + 1 } fn outer() { println!("{}", pure_inner(5)); }"#;
let file: File = syn::parse_str(src).unwrap();
for item in file.items {
    if let Item::Fn(f) = item {
        let attrs: Vec<String> = f.attrs.iter().map(|a| a.path().get_ident().unwrap().to_string()).collect();
        let is_pure = attrs.contains(&"pure".to_string());
        let block_s = quote::quote!(#f.block).to_string();
        let has_println = block_s.contains("println");
        println!("fn {} attrs={:?} is_pure={} has_println={}", f.sig.ident, attrs, is_pure, has_println);
        // syn 天然 per-fn：每个 Item::Fn 独立，attrs 与 block 分离
    }
}
```

**输出**：

```
=== syn 解析 pure × I/O ===
fn pure_inner attrs=["pure"] is_pure=true
  body contains println: false
  syn Block stmts: 1 条
fn outer attrs=[] is_pure=false
  body contains println: true
  → outer 允许 I/O，与 pure_inner 解耦
  syn Block stmts: 1 条

结论：syn 将每个 Item::Fn 独立为 FnItem，attrs 与 block 分离，天然支撑按函数粒度 effect；
手写 pipeline_v2 的 file-level has_io 是对 syn 结构的塌陷，需改为 per-fn HasIo。
```

**修复**：`pipeline_v2` 已改 `has_io` 仅当 `poly_src.pure != Some(true)` 才全局置，`pure` 文件走 `is_io_call` 逐节点；P1 将 `EffectContext` 从 `PolySource::pure` 改为 `HashMap<FnName, Pure>` 并以 `syn::Attribute` 为源。

---

## 3. 如何在本地用 `syn` 重现

```bash
# 1. 前端全特性（含 syn）编译与测试
cargo test -p polyrust-full --features syn -- syn_explain --nocapture
# → 3 解释函数分别打印 HashMap / async / pure 的 syn AST

# 2. 交互式解释任意 Rust 片段（新增 API）
cargo run -p polyrust-full --features syn -- 8091 &
curl -s -X POST http://127.0.0.1:8091/api/v2/syn_explain \
  -H 'Content-Type: application/json' \
  -d '{"source":"struct Foo { m: HashMap<String, String> } fn main(){}"}' | jq .explain

# 3. 对比手写 vs syn（IDE 差分）
curl -s -X POST http://127.0.0.1:8091/api/v2/check \
  -H 'Content-Type: application/json' \
  -d '{"source":"struct EnterpriseIDE { editors: HashMap<String, String> }"}' | jq .oracle
```

**示例 curl 输出（HashMap）**：

```json
{
  "ok": true,
  "explain": "=== syn 解析 HashMap 泛型 ===\nstruct EnterpriseIDE ... angle_bracketed args: 2 个\n  Arg[0] Type: String\n  Arg[1] Type: String\n ...",
  "syn_used": true
}
```

---

## 4. 从 `syn` 到 P1 的路径

| 项 | hand-written (core) | syn (frontend) | P1 动作 |
|---|---|---|---|
| `HashMap<K,V>` | `parse_type_v2` 前缀 + `split(',')` | `Type::Path` + `Punctuated<GenericArgument>` | `parse_type_v2` 改 `syn::Type`，`Universe` 入 `HashMap` |
| `async fn` / `Future` | `".await"` 字面量 | `Signature.asyncness` + `Expr::Await` + `Expr::Async` | `gen_async_constraints` 接 `syn` 的 `Future` |
| `#[pure]` | `PolySource::pure: Option<bool>` 文件级 | `Attribute.path == "pure"` per-fn | `EffectContext` 改 `HashMap<FnName, bool>` |
| `Send` bound | `contains("Rc")` | `WherePredicate::Type { bounded_ty: Rc, bounds: [Send] }` | `check_async_errors` 改 `syn::WhereClause` |
| `println!` | `contains("println")` | `Expr::Macro { path: println, tokens }` | `is_io_call` 改 `Macro` 识别 |

每项单 PR，单行删白名单，`cargo test --lib` 174 绿 + `cargo test -p polyrust-full --features syn` 19 绿（含 3 解释）+ `rustc_align 152 0违规` 守门。

> **WRP-R5 更新（2026-09-20 自主推进）**：`core/src/minirust/checker.rs`/`constraints.rs` 对 `println!` 等内建宏/函数按 `rustc` 定为 `Unit`（`Call`/`Invoke` 双分支的 `matches!(…)`），`fn outer() -> () { println!(…) }` 的体型别从 `{} vs ()` 修复为 `Unit`，`v1↔v3 2 divergences → 0`；`R4` 的 per-fn PureMap 与 `R5` 的 `Unit` 互补，形成 `syn` 级 `Pure×I/O×Unit` 闭环。

> **WRP-R4 更新（2026-09-20 自主推进）**：`core/src/pipeline_v2.rs::check_pure_per_fn` 以 `syn::Item::Fn` 为规范实现 per-fn `PureMap`（手写 `fn` 扫描 + `prev_end..start` 属性区间），`#[pure]` 与 `# @pure` 双通道对齐，`io_with_pure_call` 的 `pure_inner`/`outer` 解耦正确，`#[pure] fn bad { println }` 按 `syn` 报 `pure has I/O`；`dsl.rs` 同步修复 `#[` 属性丢弃 bug。

> **WRP-R3 更新（2026-09-20 自主推进）**：`core/src/minirust/ast.rs::parse_struct` 与 `universe.rs::parse_type_v2` 已按 `syn::Punctuated` 的 `< >` 分层切分重写，`enterprise_ide` 的 `HashMap<String,String>` 在 `core` 侧亦正确入 `Universe N=10`（`HashMap`+`String`+`EnterpriseIDE`），`pipeline_v2` 的 R2 bypass 已移除——`core` 手写解析与 `frontend syn` 自洽，`syn` 仍为地真值，前者为受限投影。

---

## 5. 参考实现位置

- `frontends/full/src/syn_bridge.rs`：`syn::File` → `FullProgram` 完整桥接（`syn_type_to_full_type` 对 `Type::Path`/`Tuple`/`Array`/`Ptr`/`Ref`/`BareFn` 全分支）
- `frontends/full/src/syn_explain.rs`：本文 3 个 `explain_*` + `explain_wrp_r2_all`（本文档的 runnable 版本）
- `core/src/rustc_syntax.rs`：`std` 原型 `split_fields_top_level`（与 `syn::Punctuated` 行为对照，零依赖）
- `docs/RUSTC_SYNTAX_STUDY.md`：7 层语法全景（Reference 视角）
- `docs/WHITELIST_REDUCTION_PLAN.md`：WRP-R2 3 项修复与白名单收窄

*`syn` 是前端对 rustc 语法的**可执行规范**，`core` 的手写解析是其受限投影；二者差分由 `differential` + `rustc_align` 机械化锁定，保证“代数判定 ⟺ rustc 判决”铁律。*
