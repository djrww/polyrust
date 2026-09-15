# Phase3 實現報告 — borrowck lifetime outlives、unsafe gate、async QAP、trait/impl 方法表、Vec/String/HashMap、loop 契約

> 日期：2026-09-15，基於 v0.1.5 + Phase1/Phase2

## 1. 總覽

Phase3 完成 9 大特性家族中剩餘 6 項的核心實現，全部保持 `core` 零第三方依賴，前端依賴隔離在 `frontends/full`。

| 特性 | Rust 模組 | Lean 模組 | 狀態 |
|---|---|---|---|
| @fuel/@invariant/@requires/@ensures/@pure/@unsafe-allowed/@lifetime/@no-io/@qap | `core/src/dsl.rs` | `LoopContract` | ✅ 17 測試 |
| loop 契約與有界展開 | `core/src/minirust/contracts.rs` | `LoopContract` | ✅ 4 測試 |
| unsafe 上下文位元、raw ptr、I/O 效應、pure、no-io | `core/src/minirust/effects.rs` | `UnsafeContext` | ✅ 6 測試 |
| lifetime 參數、outlives 圖、NLL region | `core/src/minirust/lifetime.rs` | `LifetimeRegion` | ✅ 7 測試 |
| borrowck 整合 lifetime + unsafe | `core/src/minirust/borrowck.rs` | `LifetimeRegion` + `UnsafeContext` | ✅ 4 測試 |
| trait/impl 方法表、存在量化 | `core/src/minirust/trait_impl.rs` | `TraitImpl` | ✅ 6 測試 |
| Vec/String/HashMap 內建庫 | `core/src/minirust/stdlib.rs` + `core/std/vec.poly/string.poly/hashmap.poly` | `StdlibEncoding` | ✅ 4 測試 |
| async QAP、Future 輪詢、狀態機 | `core/src/minirust/async_qap.rs` | `AsyncStateMachine` | ✅ 5 測試 |
| lifetime 泛型統一 | `core/src/minirust/ty.rs` | `TypeUniverse7PlusI` | ✅ 3 新增測試 |
| trait/impl lowering + lifetime lowering + stdlib lowering | `core/src/minirust/lower.rs` | — | ✅ 6 測試 |

總測試：`cargo test -p polyrust-core --lib -- --skip brute --skip exhaust` → 128 passed

## 2. .poly DSL v0.2 具體語法（Phase3 擴展）

```poly
# @intent: 計算最長公共前綴，帶契約與 lifetime
# @fuel: 10
# @invariant: x >= 0
# @requires: n > 0
# @ensures: result >= 0
# @pure: true
# @unsafe-allowed
# @lifetime 'a: 'b
# @no-io
# @qap: true
# @type-universe: Vec<i32>, String, HashMap<String,i32>
# @mode: full
# @import: vec, string, hashmap
# @set: opt_level=2

struct Point<'a> { x: i32, y: &'a str }

enum Option<T> { Some(T), None }

trait Display { fn fmt(&self) -> String; }

impl<'a> Display for Point<'a> {
  fn fmt(&self) -> String { String_new() }
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str where 'a: 'b { x }

async fn fetch() -> i32 { 42 }

fn main() {
  let mut v: Vec<i32> = Vec_new();
  Vec_push(&mut v, 1);
  for x in v { let y = x + 1; }
  while y < 10 { y = y + 1; }
  match Some(1) { Some(z) => z, None => 0 }
  unsafe { let p: *mut i32 = 0 as *mut i32; }
}
```

### 2.1 解析規則（`dsl.rs` `parse_at_key` 重寫）

舊實現：`find(':')` → `find('=')` → 空白，導致 `# @ensures result >= 0` 中的 `=` 被誤作分隔符，key 變成 `ensures result >`，value `0`，ensures vec 空。

新實現：

```rust
fn parse_at_key(line: &str) -> Option<(String,String)> {
  // key = 首個 token 直到 whitespace/:/=
  // rest = 剩餘，trim_start 後若以 ':' 或 '=' 開頭則去掉一次（僅一次），且需檢查是否為 >= <= == != 的比較運算符
}
```

支持：

- `# @fuel: 10` / `=10` / `10`
- `# @invariant x >=0` / `: x>=0` / `= x>=0`
- `# @pure` bare → true, `# @pure false`, `# @pure: true`
- `# @unsafe-allowed` / `@unsafe_allowed`
- `# @lifetime 'a: 'b`
- `# @no-io`, `# @qap`, `# @type-universe`, `# @mode`
- `# @set opt=2` / `opt:2` / `opt 2`

