//! Mini-Rust 解析器：函式定義、macro_rules! 定義、表達式（優先級爬升）。

use crate::minirust::ast::*;
use crate::minirust::lexer::{lex, Tok};

pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    next_id: usize,
}

impl Parser {
    pub fn new(src: &str, next_id: usize) -> Result<Parser, String> {
        Ok(Parser { toks: lex(src)?, pos: 0, next_id })
    }

    /// 供 parse_program 內部（fn 參數節點）使用的新 id。
    pub fn fresh_pub_id(&mut self) -> usize {
        self.fresh_id()
    }

    fn fresh_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
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

    fn expect(&mut self, t: &Tok) -> Result<(), String> {
        match self.bump() {
            Some(g) if &g == t => Ok(()),
            Some(g) => Err(format!("期望 {:?}，得到 {:?}", t, g)),
            None => Err(format!("期望 {:?}，但輸入結束", t)),
        }
    }

    fn expect_kw(&mut self, k: &str) -> Result<(), String> {
        match self.bump() {
            Some(Tok::Kw(g)) if g == k => Ok(()),
            other => Err(format!("期望關鍵字 {}，得到 {:?}", k, other)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.bump() {
            Some(Tok::Ident(s)) => Ok(s),
            other => Err(format!("期望識別字，得到 {:?}", other)),
        }
    }

    /// 解析整個程序。
    pub fn parse_program(src: &str) -> Result<Program, String> {
        let mut p = Parser::new(src, 0)?;
        let mut fns = vec![];
        let mut macros = vec![];
        let mut main_body: Option<E> = None;
        loop {
            match p.peek() {
                None => break,
                Some(Tok::Kw("fn")) => {
                    p.bump();
                    let name = p.expect_ident()?;
                    if name == "main" {
                        // fn main() { body }
                        p.expect(&Tok::LParen)?;
                        p.expect(&Tok::RParen)?;
                        p.expect(&Tok::LBrace)?;
                        let body = p.parse_seq()?;
                        p.expect(&Tok::RBrace)?;
                        main_body = Some(body);
                    } else {
                        p.expect(&Tok::LParen)?;
                        let param = p.expect_ident()?;
                        p.expect(&Tok::Colon)?;
                        let param_ty = p.parse_ty()?;
                        p.expect(&Tok::RParen)?;
                        p.expect(&Tok::Arrow)?;
                        let ret_ty = p.parse_ty()?;
                        let param_node = p.fresh_pub_id();
                        p.expect(&Tok::LBrace)?;
                        let body = p.parse_seq()?;
                        p.expect(&Tok::RBrace)?;
                        fns.push(FnDef { name, param, param_node, param_ty, ret_ty, body });
                    }
                }
                Some(Tok::Kw("macro_rules")) => {
                    p.bump();
                    p.expect(&Tok::Not)?;
                    let name = p.expect_ident()?;
                    p.expect(&Tok::LBrace)?;
                    let mut arms = vec![];
                    loop {
                        match p.peek() {
                            Some(Tok::RBrace) => {
                                p.bump();
                                break;
                            }
                            Some(Tok::LParen) => {
                                let arm = p.parse_macro_arm()?;
                                arms.push(arm);
                                // 可選分號
                                if let Some(Tok::Semi) = p.peek() {
                                    p.bump();
                                }
                            }
                            other => return Err(format!("宏定義內期望 arm，得到 {:?}", other)),
                        }
                    }
                    macros.push(MacroDef { name, arms });
                }
                other => return Err(format!("頂層期望 fn/macro_rules，得到 {:?}", other)),
            }
        }
        let main_body = main_body.ok_or("缺少 fn main()")?;
        Ok(Program { fns, macros, main_body, next_id: p.next_id })
    }

    fn parse_macro_arm(&mut self) -> Result<MacroArm, String> {
        // ( matcher ) => { template }
        self.expect(&Tok::LParen)?;
        let mut matcher = vec![];
        loop {
            match self.peek() {
                Some(Tok::RParen) => {
                    self.bump();
                    break;
                }
                _ => matcher.push(self.parse_mpat()?),
            }
        }
        self.expect(&Tok::FatArrow)?;
        self.expect(&Tok::LBrace)?;
        let mut template = vec![];
        loop {
            match self.peek() {
                Some(Tok::RBrace) => {
                    self.bump();
                    break;
                }
                _ => template.push(self.parse_ttmpl()?),
            }
        }
        Ok(MacroArm { matcher, template })
    }

    fn parse_mpat(&mut self) -> Result<MPat, String> {
        match self.peek().cloned() {
            Some(Tok::Dollar) => {
                self.bump();
                let name = self.expect_ident()?;
                self.expect(&Tok::Colon)?;
                let spec = match self.bump() {
                    Some(Tok::Ident(s)) if s == "expr" => MSpec::Expr,
                    Some(Tok::Ident(s)) if s == "ident" => MSpec::Ident,
                    other => return Err(format!("期望片段類型 expr/ident，得到 {:?}", other)),
                };
                Ok(MPat::Hole(name, spec))
            }
            Some(Tok::LParen) => {
                self.bump();
                let mut inner = vec![];
                loop {
                    match self.peek() {
                        Some(Tok::RParen) => {
                            self.bump();
                            break;
                        }
                        _ => inner.push(self.parse_mpat()?),
                    }
                }
                Ok(MPat::Group(inner))
            }
            Some(t) => {
                self.bump();
                Ok(MPat::Tok(t))
            }
            None => Err("宏模式意外結束".into()),
        }
    }

    fn parse_ttmpl(&mut self) -> Result<TTmpl, String> {
        match self.peek().cloned() {
            Some(Tok::Dollar) => {
                self.bump();
                let name = self.expect_ident()?;
                Ok(TTmpl::Ref(name))
            }
            Some(Tok::LParen) => {
                self.bump();
                let mut inner = vec![];
                loop {
                    match self.peek() {
                        Some(Tok::RParen) => {
                            self.bump();
                            break;
                        }
                        _ => inner.push(self.parse_ttmpl()?),
                    }
                }
                Ok(TTmpl::Group(Bracket::Paren, inner))
            }
            Some(Tok::LBrace) => {
                self.bump();
                let mut inner = vec![];
                loop {
                    match self.peek() {
                        Some(Tok::RBrace) => {
                            self.bump();
                            break;
                        }
                        _ => inner.push(self.parse_ttmpl()?),
                    }
                }
                Ok(TTmpl::Group(Bracket::Brace, inner))
            }
            Some(t) => {
                self.bump();
                Ok(TTmpl::Tok(t))
            }
            None => Err("宏模板意外結束".into()),
        }
    }

    pub fn parse_ty(&mut self) -> Result<Type, String> {
        match self.bump() {
            Some(Tok::Ident(s)) if s == "i32" => Ok(Type::I32),
            Some(Tok::Ident(s)) if s == "bool" => Ok(Type::Bool),
            Some(Tok::LParen) => {
                self.expect(&Tok::RParen)?;
                Ok(Type::Unit)
            }
            Some(Tok::Amp) => {
                let mutable = match self.peek() {
                    Some(Tok::Kw("mut")) => {
                        self.bump();
                        true
                    }
                    _ => false,
                };
                match self.bump() {
                    Some(Tok::Ident(s)) if s == "i32" => {
                        Ok(if mutable { Type::RefMutI32 } else { Type::RefI32 })
                    }
                    Some(Tok::Ident(s)) if s == "bool" => {
                        Ok(if mutable { Type::RefMutBool } else { Type::RefBool })
                    }
                    other => Err(format!("引用僅支援 i32/bool，得到 {:?}", other)),
                }
            }
            other => Err(format!("無法辨識的型別 {:?}", other)),
        }
    }

    /// 語句序列 + 尾表達式（fn 體、if 分支、宏轉錄結果）。
    pub fn parse_seq(&mut self) -> Result<E, String> {
        match self.peek() {
            None => Ok(E::new(self.fresh_id(), EKind::UnitLit)),
            Some(Tok::RBrace) => Ok(E::new(self.fresh_id(), EKind::UnitLit)),
            Some(Tok::Kw("let")) => {
                self.bump();
                if let Some(Tok::Kw("mut")) = self.peek() {
                    self.bump(); // let mut x —— 可變性不影響型別推導（僅借用檢查用）
                }
                let name = self.expect_ident()?;
                // 可選型別標註 let x: T = ...
                if let Some(Tok::Colon) = self.peek() {
                    self.bump();
                    let _ty = self.parse_ty()?; // 標註僅作文檔（型別由推導決定）
                }
                self.expect(&Tok::Assign)?;
                let e1 = self.parse_expr()?;
                match self.peek() {
                    Some(Tok::Semi) => {
                        self.bump();
                        let rest = self.parse_seq()?;
                        Ok(E::new(self.fresh_id(), EKind::Let(name, Box::new(e1), Box::new(rest))))
                    }
                    _ => {
                        // let 為尾語句（body 為空）
                        let rest = E::new(self.fresh_id(), EKind::UnitLit);
                        Ok(E::new(self.fresh_id(), EKind::Let(name, Box::new(e1), Box::new(rest))))
                    }
                }
            }
            _ => {
                let first = self.parse_stmt_or_expr()?;
                if let Some(Tok::Semi) = self.peek() {
                    self.bump();
                    let rest = self.parse_seq()?;
                    Ok(E::new(self.fresh_id(), EKind::Seq(Box::new(first), Box::new(rest))))
                } else {
                    Ok(first)
                }
            }
        }
    }

    fn parse_stmt_or_expr(&mut self) -> Result<E, String> {
        let e = self.parse_expr()?;
        // 賦值語句？
        if let Some(Tok::Assign) = self.peek() {
            self.bump();
            let rhs = self.parse_expr()?;
            return match e.kind {
                EKind::Var(x) => Ok(E::new(e.id, EKind::AssignVar(x, Box::new(rhs)))),
                EKind::Deref(inner) => Ok(E::new(e.id, EKind::AssignDeref(inner, Box::new(rhs)))),
                _ => Err("賦值左側須為變量或解引用".into()),
            };
        }
        Ok(e)
    }

    pub fn parse_expr(&mut self) -> Result<E, String> {
        if let Some(Tok::Kw("if")) = self.peek() {
            self.bump();
            let c = self.parse_expr()?;
            self.expect(&Tok::LBrace)?;
            let t = self.parse_seq()?;
            self.expect(&Tok::RBrace)?;
            self.expect_kw("else")?;
            self.expect(&Tok::LBrace)?;
            let f = self.parse_seq()?;
            self.expect(&Tok::RBrace)?;
            return Ok(E::new(self.fresh_id(), EKind::If(Box::new(c), Box::new(t), Box::new(f))));
        }
        if let Some(Tok::Kw("let")) = self.peek() {
            // let 表達式（巨集轉錄中 let 為尾表達式的情形：let x = e1; e2 → e2 可能為空）
            self.bump();
            let name = self.expect_ident()?;
            self.expect(&Tok::Assign)?;
            let e1 = self.parse_expr()?;
            // 若有分號，解析 body；否則 body = unit
            let body = if let Some(Tok::Semi) = self.peek() {
                self.bump();
                self.parse_seq()?
            } else {
                E::new(self.fresh_id(), EKind::UnitLit)
            };
            return Ok(E::new(self.fresh_id(), EKind::Let(name, Box::new(e1), Box::new(body))));
        }
        self.parse_and()
    }

    fn parse_and(&mut self) -> Result<E, String> {
        let mut l = self.parse_cmp()?;
        while let Some(Tok::AndAnd) = self.peek() {
            self.bump();
            let r = self.parse_cmp()?;
            l = E::new(self.fresh_id(), EKind::BinOp(BinOp::And, Box::new(l), Box::new(r)));
        }
        Ok(l)
    }

    fn parse_cmp(&mut self) -> Result<E, String> {
        let l = self.parse_add()?;
        if let Some(Tok::Lt) = self.peek() {
            self.bump();
            let r = self.parse_add()?;
            return Ok(E::new(self.fresh_id(), EKind::BinOp(BinOp::Lt, Box::new(l), Box::new(r))));
        }
        Ok(l)
    }

    fn parse_add(&mut self) -> Result<E, String> {
        let mut l = self.parse_mul()?;
        while let Some(Tok::Plus) = self.peek() {
            self.bump();
            let r = self.parse_mul()?;
            l = E::new(self.fresh_id(), EKind::BinOp(BinOp::Add, Box::new(l), Box::new(r)));
        }
        Ok(l)
    }

    fn parse_mul(&mut self) -> Result<E, String> {
        let mut l = self.parse_unary()?;
        while let Some(Tok::Star) = self.peek() {
            self.bump();
            let r = self.parse_unary()?;
            l = E::new(self.fresh_id(), EKind::BinOp(BinOp::Mul, Box::new(l), Box::new(r)));
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> Result<E, String> {
        match self.peek().cloned() {
            Some(Tok::Not) => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(E::new(self.fresh_id(), EKind::Not(Box::new(e))))
            }
            Some(Tok::Star) => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(E::new(self.fresh_id(), EKind::Deref(Box::new(e))))
            }
            Some(Tok::Amp) => {
                self.bump();
                let mutable = match self.peek() {
                    Some(Tok::Kw("mut")) => {
                        self.bump();
                        true
                    }
                    _ => false,
                };
                let name = self.expect_ident()?;
                Ok(E::new(
                    self.fresh_id(),
                    if mutable { EKind::RefMut(name) } else { EKind::Ref(name) },
                ))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<E, String> {
        match self.peek().cloned() {
            Some(Tok::Int(n)) => {
                self.bump();
                Ok(E::new(self.fresh_id(), EKind::Int(n)))
            }
            Some(Tok::Kw("true")) => {
                self.bump();
                Ok(E::new(self.fresh_id(), EKind::BoolV(true)))
            }
            Some(Tok::Kw("false")) => {
                self.bump();
                Ok(E::new(self.fresh_id(), EKind::BoolV(false)))
            }
            Some(Tok::LParen) => {
                self.bump();
                if let Some(Tok::RParen) = self.peek() {
                    self.bump();
                    return Ok(E::new(self.fresh_id(), EKind::UnitLit));
                }
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen)?;
                // 括號不建立新節點（透明）
                Ok(e)
            }
            Some(Tok::Ident(name)) => {
                self.bump();
                match self.peek() {
                    Some(Tok::LParen) => {
                        self.bump();
                        let arg = self.parse_expr()?;
                        self.expect(&Tok::RParen)?;
                        Ok(E::new(self.fresh_id(), EKind::Call(name, Box::new(arg))))
                    }
                    Some(Tok::Not) => {
                        self.bump();
                        self.expect(&Tok::LParen)?;
                        let toks = self.capture_until_close_paren()?;
                        Ok(E::new(self.fresh_id(), EKind::Invoke(name, toks)))
                    }
                    _ => Ok(E::new(self.fresh_id(), EKind::Var(name))),
                }
            }
            other => Err(format!("無法解析的 token {:?}", other)),
        }
    }

    /// 捕獲到匹配右括號為止的 token（含嵌套），消耗最後的右括號。
    fn capture_until_close_paren(&mut self) -> Result<Vec<Tok>, String> {
        let mut depth = 1usize;
        let mut out = vec![];
        loop {
            match self.bump() {
                Some(Tok::LParen) => {
                    depth += 1;
                    out.push(Tok::LParen);
                }
                Some(Tok::RParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(out);
                    }
                    out.push(Tok::RParen);
                }
                Some(t) => out.push(t),
                None => return Err("宏調用括號未閉合".into()),
            }
        }
    }

    pub fn next_id_after(&self) -> usize {
        self.next_id
    }
}

/// 解析一段 token 作為語句序列（宏轉錄結果用）。
pub fn parse_tokens_as_seq(toks: &[Tok], next_id: usize) -> Result<(E, usize), String> {
    let mut p = Parser { toks: toks.to_vec(), pos: 0, next_id };
    let e = p.parse_seq()?;
    if p.pos != p.toks.len() {
        return Err(format!("轉錄後有多餘 token：{:?}", &p.toks[p.pos..]));
    }
    Ok((e, p.next_id))
}
