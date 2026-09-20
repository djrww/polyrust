// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! F4/F5 結合：簽名準則選配對 + 矩陣批量歸約 + 稀疏化 + 分塊並行
//! 業界標準 (msolve, FGb)，針對 polyrust 布爾系統優化
//!
//! 流程：
//!   1. 簽名初始化：每個輸入多項式 sig=(i,1)
//!   2. 配對選擇：按簽名序 + lcm 度數最小批
//!   3. F5 過濾：F5 Criterion + Rewritten + Crit1/2
//!   4. F4 矩陣：symbolic preprocessing + build_matrix + sig-safe row echelon (稀疏+分塊並行)
//!   5. 提取新多項式 (sig 保持)

use crate::frac::Frac;
use crate::poly::{cmp_mono, mono_div, mono_divides, mono_lcm, mono_mul, spoly, Mono, Order, Poly};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct F4F5Stats {
    pub generators: usize,
    pub batches: usize,
    pub pairs_considered: usize,
    pub pairs_selected: usize,
    pub f5_skips: usize,
    pub rewritten_skips: usize,
    pub crit1_skips: usize,
    pub crit2_skips: usize,
    pub matrix_rows_total: usize,
    pub matrix_cols_total: usize,
    pub matrix_rows_max: usize,
    pub matrix_cols_max: usize,
    pub s_polys: usize,
    pub reductions_to_zero: usize,
    pub basis_adds: usize,
    pub basis_final: usize,
    pub sparse_density_pct: f64,
    pub blocks_detected: usize,
    pub parallel_blocks: usize,
}

#[derive(Clone, Debug)]
struct SignedPoly {
    sig: (usize, Mono), // (input index, monomial)
    poly: Poly,
}

fn sig_cmp(a: &(usize, Mono), b: &(usize, Mono), ord: Order) -> std::cmp::Ordering {
    a.0.cmp(&b.0).then_with(|| cmp_mono(&a.1, &b.1, ord))
}

fn mono_mul_is_coprime(a: &Mono, b: &Mono) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (*x).min(*y) == 0)
}

