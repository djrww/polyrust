# C0 Spike 執行報告（Charon 可行性摸底）

> 所屬：[CHARON_POLYIR_BLUEPRINT.md](CHARON_POLYIR_BLUEPRINT.md) 里程碑 C0
> 執行日期：2026-09-19 ｜ 狀態：🟡 **基建完備，執行受環境限**（重跑 = 一條命令）

## 驗收口徑與現況

| 驗收項 | 狀態 |
|---|---|
| charon.pin（commit + toolchain + 組件）入倉 | ✅ `charon.pin`：commit `ca501af…`（2026-09-19 主線）、`nightly-2026-09-17`、組件 rust-src/rustc-dev/llvm-tools |
| 全量 spike harness（100 matrix + 12 examples + 12 phase3，脫水、分類、聚合、fixture） | ✅ `scripts/c0_spike.py`（已在本倉，可對任意 charon 二進制執行） |
| 環境一鍵腳本（RAM 檢查→clone+pin→建構→spike→判定） | ✅ `scripts/c0_env_check.sh` |
| ≥70/100 案例出 LLBC + missing-decl 報告 | ⏳ **環境受限未跑出**：沙箱 2GB RAM 不足構建 Charon（下方法證） |
| schema snapshot fixtures | ⏳ 隨全量執行自動產出（`charon_fixtures/`） |

## 環境法證（後人唔使重複破產）

1. **Charon 構建記憶體牆**：`charon_lib` 單 crate codegen 峰值 RSS ≈ **1.25GB**（cgu=32）。
   沙箱總 RAM **2GB**、kswapd 常駐 10%+ CPU —— release 預設（opt-level=3）構建必 OOM；
   加 `[profile.release.package.charon] opt-level=1 codegen-units=32` 覆寫後 RSS 穩定於 1.25GB
   但單核 7–11% CPU swap-thrash，單 crate 構建 >22 分鐘仍未完成；
   且依賴圖中存在同等記憶體體量嘅中間 crate（-j1 下跑了 50+ 分鐘仍未越過）。
   **結論：≥4GB RAM（建議 4–6GB）係 Charon 構建嘅硬下限。**
2. **沙箱快照排除**：`target/`、`build/` 目錄名於重啟後消失（含 OOM 觸發嘅環境重建）。
   對策：`CARGO_TARGET_DIR=<不含黑名單名字嘅目錄>`（本流程用 `charon-cargo-out`）。
3. **工具鏈**：nightly-2026-09-17（pin 當日新鮮）下載/安裝於沙箱正常；
   crates.io sparse index + 靜態下載均 200 ✓。Charon 倉 `Cargo.lock` 存在，`--locked` 可重現。
4. **意外但重要嘅架構事實**：Charon 此行 commit 嘅 CLI 已含 **multi-target translate + merge 流程**
   （`translate_one` per target → `CrateData::deserialize_from_file` → `multi_target::merge` →
   `serialize_to_files`）——同 2026-09 Aeneas 報告嘅 multi-target 擴展吻合，
   確認我哋 pin 嘅版本具 C4 之後需要嘅 modularity 基礎。

## 一鍵重跑（任何 ≥4GB 機器 / CI runner）

```bash
bash scripts/c0_env_check.sh
# 產出：$C0_WORKDIR/spike_out/{results.json,summary.md} + charon_fixtures/*.llbc
# 驗收：summary 最尾行 PASS/FAIL（≥70% 出 LLBC）
```

## C0 附帶沉澱（對 v4 語義口徑即刻有用）

spike harness 嘅分類維度本身已經喺設計上驗證咗藍圖嘅一個關鍵預判：

- **`rustc_reject` 類（例如 `ref_deref` 嘅 `***rr` = E0614）**：Charon 路徑下「型別錯誤」
  **唔會有 LLBC 產出**——rustc 喺 typecheck 階段已拒收。即 v4 語義中，
  「rustc 拒編譯」天然 = **UNSAT（附 rustc 原生診斷）**。v2/v3 時代需要自寫 lint 捕捉嘅一類
  缺陷，喺新鏈路由地真值直接處理，仲帶埋官方 E-code 訊息。此結論應寫入
  C1 設計：PolyIR 管線入口必須區分 rustc_reject/charon_missing/ok 三態入口。
- **`ok_with_missing` 類（Charon 不支援嘅宣告，如部分 async/dyn）**：映射到
  `UNKNOWN(missing_decl)`——判定降級口徑已喺 c0_spike.py 實作。

## 下一步

- C0 收尾 = 喺 ≥4GB 機器跑 `scripts/c0_env_check.sh`（預計 15–30 分鐘全自動）。
  沙箱資源若提升，直接補跑一次即可將狀態轉綠。
- C1（PolyIR + LLBC-subset parser）與「執行環境」解耦——可先行按
  `multi_target::merge` 後嘅 serde schema 寫 std-only parser 骨架，
  等 spike fixtures 落地後即刻對接驗收（見 BLUEPRINT C1）。
