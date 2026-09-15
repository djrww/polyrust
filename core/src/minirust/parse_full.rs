//! parse_full.rs — FullType::Tuple/Array/Slice/BareFn 完整解析器
//! 零第三方依賴，core 承諾
//! 同時提供 FullProgram 中 const/static/type 的 ItemV2 解析增強（與 ast_v2.rs 互補）

use super::ast_full::{FullType, Vis, Attr, Lifetime, GenericParam, Generics, WhereClause, TypeBound, FullItem, FullProgram, StructItem, EnumItem, ConstItem, StaticItem, TypeAliasItem, StructFields, NamedField, FnItem, FnSig, FnInput, ModItem, UseItem, UseTree, ImplItem, TraitItem, MacroItem, ExternBlockItem, FullPat, FullExpr, FullStmt};
use super::universe::{TypeV2, BaseType, ExtType, parse_type_v2};

/// 解析 FullType 的入口，支持：
/// - Tuple: (i32, bool) / (T1, T2, T3)
/// - Array: [T; N] / [T; 3] / [T]（無長度視為 Slice 或 Array）
/// - Slice: [T]
/// - BareFn: fn(T1, T2) -> Ret / fn() / unsafe fn() -> Ret / fn(T) -> Ret
/// - 以及 Path, Ref, Ptr, Never, Inferred, TraitObject, ImplTrait, Paren
pub fn parse_full_type_str(s: &str) -> Result<FullType, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("空類型".to_string());
    }

    // Never !
    if s == "!" {
        return Ok(FullType::Never);
    }
    // Inferred _
    if s == "_" {
        return Ok(FullType::Inferred);
    }

    // Paren / Group: (T) 單元素括號視為 Paren，若多元素則 Tuple
    if s.starts_with('(') && s.ends_with(')') {
        let inner = s[1..s.len()-1].trim();
        if inner.is_empty() {
            // () -> Unit，視為 Tuple 空或 V2 Unit
            return Ok(FullType::Tuple(vec![]));
        }
        // 檢查是否為單一類型被括號包裹：無頂層逗號
        if !has_top_level_comma(inner) {
            // 可能是 (T) -> Paren
            let inner_ty = parse_full_type_str(inner)?;
            // 若 inner 已是 Tuple，保留，否則包 Paren
            // 為與 syn 對齊，單元素括號視為 Paren
            if matches!(inner_ty, FullType::Tuple(_)) {
                return Ok(inner_ty);
            } else {
                return Ok(FullType::Paren(Box::new(inner_ty)));
            }
        } else {
            // Tuple: (T1, T2, ...)
            let parts = split_top_level(inner, ',');
            let mut tys = Vec::with_capacity(parts.len());
            for p in parts {
                let p = p.trim();
                if p.is_empty() { continue; }
                tys.push(parse_full_type_str(p)?);
            }
            return Ok(FullType::Tuple(tys));
        }
    }

    // Array / Slice: [T; N] 或 [T]
    if s.starts_with('[') && s.ends_with(']') {
        let inner = s[1..s.len()-1].trim();
        if inner.is_empty() {
            return Err("空數組類型 []".to_string());
        }
        // 查找頂層 ';'
        if let Some(semi_idx) = find_top_level_char(inner, ';') {
            let elem_str = inner[..semi_idx].trim();
            let len_str = inner[semi_idx+1..].trim().to_string();
            let elem = Box::new(parse_full_type_str(elem_str)?);
            return Ok(FullType::Array { elem, len: Some(len_str) });
        } else {
            // 無 ';'，區分為 Slice
            // [T] 在 Rust 中可為 Slice 或 Array 無長度，這裡按 Slice 處理
            // 若需 Array 無長度，可視為 Array { len: None }
            let elem = Box::new(parse_full_type_str(inner)?);
            return Ok(FullType::Slice(elem));
        }
    }

    // BareFn: fn(params) -> ret / unsafe fn / fn() -> !
    // 支持：fn(T1, T2) -> Ret, fn() , fn(T) -> Ret, unsafe fn(), fn(T) -> Ret
    if s.starts_with("fn") || s.starts_with("unsafe fn") || s.starts_with("extern") {
        return parse_bare_fn(s);
    }

    // Ref: &T, &mut T, &'a T, &'a mut T
    if s.starts_with('&') {
        let rest = s[1..].trim();
        // lifetime?
        let (lifetime, rest) = if rest.starts_with('\'') {
            let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
            let lt = rest[..end].trim().to_string();
            (Some(lt), rest[end..].trim())
        } else {
            (None, rest)
        };
        if rest.starts_with("mut ") {
            let inner_str = rest[4..].trim();
            let inner = Box::new(parse_full_type_str(inner_str)?);
            return Ok(FullType::Ref { mutbl: true, lifetime, inner });
        } else {
            let inner = Box::new(parse_full_type_str(rest)?);
            return Ok(FullType::Ref { mutbl: false, lifetime, inner });
        }
    }

    // Ptr: *mut T, *const T
    if s.starts_with("*mut ") {
        let inner_str = s[5..].trim();
        let inner = Box::new(parse_full_type_str(inner_str)?);
        return Ok(FullType::Ptr { mutbl: true, inner });
    }
    if s.starts_with("*const ") {
        let inner_str = s[7..].trim();
        let inner = Box::new(parse_full_type_str(inner_str)?);
        return Ok(FullType::Ptr { mutbl: false, inner });
    }

    // TraitObject: dyn Trait + Send
    if s.starts_with("dyn ") {
        let bounds_str = s[4..].trim();
        let bounds = parse_type_bounds(bounds_str);
        return Ok(FullType::TraitObject { bounds, dyn_token: true });
    }
    // ImplTrait: impl Trait + Send
    if s.starts_with("impl ") {
        let bounds_str = s[5..].trim();
        let bounds = parse_type_bounds(bounds_str);
        return Ok(FullType::ImplTrait { bounds });
    }

    // Path with generic args: Vec<i32>, HashMap<K,V>, MyMod::Point, Option<T>
    // 簡化：查找 '<' ... '>' 並解析參數
    if let Some(lt_idx) = s.find('<') {
        if s.ends_with('>') {
            let path = s[..lt_idx].trim().to_string();
            let args_str = &s[lt_idx+1..s.len()-1];
            let arg_parts = split_top_level(args_str, ',');
            let mut args = Vec::new();
            for ap in arg_parts {
                let ap = ap.trim();
                if ap.is_empty() { continue; }
                args.push(parse_full_type_str(ap)?);
            }
            return Ok(FullType::Path { path, args });
        }
    }

    // 基礎路徑：單標識或 :: 分隔
    // 同時嘗試映射到 TypeV2 以保持兼容
    if let Ok(v2) = parse_type_v2(s) {
        // 若 v2 已是 Tuple/Array/Slice/BareFn 等，轉為 FullType 對應變體以滿足要求
        match v2 {
            TypeV2::Ext(ExtType::Tuple(ts)) => {
                let full_ts: Vec<FullType> = ts.into_iter().map(|t| FullType::V2(t)).collect();
                return Ok(FullType::Tuple(full_ts));
            }
            TypeV2::Ext(ExtType::Array { elem, len }) => {
                return Ok(FullType::Array { elem: Box::new(FullType::V2(*elem)), len });
            }
            TypeV2::Ext(ExtType::Slice(elem)) => {
                return Ok(FullType::Slice(Box::new(FullType::V2(*elem))));
            }
            TypeV2::Ext(ExtType::BareFn { params, ret }) => {
                let full_params: Vec<FullType> = params.into_iter().map(|p| FullType::V2(p)).collect();
                return Ok(FullType::BareFn { params: full_params, ret: Box::new(FullType::V2(*ret)), is_unsafe: false, is_async: false });
            }
            _ => {
                // 其他情況保留 V2 包裝，或轉 Path
                // 為了實現 FullType::Tuple/Array/Slice/BareFn 的顯式構造，這裡已處理
                // 其餘返回 V2
                return Ok(FullType::V2(v2));
            }
        }
    }

    //  fallback Path
    Ok(FullType::Path { path: s.to_string(), args: vec![] })
}

