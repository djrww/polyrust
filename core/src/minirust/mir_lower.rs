// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! MIR 前端（第三條 lowering）：迷你 MIR → 消失多項式約束系統 + L0′ 檢查。
//!
//! # 定位（見 docs/THEOREMS.md §6b）
//! lowering 鏈：AST → `constraints.rs`（System v1）→ `lower.rs` +
//! `constraints_v2.rs`（SystemV2）→ **`mir_lower.rs`（本模組，MIR 級）**。
//! 對齊 rustc 的組件邊界：每個 MIR `Body` 一個系統；局部位槽按數據流
//! 構造即天然分組件（兩條獨立賦值鏈 ⇒ 兩個組件）。
//!
//! # 編碼
//! * 每個 local 一個 𝔽_p 變量（**不是** one-hot 位元串）；
//! * 值域由**抽象解釋**靜態推導（常數流傳播，Add/Sub/Mul 集合運算，
//!   集合大小 ≤ 64 封頂）；
//! * 值域約束用**消失多項式** ∏_{v∈D}(x − v)：值域大小 k 的變量貢獻
//!   標準單項式界 k，而非 one-hot 的 2^k（`compare_encodings` 對照）；
//! * 算術恆等式 x_p − x_a − x_b = 0（𝔽_p 上；所有值 < p ⟺ 整數語義）。
//!
//! # L0′ 門檻
//! 生成後立即計算 |f(σ)| ≤ T·C·V^d 並檢查 < p；不保真的編碼（大值域 ×
//! 高次冪鏈）直接 **Err 拒絕**——嵌入引理是硬前提，不是事後補票。
//!
//! # 求解與認證
//! 系統變量集天然分組件 ⇒ [`crate::composition::run_decomposed`] 分組求解；
//! 值域笛卡爾積回填 σ（小型值域，≤ 10⁶ 組合封頂）；最後以
//! [`crate::certify::certify_polys`] + 值域成員檢查做獨立認證。

use crate::certify::CertReport;
use crate::composition::{run_decomposed_mode, DomainMode};
use crate::frac::Frac;
use crate::groebner::Strategy;
use crate::poly::Order;
use crate::poly::Poly;
use crate::vanishing::{l0_prime_params_of, vanishing_poly, L0PrimeParams};

/// 迷你 MIR 右值。
#[derive(Clone, Debug)]
pub enum Rv {
    /// 常數。
    Use(i64),
    /// x_a + x_b
    Add(usize, usize),
    /// x_a − x_b
    Sub(usize, usize),
    /// x_a · x_b
    Mul(usize, usize),
}

/// 迷你 MIR 語句：SSA 式（每個 local 恰賦值一次，按語句順序定義先於使用）。
#[derive(Clone, Debug)]
pub struct MirStmt {
    pub place: usize,
    pub rv: Rv,
}

/// 迷你 MIR 函數體。
#[derive(Clone, Debug)]
pub struct MirBody {
    pub name: String,
    pub n_locals: usize,
    pub stmts: Vec<MirStmt>,
}

impl MirBody {
    pub fn new(name: &str, n_locals: usize) -> MirBody {
        MirBody { name: name.into(), n_locals, stmts: Vec::new() }
    }
    pub fn assign(&mut self, place: usize, rv: Rv) -> &mut Self {
        self.stmts.push(MirStmt { place, rv });
        self
    }
}

/// 值域推導錯誤。
#[derive(Clone, Debug, PartialEq)]
pub enum LowerError {
    /// 抽象值域集合超過封頂（拒絕——改用 bool 位元串編碼屬後續工作）。
    DomainCap { local: usize, size: usize },
    /// 使用未定義的 local。
    UndefinedLocal { local: usize, at_stmt: usize },
    /// L0′ 判定不保真（|f(σ)| 界 ≥ p）。
    UnfaithfulEmbedding { bound: u128, p: u64 },
}

const DOMAIN_CAP: usize = 64;
const PRODUCT_CAP: u64 = 1_000_000;

/// 下降結果：系統 + 元資料。
#[derive(Clone, Debug)]
pub struct MirSystem {
    pub body: String,
    pub nvars: usize,
    pub polys: Vec<Poly>,
    /// 各 local 的靜態值域（與變量一一對應）。
    pub domains: Vec<Vec<i64>>,
    /// L0′ 參數（整個系統的保守界：最大項數 × 最大係數 × V^最大次數）。
    pub l0: L0PrimeParams,
}

