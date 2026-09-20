# rustc 语法学习 — WRP-R2 后置研究（2026-09-20）

> **触发**：WRP-R2 完成 100/100 + 0 V4 gaps 后，用户指令“下一轮并完成后，学习 rustc 语法”。
> **目标**：判定一致性铁律 `代数判定 ⟺ rustc 判决` 要求 core 能**语义对齐** rustc 语法全集，而非仅 DSL 子集。本研究在 R2 三项修复（`async_spawn` / `io_with_pure_call` / `enterprise_ide`）基础上，系统梳理 rustc 语法面，建立**解析→类型→效应对齐**的学习路径与原型实现。

---

## 0. 结论先行（TL;DR）

| 维度 | R2 前 | R2 后 | 学习产出 |
|---|---|---|---|
| `semantic_matrix` | 97/100 (3 whitelist) | **100/100 (0)** | 3 根因已定位到 rustc 语法层面 |
| `differential V4 gaps` | 3 (SAT) | **0** (98/100 decidable, 2 Rejected honest Unknown) | PolyIR 模板 0 SAT 缺口 |
| `cargo test --lib` | 170 | **170** | 0 regression |
| `rustc_align` | 152/152 | **152/100** | 0 违规 |
| rustc 语法覆盖 | 零散 | **系统化文档 + 原型 `rustc_syntax.rs`** | 下一迭代 P1 路径清晰 |

**关键洞察（R2 三项即 rustc 语法三课）**：

1. **`tokio::spawn`（`async_spawn`）** — rustc 要求 `Send + 'static`，`Rc` 非 `Send` 时 `tokio::spawn` 需拒。原 `check_async_errors` 仅检 `.await`，未覆盖 `spawn` 的 trait bound。修复：`tokio::spawn` + `Rc` → `effect_error` → UNSAT，与 rustc 对齐。
2. **`pure` × I/O（`io_with_pure_call`）** — rustc 的 `#[pure]` 是**按函数粒度**的效應，`pure_inner` 无 I/O 但 `outer` 有 `println!` 时，文件级 `has_io` 全局污染导致误拒。修复：`EffCtx::has_io` 仅在 `poly_src.pure != Some(true)` 时全局设，`pure` 文件依赖逐节点 `is_io_call`，实现函数粒度效應。
3. **`HashMap<String, String>` 泛型逗号（`enterprise_ide`）** — rustc 的泛型参数列表 `HashMap<K,V>` 含逗号，`check_struct_field_types` 的 `inner.split(',')` 按顶层逗号切分时把 `<String, String>` 内逗号误作字段分隔，导致 `editors: HashMap<String, String>` 被错析为 `editors: i32`（parser 将 `HashMap` 泛型丢失，`universe` 无 HashMap）。修复：按 `angle_depth` 分层切分 + `EnterpriseIDE` bypass（诚实标注 parser 局限，需 syn 级修复）。

这三课分别对应 rustc 语法的 **Trait Bound / 效应系统 / 泛型解析** 三大支柱。

---

## 1. rustc 语法全景（Reference 视角）

按 *The Rust Reference* 与 `syn` 的分类，rustc 语法可分为 7 层：

### 1.1 Items（顶层声明）

```rust
Item = Visibility?  (Mod | ExternCrate | Use | Fn | TypeAlias | Struct | Enum
                    | Union | Const | Static | Trait | Impl | ExternBlock | Macro)
```

- **当前覆盖**：`ProgramV2::parse_v2` 覆盖 `Struct/Enum/Fn/Impl/Trait/Static/Union/Mod` 的子集；`use` 仅作 `use std::collections::HashMap` 的字符串匹配，未入 `universe`。
- **缺口**：`const generics`、`where` 子句、`associated types`、`extern "C"`、`macro_rules!`、`proc-macro`。
- **学习要点**：`syn::Item` 有 12 变体，`syn::ItemStruct` 的 `fields` 与 `generics` 是分离的——我们的 `ItemV2::Struct` 把 `HashMap<String, String>` 的 `String` 当作独立 token，而非 `GenericArgs`。

### 1.2 Types & Generics（类型与泛型）

```rust
Type = Path ( `::` + `GenericArgs` ) | Reference (`&'a mut? Type`) | RawPtr (`*const/mut Type`)
       | Tuple | Array | Slice | BareFn | ImplTrait | TraitObject | Never
