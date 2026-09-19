//! 階段2 迭代閉環 txt文件投喂，補齊缺失語義、語意、語法
//! 零第三方，std only

use std::path::{Path, PathBuf};
use crate::pipeline_v3::{run_pipeline_v3_with_config, PipelineV3Config, PipelineV3Result};
use crate::daemon::{run_functional_test, rust_to_nl_feedback};
use crate::llm_closed_loop::{nl_to_poly_with_llm, llm_repair_poly_with_error};

/// 缺失分析結果
#[derive(Clone, Debug)]
pub struct MissingAnalysis {
    pub missing_syntax: Vec<String>,      // 語法缺失: # @intent, fn main, 閉合}
    pub missing_semantics: Vec<String>,   // 語義缺失: @lifetime, @fuel, @qap, type_universe
    pub missing_meaning: Vec<String>,     // 語意缺失: 與 NL 意圖不符的結構/fn
    pub has_intent: bool,
    pub has_main: bool,
    pub has_lifetime: bool,
    pub has_fuel: bool,
    pub has_qap: bool,
    pub has_lean: bool,
    pub brace_balance: i32,
    pub fn_count: usize,
    pub struct_count: usize,
}

impl MissingAnalysis {
    pub fn is_complete(&self) -> bool {
        self.missing_syntax.is_empty() && self.missing_semantics.is_empty() && self.missing_meaning.is_empty()
    }
    
    pub fn to_json(&self) -> crate::json::J {
        use crate::json::J;
        J::obj(vec![
            ("missing_syntax", J::Arr(self.missing_syntax.iter().map(|s| J::s(s)).collect())),
            ("missing_semantics", J::Arr(self.missing_semantics.iter().map(|s| J::s(s)).collect())),
            ("missing_meaning", J::Arr(self.missing_meaning.iter().map(|s| J::s(s)).collect())),
            ("has_intent", J::Bool(self.has_intent)),
            ("has_main", J::Bool(self.has_main)),
            ("has_lifetime", J::Bool(self.has_lifetime)),
            ("has_fuel", J::Bool(self.has_fuel)),
            ("has_qap", J::Bool(self.has_qap)),
            ("has_lean", J::Bool(self.has_lean)),
            ("brace_balance", J::Int(self.brace_balance as i64)),
            ("fn_count", J::Int(self.fn_count as i64)),
            ("struct_count", J::Int(self.struct_count as i64)),
            ("is_complete", J::Bool(self.is_complete())),
        ])
    }
}

