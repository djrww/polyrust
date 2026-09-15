# 編碼完備度表（M0）

欄位定義（必須五欄齊才叫「與命題 P 同級」）：

| 欄 | 含義 | 過關標準 |
|---|---|---|
| 語法 | 解析進 AST | 有節點／item，不是純字串偵測 |
| lower | 降到 Kernel IR 或明確的 v2 IR | 可指出函式 |
| 約束 | 多項式或 CDCL 子句進入 solver | 次數 ≤3、會影響 **代數** 判定 |
| Lean | 一般性定理（非只註解） | 具名模組，AuditAll 涵蓋 |
| rustc RT | 見證 → 生成碼 → 再解析 → `rustc` | 與 demo A 同級 |

判定器圖例：

- **K** = Kernel v1（`1 ∈ G` / SAT 模型）
- **E** = Surface 錯誤列表（`pipeline_v2.rs:669`）
- **P** = 預覽／示意編碼（UI notes 或計數，不判案）

## 表 A — Kernel v1（Mini-Rust 子集）

| 特性 | 語法 | lower | 約束 | Lean | rustc RT | 判定 | 評級 |
|---|:-:|:-:|:-:|:-:|:-:|:-:|---|
| 7 型宇宙 i32/bool/()/ref | ✓ | n/a | ✓ one-hot + 規則 | T9 / TypeUniverse | ✓ demo A | K | **完備** |
| let / seq / binop / if | ✓ | n/a | ✓ | OpAbstraction | ✓ | K | **完備** |
| `&` / `&mut` / deref / assign | ✓ | n/a | ✓ 借用區間子句 | BorrowOwnership | ✓ demo C | K | **完備** |
| `fn` / call | ✓ | n/a | ✓ | T9 | ✓ | K | **完備** |
| `macro_rules!` 多臂 | ✓ | 衛生展開 | ✓ 臂位元 | MacroExpansion | ✓ demo A/D | K | **完備** |
| 單位／整數／布林字面 | ✓ | n/a | ✓ | MicroInstance | ✓ | K | **完備** |

這六行是目前唯一能對外講「代數形式化」的範圍。

## 表 B — Surface v0.2 / Phase3（九特性家族）

| 特性 | 語法 | lower | 約束進代數判定 | Lean 模組 | rustc RT | 實際判定 | 評級 |
|---|:-:|:-:|:-:|---|:-:|:-:|---|
| struct | ✓ `ast_v2`/`ast_full` | ✓ product 文字 | ✗ Groebner 停用；product 與 one-hot 衝突 | ProductReduction / ProductSum | ✗ | E / P | **語法+示意** |
| enum | ✓ | ✓ sum 文字 | ✗ 同上 | SumReduction | ✗ | E / P | **語法+示意** |
| impl 方法表 | ✓ | ✓ `lower_trait_impl_method_table` | 子句存在，不驅動 K | TraitImpl | ✗ | E | **表驅動** |
| trait bound | ✓ | ✓ 存在量化位元 | 未實現時多為 UNKNOWN/E | TraitImpl | ✗ | E | **表驅動** |
| Vec/String/HashMap | ✓ + `core/std/*.poly` | ✓ `lower_stdlib_usage` | stdlib 約束計數；判定走型別錯誤字串 | StdlibEncoding | ✗ | E | **介面編碼** |
| loop/while/for + @fuel | ✓ | ✓ 有界展開 | fuel 多項式有；invariant 歸納未接 K | LoopContract | ✗ | E | **有界示意** |
| match | ✓ `parse_pat` | ✓ 決策樹→if | match 約束計數 | MatchDecisionTree | ✗ | E | **降糖示意** |
| mod / use | ✓ | ✓ flatten | 非法可見性 → 錯誤字串 | ModuleFlatten | ✗ | E | **解析期** |
| async / await | ✓ | ✓ `async_qap` 狀態機 | async 約束計數；QAP 演示不含真實 Future 語義 | AsyncStateMachine | ✗ | E | **示意** |
| I/O 效應 / @pure / @no-io | ✓ DSL | ✓ effects | 效應位元；UNSAT 來自錯誤列表 | UnsafeContext（效應段） | ✗ | E | **閘門** |
| unsafe / raw ptr | ✓ | ✓ | `(1-in_unsafe)*…` 有生成；判案走 E | UnsafeContext | ✗ | E | **閘門** |
| lifetime / outlives / NLL 雛形 | ✓ | ✓ `LifetimeGraph` | 環 → `lifetime_has_cycle` | LifetimeRegion | ✗ | E | **圖 + 環檢測** |
| 可變類型宇宙 7+i | ✓ | ✓ `universe.rs` | v2 one-hot；未接回 K 判定 | TypeUniverse7PlusI | ✗ | P | **宇宙構造** |
| @requires/@ensures | ✓ `dsl.rs` | 部分 contracts | 未進 K T9 | LoopContract 部分 | ✗ | E | **註解** |
| Pat or / range / closure / try / cast / return/break | parse 測試 + Oracle gap | 部分 | 缺口檢測是 syn Oracle | — | ✗ | S3 | **解析覆蓋 ≠ 證明** |

## 表 C — 基礎設施（非語言特性，但常被算進「完成度」）

| 件 | 狀態 | 可否當產品聲明 |
|---|---|---|
| CDCL | v1 使用 | 可 |
| Buchberger / `𝔽_{2⁶¹−1}` | v1 使用；v2 統計 | 只對 v1 可 |
| QAP + 篡改拒絕 | v1 demo/T8；v2 只驗證 field | 只對 v1 可 |
| Lean 靜態嵌入 | build.rs 可選，無工具鏈降級 | 工程可；不是 UX 賣點 |
| `polyrust serve` std HTTP | 有 | 可作 demo |
| `frontends/http` `llm` `full` | 有第三方 | 實驗前端 |
| NL 護欄 `llm.rs`（core 內 ~1800 行） | 有單元測試 | **邊界違規**：核裡不該放 prompt 編排 |

## 讀表規則

1. 只有表 A 的「完備」可以寫進發行說明。
2. 表 B 任何「✓ 語法」不得單獨宣傳為「已驗證該 Rust 特性」。
3. Lean 模組存在 ≠ 該特性已接到 Kernel 判定。Phase3 大量 Lean 是編碼語言的模型，與 `pipeline_v2` 的 E 判定並行。
4. `examples/phase3/*_{sat,unsat,unknown}.poly` 驗證的是 **E 判定與特徵偵測**，不是 `1 ∈ G`。

## 缺口優先序（供 M1/M3，M0 不開工）

1. struct/enum 的 product/sum 與 one-hot 改寫到可進 Buchberger（否則 ADT 永遠停在 E）。
2. v2 錯誤列表改為「生成 K 子句」，刪 S10 的獨立 `is_unsat`。
3. 每一個要升級的特性補三件套：SAT `.poly`、UNSAT `.poly`、rustc 生成樣本——且 UNSAT 必須是 `1 ∈ G`。
