# Unsafe 能力邊界 — 現狀、把關邏輯與健全性證明路線

> 問題：現在引擎對於 unsafe 既能力邊界？下列 unsafe 狀況，底層邏輯如何把關？裸指針、unsafe fn、實現 Unsafe Trait、多線程下既全域變量 static mut、union，最後點樣證明健全性

## 一、現狀總覽（v0.2.1）

### 1.1 已實現

| 類別 | 解析 | 效應追蹤 | 多項式約束 | Lean 證明 |
|------|------|----------|------------|-----------|
| 裸指針 `*const T` `*mut T` | `parseRawPtr` / `poly_adv_raw_ptr` tag 75 | `EffectContext.raw_ptr_ops` + `unsafe_usages` | `(1 - in_unsafe) * op =0` | `iron_unsafe_gate_*` |
| `unsafe { }` 塊 | `EKind::Call("unsafe",...)` 進入 `in_unsafe=true` | `walk` 檢測 | 同上 | `IronLaw` |
| `unsafe fn` | `FnSigV2.is_unsafe` / `FnSig.is_unsafe` | 僅標記，未強制 gate | 無獨立約束 | 無 |
| `unsafe trait` | `TraitDefV2.is_unsafe` / `TraitItem.is_unsafe` | 僅標記 | 無 | 無 |
| `static mut` | `static mut` 解析 `mutbl=true` | 無特殊處理 | 無 | 無 |
| `union` | **未解析**（關鍵字未入 lexer） | 無 | 無 | 無 |

### 1.2 把關邏輯（底層）

#### 裸指針

**Rust 側** `core/src/minirust/effects.rs`:

```rust
pub struct EffectContext {
  in_unsafe: bool,
  unsafe_allowed: bool, // 來自 # @unsafe-allowed
  unsafe_usages: Vec<usize>,
  raw_ptr_ops: Vec<usize>,
}

fn walk(e: &E, ctx: &mut EffectContext, in_unsafe_block: bool) {
  EKind::Call(name, args) => {
    if name == "unsafe" { in_unsafe=true; ... }
    if name.starts_with("raw_") || contains("ptr") {
      ctx.raw_ptr_ops.push(id);
      if !ctx.in_unsafe { ctx.unsafe_usages.push(id); }
    }
  }
  EKind::Deref(a) => { // *p 解引用
    walk(a, ctx, in_unsafe_block || in_unsafe);
  }
}

pub fn check_unsafe_gate(&self) -> Result<()> {
  if !unsafe_usages.is_empty() && !unsafe_allowed && !in_unsafe {
    Err("unsafe usage but not allowed")
  }
}
```

**多項式側** `constraints_v2.rs:gen_unsafe_constraint`:

```rust
in_unsafe_var = nvars; // boolean
op_var = nvars+1;
bool_poly: in_unsafe*(in_unsafe-1)=0
poly: (1 - in_unsafe) * op =0  // op 為 unsafe 操作變量
```

語義：`op=1` 必須 `in_unsafe=1`，否則多項式≠0 → UNSAT，QAP 證明失敗。

**Lean 側** `UnsafeContext.lean` + `IronLaw.lean`:

```lean
def checkUnsafeGate : Except :=
  if !unsafeUsages.isEmpty && !unsafeAllowed && !inUnsafe then .error

theorem iron_unsafe_gate_empty_ok
theorem iron_unsafe_gate_fails_when_unsafe_not_allowed
theorem iron_unsafe_gate_ok_when_allowed
theorem iron_unsafe_gate_ok_when_inUnsafe
theorem iron_rawPtr_parse_const / mut
```

**能力邊界**：目前僅能把關 `*p` 解引用需在 `unsafe` 塊或 `# @unsafe-allowed`。未區分 `*const` vs `*mut` 寫權限，未追蹤 `ptr::offset`、`ptr::read/write` 等。

#### unsafe fn

解析：`ast.rs: is_unsafe = first_line.contains("unsafe ")`，`parse_full.rs: parse_bare_fn` 支持 `unsafe fn()`。

