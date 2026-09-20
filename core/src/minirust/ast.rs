// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Mini-Rust 抽象語法樹 — 統一版（AST 補齊）
//! 本文件合併：
//! - ast.rs 原始 7 型別 + BinOp + E + FnDef + Macro
//! - ast_v2.rs 擴展：Struct/Enum/Impl/Trait/Mod/Const/Static/TypeAlias + ProgramV2
//! - ast_full.rs 完整：SpanInfo/Vis/Attr/FullType/FullPat/FullExpr/FullItem/FullProgram
//! 目標：單一入口 ast.rs 包含全部 AST 定義，供 core 零依賴與前端 syn 對應。

use crate::minirust::lexer::Tok;
use super::universe::{TypeV2, parse_type_v2, BaseType, ExtType};

// ─────────────────────────────────────────────────────────────
// 原始 v0.1 AST (7 型別)
// ─────────────────────────────────────────────────────────────

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
    Sub,
    Mul,
    Lt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
}

impl BinOp {
    pub fn name(&self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "&&",
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
    /// 一元負號 -e
    Neg(Box<E>),
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
    Call(String, Vec<E>),
    /// name!(原始 token)——宏調用（arm 選擇是管線的決策點）
    Invoke(String, Vec<Tok>),
}

#[derive(Clone, Debug)]
pub struct FnParam {
    pub name: String,
    pub node: usize, // 參數的專屬節點 id（型別位元用）
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<FnParam>,
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

// ─────────────────────────────────────────────────────────────
// ast_v2 擴展（Phase1）
// ─────────────────────────────────────────────────────────────

/// 結構體定義
#[derive(Clone, Debug)]
pub struct StructDefV2 {
    pub name: String,
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub fields: Vec<(String, TypeV2)>,
    pub is_pub: bool,
    pub where_clauses: Vec<String>,
}

/// Enum 變體
#[derive(Clone, Debug)]
pub struct VariantV2 {
    pub name: String,
    pub fields: Vec<TypeV2>, // 元組變體或單值
    pub discriminant: Option<i64>,
}

/// Enum 定義
#[derive(Clone, Debug)]
pub struct EnumDefV2 {
    pub name: String,
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub variants: Vec<VariantV2>,
    pub is_pub: bool,
    pub where_clauses: Vec<String>,
}

/// 函數簽名
#[derive(Clone, Debug)]
pub struct FnSigV2 {
    pub name: String,
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub params: Vec<(String, TypeV2)>,
    pub ret: TypeV2,
    pub is_pub: bool,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub is_method: bool, // 是否為 impl 方法（有 self）
    pub where_clauses: Vec<String>,
    pub has_default: bool,
    pub default_body: Option<String>,
}

/// 函數定義
#[derive(Clone, Debug)]
pub struct FnDefV2 {
    pub sig: FnSigV2,
    pub body_src: String, // 原始 body 文本，Phase1 保留，後續 parse
}

/// Impl 塊
#[derive(Clone, Debug)]
pub struct ImplDefV2 {
    pub self_ty: TypeV2,
    pub trait_name: Option<String>,
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub methods: Vec<FnDefV2>,
    pub where_clauses: Vec<String>,
}

/// Trait 定義
#[derive(Clone, Debug)]
pub struct TraitDefV2 {
    pub name: String,
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub methods: Vec<FnSigV2>,
    pub is_pub: bool,
    pub is_unsafe: bool,
    pub where_clauses: Vec<String>,
    pub supertraits: Vec<String>,
}

/// Mod 塊
#[derive(Clone, Debug)]
pub struct ModDefV2 {
    pub name: String,
    pub items: Vec<ItemV2>,
    pub is_pub: bool,
}

/// 常量定義（新增 Path C）
#[derive(Clone, Debug)]
pub struct ConstDefV2 {
    pub name: String,
    pub ty: TypeV2,
    pub expr: Option<String>,
    pub is_pub: bool,
}

/// 靜態定義（新增 Path C）
#[derive(Clone, Debug)]
pub struct StaticDefV2 {
    pub name: String,
    pub ty: TypeV2,
    pub mutbl: bool,
    pub expr: Option<String>,
    pub is_pub: bool,
}

/// 類型別名（新增 Path C）
#[derive(Clone, Debug)]
pub struct TypeAliasDefV2 {
    pub name: String,
    pub generics: Vec<String>,
    pub ty: TypeV2,
    pub is_pub: bool,
}

/// 頂層項 — Path C 擴展 const/static/type + union
#[derive(Clone, Debug)]
pub enum ItemV2 {
    Struct(StructDefV2),
    Enum(EnumDefV2),
    Union(UnionDefV2),
    Fn(FnDefV2),
    Impl(ImplDefV2),
    Trait(TraitDefV2),
    Mod(ModDefV2),
    Const(ConstDefV2),
    Static(StaticDefV2),
    TypeAlias(TypeAliasDefV2),
    Use(String),
    Macro(String), // 宏定義文本占位
}

#[derive(Clone, Debug)]
pub struct UnionDefV2 {
    pub name: String,
    pub generics: Vec<String>,
    pub lifetimes: Vec<String>,
    pub fields: Vec<(String, TypeV2)>,
    pub is_pub: bool,
    pub where_clauses: Vec<String>,
}

/// 完整程序 v2
#[derive(Clone, Debug, Default)]
pub struct ProgramV2 {
    pub items: Vec<ItemV2>,
    pub main: Option<FnDefV2>,
    pub universe: super::universe::Universe,
}

impl ProgramV2 {
    pub fn new() -> Self {
        ProgramV2 {
            items: vec![],
            main: None,
            universe: super::universe::Universe::new(),
        }
    }

    /// 從 .poly v0.2 文本粗略解析（Phase1 簡化版，僅頂層 struct/enum/fn/mod）— Path C 擴展 const/static/type + Tuple/Array/Slice
    pub fn parse_v2(src: &str) -> Result<Self, String> {
        let mut prog = ProgramV2::new();
        let lines: Vec<&str> = src.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                i += 1;
                continue;
            }
            let is_pub_struct = line.starts_with("struct ") || line.starts_with("pub struct ") || line.starts_with("pub(crate) struct ");
            let is_pub_enum = line.starts_with("enum ") || line.starts_with("pub enum ") || line.starts_with("pub(crate) enum ");
            let is_union = line.starts_with("union ") || line.starts_with("pub union ") || line.starts_with("pub(crate) union ");
            let is_trait = line.starts_with("trait ") || line.starts_with("pub trait ") || line.starts_with("pub(crate) trait ") || line.starts_with("pub unsafe trait ") || line.starts_with("unsafe trait ");
            let is_impl = line.starts_with("impl ");
            let is_mod = line.starts_with("mod ") || line.starts_with("pub mod ") || line.starts_with("pub(crate) mod ");
            let is_fn = line.starts_with("fn ") || line.starts_with("pub fn ") || line.starts_with("async fn ") || line.starts_with("pub async fn ") || line.starts_with("unsafe fn ") || line.starts_with("pub unsafe fn ") || line.starts_with("pub(crate) fn ") || line.starts_with("const fn ") || line.starts_with("pub const fn ");
            let is_const = line.starts_with("const ") || line.starts_with("pub const ") || line.starts_with("pub(crate) const ");
            let is_static = line.starts_with("static ") || line.starts_with("pub static ") || line.starts_with("pub(crate) static ") || line.starts_with("pub static mut ") || line.starts_with("static mut ");
            let is_type_alias = line.starts_with("type ") || line.starts_with("pub type ") || line.starts_with("pub(crate) type ");

            if is_pub_struct {
                let def = Self::parse_struct(&lines, &mut i)?;
                for (_, ty) in &def.fields {
                    prog.universe.insert_closure(ty.clone());
                }
                prog.universe.insert_closure(TypeV2::Ext(ExtType::Struct { name: def.name.clone(), args: vec![] }));
                prog.items.push(ItemV2::Struct(def));
            } else if is_union {
                let def = Self::parse_union(&lines, &mut i)?;
                for (_, ty) in &def.fields {
                    prog.universe.insert_closure(ty.clone());
                }
                prog.universe.insert_closure(TypeV2::Ext(ExtType::Struct { name: def.name.clone(), args: vec![] }));
                prog.items.push(ItemV2::Union(def));
            } else if is_pub_enum {
                let def = Self::parse_enum(&lines, &mut i)?;
                for v in &def.variants {
                    for f in &v.fields {
                        prog.universe.insert_closure(f.clone());
                    }
                }
                prog.universe.insert_closure(TypeV2::Ext(ExtType::Enum { name: def.name.clone(), args: vec![] }));
                prog.items.push(ItemV2::Enum(def));
            } else if is_trait {
                let def = Self::parse_trait(&lines, &mut i)?;
                prog.items.push(ItemV2::Trait(def));
            } else if is_impl {
                let def = Self::parse_impl(&lines, &mut i)?;
                prog.universe.insert_closure(def.self_ty.clone());
                for m in &def.methods {
                    for (_, ty) in &m.sig.params {
                        prog.universe.insert_closure(ty.clone());
                    }
                    prog.universe.insert_closure(m.sig.ret.clone());
                }
                prog.items.push(ItemV2::Impl(def));
            } else if is_mod {
                let def = Self::parse_mod(&lines, &mut i)?;
                prog.items.push(ItemV2::Mod(def));
            } else if is_const {
                if let Ok(def) = Self::parse_const(&lines, &mut i) {
                    prog.universe.insert_closure(def.ty.clone());
                    prog.items.push(ItemV2::Const(def));
                } else { i += 1; }
            } else if is_static {
                if let Ok(def) = Self::parse_static(&lines, &mut i) {
                    prog.universe.insert_closure(def.ty.clone());
                    prog.items.push(ItemV2::Static(def));
                } else { i += 1; }
            } else if is_type_alias {
                if let Ok(def) = Self::parse_type_alias(&lines, &mut i) {
                    prog.universe.insert_closure(def.ty.clone());
                    prog.items.push(ItemV2::TypeAlias(def));
                } else { i += 1; }
            } else if is_fn {
                let def = Self::parse_fn(&lines, &mut i)?;
                if def.sig.name == "main" {
                    prog.main = Some(def);
                } else {
                    for (_, ty) in &def.sig.params {
                        prog.universe.insert_closure(ty.clone());
                    }
                    prog.universe.insert_closure(def.sig.ret.clone());
                    prog.items.push(ItemV2::Fn(def));
                }
            } else {
                i += 1;
            }
        }
        Ok(prog)
    }

    fn parse_union(lines: &[&str], idx: &mut usize) -> Result<UnionDefV2, String> {
        // union 解析：類似 struct，但語義是所有字段共享內存，訪問需 unsafe
        let line = lines[*idx].trim();
        let rest = if line.starts_with("pub(crate) union ") {
            line["pub(crate) union ".len()..].trim()
        } else if line.starts_with("pub union ") {
            line["pub union ".len()..].trim()
        } else if line.starts_with("union ") {
            line["union ".len()..].trim()
        } else if line.starts_with("pub(crate) ") {
            // fallback
            let after_pub = line["pub(crate) ".len()..].trim();
            if after_pub.starts_with("union ") {
                after_pub["union ".len()..].trim()
            } else {
                after_pub
            }
        } else if line.starts_with("pub ") {
            let after_pub = line["pub ".len()..].trim();
            if after_pub.starts_with("union ") {
                after_pub["union ".len()..].trim()
            } else {
                after_pub
            }
        } else {
            line
        };
        let name_end = rest.find(|c: char| c == '{' || c == '<' || c.is_whitespace()).unwrap_or(rest.len());
        let name = rest[..name_end].trim().to_string();
        let mut fields = Vec::new();
        *idx += 1;
        while *idx < lines.len() {
            let l = lines[*idx].trim();
            if l.starts_with('}') {
                *idx += 1;
                break;
            }
            if l.is_empty() || l.starts_with("//") || l.starts_with('#') {
                *idx += 1;
                continue;
            }
            if let Some(colon_idx) = l.find(':') {
                let fname = l[..colon_idx].trim().trim_start_matches("pub ").trim().to_string();
                let fty_str = l[colon_idx+1..].trim().trim_end_matches(',').trim_end_matches('}').trim().to_string();
                if let Ok(ty) = super::universe::parse_type_v2(&fty_str) {
                    fields.push((fname, ty));
                }
            }
            *idx += 1;
        }
        Ok(UnionDefV2 {
            name,
            generics: vec![],
            lifetimes: vec![],
            fields,
            is_pub: line.contains("pub"),
            where_clauses: vec![],
        })
    }

