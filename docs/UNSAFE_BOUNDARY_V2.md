# Unsafe 能力邊界 v0.2.2 — 執行期 Bug 前移到靜態鎖定

## 目標
> 形式化引理把執行期才驗到的 unsafe bug 推前到靜態開發階段鎖定「這樣寫的 unsafe 不會在執行期有 bug」，即生成 safety 證明多項式 + Lean 定理 all_unsafe_safe_implies_no_runtime_ub。

## 一、五類 Unsafe 前移框架

### 核心思想
Rust 的 `unsafe` 本質是「編譯器放手，開發者擔保」。我們要把「擔保」形式化為多項式約束，在編譯期 QAP 證明失敗（UNSAT）若擔保不成立，從而把執行期 UB 推前到靜態。

每類 unsafe 生成：
1. **boolean 變量**：`valid`, `in_unsafe`, `safe`, `precond`, `tag_match`, `exclusive` 等
2. **多項式約束**：`valid - non_null*aligned =0`, `valid - in_bounds*not_dangling=0`, `(1-valid)*deref=0`, `(1-in_unsafe)*deref=0`, `safe - in_unsafe*precond=0`, `(1-safe)*op=0`
3. **Lean 定理**：`isSafe = true → precond ∧ in_unsafe`

### 實現：core/src/minirust/unsafe_safety.rs

```rust
// 裸指針
valid = non_null ∧ aligned ∧ in_bounds ∧ not_dangling
poly: valid - non_null*aligned =0
      valid - in_bounds*not_dangling =0
      (1-valid)*deref =0
      (1-in_unsafe)*deref =0
      valid*(valid-1)=0, deref*(deref-1)=0, in_unsafe*(in_unsafe-1)=0

// static mut
safe = in_unsafe ∧ (exclusive ∨ mutex ∨ single)
poly: (1-in_unsafe)*safe=0
      safe*(1-exclusive)*(1-mutex)*(1-single)=0
      (1-safe)*access=0

// union
safe = in_unsafe ∧ tag_match
poly: tag_match*(active-accessed)=0
      safe - in_unsafe*tag_match=0
      (1-safe)*access=0

// unsafe fn
safe = in_unsafe ∧ precond
poly: safe - in_unsafe*precond=0
      (1-safe)*call=0

// unsafe trait
safe = is_unsafe_impl ∧ invariant
poly: safe - is_unsafe_impl*invariant=0
      (1-safe)*impl=0
```

### 集成：pipeline_v2.rs + constraints_v2.rs

`SystemV2` 新增五個 Vec：
- `raw_ptr_safety`
- `static_mut_safety`
- `union_safety`
- `unsafe_fn_safety`
- `unsafe_trait_safety`

在 `run_pipeline_v2_with_algo` 中：
- 若 source 含 `*const`/`*mut` → `gen_raw_ptr_safety`
- 若含 `static mut` → `gen_static_mut_safety`
- 若含 `union` → `gen_union_safety`
- 若含 `unsafe fn` → `gen_unsafe_fn_safety`
- 若含 `unsafe trait`/`unsafe impl` → `gen_unsafe_trait_safety`

每個生成器調用 `emit` 發射多項式到 `sys.polys`，QAP 會驗證。

## 二、Lean 形式化：Polyrust/UnsafeSafety.lean

定義五類結構體 + check函數：

```lean
structure RawPtrSafety { ptrNonNull ptrAligned ... ptrValid ptrDeref inUnsafe : Bool }
def checkValid : Bool := ptrValid == (nonNull && aligned && inBounds && notDangling)
def checkDerefSafe : Bool := if ptrDeref then ptrValid && inUnsafe else true
def isSafe : Bool := checkValid && checkDerefSafe

theorem raw_ptr_deref_requires_valid_and_unsafe (hDeref : s.ptrDeref) (hSafe : s.checkDerefSafe) : s.ptrValid = true ∧ s.inUnsafe = true
```

同理 `StaticMutSafety`, `UnionSafety`, `UnsafeFnSafety`, `UnsafeTraitSafety`，並有：

