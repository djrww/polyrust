//! Phase3+ — Pat 解析器補齊：Or / Range
//! 零依賴，手寫遞歸下降，覆蓋 FullPat::Or 與 FullPat::Range，並兼容已有變體

use super::ast_full::{FullPat, FullType};
use super::lexer::Tok;

/// Pat 解析器（FullPat）
pub struct PatParser {
    toks: Vec<Tok>,
    pos: usize,
}

impl PatParser {
    pub fn new(toks: Vec<Tok>) -> Self {
        Self { toks, pos: 0 }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Tok) -> Result<(), String> {
        match self.bump() {
            Some(g) if &g == expected => Ok(()),
            Some(g) => Err(format!("Pat 期望 {:?}，得到 {:?}", expected, g)),
            None => Err(format!("Pat 期望 {:?}，輸入結束", expected)),
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }

    /// 入口：解析 Or（最低優先級）
    pub fn parse_pat(&mut self) -> Result<FullPat, String> {
        self.parse_or()
    }

    /// Or: a | b | c
    fn parse_or(&mut self) -> Result<FullPat, String> {
        let first = self.parse_range()?;
        let mut cases = vec![first];
        while let Some(Tok::Pipe) = self.peek() {
            self.bump(); // consume |
            // 允許 | 後緊跟另一模式
            if self.at_end() {
                break;
            }
            // 避免把 || 當 Or：若下一個也是 |，則是 OrOr，不在此層處理
            if let Some(Tok::Pipe) = self.peek() {
                break;
            }
            let next = self.parse_range()?;
            cases.push(next);
        }
        if cases.len() == 1 {
            Ok(cases.into_iter().next().unwrap())
        } else {
            Ok(FullPat::Or(cases))
        }
    }

    /// Range: PAT? .. PAT?  / PAT? ..= PAT?
    fn parse_range(&mut self) -> Result<FullPat, String> {
        // 支持前置 .. / ..= （如 ..10, ..=10）
        if let Some(Tok::DotDot) | Some(Tok::DotDotEq) = self.peek() {
            let inclusive = matches!(self.peek(), Some(Tok::DotDotEq));
            self.bump();
            // end 可選
            if self.at_end() || matches!(self.peek(), Some(Tok::Comma) | Some(Tok::RParen) | Some(Tok::RBrace) | Some(Tok::Pipe) | Some(Tok::FatArrow)) {
                return Ok(FullPat::Range {
                    start: None,
                    end: None,
                    inclusive,
                });
            }
            let end_pat = self.parse_atom()?;
            let end_str = Some(format!("{:?}", end_pat));
            return Ok(FullPat::Range {
                start: None,
                end: end_str,
                inclusive,
            });
        }

        let start = self.parse_atom()?;

        // 檢查後置 .. / ..=
        if let Some(Tok::DotDot) | Some(Tok::DotDotEq) = self.peek() {
            let inclusive = matches!(self.peek(), Some(Tok::DotDotEq));
            self.bump();
            // end 可選：如 0.. 或 0..= 
            if self.at_end() || matches!(self.peek(), Some(Tok::Comma) | Some(Tok::RParen) | Some(Tok::RBrace) | Some(Tok::Pipe) | Some(Tok::FatArrow)) {
                return Ok(FullPat::Range {
                    start: Some(format!("{:?}", start)),
                    end: None,
                    inclusive,
                });
            }
            let end_pat = self.parse_atom()?;
            return Ok(FullPat::Range {
                start: Some(format!("{:?}", start)),
                end: Some(format!("{:?}", end_pat)),
                inclusive,
            });
        }

        Ok(start)
    }

    /// 原子 Pat：Wild, Ident, Lit, Path, Tuple, Slice, Struct, Ref, Box, Macro, Type
    fn parse_atom(&mut self) -> Result<FullPat, String> {
        match self.peek().cloned() {
            Some(Tok::Ident(s)) if s == "_" => {
                self.bump();
                Ok(FullPat::Wild)
            }
            Some(Tok::Kw("mut")) => {
                self.bump();
                let inner = self.parse_atom()?;
                match inner {
                    FullPat::Ident { name, by_ref, subpat, .. } => Ok(FullPat::Ident {
                        name,
                        mutbl: true,
                        by_ref,
                        subpat,
                    }),
                    _ => Ok(inner),
                }
            }
            Some(Tok::Amp) => {
                self.bump();
                let mutbl = if let Some(Tok::Kw("mut")) = self.peek() {
                    self.bump();
                    true
                } else {
                    false
                };
                let inner = self.parse_atom()?;
                Ok(FullPat::Ref {
                    mutbl,
                    inner: Box::new(inner),
                })
            }
            Some(Tok::Ident(name)) => {
                self.bump();
                // 檢查是否為 TupleStruct: Name ( pat, ... )
                if let Some(Tok::LParen) = self.peek() {
                    self.bump();
                    let mut elems = vec![];
                    loop {
                        if let Some(Tok::RParen) = self.peek() {
                            self.bump();
                            break;
                        }
                        elems.push(self.parse_pat()?);
                        if let Some(Tok::Comma) = self.peek() {
                            self.bump();
                        } else if let Some(Tok::RParen) = self.peek() {
                            continue;
                        } else {
                            break;
                        }
                    }
                    Ok(FullPat::TupleStruct { path: name, elems })
                } else if let Some(Tok::LBrace) = self.peek() {
                    // Struct { fields, .. }
                    self.bump();
                    let mut fields = vec![];
                    let mut rest = false;
                    loop {
                        match self.peek() {
                            Some(Tok::RBrace) => { self.bump(); break; }
                            Some(Tok::DotDot) => { self.bump(); rest = true; 
                                if let Some(Tok::RBrace) = self.peek() { self.bump(); break; }
                            }
                            Some(Tok::Ident(fname)) => {
                                let fname = fname.clone();
                                self.bump();
                                // : pat or shorthand
                                let pat = if let Some(Tok::Colon) = self.peek() {
                                    self.bump();
                                    self.parse_pat()?
                                } else {
                                    FullPat::Ident { name: fname.clone(), mutbl: false, by_ref: false, subpat: None }
                                };
                                fields.push((fname, pat));
                                if let Some(Tok::Comma) = self.peek() { self.bump(); }
                            }
                            _ => { self.bump(); }
                        }
                    }
                    Ok(FullPat::Struct { path: name, fields, rest })
                } else {
                    // 檢查 @ subpat: x @ PAT
                    if let Some(Tok::Ident(at)) = self.peek() {
                        if at == "@" {
                            // Actually @ is not in lexer, but Ident "@"? We treat as special: if next token is Ident "@"
                            // For simplicity, check if next token is Ident "@" or Tok::Ident containing "@"
                            // Our lexer doesn't produce @, it would error. So skip.
                        }
                    }
                    // 檢查 : Type  (Type ascription in pat)
                    if let Some(Tok::Colon) = self.peek() {
                        self.bump();
                        // 粗略把後面直到 , | ) } 作為類型字符串
                        let mut ty_str = String::new();
                        while let Some(t) = self.peek() {
                            if matches!(t, Tok::Comma | Tok::RParen | Tok::RBrace | Tok::Pipe | Tok::FatArrow) {
                                break;
                            }
                            ty_str.push_str(&self.bump().unwrap().show());
                            ty_str.push(' ');
                        }
                        let ty = FullType::Macro(ty_str.trim().to_string());
                        Ok(FullPat::Type { pat: Box::new(FullPat::Ident { name, mutbl: false, by_ref: false, subpat: None }), ty })
                    } else {
                        // 單純 Path 或 Ident
                        if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            Ok(FullPat::Path(name))
                        } else {
                            Ok(FullPat::Ident { name, mutbl: false, by_ref: false, subpat: None })
                        }
                    }
                }
            }
            Some(Tok::Int(n)) => {
                self.bump();
                Ok(FullPat::Lit(format!("{}", n)))
            }
            Some(Tok::Str(s)) => {
                let lit = format!("\"{}\"", s);
                self.bump();
                Ok(FullPat::Lit(lit))
            }
            Some(Tok::Kw("true")) | Some(Tok::Kw("false")) => {
                let kw = self.bump().unwrap().show();
                Ok(FullPat::Lit(kw))
            }
            Some(Tok::LParen) => {
                self.bump();
                if let Some(Tok::RParen) = self.peek() {
                    self.bump();
                    Ok(FullPat::Tuple(vec![]))
                } else {
                    let mut elems = vec![];
                    loop {
                        if let Some(Tok::RParen) = self.peek() { self.bump(); break; }
                        elems.push(self.parse_pat()?);
                        if let Some(Tok::Comma) = self.peek() { self.bump(); } else { continue; }
                    }
                    if elems.len() == 1 {
                        Ok(elems.into_iter().next().unwrap())
                    } else {
                        Ok(FullPat::Tuple(elems))
                    }
                }
            }
            Some(Tok::LBrace) => {
                // Block as macro placeholder
                self.bump();
                let mut depth = 1;
                let mut inner = String::new();
                while let Some(t) = self.bump() {
                    match t {
                        Tok::LBrace => { depth += 1; inner.push('{'); }
                        Tok::RBrace => { depth -= 1; if depth == 0 { break; } inner.push('}'); }
                        other => { inner.push_str(&other.show()); inner.push(' '); }
                    }
                }
                Ok(FullPat::Macro(inner))
            }
            Some(Tok::LBrace) | Some(Tok::RBrace) => Err("unexpected brace in pat".into()),
            Some(other) => {
                // Fallback: consume and return as Macro placeholder
                let txt = self.bump().unwrap().show();
                Ok(FullPat::Macro(format!("{} ({:?})", txt, other)))
            }
            None => Err("Pat 意外結束".into()),
        }
    }
}