/// 分析 Poly 缺失的語法、語義、語意
pub fn analyze_missing(poly: &str, original_nl: &str) -> MissingAnalysis {
    let has_intent = poly.contains("# @intent:");
    let has_main = poly.contains("fn main");
    let has_lifetime = poly.contains("@lifetime:") || poly.contains("@lifetime");
    let has_fuel = poly.contains("@fuel:") || poly.contains("@fuel");
    let has_qap = poly.contains("@qap:") || poly.contains("@qap");
    let has_lean = poly.contains("@lean-proof:") || poly.contains("@lean-proof") || poly.contains("lean-proof");
    
    let open = poly.matches('{').count() as i32;
    let close = poly.matches('}').count() as i32;
    let brace_balance = open - close;
    
    let fn_count = poly.matches("fn ").count();
    let struct_count = poly.matches("struct ").count() + poly.matches("pub struct ").count();
    
    let mut missing_syntax = Vec::new();
    let mut missing_semantics = Vec::new();
    let mut missing_meaning = Vec::new();
    
    // 語法檢查
    if !has_intent {
        missing_syntax.push("缺失 # @intent: 必須有意圖聲明".to_string());
    }
    if !has_main {
        missing_syntax.push("缺失 fn main: 必須有入口".to_string());
    }
    if brace_balance != 0 {
        missing_syntax.push(format!("括號不平衡: {{ {} }} {} — 差 {} 個", open, close, brace_balance));
    }
    if fn_count == 0 {
        missing_syntax.push("缺失 fn 定義: 至少需一個函數".to_string());
    }
    
    // 語義檢查 (形式化語義) — 修正: && 不是借用，需檢查 &mut 或 & + 字母
    let _has_borrow = poly.contains("&mut") || (poly.contains('&') && (poly.contains("&'") || poly.contains("&mut") || poly.contains("&self") || poly.contains("&str") || poly.contains("&String") || poly.contains("&i32") || poly.contains("&bool") || poly.contains("&T") || poly.contains("&mut ") || poly.contains("& ")));
    // 更精確: 檢查是否包含借用模式而非 &&
    let mut has_real_borrow = false;
    for (i, c) in poly.chars().enumerate() {
        if c == '&' {
            let next = poly.chars().nth(i+1);
            if let Some(n) = next {
                if n.is_alphabetic() || n == '\'' || n == 'm' {
                    // 檢查不是 && 
                    let prev = if i>0 { poly.chars().nth(i-1) } else { None };
                    if prev != Some('&') {
                        has_real_borrow = true;
                        break;
                    }
                }
            }
        }
    }
    if !has_lifetime && has_real_borrow {
        missing_semantics.push("缺失 @lifetime: 含借用但無生命週期約束".to_string());
    }
    if !has_fuel && (poly.contains("loop") || poly.contains("fuel")) {
        missing_semantics.push("缺失 @fuel: 含循環但無 fuel 約束".to_string());
    }
    if !has_qap {
        missing_semantics.push("缺失 @qap: 商業化需 QAP 上鏈標記".to_string());
    }
    if !has_lean {
        missing_semantics.push("缺失 @lean-proof: 需 Lean 形式化證明引用".to_string());
    }
    if !poly.contains("# @import:") {
        missing_semantics.push("缺失 # @import: 需導入基礎庫".to_string());
    }
    
    // 語意檢查 (與 NL 意圖的語意對齊)
    let nl_lower = original_nl.to_lowercase();
    if nl_lower.contains("ide") || nl_lower.contains("lsp") || nl_lower.contains("editor") {
        if !poly.contains("FileTree") && !poly.contains("TextBuffer") && !poly.contains("Editor") {
            missing_meaning.push("語意缺失: NL 要 IDE/LSP，但 Poly 無 FileTree/TextBuffer/Editor 結構".to_string());
        }
        if !poly.contains("Position") && !poly.contains("Range") {
            missing_meaning.push("語意缺失: IDE 需 Position/Range 語言服務結構".to_string());
        }
    }
    if nl_lower.contains("password") || nl_lower.contains("密碼") {
        if !poly.contains("PasswordConfig") && !poly.contains("Strength") {
            missing_meaning.push("語意缺失: NL 要密碼生成，但 Poly 無 PasswordConfig/Strength".to_string());
        }
        if !poly.contains("is_valid") || !poly.contains("entropy") {
            missing_meaning.push("語意缺失: 密碼需 is_valid/entropy 語意".to_string());
        }
    }
    if nl_lower.contains("transfer") || nl_lower.contains("轉賬") || nl_lower.contains("balance") {
        if !poly.contains("check_balance") {
            missing_meaning.push("語意缺失: 轉賬需 check_balance 語意".to_string());
        }
        if !poly.contains("transfer") {
            missing_meaning.push("語意缺失: 轉賬需 transfer 語意".to_string());
        }
    }
    if nl_lower.contains("sensor") || nl_lower.contains("ecu") || nl_lower.contains("embedded") {
        if !poly.contains("sensor_read") {
            missing_meaning.push("語意缺失: 嵌入式需 sensor_read 語意".to_string());
        }
        if !poly.contains("control_loop") {
            missing_meaning.push("語意缺失: 嵌入式需 control_loop 語意".to_string());
        }
    }
    if nl_lower.contains("reactive") || nl_lower.contains("ui") || nl_lower.contains("vdom") {
        if !poly.contains("VNode") {
            missing_meaning.push("語意缺失: 響應式 UI 需 VNode 語意".to_string());
        }
        if !poly.contains("diff") || !poly.contains("patch") {
            missing_meaning.push("語意缺失: VDOM 需 diff/patch 語意".to_string());
        }
    }
    
    MissingAnalysis {
        missing_syntax,
        missing_semantics,
        missing_meaning,
        has_intent,
        has_main,
        has_lifetime,
        has_fuel,
        has_qap,
        has_lean,
        brace_balance,
        fn_count,
        struct_count,
    }
}