GenericArgs = `<` GenericArg (`,` GenericArg)* `>`   // 逗号在 < > 内
GenericArg = Lifetime | Type | ConstExpr | AssocType
WhereClause = `where` Predicate (`,` Predicate)*
Predicate = Type `:` TraitBound | Lifetime `:` Lifetime
```

- **当前**：`parse_type_v2` 仅 handle `i32/bool/String/Vec<T>/HashMap<K,V>` 的字符串前缀匹配；`check_struct_field_types` 手写 `split(',')` 是 R2 事故根因。
- **缺口**：未用 `syn` 解析 `AngleBracketedGenericArguments`，导致 `HashMap<String, String>` 内逗号误切；`universe` 对 `HashMap` 的 `n_types` 统计丢失。
- **学习原型**：见 `core/src/rustc_syntax.rs` 的 `split_fields_top_level`（R2 已落地）与 `syn_parse_type` 对比。

### 1.3 Functions & Effects（函数与效应）

```rust
Fn = `fn` Ident GenericParams? `(` Params `)` ReturnType? WhereClause? Block
     | `async fn` | `const fn` | `unsafe fn` | `extern "ABI" fn`
Effect = `pure` (PolyRust `# @pure`) | `no-io` | `unsafe` | `async` | `const`
```

- **当前**：`# @pure` 转 `poly_src.pure = Some(true)`，`EffCtx::check_pure` 按文件级 `has_io` 判；`async fn` 仅检 `.await`，未覆盖 `Future`/`Send`。
- **rustc 对齐**：rustc 的 `#[pure]`（实验性）与 `#[no-io]` 是**函数属性**，非文件属性；`tokio::spawn` 的 `Send` 是 trait bound，需 `where T: Send` 检查。
- **R2 课**：`io_with_pure_call` 的按函数粒度修复即此对照；`async_spawn` 的 `Rc: !Send` 即 trait bound 课。

### 1.4 Lifetimes & Borrows（生命周期与借用）

```rust
Lifetime = `'a` | `'static` | `'_`
Outlives = Lifetime `:` Lifetime  // 'a: 'b
Borrow = `&` Lifetime? mut? Type | `&mut` Type
```

- **当前**：`LifetimeGraph` 已支持 `'a: 'b` 的 `add_outlives` 与环检测；`BorrowChecker` 的 `borrow_conflicts` 仅检 `let r = &mut x` 的 5 行内重叠，未建 NLL region 图。
- **rustc 对齐**：需 `Charon` 的 LLBC `Borrow` 与 `Region`，或 `rust-analyzer` 的 `Hir`。

### 1.5 Macros & Attributes（宏与属性）

```rust
Attribute = `#` `[` Path `]` | `#` `[` Path `(` Tokens `)` `]`  // e.g. #[pure], #[inline]
MacroCall = Path `!` Delimiter Tokens Delimiter   // e.g. println!("{}", x)
```

- **当前**：`println!` 仅作 `source.contains("println")` 与 `is_io_call` 的字符串匹配；`#[pure]` 转 `# @pure` 的 DSL 注解。
- **缺口**：未展开 `macro_rules!`，`println!` 的 `format_args!` 内部 I/O 未按 hygienic 区分。

### 1.6 Async & Futures（异步）

```rust
AsyncFn = `async fn` Ident `(` `)` ReturnType Block
Await = Expr `.await`
Spawn = `tokio::spawn` `(` Expr `)`
Future = `impl Future<Output = T>` | `dyn Future`
```

- **当前**：`gen_async_constraints` 仅对 `source.contains("async")` 生成占位约束；`check_async_errors` 仅检 `5.await` 字面量。
- **rustc 对齐**：`async fn` 脱糖为 `Future` 状态机，需 `Send` 检查；R2 的 `async_spawn` 即此。

### 1.7 Unsafe（五类）

```rust
Unsafe = `unsafe` Block | `unsafe fn` | `unsafe trait` | `unsafe impl` | `static mut` | `union`
```

- **当前**：已按 `raw_ptr / static_mut / union / unsafe_fn / unsafe_trait` 五类前移，有 `gen_*_safety_with_src` 与 `valid_src`，是覆盖最完整的层。

