//! Mini-Rust 詞法分析。Phase1 擴展：支持 `>` 用於泛型 `Vec<T>`
//! Phase3+：補 | || ? . .. ..= as return break continue move in box 字符串字面量

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Int(i64),
    Ident(String),
    Str(String), // "..." string literal
    Kw(&'static str), // fn let if else true false mut macro_rules struct enum impl trait mod pub unsafe async await loop while for match return break continue move as in box
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Semi,
    Colon,
    Arrow,   // ->
    FatArrow, // =>
    Plus,
    Minus,
    Star,
    Lt,
    Le,     // <=
    Gt,     // >  (Phase1: 泛型閉合)
    Ge,     // >=
    EqEq,   // ==
    Ne,     // !=
    AndAnd, // &&
    OrOr,   // ||
    Pipe,   // |
    Amp,    // &
    Not,    // !
    Assign, // =
    Dollar, // $
    Question, // ?
    Dot,    // .
    DotDot, // ..
    DotDotEq, // ..=
}

impl Tok {
    pub fn show(&self) -> String {
        match self {
            Tok::Int(n) => format!("{}", n),
            Tok::Ident(s) => s.clone(),
            Tok::Str(s) => format!("\"{}\"", s),
            Tok::Kw(k) => k.to_string(),
            Tok::LParen => "(".into(),
            Tok::RParen => ")".into(),
            Tok::LBrace => "{".into(),
            Tok::RBrace => "}".into(),
            Tok::Comma => ",".into(),
            Tok::Semi => ";".into(),
            Tok::Colon => ":".into(),
            Tok::Arrow => "->".into(),
            Tok::FatArrow => "=>".into(),
            Tok::Plus => "+".into(),
            Tok::Minus => "-".into(),
            Tok::Star => "*".into(),
            Tok::Lt => "<".into(),
            Tok::Le => "<=".into(),
            Tok::Gt => ">".into(),
            Tok::Ge => ">=".into(),
            Tok::EqEq => "==".into(),
            Tok::Ne => "!=".into(),
            Tok::AndAnd => "&&".into(),
            Tok::OrOr => "||".into(),
            Tok::Pipe => "|".into(),
            Tok::Amp => "&".into(),
            Tok::Not => "!".into(),
            Tok::Assign => "=".into(),
            Tok::Dollar => "$".into(),
            Tok::Question => "?".into(),
            Tok::Dot => ".".into(),
            Tok::DotDot => "..".into(),
            Tok::DotDotEq => "..=".into(),
        }
    }
}

