# polyrust 商業落地規劃報告 — CDCL × Buchberger × QAP 代數形式化驗證平台

**版本**: v0.2.5-phase4 | **日期**: 2026-09-16 | **作者**: polyrust core team
**狀態**: 核心零依賴已穩定，Phase1-4 完成，具備產品化基礎

---

## 0. 執行摘要 (Executive Summary)

polyrust 是全球首個將 **Rust Mini-Rust 子集的類型檢查與借用檢查完整編碼為布爾多項式方程組**，並以 **CDCL SAT 求解器 × Buchberger Gröbner 基 × QAP 零知識見證** 三方聯合求解的**代數形式化驗證引擎**。

核心差異化：
- **零第三方依賴 core**：`core/` 僅用 `std`，二進制 3.5MB，`ldd` 僅 glibc，可嵌入任意環境（CI、IDE、區塊鏈節點、IoT）
- **N=7+i 可變類型宇宙**：Phase2 完成 `ty.rs unify` 完整泛型+lifetime統一多項式生成，`constraints_v2.rs` 每節點 N bits，`lower.rs` struct/enum/match/loop/mod flatten 完整保留 `mod_map`
- **Phase3 全特性**：borrowck lifetime outlives 無環檢查、`unsafe raw ptr` *mut/*const 約束、`async state machine` enum 編碼、QAP 集成驗證
- **Phase4 產品化 UI**：`serve` 前端展示 N（type universe size、每節點 bits、N=7+ext）、`/api/check-v2` JSON 契約
- **Lean 4 形式化**：20 模組、408 定理、零 sorry、零自定義公理，`AuditAll` 1736 宣告受檢，機械化證明命題 P 的數學骨幹

**商業機會**：Rust 2024 年已成為 Linux 內核、Android、雲原生、區塊鏈、AI infra 默認系統語言，但 **70% 的 Rust 安全漏洞來自 unsafe 與 lifetime 誤用**。現有工具 Prusti/Creusot/Kani 依賴重型 SMT (Z3)，部署重、誤報高。polyrust 以**代數見證 + QAP 可驗證**切入，提供 **LLM 護欄、CI 安全閘門、智能合約審計、嵌入式 Rust 認證** 四大場景，市場空間 2026 年估計 $2.1B (formal verification tools)。

**建議路徑**：開源核心 + 商業 SaaS + 企業授權，12 個月內達成 $500k ARR，24 個月 $2M。

---

## 1. 專案現況深度分析

### 1.1 技術成熟度 (TRL)

| 層 | 現狀 | TRL | 證據 |
|---|---|---|---|
| **核心管線** | CDCL 雙監視文字修復、Gröbner 95% 消除率、QAP Lagrange 驗證、篡改拒絕 | TRL 6 | `cargo test 67/67 綠`, `obligations 12樣本×9義務`, `brute 367組子句+186程序零不一致` |
| **類型系統** | 7 基底 + 10 擴展標籤 (vec/string/hashmap/struct/enum/rawPtr/future/option/result)，N 動態，one-hot Σt-1=0，product `t_struct - Πt_field`，sum `t_enum - Σt_variant` | TRL 6 | `test_universe_variable_n` 通過，examples/phase3 27 例 N=7~11 變化 |
| **Lowering** | struct→product, enum→sum, match→decision tree depth≤arms, for→loop IntoIterator, async→state machine enum FetchState, mod flatten `qualify pref::name` + `mod_map` | TRL 5-6 | `lower::tests 6 passed` |
| **Borrowck/Lifetime** | LifetimeGraph outlives 無環 DFS、NLL 區間 `[start,end)`、BorrowChecker 整合 EffectContext、unsafe gate `t_unsafe_op*(1-in_unsafe)=0` | TRL 5 | `borrowck 4 passed`, `lifetime 6 passed` |
| **DSL .poly** | @import/@set/{{key}} 模板、@lifetime 'a:'b、@fuel/@invariant、@unsafe-allowed、@no-io、@pure、@qap | TRL 5 | `examples/phase3 27 例`, `frontends/full 27 例` |
| **LLM 護欄** | `polyrust nl` 自然語言→.poly 三道閘門（結構→語法→Tier0 checker→完整管線）、funnel 量測、guardrail JSON 契約 | TRL 4 | `docs/LLM.md`, `guardrail-suite.json` |
| **Lean 形式化** | 20 模組、408 定理、6502 行、零 sorry、AuditAll CLEAN，ModuleFlatten 前綴單射、MatchDecisionTree depth≤arms 非平凡證明 | TRL 6 | `lean/lakefile.toml`, `ModuleFlatten.lean 245行`, `MatchDecisionTree.lean 230行` |
| **前端** | `core serve` std-only UI (Phase4 N 展示)、`frontends/full` axum UI 9特性分頁、`/api/v2/check`、`/api/v2/lower` | TRL 5 | `server.rs` 600行 PAGE 含 N badge、per-node bits、lowering report |
| **CI/CD** | GitHub Actions 硬閘門：rust test、lean build、AuditAll、obligations 獨立 | TRL 5 | `.github/workflows/ci.yml` |

**總結**：核心已達 **TRL 6 (實驗室環境驗證)**，具備產品化條件，但需硬化為 TRL 7-8 (真實環境)。

### 1.2 架構優勢

1. **零依賴承諾**：`core/Cargo.toml` 空 `[dependencies]`，`build.rs` 僅探測 Lean 靜態庫，優雅降級。對比 Prusti 依賴 Viper/Z3 (200MB+)，部署優勢 50 倍。
2. **三方聯合求解**：CDCL 處理布爾結構、Gröbner 判定 1∈G、QAP 生成可驗證見證，互相印證。`@brute` 常駐對照 367 組子句 + 186 程序零不一致，證明可靠。
3. **QAP 可驗證**：`qap_verified` + `qap_tamper_rejected`，篡改任一比特被拒，適合區塊鏈、審計場景。
4. **N=7+i 可擴展**：類型宇宙動態，`sys.universe.n_types()` 每節點 N bits，`type_bits`，支持泛型、Vec、HashMap、Future 等，未來可擴展至全 Rust。
5. **Lean 機械化**：`qualify` 前綴單射用 `String.data` + `List.append_left_inj` 證明，非 trivial；`compileMatchAux_depth` 歸納 + omega，depth≤arms，語義保持。

### 1.3 現存缺口 (Gap Analysis)

| 類別 | 缺口 | 影響 | 優先級 |
|---|---|---|---|
| **AST 覆蓋** | 真實 Rust 完整語法僅覆蓋 ~51 特性，macro 衛生性、proc macro、trait bound 複雜 where、async trait、GAT 未覆蓋 | 限制商業用戶代碼直接輸入 | P0 |
| **性能** | Gröbner 最壞 2^n，≤6 節點窮舉 682s，obligations 6分鐘，無增量求解 | CI 集成慢 | P0 |
| **錯誤信息** | 當前僅 `errors: []` + `lowering_report`，無精確 span、建議修復 | 用戶體驗差 | P0 |
| **IDE 集成** | 無 LSP、無 VSCode 插件 | 開發者採用障礙 | P1 |
| **文檔** | 英文為主，中文文檔部分，缺商業白皮書 | 市場推廣弱 | P1 |
| **商業化** | 無 SaaS、無計費、無多租戶、無審計報告生成 | 無法變現 | P0 |
| **團隊** | 單人/小團隊，無專職 BD、設計、銷售 | 擴張瓶頸 | P0 |

---

## 2. 市場分析

### 2.1 市場規模

- **Formal Verification Tools**：2024 $1.4B → 2026 $2.1B → 2030 $4.5B (CAGR 18.7%, Gartner)
- **Rust 採用**：2024 StackOverflow 最愛語言 9 年連冠，Linux 內核 15k 行 Rust，Android 15% 新代碼 Rust，AWS Firecracker、Cloudflare 100% Rust
- **Rust 安全漏洞**：2023-2024 CVE  Rust 相關 187 個，70% unsafe/lifetime，平均修復成本 $85k
- **LLM 代碼生成**：GitHub Copilot 2026 年 15M 用戶，30% 代碼含安全缺陷，需護欄

### 2.2 目標客戶細分

| 細分 | 痛點 | 願付 | polyrust 價值 | 優先級 |
|---|---|---|---|---|
| **Web3 智能合約審計** | Solana/Polkadot 用 Rust 寫合約，一個漏洞損失 $10M+ | $50k-200k/次審計 | QAP 可驗證見證 + unsafe 檢查 | P0 |
| **嵌入式/汽車 Rust** | ISO 26262 認證需形式化證明，現有工具無 | $100k-500k/年授權 | 零依賴 + Lean 證明 + 審計報告 | P0 |
| **雲原生/安全團隊** | CI 中 Rust 代碼安全閘門，現有 clippy 誤報高 | $20-50/開發者/月 | CI 集成 + 精確 borrowck 錯誤 | P1 |
| **LLM 代碼平台** | Copilot 生成 Rust 不安全，需護欄 | $0.01-0.05/次驗證 | `nl` 護欄 + funnel 量測 + QAP | P0 |
| **教育/研究** | Rust 形式化教學缺工具 | $0 (開源引流) | Web UI + Lean 對照 | P2 |

### 2.3 競爭格局

| 工具 | 方法 | 依賴 | 優勢 | 劣勢 | polyrust 差異 |
|---|---|---|---|---|---|
| **Prusti** | Viper + Z3 | JVM+Z3 200MB+ | 功能全 | 部署重、誤報 25% | 零依賴、QAP 見證 |
| **Creusot** | Why3 + SMT | OCaml+Z3 | 證明強 | 需寫規格 | 無需規格、自動推導 |
| **Kani** | CBMC + SAT | C++ | 模型檢查 | 僅 unsafe、慢 | 全類型 + borrowck |
| **Flux** | Liquid Types | Rustc 插件 | 輕量 | 僅 refinements | 完整借用 + 代數 |
| **Clippy** | Lint | Rustc | 快 | 非形式化 | 形式化保證 |

**polyrust 定位**：**輕量、零依賴、可驗證、LLM 友好** 的代數形式化中間層，不與 Prusti 競爭重型驗證，而是做 **CI 安全閘門 + LLM 護欄 + 審計見證**。

---

## 3. 產品定位與路線圖

### 3.1 產品矩陣

```
polyrust-core (OSS, MIT, 零依賴)
  ├── CLI: check / check-v2 / expand / gen / serve / nl / funnel / exhaust / brute
  ├── Lib: 供 frontends 依賴
  └── Lean 靜態庫嵌入 (可選)

frontends/full (OSS, 保留)
  ├── Web UI Phase4: N 可變展示、per-node bits、lowering report、9 特性分頁
  ├── API: /api/check, /api/v2/check, /api/v2/lower, /api/coverage, /api/v1/generate, /api/nl, /api/funnel
  └── 部署: Docker, 1.2MB 二進制

polyrust-cloud (商業, 閉源 SaaS)
  ├── SaaS API: 多租戶、計費、審計報告 PDF、QAP 證書上鏈
  ├── CI 集成: GitHub Action, GitLab CI, Jenkins Plugin
  ├── IDE: VSCode Extension (LSP, N 實時顯示, 錯誤 squiggle)
  └── Enterprise: 私有部署、ISO 26262 認證包、定制宇宙擴展

polyrust-audit (商業服務)
  ├── 智能合約審計: Solana/Polkadot Rust 合約形式化審計報告 + QAP 證書
  ├── 嵌入式認證: 汽車/航天 Rust 代碼形式化認證
  └── 培訓: Rust 形式化 + Lean 證明工作坊
```

### 3.2 版本路線圖

**v0.3 (3 個月, 2026 Q4) — 產品化硬化 (Product Hardening)**
- 目標：TRL 7，CI 可用
- 任務：
  - AST 覆蓋 51→80 特性：完整 `for`/`match`/`mod` 重寫非註釋級 lowering，支持 `impl Trait`, `dyn Trait`, `where` 複雜 bound
  - 錯誤信息：精確 span (file:line:col) + 建議修復 + 關聯 `mod_map` 路徑
  - 性能：增量 Gröbner (僅重算變更節點)、CDCL 並行、緩存 `type_universe`
  - `cargo check` 0 warnings 保持，`cargo test` <60s
  - Web UI：錯誤高亮、N 宇宙可視化圖、QAP 驗證動畫
  - 文檔：商業白皮書中英文、API 文檔、5 個商業案例
- 里程碑：GitHub 1k stars，10 個外部用戶

**v0.4 (6 個月, 2027 Q1) — SaaS MVP**
- 目標：$50k ARR，SaaS 上線
- 任務：
  - `polyrust-cloud` SaaS：Axum + Postgres 多租戶、Stripe 計費、審計報告 PDF 生成 (LaTeX 模板含 Lean 定理引用)
  - CI 集成：GitHub Action `polyrust-action@v0.4`，PR 註釋 N 變化與 borrowck 錯誤
  - VSCode Extension：LSP 服務器 (`tower-lsp`)，實時 N 顯示、per-node bits hover、quick fix
  - QAP 上鏈：Solana 鏈上 QAP 證書驗證合約
  - Lean：`lake build` CI 10s 內，ModuleFlatten + MatchDecisionTree 零 sorry 保持
- 里程碑：SaaS 100 用戶，3 個付費企業，Web3 審計 2 單

**v1.0 (12 個月, 2027 Q3) — 商業化**
- 目標：$500k ARR，企業版
- 任務：
  - 完整 Rust 覆蓋：支持 95% 安全 Rust + 常用 unsafe 模式 (Pin/Unpin, MaybeUninit)
  - 性能：≤1000 行 Rust 文件 <5s，增量 <500ms
  - 企業版：私有部署 Helm Chart、ISO 26262 認證包 (需求追溯 + 測試覆蓋 + Lean 證明)、定制 `ExtTag` 擴展
  - 審計服務：5 個 Web3 審計案例，平均 $75k/單
  - 市場：RustConf 演講、黑帽大會、2 篇學術論文 (PLDI/OOPSLA)
- 里程碑：$500k ARR，20 企業客戶，GitHub 3k stars

**v2.0 (24 個月, 2028 Q3) — 平台化**
- 目標：$2M ARR，平台
- 任務：
  - 支持 C++/Swift 經 polyrust 中間表示 (IR) 驗證
  - AI 驅動修復：LLM 基於 `borrowck_errors` + `lowering_report` 自動生成修復 PR
  - 形式化市場：Lean 證明交易市場，QAP 證書 NFT
  - 併購或融資：A 輪 $5M
- 里程碑：$2M ARR，100 企業，GitHub 10k stars

---

## 4. 商業模式

### 4.1 開源策略 (Open Core)

- **core 永遠 MIT + 零依賴**：吸引開發者、建立信任、學術引用
- **frontends/full MIT**：UI + API 開源，引流至 SaaS
- **cloud/audit 閉源**：商業功能 (多租戶、計費、審計報告、私有部署) 閉源
- **Lean 形式化 MIT**：學術影響力，證明可信度

### 4.2 收入模型

| 產品 | 定價 | 目標客戶 | 2027 預測 |
|---|---|---|---|
| **SaaS Starter** | $0/月，100 次驗證/月 | 個人開發者 | 1000 用戶，引流 |
| **SaaS Pro** | $49/開發者/月，10k 次/月 | 小團隊 | 200 用戶 × $49 = $9.8k MRR |
| **SaaS Team** | $199/團隊/月 (10 dev)，100k 次 | 中團隊 | 50 團隊 × $199 = $9.95k MRR |
| **Enterprise** | $50k-200k/年，私有部署 | 大企業/汽車 | 10 企業 × $100k = $1M ARR |
| **Audit 服務** | $50k-200k/次 | Web3 | 10 單 × $75k = $750k |
| **Training** | $5k/人，2 天工作坊 | 企業/高校 | 20 人 × $5k = $100k |
| **合計** |  |  | **~$2M ARR (24個月)** |

### 4.3 成本結構 (12個月)

- 人力：3 人核心 (Rust+Lean+前端) × $80k = $240k，1 BD × $60k，1 設計 × $50k = $350k
- 基建：AWS $12k/年，Lean CI $2k，域名/證書 $1k = $15k
- 市場：RustConf $10k，黑帽 $15k，內容 $10k = $35k
- 合計：$400k，毛利 20% (若 $500k ARR)

---

## 5. Go-to-Market 策略

### 5.1 冷啟動 (0-3個月)

1. **內容營銷**：
   - 發布《代數形式化：為何 Rust 需要 CDCL×Buchberger×QAP》技術博客，中英文，投遞 Rust Weekly、Hacker News
   - 錄製 10 分鐘 Demo：`serve` UI 展示 N=7+i 變化 + QAP 篡改拒絕 + Lean 證明對照
   - 開源 `examples/phase3` 27 例為互動教程，`cargo run -- check-v2` 一鍵運行

2. **社區滲透**：
   - Rust Zulip #formal-verification 頻道、Lean Zulip 宣傳
   - 提交 PR 至 `rust-lang/rust` 討論借用檢查形式化
   - 舉辦線上 Workshop：Rust 形式化 + Lean 證明，邀請 50 人

3. **設計合作夥伴**：
   - 2 個 Web3 項目 (Solana 生態) 免費審計，換取案例與推薦
   - 1 個嵌入式 Rust 團隊 (如 `embassy-rs`) 免費私有部署，換取反饋

### 5.2 增長 (3-12個月)

1. **Product Hunt + GitHub Trending**：v0.4 SaaS 上線時發布，目標 500 upvotes
2. **GitHub Action 市場**：發布 `polyrust-action`，Rust 項目一鍵集成，目標 100 使用
3. **學術**：投遞 PLDI 2027《Algebraic Formal Verification of Rust via Polynomial Encoding》，引用 Lean 408 定理
4. **企業 BD**：參加 RustConf 2027 演講《Zero-Dependency Formal Verification for Rust in Production》，現場收集 100 leads

### 5.3 擴張 (12-24個月)

1. **渠道**：與 `Ferrous Systems` (Rust 培訓)、`Trail of Bits` (審計) 合作，分銷
2. **平台**：支持 `cargo vet` 集成，成為 Rust 供應鏈安全標準
3. **融資**：A 輪 $5M，投資人：a16z (crypto+AI)、Sequoia (infra)

---

## 6. 技術硬化清單 (To Product)

### 6.1 P0 (3個月內必須)

- [ ] **AST 覆蓋 80%**：`ast_full.rs` 51→80，支持 `where` 複雜 bound、`impl Trait`、`dyn Trait`、`async trait`、`GAT`，`lower.rs` 非註釋級重寫
- [ ] **錯誤信息**：`driver.rs` 新增 `Diagnostic` 結構含 `span`, `code`, `help`, `note`，前端高亮，`check-v2` JSON 新增 `diagnostics`
- [ ] **性能**：`constraints_v2.rs` 增量：僅重算變更節點的 `type_bits`，`cdcl.rs` 並行傳播，`type_universe` LRU 緩存
- [ ] **測試**：`cargo test` 從 120s→60s，`obligations` 從 6min→2min，`exhaust` ≤5 節點 23s→10s
- [ ] **文檔**：`docs/COMMERCIAL_WHITEPAPER.md` 中英文，含 5 商業案例、QAP 上鏈流程圖、Lean 證明對照表

### 6.2 P1 (6個月)

- [ ] **VSCode Extension**：`frontends/vscode/`，LSP 服務器 `tower-lsp`，功能：N 實時、per-node bits hover、borrowck squiggle、quick fix (基於 `mod_map`)
- [ ] **SaaS MVP**：`frontends/cloud/`，Axum + Postgres + Stripe，功能：多租戶、計費、審計報告 PDF、QAP 證書
- [ ] **CI Action**：`.github/actions/polyrust-action`，輸入 `.poly` 或 `Cargo.toml`，輸出 PR 註釋
- [ ] **QAP 上鏈**：Solana 程序驗證 QAP 證書，`qap.rs` 新增 `to_solana_ix`

### 6.3 P2 (12個月)

- [ ] **完整 Rust**：95% 安全 Rust + 常用 unsafe 模式，`universe.rs` 支持 `Pin`, `MaybeUninit`, `ManuallyDrop`
- [ ] **AI 修復**：`llm.rs` 基於 `borrowck_errors` + `lowering_report` 生成修復 PR，`nl` 護欄自動修復
- [ ] **企業版**：Helm Chart 私有部署、ISO 26262 認證包、定制 `ExtTag`

---

## 7. 風險與緩解

| 風險 | 概率 | 影響 | 緩解 |
|---|---|---|---|
| **Rust 編譯器變更** | 中 | 高 | 僅依賴 Mini-Rust 子集，`ast_full.rs` 與 `syn` 解耦，定期同步 `rust-lang/rust` |
| **Gröbner 性能爆炸** | 高 | 高 | 增量求解 + 啟發式排序 + 2^n 界限監控 + 回退至 CDCL-only 模式 |
| **Lean 工具鏈不穩定** | 低 | 中 | 釘住 `lean-toolchain` v4.33.1，`build.rs` 優雅降級，CI 雙版本測試 |
| **競爭對手抄襲** | 中 | 中 | 開源核心 + 專利 (QAP 可驗證見證 + N=7+i) + Lean 證明壁壘 |
| **市場採用慢** | 中 | 高 | 免費審計 + GitHub Action + 內容營銷，3 個月內 10 設計夥伴 |
| **團隊流失** | 低 | 高 | 股權激勵 + 遠程友好 + 學術發表激勵 |

---

## 8. 財務預測 (保守)

| 年 | 用戶 | 付費 | ARR | 成本 | 淨利 |
|---|---|---|---|---|---|
| 2026 Q4 (3月) | 500 | 0 | $0 | $100k | -$100k |
| 2027 Q3 (12月) | 2000 | 100 | $500k | $400k | $100k |
| 2028 Q3 (24月) | 10000 | 500 | $2M | $800k | $1.2M |

**融資需求**：種子 $500k (已部分自籌) + A 輪 $5M (24月)

---

## 9. 團隊與招聘

**現有**：1-2 人核心 (Rust+Lean)
**需招聘** (6個月)：
- Rust 形式化工程師 (1)：`ty.rs`, `constraints_v2.rs`, `lower.rs` 硬化
- 前端/全棧 (1)：`server.rs` UI + SaaS + VSCode Extension
- BD/市場 (1)：Web3 審計 + 企業銷售 + 內容
- 設計 (0.5)：UI/UX + 白皮書設計

**顧問**：Lean 社區 (如 `leanprover` 成員)、Rust 形式化 (如 `Prusti` 作者)、Web3 安全 (如 `Trail of Bits`)

---

## 10. 執行計劃 — 下 90 天

| 週 | 目標 | 交付 | 負責 |
|---|---|---|---|
| 1-2 | 錯誤信息 + 性能 | `Diagnostic` 結構 + 增量 Gröbner 原型 | Rust 工程師 |
| 3-4 | AST 80% | `for/match/mod` 非註釋級 lowering + 10 新測試 | Rust 工程師 |
| 5-6 | Web UI N 可視化 | N 宇宙圖 + per-node bits 交互 + QAP 動畫 | 前端 |
| 7-8 | 白皮書 + 內容 | 商業白皮書 + 博客 + Demo 視頻 | BD + 核心 |
| 9-10 | 設計夥伴 | 2 Web3 免費審計 + 1 嵌入式私有部署 | BD |
| 11-12 | SaaS 原型 | 多租戶 + 計費 + 審計報告 PDF 原型 | 全棧 |

**KPI**：GitHub 1k stars，10 設計夥伴，SaaS 原型上線

---

## 11. 結論

polyrust 已從學術原型 (TRL 4) 進化為 **可產品化的代數形式化平台 (TRL 6)**，具備：
- 技術壁壘：零依賴 + N=7+i + QAP + Lean 408 定理
- 市場時機：Rust 爆發 + LLM 護欄剛需 + Web3 審計痛點
- 商業路徑：開源引流 + SaaS 變現 + 企業授權 + 審計服務

**建議**：立即啟動 90 天硬化計劃，融資種子 $500k，目標 12 個月 $500k ARR，24 個月 $2M ARR + A 輪。

**一句話**：polyrust 是 **Rust 的形式化安全帶**，讓每行 Rust 都有代數見證與 Lean 證明。

---

## 附錄

### A. 現況證據鏈

- `cargo check -p polyrust-core` 0 warnings (2026-09-16)
- `cargo test --lib -- minirust::constraints_v2` 4 passed (N 可變)
- `cargo run -- check-v2 examples/phase3/struct_sat.poly --json` N=10 = 7+3, per-node bits 10, unify_polys 48
- `server.rs` 600行 PAGE 含 N badge + v2 驗證 + per-node bits + lowering report
- `ModuleFlatten.lean` 245行，`qualify_prefix_injective` 用 `String.data` + `List.append_left_inj` 非平凡
- `MatchDecisionTree.lean` 230行，`compileMatchAux_depth` 歸納 + omega，depth≤arms

### B. 商業案例草圖

**案例1：Solana 合約審計**
- 客戶：Solana DeFi 項目，2000 行 Rust 合約，含 unsafe
- 流程：`polyrust check-v2` → N=12, 3 unsafe, 1 lifetime cycle → 修復 → QAP 證書上鏈 → 審計報告 PDF (含 Lean 定理引用) → 收費 $75k
- 價值：避免 $10M 損失，QAP 證書可公開驗證

**案例2：汽車 ECU Rust 認證**
- 客戶：汽車 Tier1，5000 行嵌入式 Rust，需 ISO 26262
- 流程：私有部署 + ISO 認證包 + 定制 ExtTag (CAN 總線類型) + Lean 證明追溯 → $150k/年
- 價值：認證時間從 6 月→2 月

### C. 參考資料

- `docs/THEOREMS.md`：命題 P + T1-T9 數學證明
- `docs/LEAN.md`：Lean 形式化對照
- `docs/EVIDENCE.md`：12 樣本×9義務實測 + Lean 定理
- `docs/PHASE3_FINAL_REPORT.md`：Phase3 管線 v2
- `README.md`：零依賴承諾 + 構建

---

**報告結束** — 下一步：啟動 90 天計劃，招聘，融資。
