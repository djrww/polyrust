# M1 — 單一判定器適配

日期：2026-09-16  
前置：`docs/M0_BASELINE.md`

## 決策

不重寫 CDCL / Groebner / QAP。不刪 `ast_v2`。

規則：

1. **Core IR** 仍是 `minirust::ast::Program`（v1 Parser）。
2. `ast_v2` / `ast_full` 只當 surface。
3. `pipeline_v2` 在 S10 之後呼叫 `engine::try_kernel`。
4. v1 Parser 成功 ⇒ `engine = "kernel"`，`is_unsat` **覆寫為** Kernel 的 `1 ∈ G` 判定。
5. 否則 `engine = "surface-errors"`，JSON 必須帶 `"engine"` 欄，不得假裝命題 P。

## 新增

| 檔 | 作用 |
|---|---|
| `core/src/engine.rs` | `Engine`、`try_kernel`、`is_kernel_subset` |
| `pipeline_v2::PipelineV2Result.engine` | `"kernel"` / `"surface-errors"` |
| `check-v2` JSON | `engine`, `kernel_unsat` |

## 回歸契約

- demo A–D、`obligations`、`pipeline::run_pipeline` **零行為變更**（M1 只從 v2 呼叫 v1，不改 v1）。
- 既有 `pipeline_v2` 單元測試用 struct / lifetime / unsafe 來源，v1 Parser 失敗，判定仍走錯誤列表。

## 未完成（M1.1 / M3）

- 把 struct product/sum 真正 lower 成 v1 `System`（現在 struct 仍是 surface-errors）。
- 刪 S10 獨立判定——要等表 B 特性都能進 Kernel。
- `llm.rs` 搬出 core。
