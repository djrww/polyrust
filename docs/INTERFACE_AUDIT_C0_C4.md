# C0–C4 接口鏈審計報告（2026-09-20）

目標：複檢 C0–C4 所有接口——**真 rustc → Charon LLBC（IR 層）→ core → 真實出口**，
全鏈無 mock/stub/fake；自寫 JSON parse 轉真 serde（放前端）；清 bug/債/重。

## 1. 全鏈真實性驗證

```
真源碼語料（semantic_matrix 100 例 + examples/*.poly + examples/phase3/*.poly）
   │  scripts/c0_spike.py（subprocess 調真 charon binary，
   │  `charon rustc -- <file> --crate-type=rlib --edition=2021`）
   ▼
真 rustc nightly-2026-09-17（charon 內嵌 rustc driver）
   │  真編譯器錯誤（E0xxx）→ UNSAT 地真值；accept → LLBC
   ▼
LLBC JSON（core/tests/charon_fixtures/*.llbc，54 個 snapshot 入倉，pinned ca501af6）
   │  frontends/llbc —— **真 serde_json**（遷移後；詳 §2）
   ▼
core::charon_llbc::Value 樹（零依賴記憶體樹）
   │  core::charon_llbc::LlbcRoot::from_value —— **法證白名單 schema walker**
   │  （多 key/少 key/未知 variant/型別唔對 → 硬錯；C2 起嘅語義漂移防線）
   ▼
core::llbc_body（LLBC → typed FunBody，statement/operand/rvalue 白名單硬錯）
   ▼
core::llbc_lower（SSA path lowering + continuation switch/loop/call + fuel
   + contract premise + assert_obligations）
   ▼
Gröbner 判定（SAT / UNSAT / Unknown + reason）
   ▼
真實出口：
   • differential_v4.rs 差分套件（CI 閘門：matrix 10 + loop 15 + struct 15
     + commercial 10/10 + contract 端到端 ×2）——C1–C4 驗收出口
   • c0_spike.py results.json/summary.md（fixture 溯源 + 漂移報告）
   • frontends/llbc `parse_llbc_text/parse_llbc_file`（產品級 API，
     供後續 CLI/服務出口用）
```

**mock/stub/fake 審計結果**：

| 位置 | 性質 | 處置 |
|---|---|---|
| `core/src/qap_groth16.rs` | **假 prover**：`DefaultHasher` 生成「Groth16 proof」、`verified: true` 硬編碼、假 on-chain verifier、模組聲稱「arkworks/bellman 真實」 | **整模組鏟除**（commit 3128abb）。`docs/PHASE_B_COMPLETION_REPORT.md` §5 嘅「Groth16 真實 ✅」係過度宣稱，以此報告更正；真 prover 如需要，新開任務（arkworks），嚴禁再出「結構兼容嘅假」 |
| `core/src/llm.rs` `MockProvider` | LLM 護欄鏈（NL→.poly）嘅離線測試 provider，文件明示「內建測試用」；**唔喺 C0–C4 鏈上** | 保留（合理 test double），文件標記清晰 |
| `frontends/full/src/oracle.rs` light-stub | syn oracle 嘅 light-feature 降級（feature-gated），明示 | 保留（明確降級策略，唔係假實現） |
| C0–C4 鏈本身 | 無 todo!/unimplemented!/mock/stub/fake | ✓ |

## 2. serde 遷移（自寫 parse → 真 serde）

**邊界重劃**（架構決策）：

- 「字節 → JSON 樹」全部移交 `frontends/llbc`（真 serde_json：轉義、
  代理對、數字邊界、巢深全部由 serde_json 保證）。
- core 再無 JSON lexer（刪 ~190 行手寫 `Parser`/`parse_json`）；core 保留：
  - `charon_llbc::Value` 零依賴記憶體樹（唔係 parser 職責）
  - `LlbcRoot::from_value` **schema walker**——呢個**唔係 JSON parsing**，
    係 C2 以嚟嘅法證白名單防線（未知 variant 硬錯），永遠留 core。
- `core [dependencies]` 繼續為空（承諾不變）；serde 全部喺 frontends/*。
- core 測試經 serde_json（**dev-dependencies only**）；曾試 dev-dep 循環
  （core → frontends/llbc）——crate 分裂導致型別唔 unify（E0308），棄用，
  改 test bridge（`value_for_test/root_for_test`），同 frontends/llbc 嘅
  `from_serde_value` 逐行同源，frontend parity 測試用真 fixture 鎖一致。

**語義變化（唯一一處，已記錄）**：數字原文以 serde_json 規範化
（`-12.5e3 → -12500.0`）；LLBC number 全部係 id/整數字面量，`num_i64`
（schema 層）唔受影響；round-trip 測試照綠。

## 3. Bug / 債 / 重：修復清單（commit 3128abb）

| 類 | 問題 | 修法 |
|---|---|---|
| 假 | qap_groth16 假 prover（§1 表） | 鏟除整模組 |
| 債 | core 手寫 JSON lexer | serde 遷移（§2） |
| 債 | `frontends/full/src/main.rs.bak` 意外入倉（372 行過期備份） | git rm |
| bug 級警告 | 死賦值 8 處（unused_assignments：init 值必被覆寫） | rustc 證明下去 init 留型別宣告 |
| bug 級警告 | `deref_base`、`eliminated_total` 死計算 | 刪/底線化 |
| 債 | `attempts` 死賦值模式（每 arm assign 1） | 等式化 `let attempts = 1`（語義可證明相同） |
| 重 | workspace lib 警告 24 個（unused import/var 等） | **清零** |
| 債 | clippy ~250 style lint | `cargo clippy --fix`（MachineApplicable，51 檔 +235/−266）＋ `frontends/full/src/ir.rs` 骨架保真聲明 + ide main rust_code 去 init |

## 4. 接受技術債（認領，唔係漏）

1. **`core/src/llm.rs` 自寫 JSON 讀取器**（~180 行）：LLM 回包解析喺 core——
   因為 entire LLM provider 層（http:// 裸 TCP + https:// system curl）架構上喺
   core（零依賴承諾下唯一選擇）。改成 serde 要連埋 provider 層搬出 core，
   屬獨立架構任務；已審計：parser 行為正確（代理對/巢深處理有測試覆蓋）。
2. **clippy 剩餘 ~87 個非 MachineApplicable lint**（strip_prefix 26、
   complex-type 6、doc 縮排 6、loop-index 迭代式 16 等）——style 級，
   逐個人工改寫嘅回歸風險 > 收益；建議加入 CI 只擋 correctness 類。
3. **v4 產品出口**：v4 → differential suite（CI 閘門）+ library API +
   `frontends/llbc` parse API 已定；一鍵 CLI 出口（`polyrust-llbc-check`）
   係建議下一步（若要做：frontend bin 行 analyze_module → verdict JSON）。

## 5. 環境教訓重申

sandbox 快照唔保存 `.git/`、`~/.cargo/bin` proxies、`~/.rustup`：
每個 turn 重開要 `ln -f rustup cargo rustc ...`＋`rustup default stable`；
重要 commit 要即時 push 或者 `git format-patch` 備份入 workspace
（今次已備份 `/home/user/audit-harden-commit.patch`）。

## 6. 驗收現況

- `cargo test -p polyrust-core -p polyrust-llbc --lib`：**126 + 4 全綠**
- `cargo build --workspace --lib`：**0 warning**
- `cargo clippy --workspace --all-targets`：correctness 類 0；剩 style 級（§4.2）
- license-scan：全檔 AGPL-3.0+Commercial 雙授權標頭 ✓
