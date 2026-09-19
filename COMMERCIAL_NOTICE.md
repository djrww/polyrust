# COMMERCIAL NOTICE — PolyRust 商業使用權責說明

> 中英雙語 / Bilingual: 中文在前，English follows

---

## 中文 — 使用權責（必讀）

### 1. 雙重授權總覽

PolyRust 採用 **AGPL-3.0 + 商業授權** 雙重授權：

- **開源選項**：`LICENSE.AGPL-3.0` — 若你願意遵守 AGPL-3.0 的網絡 copyleft 義務（修改後以網絡服務形式提供時，必須向用戶提供完整對應源碼），可免費使用。
- **商業選項**：若你無法遵守 AGPL-3.0（例如閉源產品、SaaS 不開源、需要擔保/賠償/企業支持），必須取得商業授權。

未取得商業授權而以閉源/SaaS 形式使用，即違反 AGPL-3.0 第13條，可能導致版權侵權。

### 2. 什麼情況必須商業授權？

以下情況 **必須** 商業授權，否則視為違反 AGPL-3.0：

| 場景 | 是否需商業授權 | 說明 |
|------|----------------|------|
| 內嵌到閉源 App/固件/IDE 插件，不開源 | ✅ 必須 | AGPL-3.0 要求衍生作品整體 AGPL-3.0 |
| 以 SaaS/API/雲服務運行修改版，不提供源碼 | ✅ 必須 | AGPL-3.0 §13 要求網絡提供對應源碼 |
| 修改 `core/` 的 QAP/Groebner/CDCL/unsafe_safety 邏輯後私有部署 | ✅ 必須 | 核心管線屬 covered work |
| 需要擔保、賠償、SLA、ISO 26262 合規背書 | ✅ 必須 | AGPL-3.0 無擔保 (§15-16) |
| 個人學習、研究、開源項目且願意開源修改 | ❌ 可用 AGPL-3.0 | 遵守 AGPL-3.0 即可 |
| 內部使用未對外提供網絡交互 | ❌ 可用 AGPL-3.0 | 未 convey，無需開源，但建議商業以獲支持 |

### 3. 商業授權賦予的權利

取得商業授權後，你獲得：

- **無需開源**：可在閉源產品、SaaS、on-prem 中使用、修改、分發，無需提供對應源碼
- **專利授權**：貢獻者的必要專利主張的非排他、全球、免版稅許可（可談判範圍）
- **商標使用**：有限度的 "Powered by PolyRust" 商標使用（需書面）
- **企業支持**：可選 SLA、Lean 形式化證明追溯、F4/F5 性能調優、審計報告定制
- **QAP 鏈上證書**：商業版可生成 Solana/EVM 可驗證載荷的企業簽名證書

### 4. 商業授權的義務與限制

即使商業授權，你仍需：

- **保留版權聲明**：所有源文件頭保留 `Copyright (C) PolyRust Team` 及 `Dual Licensed: AGPL-3.0 OR Commercial`
- **禁止再授權為 AGPL-3.0 以規避**：商業授權的代碼不得以 AGPL-3.0 名義再分發給第三方以規避商業條款
- **第三方依賴合規**：`frontends/*` 依賴的 `axum/tokio/syn/ureq` 等仍需遵守其 MIT/Apache-2.0 許可
- **禁止移除安全門控**：不得移除 `unsafe_safety.rs` / `borrowck.rs` / `effects.rs` 中的 safety 多項式 `(1-valid)*deref=0` 等，否則失去 `all_unsafe_safe_implies_no_runtime_ub` 證明，商業擔保失效
- **審計追溯**：商業部署需保留 `audit_report.json` / `qap.json` 至少 3 年，供合規審計
- **漏洞披露**：發現 `static mut` data race、`union` tag mismatch 等安全問題，需在 90 天內向維護者披露

### 5. 如何取得商業授權？

1. 經商業授權聯絡渠道申請（**TBA — 首次商業發佈前公佈；過渡期請開倉庫 issue，標題 `[Commercial License Request] + 公司名`**）
2. 或在 GitHub 開 issue，標題 `[Commercial License Request]`，說明使用場景、是否修改 core、預計用戶數
3. 我們將提供商業授權協議（MNDA + 商業許可），含授權費、支持範圍、擔保條款

商業授權費參考（可談判）：
- Startup (<10人)：$2k/年
- SME (<100人)：$10k/年
- Enterprise：$50k/年 + 定制

### 6. 違規後果

- 未遵守 AGPL-3.0 又未取得商業授權，自動終止授權（AGPL-3.0 §8），需停止使用並刪除所有副本
- 首次違規可在 30 天內補救（提供源碼或取得商業授權）可恢復授權
- 惡意移除 safety 證明多項式或 Lean 定理，導致運行期 UB，商業擔保失效且可能承擔賠償

### 7. 聯繫與爭議

- 技術問題：GitHub Issues
- 商業/法律：**TBA（首次商業發佈前由維護者公佈；過渡期於倉庫開 issue）**
- 管轄法律：香港特別行政區法律，爭議提交香港國際仲裁中心 (HKIAC)

