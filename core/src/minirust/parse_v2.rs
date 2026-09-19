//! Phase1 — 擴展詞法與類型解析（7+i）
//! 支援 Vec<T>, String, HashMap<K,V>, Option<T>, Result<T,E>, *mut T, *const T, &'a T 等

use super::lexer::{lex, Tok};
use super::universe::parse_type_v2;

/// 擴展關鍵字
pub const EXT_KW: &[&str] = &[
    "struct", "enum", "impl", "trait", "mod", "use", "pub", "unsafe", "async", "await",
    "loop", "while", "for", "match", "where", "dyn", "Self", "super", "crate", "self",
];

/// 擴展詞法：在原 lex 基礎上，識別更多關鍵字與符號 :: 'a
pub fn lex_v2(src: &str) -> Result<Vec<Tok>, String> {
    // 複用原 lex，但後處理把 EXT_KW 標為 Kw
    let mut toks = lex(src)?;
    for tok in &mut toks {
        if let Tok::Ident(s) = tok {
            if EXT_KW.contains(&s.as_str()) {
                // 將 Ident 轉為 Kw（用 Box 洩漏靜態化，Phase1 簡化）
                let kw_static: &'static str = match s.as_str() {
                    "struct" => "struct",
                    "enum" => "enum",
                    "impl" => "impl",
                    "trait" => "trait",
                    "mod" => "mod",
                    "use" => "use",
                    "pub" => "pub",
                    "unsafe" => "unsafe",
                    "async" => "async",
                    "await" => "await",
                    "loop" => "loop",
                    "while" => "while",
                    "for" => "for",
                    "match" => "match",
                    "where" => "where",
                    "dyn" => "dyn",
                    "Self" => "Self",
                    "super" => "super",
                    "crate" => "crate",
                    "self" => "self",
                    _ => continue,
                };
                *tok = Tok::Kw(kw_static);
            }
        }
    }
    Ok(toks)
}

/// 從 token 流中提取類型字符串（用於 7+i 解析）— 優化 with_capacity
pub fn extract_type_tokens(toks: &[Tok], start: usize) -> (String, usize) {
    let mut s = String::with_capacity(64);
    let mut depth = 0;
    let mut i = start;
    while i < toks.len() {
        match &toks[i] {
            Tok::Ident(id) => {
                s.push_str(id);
                s.push(' ');
            }
            Tok::Kw(k) => {
                s.push_str(k);
                s.push(' ');
            }
            Tok::Lt => {
                s.push('<');
                depth += 1;
            }
            Tok::Gt => {
                s.push('>');
                if depth > 0 {
                    depth -= 1;
                }
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            Tok::Comma => {
                if depth == 0 {
                    break;
                }
                s.push(',');
            }
            Tok::Amp => s.push('&'),
            Tok::Star => s.push('*'),
            Tok::Colon => s.push(':'),
            Tok::LParen => {
                s.push('(');
                depth += 1;
            }
            Tok::RParen => {
                if depth == 0 {
                    break;
                }
                s.push(')');
                depth -= 1;
            }
            Tok::LBrace | Tok::RBrace | Tok::Semi => break,
            _ => {
                if depth > 0 && matches!(toks[i], Tok::Ge) {
                    // >= 在泛型中可能是 > + = ? 簡化
                    s.push('>');
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                } else if depth == 0 {
                    break;
                }
            }
        }
        i += 1;
    }
    (s.trim().to_string(), i)
}

/// 測試：從源碼中提取所有類型並構造宇宙
pub fn build_universe_from_src(src: &str) -> Result<super::universe::Universe, String> {
    let mut universe = super::universe::Universe::new();
    // 粗略正則：尋找 : Type 或 -> Type 或 <Type> 模式
    // Phase1 簡化：用 parse_type_v2 嘗試解析常見類型
    let type_patterns = [
        "Vec<i32>", "Vec<String>", "String", "HashMap<String,i32>",
        "Option<i32>", "Result<i32,bool>", "*mut i32", "*const i32",
        "&i32", "&mut i32", "Future<i32>", "Point", "Option",
    ];
    for pat in type_patterns {
        if src.contains(pat) {
            if let Ok(ty) = parse_type_v2(pat) {
                universe.insert_closure(ty);
            }
        }
    }
    // 額外：嘗試從 : 後面提取
    for line in src.lines() {
        if let Some(colon) = line.find(':') {
            let after = line[colon+1..].trim();
            let end = after.find(|c| c==',' || c==';' || c=='{' || c==')').unwrap_or(after.len());
            let ty_str = after[..end].trim();
            if !ty_str.is_empty() {
                if let Ok(ty) = parse_type_v2(ty_str) {
                    universe.insert_closure(ty);
                }
            }
        }
    }
    Ok(universe)
}

/// 實際使用：parse_v2 文件清單與統計 — 優化 with_capacity
pub fn parse_v2_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("parse_v2.rs", "擴展詞法 EXT_KW + 類型提取 + Universe 構建 — 優化 with_capacity", "core/src/minirust/parse_v2.rs"),
        ("lexer.rs", "Tok 詞法 — v2 依賴", "core/src/minirust/lexer.rs"),
        ("universe.rs", "TypeV2 Universe — v2 依賴", "core/src/minirust/universe.rs"),
    ]
}
pub fn parse_v2_summary(src: &str) -> String {
    let mut out = String::with_capacity(512);
    match build_universe_from_src(src) {
        Ok(uni) => {
            out.push_str(&format!("parse_v2: src_len={} universe_N={} ext={}\n", src.len(), uni.n_types(), uni.n_ext()));
            out.push_str(&uni.display());
        }
        Err(e) => out.push_str(&format!("parse_v2 error: {}\n", e)),
    }
    out
}


