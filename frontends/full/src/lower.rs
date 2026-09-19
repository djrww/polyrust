// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Phase2 — 前端 lower.rs：封裝 core 的 lower 模組，提供完整 lowering 流水線
//! Phase3 — 擴展 trait/impl 方法表、lifetime、stdlib、async、contracts、effects、borrowck

use polyrust_core::minirust::ast_v2::ProgramV2;
use polyrust_core::minirust::lower::{
    lower_program, Lowered, parse_match_to_decision_tree, decision_tree_to_if_chain,
    parse_for_to_loop, for_loop_to_loop_text,
    lower_trait_impl_method_table, lower_lifetimes, lower_stdlib_usage,
};

pub use polyrust_core::minirust::lower::{MatchArm, MatchDecisionTree, ForLoop, product_poly_text, sum_poly_text, lower_body_text};

/// 前端 lowering 主入口
pub fn lower_full(src: &str) -> Result<Lowered, String> {
    let prog = ProgramV2::parse_v2(src)?;
    lower_program(prog)
}

/// 將 match 決策樹轉為 if 鏈（前端展示）
pub fn lower_match(src: &str) -> Result<String, String> {
    let tree = parse_match_to_decision_tree(src)?;
    Ok(decision_tree_to_if_chain(&tree))
}

/// 將 for 轉為 loop
pub fn lower_for(src: &str) -> Result<String, String> {
    let fl = parse_for_to_loop(src)?;
    Ok(for_loop_to_loop_text(&fl))
}

