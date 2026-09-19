# IDE 前端準備工作報告 — v0.2.1

## 任務
- 建立 IDE 資料夾 `frontends/ide` 作為正式前端
- 用 `cargo deny` 掃描現時 IDE 依賴，確認是否商業友好
- 開新分支 v0.2.1 並推送

## 完成

### 1. IDE 資料夾
- Cargo.toml：商業友好依賴 axum, tokio, serde, tower-http, tower, notify, lsp-types, ropey
- main.rs：完整 IDE 後端 file-tree/open/save/compile/format/lint/analyze/git-status/safety-gates + Web UI，真實管線
- README.md：功能與授權說明

### 2. cargo deny
- licenses ok, bans ok, sources ok, advisories ok
- 無 GPL/LGPL，全部商業友好

### 3. 其他
- Cargo.toml workspace 加入 frontends/ide，version 0.2.1，license AGPL-3.0-only OR MIT OR Apache-2.0
- deny.toml 新增
- CI 新增 deny job
- 同步 AGPL 雙重授權文件

### 4. 驗證
cargo check -p polyrust-ide --features light PASS
cargo deny check all PASS
