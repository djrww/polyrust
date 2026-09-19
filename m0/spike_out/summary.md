# C0 Charon Spike — 執行結果

## 有效轉換率：**117/136 = 86.0%**（已排除設計 UNSAT 16 例）

總案例 152；出 LLBC **117**（77.0%）；ok 112 ｜ ok_with_missing 5 ｜ rustc_reject 35 ｜ charon_err 0 ｜ timeout 0

## 按類別

| 類別 | ok | ok+missing | rustc_reject | charon_err | timeout |
|---|---|---|---|---|---|
| examples | 20 | 1 | 31 | 0 | 0 |
| matrix/async_io | 5 | 4 | 1 | 0 | 0 |
| matrix/basic | 10 | 0 | 0 | 0 | 0 |
| matrix/borrowck | 14 | 0 | 1 | 0 | 0 |
| matrix/commercial | 10 | 0 | 0 | 0 | 0 |
| matrix/loop_match | 15 | 0 | 0 | 0 | 0 |
| matrix/struct_enum | 13 | 0 | 2 | 0 | 0 |
| matrix/unsafe | 10 | 0 | 0 | 0 | 0 |
| matrix/vec_string | 15 | 0 | 0 | 0 | 0 |

## 明細（非 ok）

- `enum_color` [matrix/struct_enum] → **rustc_reject**  error[E0369]: binary operation `==` cannot be applied to type `Color`
- `struct_default` [matrix/struct_enum] → **rustc_reject**  error[E0599]: no associated function or constant named `default` found for struct `Config` in the current scope
- `ref_deref` [matrix/borrowck] → **rustc_reject**  error[E0614]: type `{integer}` cannot be dereferenced
- `async_simple` [matrix/async_io] → **ok_with_missing**  
- `async_await` [matrix/async_io] → **ok_with_missing**  
- `async_spawn` [matrix/async_io] → **rustc_reject**  error[E0433]: cannot find module or crate `tokio` in this scope
- `future_combinator` [matrix/async_io] → **ok_with_missing**  
- `async_loop` [matrix/async_io] → **ok_with_missing**  
- `examples/bad` [examples] → **rustc_reject**  error[E0277]: cannot add `bool` to `{integer}`
- `examples/demo_7plusI` [examples] → **rustc_reject**  error[E0425]: cannot find type `HashMap` in this scope
- `examples/demo_phase2` [examples] → **rustc_reject**  error[E0425]: cannot find type `HashMap` in this scope
- `examples/full_syntax` [examples] → **rustc_reject**  error[E0658]: `impl Trait` in type aliases is unstable
- `examples/template` [examples] → **rustc_reject**  error: cannot find macro `sqr` in this scope
- `phase3/async_sat` [examples] → **ok_with_missing**  
- `phase3/async_unknown` [examples] → **rustc_reject**  error[E0728]: `await` is only allowed inside `async` functions and blocks
- `phase3/async_unsat` [examples] → **rustc_reject**  error[E0728]: `await` is only allowed inside `async` functions and blocks
- `phase3/break_unsat` [examples] → **rustc_reject**  error[E0268]: `break` outside of a loop or labeled block
- `phase3/cast_sat` [examples] → **rustc_reject**  error[E0596]: cannot borrow `y` as mutable, as it is not declared as mutable
- `phase3/cast_unsat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/closure_unsat` [examples] → **rustc_reject**  error[E0277]: cannot add `{integer}` to `bool`
- `phase3/io_sat` [examples] → **rustc_reject**  error[E0423]: cannot find function `println` in this scope
- `phase3/io_unknown` [examples] → **rustc_reject**  error[E0423]: cannot find function `println` in this scope
- `phase3/io_unsat` [examples] → **rustc_reject**  error[E0423]: cannot find function `println` in this scope
- `phase3/match_sat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/match_unsat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/mod_unsat` [examples] → **rustc_reject**  error[E0603]: struct `Point` is private
- `phase3/pat_or_sat` [examples] → **rustc_reject**  error[E0408]: variable `x` is not bound in all patterns
- `phase3/pat_or_unsat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/pat_range_unsat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/return_break_unsat` [examples] → **rustc_reject**  error[E0268]: `break` outside of a loop or labeled block
- `phase3/return_unsat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/struct_unknown` [examples] → **rustc_reject**  error[E0599]: no method named `fmt` found for struct `Point` in the current scope
- `phase3/struct_unsat` [examples] → **rustc_reject**  error[E0308]: mismatched types
- `phase3/try_cast_sat` [examples] → **rustc_reject**  error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `std::ops::FromResidual`)
- `phase3/try_cast_unsat` [examples] → **rustc_reject**  error[E0277]: the `?` operator can only be applied to values that implement `std::ops::Try`
- `phase3/try_sat` [examples] → **rustc_reject**  error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `std::ops::FromResidual`)
- `phase3/try_unsat` [examples] → **rustc_reject**  error[E0277]: the `?` operator can only be applied to values that implement `std::ops::Try`
- `phase3/vec_sat` [examples] → **rustc_reject**  error[E0425]: cannot find type `HashMap` in this scope
- `phase3/vec_unknown` [examples] → **rustc_reject**  error[E0425]: cannot find type `HashMap` in this scope
- `phase3/vec_unsat` [examples] → **rustc_reject**  error[E0425]: cannot find function, tuple struct or tuple variant `Vec_new` in this scope
