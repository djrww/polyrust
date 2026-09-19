// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! 係數域：質域 𝔽_p，p = 2⁶¹ − 1（見 fp.rs）。
//!
//! # 為什麼用 𝔽_p 而非 ℚ
//! Gröbner 基在 ℚ 上會發生係數/分母複利爆炸（make_monic 的分數傳播），
//! i128 必然溢出；𝔽_p 上係數恆有界於 [0, p)，無爆炸（SNARK 與布爾代數求解的標準做法）。
//!
//! # 正確性（0/1 嵌入引理，證明見 docs/THEOREMS.md）
//! 本專案的多項式全部具有小整數係數（|c| < 2^16）、次數 ≤ 6、項數 < 2^12：
//! 1. 對 σ ∈ {0,1}^n：F(σ) 在 ℤ 上的值 |F(σ)| < 2^30 < p ⇒ F(σ) ≡ 0 (mod p) ⟺ F(σ) = 0。
//!    故 SAT 側見證可直接驗證（管線對 σ 逐多項式求值）。
//! 2. UNSAT 側：1 ∈ ⟨F ∪ B⟩_{𝔽_p} ⇒（強 Nullstellensatz）F ∪ B 在 𝔽̄_p 無公共根；
//!    而 {0,1}^n 的點在 ℚ 與 𝔽_p 中取值一致，故 ℚ 側亦無 0/1 解 ⇒ UNSAT 判定保真。

pub use crate::fp::Fp as Frac;

/// 實際使用：frac 文件清單 — 零依賴優化
pub fn frac_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("frac.rs", "係數域 ℚ→𝔽_p 封裝 — 優化 shim", "core/src/frac.rs"),
        ("fp.rs", "𝔽_p 2^61-1 實現 — frac 依賴", "core/src/fp.rs"),
    ]
}
