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

- `enum_color` [matrix/struct_enum] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `struct_default` [matrix/struct_enum] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `ref_deref` [matrix/borrowck] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `async_simple` [matrix/async_io] → **ok_with_missing**  
- `async_await` [matrix/async_io] → **ok_with_missing**  
- `async_spawn` [matrix/async_io] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `future_combinator` [matrix/async_io] → **ok_with_missing**  
- `async_loop` [matrix/async_io] → **ok_with_missing**  
- `examples/bad` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `examples/demo_7plusI` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `examples/demo_phase2` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `examples/full_syntax` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `examples/template` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/async_sat` [examples] → **ok_with_missing**  
- `phase3/async_unknown` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/async_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/break_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/cast_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/cast_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/closure_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/io_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/io_unknown` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/io_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/match_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/match_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/mod_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/pat_or_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/pat_or_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/pat_range_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/return_break_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/return_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/struct_unknown` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/struct_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/try_cast_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/try_cast_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/try_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/try_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/vec_sat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/vec_unknown` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
- `phase3/vec_unsat` [examples] → **rustc_reject**  warning: `cargo miri setup` failed for target `x86_64-unknown-linux-gnu`; falling back to rustc's default sysroot: error: 'cargo-miri' is not installed for the 