/// 值域集合運算（小集合直積，封頂拒絕）。
fn combine(a: &[i64], b: &[i64], f: impl Fn(i64, i64) -> i64) -> Result<Vec<i64>, (usize, usize)> {
    let mut out: Vec<i64> = Vec::with_capacity(a.len() * b.len());
    for &x in a {
        for &y in b {
            out.push(f(x, y));
        }
    }
    out.sort_unstable();
    out.dedup();
    if out.len() > DOMAIN_CAP {
        return Err((a.len(), b.len()));
    }
    Ok(out)
}

/// 抽象解釋：由常數流推導每個 local 的靜態值域。
fn infer_domains(body: &MirBody) -> Result<Vec<Vec<i64>>, LowerError> {
    let mut domains: Vec<Vec<i64>> = vec![Vec::new(); body.n_locals];
    let mut defined = vec![false; body.n_locals];
    for (k, s) in body.stmts.iter().enumerate() {
        let need = match &s.rv {
            Rv::Use(_) => vec![],
            Rv::Add(a, b) | Rv::Sub(a, b) | Rv::Mul(a, b) => vec![*a, *b],
        };
        for &d in &need {
            if d >= body.n_locals || !defined[d] {
                return Err(LowerError::UndefinedLocal { local: d, at_stmt: k });
            }
        }
        if s.place >= body.n_locals {
            return Err(LowerError::UndefinedLocal { local: s.place, at_stmt: k });
        }
        let dom = match &s.rv {
            Rv::Use(v) => vec![*v],
            Rv::Add(a, b) => combine(&domains[*a], &domains[*b], |x, y| x + y)
                .map_err(|(sa, sb)| LowerError::DomainCap { local: s.place, size: sa * sb })?,
            Rv::Sub(a, b) => combine(&domains[*a], &domains[*b], |x, y| x - y)
                .map_err(|(sa, sb)| LowerError::DomainCap { local: s.place, size: sa * sb })?,
            Rv::Mul(a, b) => combine(&domains[*a], &domains[*b], |x, y| x * y)
                .map_err(|(sa, sb)| LowerError::DomainCap { local: s.place, size: sa * sb })?,
        };
        domains[s.place] = dom;
        defined[s.place] = true;
    }
    Ok(domains)
}

/// MIR → 約束系統（值域消失多項式 + 算術恆等式），含 L0′ 門檻。
pub fn lower(body: &MirBody) -> Result<MirSystem, LowerError> {
    let domains = infer_domains(body)?;
    let n = body.n_locals;
    let mut polys: Vec<Poly> = Vec::new();
    // 值域約束：每個 local 的消失多項式（域空值域 = 未賦值 ⇒ 恆零多項式已由 from_terms 正規化）
    for (i, d) in domains.iter().enumerate() {
        if d.is_empty() {
            continue; // 未賦值 local：無約束（definedness 由調用方另行保證）
        }
        polys.push(vanishing_poly(i, d, n));
    }
    // 算術恆等式
    for s in &body.stmts {
        let p = Poly::var(s.place, Frac::ONE, n);
        let eq = match &s.rv {
            Rv::Use(v) => p.sub(&Poly::constant(Frac::from_i64(*v))),
            Rv::Add(a, b) => {
                p.sub(&Poly::var(*a, Frac::ONE, n)).sub(&Poly::var(*b, Frac::ONE, n))
            }
            Rv::Sub(a, b) => {
                p.sub(&Poly::var(*a, Frac::ONE, n)).add(&Poly::var(*b, Frac::ONE, n))
            }
            Rv::Mul(a, b) => {
                p.sub(&Poly::var(*a, Frac::ONE, n).mul(&Poly::var(*b, Frac::ONE, n)))
            }
        };
        polys.push(eq);
    }
    // L0′ 門檻：V = 最大 |值域值|，次數/項數/係數取全系統實測最大
    let v_max = domains
        .iter()
        .flatten()
        .map(|v| v.unsigned_abs())
        .max()
        .unwrap_or(0);
    let mut l0 = L0PrimeParams {
        n_terms: 0,
        max_abs_coeff: 0,
        max_abs_value: v_max,
        degree: 0,
    };
    for p in &polys {
        let lp = l0_prime_params_of(p, v_max, u32::MAX);
        l0.n_terms = l0.n_terms.max(lp.n_terms);
        l0.max_abs_coeff = l0.max_abs_coeff.max(lp.max_abs_coeff);
        l0.degree = l0.degree.max(lp.degree);
    }
    if !l0.faithful() {
        return Err(LowerError::UnfaithfulEmbedding {
            bound: l0.eval_bound().unwrap_or(u128::MAX),
            p: crate::fp::P,
        });
    }
    Ok(MirSystem { body: body.name.clone(), nvars: n, polys, domains, l0 })
}

