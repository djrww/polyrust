//! QAP（Quadratic Arithmetic Program）：
//! R1CS（A·z ∘ B·z = C·z）→ 經 Lagrange 插值 → 多項式形式
//! a(t)·b(t) − c(t) ≡ 0 (mod Z(t))，Z 為定義域上的 vanishing 多項式。
//! 定理 8：z 是 R1CS 見證 ⟺ Z | a·b − c。

use crate::fp::{Fp, P};

/// 𝔽_p 上的一元多項式（低次到高次）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniPoly {
    pub c: Vec<Fp>,
}

impl UniPoly {
    pub fn zero() -> UniPoly {
        UniPoly { c: vec![] }
    }
    pub fn constant(k: Fp) -> UniPoly {
        if k.is_zero() {
            UniPoly::zero()
        } else {
            UniPoly { c: vec![k] }
        }
    }
    pub fn from_coeffs(c: Vec<Fp>) -> UniPoly {
        let mut p = UniPoly { c };
        p.trim();
        p
    }
    fn trim(&mut self) {
        while self.c.last().map_or(false, |k| k.is_zero()) {
            self.c.pop();
        }
    }
    pub fn is_zero(&self) -> bool {
        self.c.is_empty()
    }
    pub fn deg(&self) -> Option<usize> {
        if self.is_zero() {
            None
        } else {
            Some(self.c.len() - 1)
        }
    }
    pub fn add(&self, o: &UniPoly) -> UniPoly {
        let n = self.c.len().max(o.c.len());
        let mut c = Vec::with_capacity(n);
        for i in 0..n {
            let a = self.c.get(i).copied().unwrap_or(Fp::zero());
            let b = o.c.get(i).copied().unwrap_or(Fp::zero());
            c.push(a + b);
        }
        UniPoly::from_coeffs(c)
    }
    pub fn sub(&self, o: &UniPoly) -> UniPoly {
        self.add(&o.neg())
    }
    pub fn neg(&self) -> UniPoly {
        UniPoly::from_coeffs(self.c.iter().map(|k| k.neg()).collect())
    }
    pub fn mul(&self, o: &UniPoly) -> UniPoly {
        if self.is_zero() || o.is_zero() {
            return UniPoly::zero();
        }
        let mut c = vec![Fp::zero(); self.c.len() + o.c.len() - 1];
        for (i, &a) in self.c.iter().enumerate() {
            for (j, &b) in o.c.iter().enumerate() {
                c[i + j] = c[i + j] + a * b;
            }
        }
        UniPoly::from_coeffs(c)
    }
    pub fn scale(&self, k: Fp) -> UniPoly {
        UniPoly::from_coeffs(self.c.iter().map(|c| *c * k).collect())
    }
    pub fn eval(&self, x: Fp) -> Fp {
        let mut acc = Fp::zero();
        for &k in self.c.iter().rev() {
            acc = acc * x + k;
        }
        acc
    }
    /// 長除法。回傳 (商, 餘式)。
    pub fn divmod(&self, d: &UniPoly) -> (UniPoly, UniPoly) {
        assert!(!d.is_zero(), "除以零多項式");
        let dd = d.deg().unwrap();
        let inv_lead = d.c[dd].inv();
        let mut r = self.clone();
        let mut q = vec![Fp::zero(); self.c.len().saturating_sub(dd).max(1)];
        while r.deg().map_or(false, |rd| rd >= dd) {
            let rd = r.deg().unwrap();
            let shift = rd - dd;
            let k = r.c[rd] * inv_lead;
            q[shift] = k;
            // r -= k * x^shift * d
            for i in 0..=dd {
                let idx = i + shift;
                r.c[idx] = r.c[idx] - k * d.c[i];
            }
            r.trim();
        }
        (UniPoly::from_coeffs(q), r)
    }
}

