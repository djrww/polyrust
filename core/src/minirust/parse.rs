//! Mini-Rust 解析器：函式定義、macro_rules! 定義、表達式（優先級爬升）。

use crate::minirust::ast::*;
use crate::minirust::lexer::{lex, Tok};

/// Parse 模塊的 AST_SYNTAX_INVENTORY — 把 parse.rs 列入
/// 對應 ast.rs 的 AST_SYNTAX_INVENTORY，列出所有 parse 相關文件
pub const AST_SYNTAX_INVENTORY: &[(&str, &str, &str)] = &[
    ("parse.rs", "原始 Mini-Rust 解析 7型別 — Parser::parse_program, Type 7 + BinOp 9 + EKind 17", "core/src/minirust/parse.rs"),
    ("parse_v2.rs", "ProgramV2 解析 — struct/enum/fn/impl/trait/mod/const/static/type", "core/src/minirust/parse_v2.rs"),
    ("parse_pat.rs", "FullPat 解析 — 14 變體 Slice/Or/Range 100%", "core/src/minirust/parse_pat.rs"),
    ("parse_expr.rs", "FullExpr 解析 — 34 變體 Closure/Index/Await/Try/Cast/Range 100%", "core/src/minirust/parse_expr.rs"),
    ("parse_full.rs", "FullType 解析 — 15 變體 100% + 39 正例 + 9 反例 + 15 Pat + 39 Expr + 4 parse 比較", "core/src/minirust/parse_full.rs"),
    ("lexer.rs", "詞法 Tok 定義", "core/src/minirust/lexer.rs"),
    ("ast.rs", "統一 AST 單一來源 — 被 parse 依賴的 Type/EKind/Full* 定義", "core/src/minirust/ast.rs"),
];

/// 同名別名，方便從 parse 模塊直接訪問
pub const PARSE_AST_SYNTAX_INVENTORY: &[(&str, &str, &str)] = AST_SYNTAX_INVENTORY;

/// Parse 文件清單 — 包含 parse.rs 自身 — 優化版：預分配
pub fn parse_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    // 實際使用：直接引用靜態切片，避免不必要分配
    AST_SYNTAX_INVENTORY.to_vec()
}
/// 零分配版本：返回靜態切片，供 pipeline_v2 高頻調用
pub fn parse_file_list_static() -> &'static [(&'static str, &'static str, &'static str)] {
    AST_SYNTAX_INVENTORY
}
/// 實際使用：供 pipeline_v2 消費的文本列表，with_capacity 優化
pub fn parse_file_list_for_pipeline() -> Vec<String> {
    let mut out = Vec::with_capacity(AST_SYNTAX_INVENTORY.len());
    for (name, desc, path) in AST_SYNTAX_INVENTORY {
        let mut s = String::with_capacity(name.len() + desc.len() + path.len() + 8);
        s.push_str(name);
        s.push_str(": ");
        s.push_str(desc);
        s.push_str(" @ ");
        s.push_str(path);
        out.push(s);
    }
    out
}