/// Phase3：完整 lowering 報告（含 trait/impl、lifetime、stdlib、async、contracts）
pub fn lowering_report(src: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Source:\n{}\n\n", src));

    // DSL 契約解析
    if let Ok(poly_src) = polyrust_core::dsl::load_poly(src) {
        out.push_str(&format!("# DSL contracts: fuel={:?}, invariants={:?}, pure={:?}, unsafe_allowed={}, lifetimes={:?}, no_io={}, type_universe={:?}\n",
            poly_src.fuel, poly_src.invariants, poly_src.pure, poly_src.unsafe_allowed, poly_src.lifetimes, poly_src.no_io, poly_src.type_universe));
        if !poly_src.lifetimes.is_empty() {
            let graph = polyrust_core::minirust::lifetime::LifetimeGraph::from_poly_source(&poly_src);
            out.push_str(&format!("# Lifetime graph: {} lifetimes, {} edges, has_cycle={}\n",
                graph.lifetimes.len(), graph.outlives.len(), graph.has_cycle()));
            for (longer, shorters) in &graph.outlives {
                for shorter in shorters {
                    out.push_str(&format!("#   {}: {} \n", longer.0, shorter.0));
                }
            }
        }
        if !poly_src.invariants.is_empty() || poly_src.fuel.is_some() {
            let contract = polyrust_core::minirust::contracts::LoopContract::from_poly_source(&poly_src);
            out.push_str(&format!("# Loop contract: fuel_or_default={}, invariants={:?}\n",
                contract.fuel_or_default(), contract.invariants));
            for poly in contract.fuel_poly_text(0) {
                out.push_str(&format!("#   {}\n", poly));
            }
            for poly in contract.invariant_poly_text(0) {
                out.push_str(&format!("#   {}\n", poly));
            }
        }
        let eff_ctx = polyrust_core::minirust::effects::EffectContext::from_poly_source(&poly_src);
        out.push_str(&format!("# Effect context: unsafe_allowed={}, no_io={}, pure={:?}\n",
            eff_ctx.unsafe_allowed, eff_ctx.no_io, eff_ctx.pure));
        if let Some(tu) = &poly_src.type_universe {
            let reg = polyrust_core::minirust::stdlib::StdlibRegistry::from_type_universe(tu);
            out.push_str(&format!("# Stdlib registry: vec={}, string={}, hashmap={}\n",
                reg.vec_encodings.len(), reg.string_encodings.len(), reg.hashmap_encodings.len()));
            for poly in polyrust_core::minirust::stdlib::r1cs_for_stdlib(tu) {
                out.push_str(&format!("#   {}\n", poly));
            }
        }
    }

    match ProgramV2::parse_v2(src) {
        Ok(prog) => {
            out.push_str(&format!("\n# Parsed: {} items, N={}\n", prog.items.len(), prog.universe.n_types()));
            out.push_str(&prog.display());
            out.push_str("\n");

            // Phase3: trait/impl 方法表
            let method_table = lower_trait_impl_method_table(&prog);
            out.push_str(&format!("# Method table: {} traits, {} impls\n", method_table.traits.len(), method_table.impls.len()));
            for ((ty, tr), _) in &method_table.impl_map {
                out.push_str(&format!("#   {}: {} \n", ty, tr));
            }
            for (ty, idxs) in &method_table.inherent_map {
                out.push_str(&format!("#   inherent {}: {} methods\n", ty, idxs.len()));
            }

            // Phase3: lifetime graph
            match lower_lifetimes(&prog) {
                Ok(graph) => {
                    out.push_str(&format!("# Lifetime lowering: {} lifetimes, has_cycle={}\n", graph.lifetimes.len(), graph.has_cycle()));
                }
                Err(e) => out.push_str(&format!("# Lifetime error: {}\n", e)),
            }

            // Phase3: stdlib usage
            let stdlib_reg = lower_stdlib_usage(&prog);
            out.push_str(&format!("# Stdlib usage: vec={}, string={}, hashmap={}\n",
                stdlib_reg.vec_encodings.len(), stdlib_reg.string_encodings.len(), stdlib_reg.hashmap_encodings.len()));

            // Phase3: async state machines
            for item in &prog.items {
                if let polyrust_core::minirust::ast_v2::ItemV2::Fn(f) = item {
                    if f.sig.is_async {
                        let sm = polyrust_core::minirust::async_qap::lower_async_fn(&f.sig.name, &f.body_src);
                        out.push_str(&format!("# Async fn {}: {} await points, {} states\n", f.sig.name, sm.num_await_points, sm.states.len()));
                        for poly in sm.poly_text() {
                            out.push_str(&format!("#   {}\n", poly));
                        }
                    }
                }
            }

            match lower_program(prog) {
                Ok(lowered) => {
                    out.push_str(&format!("\n# Lowered: {} items, products: {}, sums: {}\n",
                        lowered.program.items.len(), lowered.products.len(), lowered.sums.len()));
                    for (name, fields) in &lowered.products {
                        out.push_str(&format!("# Product {}: {} fields -> {}\n",
                            name, fields.len(),
                            fields.iter().map(|(n,ty)| format!("{}:{}", n, ty.name())).collect::<Vec<_>>().join(", ")));
                        let indices: Vec<usize> = fields.iter().filter_map(|(_,ty)| lowered.program.universe.index_of(ty)).collect();
                        if !indices.is_empty() {
                            let poly = product_poly_text(name, 0, &indices);
                            out.push_str(&format!("#   {}\n", poly));
                        }
                    }
                    for (name, variants) in &lowered.sums {
                        out.push_str(&format!("# Sum {}: {} variants\n", name, variants.len()));
                    }
                    if !lowered.mod_map.is_empty() {
                        out.push_str(&format!("# Mod map: {:?}\n", lowered.mod_map));
                    }
                    if !lowered.generated.is_empty() {
                        out.push_str(&format!("# Generated {} aux items (async state machines etc)\n", lowered.generated.len()));
                        for gen in &lowered.generated {
                            out.push_str(&format!("#   {:?}\n", gen));
                        }
                    }
                    out.push_str("\n# Lowered program:\n");
                    out.push_str(&lowered.program.display());
                }
                Err(e) => {
                    out.push_str(&format!("# Lowering error: {}\n", e));
                }
            }
        }
        Err(e) => {
            out.push_str(&format!("# Parse error: {}\n", e));
        }
    }

    // 嘗試 match lowering
    if src.contains("match ") {
        out.push_str("\n# Match lowering:\n");
        for line in src.lines() {
            if line.trim().starts_with("let") && line.contains("match") {
                if let Some(pos) = line.find("match") {
                    if let Ok(tree) = parse_match_to_decision_tree(&line[pos..]) {
                        out.push_str(&decision_tree_to_if_chain(&tree));
                    }
                }
            }
        }
    }

    // 嘗試 for lowering
    if src.contains("for ") && src.contains(" in ") {
        out.push_str("\n# For lowering:\n");
        for line in src.lines() {
            let t = line.trim();
            if t.starts_with("for ") {
                if let Ok(fl) = parse_for_to_loop(t) {
                    out.push_str(&for_loop_to_loop_text(&fl));
                    out.push_str("\n");
                }
            }
        }
    }

    // Phase3: contracts unrolling examples
    if src.contains("while ") || src.contains("for ") {
        out.push_str("\n# Loop contracts unrolling (Phase3):\n");
        if src.contains("while ") {
            let example = polyrust_core::minirust::contracts::unroll_while("x < 10", "x = x + 1;", 3, &["x >=0".to_string()]);
            out.push_str(&example);
            out.push_str("\n");
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_full_struct() {
        let src = r#"
            struct Point { x: i32, y: i32 }
            fn main() { let p = Point { x: 3, y: 4 }; }
        "#;
        let lowered = lower_full(src).unwrap();
        assert!(lowered.products.contains_key("Point"));
        println!("{}", lowering_report(src));
    }

    #[test]
    fn test_lower_full_match() {
        let src = r#"
            enum Option<T> { Some(T), None }
            fn main() {
                let x = Option::Some(5);
                let y = match x { Some(v) => v, None => 0 };
            }
        "#;
        let report = lowering_report(src);
        println!("{}", report);
        assert!(report.contains("if chain") || report.contains("decision tree") || report.contains("match"));
    }

    #[test]
    fn test_lower_full_mod() {
        let src = r#"
            mod utils {
                pub struct Point { x: i32, y: i32 }
            }
            fn main() { let p = utils::Point { x: 1, y: 2 }; }
        "#;
        let lowered = lower_full(src).unwrap();
        println!("{:?}", lowered.mod_map);
        assert!(lowered.program.items.iter().any(|it| matches!(it, polyrust_core::minirust::ast_v2::ItemV2::Struct(s) if s.name.contains("Point"))));
    }

    #[test]
    fn test_lower_phase3_features() {
        let src = r#"
            # @fuel: 5
            # @invariant: x >=0
            # @lifetime 'a: 'b
            # @unsafe-allowed
            # @type-universe: Vec<i32>, HashMap<String,i32>
            trait Display { fn fmt(&self) -> String; }
            impl Display for Point { fn fmt(&self) -> String { String::from("hi") } }
            struct Point { x: i32, y: i32 }
            async fn fetch() -> i32 { 42 }
            fn main() {
                let v: Vec<i32> = Vec::new();
                let x = fetch().await;
            }
        "#;
        let report = lowering_report(src);
        println!("{}", report);
        assert!(report.contains("Method table"));
        assert!(report.contains("Lifetime"));
        assert!(report.contains("Stdlib"));
        assert!(report.contains("Async"));
        assert!(report.contains("fuel"));
    }
}
