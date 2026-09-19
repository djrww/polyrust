// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! C4 合約模式：`# @require <expr>` / `# @ensure <expr>` 註解掃描 + 語義分級。
//!
//! 語義分級（誠實三級，唔准當全部 exact）：
//! - `ExactEq`：`ident == <int 字面量>` —— predicate 可直接落成 premise poly `var − c = 0`。
//! - `CmpAbstract`：兩邊係 `(ident|int) [+−*/] (ident|int)` 嘅比較（<, <=, >, >=, !=）——
//!   非線性/排序關係喺 𝔽_p 唔可直接表達 → 新鮮 bool 鎖定 + b=1 premise（**抽象**）。
//! - `Opaque`：`sum(0..i)`、`result`、函數調用等超出子集 —— 只記 marker 唔加約束。
//!
//! ensure 子句：C4 只**收集+報告**（檢查面統計）——屬性證明（UNS/premise 反證）屬後續里程碑，
//! 唔准喺度扮已 enforce。

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClauseKind {
    /// `ident == int` ✓ 直接 premise
    ExactEq,
    /// 算術比較（抽象 bool 鎖定）
    CmpAbstract,
    /// 超子集（marker only）
    Opaque,
}

#[derive(Debug, Clone)]
pub struct ContractClause {
    /// 原文（报告用）
    pub text: String,
    pub kind: ClauseKind,
    pub is_require: bool,
    /// ExactEq 專用：(param_name, value)
    pub eq: Option<(String, i128)>,
}

#[derive(Debug, Clone, Default)]
pub struct V4Contract {
    pub requires: Vec<ContractClause>,
    pub ensures: Vec<ContractClause>,
}

const CMP_OPS: [&str; 6] = ["==", "!=", "<=", ">=", "<", ">"];

fn is_term(s: &str, param_names: &[String]) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if s.parse::<i128>().is_ok() {
        return true;
    }
    // ident 必須係已知輸入參數名（避免將任意 ident 當 exact —— 安全網）
    param_names.iter().any(|n| n == s)
}

/// 嘗試解析 `ident == <int>`（兩邊可調轉）
fn try_exact_eq(expr: &str, param_names: &[String]) -> Option<(String, i128)> {
    let (l, r) = expr.split_once("==")?;
    let (l, r) = (l.trim(), r.trim());
    if let (Ok(v), true) = (r.parse::<i128>(), param_names.iter().any(|n| n == l)) {
        return Some((l.to_string(), v));
    }
    if let (Ok(v), true) = (l.parse::<i128>(), param_names.iter().any(|n| n == r)) {
        return Some((r.to_string(), v));
    }
    None
}

fn has_arith_term_shape(side: &str, param_names: &[String]) -> bool {
    let mut parts = side.split(|c| c == '+' || c == '-' || c == '*' || c == '/' || c == '%');
    side.chars().all(|c| c.is_whitespace() || c.is_alphabetic() || c=='_' || c.is_ascii_digit() || "+-*/%()".contains(c))
        && parts.all(|p| is_term(p, param_names) || p.trim().parse::<i128>().is_ok())
}

fn classify(expr: &str, is_require: bool, param_names: &[String]) -> ContractClause {
    if let Some(eq) = try_exact_eq(expr, param_names) {
        return ContractClause { text: expr.to_string(), kind: ClauseKind::ExactEq, is_require, eq: Some(eq) };
    }
    for op in CMP_OPS {
        if op == "==" {
            continue; // 上面已處理
        }
        if let Some((l, r)) = expr.split_once(op) {
            if has_arith_term_shape(l, param_names) && has_arith_term_shape(r, param_names) {
                return ContractClause { text: expr.to_string(), kind: ClauseKind::CmpAbstract, is_require, eq: None };
            }
        }
    }
    ContractClause { text: expr.to_string(), kind: ClauseKind::Opaque, is_require, eq: None }
}

/// 掃描 source 中 `# @require …` / `# @ensure …`（DSL 註解軌；`#[polyrust::require]` attr 軌後補）。
/// `param_names` = 目標 fn 嘅輸入名（用於 ExactEq 安全網）；ensure 可引用 `result`。
pub fn parse_contract(src: &str, param_names: &[String]) -> V4Contract {
    let mut c = V4Contract::default();
    for ln in src.lines() {
        let t = ln.trim_start();
        if let Some(rest) = t.strip_prefix("# @require") {
            let expr = rest.trim_start_matches(|ch: char| ch == ':' || ch.is_whitespace());
            if !expr.is_empty() {
                c.requires.push(classify(expr, true, param_names));
            }
        } else if let Some(rest) = t.strip_prefix("# @ensure") {
            let expr = rest.trim_start_matches(|ch: char| ch == ':' || ch.is_whitespace());
            if !expr.is_empty() {
                c.ensures.push(classify(expr, false, param_names));
            }
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        ["x", "n", "length", "result"].iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn exact_eq_parse() {
        let c = parse_contract("# @require length == 8\nfn f(length: usize) {}", &names());
        assert_eq!(c.requires.len(), 1);
        assert_eq!(c.requires[0].kind, ClauseKind::ExactEq);
        assert_eq!(c.requires[0].eq, Some(("length".to_string(), 8)));
    }

    #[test]
    fn cmp_abstract_parse() {
        let c = parse_contract("# @require x >= 0\nfn f(x: i32){}", &names());
        assert_eq!(c.requires[0].kind, ClauseKind::CmpAbstract);
    }

    #[test]
    fn opaque_and_ensure() {
        let src = "# @require s == sum(0..i)\n# @ensure result == x * x\nfn f(x: i32){}";
        let c = parse_contract(src, &names());
        assert_eq!(c.requires[0].kind, ClauseKind::Opaque);
        assert_eq!(c.ensures.len(), 1);
        // ensure 用 result（唔喺參數名單都唔緊要 —— 屬未知 ident → Opaque 而非 ExactEq 誤判）：
        assert_eq!(c.ensures[0].kind, ClauseKind::Opaque);
    }

    #[test]
    fn reverse_exact_eq() {
        let c = parse_contract("# @require 100 == n\nfn f(n: i32){}", &names());
        assert_eq!(c.requires[0].eq, Some(("n".to_string(), 100)));
    }

    #[test]
    fn foreign_ident_not_exact() {
        // 未見參數名嘅 ident 唔准落 ExactEq（安全網）
        let c = parse_contract("# @require q == 5\nfn f(x: i32){}", &["x".to_string()]);
        assert_ne!(c.requires[0].kind, ClauseKind::ExactEq);
    }

    #[test]
    fn colon_form_supported() {
        let c = parse_contract("# @require: x == 42\nfn f(x: i32){}", &["x".to_string()]);
        assert_eq!(c.requires[0].eq, Some(("x".to_string(), 42)));
    }
}