fn parse_bare_fn(s: &str) -> Result<FullType, String> {
    let mut is_unsafe = false;
    let mut rest = s.trim();
    if rest.starts_with("unsafe ") {
        is_unsafe = true;
        rest = rest[7..].trim();
    }
    if !rest.starts_with("fn") {
        return Err(format!("非 BareFn: {}", s));
    }
    rest = rest[2..].trim(); // after fn
    // 可能是 fn(params) 或 fn (params) 等
    let paren_start = rest.find('(').ok_or_else(|| format!("BareFn 缺少 '(': {}", s))?;
    let paren_end = find_matching_paren(rest, paren_start).ok_or_else(|| format!("BareFn 括號不匹配: {}", s))?;
    let params_str = rest[paren_start+1..paren_end].trim();
    let after_paren = rest[paren_end+1..].trim();

    let mut params = Vec::new();
    if !params_str.is_empty() {
        let parts = split_top_level(params_str, ',');
        for p in parts {
            let p = p.trim();
            if p.is_empty() { continue; }
            params.push(parse_full_type_str(p)?);
        }
    }

    // ret: -> Type 或默認 ()
    let ret = if let Some(arrow_idx) = after_paren.find("->") {
        let ret_str = after_paren[arrow_idx+2..].trim();
        Box::new(parse_full_type_str(ret_str)?)
    } else {
        Box::new(FullType::Tuple(vec![])) // () as unit
    };

    Ok(FullType::BareFn { params, ret, is_unsafe, is_async: false })
}