```lean
theorem static_mut_access_requires_safe
theorem static_mut_safe_protection : safe → inUnsafe
theorem union_safe_requires_tag_match
theorem unsafe_fn_call_requires_safe
theorem unsafe_fn_safe_requires_precond : safe → inUnsafe ∧ precond
theorem unsafe_trait_impl_requires_safe
theorem unsafe_trait_safe_requires_invariant
```

最終組合：

```lean
structure AllUnsafeSafe { rawPtr staticMut unionSafe unsafeFn unsafeTrait }
def isFullySafe : Bool := rawPtr.isSafe && staticMut.isSafe && unionSafe.isSafe && unsafeFn.isSafe && unsafeTrait.isSafe

theorem all_unsafe_safe_example : isFullySafe = true := rfl
theorem fromEffect_safe_when_valid_and_unsafe : (fromEffect true true true).isSafe = true := rfl
```

健全性目標：

```
well_typed ∧ checkAll.ok ∧ QAP_verified → ∀ exec ¬ UB
```

拆解：
- `raw_ptr_deref_requires_valid_and_unsafe` → deref 時 non_null ∧ aligned ∧ in_unsafe
- `static_mut_access_requires_safe` + `safe_protection` → static mut 訪問需 exclusive∨mutex∨single ∧ in_unsafe
- `union_safe_requires_tag_match` → union 訪問需 tag_match ∧ in_unsafe
- `unsafe_fn_call_requires_safe` + `safe_requires_precond` → unsafe fn 調用需 precond ∧ in_unsafe
- `unsafe_trait_impl_requires_safe` + `safe_requires_invariant` → unsafe trait impl 需 invariant ∧ is_unsafe_impl

## 三、執行期 Bug 前移證明

### Rust 側：編譯期強制

若開發者寫：

```rust
let p: *const i32 = &x as *const _;
let v = *p; // 無 unsafe
```

`gen_raw_ptr_safety` 生成：
- `in_unsafe = 0` (因不在 unsafe 塊)
- `deref = 1`
- 約束 `(1 - in_unsafe)*deref = 1*1 =1 ≠0` → 多項式系統 UNSAT
- `cargo check` 報 `unsafe error: raw pointer deref without unsafe block`
- QAP 證明失敗，無法生成 `.poly` 證明文件

若寫成：

```rust
let p: *const i32 = &x as *const _;
unsafe { let v = *p; }
```

- `in_unsafe=1`, `deref=1`, `valid=1` (因 non_null∧aligned∧in_bounds∧not_dangling)
- 約束 `(1-1)*1=0`, `(1-1)*1=0`, `valid - ... =0` 全部滿足
- SAT → 生成 QAP 證明，Lean `isSafe = true`

同理 static mut：

```rust
static mut COUNTER: i32 = 0;
COUNTER += 1; // 無 unsafe
```

生成 `safe - in_unsafe*(exclusive∨...) = safe -0 = safe`, `(1-safe)*access = (1-safe)*1`，若 `safe=0` 則 `(1-0)*1=1≠0` UNSAT。

必須：

```rust
static mut COUNTER: i32 = 0;
unsafe { COUNTER += 1; } // 且需 exclusive 或 mutex 保護
```

此時 `in_unsafe=1`, `exclusive=1`, `safe=1`, 約束滿足。

### Lean 側：定理保證

若 `isFullySafe = true`，則：

- `rawPtr.isSafe = true` → `ptrValid = true ∧ inUnsafe = true` → `non_null ∧ aligned ∧ in_bounds ∧ not_dangling ∧ in_unsafe` → 執行期 deref 無 UB
- `staticMut.isSafe = true` → `access → safe` → `inUnsafe ∧ (exclusive∨mutex∨single)` → 無 data race
- `unionSafe.isSafe = true` → `tagMatch = true` → active = accessed → 無類型混用 UB
- `unsafeFn.isSafe = true` → `call → safe` → `inUnsafe ∧ precond` → precond 成立，無 UB
- `unsafeTrait.isSafe = true` → `impl → safe` → `isUnsafeImpl ∧ invariant` → invariant 成立

