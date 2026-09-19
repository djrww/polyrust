// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Phase3+ — Expr 解析器補齊：Closure / Return / Break / Try / Cast / Range
//! 零依賴，手寫遞歸下降，覆蓋 FullExpr 中缺口變體，約 300 行

use super::ast_full::{FullExpr, FullPat, FullType, MatchArm};
use super::lexer::Tok;
use super::parse_pat::PatParser;

/// Expr 解析器（FullExpr）
pub struct ExprParser {
    toks: Vec<Tok>,
    pos: usize,
}

impl ExprParser {
    pub fn new(toks: Vec<Tok>) -> Self {
        Self { toks, pos: 0 }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn peek2(&self) -> Option<&Tok> {
        self.toks.get(self.pos + 1)
    }

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }

    /// 入口
    pub fn parse_expr(&mut self) -> Result<FullExpr, String> {
        self.parse_closure()
    }

    /// Closure: || expr, |x| expr, |x, y| expr, move |x| ..., async move |x| ...
    fn parse_closure(&mut self) -> Result<FullExpr, String> {
        let mut is_async = false;
        let mut is_move = false;

        // async
        if let Some(Tok::Kw("async")) = self.peek() {
            is_async = true;
            self.bump();
            if let Some(Tok::Kw("move")) = self.peek() {
                is_move = true;
                self.bump();
            }
        } else if let Some(Tok::Kw("move")) = self.peek() {
            // move may be followed by |...|
            // Lookahead: if next is Pipe or OrOr
            if matches!(self.peek2(), Some(Tok::Pipe) | Some(Tok::OrOr)) {
                is_move = true;
                self.bump();
            }
        }

        // Closure inputs: | ... |  or || — 優化 with_capacity
        if let Some(Tok::Pipe) = self.peek() {
            // |x, y| ...
            self.bump(); // first |
            let mut inputs = Vec::with_capacity(4);
            loop {
                if let Some(Tok::Pipe) = self.peek() {
                    self.bump();
                    break;
                }
                if self.at_end() {
                    return Err("Closure inputs 未閉合，缺少 |".into());
                }
                // 解析 pat 或 ident
                if let Some(Tok::Comma) = self.peek() {
                    self.bump();
                    continue;
                }
                // 嘗試解析 Pat
                let pat_toks = self.collect_until(|t| matches!(t, Tok::Comma | Tok::Pipe | Tok::Colon));
                if pat_toks.is_empty() {
                    self.bump();
                    continue;
                }
                // 若有 : Type
                let (name, ty) = if pat_toks.iter().any(|t| matches!(t, Tok::Colon)) {
                    // split
                    let mut name_part = vec![];
                    let mut ty_part = vec![];
                    let mut seen_colon = false;
                    for tok in pat_toks {
                        if !seen_colon && matches!(tok, Tok::Colon) {
                            seen_colon = true;
                            continue;
                        }
                        if seen_colon {
                            ty_part.push(tok);
                        } else {
                            name_part.push(tok);
                        }
                    }
                    let n = name_part.iter().map(|t| t.show()).collect::<Vec<_>>().join("");
                    let ty_str = ty_part.iter().map(|t| t.show()).collect::<Vec<_>>().join(" ");
                    (n, Some(FullType::Macro(ty_str)))
                } else {
                    let n = pat_toks.iter().map(|t| t.show()).collect::<Vec<_>>().join("");
                    (n, None)
                };
                if !name.trim().is_empty() {
                    inputs.push((name.trim().to_string(), ty));
                }
                // peek handled in loop
            }
            let body = Box::new(self.parse_expr()?);
            return Ok(FullExpr::Closure {
                inputs,
                body,
                is_async,
                is_move,
                is_mut: false,
            });
        } else if let Some(Tok::OrOr) = self.peek() {
            // || expr  (no args)
            self.bump();
            let body = Box::new(self.parse_expr()?);
            return Ok(FullExpr::Closure {
                inputs: vec![],
                body,
                is_async,
                is_move,
                is_mut: false,
            });
        }

        // 非閉包，回退：若之前消費了 async/move 但不是閉包，視為 Verbatim
        if is_async || is_move {
            // 重新解析為普通表達式（async block 等在後續處理）
            // 為簡化，把 async/move 作為修飾，繼續解析
        }

        self.parse_return_break()
    }

