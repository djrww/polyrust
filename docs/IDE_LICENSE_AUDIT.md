# IDE 前端依賴授權審計報告 — v0.2.1

> 目標：確認 `frontends/ide` 全部依賴商業友好，`cargo deny` 全綠，無 GPL/AGPL/LGPL 傳遞依賴（除 workspace 自身雙重授權）。

## 結果

```
cargo deny check all
# licenses ok, bans ok, sources ok, advisories ok
```

僅 warnings: duplicate bitflags/mio/syn/tower（notify vs axum 歷史版本差異），屬 warn 級別。

## 授權分佈

- MIT: axum, tokio, tower, hyper, bytes, etc.
- Apache-2.0: serde, etc.
- BSD-3-Clause: matchit, subtle
- ISC: inotify
- CC0-1.0: notify
- CDLA-Permissive-2.0: webpki-roots (permissive)
- Unicode-3.0: icu_*
- AGPL-3.0-only: 僅 workspace 成員

無 GPL/LGPL，商業友好。

## 直接依賴

| Crate | License | 商業友好 |
|-------|---------|----------|
| axum 0.7 | MIT | ✅ |
| tokio 1 | MIT | ✅ |
| serde 1 | MIT OR Apache-2.0 | ✅ |
| tower-http 0.6 | MIT | ✅ |
| tower 0.4 | MIT | ✅ |
| notify 6 | CC0-1.0 OR MIT OR Apache-2.0 | ✅ |
| lsp-types 0.94 | MIT OR Apache-2.0 | ✅ |
| ropey 1.6 | Apache-2.0 | ✅ |

## 商業結論

- IDE 前端可商業閉源：依賴全部商業友好，無 GPL 污染
- 核心零依賴：AGPL-3.0-only OR Commercial
- Safety Gates 不可移除：COMMERCIAL_NOTICE.md §5
