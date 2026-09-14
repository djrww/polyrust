//! polyrust：Mini-Rust 宏的代數形式化管線
//! CDCL（子句學習）× Buchberger（Gröbner 基化簡）× QAP（算術程序驗證）
//!
//! 執行：cargo run --release [-- demo|obligations|all]

mod cdcl;
mod codegen;
mod driver;
mod dsl;
mod fp;
mod formal;
mod frac;
mod groebner;
mod json;
mod llm;
mod minirust;
mod obligations;
mod pipeline;
mod poly;
mod qap;
mod server;

use pipeline::PipelineResult;

const DEMO_A: &str = r#"
// Demo A：良構程序 —— sqr! 宏（表達式重複使用）+ 函式語義對照
fn sqr(x: i32) -> i32 { x * x }
macro_rules! sqr { ($e:expr) => { $e * $e } }
fn main() {
    let a = 5;
    let b = sqr!(a + 1);
    let c = sqr(3) + sqr!(2);
}
"#;

const DEMO_B: &str = r#"
// Demo B：型別錯誤 —— bad! 宏生成 1 + true
macro_rules! bad { ($e:expr) => { $e + true } }
fn main() {
    let b = bad!(1);
}
"#;

const DEMO_C: &str = r#"
// Demo C：借用錯誤 —— 兩個重疊的 &mut（存活區間 [r1 建立點, 最後使用點] 重疊）
macro_rules! twice_mut { ($v:ident) => { let r1 = &mut $v; let r2 = &mut $v; *r1 + *r2 } }
fn main() {
    let x = 0;
    let u = twice_mut!(x);
}
"#;

const DEMO_D: &str = r#"
// Demo D：型別導向臂選擇 —— 兩臂皆語法匹配，只有型別正確的臂可行
macro_rules! pick { ($a:expr) => { $a + 1 }; ($a:expr) => { !$a } }
fn main() {
    let u = 3;
    let v = pick!(u);
}
"#;

fn hr(title: &str) {
    println!("\n{}", "═".repeat(72));
    println!("  {}", title);
    println!("{}", "═".repeat(72));
}

fn report(r: &PipelineResult, verbose_poly: bool) {
    let src_first_line = "";
    let _ = src_first_line;
    println!("□ 來源（見上方源碼）");
    println!("□ 宏展開（衛生轉錄；模板引入的識別字改名 name#hN）：");
    for l in &r.expansion_log {
        println!("    {}", l);
    }
    println!("□ Ground truth（直接型別檢查器，exists-arm 語義）：{}", r.checker_msg);
    println!("□ 代數編碼：變量 {} 個（型別位元 t_v:τ + 臂位元 a + 借用位元 b），多項式生成元 {} 條，子句 {} 條",
        r.n_vars, r.n_polys, r.n_clauses);
    println!("□ CDCL(T) 迴圈：{} 輪 | 決策 {} 傳播 {} 衝突 {} | 學習子句 {} 條",
        r.cdcl_rounds, r.cdcl_stats.decisions, r.cdcl_stats.propagations, r.cdcl_stats.conflicts, r.cdcl_stats.learned);
    for lc in r.learned_clauses.iter().take(6) {
        let txt: Vec<String> = lc
            .iter()
            .map(|&l| {
                let v = cdcl::lit_var(l);
                format!("{}{}", if cdcl::lit_positive(l) { "" } else { "¬" }, v)
            })
            .collect();
        println!("    學習子句：({})", txt.join(" ∨ "));
    }
    let s = &r.gb_stats;
    println!("□ Buchberger（grevlex，雙準則）：");
    println!("    生成元 {} | 配對 {} | 第一準則消除 {} | 第二準則消除 {} | 實算 S-多項式 {}（歸零 {}）| 基擴充 {} | 約化基 {} 條",
        s.generators, s.pairs_considered, s.crit1_skips, s.crit2_skips, s.s_polys, s.reductions_to_zero, s.basis_adds, r.reduced_basis.len());
    println!("    化簡率：{} 條生成元 → {} 條約化基（規範形式）",
        r.n_polys, r.reduced_basis.len());
    if verbose_poly {
        println!("    （約化基元素展示略——見 obligations / 單元測試輸出）");
    }
    if r.is_unsat {
        println!("□ 判定：1 ∈ G ⇒ 系統矛盾 ⇒ 程序不可定型（UNSAT）");
        if let Some(c) = r.reduced_basis.first() {
            if c.is_constant().map_or(false, |x| x.is_one()) {
                println!("    Gröbner 基 = {{ 1 }}：理想含 1，由 Nullstellensatz（布爾域上根式）⇒ 無 0/1 解");
            }
        }
    } else {
        println!("□ 判定：1 ∉ G ⇒ 系統可解 ⇒ SAT；求解得見證 σ（0/1 賦值）");
        let types: Vec<String> = {
            let mut v: Vec<(usize, String)> = r
                .node_types
                .iter()
                .map(|(n, t)| (*n, format!("node{}:{}", n, t.name())))
                .collect();
            v.sort();
            v.into_iter().map(|(_, s)| s).take(14).collect()
        };
        println!("    型別解碼（節點:型別）：{}{}", types.join(", "), if r.node_types.len() > 14 { " …" } else { "" });
        if !r.arm_choice.is_empty() {
            let arms: Vec<String> =
                r.arm_choice.iter().map(|(k, v)| format!("invoke#{}→臂{}", k, v + 1)).collect();
            println!("    臂選擇：{}", arms.join(", "));
        }
        println!("□ QAP：R1CS 約束 {} 條 | 導線 {} 條 | wire 多項式次數 ≤ {} | 驗證 {}",
            r.r1cs_constraints, r.r1cs_wires, r.qap_max_degree,
            match r.qap_verified { Some(true) => "通過（Z | a·b−c）", Some(false) => "★失敗★", None => "?" });
        println!("    竄改見證（位元 1→2）：{}",
            match r.qap_tamper_rejected { Some(true) => "正確拒絕 ✓", Some(false) => "★未拒絕★", None => "?" });
    }
    if let Some(f) = &r.generated_file {
        println!("□ 代碼生成：{}（round-trip 重解析 ✓{}）",
            f,
            match r.rustc_compiles { Some(true) => "，rustc 編譯 ✓", Some(false) => "，rustc 編譯 ✗", None => "" });
    }
    println!("□ 一致性：管線判定 {} ground truth {}",
        if r.is_unsat { "UNSAT" } else { "SAT" },
        if r.checker_ok { "接受" } else { "拒絕" });
    println!("   ⇒ {}", if r.agrees { "一致 ✓（定理 1/2/6 的實例）" } else { "★不一致★（bug）" });
}

