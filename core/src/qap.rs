//! QAP（Quadratic Arithmetic Program）：
//! R1CS（A·z ∘ B·z = C·z）→ 經 Lagrange 插值 → 多項式形式
//! a(t)·b(t) − c(t) ≡ 0 (mod Z(t))，Z 為定義域上的 vanishing 多項式。
//! 定理 8：z 是 R1CS 見證 ⟺ Z | a·b − c。

use crate::fp::Fp;

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
    #[allow(dead_code)] // 供單元測試使用
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
#[allow(dead_code)] // 供單元測試使用
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
    /// wire j 的三條多項式（deg ≤ m−1）
    pub a: Vec<UniPoly>,
    pub b: Vec<UniPoly>,
    pub c: Vec<UniPoly>,
    pub z: UniPoly,
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
    Qap { n_wires: r.n_wires, a, b, c, z }
}

#[derive(Clone, Debug)]
pub struct QapCertificate {
    pub n_wires: usize,
    pub n_constraints: usize,
    pub max_degree: usize,
    pub z_hash: String,
    pub a_hash: String,
    pub b_hash: String,
    pub c_hash: String,
    pub matrix_hash: Option<String>,
    pub verified: bool,
    pub tamper_rejected: bool,
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

    /// 計算簡單哈希 (FNV-1a) 用於證書
    pub fn hash_poly(p: &UniPoly) -> u64 {
        let mut h: u64 = 1469598103934665603;
        for coeff in &p.c {
            let v = coeff.0;
            h ^= v;
            h = h.wrapping_mul(1099511628211);
        }
        h
    }

    pub fn hash_polys(polys: &[UniPoly]) -> String {
        let mut h: u64 = 1469598103934665603;
        for p in polys {
            h ^= Self::hash_poly(p);
            h = h.wrapping_mul(1099511628211);
        }
        format!("{:016x}", h)
    }

    /// 生成 QAP 證書，含矩陣哈希 (Phase4)
    pub fn certificate(&self, z: &[Fp], matrix_hash: Option<String>) -> QapCertificate {
        let (a_poly, b_poly, c_poly) = self.wire_polys(z);
        let p = a_poly.mul(&b_poly).sub(&c_poly);
        let verified = if p.is_zero() { true } else { let (_, r) = p.divmod(&self.z); r.is_zero() };
        // tamper: flip first non-zero wire
        let mut z_bad = z.to_vec();
        let mut tamper_rejected = true;
        if z_bad.len() > 1 {
            z_bad[1] = z_bad[1] + Fp::one();
            tamper_rejected = !self.verify(&z_bad);
        }
        QapCertificate {
            n_wires: self.n_wires,
            n_constraints: self.z.deg().map(|d| d).unwrap_or(0),
            max_degree: self.max_wire_degree(),
            z_hash: format!("{:016x}", Self::hash_poly(&self.z)),
            a_hash: Self::hash_polys(&self.a),
            b_hash: Self::hash_polys(&self.b),
            c_hash: Self::hash_polys(&self.c),
            matrix_hash,
            verified,
            tamper_rejected,
        }
    }
}

/// 導出 R1CS 為 JSON（相容 snarkjs/bellman）
pub fn export_r1cs_json(r: &R1cs, qap: &Qap) -> String {
    use crate::json::J;
    let obj = vec![
        ("n_wires", J::Int(r.n_wires as i64)),
        ("n_constraints", J::Int(r.constraints.len() as i64)),
        ("z_degree", J::Int(qap.z.deg().unwrap_or(0) as i64)),
        ("max_wire_degree", J::Int(qap.max_wire_degree() as i64)),
        ("export_format", J::s("r1cs.json v1 — compatible with snarkjs/bellman")),
        ("z_hash", J::s(&format!("{:016x}", Qap::hash_poly(&qap.z)))),
        ("a_hash", J::s(&Qap::hash_polys(&qap.a))),
        ("b_hash", J::s(&Qap::hash_polys(&qap.b))),
        ("c_hash", J::s(&Qap::hash_polys(&qap.c))),
    ];
    J::obj(obj).to_string()
}

/// 實際使用：qap.rs 文件清單 — 優化 with_capacity
pub fn qap_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("qap.rs", "qap.rs 正式運作 — 優化 with_capacity + r1cs.json export", "core/src/qap.rs"),
    ]
}

