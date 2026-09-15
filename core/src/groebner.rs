//! Buchberger 演算法：S-多項式循環、兩條消去準則（第一準則 = LM 互素；
//! 第二準則 = 鏈準則）、約化 Gröbner 基，以及布爾多項式系統（含域多項式
//! x²−x）的回溯求解。

use crate::frac::Frac;
use crate::poly::{div_rem, mono_divides, mono_lcm, spoly, Mono, Order, Poly};
use std::collections::BTreeSet;

/// 配對選擇策略（定理 7a：不同策略 ⇒ 相同約化基）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// 「常規選擇策略」：選 lcm 最小的配對（實務最佳）。
    Normal,
    /// FIFO（配對產生順序）。
    Fifo,
}

#[derive(Clone, Debug, Default)]
pub struct GroebnerStats {
    pub generators: usize,
    pub pairs_considered: usize,
    pub crit1_skips: usize,     // 第一準則（LM 互素）消除
    pub crit2_skips: usize,     // 第二準則（鏈準則）消除
    pub s_polys: usize,         // 實際計算的 S-多項式
    pub reductions_to_zero: usize,
    pub basis_adds: usize,      // 加入基的非零餘式
    pub basis_final: usize,     // 最終（約化前）基大小
}

/// 布爾域多項式：對每個變量 x 加上 x² − x。
pub fn field_polys(nvars: usize) -> Vec<Poly> {
    (0..nvars)
        .map(|i| {
            Poly::var(i, Frac::ONE, nvars)
                .pow(2)
                .sub(&Poly::var(i, Frac::ONE, nvars))
        })
        .collect()
}

fn pair_key(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

/// 最小堆項：以 lcm(LM(gᵢ),LM(gⱼ)) 為鍵（常規選擇策略）。
#[derive(Clone)]
struct HeapItem {
    lcm: Mono,
    i: usize,
    j: usize,
}
impl PartialEq for HeapItem {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == std::cmp::Ordering::Equal
    }
}
impl Eq for HeapItem {}
impl PartialOrd for HeapItem {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for HeapItem {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        // 反轉（BinaryHeap 為最大堆 ⇒ 我們要 lcm 最小者先出）
        crate::poly::cmp_mono(&self.lcm, &o.lcm, ORD_FOR_HEAP.with(|c| *c.borrow()))
            .reverse()
            .then(self.i.cmp(&o.i)).then(self.j.cmp(&o.j))
    }
}
thread_local! {
    static ORD_FOR_HEAP: std::cell::RefCell<Order> = std::cell::RefCell::new(Order::GrevLex);
}

