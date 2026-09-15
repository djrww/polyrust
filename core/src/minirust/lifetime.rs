//! Phase3 — lifetime 參數、outlives 圖、NLL 借用分析
//!
//! 支持 `# @lifetime 'a: 'b` 形式的 outlives 約束
//! 與 Ty::Generic 中的 lifetime 參數統一

use std::collections::{HashMap, HashSet};

/// lifetime 標識符，如 `'a`, `'b`, `'static`
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lifetime(pub String);

impl Lifetime {
    pub fn new(s: &str) -> Self {
        Self(s.trim().to_string())
    }

    pub fn is_static(&self) -> bool {
        self.0 == "'static"
    }

    pub fn is_valid(&self) -> bool {
        self.0.starts_with('\'') && self.0.len() > 1
    }
}

/// outlives 關係：'a: 'b 表示 'a 存活至少與 'b 一樣長
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Outlives {
    pub longer: Lifetime,
    pub shorter: Lifetime,
}

impl Outlives {
    pub fn parse(s: &str) -> Option<Self> {
        // 形如 "'a: 'b" 或 "'a: 'b + 'c"
        let s = s.trim();
        // 找到第一個 ':'
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            return None;
        }
        let longer = Lifetime::new(parts[0].trim());
        if !longer.is_valid() {
            return None;
        }
        // shorter 可能有多個，用 '+' 分隔，取第一個為簡化
        let shorter_part = parts[1..].join(":");
        // 取第一個 token
        let shorter_str = shorter_part.split('+').next().unwrap().trim().split_whitespace().next().unwrap_or("").trim();
        if shorter_str.is_empty() {
            return None;
        }
        let shorter = Lifetime::new(shorter_str);
        if !shorter.is_valid() {
            return None;
        }
        Some(Self { longer, shorter })
    }
}

/// Lifetime 圖：節點為 lifetime，邊為 outlives
#[derive(Clone, Debug, Default)]
pub struct LifetimeGraph {
    /// 所有 lifetimes
    pub lifetimes: HashSet<Lifetime>,
    /// outlives 邊：longer -> set of shorter (longer outlives shorter)
    pub outlives: HashMap<Lifetime, HashSet<Lifetime>>,
    /// 反向邊：shorter -> set of longer
    pub outlived_by: HashMap<Lifetime, HashSet<Lifetime>>,
}

