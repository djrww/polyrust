//! 一致性 oracle（窮舉驗證）：借鏡 `rlzl` 的「暴力法做 ground truth」哲學。
//!
//! 在**有界的細程序空間**內窮舉所有 `.poly` 程序，逐個跑：
//! - **ground truth**：獨立直接檢查器（`checker::check_program`，直接推導）
//! - **代數管線**：`gen_constraints` → 域多項式 → `solve_boolean`（T6 判定核心）
//!
//! 斷言兩條路徑的**判定完全一致**（SAT ⟺ 接受），且 SAT 時管線見證解碼出的
//! 逐節點型別與檢查器推導一致。任何不一致 ⇒ 失敗並印出反例程式。
//!
//! 空間：表達式節點數 ≤ `max_size`，構造子 = 字面量 {0,1,true,false,()}、
//! 一元 {-, !}、二元 {+, ==}、`if/else`、`let`。無宏、無引用 ⇒ 約束系統無子句，
//! 判定化約為純多項式布爾求解（與 `run_pipeline` 同一批生成元）。

use std::time::Instant;

use crate::groebner::{field_polys, solve_boolean};
use crate::minirust::ast::Type;
use crate::minirust::checker::check_program;
use crate::minirust::constraints::gen_constraints;
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;

/// 窮舉用的小表達式（枚举側，與 `minirust::ast` 解耦）。
/// `Var(i)` 為 de Bruijn 索引：0 = 最內層綁定。
#[derive(Clone, Debug)]
enum Sx {
    LitI(i64),
    LitB(bool),
    Unit,
    Var(usize),
    Neg(Box<Sx>),
    Not(Box<Sx>),
    Add(Box<Sx>, Box<Sx>),
    Eq(Box<Sx>, Box<Sx>),
    If(Box<Sx>, Box<Sx>, Box<Sx>),
    Let(Box<Sx>, Box<Sx>),
}

/// 枚舉「純表達式」（不含頂層 let；可安全做運算元）恰好 `size` 節點。
/// `let` 只合法於序列位置，故運算元位置嚴禁 let——避免產生語法非法程序。
fn gen_pure(size: usize, bound: usize, out: &mut Vec<Sx>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    if size == 1 {
        out.push(Sx::LitI(0));
        out.push(Sx::LitI(1));
        out.push(Sx::LitB(true));
        out.push(Sx::LitB(false));
        out.push(Sx::Unit);
        for i in 0..bound {
            out.push(Sx::Var(i));
        }
        return;
    }
    // 一元：節點 = 1 + 子（子須為純表達式）
    let mut children = Vec::new();
    gen_pure(size - 1, bound, &mut children, cap);
    for c in &children {
        out.push(Sx::Neg(Box::new(c.clone())));
        out.push(Sx::Not(Box::new(c.clone())));
    }
    // 二元：節點 = 1 + 左 + 右（兩側皆純表達式）
    for i in 1..=size - 2 {
        let mut ls = Vec::new();
        let mut rs = Vec::new();
        gen_pure(i, bound, &mut ls, cap);
        gen_pure(size - 1 - i, bound, &mut rs, cap);
        for l in &ls {
            for r in &rs {
                out.push(Sx::Add(Box::new(l.clone()), Box::new(r.clone())));
                out.push(Sx::Eq(Box::new(l.clone()), Box::new(r.clone())));
            }
        }
    }
    // if：節點 = 1 + 條件 + 兩分支。條件須純；分支為序列位置（可含 let）。
    for a in 1..=size.saturating_sub(3) {
        for b in 1..=size - 1 - a {
            let c = size - 1 - a - b;
            if c < 1 {
                continue;
            }
            let mut cs = Vec::new();
            let mut ts = Vec::new();
            let mut fs = Vec::new();
            gen_pure(a, bound, &mut cs, cap);
            gen_seq(b, bound, &mut ts, cap);
            gen_seq(c, bound, &mut fs, cap);
            for c in &cs {
                for t in &ts {
                    for f in &fs {
                        out.push(Sx::If(
                            Box::new(c.clone()),
                            Box::new(t.clone()),
                            Box::new(f.clone()),
                        ));
                    }
                }
            }
        }
    }
}

