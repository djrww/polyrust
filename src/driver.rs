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

pub(crate) fn error_json(name: &str, poly: Option<&PolySource>, mode: &str, msg: &str) -> J {
    J::obj(vec![
        ("api_version", J::s("0.1")),
        ("mode", J::s(mode)),
        ("source", J::s(name)),
        ("intent", J::opt_str(poly.and_then(|p| p.intent.as_deref()))),
        ("status", J::s("error")),
        ("error", J::s(msg)),
    ])
}

/// `check` 核心：由（名稱, 文本, 來源目錄）→ JSON。`bool` 表示 status == ok。
pub(crate) fn check_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
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
pub(crate) fn expand_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
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
pub(crate) fn generate_text_json(name: &str, text: &str, base: Option<&Path>) -> (J, bool) {
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
    let positional: Vec<&String> = args
        .iter()
        .skip(2)
        .filter(|a| !a.starts_with("--") && a.as_str() != "-j")
        .collect();
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

    if json {
        println!("{}", llm::guardrail_to_json(&nl, &result));
        return if result.ok { 0 } else { 1 };
    }

    // 人類可讀輸出
    println!("□ 需求：{}", nl);
    println!("□ Provider：{}", result.provider);
    println!("□ 護欄（三道閘門：結構 → 語法 → 完整代數管線語義）：");
    for a in &result.attempts {
        let mark = match a.outcome.as_str() {
            "sat" => "✓ 通過",
            "unsat" => "✗ 語義拒絕（UNSAT）",
            _ => "✗ 結構/語法拒絕",
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
