// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! 認證路徑（Certification path）：rustc／外部 oracle 負責判決，
//! polyrust 負責**重建見證 σ 並以多項式時間直接求值驗證**。
//!
//! 動機（見 docs/THEOREMS.md §6b T10 工程面）：過渡到真 rustc 後，把指數級
//! 的「搜索」（Buchberger/CDCL 求解）反轉為多項式級的「檢查」——
//! 由外部判決產物（推斷型別、region 圖、Chalk 代換）重建位元賦值 σ，
//! 然後對全部生成元＋域多項式逐個求值。驗證複雜度 = O(系統總項數)，
//! 與 2ⁿ 完全無關；這是「2ⁿ 從束縛退化為摸不到的天花板」的執行層體現。
//!
//! 與義務自證（obligations）的關係：T1 義務驗證的是「已知合法推導 D ⇒
//! σ_D 歸零」；本模組驗證的是「**任意外部來源**的位元賦值 ⇒ 是否歸零」，
//! 來源可以是管線自身（回歸測試）、rustc 的產物、或人工指定的註解。
//! 來源不合法時回傳 `certified = false` 與首個失敗位置，絕不拋錯。

use crate::frac::Frac;
use crate::groebner::field_polys;
use crate::poly::Poly;

/// 外部判決的位元賦值（部分或完整）。
/// `bits[i] = None` 表示 oracle 未指定該位，由 [`complete`] 按 one-hot 組補全。
#[derive(Clone, Debug, Default)]
pub struct OracleBits {
    pub bits: Vec<Option<bool>>,
    /// one-hot 組：每組內恰一位必為 1（型別位元、臂位元等）。
    /// 組內若一位都未指定 ⇒ 補第一個缺失位為 1；若 ≥2 位為 1 ⇒ 無效輸入。
    pub one_hot_groups: Vec<Vec<usize>>,
}

/// 認證報告。
#[derive(Clone, Debug, Default)]
pub struct CertReport {
    /// 全部多項式在 σ 下歸零（𝔽_p 上；由引理 L0 與整數求值一致）。
    pub certified: bool,
    /// 求值檢查的多項式總數（生成元＋域多項式）。
    pub checked: usize,
    /// 首個未歸零的多項式索引與其在 σ 下的值（None = 全部通過）。
    pub first_bad: Option<(usize, Frac)>,
    /// 由 one-hot 補全機制填上的位元數。
    pub completed_bits: usize,
    /// oracle 輸入本身無效（如 one-hot 組內 ≥2 個 1、索引越界）。
    pub oracle_invalid: Option<String>,
}

/// 補全部分賦值：缺失位先設 0，再對每個 one-hot 組強制恰一位為 1。
/// 回傳（完整賦值, 補全的位數）或錯誤說明。
pub fn complete(oracle: &OracleBits, nvars: usize) -> Result<(Vec<Frac>, usize), String> {
    if oracle.bits.len() != nvars {
        return Err(format!(
            "oracle 位數 {} ≠ 系統變量數 {}",
            oracle.bits.len(),
            nvars
        ));
    }
    let mut sigma: Vec<Frac> = oracle
        .bits
        .iter()
        .map(|b| if b == &Some(true) { Frac::ONE } else { Frac::ZERO })
        .collect();
    let mut completed = 0usize;
    for b in oracle.bits.iter() {
        if b.is_none() {
            completed += 1;
        }
    }
    for g in &oracle.one_hot_groups {
        let mut ones = 0usize;
        let mut first_unset: Option<usize> = None;
        for &i in g {
            if i >= nvars {
                return Err(format!("one-hot 組含越界變量 {}", i));
            }
            match oracle.bits[i] {
                Some(true) => ones += 1,
                Some(false) => {}
                None => {
                    if first_unset.is_none() {
                        first_unset = Some(i);
                    }
                }
            }
        }
        match ones {
            0 => {
                // 組內全未指定或全 0：補第一個未指定位為 1
                if let Some(i) = first_unset {
                    sigma[i] = Frac::ONE;
                } else {
                    return Err(format!("one-hot 組 {:?} 全部為 0，無法補全", g));
                }
            }
            1 => {}
            k => return Err(format!("one-hot 組 {:?} 有 {} 個 1，輸入無效", g, k)),
        }
    }
    Ok((sigma, completed))
}

