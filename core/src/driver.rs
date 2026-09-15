//! 命令驅動：`check` / `expand` 兩個「可輸入」子命令，以及其 JSON 出口。
//!
//! 這是「驗證型工具 + LLM 接口」的落地層：
//! - `check <file.poly>`：走完整管線（驗證 + 求解 + 代碼生成），輸出判定。
//! - `expand <file.poly>`：純宏展開（描述 → 展開碼，即 DSL 生成的第一層）。
//! - `--json` / `--stdin`：結構化出口，供前端與 LLM agent 呼叫。
//!
//! 核心計算抽成 `check_text_json` / `expand_text_json`，供 CLI（此檔）與
//! HTTP server（`server.rs`）共用，確保兩者輸出同一份契約。

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::dsl::{resolve, PolySource};
use crate::json::J;
use crate::pipeline::{run_pipeline, PipelineResult};

/// 讀取 `.poly` 來源。`path == "-"` 時改讀 stdin（LLM agent 模式）。
/// 回傳 (名稱, 文本, 來源目錄)；stdin 無來源目錄（base = None）。
fn read_source(path: &str) -> Result<(String, String, Option<PathBuf>), String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("讀取 stdin 失敗：{}", e))?;
        return Ok(("stdin".to_string(), buf, None));
    }
    let p = Path::new(path);
    let text = std::fs::read_to_string(p).map_err(|e| format!("讀取 {} 失敗：{}", path, e))?;
    let name = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());
    let base = p.parent().map(|d| d.to_path_buf());
    Ok((name, text, base))
}

/// 判定字串。
fn verdict(p: &PipelineResult) -> &'static str {
    if p.is_unsat {
        "UNSAT"
    } else {
        "SAT"
    }
}

/// 把管線結果序列化成 JSON（LLM 接口契約）。
pub(crate) fn pipeline_to_json(name: &str, poly: &PolySource, p: &PipelineResult) -> J {
    let node_types: Vec<(String, J)> = {
        let mut v: Vec<(usize, &crate::minirust::ast::Type)> =
            p.node_types.iter().map(|(k, v)| (*k, v)).collect();
        v.sort_by_key(|(k, _)| *k);
        v.into_iter()
            .map(|(k, t)| (format!("node{}", k), J::s(t.name())))
            .collect()
    };
    let arm_choice: Vec<(String, J)> = {
        let mut v: Vec<(usize, usize)> =
            p.arm_choice.iter().map(|(k, v)| (*k, *v)).collect();
        v.sort_by_key(|(k, _)| *k);
        v.into_iter()
            .map(|(k, a)| (format!("invoke#{}", k), J::Int(a as i64)))
            .collect()
    };
    let metadata: Vec<(String, J)> = poly
        .metadata
        .iter()
        .map(|(k, v)| (k.clone(), J::s(v)))
        .collect();

    J::obj(vec![
        ("api_version", J::s("0.1")),
        ("mode", J::s("check")),
        ("source", J::s(name)),
        ("intent", J::opt_str(poly.intent.as_deref())),
        ("metadata", J::Obj(metadata)),
        ("status", J::s("ok")),
        ("verdict", J::s(verdict(p))),
        ("typechecks", J::Bool(p.checker_ok)),
        ("checker_msg", J::s(&p.checker_msg)),
        ("agrees", J::Bool(p.agrees)),
        ("stats", J::obj(vec![
            ("n_vars", J::Int(p.n_vars as i64)),
            ("n_polys", J::Int(p.n_polys as i64)),
            ("n_clauses", J::Int(p.n_clauses as i64)),
            ("cdcl_rounds", J::Int(p.cdcl_rounds as i64)),
            ("cdcl_decisions", J::Int(p.cdcl_stats.decisions as i64)),
            ("cdcl_propagations", J::Int(p.cdcl_stats.propagations as i64)),
            ("cdcl_conflicts", J::Int(p.cdcl_stats.conflicts as i64)),
            ("cdcl_learned", J::Int(p.cdcl_stats.learned as i64)),
            ("gb_generators", J::Int(p.gb_stats.generators as i64)),
            ("gb_pairs_considered", J::Int(p.gb_stats.pairs_considered as i64)),
            ("gb_s_polys", J::Int(p.gb_stats.s_polys as i64)),
            ("gb_basis_adds", J::Int(p.gb_stats.basis_adds as i64)),
            ("reduced_basis_size", J::Int(p.reduced_basis.len() as i64)),
            ("r1cs_constraints", J::Int(p.r1cs_constraints as i64)),
            ("r1cs_wires", J::Int(p.r1cs_wires as i64)),
            ("qap_max_degree", J::Int(p.qap_max_degree as i64)),
            ("qap_verified", J::opt_bool(p.qap_verified)),
            ("qap_tamper_rejected", J::opt_bool(p.qap_tamper_rejected)),
        ])),
        ("result", J::obj(vec![
            ("node_types", J::Obj(node_types)),
            ("arm_choice", J::Obj(arm_choice)),
        ])),
        ("generated", J::obj(vec![
            ("code", J::opt_str(p.generated_code.as_deref())),
            ("file", J::opt_str(p.generated_file.as_deref())),
            ("rustc_compiles", J::opt_bool(p.rustc_compiles)),
        ])),
        ("expansion_log", J::Arr(p.expansion_log.iter().map(|s| J::s(s)).collect())),
    ])
}