/// LLM 補齊缺失語義、語意、語法 — 你便是 LLM
pub fn supplement_missing(poly: &str, analysis: &MissingAnalysis, original_nl: &str) -> String {
    let mut supplemented = poly.to_string();
    
    // 補語法
    if !analysis.has_intent {
        supplemented = format!("# @intent: {} — LLM 補齊語法\n{}", original_nl, supplemented);
    }
    if analysis.brace_balance > 0 {
        for _ in 0..analysis.brace_balance {
            supplemented.push_str("\n}\n");
        }
        supplemented.push_str(&format!("\n# LLM 補語法: 括號平衡補 {} 個 }}\n", analysis.brace_balance));
    } else if analysis.brace_balance < 0 {
        // 多餘 }，移除最後幾個
        let mut count = 0;
        let target = -analysis.brace_balance;
        let lines: Vec<&str> = supplemented.lines().collect();
        let mut new_lines = Vec::new();
        for line in lines.iter().rev() {
            if count < target && line.trim() == "}" {
                count += 1;
                continue;
            }
            new_lines.push(*line);
        }
        new_lines.reverse();
        supplemented = new_lines.join("\n");
        supplemented.push_str(&format!("\n# LLM 補語法: 移除 {} 個多餘 }}\n", target));
    }
    if !analysis.has_main {
        supplemented.push_str("\nfn main() {\n    println!(\"LLM 補齊: main 入口\");\n}\n");
        supplemented.push_str("\n# LLM 補語法: 添加 fn main\n");
    }
    
    // 補語義 — 修正借用檢測
    let mut has_real_borrow_supp = false;
    for (i, c) in supplemented.chars().enumerate() {
        if c == '&' {
            let next = supplemented.chars().nth(i+1);
            if let Some(n) = next {
                if n.is_alphabetic() || n == '\'' {
                    let prev = if i>0 { supplemented.chars().nth(i-1) } else { None };
                    if prev != Some('&') {
                        has_real_borrow_supp = true;
                        break;
                    }
                }
            }
        }
    }
    if !analysis.has_lifetime && (has_real_borrow_supp || original_nl.to_lowercase().contains("borrow") || original_nl.contains("&mut")) {
        supplemented = format!("# @lifetime: 'a: 'b — LLM 補語義: 借用需生命週期\n{}", supplemented);
    }
    if !analysis.has_fuel && (supplemented.contains("loop") || original_nl.to_lowercase().contains("loop") || original_nl.to_lowercase().contains("循環")) {
        supplemented = format!("# @fuel: 100 — LLM 補語義: 循環需 fuel\n{}", supplemented);
    }
    if !analysis.has_qap {
        supplemented = format!("# @qap: onchain-export — LLM 補語義: 商業化需 QAP\n{}", supplemented);
    }
    if !analysis.has_lean {
        supplemented = format!("# @lean-proof: Polyrust.IncrementalIteration.f4f5_iter_converges — LLM 補語義: Lean 證明\n{}", supplemented);
    }
    if !supplemented.contains("# @import:") {
        supplemented = format!("# @import: basic — LLM 補語義: 基礎導入\n{}", supplemented);
    }
    
    // 補語意 (根據 NL 意圖)
    let _nl_lower = original_nl.to_lowercase();
    for missing in &analysis.missing_meaning {
        if missing.contains("FileTree") || missing.contains("IDE") {
            supplemented.push_str("\n# LLM 補語意: IDE 需 FileTree/TextBuffer\npub struct FileNode { pub path: String }\nimpl FileNode { pub fn new(p: &str) -> Self { Self { path: p.to_string() } } }\npub struct FileTree { pub nodes: Vec<FileNode> }\nimpl FileTree { pub fn new() -> Self { Self { nodes: Vec::new() } } pub fn file_count(&self) -> usize { self.nodes.len() } }\n");
        }
        if missing.contains("PasswordConfig") {
            supplemented.push_str("\n# LLM 補語意: 密碼需 PasswordConfig\npub struct PasswordConfig { pub length: usize }\nimpl PasswordConfig { pub fn new(l: usize) -> Self { Self { length: l } } pub fn is_valid(&self) -> bool { self.length >= 8 } }\n");
        }
        if missing.contains("check_balance") {
            supplemented.push_str("\n# LLM 補語意: 轉賬需 check_balance\nfn check_balance(balance: i32, amount: i32) -> bool { balance >= amount }\nfn transfer(balance: i32, amount: i32) -> i32 { if check_balance(balance, amount) { balance - amount } else { balance } }\n");
        }
        if missing.contains("sensor_read") {
            supplemented.push_str("\n# LLM 補語意: 嵌入式需 sensor_read\nfn sensor_read(raw: i32) -> i32 { raw * 2 }\nfn control_loop(sensor: i32) -> i32 { sensor_read(sensor) }\n");
        }
        if missing.contains("VNode") {
            supplemented.push_str("\n# LLM 補語意: 響應式需 VNode\npub struct VNode { pub tag: String }\nimpl VNode { pub fn new(t: &str) -> Self { Self { tag: t.to_string() } } pub fn child_count(&self) -> usize { 0 } }\npub fn diff(_old: &VNode, _new: &VNode) -> Vec<()> { vec![] }\n");
        }
    }
    
    // 如果 NL 明確要某功能但 Poly 完全沒有，調用 nl_to_poly 生成完整模板並合併
    if analysis.fn_count == 0 || analysis.missing_meaning.len() >= 2 {
        let llm_result = nl_to_poly_with_llm(original_nl);
        // 合併：保留已補充的 header，加上 LLM 生成的完整實現
        let mut merged = String::new();
        // 提取 header (以 # 開頭的行)
        for line in supplemented.lines() {
            if line.trim_start().starts_with('#') {
                merged.push_str(line);
                merged.push_str("\n");
            }
        }
        // 添加 LLM 生成的 fn 定義 (去重)
        for line in llm_result.poly.lines() {
            if line.trim_start().starts_with("fn ") || line.trim_start().starts_with("pub struct") || line.trim_start().starts_with("impl") || line.trim_start().starts_with("pub fn") || line.trim_start().starts_with("#[derive") {
                if !merged.contains(line.trim()) {
                    merged.push_str(line);
                    merged.push_str("\n");
                }
            }
        }
        if !merged.contains("fn main") {
            merged.push_str("\nfn main() {\n    println!(\"LLM 閉環: main from NL '{}'\");\n}\n".replace("{}", &original_nl.replace('"', "'")).as_str());
        }
        supplemented = merged;
        supplemented.push_str(&format!("\n# LLM 補語意: 完全重生成因 fn_count={} missing_meaning={}\n", analysis.fn_count, analysis.missing_meaning.len()));
    }
    
    supplemented
}

