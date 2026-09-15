//! Phase3 — 迴圈契約與有界展開：@fuel, @invariant, @requires, @ensures
//!
//! 基於 `dsl.rs` 解析的契約，生成多項式與子句。

use crate::frac::Frac;
use crate::poly::Poly;

/// 迴圈契約
#[derive(Clone, Debug, Default)]
pub struct LoopContract {
    /// 有界展開次數，None 表示預設 3
    pub fuel: Option<usize>,
    /// 不變量列表（bool 表達式文本）
    pub invariants: Vec<String>,
    /// 前置條件
    pub requires: Vec<String>,
    /// 後置條件
    pub ensures: Vec<String>,
}

impl LoopContract {
    pub fn from_poly_source(src: &crate::dsl::PolySource) -> Self {
        Self {
            fuel: src.fuel,
            invariants: src.invariants.clone(),
            requires: src.requires.clone(),
            ensures: src.ensures.clone(),
        }
    }

    pub fn fuel_or_default(&self) -> usize {
        self.fuel.unwrap_or(3)
    }

    /// 生成不變量多項式文本（用於報告）
    pub fn invariant_poly_text(&self, node_id: usize) -> Vec<String> {
        self.invariants.iter().enumerate().map(|(i, inv)| {
            format!("t{}_{} - 1 = 0  # invariant {}: {}", node_id, format!("inv{}", i), i, inv)
        }).collect()
    }

    /// 生成 fuel counter 約束文本
    pub fn fuel_poly_text(&self, node_id: usize) -> Vec<String> {
        let k = self.fuel_or_default();
        (0..k).map(|i| {
            format!("c{}_{} - c{}_{} - 1 = 0  # fuel counter {} -> {}", node_id, i+1, node_id, i, i, i+1)
        }).collect()
    }
}

/// 有界展開：將 loop/while/for 展開 K 次
#[derive(Clone, Debug)]
pub struct BoundedLoop {
    pub original: String,
    pub fuel: usize,
    pub unrolled: String,
}

/// 將 while 展開為有界 if 鏈
pub fn unroll_while(cond: &str, body: &str, fuel: usize, invariants: &[String]) -> String {
    let mut out = String::new();
    if !invariants.is_empty() {
        out.push_str(&format!("// @invariant: {}\n", invariants.join(", ")));
        out.push_str(&format!("// assert invariant before loop\n"));
    }
    out.push_str(&format!("// while {} with fuel={}\n", cond, fuel));
    out.push_str("{\n");
    out.push_str(&format!("  let mut __fuel = {};\n", fuel));
    out.push_str("  loop {\n");
    out.push_str(&format!("    if __fuel <= 0 {{ break; }}\n"));
    out.push_str(&format!("    if !({}) {{ break; }}\n", cond));
    if !invariants.is_empty() {
        for inv in invariants {
            out.push_str(&format!("    // assert {} \n    if !({}) {{ panic!(\"invariant violated\"); }}\n", inv, inv));
        }
    }
    out.push_str(&format!("    {}\n", body));
    out.push_str("    __fuel = __fuel - 1;\n");
    out.push_str("  }\n");
    if !invariants.is_empty() {
        for inv in invariants {
            out.push_str(&format!("  // assert {} after loop\n", inv));
        }
    }
    out.push_str("}\n");
    out
}

/// 將 for 展開（調用 lower.rs 的邏輯，但加入 fuel）
pub fn unroll_for(pat: &str, iter: &str, body: &str, fuel: usize, invariants: &[String]) -> String {
    let mut out = String::new();
    if !invariants.is_empty() {
        out.push_str(&format!("// @invariant: {}\n", invariants.join(", ")));
    }
    out.push_str(&format!("// for {} in {} with fuel={}\n", pat, iter, fuel));
    out.push_str("{\n");
    out.push_str(&format!("  let mut __iter = {}.into_iter();\n", iter));
    out.push_str(&format!("  let mut __fuel = {};\n", fuel));
    out.push_str("  loop {\n");
    out.push_str("    if __fuel <= 0 { break; }\n");
    out.push_str(&format!("    match __iter.next() {{\n      Some({}) => {{\n", pat));
    if !invariants.is_empty() {
        for inv in invariants {
            out.push_str(&format!("        // assert {}\n", inv));
        }
    }
    out.push_str(&format!("        {}\n", body));
    out.push_str("      },\n");
    out.push_str("      None => break,\n");
    out.push_str("    }\n");
    out.push_str("    __fuel = __fuel - 1;\n");
    out.push_str("  }\n");
    out.push_str("}\n");
    out
}

/// 生成契約多項式：invariant 必須為 true
/// t_inv -1 =0
pub fn invariant_poly(nvars: usize, inv_var: usize) -> Poly {
    Poly::var(inv_var, Frac::ONE, nvars).sub(&Poly::constant(Frac::ONE))
}

/// 生成 fuel 多項式：c_{i+1} - c_i +1 =0? 實際 c_{i+1} = c_i -1
pub fn fuel_counter_poly(nvars: usize, c_i: usize, c_next: usize) -> Poly {
    // c_next - (c_i -1) =0 => c_next - c_i +1 =0
    Poly::var(c_next, Frac::ONE, nvars)
        .sub(&Poly::var(c_i, Frac::ONE, nvars))
        .add(&Poly::constant(Frac::ONE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unroll_while() {
        let code = unroll_while("x < 10", "x = x + 1;", 5, &["x >=0".to_string()]);
        println!("{}", code);
        assert!(code.contains("__fuel"));
        assert!(code.contains("x >=0"));
    }

    #[test]
    fn test_unroll_for() {
        let code = unroll_for("x", "v", "sum = sum + x;", 3, &[]);
        println!("{}", code);
        assert!(code.contains("into_iter"));
        assert!(code.contains("__fuel"));
    }

    #[test]
    fn test_loop_contract_from_source() {
        let text = "# @fuel: 10\n# @invariant x >=0\nfn main() { let x = 0; }";
        let src = crate::dsl::load_poly(text).unwrap();
        let contract = LoopContract::from_poly_source(&src);
        assert_eq!(contract.fuel, Some(10));
        assert_eq!(contract.invariants.len(), 1);
        assert_eq!(contract.fuel_or_default(), 10);
    }

    #[test]
    fn test_fuel_default() {
        let contract = LoopContract::default();
        assert_eq!(contract.fuel_or_default(), 3);
    }
}
