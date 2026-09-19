//! PolyCache — Phase A 增量管線：hash(src) → (n_vars, groebner_basis, timestamp)
//! 目標：文件改動僅重算受影響節點，避免全量 F4/F5

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::poly::Poly;

#[derive(Clone, Debug)]
pub struct CachedEntry {
    pub n_vars: usize,
    pub n_polys: usize,
    pub groebner_len: usize,
    pub groebner_hash: u64,
    pub timestamp: u64,
    pub qap_verified: bool,
}

pub struct PolyCache {
    map: HashMap<u64, CachedEntry>,
    pub hits: usize,
    pub misses: usize,
}

impl PolyCache {
    pub fn len(&self) -> usize { self.map.len() }
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn hash_src(src: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        src.hash(&mut hasher);
        hasher.finish()
    }

    pub fn hash_polys(polys: &[Poly]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for p in polys {
            // 簡化：hash 項數 + 首項係數
            p.terms.len().hash(&mut hasher);
            if let Some((_, c)) = p.terms.first() {
                format!("{:?}", c).hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    pub fn get(&mut self, src: &str) -> Option<&CachedEntry> {
        let h = Self::hash_src(src);
        if let Some(e) = self.map.get(&h) {
            self.hits += 1;
            Some(e)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, src: &str, n_vars: usize, n_polys: usize, groebner: &[Poly], qap_verified: bool) {
        let h = Self::hash_src(src);
        let gh = Self::hash_polys(groebner);
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.map.insert(
            h,
            CachedEntry {
                n_vars,
                n_polys,
                groebner_len: groebner.len(),
                groebner_hash: gh,
                timestamp: ts,
                qap_verified,
            },
        );
    }

    pub fn stats(&self) -> String {
        format!(
            "cache: {} entries, hits={} misses={} hit_rate={:.1}%",
            self.map.len(),
            self.hits,
            self.misses,
            if self.hits + self.misses == 0 {
                0.0
            } else {
                self.hits as f64 / (self.hits + self.misses) as f64 * 100.0
            }
        )
    }

    pub fn should_recompute(&mut self, src: &str) -> bool {
        self.get(src).is_none()
    }
}

impl Default for PolyCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn poly_cache_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![("poly_cache.rs", "PolyCache 增量管線 — Phase A", "core/src/poly_cache.rs")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = PolyCache::new();
        let src = "fn sqr(x: i32) -> i32 { x * x }";
        assert!(cache.get(src).is_none());
        assert_eq!(cache.misses, 1);
        cache.insert(src, 10, 20, &[], true);
        assert!(cache.get(src).is_some());
        assert_eq!(cache.hits, 1);
        let entry = cache.get(src).unwrap();
        assert_eq!(entry.n_vars, 10);
        assert!(entry.qap_verified);
    }

    #[test]
    fn test_hash_stability() {
        let s1 = "fn a() {}";
        let s2 = "fn a() {}";
        let s3 = "fn b() {}";
        assert_eq!(PolyCache::hash_src(s1), PolyCache::hash_src(s2));
        assert_ne!(PolyCache::hash_src(s1), PolyCache::hash_src(s3));
    }
}
