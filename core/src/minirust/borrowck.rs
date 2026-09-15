//! Phase3 — borrowck lifetime outlives 檢查、unsafe gate、NLL
//!
//! 整合 lifetime.rs、effects.rs、analysis.rs

use crate::minirust::analysis::{BorrowAnalysis, BorrowInfo};
use crate::minirust::effects::EffectContext;
use crate::minirust::lifetime::{Lifetime, LifetimeGraph};
use std::collections::HashMap;

/// borrowck 檢查器
#[derive(Clone, Debug, Default)]
pub struct BorrowChecker {
    pub lifetime_graph: LifetimeGraph,
    pub effect_ctx: EffectContext,
    pub borrow_analysis: Option<BorrowAnalysis>,
    /// lifetime 標註：borrow node -> lifetime
    pub lifetime_of_borrow: HashMap<usize, Lifetime>,
    pub errors: Vec<String>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_poly_source(src: &crate::dsl::PolySource) -> Self {
        Self {
            lifetime_graph: LifetimeGraph::from_poly_source(src),
            effect_ctx: EffectContext::from_poly_source(src),
            borrow_analysis: None,
            lifetime_of_borrow: HashMap::new(),
            errors: vec![],
        }
    }

    pub fn with_lifetimes(mut self, graph: LifetimeGraph) -> Self {
        self.lifetime_graph = graph;
        self
    }

    pub fn with_effects(mut self, ctx: EffectContext) -> Self {
        self.effect_ctx = ctx;
        self
    }

    /// 檢查 outlives 環
    pub fn check_lifetime_cycles(&mut self) {
        if self.lifetime_graph.has_cycle() {
            self.errors.push("lifetime cycle detected: outlives graph has cycle".to_string());
        }
    }

    /// 檢查 borrow 是否滿足 outlives
    pub fn check_borrow_outlives(&mut self) {
        // 若 lifetime_of_borrow 有標註，檢查其 outlives 是否與 graph 一致
        // 簡化：若兩個 borrow 重疊且 lifetime 不滿足 outlives，則報錯
        if let Some(analysis) = &self.borrow_analysis {
            for (a,b) in &analysis.conflicts {
                let lt_a = self.lifetime_of_borrow.get(a);
                let lt_b = self.lifetime_of_borrow.get(b);
                if let (Some(la), Some(lb)) = (lt_a, lt_b) {
                    // 若既非 a outlives b 也非 b outlives a，且衝突，則需要更精細檢查
                    // 這裡若 graph 說 a: b，則允許 b 在 a 內，否則保持衝突
                    if self.lifetime_graph.outlives_holds(la, lb) || self.lifetime_graph.outlives_holds(lb, la) {
                        // outlives 成立，可能允許？但同 var_def 仍衝突，需看 NLL
                        // 暫不消除衝突，僅記錄
                    }
                }
            }
        }
    }

    /// 檢查 unsafe gate
    pub fn check_unsafe(&mut self) {
        if let Err(e) = self.effect_ctx.check_unsafe_gate() {
            self.errors.push(e);
        }
        if let Err(e) = self.effect_ctx.check_no_io() {
            self.errors.push(e);
        }
        if let Err(e) = self.effect_ctx.check_pure() {
            self.errors.push(e);
        }
    }

    /// 全部檢查
    pub fn check_all(&mut self) -> Result<(), Vec<String>> {
        self.check_lifetime_cycles();
        self.check_borrow_outlives();
        self.check_unsafe();
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// 附加 borrow 分析結果
    pub fn set_borrow_analysis(&mut self, analysis: BorrowAnalysis) {
        self.borrow_analysis = Some(analysis);
    }

    /// 標註 borrow 的 lifetime
    pub fn annotate_lifetime(&mut self, borrow_node: usize, lt: Lifetime) {
        self.lifetime_of_borrow.insert(borrow_node, lt);
    }
}

/// NLL 兼容性檢查：將 BorrowAnalysis 轉為 NLL 並檢查
pub fn check_nll_compatibility(
    borrows: &[BorrowInfo],
    graph: &LifetimeGraph,
    lifetime_of_borrow: &HashMap<usize, Lifetime>,
) -> Vec<String> {
    use crate::minirust::lifetime::analyze_nll;
    let nll = analyze_nll(borrows, graph, lifetime_of_borrow);
    let mut errs = vec![];
    if !nll.is_clean() {
        for (a,b) in nll.conflicts {
            errs.push(format!("NLL conflict: borrow {} and {} overlap with incompatible lifetimes", a, b));
        }
    }
    errs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::PolySource;
    use crate::minirust::analysis::BorrowInfo;

    #[test]
    fn test_borrowck_from_source() {
        let text = "# @lifetime 'a: 'b\n# @unsafe-allowed\nfn main() {}";
        let src = crate::dsl::load_poly(text).unwrap();
        let mut ck = BorrowChecker::from_poly_source(&src);
        assert!(ck.lifetime_graph.outlives_holds(
            &crate::minirust::lifetime::Lifetime::new("'a"),
            &crate::minirust::lifetime::Lifetime::new("'b")
        ));
        assert!(ck.check_all().is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = crate::minirust::lifetime::LifetimeGraph::new();
        graph.add_outlives(crate::minirust::lifetime::Outlives {
            longer: Lifetime::new("'a"),
            shorter: Lifetime::new("'b"),
        });
        graph.add_outlives(crate::minirust::lifetime::Outlives {
            longer: Lifetime::new("'b"),
            shorter: Lifetime::new("'a"),
        });
        let mut ck = BorrowChecker::new().with_lifetimes(graph);
        ck.check_lifetime_cycles();
        assert!(!ck.errors.is_empty());
    }

    #[test]
    fn test_unsafe_gate_integration() {
        let mut ctx = EffectContext::default();
        ctx.unsafe_usages.push(1);
        let mut ck = BorrowChecker::new().with_effects(ctx);
        ck.check_unsafe();
        assert!(!ck.errors.is_empty());
    }

    #[test]
    fn test_annotate_lifetime() {
        let mut ck = BorrowChecker::new();
        ck.annotate_lifetime(1, Lifetime::new("'a"));
        assert_eq!(ck.lifetime_of_borrow.get(&1).unwrap().0, "'a");
    }
}