/// 便捷函數：從 token 切片解析 Pat
pub fn parse_pat_from_tokens(toks: &[Tok]) -> Result<FullPat, String> {
    let mut p = PatParser::new(toks.to_vec());
    let pat = p.parse_pat()?;
    Ok(pat)
}

/// 從源碼字符串解析 Pat（用於測試與補全）
pub fn parse_pat_str(src: &str) -> Result<FullPat, String> {
    let toks = super::lexer::lex(src)?;
    parse_pat_from_tokens(&toks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_or_pat() {
        let pat = parse_pat_str("Some(x) | None").unwrap();
        match pat {
            FullPat::Or(v) => assert_eq!(v.len(), 2),
            _ => panic!("expected Or, got {:?}", pat),
        }
    }

    #[test]
    fn test_range_pat() {
        let pat = parse_pat_str("0..10").unwrap();
        match pat {
            FullPat::Range { inclusive: false, .. } => {},
            _ => panic!("expected Range, got {:?}", pat),
        }
        let pat2 = parse_pat_str("0..=10").unwrap();
        match pat2 {
            FullPat::Range { inclusive: true, .. } => {},
            _ => panic!("expected inclusive Range"),
        }
        let pat3 = parse_pat_str("'a'..='z'").unwrap();
        match pat3 {
            FullPat::Range { inclusive: true, .. } => {},
            _ => panic!("expected char range"),
        }
    }

    #[test]
    fn test_wild_and_ident() {
        let pat = parse_pat_str("_").unwrap();
        assert!(matches!(pat, FullPat::Wild));
        let pat2 = parse_pat_str("mut x").unwrap();
        match pat2 {
            FullPat::Ident { mutbl: true, name, .. } => assert_eq!(name, "x"),
            _ => panic!("expected mut ident"),
        }
    }
}
