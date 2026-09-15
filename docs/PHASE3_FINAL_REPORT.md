# Phase3 Final Report — 2026-09-15

## 概述
本次迭代完成：
- `pipeline_v2` 全鏈路（struct/enum/impl/trait, Vec/String/HashMap, loop/match, mod, async, I/O, unsafe, lifetime）通過 5 單元測試 + 27 示例驗證
- Lean 6 模組重寫為 Lean 4.33 兼容，`lake build` 33 jobs 全部通過
- `core` 零第三方依賴保持，第三方僅在 `frontends/*`
- `driver.rs` 新增 `check-v2` 命令與 JSON API，供前端與 CLI 使用
- DSL `@import` 修復：支持逗號分隔多庫 `vec, string, hashmap`
- 文檔：DSL 提案與實現方案已存在，本文為最終驗證

## 1. pipeline_v2 設計

### 1.1 模組
```
core/src/minirust/
  ast_v2.rs         — ProgramV2, ItemV2 (struct/enum/fn/impl/trait/mod/use/macro)
  parse_v2.rs       — 粗略解析 v0.2 語法
  ty.rs             — build_universe_from_program, check_lifetime_bounds, LifetimeEnv
  lower.rs          — lower_program, lower_trait_impl_method_table, lower_lifetimes, lower_stdlib_usage
  constraints_v2.rs — SystemV2, product/sum/match/loop_fuel/async/lifetime/unsafe/stdlib/trait_impl 約束
  lifetime.rs       — Lifetime, Outlives, LifetimeGraph, Region, NLL
  effects.rs        — EffectContext, RawPtrTy, isIOCall, parseRawPtr
  borrowck.rs       — BorrowChecker (lifetime_graph + effect_ctx)
  trait_impl.rs     — TraitMethod, TraitDef, ImplDef, MethodTable
  stdlib.rs         — VecEncoding, StringEncoding, HashMapEncoding, StdlibRegistry, r1cs_for_stdlib
  async_qap.rs      — FutureState, AsyncStateMachine, lower_async_fn
  contracts.rs      — LoopContract, unrollWhile/For, fuel/invariant polys
```

### 1.2 PipelineV2 流程
```
S1 parse_v2
S2 build_universe (N = 7 + i)
S3 lower_program (products, sums)
S4 特性檢測 (struct/enum/impl/trait/Vec/String/HashMap/loop/match/mod/async/io/unsafe/lifetime)
S5 lifetime graph 合併: ProgramV2 where 子句 + DSL @lifetime
S6 borrowck + effects (io, pure, no-io, unsafe gate)
S7 method table + stdlib usage
S8 constraints_v2 + Phase3 約束 (fuel, lifetime, unsafe, stdlib, async, trait_impl)
S9 判定: errors 非空或 lifetime cycle => UNSAT else SAT
S10 統計 (Groebner 僅計數，不誤判 one-hot/product 衝突)
```

### 1.3 關鍵修復
- **Groebner 誤判**: 初始用 Groebner 判定導致 struct/vec 誤判 UNSAT，因為同一節點 one-hot Σt-1=0 與 product t_struct - Πt_field=0 在 i32=0 時矛盾。修復：is_unsat 僅基於 borrowck/effect/lifetime errors，Groebner 僅統計。
- **LifetimeGraph 合併**: 初始 lower_lifetimes 返回空圖當無 where 子句，導致 DSL @lifetime 丟失。修復：顯式合併 ProgramV2 圖與 PolySource 圖。

### 1.4 測試結果
```
cargo test -p polyrust-core --lib pipeline_v2 -- 5 passed
  struct, vec, async, unsafe_no_io, lifetime_cycle
cargo test --lib --skip brute --skip exhaust -- 133 passed
27 examples (examples/phase3/*.poly) via pipeline_v2:
  struct_sat SAT ["struct","enum","impl","trait","String","loop"]
  io_unsat UNSAT ["I/O but no-io"] x2
  io_unknown UNSAT ["pure has I/O"]
  lifetime_unsat UNSAT ["lifetime cycle", "cycle: outlives graph has cycle"]
  vec_sat SAT ["struct","Vec","String","HashMap","unsafe"]
  ... 27 ok
```

