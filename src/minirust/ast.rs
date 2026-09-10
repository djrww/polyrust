//! Mini-Rust 抽象語法樹與型別宇宙。

use crate::minirust::lexer::Tok;

/// 型別宇宙（平坦化，共 7 種）：
/// i32, bool, (), &i32, &mut i32, &bool, &mut bool
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Type {
    I32,
    Bool,
    Unit,
    RefI32,
    RefMutI32,
    RefBool,
    RefMutBool,
}

pub const ALL_TYPES: [Type; 7] = [
    Type::I32,
    Type::Bool,
    Type::Unit,
    Type::RefI32,
    Type::RefMutI32,
    Type::RefBool,
    Type::RefMutBool,
];

pub const N_TYPES: usize = 7;

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::I32 => "i32",
            Type::Bool => "bool",
            Type::Unit => "()",
            Type::RefI32 => "&i32",
            Type::RefMutI32 => "&mut i32",
            Type::RefBool => "&bool",
            Type::RefMutBool => "&mut bool",
        }
    }
    pub fn index(&self) -> usize {
        ALL_TYPES.iter().position(|t| t == self).unwrap()
    }
    pub fn from_index(i: usize) -> Type {
        ALL_TYPES[i]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Mul,
    Lt,
    And,
}

impl BinOp {
    pub fn name(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Mul => "*",
            BinOp::Lt => "<",
            BinOp::And => "&&",
        }
    }
    pub fn rule_name(&self) -> &'static str {
        match self {
            BinOp::Add => "T-Add",
            BinOp::Mul => "T-Mul",
            BinOp::Lt => "T-Lt",
            BinOp::And => "T-And",
        }
    }
}

/// 表達式節點（帶唯一 id，供約束生成與型別解碼）。
#[derive(Clone, Debug)]
pub struct E {
    pub id: usize,
    pub kind: EKind,
}

#[derive(Clone, Debug)]
pub enum EKind {
    Int(i64),
    BoolV(bool),
    UnitLit,
    Var(String),
    /// let x = e1; e2
    Let(String, Box<E>, Box<E>),
    /// e1; e2（丟棄 e1 的值）
    Seq(Box<E>, Box<E>),
    BinOp(BinOp, Box<E>, Box<E>),
    Not(Box<E>),
    If(Box<E>, Box<E>, Box<E>),
    /// &x
    Ref(String),
    /// &mut x
    RefMut(String),
    /// *e
    Deref(Box<E>),
    /// x = e;（回傳 unit）
    AssignVar(String, Box<E>),
    /// *lhs = e;（lhs 須為 &mut）
    AssignDeref(Box<E>, Box<E>),
    /// f(e)
    Call(String, Box<E>),
    /// name!(原始 token)——宏調用（arm 選擇是管線的決策點）
    Invoke(String, Vec<Tok>),
}

#[derive(Clone, Debug)]
pub struct FnDef {
    pub name: String,
    pub param: String,
    pub param_node: usize, // 參數的專屬節點 id（型別位元用）
    pub param_ty: Type,
    pub ret_ty: Type,
    pub body: E,
}

/// 宏匹配模式（token 層）
#[derive(Clone, Debug, PartialEq)]
pub enum MPat {
    Tok(Tok),
    Group(Vec<MPat>),
    Hole(String, MSpec),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MSpec {
    Expr,
    Ident,
}

/// 宏轉錄模板
#[derive(Clone, Debug, PartialEq)]
pub enum TTmpl {
    Tok(Tok),
    Group(Bracket, Vec<TTmpl>),
    Ref(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bracket {
    Paren,
    Brace,
}

impl Bracket {
    pub fn open(&self) -> Tok {
        match self {
            Bracket::Paren => Tok::LParen,
            Bracket::Brace => Tok::LBrace,
        }
    }
    pub fn close(&self) -> Tok {
        match self {
            Bracket::Paren => Tok::RParen,
            Bracket::Brace => Tok::RBrace,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MacroArm {
    pub matcher: Vec<MPat>,   // 頂層 = 括號組內的模式
    pub template: Vec<TTmpl>, // 大括號組內的模板
}

#[derive(Clone, Debug)]
pub struct MacroDef {
    pub name: String,
    pub arms: Vec<MacroArm>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub fns: Vec<FnDef>,
    pub macros: Vec<MacroDef>,
    pub main_body: E,
    pub next_id: usize,
}

impl E {
    pub fn new(id: usize, kind: EKind) -> E {
        E { id, kind }
    }
}