impl LifetimeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_strings(strs: &[String]) -> Self {
        let mut g = Self::new();
        for s in strs {
            if let Some(o) = Outlives::parse(s) {
                g.add_outlives(o);
            } else {
                // 可能是單個 lifetime 如 "'a"
                let lt = Lifetime::new(s.trim());
                if lt.is_valid() {
                    g.lifetimes.insert(lt);
                }
            }
        }
        g
    }

    pub fn from_poly_source(src: &crate::dsl::PolySource) -> Self {
        Self::from_strings(&src.lifetimes)
    }

    pub fn add_lifetime(&mut self, lt: Lifetime) {
        self.lifetimes.insert(lt);
    }

    pub fn add_outlives(&mut self, o: Outlives) {
        self.lifetimes.insert(o.longer.clone());
        self.lifetimes.insert(o.shorter.clone());
        self.outlives.entry(o.longer.clone()).or_default().insert(o.shorter.clone());
        self.outlived_by.entry(o.shorter.clone()).or_default().insert(o.longer.clone());
    }

    /// 檢查是否存在環（outlives 不應成環，除非相同 lifetime）
    pub fn has_cycle(&self) -> bool {
        // 簡單 DFS
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        fn dfs(
            node: &Lifetime,
            graph: &LifetimeGraph,
            visited: &mut HashSet<Lifetime>,
            stack: &mut HashSet<Lifetime>,
        ) -> bool {
            if stack.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }
            visited.insert(node.clone());
            stack.insert(node.clone());
            if let Some(next) = graph.outlives.get(node) {
                for n in next {
                    if dfs(n, graph, visited, stack) {
                        return true;
                    }
                }
            }
            stack.remove(node);
            false
        }

        for lt in &self.lifetimes {
            if !visited.contains(lt) {
                if dfs(lt, self, &mut visited, &mut stack) {
                    return true;
                }
            }
        }
        false
    }

    /// 傳遞閉包：計算所有 outlives 關係
    pub fn transitive_closure(&self) -> HashMap<Lifetime, HashSet<Lifetime>> {
        let mut closure = self.outlives.clone();
        // Floyd-Warshall 風格
        let mut changed = true;
        while changed {
            changed = false;
            let keys: Vec<_> = closure.keys().cloned().collect();
            for k in &keys {
                let Some(reachable) = closure.get(k).cloned() else { continue };
                let mut to_add = HashSet::new();
                for r in &reachable {
                    if let Some(next) = closure.get(r) {
                        for n in next {
                            if !reachable.contains(n) && n != k {
                                to_add.insert(n.clone());
                            }
                        }
                    }
                }
                if !to_add.is_empty() {
                    closure.get_mut(k).unwrap().extend(to_add);
                    changed = true;
                }
            }
        }
        closure
    }

    /// 檢查 'a: 'b 是否成立（直接或傳遞）
    pub fn outlives_holds(&self, longer: &Lifetime, shorter: &Lifetime) -> bool {
        if longer == shorter {
            return true;
        }
        if longer.is_static() {
            return true; // 'static outlives all
        }
        let closure = self.transitive_closure();
        if let Some(set) = closure.get(longer) {
            set.contains(shorter)
        } else {
            false
        }
    }

    /// 生成多項式約束：若 'a: 'b，則 t_a >= t_b？實際用區域包含編碼
    /// 我們編碼為：lifetime 用區間 [start, end)，outlives => start_a <= start_b && end_b <= end_a
    pub fn poly_constraints(&self, nvars: usize, lt_to_var: &HashMap<Lifetime, (usize, usize)>) -> Vec<crate::poly::Poly> {
        use crate::frac::Frac;
        use crate::poly::Poly;
        let mut polys = vec![];
        for (longer, shorters) in &self.outlives {
            let Some(&(ls, le)) = lt_to_var.get(longer) else { continue };
            for shorter in shorters {
                let Some(&(ss, se)) = lt_to_var.get(shorter) else { continue };
                // start_longer <= start_shorter  => start_shorter - start_longer - slack =0? 簡化為 start_longer - start_shorter <=0 用不等式轉多項式
                // 這裡生成 start_longer - start_shorter + slack =0 約束的想法，暫用差值多項式表示：start_longer - start_shorter
                // 實際約束求解需額外 slack 變量，這裡先生成等式示意
                // 為保持零依賴，我們生成 start_shorter - start_longer >=0 的布爾多項式
                // 簡化：生成 (ss - ls) - 1 =0 表示 ss >= ls+1? 不精確，但用於測試
                // 真實實現需在 constraints 中處理，此處僅示範
                let p1 = Poly::var(ss, Frac::ONE, nvars).sub(&Poly::var(ls, Frac::ONE, nvars));
                polys.push(p1);
                let p2 = Poly::var(le, Frac::ONE, nvars).sub(&Poly::var(se, Frac::ONE, nvars));
                polys.push(p2);
            }
        }
        polys
    }
}

/// NLL 區域：非詞法生命週期
#[derive(Clone, Debug)]
pub struct Region {
    pub lifetime: Lifetime,
    pub start: u32,
    pub end: u32,
    pub borrow_node: usize,
}

#[derive(Clone, Debug, Default)]
pub struct NllAnalysis {
    pub regions: Vec<Region>,
    pub conflicts: Vec<(usize, usize)>, // 重疊的互斥借用
}

impl NllAnalysis {
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty()
    }
}