pub fn error_json(name: &str, poly: Option<&PolySource>, mode: &str, msg: &str) -> J {
    J::obj(vec![
        ("api_version", J::s("0.1")),
        ("mode", J::s(mode)),
        ("source", J::s(name)),
        ("intent", J::opt_str(poly.and_then(|p| p.intent.as_deref()))),
        ("status", J::s("error")),
        ("error", J::s(msg)),
    ])
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase3 v2 pipeline JSON
// ─────────────────────────────────────────────────────────────────────────────

fn verdict_v2(p: &crate::pipeline_v2::PipelineV2Result) -> &'static str {
    if p.is_unsat { "UNSAT" } else { "SAT" }
}

fn pipeline_v2_to_json(name: &str, poly: &PolySource, p: &crate::pipeline_v2::PipelineV2Result) -> J {
    let metadata: Vec<(String, J)> = poly
        .metadata
        .iter()
        .map(|(k, v)| (k.clone(), J::s(v)))
        .collect();
    J::obj(vec![
        ("api_version", J::s("0.2")),
        ("mode", J::s("check-v2")),
        ("source", J::s(name)),
        ("intent", J::opt_str(poly.intent.as_deref())),
        ("metadata", J::Obj(metadata)),
        ("status", J::s("ok")),
        ("engine", J::s(&p.engine)),
        ("verdict", J::s(verdict_v2(p))),
        ("kernel_unsat", J::opt_bool(p.kernel_unsat)),
        ("features_used", J::Arr(p.features_used.iter().map(|s| J::s(s)).collect())),
        ("type_universe_size", J::Int(p.type_universe_size as i64)),
        ("stats", J::obj(vec![
            ("n_vars", J::Int(p.n_vars as i64)),
            ("n_polys", J::Int(p.n_polys as i64)),
            ("n_clauses", J::Int(p.n_clauses as i64)),
            ("n_products", J::Int(p.n_products as i64)),
            ("n_sums", J::Int(p.n_sums as i64)),
            ("n_matches", J::Int(p.n_matches as i64)),
            ("n_loop_fuel", J::Int(p.n_loop_fuel as i64)),
            ("n_async", J::Int(p.n_async as i64)),
            ("n_lifetime", J::Int(p.n_lifetime as i64)),
            ("n_unsafe", J::Int(p.n_unsafe as i64)),
            ("n_stdlib", J::Int(p.n_stdlib as i64)),
            ("n_trait_impl", J::Int(p.n_trait_impl as i64)),
            ("r1cs_constraints", J::Int(p.r1cs_constraints as i64)),
            ("lifetime_has_cycle", J::Bool(p.lifetime_has_cycle)),
        ])),
        ("errors", J::Arr(p.errors.iter().map(|s| J::s(s)).collect())),
        ("warnings", J::Arr(p.warnings.iter().map(|s| J::s(s)).collect())),
        ("borrowck_errors", J::Arr(p.borrowck_errors.iter().map(|s| J::s(s)).collect())),
        ("effect_errors", J::Arr(p.effect_errors.iter().map(|s| J::s(s)).collect())),
        ("lowering_report", J::s(&p.lowering_report)),
        ("qap_verified", J::opt_bool(p.qap_verified)),
    ])
}

pub fn check_v2_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
    let poly = match resolve(text, base) {
        Ok(p) => p,
        Err(e) => return (error_json(name, None, "check-v2", &e), false),
    };
    match crate::pipeline_v2::run_pipeline_v2(name, &poly.source, &poly) {
        Ok(p) => {
            let ok = !p.is_unsat;
            // ok still returns true for SAT, but even UNSAT is a valid verification result, so status ok
            (pipeline_v2_to_json(name, &poly, &p), true)
        }
        Err(e) => (error_json(name, Some(&poly), "check-v2", &e), false),
    }
}

