//! 質域 𝔽_p，p = 2⁶¹ − 1（Mersenne 質數）。QAP 見 qap.rs。
//! u128 中間乘積保證無溢出（61+61 = 122 < 128）。

pub const P: u64 = (1u64 << 61) - 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Fp(pub u64);

impl Fp {
    pub const ZERO: Fp = Fp(0);
    pub const ONE: Fp = Fp(1);

    pub fn zero() -> Fp {
        Fp(0)
    }
    pub fn one() -> Fp {
        Fp(1)
    }
    pub fn from_i64(x: i64) -> Fp {
        let m = (x as i128).rem_euclid(P as i128) as u64;
        Fp(m)
    }
    #[allow(dead_code)] // 供單元測試使用
    pub fn from_frac(num: i128, den: i128) -> Fp {
        // 有理數 → 𝔽_p（den 與 p 互質；本專案 den | 小整數）
        let n = Fp::from_i64(num as i64);
        let d = Fp::from_i64(den as i64);
        n * d.inv()
    }
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
    pub fn is_one(&self) -> bool {
        self.0 == 1
    }
    /// 表示法符號（僅供顯示）：殘差 ∈ (P/2, P) 視為負數 −(P−v)。
    pub fn signum(&self) -> i32 {
        if self.0 == 0 {
            0
        } else if self.0 <= P / 2 {
            1
        } else {
            -1
        }
    }
    pub fn add(&self, o: &Fp) -> Fp {
        let s = self.0 + o.0;
        Fp(if s >= P { s - P } else { s })
    }
    pub fn sub(&self, o: &Fp) -> Fp {
        if self.0 >= o.0 {
            Fp(self.0 - o.0)
        } else {
            Fp(P - o.0 + self.0)
        }
    }
    pub fn neg(&self) -> Fp {
        if self.0 == 0 {
            Fp(0)
        } else {
            Fp(P - self.0)
        }
    }
    pub fn mul(&self, o: &Fp) -> Fp {
        Fp(((self.0 as u128 * o.0 as u128) % P as u128) as u64)
    }
    pub fn div(&self, o: &Fp) -> Fp {
        assert!(!o.is_zero(), "除以零");
        self.mul(&o.inv())
    }
    pub fn pow(self, mut e: u64) -> Fp {
        let mut r = Fp::one();
        let mut b = self;
        while e > 0 {
            if e & 1 == 1 {
                r = r * b;
            }
            b = b * b;
            e >>= 1;
        }
        r
    }
    pub fn inv(self) -> Fp {
        assert!(self.0 != 0, "0 無逆元");
        self.pow(P - 2)
    }
}

impl std::ops::Add for Fp {
    type Output = Fp;
    fn add(self, o: Fp) -> Fp {
        Fp::add(&self, &o)
    }
}
impl std::ops::Sub for Fp {
    type Output = Fp;
    fn sub(self, o: Fp) -> Fp {
        Fp::sub(&self, &o)
    }
}
impl std::ops::Mul for Fp {
    type Output = Fp;
    fn mul(self, o: Fp) -> Fp {
        Fp::mul(&self, &o)
    }
}
impl std::ops::Neg for Fp {
    type Output = Fp;
    fn neg(self) -> Fp {
        Fp::neg(&self)
    }
}
impl std::ops::AddAssign for Fp {
    fn add_assign(&mut self, o: Fp) {
        *self = Fp::add(self, &o);
    }
}

impl std::fmt::Display for Fp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 <= P / 2 {
            return write!(f, "{}", self.0);
        }
        // 嘗試小分數 n/d（d ≤ 36）的辨識，便於閱讀（如 inv(2) → 1/2）— 優化 with_capacity
        for d in 2u64..=36 {
            let n = ((self.0 as u128 * d as u128) % P as u128) as u64;
            if n > 0 && n <= 36 {
                return write!(f, "{}/{}", n, d);
            }
        }
        write!(f, "-{}", P - self.0)
    }
}

/// 實際使用：fp 文件清單 — 優化 with_capacity
pub fn fp_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("fp.rs", "質域 𝔽_p 2^61-1 — 優化無溢出乘法", "core/src/fp.rs"),
    ]
}
pub fn fp_stats() -> String {
    let mut s = String::with_capacity(64);
    s.push_str(&format!("Fp P=2^61-1={} ZERO={} ONE={}\n", P, Fp::ZERO.0, Fp::ONE.0));
    s
}


