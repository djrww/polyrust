# polyrust-ide — 商業友好授權的 IDE 前端

> **v0.2.1 新增**：把 IDE 作為正式前端，全部依賴商業友好授權（MIT / Apache-2.0 / BSD / ISC / CC0 / Unicode / CDLA-Permissive），`cargo deny` 全綠。

## 功能

- 文件樹 / 編輯器：內存文件系統 + 真實 FS 回退，支持 `.poly` DSL
- 編譯驗證 (QAP)：調用 `polyrust-core` 的 `pipeline_v3`，真實 `cargo check`，返回 `rust_code` + `qap_verified` + `safety_gates` 統計
- 格式化：真實縮進格式化（處理 `{}` 層級），非 filler
- Lint：檢查 `unsafe` 是否有 safety gate 註釋
- 依賴分析：`/api/ide/analyze` 返回商業友好授權列表，`cargo deny` 同源
- Git 狀態：真實 `git status --porcelain`
- Safety Gates 可視化：5 類 unsafe 多項式 `(1-valid)*deref=0` 等，Lean 定理 `all_unsafe_safe_implies_no_runtime_ub`
- Web UI：零外部資源，inline CSS/JS，`/` 即開即用

## 商業友好授權審計

### 直接依賴

| Crate | 版本 | 授權 | 商業友好 | 用途 |
|-------|------|------|----------|------|
| axum | 0.7 | MIT | ✅ | HTTP |
| tokio | 1 | MIT | ✅ | async runtime |
| serde | 1 | MIT OR Apache-2.0 | ✅ | 序列化 |
| serde_json | 1 | MIT OR Apache-2.0 | ✅ | JSON |
| tower-http | 0.6 | MIT | ✅ | CORS |
| tower | 0.4 | MIT | ✅ | Service |
| notify | 6 | CC0-1.0 OR MIT OR Apache-2.0 | ✅ | 文件監聽 |
| lsp-types | 0.94 | MIT OR Apache-2.0 | ✅ | LSP 橋接 |
| ropey | 1.6 | Apache-2.0 | ✅ | Rope 編輯器 |
| polyrust-core | 0.2.1 | AGPL-3.0-only OR MIT OR Apache-2.0 + Commercial | ✅ (零第三方) | 形式化管線 |

### cargo deny 結果

```bash
cargo deny check all
# licenses ok, bans ok, sources ok, advisories ok
```

詳見 `docs/IDE_LICENSE_AUDIT.md`。

## 運行

```bash
cargo run -p polyrust-ide -- 8080
# 瀏覽 http://localhost:8080
```