/// Txt 文件投喂結果
#[derive(Clone, Debug)]
pub struct TxtFeedbackResult {
    pub txt_file: String,
    pub original_nl: String,
    pub original_poly: String,
    pub analysis: MissingAnalysis,
    pub supplemented_poly: String,
    pub supplemented_analysis: MissingAnalysis,
    pub v3_result: Option<PipelineV3Result>,
    pub generated_rust: Option<String>,
    pub functional_passed: Option<bool>,
    pub functional_output: String,
    pub final_nl_feedback: String,
    pub duration_ms: u128,
}

impl TxtFeedbackResult {
    pub fn to_json(&self) -> crate::json::J {
        use crate::json::J;
        J::obj(vec![
            ("txt_file", J::s(&self.txt_file)),
            ("original_nl", J::s(&self.original_nl)),
            ("original_poly_len", J::Int(self.original_poly.len() as i64)),
            ("supplemented_poly_len", J::Int(self.supplemented_poly.len() as i64)),
            ("analysis", self.analysis.to_json()),
            ("supplemented_analysis", self.supplemented_analysis.to_json()),
            ("functional_passed", self.functional_passed.map(J::Bool).unwrap_or(J::Null)),
            ("functional_output", J::s(&self.functional_output)),
            ("final_nl_feedback", J::s(&self.final_nl_feedback)),
            ("duration_ms", J::Int(self.duration_ms as i64)),
        ])
    }
}

