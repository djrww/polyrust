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
//! 一元 {-, !, *}、二元 {+, ==}、`if/else`、`let`、引用 {&, &mut}、賦值
//! {x = e, *e = e}、語句序列。無宏（宏側覆蓋見 `obligations` 與
//! `tier0_equivalence_corpus`）；含借用 ⇒ 約束系統帶互斥子句，
//! 判定化約為「多項式 + 子句多項式」的布爾求解（與管線 S6 同一編碼）。

use std::time::Instant;

use crate::groebner::{field_polys, solve_boolean};
use crate::minirust::ast::Type;
use crate::minirust::checker::check_program;
use crate::minirust::constraints::gen_constraints;
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;

/// 窮舉用的小表達式（枚举側，與 `minirust::ast` 解耦）。
/// `Var(i)` 為 de Bruijn 索引：0 = 最內層綁定。
/// `pub(crate)`：供 `brute.rs` 共用同一枚舉空間（@brute 對照常態化）。
#[derive(Clone, Debug)]
pub(crate) enum Sx {
    LitI(i64),
    LitB(bool),
    Unit,
    Var(usize),
    Neg(Box<Sx>),
    Not(Box<Sx>),
    /// *e（解引用）
    Deref(Box<Sx>),
    /// &x（借用綁定 i 的變數）
    Ref(usize),
    /// &mut x
    RefMut(usize),
    Add(Box<Sx>, Box<Sx>),
    Eq(Box<Sx>, Box<Sx>),
    If(Box<Sx>, Box<Sx>, Box<Sx>),
    Let(Box<Sx>, Box<Sx>),
    /// 語句序列 a; b（a 的值丟棄）
    Seq(Box<Sx>, Box<Sx>),
    /// x = e（對綁定 i 的變數賦值；語句）
    AssignVar(usize, Box<Sx>),
    /// *lhs = rhs（lhs 為引用表達式；語句）
    AssignDeref(Box<Sx>, Box<Sx>),
    /// f_i(args)——呼叫第 i 個 fn（實參個數可與形參不符，覆蓋邊界）
    Call(usize, Vec<Sx>),
    /// m_i!(args)——調用第 i 個宏（實參個數可與匹配器不符，覆蓋邊界）
    Invoke(usize, Vec<Sx>),
}

/// 枚舉上下文：空間內有幾個 fn、幾個宏（供 Call/Invoke 葉）。
type Ctx2 = (usize, usize);