---

## English — Rights and Responsibilities

### 1. Dual License Overview

PolyRust is dual-licensed:

- **Open Source**: AGPL-3.0 (`LICENSE.AGPL-3.0`) — free if you comply with AGPL-3.0, especially Section 13 network source offer.
- **Commercial**: Proprietary license for those who cannot comply with AGPL-3.0.

Using PolyRust in closed-source or SaaS without complying with AGPL-3.0 and without a commercial license violates AGPL-3.0 §13 and may constitute copyright infringement.

### 2. When Commercial License is Required

| Scenario | Commercial Required? | Notes |
|----------|----------------------|-------|
| Embed in closed-source app/firmware/IDE without open sourcing | ✅ Yes | AGPL-3.0 requires whole derived work under AGPL-3.0 |
| Run modified version as SaaS/API without source offer | ✅ Yes | AGPL-3.0 §13 network Corresponding Source |
| Modify `core/` QAP/Groebner/CDCL/unsafe_safety and deploy privately | ✅ Yes | Core pipeline is covered work |
| Need warranty, indemnity, SLA, ISO 26262 attestation | ✅ Yes | AGPL-3.0 has no warranty (§15-16) |
| Personal learning, research, open source with source disclosure | ❌ AGPL-3.0 ok | Comply with AGPL-3.0 |
| Internal use without network interaction | ❌ AGPL-3.0 ok | No convey, but commercial recommended for support |

### 3. Rights Granted Under Commercial License

- **No source disclosure**: Use, modify, distribute in closed-source, SaaS, on-prem without Corresponding Source offer
- **Patent grant**: Non-exclusive, worldwide, royalty-free under contributors' essential claims (negotiable scope)
- **Trademark**: Limited "Powered by PolyRust" use with written permission
- **Enterprise support**: Optional SLA, Lean proof traceability, F4/F5 tuning, custom audit reports
- **QAP on-chain**: Enterprise-signed certificates for Solana/EVM verifiable payloads

### 4. Obligations and Restrictions Even With Commercial

- **Keep copyright**: Preserve `Copyright (C) PolyRust Team` and `Dual Licensed: AGPL-3.0 OR Commercial` in file headers
- **No re-licensing to evade**: Commercial-licensed code may not be redistributed as AGPL-3.0 to third parties to circumvent commercial terms
- **Third-party compliance**: `frontends/*` dependencies (axum, tokio, syn, ureq, etc.) remain under MIT/Apache-2.0
- **Do not remove safety gates**: Do not remove safety polynomials `(1-valid)*deref=0`, `(1-in_unsafe)*deref=0`, `safe - in_unsafe*precond=0` etc. in `unsafe_safety.rs` / `borrowck.rs` / `effects.rs`, otherwise `all_unsafe_safe_implies_no_runtime_ub` proof breaks and commercial warranty voids
- **Audit retention**: Keep `audit_report.json` / `qap.json` for at least 3 years for compliance
- **Vulnerability disclosure**: Report safety issues (static mut data race, union tag mismatch) to maintainers within 90 days

### 5. How to Obtain Commercial License

1. Use the commercial-licensing channel (**TBA — announced before first commercial release; meanwhile open a repository issue titled `[Commercial License Request] + Company`**)
2. Or open GitHub issue titled `[Commercial License Request]` describing use case, core modifications, estimated users
3. We will provide commercial agreement (MNDA + license) with fee, support, warranty

Indicative fees (negotiable):
- Startup (<10): $2k/year
- SME (<100): $10k/year
- Enterprise: $50k/year + custom

### 6. Consequences of Violation

- Using without AGPL-3.0 compliance and without commercial license automatically terminates license (AGPL-3.0 §8). You must cease use and delete copies.
- First-time violation may be cured within 30 days by providing source or obtaining commercial license.
- Malicious removal of safety polynomials or Lean theorems causing runtime UB voids commercial warranty and may incur liability.

### 7. Contact and Dispute

- Technical: GitHub Issues
- Commercial/Legal: **TBA (announced before first commercial release; open an issue meanwhile)**
- Governing law: HKSAR, disputes to HKIAC.

---

## 附錄：如何在源文件標註

Rust:
```rust
// PolyRust — Dual Licensed: AGPL-3.0 OR Commercial
// Copyright (C) 2024-2026 PolyRust Team
// See LICENSE and COMMERCIAL_NOTICE.md
// This file is part of PolyRust. If you use it under AGPL-3.0, you must
// comply with AGPL-3.0 §13 network source offer. For commercial use, see COMMERCIAL_NOTICE.md
```

Lean:
```lean
/- PolyRust — Dual Licensed: AGPL-3.0 OR Commercial
   Copyright (C) 2024-2026 PolyRust Team -/
```

Poly DSL:
```poly
# @license: AGPL-3.0 OR Commercial
# @copyright: PolyRust Team 2024-2026
```

---

**生效**：本通知自 v0.2.2 起生效。之前 MIT/Apache-2.0 版本仍按原許可，但新貢獻適用 AGPL-3.0 + Commercial。

**更新**：2026-09-17 香港
