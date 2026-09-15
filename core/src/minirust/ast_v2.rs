//! Phase1 — 擴展 AST v2：支持 struct/enum/impl/trait 等頂層項
//! 本文件定義 Surface AST，後續 lowering 至 core Program

use super::universe::{TypeV2, parse_type_v2};

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

/// 頂層項 — Path C 擴展 const/static/type
#[derive(Clone, Debug)]
pub enum ItemV2 {
    Struct(StructDefV2),
    Enum(EnumDefV2),
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
            // 支持 pub struct / pub enum / pub mod 等 — Path C 增加 const/static/type
            let is_pub_struct = line.starts_with("struct ") || line.starts_with("pub struct ") || line.starts_with("pub(crate) struct ");
            let is_pub_enum = line.starts_with("enum ") || line.starts_with("pub enum ") || line.starts_with("pub(crate) enum ");
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
                prog.universe.insert_closure(super::universe::TypeV2::Ext(super::universe::ExtType::Struct { name: def.name.clone(), args: vec![] }));
                prog.items.push(ItemV2::Struct(def));
            } else if is_pub_enum {
                let def = Self::parse_enum(&lines, &mut i)?;
                for v in &def.variants {
                    for f in &v.fields {
                        prog.universe.insert_closure(f.clone());
                    }
                }
                prog.universe.insert_closure(super::universe::TypeV2::Ext(super::universe::ExtType::Enum { name: def.name.clone(), args: vec![] }));
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

    fn parse_struct(lines: &[&str], idx: &mut usize) -> Result<StructDefV2, String> {
        let line = lines[*idx].trim();
        // struct Point { x: i32, y: i32 } 或多行
        let is_pub = line.starts_with("pub ");
        let rest = if is_pub { line[4..].trim() } else { line };
        let after_struct = rest[7..].trim(); // after "struct "
        let name_end = after_struct.find(|c: char| c == '{' || c == '<' || c.is_whitespace()).unwrap_or(after_struct.len());
        let name = after_struct[..name_end].trim().to_string();
        // 收集字段，直到 }
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
        // 簡單解析字段：x: i32, y: i32
        if let Some(start) = collected.find('{') {
            if let Some(end) = collected.rfind('}') {
                let inner = &collected[start+1..end];
                for part in inner.split(',') {
                    let part = part.trim();
                    if part.is_empty() { continue; }
                    if let Some(colon) = part.find(':') {
                        let fname = part[..colon].trim().trim_start_matches("pub ").to_string();
                        let fty_str = part[colon+1..].trim();
                        let fty = parse_type_v2(fty_str).unwrap_or(super::universe::TypeV2::Base(super::universe::BaseType::I32));
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
                    // Some(T) 或 None
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
        // 簡化解析簽名
        let first_line = collected.lines().next().unwrap_or("").trim();
        let is_pub = first_line.contains("pub ");
        let is_async = first_line.contains("async ");
        let is_unsafe = first_line.contains("unsafe ");
        let fn_pos = first_line.find("fn ").unwrap_or(0);
        let after_fn = &first_line[fn_pos+3..];
        let name_end = after_fn.find(|c: char| c == '(' || c == '<').unwrap_or(after_fn.len());
        let name = after_fn[..name_end].trim().to_string();
        // params 與 ret 簡化為 i32
        let sig = FnSigV2 {
            name,
            generics: vec![],
            lifetimes: vec![],
            params: vec![],
            ret: super::universe::TypeV2::Base(super::universe::BaseType::Unit),
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
        // impl Point { ... } 或 impl Display for Point
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
        let self_ty = parse_type_v2(self_ty_str).unwrap_or(super::universe::TypeV2::Base(super::universe::BaseType::I32));
        // 收集方法
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
        // 簡化：從 collected 中提取 fn
        for l in collected.lines() {
            let lt = l.trim();
            if lt.starts_with("fn ") {
                // 占位
                let sig = FnSigV2 {
                    name: "method".to_string(),
                    generics: vec![],
                    lifetimes: vec![],
                    params: vec![],
                    ret: super::universe::TypeV2::Base(super::universe::BaseType::Unit),
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
            // 收集內部文本
            let mut inner_lines: Vec<String> = vec![];
            // 處理同一行 '{' 之後的部分
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
                // 若已閉合，檢查是否有 '}' 前的內容
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
            // 遞歸解析內部項（簡化：逐行嘗試 struct/enum/fn/mod）
            if !inner_lines.is_empty() {
                let inner_src = inner_lines.join("\n");
                if let Ok(inner_prog) = ProgramV2::parse_v2(&inner_src) {
                    items.extend(inner_prog.items);
                } else {
                    // 降級：手動解析簡單情況
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
        // const NAME: TY = EXPR;
        let after_const = rest[6..].trim(); // after "const "
        let name_end = after_const.find(|c: char| c == ':' || c == '=' || c.is_whitespace()).unwrap_or(after_const.len());
        let name = after_const[..name_end].trim().to_string();
        let mut ty = super::universe::TypeV2::Base(super::universe::BaseType::I32);
        let mut expr = None;
        if let Some(colon) = after_const.find(':') {
            let after_colon = after_const[colon+1..].trim();
            let eq_pos = after_colon.find('=');
            let ty_str = if let Some(eq) = eq_pos { after_colon[..eq].trim() } else { after_colon.trim().trim_end_matches(';') };
            ty = parse_type_v2(ty_str).unwrap_or(super::universe::TypeV2::Base(super::universe::BaseType::I32));
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
        let mut ty = super::universe::TypeV2::Base(super::universe::BaseType::I32);
        let mut expr = None;
        if let Some(colon) = rest.find(':') {
            let after_colon = rest[colon+1..].trim();
            let eq_pos = after_colon.find('=');
            let ty_str = if let Some(eq) = eq_pos { after_colon[..eq].trim() } else { after_colon.trim().trim_end_matches(';') };
            ty = parse_type_v2(ty_str).unwrap_or(super::universe::TypeV2::Base(super::universe::BaseType::I32));
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
        let after_type = rest[5..].trim(); // after "type "
        let name_end = after_type.find(|c: char| c == '<' || c == '=' || c.is_whitespace()).unwrap_or(after_type.len());
        let name = after_type[..name_end].trim().to_string();
        let mut generics = vec![];
        // 泛型 <T, U>
        if let Some(lt) = after_type.find('<') {
            if let Some(gt) = after_type.find('>') {
                let gen_str = &after_type[lt+1..gt];
                generics = gen_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
        }
        let mut ty = super::universe::TypeV2::Base(super::universe::BaseType::I32);
        if let Some(eq) = after_type.find('=') {
            let ty_str = after_type[eq+1..].trim().trim_end_matches(';').to_string();
            ty = parse_type_v2(&ty_str).unwrap_or(super::universe::TypeV2::Base(super::universe::BaseType::I32));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_struct() {
        let src = "struct Point { x: i32, y: i32 } fn main() {}";
        let prog = ProgramV2::parse_v2(src).unwrap();
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            ItemV2::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("not struct"),
        }
        assert!(prog.universe.n_types() >= 7);
    }

    #[test]
    fn test_parse_enum() {
        let src = "enum Option<T> { Some(T), None } fn main() {}";
        let prog = ProgramV2::parse_v2(src).unwrap();
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_parse_full() {
        let src = r#"
            struct Point { x: i32, y: i32 }
            enum Option<T> { Some(T), None }
            trait Display { fn fmt(&self) -> String; }
            impl Point { fn new(x: i32, y: i32) -> Point { Point { x: x, y: y } } }
            fn main() { let p = Point { x: 3, y: 4 }; }
        "#;
        let prog = ProgramV2::parse_v2(src).unwrap();
        println!("{}", prog.display());
        assert!(prog.universe.n_types() > 7);
        assert!(prog.universe.n_ext() >= 2);
    }
}