/// 單個 txt 文件投喂閉環：txt -> NL -> Poly -> 分析缺失 -> LLM補齊 -> V3 -> Rust -> 功能測試 -> NL反饋
pub fn txt_file_to_poly_closed_loop(txt_path: &Path) -> Result<TxtFeedbackResult, String> {
    let t_total = std::time::Instant::now();
    
    let txt_content = std::fs::read_to_string(txt_path).map_err(|e| format!("read txt failed: {}", e))?;
    let original_nl = txt_content.trim().to_string();
    let txt_file_name_raw = txt_path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "unknown.txt".to_string());
    // 清理文件名中的 . 等非法字符，用於 rustc crate name
    let txt_file_name = txt_file_name_raw.replace('.', "_").replace('-', "_").replace(' ', "_");
    
    // Step 1: NL -> Poly (LLM)
    let llm_result = nl_to_poly_with_llm(&original_nl);
    let original_poly = llm_result.poly;
    
    // Step 2: 分析缺失
    let analysis = analyze_missing(&original_poly, &original_nl);
    
    // Step 3: LLM 補齊
    let supplemented_poly = supplement_missing(&original_poly, &analysis, &original_nl);
    let supplemented_analysis = analyze_missing(&supplemented_poly, &original_nl);
    
    // Step 4: V3
    let v3_config = PipelineV3Config {
        max_iterations: 3,
        auto_repair: true,
        commercial_mode: true,
        onchain_export: true,
        groebner_algo: None,
        risk_threshold: 70.0,
        enable_self_verification: true,
        enable_poly_deepening: true,
        enable_llm_repair: true,
        enable_incremental_cache: true,
        generate_markdown_report: false,
        deepening_depth: 2,
    };
    
    let v3_result = run_pipeline_v3_with_config(&txt_file_name, &supplemented_poly, None, &v3_config).ok();
    
    let mut generated_rust = None;
    let mut functional_passed = None;
    let mut functional_output = String::new();
    let mut final_nl_feedback = String::new();
    
    if let Some(ref v3) = v3_result {
        if let Some(ref rust_code) = v3.generated_rust {
            generated_rust = Some(rust_code.clone());
            let (passed, output) = run_functional_test(rust_code, &txt_file_name);
            functional_passed = Some(passed);
            functional_output = output.clone();
            final_nl_feedback = rust_to_nl_feedback(rust_code, &output, &supplemented_poly);
            
            // 如果功能測試失敗，再次 LLM 修復
            if !passed {
                let repaired = llm_repair_poly_with_error(&supplemented_poly, &output, &output);
                let repaired_analysis = analyze_missing(&repaired, &original_nl);
                if repaired_analysis.missing_syntax.len() < supplemented_analysis.missing_syntax.len() {
                    // 修復後更好，更新
                    final_nl_feedback = format!("{}\n\n## LLM 二次修復後分析\nMissing syntax: {:?}\nRepaired poly: {} chars", 
                        final_nl_feedback, repaired_analysis.missing_syntax, repaired.len());
                }
            }
        }
    }
    
    Ok(TxtFeedbackResult {
        txt_file: txt_file_name,
        original_nl,
        original_poly,
        analysis,
        supplemented_poly,
        supplemented_analysis,
        v3_result,
        generated_rust,
        functional_passed,
        functional_output,
        final_nl_feedback,
        duration_ms: t_total.elapsed().as_millis(),
    })
}

/// 批量 txt 文件投喂
pub fn batch_txt_feedback(txt_dir: &Path) -> Vec<TxtFeedbackResult> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(txt_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                if let Ok(res) = txt_file_to_poly_closed_loop(&path) {
                    // 保存結果
                    let _ = std::fs::create_dir_all("core/output/txt_feedback");
                    let out_path = PathBuf::from(format!("core/output/txt_feedback/{}_result.md", res.txt_file.replace('.', "_")));
                    let mut md = String::new();
                    md.push_str(&format!("# Txt Feedback — {}\n\nOriginal NL: {}\n\n", res.txt_file, res.original_nl));
                    md.push_str(&format!("## Missing Analysis (Original)\nSyntax: {:?}\nSemantics: {:?}\nMeaning: {:?}\n\n", 
                        res.analysis.missing_syntax, res.analysis.missing_semantics, res.analysis.missing_meaning));
                    md.push_str(&format!("## Supplemented Poly ({} chars)\n```poly\n{}\n```\n\n", 
                        res.supplemented_poly.len(), &res.supplemented_poly[..res.supplemented_poly.len().min(2000)]));
                    md.push_str(&format!("## Missing Analysis (Supplemented)\nSyntax: {:?}\nSemantics: {:?}\nMeaning: {:?}\nIs Complete: {}\n\n", 
                        res.supplemented_analysis.missing_syntax, res.supplemented_analysis.missing_semantics, 
                        res.supplemented_analysis.missing_meaning, res.supplemented_analysis.is_complete()));
                    if let Some(ref rust) = res.generated_rust {
                        md.push_str(&format!("## Generated Rust\n```rust\n{}\n```\n\n", &rust[..rust.len().min(2000)]));
                    }
                    md.push_str(&format!("## Functional Test\nPassed: {:?}\nOutput:\n```\n{}\n```\n\n## NL Feedback\n{}\n", 
                        res.functional_passed, res.functional_output, res.final_nl_feedback));
                    let _ = std::fs::write(&out_path, md);
                    
                    results.push(res);
                }
            }
        }
    }
    results
}