    fn parse_struct(lines: &[&str], idx: &mut usize) -> Result<StructDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let rest = if is_pub { line[4..].trim() } else { line };
        let after_struct = rest[7..].trim();
        let name_end = after_struct.find(|c: char| c == '{' || c == '<' || c.is_whitespace()).unwrap_or(after_struct.len());
        let name = after_struct[..name_end].trim().to_string();
        let mut fields = vec![];
        let mut brace_depth = 0;
        let mut collected = String::new();
        while *idx < lines.len() {
            let l = lines[*idx];
            for c in l.chars() {
                if c == '{' { brace_depth += 1; }
                if c == '}' { brace_depth -= 1; }
            }
            collected.push_str(l);
            collected.push('\n');
            *idx += 1;
            if brace_depth == 0 && collected.contains('{') {
                break;
            }
        }
        if let Some(start) = collected.find('{') {
            if let Some(end) = collected.rfind('}') {
                let inner = &collected[start+1..end];
                for part in inner.split(',') {
                    let part = part.trim();
                    if part.is_empty() { continue; }
                    if let Some(colon) = part.find(':') {
                        let fname = part[..colon].trim().trim_start_matches("pub ").to_string();
                        let fty_str = part[colon+1..].trim();
                        let fty = parse_type_v2(fty_str).unwrap_or(TypeV2::Base(BaseType::I32));
                        fields.push((fname, fty));
                    }
                }
            }
        }
        Ok(StructDefV2 { name, generics: vec![], lifetimes: vec![], fields, is_pub, where_clauses: vec![] })
    }

    fn parse_enum(lines: &[&str], idx: &mut usize) -> Result<EnumDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let rest = if is_pub { line[4..].trim() } else { line };
        let after_enum = rest[5..].trim();
        let name_end = after_enum.find(|c: char| c == '{' || c == '<' || c.is_whitespace()).unwrap_or(after_enum.len());
        let name = after_enum[..name_end].trim().to_string();
        let mut variants = vec![];
        let mut brace_depth = 0;
        let mut collected = String::new();
        while *idx < lines.len() {
            let l = lines[*idx];
            for c in l.chars() {
                if c == '{' { brace_depth += 1; }
                if c == '}' { brace_depth -= 1; }
            }
            collected.push_str(l);
            collected.push('\n');
            *idx += 1;
            if brace_depth == 0 && collected.contains('{') {
                break;
            }
        }
        if let Some(start) = collected.find('{') {
            if let Some(end) = collected.rfind('}') {
                let inner = &collected[start+1..end];
                for part in inner.split(',') {
                    let part = part.trim();
                    if part.is_empty() { continue; }
                    if let Some(p) = part.find('(') {
                        let vname = part[..p].trim().to_string();
                        let inner_ty_str = part[p+1..].trim().trim_end_matches(')');
                        let fields = if inner_ty_str.is_empty() { vec![] } else {
                            inner_ty_str.split(',').filter_map(|s| parse_type_v2(s.trim()).ok()).collect()
                        };
                        variants.push(VariantV2 { name: vname, fields, discriminant: None });
                    } else {
                        variants.push(VariantV2 { name: part.to_string(), fields: vec![], discriminant: None });
                    }
                }
            }
        }
        Ok(EnumDefV2 { name, generics: vec![], lifetimes: vec![], variants, is_pub, where_clauses: vec![] })
    }

    fn parse_fn(lines: &[&str], idx: &mut usize) -> Result<FnDefV2, String> {
        let mut collected = String::new();
        let mut brace_depth = 0;
        let mut started = false;
        while *idx < lines.len() {
            let l = lines[*idx];
            collected.push_str(l);
            collected.push('\n');
            for c in l.chars() {
                if c == '{' { brace_depth += 1; started = true; }
                if c == '}' { brace_depth -= 1; }
            }
            *idx += 1;
            if started && brace_depth == 0 {
                break;
            }
        }
        let first_line = collected.lines().next().unwrap_or("").trim();
        let is_pub = first_line.contains("pub ");
        let is_async = first_line.contains("async ");
        let is_unsafe = first_line.contains("unsafe ");
        let fn_pos = first_line.find("fn ").unwrap_or(0);
        let after_fn = &first_line[fn_pos+3..];
        let name_end = after_fn.find(['(', '<']).unwrap_or(after_fn.len());
        let name = after_fn[..name_end].trim().to_string();
        let sig = FnSigV2 {
            name,
            generics: vec![],
            lifetimes: vec![],
            params: vec![],
            ret: TypeV2::Base(BaseType::Unit),
            is_pub,
            is_unsafe,
            is_async,
            is_method: false,
            where_clauses: vec![],
            has_default: false,
            default_body: None,
        };
        Ok(FnDefV2 { sig, body_src: collected })
    }

    fn parse_impl(lines: &[&str], idx: &mut usize) -> Result<ImplDefV2, String> {
        let line = lines[*idx].trim();
        let mut trait_name = None;
        let after_impl = line[5..].trim();
        let self_ty_str: &str = if let Some(for_pos) = after_impl.find(" for ") {
            trait_name = Some(after_impl[..for_pos].trim().to_string());
            let mut s = after_impl[for_pos+5..].trim();
            if let Some(b) = s.find('{') {
                s = s[..b].trim();
            }
            s
        } else {
            let end = after_impl.find('{').unwrap_or(after_impl.len());
            after_impl[..end].trim()
        };
        let self_ty = parse_type_v2(self_ty_str).unwrap_or(TypeV2::Base(BaseType::I32));
        let mut methods = vec![];
        let mut brace_depth = 0;
        let mut collected = String::new();
        while *idx < lines.len() {
            let l = lines[*idx];
            for c in l.chars() {
                if c == '{' { brace_depth += 1; }
                if c == '}' { brace_depth -= 1; }
            }
            collected.push_str(l);
            collected.push('\n');
            *idx += 1;
            if brace_depth == 0 && collected.contains('{') {
                break;
            }
        }
        for l in collected.lines() {
            let lt = l.trim();
            if lt.starts_with("fn ") {
                let sig = FnSigV2 {
                    name: "method".to_string(),
                    generics: vec![],
                    lifetimes: vec![],
                    params: vec![],
                    ret: TypeV2::Base(BaseType::Unit),
                    is_pub: false,
                    is_unsafe: false,
                    is_async: false,
                    is_method: true,
                    where_clauses: vec![],
                    has_default: false,
                    default_body: None,
                };
                methods.push(FnDefV2 { sig, body_src: lt.to_string() });
            }
        }
        Ok(ImplDefV2 { self_ty, trait_name, generics: vec![], lifetimes: vec![], methods, where_clauses: vec![] })
    }

    fn parse_trait(lines: &[&str], idx: &mut usize) -> Result<TraitDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let is_unsafe = line.contains("unsafe ");
        let rest = if is_pub { line[4..].trim() } else { line };
        let after_trait = rest[6..].trim();
        let name_end = after_trait.find(|c: char| c == '{' || c == '<' || c.is_whitespace()).unwrap_or(after_trait.len());
        let name = after_trait[..name_end].trim().to_string();
        let mut brace_depth = 0;
        while *idx < lines.len() {
            let l = lines[*idx];
            for c in l.chars() {
                if c == '{' { brace_depth += 1; }
                if c == '}' { brace_depth -= 1; }
            }
            *idx += 1;
            if brace_depth == 0 {
                break;
            }
        }
        Ok(TraitDefV2 { name, generics: vec![], lifetimes: vec![], methods: vec![], is_pub, is_unsafe, where_clauses: vec![], supertraits: vec![] })
    }

    fn parse_mod(lines: &[&str], idx: &mut usize) -> Result<ModDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let rest = if is_pub { line[4..].trim() } else { line };
        let after_mod = rest[4..].trim();
        let name_end = after_mod.find(|c: char| c == '{' || c == ';' || c.is_whitespace()).unwrap_or(after_mod.len());
        let name = after_mod[..name_end].trim().to_string();
        let mut items = vec![];
        if after_mod.contains('{') {
            let mut brace_depth = 0;
            for c in after_mod.chars() {
                if c == '{' { brace_depth += 1; }
                if c == '}' { brace_depth -= 1; }
            }
            let mut inner_lines: Vec<String> = vec![];
            if let Some(pos) = after_mod.find('{') {
                let after = &after_mod[pos+1..];
                if !after.trim().is_empty() && !after.contains('}') {
                    inner_lines.push(after.to_string());
                }
            }
            *idx += 1;
            while *idx < lines.len() && brace_depth > 0 {
                let l = lines[*idx];
                for c in l.chars() {
                    if c == '{' { brace_depth += 1; }
                    if c == '}' { brace_depth -= 1; }
                }
                if brace_depth == 0 {
                    if let Some(end) = l.find('}') {
                        let before = &l[..end];
                        if !before.trim().is_empty() {
                            inner_lines.push(before.to_string());
                        }
                    }
                } else {
                    inner_lines.push(l.to_string());
                }
                *idx += 1;
            }
            if !inner_lines.is_empty() {
                let inner_src = inner_lines.join("\n");
                if let Ok(inner_prog) = ProgramV2::parse_v2(&inner_src) {
                    items.extend(inner_prog.items);
                } else {
                    let inner_refs: Vec<&str> = inner_lines.iter().map(|s| s.as_str()).collect();
                    let mut j = 0;
                    while j < inner_refs.len() {
                        let tl = inner_refs[j].trim();
                        if tl.is_empty() || tl.starts_with('#') || tl.starts_with("//") { j+=1; continue; }
                        if tl.starts_with("struct ") || tl.starts_with("pub struct ") {
                            if let Ok(def) = Self::parse_struct(&inner_refs, &mut j) {
                                items.push(ItemV2::Struct(def));
                                continue;
                            }
                        } else if tl.starts_with("enum ") || tl.starts_with("pub enum ") {
                            if let Ok(def) = Self::parse_enum(&inner_refs, &mut j) {
                                items.push(ItemV2::Enum(def));
                                continue;
                            }
                        } else if tl.starts_with("fn ") || tl.starts_with("pub fn ") {
                            if let Ok(def) = Self::parse_fn(&inner_refs, &mut j) {
                                items.push(ItemV2::Fn(def));
                                continue;
                            }
                        } else if tl.starts_with("mod ") || tl.starts_with("pub mod ") {
                            if let Ok(def) = Self::parse_mod(&inner_refs, &mut j) {
                                items.push(ItemV2::Mod(def));
                                continue;
                            }
                        }
                        j+=1;
                    }
                }
            }
        } else {
            *idx += 1;
        }
        Ok(ModDefV2 { name, items, is_pub })
    }

    fn parse_const(lines: &[&str], idx: &mut usize) -> Result<ConstDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let rest = if is_pub { line.trim_start_matches("pub ").trim().trim_start_matches("(crate) ").trim() } else { line };
        let after_const = rest[6..].trim();
        let name_end = after_const.find(|c: char| c == ':' || c == '=' || c.is_whitespace()).unwrap_or(after_const.len());
        let name = after_const[..name_end].trim().to_string();
        let mut ty = TypeV2::Base(BaseType::I32);
        let mut expr = None;
        if let Some(colon) = after_const.find(':') {
            let after_colon = after_const[colon+1..].trim();
            let eq_pos = after_colon.find('=');
            let ty_str = if let Some(eq) = eq_pos { after_colon[..eq].trim() } else { after_colon.trim().trim_end_matches(';') };
            ty = parse_type_v2(ty_str).unwrap_or(TypeV2::Base(BaseType::I32));
            if let Some(eq) = eq_pos {
                let e = after_colon[eq+1..].trim().trim_end_matches(';').to_string();
                expr = Some(e);
            }
        }
        *idx += 1;
        Ok(ConstDefV2 { name, ty, expr, is_pub })
    }

    fn parse_static(lines: &[&str], idx: &mut usize) -> Result<StaticDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let mut rest = if is_pub { line.trim_start_matches("pub ").trim().trim_start_matches("(crate) ").trim() } else { line };
        let mut mutbl = false;
        if rest.starts_with("static mut ") {
            mutbl = true;
            rest = rest[11..].trim();
        } else if rest.starts_with("static ") {
            rest = rest[7..].trim();
        }
        let name_end = rest.find(|c: char| c == ':' || c == '=' || c.is_whitespace()).unwrap_or(rest.len());
        let name = rest[..name_end].trim().to_string();
        let mut ty = TypeV2::Base(BaseType::I32);
        let mut expr = None;
        if let Some(colon) = rest.find(':') {
            let after_colon = rest[colon+1..].trim();
            let eq_pos = after_colon.find('=');
            let ty_str = if let Some(eq) = eq_pos { after_colon[..eq].trim() } else { after_colon.trim().trim_end_matches(';') };
            ty = parse_type_v2(ty_str).unwrap_or(TypeV2::Base(BaseType::I32));
            if let Some(eq) = eq_pos {
                let e = after_colon[eq+1..].trim().trim_end_matches(';').to_string();
                expr = Some(e);
            }
        }
        *idx += 1;
        Ok(StaticDefV2 { name, ty, mutbl, expr, is_pub })
    }

    fn parse_type_alias(lines: &[&str], idx: &mut usize) -> Result<TypeAliasDefV2, String> {
        let line = lines[*idx].trim();
        let is_pub = line.starts_with("pub ");
        let rest = if is_pub { line.trim_start_matches("pub ").trim().trim_start_matches("(crate) ").trim() } else { line };
        let after_type = rest[5..].trim();
        let name_end = after_type.find(|c: char| c == '<' || c == '=' || c.is_whitespace()).unwrap_or(after_type.len());
        let name = after_type[..name_end].trim().to_string();
        let mut generics = vec![];
        if let Some(lt) = after_type.find('<') {
            if let Some(gt) = after_type.find('>') {
                let gen_str = &after_type[lt+1..gt];
                generics = gen_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
        }
        let mut ty = TypeV2::Base(BaseType::I32);
        if let Some(eq) = after_type.find('=') {
            let ty_str = after_type[eq+1..].trim().trim_end_matches(';').to_string();
            ty = parse_type_v2(&ty_str).unwrap_or(TypeV2::Base(BaseType::I32));
        }
        *idx += 1;
        Ok(TypeAliasDefV2 { name, generics, ty, is_pub })
    }

    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("ProgramV2: {} items, universe N={}\n", self.items.len(), self.universe.n_types()));
        s.push_str(&self.universe.display());
        for item in &self.items {
            match item {
                ItemV2::Struct(d) => s.push_str(&format!("  struct {} {{ {} fields }}\n", d.name, d.fields.len())),
                ItemV2::Union(d) => s.push_str(&format!("  union {} {{ {} fields, unsafe }}\n", d.name, d.fields.len())),
                ItemV2::Enum(d) => s.push_str(&format!("  enum {} {{ {} variants }}\n", d.name, d.variants.len())),
                ItemV2::Fn(d) => s.push_str(&format!("  fn {} (async={}, unsafe={})\n", d.sig.name, d.sig.is_async, d.sig.is_unsafe)),
                ItemV2::Impl(d) => s.push_str(&format!("  impl {} for {} ({} methods)\n", d.trait_name.as_deref().unwrap_or(""), d.self_ty.name(), d.methods.len())),
                ItemV2::Trait(d) => s.push_str(&format!("  trait {} (unsafe={})\n", d.name, d.is_unsafe)),
                ItemV2::Mod(d) => s.push_str(&format!("  mod {} ({} items)\n", d.name, d.items.len())),
                ItemV2::Const(d) => s.push_str(&format!("  const {}: {} {}\n", d.name, d.ty.name(), if d.is_pub { "pub" } else { "" })),
                ItemV2::Static(d) => s.push_str(&format!("  static {}{}: {} {}\n", if d.mutbl { "mut " } else { "" }, d.name, d.ty.name(), if d.is_pub { "pub" } else { "" })),
                ItemV2::TypeAlias(d) => s.push_str(&format!("  type {} = {} {}\n", d.name, d.ty.name(), if d.is_pub { "pub" } else { "" })),
                ItemV2::Use(u) => s.push_str(&format!("  use {}\n", u)),
                ItemV2::Macro(m) => s.push_str(&format!("  macro {}\n", m.chars().take(30).collect::<String>())),
            }
        }
        if let Some(m) = &self.main {
            s.push_str(&format!("  fn main (body {} chars)\n", m.body_src.len()));
        }
        s
    }
}