/// 枚舉「純表達式」（不含頂層 let/賦值；可安全做運算元）恰好 `size` 節點。
/// `let` 與賦值只合法於序列位置，故運算元位置嚴禁——避免產生語法非法程序。
fn gen_pure(size: usize, bound: usize, ctx: Ctx2, out: &mut Vec<Sx>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    let (nfns, nmacros) = ctx;
    if size == 1 {
        out.push(Sx::LitI(0));
        out.push(Sx::LitI(1));
        out.push(Sx::LitB(true));
        out.push(Sx::LitB(false));
        out.push(Sx::Unit);
        for i in 0..bound {
            out.push(Sx::Var(i));
            out.push(Sx::Ref(i));
            out.push(Sx::RefMut(i));
        }
        for fi in 0..nfns {
            out.push(Sx::Call(fi, Vec::new())); // 0 實參
        }
        return;
    }
    // 一元：節點 = 1 + 子（子須為純表達式）
    let mut children = Vec::new();
    gen_pure(size - 1, bound, ctx, &mut children, cap);
    for c in &children {
        out.push(Sx::Neg(Box::new(c.clone())));
        out.push(Sx::Not(Box::new(c.clone())));
        out.push(Sx::Deref(Box::new(c.clone())));
    }
    // 二元：節點 = 1 + 左 + 右（兩側皆純表達式）
    for i in 1..=size - 2 {
        let mut ls = Vec::new();
        let mut rs = Vec::new();
        gen_pure(i, bound, ctx, &mut ls, cap);
        gen_pure(size - 1 - i, bound, ctx, &mut rs, cap);
        for l in &ls {
            for r in &rs {
                out.push(Sx::Add(Box::new(l.clone()), Box::new(r.clone())));
                out.push(Sx::Eq(Box::new(l.clone()), Box::new(r.clone())));
            }
        }
    }
    // Call / Invoke（1..=2 實參）：節點 = 1 + Σ實參
    for nargs in 1..=2usize {
        for split in splits(size - 1, nargs) {
            let mut arg_lists: Vec<Vec<Sx>> = vec![Vec::new()];
            for &sz in &split {
                let mut part = Vec::new();
                gen_pure(sz, bound, ctx, &mut part, cap);
                let mut next = Vec::new();
                for list in &arg_lists {
                    for a in &part {
                        let mut l2 = list.clone();
                        l2.push(a.clone());
                        next.push(l2);
                    }
                }
                arg_lists = next;
                if arg_lists.is_empty() {
                    break;
                }
            }
            for fi in 0..nfns {
                for args in &arg_lists {
                    out.push(Sx::Call(fi, args.clone()));
                }
            }
            for mi in 0..nmacros {
                for args in &arg_lists {
                    out.push(Sx::Invoke(mi, args.clone()));
                }
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
            gen_pure(a, bound, ctx, &mut cs, cap);
            gen_seq(b, bound, ctx, &mut ts, cap);
            gen_seq(c, bound, ctx, &mut fs, cap);
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

/// 把 `total` 拆成 `parts` 個 ≥1 的有序份額。
fn splits(total: usize, parts: usize) -> Vec<Vec<usize>> {
    if parts == 0 {
        return if total == 0 { vec![vec![]] } else { vec![] };
    }
    let mut out = Vec::with_capacity(8);
    for first in 1..=total.saturating_sub(parts - 1) {
        for rest in splits(total - first, parts - 1) {
            let mut v = vec![first];
            v.extend(rest);
            out.push(v);
        }
    }
    out
}

/// 枚舉「語句」（序列位置的頭元素）恰好 `size` 節點。
fn gen_stmt(size: usize, bound: usize, ctx: Ctx2, out: &mut Vec<Sx>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    // 純表達式也可作語句（值丟棄）
    gen_pure(size, bound, ctx, out, cap);
    if bound == 0 {
        return; // 無綁定變數 ⇒ 賦值語句不可能
    }
    // x = e：節點 = 1 + rhs（純）
    if size >= 2 {
        let mut rs = Vec::new();
        gen_pure(size - 1, bound, ctx, &mut rs, cap);
        for i in 0..bound {
            for r in &rs {
                out.push(Sx::AssignVar(i, Box::new(r.clone())));
            }
        }
    }
    // *lhs = rhs：節點 = 1 + lhs + rhs（皆純）
    if size >= 3 {
        for i in 1..=size - 2 {
            let mut ls = Vec::new();
            let mut rs = Vec::new();
            gen_pure(i, bound, ctx, &mut ls, cap);
            gen_pure(size - 1 - i, bound, ctx, &mut rs, cap);
            for l in &ls {
                for r in &rs {
                    out.push(Sx::AssignDeref(Box::new(l.clone()), Box::new(r.clone())));
                }
            }
        }
    }
}

/// 枚舉「序列表達式」（可含頂層 let / 賦值語句）恰好 `size` 節點。
/// `pub(crate)`：供 `brute.rs` 共用。
pub(crate) fn gen_seq(size: usize, bound: usize, ctx: Ctx2, out: &mut Vec<Sx>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    // 純表達式都是合法序列表達式
    gen_pure(size, bound, ctx, out, cap);
    if size <= 2 {
        return;
    }
    // let：節點 = 1 + 綁定式（純）+ 體（序列，作用域多一個變數）
    for i in 1..=size - 2 {
        let mut es = Vec::new();
        let mut bs = Vec::new();
        gen_pure(i, bound, ctx, &mut es, cap);
        gen_seq(size - 1 - i, bound + 1, ctx, &mut bs, cap);
        for e in &es {
            for b in &bs {
                out.push(Sx::Let(Box::new(e.clone()), Box::new(b.clone())));
            }
        }
    }
    // 語句序列 stmt; rest：節點 = 1 + stmt + rest
    for i in 1..=size - 2 {
        let mut ss = Vec::new();
        let mut rs = Vec::new();
        gen_stmt(i, bound, ctx, &mut ss, cap);
        gen_seq(size - 1 - i, bound, ctx, &mut rs, cap);
        for s in &ss {
            for r in &rs {
                out.push(Sx::Seq(Box::new(s.clone()), Box::new(r.clone())));
            }
        }
    }
}

/// 渲染成 Mini-Rust 源碼（全括號化，無優先級歧義）。`pub(crate)`：供 `brute.rs` 共用。
pub(crate) fn render(e: &Sx, env: &mut Vec<String>, next_name: &mut usize) -> String {
    match e {
        Sx::LitI(n) => n.to_string(),
        Sx::LitB(b) => if *b { "true".into() } else { "false".into() },
        Sx::Unit => "()".into(),
        Sx::Var(i) => env[env.len() - 1 - i].clone(),
        Sx::Ref(i) => format!("&{}", env[env.len() - 1 - i]),
        Sx::RefMut(i) => format!("&mut {}", env[env.len() - 1 - i]),
        Sx::Neg(c) => format!("(-{})", render(c, env, next_name)),
        Sx::Not(c) => format!("(!{})", render(c, env, next_name)),
        Sx::Deref(c) => format!("(*({}))", render(c, env, next_name)),
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
        Sx::Seq(a, b) => format!("{}; {}", render(a, env, next_name), render(b, env, next_name)),
        Sx::AssignVar(i, rhs) => {
            let name = env[env.len() - 1 - i].clone();
            format!("{} = {}", name, render(rhs, env, next_name))
        }
        Sx::AssignDeref(lhs, rhs) => format!(
            "*({}) = {}",
            render(lhs, env, next_name),
            render(rhs, env, next_name)
        ),
        Sx::Call(fi, args) => {
            let args: Vec<String> = args.iter().map(|a| render(a, env, next_name)).collect();
            format!("f{}({})", fi, args.join(", "))
        }
        Sx::Invoke(mi, args) => {
            let args: Vec<String> = args.iter().map(|a| render(a, env, next_name)).collect();
            format!("m{}!({})", mi, args.join(", "))
        },
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

/// 單個程序對照：完整代數判定 ⟺ 獨立檢查器；SAT 時逐節點型別解碼一致。
fn check_one(src: &str, rep: &mut OracleReport) {
    rep.total += 1;
    // ── 兩條路徑 ──
    let p = match Parser::parse_program(src) {
        Ok(p) => p,
        Err(err) => {
            rep.internal_errors
                .push(format!("解析失敗：{}\n程式：{}", err, src));
            return;
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
            return;
        }
    };
    // 子句（借用互斥／臂互斥）→ 多項式（與管線 S6 同一編碼：clause_to_poly）
    let mut all = sys.polys.clone();
    all.extend(field_polys(sys.nvars));
    for cl in &sys.clauses {
        all.push(crate::pipeline::clause_to_poly(cl, sys.nvars));
    }
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
        return;
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
                let ok = set.len() == 1 && expect.map_or(false, |ty| ty.index() == set[0]);
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

/// 窮舉 ≤ `max_size` 節點的全部表達式（無宏無 fn），逐一對照兩條路徑。
/// `cap` 限制枚舉總數（防止組合爆炸；觸頂時 `capped = true`）。
pub fn run_oracle(max_size: usize, cap: usize) -> OracleReport {
    let t0 = Instant::now();
    let mut rep = OracleReport {
        max_size,
        ..Default::default()
    };

    for size in 1..=max_size {
        let mut exprs = Vec::new();
        gen_seq(size, 0, (0, 0), &mut exprs, cap);
        if exprs.len() >= cap {
            rep.capped = true;
        }
        for e in &exprs {
            if rep.total >= cap as u64 {
                rep.capped = true;
                break;
            }
            let mut env: Vec<String> = Vec::new();
            let mut next_name = 0usize;
            let body = render(e, &mut env, &mut next_name);
            let src = format!("fn main() {{\n{}\n}}", body);
            check_one(&src, &mut rep);
        }
    }
    rep.elapsed_ms = t0.elapsed().as_millis();
    rep
}

// ─────────────────────────────────────────────────────────────────────────────
// V2 空間：宏（臂位元／互斥子句／exists-arm）+ fn 定義/調用
// ─────────────────────────────────────────────────────────────────────────────

/// 宏匹配器（固定三種，覆蓋 expr/ident 兩類洞與單雙參）。
#[derive(Clone, Copy, Debug, PartialEq)]
enum Matcher {
    E1, // ($e:expr)
    E2, // ($e:expr, $f:expr)
    I1, // ($v:ident)
}

/// 宏臂模板：帶洞的表達式（渲染進 `macro_rules!` 定義）。
#[derive(Clone, Debug)]
enum Tx {
    TLitI(i64),
    TLitB(bool),
    TUnit,
    HoleE(usize), // $e (0) / $f (1)
    HoleI,        // $v
    TNeg(Box<Tx>),
    TNot(Box<Tx>),
    TDeref(Box<Tx>),
    TAdd(Box<Tx>, Box<Tx>),
    TEq(Box<Tx>, Box<Tx>),
    TRefH,    // & $v（I1 限定）
    TRefMutH, // &mut $v（I1 限定）
}

/// 宏定義（枚舉側）：匹配器 + 1..=2 個臂。
#[derive(Clone, Debug)]
struct SMacro {
    matcher: Matcher,
    arms: Vec<Tx>,
}

/// fn 定義（枚舉側）：參數型別 + 回傳型別 + 體。
#[derive(Clone, Debug)]
struct SFn {
    params: Vec<Type>,
    ret: Type,
    body: Sx,
}

/// 生成模板（恰好 `size` 節點；洞須符合匹配器）。
fn gen_templates(size: usize, matcher: Matcher, out: &mut Vec<Tx>, cap: usize) {
    if out.len() >= cap {
        return;
    }
    let nholes = match matcher {
        Matcher::E1 => 1,
        Matcher::E2 => 2,
        Matcher::I1 => 0,
    };
    if size == 1 {
        out.push(Tx::TLitI(0));
        out.push(Tx::TLitI(1));
        out.push(Tx::TLitB(true));
        out.push(Tx::TLitB(false));
        out.push(Tx::TUnit);
        for h in 0..nholes {
            out.push(Tx::HoleE(h));
        }
        if matcher == Matcher::I1 {
            out.push(Tx::HoleI);
            out.push(Tx::TRefH);
            out.push(Tx::TRefMutH);
        }
        return;
    }
    let mut children = Vec::new();
    gen_templates(size - 1, matcher, &mut children, cap);
    for c in &children {
        out.push(Tx::TNeg(Box::new(c.clone())));
        out.push(Tx::TNot(Box::new(c.clone())));
        out.push(Tx::TDeref(Box::new(c.clone())));
    }
    for i in 1..=size - 2 {
        let mut ls = Vec::new();
        let mut rs = Vec::new();
        gen_templates(i, matcher, &mut ls, cap);
        gen_templates(size - 1 - i, matcher, &mut rs, cap);
        for l in &ls {
            for r in &rs {
                out.push(Tx::TAdd(Box::new(l.clone()), Box::new(r.clone())));
                out.push(Tx::TEq(Box::new(l.clone()), Box::new(r.clone())));
            }
        }
    }
}

fn render_template(t: &Tx) -> String {
    match t {
        Tx::TLitI(n) => n.to_string(),
        Tx::TLitB(b) => if *b { "true".into() } else { "false".into() },
        Tx::TUnit => "()".into(),
        Tx::HoleE(0) => "$e".into(),
        Tx::HoleE(_) => "$f".into(),
        Tx::HoleI => "$v".into(),
        Tx::TNeg(c) => format!("(-{})", render_template(c)),
        Tx::TNot(c) => format!("(!{})", render_template(c)),
        Tx::TDeref(c) => format!("(*({}))", render_template(c)),
        Tx::TAdd(a, b) => format!("({} + {})", render_template(a), render_template(b)),
        Tx::TEq(a, b) => format!("({} == {})", render_template(a), render_template(b)),
        Tx::TRefH => "&$v".into(),
        Tx::TRefMutH => "&mut $v".into(),
    }
}

fn render_matcher(m: Matcher) -> &'static str {
    match m {
        Matcher::E1 => "($e:expr)",
        Matcher::E2 => "($e:expr, $f:expr)",
        Matcher::I1 => "($v:ident)",
    }
}

/// 生成全部宏定義（代價 = 1 + 臂數 + Σ模板節點 ≤ `budget`）。
fn gen_macros(budget: usize, cap: usize) -> Vec<SMacro> {
    let mut out = Vec::with_capacity(8);
    for matcher in [Matcher::E1, Matcher::E2, Matcher::I1] {
        for narms in 1..=2usize {
            let overhead = 1 + narms;
            if budget < overhead + narms {
                continue; // 每臂模板至少 1 節點
            }
            // 模板節點總預算分配到各臂（每臂 ≥1）
            for split in splits(budget - overhead, narms) {
                let mut arm_lists: Vec<Vec<Tx>> = vec![Vec::new()];
                for &sz in &split {
                    let mut part = Vec::new();
                    gen_templates(sz, matcher, &mut part, cap);
                    let mut next = Vec::new();
                    for list in &arm_lists {
                        for t in &part {
                            let mut l2 = list.clone();
                            l2.push(t.clone());
                            next.push(l2);
                        }
                    }
                    arm_lists = next;
                    if arm_lists.len() > cap {
                        arm_lists.truncate(cap);
                    }
                }
                for arms in arm_lists {
                    out.push(SMacro { matcher, arms });
                    if out.len() >= cap {
                        return out;
                    }
                }
            }
        }
    }
    out
}

fn render_macro(m: &SMacro, name: &str) -> String {
    let arms: Vec<String> = m
        .arms
        .iter()
        .map(|t| format!("{} => {{ {} }}", render_matcher(m.matcher), render_template(t)))
        .collect();
    format!("macro_rules! {} {{ {} }}", name, arms.join("; "))
}

/// fn 簽名空間：參數 0..=2 個（i32/bool），回傳 i32/bool/unit。
fn gen_signatures() -> Vec<(Vec<Type>, Type)> {
    let ptys = [Type::I32, Type::Bool];
    let rtys = [Type::I32, Type::Bool, Type::Unit];
    let mut out = Vec::with_capacity(8);
    for &r in &rtys {
        out.push((Vec::new(), r));
        for &p0 in &ptys {
            out.push((vec![p0], r));
            for &p1 in &ptys {
                out.push((vec![p0, p1], r));
            }
        }
    }
    out
}

/// 生成 fn 定義列表（總代價 = Σ(1 + 體節點) == `budget`；`nfns` 個）。
fn gen_fns(budget: usize, nfns: usize, nmacros: usize, cap: usize) -> Vec<Vec<SFn>> {
    if nfns == 0 {
        return if budget == 0 { vec![Vec::new()] } else { Vec::new() };
    }
    if budget < nfns {
        return Vec::new();
    }
    let sigs = gen_signatures();
    // 體節點預算分配（每體 ≥1）
    let mut out: Vec<Vec<SFn>> = Vec::new();
    for split in splits(budget - nfns, nfns) {
        // 逐個 fn 做笛卡爾積
        let mut partial: Vec<Vec<SFn>> = vec![Vec::new()];
        for &sz in &split {
            let mut next = Vec::new();
            for (params, ret) in &sigs {
                let mut bodies = Vec::new();
                gen_seq(sz, params.len(), (nfns, nmacros), &mut bodies, cap);
                for list in &partial {
                    for body in &bodies {
                        let mut l2 = list.clone();
                        l2.push(SFn {
                            params: params.clone(),
                            ret: *ret,
                            body: body.clone(),
                        });
                        next.push(l2);
                        if next.len() >= cap {
                            return next;
                        }
                    }
                }
            }
            partial = next;
        }
        out.extend(partial);
        if out.len() >= cap {
            out.truncate(cap);
            return out;
        }
    }
    out
}

fn render_fn(f: &SFn, name: &str, next_name: &mut usize) -> String {
    let mut env: Vec<String> = Vec::new();
    let params: Vec<String> = f
        .params
        .iter()
        .map(|ty| {
            let pname = format!("p{}", next_name);
            *next_name += 1;
            env.push(pname.clone());
            format!("{}: {}", pname, ty.name())
        })
        .collect();
    let body = render(&f.body, &mut env, next_name);
    format!(
        "fn {}({}) -> {} {{ {} }}",
        name,
        params.join(", "),
        f.ret.name(),
        body
    )
}

/// V2 窮舉：≤ `budget` 總節點的全部「宏(0..1) × fn(0..2) × main」程序。
///
/// 覆蓋：臂位元／臂互斥子句、exists-arm 語義、轉錄失敗臂、未定義宏/無匹配臂、
/// fn 調用（含實參個數不符）、宏×借用交互（I1 模板的 &mut $v 對）。
pub fn run_oracle_full(budget: usize, cap: usize) -> OracleReport {
    let t0 = Instant::now();
    let mut rep = OracleReport {
        max_size: budget,
        ..Default::default()
    };

    for nmacros in 0..=1usize {
        for nfns in 0..=2usize {
            for bm in 0..=budget {
                if (nmacros == 0) != (bm == 0) {
                    continue; // 無宏 ⇒ 宏預算必 0；有宏 ⇒ 最小代價 3
                }
                if nmacros == 1 && bm < 3 {
                    continue;
                }
                for bf in 0..=budget - bm {
                    if (nfns == 0) != (bf == 0) {
                        continue;
                    }
                    if nfns > 0 && bf < nfns {
                        continue;
                    }
                    let bmain = budget - bm - bf;
                    if bmain < 1 {
                        continue;
                    }
                    let macros: Vec<Vec<SMacro>> = if nmacros == 1 {
                        // 每個宏定義自成一個程序配置（恰好一個宏）
                        let ms = gen_macros(bm, cap);
                        if ms.is_empty() {
                            continue;
                        }
                        ms.into_iter().map(|m| vec![m]).collect()
                    } else {
                        vec![Vec::new()] // 單一情況：無宏
                    };
                    let fn_lists = gen_fns(bf, nfns, nmacros, cap);
                    if fn_lists.is_empty() {
                        continue;
                    }
                    let mut mains = Vec::new();
                    gen_seq(bmain, 0, (nfns, nmacros), &mut mains, cap);
                    for ml in &macros {
                        for fns in &fn_lists {
                            for main in &mains {
                                if rep.total >= cap as u64 {
                                    rep.capped = true;
                                    rep.elapsed_ms = t0.elapsed().as_millis();
                                    return rep;
                                }
                                let mut next_name = 0usize;
                                let mut src = String::new();
                                for (i, m) in ml.iter().enumerate() {
                                    src.push_str(&render_macro(m, &format!("m{}", i)));
                                    src.push('\n');
                                }
                                for (i, f) in fns.iter().enumerate() {
                                    src.push_str(&render_fn(f, &format!("f{}", i), &mut next_name));
                                    src.push('\n');
                                }
                                let mut env: Vec<String> = Vec::new();
                                let body = render(main, &mut env, &mut next_name);
                                src.push_str(&format!("fn main() {{\n{}\n}}", body));
                                check_one(&src, &mut rep);
                            }
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

/// 實際使用：exhaust.rs 文件清單 — 優化 with_capacity
pub fn exhaust_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("exhaust.rs", "exhaust.rs 正式運作 — 優化 with_capacity", "core/src/exhaust.rs"),
    ]
}