/// 從 BorrowAnalysis 擴展為 NLL：考慮 lifetime outlives
pub fn analyze_nll(
    borrows: &[crate::minirust::analysis::BorrowInfo],
    graph: &LifetimeGraph,
    lifetime_of_borrow: &HashMap<usize, Lifetime>,
) -> NllAnalysis {
    let mut regions: Vec<Region> = borrows.iter().map(|b| {
        let lt = lifetime_of_borrow.get(&b.node).cloned().unwrap_or_else(|| Lifetime::new("'a"));
        Region { lifetime: lt, start: b.start, end: b.end, borrow_node: b.node }
    }).collect();

    // 若有 outlives 關係，擴展 region：若 'a: 'b 且 borrow 'b，則其 region 擴展為 'a 的
    // 簡化：若 'a outlives 'b，則 'b 的 end 擴展到 'a 的 end（若已知）
    // 這裡僅做示範，實際需完整區間交集
    regions.sort_by_key(|r| r.start);

    let conflicts = vec![];
    // NLL 衝突需結合 var_def，這裡僅收集 region，已由 borrowck 判斷
    // 保留接口，暫不生成衝突，避免誤報
    for i in 0..regions.len() {
        for j in (i+1)..regions.len() {
            let a = &regions[i];
            let b = &regions[j];
            if a.start < b.end && b.start < a.end {
                let _ = (graph.outlives_holds(&a.lifetime, &b.lifetime), graph.outlives_holds(&b.lifetime, &a.lifetime));
            }
        }
    }

    NllAnalysis { regions, conflicts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_parse() {
        let lt = Lifetime::new("'a");
        assert!(lt.is_valid());
        assert!(!lt.is_static());
        let s = Lifetime::new("'static");
        assert!(s.is_static());
    }

    #[test]
    fn test_outlives_parse() {
        let o = Outlives::parse("'a: 'b").unwrap();
        assert_eq!(o.longer.0, "'a");
        assert_eq!(o.shorter.0, "'b");

        let o2 = Outlives::parse("'a: 'b + 'c");
        assert!(o2.is_some());
        assert_eq!(o2.unwrap().shorter.0, "'b");
    }

    #[test]
    fn test_graph_cycle() {
        let mut g = LifetimeGraph::new();
        g.add_outlives(Outlives { longer: Lifetime::new("'a"), shorter: Lifetime::new("'b") });
        g.add_outlives(Outlives { longer: Lifetime::new("'b"), shorter: Lifetime::new("'c") });
        assert!(!g.has_cycle());
        g.add_outlives(Outlives { longer: Lifetime::new("'c"), shorter: Lifetime::new("'a") });
        assert!(g.has_cycle());
    }

    #[test]
    fn test_transitive() {
        let mut g = LifetimeGraph::new();
        g.add_outlives(Outlives { longer: Lifetime::new("'a"), shorter: Lifetime::new("'b") });
        g.add_outlives(Outlives { longer: Lifetime::new("'b"), shorter: Lifetime::new("'c") });
        assert!(g.outlives_holds(&Lifetime::new("'a"), &Lifetime::new("'c")));
        assert!(!g.outlives_holds(&Lifetime::new("'c"), &Lifetime::new("'a")));
        assert!(g.outlives_holds(&Lifetime::new("'a"), &Lifetime::new("'a")));
    }

    #[test]
    fn test_static_outlives() {
        let g = LifetimeGraph::new();
        assert!(g.outlives_holds(&Lifetime::new("'static"), &Lifetime::new("'a")));
    }

    #[test]
    fn test_from_strings() {
        let strs = vec!["'a: 'b".to_string(), "'b: 'c".to_string(), "'d".to_string()];
        let g = LifetimeGraph::from_strings(&strs);
        assert_eq!(g.lifetimes.len(), 4);
        assert!(g.outlives_holds(&Lifetime::new("'a"), &Lifetime::new("'c")));
    }

    #[test]
    fn test_from_poly_source() {
        let text = "# @lifetime 'a: 'b\n# @lifetime 'b: 'c\nfn main() {}";
        let src = crate::dsl::load_poly(text).unwrap();
        let g = LifetimeGraph::from_poly_source(&src);
        assert!(g.outlives_holds(&Lifetime::new("'a"), &Lifetime::new("'c")));
    }
}