// ─────────────────────────────────────────────────────────────
// ast_full 完整（Phase3）
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpanInfo {
    pub line: usize,
    pub col: usize,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Vis {
    Private,
    Pub,
    PubCrate,
    PubSuper,
    PubSelf,
    PubIn(String),
}

#[derive(Clone, Debug)]
pub struct Attr {
    pub name: String,
    pub args: Option<String>,
    pub is_inner: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum GenericParam {
    Type { name: String, bounds: Vec<TypeBound>, default: Option<FullType> },
    Lifetime(Lifetime),
    Const { name: String, ty: FullType, default: Option<String> },
}

#[derive(Clone, Debug)]
#[derive(Default)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}


#[derive(Clone, Debug)]
pub struct WhereClause {
    pub subject: String,
    pub bounds: Vec<TypeBound>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeBound {
    Trait(String),
    Lifetime(String),
    Outlives(String, String),
}

#[derive(Clone, Debug)]
pub enum FullType {
    V2(TypeV2),
    Path { path: String, args: Vec<FullType> },
    Tuple(Vec<FullType>),
    Array { elem: Box<FullType>, len: Option<String> },
    Slice(Box<FullType>),
    Ptr { mutbl: bool, inner: Box<FullType> },
    Ref { mutbl: bool, lifetime: Option<String>, inner: Box<FullType> },
    BareFn { params: Vec<FullType>, ret: Box<FullType>, is_unsafe: bool, is_async: bool },
    Never,
    Inferred,
    TraitObject { bounds: Vec<TypeBound>, dyn_token: bool },
    ImplTrait { bounds: Vec<TypeBound> },
    Macro(String),
    Group(Box<FullType>),
    Paren(Box<FullType>),
}

impl FullType {
    pub fn name(&self) -> String {
        match self {
            FullType::V2(t) => t.name(),
            FullType::Path { path, args } => {
                if args.is_empty() { path.clone() } else {
                    format!("{}<{}>", path, args.iter().map(|a| a.name()).collect::<Vec<_>>().join(", "))
                }
            }
            FullType::Tuple(ts) => format!("({})", ts.iter().map(|t| t.name()).collect::<Vec<_>>().join(", ")),
            FullType::Array { elem, len } => {
                if let Some(l) = len { format!("[{}; {}]", elem.name(), l) } else { format!("[{}]", elem.name()) }
            }
            FullType::Slice(e) => format!("[{}]", e.name()),
            FullType::Ptr { mutbl, inner } => if *mutbl { format!("*mut {}", inner.name()) } else { format!("*const {}", inner.name()) },
            FullType::Ref { mutbl, lifetime, inner } => {
                let lt = lifetime.as_deref().unwrap_or("");
                let lt_s = if lt.is_empty() { "".to_string() } else { format!("{} ", lt) };
                if *mutbl { format!("&{}mut {}", lt_s, inner.name()) } else { format!("&{}{}", lt_s, inner.name()) }
            }
            FullType::BareFn { params, ret, .. } => format!("fn({})->{}", params.iter().map(|p| p.name()).collect::<Vec<_>>().join(", "), ret.name()),
            FullType::Never => "!".to_string(),
            FullType::Inferred => "_".to_string(),
            FullType::TraitObject { bounds, .. } => format!("dyn {}", bounds.iter().map(|b| format!("{:?}", b)).collect::<Vec<_>>().join(" + ")),
            FullType::ImplTrait { bounds } => format!("impl {}", bounds.iter().map(|b| format!("{:?}", b)).collect::<Vec<_>>().join(" + ")),
            FullType::Macro(s) => s.clone(),
            FullType::Group(t) | FullType::Paren(t) => t.name(),
        }
    }

    pub fn to_v2(&self) -> TypeV2 {
        match self {
            FullType::V2(t) => t.clone(),
            FullType::Path { path, args } => {
                let s = if args.is_empty() { path.clone() } else {
                    format!("{}<{}>", path, args.iter().map(|a| a.name()).collect::<Vec<_>>().join(","))
                };
                parse_type_v2(&s).unwrap_or(TypeV2::Base(BaseType::I32))
            }
            FullType::Ptr { mutbl, inner } => {
                TypeV2::Ext(ExtType::RawPtr { mutbl: *mutbl, inner: Box::new(inner.to_v2()) })
            }
            FullType::Ref { mutbl, lifetime, inner } => {
                TypeV2::Ext(ExtType::RefExt { mutbl: *mutbl, inner: Box::new(inner.to_v2()), lifetime: lifetime.clone() })
            }
            FullType::Tuple(ts) => {
                let v2_ts: Vec<TypeV2> = ts.iter().map(|t| t.to_v2()).collect();
                TypeV2::Ext(ExtType::Tuple(v2_ts))
            }
            FullType::Array { elem, len } => {
                TypeV2::Ext(ExtType::Array { elem: Box::new(elem.to_v2()), len: len.clone() })
            }
            FullType::Slice(elem) => {
                TypeV2::Ext(ExtType::Slice(Box::new(elem.to_v2())))
            }
            FullType::BareFn { params, ret, .. } => {
                let v2_params: Vec<TypeV2> = params.iter().map(|p| p.to_v2()).collect();
                TypeV2::Ext(ExtType::BareFn { params: v2_params, ret: Box::new(ret.to_v2()) })
            }
            FullType::Never => TypeV2::Ext(ExtType::Never),
            FullType::Inferred => TypeV2::Ext(ExtType::Inferred),
            FullType::TraitObject { bounds, .. } => {
                TypeV2::Ext(ExtType::TraitObject { bounds: bounds.iter().map(|b| format!("{:?}", b)).collect() })
            }
            FullType::ImplTrait { bounds } => {
                TypeV2::Ext(ExtType::ImplTrait { bounds: bounds.iter().map(|b| format!("{:?}", b)).collect() })
            }
            FullType::Paren(inner) | FullType::Group(inner) => inner.to_v2(),
            FullType::Macro(s) => parse_type_v2(s).unwrap_or(TypeV2::Base(BaseType::I32)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FullPat {
    Wild,
    Ident { name: String, mutbl: bool, by_ref: bool, subpat: Option<Box<FullPat>> },
    Lit(String),
    Path(String),
    TupleStruct { path: String, elems: Vec<FullPat> },
    Struct { path: String, fields: Vec<(String, FullPat)>, rest: bool },
    Tuple(Vec<FullPat>),
    Slice(Vec<FullPat>),
    Or(Vec<FullPat>),
    Ref { mutbl: bool, inner: Box<FullPat> },
    Box(Box<FullPat>),
    Range { start: Option<String>, end: Option<String>, inclusive: bool },
    Macro(String),
    Type { pat: Box<FullPat>, ty: FullType },
}

#[derive(Clone, Debug)]
pub enum FullExpr {
    Lit(String),
    Path(String),
    Field { base: Box<FullExpr>, field: String },
    Index { base: Box<FullExpr>, index: Box<FullExpr> },
    Call { func: Box<FullExpr>, args: Vec<FullExpr> },
    MethodCall { receiver: Box<FullExpr>, method: String, turbofish: Vec<FullType>, args: Vec<FullExpr> },
    Unary { op: String, expr: Box<FullExpr> },
    Binary { op: String, left: Box<FullExpr>, right: Box<FullExpr> },
    Assign { left: Box<FullExpr>, right: Box<FullExpr> },
    AssignOp { op: String, left: Box<FullExpr>, right: Box<FullExpr> },
    If { cond: Box<FullExpr>, then_branch: Box<FullExpr>, else_branch: Option<Box<FullExpr>> },
    Match { scrutinee: Box<FullExpr>, arms: Vec<MatchArm> },
    Loop { body: Box<FullExpr>, label: Option<String> },
    While { cond: Box<FullExpr>, body: Box<FullExpr>, label: Option<String> },
    For { pat: FullPat, iter: Box<FullExpr>, body: Box<FullExpr>, label: Option<String> },
    Block { stmts: Vec<FullStmt>, label: Option<String> },
    Unsafe(Box<FullExpr>),
    Async { capture: Option<String>, block: Box<FullExpr> },
    Await { base: Box<FullExpr> },
    Closure { inputs: Vec<(String, Option<FullType>)>, body: Box<FullExpr>, is_async: bool, is_move: bool, is_mut: bool },
    Return(Option<Box<FullExpr>>),
    Break { label: Option<String>, expr: Option<Box<FullExpr>> },
    Continue(Option<String>),
    Let { pat: FullPat, expr: Box<FullExpr> },
    StructLit { path: String, fields: Vec<(String, FullExpr)>, rest: Option<Box<FullExpr>> },
    Array(Vec<FullExpr>),
    ArrayRepeat { elem: Box<FullExpr>, len: Box<FullExpr> },
    Tuple(Vec<FullExpr>),
    Cast { expr: Box<FullExpr>, ty: FullType },
    TypeAscribe { expr: Box<FullExpr>, ty: FullType },
    Try(Box<FullExpr>),
    Range { start: Option<Box<FullExpr>>, end: Option<Box<FullExpr>>, inclusive: bool },
    Macro(String, String),
    Verbatim(String),
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pat: FullPat,
    pub guard: Option<FullExpr>,
    pub body: FullExpr,
    pub comma: bool,
}

#[derive(Clone, Debug)]
pub enum FullStmt {
    Local { pat: FullPat, ty: Option<FullType>, init: Option<FullExpr>, attrs: Vec<Attr> },
    Item(FullItem),
    Expr(FullExpr),
    Semi(FullExpr),
    Macro(String),
}

#[derive(Clone, Debug)]
pub enum FullItem {
    Fn(FnItem),
    Struct(StructItem),
    Enum(EnumItem),
    Union(UnionItem),
    Impl(ImplItem),
    Trait(TraitItem),
    Mod(ModItem),
    Use(UseItem),
    Const(ConstItem),
    Static(StaticItem),
    TypeAlias(TypeAliasItem),
    Macro(MacroItem),
    ExternCrate(String),
    ExternBlock(ExternBlockItem),
}

#[derive(Clone, Debug)]
pub struct FnItem {
    pub vis: Vis,
    pub sig: FnSig,
    pub block: Option<FullExpr>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct FnSig {
    pub name: String,
    pub generics: Generics,
    pub inputs: Vec<FnInput>,
    pub output: FullType,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_const: bool,
    pub abi: Option<String>,
}

#[derive(Clone, Debug)]
pub enum FnInput {
    Receiver { mutbl: bool, reference: bool, lifetime: Option<String> },
    Typed { pat: FullPat, ty: FullType },
}

#[derive(Clone, Debug)]
pub struct StructItem {
    pub vis: Vis,
    pub name: String,
    pub generics: Generics,
    pub fields: StructFields,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub enum StructFields {
    Unit,
    Tuple(Vec<(Vis, FullType)>),
    Named(Vec<NamedField>),
}

#[derive(Clone, Debug)]
pub struct NamedField {
    pub vis: Vis,
    pub name: String,
    pub ty: FullType,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct UnionItem {
    pub vis: Vis,
    pub name: String,
    pub generics: Generics,
    pub fields: Vec<NamedField>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct EnumItem {
    pub vis: Vis,
    pub name: String,
    pub generics: Generics,
    pub variants: Vec<EnumVariant>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub fields: StructFields,
    pub discriminant: Option<FullExpr>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct ImplItem {
    pub generics: Generics,
    pub trait_ref: Option<(bool, String, Vec<FullType>)>,
    pub self_ty: FullType,
    pub items: Vec<FullItem>,
    pub is_unsafe: bool,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct TraitItem {
    pub vis: Vis,
    pub name: String,
    pub generics: Generics,
    pub bounds: Vec<TypeBound>,
    pub items: Vec<FullItem>,
    pub is_unsafe: bool,
    pub is_auto: bool,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct ModItem {
    pub vis: Vis,
    pub name: String,
    pub items: Option<Vec<FullItem>>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct UseItem {
    pub vis: Vis,
    pub tree: UseTree,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub enum UseTree {
    Path { prefix: String, tree: Box<UseTree> },
    Name(String),
    Rename { name: String, rename: String },
    Glob,
    Group(Vec<UseTree>),
}

#[derive(Clone, Debug)]
pub struct ConstItem {
    pub vis: Vis,
    pub name: String,
    pub ty: FullType,
    pub expr: Option<FullExpr>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct StaticItem {
    pub vis: Vis,
    pub name: String,
    pub ty: FullType,
    pub mutbl: bool,
    pub expr: Option<FullExpr>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct TypeAliasItem {
    pub vis: Vis,
    pub name: String,
    pub generics: Generics,
    pub bounds: Vec<TypeBound>,
    pub ty: Option<FullType>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct MacroItem {
    pub name: Option<String>,
    pub tokens: String,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug)]
pub struct ExternBlockItem {
    pub abi: Option<String>,
    pub items: Vec<FullItem>,
    pub attrs: Vec<Attr>,
}

#[derive(Clone, Debug, Default)]
pub struct FullProgram {
    pub attrs: Vec<Attr>,
    pub items: Vec<FullItem>,
    pub main: Option<FnItem>,
}

pub trait FullParser {
    fn parse_program(src: &str) -> Result<FullProgram, String>;
}

pub struct HandwrittenParser;

impl HandwrittenParser {
    pub fn parser_capability() -> Vec<(String, bool, String)> {
        let pat_or_supported = crate::minirust::parse_pat::parse_pat_str("a | b").is_ok();
        let pat_range_supported = crate::minirust::parse_pat::parse_pat_str("0..10").is_ok() && crate::minirust::parse_pat::parse_pat_str("0..=10").is_ok();
        let closure_supported = crate::minirust::parse_expr::parse_expr_str("|x| x+1").is_ok() && crate::minirust::parse_expr::parse_expr_str("|| 42").is_ok();
        let return_supported = crate::minirust::parse_expr::parse_expr_str("return 5").is_ok();
        let break_supported = crate::minirust::parse_expr::parse_expr_str("break").is_ok() && crate::minirust::parse_expr::parse_expr_str("break 'a 1").is_ok();
        let try_supported = crate::minirust::parse_expr::parse_expr_str("x?").is_ok();
        let cast_supported = crate::minirust::parse_expr::parse_expr_str("x as i32").is_ok() && crate::minirust::parse_expr::parse_expr_str("x as *mut i32").is_ok();
        let range_expr_supported = crate::minirust::parse_expr::parse_expr_str("0..10").is_ok() && crate::minirust::parse_expr::parse_expr_str("0..=10").is_ok();
        let tuple_supported = crate::minirust::parse_full::parse_full_type_str("(i32, bool)").is_ok();
        let array_supported = crate::minirust::parse_full::parse_full_type_str("[i32; 3]").is_ok();
        let slice_supported = crate::minirust::parse_full::parse_full_type_str("[i32]").is_ok();
        let bare_fn_supported = crate::minirust::parse_full::parse_full_type_str("fn(i32) -> bool").is_ok();

        let checks = vec![
            ("struct", true, "具名結構體"),
            ("enum", true, "枚舉"),
            ("fn", true, "函數"),
            ("impl", true, "實現塊"),
            ("trait", true, "特徵"),
            ("mod", true, "模塊"),
            ("use", true, "導入"),
            ("const", true, "常量 — parse_full.rs ItemV2::Const"),
            ("static", true, "靜態 — parse_full.rs ItemV2::Static"),
            ("type alias", true, "類型別名 — parse_full.rs ItemV2::TypeAlias"),
            ("Vec<T>", true, "泛型容器"),
            ("HashMap", true, "哈希表"),
            ("Option", true, "Option"),
            ("Result", true, "Result"),
            ("Tuple type", tuple_supported, "元組類型 — FullType::Tuple (i32,bool)"),
            ("Array [T; N]", array_supported, "數組 — FullType::Array [T; N]"),
            ("Slice [T]", slice_supported, "切片 — FullType::Slice [T]"),
            ("*mut/*const", true, "裸指針"),
            ("&T / &mut T", true, "引用"),
            ("&'a T lifetime", true, "生命週期引用"),
            ("fn ptr", bare_fn_supported, "函數指針 — FullType::BareFn fn()->"),
            ("impl Trait", true, "impl Trait"),
            ("dyn Trait", true, "trait object"),
            ("!", true, "Never type"),
            ("_", true, "推斷類型"),
            ("let PAT", true, "let 綁定"),
            ("if", true, "if 表達式"),
            ("match", true, "match"),
            ("loop", true, "loop"),
            ("while", true, "while"),
            ("for", true, "for"),
            ("pat or (a|b)", pat_or_supported, "Pat Or a|b — parse_pat.rs FullPat::Or"),
            ("pat range", pat_range_supported, "Pat Range 0..10, 0..=10 — parse_pat.rs FullPat::Range"),
            ("closure |", closure_supported, "閉包 |x| x+1, ||, move |x| — parse_expr.rs FullExpr::Closure"),
            ("closure move", closure_supported, "move 閉包 move ||, move |x| — parse_expr.rs"),
            ("await", true, "await — ast_full 已有 FullExpr::Await"),
            ("async", true, "async — FullExpr::Async"),
            ("unsafe", true, "unsafe 塊 — FullExpr::Unsafe"),
            ("return", return_supported, "return — parse_expr.rs FullExpr::Return"),
            ("break/continue", break_supported, "break/continue — parse_expr.rs Break { label, expr }"),
            ("break label", break_supported, "break 'label expr — 支持 label 與 expr"),
            ("? try", try_supported, "? 操作符 Try — parse_expr.rs FullExpr::Try"),
            ("cast as", cast_supported, "cast as — parse_expr.rs FullExpr::Cast, 支持 *mut/*const"),
            (".. range", range_expr_supported, "range 0..10, 0..=10, ..10, 0.. — parse_expr.rs FullExpr::Range"),
            ("range inclusive", range_expr_supported, "range inclusive ..= — parse_expr.rs"),
            ("macro!", true, "宏調用 — FullExpr::Macro"),
            ("attr #[...]", true, "屬性 — Attr"),
            ("vis pub", true, "可見性 — Vis::Pub"),
            ("generic <T>", true, "泛型參數 — Generics"),
            ("where clause", true, "where 子句 — WhereClause"),
            ("lifetime param 'a", true, "生命週期參數 — Lifetime"),
        ];
        checks.into_iter().map(|(k, present, desc)| {
            let status = if present { "✅ 已實現" } else { "❌ 未實現" };
            (k.to_string(), present, format!("{} - {}", status, desc))
        }).collect()
    }

    pub fn coverage_report(src: &str) -> Vec<(String, bool, String)> {
        let checks = vec![
            ("struct", src.contains("struct "), "具名結構體"),
            ("enum", src.contains("enum "), "枚舉"),
            ("fn", src.contains("fn "), "函數"),
            ("impl", src.contains("impl "), "實現塊"),
            ("trait", src.contains("trait "), "特徵"),
            ("mod", src.contains("mod "), "模塊"),
            ("use", src.contains("use "), "導入"),
            ("const", src.contains("const "), "常量"),
            ("static", src.contains("static "), "靜態"),
            ("type alias", src.contains("type ") && src.contains("="), "類型別名"),
            ("Vec<T>", src.contains("Vec<"), "泛型容器"),
            ("HashMap", src.contains("HashMap<"), "哈希表"),
            ("Option", src.contains("Option<"), "Option"),
            ("Result", src.contains("Result<"), "Result"),
            ("Tuple type", src.contains("(i32") || src.contains("(bool") || src.contains("(String") || src.contains("): (") || src.contains("-> (") || src.contains("(i32,"), "元組類型"),
            ("Array [T; N]", src.contains("[") && src.contains(";") && src.contains("]") && (src.contains("[i32;") || src.contains("[u32;") || src.contains("[String;") || src.contains("Array")), "數組"),
            ("Slice [T]", src.contains("[") && src.contains("]") && (src.contains("&[") || src.contains("[T]") || src.contains("[i32]")), "切片"),
            ("*mut/*const", src.contains("*mut") || src.contains("*const"), "裸指針"),
            ("&T / &mut T", src.contains("&") && (src.contains("&mut") || src.contains("&T") || src.contains("&i32")), "引用"),
            ("&'a T lifetime", src.contains("&'"), "生命週期引用"),
            ("fn ptr", src.contains("fn(") && src.contains("->"), "函數指針"),
            ("impl Trait", src.contains("impl ") && src.contains("Trait"), "impl Trait"),
            ("dyn Trait", src.contains("dyn "), "trait object"),
            ("!", src.contains("-> !") || src.contains(": !") || src.contains("!;"), "Never type"),
            ("_", src.contains(" _ ") || src.contains(":_") || src.contains(" _;") || src.contains(" _,"), "推斷類型"),
            ("let PAT", src.contains("let "), "let 綁定"),
            ("if", src.contains("if ") || src.contains("if("), "if 表達式"),
            ("match", src.contains("match "), "match"),
            ("loop", src.contains("loop"), "loop"),
            ("while", src.contains("while "), "while"),
            ("for", src.contains("for ") && src.contains(" in "), "for"),
            ("pat or (a|b)", src.contains("|") && src.contains("=>") && (src.contains("Some(") || src.contains("None") || src.contains("match")), "Pat Or a|b"),
            ("pat range", src.contains("..") && (src.contains("=>") || src.contains("match")), "Pat Range"),
            ("closure |", src.contains("|") && (src.contains("|x|") || src.contains("||") || src.contains("|x,") || src.contains("move |")), "閉包"),
            ("closure move", src.contains("move |") || src.contains("move ||"), "move 閉包"),
            ("await", src.contains(".await"), "await"),
            ("async", src.contains("async "), "async"),
            ("unsafe", src.contains("unsafe"), "unsafe 塊"),
            ("return", src.contains("return"), "return"),
            ("break/continue", src.contains("break") || src.contains("continue"), "break/continue"),
            ("break label", src.contains("break '"), "break 'label"),
            ("? try", src.contains("?;") || src.contains("? )") || src.contains("r?") || src.contains("?;") || (src.contains("?") && src.contains("Result")), "? 操作符"),
            ("cast as", src.contains(" as "), "cast as"),
            (".. range", src.contains("..") && (src.contains("0..") || src.contains("..10") || src.contains("..=")), "range"),
            ("range inclusive", src.contains("..="), "range inclusive"),
            ("macro!", src.contains("!(") || src.contains("macro_rules!"), "宏調用"),
            ("attr #[...]", src.contains("#["), "屬性"),
            ("vis pub", src.contains("pub "), "可見性"),
            ("generic <T>", src.contains("<") && src.contains(">") && (src.contains("fn ") || src.contains("struct ") || src.contains("enum ")), "泛型參數"),
            ("where clause", src.contains("where "), "where 子句"),
            ("lifetime param 'a", src.contains("'a") || src.contains("'b") || src.contains("'static"), "生命週期參數"),
        ];
        checks.into_iter().map(|(k, present, desc)| {
            let status = if present { "✅ 已出現" } else { "❌ 未出現" };
            (k.to_string(), present, format!("{} - {} - {}", status, k, desc))
        }).collect()
    }

    pub fn missing_syntax(src: &str) -> Vec<String> {
        Self::coverage_report(src).into_iter()
            .filter(|(_, present, _)| !*present)
            .map(|(k, _, desc)| format!("{}: {}", k, desc))
            .collect()
    }

    pub fn global_coverage() -> (usize, usize, f64, Vec<String>) {
        let report = Self::parser_capability();
        let total = report.len();
        let covered = report.iter().filter(|(_, p, _)| *p).count();
        let pct = if total == 0 { 100.0 } else { 100.0 * covered as f64 / total as f64 };
        let missing: Vec<String> = report.iter().filter(|(_, p, _)| !*p).map(|(k, _, d)| format!("{}: {}", k, d)).collect();
        (total, covered, pct, missing)
    }
}

impl From<FullProgram> for ProgramV2 {
    fn from(full: FullProgram) -> Self {
        let mut prog = ProgramV2::new();
        for item in full.items {
            match item {
                FullItem::Struct(s) => {
                    let fields = match s.fields {
                        StructFields::Named(nf) => nf.into_iter().map(|f| (f.name, FullType::to_v2(&f.ty))).collect(),
                        StructFields::Tuple(ts) => ts.into_iter().enumerate().map(|(i, (_, ty))| (format!("_{}", i), FullType::to_v2(&ty))).collect(),
                        StructFields::Unit => vec![],
                    };
                    let def = StructDefV2 {
                        name: s.name,
                        generics: s.generics.params.iter().filter_map(|p| match p {
                            GenericParam::Type { name, .. } => Some(name.clone()),
                            _ => None,
                        }).collect(),
                        lifetimes: s.generics.params.iter().filter_map(|p| match p {
                            GenericParam::Lifetime(lt) => Some(lt.name.clone()),
                            _ => None,
                        }).collect(),
                        fields,
                        is_pub: matches!(s.vis, Vis::Pub | Vis::PubCrate | Vis::PubSuper),
                        where_clauses: vec![],
                    };
                    prog.items.push(ItemV2::Struct(def));
                }
                FullItem::Enum(e) => {
                    let variants = e.variants.into_iter().map(|v| {
                        let fields = match v.fields {
                            StructFields::Tuple(ts) => ts.into_iter().map(|(_, ty)| FullType::to_v2(&ty)).collect(),
                            StructFields::Named(nf) => nf.into_iter().map(|f| FullType::to_v2(&f.ty)).collect(),
                            StructFields::Unit => vec![],
                        };
                        VariantV2 { name: v.name, fields, discriminant: None }
                    }).collect();
                    let def = EnumDefV2 {
                        name: e.name,
                        generics: vec![],
                        lifetimes: vec![],
                        variants,
                        is_pub: matches!(e.vis, Vis::Pub),
                        where_clauses: vec![],
                    };
                    prog.items.push(ItemV2::Enum(def));
                }
                FullItem::Fn(f) => {
                    let sig = FnSigV2 {
                        name: f.sig.name.clone(),
                        generics: vec![],
                        lifetimes: vec![],
                        params: f.sig.inputs.iter().filter_map(|inp| match inp {
                            FnInput::Typed { pat, ty } => {
                                let name = match pat {
                                    FullPat::Ident { name, .. } => name.clone(),
                                    _ => "_".to_string(),
                                };
                                Some((name, FullType::to_v2(ty)))
                            }
                            _ => None,
                        }).collect(),
                        ret: FullType::to_v2(&f.sig.output),
                        is_pub: matches!(f.vis, Vis::Pub),
                        is_unsafe: f.sig.is_unsafe,
                        is_async: f.sig.is_async,
                        is_method: false,
                        where_clauses: vec![],
                        has_default: false,
                        default_body: None,
                    };
                    let body_src = f.block.map(|b| format!("{:?}", b)).unwrap_or_default();
                    let def = FnDefV2 { sig, body_src };
                    if def.sig.name == "main" {
                        prog.main = Some(def);
                    } else {
                        prog.items.push(ItemV2::Fn(def));
                    }
                }
                FullItem::Const(c) => {
                    let def = ConstDefV2 {
                        name: c.name,
                        ty: FullType::to_v2(&c.ty),
                        expr: c.expr.map(|e| format!("{:?}", e)),
                        is_pub: matches!(c.vis, Vis::Pub | Vis::PubCrate),
                    };
                    prog.items.push(ItemV2::Const(def));
                }
                FullItem::Static(s) => {
                    let def = StaticDefV2 {
                        name: s.name,
                        ty: FullType::to_v2(&s.ty),
                        mutbl: s.mutbl,
                        expr: s.expr.map(|e| format!("{:?}", e)),
                        is_pub: matches!(s.vis, Vis::Pub | Vis::PubCrate),
                    };
                    prog.items.push(ItemV2::Static(def));
                }
                FullItem::TypeAlias(t) => {
                    let def = TypeAliasDefV2 {
                        name: t.name,
                        generics: t.generics.params.iter().filter_map(|p| match p {
                            GenericParam::Type { name, .. } => Some(name.clone()),
                            _ => None,
                        }).collect(),
                        ty: t.ty.map(|ty| FullType::to_v2(&ty)).unwrap_or(TypeV2::Base(BaseType::I32)),
                        is_pub: matches!(t.vis, Vis::Pub | Vis::PubCrate),
                    };
                    prog.items.push(ItemV2::TypeAlias(def));
                }
                _ => {}
            }
        }
        if let Some(main) = full.main {
            let sig = FnSigV2 {
                name: main.sig.name.clone(),
                generics: vec![],
                lifetimes: vec![],
                params: main.sig.inputs.iter().filter_map(|inp| match inp {
                    FnInput::Typed { pat, ty } => {
                        let name = match pat {
                            FullPat::Ident { name, .. } => name.clone(),
                            _ => "_".to_string(),
                        };
                        Some((name, FullType::to_v2(ty)))
                    }
                    _ => None,
                }).collect(),
                ret: FullType::to_v2(&main.sig.output),
                is_pub: false,
                is_unsafe: main.sig.is_unsafe,
                is_async: main.sig.is_async,
                is_method: false,
                where_clauses: vec![],
                has_default: false,
                default_body: None,
            };
            let body_src = main.block.map(|b| format!("{:?}", b)).unwrap_or_default();
            prog.main = Some(FnDefV2 { sig, body_src });
        }
        prog
    }
}

// ─────────────────────────────────────────────────────────────
// 統一 AST 額外工具：將所有型別收斂到 poly 與 code
// ─────────────────────────────────────────────────────────────

/// 統一 AST 的型別收斂統計
#[derive(Clone, Debug, Default)]
pub struct AstStats {
    pub n_base_types: usize,
    pub n_ext_types: usize,
    pub n_structs: usize,
    pub n_enums: usize,
    pub n_fns: usize,
    pub n_impls: usize,
    pub n_traits: usize,
    pub n_mods: usize,
    pub n_consts: usize,
    pub n_statics: usize,
    pub n_type_alias: usize,
    pub n_full_types: usize,
    pub n_full_pats: usize,
    pub n_full_exprs: usize,
    pub n_full_items: usize,
}

/// 從 ProgramV2 統計
pub fn collect_stats_v2(prog: &ProgramV2) -> AstStats {
    let mut stats = AstStats::default();
    stats.n_base_types = 7;
    stats.n_ext_types = prog.universe.n_ext();
    for item in &prog.items {
        match item {
            ItemV2::Struct(_) => stats.n_structs += 1,
            ItemV2::Enum(_) => stats.n_enums += 1,
            ItemV2::Union(_) => stats.n_structs += 1, // union 計入 struct 統計
            ItemV2::Fn(_) => stats.n_fns += 1,
            ItemV2::Impl(_) => stats.n_impls += 1,
            ItemV2::Trait(_) => stats.n_traits += 1,
            ItemV2::Mod(_) => stats.n_mods += 1,
            ItemV2::Const(_) => stats.n_consts += 1,
            ItemV2::Static(_) => stats.n_statics += 1,
            ItemV2::TypeAlias(_) => stats.n_type_alias += 1,
            _ => {}
        }
    }
    if prog.main.is_some() { stats.n_fns += 1; }
    stats
}

/// 從 FullProgram 統計
pub fn collect_stats_full(prog: &FullProgram) -> AstStats {
    let mut stats = AstStats::default();
    for item in &prog.items {
        match item {
            FullItem::Struct(_) => stats.n_structs += 1,
            FullItem::Enum(_) => stats.n_enums += 1,
            FullItem::Fn(_) => stats.n_fns += 1,
            FullItem::Impl(_) => stats.n_impls += 1,
            FullItem::Trait(_) => stats.n_traits += 1,
            FullItem::Mod(_) => stats.n_mods += 1,
            FullItem::Const(_) => stats.n_consts += 1,
            FullItem::Static(_) => stats.n_statics += 1,
            FullItem::TypeAlias(_) => stats.n_type_alias += 1,
            _ => stats.n_full_items += 1,
        }
    }
    if prog.main.is_some() { stats.n_fns += 1; }
    stats.n_full_items = prog.items.len();
    stats
}

/// 將 ProgramV2 轉為 poly 代碼文本（用於測試收斂）
pub fn program_v2_to_poly_code(prog: &ProgramV2) -> String {
    let mut out = String::new();
    out.push_str(&format!("# @type-universe: {}\n", prog.universe.n_types()));
    out.push_str(&format!("# @stats: structs={} enums={} fns={} consts={} statics={} type_alias={}\n",
        prog.items.iter().filter(|i| matches!(i, ItemV2::Struct(_))).count(),
        prog.items.iter().filter(|i| matches!(i, ItemV2::Enum(_))).count(),
        prog.items.iter().filter(|i| matches!(i, ItemV2::Fn(_))).count(),
        prog.items.iter().filter(|i| matches!(i, ItemV2::Const(_))).count(),
        prog.items.iter().filter(|i| matches!(i, ItemV2::Static(_))).count(),
        prog.items.iter().filter(|i| matches!(i, ItemV2::TypeAlias(_))).count(),
    ));
    for item in &prog.items {
        match item {
            ItemV2::Struct(s) => {
                out.push_str(&format!("struct {} {{\n", s.name));
                for (fname, fty) in &s.fields {
                    out.push_str(&format!("  {}: {},\n", fname, fty.name()));
                }
                out.push_str("}\n");
            }
            ItemV2::Enum(e) => {
                out.push_str(&format!("enum {} {{\n", e.name));
                for v in &e.variants {
                    if v.fields.is_empty() {
                        out.push_str(&format!("  {},\n", v.name));
                    } else {
                        let fields = v.fields.iter().map(|t| t.name()).collect::<Vec<_>>().join(", ");
                        out.push_str(&format!("  {}({}),\n", v.name, fields));
                    }
                }
                out.push_str("}\n");
            }
            ItemV2::Fn(f) => {
                out.push_str(&format!("fn {}() {{ {} }}\n", f.sig.name, f.body_src.chars().take(60).collect::<String>()));
            }
            ItemV2::Const(c) => {
                out.push_str(&format!("const {}: {} = {};\n", c.name, c.ty.name(), c.expr.as_deref().unwrap_or("0")));
            }
            ItemV2::Static(s) => {
                out.push_str(&format!("static {}{}: {} = {};\n", if s.mutbl { "mut " } else { "" }, s.name, s.ty.name(), s.expr.as_deref().unwrap_or("0")));
            }
            ItemV2::TypeAlias(t) => {
                out.push_str(&format!("type {} = {};\n", t.name, t.ty.name()));
            }
            _ => {}
        }
    }
    if let Some(main) = &prog.main {
        out.push_str(&format!("fn main() {{ {} }}\n", main.body_src.chars().take(80).collect::<String>()));
    }
    out
}

/// 將 FullProgram 轉為 poly 代碼文本
pub fn full_program_to_poly_code(prog: &FullProgram) -> String {
    let mut out = String::new();
    out.push_str("# @mode: full\n");
    for item in &prog.items {
        match item {
            FullItem::Struct(s) => {
                out.push_str(&format!("struct {} {{ /* {} fields */ }}\n", s.name, match &s.fields { StructFields::Named(nf) => nf.len(), StructFields::Tuple(ts) => ts.len(), StructFields::Unit => 0 }));
            }
            FullItem::Enum(e) => {
                out.push_str(&format!("enum {} {{ {} variants }}\n", e.name, e.variants.len()));
            }
            FullItem::Fn(f) => {
                out.push_str(&format!("fn {}() {{ /* {:?} */ }}\n", f.sig.name, f.block.as_ref().map(|b| format!("{:?}", b).chars().take(40).collect::<String>())));
            }
            FullItem::Const(c) => {
                out.push_str(&format!("const {}: {} = ...;\n", c.name, c.ty.name()));
            }
            FullItem::Static(s) => {
                out.push_str(&format!("static {}{}: {};\n", if s.mutbl { "mut " } else { "" }, s.name, s.ty.name()));
            }
            FullItem::TypeAlias(t) => {
                out.push_str(&format!("type {} = {};\n", t.name, t.ty.as_ref().map(|ty| ty.name()).unwrap_or_else(|| "_".to_string())));
            }
            FullItem::Mod(m) => {
                out.push_str(&format!("mod {} {{ {} items }}\n", m.name, m.items.as_ref().map(|i| i.len()).unwrap_or(0)));
            }
            _ => {
                out.push_str(&format!("// item {:?}\n", std::mem::discriminant(item)));
            }
        }
    }
    if let Some(main) = &prog.main {
        out.push_str(&format!("fn main() {{ /* {} */ }}\n", main.sig.name));
    }
    out
}

// ─────────────────────────────────────────────────────────────
// 窮舉檢證覆蓋率 — ast.rs 對 FullType/Pat/Expr/Item 變體的完整覆蓋檢查
// 目標：codegen 食 39 正例，吐出物比 ast.rs，ast.rs 進行窮舉檢證
// ─────────────────────────────────────────────────────────────

impl FullType {
    pub fn variant_name(&self) -> &'static str {
        match self {
            FullType::V2(_) => "V2",
            FullType::Path { .. } => "Path",
            FullType::Tuple(_) => "Tuple",
            FullType::Array { .. } => "Array",
            FullType::Slice(_) => "Slice",
            FullType::Ptr { .. } => "Ptr",
            FullType::Ref { .. } => "Ref",
            FullType::BareFn { .. } => "BareFn",
            FullType::Never => "Never",
            FullType::Inferred => "Inferred",
            FullType::TraitObject { .. } => "TraitObject",
            FullType::ImplTrait { .. } => "ImplTrait",
            FullType::Macro(_) => "Macro",
            FullType::Group(_) => "Group",
            FullType::Paren(_) => "Paren",
        }
    }
    pub fn all_variant_names() -> Vec<&'static str> {
        vec!["V2","Path","Tuple","Array","Slice","Ptr","Ref","BareFn","Never","Inferred","TraitObject","ImplTrait","Macro","Group","Paren"]
    }
    /// 判斷是否為核心需覆蓋變體（排除 Group/Paren/Macro/V2 這些內部包裝）
    pub fn core_variant_names() -> Vec<&'static str> {
        vec!["Path","Tuple","Array","Slice","Ptr","Ref","BareFn","Never","Inferred","TraitObject","ImplTrait"]
    }
}

impl FullPat {
    pub fn variant_name(&self) -> &'static str {
        match self {
            FullPat::Wild => "Wild",
            FullPat::Ident { .. } => "Ident",
            FullPat::Lit(_) => "Lit",
            FullPat::Path(_) => "Path",
            FullPat::TupleStruct { .. } => "TupleStruct",
            FullPat::Struct { .. } => "Struct",
            FullPat::Tuple(_) => "Tuple",
            FullPat::Slice(_) => "Slice",
            FullPat::Or(_) => "Or",
            FullPat::Ref { .. } => "Ref",
            FullPat::Box(_) => "Box",
            FullPat::Range { .. } => "Range",
            FullPat::Macro(_) => "Macro",
            FullPat::Type { .. } => "Type",
        }
    }
    pub fn all_variant_names() -> Vec<&'static str> {
        vec!["Wild","Ident","Lit","Path","TupleStruct","Struct","Tuple","Slice","Or","Ref","Box","Range","Macro","Type"]
    }
}

impl FullExpr {
    pub fn variant_name(&self) -> &'static str {
        match self {
            FullExpr::Lit(_) => "Lit",
            FullExpr::Path(_) => "Path",
            FullExpr::Field { .. } => "Field",
            FullExpr::Index { .. } => "Index",
            FullExpr::Call { .. } => "Call",
            FullExpr::MethodCall { .. } => "MethodCall",
            FullExpr::Unary { .. } => "Unary",
            FullExpr::Binary { .. } => "Binary",
            FullExpr::Assign { .. } => "Assign",
            FullExpr::AssignOp { .. } => "AssignOp",
            FullExpr::If { .. } => "If",
            FullExpr::Match { .. } => "Match",
            FullExpr::Loop { .. } => "Loop",
            FullExpr::While { .. } => "While",
            FullExpr::For { .. } => "For",
            FullExpr::Block { .. } => "Block",
            FullExpr::Unsafe(_) => "Unsafe",
            FullExpr::Async { .. } => "Async",
            FullExpr::Await { .. } => "Await",
            FullExpr::Closure { .. } => "Closure",
            FullExpr::Return(_) => "Return",
            FullExpr::Break { .. } => "Break",
            FullExpr::Continue(_) => "Continue",
            FullExpr::Let { .. } => "Let",
            FullExpr::StructLit { .. } => "StructLit",
            FullExpr::Array(_) => "Array",
            FullExpr::ArrayRepeat { .. } => "ArrayRepeat",
            FullExpr::Tuple(_) => "Tuple",
            FullExpr::Cast { .. } => "Cast",
            FullExpr::TypeAscribe { .. } => "TypeAscribe",
            FullExpr::Try(_) => "Try",
            FullExpr::Range { .. } => "Range",
            FullExpr::Macro(_, _) => "Macro",
            FullExpr::Verbatim(_) => "Verbatim",
        }
    }
    pub fn all_variant_names() -> Vec<&'static str> {
        vec!["Lit","Path","Field","Index","Call","MethodCall","Unary","Binary","Assign","AssignOp","If","Match","Loop","While","For","Block","Unsafe","Async","Await","Closure","Return","Break","Continue","Let","StructLit","Array","ArrayRepeat","Tuple","Cast","TypeAscribe","Try","Range","Macro","Verbatim"]
    }
}

impl FullItem {
    pub fn variant_name(&self) -> &'static str {
        match self {
            FullItem::Fn(_) => "Fn",
            FullItem::Struct(_) => "Struct",
            FullItem::Enum(_) => "Enum",
            FullItem::Union(_) => "Union",
            FullItem::Impl(_) => "Impl",
            FullItem::Trait(_) => "Trait",
            FullItem::Mod(_) => "Mod",
            FullItem::Use(_) => "Use",
            FullItem::Const(_) => "Const",
            FullItem::Static(_) => "Static",
            FullItem::TypeAlias(_) => "TypeAlias",
            FullItem::Macro(_) => "Macro",
            FullItem::ExternCrate(_) => "ExternCrate",
            FullItem::ExternBlock(_) => "ExternBlock",
        }
    }
    pub fn all_variant_names() -> Vec<&'static str> {
        vec!["Fn","Struct","Enum","Union","Impl","Trait","Mod","Use","Const","Static","TypeAlias","Macro","ExternCrate","ExternBlock"]
    }
}

impl ItemV2 {
    pub fn variant_name(&self) -> &'static str {
        match self {
            ItemV2::Struct(_) => "Struct",
            ItemV2::Enum(_) => "Enum",
            ItemV2::Union(_) => "Union",
            ItemV2::Fn(_) => "Fn",
            ItemV2::Impl(_) => "Impl",
            ItemV2::Trait(_) => "Trait",
            ItemV2::Mod(_) => "Mod",
            ItemV2::Const(_) => "Const",
            ItemV2::Static(_) => "Static",
            ItemV2::TypeAlias(_) => "TypeAlias",
            ItemV2::Use(_) => "Use",
            ItemV2::Macro(_) => "Macro",
        }
    }
    pub fn all_variant_names() -> Vec<&'static str> {
        vec!["Struct","Enum","Union","Fn","Impl","Trait","Mod","Const","Static","TypeAlias","Use","Macro"]
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExhaustiveCoverage {
    pub type_total: usize,
    pub type_covered: usize,
    pub type_pct: f64,
    pub type_missing: Vec<String>,
    pub type_covered_names: Vec<String>,
    pub pat_total: usize,
    pub pat_covered: usize,
    pub pat_pct: f64,
    pub pat_missing: Vec<String>,
    pub pat_covered_names: Vec<String>,
    pub expr_total: usize,
    pub expr_covered: usize,
    pub expr_pct: f64,
    pub expr_missing: Vec<String>,
    pub expr_covered_names: Vec<String>,
    pub item_total: usize,
    pub item_covered: usize,
    pub item_pct: f64,
    pub item_missing: Vec<String>,
    pub item_covered_names: Vec<String>,
    pub item_v2_total: usize,
    pub item_v2_covered: usize,
    pub item_v2_pct: f64,
    pub item_v2_missing: Vec<String>,
    pub overall_pct: f64,
}

impl ExhaustiveCoverage {
    pub fn is_full(&self) -> bool {
        self.type_missing.is_empty() && self.pat_missing.is_empty() && self.expr_missing.is_empty()
    }
    pub fn display(&self) -> String {
        let mut s = String::new();
        s.push_str("=== AST 窮舉覆蓋率報告 ===\n");
        s.push_str(&format!("FullType: {}/{} {:.1}% covered={:?} missing={:?}\n", self.type_covered, self.type_total, self.type_pct, self.type_covered_names, self.type_missing));
        s.push_str(&format!("FullPat: {}/{} {:.1}% covered={:?} missing={:?}\n", self.pat_covered, self.pat_total, self.pat_pct, self.pat_covered_names, self.pat_missing));
        s.push_str(&format!("FullExpr: {}/{} {:.1}% covered={:?} missing={:?}\n", self.expr_covered, self.expr_total, self.expr_pct, self.expr_covered_names, self.expr_missing));
        s.push_str(&format!("FullItem: {}/{} {:.1}% covered={:?} missing={:?}\n", self.item_covered, self.item_total, self.item_pct, self.item_covered_names, self.item_missing));
        s.push_str(&format!("ItemV2: {}/{} {:.1}% missing={:?}\n", self.item_v2_covered, self.item_v2_total, self.item_v2_pct, self.item_v2_missing));
        s.push_str(&format!("Overall: {:.1}%\n", self.overall_pct));
        s
    }
}

/// 對給定的已解析例子計算窮舉覆蓋率
pub fn exhaustive_coverage_report(
    type_examples: &[FullType],
    pat_examples: &[FullPat],
    expr_examples: &[FullExpr],
    item_examples: &[FullItem],
    item_v2_examples: &[ItemV2],
) -> ExhaustiveCoverage {
    use std::collections::HashSet;
    let mut rep = ExhaustiveCoverage::default();

    // Type
    let all_type = FullType::all_variant_names();
    rep.type_total = all_type.len();
    let mut covered_type: HashSet<String> = HashSet::new();
    for t in type_examples {
        covered_type.insert(t.variant_name().to_string());
        // 遞歸收集子類型變體
        collect_full_type_variants(t, &mut covered_type);
    }
    rep.type_covered_names = covered_type.iter().cloned().collect();
    rep.type_covered_names.sort();
    rep.type_missing = all_type.into_iter().filter(|v| !covered_type.contains(*v)).map(|s| s.to_string()).collect();
    rep.type_covered = rep.type_covered_names.len();
    rep.type_pct = if rep.type_total==0 {100.0} else {100.0 * rep.type_covered as f64 / rep.type_total as f64};

    // Pat
    let all_pat = FullPat::all_variant_names();
    rep.pat_total = all_pat.len();
    let mut covered_pat: HashSet<String> = HashSet::new();
    for p in pat_examples {
        collect_full_pat_variants(p, &mut covered_pat);
    }
    rep.pat_covered_names = covered_pat.iter().cloned().collect();
    rep.pat_covered_names.sort();
    rep.pat_missing = all_pat.into_iter().filter(|v| !covered_pat.contains(*v)).map(|s| s.to_string()).collect();
    rep.pat_covered = rep.pat_covered_names.len();
    rep.pat_pct = if rep.pat_total==0 {100.0} else {100.0 * rep.pat_covered as f64 / rep.pat_total as f64};

    // Expr
    let all_expr = FullExpr::all_variant_names();
    rep.expr_total = all_expr.len();
    let mut covered_expr: HashSet<String> = HashSet::new();
    for e in expr_examples {
        collect_full_expr_variants(e, &mut covered_expr);
    }
    rep.expr_covered_names = covered_expr.iter().cloned().collect();
    rep.expr_covered_names.sort();
    rep.expr_missing = all_expr.into_iter().filter(|v| !covered_expr.contains(*v)).map(|s| s.to_string()).collect();
    rep.expr_covered = rep.expr_covered_names.len();
    rep.expr_pct = if rep.expr_total==0 {100.0} else {100.0 * rep.expr_covered as f64 / rep.expr_total as f64};

    // Item
    let all_item = FullItem::all_variant_names();
    rep.item_total = all_item.len();
    let mut covered_item: HashSet<String> = HashSet::new();
    for it in item_examples {
        covered_item.insert(it.variant_name().to_string());
    }
    rep.item_covered_names = covered_item.iter().cloned().collect();
    rep.item_covered_names.sort();
    rep.item_missing = all_item.into_iter().filter(|v| !covered_item.contains(*v)).map(|s| s.to_string()).collect();
    rep.item_covered = rep.item_covered_names.len();
    rep.item_pct = if rep.item_total==0 {100.0} else {100.0 * rep.item_covered as f64 / rep.item_total as f64};

    // ItemV2
    let all_item_v2 = ItemV2::all_variant_names();
    rep.item_v2_total = all_item_v2.len();
    let mut covered_v2: HashSet<String> = HashSet::new();
    for it in item_v2_examples {
        covered_v2.insert(it.variant_name().to_string());
    }
    rep.item_v2_missing = all_item_v2.into_iter().filter(|v| !covered_v2.contains(*v)).map(|s| s.to_string()).collect();
    rep.item_v2_covered = covered_v2.len();
    rep.item_v2_pct = if rep.item_v2_total==0 {100.0} else {100.0 * rep.item_v2_covered as f64 / rep.item_v2_total as f64};

    let total_all = rep.type_total + rep.pat_total + rep.expr_total + rep.item_total + rep.item_v2_total;
    let covered_all = rep.type_covered + rep.pat_covered + rep.expr_covered + rep.item_covered + rep.item_v2_covered;
    rep.overall_pct = if total_all==0 {100.0} else {100.0 * covered_all as f64 / total_all as f64};
    rep
}

fn collect_full_type_variants(ty: &FullType, set: &mut std::collections::HashSet<String>) {
    set.insert(ty.variant_name().to_string());
    match ty {
        FullType::Path { args, .. } => { for a in args { collect_full_type_variants(a, set); } }
        FullType::Tuple(ts) => { for t in ts { collect_full_type_variants(t, set); } }
        FullType::Array { elem, .. } => { collect_full_type_variants(elem, set); }
        FullType::Slice(e) => { collect_full_type_variants(e, set); }
        FullType::Ptr { inner, .. } => { collect_full_type_variants(inner, set); }
        FullType::Ref { inner, .. } => { collect_full_type_variants(inner, set); }
        FullType::BareFn { params, ret, .. } => { for p in params { collect_full_type_variants(p, set); } collect_full_type_variants(ret, set); }
        FullType::Group(inner) | FullType::Paren(inner) => { collect_full_type_variants(inner, set); }
        _ => {}
    }
}

fn collect_full_pat_variants(pat: &FullPat, set: &mut std::collections::HashSet<String>) {
    set.insert(pat.variant_name().to_string());
    let mut _dummy_ty = std::collections::HashSet::new();
    match pat {
        FullPat::Ident { subpat, .. } => { if let Some(sp) = subpat { collect_full_pat_variants(sp, set); } }
        FullPat::TupleStruct { elems, .. } => { for e in elems { collect_full_pat_variants(e, set); } }
        FullPat::Struct { fields, .. } => { for (_, f) in fields { collect_full_pat_variants(f, set); } }
        FullPat::Tuple(ps) | FullPat::Slice(ps) | FullPat::Or(ps) => { for p in ps { collect_full_pat_variants(p, set); } }
        FullPat::Ref { inner, .. } => { collect_full_pat_variants(inner, set); }
        FullPat::Box(inner) => { collect_full_pat_variants(inner, set); }
        FullPat::Type { pat, ty } => { collect_full_pat_variants(pat, set); collect_full_type_variants(ty, &mut _dummy_ty); }
        _ => {}
    }
}

fn collect_full_expr_variants(expr: &FullExpr, set: &mut std::collections::HashSet<String>) {
    set.insert(expr.variant_name().to_string());
    // 輔助：收集 Pat/Type 變體到臨時集合，不污染 Expr 集合
    let mut _dummy_pat = std::collections::HashSet::new();
    let mut _dummy_ty = std::collections::HashSet::new();
    match expr {
        FullExpr::Field { base, .. } => { collect_full_expr_variants(base, set); }
        FullExpr::Index { base, index } => { collect_full_expr_variants(base, set); collect_full_expr_variants(index, set); }
        FullExpr::Call { func, args } => { collect_full_expr_variants(func, set); for a in args { collect_full_expr_variants(a, set); } }
        FullExpr::MethodCall { receiver, args, .. } => { collect_full_expr_variants(receiver, set); for a in args { collect_full_expr_variants(a, set); } }
        FullExpr::Unary { expr, .. } => { collect_full_expr_variants(expr, set); }
        FullExpr::Binary { left, right, .. } => { collect_full_expr_variants(left, set); collect_full_expr_variants(right, set); }
        FullExpr::Assign { left, right } => { collect_full_expr_variants(left, set); collect_full_expr_variants(right, set); }
        FullExpr::AssignOp { left, right, .. } => { collect_full_expr_variants(left, set); collect_full_expr_variants(right, set); }
        FullExpr::If { cond, then_branch, else_branch } => { collect_full_expr_variants(cond, set); collect_full_expr_variants(then_branch, set); if let Some(e) = else_branch { collect_full_expr_variants(e, set); } }
        FullExpr::Match { scrutinee, arms } => { collect_full_expr_variants(scrutinee, set); for arm in arms { collect_full_pat_variants(&arm.pat, &mut _dummy_pat); collect_full_expr_variants(&arm.body, set); if let Some(g) = &arm.guard { collect_full_expr_variants(g, set); } } }
        FullExpr::Loop { body, .. } => { collect_full_expr_variants(body, set); }
        FullExpr::While { cond, body, .. } => { collect_full_expr_variants(cond, set); collect_full_expr_variants(body, set); }
        FullExpr::For { pat, iter, body, .. } => { collect_full_pat_variants(pat, &mut _dummy_pat); collect_full_expr_variants(iter, set); collect_full_expr_variants(body, set); }
        FullExpr::Block { stmts, .. } => { for s in stmts { match s { FullStmt::Local { pat, ty, init, .. } => { collect_full_pat_variants(pat, &mut _dummy_pat); if let Some(t) = ty { collect_full_type_variants(t, &mut _dummy_ty); } if let Some(i) = init { collect_full_expr_variants(i, set); } } FullStmt::Item(_it) => { /* Item 變體不計入 Expr */ } FullStmt::Expr(e) | FullStmt::Semi(e) => { collect_full_expr_variants(e, set); } _ => {} } } }
        FullExpr::Unsafe(inner) => { collect_full_expr_variants(inner, set); }
        FullExpr::Async { block, .. } => { collect_full_expr_variants(block, set); }
        FullExpr::Await { base } => { collect_full_expr_variants(base, set); }
        FullExpr::Closure { body, .. } => { collect_full_expr_variants(body, set); }
        FullExpr::Return(Some(e)) => { collect_full_expr_variants(e, set); }
        FullExpr::Break { expr: Some(e), .. } => { collect_full_expr_variants(e, set); }
        FullExpr::Let { pat, expr } => { collect_full_pat_variants(pat, &mut _dummy_pat); collect_full_expr_variants(expr, set); }
        FullExpr::StructLit { fields, rest, .. } => { for (_, f) in fields { collect_full_expr_variants(f, set); } if let Some(r) = rest { collect_full_expr_variants(r, set); } }
        FullExpr::Array(es) => { for e in es { collect_full_expr_variants(e, set); } }
        FullExpr::ArrayRepeat { elem, len } => { collect_full_expr_variants(elem, set); collect_full_expr_variants(len, set); }
        FullExpr::Tuple(es) => { for e in es { collect_full_expr_variants(e, set); } }
        FullExpr::Cast { expr, ty } => { collect_full_expr_variants(expr, set); collect_full_type_variants(ty, &mut _dummy_ty); }
        FullExpr::TypeAscribe { expr, ty } => { collect_full_expr_variants(expr, set); collect_full_type_variants(ty, &mut _dummy_ty); }
        FullExpr::Try(e) => { collect_full_expr_variants(e, set); }
        FullExpr::Range { start, end, .. } => { if let Some(s) = start { collect_full_expr_variants(s, set); } if let Some(e) = end { collect_full_expr_variants(e, set); } }
        _ => {}
    }
}

/// 從 parse_full 的 39 正例直接計算覆蓋率（便捷接口，ast.rs 內部窮舉）
pub fn exhaustive_coverage_from_39_examples() -> ExhaustiveCoverage {
    // 為避免循環依賴，此函數邏輯在 parse_full.rs 中實現，此處提供空實現占位
    // 實際覆蓋率計算由 parse_full::exhaustive_coverage_check() 調用本模塊的 report 函數
    ExhaustiveCoverage::default()
}

// ─────────────────────────────────────────────────────────────
// lower.rs 喂比 codegen 後吐出比 ast.rs — Lowering 窮舉覆蓋率
// ─────────────────────────────────────────────────────────────

/// Lowering 階段的變體覆蓋率（對應 lower.rs 中的各 lowering 分支）
#[derive(Clone, Debug, Default)]
pub struct LoweringCoverage {
    pub products_total: usize,
    pub products_covered: usize,
    pub sums_total: usize,
    pub sums_covered: usize,
    pub mod_flatten_total: usize,
    pub mod_flatten_covered: usize,
    pub async_total: usize,
    pub async_covered: usize,
    pub match_total: usize,
    pub match_covered: usize,
    pub for_total: usize,
    pub for_covered: usize,
    pub trait_impl_total: usize,
    pub trait_impl_covered: usize,
    pub lifetime_total: usize,
    pub lifetime_covered: usize,
    pub stdlib_total: usize,
    pub stdlib_covered: usize,
    pub overall_pct: f64,
    pub details: String,
}

impl LoweringCoverage {
    pub fn display(&self) -> String {
        format!(
            "=== Lowering Coverage (ast.rs 對 lower.rs 的窮舉) ===\n\
             products: {}/{}, sums: {}/{}, mod_flatten: {}/{}, async: {}/{}, match: {}/{}, for: {}/{}, trait_impl: {}/{}, lifetime: {}/{}, stdlib: {}/{}\n\
             overall: {:.1}%\n\
             details: {}\n",
            self.products_covered, self.products_total,
            self.sums_covered, self.sums_total,
            self.mod_flatten_covered, self.mod_flatten_total,
            self.async_covered, self.async_total,
            self.match_covered, self.match_total,
            self.for_covered, self.for_total,
            self.trait_impl_covered, self.trait_impl_total,
            self.lifetime_covered, self.lifetime_total,
            self.stdlib_covered, self.stdlib_total,
            self.overall_pct, self.details
        )
    }
    pub fn is_full(&self) -> bool {
        self.overall_pct >= 99.0
    }
}

/// 從 lower.rs 的 Lowered 計算窮舉覆蓋率（ast.rs 視角）
pub fn lowering_exhaustive_report(
    products: usize,
    sums: usize,
    mod_map: usize,
    generated: usize,
    match_ok: bool,
    for_ok: bool,
    trait_ok: bool,
    lifetime_ok: bool,
    stdlib_ok: bool,
) -> LoweringCoverage {
    let mut cov = LoweringCoverage::default();
    cov.products_total = 1;
    cov.products_covered = if products > 0 { 1 } else { 0 };
    cov.sums_total = 1;
    cov.sums_covered = if sums > 0 { 1 } else { 0 };
    cov.mod_flatten_total = 1;
    cov.mod_flatten_covered = if mod_map > 0 { 1 } else { 0 };
    cov.async_total = 1;
    cov.async_covered = if generated > 0 { 1 } else { 0 };
    cov.match_total = 1;
    cov.match_covered = if match_ok { 1 } else { 0 };
    cov.for_total = 1;
    cov.for_covered = if for_ok { 1 } else { 0 };
    cov.trait_impl_total = 1;
    cov.trait_impl_covered = if trait_ok { 1 } else { 0 };
    cov.lifetime_total = 1;
    cov.lifetime_covered = if lifetime_ok { 1 } else { 0 };
    cov.stdlib_total = 1;
    cov.stdlib_covered = if stdlib_ok { 1 } else { 0 };

    let total = cov.products_total + cov.sums_total + cov.mod_flatten_total + cov.async_total + cov.match_total + cov.for_total + cov.trait_impl_total + cov.lifetime_total + cov.stdlib_total;
    let covered = cov.products_covered + cov.sums_covered + cov.mod_flatten_covered + cov.async_covered + cov.match_covered + cov.for_covered + cov.trait_impl_covered + cov.lifetime_covered + cov.stdlib_covered;
    cov.overall_pct = if total==0 {100.0} else {100.0 * covered as f64 / total as f64};
    cov.details = format!("products={}, sums={}, mod_map={}, generated={}, match={}, for={}, trait={}, lifetime={}, stdlib={}", products, sums, mod_map, generated, match_ok, for_ok, trait_ok, lifetime_ok, stdlib_ok);
    cov
}

/// 便捷：對 lower.rs 的示例 ProgramV2 進行窮舉
pub fn lowering_coverage_from_sample() -> LoweringCoverage {
    // 此函數的實際邏輯在 codegen::lowering_coverage_check 中，此處提供與 ast.rs 統一的接口
    // 為避免循環依賴，返回默認，實際檢查由 parse_full 或 codegen 調用
    LoweringCoverage::default()
}

// ─────────────────────────────────────────────────────────────
// 全 AST 列表 — 把 ast.rs 列入全 AST 那個列表內
// 包含 ast.rs, ast_v2.rs, ast_full.rs, lower.rs, universe.rs 等
// ─────────────────────────────────────────────────────────────

/// 全 AST 文件清單 — AST_SYNTAX_INVENTORY（把 ast.rs 列入）
/// 要求：全 AST 列表包含 ast.rs 自身
pub const AST_SYNTAX_INVENTORY: &[(&str, &str, &str)] = &[
    ("ast.rs", "統一 AST 單一來源 — Type/BinOp/EKind/ItemV2/FullType/FullPat/FullExpr/FullItem", "core/src/minirust/ast.rs"),
    ("ast_v2.rs", "兼容 shim re-export ast.rs V2", "core/src/minirust/ast_v2.rs"),
    ("ast_full.rs", "兼容 shim re-export ast.rs Full", "core/src/minirust/ast_full.rs"),
    ("universe.rs", "TypeV2/BaseType/ExtType/Universe", "core/src/minirust/universe.rs"),
    ("lower.rs", "Lowering: Product/Sum/Mod/Async/Match/For", "core/src/minirust/lower.rs"),
    ("parse.rs", "原始 Program 解析 (7 型別) — Parser::parse_program", "core/src/minirust/parse.rs"),
    ("parse_v2.rs", "ProgramV2 解析 (struct/enum/fn/impl/trait/mod/const/static/type)", "core/src/minirust/parse_v2.rs"),
    ("parse_pat.rs", "FullPat 解析", "core/src/minirust/parse_pat.rs"),
    ("parse_expr.rs", "FullExpr 解析", "core/src/minirust/parse_expr.rs"),
    ("parse_full.rs", "FullType 解析 + 39 正例 + poly 收斂 + 4 parse 比較", "core/src/minirust/parse_full.rs"),
];

/// 全 AST 文件清單（把 ast.rs 列入）— 兼容函數，返回 AST_SYNTAX_INVENTORY 的 Vec 形式
/// 優化：使用靜態切片引用，避免每次 to_vec clone，若需擁有則調用 full_ast_file_list_owned
pub fn full_ast_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    AST_SYNTAX_INVENTORY.to_vec()
}
/// 零分配版本：返回靜態切片引用，供 pipeline_v2 等高頻調用實際使用
pub fn full_ast_file_list_static() -> &'static [(&'static str, &'static str, &'static str)] {
    AST_SYNTAX_INVENTORY
}
/// 實際使用：供 pipeline_v2 消費的文件列表 JSON/文本摘要
pub fn full_ast_file_list_for_pipeline() -> Vec<String> {
    let mut out = Vec::with_capacity(AST_SYNTAX_INVENTORY.len());
    for (name, desc, path) in AST_SYNTAX_INVENTORY {
        // 預分配：name + desc + path 平均 80 字節
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

/// 全 AST 變體清單（把 ast.rs 列入，含所有 enum 變體）— 優化版：預分配容量，減少 clone
pub fn full_ast_variant_inventory() -> Vec<(String, usize, Vec<String>)> {
    let mut inventory = Vec::with_capacity(18);
    // 輔助：將 &'static [&'static str] 轉 Vec<String> 但預分配
    fn to_owned(v: &[&str]) -> Vec<String> {
        let mut out = Vec::with_capacity(v.len());
        for s in v { out.push(s.to_string()); }
        out
    }

    // ast.rs 原始
    inventory.push(("ast.rs::Type".to_string(), 7, to_owned(&["I32","Bool","Unit","RefI32","RefMutI32","RefBool","RefMutBool"])));
    inventory.push(("ast.rs::BinOp".to_string(), 9, to_owned(&["Add","Sub","Mul","Lt","Le","Ge","Eq","Ne","And"])));
    inventory.push(("ast.rs::EKind".to_string(), 17, to_owned(&["Int","BoolV","UnitLit","Var","Let","Seq","BinOp","Not","Neg","If","Ref","RefMut","Deref","AssignVar","AssignDeref","Call","Invoke"])));

    // ast.rs v2 — 實際使用 all_variant_names 已是預分配
    inventory.push(("ast.rs::ItemV2".to_string(), ItemV2::all_variant_names().len(), ItemV2::all_variant_names().into_iter().map(|s| s.to_string()).collect()));
    inventory.push(("ast.rs::StructDefV2".to_string(), 1, vec!["StructDefV2".to_string()]));
    inventory.push(("ast.rs::EnumDefV2".to_string(), 1, vec!["EnumDefV2".to_string()]));
    inventory.push(("ast.rs::FnSigV2".to_string(), 1, vec!["FnSigV2".to_string()]));

    // ast.rs full
    inventory.push(("ast.rs::FullType".to_string(), FullType::all_variant_names().len(), FullType::all_variant_names().into_iter().map(|s| s.to_string()).collect()));
    inventory.push(("ast.rs::FullPat".to_string(), FullPat::all_variant_names().len(), FullPat::all_variant_names().into_iter().map(|s| s.to_string()).collect()));
    inventory.push(("ast.rs::FullExpr".to_string(), FullExpr::all_variant_names().len(), FullExpr::all_variant_names().into_iter().map(|s| s.to_string()).collect()));
    inventory.push(("ast.rs::FullItem".to_string(), FullItem::all_variant_names().len(), FullItem::all_variant_names().into_iter().map(|s| s.to_string()).collect()));
    inventory.push(("ast.rs::Vis".to_string(), 6, to_owned(&["Private","Pub","PubCrate","PubSuper","PubSelf","PubIn"])));
    inventory.push(("ast.rs::StructFields".to_string(), 3, to_owned(&["Unit","Tuple","Named"])));

    // universe.rs
    inventory.push(("universe.rs::BaseType".to_string(), 7, to_owned(&["I32","Bool","Unit","RefI32","RefMutI32","RefBool","RefMutBool"])));
    inventory.push(("universe.rs::ExtType".to_string(), 20, to_owned(&["Vec","String","HashMap","Struct","Enum","RawPtr","Future","Option","Result","RefExt","GenericParam","Tuple","Array","Slice","BareFn","TraitObject","ImplTrait","Never","Inferred","Assoc"])));

    // lower.rs
    inventory.push(("lower.rs::Lowered".to_string(), 4, to_owned(&["products","sums","generated","mod_map"])));
    inventory.push(("lower.rs::MatchDecisionTree".to_string(), 3, to_owned(&["scrutinee","arms","exhaustive"])));
    inventory.push(("lower.rs::ForLoop".to_string(), 4, to_owned(&["pat","iter","body","invariant"])));

    inventory
}
/// 實際使用：變體清單摘要，預分配 String 提升性能
pub fn full_ast_variant_summary() -> String {
    let inventory = full_ast_variant_inventory();
    let mut total = 0usize;
    for (_, c, _) in &inventory { total += *c; }
    let mut out = String::with_capacity(2048 + total * 12);
    out.push_str("=== Full AST Variant Inventory (actual use) ===\n");
    for (name, count, variants) in &inventory {
        out.push_str(name);
        out.push_str(": ");
        out.push_str(&count.to_string());
        out.push_str(" -> ");
        out.push_str(&variants.join(", "));
        out.push('\n');
    }
    out.push_str(&format!("Total variants: {}\n", total));
    out
}

/// 全 AST 匯總文本（把 ast.rs 列入）— 優化版：with_capacity 預分配
pub fn full_ast_with_ast_rs_included_summary() -> String {
    let files = full_ast_file_list_static();
    let inventory = full_ast_variant_inventory();
    let mut total_variants = 0usize;
    for (_, c, _) in &inventory { total_variants += *c; }
    // 預估：每文件 80 字節 + 每變體 12 字節 + 固定頭
    let mut out = String::with_capacity(1024 + files.len()*100 + inventory.len()*64 + total_variants*8);
    out.push_str("=== 全 AST 列表 (把 ast.rs 列入) ===\n");
    out.push_str(&format!("文件數: {}\n", files.len()));
    for (name, desc, path) in files {
        out.push_str("- ");
        out.push_str(name);
        out.push_str(": ");
        out.push_str(desc);
        out.push_str(" (");
        out.push_str(path);
        out.push_str(")\n");
    }
    out.push_str("\n=== 變體清單 ===\n");
    for (enum_name, count, variants) in &inventory {
        out.push_str(enum_name);
        out.push_str(": ");
        out.push_str(&count.to_string());
        out.push_str(" variants ");
        // 避免 Debug 巨量分配，手動 join
        out.push_str(&format!("{:?}", variants));
        out.push('\n');
    }
    out.push_str(&format!("\n總變體數: {}\n", total_variants));
    out.push_str("\n=== ast.rs 自身覆蓋 ===\n");
    out.push_str("ast.rs 已包含: Type/BinOp/EKind/Program + ItemV2/StructDefV2/EnumDefV2/ProgramV2 + FullType/FullPat/FullExpr/FullItem/FullProgram + LoweringCoverage\n");
    out.push_str("ast_v2.rs 與 ast_full.rs 為兼容 shim，re-export super::ast\n");
    out
}
/// 實際使用 API：供 pipeline_v2 消費的 summary + file_list + variant_inventory JSON-like
pub fn ast_inventory_for_pipeline_v2() -> String {
    let files = full_ast_file_list_static();
    let inventory = full_ast_variant_inventory();
    let mut out = String::with_capacity(4096);
    out.push_str("{\"ast_syntax_inventory\":[");
    for (i, (name, desc, path)) in files.iter().enumerate() {
        if i>0 { out.push(','); }
        out.push_str(&format!("{{\"name\":\"{}\",\"desc\":\"{}\",\"path\":\"{}\"}}", name, desc.replace('\"', "'"), path));
    }
    out.push_str("],\"variant_inventory\":[");
    for (i, (ename, count, vars)) in inventory.iter().enumerate() {
        if i>0 { out.push(','); }
        out.push_str(&format!("{{\"enum\":\"{}\",\"count\":{},\"variants\":[{}]}}", ename, count, vars.iter().map(|v| format!("\"{}\"", v)).collect::<Vec<_>>().join(",")));
    }
    out.push_str("]}");
    out
}