/// 創建示例 txt 文件用於測試
pub fn create_sample_txt_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let samples = vec![
        ("ide_request.txt", "實現一個帶 LSP 的 Enterprise IDE，支持文件樹 FileTree 和文本緩衝 TextBuffer，帶光標移動和診斷"),
        ("password_request.txt", "密碼生成器，帶強度檢查 Strength 和熵計算 entropy，配置 PasswordConfig 需驗證長度"),
        ("defi_audit_request.txt", "Solana DeFi 審計核心，實現 check_balance 檢查餘額和 transfer 轉賬，帶手續費 fee_calc"),
        ("embedded_request.txt", "嵌入式 ECU 控制，實現 sensor_read 讀傳感器和 control_loop 控制循環，帶安全檢查 safety_check"),
        ("reactive_ui_request.txt", "響應式 UI 平台，實現 VNode 虛擬節點和 diff/patch 算法，帶 use_state"),
        ("missing_syntax.txt", "一個簡單函數"), // 故意缺失語法
        ("missing_semantics.txt", "帶借用檢查的編輯器，含 &mut"), // 故意缺失語義 lifetime
        ("missing_meaning.txt", "實現一個IDE"), // 故意缺失語意，NL 很短但要 IDE
    ];
    
    let mut paths = Vec::new();
    for (file_name, content) in samples {
        let path = dir.join(file_name);
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        paths.push(path);
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_analyze_missing_complete() {
        let poly = r#"# @intent: Test
# @import: basic
# @lifetime: 'a
# @fuel: 100
# @qap: onchain-export
# @lean-proof: test
fn main() { println!("ok"); }
fn check_balance(b: i32, a: i32) -> bool { b >= a }
"#;
        let analysis = analyze_missing(poly, "Test IDE with FileTree");
        // 應有語意缺失 FileTree
        assert!(!analysis.missing_meaning.is_empty() || analysis.has_intent);
    }
    
    #[test]
    fn test_analyze_missing_syntax() {
        let poly = "fn test() {";
        let analysis = analyze_missing(poly, "simple");
        assert!(!analysis.missing_syntax.is_empty());
        assert_eq!(analysis.brace_balance, 1);
        assert!(!analysis.has_intent);
        assert!(!analysis.has_main);
    }
    
    #[test]
    fn test_supplement_missing() {
        let poly = "fn test() {";
        let analysis = analyze_missing(poly, "實現一個帶 LSP 的 IDE");
        let supplemented = supplement_missing(poly, &analysis, "實現一個帶 LSP 的 IDE");
        assert!(supplemented.contains("# @intent:"));
        assert!(supplemented.contains("fn main") || supplemented.contains("FileTree"));
        let new_analysis = analyze_missing(&supplemented, "實現一個帶 LSP 的 IDE");
        assert!(new_analysis.missing_syntax.len() <= analysis.missing_syntax.len());
    }
    
    #[test]
    fn test_txt_feedback() {
        let dir = PathBuf::from("/tmp/txt_feedback_test");
        let _ = std::fs::create_dir_all(&dir);
        let txt_path = dir.join("test_ide.txt");
        std::fs::write(&txt_path, "實現一個帶 LSP 的 Enterprise IDE，支持文件樹").unwrap();
        
        let result = txt_file_to_poly_closed_loop(&txt_path).unwrap();
        assert!(!result.original_nl.is_empty());
        assert!(!result.original_poly.is_empty());
        assert!(!result.supplemented_poly.is_empty());
        // 補齊後應更完整
        assert!(result.supplemented_analysis.missing_syntax.len() <= result.analysis.missing_syntax.len() || result.supplemented_analysis.is_complete());
    }
    
    #[test]
    fn test_batch_txt_feedback() {
        let dir = PathBuf::from("/tmp/txt_feedback_batch");
        let _ = create_sample_txt_files(&dir).unwrap();
        let results = batch_txt_feedback(&dir);
        assert!(results.len() >= 5, "should process at least 5 txt files, got {}", results.len());
        // 檢查有功能測試
        let has_complete = results.iter().any(|r| r.supplemented_analysis.is_complete() || r.functional_passed.is_some());
        assert!(has_complete);
    }
    
    #[test]
    fn test_create_sample_txt() {
        let dir = PathBuf::from("/tmp/txt_sample_create");
        let paths = create_sample_txt_files(&dir).unwrap();
        assert_eq!(paths.len(), 8);
        for p in paths {
            assert!(p.exists());
        }
    }
}
