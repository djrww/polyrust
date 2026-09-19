# C2 Lowering 記檔（2026-09-20）

## 交付
- `core/src/llbc_body.rs`：Structured body 類型化 parser（白名單硬錯）。法證實錄三個隱藏形狀：
  1. Op token 兩形態：plain `"Gt"` vs object variant `{"Rem":"UB"}`（payload=UB 檢查位，C4 再用）
  2. Place Projection Field（checked-op 嘅 `.{0,1}` 二元組，overflow flag=.1）
  3. Const 去重表：`"Value":[id,…]` 定點 ↔ `"Deduplicated":id` 引用（while_loop/fact 必須）
  4. 型別真源 = signature（inline literal）；locals 表 Deduplicated 內建型 → Opaque（唔鎖定）
- `core/src/llbc_lower.rs`：SSA path lowering（continuation-passing，免 φ 合流）+ GB 判定。
  語義表見檔頂註：+−× 精確；cmp/divrem/bitop 抽象 bool/自由變數；Assert 放寬（marker）；
  Call inline fuel=2；Loop unroll-1——全部 abstract/bounded marker 強制呈報。
- `core/src/differential_v4.rs`：v3↔v4 basic 差分 + 時延表。

## 驗收截圖（差分測試實跑）
| case | v3 | v4 | paths | vars | polys | 耗時 | markers |
|---|---|---|---|---|---|---|---|
| sqr/add | SAT | SAT ✓ | 1 | 6-7 | 4 | <1ms | assert-abstracted |
| max/abs | SAT | SAT ✓ | 2 | 9-13 | 7-10 | <1ms | cmp-abstraction |
| is_even | SAT | SAT ✓ | 1 | 14 | 11 | 1ms | +divrem/bitop |
| fact/pow/gcd | SAT | SAT ✓ | 4 | 55-70 | 37-54 | 75-195ms | +call-inline-fuel |
| fib | SAT | SAT ✓ | 26 | 619 | 121 | 48s(debug) | +call-inline-fuel |
| sum_range | SAT | SAT ✓ | 2 | 18 | 8 | 1ms | loop-unroll-1 |

## 有界標記審計線（差分測試第二斷言）
recursion 案例必帶 `call-inline-fuel`、loop 必帶 `loop-unroll-1`——唔准悄悄當精確 SAT。

## perf backlog（C3 開頭做）
fib 26 paths × 619 vars → debug Classic GB 48s。方向：SAT 短路已有（首條 feasible 即 break），
加 triangular fast-path（SSA 定義鏈全對角 → 免 GB 即判 SAT），預期 <1s。