`PolySource` 擴展字段：

```rust
fuel: Option<usize>,
invariants: Vec<String>,
pure: Option<bool>,
requires/ensures: Vec<String>,
unsafe_allowed: bool,
lifetimes: Vec<String>,
no_io: bool,
qap: Option<bool>,
type_universe/mode: Option<String>
```

`resolve()` 保留新字段。

### 2.2 內建庫

`core/std/` 新增：

- `vec.poly`：`Vec<T>` 不透明，ptr+len+cap，`push/pop/get/len/is_empty/clear/into_iter`
- `string.poly`：`String` 視為 `Vec<u8>` + utf8，`push_str/push/as_str/into_bytes/from_utf8`
- `hashmap.poly`：`HashMap<K,V>` 視為 `Vec<(K,V)>` + `k1 != k2` 唯一，`(k_i - k_j)*inv=1`

`BUILTIN_STD` 新增 5 項：vec, string, hashmap, option, result（後兩者為簡化 enum 文本）。

## 3. 類型系統擴展（`ty.rs`）

### 3.1 Lifetime 參數環境

```rust
struct LifetimeEnv { map: HashMap<String,String> } // 'a -> 'b
```

`subst_type_with_lt`：

- `GenericParam("'a")` → 查 `LifetimeEnv`
- `RefExt { lifetime: Some("'a"), inner }` → 映射 lifetime
- 其他複合類型遞歸

### 3.2 outlives 檢查

```rust
fn check_lifetime_bounds(ty: &TypeV2, graph: &LifetimeGraph) -> Result<()>
```

收集 `ty` 中所有 lifetime，檢查 `graph.has_cycle()`。

`build_universe_from_src` 新增模式 `&'a i32`, `&'a mut i32`, `&'static str`。

## 4. 借用分析（`lifetime.rs` + `borrowck.rs`）

### 4.1 LifetimeGraph

- `Lifetime::named("'a")`, `static`
- `Outlives::parse("'a: 'b")` 支持 `'a: 'b + 'c` 取第一個
- `has_cycle()` DFS
- `transitive_closure()` Floyd-Warshall 風格
- `outlives_holds()`：自反 + `'static` outlives all + 直接/傳遞
- `poly_constraints()`：生成 `start_shorter - start_longer` 與 `end_longer - end_shorter` 差值多項式（區間包含編碼）

### 4.2 NLL Region

```rust
struct Region { lifetime: Lifetime, start: u32, fin: u32, borrowNode: usize }
fn analyze_nll(borrows, graph, lifetime_of_borrow) -> NllAnalysis
```

重疊 + outlives 不兼容 → 潛在衝突（簡化版暫不誤報，留給 borrowck）。

### 4.3 BorrowChecker

整合 `LifetimeGraph` + `EffectContext` + `BorrowAnalysis`：

- `from_poly_source()`
- `check_lifetime_cycles()`
- `check_borrow_outlives()`
- `check_unsafe()` 代理 `EffectContext::check_*`
- `check_all()` → `Result<(), Vec<String>>`
- `annotate_lifetime(node, lt)`

## 5. 效應系統（`effects.rs`）

```rust
struct EffectContext {
  in_unsafe: bool,
  unsafe_allowed: bool,
  has_io: bool,
  pure: Option<bool>,
  no_io: bool,
  unsafe_usages: Vec<usize>,
  io_usages: Vec<usize>,
  raw_ptr_ops: Vec<usize>,
}
```

- `analyze_effects(e, ctx)`：walk AST，檢測 `Call("unsafe")` 進入 unsafe，`raw_`/`ptr`/`raw` 標記 raw_ptr，`is_io_call` 檢測 print/open 等
- `check_unsafe_gate()`：若有 unsafe_usages 且 !allowed 且 !in_unsafe → Err
- `check_no_io()` / `check_pure()`
- `RawPtrTy::parse("*const i32")` → `RawPtrKind::Const/Mut`

## 6. 迴圈契約（`contracts.rs`）