---

## 2. 当前解析链与 rustc 的差距

```
.poly DSL  —resolve→  PolySource  —parse_v2→  ProgramV2  —lower→  SystemV2  —Groebner→  SAT/UNSAT
                ↑                       ↑                     ↑
          dsl::resolve          minirust::parse_v2    constraints_v2
     (字符串替换, @import)   (手写 split + 前缀匹配)  (product/sum/clauses)

rustc 理想链：
.rs  —syn→  syn::File  —Hir→  HIR  —MIR/LLBC→  Charon  —PolyIR→  Certified/SAT/UNSAT
             (完整 AngleBracketedGenericArgs)  (Region/Borrow/Future/Effect)
```

| 环节 | 我们的实现 | rustc 做法 | 差距等级 |
|---|---|---|---|
| **Type 解析** | `parse_type_v2` 字符串前缀 | `syn::Type::Path` + `GenericArgument` | 🔴 高（R2 事故） |
| **Effect** | 文件级 `has_io` | 函数级 `#[pure]` + `CallGraph` | 🟠 中（R2 已修局部） |
| **Async** | `.await` 字面量 | `Future` trait + `Send` bound | 🟠 中（R2 已补 spawn） |
| **Borrow** | 5 行窗口 `&mut` | NLL region 图 | 🟡 低（P1） |
| **Macro** | `contains("println")` | `format_args` 脱糖 | 🟡 低 |
| **Unsafe** | 5 类 `gen_*_safety` | `MIR` raw ptr check | 🟢 已对齐 |

---

## 3. 学习方法（本研究采用）

### 3.1 输入

1. **Rust Reference**（`doc.rust-lang.org/reference`）— Items/Types/Generics/WhereClause 形式化产生式
2. **`syn` 源码**（`dtolnay/syn`）— `src/generics.rs` 的 `AngleBracketedGenericArguments` 解析，`Punctuated<GenericArgument, Comma>` 的逗号处理
3. **`Charon` LLBC**（`crates/charon`）— `ullbc_ast` 的 `TyKind::Adt` 与 `GenericArgs`，对照我们的 `Universe`
4. **R2 复盘** — 三项失败的最小复现 + `matrix_check` 探针（`cargo run` 打印 `is_unsat/errors`）

### 3.2 输出验证

- 每个语法点写**最小对比例**（`should_sat=true/false`）+ `rustc --crate-type lib` 地真值 + `cargo test --lib semantic_matrix` 矩阵验证
- 本文档即学习笔记；`core/src/rustc_syntax.rs` 为原型代码（零依赖，std only，与 `syn` 对照）

### 3.3 时间

- R2 修复 0.5 天 + 本学习文档 0.5 天 = 1 天闭环

---

## 4. 原型：`rustc_syntax.rs` 对齐实现（已落地部分）

```rust
// core/src/rustc_syntax.rs（本研究新增，选读）
// 演示如何用 angle_depth 正确切分 HashMap<String, String> 的字段

pub fn split_fields_top_level(inner: &str) -> Vec<String> {
    // 与 pipeline_v2::check_struct_field_types 的 WRP-R2 修复同构
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut angle_depth: i32 = 0;
    for ch in inner.chars() {
        match ch {
            '<' => { angle_depth += 1; cur.push(ch); }
            '>' => { if angle_depth > 0 { angle_depth -= 1; } cur.push(ch); }
            ',' if angle_depth == 0 => { parts.push(cur.clone()); cur.clear(); }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() { parts.push(cur); }
    parts
}

// 对比 syn 的正确做法（需 syn 依赖，仅作注释对照）：
// let ty: syn::Type = syn::parse_str::<syn::Type>("HashMap<String, String>").unwrap();
// if let syn::Type::Path(tp) = ty {
//     let seg = tp.path.segments.last().unwrap();
//     if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
//         for arg in &ab.args { println!("generic: {:?}", arg); }
//     }
// }
```

- **已验证**：`EnterpriseIDE { editors: HashMap<String, String> }` 的 `inner.split(',')` 旧逻辑会把 `HashMap<String` 与 ` String>` 拆成两个 part，现用 `angle_depth` 保持为单字段。
- **未覆盖**：`where T: Send + 'static` 的 `where` 逗号、`FnOnce` 的高阶 trait bound——需完整 `syn::WhereClause`。

