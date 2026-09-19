//! 多變量多項式（係數於 ℚ）：單項式、三種單項式序（lex / grlex / grevlex）、
//! 多元除法、S-多項式。這是 Buchberger 演算法與代數編碼的基礎設施。

use crate::frac::Frac;
use std::collections::BTreeMap;

/// 單項式：指數向量（缺項視為 0；長度不一致時隱式補零）。
pub type Mono = Vec<u32>;

fn pad_to(m: &Mono, n: usize) -> Mono {
    let mut r = m.to_vec();
    while r.len() < n {
        r.push(0);
    }
    r
}

fn harmonized(a: &Mono, b: &Mono) -> (Mono, Mono) {
    let n = a.len().max(b.len());
    if a.len() == n && b.len() == n {
        (a.clone(), b.clone())
    } else {
        (pad_to(a, n), pad_to(b, n))
    }
}

pub fn mono_mul(a: &Mono, b: &Mono) -> Mono {
    let (a, b) = harmonized(a, b);
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

pub fn mono_divides(a: &Mono, b: &Mono) -> bool {
    // a | b  ⟺  ∀i: a[i] <= b[i]（缺項 = 0）
    let (a, b) = harmonized(a, b);
    a.iter().zip(b.iter()).all(|(x, y)| x <= y)
}

pub fn mono_div(a: &Mono, b: &Mono) -> Mono {
    let (a, b) = harmonized(a, b);
    a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
}

pub fn mono_lcm(a: &Mono, b: &Mono) -> Mono {
    let (a, b) = harmonized(a, b);
    a.iter().zip(b.iter()).map(|(x, y)| (*x).max(*y)).collect()
}


pub fn mono_is_one(m: &Mono) -> bool {
    m.iter().all(|&e| e == 0)
}

pub fn mono_deg(m: &Mono) -> u32 {
    m.iter().sum()
}

/// 單項式序。所有序皆為良序（Dickson 引理 ⇒ 每個單項理想有限生成）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    /// 字典序 lex（消元用；定理 6 的三角化求解）
    #[allow(dead_code)] // 僅單元測試構造
    Lex,
    /// 全次數優先 + 字典序（grlex）
    #[allow(dead_code)] // 保留完整單項式序 API
    GrLex,
    /// 全次數優先 + 逆字典序（grevlex；Buchberger 實務首選）
    GrevLex,
}

pub fn cmp_mono(a: &Mono, b: &Mono, ord: Order) -> std::cmp::Ordering {
    let (a, b) = harmonized(a, b);
    match ord {
        Order::Lex => cmp_vec_lex(&a, &b),
        Order::GrLex => match mono_deg(&a).cmp(&mono_deg(&b)) {
            std::cmp::Ordering::Equal => cmp_vec_lex(&a, &b),
            o => o,
        },
        Order::GrevLex => match mono_deg(&a).cmp(&mono_deg(&b)) {
            std::cmp::Ordering::Equal => {
                // 逆字典序：比較 (a−b) 最右非零分量，負者為大
                for i in (0..a.len()).rev() {
                    match a[i].cmp(&b[i]) {
                        std::cmp::Ordering::Equal => continue,
                        std::cmp::Ordering::Less => return std::cmp::Ordering::Greater,
                        std::cmp::Ordering::Greater => return std::cmp::Ordering::Less,
                    }
                }
                std::cmp::Ordering::Equal
            }
            o => o,
        },
    }
}

fn cmp_vec_lex(a: &[u32], b: &[u32]) -> std::cmp::Ordering {
    for i in 0..a.len() {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Equal => continue,
            o => return o,
        }
    }
    std::cmp::Ordering::Equal
}

/// 多項式：項的集合。不變量：無零係數、無重複單項式、所有單項式等長。
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Poly {
    pub terms: Vec<(Mono, Frac)>,
}

impl Poly {
    pub fn zero() -> Poly {
        Poly { terms: vec![] }
    }

    pub fn constant(c: Frac) -> Poly {
        if c.is_zero() {
            Poly::zero()
        } else {
            Poly { terms: vec![(vec![], c)] }
        }
    }

    pub fn from_terms(terms: Vec<(Mono, Frac)>) -> Poly {
        // 正規化：所有單項式補齊至相同長度（缺項 = 0）
        let n = terms.iter().map(|(m, _)| m.len()).max().unwrap_or(0);
        let mut map: BTreeMap<Mono, Frac> = BTreeMap::new();
        for (m, c) in terms {
            if c.is_zero() {
                continue;
            }
            *map.entry(pad_to(&m, n)).or_insert(Frac::ZERO) += c;
        }
        let terms: Vec<(Mono, Frac)> =
            map.into_iter().filter(|(_, c)| !c.is_zero()).collect();
        Poly { terms }
    }