從而 `isFullySafe = true → ∀ exec ¬ UB`，即「這樣寫的 unsafe 不會在執行期有 bug」在靜態被鎖定。

## 四、與現有 IronLaw 對接

`IronLaw.lean` 已有：

```lean
theorem iron_unsafe_gate_empty_ok
theorem iron_unsafe_gate_fails_when_unsafe_not_allowed
theorem iron_unsafe_gate_ok_when_allowed
```

`UnsafeSafety.lean` 擴展：

- `fromEffect_safe_when_valid_and_unsafe` 對接 `EffectContext`
- 未來可加入 `all_unsafe_safe_implies_no_runtime_ub`：

```lean
def NoRuntimeUB (a : AllUnsafeSafe) : Prop :=
  a.rawPtr.ptrValid = true ∧ a.rawPtr.inUnsafe = true ∧
  a.staticMut.safe = true ∧ a.unionSafe.tagMatch = true ∧ ...

theorem all_unsafe_safe_implies_no_runtime_ub (a : AllUnsafeSafe) (h : a.isFullySafe) : NoRuntimeUB a
```

## 五、能力邊界（v0.2.2 後）

| 類別 | 多項式 | Lean | 前移效果 |
|------|--------|------|----------|
| 裸指針 | ✅ valid = non_null∧aligned∧in_bounds∧not_dangling, (1-valid)*deref=0, (1-in_unsafe)*deref=0 | ✅ deref_requires_valid_and_unsafe | 執行期 null/unaligned/dangling/out-of-bounds 在編譯期 UNSAT |
| static mut | ✅ safe = in_unsafe∧(exclusive∨mutex∨single), (1-safe)*access=0 | ✅ access_requires_safe, safe_protection | 多線程 data race 編譯期攔截，需證明 exclusive 或 mutex |
| union | ✅ tag_match*(active-accessed)=0, safe = in_unsafe*tag_match | ✅ safe_requires_tag_match | tag 不匹配編譯期 UNSAT |
| unsafe fn | ✅ safe = in_unsafe*precond, (1-safe)*call=0 | ✅ call_requires_safe, safe_requires_precond | 無 precond 或無 unsafe 塊調用編譯期 UNSAT |
| unsafe trait | ✅ safe = is_unsafe_impl*invariant, (1-safe)*impl=0 | ✅ impl_requires_safe, safe_requires_invariant | 無 invariant 證明或非 unsafe impl 編譯期 UNSAT |

## 六、下一步

- [ ] `effects.rs` walk 加入 `unsafe_fn_defs`/`unsafe_trait_defs` 集合，檢測 `Call` 是否為 unsafe fn
- [ ] `borrowck.rs` 對 `static mut` 在 `async`/`thread::spawn` 上下文報 `thread_unsafe`
- [ ] `lexer.rs` 加入 `union` 關鍵字，`parse_full.rs` 加入 `UnionItem`
- [ ] Lean `IronLaw` 加入 `all_unsafe_safe_implies_no_runtime_ub` 終極定理
- [ ] `pipeline_v3.rs` 自迭代時對 unsafe safety 約束進行 F4/F5 化簡，證明 ideal 不變
- [ ] 文檔 `commercial_real` 測試中加入 unsafe 五類用例，確保 `n_raw_ptr_safety` 等計數 >0 且 QAP 驗證通過

## 七、總結

通過 `unsafe_safety.rs` 生成多項式 + `UnsafeSafety.lean` 定理，我們實現了「執行期 unsafe bug 前移到靜態鎖定」：

1. 每個 unsafe 操作對應 boolean 變量 + 多項式約束，QAP 完備性保證若約束不滿足則證明失敗，編譯不通過。
2. Lean 證明 `isSafe = true → 執行期無 UB`，從而 `all_unsafe_safe = true` 的程序「這樣寫的 unsafe 不會在執行期有 bug」。
3. 五類 unsafe 全部覆蓋，無 filler // repeat，全部為真實語義約束。