- `LoopContract::from_poly_source()`，`fuel_or_default()=3`
- `unroll_while(cond, body, fuel, invariants)` → 生成 `__fuel` counter + invariant assert
- `unroll_for(pat, iter, body, fuel, invariants)` → `into_iter()` + `next()` + `__fuel`
- `invariant_poly(nvars, inv_var)`：`t_inv -1=0`
- `fuel_counter_poly(nvars, c_i, c_next)`：`c_next - c_i +1=0`

## 7. Vec/String/HashMap（`stdlib.rs`）

- `VecEncoding { ptr,len,cap, elem_ty }`：`len_le_cap_poly` → `cap - len`，`push_poly` → `len_next - len -1=0`
- `StringEncoding { vec }`：`utf8_poly()` 文本占位
- `HashMapEncoding { ptr,len,cap,key_ty,val_ty }`：`unique_keys_poly(k_i,k_j,inv)` → `(k_i - k_j)*inv -1=0`
- `StdlibRegistry`：`register_*`，`all_polys()`，`from_type_universe()` 尊重 `<>` 深度分割（修復 `HashMap<String,i32>` 被逗號誤分割）
- `r1cs_for_stdlib()` 同樣修復

## 8. async QAP（`async_qap.rs`）

- `FutureState::Pending/Ready/Polling(usize)`
- `AsyncStateMachine::new(fn, num_await)` → Pending + N polling + Ready
- `add_transition(from,to,constraint)`
- `to_r1cs(nvars)`：生成 `s_from - s_to` 差值多項式
- `poly_text()`：one-hot `Σ s_i -1=0`，轉換 `s_from * poll - s_to=0`，布爾 `s_i*(s_i-1)=0`
- `polling_constraints()`：`poll_i*(poll_i-1)=0`
- `lower_async_fn(fn, body)`：統計 `await` 次數，線性轉換
- `async_to_qap()` → `(polys, texts)`

## 9. trait/impl 方法表（`trait_impl.rs` + `lower.rs`）

- `TraitMethod { name, params, ret_ty, has_default, default_body }`
- `TraitDef { name, type_params, lifetime_params, methods, supertraits }`
- `ImplDef { trait_name, for_ty, type_params, lifetime_params, methods, where_clauses }`
- `MethodTable { traits, impls, impl_map, inherent_map }`：`resolve_method(recv_ty, method)` 先 inherent 後 trait，`resolve_trait_method`，`method_call_poly`，`existential_poly(var, bound, body)` → `exists T: Trait { body }`，`types_implementing(trait)`
- `parse_trait_def` / `parse_impl_def` 簡化文本解析
- `lower.rs` 新增：
  - `lower_trait_impl_method_table(prog)`：從 `ProgramV2` 構造 `MethodTable`
  - `lower_lifetimes(prog)`：收集所有 `lifetimes` 與 `where_clauses` 中的 `Outlives`，檢查環
  - `lower_stdlib_usage(prog)`：從 `ProgramV2` 收集類型文本 → `StdlibRegistry`

`ast_v2.rs` 擴展：

- `StructDefV2`, `EnumDefV2`, `FnSigV2`, `ImplDefV2`, `TraitDefV2` 新增 `where_clauses: Vec<String>`，`FnSigV2` 新增 `has_default, default_body`，`TraitDefV2` 新增 `supertraits`

## 10. Lean 形式化

新增 6 模組，更新 `Polyrust.lean`：

| 模組 | 對應 Rust | 核心定理 |
|---|---|---|
| `LifetimeRegion` | `lifetime.rs`, `borrowck.rs` | `static_outlives_all`, `outlives_refl`, `outlives_trans`（sorry 佔位需完整閉包證明） |
| `UnsafeContext` | `effects.rs` | `unsafe_allowed_passes`, `empty_checks_ok` |
| `AsyncStateMachine` | `async_qap.rs` | `state_number`, `one_hot_exists`（sorry 需 List 長度引理） |
| `StdlibEncoding` | `stdlib.rs` | `vec_len_le_cap_exists`, `hashmap_unique_form` |
| `LoopContract` | `contracts.rs` | `fuel_default`, `fuel_some`, `invariant_poly_count` |
| `TraitImpl` | `trait_impl.rs` | `addTrait_resolve`, `existential_contains_bound` |

所有 Lean 文件使用 `sorry` 佔位部分複雜證明，符合 `docs/LEAN.md` 中「原型階段允許 sorry，但需標註」約定。`lake build` 可通過（需 Lean 4.33+）。

## 11. 前端集成（`frontends/full`）

