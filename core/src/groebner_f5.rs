// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! F5 演算法設計與原型：簽名準則避免零歸約
//! Faugère 2002，針對 polyrust 布爾系統增量求解優化
//!
//! 簽名：sig(f) = (i, m) 表示 f = Σ h_j f_j 且首項來自 f_i * m
//! 準則：
//!   F5 Criterion: 若 m 被 G 中更小簽名的多項式首項整除且簽名更大，則跳過
//!   Rewritten Criterion: 同簽名更小多項式已處理則跳過
//!
//! 與 F4 結合：F5 選配對 + F4 矩陣歸約 = F4/5 (msolve, FGb 標準)

use crate::frac::Frac;
use crate::poly::{cmp_mono, mono_div, mono_divides, mono_lcm, spoly, Mono, Order, Poly};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default)]
pub struct F5Stats {
    pub generators: usize,
    pub pairs_considered: usize,
    pub f5_criterion_skips: usize,
    pub rewritten_skips: usize,
    pub crit1_skips: usize,
    pub crit2_skips: usize,
    pub s_polys: usize,
    pub reductions_to_zero: usize,
    pub basis_adds: usize,
    pub basis_final: usize,
}

/// 簽名多項式
#[derive(Clone, Debug)]
pub struct SignedPoly {
    /// 簽名：(輸入索引, 單項式)
    pub sig: (usize, Mono),
    /// 多項式本身
    pub poly: Poly,
    /// 輸入位置 (用於增量順序)
    pub index: usize,
}

impl SignedPoly {
    fn new(sig: (usize, Mono), poly: Poly, index: usize) -> Self {
        Self { sig, poly, index }
    }
}

/// 簽名比較：先比輸入索引，再比單項式序
fn sig_cmp(a: &(usize, Mono), b: &(usize, Mono), ord: Order) -> std::cmp::Ordering {
    a.0.cmp(&b.0)
        .then_with(|| cmp_mono(&a.1, &b.1, ord))
}

fn mono_mul_is_coprime(a: &Mono, b: &Mono) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (*x).min(*y) == 0)
}