把關：**缺口**。目前 `EffectContext` 未檢查 `unsafe fn` 的調用需在 unsafe 上下文。應擴展：

- 定義 `fn` 時若 `is_unsafe=true`，記錄 `unsafe_fn_defs`
- 調用 `unsafe fn` 時，`walk` 檢查是否 `in_unsafe`，否則 push `unsafe_usages`
- 多項式：同 `(1 - in_unsafe)*call_unsafe_fn =0`

#### 實現 Unsafe Trait

解析：`TraitDefV2.is_unsafe`，`ImplItem.is_unsafe`。

把關：**缺口**。Rust 規則：`impl unsafe trait` 本身需 `unsafe` 塊？實際 `unsafe trait` 的實現必須在 `unsafe impl`。目前未檢查。

應：

- `unsafe trait` 定義時標記 `is_unsafe`
- `impl` 時若 `trait_ref` 是 unsafe trait，則要求 `impl is_unsafe=true`，否則 `unsafe_usages`
- 約束：`is_unsafe_trait_impl` boolean → `(1 - in_unsafe_impl)*op =0`，或更精細 `unsafe_impl_allowed`

#### static mut（多線程全局）

解析：`static mut` → `mutbl=true`。

把關：**嚴重缺口**。`static mut` 在 Rust 2024 已 deprecated，訪問必須 `unsafe`，且多線程下 data race 未定義行為。

現狀：僅當 `static mut`，未強制 unsafe。

應：

- 讀寫 `static mut` 視為 `raw_ptr_ops` + `unsafe_usages`
- 多線程：引入 `Send/Sync` 效應位元。`static mut` 若在 `async` 或 `thread::spawn` 上下文，需額外約束 `is_send =0` 或要求 `Sync`。
- 多項式：`t_static_mut_access * (1 - in_unsafe) =0` 且 `t_static_mut_access * is_threaded =0` 除非有 `Mutex` 包裝
- 建議：直接拒絕 `static mut`，強制改 `static` + `Mutex` / `Atomic`，或 `# @unsafe-allowed` + `# @thread-safe: false` 標註

#### union

解析：**未實現**。`lexer.rs` 未包含 `union` 關鍵字，`parse_full.rs` 無 union 分支。

把關：**缺口**。`union` 字段訪問必須 `unsafe`，且所有字段共享內存。

應：

- lexer 加入 `union`
- `FullItem::Union` 類似 `StructItem`，字段 `is_unsafe_access=true`
- 效應：`union` 字段讀寫 → `unsafe_usages`
- 約束：`(1 - in_unsafe)*union_access=0`
- Lean：`inductive UnionField` + `checkUnionAccess`

## 二、健全性證明路線

### 2.1 現有鐵律

`IronLaw.lean` 已證明：

- one-hot 排他 `Σ s_i -1=0` + `s_i*(s_i-1)=0`
- field 多項式 `x² - x =0` (bool)
- 借用衝突互斥
- lifetime 無環 `static` 出超所有、無自環
- **unsafe 邊界**（如上）
- watch 移動保 SAT
- F4/F5 不變量

### 2.2 需要新增的定理

#### 裸指針

```lean
theorem raw_ptr_deref_requires_unsafe :
  ∀ ctx node ty, parseRawPtr ty = some raw → 
  checkRawPtrDeref raw ctx node = .ok ↔ (ctx.inUnsafe || ctx.unsafeAllowed)

theorem unsafe_gate_poly_sound :
  (1 - in_unsafe)*op =0 ∧ op=1 → in_unsafe=1
```

#### unsafe fn

```lean
structure UnsafeFnCall where
  fnName : String
  isUnsafeFn : Bool
  inUnsafe : Bool

def checkUnsafeFnCall (c : UnsafeFnCall) : Except :=
  if c.isUnsafeFn && !c.inUnsafe then .error else .ok

theorem unsafe_fn_call_requires_unsafe_block
```

#### Unsafe Trait

```lean
theorem unsafe_trait_impl_requires_unsafe_impl :
  ∀ trait impl, trait.isUnsafe → impl.traitName = trait.name → impl.isUnsafe = true
```