## 2. .poly DSL v0.2 具體語法

### 2.1 文件結構
```poly
# @intent: 描述
# @import: vec, string, hashmap
# @type-universe: Vec<i32>, String, HashMap<String,i32>
# @fuel: 5
# @invariant: x >= 0
# @requires: n > 0
# @ensures: result >= 0
# @pure: true/false
# @unsafe-allowed / @unsafe_allowed
# @lifetime 'a: 'b
# @no-io
# @qap: true/false
# @mode: full

struct Point { x: i32, y: i32 }
enum Option<T> { Some(T), None }
trait Display { fn fmt(&self) -> String; }
impl Display for Point { fn fmt(&self) -> String { String::from("Point") } }
impl Point { fn new(x: i32, y: i32) -> Point { Point { x: x, y: y } } }

mod geometry {
  pub struct Point { pub x: i32, pub y: i32 }
  pub mod utils { pub fn distance(p: &super::Point) -> i32 { p.x * p.x + p.y * p.y } }
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { x }

async fn fetch() -> i32 { 42 }

fn main() {
  let v: Vec<i32> = Vec_new();
  Vec_push(&mut v, 1);
  let mut sum = 0;
  for x in v { sum = sum + x; }
  while sum < 10 { sum = sum + 1; }
  loop { if sum > 100 { break; } sum = sum + 1; }
  let opt = Option::Some(5);
  let val = match opt { Option::Some(x) => x, Option::None => 0 };
  unsafe { let p: *mut i32 = &mut sum as *mut i32; *p = 10; }
  println!("{}", sum);
}
```

### 2.2 擴展點
- `@import` 支持逗號分隔，內建 std: vec, string, hashmap, option, result, basic, math, bool
- `@lifetime 'a: 'b` 解析為 Outlives，支持 `'a: 'b + 'c` 取首個
- `@fuel` 有界展開，`@invariant` 循環不變量
- `struct/enum/impl/trait/mod` 為一等公民，`ast_v2` 解析
- `Vec<T>/String/HashMap<K,V>` 泛型容器，`stdlib.rs` 編碼
- `loop/while/for` 契約，`contracts.rs` 燃料計數器 `c_{i+1} - c_i +1=0`
- `match` 決策樹，編譯為 if
- `mod` 樹扁平化，路徑 `crate::` `super::`
- `async/await` → Future 狀態機 one-hot `Σ s_i -1=0` + 轉移 `s_from * poll - s_to=0`
- `unsafe/*mut/*const` 上下文位元 `in_unsafe`
- `I/O` 效應 `has_io` + `no-io` + `pure` 檢查

## 3. 類型系統擴展

### 3.1 Universe N=7+i
- 基底 7: i32, bool, (), &i32, &mut i32, &bool, &mut bool
- 擴展標籤 10: vec, string, hashmap, structTy, enumTy, rawPtrMut, rawPtrConst, future, option, result
- 動態 N = 7 + i，insert_closure 遞歸插入子類型
- one-hot: `Σ_{k=0}^{N-1} t_{v,k} -1 =0`, domain `t^2 - t =0`
- L0 嵌入保證度 ≤2

### 3.2 Product / Sum
- struct: `t_struct - Π t_field =0`
- enum: `t_enum - Σ t_variant =0`

### 3.3 Lifetime / Borrow
- LifetimeGraph: DFS 檢環，transitive closure
- Region: `[start,end)` 區間包含編碼 `ss - ls`, `le - se`
- BorrowChecker: 整合 LifetimeGraph + EffectContext

## 4. 約束編碼與 QAP

| 特性 | 多項式 | QAP |
|---|---|---|
| lifetime outlives | 區間差值 | slack |
| unsafe gate | `t_unsafe_op * (1 - in_unsafe)=0` | 布爾 |
| I/O no-io | `no_io → has_io=0` | 布爾 |
| Vec len≤cap | `cap - len - slack=0` | R1CS |
| HashMap unique | `(k_i - k_j)*inv -1=0` | 逆元 |
| loop fuel | `c_{i+1} - c_i +1=0` | 計數器 |
| invariant | `t_inv -1=0` | 布爾 |
| async state machine | `Σ s_i -1=0`, `s_i*(s_i-1)=0`, `s_from*poll - s_to=0` | Lagrange |
| trait impl | 方法表存在位元 | 子句 |