/// 核心驗證：對 `polys` 逐個在 σ 上直接求值，全部歸零才算通過。
/// 多項式時間（O(總項數)），不做任何求解。
pub fn certify_polys(polys: &[Poly], sigma: &[Frac]) -> CertReport {
    let mut rep = CertReport::default();
    rep.checked = polys.len();
    for (i, p) in polys.iter().enumerate() {
        let v = p.eval_full(sigma);
        if !v.is_zero() {
            rep.first_bad = Some((i, v));
            return rep;
        }
    }
    rep.certified = true;
    rep
}

/// 認證一個完整賦值對「生成元 ∪ 域多項式」的滿足性。
pub fn certify_system(gens: &[Poly], nvars: usize, sigma: &[Frac]) -> CertReport {
    if sigma.len() != nvars {
        let mut r = CertReport::default();
        r.oracle_invalid = Some(format!("σ 長度 {} ≠ 變量數 {}", sigma.len(), nvars));
        return r;
    }
    // 0/1 合法性：任何非布爾值直接拒絕（這是「竄改見證」測試的正式入口）
    for (i, v) in sigma.iter().enumerate() {
        if !v.is_zero() && !v.is_one() {
            let mut r = CertReport::default();
            r.oracle_invalid = Some(format!("變量 {} 賦值非布爾: {}", i, v));
            return r;
        }
    }
    let mut all: Vec<Poly> = gens.to_vec();
    all.extend(field_polys(nvars));
    certify_polys(&all, sigma)
}