/// Lagrange 插值：過點集 {(x_i, y_i)} 的唯一 ≤ m−1 次多項式。
pub fn lagrange_interpolate(points: &[(Fp, Fp)]) -> UniPoly {
    let m = points.len();
    let mut acc = UniPoly::zero();
    for (i, &(xi, yi)) in points.iter().enumerate() {
        // ℓ_i(t) = ∏_{j≠i} (t − x_j) / (x_i − x_j)
        let mut num = UniPoly::constant(Fp::one());
        let mut den = Fp::one();
        for (j, &(xj, _)) in points.iter().enumerate() {
            if i != j {
                num = num.mul(&UniPoly::from_coeffs(vec![xj.neg(), Fp::one()]));
                den = den * (xi - xj);
            }
        }
        acc = acc.add(&num.scale(yi * den.inv()));
    }
    let _ = m;
    acc
}

/// 稀疏線性形式：wire → 係數。
pub type Linear = Vec<(usize, Fp)>;

/// R1CS：m 條約束 (A_i·z)(B_i·z) = C_i·z，z = (1, w_1..w_{n-1})。
#[derive(Clone, Debug, Default)]
pub struct R1cs {
    pub n_wires: usize,
    pub constraints: Vec<(Linear, Linear, Linear)>, // (A, B, C)
    /// 中間導線的來源約束（生成見證用）
    pub intermediates: Vec<Option<usize>>,
}

impl R1cs {
    pub fn eval_linear(l: &Linear, z: &[Fp]) -> Fp {
        let mut acc = Fp::zero();
        for &(w, k) in l {
            acc = acc + k * z[w];
        }
        acc
    }

    /// 由（布爾）變量賦值構造完整見證（含中間導線）。
    pub fn witness(&self, vals: &[Fp]) -> Vec<Fp> {
        let mut z = vec![Fp::zero(); self.n_wires];
        z[0] = Fp::one();
        for (i, &v) in vals.iter().enumerate() {
            if i + 1 < self.n_wires {
                z[i + 1] = v;
            }
        }
        // 中間導線依約束順序求值（intermediates[ci] = 該約束定義的導線）
        for ci in 0..self.constraints.len() {
            if let Some(w) = self.intermediates.get(ci).copied().flatten() {
                let (a, b, _) = &self.constraints[ci];
                let v = R1cs::eval_linear(a, &z) * R1cs::eval_linear(b, &z);
                z[w] = v;
            }
        }
        z
    }
}

/// QAP：每條導線的 A/B/C 多項式 + vanishing 多項式 Z。
pub struct Qap {
    pub n_wires: usize,
    pub n_constraints: usize,
    /// wire j 的三條多項式（deg ≤ m−1）
    pub a: Vec<UniPoly>,
    pub b: Vec<UniPoly>,
    pub c: Vec<UniPoly>,
    pub z: UniPoly,
    pub domain: Vec<Fp>, // t_i = 1..=m
}

/// 由 R1CS 構造 QAP（定義域 H = {1..m}）。
/// 效率關鍵：Lagrange 基底 ℓ_i(t) = Z(t)/((t−t_i)·Z′(t_i)) 只算一次（每個 O(m)），
/// 每條導線的多項式 = 其非零係數的稀疏組裝 Σ_i k_i·ℓ_i。
pub fn qap_from_r1cs(r: &R1cs) -> Qap {
    let m = r.constraints.len();
    assert!(m >= 1, "R1CS 至少一條約束");
    let domain: Vec<Fp> = (1..=m).map(|i| Fp::from_i64(i as i64)).collect();
    // Z(t) = ∏_{i∈H} (t − i)
    let mut z = UniPoly::constant(Fp::one());
    for &t in &domain {
        z = z.mul(&UniPoly::from_coeffs(vec![t.neg(), Fp::one()]));
    }
    // 基底：ℓ_i = Z / (t − t_i) 正規化。Z′(t_i) = ∏_{j≠i}(t_i − t_j)
    let mut basis: Vec<UniPoly> = Vec::with_capacity(m);
    for (i, &ti) in domain.iter().enumerate() {
        let mut deriv = Fp::one();
        for (j, &tj) in domain.iter().enumerate() {
            if i != j {
                deriv = deriv * (ti - tj);
            }
        }
        let (q, rem) = z.divmod(&UniPoly::from_coeffs(vec![ti.neg(), Fp::one()]));
        debug_assert!(rem.is_zero(), "Z 的根除法應無餘式");
        basis.push(q.scale(deriv.inv()));
    }
    // 每條導線：稀疏組裝
    let mut a = vec![UniPoly::zero(); r.n_wires];
    let mut b = vec![UniPoly::zero(); r.n_wires];
    let mut c = vec![UniPoly::zero(); r.n_wires];
    for (i, (ca, cb, cc)) in r.constraints.iter().enumerate() {
        for &(w, k) in ca {
            if !k.is_zero() {
                a[w] = a[w].add(&basis[i].scale(k));
            }
        }
        for &(w, k) in cb {
            if !k.is_zero() {
                b[w] = b[w].add(&basis[i].scale(k));
            }
        }
        for &(w, k) in cc {
            if !k.is_zero() {
                c[w] = c[w].add(&basis[i].scale(k));
            }
        }
    }
    Qap { n_wires: r.n_wires, n_constraints: m, a, b, c, z, domain }
}

