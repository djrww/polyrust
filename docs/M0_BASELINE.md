# M0 誠實基線 — 2026-09-16

> 範圍：檢視樹 `origin/v0.1.6` @ `a4377aa`。  
> 本里程碑**不新增語言特性**。只對齊身分、寫死測試閘門、公布編碼完備度。

## 0. 遠端事實（已核對 `git ls-remote`）

| 項目 | 事實 |
|---|---|
| 使用者指定的樹 | `https://github.com/djrww/polyrust/tree/v0.1.6` |
| `v0.1.6` 是什麼 | **branch**，不是 annotated tag |
| 遠端 heads | `main`, `v0.1.0`…`v0.1.6` |
| 遠端 tags | **只有** `refs/tags/v0.1.0`（與 `heads/v0.1.0` 同 SHA `b5167d4`） |
| 本樹 SHA | `a4377aaf7140a2dc8d9da57c46959b9ae7205815` |
| 本樹 `workspace.package.version`（對齊前） | `0.2.0` |
| 本樹 `workspace.package.version`（M0 對齊後） | `0.1.6` |
| README「驗證狀態」日期 | 2026-09-10，數字滯後 |
| Phase3 報告日期 | 2026-09-15 |

**身分規則（M0 起）**

- crate / workspace 版本 = **0.1.6**，與被檢視的 branch 同名。
- 「朝 0.2 的表面擴張」標為 **experimental / 非發行級**，不得再把 workspace 寫成 0.2.0，直到 M1 IR 合併且 v1 demo 回歸通過後另開 `v0.2.0` tag。
- 對外只承認一個已發行 tag：`v0.1.0`。`v0.1.6` 在打 tag 之前一律稱 **snapshot**。

## 1. 兩條管線，兩個「判定」

| 管線 | 入口 | 判定含義 | rustc round-trip |
|---|---|---|---|
| **Kernel v1** | `pipeline::run_pipeline` / `polyrust demo\|check\|obligations` | CDCL(T) ↔ Buchberger；`1 ∈ G` ⟺ UNSAT | demo A/D 有；義務 T9 有 |
| **Surface v2** | `pipeline_v2` / `check-v2` | `is_unsat = errors ≠ ∅ ∨ lifetime_has_cycle`（`pipeline_v2.rs` S10） | **無**與 v1 同等的 codegen+rustc 閘 |

v2 自己的註解（S11）：product 約束與 one-hot 衝突，Groebner **只統計不判案**；QAP 只跑 field + one-hot 演示，不含 product/sum。

因此：Phase3 的 SAT/UNSAT **不是**命題 P 的代數判定。文件若寫「9 特性全鏈路通過」必須同時寫清判定器是哪一個。

## 2. 數字對賬（本 session 靜態盤點，未重跑 obligations）

| 來源 | 測試 | Lean | 備註 |
|---|---|---|---|
| README / EVIDENCE.md（2026-09-10） | 17/17 | 14 模組 / 273 定理 | **過期** |
| PHASE3_REPORT | 128 passed（skip brute/exhaust） | Phase3 模組表 | 過程數字 |
| PHASE3_FINAL_REPORT | 133 passed（skip brute/exhaust）；pipeline_v2 5 passed | 6 模組重寫；lake 33 jobs | 過程數字 |
| LEAN.md（樹內最新） | — | 34 模組 + 根；652 定理；AuditAll 3074 宣告 | 以 LEAN.md 為準 |
| 本 session `#[test]` 計數（core `*.rs`） | **166** 個屬性 | `lean/Polyrust/*.lean` **34** 個檔 | 屬性數 ≠ 執行通過數 |

本環境 rustc = 1.75.0，低於文件寫的 1.98.1；**M0 不把「本機 cargo test 綠」寫進基線**。以 CI 硬閘門與原始碼計數為準。

## 3. 交付物

| 檔 | 作用 |
|---|---|
| `docs/M0_BASELINE.md` | 本文件 |
| `docs/TEST_MATRIX.md` | 寫死的測試閘門 |
| `docs/ENCODING_COMPLETENESS.md` | 特性 × 五欄完備度 |
| `README.md` 頂部 M0 橫幅 | 對外不再引用 17/17 與「已發行 0.2」 |
| `Cargo.toml` version `0.1.6` | 與 branch 同名 |

## 4. M0 完成定義

- [x] 遠端 tag/branch 身分核對
- [x] version 不再寫 0.2.0
- [x] README 不再把 2026-09-10 的 17/17 當現況
- [x] 測試矩陣獨立成文
- [x] 完備度表獨立成文，v1 / v2 分欄
- [ ] 於 `djrww/polyrust` 打 annotated tag `v0.1.6`（需維護者推送；本環境無寫入權）
- [ ] 在 CI 跑綠後把 `docs/evidence/` 從 17/17 重產出（屬 M0 收尾，需 1.85+ toolchain）

## 5. 下一里程碑入口

M1 只做一件事：Surface 語法 **一次 lower** 進 Kernel `System`，v2 不得再擁有獨立「判定器」。在那之前，`check-v2` 的 JSON 必須帶 `"engine": "surface-errors"`，禁止寫成與 `check` 相同的 `verdict` 語意。