/// Buchberger 主循環。輸入生成元 fs（理想 J = ⟨fs⟩），輸出（未約化的）Gröbner 基。
pub fn buchberger(fs: &[Poly], ord: Order, strat: Strategy, use_criteria: bool) -> (Vec<Poly>, GroebnerStats) {
    ORD_FOR_HEAP.with(|c| *c.borrow_mut() = ord);
    let mut stats = GroebnerStats { generators: fs.len(), ..Default::default() };
    let mut g: Vec<Poly> = fs.iter().filter(|f| !f.is_zero()).cloned().collect();
    for p in g.iter_mut() {
        p.make_monic(ord);
    }
    // 去除重複
    let mut seen: BTreeSet<Vec<(Mono, Frac)>> = BTreeSet::new();
    g.retain(|p| seen.insert(p.terms.clone()));

    // 快取各基元素的首單項式（效能關鍵）
    let mut lms: Vec<Mono> = g.iter().map(|p| p.lm(ord).unwrap()).collect();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    let mut heap: std::collections::BinaryHeap<HeapItem> = std::collections::BinaryHeap::new();
    for i in 0..g.len() {
        for j in (i + 1)..g.len() {
            pairs.push((i, j));
            heap.push(HeapItem { lcm: mono_lcm(&lms[i], &lms[j]), i, j });
        }
    }
    // 已「閉合」的配對（S-多項式已歸約為 0）—— 第二準則的依據
    let mut closed: BTreeSet<(usize, usize)> = BTreeSet::new();

    while match strat {
        Strategy::Fifo => !pairs.is_empty(),
        Strategy::Normal => !heap.is_empty(),
    } {
        // 選擇配對
        let sel = match strat {
            Strategy::Normal => {
                // lcm(LM) 最小者（最小堆，O(log n)）
                match heap.pop() {
                    Some(h) => (h.i, h.j),
                    None => break,
                }
            }
            Strategy::Fifo => match pairs.first().copied() {
                Some(p) => {
                    pairs.remove(0);
                    p
                }
                None => break,
            },
        };
        let (i, j) = sel;
        stats.pairs_considered += 1;
        if std::env::var("GB_DBG").is_ok() && stats.pairs_considered % 200 == 0 {
            eprintln!("  pair#{} basis={} pairs_left={}", stats.pairs_considered, g.len(), pairs.len());
        }

        let lmi = lms[i].clone();
        let lmj = lms[j].clone();
        let l = mono_lcm(&lmi, &lmj);

        // 第一準則：LM(f)·LM(g) = lcm ⇔ 互素 ⇒ S(f,g) → 0
        if use_criteria && mono_mul_is_coprime(&lmi, &lmj) {
            stats.crit1_skips += 1;
            continue;
        }
        // 第二準則（鏈準則）：∃k，LM(g_k) | lcm 且 S(g_i,g_k)、S(g_j,g_k) 均已歸約為 0
        if use_criteria {
            let mut found = false;
            for k in 0..g.len() {
                if k == i || k == j {
                    continue;
                }
                let lmk = &lms[k];
                if mono_divides(lmk, &l)
                    && closed.contains(&pair_key(i, k))
                    && closed.contains(&pair_key(j, k))
                {
                    found = true;
                    break;
                }
            }
            if found {
                stats.crit2_skips += 1;
                continue;
            }
        }

        stats.s_polys += 1;
        let s = spoly(&g[i], &g[j], ord);
        let r = div_rem(&s, &g, ord);
        if r.is_zero() {
            stats.reductions_to_zero += 1;
            closed.insert(pair_key(i, j));
        } else {
            stats.basis_adds += 1;
            let mut r = r;
            r.make_monic(ord);
            let new_idx = g.len();
            for k in 0..new_idx {
                match strat {
                    Strategy::Fifo => pairs.push((k, new_idx)),
                    Strategy::Normal => heap.push(HeapItem {
                        lcm: mono_lcm(&lms[k], &r.lm(ord).unwrap()),
                        i: k,
                        j: new_idx,
                    }),
                }
            }
            lms.push(r.lm(ord).unwrap());
            g.push(r);
        }
    }
    stats.basis_final = g.len();
    (g, stats)
}

fn mono_mul_is_coprime(a: &Mono, b: &Mono) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (*x).min(*y) == 0)
}

/// 約化 Gröbner 埯：每個元素 monic、任一項不被其他元素的首項整除、
/// 互為唯一範式。定理 7a：固定單項式序 ⇒ 唯一。
pub fn reduce_to_reduced_gb(mut g: Vec<Poly>, ord: Order) -> Vec<Poly> {
    if std::env::var("GB_DBG").is_ok() {
        eprintln!("  reduce: input basis {}", g.len());
    }
    let mut iterations = 0usize;
    loop {
        iterations += 1;
        if std::env::var("GB_DBG").is_ok() {
            eprintln!("  reduce: pass {}", iterations);
        }
        assert!(iterations < 50, "約化不收斂（內部錯誤）");
        let mut changed = false;
        let mut i = 0;
        while i < g.len() {
            if g[i].is_zero() {
                g.remove(i);
                changed = true;
                continue;
            }
            let others: Vec<Poly> = g
                .iter()
                .enumerate()
                .filter(|(k, _)| *k != i)
                .map(|(_, p)| p.clone())
                .collect();
            let r = div_rem(&g[i], &others, ord);
            if r != g[i] {
                changed = true;
                g[i] = r;
            }
            i += 1;
        }
        if !changed {
            break;
        }
    }
    g.retain(|p| !p.is_zero());
    for p in g.iter_mut() {
        p.make_monic(ord);
    }
    // 去除重複
    let mut seen: BTreeSet<Vec<(Mono, Frac)>> = BTreeSet::new();
    g.retain(|p| seen.insert(p.terms.clone()));
    g.sort_by(|a, b| {
        let ma = a.lm(ord).unwrap();
        let mb = b.lm(ord).unwrap();
        crate::poly::cmp_mono(&ma, &mb, ord)
    });
    g
}

