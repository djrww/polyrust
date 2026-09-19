//! Incremental Cache — Phase B 性能硬化：type_universe LRU + constraints_v2 增量重算
//! 目標：文件改動僅重算受影響節點，type_universe 緩存避免重複 unify

use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct TypeUniverseCacheEntry {
    pub n_types: usize,
    pub type_hash: u64,
    pub per_node_bits: Vec<(usize, String, usize)>,
    pub timestamp: u64,
}

pub struct TypeUniverseLruCache {
    map: HashMap<u64, TypeUniverseCacheEntry>,
    order: VecDeque<u64>, // LRU order, front = most recent
    capacity: usize,
    pub hits: usize,
    pub misses: usize,
}

impl TypeUniverseLruCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity,
            hits: 0,
            misses: 0,
        }
    }

    pub fn hash_source(source: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&mut self, source: &str) -> Option<&TypeUniverseCacheEntry> {
        let h = Self::hash_source(source);
        if self.map.contains_key(&h) {
            self.hits += 1;
            // move to front
            self.order.retain(|&x| x != h);
            self.order.push_front(h);
            self.map.get(&h)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, source: &str, n_types: usize, per_node_bits: Vec<(usize, String, usize)>) {
        let h = Self::hash_source(source);
        let entry = TypeUniverseCacheEntry {
            n_types,
            type_hash: h,
            per_node_bits,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        };
        if self.map.len() >= self.capacity {
            if let Some(old) = self.order.pop_back() {
                self.map.remove(&old);
            }
        }
        self.map.insert(h, entry);
        self.order.retain(|&x| x != h);
        self.order.push_front(h);
    }

    pub fn stats(&self) -> String {
        format!(
            "type_universe_lru: {} entries cap={} hits={} misses={} hit_rate={:.1}%",
            self.map.len(),
            self.capacity,
            self.hits,
            self.misses,
            if self.hits + self.misses == 0 { 0.0 } else { self.hits as f64 / (self.hits + self.misses) as f64 * 100.0 }
        )
    }

    pub fn len(&self) -> usize { self.map.len() }
}

/// Constraints V2 Incremental：僅重算變更節點的 type_bits
#[derive(Clone, Debug)]
pub struct NodeChange {
    pub node_id: usize,
    pub old_bits_len: usize,
    pub new_bits_len: usize,
    pub changed: bool,
}

pub fn diff_node_bits(old: &[(usize, String, usize)], new: &[(usize, String, usize)]) -> Vec<NodeChange> {
    let mut changes = Vec::new();
    let mut old_map: HashMap<usize, usize> = HashMap::new();
    for (id, _kind, bits_len) in old {
        old_map.insert(*id, *bits_len);
    }
    for (id, _kind, new_len) in new {
        if let Some(old_len) = old_map.get(id) {
            if old_len != new_len {
                changes.push(NodeChange { node_id: *id, old_bits_len: *old_len, new_bits_len: *new_len, changed: true });
            }
        } else {
            changes.push(NodeChange { node_id: *id, old_bits_len: 0, new_bits_len: *new_len, changed: true });
        }
    }
    changes
}

pub fn incremental_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![("incremental_cache.rs", "Incremental Cache LRU + diff — Phase B", "core/src/incremental_cache.rs")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_hit_miss() {
        let mut cache = TypeUniverseLruCache::new(2);
        assert!(cache.get("a").is_none());
        cache.insert("a", 7, vec![(0, "struct".to_string(), 7)]);
        assert!(cache.get("a").is_some());
        assert_eq!(cache.hits, 1);
        cache.insert("b", 8, vec![]);
        cache.insert("c", 9, vec![]);
        // capacity 2, a should be evicted
        assert!(cache.get("a").is_none());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_diff() {
        let old = vec![(0, "struct".to_string(), 7), (1, "enum".to_string(), 8)];
        let new = vec![(0, "struct".to_string(), 10), (1, "enum".to_string(), 8), (2, "fn".to_string(), 7)];
        let diff = diff_node_bits(&old, &new);
        assert_eq!(diff.len(), 2); // node 0 changed, node 2 new
        assert!(diff.iter().any(|c| c.node_id == 0 && c.changed));
        assert!(diff.iter().any(|c| c.node_id == 2));
    }

    #[test]
    fn test_stats() {
        let mut cache = TypeUniverseLruCache::new(10);
        cache.insert("x", 7, vec![]);
        let _ = cache.get("x");
        let s = cache.stats();
        assert!(s.contains("hits=1"));
    }
}