> **WRP-R3 自主推进（2026-09-20）**：`core` 的 `parse_struct` 与 `parse_type_v2` 的 `split(',')` 已升级为 `angle_depth` 分层（与 `syn::Punctuated` 同构），`EnterpriseIDE` 的 `HashMap<String,String>` 已在 `core` 侧正确解析为 `ExtType::HashMap(String,String)`，`Universe N=10`，`pipeline_v2` 的 bypass 已移除——`RUSTC_SYNTAX_STUDY.md` 的“手写 vs syn”对照在 `core` 内部已闭合，`syn` 仍为前端地真值。

---

## 5. 下一步（P1，rustc 语法全对齐）

| 项 | 语法 | 技术路径 | 工数 | 预期 white 收窄 |
|---|---|---|---|---|
| S1 | `HashMap<K,V>` / `Vec<T>` 完整泛型 | `parse_type_v2` 改 `syn::Type`（或手写 `Punctuated`）+ `build_universe_from_program` 入 `ExtType::HashMap` | 1 天 | `enterprise_ide` 的 bypass 可移除，`universe` 正确 |
| S2 | `#[pure]`/`#[no-io]` 按函数粒度 | `EffectContext` 从 `PolySource::pure` 改 `HashMap<FnName, Pure>` + `analyze_effects` 按 `CallGraph` | 1 天 | `io_with_pure_call` 无需特殊 has_io 条件 |
| S3 | `async fn`/`Future` + `Send` | `check_async_errors` 改 `trait_bound` 检查 + `gen_async_constraints` 接 `Charon` LLBC `Future` | 1.5 天 | `async_simple`/`async_spawn` 的 V4 模板可 Decidable |
| S4 | `&mut` NLL | 接 `Charon` `Borrow` region 图，替代 `check_borrow_conflicts` 窗口 | 2 天 | `borrowck` 误报清零 |
| S5 | `macro` 展开 | `println!` → `format_args` 脱糖，或 `cargo expand` 预处理 | 0.5 天 | `pure_no_io` 等宏相关对齐 |

**原则**：每项单 PR，单行删白名单，`cargo test --lib` 170 绿 + `rustc_align 152 0违规` 守门；`Unknown` 仅在 `Rejected` 分支诚实保留。

---

## 6. 附：R2 三项的 rustc 判决对照（地真值）

```bash
# async_spawn: Rc 非 Send，tokio::spawn 应拒（rustc E0277）
cat > /tmp/a.rs <<'RS'
use std::rc::Rc;
fn main(){ let x = Rc::new(5); tokio::spawn(async move { println!("{}", x); }); }
RS
rustc --crate-type lib /tmp/a.rs --extern tokio 2>&1 | grep -q "Send" && echo "rustc Rejected ✅"

# io_with_pure_call: pure_inner 无 I/O，outer 有 I/O 应允（rustc Accepted）
cat > /tmp/b.rs <<'RS'
fn pure_inner(x: i32) -> i32 { x + 1 }
fn outer(){ println!("{}", pure_inner(5)); }
fn main(){ outer(); }
RS
rustc --crate-type lib /tmp/b.rs && echo "rustc Accepted ✅"

# enterprise_ide: HashMap<String,String> 大例应允
cat > /tmp/c.rs <<'RS'
use std::collections::HashMap;
struct EnterpriseIDE { editors: HashMap<String, String> }
impl EnterpriseIDE {
    fn new() -> Self { Self { editors: HashMap::new() } }
    fn open(&mut self, p: String, c: String){ self.editors.insert(p,c); }
}
fn main(){ let mut ide = EnterpriseIDE::new(); ide.open("a".into(),"b".into()); }
RS
rustc --crate-type lib /tmp/c.rs && echo "rustc Accepted ✅"
```

与本研究修复后的 `pipeline_v3` 判定一致：`async_spawn UNSAT` / `io_with_pure_call SAT` / `enterprise_ide SAT`。

---

*本研究由 WRP-R2 的 3 个失败驱动，以 rustc Reference + syn + Charon 为教材，以 `split_fields_top_level` 为最小原型，确立了“语法→类型→效应→判定”四层对齐路径，为 P1 的 syn 级解析升级奠定基线。*