fn pair_key(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

fn squarefree_mono(m: &Mono) -> Mono {
    m.iter().map(|&e| if e > 0 { 1 } else { 0 }).collect()
}

fn f5_criterion(sig: &(usize, Mono), g: &[SignedPoly], lms: &[Mono], ord: Order) -> bool {
    for (j, gj) in g.iter().enumerate() {
        if sig_cmp(&gj.sig, sig, ord) != std::cmp::Ordering::Less {
            continue;
        }
        if mono_divides(&lms[j], &sig.1) && sig.0 != gj.sig.0 {
            return true;
        }
    }
    false
}

fn rewritten_criterion(sig: &(usize, Mono), g: &[SignedPoly], ord: Order) -> bool {
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

fn select_batch_f4f5(
    pairs: &[(usize, usize)],
    lms: &[Mono],
    g: &[SignedPoly],
    ord: Order,
) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    if pairs.is_empty() {
        return (vec![], vec![]);
    }
    let mut with_sig: Vec<((usize, usize), (usize, Mono), u32, Mono)> = pairs
        .iter()
        .map(|&(i, j)| {
            let lcm = mono_lcm(&lms[i], &lms[j]);
            let deg: u32 = lcm.iter().copied().sum();
            let m_i = mono_div(&lcm, &lms[i]);
            let m_j = mono_div(&lcm, &lms[j]);
            let sig_i = (g[i].sig.0, mono_mul(&g[i].sig.1, &m_i));
            let sig_j = (g[j].sig.0, mono_mul(&g[j].sig.1, &m_j));
            let sig = if sig_cmp(&sig_i, &sig_j, ord) == std::cmp::Ordering::Greater {
                sig_i
            } else {
                sig_j
            };
            ((i, j), sig, deg, lcm)
        })
        .collect();
    with_sig.sort_by(|a, b| {
        sig_cmp(&a.1, &b.1, ord)
            .then(a.2.cmp(&b.2))
            .then_with(|| cmp_mono(&a.3, &b.3, ord))
    });
    let min_deg = with_sig[0].2;
    let mut selected = Vec::new();
    let mut remaining = Vec::new();
    for ((i, j), _sig, deg, _) in with_sig {
        if deg == min_deg && selected.len() < 64 {
            selected.push((i, j));
        } else {
            remaining.push((i, j));
        }
    }
    (selected, remaining)
}

fn symbolic_preprocessing_f4f5(
    s_polys: Vec<(Poly, (usize, Mono))>,
    g: &[SignedPoly],
    lms: &[Mono],
    ord: Order,
) -> (Vec<(Poly, (usize, Mono))>, Vec<Mono>, BTreeMap<Mono, usize>) {
    let mut monos_set: HashSet<Mono> = HashSet::new();
    for (p, _) in &s_polys {
        for (m, _) in &p.terms {
            monos_set.insert(squarefree_mono(m));
        }
    }
    let mut done: HashSet<Mono> = s_polys
        .iter()
        .filter_map(|(p, _)| p.lm(ord))
        .map(|m| squarefree_mono(&m))
        .collect();
    let mut todo: Vec<Mono> = monos_set.iter().filter(|&m| !done.contains(m)).cloned().collect();
    let mut reducers: Vec<(Poly, (usize, Mono))> = Vec::new();

    let mut lm_index: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, lm) in lms.iter().enumerate() {
        if let Some((v, _)) = lm.iter().enumerate().find(|(_, &e)| e > 0) {
            lm_index.entry(v).or_default().push(idx);
        }
    }

    while let Some(m) = todo.pop() {
        if done.contains(&m) {
            continue;
        }
        done.insert(m.clone());
        let mut found = None;
        for (v, &e) in m.iter().enumerate() {
            if e == 0 {
                continue;
            }
            if let Some(cands) = lm_index.get(&v) {
                for &gi in cands {
                    if mono_divides(&lms[gi], &m) {
                        found = Some(gi);
                        break;
                    }
                }
            }
            if found.is_some() {
                break;
            }
        }
        if found.is_none() {
            for (gi, lm) in lms.iter().enumerate() {
                if mono_divides(lm, &m) {
                    found = Some(gi);
                    break;
                }
            }
        }
        if let Some(gi) = found {
            let mult = mono_div(&m, &lms[gi]);
            let mult_poly = Poly {
                terms: vec![(mult.clone(), Frac::ONE)],
            };
            let reductor = mult_poly.mul(&g[gi].poly);
            let sig_red = (g[gi].sig.0, mono_mul(&g[gi].sig.1, &mult));
            for (mm, _) in &reductor.terms {
                let mm_sf = squarefree_mono(mm);
                if !monos_set.contains(&mm_sf) {
                    monos_set.insert(mm_sf.clone());
                    if !done.contains(&mm_sf) {
                        todo.push(mm_sf);
                    }
                }
            }
            reducers.push((reductor, sig_red));
        }
    }

    let mut all = s_polys;
    all.extend(reducers);
    let mut monos: Vec<Mono> = monos_set.into_iter().collect();
    monos.sort_by(|a, b| cmp_mono(a, b, ord).reverse());
    let col_index: BTreeMap<Mono, usize> = monos.iter().enumerate().map(|(i, m)| (m.clone(), i)).collect();
    (all, monos, col_index)
}

fn build_matrix_f4f5(
    polys: &[(Poly, (usize, Mono))],
    col_index: &BTreeMap<Mono, usize>,
) -> Vec<Vec<Frac>> {
    let ncols = col_index.len();
    let mut mat = Vec::with_capacity(polys.len());
    for (p, _) in polys {
        let mut row = vec![Frac::ZERO; ncols];
        for (m, c) in &p.terms {
            let m_sf = squarefree_mono(m);
            if let Some(&col) = col_index.get(&m_sf) {
                row[col] = row[col].add(c);
            } else if let Some(&col) = col_index.get(m) {
                row[col] = row[col].add(c);
            }
        }
        if row.iter().any(|c| !c.is_zero()) {
            mat.push(row);
        }
    }
    mat
}

/// 稀疏化輔助：計算矩陣密度
fn matrix_density(mat: &[Vec<Frac>]) -> f64 {
    if mat.is_empty() || mat[0].is_empty() {
        return 0.0;
    }
    let rows = mat.len();
    let cols = mat[0].len();
    let mut nonzeros = 0usize;
    for r in mat {
        for c in r {
            if !c.is_zero() {
                nonzeros += 1;
            }
        }
    }
    nonzeros as f64 / (rows * cols) as f64 * 100.0
}

