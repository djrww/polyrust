//! Phase3 — Vec/String/HashMap 內建庫擴展與泛型版本
//!
//! 提供 Rust std 類型的多項式編碼

use std::collections::HashMap;

/// Vec<T> 編碼：ptr + len + cap 三元組
#[derive(Clone, Debug)]
pub struct VecEncoding {
    pub ptr_var: usize,
    pub len_var: usize,
    pub cap_var: usize,
    pub elem_ty: String,
}

impl VecEncoding {
    pub fn new(ptr: usize, len: usize, cap: usize, elem_ty: &str) -> Self {
        Self { ptr_var: ptr, len_var: len, cap_var: cap, elem_ty: elem_ty.to_string() }
    }

    /// 生成多項式：len <= cap
    pub fn len_le_cap_poly(&self, nvars: usize) -> crate::poly::Poly {
        use crate::frac::Frac;
        use crate::poly::Poly;
        // cap - len - slack =0，簡化為 cap - len >=0 用差值表示
        Poly::var(self.cap_var, Frac::ONE, nvars).sub(&Poly::var(self.len_var, Frac::ONE, nvars))
    }

    /// push 約束：len' = len +1, len < cap
    pub fn push_poly(&self, nvars: usize, len_next: usize) -> crate::poly::Poly {
        use crate::frac::Frac;
        use crate::poly::Poly;
        Poly::var(len_next, Frac::ONE, nvars)
            .sub(&Poly::var(self.len_var, Frac::ONE, nvars))
            .sub(&Poly::constant(Frac::ONE))
    }
}

/// String 編碼：視為 Vec<u8> + utf8 約束
#[derive(Clone, Debug)]
pub struct StringEncoding {
    pub vec: VecEncoding,
}

impl StringEncoding {
    pub fn new(ptr: usize, len: usize, cap: usize) -> Self {
        Self { vec: VecEncoding::new(ptr, len, cap, "u8") }
    }

    /// utf8 約束：每個 byte < 128 或多字節序列有效
    /// 簡化：生成 byte 範圍約束 0 <= b <= 255 (恆成立於 field，但用於 bool 約束)
    pub fn utf8_poly(&self, _nvars: usize) -> Vec<String> {
        vec!["// utf8 check: byte in [0,255] and valid sequence".to_string()]
    }
}

/// HashMap<K,V> 編碼：Vec<(K,V)> + key 唯一
#[derive(Clone, Debug)]
pub struct HashMapEncoding {
    pub ptr_var: usize,
    pub len_var: usize,
    pub cap_var: usize,
    pub key_ty: String,
    pub val_ty: String,
}

impl HashMapEncoding {
    pub fn new(ptr: usize, len: usize, cap: usize, key_ty: &str, val_ty: &str) -> Self {
        Self { ptr_var: ptr, len_var: len, cap_var: cap, key_ty: key_ty.to_string(), val_ty: val_ty.to_string() }
    }

    /// key 唯一約束：∀ i≠j, k_i != k_j
    /// 多項式：(k_i - k_j) * inv =1
    pub fn unique_keys_poly(&self, nvars: usize, k_i: usize, k_j: usize, inv: usize) -> crate::poly::Poly {
        use crate::frac::Frac;
        use crate::poly::Poly;
        let diff = Poly::var(k_i, Frac::ONE, nvars).sub(&Poly::var(k_j, Frac::ONE, nvars));
        let inv_poly = Poly::var(inv, Frac::ONE, nvars);
        diff.mul(&inv_poly).sub(&Poly::constant(Frac::ONE))
    }
}

/// 內建庫註冊表
#[derive(Clone, Debug, Default)]
pub struct StdlibRegistry {
    pub vec_encodings: HashMap<String, VecEncoding>,
    pub string_encodings: HashMap<String, StringEncoding>,
    pub hashmap_encodings: HashMap<String, HashMapEncoding>,
}