fn parse_type_bounds(s: &str) -> Vec<TypeBound> {
    split_top_level(s, '+')
        .into_iter()
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty())
        .map(|b| {
            if b.starts_with('\'') {
                TypeBound::Lifetime(b)
            } else {
                TypeBound::Trait(b)
            }
        })
        .collect()
}

/// 是否包含頂層逗號（不在 < > ( ) [ ] 內）
fn has_top_level_comma(s: &str) -> bool {
    find_top_level_char(s, ',').is_some()
}

fn find_top_level_char(s: &str, target: char) -> Option<usize> {
    let mut depth_angle = 0;
    let mut depth_paren = 0;
    let mut depth_brack = 0;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth_angle += 1,
            '>' => if depth_angle > 0 { depth_angle -= 1; },
            '(' => depth_paren += 1,
            ')' => if depth_paren > 0 { depth_paren -= 1; },
            '[' => depth_brack += 1,
            ']' => if depth_brack > 0 { depth_brack -= 1; },
            _ => {}
        }
        if c == target && depth_angle == 0 && depth_paren == 0 && depth_brack == 0 {
            return Some(i);
        }
    }
    None
}

fn split_top_level(s: &str, delim: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth_angle = 0;
    let mut depth_paren = 0;
    let mut depth_brack = 0;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '<' => depth_angle += 1,
            '>' => if depth_angle > 0 { depth_angle -= 1; },
            '(' => depth_paren += 1,
            ')' => if depth_paren > 0 { depth_paren -= 1; },
            '[' => depth_brack += 1,
            ']' => if depth_brack > 0 { depth_brack -= 1; },
            _ => {}
        }
        if c == delim && depth_angle == 0 && depth_paren == 0 && depth_brack == 0 {
            parts.push(s[start..i].to_string());
            start = i + 1;
        }
    }
    parts.push(s[start..].to_string());
    parts
}

fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.get(start) != Some(&b'(') {
        return None;
    }
    let mut depth = 0;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if b == b'(' { depth += 1; }
        if b == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