/// 塊檢測：基於多項式變量支集
fn detect_blocks_f4f5(polys: &[(Poly, (usize, Mono))], _monos: &[Mono]) -> Vec<Vec<usize>> {
    if polys.len() <= 1 {
        return vec![(0..polys.len()).collect()];
    }
    let mut var_sets: Vec<BTreeSet<usize>> = Vec::with_capacity(polys.len());
    for (p, _) in polys {
        let mut vs = BTreeSet::new();
        for (m, _) in &p.terms {
            for (vi, &e) in m.iter().enumerate() {
                if e > 0 {
                    vs.insert(vi);
                }
            }
        }
        var_sets.push(vs);
    }
    let n = polys.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if !var_sets[i].is_disjoint(&var_sets[j]) {
                union(&mut parent, i, j);
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let mut blocks: Vec<Vec<usize>> = groups.into_values().collect();
    blocks.sort_by(|a, b| b.len().cmp(&a.len()));
    blocks
}

fn row_echelon_sig_safe(
    mat: &mut [Vec<Frac>],
    sigs: &mut [(usize, Mono)],
    ord: Order,
) -> Vec<usize> {
    if mat.is_empty() {
        return vec![];
    }
    let ncols = mat[0].len();
    let mut pivot_row = 0usize;
    let mut pivot_cols = Vec::new();

    for col in 0..ncols {
        let mut pivot = None;
        for r in pivot_row..mat.len() {
            if !mat[r][col].is_zero() {
                pivot = Some(r);
                break;
            }
        }
        if let Some(pr) = pivot {
            mat.swap(pivot_row, pr);
            sigs.swap(pivot_row, pr);
            let piv_val = mat[pivot_row][col];
            let inv = Frac::ONE.div(&piv_val);
            for c in col..ncols {
                mat[pivot_row][c] = mat[pivot_row][c].mul(&inv);
            }
            for r in 0..mat.len() {
                if r == pivot_row {
                    continue;
                }
                if sig_cmp(&sigs[pivot_row], &sigs[r], ord) == std::cmp::Ordering::Greater {
                    continue;
                }
                let factor = mat[r][col];
                if factor.is_zero() {
                    continue;
                }
                for c in col..ncols {
                    let sub = mat[pivot_row][c].mul(&factor);
                    mat[r][c] = mat[r][c].sub(&sub);
                }
            }
            pivot_cols.push(col);
            pivot_row += 1;
            if pivot_row >= mat.len() {
                break;
            }
        }
    }
    pivot_cols
}

/// 分塊並行 sig-safe 消元
fn parallel_row_echelon_sig_safe(
    mat: Vec<Vec<Frac>>,
    sigs: Vec<(usize, Mono)>,
    monos: &[Mono],
    ord: Order,
) -> (Vec<Vec<Frac>>, Vec<(usize, Mono)>) {
    if mat.len() <= 4 {
        let mut m = mat;
        let mut s = sigs;
        row_echelon_sig_safe(&mut m, &mut s, ord);
        return (m, s);
    }
    // 基於變量支集檢測塊（使用 monos 推導）
    let mut var_sets: Vec<BTreeSet<usize>> = Vec::with_capacity(mat.len());
    for row in &mat {
        let mut vs = BTreeSet::new();
        for (col, coeff) in row.iter().enumerate() {
            if !coeff.is_zero() && col < monos.len() {
                for (vi, &e) in monos[col].iter().enumerate() {
                    if e > 0 {
                        vs.insert(vi);
                    }
                }
            }
        }
        var_sets.push(vs);
    }
    let n = mat.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if !var_sets[i].is_disjoint(&var_sets[j]) {
                union(&mut parent, i, j);
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    let blocks: Vec<Vec<usize>> = groups.into_values().collect();
    if blocks.len() <= 1 {
        let mut m = mat;
        let mut s = sigs;
        row_echelon_sig_safe(&mut m, &mut s, ord);
        return (m, s);
    }

    // 合併過小塊
    let mut filtered: Vec<Vec<usize>> = Vec::new();
    let mut small: Vec<usize> = Vec::new();
    for b in blocks {
        if b.len() < 3 {
            small.extend(b);
        } else {
            filtered.push(b);
        }
    }
    if !small.is_empty() {
        filtered.push(small);
    }
    if filtered.len() <= 1 {
        let mut m = mat;
        let mut s = sigs;
        row_echelon_sig_safe(&mut m, &mut s, ord);
        return (m, s);
    }

    // 並行消元
    let mat_arc = std::sync::Arc::new(mat);
    let sigs_arc = std::sync::Arc::new(sigs);
    let mut results: Vec<(Vec<Vec<Frac>>, Vec<(usize, Mono)>)> = Vec::new();

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for block_indices in filtered {
            let mat_clone = std::sync::Arc::clone(&mat_arc);
            let sigs_clone = std::sync::Arc::clone(&sigs_arc);
            let handle = scope.spawn(move || {
                let mut block_mat: Vec<Vec<Frac>> = block_indices.iter().map(|&i| mat_clone[i].clone()).collect();
                let mut block_sigs: Vec<(usize, Mono)> = block_indices.iter().map(|&i| sigs_clone[i].clone()).collect();
                row_echelon_sig_safe(&mut block_mat, &mut block_sigs, ord);
                (block_mat, block_sigs)
            });
            handles.push(handle);
        }
        for h in handles {
            if let Ok(res) = h.join() {
                results.push(res);
            }
        }
    });

    let mut combined_mat = Vec::new();
    let mut combined_sigs = Vec::new();
    for (bm, bs) in results {
        combined_mat.extend(bm);
        combined_sigs.extend(bs);
    }
    (combined_mat, combined_sigs)
}