/// 值域笛卡爾積求解（小型值域；組合數 ≤ PRODUCT_CAP）。
pub fn solve_domains(sys: &MirSystem) -> Option<Vec<Frac>> {
    let mut total: u64 = 1;
    for d in &sys.domains {
        total = total.checked_mul(d.len().max(1) as u64)?;
        if total > PRODUCT_CAP {
            return None;
        }
    }
    let mut sigma: Vec<Frac> = sys.domains.iter().map(|d| Frac::from_i64(d.first().copied().unwrap_or(0))).collect();
    fn rec(
        polys: &[Poly],
        domains: &[Vec<i64>],
        i: usize,
        sigma: &mut Vec<Frac>,
    ) -> Option<()> {
        if i == domains.len() {
            return if polys.iter().all(|p| p.eval_full(sigma).is_zero()) {
                Some(())
            } else {
                None
            };
        }
        if domains[i].is_empty() {
            return rec(polys, domains, i + 1, sigma);
        }
        for &v in &domains[i] {
            sigma[i] = Frac::from_i64(v);
            if rec(polys, domains, i + 1, sigma).is_some() {
                return Some(());
            }
        }
        None
    }
    rec(&sys.polys, &sys.domains, 0, &mut sigma)?;
    Some(sigma)
}

/// 獨立認證：值域成員 + 全多項式直接求值（多項式時間；與 2ⁿ 無關）。
pub fn certify_mir(sys: &MirSystem, sigma: &[Frac]) -> CertReport {
    if sigma.len() != sys.nvars {
        return CertReport::default();
    }
    for (i, v) in sigma.iter().enumerate() {
        if sys.domains[i].is_empty() {
            continue; // 未賦值 local：無值域約束
        }
        let vi = v.0 as i128;
        // 𝔽_p 表示 → 最小絕對值代表 → 必須是值域成員
        let signed = if (v.0 as u64) > crate::fp::P / 2 {
            vi - crate::fp::P as i128
        } else {
            vi
        };
        if !sys.domains[i].iter().any(|&d| (d as i128) == signed) {
            return CertReport::default();
        }
    }
    crate::certify::certify_polys(&sys.polys, sigma)
}