    pub fn var(idx: usize, coeff: Frac, nvars: usize) -> Poly {
        let mut m = vec![0u32; nvars];
        m[idx] = 1;
        Poly::from_terms(vec![(m, coeff)])
    }

    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn is_constant(&self) -> Option<Frac> {
        match self.terms.as_slice() {
            [] => Some(Frac::ZERO),
            [(m, c)] if mono_is_one(m) => Some(*c),
            _ => None,
        }
    }

    /// 首項（leading term）：(單項式, 係數)。
    pub fn lt(&self, ord: Order) -> Option<(Mono, Frac)> {
        self.lm(ord).map(|m| {
            let c = self.terms.iter().find(|(t, _)| *t == m).unwrap().1;
            (m, c)
        })
    }

    /// 首單項式（leading monomial）。
    pub fn lm(&self, ord: Order) -> Option<Mono> {
        let mut best: Option<&Mono> = None;
        for (m, _) in &self.terms {
            best = Some(match best {
                None => m,
                Some(b) => {
                    if cmp_mono(m, b, ord) == std::cmp::Ordering::Greater {
                        m
                    } else {
                        b
                    }
                }
            });
        }
        best.cloned()
    }

    /// 首係數。
    pub fn lc(&self, ord: Order) -> Option<Frac> {
        self.lt(ord).map(|(_, c)| c)
    }

    pub fn make_monic(&mut self, ord: Order) {
        if let Some(c) = self.lc(ord) {
            if !c.is_one() {
                let inv = Frac::ONE.div(&c);
                self.terms = self
                    .terms
                    .iter()
                    .map(|(m, k)| (m.clone(), k.mul(&inv)))
                    .collect();
            }
        }
    }

    pub fn add(&self, o: &Poly) -> Poly {
        let mut terms = self.terms.clone();
        terms.extend(o.terms.iter().cloned());
        Poly::from_terms(terms)
    }

    pub fn sub(&self, o: &Poly) -> Poly {
        let mut terms = self.terms.clone();
        terms.extend(o.terms.iter().map(|(m, c)| (m.clone(), c.neg())));
        Poly::from_terms(terms)
    }


    pub fn mul(&self, o: &Poly) -> Poly {
        let mut terms = Vec::with_capacity(self.terms.len() * o.terms.len().max(1));
        for (m1, c1) in &self.terms {
            for (m2, c2) in &o.terms {
                terms.push((mono_mul(m1, m2), c1.mul(c2)));
            }
        }
        Poly::from_terms(terms)
    }

    /// 冪（僅用於小次數）。
    pub fn pow(&self, k: u32) -> Poly {
        let mut r = Poly::constant(Frac::ONE);
        for _ in 0..k {
            r = r.mul(self);
        }
        r
    }

    /// 變量代入 x_idx := v（求值同態；定理 7 的求值同態基礎）。
    pub fn subst_var(&self, idx: usize, v: &Frac) -> Poly {
        let mut terms = Vec::with_capacity(self.terms.len());
        for (m, c) in &self.terms {
            let e = m.get(idx).copied().unwrap_or(0);
            if e == 0 {
                terms.push((m.clone(), *c));
            } else {
                // c · v^e ·（其餘變量）
                let mut cf = *c;
                for _ in 0..e {
                    cf = cf.mul(v);
                }
                let mut m2 = m.clone();
                while m2.len() <= idx {
                    m2.push(0);
                }
                m2[idx] = 0;
                terms.push((m2, cf));
            }
        }
        Poly::from_terms(terms)
    }

    /// 完整賦值求值（全部變量皆已給值）。
    pub fn eval_full(&self, vals: &[Frac]) -> Frac {
        let mut acc = Frac::ZERO;
        for (m, c) in &self.terms {
            let mut t = *c;
            for (i, &e) in m.iter().enumerate() {
                for _ in 0..e {
                    t = t.mul(&vals[i]);
                }
            }
            acc = acc.add(&t);
        }
        acc
    }


    /// 是否為某單一變量的線性式 c1·x + c0。回傳 (變量索引, c1, c0)。
    pub fn as_single_linear(&self) -> Option<(usize, Frac, Frac)> {
        let mut var: Option<usize> = None;
        let mut c1 = Frac::ZERO;
        let mut c0 = Frac::ZERO;
        for (m, c) in &self.terms {
            match mono_deg(m) {
                0 => c0 = *c,
                1 => {
                    let i = m.iter().position(|&e| e == 1).unwrap();
                    match var {
                        None => {
                            var = Some(i);
                            c1 = *c;
                        }
                        Some(j) if j == i => c1 = c1.add(c),
                        _ => return None, // 兩個不同變量 ⇒ 非單變量線性
                    }
                }
                _ => return None, // 次數 ≥ 2 ⇒ 非線性
            }
        }
        var.filter(|_| !c1.is_zero()).map(|i| (i, c1, c0))
    }