pub fn cmd_check_v2(args: &[String], json: bool) -> i32 {
    let path = args.get(2).cloned().unwrap_or_else(|| "-".to_string());
    let (name, text, base) = match read_source(&path) {
        Ok(x) => x,
        Err(e) => {
            if json {
                println!("{}", error_json(&path, None, "check-v2", &e));
            } else {
                eprintln!("錯誤：{}", e);
            }
            return 1;
        }
    };
    if json {
        let (j, ok) = check_v2_text_json(&name, &text, base.as_deref());
        println!("{}", j);
        return if ok { 0 } else { 1 };
    }
    // 人類可讀
    let poly = match resolve(&text, base.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("錯誤：{}", e);
            return 1;
        }
    };
    match crate::pipeline_v2::run_pipeline_v2(&name, &poly.source, &poly) {
        Ok(p) => {
            if let Some(i) = &poly.intent {
                println!("意圖：{}", i);
            }
            println!("來源：{}  判定：{}  engine：{}  特性：{}", name, verdict_v2(&p), p.engine, p.features_used.join(", "));
            println!(
                "  編碼：變量 {} | 多項式 {} | 子句 {} | 積 {} | 和 {} | match {} | fuel {} | async {} | lifetime {} | unsafe {} | stdlib {} | trait_impl {}",
                p.n_vars, p.n_polys, p.n_clauses, p.n_products, p.n_sums, p.n_matches, p.n_loop_fuel, p.n_async, p.n_lifetime, p.n_unsafe, p.n_stdlib, p.n_trait_impl
            );
            println!("  類型宇宙：{}  lifetime cycle：{}  QAP：{:?}", p.type_universe_size, p.lifetime_has_cycle, p.qap_verified);
            if !p.errors.is_empty() {
                println!("  錯誤：{}", p.errors.join("; "));
            }
            if !p.warnings.is_empty() {
                println!("  警告：{}", p.warnings.join("; "));
            }
            println!("\n── lowering ──\n{}", p.lowering_report);
            0
        }
        Err(e) => {
            eprintln!("錯誤：{}", e);
            1
        }
    }
}

/// `check` 核心：由（名稱, 文本, 來源目錄）→ JSON。`bool` 表示 status == ok。
pub fn check_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
    let poly = match resolve(text, base) {
        Ok(p) => p,
        Err(e) => return (error_json(name, None, "check", &e), false),
    };
    match run_pipeline(name, &poly.source, true) {
        Ok(p) => (pipeline_to_json(name, &poly, &p), true),
        Err(e) => (error_json(name, Some(&poly), "check", &e), false),
    }
}

/// `expand` 核心：由（名稱, 文本, 來源目錄）→ JSON。`bool` 表示 status == ok。
pub fn expand_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
    let poly = match resolve(text, base) {
        Ok(p) => p,
        Err(e) => return (error_json(name, None, "expand", &e), false),
    };
    let p = match crate::minirust::parse::Parser::parse_program(&poly.source) {
        Ok(p) => p,
        Err(e) => return (error_json(name, Some(&poly), "expand", &e), false),
    };
    let mut exp = crate::minirust::macros::Expander::new(p.macros.clone(), p.next_id);
    match crate::minirust::checker::check_program(&p, &mut exp) {
        Ok(_) => {
            let mut items: Vec<(usize, usize, String)> = exp
                .memo_text
                .iter()
                .map(|((node, arm), text)| (*node, *arm, text.clone()))
                .collect();
            items.sort();
            let macros: Vec<J> = p
                .macros
                .iter()
                .map(|m| J::obj(vec![
                    ("name", J::s(&m.name)),
                    ("arms", J::Int(m.arms.len() as i64)),
                ]))
                .collect();
            let expansions: Vec<J> = items
                .iter()
                .map(|(node, arm, text)| J::obj(vec![
                    ("node", J::Int(*node as i64)),
                    ("arm", J::Int(*arm as i64)),
                    ("text", J::s(text)),
                ]))
                .collect();
            let metadata: Vec<(String, J)> = poly
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), J::s(v)))
                .collect();
            let out = J::obj(vec![
                ("api_version", J::s("0.1")),
                ("mode", J::s("expand")),
                ("source", J::s(name)),
                ("intent", J::opt_str(poly.intent.as_deref())),
                ("metadata", J::Obj(metadata)),
                ("status", J::s("ok")),
                ("macros", J::Arr(macros)),
                ("expansions", J::Arr(expansions)),
            ]);
            (out, true)
        }
        Err(e) => (error_json(name, Some(&poly), "expand", &e), false),
    }
}

/// `polyrust check <file.poly> [--json]`：完整管線（驗證 + 求解 + 代碼生成）。
pub fn cmd_check(args: &[String], json: bool) -> i32 {
    let path = args.get(2).cloned().unwrap_or_else(|| "-".to_string());
    let (name, text, base) = match read_source(&path) {
        Ok(x) => x,
        Err(e) => {
            if json {
                println!("{}", error_json(&path, None, "check", &e));
            } else {
                eprintln!("錯誤：{}", e);
            }
            return 1;
        }
    };

    if json {
        let (j, ok) = check_text_json(&name, &text, base.as_deref());
        println!("{}", j);
        return if ok { 0 } else { 1 };
    }

    // 人類可讀路徑
    let poly = match resolve(&text, base.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("錯誤：{}", e);
            return 1;
        }
    };
    match run_pipeline(&name, &poly.source, true) {
        Ok(p) => {
            print_check_human(&name, &poly, &p);
            0
        }
        Err(e) => {
            eprintln!("錯誤：{}", e);
            1
        }
    }
}