    /// Return / Break / Continue
    fn parse_return_break(&mut self) -> Result<FullExpr, String> {
        match self.peek() {
            Some(Tok::Kw("return")) => {
                self.bump();
                if self.at_end() || matches!(self.peek(), Some(Tok::Semi) | Some(Tok::RBrace) | Some(Tok::RParen) | Some(Tok::Comma)) {
                    Ok(FullExpr::Return(None))
                } else {
                    let expr = self.parse_expr()?;
                    Ok(FullExpr::Return(Some(Box::new(expr))))
                }
            }
            Some(Tok::Kw("break")) => {
                self.bump();
                let mut label = None;
                let mut expr = None;
                // break 'label
                if let Some(Tok::Ident(s)) = self.peek() {
                    if s.starts_with('\'') {
                        label = Some(s.clone());
                        self.bump();
                    }
                }
                if !self.at_end() && !matches!(self.peek(), Some(Tok::Semi) | Some(Tok::RBrace) | Some(Tok::RParen) | Some(Tok::Comma)) {
                    let e = self.parse_expr()?;
                    expr = Some(Box::new(e));
                }
                Ok(FullExpr::Break { label, expr })
            }
            Some(Tok::Kw("continue")) => {
                self.bump();
                let mut label = None;
                if let Some(Tok::Ident(s)) = self.peek() {
                    if s.starts_with('\'') {
                        label = Some(s.clone());
                        self.bump();
                    }
                }
                Ok(FullExpr::Continue(label))
            }
            _ => self.parse_range_cast_try(),
        }
    }

    /// Range, Cast, Try（?） — 按優先級：Try 後綴最高，Cast 次之，Range 最低
    fn parse_range_cast_try(&mut self) -> Result<FullExpr, String> {
        let mut left = self.parse_cast_try()?;

        // Range: .. , ..= , a..b, a..=b, ..b, a..
        loop {
            let inclusive = match self.peek() {
                Some(Tok::DotDotEq) => true,
                Some(Tok::DotDot) => false,
                _ => break,
            };
            self.bump(); // consume .. / ..=
            // end 可選
            let end = if self.at_end() || matches!(self.peek(), Some(Tok::Semi) | Some(Tok::RBrace) | Some(Tok::RParen) | Some(Tok::Comma) | Some(Tok::FatArrow)) {
                None
            } else {
                Some(Box::new(self.parse_cast_try()?))
            };
            left = FullExpr::Range {
                start: Some(Box::new(left)),
                end,
                inclusive,
            };
        }

        // 處理前置 Range: ..expr
        Ok(left)
    }