pub fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let b: Vec<char> = src.chars().collect();
    let mut i = 0usize;
    let mut out = vec![];
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == '*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(b.len());
            continue;
        }
        if c.is_ascii_digit() {
            let mut n = 0i64;
            while i < b.len() && b[i].is_ascii_digit() {
                n = n
                    .checked_mul(10)
                    .and_then(|x| x.checked_add((b[i] as u64 - '0' as u64) as i64))
                    .ok_or_else(|| format!("整數字面量過大（位置 {}）", i))?;
                i += 1;
            }
            out.push(Tok::Int(n));
            continue;
        }
        // String literal "..."
        if c == '"' {
            i += 1;
            let mut s = String::new();
            while i < b.len() {
                let ch = b[i];
                if ch == '"' {
                    i += 1;
                    break;
                }
                if ch == '\\' && i + 1 < b.len() {
                    let nxt = b[i + 1];
                    match nxt {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        _ => { s.push('\\'); s.push(nxt); }
                    }
                    i += 2;
                } else {
                    s.push(ch);
                    i += 1;
                }
            }
            out.push(Tok::Str(s));
            continue;
        }
        // Char literal 'x' or lifetime 'a — treat 'x' as Ident, lifetime as Ident
        if c == '\'' {
            // Lookahead: if next is alphabetic and then non-alphanumeric or end, could be lifetime
            // If pattern like 'a' or 'b' with closing ', treat as Ident char literal simplified
            let mut s = String::new();
            s.push(c);
            i += 1;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_' ) {
                s.push(b[i]);
                i += 1;
            }
            // If we have closing ' for char literal like 'a' (s = "'a", next char = "'")
            if i < b.len() && b[i] == '\'' && s.len() > 1 {
                s.push('\'');
                i += 1;
                // treat as Ident for simplicity (e.g., "'a'" or "'a")
                out.push(Tok::Ident(s));
            } else {
                out.push(Tok::Ident(s));
            }
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                s.push(b[i]);
                i += 1;
            }
            out.push(match s.as_str() {
                "fn" | "let" | "if" | "else" | "true" | "false" | "mut" | "macro_rules"
                | "struct" | "enum" | "impl" | "trait" | "mod" | "use" | "pub" | "unsafe"
                | "async" | "await" | "loop" | "while" | "for" | "match" | "where" | "dyn"
                | "Self" | "super" | "crate" | "self"
                | "return" | "break" | "continue" | "move" | "as" | "in" | "box" => {
                    Tok::Kw(match s.as_str() {
                        "fn" => "fn",
                        "let" => "let",
                        "if" => "if",
                        "else" => "else",
                        "true" => "true",
                        "false" => "false",
                        "mut" => "mut",
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
                        "return" => "return",
                        "break" => "break",
                        "continue" => "continue",
                        "move" => "move",
                        "as" => "as",
                        "in" => "in",
                        "box" => "box",
                        _ => "macro_rules",
                    })
                }
                _ => Tok::Ident(s),
            });
            continue;
        }
        // Multi-char symbols
        if c == '-' && i + 1 < b.len() && b[i + 1] == '>' {
            out.push(Tok::Arrow);
            i += 2;
            continue;
        }
        if c == '=' && i + 1 < b.len() && b[i + 1] == '>' {
            out.push(Tok::FatArrow);
            i += 2;
            continue;
        }
        if c == '=' && i + 1 < b.len() && b[i + 1] == '=' {
            out.push(Tok::EqEq);
            i += 2;
            continue;
        }
        if c == '!' && i + 1 < b.len() && b[i + 1] == '=' {
            out.push(Tok::Ne);
            i += 2;
            continue;
        }
        if c == '<' && i + 1 < b.len() && b[i + 1] == '=' {
            out.push(Tok::Le);
            i += 2;
            continue;
        }
        if c == '>' && i + 1 < b.len() && b[i + 1] == '=' {
            out.push(Tok::Ge);
            i += 2;
            continue;
        }
        if c == '&' && i + 1 < b.len() && b[i + 1] == '&' {
            out.push(Tok::AndAnd);
            i += 2;
            continue;
        }
        if c == '|' && i + 1 < b.len() && b[i + 1] == '|' {
            out.push(Tok::OrOr);
            i += 2;
            continue;
        }
        if c == '.' && i + 1 < b.len() && b[i + 1] == '.' {
            if i + 2 < b.len() && b[i + 2] == '=' {
                out.push(Tok::DotDotEq);
                i += 3;
            } else {
                out.push(Tok::DotDot);
                i += 2;
            }
            continue;
        }
        if c == ':' && i + 1 < b.len() && b[i + 1] == ':' {
            out.push(Tok::Colon);
            out.push(Tok::Colon);
            i += 2;
            continue;
        }
        let t = match c {
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            ',' => Tok::Comma,
            ';' => Tok::Semi,
            ':' => Tok::Colon,
            '+' => Tok::Plus,
            '-' => Tok::Minus,
            '*' => Tok::Star,
            '<' => Tok::Lt,
            '>' => Tok::Gt,
            '&' => Tok::Amp,
            '!' => Tok::Not,
            '=' => Tok::Assign,
            '$' => Tok::Dollar,
            '|' => Tok::Pipe,
            '?' => Tok::Question,
            '.' => Tok::Dot,
            other => {
                let ctx: String = b[i.saturating_sub(10)..(i + 12).min(b.len())].iter().collect();
                return Err(format!("無法辨識的字元 '{}'（上下文：…{}…）", other, ctx))
            }
        };
        out.push(t);
        i += 1;
    }
    Ok(out)
}