/// 分組求解並回報（T10 組合路徑；`Provided` 模式——域約束是系統內的
/// 消失多項式，不可再加布爾域 x²−x，否則多值變量會被誤判 UNSAT）。
/// 組合界為 ∏kᵢ（各 local 值域大小之積），可用
/// [`crate::vanishing::standard_bound_product`] 計算。
pub fn solve_decomposed(sys: &MirSystem) -> crate::composition::DecompReport {
    run_decomposed_mode(&sys.polys, sys.nvars, Order::GrevLex, Strategy::Normal, DomainMode::Provided)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::components_of;
    use crate::vanishing::compare_encodings;

    fn frac_i(v: i64) -> Frac {
        Frac::from_i64(v)
    }

    #[test]
    fn mir_chain_lower_solve_certify() {
        // x0 = 2; x1 = x0 + x0; x2 = x1 − 1   ⇒ (2, 4, 3)
        let mut b = MirBody::new("chain", 3);
        b.assign(0, Rv::Use(2)).assign(1, Rv::Add(0, 0)).assign(2, Rv::Sub(1, 0));
        let sys = lower(&b).expect("下降失敗");
        assert_eq!(sys.domains, vec![vec![2], vec![4], vec![2]]);
        assert!(sys.l0.faithful());
        let sigma = solve_domains(&sys).expect("可解");
        assert_eq!(sigma, vec![frac_i(2), frac_i(4), frac_i(2)]);
        let rep = certify_mir(&sys, &sigma);
        assert!(rep.certified, "MIR 見證必須通過認證: {:?}", rep);
    }

    #[test]
    fn mir_independent_chains_decompose() {
        // 鏈 A：x0=1, x1=x0+x0；鏈 B：x2=5, x3=x2·x2 —— 數據流天然分組件
        let mut b = MirBody::new("two-chains", 4);
        b.assign(0, Rv::Use(1))
            .assign(1, Rv::Add(0, 0))
            .assign(2, Rv::Use(5))
            .assign(3, Rv::Mul(2, 2));
        let sys = lower(&b).expect("下降失敗");
        let (comps, unused) = components_of(&sys.polys, sys.nvars);
        assert_eq!(comps.len(), 2, "兩條獨立鏈應分為 2 組件");
        assert_eq!(unused, 0);
        let rep = solve_decomposed(&sys);
        assert!(rep.union_basis_is_gb, "並基必須是 Gröbner 基");
        assert!(!rep.any_unsat);
        let sigma = solve_domains(&sys).expect("可解");
        let cert = certify_mir(&sys, &sigma);
        assert!(cert.certified);
    }

    #[test]
    fn mir_branchy_domain_vanishing_beats_one_hot() {
        // 模擬分支合流：x1 ∈ {2,3}（經中間鏈擴張）；值域多值 ⇒ 消失多項式編碼
        // x0 = 2; x1 = x0·x0; x2 = x0 + x1  ⇒ x2 ∈ {6}——此例域仍單點；
        // 改以顯式兩值常數鏈檢查編碼對照：值域 {0..5} 時 1 變量（界 5）優於 5 位元（界 32）
        let c = compare_encodings(5);
        assert_eq!(c.one_hot_bits, 5);
        assert_eq!(c.vanishing_vars, 1);
        assert_eq!(c.vanishing_bound, 5);
        assert!(c.vanishing_bound < c.one_hot_bound.unwrap());

        // 多值值域實例：Add 合流（x2 = x0 + x1，x0 ∈ {0,1}，x1 ∈ {0,1} ⇒ x2 ∈ {0,1,2}）
        // 直接用域狀態模擬：構造僅含值域約束 + 恆等式 z = a + b 的系統
        let n = 3;
        let mut polys = Vec::new();
        polys.push(vanishing_poly(0, &[0, 1], n));
        polys.push(vanishing_poly(1, &[0, 1], n));
        polys.push(vanishing_poly(2, &[0, 1, 2], n));
        polys.push(
            Poly::var(2, Frac::ONE, n)
                .sub(&Poly::var(0, Frac::ONE, n))
                .sub(&Poly::var(1, Frac::ONE, n)),
        );
        let sys = MirSystem {
            body: "adds".into(),
            nvars: n,
            polys: polys.clone(),
            domains: vec![vec![0, 1], vec![0, 1], vec![0, 1, 2]],
            l0: L0PrimeParams { n_terms: 4, max_abs_coeff: 3, max_abs_value: 2, degree: 3 },
        };
        assert!(sys.l0.faithful());
        let sigma = solve_domains(&sys).expect("可解");
        let a = sigma[0].0 as i64;
        let b = sigma[1].0 as i64;
        assert_eq!(sigma[2], frac_i(a + b), "恆等式必須成立");
        let cert = certify_mir(&sys, &sigma);
        assert!(cert.certified);
        // 竄改：x2 設成值域內但恆等式錯的值
        let mut bad = sigma.clone();
        bad[2] = frac_i(if a + b == 0 { 1 } else { 0 });
        if bad[2] != sigma[2] {
            assert!(!certify_mir(&sys, &bad).certified, "竄改必須被拒");
        }
        let _ = polys; // 已用
    }

    #[test]
    fn mir_rejects_use_before_def_and_cap() {
        let mut b = MirBody::new("bad-order", 2);
        b.assign(0, Rv::Add(1, 1)).assign(1, Rv::Use(1));
        match lower(&b) {
            Err(LowerError::UndefinedLocal { local: 1, .. }) => {}
            other => panic!("應報未定義使用，得到 {:?}", other.map(|_| ())),
        }
        // 值域爆炸：14 層平方鏈（每層集合大小 ×2 遠超封頂）
        let mut b2 = MirBody::new("boom", 15);
        b2.assign(0, Rv::Use(0));
        for i in 1..15 {
            b2.assign(i, Rv::Mul(i - 1, i - 1));
        }
        // x0=0 使每層域仍是單點 {0}——不爆。用 x0 ∈ {1,2} 兩值鏈：
        // 實測以 Add 鏈爆集合：x1=x0+x0（≤4 元素）… 直接構造多值直積封頂：
        let mut b3 = MirBody::new("cap", 11);
        b3.assign(0, Rv::Use(0));
        b3.assign(1, Rv::Add(0, 0));
        for i in 2..11 {
            b3.assign(i, Rv::Add(i - 1, i - 1));
        }
        // 封頂路徑以直積上限語義測：兩個 100 元素集合相加 → 199 個不同和 > 64 ⇒ 拒絕
        let big: Vec<i64> = (0..100).collect();
        assert!(combine(&big, &big, |x, y| x + y).is_err());
        // 單點 Add 鏈不爆集合、仍保真可下降
        assert!(lower(&b3).is_ok());
        let _ = b2;
    }

    #[test]
    fn mir_unfaithful_embedding_rejected() {
        // 40000 → 1.6e9 → 2.56e18（i64 安全）：V = 2.56e18、d = 2 ⇒
        // 界 ≈ 3·(2.56e18)² ≈ 2e37 ≫ p ⇒ L0′ 拒絕此編碼
        let mut b = MirBody::new("pow-chain", 3);
        b.assign(0, Rv::Use(40000)).assign(1, Rv::Mul(0, 0)).assign(2, Rv::Mul(1, 1));
        match lower(&b) {
            Err(LowerError::UnfaithfulEmbedding { bound, p }) => {
                assert!(bound >= p as u128, "界 {} 應 ≥ p {}", bound, p);
            }
            other => panic!("應報嵌入不保真，得到 {:?}", other.map(|s| s.l0)),
        }
    }

    #[test]
    fn mir_small_power_chain_still_faithful() {
        // 平方鏈 2→4→16→256→65536：V = 65536、d = 2 ⇒ 界 ≈ 3·2³² < p ⇒ 可下降
        let mut b = MirBody::new("pow", 5);
        b.assign(0, Rv::Use(2));
        for i in 1..5 {
            b.assign(i, Rv::Mul(i - 1, i - 1));
        }
        let sys = lower(&b).expect("下降失敗");
        assert_eq!(sys.domains[4], vec![65536]);
        assert!(sys.l0.faithful());
        let sigma = solve_domains(&sys).unwrap();
        assert_eq!(sigma[4], frac_i(65536));
        assert!(certify_mir(&sys, &sigma).certified);
    }

    #[test]
    fn mir_undefined_local_rejected() {
        let mut b = MirBody::new("undef", 3);
        // Add 讀取未賦值的 local（0、1 均未賦值，報掃描到的第一個）⇒ 拒絕
        b.assign(0, Rv::Add(0, 1));
        match lower(&b) {
            Err(LowerError::UndefinedLocal { local: 0, .. }) => {}
            other => panic!("應報未定義，得到 {:?}", other.map(|_| ())),
        }
        // Use(1) 是常數 1：完全合法；local 1 為未賦值（空值域，求解/認證跳過）
        let mut b2 = MirBody::new("useconst", 2);
        b2.assign(0, Rv::Use(1));
        let sys = lower(&b2).expect("Use 常數應合法");
        assert_eq!(sys.domains[0], vec![1]);
        assert!(sys.domains[1].is_empty());
        let sigma = solve_domains(&sys).unwrap();
        assert!(certify_mir(&sys, &sigma).certified);
    }

    #[test]
    fn mir_negative_values_ok() {
        // x0 = 5; x1 = 0; x2 = x1 − x0 ⇒ −5；x3 = x2 + x0 ⇒ 0
        // （負值域經 rem_euclid 入 𝔽_p；認證用最小絕對值代表還原）
        let mut b = MirBody::new("neg", 4);
        b.assign(0, Rv::Use(5))
            .assign(1, Rv::Use(0))
            .assign(2, Rv::Sub(1, 0))
            .assign(3, Rv::Add(2, 0));
        let sys = lower(&b).expect("下降失敗");
        assert_eq!(sys.domains[2], vec![-5]);
        let sigma = solve_domains(&sys).unwrap();
        assert_eq!(sigma[2], frac_i(-5));
        assert_eq!(sigma[3], frac_i(0));
        assert!(certify_mir(&sys, &sigma).certified);
    }
}