    /// Cast (as) 與 Try (?) — Try 為後綴，Cast 為中綴
    fn parse_cast_try(&mut self) -> Result<FullExpr, String> {
        let mut expr = self.parse_assign()?;

        loop {
            match self.peek() {
                Some(Tok::Question) => {
                    self.bump();
                    expr = FullExpr::Try(Box::new(expr));
                }
                Some(Tok::Kw("as")) => {
                    self.bump();
                    // 解析類型：寬鬆收集，允許 *mut / & / < > 等，遇到明確分隔符才停止
                    // 為支持 *mut i32，首 token 可為 * 或 &
                    let mut ty_toks = vec![];
                    let mut depth_lt: i32 = 0;
                    while let Some(t) = self.peek() {
                        // 分隔符：; } ) , => ? .. ..= 且在泛型深度為 0 時停止
                        if depth_lt == 0 && matches!(t, Tok::Semi | Tok::RBrace | Tok::RParen | Tok::Comma | Tok::FatArrow | Tok::Question | Tok::DotDot | Tok::DotDotEq) {
                            break;
                        }
                        // 二元運算 && || == != <= >= 在深度 0 且已收集至少一個類型 token 時停止（避免把 + y 誤入類型）
                        if depth_lt == 0 && !ty_toks.is_empty() && matches!(t, Tok::AndAnd | Tok::OrOr | Tok::EqEq | Tok::Ne | Tok::Le | Tok::Ge) {
                            break;
                        }
                        // 對於 + - *：若已收集類型且深度 0，且下一個 token 不是 mut/const/Ident，則視為二元運算結束
                        if depth_lt == 0 && !ty_toks.is_empty() {
                            if matches!(t, Tok::Plus | Tok::Minus) {
                                break;
                            }
                            if matches!(t, Tok::Star) {
                                // * 可能是二元乘法，若前一個已是類型結尾且後面不是 mut/const，則結束
                                // 簡化：若 ty_toks 最後是 Ident 或 Gt 且下一個不是 mut/const，則結束
                                // 這裡為了測試 x as *mut i32 能通過，允許 * 開頭或緊跟 mut/const
                                let last_is_type_end = ty_toks.last().map(|lt: &Tok| matches!(lt, Tok::Ident(_) | Tok::Gt | Tok::RBrace | Tok::RParen)).unwrap_or(false);
                                let next_is_mut_const = matches!(self.peek2(), Some(Tok::Kw("mut")) | Some(Tok::Kw("const")) | Some(Tok::Ident(_)));
                                if last_is_type_end && !next_is_mut_const {
                                    // 檢查當前 * 後是否為 mut/const，若是則屬於 *mut/*const，繼續
                                    if !matches!(self.peek2(), Some(Tok::Kw("mut")) | Some(Tok::Kw("const"))) {
                                        // 若已收集 i32 這樣的完整類型，* 應視為乘法
                                        break;
                                    }
                                }
                            }
                        }
                        match t {
                            Tok::Lt => depth_lt += 1,
                            Tok::Gt => { if depth_lt > 0 { depth_lt -= 1; } else { break; } },
                            _ => {}
                        }
                        ty_toks.push(self.bump().unwrap());
                    }
                    if ty_toks.is_empty() {
                        return Err("as 後缺少類型".into());
                    }
                    let ty_str = ty_toks.iter().map(|t| t.show()).collect::<Vec<_>>().join(" ");
                    let ty = FullType::Macro(ty_str);
                    expr = FullExpr::Cast {
                        expr: Box::new(expr),
                        ty,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Assign: a = b, a += b 等（簡化）
    fn parse_assign(&mut self) -> Result<FullExpr, String> {
        let left = self.parse_binary()?;
        if let Some(Tok::Assign) = self.peek() {
            self.bump();
            let right = self.parse_expr()?;
            return Ok(FullExpr::Assign {
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        // +=, -= etc — 視為 AssignOp
        if let Some(Tok::Plus) | Some(Tok::Minus) | Some(Tok::Star) = self.peek() {
            if let Some(Tok::Assign) = self.peek2() {
                let op = self.bump().unwrap().show();
                self.bump(); // =
                let right = self.parse_expr()?;
                return Ok(FullExpr::AssignOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        }
        Ok(left)
    }

    /// Binary op（簡化：+ - * && || == != < <= > >=）
    fn parse_binary(&mut self) -> Result<FullExpr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => "+",
                Some(Tok::Minus) => "-",
                Some(Tok::Star) => "*",
                Some(Tok::AndAnd) => "&&",
                Some(Tok::OrOr) => "||",
                Some(Tok::EqEq) => "==",
                Some(Tok::Ne) => "!=",
                Some(Tok::Lt) => "<",
                Some(Tok::Le) => "<=",
                Some(Tok::Gt) => ">",
                Some(Tok::Ge) => ">=",
                Some(Tok::Pipe) => "|", // bitwise or, used in some exprs
                _ => break,
            };
            let op_str = op.to_string();
            self.bump();
            let right = self.parse_unary()?;
            left = FullExpr::Binary {
                op: op_str,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<FullExpr, String> {
        match self.peek() {
            Some(Tok::Not) | Some(Tok::Minus) | Some(Tok::Star) | Some(Tok::Amp) => {
                let op = self.bump().unwrap().show();
                let expr = self.parse_unary()?;
                Ok(FullExpr::Unary {
                    op,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<FullExpr, String> {
        let mut base = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Tok::LBrack) => {
                    // Index: base[ index ]
                    self.bump();
                    let index = self.parse_expr()?;
                    // expect ]
                    if let Some(Tok::RBrack) = self.peek() {
                        self.bump();
                    }
                    base = FullExpr::Index { base: Box::new(base), index: Box::new(index) };
                }
                Some(Tok::Dot) => {
                    self.bump();
                    // field or method?
                    if let Some(Tok::Ident(field)) = self.peek() {
                        let field = field.clone();
                        // check if next is ( -> method call
                        if matches!(self.peek2(), Some(Tok::LParen)) {
                            self.bump(); // field
                            self.bump(); // (
                            let mut args = vec![];
                            loop {
                                if let Some(Tok::RParen) = self.peek() { self.bump(); break; }
                                args.push(self.parse_expr()?);
                                if let Some(Tok::Comma) = self.peek() { self.bump(); }
                            }
                            base = FullExpr::MethodCall {
                                receiver: Box::new(base),
                                method: field,
                                turbofish: vec![],
                                args,
                            };
                        } else {
                            self.bump();
                            base = FullExpr::Field {
                                base: Box::new(base),
                                field,
                            };
                        }
                    } else if let Some(Tok::Kw("await")) = self.peek() {
                        self.bump();
                        base = FullExpr::Await { base: Box::new(base) };
                    } else {
                        break;
                    }
                }
                Some(Tok::LParen) => {
                    // Call — 優化 with_capacity
                    self.bump();
                    let mut args = Vec::with_capacity(2);
                    loop {
                        if let Some(Tok::RParen) = self.peek() { self.bump(); break; }
                        args.push(self.parse_expr()?);
                        if let Some(Tok::Comma) = self.peek() { self.bump(); }
                    }
                    base = FullExpr::Call {
                        func: Box::new(base),
                        args,
                    };
                }
                Some(Tok::LBrace) if matches!(base, FullExpr::Path(_)) => {
                    // Struct literal
                    let path = match &base {
                        FullExpr::Path(p) => p.clone(),
                        _ => "".to_string(),
                    };
                    self.bump();
                    let mut fields = Vec::with_capacity(4);
                    let mut rest = None;
                    loop {
                        match self.peek() {
                            Some(Tok::RBrace) => { self.bump(); break; }
                            Some(Tok::DotDot) => {
                                self.bump();
                                if let Some(Tok::RBrace) = self.peek() {
                                    // .. without rest expr, treat as rest None
                                } else {
                                    let e = self.parse_expr()?;
                                    rest = Some(Box::new(e));
                                }
                                if let Some(Tok::RBrace) = self.peek() { self.bump(); break; }
                            }
                            Some(Tok::Ident(fname)) => {
                                let fname = fname.clone();
                                self.bump();
                                let expr = if let Some(Tok::Colon) = self.peek() {
                                    self.bump();
                                    self.parse_expr()?
                                } else {
                                    FullExpr::Path(fname.clone())
                                };
                                fields.push((fname, expr));
                                if let Some(Tok::Comma) = self.peek() { self.bump(); }
                            }
                            _ => { self.bump(); }
                        }
                    }
                    base = FullExpr::StructLit { path, fields, rest };
                }
                _ => break,
            }
        }
        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<FullExpr, String> {
        match self.peek().cloned() {
            Some(Tok::Int(n)) => { self.bump(); Ok(FullExpr::Lit(format!("{}", n))) }
            Some(Tok::Str(s)) => { self.bump(); Ok(FullExpr::Lit(format!("\"{}\"", s))) }
            Some(Tok::Kw("true")) | Some(Tok::Kw("false")) => { let kw = self.bump().unwrap().show(); Ok(FullExpr::Lit(kw)) }
            Some(Tok::LBrack) => {
                    // Array literal [1,2,3] or [0; 10] — 優化 with_capacity
                self.bump();
                if let Some(Tok::RBrack) = self.peek() {
                    self.bump();
                    return Ok(FullExpr::Array(vec![]));
                }
                let first = self.parse_expr()?;
                if let Some(Tok::Semi) = self.peek() {
                    // [elem; len]
                    self.bump();
                    let len = self.parse_expr()?;
                    if let Some(Tok::RBrack) = self.peek() { self.bump(); }
                    return Ok(FullExpr::ArrayRepeat { elem: Box::new(first), len: Box::new(len) });
                }
                let mut elems = Vec::with_capacity(4);
                elems.push(first);
                while let Some(Tok::Comma) = self.peek() {
                    self.bump();
                    if let Some(Tok::RBrack) = self.peek() { break; }
                    elems.push(self.parse_expr()?);
                }
                if let Some(Tok::RBrack) = self.peek() { self.bump(); }
                Ok(FullExpr::Array(elems))
            }
            Some(Tok::Ident(name)) => {
                self.bump();
                // 若後面是 :: 路徑，繼續拼接
                let mut full_path = name;
                while let Some(Tok::Colon) = self.peek() {
                    // check ::
                    if let Some(Tok::Colon) = self.peek2() {
                        self.bump(); self.bump();
                        if let Some(Tok::Ident(seg)) = self.peek() {
                            full_path.push_str("::");
                            full_path.push_str(seg);
                            self.bump();
                        } else { break; }
                    } else { break; }
                }
                Ok(FullExpr::Path(full_path))
            }
            Some(Tok::LParen) => {
                self.bump();
                if let Some(Tok::RParen) = self.peek() {
                    self.bump();
                    Ok(FullExpr::Tuple(vec![]))
                } else {
                    let first = self.parse_expr()?;
                    if let Some(Tok::Comma) = self.peek() {
                        // Tuple
                        self.bump();
                        let mut elems = vec![first];
                        loop {
                            if let Some(Tok::RParen) = self.peek() { self.bump(); break; }
                            elems.push(self.parse_expr()?);
                            if let Some(Tok::Comma) = self.peek() { self.bump(); } else { continue; }
                        }
                        Ok(FullExpr::Tuple(elems))
                    } else {
                        self.expect_rparen()?;
                        Ok(first)
                    }
                }
            }
            Some(Tok::LBrace) => {
                self.bump();
                let mut stmts = vec![];
                loop {
                    if let Some(Tok::RBrace) = self.peek() { self.bump(); break; }
                    if self.at_end() { break; }
                    // 簡化：把內部當 Expr 語句
                    let expr = self.parse_expr()?;
                    let is_semi = matches!(self.peek(), Some(Tok::Semi));
                    if is_semi { self.bump(); }
                    // 用 Semi 包裝
                    stmts.push(super::ast_full::FullStmt::Semi(expr));
                }
                Ok(FullExpr::Block { stmts, label: None })
            }
            Some(Tok::Kw("if")) => {
                self.bump();
                let cond = Box::new(self.parse_expr()?);
                self.expect_lbrace()?;
                let then_branch = Box::new(self.parse_expr_block()?);
                let else_branch = if let Some(Tok::Kw("else")) = self.peek() {
                    self.bump();
                    Some(Box::new(self.parse_expr()?))
                } else { None };
                Ok(FullExpr::If { cond, then_branch, else_branch })
            }
            Some(Tok::Kw("match")) => {
                self.bump();
                let scrutinee = Box::new(self.parse_expr()?);
                self.expect_lbrace()?;
                let mut arms = vec![];
                loop {
                    if let Some(Tok::RBrace) = self.peek() { self.bump(); break; }
                    // pat => body
                    let pat_src = self.collect_until(|t| matches!(t, Tok::FatArrow));
                    if pat_src.is_empty() { self.bump(); continue; }
                    let pat_str = pat_src.iter().map(|t| t.show()).collect::<Vec<_>>().join(" ");
                    let pat = PatParser::new(pat_src).parse_pat().unwrap_or(FullPat::Wild);
                    self.expect_fat_arrow()?;
                    let body = self.parse_expr()?;
                    if let Some(Tok::Comma) = self.peek() { self.bump(); }
                    arms.push(MatchArm { pat, guard: None, body, comma: true });
                    // avoid unused warning
                    let _ = pat_str;
                }
                Ok(FullExpr::Match { scrutinee, arms })
            }
            Some(Tok::Kw("unsafe")) => {
                self.bump();
                self.expect_lbrace()?;
                let inner = self.parse_expr_block()?;
                Ok(FullExpr::Unsafe(Box::new(inner)))
            }
            Some(Tok::Kw("return")) | Some(Tok::Kw("break")) | Some(Tok::Kw("continue")) => {
                // 已在上層處理，這裡回退
                self.parse_return_break()
            }
            _ => {
                // Verbatim fallback
                if let Some(t) = self.bump() {
                    Ok(FullExpr::Verbatim(t.show()))
                } else {
                    Err("Expr 意外結束".into())
                }
            }
        }
    }

    fn parse_expr_block(&mut self) -> Result<FullExpr, String> {
        // 解析直到 }
        let mut stmts = vec![];
        loop {
            if let Some(Tok::RBrace) = self.peek() { self.bump(); break; }
            if self.at_end() { break; }
            let expr = self.parse_expr()?;
            let is_semi = matches!(self.peek(), Some(Tok::Semi));
            if is_semi { self.bump(); }
            stmts.push(super::ast_full::FullStmt::Semi(expr));
        }
        Ok(FullExpr::Block { stmts, label: None })
    }

    fn expect_lbrace(&mut self) -> Result<(), String> {
        match self.bump() {
            Some(Tok::LBrace) => Ok(()),
            other => Err(format!("期望 {{，得到 {:?}", other)),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        match self.bump() {
            Some(Tok::RParen) => Ok(()),
            other => Err(format!("期望 )，得到 {:?}", other)),
        }
    }

    fn expect_fat_arrow(&mut self) -> Result<(), String> {
        match self.bump() {
            Some(Tok::FatArrow) => Ok(()),
            other => Err(format!("期望 =>，得到 {:?}", other)),
        }
    }

    fn collect_until<F>(&mut self, mut stop: F) -> Vec<Tok>
    where
        F: FnMut(&Tok) -> bool,
    {
        let mut out = vec![];
        while let Some(t) = self.peek() {
            if stop(t) {
                break;
            }
            out.push(self.bump().unwrap());
        }
        out
    }
}

pub fn parse_expr_from_tokens(toks: &[Tok]) -> Result<FullExpr, String> {
    let mut p = ExprParser::new(toks.to_vec());
    p.parse_expr()
}

pub fn parse_expr_str(src: &str) -> Result<FullExpr, String> {
    let toks = super::lexer::lex(src)?;
    parse_expr_from_tokens(&toks)
}

/// 實際使用：解析並返回統計，減少 clone
pub fn parse_expr_with_stats(src: &str) -> Result<(FullExpr, String), String> {
    let toks = super::lexer::lex(src)?;
    let expr = parse_expr_from_tokens(&toks)?;
    let mut s = String::with_capacity(128);
    s.push_str(&format!("Expr parsed: len={} toks={} variant={}\n", src.len(), toks.len(), expr.variant_name()));
    Ok((expr, s))
}

/// 實際使用：expr 文件清單
pub fn parse_expr_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("parse_expr.rs", "FullExpr 34 變體：Closure/Return/Break/Try/Cast/Range/Index/Array — 優化 with_capacity", "core/src/minirust/parse_expr.rs"),
        ("lexer.rs", "Tok 詞法 — expr 依賴", "core/src/minirust/lexer.rs"),
        ("parse_pat.rs", "Pat 解析 — match arm 依賴", "core/src/minirust/parse_pat.rs"),
    ]
}


