# 信任計算基（TCB）白皮書 —— Charon/rustc 章節（C5.3，2026-09-20）

本文定義 polyrust v0.3（C0–C5 之後）嘅信任計算基（Trusted Computing Base）：
**要相信判決，你必須信邊啲組件；以及每個組件嘅信任根據係咩。**
含殘餘風險同緩解。冇呢篇嘢之前，TCB 只存在喺散落嘅 C 級記檔度。

## 1. TCB 邊界圖

```
                        【信任層 T0：外部工具（唔audit，靠聲譽/pin）】
 ┌───────────────────────────────────────────────────────────────────┐
 │ 真 rustc（pinned nightly-2026-09-17）  真 charon（pinned ca501af6） │
 │   信任根據：上游編譯器/抽取器實戰經受；行為以 E-code/LLBC 可觀測；   │
 │   供應鏈錨：官方 rustup toolchain + release artifact（c0-charon-spike│
 │   tar，欲重建由源編譯對板）；Aeneas 技術報告明示「trust Charon        │
 │   correctly extracts MIR→LLBC」（arXiv 2609.15648）——我哋跟主流一致。│
 └───────────────────────────────┬───────────────────────────────────┘
                                 ▼ LLBC JSON（語法層）
                        【信任層 T1：解析 frontends/*】
 ┌───────────────────────────────────────────────────────────────────┐
 │ serde_json（業界標準 parser）＋ frontends/llbc 橋（13 行逐型映射，    │
 │ parity 測試鎖定）                                                   │
 │   —— 呢層出錯只會出「樹錯」，下一層 schema walker 會硬擋              │
 └───────────────────────────────┬───────────────────────────────────┘
                                 ▼ Value 樹
                        【信任層 T2：core 白名單防線（細、全部可測）】
 ┌───────────────────────────────────────────────────────────────────┐
 │ charon_llbc::LlbcRoot::from_value（root 3 key/translated 13 key/     │
 │ body 5 variant/keyset/duplicate 全硬錯）                            │
 │ llbc_body（statement/operand/rvalue/projection 白名單硬錯）           │
 │   —— 任何 schema 漂移 ⇒ 硬錯，唔會靜默走歪（C2 起嘅語義漂移防線）      │
 └───────────────────────────────┬───────────────────────────────────┘
                                 ▼ typed FunBody
                        【信任層 T3：語意 core（有 Lean 證明背書）】
 ┌───────────────────────────────────────────────────────────────────┐
 │ llbc_lower（SSA path lowering、switch/loop/call、fuel、contract）     │
 │   • bounded unroll 編碼忠實+抽象單調+UNSAT 反映:                     │
 │     Lean `Polyrust.UnrollSound.{steps_add,steps_mono,               │
 │     unroll_unsat_reflects}`（#print axioms = 僅 propext）           │
 │   • triangular presolve 健全性：單元測試（定義鏈消去唔扭判決）         │
 │   • abstraction markers：語義表（llbc_lower.rs 頂註白紙黑字）        │
 │   • contract premise：ExactEq 矛盾→UNSAT 端到端測試（硬證據）         │
 │ Gröbner 判定核心（Buchberger/F4/F5、CDCL）：                          │
 │   既有 Lean 鏈（SPoly、BoolNullstellensatz、ClauseAlgebra、           │
 │   T6Certificate……見 lean/Audit.out，全鏈零 sorry）                   │
 └───────────────────────────────┬───────────────────────────────────┘
                                 ▼ 判定 + markers + obligations
                        【信任層 T4：出口】
 ┌───────────────────────────────────────────────────────────────────┐
 │ CLI `polyrust-llbc-check`（統一 JSON、exit code 契約、rustc-reject    │
 │ →UNSAT 映射）；差分套件（CI 閘門）；c0_spike.py（fixture 溯源）        │
 └───────────────────────────────────────────────────────────────────┘
```

## 2. 信任根據明細（「信乜、點解信、點驗」）

| 組件 | 層 | 信乜 | 根據/驗證 |
|---|---|---|---|
| rustc | T0 | E-code 錯誤=真語義拒絕；LLBC 輸入=真編譯產物 | 上游編譯器；pinned nightly；reject→UNSAT 映射口徑喺 C2 差分逐位驗證 |
| charon | T0 | MIR→LLBC 抽取忠實 | Aeneas 同一信任假設（arXiv 2609.15648 明示）；pinned ca501af6；fixtures snapshot+c0_spike 可重生成 |
| serde_json | T1 | JSON 語法正確 | 業界標準；syntax 錯即拒（唔會產生歪樹畀下游） |
| frontends/llbc 橋 | T1 | 樹映射無損 | 真 fixture parity 測試；數字 i64 保留測試 |
| schema walker | T2 | 未知形狀必被硬擋 | 單元測試三線（unknown key/variant/missing key）全覆蓋 |
| llbc_body/lower | T3 | 語義編碼正確 | Lean UnrollSound 三定理 + ；；54 個 fixture 差分全線（loop 15、struct 15、commercial 10、contract 2） |
| GB 核心 | T3 | SAT/UNSAT 判決健全 | lean/Audit.out 全鏈 propext-only 既有定理 |
| CLI | T4 | 判決正確呈現 | 端到端 bad.rs/ok.rs 雙案 + exit code 單元測試 + llbc 直達測試 |

## 3. 明確**唔**喺 TCB 嘅嘢（唔使你信）

- **Markers/class 系統嘅精確度**——唔精確只影響「報告豐富度」；
  判決方向由 `unroll_unsat_reflects` 保證唔會變假 UNSAT。
- **presolve、timing 表**：屬性能優化；健全性有單元測試註解，
  就算壞咗只會變慢唔會變錯。
- **proxy/工具鏈快照**：環境損壞全部 fail-loud（charon 找不到 → 明確錯誤）。

## 4. 殘餘風險同緩解

1. **LLBC schema 隨 charon 升 pin 漂移**——緩解：白名單硬錯 + `charon.pin` + 
   c0_spike 再生成 fixtures 流程；升 pin 不經版控白名單審計唔准過。
2. **rustc/charon 供應鏈**——緩解：官方 toolchain 通道 + pinned artifact；
   release tar 有源對板說明（c0-charon-spike）。
3. **抽象層真冇寫錯？**——`steps_mono` 把「抽象放寬唔刪執行」形式化咗；
   剩低嘅係**人寫 marker 表時放寬方向搞反**（會變假 UNSAT）——
   緩解：差分套件 commercial/struct 25 例檢視判決一致（UNSAT 一方現假即紅燈）。
4. **Lean 證明同 Rust 實作嘅對接**——Lean 證嘅係「關係語義層」，Rust 嘅
   unroll/抽象係「呢層嘅一個實例化」——對接口嘅**審計單元**係
   llbc_lower.rs 頂註語義表（白紙黑字列邊啲節點抽象化），
   由 C2 法證規則（實測先入白名單）擋住慢慢膨脹。

## 5. 變更紀律

- 任何新 marker/抽象類別：必須（a）入語義表；（b）有新 fixture 差分；
  （c）UNSAT 方向檢視（唔能製造新 UNSAT）。
- 任何 TCB 邊界改動（新外部工具、新 layer）：必須更新本文件邊界圖。