/// 一站式：部分 oracle → 補全 → 驗證。
pub fn certify_from_oracle(gens: &[Poly], nvars: usize, oracle: &OracleBits) -> CertReport {
    let mut rep = CertReport::default();
    match complete(oracle, nvars) {
        Ok((sigma, completed)) => {
            rep = certify_system(gens, nvars, &sigma);
            rep.completed_bits = completed;
            rep
        }
        Err(e) => {
            rep.oracle_invalid = Some(e);
            rep
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groebner::solve_boolean;

    fn x(i: usize, n: usize) -> Poly {
        Poly::var(i, Frac::ONE, n)
    }

    /// 小型系統：one-hot 組 {0,1}（型別位），x2 := x0（規則方程），
    /// 子句 (¬x2 ∨ x3) ⇒ (1−x2)·x3 補形：P_C = (1−(1−x2))·(1−x3) = x2·(1−x3)。
    fn mini_system() -> (Vec<Poly>, usize, Vec<Vec<usize>>) {
        let n = 4;
        let gens = vec![
            x(0, n).add(&x(1, n)).sub(&Poly::constant(Frac::ONE)), // one-hot Σ = 1
            x(2, n).sub(&x(0, n)),                                  // 規則方程
            x(2, n).mul(&Poly::constant(Frac::ONE).sub(&x(3, n))),  // 子句多項式
        ];
        (gens, n, vec![vec![0, 1]])
    }

    #[test]
    fn certify_full_witness_from_solver() {
        let (gens, n, _groups) = mini_system();
        let mut sys: Vec<Poly> = gens.clone();
        sys.extend(field_polys(n));
        let sigma = solve_boolean(&sys, n).expect("小型系統應可解");
        let rep = certify_system(&gens, n, &sigma);
        assert!(rep.certified, "求解器產出的 σ 必須通過獨立求值驗證: {:?}", rep);
        assert_eq!(rep.checked, gens.len() + n);
    }

    #[test]
    fn certify_partial_oracle_completed_by_one_hot() {
        let (gens, n, groups) = mini_system();
        // oracle 指定 x0 = x2 = x3 = 1，唯獨 x1 缺失 —— one-hot 組 {0,1} 已有
        // x0 = 1 滿足，x1 補 0；規則方程與子句由 oracle 直接滿足
        let oracle = OracleBits {
            bits: vec![Some(true), None, Some(true), Some(true)],
            one_hot_groups: groups,
        };
        let rep = certify_from_oracle(&gens, n, &oracle);
        assert!(rep.certified, "部分 oracle 補全後應通過: {:?}", rep);
        assert_eq!(rep.completed_bits, 1);
    }

    #[test]
    fn certify_one_hot_group_fully_unset_gets_default() {
        let (gens, n, groups) = mini_system();
        // one-hot 組 {0,1} 全缺 ⇒ 補 x0 = 1（組內第一個缺失位）
        let oracle = OracleBits {
            bits: vec![None; n],
            one_hot_groups: groups,
        };
        let (sigma, completed) = complete(&oracle, n).unwrap();
        assert_eq!(sigma[0], Frac::ONE);
        assert_eq!(sigma[1], Frac::ZERO);
        assert_eq!(completed, n);
        // 補全是機械填充、不是求解器：規則方程 x2 = x0 仍要求 x2 = 1，
        // 故認證會在規則方程（索引 1）處如實回報失敗——驗證不做修復
        let rep = certify_from_oracle(&gens, n, &oracle);
        assert!(!rep.certified);
        assert_eq!(rep.first_bad.as_ref().map(|(i, _)| *i), Some(1));
    }

    #[test]
    fn certify_rejects_tampered_witness() {
        let (gens, n, _groups) = mini_system();
        let mut sys: Vec<Poly> = gens.clone();
        sys.extend(field_polys(n));
        let mut sigma = solve_boolean(&sys, n).unwrap();
        // 竄改：翻轉 x0（破壞 one-hot 與規則方程）
        sigma[0] = if sigma[0].is_one() { Frac::ZERO } else { Frac::ONE };
        let rep = certify_system(&gens, n, &sigma);
        assert!(!rep.certified, "竄改見證必須被拒絕");
        assert!(rep.first_bad.is_some(), "應回報首個失敗約束");
    }

    #[test]
    fn certify_rejects_non_boolean_value() {
        let (gens, n, _groups) = mini_system();
        let mut sigma = vec![Frac::ZERO; n];
        sigma[2] = Frac::from_i64(2); // 位元 1→2 式竄改
        let rep = certify_system(&gens, n, &sigma);
        assert!(!rep.certified);
        assert!(rep.oracle_invalid.is_some(), "非布爾賦值應標記為無效輸入");
    }

    #[test]
    fn certify_rejects_invalid_one_hot_oracle() {
        let (gens, n, groups) = mini_system();
        // 同一 one-hot 組內兩個 1 ⇒ 輸入本身無效
        let oracle = OracleBits {
            bits: vec![Some(true), Some(true), None, None],
            one_hot_groups: groups,
        };
        let rep = certify_from_oracle(&gens, n, &oracle);
        assert!(!rep.certified);
        assert!(rep.oracle_invalid.is_some());
    }

    #[test]
    fn certify_rejects_wrong_oracle_with_full_bits() {
        let (gens, n, groups) = mini_system();
        // x1 = 1（one-hot 選另一臂）⇒ x2 應 = x0 = 0，但 oracle 堅持 x2 = 1
        let oracle = OracleBits {
            bits: vec![Some(false), Some(true), Some(true), Some(true)],
            one_hot_groups: groups,
        };
        let rep = certify_from_oracle(&gens, n, &oracle);
        assert!(!rep.certified, "與規則方程矛盾的完整賦值必須被拒絕");
    }
}