#### static mut + Send/Sync

```lean
inductive ThreadSafety where
| safe
| unsafeStaticMut

theorem static_mut_requires_unsafe :
  staticMutAccess → inUnsafe

theorem static_mut_not_send :
  staticMutAccess ∧ isThreaded → ¬ isSend
```

#### union

```lean
theorem union_access_requires_unsafe
```

### 2.3 端到端健全性

最終目標：

```
well_typed_program ∧ checkAll = ok ∧ QAP_verified → 
  ∀ execution, ¬ undefined_behavior
```

拆解：

1. **TypeUniverse**：`TypeUniverse7PlusI` 已證明基礎類型 + 擴展類型計數
2. **BorrowOwnership**：`borrow_sat ↔ clean` 雙向
3. **LifetimeRegion**：`Outlives` 無環 + `Region` 包含
4. **UnsafeContext**：上述五類 unsafe 全部 gate
5. **T9EndToEnd**：`typable ↔ root` + `parse/gen round-trip`
6. **QAP**：`qap.rs` 生成 R1CS，`Groebner` 證明 ideal 成員

證明策略：Lean 中定義 `SafeExecution` 謂詞，排除：

- raw ptr deref outside unsafe
- unsafe fn call outside unsafe
- unsafe trait impl without unsafe
- static mut data race (兩個線程同時寫)
- union 字段混用

然後證明：`EffectContext.checkAll.isOk → SafeExecution`。

## 三、能力邊界（誠實版）

| 類別 | v0.2.1 能 | 不能 | 風險 |
|------|-----------|------|------|
| 裸指針 | 檢測 `*p` 需 unsafe | 未區分 `offset`/`read`/`write` 權限，未追蹤 `ptr::null` | 中：可能漏 `ptr::add` |
| unsafe fn | 解析標記 | 調用未強制 unsafe | 高：可繞過 |
| Unsafe Trait | 解析標記 | impl 未強制 unsafe | 高 |
| static mut | 解析 | 未強制 unsafe，未檢測多線程 | 極高：data race |
| union | 未解析 | 全部缺 | 中 |

**建議**：

1. 立即在 `effects.rs` 加入 `unsafe fn` 調用檢查（類似 raw ptr）
2. `static mut` 直接在 `borrowck.rs` 報錯，要求改 `Mutex`，除非 `# @unsafe-allowed` + 明確 `# @thread-unsafe`
3. `union` 加入 lexer + 解析，標記為需 unsafe
4. Lean 同步加入上述鐵律，確保 `lake build` 全綠

## 四、下一步實現（代碼草案）

```rust
// effects.rs 新增
pub unsafe_fn_defs: HashSet<String>,
pub unsafe_trait_defs: HashSet<String>,

fn walk(Call(name, args)) {
  if unsafe_fn_defs.contains(name) && !in_unsafe {
    unsafe_usages.push(id);
  }
  if name == "static_mut_read" || name == "static_mut_write" {
    unsafe_usages.push(id);
    raw_ptr_ops.push(id);
    // 多線程檢測：若在 async/thread 上下文
    if is_threaded { thread_unsafe_usages.push(id); }
  }
  if name.starts_with("union_access") && !in_unsafe {
    unsafe_usages.push(id);
  }
}
```

Lean：

```lean
theorem static_mut_thread_unsafe :
  staticMutAccess ∧ isThreaded → checkAll = .error
```

這樣可證明：若程序通過 `checkAll`，則無上述五類 UB。

## 五、總結

目前引擎對 unsafe 的把關僅 **裸指針解引用** 有完整多項式 + Lean 鐵律，其餘四類為標記級別，存在繞過。健全性證明需擴展 `EffectContext` + `IronLaw` 到五類，並用 `(1 - in_unsafe)*op=0` 統一建模，最終在 Lean 中證明 `checkAll.ok → SafeExecution`。

建議優先級：`static mut` (極高風險) > `unsafe fn` (高) > `Unsafe Trait` (高) > `union` (中) > 裸指針細化 (中)。
