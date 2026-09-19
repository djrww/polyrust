// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! 多值變量的消失多項式（vanishing polynomial）編碼 + 引理 L0′。
//!
//! # 動機（見 docs/THEOREMS.md §2b、§6b）
//! 過渡到真 rustc／Chalk 後會遇到**多值變量**（值域大小 k > 2 的變量，
//! 例如候選型別集、obligation 的解空間）。兩種編碼的代數終止界截然不同：
//!
//! | 編碼 | 變量數 | 該變量貢獻的標準單項式界 |
//! |---|---|---|
//! | one-hot（k 個布爾位 + Σ=1） | k | 2^k（名義；Σ=1 收緊後為 k） |
//! | 消失多項式 ∏_v (x−v)（單變量、次數 k） | 1 | **k** |
//!
//! 消失多項式把每個多值變量的界由指數降為線性；全系統界即一般化的
//! **∏ᵢ kᵢ**（T4 的 2ⁿ 是 kᵢ ≡ 2 的特例）。
//!
//! # 布爾情形的一致性
//! 值域 {0,1} 的消失多項式 x(x−1) = x²−x 恰好就是域多項式——
//! 本模組是 [`crate::groebner::field_polys`] 的直接推廣（測試鎖定此事實）。
//!
//! # 引理 L0′（嵌入保真的新數值界）
//! 值域 ⊆ [−V, V] 的賦值下，項數 T、係數絕對值 ≤ C、次數 ≤ d 的多項式滿足
//! |f(σ)| ≤ T·C·V^d；只要此界 < p = 2⁶¹−1，𝔽_p 上的歸零判定與整數意義一致。
//! 原 L0 是 V = 1 的特例（T < 2¹²、C < 2¹⁶、d ≤ 6 ⇒ 2²⁸ < p）。

use crate::fp::P;
use crate::frac::Frac;
use crate::poly::Poly;

/// 消失多項式：∏_{v∈domain} (x_var − v)。
/// 在值域內任一點求值為零，值域外（整數意義）非零。
pub fn vanishing_poly(var: usize, domain: &[i64], nvars: usize) -> Poly {
    let mut acc = Poly::constant(Frac::ONE);
    for &v in domain {
        let factor = Poly::var(var, Frac::ONE, nvars).sub(&Poly::constant(Frac::from_i64(v)));
        acc = acc.mul(&factor);
    }
    acc
}

/// 域多項式的多值推廣：以消失多項式取代 x²−x。
/// `domain = &[0,1]` 時與 [`crate::groebner::field_polys`] 的對應分量一致。
pub fn field_poly_for_domain(var: usize, domain: &[i64], nvars: usize) -> Poly {
    vanishing_poly(var, domain, nvars)
}

/// 係數的最小絕對值代表（適用於小整數係數的 𝔽_p 元素）。
pub fn coeff_abs_repr(c: &Frac) -> u64 {
    if c.0 <= P / 2 {
        c.0
    } else {
        P - c.0
    }
}

/// L0′ 參數包：由多項式實際內容推導（項數、最大係數絕對值），
/// 加上調用方提供的值域界 V 與次數界 d。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct L0PrimeParams {
    pub n_terms: u64,
    pub max_abs_coeff: u64,
    pub max_abs_value: u64,
    pub degree: u32,
}

impl L0PrimeParams {
    /// |f(σ)| ≤ T·C·V^d（三角不等式 + |xᵢ| ≤ V + 單項式值 ≤ V^d）。
    /// 溢出時回傳 None（即「界無法保證」⇒ 不保真，應拒絕編碼）。
    pub fn eval_bound(&self) -> Option<u128> {
        let mut acc = (self.n_terms as u128).checked_mul(self.max_abs_coeff as u128)?;
        for _ in 0..self.degree {
            acc = acc.checked_mul(self.max_abs_value as u128)?;
        }
        Some(acc)
    }

    /// 嵌入保真：界 < p ⇒ 𝔽_p 歸零判定與整數意義一致（L0′ 結論）。
    pub fn faithful(&self) -> bool {
        match self.eval_bound() {
            Some(b) => b < (P as u128),
            None => false,
        }
    }
}

/// 由多項式實測參數（項數、最大係數）＋外部給定的值域界/次數界構造 L0′ 參數。
pub fn l0_prime_params_of(p: &Poly, max_abs_value: u64, degree_cap: u32) -> L0PrimeParams {
    let n_terms = p.terms.len() as u64;
    let max_abs_coeff = p.terms.iter().map(|(_, c)| coeff_abs_repr(c)).max().unwrap_or(0);
    let degree = p.terms.iter().map(|(m, _)| m.iter().sum::<u32>()).max().unwrap_or(0);
    let _ = degree_cap; // 次數以實測為準（保真界寧緊勿鬆）；cap 僅供調用方參考
    L0PrimeParams {
        n_terms,
        max_abs_coeff,
        max_abs_value,
        degree,
    }
}

/// 單個多值變量的兩種編碼對比。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncodingComparison {
    /// 值域大小。
    pub k_values: usize,
    /// one-hot：所需布爾位數。
    pub one_hot_bits: usize,
    /// one-hot：該變量貢獻的標準單項式名義界 = 2^k（None = 溢出）。
    pub one_hot_bound: Option<u128>,
    /// 消失多項式：所需變量數（恆 1）。
    pub vanishing_vars: usize,
    /// 消失多項式：多項式次數 = k。
    pub vanishing_degree: usize,
    /// 消失多項式：該變量貢獻的標準單項式界 = k。
    pub vanishing_bound: u128,
}