/// 約化 Gröbner 基 = buchberger + 互約化。
pub fn reduced_groebner(
    fs: &[Poly],
    ord: Order,
    strat: Strategy,
    use_criteria: bool,
) -> (Vec<Poly>, GroebnerStats) {
    let (g, mut stats) = buchberger(fs, ord, strat, use_criteria);
    let red = reduce_to_reduced_gb(g, ord);
    stats.basis_final = red.len();
    (red, stats)
}

/// 正規範式（對 Gröbner 基）。
pub fn normal_form(f: &Poly, g: &[Poly], ord: Order) -> Poly {
    div_rem(f, g, ord)
}

/// 布爾系統求解：回溯 + 單元傳播。
/// 輸入 fs（應含域多項式）。回傳 0/1 賦值（None = 無解）。
/// 前置條件：呼叫者宜先以 Gröbner 基確認可解（定理 6）。
pub fn solve_boolean(fs: &[Poly], nvars: usize) -> Option<Vec<Frac>> {
    let mut steps = 0usize;
    let mut assign: Vec<Option<Frac>> = vec![None; nvars];
    let polys: Vec<Poly> = fs.iter().filter(|f| !f.is_zero()).cloned().collect();
    solve_rec(polys, &mut assign, &mut steps)
}

fn solve_rec(polys: Vec<Poly>, assign: &mut Vec<Option<Frac>>, steps: &mut usize) -> Option<Vec<Frac>> {
    *steps += 1;
    assert!(*steps < 500_000, "布爾求解步數超限");
    // 化簡 + 單元傳播至定點
    let mut polys = polys;
    loop {
        // 常數非零 ⇒ 失敗
        for p in &polys {
            if let Some(c) = p.is_constant() {
                if !c.is_zero() {
                    return None;
                }
            }
        }
        polys.retain(|p| !p.is_zero());
        // 線性單元：c1·x + c0 = 0 ⇒ x = −c0/c1，值必須 ∈ {0,1}
        let mut propagated = false;
        let mut next = Vec::with_capacity(polys.len());
        let mut fail = false;
        for p in polys.drain(..) {
            if let Some((i, c1, c0)) = p.as_single_linear() {
                if assign[i].is_none() {
                    let v = c0.neg().div(&c1);
                    if v.is_zero() || v.is_one() {
                        assign[i] = Some(v);
                        propagated = true;
                        continue; // 該約束已滿足，丟棄
                    } else {
                        fail = true;
                        break;
                    }
                }
            }
            next.push(p);
        }
        polys = next;
        if fail {
            return None;
        }
        if !propagated {
            break;
        }
        // 重新代入已賦值變量
        for i in 0..assign.len() {
            if let Some(v) = assign[i] {
                polys = polys.iter().map(|p| p.subst_var(i, &v)).collect();
            }
        }
    }
    // 找下一個未賦值變量
    match (0..assign.len()).find(|i| assign[*i].is_none()) {
        None => {
            // 全部賦值：最終驗證
            if polys.iter().all(|p| p.is_zero()) {
                Some(assign.iter().map(|v| v.unwrap()).collect())
            } else {
                None
            }
        }
        Some(i) => {
            for val in [Frac::ONE, Frac::ZERO] {
                let mut a2 = assign.clone();
                a2[i] = Some(val);
                let p2: Vec<Poly> = polys.iter().map(|p| p.subst_var(i, &val)).collect();
                if let Some(sol) = solve_rec(p2, &mut a2, steps) {
                    *assign = a2;
                    return Some(sol);
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poly::Poly;

    fn var(i: usize, n: usize) -> Poly {
        Poly::var(i, Frac::ONE, n)
    }
    fn c(x: i64) -> Poly {
        Poly::constant(Frac::from_i64(x))
    }

    #[test]
    fn test_simple_gb() {
        // ⟨x − 1, y − 1⟩ 的 Gröbner 基即自身（已約化）
        let n = 2;
        let f1 = var(0, n).sub(&c(1));
        let f2 = var(1, n).sub(&c(1));
        let (g, _) = reduced_groebner(&[f1.clone(), f2.clone()], Order::Lex, Strategy::Normal, true);
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn test_inconsistent_boolean() {
        // x = 1, x = 0（經由 x²−x 域多項式）⇒ 基 = {1}
        let n = 1;
        let f = vec![
            var(0, n).sub(&c(1)), // x − 1
            var(0, n),            // x
        ];
        let all: Vec<Poly> = f.iter().chain(field_polys(n).iter()).cloned().collect();
        let (g, _) = reduced_groebner(&all, Order::GrevLex, Strategy::Normal, true);
        assert_eq!(g.len(), 1);
        assert!(g[0].is_constant().map(|c| c.is_one()).unwrap_or(false));
    }

    #[test]
    fn test_solve_boolean() {
        // x·y = 0, x + y = 1（含域多項式）⇒ 解 (1,0) 或 (0,1)
        let n = 2;
        let fs = vec![
            var(0, n).mul(&var(1, n)),
            var(0, n).add(&var(1, n)).sub(&c(1)),
        ];
        let mut all: Vec<Poly> = fs;
        all.extend(field_polys(n));
        let sol = solve_boolean(&all, n).unwrap();
        assert!(sol[0].is_zero() || sol[1].is_zero());
        assert!(sol[0].add(&sol[1]).is_one());
    }

    fn is_gb(g: &[Poly], ord: Order) -> usize {
        let mut bad = 0;
        for i in 0..g.len() {
            for j in (i + 1)..g.len() {
                let s = crate::poly::spoly(&g[i], &g[j], ord);
                let r = div_rem(&s, g, ord);
                if !r.is_zero() {
                    bad += 1;
                }
            }
        }
        bad
    }

    #[test]
    fn test_diagnose_gb_mismatch() {
        let n = 6;
        let v = |i: usize| Poly::var(i, Frac::ONE, n);
        let c = |x: i64| Poly::constant(Frac::from_i64(x));
        let mut fs = vec![
            v(0).add(&v(1)).sub(&c(1)),
            v(2).add(&v(3)).sub(&c(1)),
            v(4).sub(&v(0).mul(&v(2))),
            v(4).add(&v(5)).sub(&c(1)),
        ];
        fs.extend(field_polys(n));
        let (g1, s1) = reduced_groebner(&fs, Order::GrevLex, Strategy::Normal, true);
        let (g2, s2) = reduced_groebner(&fs, Order::GrevLex, Strategy::Fifo, false);
        eprintln!("g1={} (S={} add={}) g2={} (S={} add={})", g1.len(), s1.s_polys, s1.basis_adds, g2.len(), s2.s_polys, s2.basis_adds);
        eprintln!("g1 bad pairs: {}", is_gb(&g1, Order::GrevLex));
        eprintln!("g2 bad pairs: {}", is_gb(&g2, Order::GrevLex));
        eprintln!("g1 == g2: {}", g1 == g2);
        let names: Vec<String> = ["x0","x1","x2","x3","x4","x5"].iter().map(|s| s.to_string()).collect();
        for p in &g1 {
            if !g2.contains(p) {
                eprintln!("  only in g1: {}", p.display(&names));
            }
        }
        for p in &g2 {
            if !g1.contains(p) {
                eprintln!("  only in g2: {}", p.display(&names));
            }
        }
        let g1b = reduce_to_reduced_gb(g1.clone(), Order::GrevLex);
        let g2b = reduce_to_reduced_gb(g2.clone(), Order::GrevLex);
        eprintln!("g1b==g2b: {} (g1 changed: {}, g2 changed: {})", g1b == g2b, g1b != g1, g2b != g2);
        assert!(g1b == g2b, "約化基不唯一");
    }

    #[test]
    fn test_criteria_preserve_basis() {
        // 帶/不帶準則 ⇒ 相同約化基（定理 5 / 7 的實例）
        let n = 3;
        let f1 = var(0, n).mul(&var(1, n)).sub(&c(1)); // xy − 1
        let f2 = var(1, n).sub(&c(1));                 // y − 1
        let f3 = var(2, n).sub(&var(0, n));            // z − x
        let fs = vec![f1, f2, f3];
        let (g1, s1) = reduced_groebner(&fs, Order::GrevLex, Strategy::Normal, true);
        let (g2, s2) = reduced_groebner(&fs, Order::GrevLex, Strategy::Fifo, false);
        assert_eq!(g1, g2);
        assert!(s1.crit1_skips + s1.crit2_skips > 0 || s1.s_polys == s2.s_polys);
        let _ = s2;
    }
}