/// 枚舉「序列表達式」（可含頂層 let）恰好 `size` 節點。
fn gen_seq(size: usize, bound: usize, out: &mut Vec<Sx>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    // 純表達式都是合法序列表達式
    gen_pure(size, bound, out, cap);
    if size <= 2 {
        return;
    }
    // let：節點 = 1 + 綁定式（純）+ 體（序列，作用域多一個變數）
    for i in 1..=size - 2 {
        let mut es = Vec::new();
        let mut bs = Vec::new();
        gen_pure(i, bound, &mut es, cap);
        gen_seq(size - 1 - i, bound + 1, &mut bs, cap);
        for e in &es {
            for b in &bs {
                out.push(Sx::Let(Box::new(e.clone()), Box::new(b.clone())));
            }
        }
    }
}

/// 渲染成 Mini-Rust 源碼（全括號化，無優先級歧義）。
fn render(e: &Sx, env: &mut Vec<String>, next_name: &mut usize) -> String {
    match e {
        Sx::LitI(n) => n.to_string(),
        Sx::LitB(b) => if *b { "true".into() } else { "false".into() },
        Sx::Unit => "()".into(),
        Sx::Var(i) => env[env.len() - 1 - i].clone(),
        Sx::Neg(c) => format!("(-{})", render(c, env, next_name)),
        Sx::Not(c) => format!("(!{})", render(c, env, next_name)),
        Sx::Add(a, b) => format!(
            "({} + {})",
            render(a, env, next_name),
            render(b, env, next_name)
        ),
        Sx::Eq(a, b) => format!(
            "({} == {})",
            render(a, env, next_name),
            render(b, env, next_name)
        ),
        // if 自包括號：做運算元時 `(... if ...)` 合法（LParen → parse_expr 接 if）；
        // 做序列語句時括號透明亦合法。
        Sx::If(c, t, f) => format!(
            "(if {} {{ {} }} else {{ {} }})",
            render(c, env, next_name),
            render(t, env, next_name),
            render(f, env, next_name)
        ),
        Sx::Let(e1, e2) => {
            let name = format!("v{}", next_name);
            *next_name += 1;
            let s1 = render(e1, env, next_name); // e1 在外層作用域
            env.push(name.clone());
            let s2 = render(e2, env, next_name);
            env.pop();
            format!("let {} = {}; {}", name, s1, s2)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct OracleReport {
    pub max_size: usize,
    pub total: u64,
    pub sat: u64,
    pub unsat: u64,
    pub mismatches: Vec<String>,   // 判定不一致的反例
    pub type_mismatches: Vec<String>, // 型別解碼不一致的反例
    pub internal_errors: Vec<String>, // 解析/約束生成錯誤（不應發生）
    pub elapsed_ms: u128,
    pub capped: bool,
}

impl OracleReport {
    pub fn pass(&self) -> bool {
        self.mismatches.is_empty() && self.type_mismatches.is_empty() && self.internal_errors.is_empty()
    }
}

/// 窮舉 ≤ `max_size` 節點的全部表達式，逐一對照兩條路徑。
/// `cap` 限制枚舉總數（防止組合爆炸；觸頂時 `capped = true`）。
pub fn run_oracle(max_size: usize, cap: usize) -> OracleReport {
    let t0 = Instant::now();
    let mut rep = OracleReport {
        max_size,
        ..Default::default()
    };

    for size in 1..=max_size {
        let mut exprs = Vec::new();
        gen_seq(size, 0, &mut exprs, cap);
        if exprs.len() >= cap {
            rep.capped = true;
        }
        for e in &exprs {
            if rep.total >= cap as u64 {
                rep.capped = true;
                break;
            }
            rep.total += 1;
            let mut env: Vec<String> = Vec::new();
            let mut next_name = 0usize;
            let body = render(e, &mut env, &mut next_name);
            let src = format!("fn main() {{\n{}\n}}", body);

            // ── 兩條路徑 ──
            let p = match Parser::parse_program(&src) {
                Ok(p) => p,
                Err(err) => {
                    rep.internal_errors
                        .push(format!("解析失敗：{}\n程式：{}", err, src));
                    continue;
                }
            };
            let mut exp_c = Expander::new(p.macros.clone(), p.next_id);
            let checker = check_program(&p, &mut exp_c);
            let mut exp_g = Expander::new(p.macros.clone(), p.next_id);
            let sys = match gen_constraints(&p, &mut exp_g) {
                Ok(s) => s,
                Err(err) => {
                    rep.internal_errors
                        .push(format!("約束生成失敗：{}\n程式：{}", err, src));
                    continue;
                }
            };
            if !sys.clauses.is_empty() {
                rep.internal_errors.push(format!(
                    "oracle 空間不應有子句（無宏無引用）\n程式：{}",
                    src
                ));
                continue;
            }
            let mut all = sys.polys.clone();
            all.extend(field_polys(sys.nvars));
            let witness = solve_boolean(&all, sys.nvars);

            // ── 斷言 1：判定等價（T6）──
            let pipe_sat = witness.is_some();
            let checker_ok = checker.is_ok();
            if pipe_sat != checker_ok {
                rep.mismatches.push(format!(
                    "判定不一致：管線={} 檢查器={}\n程式：{}",
                    if pipe_sat { "SAT" } else { "UNSAT" },
                    match &checker {
                        Ok(_) => "接受".to_string(),
                        Err(e) => format!("拒絕：{}", e),
                    },
                    src
                ));
                continue;
            }
            if pipe_sat {
                rep.sat += 1;
            } else {
                rep.unsat += 1;
            }

            // ── 斷言 2：SAT 時逐節點型別解碼一致（T2）──
            if let Some(sigma) = &witness {
                if let Ok(ds) = &checker {
                    let mut merged: std::collections::HashMap<usize, Type> =
                        std::collections::HashMap::new();
                    for d in ds {
                        merged.extend(d.node_types.iter().map(|(k, v)| (*k, *v)));
                    }
                    for (node, tvars) in &sys.node_type {
                        let set: Vec<usize> = (0..tvars.len())
                            .filter(|t| sigma[tvars[*t]].is_one())
                            .collect();
                        let expect = merged.get(node).copied();
                        let ok = set.len() == 1
                            && expect.map_or(false, |ty| ty.index() == set[0]);
                        if !ok {
                            rep.type_mismatches.push(format!(
                                "節點 {} 型別解碼不一致：管線位元 {:?} 檢查器 {:?}\n程式：{}",
                                node, set, expect, src
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }
    rep.elapsed_ms = t0.elapsed().as_millis();
    rep
}

// ─────────────────────────────────────────────────────────────────────────────
// 測試：窮舉小空間，兩條路徑必須完全一致
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_exhaustive_consistency() {
        // ≤5 節點：約數千個程序，秒級；判定與型別解碼都必須零不一致
        let rep = run_oracle(5, 200_000);
        assert!(rep.total > 0);
        assert!(rep.pass(), "oracle 不一致：{:?}", rep);
        // 兩個方向都必須被覆蓋到（否則窮舉失去意義）
        assert!(rep.sat > 0, "應有 SAT 樣本");
        assert!(rep.unsat > 0, "應有 UNSAT 樣本");
    }

    #[test]
    fn oracle_space_contains_both_verdicts_small() {
        // 即使 ≤3 節點也已有型別錯配（例如 (1 + true)）
        let rep = run_oracle(3, 100_000);
        assert!(rep.sat > 0 && rep.unsat > 0, "{:?}", rep);
        assert!(rep.pass());
    }

    #[test]
    fn render_let_var_roundtrip() {
        // let 綁定 + 變數引用的渲染必須可被解析器接受
        let e = Sx::Let(
            Box::new(Sx::LitI(1)),
            Box::new(Sx::Add(Box::new(Sx::Var(0)), Box::new(Sx::LitI(0)))),
        );
        let mut env = Vec::new();
        let mut n = 0;
        let body = render(&e, &mut env, &mut n);
        let src = format!("fn main() {{\n{}\n}}", body);
        let p = Parser::parse_program(&src).expect("let/var 渲染應可解析");
        let mut exp = Expander::new(p.macros.clone(), p.next_id);
        assert!(check_program(&p, &mut exp).is_ok());
    }
}
