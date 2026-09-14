//! Mini-Rust 詞法分析。

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Int(i64),
    Ident(String),
    Kw(&'static str), // fn let if else true false mut macro_rules
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
    Ge,     // >=
    EqEq,   // ==
    Ne,     // !=
    AndAnd, // &&
    Amp,    // &
    Not,    // !
    Assign, // =
    Dollar, // $
}

impl Tok {
    pub fn show(&self) -> String {
        match self {
            Tok::Int(n) => format!("{}", n),
            Tok::Ident(s) => s.clone(),
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
            Tok::Ge => ">=".into(),
            Tok::EqEq => "==".into(),
            Tok::Ne => "!=".into(),
            Tok::AndAnd => "&&".into(),
            Tok::Amp => "&".into(),
            Tok::Not => "!".into(),
            Tok::Assign => "=".into(),
            Tok::Dollar => "$".into(),
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
            // 區塊註解 /* ... */
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
                n = n * 10 + (b[i] as u64 - '0' as u64) as i64;
                i += 1;
            }
            out.push(Tok::Int(n));
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                s.push(b[i]);
                i += 1;
            }
            out.push(match s.as_str() {
                "fn" | "let" | "if" | "else" | "true" | "false" | "mut" | "macro_rules" => {
                    Tok::Kw(match s.as_str() {
                        "fn" => "fn",
                        "let" => "let",
                        "if" => "if",
                        "else" => "else",
                        "true" => "true",
                        "false" => "false",
                        "mut" => "mut",
                        _ => "macro_rules",
                    })
                }
                _ => Tok::Ident(s),
            });
            continue;
        }
        // 多字符符號
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
            '>' => return Err(format!("孤立的 '>'（提示：比較請用 >=，見上下文：…{}…）", {
                let ctx: String = b[i.saturating_sub(10)..(i + 12).min(b.len())].iter().collect();
                ctx
            })),
            '&' => Tok::Amp,
            '!' => Tok::Not,
            '=' => Tok::Assign,
            '$' => Tok::Dollar,
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