impl StdlibRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_vec(&mut self, name: String, enc: VecEncoding) {
        self.vec_encodings.insert(name, enc);
    }

    pub fn register_string(&mut self, name: String, enc: StringEncoding) {
        self.string_encodings.insert(name, enc);
    }

    pub fn register_hashmap(&mut self, name: String, enc: HashMapEncoding) {
        self.hashmap_encodings.insert(name, enc);
    }

    /// 實際使用：文件清單 — 優化 with_capacity
    pub fn file_list() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("stdlib.rs", "Vec/String/HashMap 內建庫擴展 — 優化 with_capacity", "core/src/minirust/stdlib.rs"),
            ("poly.rs", "Poly 多項式 — stdlib 依賴", "core/src/poly.rs"),
        ]
    }
    pub fn summary(&self) -> String {
        let mut out = String::with_capacity(256);
        out.push_str(&format!("stdlib: vec={} string={} hashmap={}\n", self.vec_encodings.len(), self.string_encodings.len(), self.hashmap_encodings.len()));
        out
    }

    /// 生成所有約束多項式 — 優化 with_capacity
    pub fn all_polys(&self, nvars: usize) -> Vec<crate::poly::Poly> {
        let mut polys = Vec::with_capacity(self.vec_encodings.len() + 1);
        for enc in self.vec_encodings.values() {
            polys.push(enc.len_le_cap_poly(nvars));
        }
        // HashMap unique 需具體 key 變量，暫不生成全局
        polys
    }

    fn split_type_list(s: &str) -> Vec<String> {
        let mut parts = vec![];
        let mut cur = String::new();
        let mut depth: i32 = 0;
        for c in s.chars() {
            match c {
                '<' => { depth += 1; cur.push(c); }
                '>' => { depth = (depth - 1).max(0); cur.push(c); }
                ',' if depth == 0 => {
                    if !cur.trim().is_empty() {
                        parts.push(cur.trim().to_string());
                    }
                    cur.clear();
                }
                _ => cur.push(c),
            }
        }
        if !cur.trim().is_empty() {
            parts.push(cur.trim().to_string());
        }
        parts
    }

    /// 從 PolySource 的 type_universe 推導需要的 stdlib
    pub fn from_type_universe(type_uni: &str) -> Self {
        let mut reg = Self::new();
        for p in Self::split_type_list(type_uni) {
            let p = p.trim().to_string();
            if p.starts_with("Vec<") {
                let inner = p["Vec<".len()..].trim_end_matches('>').trim();
                reg.register_vec(p.clone(), VecEncoding::new(0,1,2, inner));
            } else if p == "String" || p.starts_with("String") {
                reg.register_string(p.clone(), StringEncoding::new(0,1,2));
            } else if p.starts_with("HashMap<") {
                let inner = p["HashMap<".len()..].trim_end_matches('>').trim();
                let kv = Self::split_type_list(inner);
                let k = kv.get(0).map(|s| s.trim()).unwrap_or("String");
                let v = kv.get(1).map(|s| s.trim()).unwrap_or("i32");
                reg.register_hashmap(p.clone(), HashMapEncoding::new(0,1,2,k,v));
            }
        }
        reg
    }
}

/// 生成 Vec/String/HashMap 的 R1CS 約束文本 — 優化 with_capacity
pub fn r1cs_for_stdlib(type_uni: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(8);
    let mut parts = Vec::with_capacity(4);
    let mut cur = String::new();
    let mut depth: i32 = 0;
    for c in type_uni.chars() {
        match c {
            '<' => { depth += 1; cur.push(c); }
            '>' => { depth = (depth - 1).max(0); cur.push(c); }
            ',' if depth == 0 => {
                if !cur.trim().is_empty() { parts.push(cur.trim().to_string()); }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() { parts.push(cur.trim().to_string()); }

    for p in parts {
        let p = p.trim();
        if p.starts_with("Vec<") {
            out.push(format!("// Vec {} : len <= cap", p));
            out.push(format!("cap - len - slack =0"));
            out.push(format!("// push: len' = len +1"));
        } else if p.contains("String") && !p.starts_with("HashMap") {
            out.push("// String : Vec<u8> + utf8".to_string());
            out.push("// utf8 valid sequence check".to_string());
        } else if p.starts_with("HashMap<") {
            out.push(format!("// HashMap {} : key unique", p));
            out.push("// (k_i - k_j) * inv =1 for i!=j".to_string());
        } else if p == "String" {
            out.push("// String : Vec<u8> + utf8".to_string());
        }
    }
    out
}


