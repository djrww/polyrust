//! Diagnostic — Phase B P0 硬化：精確 span + code + help + note
//! 目標：錯誤信息從 Vec<String> 升級為結構化 Diagnostic，含 file:line:col，關聯 mod_map 路徑

use crate::json::J;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn new(file: impl Into<String>, line: usize, col: usize) -> Self {
        Self { file: file.into(), line, col, end_line: line, end_col: col + 1 }
    }
    pub fn new_range(file: impl Into<String>, line: usize, col: usize, end_line: usize, end_col: usize) -> Self {
        Self { file: file.into(), line, col, end_line, end_col }
    }
    pub fn dummy() -> Self {
        Self { file: "<unknown>".to_string(), line: 0, col: 0, end_line: 0, end_col: 0 }
    }
    pub fn from_line_content(file: &str, line_idx: usize, content: &str, needle: &str) -> Self {
        // 在 content 中查找 needle，返回 span
        let col = content.find(needle).unwrap_or(0);
        Self {
            file: file.to_string(),
            line: line_idx + 1,
            col,
            end_line: line_idx + 1,
            end_col: col + needle.len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticCode {
    BorrowConflict,
    LifetimeCycle,
    LifetimeOutlives,
    UnsafeGate,
    RawPtrSafety,
    StaticMutSafety,
    UnionSafety,
    UnsafeFnSafety,
    UnsafeTraitSafety,
    EffectError,
    StructTypeMismatch,
    VecTypeMismatch,
    LoopInvariant,
    AsyncAwait,
    MatchExhaustive,
    ModulePrivate,
    ParseError,
    GroebnerUnsat,
    QapTamper,
    Unknown,
}

impl DiagnosticCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BorrowConflict => "E0502",
            Self::LifetimeCycle => "E0267",
            Self::LifetimeOutlives => "E0308-lifetime",
            Self::UnsafeGate => "E0133",
            Self::RawPtrSafety => "E_SAFETY_RAW_PTR",
            Self::StaticMutSafety => "E_SAFETY_STATIC_MUT",
            Self::UnionSafety => "E_SAFETY_UNION",
            Self::UnsafeFnSafety => "E_SAFETY_UNSAFE_FN",
            Self::UnsafeTraitSafety => "E_SAFETY_UNSAFE_TRAIT",
            Self::EffectError => "E_EFFECT",
            Self::StructTypeMismatch => "E0308-struct",
            Self::VecTypeMismatch => "E0308-vec",
            Self::LoopInvariant => "E_LOOP_INVARIANT",
            Self::AsyncAwait => "E0728",
            Self::MatchExhaustive => "E0004",
            Self::ModulePrivate => "E0603",
            Self::ParseError => "E_PARSE",
            Self::GroebnerUnsat => "E_GROEBNER_UNSAT",
            Self::QapTamper => "E_QAP_TAMPER",
            Self::Unknown => "E0000",
        }
    }
    pub fn from_error_str(s: &str) -> Self {
        if s.contains("borrow conflict") { Self::BorrowConflict }
        else if s.contains("lifetime cycle") { Self::LifetimeCycle }
        else if s.contains("outlives") { Self::LifetimeOutlives }
        else if s.contains("unsafe error") && s.contains("raw") { Self::RawPtrSafety }
        else if s.contains("static_mut") { Self::StaticMutSafety }
        else if s.contains("union") { Self::UnionSafety }
        else if s.contains("unsafe fn") { Self::UnsafeFnSafety }
        else if s.contains("unsafe trait") { Self::UnsafeTraitSafety }
        else if s.contains("unsafe") { Self::UnsafeGate }
        else if s.contains("effect") { Self::EffectError }
        else if s.contains("struct") && s.contains("type mismatch") { Self::StructTypeMismatch }
        else if s.contains("Vec") && s.contains("type mismatch") { Self::VecTypeMismatch }
        else if s.contains("loop invariant") { Self::LoopInvariant }
        else if s.contains("async error") { Self::AsyncAwait }
        else if s.contains("match error") { Self::MatchExhaustive }
        else if s.contains("module error") { Self::ModulePrivate }
        else if s.contains("1 ∈ G") || s.contains("Groebner") { Self::GroebnerUnsat }
        else if s.contains("parse") { Self::ParseError }
        else { Self::Unknown }
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub span: Span,
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub help: Option<String>,
    pub note: Option<String>,
    pub related: Vec<Span>,
    pub lean_ref: Option<String>,
    pub mod_path: Option<String>,
}