/// Parse 全 AST 列表含 parse.rs — 詳細文本 — 優化 with_capacity
pub fn parse_ast_syntax_inventory_summary() -> String {
    // 預估容量：頭 200 + 每文件 120
    let mut out = String::with_capacity(256 + AST_SYNTAX_INVENTORY.len()*128);
    out.push_str("=== Parse AST_SYNTAX_INVENTORY (把 parse.rs 列入) ===\n");
    out.push_str(&format!("文件數: {}\n", AST_SYNTAX_INVENTORY.len()));
    for (name, desc, path) in AST_SYNTAX_INVENTORY {
        out.push_str("- ");
        out.push_str(name);
        out.push_str(": ");
        out.push_str(desc);
        out.push_str(" (");
        out.push_str(path);
        out.push_str(")\n");
    }
    out.push_str("\n--- 能力對照 ---\n");
    out.push_str("parse.rs: 7 Type + 17 EKind = 24 變體基礎\n");
    out.push_str("parse_v2.rs: 11 ItemV2 + 34 擴展\n");
    out.push_str("parse_pat.rs: 14 FullPat 100%\n");
    out.push_str("parse_expr.rs: 34 FullExpr 100%\n");
    out.push_str("parse_full.rs: 15 FullType 100% + 39/9/15/39 正反例\n");
    out.push_str("lexer.rs: Tok 詞法\n");
    out.push_str("ast.rs: 統一 AST 被依賴\n");
    out
}
/// 實際使用：pipeline_v2 消費的 JSON-like 摘要
pub fn parse_inventory_for_pipeline_v2() -> String {
    let mut out = String::with_capacity(2048);
    out.push_str("{\"parse_syntax_inventory\":[");
    for (i, (name, desc, path)) in AST_SYNTAX_INVENTORY.iter().enumerate() {
        if i>0 { out.push(','); }
        out.push_str(&format!("{{\"name\":\"{}\",\"desc\":\"{}\",\"path\":\"{}\"}}", name, desc.replace('\"', "'"), path));
    }
    out.push_str("]}");
    out
}
/// 實際使用：帶統計的解析入口，返回 Program + 變體覆蓋信息
pub fn parse_program_with_stats(src: &str) -> Result<(Program, String), String> {
    let prog = Parser::parse_program(src)?;
    let mut summary = String::with_capacity(256);
    summary.push_str(&format!("Program: {} fns, {} macros, next_id={}\n", prog.fns.len(), prog.macros.len(), prog.next_id));
    // 統計 EKind 覆蓋（簡化）
    summary.push_str(&format!("main_body id={}\n", prog.main_body.id));
    Ok((prog, summary))
}

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
                        // 參數列表（name: ty，逗號分隔）
                        let mut params = vec![];
                        loop {
                            match p.peek() {
                                Some(Tok::RParen) => {
                                    p.bump();
                                    break;
                                }
                                _ => {
                                    let pname = p.expect_ident()?;
                                    p.expect(&Tok::Colon)?;
                                    let pty = p.parse_ty()?;
                                    let pnode = p.fresh_pub_id();
                                    params.push(FnParam { name: pname, node: pnode, ty: pty });
                                    match p.peek() {
                                        Some(Tok::Comma) => {
                                            p.bump();
                                        }
                                        Some(Tok::RParen) => {}
                                        other => {
                                            return Err(format!(
                                                "參數列表期望 ',' 或 ')'，得到 {:?}",
                                                other
                                            ))
                                        }
                                    }
                                }
                            }
                        }
                        p.expect(&Tok::Arrow)?;
                        let ret_ty = p.parse_ty()?;
                        p.expect(&Tok::LBrace)?;
                        let body = p.parse_seq()?;
                        p.expect(&Tok::RBrace)?;
                        fns.push(FnDef { name, params, ret_ty, body });
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
        let op = match self.peek() {
            Some(Tok::Lt) => Some(BinOp::Lt),
            Some(Tok::Le) => Some(BinOp::Le),
            Some(Tok::Ge) => Some(BinOp::Ge),
            Some(Tok::EqEq) => Some(BinOp::Eq),
            Some(Tok::Ne) => Some(BinOp::Ne),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let r = self.parse_add()?;
            return Ok(E::new(self.fresh_id(), EKind::BinOp(op, Box::new(l), Box::new(r))));
        }
        Ok(l)
    }

    fn parse_add(&mut self) -> Result<E, String> {
        let mut l = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => Some(BinOp::Add),
                Some(Tok::Minus) => Some(BinOp::Sub),
                _ => None,
            };
            match op {
                Some(op) => {
                    self.bump();
                    let r = self.parse_mul()?;
                    l = E::new(self.fresh_id(), EKind::BinOp(op, Box::new(l), Box::new(r)));
                }
                None => break,
            }
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
            Some(Tok::Minus) => {
                self.bump();
                let e = self.parse_unary()?;
                Ok(E::new(self.fresh_id(), EKind::Neg(Box::new(e))))
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
                        let mut args = vec![];
                        loop {
                            match self.peek() {
                                Some(Tok::RParen) => {
                                    self.bump();
                                    break;
                                }
                                _ => {
                                    args.push(self.parse_expr()?);
                                    match self.peek() {
                                        Some(Tok::Comma) => {
                                            self.bump();
                                        }
                                        Some(Tok::RParen) => {}
                                        other => {
                                            return Err(format!(
                                                "實參列表期望 ',' 或 ')'，得到 {:?}",
                                                other
                                            ))
                                        }
                                    }
                                }
                            }
                        }
                        Ok(E::new(self.fresh_id(), EKind::Call(name, args)))
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