    /// 用變量名渲染。names[i] 為變量 i 的名字。
    pub fn display(&self, names: &[String]) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut s = String::new();
        for (i, (m, c)) in self.terms.iter().enumerate() {
            if i > 0 {
                s.push_str(if c.signum() >= 0 { " + " } else { " - " });
            } else if c.signum() < 0 {
                s.push('-');
            }
            let mag = if c.signum() < 0 { c.neg() } else { *c };
            let mut has_var = false;
            for (j, &e) in m.iter().enumerate() {
                if e > 0 {
                    has_var = true;
                    let name = names.get(j).map(|s| s.as_str()).unwrap_or("?");
                    if e == 1 {
                        s.push_str(name);
                    } else {
                        s.push_str(&format!("{}^{}", name, e));
                    }
                }
            }
            if !has_var || !mag.is_one() {
                if has_var && !mag.is_one() {
                    s.push_str(&format!("{}", mag));
                    s.push('*');
                } else if !has_var {
                    s.push_str(&format!("{}", mag));
                }
            }
        }
        s
    }
}

// ── 稀疏單項式（除法熱路徑用）──
type SMono = Vec<(usize, u32)>; // (var, exp) 按 var 升冪

fn to_sparse(m: &Mono) -> SMono {
    m.iter().enumerate().filter(|(_, &e)| e > 0).map(|(i, &e)| (i, e)).collect()
}

fn from_sparse(m: &SMono) -> Mono {
    let n = m.last().map_or(0, |(v, _)| v + 1);
    let mut d = vec![0u32; n];
    for &(v, e) in m {
        d[v] = e;
    }
    d
}

fn sdeg(m: &SMono) -> u32 {
    m.iter().map(|(_, e)| e).sum()
}

fn sget(m: &SMono, v: usize) -> u32 {
    // 二元搜尋（m 按 var 升冪）
    match m.binary_search_by_key(&v, |&(vv, _)| vv) {
        Ok(i) => m[i].1,
        Err(_) => 0,
    }
}

fn cmp_sparse(a: &SMono, b: &SMono, ord: Order) -> std::cmp::Ordering {
    match ord {
        Order::Lex => cmp_slex(a, b),
        Order::GrLex => match sdeg(a).cmp(&sdeg(b)) {
            std::cmp::Ordering::Equal => cmp_slex(a, b),
            o => o,
        },
        Order::GrevLex => match sdeg(a).cmp(&sdeg(b)) {
            std::cmp::Ordering::Equal => {
                // 從最高變量往低：第一個指數不同處，指數小者為大
                let va = a.last().map(|(v, _)| *v).unwrap_or(0);
                let vb = b.last().map(|(v, _)| *v).unwrap_or(0);
                let hi = va.max(vb);
                for v in (0..=hi).rev() {
                    let (ea, eb) = (sget(a, v), sget(b, v));
                    if ea != eb {
                        return if ea < eb {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Less
                        };
                    }
                }
                std::cmp::Ordering::Equal
            }
            o => o,
        },
    }
}

fn cmp_slex(a: &SMono, b: &SMono) -> std::cmp::Ordering {
    let va = a.last().map(|(v, _)| *v).unwrap_or(0);
    let vb = b.last().map(|(v, _)| *v).unwrap_or(0);
    let hi = va.max(vb);
    for v in 0..=hi {
        let (ea, eb) = (sget(a, v), sget(b, v));
        if ea != eb {
            return ea.cmp(&eb);
        }
    }
    std::cmp::Ordering::Equal
}

fn sdivides(a: &SMono, b: &SMono) -> bool {
    for &(v, e) in a {
        if sget(b, v) < e {
            return false;
        }
    }
    true
}

fn smul(a: &SMono, b: &SMono) -> SMono {
    let mut out = a.clone();
    for &(v, e) in b {
        match out.binary_search_by_key(&v, |&(vv, _)| vv) {
            Ok(i) => out[i].1 += e,
            Err(i) => out.insert(i, (v, e)),
        }
    }
    out
}

fn sdiv(a: &SMono, b: &SMono) -> SMono {
    let mut out = a.clone();
    for &(v, e) in b {
        let i = out.binary_search_by_key(&v, |&(vv, _)| vv).expect("不整除");
        out[i].1 -= e;
        if out[i].1 == 0 {
            out.remove(i);
        }
    }
    out
}

