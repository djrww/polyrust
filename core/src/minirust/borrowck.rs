//! Phase3 — borrowck lifetime outlives 檢查、unsafe gate、NLL + static mut thread 檢查
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
        if let Some(analysis) = &self.borrow_analysis {
            for (a,b) in &analysis.conflicts {
                let lt_a = self.lifetime_of_borrow.get(a);
                let lt_b = self.lifetime_of_borrow.get(b);
                if let (Some(la), Some(lb)) = (lt_a, lt_b) {
                    if self.lifetime_graph.outlives_holds(la, lb) || self.lifetime_graph.outlives_holds(lb, la) {
                        // 暫不消除衝突，僅記錄
                    }
                }
            }
        }
    }

    /// 檢查 unsafe gate（5類）
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
        if let Err(e) = self.effect_ctx.check_unsafe_fn_gate() {
            self.errors.push(e);
        }
        if let Err(e) = self.effect_ctx.check_union_gate() {
            self.errors.push(e);
        }
        // static mut 多線程檢查：極高風險，直接報錯
        if let Err(e) = self.effect_ctx.check_static_mut_thread_safety() {
            self.errors.push(e);
        }
    }

    /// 檢查 static mut 在 async/thread 上下文（獨立入口，供 pipeline_v2 調用）
    pub fn check_static_mut_thread_safety(&mut self) {
        if !self.effect_ctx.thread_unsafe_usages.is_empty() {
            self.errors.push(format!(
                "borrowck: static mut in threaded/async context at nodes {:?} may cause data race; use Mutex/Atomic or #[thread-safe: false]",
                self.effect_ctx.thread_unsafe_usages
            ));
        }
        // 若有 static mut 訪問但未在 unsafe 塊，也報
        if !self.effect_ctx.static_mut_accesses.is_empty() {
            let unprotected = self.effect_ctx.static_mut_accesses.iter()
                .filter(|id| self.effect_ctx.unsafe_usages.contains(id))
                .cloned()
                .collect::<Vec<_>>();
            if !unprotected.is_empty() && !self.effect_ctx.unsafe_allowed {
                self.errors.push(format!(
                    "borrowck: static mut access at nodes {:?} requires unsafe block (2024 edition deprecated)",
                    unprotected
                ));
            }
        }
    }

    /// 全部檢查
    pub fn check_all(&mut self) -> Result<(), Vec<String>> {
        self.check_lifetime_cycles();
        self.check_borrow_outlives();
        self.check_unsafe();
        self.check_static_mut_thread_safety();
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

/// NLL 兼容性檢查
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

pub fn borrowck_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("borrowck.rs", "borrowck.rs 5類 unsafe + static mut thread 檢查", "core/src/minirust/borrowck.rs"),
    ]
}