/// 人類可讀的 check 輸出。
fn print_check_human(name: &str, poly: &PolySource, p: &PipelineResult) {
    if let Some(i) = &poly.intent {
        println!("意圖：{}", i);
    }
    println!("來源：{}", name);
    println!(
        "判定：{}（可定型） | 型別檢查：{} | 一致性：{}",
        verdict(p),
        if p.checker_ok { "接受" } else { "拒絕" },
        if p.agrees { "一致 ✓" } else { "★不一致★" }
    );
    println!(
        "  代數編碼：變量 {} | 生成元 {} | 子句 {} | CDCL {} 輪",
        p.n_vars, p.n_polys, p.n_clauses, p.cdcl_rounds
    );
    println!(
        "  QAP：約束 {} | 導線 {} | 驗證 {}",
        p.r1cs_constraints,
        p.r1cs_wires,
        match p.qap_verified {
            Some(true) => "通過 ✓",
            Some(false) => "★失敗★",
            None => "?",
        }
    );
    if !p.node_types.is_empty() {
        let mut v: Vec<_> = p.node_types.iter().collect();
        v.sort_by_key(|(k, _)| *k);
        let s: Vec<String> = v
            .into_iter()
            .map(|(k, t)| format!("node{}:{}", k, t.name()))
            .collect();
        println!("  型別解碼：{}", s.join(", "));
    }
    if let Some(f) = &p.generated_file {
        println!(
            "  生成碼：{}（rustc 編譯 {}）",
            f,
            match p.rustc_compiles {
                Some(true) => "✓",
                Some(false) => "✗",
                None => "?",
            }
        );
    }
    if let Some(code) = &p.generated_code {
        println!("\n──────────────── 生成 Rust 代碼 ────────────────");
        println!("{}", code);
    }
}

/// `polyrust expand <file.poly> [--json]`：純宏展開（DSL 生成的第一層）。
pub fn cmd_expand(args: &[String], json: bool) -> i32 {
    let path = args.get(2).cloned().unwrap_or_else(|| "-".to_string());
    let (name, text, base) = match read_source(&path) {
        Ok(x) => x,
        Err(e) => {
            if json {
                println!("{}", error_json(&path, None, "expand", &e));
            } else {
                eprintln!("錯誤：{}", e);
            }
            return 1;
        }
    };

    if json {
        let (j, ok) = expand_text_json(&name, &text, base.as_deref());
        println!("{}", j);
        return if ok { 0 } else { 1 };
    }

    // 人類可讀路徑
    let poly = match resolve(&text, base.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("錯誤：{}", e);
            return 1;
        }
    };
    let p = match crate::minirust::parse::Parser::parse_program(&poly.source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("錯誤：{}", e);
            return 1;
        }
    };
    let mut exp = crate::minirust::macros::Expander::new(p.macros.clone(), p.next_id);
    match crate::minirust::checker::check_program(&p, &mut exp) {
        Ok(_) => {
            let mut items: Vec<(usize, usize, String)> = exp
                .memo_text
                .iter()
                .map(|((node, arm), text)| (*node, *arm, text.clone()))
                .collect();
            items.sort();
            if let Some(i) = &poly.intent {
                println!("意圖：{}", i);
            }
            println!("來源：{}", name);
            for m in &p.macros {
                println!("宏：{}（{} 個臂）", m.name, m.arms.len());
            }
            if items.is_empty() {
                println!("（無宏調用）");
            }
            for (node, arm, text) in &items {
                println!("invoke#{} 臂{} → {}", node, arm + 1, text);
            }
            0
        }
        Err(e) => {
            eprintln!("錯誤：{}", e);
            1
        }
    }
}

/// `generate` 核心：由（名稱, 文本, 來源目錄）→ JSON。聚焦於生成碼。
/// 這是 Phase 2 的「描述 → 生成碼」出口，供 LLM agent 與前端呼叫。
pub fn generate_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
    let poly = match resolve(text, base) {
        Ok(p) => p,
        Err(e) => return (error_json(name, None, "generate", &e), false),
    };
    match run_pipeline(name, &poly.source, true) {
        Ok(p) => {
            let ok = p.generated_code.is_some();
            let out = J::obj(vec![
                ("api_version", J::s("0.1")),
                ("mode", J::s("generate")),
                ("source", J::s(name)),
                ("intent", J::opt_str(poly.intent.as_deref())),
                ("status", J::s(if ok { "ok" } else { "error" })),
                ("verdict", J::s(verdict(&p))),
                ("typechecks", J::Bool(p.checker_ok)),
                ("generated_code", J::opt_str(p.generated_code.as_deref())),
                ("generated_file", J::opt_str(p.generated_file.as_deref())),
                ("rustc_compiles", J::opt_bool(p.rustc_compiles)),
            ]);
            (out, ok)
        }
        Err(e) => (error_json(name, Some(&poly), "generate", &e), false),
    }
}