impl Diagnostic {
    pub fn error(span: Span, code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            span,
            code,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            help: None,
            note: None,
            related: vec![],
            lean_ref: None,
            mod_path: None,
        }
    }
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
    pub fn with_related(mut self, related: Vec<Span>) -> Self {
        self.related = related;
        self
    }
    pub fn with_lean(mut self, lean_ref: impl Into<String>) -> Self {
        self.lean_ref = Some(lean_ref.into());
        self
    }
    pub fn with_mod_path(mut self, mod_path: impl Into<String>) -> Self {
        self.mod_path = Some(mod_path.into());
        self
    }

    pub fn to_json(&self) -> J {
        let mut fields = vec![
            ("file", J::s(&self.span.file)),
            ("line", J::Int(self.span.line as i64)),
            ("col", J::Int(self.span.col as i64)),
            ("end_line", J::Int(self.span.end_line as i64)),
            ("end_col", J::Int(self.span.end_col as i64)),
            ("code", J::s(self.code.as_str())),
            ("severity", J::s(self.severity.as_str())),
            ("message", J::s(&self.message)),
        ];
        if let Some(h) = &self.help {
            fields.push(("help", J::s(h)));
        }
        if let Some(n) = &self.note {
            fields.push(("note", J::s(n)));
        }
        if let Some(l) = &self.lean_ref {
            fields.push(("lean_ref", J::s(l)));
        }
        if let Some(m) = &self.mod_path {
            fields.push(("mod_path", J::s(m)));
        }
        if !self.related.is_empty() {
            let rel: Vec<J> = self.related.iter().map(|s| {
                J::obj(vec![
                    ("file", J::s(&s.file)),
                    ("line", J::Int(s.line as i64)),
                    ("col", J::Int(s.col as i64)),
                ])
            }).collect();
            fields.push(("related", J::Arr(rel)));
        }
        J::obj(fields)
    }

    pub fn format_human(&self) -> String {
        let mut out = format!(
            "{}:{}:{}: {}[{}]: {}",
            self.span.file,
            self.span.line,
            self.span.col,
            self.severity.as_str(),
            self.code.as_str(),
            self.message
        );
        if let Some(h) = &self.help {
            out.push_str(&format!("\n  help: {}", h));
        }
        if let Some(n) = &self.note {
            out.push_str(&format!("\n  note: {}", n));
        }
        if let Some(l) = &self.lean_ref {
            out.push_str(&format!("\n  lean: {}", l));
        }
        out
    }
}

/// 從舊的 Vec<String> errors 轉換為結構化 Diagnostic，帶 span 推斷
pub fn errors_to_diagnostics(file: &str, source: &str, errors: &[String]) -> Vec<Diagnostic> {
    let lines: Vec<&str> = source.lines().collect();
    let mut diags = Vec::with_capacity(errors.len());
    for err in errors {
        let code = DiagnosticCode::from_error_str(err);
        // 嘗試從錯誤中提取行號或關鍵字定位
        let (line_idx, col, needle) = infer_span_from_error(&lines, err);
        let span = if line_idx < lines.len() {
            Span::from_line_content(file, line_idx, lines[line_idx], &needle)
        } else {
            Span::new(file, line_idx + 1, col)
        };
        let mut diag = Diagnostic::error(span, code.clone(), err.clone());
        // 根據 code 生成 help
        diag.help = Some(generate_help_for_code(&code, err));
        diag.note = Some(generate_note_for_code(&code));
        diag.lean_ref = Some(lean_ref_for_code(&code));
        // mod_path 推斷
        if err.contains("geometry") || err.contains("Point") {
            diag.mod_path = Some("geometry::Point".to_string());
        }
        diags.push(diag);
    }
    diags
}