/// 多元除法（快速版）：f 對 [g1..gk] 取餘式。
/// 餘式 r 滿足：f = Σ qᵢgᵢ + r，且 r 的任一單項式不被任何 LM(gᵢ) 整除。
pub fn div_rem(f: &Poly, gs: &[Poly], ord: Order) -> Poly {
    use std::collections::HashMap;
    // 預計算除子首項（稀疏表示）
    let mut divs: Vec<(SMono, Frac, &Poly)> = vec![];
    for g in gs {
        if let Some((m, c)) = g.lt(ord) {
            if m.is_empty() || m.iter().all(|&e| e == 0) {
                // 非零常數除子 ⇒ 整除一切
                return Poly::zero();
            }
            divs.push((to_sparse(&m), c, g));
        }
    }
    // 索引：首項的最小變量 → 除子編號
    let mut index: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, (lm, _, _)) in divs.iter().enumerate() {
        let first = lm[0].0;
        index.entry(first).or_default().push(i);
    }
    // 被除式項表
    let mut terms: HashMap<SMono, Frac> = f
        .terms
        .iter()
        .map(|(m, c)| (to_sparse(m), *c))
        .collect();
    let mut rem: Vec<(Mono, Frac)> = vec![];
    let mut guard = 0usize;
    while !terms.is_empty() {
        guard += 1;
        if std::env::var("GB_DBG").is_ok() && guard % 20_000 == 0 {
            eprintln!("    div_rem step {} terms {}", guard, terms.len());
        }
        assert!(guard < 2_000_000, "除法步數超限");
        // 找當前最大單項式
        let (mk, mc) = {
            let mut best: Option<(&SMono, &Frac)> = None;
            for (k, c) in terms.iter() {
                best = Some(match best {
                    None => (k, c),
                    Some((bk, _)) => {
                        if cmp_sparse(k, bk, ord) == std::cmp::Ordering::Greater {
                            (k, c)
                        } else {
                            best.unwrap()
                        }
                    }
                });
            }
            let (k, c) = best.expect("非空");
            (k.clone(), *c)
        };
        // 找可整除的除子（候選 = 首項最小變量出現在 mk 中）
        let mut found: Option<usize> = None;
        'search: for &(v, _) in mk.iter() {
            if let Some(cands) = index.get(&v) {
                for &di in cands {
                    if sdivides(&divs[di].0, &mk) {
                        found = Some(di);
                        break 'search;
                    }
                }
            }
        }
        match found {
            Some(di) => {
                let (lm, lc, g) = &divs[di];
                let qm = sdiv(&mk, lm);
                let qc = mc.div(lc);
                for (m2, c2) in &g.terms {
                    let pm = smul(&qm, &to_sparse(m2));
                    let pc = qc.mul(c2).neg();
                    let e = terms.entry(pm).or_insert(Frac::ZERO);
                    *e = e.add(&pc);
                    if e.is_zero() {
                        // 需先取出鍵再刪（借用問題：e 借用 terms）
                        // 改用標記後清理
                    }
                }
                terms.retain(|_, c| !c.is_zero());
            }
            None => {
                terms.remove(&mk);
                rem.push((from_sparse(&mk), mc));
            }
        }
    }
    Poly::from_terms(rem)
}

/// S-多項式：S(f,g) = (L/LT(f))·f − (L/LT(g))·g，其中 L = lcm(LM(f), LM(g))。— 優化 with_capacity
pub fn spoly(f: &Poly, g: &Poly, ord: Order) -> Poly {
    let (mf, cf) = f.lt(ord).unwrap();
    let (mg, cg) = g.lt(ord).unwrap();
    let l = mono_lcm(&mf, &mg);
    let q1 = Poly { terms: vec![(mono_div(&l, &mf), Frac::ONE.div(&cf))] };
    let q2 = Poly { terms: vec![(mono_div(&l, &mg), Frac::ONE.div(&cg))] };
    q1.mul(f).sub(&q2.mul(g))
}

/// 實際使用：poly 文件清單與優化統計 — 零依賴
pub fn poly_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("poly.rs", "多變量多項式：單項式序 + 除法 + S-多項式 — 優化 with_capacity", "core/src/poly.rs"),
        ("frac.rs", "有理數 ℚ — poly 依賴", "core/src/frac.rs"),
    ]
}
pub fn poly_stats(p: &Poly) -> String {
    let mut out = String::with_capacity(128);
    out.push_str(&format!("Poly terms={} deg_max={}\n", p.terms.len(), p.terms.iter().map(|(m,_)| mono_deg(m)).max().unwrap_or(0)));
    out
}