/// k 值變量的編碼對比：2^k（one-hot 名義界） vs k（消失多項式界）。
pub fn compare_encodings(k: usize) -> EncodingComparison {
    EncodingComparison {
        k_values: k,
        one_hot_bits: k,
        one_hot_bound: if k < 127 { Some(1u128 << k) } else { None },
        vanishing_vars: 1,
        vanishing_degree: k,
        vanishing_bound: k as u128,
    }
}

/// 一般化終止界 ∏ᵢ kᵢ（T4 之 2ⁿ 在 kᵢ ≡ 2 時即此式的特例）。
/// 溢出回傳 None。
pub fn standard_bound_product(ks: &[usize]) -> Option<u128> {
    let mut acc = 1u128;
    for &k in ks {
        acc = acc.checked_mul(k as u128)?;
    }
    Some(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_vanishing_is_field_poly() {
        // 值域 {0,1} 的消失多項式 = x²−x（域多項式）—— 一致性鎖定
        let vp = vanishing_poly(0, &[0, 1], 1);
        let fp = crate::groebner::field_polys(1);
        assert_eq!(vp, fp[0], "k=2 的消失多項式必須等於域多項式");
    }

    #[test]
    fn vanishing_zero_on_domain_nonzero_outside() {
        let domain = [0i64, 1, 2, 3];
        let vp = vanishing_poly(0, &domain, 1);
        for &v in &domain {
            let val = vp.eval_full(&[Frac::from_i64(v)]);
            assert!(val.is_zero(), "值域點 {} 必須歸零", v);
        }
        for v in [4i64, 5, -1, 7] {
            let val = vp.eval_full(&[Frac::from_i64(v)]);
            assert!(!val.is_zero(), "值域外點 {} 不應歸零", v);
        }
    }

    #[test]
    fn vanishing_coefficients_stay_small() {
        // ∏_{0..4}(x−v) 的係數為初等對稱函數，最大 |c| = 50（遠 < 2¹⁶）
        let vp = vanishing_poly(0, &[0, 1, 2, 3, 4], 1);
        let maxc = vp.terms.iter().map(|(_, c)| coeff_abs_repr(c)).max().unwrap();
        assert!(maxc < (1 << 16), "小值域消失多項式的係數必須仍在 L0′ 界內: {}", maxc);
        // 次數 = |domain|
        assert_eq!(vp.terms.iter().map(|(m, _)| m.iter().sum::<u32>()).max(), Some(5));
    }

    #[test]
    fn l0_prime_recovers_original_l0() {
        // 原 L0：T < 2¹²、C < 2¹⁶、V = 1、d ≤ 6 ⇒ 界 2²⁸ < p
        let params = L0PrimeParams {
            n_terms: 1 << 12,
            max_abs_coeff: 1 << 16,
            max_abs_value: 1,
            degree: 6,
        };
        assert_eq!(params.eval_bound(), Some(1u128 << 28));
        assert!(params.faithful());
    }

    #[test]
    fn l0_prime_multivalued_still_faithful() {
        // 多值編碼實例：值域 {0,1,2,3}（V=3）、次數 6、項數 2¹²、係數 2¹⁶
        // 界 = 2²⁸ · 3⁶ ≈ 1.86e11，仍然 < 2⁶¹−1
        let params = L0PrimeParams {
            n_terms: 1 << 12,
            max_abs_coeff: 1 << 16,
            max_abs_value: 3,
            degree: 6,
        };
        let b = params.eval_bound().unwrap();
        assert_eq!(b, (1u128 << 28) * 729);
        assert!(params.faithful());
    }

    #[test]
    fn l0_prime_detects_unfaithful_encoding() {
        // V = 10⁶、d = 6 ⇒ V^d = 10³⁶ ≫ p：必須判為不保真（拒絕此編碼）
        let params = L0PrimeParams {
            n_terms: 1,
            max_abs_coeff: 1,
            max_abs_value: 1_000_000,
            degree: 6,
        };
        assert!(!params.faithful(), "超大值域會破壞嵌入保真，必須被標記");
    }

    #[test]
    fn l0_prime_params_from_poly() {
        let vp = vanishing_poly(0, &[0, 1, 2, 3], 1);
        let params = l0_prime_params_of(&vp, 3, 10);
        assert_eq!(params.n_terms, 4); // x⁴ −6x³ +11x² −6x
        assert!(params.max_abs_coeff >= 11); // 11 = e₂({1,2,3})
        assert!(params.faithful());
    }

    #[test]
    fn vanishing_beats_one_hot_bound() {
        for k in [3usize, 4, 8, 16] {
            let c = compare_encodings(k);
            let oh = c.one_hot_bound.unwrap();
            assert!(
                c.vanishing_bound < oh,
                "k={}: 消失多項式界 {} 必須小於 one-hot 界 {}",
                k,
                c.vanishing_bound,
                oh
            );
            // 界收緊倍數 = 2^k / k ≥ 2（k ≥ 2）
            assert!(oh / c.vanishing_bound >= 2);
        }
    }

    #[test]
    fn product_bound_specializes_to_2n() {
        // kᵢ ≡ 2 ⇒ ∏ kᵢ = 2ⁿ（與 T4 一致）
        let ks = vec![2usize; 8];
        assert_eq!(standard_bound_product(&ks), Some(256));
        // 混合值域：3·4·5 = 60 ≪ 2^(3+4+5) = 4096
        let mixed = vec![3usize, 4, 5];
        let prod = standard_bound_product(&mixed).unwrap();
        assert_eq!(prod, 60);
        assert!(prod < (1u128 << 12));
    }
}