fn infer_span_from_error(lines: &[&str], err: &str) -> (usize, usize, String) {
    // 簡單啟發式：根據錯誤類型查找關鍵字
    let needle = if err.contains("borrow conflict") {
        "&mut".to_string()
    } else if err.contains("lifetime") {
        "'a".to_string()
    } else if err.contains("raw pointer") || err.contains("raw_ptr") {
        "*mut".to_string()
    } else if err.contains("static_mut") {
        "static mut".to_string()
    } else if err.contains("union") {
        "union".to_string()
    } else if err.contains("unsafe fn") {
        "unsafe fn".to_string()
    } else if err.contains("unsafe trait") {
        "unsafe trait".to_string()
    } else if err.contains("struct") {
        "struct".to_string()
    } else if err.contains("Vec") {
        "Vec".to_string()
    } else if err.contains("loop") {
        "loop".to_string()
    } else if err.contains("async") || err.contains("await") {
        "await".to_string()
    } else if err.contains("match") {
        "match".to_string()
    } else if err.contains("module") {
        "mod".to_string()
    } else {
        // 取錯誤首詞
        err.split_whitespace().next().unwrap_or("").to_string()
    };
    for (i, line) in lines.iter().enumerate() {
        if line.contains(&needle) {
            return (i, line.find(&needle).unwrap_or(0), needle);
        }
    }
    (0, 0, needle)
}

fn generate_help_for_code(code: &DiagnosticCode, err: &str) -> String {
    match code {
        DiagnosticCode::BorrowConflict => "避免同時存在 &mut 借用，縮短可變借用作用域或使用 clone".to_string(),
        DiagnosticCode::LifetimeCycle => "檢查 'a: 'b 約束是否存在環，使用 DFS 檢測環路徑重構".to_string(),
        DiagnosticCode::RawPtrSafety => "確保 valid = non_null∧aligned∧in_bounds∧not_dangling 且 (1-valid)*deref=0, (1-in_unsafe)*deref=0".to_string(),
        DiagnosticCode::StaticMutSafety => "需 safe = in_unsafe∧(exclusive∨mutex∨single) 且 (1-safe)*access=0，檢查是否在 thread::spawn/async 上下文".to_string(),
        DiagnosticCode::UnionSafety => "需 tag_match*(active-accessed)=0 且 safe = in_unsafe*tag_match".to_string(),
        DiagnosticCode::UnsafeFnSafety => "需 safe = in_unsafe*precond 且 (1-safe)*call=0，確保 precond 成立".to_string(),
        DiagnosticCode::UnsafeTraitSafety => "需 safe = is_unsafe_impl*invariant 且 (1-safe)*impl=0".to_string(),
        DiagnosticCode::EffectError => "檢查 @pure/@no-io 注解與實際 I/O 操作一致性".to_string(),
        DiagnosticCode::StructTypeMismatch => format!("檢查字段類型是否匹配 product 約束 t_struct - Πt_field: {}", err),
        DiagnosticCode::VecTypeMismatch => "檢查 Vec push 元素類型與聲明一致".to_string(),
        DiagnosticCode::LoopInvariant => "檢查 @invariant fuel >0 與循環遞增".to_string(),
        DiagnosticCode::AsyncAwait => "await 僅能在 Future 上調用，檢查 async 塊返回".to_string(),
        DiagnosticCode::MatchExhaustive => "添加缺失的模式分支或 _ 通配".to_string(),
        DiagnosticCode::ModulePrivate => "檢查 mod 內私有項訪問，使用 pub 或 crate::".to_string(),
        _ => "查看 lowering_report 與 Lean 證明引用".to_string(),
    }
}