impl Qap {
    /// 見證導線多項式 a(t), b(t), c(t)。
    pub fn wire_polys(&self, z: &[Fp]) -> (UniPoly, UniPoly, UniPoly) {
        let mut a = UniPoly::zero();
        let mut b = UniPoly::zero();
        let mut c = UniPoly::zero();
        for w in 0..self.n_wires {
            if !z[w].is_zero() {
                a = a.add(&self.a[w].scale(z[w]));
                b = b.add(&self.b[w].scale(z[w]));
                c = c.add(&self.c[w].scale(z[w]));
            }
        }
        (a, b, c)
    }

    /// QAP 驗證：a·b − c ≡ 0 (mod Z)？
    pub fn verify(&self, z: &[Fp]) -> bool {
        let (a, b, c) = self.wire_polys(z);
        let p = a.mul(&b).sub(&c);
        if p.is_zero() {
            return true;
        }
        let (_, r) = p.divmod(&self.z);
        r.is_zero()
    }

    pub fn max_wire_degree(&self) -> usize {
        self.a
            .iter()
            .chain(self.b.iter())
            .chain(self.c.iter())
            .map(|p| p.deg().unwrap_or(0))
            .max()
            .unwrap_or(0)
    }
}

/// 有限的輔助：輸出域大小（供報告）。
pub fn prime() -> u64 {
    P
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unipoly_divmod() {
        // (t^2 − 1) ÷ (t − 1) = t + 1
        let p = UniPoly::from_coeffs(vec![Fp::from_i64(-1), Fp::zero(), Fp::one()]);
        let d = UniPoly::from_coeffs(vec![Fp::from_i64(-1), Fp::one()]);
        let (q, r) = p.divmod(&d);
        assert!(r.is_zero());
        assert_eq!(q.c, vec![Fp::one(), Fp::one()]);
    }

    #[test]
    fn test_lagrange() {
        // 過 (0,1),(1,3),(2,2)：唯一二次多項式
        let pts = vec![
            (Fp::zero(), Fp::from_i64(1)),
            (Fp::one(), Fp::from_i64(3)),
            (Fp::from_i64(2), Fp::from_i64(2)),
        ];
        let p = lagrange_interpolate(&pts);
        for (x, y) in &pts {
            assert_eq!(p.eval(*x), *y);
        }
        assert_eq!(p.deg(), Some(2));
    }

    #[test]
    fn test_qap_sqr() {
        // 約束：y = x·x（x = wire1, y = wire2）
        let r = R1cs {
            n_wires: 3,
            constraints: vec![(
                vec![(1, Fp::one())], // A = x
                vec![(1, Fp::one())], // B = x
                vec![(2, Fp::one())], // C = y
            )],
            intermediates: vec![None],
        };
        let q = qap_from_r1cs(&r);
        // 見證 x=3, y=9
        let z = vec![Fp::one(), Fp::from_i64(3), Fp::from_i64(9)];
        assert!(q.verify(&z));
        // 竄改 y=8 ⇒ 失敗
        let z_bad = vec![Fp::one(), Fp::from_i64(3), Fp::from_i64(8)];
        assert!(!q.verify(&z_bad));
    }
}