fn print_banner() {
    println!("╔════════════════════════════════════════════════════════════════════╗");
    println!("║  polyrust — Rust 宏的代數形式化：CDCL × Buchberger × QAP          ║");
    println!("╚════════════════════════════════════════════════════════════════════╝");
    println!("  〔{}〕", formal::startup_report());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let json = args.iter().any(|a| a == "--json" || a == "-j");

    // 「可輸入」子命令：check / expand（含 --json 出口，供前端與 LLM 呼叫）
    if mode == "check" {
        if !json {
            print_banner();
        }
        std::process::exit(driver::cmd_check(&args, json));
    }
    if mode == "expand" {
        if !json {
            print_banner();
        }
        std::process::exit(driver::cmd_expand(&args, json));
    }
    if mode == "serve" {
        let port: u16 = args
            .get(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);
        if let Err(e) = server::serve(port) {
            eprintln!("server 錯誤：{}", e);
            std::process::exit(1);
        }
        return;
    }
    if mode == "nl" {
        // 自然語言 → .poly（LLM 護欄）。文字來源：args[2] 或 `-`（stdin）。
        if !json {
            print_banner();
        }
        std::process::exit(driver::cmd_nl(&args, json));
    }
    if mode == "gen" {
        // 檔案模式（.poly / 路徑 / stdin）：與 check/expand 一致，json 不印 banner。
        let arg2 = args.get(2).map(|s| s.as_str()).unwrap_or("A");
        let is_file = arg2.contains('/') || arg2.ends_with(".poly") || arg2 == "-";
        if is_file {
            if !json {
                print_banner();
            }
            std::process::exit(driver::cmd_gen(&args, json));
        }
        // demo 簡寫 A/B/C/D：往下走（印 banner 後處理）
    }

    print_banner();

    if mode == "all" || mode == "demo" {
        let demos: Vec<(&str, &str, bool)> = vec![
            ("demoA", DEMO_A, true),
            ("demoB", DEMO_B, false),
            ("demoC", DEMO_C, false),
            ("demoD", DEMO_D, true),
        ];
        for (name, src, codegen) in demos {
            hr(&format!("{}：{}", name, src.lines().nth(1).unwrap_or("").trim()));
            match pipeline::run_pipeline(name, src, codegen) {
                Ok(r) => report(&r, true),
                Err(e) => println!("管線錯誤：{}", e),
            }
        }
    }

    if mode == "debug" {
        // 除錯：σ_D 驗證（定理 1 的手動檢查）
        let args2: Vec<String> = std::env::args().collect();
        let src = std::fs::read_to_string(args2.get(2).expect("用法：polyrust debug <file>")).unwrap();
        let p = minirust::parse::Parser::parse_program(&src).unwrap();
        let mut exp = minirust::macros::Expander::new(p.macros.clone(), p.next_id);
        let ds = minirust::checker::check_program(&p, &mut exp).unwrap();
        let d = &ds[0];
        println!("推導：main = {:?}, 臂 {:?}", d.ty, d.arm_choice);
        let sys = minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
        println!("系統：{} vars, {} polys, {} clauses", sys.nvars, sys.polys.len(), sys.clauses.len());
        let mut sigma = vec![frac::Frac::ZERO; sys.nvars];
        for (node, t) in &d.node_types {
            if let Some(ts) = sys.node_type.get(node) {
                sigma[ts[t.index()]] = frac::Frac::ONE;
            }
        }
        for (inv, arm) in &d.arm_choice {
            if let Some(avs) = sys.arm_vars.get(inv) {
                sigma[avs[*arm]] = frac::Frac::ONE;
            }
        }
        for (_, bv) in &sys.borrow_vars {
            sigma[*bv] = frac::Frac::ONE;
        }
        let mut bad = 0;
        for (i, f) in sys.polys.iter().enumerate() {
            let v = f.eval_full(&sigma);
            if !v.is_zero() {
                bad += 1;
                if bad <= 14 {
                    println!("  違反 [{}] {} = 0（求值 = {}）", i, f.display(&sys.names), v);
                }
            }
        }
        for i in 0..sys.nvars {
            let f = poly::Poly::var(i, frac::Frac::ONE, sys.nvars).pow(2)
                .sub(&poly::Poly::var(i, frac::Frac::ONE, sys.nvars));
            if !f.eval_full(&sigma).is_zero() {
                println!("  違反域多項式 x{}", i);
            }
        }
        // 系統本身可解嗎？
        {
            use poly::Order;
            use groebner::{field_polys, reduced_groebner, solve_boolean, Strategy};
            let mut all = sys.polys.clone();
            all.extend(field_polys(sys.nvars));
            let sol = solve_boolean(&all, sys.nvars);
            println!("solve_boolean：{}", if sol.is_some() { "有解" } else { "無解" });
            let (g, s) = reduced_groebner(&all, Order::GrevLex, Strategy::Normal, true);
            let unsat = g.len() == 1 && g[0].is_constant().map_or(false, |c| c.is_one());
            println!("GB：{}（基 {} 條；S={} 基擴充={}）", if unsat { "1∈G" } else { "可解" }, g.len(), s.s_polys, s.basis_adds);
            if let Some(sol) = &sol {
                let bad = all.iter().filter(|f| !f.eval_full(sol).is_zero()).count();
                println!("見證違反 {} 條（應為 0）", bad);
                // 違反子句？
                for (ci, c) in sys.clauses.iter().enumerate() {
                    let ok = c.iter().any(|&l| {
                        let v = cdcl::lit_var(l);
                        sol[v] == frac::Frac::from_i64(cdcl::lit_positive(l) as i64)
                    });
                    if !ok { println!("見證違反子句 [{}]", ci); }
                }
            }
        }
        let mut missing = vec![];
        for node in sys.node_type.keys() {
            if !d.node_types.contains_key(node) {
                missing.push(*node);
            }
        }
        missing.sort();
        println!("共 {} 條約束被 σ_D 違反；推導未覆蓋節點：{:?}", bad, missing);
        return;
    }

    if mode == "gen" {
        // 只處理 demo 簡寫 A/B/C/D（檔案模式已在上方提前處理）。
        let arg2 = args.get(2).map(|s| s.as_str()).unwrap_or("A");
        let (src, cg) = match arg2 {
            "A" => (DEMO_A, true),
            "D" => (DEMO_D, true),
            "B" => (DEMO_B, false),
            "C" => (DEMO_C, false),
            _ => (DEMO_A, true),
        };
        match pipeline::run_pipeline("demo", src, cg) {
            Ok(r) => {
                println!("{}", r.generated_code.clone().unwrap_or_else(|| "(no code)".to_string()));
            }
            Err(e) => println!("錯誤：{}", e),
        }
        return;
    }

    if mode == "all" || mode == "obligations" {
        hr("義務自證（九條定理的機械化檢查）");
        let results = obligations::run_all();
        let mut all_pass = true;
        for r in &results {
            println!("\n[{}] {} — {}", r.id, r.name, if r.pass { "PASS ✓" } else { "FAIL ✗" });
            println!("    義務：{}", r.statement);
            println!("    見證：{}", r.detail);
            all_pass &= r.pass;
        }
        hr("總結");
        println!("九條義務自證：{}",
            if all_pass { "全部通過 ✓✓✓（命題 P 成立的機械見證；數學證明見 docs/THEOREMS.md）" } else { "有失敗項 ✗" });
        std::process::exit(if all_pass { 0 } else { 1 });
    }
}
