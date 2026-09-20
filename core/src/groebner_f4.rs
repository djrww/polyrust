// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! F4 演算法硬化版：稀疏 Fp 矩陣 + 塊對角 + 增量配對 + 布爾平方自由化
//! 針對 polyrust 布爾稀疏系統優化，Frac = Fp (p=2^61-1)
//! 新增：分塊並行歸約（std::thread::scope，零依賴）

use crate::frac::Frac;
use crate::poly::{cmp_mono, mono_div, mono_divides, mono_lcm, spoly, Mono, Order, Poly};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct F4Stats {
    pub generators: usize,
    pub batches: usize,
    pub pairs_considered: usize,
    pub pairs_selected: usize,
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
    pub parallel_speedup: f64,
}

fn squarefree_mono(m: &Mono) -> Mono {
    m.iter().map(|&e| if e > 0 { 1 } else { 0 }).collect()
}

fn mono_mul_is_coprime(a: &Mono, b: &Mono) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (*x).min(*y) == 0)
}

fn pair_key(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

/// 稀疏行：col -> coeff
type SparseRow = BTreeMap<usize, Frac>;

/// F4 選擇策略：lcm 度數最小批次，截斷 64
fn f4_select_pairs(pairs: &[(usize, usize)], lms: &[Mono], ord: Order) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    if pairs.is_empty() { return (vec![], vec![]); }
    let mut with_deg: Vec<((usize, usize), u32, Mono)> = pairs.iter().map(|&(i,j)| {
        let lcm = mono_lcm(&lms[i], &lms[j]);
        let deg: u32 = lcm.iter().copied().sum();
        ((i,j), deg, lcm)
    }).collect();
    with_deg.sort_by(|a,b| a.1.cmp(&b.1).then_with(|| cmp_mono(&a.2, &b.2, ord)));
    let min_deg = with_deg[0].1;
    let mut selected = Vec::new();
    let mut remaining = Vec::new();
    for ((i,j), deg, _) in with_deg {
        if deg == min_deg { selected.push((i,j)); } else { remaining.push((i,j)); }
    }
    if selected.len() > 64 {
        remaining.extend(selected.drain(64..));
    }
    (selected, remaining)
}

/// 符號預處理：收集單項式 + 歸約子，含 LM 索引加速
fn symbolic_preprocessing(
    s_polys: Vec<Poly>,
    g: &[Poly],
    lms: &[Mono],
    ord: Order,
) -> (Vec<Poly>, Vec<Mono>, BTreeMap<Mono, usize>) {
    let mut monos_set: HashSet<Mono> = HashSet::new();
    for p in &s_polys {
        for (m, _) in &p.terms { monos_set.insert(squarefree_mono(m)); }
    }
    let mut done: HashSet<Mono> = s_polys.iter().filter_map(|p| p.lm(ord)).map(|m| squarefree_mono(&m)).collect();
    let mut todo: Vec<Mono> = monos_set.iter().filter(|&m| !done.contains(m)).cloned().collect();
    let mut reducers: Vec<Poly> = Vec::new();

    let mut lm_index: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, lm) in lms.iter().enumerate() {
        if let Some((v,_)) = lm.iter().enumerate().find(|(_, &e)| e>0) {
            lm_index.entry(v).or_default().push(idx);
        }
    }

    while let Some(m) = todo.pop() {
        if done.contains(&m) { continue; }
        done.insert(m.clone());
        let mut found = None;
        for (v, &e) in m.iter().enumerate() {
            if e==0 { continue; }
            if let Some(cands) = lm_index.get(&v) {
                for &gi in cands {
                    if mono_divides(&lms[gi], &m) { found = Some(gi); break; }
                }
            }
            if found.is_some() { break; }
        }
        if found.is_none() {
            for (gi, lm) in lms.iter().enumerate() {
                if mono_divides(lm, &m) { found = Some(gi); break; }
            }
        }
        if let Some(gi) = found {
            let mult = mono_div(&m, &lms[gi]);
            let mult_poly = Poly { terms: vec![(mult, Frac::ONE)] };
            let reductor = mult_poly.mul(&g[gi]);
            for (mm, _) in &reductor.terms {
                let mm_sf = squarefree_mono(mm);
                if !monos_set.contains(&mm_sf) {
                    monos_set.insert(mm_sf.clone());
                    if !done.contains(&mm_sf) { todo.push(mm_sf); }
                }
            }
            reducers.push(reductor);
        }
    }

    let mut all_polys = s_polys;
    all_polys.extend(reducers);
    let mut monos: Vec<Mono> = monos_set.into_iter().collect();
    monos.sort_by(|a,b| cmp_mono(a,b,ord).reverse());
    let col_index: BTreeMap<Mono, usize> = monos.iter().enumerate().map(|(i,m)| (m.clone(), i)).collect();
    (all_polys, monos, col_index)
}

