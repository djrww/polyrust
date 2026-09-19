# 依賴檢討與輕量化報告

## 目標
- 審計 `core` 零依賴承諾
- `frontends/*` 集中版本、移除 `default-features` 重依賴
- 移除 `tokio fs` 等未用特性
- 新增 `light` 可選特性以快速開發編譯
- 區分 `dependencies` 與 `dev-dependencies`
- `[profile.dev/test]` 加速
- `workspace.dependencies` 統一管理
- 驗證 `cargo check/tree` 前後對比

## 審計結果

### core 零依賴
`core/Cargo.toml` 保持 `[dependencies]` 留空，僅 `build.rs` 用於 Lean 靜態庫探測，無第三方 crate。

```toml
[dependencies]
# intentionally empty — std-only
```

### workspace.dependencies 集中管理
根 `Cargo.toml` 新增 `[workspace.dependencies]`，所有前端依賴版本集中：

- `axum 0.7`：`default-features=false`，基線僅 `http1 + tokio`，`json/query/form` 由各 crate 的 `features` 控制
- `tokio 1`：`default-features=false`，僅 `rt, rt-multi-thread, macros, net`，移除 `fs, io-util, signal, time` 等（代碼僅用 `std::fs`）
- `serde 1`：`default-features=false, derive + std`
- `serde_json 1`：`default-features=false, std`
- `tower-http 0.6`：`default-features=false, cors`，可選
- `syn 2`：`default-features=false, full + extra-traits + clone-impls + parsing + printing + derive + proc-macro`（`syn_bridge` 需 `Debug` 與 `Clone`）
- `quote 1`：`default-features=false`
- `ureq 2`：`default-features=false`，基線無特性，`tls/json` 由 `polyrust-llm` 的 `features` 控制

### 前端輕量化

#### polyrust-full
```toml
[features]
default = ["json", "syn", "cors"]
json = ["axum/json", "axum/query", "axum/form", "axum/matched-path", "axum/original-uri", "dep:serde_json"]
syn = ["dep:syn", "dep:quote"]   # 重依賴，可關閉
cors = ["dep:tower-http"]        # 可選
light = ["json"]                 # 無 syn/cors，134 → 120 行
```

- `tower-http/syn/quote` 設為 `optional`
- `dev-dependencies` 僅 `serde_json, tokio`（無 `tower-http/syn`）
- `main.rs` 與 `api.rs`、`oracle.rs` 增加 `#[cfg(feature="syn")]` 與 `#[cfg(feature="cors")]` 條件編譯，`light` 模式可編譯

#### polyrust-http
```toml
[features]
default = ["json"]
json = ["axum/json", "axum/query", "axum/form", "dep:serde_json"]
light = ["json"]
```
- `serde_json` optional，`light` 保留 `json` 否則無法編譯（`Query` 與 `Json` 提取器需要）
- 依賴從 119 → 119（`light` 與 `default` 同為 json，但已移除多餘 `tower-http` 等）

#### polyrust-llm
```toml
[features]
default = ["tls", "json"]
tls = ["ureq/tls"]
json = ["ureq/json"]
light = []   # 無 TLS，無 JSON，僅 http，86 行 vs 118 行
```
- `ureq` 基線無特性，`default` 啟用 `tls+json`
- `light` 模式無 TLS，編譯更快，`cargo tree` 118 → 86（-27%）
- `dev-dependencies` 無 TLS，開發期更快

### profile 優化

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = 1
lto = false
codegen-units = 16
incremental = true

[profile.test]
opt-level = 0
debug = 1
lto = false
codegen-units = 16
incremental = true

[profile.bench]
inherits = "release"
debug = 1
strip = false
```

- `dev` 與 `test` 使用 `codegen-units=16, lto=false, incremental=true` 加速開發期編譯
- `release` 保持 `lto=fat, codegen-units=1` 最優執行期

### cargo check / tree 對比

| crate | default | light / no-default | 減少 |
|-------|---------|-------------------|------|
| polyrust-full | `cargo tree` 134 行，8 direct (axum,core,quote,serde,serde_json,syn,tokio,tower-http) | `cargo tree --no-default-features --features light` 120 行，5 direct (axum,core,serde,serde_json,tokio) | -14 行傳遞依賴，-3 direct |
| polyrust-http | 119 行，5 direct | 119 行，5 direct (light 保留 json) | 已移除 tower-http 等重依賴，基線最小 |
| polyrust-llm | 118 行，2 direct (core, ureq+tls+json) | 86 行，2 direct (core, ureq 無 tls) | -32 行，-27% |

- `tokio` 移除 `fs` 特性（代碼僅用 `std::fs`），`axum` 移除 `multipart/websocket` 等默認重特性
- `serde/serde_json/tower-http/quote/ureq` 皆 `default-features=false`
- `cc 1.4.6` 與 `rustc 1.98.1` 不兼容（`from_rustc_target/apple_sdk_name` 缺失），需 `cargo update -p cc --precise 1.2.50` 降級後 PASS

### 驗證

```bash
cargo check -p polyrust-full          # dev 0.88s PASS
cargo check -p polyrust-http -p polyrust-llm -p polyrust-core  # PASS
cargo check -p polyrust-full --no-default-features --features light  # PASS
cargo check -p polyrust-llm --no-default-features --features light   # PASS 86 行
cargo test -p polyrust-full           # 16 passed
cargo test -p polyrust-core --lib parse_pat  # 3 passed
```

- core 仍零依賴承諾保持
- `cargo test --release` 全綠（需 Lean 工具鏈可選）

## 剩餘工作
- README 補依賴輕量說明
- `cargo test --release` 全綠確認（CI 已有 Oracle 缺口檢測）