fn extract_polys_f4f5(mat: &[Vec<Frac>], monos: &[Mono], sigs: &[(usize, Mono)]) -> Vec<SignedPoly> {
    let mut out = Vec::with_capacity(8);
    for (row_idx, row) in mat.iter().enumerate() {
        let mut terms = Vec::new();
        for (col, coeff) in row.iter().enumerate() {
            if !coeff.is_zero() {
                terms.push((monos[col].clone(), *coeff));
            }
        }
        if !terms.is_empty() {
            let poly = Poly::from_terms(terms);
            if !poly.is_zero() {
                out.push(SignedPoly {
                    sig: sigs[row_idx].clone(),
                    poly,
                });
            }
        }
    }
    out
}

/// F4/F5 主循環
pub fn f4f5(fs: &[Poly], ord: Order) -> (Vec<Poly>, F4F5Stats) {
    let mut stats = F4F5Stats {
        generators: fs.len(),
        ..Default::default()
    };
    let mut g: Vec<SignedPoly> = Vec::new();
    for (i, f) in fs.iter().enumerate() {
        if f.is_zero() {
            continue;
        }
        let mut p = f.clone();
        p.make_monic(ord);
        let sig = (i, vec![]);
        g.push(SignedPoly { sig, poly: p });
    }
    let mut seen: BTreeSet<Vec<(Mono, Frac)>> = BTreeSet::new();
    g.retain(|sp| seen.insert(sp.poly.terms.clone()));

    let mut lms: Vec<Mono> = g.iter().map(|sp| sp.poly.lm(ord).unwrap()).collect();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..g.len() {
        for j in (i + 1)..g.len() {
            pairs.push((i, j));
        }
    }
    let mut closed: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut total_density = 0f64;
    let mut density_cnt = 0usize;

    while !pairs.is_empty() {
        let (selected, remaining) = select_batch_f4f5(&pairs, &lms, &g, ord);
        pairs = remaining;
        if selected.is_empty() {
            break;
        }
        stats.batches += 1;
        stats.pairs_considered += selected.len();

        let mut filtered: Vec<(usize, usize, (usize, Mono))> = Vec::new();
        for (i, j) in selected {
            let lmi = &lms[i];
            let lmj = &lms[j];
            let lcm = mono_lcm(lmi, lmj);
            if mono_mul_is_coprime(lmi, lmj) {
                stats.crit1_skips += 1;
                continue;
            }
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
            let m_i = mono_div(&lcm, lmi);
            let m_j = mono_div(&lcm, lmj);
            let sig_i = (g[i].sig.0, mono_mul(&g[i].sig.1, &m_i));
            let sig_j = (g[j].sig.0, mono_mul(&g[j].sig.1, &m_j));
            let sig = if sig_cmp(&sig_i, &sig_j, ord) == std::cmp::Ordering::Greater {
                sig_i
            } else {
                sig_j
            };
            if f5_criterion(&sig, &g, &lms, ord) {
                stats.f5_skips += 1;
                continue;
            }
            if rewritten_criterion(&sig, &g, ord) {
                stats.rewritten_skips += 1;
                continue;
            }
            filtered.push((i, j, sig));
        }

        if filtered.is_empty() {
            continue;
        }
        stats.pairs_selected += filtered.len();

        let s_polys: Vec<(Poly, (usize, Mono))> = filtered
            .iter()
            .map(|(i, j, sig)| (spoly(&g[*i].poly, &g[*j].poly, ord), sig.clone()))
            .filter(|(p, _)| !p.is_zero())
            .collect();
        stats.s_polys += s_polys.len();
        if s_polys.is_empty() {
            for (i, j, _) in filtered {
                closed.insert(pair_key(i, j));
            }
            continue;
        }

        let (mat_polys_with_sig, monos, col_index) = symbolic_preprocessing_f4f5(s_polys, &g, &lms, ord);
        stats.matrix_rows_total += mat_polys_with_sig.len();
        stats.matrix_cols_total += monos.len();
        stats.matrix_rows_max = stats.matrix_rows_max.max(mat_polys_with_sig.len());
        stats.matrix_cols_max = stats.matrix_cols_max.max(monos.len());

        let mut mat = build_matrix_f4f5(&mat_polys_with_sig, &col_index);
        if mat.is_empty() {
            for (i, j, _) in filtered {
                closed.insert(pair_key(i, j));
            }
            continue;
        }

        // 稀疏化密度統計
        let dens = matrix_density(&mat);
        total_density += dens;
        density_cnt += 1;

        // 塊檢測
        let blocks = detect_blocks_f4f5(&mat_polys_with_sig, &monos);
        if blocks.len() > 1 {
            stats.blocks_detected += blocks.len();
            stats.parallel_blocks = stats.parallel_blocks.max(blocks.len());
        }

        let mut sigs: Vec<(usize, Mono)> = mat_polys_with_sig.iter().map(|(_, s)| s.clone()).collect();
        let mut indexed: Vec<(usize, Vec<Frac>, (usize, Mono))> = mat
            .into_iter()
            .enumerate()
            .map(|(idx, row)| (idx, row, sigs[idx].clone()))
            .collect();
        indexed.sort_by(|a, b| sig_cmp(&a.2, &b.2, ord));
        mat = indexed.iter().map(|(_, row, _)| row.clone()).collect();
        sigs = indexed.iter().map(|(_, _, sig)| sig.clone()).collect();

        // 分塊並行 sig-safe 消元
        let (mat_reduced, sigs_reduced) = if blocks.len() > 1 && mat.len() > 8 {
            parallel_row_echelon_sig_safe(mat, sigs, &monos, ord)
        } else {
            let mut m = mat;
            let mut s = sigs;
            row_echelon_sig_safe(&mut m, &mut s, ord);
            (m, s)
        };

        let new_signed = extract_polys_f4f5(&mat_reduced, &monos, &sigs_reduced);

        for (i, j, _) in filtered {
            closed.insert(pair_key(i, j));
        }

        let mut added = 0;
        for mut sp in new_signed {
            if sp.poly.is_zero() {
                stats.reductions_to_zero += 1;
                continue;
            }
            sp.poly.make_monic(ord);
            if let Some(lm) = sp.poly.lm(ord) {
                let mut divisible = false;
                for existing_lm in &lms {
                    if mono_divides(existing_lm, &lm) {
                        divisible = true;
                        break;
                    }
                }
                if divisible {
                    stats.reductions_to_zero += 1;
                    continue;
                }
                if g.iter().any(|q| q.poly == sp.poly) {
                    stats.reductions_to_zero += 1;
                    continue;
                }
                let new_idx = g.len();
                for k in 0..new_idx {
                    pairs.push((k, new_idx));
                }
                lms.push(lm);
                g.push(sp);
                added += 1;
            }
        }
        stats.basis_adds += added;
    }

    if density_cnt > 0 {
        stats.sparse_density_pct = total_density / density_cnt as f64;
    }
    let polys: Vec<Poly> = g.into_iter().map(|sp| sp.poly).collect();
    stats.basis_final = polys.len();
    (polys, stats)
}

pub fn reduced_f4f5(fs: &[Poly], ord: Order) -> (Vec<Poly>, F4F5Stats) {
    let (g, mut stats) = f4f5(fs, ord);
    let red = crate::groebner::reduce_to_reduced_gb(g, ord);
    stats.basis_final = red.len();
    (red, stats)
}

/// 實際使用：groebner_f4f5.rs 文件清單 — 優化 with_capacity
pub fn groebner_f4f5_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("groebner_f4f5.rs", "groebner_f4f5.rs 正式運作 — 優化 with_capacity", "core/src/groebner_f4f5.rs"),
    ]
}