/// `polyrust gen <file.poly> [--json]`：描述 → 生成碼（含 `@import` / `@set`）。
pub fn cmd_gen(args: &[String], json: bool) -> i32 {
    let path = args.get(2).cloned().unwrap_or_else(|| "-".to_string());
    let (name, text, base) = match read_source(&path) {
        Ok(x) => x,
        Err(e) => {
            if json {
                println!("{}", error_json(&path, None, "generate", &e));
            } else {
                eprintln!("錯誤：{}", e);
            }
            return 1;
        }
    };

    if json {
        let (j, ok) = generate_text_json(&name, &text, base.as_deref());
        println!("{}", j);
        return if ok { 0 } else { 1 };
    }

    // 人類可讀：只印生成碼
    match resolve(&text, base.as_deref()) {
        Ok(poly) => match run_pipeline(&name, &poly.source, true) {
            Ok(p) => {
                if let Some(code) = &p.generated_code {
                    println!("{}", code);
                    0
                } else {
                    eprintln!("無生成碼（判定 {}）", verdict(&p));
                    1
                }
            }
            Err(e) => {
                eprintln!("錯誤：{}", e);
                1
            }
        },
        Err(e) => {
            eprintln!("錯誤：{}", e);
            1
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// `nl`：自然語言 → .poly（LLM 護欄）
// ─────────────────────────────────────────────────────────────────────────────

/// 從 CLI 參數解析 LLM 設定（`--provider`、`--model`、`--base-url`、`--api-key`、
/// `--attempts`、`--temperature`、`--max-tokens`）。
fn parse_llm_config(args: &[String]) -> crate::llm::LlmConfig {
    use crate::llm::LlmConfig;
    let mut cfg = LlmConfig::default();
    let mut i = 0;
    let get = |i: usize| args.get(i).cloned();
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--provider" => {
                if let Some(v) = get(i + 1) {
                    cfg.provider = v;
                    i += 1;
                }
            }
            "--model" => {
                if let Some(v) = get(i + 1) {
                    cfg.model = Some(v);
                    i += 1;
                }
            }
            "--base-url" => {
                if let Some(v) = get(i + 1) {
                    cfg.base_url = Some(v);
                    i += 1;
                }
            }
            "--api-key" => {
                if let Some(v) = get(i + 1) {
                    cfg.api_key = Some(v);
                    i += 1;
                }
            }
            "--attempts" => {
                if let Some(v) = get(i + 1) {
                    if let Ok(n) = v.parse::<usize>() {
                        cfg.attempts = n.clamp(1, 16);
                    }
                    i += 1;
                }
            }
            "--temperature" => {
                if let Some(v) = get(i + 1) {
                    if let Ok(t) = v.parse::<f64>() {
                        cfg.temperature = t;
                    }
                    i += 1;
                }
            }
            "--max-tokens" => {
                if let Some(v) = get(i + 1) {
                    if let Ok(n) = v.parse::<u64>() {
                        cfg.max_tokens = n;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    cfg
}

/// `polyrust nl "<自然語言>" [--provider … --model … --json]`。
///
/// LLM 把自然語言翻成 `.poly`；每一輪輸出都經三道護欄閘門（結構 → 語法 →
/// 完整代數管線語義），失敗的精確原因會回餵給 LLM 修復。通過才生成 Rust 碼。
pub fn cmd_nl(args: &[String], json: bool) -> i32 {
    use crate::llm;

    // 自然語言來源：第一個非旗標位置參數，或 `-` 讀 stdin。
    // 帶值旗標（--provider X 等）的「值」不算位置參數，避免誤讀。
    const VALUE_FLAGS: &[&str] = &[
        "--provider",
        "--model",
        "--base-url",
        "--api-key",
        "--attempts",
        "--temperature",
        "--max-tokens",
        "--funnel-log",
    ];
    let positional: Vec<&String> = {
        let mut v: Vec<&String> = Vec::new();
        let mut skip_next = false;
        for a in args.iter().skip(2) {
            if skip_next {
                skip_next = false;
                continue;
            }
            if a.starts_with("--") {
                if VALUE_FLAGS.contains(&a.as_str()) {
                    skip_next = true;
                }
                continue;
            }
            if a == "-j" {
                continue;
            }
            v.push(a);
        }
        v
    };
    let raw = positional.first().cloned().cloned().unwrap_or_else(|| "-".to_string());
    let nl = if raw == "-" {
        let mut buf = String::new();
        if std::io::stdin().read_to_string(&mut buf).is_err() {
            if json {
                println!("{}", error_json("nl", None, "nl", "讀取 stdin 失敗"));
            } else {
                eprintln!("錯誤：讀取 stdin 失敗");
            }
            return 1;
        }
        buf
    } else {
        raw
    };
    let nl = nl.trim().to_string();
    if nl.is_empty() {
        let msg = "自然語言需求為空（用引號包住，或用 - 從 stdin 讀取）";
        if json {
            println!("{}", error_json("nl", None, "nl", msg));
        } else {
            eprintln!("錯誤：{}", msg);
        }
        return 1;
    }

    let cfg = parse_llm_config(args);

    // 漏斗日誌路徑（選填）：`--funnel-log <path>`；無則看環境變數（append 時處理）
    let funnel_path: Option<String> = args
        .iter()
        .position(|a| a == "--funnel-log")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let provider = match llm::build_provider(&cfg) {
        Ok(p) => p,
        Err(e) => {
            if json {
                println!("{}", error_json("nl", None, "nl", &e));
            } else {
                eprintln!("provider 設定錯誤：{}", e);
            }
            return 1;
        }
    };

    let result = llm::run_guardrail(provider.as_ref(), &nl, cfg.attempts, true);

    // 漏斗量測：把本次運行的 JSON 契約行追加到日誌（有設定路徑才寫）
    let log_line = llm::guardrail_to_json(&nl, &result).to_string();
    if let Err(e) = llm::funnel_log_append(funnel_path.as_deref(), &log_line) {
        eprintln!("漏斗日誌寫入失敗（不影響結果）：{}", e);
    }

    if json {
        println!("{}", log_line);
        return if result.ok { 0 } else { 1 };
    }

    // 人類可讀輸出
    println!("□ 需求：{}", nl);
    println!("□ Provider：{}", result.provider);
    println!("□ 護欄（結構 → 語法 → Tier-0 checker 快篩 → 完整代數管線）：");
    for a in &result.attempts {
        let mark = match a.outcome.as_str() {
            "sat" => "✓ 通過",
            "unsat" => "✗ 語義拒絕（完整管線 UNSAT）",
            "checker-reject" => "✗ Tier-0 checker 拒絕（等價 UNSAT，已省完整管線）",
            "structure-error" => "✗ 結構拒絕（圍欄/@intent）",
            _ => "✗ 語法拒絕",
        };
        println!("    第 {} 輪：{}", a.n, mark);
        if let Some(fb) = &a.feedback {
            for line in fb.lines().take(2) {
                println!("        ↳ {}", line);
            }
        }
    }
    if result.ok {
        println!("□ 判定：SAT — 護欄通過，形式化管線接受");
        if let Some(poly) = &result.poly {
            println!("□ 接受的 .poly：");
            for line in poly.lines() {
                println!("    {}", line);
            }
        }
        if let Some(f) = &result.generated_file {
            println!(
                "□ 生成 Rust：{}（rustc 編譯 {}）",
                f,
                match result.rustc_compiles {
                    Some(true) => "✓",
                    Some(false) => "✗",
                    None => "?",
                }
            );
        }
        0
    } else {
        println!("□ 判定：{} — {}", result.verdict, result.final_reason);
        1
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// `exhaust`：一致性 oracle（窮舉細程序空間）
// ─────────────────────────────────────────────────────────────────────────────

/// `polyrust exhaust [--space expr|full] [--size N] [--cap M] [--json]`。
///
/// 窮舉 ≤N 節點的全部小程序，逐一對照「獨立檢查器」與「代數管線判定」，
/// 不一致即失敗（借鏡 `rlzl` 暴力法 oracle 哲學；詳見 `docs/THEOREMS.md`）。
///
/// - `--space expr`（預設）：表達式空間（字面量/運算/if/let/引用/賦值；無宏無 fn）。
/// - `--space full`：全空間（宏 0..1 個 ×1..2 臂、fn 0..2 個、Call/Invoke 含邊界實參）。
pub fn cmd_exhaust(args: &[String], json: bool) -> i32 {
    let mut size = 5usize;
    let mut cap = 200_000usize;
    let mut space = "expr".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--size" => {
                if let Some(v) = args.get(i + 1) {
                    size = v.parse().unwrap_or(5).clamp(1, 9);
                    i += 1;
                }
            }
            "--cap" => {
                if let Some(v) = args.get(i + 1) {
                    cap = v.parse().unwrap_or(200_000).clamp(1, 10_000_000);
                    i += 1;
                }
            }
            "--space" => {
                if let Some(v) = args.get(i + 1) {
                    space = v.clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let rep = match space.as_str() {
        "full" => crate::exhaust::run_oracle_full(size, cap),
        _ => crate::exhaust::run_oracle(size, cap),
    };

    if json {
        let mismatches: Vec<crate::json::J> = rep
            .mismatches
            .iter()
            .chain(rep.type_mismatches.iter())
            .chain(rep.internal_errors.iter())
            .map(|s| crate::json::J::s(s))
            .collect();
        println!(
            "{}",
            crate::json::J::obj(vec![
                ("api_version", crate::json::J::s("0.1")),
                ("mode", crate::json::J::s("exhaust")),
                ("space", crate::json::J::s(if space == "full" { "full" } else { "expr" })),
                ("max_size", crate::json::J::Int(rep.max_size as i64)),
                ("total", crate::json::J::Int(rep.total as i64)),
                ("sat", crate::json::J::Int(rep.sat as i64)),
                ("unsat", crate::json::J::Int(rep.unsat as i64)),
                ("verdict_mismatches", crate::json::J::Int(rep.mismatches.len() as i64)),
                ("type_mismatches", crate::json::J::Int(rep.type_mismatches.len() as i64)),
                ("internal_errors", crate::json::J::Int(rep.internal_errors.len() as i64)),
                ("details", crate::json::J::Arr(mismatches)),
                ("capped", crate::json::J::Bool(rep.capped)),
                ("pass", crate::json::J::Bool(rep.pass())),
                ("elapsed_ms", crate::json::J::Int(rep.elapsed_ms as i64)),
            ])
        );
        return if rep.pass() { 0 } else { 1 };
    }

    println!(
        "□ 窮舉空間（{}）：≤{} 總節點{}{}",
        if space == "full" {
            "full：宏 0..1 ×1..2 臂 + fn 0..2 + Call/Invoke 邊界"
        } else {
            "expr：字面量/-/!/*/+/==/if/let/&/&mut/賦值；無宏無 fn"
        },
        rep.max_size,
        if space == "full" { "（宏×fn×main 全組合）" } else { " 表達式" },
        if rep.capped { "（已觸上限截斷）" } else { "" }
    );
    println!("□ 程序總數：{}（SAT {} / UNSAT {}）", rep.total, rep.sat, rep.unsat);
    println!(
        "□ 判定一致性（管線 ⟺ 檢查器）：{}",
        if rep.mismatches.is_empty() {
            format!("{} 個不一致（全部一致 ✓）", 0)
        } else {
            format!("★ {} 個不一致 ★", rep.mismatches.len())
        }
    );
    println!(
        "□ 型別解碼一致性（SAT 逐節點）：{}",
        if rep.type_mismatches.is_empty() {
            "全部一致 ✓"
        } else {
            "★ 有不一致 ★"
        }
    );
    println!(
        "□ 解析失敗（生成器內部錯誤）：{}{}",
        rep.internal_errors.len(),
        if rep.internal_errors.is_empty() { " ✓" } else { " ★" }
    );
    for m in rep.mismatches.iter().chain(rep.type_mismatches.iter()).take(5) {
        println!("──── 反例 ────\n{}", m);
    }
    println!("□ 耗時：{} ms", rep.elapsed_ms);
    if rep.pass() {
        println!("□ 結論：兩條路徑在有界空間上完全一致（命題 P 判定等價的窮舉見證）");
        0
    } else {
        1
    }
}

/// `polyrust funnel [日誌路徑] [--json]`：護欄漏斗量測（借鏡 `rlzl` 測量文化）。
///
/// 讀取 `nl` 運行累積的 NDJSON 漏斗日誌（路徑優先序：位置參數 >
/// `POLYRUST_FUNNEL_LOG` > 預設 `polyrust-funnel.ndjson`），聚合
/// 「結構 → 語法 → Tier-0 checker → 完整管線」各層的攔截與通過數。
pub fn cmd_funnel(args: &[String], json: bool) -> i32 {
    let path = args
        .get(2)
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .or_else(|| std::env::var("POLYRUST_FUNNEL_LOG").ok())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "polyrust-funnel.ndjson".to_string());

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            let msg = format!(
                "讀取漏斗日誌 {} 失敗：{}（先以 `polyrust nl ... --funnel-log {}` 或環境變數 POLYRUST_FUNNEL_LOG 累積運行記錄）",
                path, e, path
            );
            if json {
                println!("{}", error_json("funnel", None, "funnel", &msg));
            } else {
                eprintln!("錯誤：{}", msg);
            }
            return 1;
        }
    };

    let stats = match crate::llm::funnel_from_json_text(&text) {
        Ok(s) => s,
        Err(e) => {
            if json {
                println!("{}", error_json("funnel", None, "funnel", &e));
            } else {
                eprintln!("錯誤：{}", e);
            }
            return 1;
        }
    };

    if json {
        println!("{}", crate::llm::funnel_to_json(&stats));
        return 0;
    }

    println!("□ 漏斗日誌：{}", path);
    if stats.runs == 0 {
        println!("□ 日誌為空（尚無護欄運行記錄）");
        return 0;
    }
    println!(
        "□ 運行：{} 次（成功 {}，成功率 {:.1}%）| 總輪次：{}",
        stats.runs,
        stats.ok_runs,
        100.0 * stats.ok_runs as f64 / stats.runs as f64,
        stats.attempts_total
    );
    println!("□ 漏斗（每層到達 → 通過）：");
    for (label, v) in stats.funnel_rows() {
        println!("    {}：{}", label, v);
    }
    println!(
        "□ 攔截分佈：結構 {} | 語法 {} | Tier-0 checker {} | 完整管線 UNSAT {} | 管線錯誤 {} | provider 錯誤 {}",
        stats.structure_rejects,
        stats.syntax_rejects,
        stats.checker_rejects,
        stats.unsat_rejects,
        stats.unresolved,
        stats.provider_error
    );
    if stats.sat > 0 {
        println!(
            "□ 平均收斂輪次（通過時）：{:.2}",
            stats.sat_attempt_sum as f64 / stats.sat as f64
        );
    }
    for (p, runs, ok) in &stats.providers {
        println!("    provider {}：{} 次運行，{} 次成功", p, runs, ok);
    }
    0
}

/// `polyrust brute [--size N] [--json]`：@brute 對照常態化（仿 `rlzl`）。
///
/// 暴力法 ⟺ 代數法逐位元比對，兩層：
/// - 子句層：300 組隨機 + 結構化子句集，CDCL ⟺ 暴力枚舉；學習子句蘊涵驗證。
/// - 約束層：≤N 節點表達式全空間 + 定置宏/函式樣本；
///   結構化暴力枚舉 ⟺ Gröbner 判定 ⟺ solve_boolean ⟺ 檢查器；
///   SAT 見證逐位元求值驗證 + one-hot 解碼健全性。
pub fn cmd_brute(args: &[String], json: bool) -> i32 {
    let mut size = 3usize;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--size" {
            if let Some(v) = args.get(i + 1) {
                size = v.parse().unwrap_or(3).clamp(1, 6);
            }
            i += 1;
        }
        i += 1;
    }

    let t0 = std::time::Instant::now();
    let clauses = crate::brute::cross_check_clauses(300, 0xC0FFEE);
    let rep = crate::brute::run_brute_cross(size);
    let elapsed = t0.elapsed().as_millis();
    let pass = clauses.mismatches.is_empty() && rep.mismatches.is_empty();

    if json {
        let details: Vec<crate::json::J> = clauses
            .mismatches
            .iter()
            .chain(rep.mismatches.iter())
            .map(|s| crate::json::J::s(s))
            .collect();
        println!(
            "{}",
            crate::json::J::obj(vec![
                ("api_version", crate::json::J::s("0.1")),
                ("mode", crate::json::J::s("brute")),
                ("clause_sets", crate::json::J::Int(clauses.sets as i64)),
                ("clause_sat_sets", crate::json::J::Int(clauses.sat_sets as i64)),
                ("learned_checked", crate::json::J::Int(clauses.learned_checked as i64)),
                ("programs", crate::json::J::Int(rep.programs as i64)),
                ("brute_searched", crate::json::J::Int(rep.brute_searched as i64)),
                ("witnesses_checked", crate::json::J::Int(rep.witnesses_checked as i64)),
                ("sat", crate::json::J::Int(rep.sat as i64)),
                ("unsat", crate::json::J::Int(rep.unsat as i64)),
                ("mismatches", crate::json::J::Arr(details)),
                ("pass", crate::json::J::Bool(pass)),
                ("elapsed_ms", crate::json::J::Int(elapsed as i64)),
            ])
        );
        return if pass { 0 } else { 1 };
    }

    println!("□ 子句層對照：{} 組（隨機 300 + 結構化；SAT {} 組）", clauses.sets, clauses.sat_sets);
    println!(
        "    CDCL ⟺ 暴力：{}；學習子句蘊涵驗證：{} 次{}",
        if clauses.mismatches.is_empty() { "全部一致 ✓" } else { "★ 有不一致 ★" },
        clauses.learned_checked,
        if clauses.mismatches.is_empty() { "" } else { "（含失敗）" }
    );
    println!(
        "□ 約束層對照：{} 個程序（表達式 ≤{} 節點 + 定置宏/函式樣本）",
        rep.programs, size
    );
    println!(
        "    結構化暴力搜索：{} 個 | 見證逐位元驗證：{} 次 | SAT {} / UNSAT {}",
        rep.brute_searched, rep.witnesses_checked, rep.sat, rep.unsat
    );
    println!(
        "    暴力 ⟺ Gröbner ⟺ solve_boolean ⟺ 檢查器：{}",
        if rep.mismatches.is_empty() { "四方全部一致 ✓" } else { "★ 有不一致 ★" }
    );
    for m in clauses.mismatches.iter().chain(rep.mismatches.iter()).take(5) {
        println!("──── 反例 ────\n{}", m);
    }
    println!("□ 耗時：{} ms", elapsed);
    if pass {
        println!("□ 結論：暴力法與代數法逐位元一致（@brute 對照常態化通過）");
        0
    } else {
        1
    }
}