// ─── ItemV2 const/static/type 解析增強（與 ast_v2.rs 互補）────────────────────
/// 從單行解析 const/static/type 為 ItemV2，若成功返回對應 ItemV2
/// ast_v2.rs 已有 3 分支（parse_const, parse_static, parse_type_alias），此處提供獨立函數供外部調用與測試
pub fn parse_item_v2_from_line(line: &str) -> Option<super::ast_v2::ItemV2> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    // const
    if line.starts_with("const ") || line.starts_with("pub const ") || line.starts_with("pub(crate) const ") {
        // 復用 ast_v2::ProgramV2::parse_v2 的單行解析邏輯簡化版
        // const NAME: TY = EXPR;
        let is_pub = line.starts_with("pub");
        let rest = if is_pub {
            line.trim_start_matches("pub ").trim().trim_start_matches("(crate) ").trim()
        } else { line };
        if !rest.starts_with("const ") { return None; }
        let after_const = rest[6..].trim();
        let name_end = after_const.find(|c: char| c == ':' || c == '=' || c.is_whitespace()).unwrap_or(after_const.len());
        let name = after_const[..name_end].trim().to_string();
        if name.is_empty() { return None; }
        let mut ty = super::universe::TypeV2::Base(BaseType::I32);
        let mut expr = None;
        if let Some(colon) = after_const.find(':') {
            let after_colon = after_const[colon+1..].trim();
            if let Some(eq_pos) = after_colon.find('=') {
                let ty_str = after_colon[..eq_pos].trim();
                if let Ok(parsed_ty) = super::universe::parse_type_v2(ty_str) {
                    ty = parsed_ty;
                }
                let e = after_colon[eq_pos+1..].trim().trim_end_matches(';').to_string();
                expr = Some(e);
            } else {
                let ty_str = after_colon.trim().trim_end_matches(';');
                if let Ok(parsed_ty) = super::universe::parse_type_v2(ty_str) {
                    ty = parsed_ty;
                }
            }
        }
        let def = super::ast_v2::ConstDefV2 { name, ty, expr, is_pub };
        return Some(super::ast_v2::ItemV2::Const(def));
    }

    // static
    if line.starts_with("static ") || line.starts_with("pub static ") || line.starts_with("pub(crate) static ") || line.starts_with("pub static mut ") || line.starts_with("static mut ") {
        let is_pub = line.starts_with("pub");
        let mut rest = if is_pub {
            line.trim_start_matches("pub ").trim().trim_start_matches("(crate) ").trim()
        } else { line };
        let mut mutbl = false;
        if rest.starts_with("static mut ") {
            mutbl = true;
            rest = rest[11..].trim();
        } else if rest.starts_with("static ") {
            rest = rest[7..].trim();
        } else {
            return None;
        }
        let name_end = rest.find(|c: char| c == ':' || c == '=' || c.is_whitespace()).unwrap_or(rest.len());
        let name = rest[..name_end].trim().to_string();
        if name.is_empty() { return None; }
        let mut ty = super::universe::TypeV2::Base(BaseType::I32);
        let mut expr = None;
        if let Some(colon) = rest.find(':') {
            let after_colon = rest[colon+1..].trim();
            if let Some(eq_pos) = after_colon.find('=') {
                let ty_str = after_colon[..eq_pos].trim();
                if let Ok(parsed_ty) = super::universe::parse_type_v2(ty_str) {
                    ty = parsed_ty;
                }
                let e = after_colon[eq_pos+1..].trim().trim_end_matches(';').to_string();
                expr = Some(e);
            } else {
                let ty_str = after_colon.trim().trim_end_matches(';');
                if let Ok(parsed_ty) = super::universe::parse_type_v2(ty_str) {
                    ty = parsed_ty;
                }
            }
        }
        let def = super::ast_v2::StaticDefV2 { name, ty, mutbl, expr, is_pub };
        return Some(super::ast_v2::ItemV2::Static(def));
    }

    // type alias
    if line.starts_with("type ") || line.starts_with("pub type ") || line.starts_with("pub(crate) type ") {
        let is_pub = line.starts_with("pub");
        let rest = if is_pub {
            line.trim_start_matches("pub ").trim().trim_start_matches("(crate) ").trim()
        } else { line };
        if !rest.starts_with("type ") { return None; }
        let after_type = rest[5..].trim();
        let name_end = after_type.find(|c: char| c == '<' || c == '=' || c.is_whitespace()).unwrap_or(after_type.len());
        let name = after_type[..name_end].trim().to_string();
        if name.is_empty() { return None; }
        let mut generics = vec![];
        if let Some(lt) = after_type.find('<') {
            if let Some(gt) = after_type.find('>') {
                let gen_str = &after_type[lt+1..gt];
                generics = gen_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
        }
        let mut ty = super::universe::TypeV2::Base(BaseType::I32);
        if let Some(eq) = after_type.find('=') {
            let ty_str = after_type[eq+1..].trim().trim_end_matches(';').to_string();
            if let Ok(parsed_ty) = super::universe::parse_type_v2(&ty_str) {
                ty = parsed_ty;
            }
        }
        let def = super::ast_v2::TypeAliasDefV2 { name, generics, ty, is_pub };
        return Some(super::ast_v2::ItemV2::TypeAlias(def));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_type_tuple() {
        let ty = parse_full_type_str("(i32, bool)").unwrap();
        match ty {
            FullType::Tuple(ts) => {
                assert_eq!(ts.len(), 2);
                assert_eq!(ts[0].name(), "i32");
                assert_eq!(ts[1].name(), "bool");
            }
            _ => panic!("expected Tuple, got {:?}", ty),
        }
    }

    #[test]
    fn test_full_type_nested_tuple() {
        let ty = parse_full_type_str("(i32, (bool, String))").unwrap();
        match ty {
            FullType::Tuple(ts) => {
                assert_eq!(ts.len(), 2);
                assert_eq!(ts[0].name(), "i32");
                match &ts[1] {
                    FullType::Tuple(inner) => assert_eq!(inner.len(), 2),
                    _ => panic!("expected inner Tuple"),
                }
            }
            _ => panic!("expected Tuple"),
        }
    }

    #[test]
    fn test_full_type_array() {
        let ty = parse_full_type_str("[i32; 3]").unwrap();
        match ty {
            FullType::Array { elem, len } => {
                assert_eq!(elem.name(), "i32");
                assert_eq!(len.unwrap(), "3");
            }
            _ => panic!("expected Array, got {:?}", ty),
        }
    }

    #[test]
    fn test_full_type_slice() {
        let ty = parse_full_type_str("[i32]").unwrap();
        match ty {
            FullType::Slice(elem) => {
                assert_eq!(elem.name(), "i32");
            }
            _ => panic!("expected Slice, got {:?}", ty),
        }
    }

    #[test]
    fn test_full_type_bare_fn() {
        let cases = vec![
            ("fn(i32) -> bool", 1, "bool"),
            ("fn(i32, bool) -> String", 2, "String"),
            ("fn() -> ()", 0, "()"),
            ("fn() -> i32", 0, "i32"),
            ("unsafe fn(i32) -> bool", 1, "bool"),
        ];
        for (src, param_len, ret_name) in cases {
            let ty = parse_full_type_str(src).unwrap();
            match ty {
                FullType::BareFn { params, ret, is_unsafe, .. } => {
                    assert_eq!(params.len(), param_len, "src={}", src);
                    assert_eq!(ret.name(), ret_name, "src={}", src);
                    if src.starts_with("unsafe") {
                        assert!(is_unsafe);
                    }
                }
                _ => panic!("expected BareFn for {}, got {:?}", src, ty),
            }
        }
    }

    #[test]
    fn test_full_type_bare_fn_nested() {
        let ty = parse_full_type_str("fn(Vec<i32>) -> Option<String>").unwrap();
        match ty {
            FullType::BareFn { params, ret, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name(), "Vec<i32>");
                assert_eq!(ret.name(), "Option<String>");
            }
            _ => panic!("expected BareFn"),
        }
    }

    #[test]
    fn test_item_v2_const() {
        let line = "pub const MAX: i32 = 5;";
        let item = parse_item_v2_from_line(line).unwrap();
        match item {
            super::super::ast_v2::ItemV2::Const(c) => {
                assert_eq!(c.name, "MAX");
                assert!(c.is_pub);
                assert_eq!(c.ty.name(), "i32");
            }
            _ => panic!("expected Const"),
        }
    }

    #[test]
    fn test_item_v2_static() {
        let line = "pub static mut S: i32 = 0;";
        let item = parse_item_v2_from_line(line).unwrap();
        match item {
            super::super::ast_v2::ItemV2::Static(s) => {
                assert_eq!(s.name, "S");
                assert!(s.mutbl);
                assert!(s.is_pub);
            }
            _ => panic!("expected Static"),
        }
    }

    #[test]
    fn test_item_v2_type_alias() {
        let line = "pub type MyAlias = Vec<i32>;";
        let item = parse_item_v2_from_line(line).unwrap();
        match item {
            super::super::ast_v2::ItemV2::TypeAlias(t) => {
                assert_eq!(t.name, "MyAlias");
                assert!(t.is_pub);
                assert_eq!(t.ty.name(), "Vec<i32>");
            }
            _ => panic!("expected TypeAlias"),
        }
    }

    #[test]
    fn test_item_v2_via_program() {
        // 通過 ProgramV2::parse_v2 驗證 ast_v2.rs 已有 3 分支
        let src = r#"
            pub const MAX: i32 = 5;
            pub static S: i32 = 0;
            pub type MyAlias = Vec<i32>;
            fn main() {}
        "#;
        let prog = super::super::ast_v2::ProgramV2::parse_v2(src).unwrap();
        assert_eq!(prog.items.len(), 3);
        // 檢查三分支存在
        let has_const = prog.items.iter().any(|it| matches!(it, super::super::ast_v2::ItemV2::Const(_)));
        let has_static = prog.items.iter().any(|it| matches!(it, super::super::ast_v2::ItemV2::Static(_)));
        let has_type = prog.items.iter().any(|it| matches!(it, super::super::ast_v2::ItemV2::TypeAlias(_)));
        assert!(has_const, "const 分支缺失");
        assert!(has_static, "static 分支缺失");
        assert!(has_type, "type alias 分支缺失");
    }
}