/// 構建稀疏矩陣
fn build_sparse_matrix(polys: &[Poly], col_index: &BTreeMap<Mono, usize>) -> Vec<SparseRow> {
    let mut mat: Vec<SparseRow> = Vec::with_capacity(polys.len());
    for p in polys {
        let mut row: SparseRow = BTreeMap::new();
        for (m, c) in &p.terms {
            let m_sf = squarefree_mono(m);
            if let Some(&col) = col_index.get(&m_sf) {
                let entry = row.entry(col).or_insert(Frac::ZERO);
                *entry = entry.add(c);
                if entry.is_zero() { row.remove(&col); }
            } else if let Some(&col) = col_index.get(m) {
                let entry = row.entry(col).or_insert(Frac::ZERO);
                *entry = entry.add(c);
                if entry.is_zero() { row.remove(&col); }
            }
        }
        if !row.is_empty() { mat.push(row); }
    }
    mat
}

/// 檢測塊對角：按變量支集分連通分量（基於多項式）
fn detect_blocks(polys: &[Poly], _monos: &[Mono]) -> Vec<Vec<usize>> {
    if polys.len() <= 1 { return vec![(0..polys.len()).collect()]; }
    let mut var_sets: Vec<BTreeSet<usize>> = Vec::with_capacity(polys.len());
    for p in polys {
        let mut vs = BTreeSet::new();
        for (m, _) in &p.terms {
            for (vi, &e) in m.iter().enumerate() { if e>0 { vs.insert(vi); } }
        }
        var_sets.push(vs);
    }
    let n = polys.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x]!=x { parent[x]=find(parent, parent[x]); }
        parent[x]
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra!=rb { parent[rb]=ra; }
    }
    for i in 0..n {
        for j in (i+1)..n {
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
    if blocks.len() > 1 {
        blocks.sort_by(|a,b| b.len().cmp(&a.len()));
        blocks
    } else {
        blocks
    }
}

/// 稀疏矩陣專用塊檢測：基於行涉及的變量
fn detect_blocks_sparse(mat: &[SparseRow], monos: &[Mono]) -> Vec<Vec<usize>> {
    if mat.len() <= 1 { return vec![(0..mat.len()).collect()]; }
    let mut var_sets: Vec<BTreeSet<usize>> = Vec::with_capacity(mat.len());
    for row in mat {
        let mut vs = BTreeSet::new();
        for &col in row.keys() {
            if col < monos.len() {
                for (vi, &e) in monos[col].iter().enumerate() {
                    if e>0 { vs.insert(vi); }
                }
            }
        }
        var_sets.push(vs);
    }
    let n = mat.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x]!=x { parent[x]=find(parent, parent[x]); }
        parent[x]
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra!=rb { parent[rb]=ra; }
    }
    for i in 0..n {
        for j in (i+1)..n {
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
    blocks.sort_by(|a,b| b.len().cmp(&a.len()));
    blocks
}

/// 稀疏行階梯化 (Gauss-Jordan) 返回主元列
fn row_echelon_sparse(mat: &mut [SparseRow], ncols: usize) -> Vec<usize> {
    if mat.is_empty() { return vec![]; }
    let mut pivot_row = 0usize;
    let mut pivot_cols = Vec::new();
    let mut col_pivot: BTreeMap<usize, usize> = BTreeMap::new();

    for col in 0..ncols {
        let mut pivot = None;
        for r in pivot_row..mat.len() {
            if let Some(c) = mat[r].get(&col) { if !c.is_zero() { pivot = Some(r); break; } }
        }
        if let Some(pr) = pivot {
            mat.swap(pivot_row, pr);
            let piv_val = *mat[pivot_row].get(&col).unwrap();
            let inv = Frac::ONE.div(&piv_val);
            let cols_to_update: Vec<usize> = mat[pivot_row].keys().cloned().collect();
            for c in cols_to_update {
                let val = mat[pivot_row].get(&c).unwrap().mul(&inv);
                if val.is_zero() { mat[pivot_row].remove(&c); } else { mat[pivot_row].insert(c, val); }
            }
            for r in 0..mat.len() {
                if r == pivot_row { continue; }
                if let Some(&factor) = mat[r].get(&col) {
                    if factor.is_zero() { continue; }
                    let pivot_entries: Vec<(usize, Frac)> = mat[pivot_row].iter().map(|(&c,&v)| (c,v)).collect();
                    for (c, pv) in pivot_entries {
                        if c < col { continue; }
                        let sub = pv.mul(&factor);
                        let entry = mat[r].entry(c).or_insert(Frac::ZERO);
                        *entry = entry.sub(&sub);
                        if entry.is_zero() { mat[r].remove(&c); }
                    }
                    mat[r].remove(&col);
                }
            }
            col_pivot.insert(col, pivot_row);
            pivot_cols.push(col);
            pivot_row += 1;
            if pivot_row >= mat.len() { break; }
        }
    }
    pivot_cols
}

/// 分塊並行歸約：每塊獨立高斯消元，零依賴線程池
fn parallel_row_echelon_blocks(mat: Vec<SparseRow>, ncols: usize, monos: &[Mono]) -> Vec<SparseRow> {
    if mat.len() <= 4 {
        let mut m = mat;
        row_echelon_sparse(&mut m, ncols);
        return m;
    }
    let blocks = detect_blocks_sparse(&mat, monos);
    if blocks.len() <= 1 {
        let mut m = mat;
        row_echelon_sparse(&mut m, ncols);
        return m;
    }
    let mut filtered_blocks: Vec<Vec<usize>> = Vec::new();
    let mut small_acc: Vec<usize> = Vec::new();
    for b in blocks {
        if b.len() < 3 {
            small_acc.extend(b);
        } else {
            filtered_blocks.push(b);
        }
    }
    if !small_acc.is_empty() {
        filtered_blocks.push(small_acc);
    }
    if filtered_blocks.len() <= 1 {
        let mut m = mat;
        row_echelon_sparse(&mut m, ncols);
        return m;
    }

    let mat_arc = std::sync::Arc::new(mat);
    let mut results: Vec<Vec<SparseRow>> = Vec::new();

    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for block_indices in filtered_blocks {
            let mat_clone = std::sync::Arc::clone(&mat_arc);
            let handle = s.spawn(move || {
                let mut block_rows: Vec<SparseRow> = block_indices.iter().map(|&i| mat_clone[i].clone()).collect();
                row_echelon_sparse(&mut block_rows, ncols);
                block_rows
            });
            handles.push(handle);
        }
        for h in handles {
            if let Ok(br) = h.join() {
                results.push(br);
            }
        }
    });

    results.into_iter().flatten().collect()
}