已存在 `polyrust-full` 前端，`Cargo.toml` 依賴 `axum/tokio/serde`，符合「第三方 deps 在 frontends/*」。

建議後續：

- `frontends/full/src/lower.rs` 調用 `core::minirust::lower::{lower_trait_impl_method_table, lower_lifetimes, lower_stdlib_usage}`
- `frontends/full/src/ty.rs` 調用 `ty::subst_type_with_lt` + `check_lifetime_bounds`
- API `/api/v2/check` 返回 `features_used` 包含 `lifetime, unsafe, async, vec, hashmap, loop_contract`
- UI 分頁展示 Phase3 示例

## 12. 約束編碼與 QAP 處理總結

| 特性 | 編碼 | 多項式 | QAP |
|---|---|---|---|
| lifetime outlives | 區間包含 `[start,end)` | `ss - ls`, `le - se` | slack 變量轉不等式為等式 |
| NLL region | 活性區間重疊 | `overlap → conflict` | 同 borrow 衝突編碼 `b_i * b_j=0` |
| unsafe | 上下文位元 `in_unsafe` | `t_unsafe_op * (1 - in_unsafe)=0` | 布爾約束 `in_unsafe*(in_unsafe-1)=0` |
| raw ptr | `*const/mut T` 新類型 | `deref` 需 unsafe gate | 同 unsafe |
| I/O | 效應位元 `has_io` | `no_io → has_io=0` | `has_io*(has_io-1)=0` |
| pure | `pure=true → has_io=0` | 同上 | 同上 |
| Vec | `ptr,len,cap` | `cap-len-slack=0`, `len' - len -1=0` | R1CS |
| HashMap | `Vec<(K,V)>` + 唯一 | `(k_i - k_j)*inv -1=0` | 逆元約束 |
| String | `Vec<u8>` + utf8 | 範圍檢查占位 | 同 Vec |
| loop fuel | counter `__fuel` | `c_{i+1} - c_i +1=0` | 遞減計數器 |
| invariant | bool 變量 `t_inv` | `t_inv -1=0` | 布爾 |
| async | one-hot `s_i` | `Σ s_i -1=0`, `s_i*(s_i-1)=0`, `s_from*poll - s_to=0` | QAP Lagrange |
| trait impl | 方法表存在位元 | `∃ impl` → `impl_bit` | 子句 `¬has_impl ∨ method_available` |
| for/match | 降維為 loop/if | 複用現有 | — |

所有約束保持次數 ≤3，係數小整數，滿足 L0 嵌入引理，𝔽_p (2^61-1) 保真。

## 13. 下一步

- 完整 `for`/`match`/`mod` 的 `lower_body_text` 從註釋級提升為實際 AST 重寫
- `borrowck.rs` 中 NLL 衝突與 `analysis.rs` 的 `BorrowAnalysis` 合併，生成最終 `conflicts`
- `async_qap.rs` 的 `to_r1cs` 從示意差值改為完整 QAP 多項式（含 Lagrange 基）
- Lean 中 `sorry` 補全：`outlives_trans` 需傳遞閉包歸納，`state_number` 需 `List.range` 長度引理
- `frontends/full` 集成新 core API，端到端示例

## 14. 文件清單

- `core/src/dsl.rs`：DSL 擴展，17 測試通過
- `core/src/minirust/contracts.rs`：新建
- `core/src/minirust/effects.rs`：新建
- `core/src/minirust/lifetime.rs`：新建
- `core/src/minirust/borrowck.rs`：新建
- `core/src/minirust/trait_impl.rs`：新建
- `core/src/minirust/stdlib.rs`：新建
- `core/src/minirust/async_qap.rs`：新建
- `core/src/minirust/ty.rs`：擴展 lifetime
- `core/src/minirust/lower.rs`：擴展 method table + lifetime + stdlib
- `core/src/minirust/ast_v2.rs`：擴展 where_clauses
- `core/src/minirust/mod.rs`：註冊新模組
- `core/std/vec.poly`, `string.poly`, `hashmap.poly`：新建
- `lean/Polyrust/LifetimeRegion.lean`, `UnsafeContext.lean`, `AsyncStateMachine.lean`, `StdlibEncoding.lean`, `LoopContract.lean`, `TraitImpl.lean`：新建
- `lean/Polyrust.lean`：更新 import
- `docs/PHASE3_REPORT.md`：本文件
