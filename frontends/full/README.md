# polyrust-full v0.2 原型前端

全 Rust 特性家族的擴展前端，實現 `docs/EXTENSION_PLAN.md` 中的架構。

## 功能

支援 9 大特性家族：

1. **struct / enum / impl / trait** → 積型/和型/方法表/存在量化
2. **Vec / String / HashMap** → 泛型容器 + 類型統一
3. **loop / while / for** → 有界展開 + @invariant 契約
4. **match** → 決策樹編譯 + 窮舉檢查
5. **mod 樹** → 模組扁平化 + 可見性
6. **async / await** → Future 狀態機
7. **I/O** → 效應系統 Result<T,E>
8. **unsafe / raw ptr** → 上下文位元 in_unsafe
9. **lifetime 'a** → outlives 圖 + NLL region

## 運行

```bash
cargo build -p polyrust-full --release
./target/release/polyrust-full 8091
# UI: http://localhost:8091/
# API: POST /api/v2/check {source}
```

## API

### POST /api/v2/check
```json
{ "source": "struct Point { x: i32 } fn main() {...}" }
```
→
```json
{
  "api_version": "0.2",
  "verdict": "SAT|UNSAT|UNKNOWN",
  "features_used": ["struct","enum"],
  "type_universe_size": 12,
  "stats": {"n_vars":..., "n_polys":...},
  "lowered_poly": "# lowered ...",
  "encoding_notes": [{"feature":"struct","encoding":"..."}]
}
```

### POST /api/v2/lower
僅降維，返回 core 兼容的 .poly v0.1

### 兼容 v0.1
- POST /api/check
- POST /api/expand
- GET /health

## UI

左側 9 特性分頁，每個帶可運行示例。中央編輯器 + 三標籤（源碼/降維後/編碼說明）。右側結果顯示 verdict、類型宇宙大小、編碼說明、降維輸出。

## 與 core 關係

- core 保持零依賴，`cargo test` 67 測試仍通過
- full 前端依賴 `polyrust-core` + `axum` + `tokio` + `serde`
- lowering 輸出可餵給 core 管線，複用 CDCL/Buchberger/QAP
- 未來：full 的 `lower.rs` 完整實現後，core 無需改動即可驗證 80% 安全 Rust

## 文件

- `src/ir.rs`: Surface AST + 特性檢測 + 降維原型
- `src/encoding.rs`: 9 特性的多項式編碼文檔
- `src/api.rs`: v0.2 API
- `src/main.rs`: axum 服務 + 內嵌 UI
- `../docs/EXTENSION_PLAN.md`: 完整設計藍圖