fn generate_note_for_code(code: &DiagnosticCode) -> String {
    match code {
        DiagnosticCode::BorrowConflict => "Borrowck: t_borrow_conflict * overlap =0, Lean: Polyrust.Borrowck.borrow_conflict_unsat_mono".to_string(),
        DiagnosticCode::LifetimeCycle => "Lifetime: outlives 關係需無環，Lean: Polyrust.Lifetime.outlives_cycle_unsat".to_string(),
        DiagnosticCode::RawPtrSafety => "Iron Law: iron_raw_ptr_safe_no_ub, F4 ideal 不變".to_string(),
        DiagnosticCode::StaticMutSafety => "Iron Law: iron_static_mut_safe_no_data_race, 需 exclusive∨mutex".to_string(),
        DiagnosticCode::UnionSafety => "Iron Law: iron_union_safe_no_type_pun".to_string(),
        DiagnosticCode::UnsafeFnSafety => "Lean: Polyrust.UnsafeEmitProof.unsafe_fn_emit_implies_precond".to_string(),
        DiagnosticCode::UnsafeTraitSafety => "Lean: Polyrust.UnsafeEmitProof.unsafe_trait_emit_implies_invariant".to_string(),
        _ => "參見 docs/UNSAFE_BOUNDARY_V2.md 與 Lean 形式化".to_string(),
    }
}

fn lean_ref_for_code(code: &DiagnosticCode) -> String {
    match code {
        DiagnosticCode::BorrowConflict => "Polyrust.Borrowck.borrow_conflict_unsat_mono".to_string(),
        DiagnosticCode::LifetimeCycle => "Polyrust.Lifetime.outlives_cycle_unsat".to_string(),
        DiagnosticCode::RawPtrSafety => "Polyrust.IronLaw.iron_raw_ptr_safe_no_ub".to_string(),
        DiagnosticCode::StaticMutSafety => "Polyrust.IronLaw.iron_static_mut_safe_no_data_race".to_string(),
        DiagnosticCode::UnionSafety => "Polyrust.IronLaw.iron_union_safe_no_type_pun".to_string(),
        DiagnosticCode::UnsafeFnSafety => "Polyrust.IronLaw.iron_unsafe_fn_safe_no_ub".to_string(),
        DiagnosticCode::UnsafeTraitSafety => "Polyrust.IronLaw.iron_unsafe_trait_safe_no_ub".to_string(),
        DiagnosticCode::GroebnerUnsat => "Polyrust.F4.f4_ideal_invariant".to_string(),
        _ => "Polyrust.IncrementalIteration.f4f5_iter_converges".to_string(),
    }
}

pub fn diagnostic_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![("diagnostic.rs", "Diagnostic 精確 span + code + help — Phase B P0", "core/src/diagnostic.rs")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span() {
        let s = Span::new("foo.rs", 10, 5);
        assert_eq!(s.line, 10);
        assert_eq!(s.file, "foo.rs");
    }

    #[test]
    fn test_code_from_str() {
        assert_eq!(DiagnosticCode::from_error_str("borrow conflict: &mut at lines 1 and 2"), DiagnosticCode::BorrowConflict);
        assert_eq!(DiagnosticCode::from_error_str("unsafe error: raw pointer deref without unsafe block"), DiagnosticCode::RawPtrSafety);
        assert_eq!(DiagnosticCode::from_error_str("lifetime cycle detected"), DiagnosticCode::LifetimeCycle);
    }

    #[test]
    fn test_errors_to_diagnostics() {
        let src = "fn main() { let mut x = 1; let r1 = &mut x; let r2 = &mut x; }";
        let errors = vec!["borrow conflict: &mut at lines 1 and 1 overlap".to_string()];
        let diags = errors_to_diagnostics("test.rs", src, &errors);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::BorrowConflict);
        assert!(diags[0].help.is_some());
        let json = diags[0].to_json().to_string();
        assert!(json.contains("E0502"));
    }

    #[test]
    fn test_format_human() {
        let diag = Diagnostic::error(Span::new("a.rs", 1, 0), DiagnosticCode::BorrowConflict, "borrow conflict")
            .with_help("use clone")
            .with_note("see book");
        let s = diag.format_human();
        assert!(s.contains("a.rs:1:0"));
        assert!(s.contains("help"));
    }
}