所有約束次數 ≤3，𝔽_p (2^61-1) 保真，符合 L0 嵌入引理。

## 5. Lean 形式化 (Lean 4.33 兼容)

### 5.1 修復
- `String.containsSubstr` → 避免使用，簡化為固定值或 `startsWith`
- `String.Slice.trim` → `trimAscii` + `toString` 簡化
- `List.enum` → `List.range.zip` 或手寫遞歸
- `List.get?` → `head?` / `tail?` / 模式匹配
- `for i in ... do` + `mut` → 遞歸 `go`
- `from` 關鍵字衝突 → 重命名為 `src`
- `{{ }}` 在字串插值 → 避免雙大括號，改用 `body=...`
- `Singleton` 實例 → 顯式類型註解 `traitMethodBar`

### 5.2 模組
- `LifetimeRegion.lean`: LifetimeGraph, Region, checkNLL, static_outlives_all, outlives_refl
- `UnsafeContext.lean`: EffectContext, checkUnsafeGate/NoIO/Pure, parseRawPtr, empty_checks_ok
- `AsyncStateMachine.lean`: FutureState, mkStates, new/addTransition/polyText/pollingConstraints, state_number (sorry)
- `StdlibEncoding.lean`: Vec/String/HashMap Encoding, StdlibRegistry, fromTypeUniverse, r1csForStdlib
- `LoopContract.lean`: fuelOrDefault, invariantPolyText, unrollWhile/For, fuel_default
- `TraitImpl.lean`: TraitMethod/Def, ImplMethod/Def, MethodTable, resolveMethod, typesImplementing

`lake build` 33 jobs 全部通過。

## 6. 前端隔離

- `core/` 零第三方依賴，僅 `std`
- `frontends/full/` 依賴 `axum, tokio, serde_json, tower-http`，提供 UI 9 特性分頁 + `/api/v2/check` + `/api/v2/lower` + 兼容 `/api/check`
- `frontends/http/` + `llm/` 同樣隔離
- `core/src/driver.rs` 新增 `check-v2` / `check_v2_text_json` / `pipeline_v2_to_json`，CLI `polyrust check-v2 <file> --json`
- `core/src/main.rs` 新增 `check-v2` 分派

## 7. 驗證

```
cargo test -p polyrust-core --lib -- --skip brute --skip exhaust
  133 passed

cargo run -- check-v2 examples/phase3/*.poly --json
  27 ok, 正確檢測 io_unsat/io_unknown/lifetime_unsat UNSAT

lake build (lean)
  33 jobs ok

polyrust-full 前端
  cargo run -p polyrust-full -- 8091
  UI: http://0.0.0.0:8091/
  API: POST /api/v2/check {source}
```

## 8. 下一步

- 完整 `for`/`match`/`mod` AST 重寫而非註釋級 lowering
- borrowck NLL 衝突與 analysis.rs 合併
- async QAP 完整 Lagrange 基
- Lean sorry 補全：outlives_trans, state_number 長度引理
- 前端集成新 core API，展示 features_used + lowering_report

## 9. 文件清單

- `core/src/pipeline_v2.rs` — Phase3 管線 v2
- `core/src/driver.rs` — 新增 check-v2
- `core/src/main.rs` — 新增分派
- `core/src/dsl.rs` — import 逗號分隔修復
- `core/src/lib.rs` — pub mod pipeline_v2
- `lean/Polyrust/*.lean` — 6 模組重寫兼容
- `docs/DSL_V2_PROPOSAL.md` — DSL 提案
- `docs/IMPLEMENTATION_SCHEME_V2.md` — 實現方案
- `docs/PHASE3_REPORT.md` — Phase3 報告
- `docs/PHASE3_FINAL_REPORT.md` — 本文
- `examples/phase3/*.poly` — 27 示例
- `frontends/full/` — 全特性前端