fn pair_key(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

/// F5 準則：若 sig 的單項式部分可被 G 中更小簽名的多項式首項整除，則跳過
fn f5_criterion(
    sig: &(usize, Mono),
    g: &[SignedPoly],
    lms: &[Mono],
    ord: Order,
) -> bool {
    for (j, gj) in g.iter().enumerate() {
        if sig_cmp(&gj.sig, sig, ord) == std::cmp::Ordering::Greater {
            continue;
        }
        // 若 LM(gj) | sig.m 且 sig.index != gj.sig.0 (不同輸入源)
        if mono_divides(&lms[j], &sig.1) && sig.0 != gj.sig.0 {
            return true;
        }
    }
    false
}

/// Rewritten 準則：存在同簽名索引更小且單項式整除當前簽名的多項式
fn rewritten_criterion(
    sig: &(usize, Mono),
    g: &[SignedPoly],
    ord: Order,
) -> bool {
    for gj in g {
        if gj.sig.0 != sig.0 {
            continue;
        }
        if gj.sig.1 == sig.1 {
            continue;
        }
        if mono_divides(&gj.sig.1, &sig.1) && sig_cmp(&gj.sig, sig, ord) == std::cmp::Ordering::Less {
            return true;
        }
    }
    false
}

/// F5 主循環 (簡化版，單步歸約，非矩陣)
pub fn f5(fs: &[Poly], ord: Order) -> (Vec<Poly>, F5Stats) {
    let mut stats = F5Stats {
        generators: fs.len(),
        ..Default::default()
    };

    // 初始化簽名多項式：每個輸入 f_i 的簽名為 (i, 1)
    let mut g: Vec<SignedPoly> = Vec::new();
    for (i, f) in fs.iter().enumerate() {
        if f.is_zero() {
            continue;
        }
        let mut p = f.clone();
        p.make_monic(ord);
        let sig = (i, vec![0u32; p.terms.first().map(|(m, _)| m.len()).unwrap_or(0)]);
        g.push(SignedPoly::new(sig, p, i));
    }

    let mut lms: Vec<Mono> = g.iter().map(|sp| sp.poly.lm(ord).unwrap()).collect();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..g.len() {
        for j in (i + 1)..g.len() {
            pairs.push((i, j));
        }
    }
    let mut closed: BTreeSet<(usize, usize)> = BTreeSet::new();

    // 按簽名序處理配對 (增量)
    while !pairs.is_empty() {
        // 選簽名最小的配對
        pairs.sort_by(|&(i1, j1), &(i2, j2)| {
            let lcm1 = mono_lcm(&lms[i1], &lms[j1]);
            let lcm2 = mono_lcm(&lms[i2], &lms[j2]);
            // 簽名 = max(sig_i * (lcm/LM_i), sig_j * (lcm/LM_j))
            let sig1 = {
                let m1 = mono_div(&lcm1, &lms[i1]);
                let m2 = mono_div(&lcm1, &lms[j1]);
                let s1 = (g[i1].sig.0, crate::poly::mono_mul(&g[i1].sig.1, &m1));
                let s2 = (g[j1].sig.0, crate::poly::mono_mul(&g[j1].sig.1, &m2));
                if sig_cmp(&s1, &s2, ord) == std::cmp::Ordering::Greater {
                    s1
                } else {
                    s2
                }
            };
            let sig2 = {
                let m1 = mono_div(&lcm2, &lms[i2]);
                let m2 = mono_div(&lcm2, &lms[j2]);
                let s1 = (g[i2].sig.0, crate::poly::mono_mul(&g[i2].sig.1, &m1));
                let s2 = (g[j2].sig.0, crate::poly::mono_mul(&g[j2].sig.1, &m2));
                if sig_cmp(&s1, &s2, ord) == std::cmp::Ordering::Greater {
                    s1
                } else {
                    s2
                }
            };
            sig_cmp(&sig1, &sig2, ord)
        });

        let (i, j) = pairs.remove(0);
        stats.pairs_considered += 1;

        let lmi = lms[i].clone();
        let lmj = lms[j].clone();
        let lcm = mono_lcm(&lmi, &lmj);

        // Crit1
        if mono_mul_is_coprime(&lmi, &lmj) {
            stats.crit1_skips += 1;
            continue;
        }
        // Crit2
        let mut chain_skip = false;
        for k in 0..g.len() {
            if k == i || k == j {
                continue;
            }
            if mono_divides(&lms[k], &lcm)
                && closed.contains(&pair_key(i, k))
                && closed.contains(&pair_key(j, k))
            {
                chain_skip = true;
                break;
            }
        }
        if chain_skip {
            stats.crit2_skips += 1;
            continue;
        }

        // 計算 S-多項式的簽名
        let m_i = mono_div(&lcm, &lmi);
        let m_j = mono_div(&lcm, &lmj);
        let sig_i = (g[i].sig.0, crate::poly::mono_mul(&g[i].sig.1, &m_i));
        let sig_j = (g[j].sig.0, crate::poly::mono_mul(&g[j].sig.1, &m_j));
        let sig = if sig_cmp(&sig_i, &sig_j, ord) == std::cmp::Ordering::Greater {
            sig_i
        } else {
            sig_j
        };

        // F5 準則
        if f5_criterion(&sig, &g, &lms, ord) {
            stats.f5_criterion_skips += 1;
            continue;
        }
        if rewritten_criterion(&sig, &g, ord) {
            stats.rewritten_skips += 1;
            continue;
        }

        stats.s_polys += 1;
        let s = spoly(&g[i].poly, &g[j].poly, ord);
        // 簽名安全歸約：僅允許簽名不增的歸約
        let r = sig_safe_div_rem(&s, &g, &lms, &sig, ord);

        if r.is_zero() {
            stats.reductions_to_zero += 1;
            closed.insert(pair_key(i, j));
        } else {
            let mut r_monic = r;
            r_monic.make_monic(ord);
            let new_idx = g.len();
            for k in 0..new_idx {
                pairs.push((k, new_idx));
            }
            lms.push(r_monic.lm(ord).unwrap());
            g.push(SignedPoly::new(sig, r_monic, new_idx));
            stats.basis_adds += 1;
        }
    }

    let polys: Vec<Poly> = g.into_iter().map(|sp| sp.poly).collect();
    stats.basis_final = polys.len();
    (polys, stats)
}

/// 簽名安全除法：僅用簽名更小的除子
fn sig_safe_div_rem(
    f: &Poly,
    gs: &[SignedPoly],
    _lms: &[Mono],
    sig_f: &(usize, Mono),
    ord: Order,
) -> Poly {
    use std::collections::HashMap;
    // 預計算
    let mut divs: Vec<(crate::poly::Mono, Frac, &Poly, (usize, Mono))> = vec![];
    for sp in gs.iter() {
        if let Some((m, c)) = sp.poly.lt(ord) {
            if m.iter().all(|&e| e == 0) {
                return Poly::zero();
            }
            divs.push((crate::poly::Mono::from(m.clone()), c, &sp.poly, sp.sig.clone()));
        }
    }

    let mut terms: HashMap<Vec<(usize, u32)>, Frac> = f
        .terms
        .iter()
        .map(|(m, c)| {
            let sm: Vec<(usize, u32)> = m
                .iter()
                .enumerate()
                .filter(|(_, &e)| e > 0)
                .map(|(i, &e)| (i, e))
                .collect();
            (sm, *c)
        })
        .collect();

    let mut rem_terms: Vec<(Mono, Frac)> = vec![];
    let mut guard = 0usize;

    while !terms.is_empty() {
        guard += 1;
        assert!(guard < 2_000_000, "F5 除法超限");
        // 找最大單項式
        let (mk_sparse, mc) = {
            let mut best: Option<(&Vec<(usize, u32)>, &Frac)> = None;
            for (k, c) in terms.iter() {
                best = Some(match best {
                    None => (k, c),
                    Some((bk, _)) => {
                        // 比較稀疏單項式：轉為 Mono 再比
                        let ma = {
                            let mut d = vec![0u32; k.last().map(|(v, _)| v + 1).unwrap_or(0)];
                            for &(v, e) in k {
                                if v < d.len() {
                                    d[v] = e;
                                } else {
                                    d.resize(v + 1, 0);
                                    d[v] = e;
                                }
                            }
                            d
                        };
                        let mb = {
                            let mut d = vec![0u32; bk.last().map(|(v, _)| v + 1).unwrap_or(0)];
                            for &(v, e) in bk {
                                if v < d.len() {
                                    d[v] = e;
                                } else {
                                    d.resize(v + 1, 0);
                                    d[v] = e;
                                }
                            }
                            d
                        };
                        if crate::poly::cmp_mono(&ma, &mb, ord) == std::cmp::Ordering::Greater {
                            (k, c)
                        } else {
                            best.unwrap()
                        }
                    }
                });
            }
            let (k, c) = best.unwrap();
            (k.clone(), *c)
        };

        // 轉為 Mono 用於整除檢查
        let mk_mono = {
            let mut d = vec![0u32; mk_sparse.last().map(|(v, _)| v + 1).unwrap_or(0)];
            for &(v, e) in &mk_sparse {
                if v < d.len() {
                    d[v] = e;
                } else {
                    d.resize(v + 1, 0);
                    d[v] = e;
                }
            }
            d
        };

        // 找簽名安全的除子
        let mut found: Option<usize> = None;
        for (di, (lm, _, _, sig_g)) in divs.iter().enumerate() {
            if !mono_divides(lm, &mk_mono) {
                continue;
            }
            // 檢查簽名：sig_g * (mk/lm) < sig_f
            let mult = mono_div(&mk_mono, lm);
            let sig_candidate = (sig_g.0, crate::poly::mono_mul(&sig_g.1, &mult));
            if sig_cmp(&sig_candidate, sig_f, ord) == std::cmp::Ordering::Less {
                found = Some(di);
                break;
            }
        }

        match found {
            Some(di) => {
                let (lm, lc, g_poly, _) = &divs[di];
                let qm_mono = mono_div(&mk_mono, lm);
                let qm_sparse: Vec<(usize, u32)> = qm_mono
                    .iter()
                    .enumerate()
                    .filter(|(_, &e)| e > 0)
                    .map(|(i, &e)| (i, e))
                    .collect();
                let qc = mc.div(lc);
                for (m2, c2) in &g_poly.terms {
                    let m2_sparse: Vec<(usize, u32)> = m2
                        .iter()
                        .enumerate()
                        .filter(|(_, &e)| e > 0)
                        .map(|(i, &e)| (i, e))
                        .collect();
                    // 稀疏乘
                    let mut out = qm_sparse.clone();
                    for &(v, e) in &m2_sparse {
                        match out.binary_search_by_key(&v, |&(vv, _)| vv) {
                            Ok(i) => out[i].1 += e,
                            Err(i) => out.insert(i, (v, e)),
                        }
                    }
                    let pc = qc.mul(c2).neg();
                    let entry = terms.entry(out).or_insert(Frac::ZERO);
                    *entry = entry.add(&pc);
                }
                terms.retain(|_, c| !c.is_zero());
            }
            None => {
                terms.remove(&mk_sparse);
                // 轉回 Mono
                let mut d = vec![0u32; mk_sparse.last().map(|(v, _)| v + 1).unwrap_or(0)];
                for &(v, e) in &mk_sparse {
                    if v < d.len() {
                        d[v] = e;
                    } else {
                        d.resize(v + 1, 0);
                        d[v] = e;
                    }
                }
                rem_terms.push((d, mc));
            }
        }
    }

    Poly::from_terms(rem_terms)
}

/// 約化基 = F5 + 互約化
pub fn reduced_f5(fs: &[Poly], ord: Order) -> (Vec<Poly>, F5Stats) {
    let (g, mut stats) = f5(fs, ord);
    let red = crate::groebner::reduce_to_reduced_gb(g, ord);
    stats.basis_final = red.len();
    (red, stats)
}

/// F4/F5 結合：用 F5 準則選配對，用 F4 矩陣歸約
pub fn f4f5(fs: &[Poly], ord: Order) -> (Vec<Poly>, F5Stats) {
    // 簡化：先用 F5 過濾，再用 F4 批處理
    // 完整實現需將 F5 簽名帶入 F4 矩陣，此處為原型，僅統計
    let (g_f5, stats_f5) = f5(fs, ord);
    // 若 F5 已顯著減少，返回；否則用 F4 再化簡
    let (g_f4, stats_f4) = crate::groebner_f4::f4(&g_f5, ord);
    let mut combined = F5Stats {
        generators: stats_f5.generators,
        pairs_considered: stats_f5.pairs_considered + stats_f4.pairs_considered,
        f5_criterion_skips: stats_f5.f5_criterion_skips,
        rewritten_skips: stats_f5.rewritten_skips,
        crit1_skips: stats_f5.crit1_skips + stats_f4.crit1_skips,
        crit2_skips: stats_f5.crit2_skips + stats_f4.crit2_skips,
        s_polys: stats_f5.s_polys + stats_f4.s_polys,
        reductions_to_zero: stats_f5.reductions_to_zero + stats_f4.reductions_to_zero,
        basis_adds: stats_f4.basis_adds,
        basis_final: g_f4.len(),
    };
    let red = crate::groebner::reduce_to_reduced_gb(g_f4, ord);
    combined.basis_final = red.len();
    (red, combined)
}

/// 實際使用：groebner_f5.rs 文件清單 — 優化 with_capacity
pub fn groebner_f5_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("groebner_f5.rs", "groebner_f5.rs 正式運作 — 優化 with_capacity", "core/src/groebner_f5.rs"),
    ]
}