fn extract_polys_sparse(mat: &[SparseRow], monos: &[Mono]) -> Vec<Poly> {
    let mut polys = Vec::with_capacity(8);
    for row in mat {
        let mut terms = Vec::new();
        for (&col, &coeff) in row.iter() {
            if !coeff.is_zero() { terms.push((monos[col].clone(), coeff)); }
        }
        if !terms.is_empty() { polys.push(Poly::from_terms(terms)); }
    }
    polys
}

pub fn f4(fs: &[Poly], ord: Order) -> (Vec<Poly>, F4Stats) {
    let mut stats = F4Stats { generators: fs.len(), ..Default::default() };
    let mut g: Vec<Poly> = fs.iter().filter(|f| !f.is_zero()).cloned().collect();
    for p in g.iter_mut() { p.make_monic(ord); }
    let mut seen: BTreeSet<Vec<(Mono, Frac)>> = BTreeSet::new();
    g.retain(|p| seen.insert(p.terms.clone()));
    let mut lms: Vec<Mono> = g.iter().map(|p| p.lm(ord).unwrap()).collect();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..g.len() { for j in (i+1)..g.len() { pairs.push((i,j)); } }
    let mut closed: BTreeSet<(usize, usize)> = BTreeSet::new();

    let mut total_density = 0f64;
    let mut density_count = 0usize;

    while !pairs.is_empty() {
        let (selected, remaining) = f4_select_pairs(&pairs, &lms, ord);
        pairs = remaining;
        if selected.is_empty() { break; }
        stats.batches += 1;
        stats.pairs_considered += selected.len();

        let mut filtered = Vec::new();
        for (i,j) in selected {
            let lmi = &lms[i]; let lmj = &lms[j];
            let lcm = mono_lcm(lmi, lmj);
            if mono_mul_is_coprime(lmi, lmj) { stats.crit1_skips+=1; continue; }
            let mut skip=false;
            for k in 0..g.len() {
                if k==i||k==j { continue; }
                if mono_divides(&lms[k], &lcm) && closed.contains(&pair_key(i,k)) && closed.contains(&pair_key(j,k)) { skip=true; break; }
            }
            if skip { stats.crit2_skips+=1; continue; }
            filtered.push((i,j));
        }
        if filtered.is_empty() { continue; }
        stats.pairs_selected += filtered.len();

        let s_polys: Vec<Poly> = filtered.iter().map(|&(i,j)| spoly(&g[i], &g[j], ord)).filter(|p| !p.is_zero()).collect();
        stats.s_polys += s_polys.len();
        if s_polys.is_empty() {
            for (i,j) in filtered { closed.insert(pair_key(i,j)); }
            continue;
        }

        let (mat_polys, monos, col_index) = symbolic_preprocessing(s_polys, &g, &lms, ord);
        stats.matrix_rows_total += mat_polys.len();
        stats.matrix_cols_total += monos.len();
        stats.matrix_rows_max = stats.matrix_rows_max.max(mat_polys.len());
        stats.matrix_cols_max = stats.matrix_cols_max.max(monos.len());

        let sparse_mat = build_sparse_matrix(&mat_polys, &col_index);
        if sparse_mat.is_empty() {
            for (i,j) in filtered { closed.insert(pair_key(i,j)); }
            continue;
        }

        let ncols = monos.len();
        if ncols>0 {
            let nonzeros: usize = sparse_mat.iter().map(|r| r.len()).sum();
            let density = nonzeros as f64 / (sparse_mat.len() * ncols) as f64 * 100.0;
            total_density += density;
            density_count += 1;
        }

        let blocks = detect_blocks(&mat_polys, &monos);
        if blocks.len() > 1 {
            stats.blocks_detected += blocks.len();
            stats.parallel_blocks = stats.parallel_blocks.max(blocks.len());
        }

        let reduced_mat = if blocks.len() > 1 && sparse_mat.len() > 8 {
            parallel_row_echelon_blocks(sparse_mat, ncols, &monos)
        } else {
            let mut m = sparse_mat;
            row_echelon_sparse(&mut m, ncols);
            m
        };

        let new_polys_raw = extract_polys_sparse(&reduced_mat, &monos);

        let mut new_polys = Vec::new();
        for mut p in new_polys_raw {
            if p.is_zero() { stats.reductions_to_zero+=1; continue; }
            p.make_monic(ord);
            if let Some(lm) = p.lm(ord) {
                let mut divisible=false;
                for existing_lm in &lms { if mono_divides(existing_lm, &lm) { divisible=true; break; } }
                if divisible { stats.reductions_to_zero+=1; continue; }
                if g.iter().any(|q| q==&p) { stats.reductions_to_zero+=1; continue; }
                new_polys.push(p);
            }
        }

        for (i,j) in filtered { closed.insert(pair_key(i,j)); }
        if new_polys.is_empty() { continue; }

        for p in new_polys {
            let new_idx = g.len();
            let lm = p.lm(ord).unwrap();
            for k in 0..new_idx { pairs.push((k, new_idx)); }
            lms.push(lm);
            g.push(p);
            stats.basis_adds+=1;
        }
    }

    if density_count>0 { stats.sparse_density_pct = total_density / density_count as f64; }
    stats.basis_final = g.len();
    (g, stats)
}

pub fn reduced_f4(fs: &[Poly], ord: Order) -> (Vec<Poly>, F4Stats) {
    let (g, mut stats) = f4(fs, ord);
    let red = crate::groebner::reduce_to_reduced_gb(g, ord);
    stats.basis_final = red.len();
    (red, stats)
}

/// 實際使用：groebner_f4.rs 文件清單 — 優化 with_capacity
pub fn groebner_f4_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("groebner_f4.rs", "groebner_f4.rs 正式運作 — 優化 with_capacity", "core/src/groebner_f4.rs"),
    ]
}

