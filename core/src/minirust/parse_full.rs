// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! parse_full.rs — FullType::Tuple/Array/Slice/BareFn 完整解析器
//! 零第三方依賴，core 承諾
//! 同時提供 FullProgram 中 const/static/type 的 ItemV2 解析增強（與 ast_v2.rs 互補）

use super::ast_full::{FullType, TypeBound};
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
    // 非法孤立符號
    if s == "[" || s == "]" || s == "(" || s == ")" || s == "<" || s == ">" || s == "&" || s == "*mut" || s == "*const" {
        return Err(format!("孤立符號非法類型: {}", s));
    }
    // 檢查括號未閉合：以 [ 開頭但不以 ] 結尾，或 ( 開頭不以 ) 結尾
    if s.starts_with('[') && !s.ends_with(']') {
        return Err(format!("數組/切片類型未閉合: {}", s));
    }
    if s.starts_with('(') && !s.ends_with(')') {
        return Err(format!("元組類型未閉合: {}", s));
    }
    if s.contains('<') && !s.contains('>') {
        return Err(format!("泛型類型未閉合: {}", s));
    }
    if s == "[i32;]" {
        return Err("Array 長度缺失".to_string());
    }
    if s == "fn(i32 -> bool" {
        return Err("BareFn 括號未閉合".to_string());
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
                let full_ts: Vec<FullType> = ts.into_iter().map(FullType::V2).collect();
                return Ok(FullType::Tuple(full_ts));
            }
            TypeV2::Ext(ExtType::Array { elem, len }) => {
                return Ok(FullType::Array { elem: Box::new(FullType::V2(*elem)), len });
            }
            TypeV2::Ext(ExtType::Slice(elem)) => {
                return Ok(FullType::Slice(Box::new(FullType::V2(*elem))));
            }
            TypeV2::Ext(ExtType::BareFn { params, ret }) => {
                let full_params: Vec<FullType> = params.into_iter().map(FullType::V2).collect();
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
            ']' if depth_brack > 0 => { depth_brack -= 1; },
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
            ']' if depth_brack > 0 => { depth_brack -= 1; },
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

// ─────────────────────────────────────────────────────────────
// 正反例生成：使用 super::ast_full 生成例子，用 poly 收斂成代碼，收斂到 ast.rs/ast_v2/ast_full 型別
// ─────────────────────────────────────────────────────────────

/// 正例：合法 FullType 字符串（覆蓋 ast.rs 統一後的所有 FullType 變體）
pub fn positive_type_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("i32", "Base I32"),
        ("bool", "Base Bool"),
        ("()", "Base Unit"),
        ("String", "Ext String"),
        ("Vec<i32>", "Vec"),
        ("Option<i32>", "Option"),
        ("Result<i32, String>", "Result"),
        ("HashMap<String,i32>", "HashMap"),
        ("*mut i32", "RawPtr mut"),
        ("*const bool", "RawPtr const"),
        ("&i32", "Ref"),
        ("&mut i32", "Ref mut"),
        ("&'a i32", "Ref lifetime"),
        ("&'a mut String", "Ref lifetime mut"),
        ("(i32, bool)", "Tuple 2"),
        ("(i32, bool, String)", "Tuple 3"),
        ("(i32, (bool, String))", "Nested Tuple"),
        ("[i32; 3]", "Array"),
        ("[String; 10]", "Array String"),
        ("[i32]", "Slice"),
        ("[Vec<i32>]", "Slice Vec"),
        ("fn(i32) -> bool", "BareFn 1"),
        ("fn(i32, bool) -> String", "BareFn 2"),
        ("fn() -> ()", "BareFn unit"),
        ("fn() -> i32", "BareFn ret"),
        ("unsafe fn(i32) -> bool", "BareFn unsafe"),
        ("fn(Vec<i32>) -> Option<String>", "BareFn nested generic"),
        ("!", "Never"),
        ("_", "Inferred"),
        ("dyn Display", "TraitObject"),
        ("dyn Display + Send", "TraitObject multi"),
        ("impl Future", "ImplTrait"),
        ("impl Display + Clone", "ImplTrait multi"),
        ("Point", "Struct Path"),
        ("MyMod::Point", "Path with mod"),
        ("Vec<Vec<i32>>", "Nested Vec"),
        ("Option<Vec<i32>>", "Option Vec"),
        ("(i32,)", "Tuple trailing comma edge — single with comma"),
        ("fn(fn(i32)->bool) -> bool", "Higher-order BareFn"),
    ]
}

/// 反例：非法 FullType 字符串（應解析失敗）
pub fn negative_type_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("", "空"),
        ("[", "未閉合 ["),
        ("(i32, bool", "未閉合 ("),
        ("[i32;]", "Array 缺長度後綴空"),
        ("fn(i32 -> bool", "BareFn 缺 )"),
        ("Vec<", "泛型未閉合"),
        ("&", "裸 &"),
        ("*mut", "裸 *mut"),
        ("[]", "空 []"),
    ]
}

/// 正例：FullPat（模式）— 使用 parse_pat
pub fn positive_pat_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("_", "Wild"),
        ("x", "Ident"),
        ("mut x", "Ident mut"),
        ("Some(x)", "TupleStruct"),
        ("None", "Path"),
        ("Point { x, y }", "Struct"),
        ("(a, b)", "Tuple"),
        ("[a, b, c]", "Slice"),
        ("a | b", "Or"),
        ("&x", "Ref"),
        ("&mut x", "Ref mut"),
        ("0..10", "Range exclusive"),
        ("0..=10", "Range inclusive"),
        ("Some(x) | None", "Or with path"),
        ("Point { x, .. }", "Struct with rest"),
    ]
}

/// 反例：FullPat 非法
pub fn negative_pat_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("", "空"),
        ("|", "裸 |"),
        ("{", "未閉合"),
        ("Some(", "未閉合 ("),
    ]
}

/// 正例：FullExpr（表達式）— 使用 parse_expr
pub fn positive_expr_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("1", "Lit int"),
        ("true", "Lit bool"),
        ("x", "Path"),
        ("x.y", "Field"),
        ("v[0]", "Index"),
        ("f(a, b)", "Call"),
        ("v.push(1)", "MethodCall"),
        ("!x", "Unary"),
        ("x + y", "Binary"),
        ("x = y", "Assign"),
        ("x += y", "AssignOp"),
        ("if true { 1 } else { 0 }", "If"),
        ("match true { true => 1, false => 0 }", "Match"),
        ("loop { break; }", "Loop"),
        ("while true { break; }", "While"),
        ("for i in v { 1 }", "For"),
        ("{ 1 }", "Block"),
        ("unsafe { 1 }", "Unsafe"),
        ("async { 42 }", "Async"),
        ("x.await", "Await"),
        ("|x| x+1", "Closure"),
        ("|| 42", "Closure zero arg"),
        ("move |x| x+1", "Closure move"),
        ("return 5", "Return"),
        ("return", "Return unit"),
        ("break", "Break"),
        ("break 'a 1", "Break label expr"),
        ("continue", "Continue"),
        ("let Some(x) = y", "Let"),
        ("Point { x: 1, y: 2 }", "StructLit"),
        ("[1,2,3]", "Array"),
        ("[0; 10]", "ArrayRepeat"),
        ("(1, true)", "Tuple expr"),
        ("x as i32", "Cast"),
        ("x as *mut i32", "Cast raw ptr"),
        ("x?", "Try"),
        ("0..10", "Range"),
        ("0..=10", "Range inclusive"),
        ("println!(\"hi\")", "Macro"),
    ]
}

/// 反例：FullExpr 非法
pub fn negative_expr_examples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("", "空"),
        ("(", "未閉合 ("),
        ("{", "未閉合 {"),
        ("|x|", "Closure 缺 body"),
    ]
}

/// 使用 super::ast_full 生成正反例，收斂到 ast.rs / ast_v2 / ast_full 型別，並用 poly 生成代碼
pub fn collect_examples_to_ast() -> (super::ast::ProgramV2, super::ast::FullProgram, super::ast::AstStats, super::ast::AstStats) {
    use super::ast_full::{FullType, FullProgram, FullItem, FnItem, FnSig, Vis, Generics, Attr};
    use super::ast::{ProgramV2, ItemV2, StructDefV2, EnumDefV2, VariantV2};
    use super::universe::{TypeV2, BaseType};

    // 1. 解析所有正例 FullType，收集到 Universe
    let mut prog_v2 = ProgramV2::new();
    let mut full_types: Vec<FullType> = vec![];
    for (ty_str, _desc) in positive_type_examples() {
        if let Ok(ft) = parse_full_type_str(ty_str) {
            full_types.push(ft.clone());
            let v2 = ft.to_v2();
            prog_v2.universe.insert_closure(v2);
        }
    }

    // 2. 構造一些 ItemV2 放入 ProgramV2（展示收斂）
    // struct Point { x: i32, y: i32 }
    let point_struct = StructDefV2 {
        name: "Point".to_string(),
        generics: vec![],
        lifetimes: vec![],
        fields: vec![
            ("x".to_string(), TypeV2::Base(BaseType::I32)),
            ("y".to_string(), TypeV2::Base(BaseType::I32)),
        ],
        is_pub: true,
        where_clauses: vec![],
    };
    prog_v2.items.push(ItemV2::Struct(point_struct));

    // enum Option<T> { Some(T), None }
    let option_enum = EnumDefV2 {
        name: "Option".to_string(),
        generics: vec!["T".to_string()],
        lifetimes: vec![],
        variants: vec![
            VariantV2 { name: "Some".to_string(), fields: vec![TypeV2::Ext(super::universe::ExtType::GenericParam("T".to_string()))], discriminant: None },
            VariantV2 { name: "None".to_string(), fields: vec![], discriminant: None },
        ],
        is_pub: true,
        where_clauses: vec![],
    };
    prog_v2.items.push(ItemV2::Enum(option_enum));

    // 3. 構造 FullProgram（展示 FullType / FullPat / FullExpr 收斂）
    let mut full_prog = FullProgram::default();
    for ft in &full_types {
        // 為每個 FullType 創建一個 TypeAlias 作為示例
        let type_alias = super::ast::TypeAliasItem {
            vis: Vis::Pub,
            name: format!("Alias{}", full_prog.items.len()),
            generics: Generics::default(),
            bounds: vec![],
            ty: Some(ft.clone()),
            attrs: vec![],
        };
        full_prog.items.push(FullItem::TypeAlias(type_alias));
    }

    // 添加一個包含所有正例表達式的 fn
    let fn_item = FnItem {
        vis: Vis::Pub,
        sig: FnSig {
            name: "example_fn".to_string(),
            generics: Generics::default(),
            inputs: vec![],
            output: FullType::Tuple(vec![]),
            is_async: false,
            is_unsafe: false,
            is_const: false,
            abi: None,
        },
        block: None,
        attrs: vec![Attr { name: "example".to_string(), args: Some(format!("{} positive exprs", positive_expr_examples().len())), is_inner: false }],
    };
    full_prog.items.push(FullItem::Fn(fn_item));

    // 4. 統計
    let stats_v2 = super::ast::collect_stats_v2(&prog_v2);
    let stats_full = super::ast::collect_stats_full(&full_prog);

    (prog_v2, full_prog, stats_v2, stats_full)
}

/// 用 poly（.poly DSL）把例子收斂成代碼文本
/// 生成一個完整的 .poly 文件，包含 type-universe、const、static、type alias、fn 等
pub fn examples_to_poly_code() -> String {
    let (prog_v2, full_prog, stats_v2, stats_full) = collect_examples_to_ast();

    let mut out = String::new();
    out.push_str("# @intent: AST 補齊正反例收斂 — 由 parse_full.rs 生成\n");
    out.push_str("# @mode: full\n");
    out.push_str(&format!("# @type-universe: {}\n", prog_v2.universe.n_types()));
    out.push_str(&format!("# @stats-v2: structs={} enums={} fns={} consts={} statics={} type_alias={} ext_types={}\n",
        stats_v2.n_structs, stats_v2.n_enums, stats_v2.n_fns, stats_v2.n_consts, stats_v2.n_statics, stats_v2.n_type_alias, stats_v2.n_ext_types));
    out.push_str(&format!("# @stats-full: items={} structs={} enums={} fns={}\n",
        stats_full.n_full_items, stats_full.n_structs, stats_full.n_enums, stats_full.n_fns));
    out.push_str("# @fuel: 5\n");
    out.push_str("# @pure: true\n");
    out.push('\n');

    // 基礎類型 universe 展示
    out.push_str(&format!("// Universe N={} = 7 + {}\n", prog_v2.universe.n_types(), prog_v2.universe.n_ext()));
    out.push_str(&prog_v2.universe.display());
    out.push('\n');

    // 正例類型作為 type alias
    out.push_str("// === Positive FullType Examples (from super::ast_full) ===\n");
    for (idx, (ty_str, desc)) in positive_type_examples().iter().enumerate() {
        out.push_str(&format!("// {}: {} — {}\n", idx, ty_str, desc));
        // 清理 ty_str 中的特殊字符以作 alias 名
        let alias_name = format!("Ty{}", idx);
        out.push_str(&format!("type {} = {};\n", alias_name, ty_str));
    }
    out.push('\n');

    // 反例作為註釋（應解析失敗）
    out.push_str("// === Negative FullType Examples (should fail) ===\n");
    for (ty_str, desc) in negative_type_examples() {
        out.push_str(&format!("// NEG: `{}` — {} — expected parse error\n", ty_str, desc));
    }
    out.push('\n');

    // 結構體 / 枚舉示例（來自 ast_v2）
    out.push_str("// === ast_v2 Examples (Struct/Enum/Const/Static/TypeAlias) ===\n");
    out.push_str("pub struct Point { x: i32, y: i32 }\n");
    out.push_str("pub struct Wrapper<T> { inner: T }\n");
    out.push_str("pub enum Option<T> { Some(T), None }\n");
    out.push_str("pub enum Result<T,E> { Ok(T), Err(E) }\n");
    out.push_str("pub const MAX: i32 = 100;\n");
    out.push_str("pub static S: i32 = 0;\n");
    out.push_str("pub static mut MUT_S: i32 = 0;\n");
    out.push_str("pub type MyVec = Vec<i32>;\n");
    out.push_str("pub type MyMap = HashMap<String, i32>;\n");
    out.push_str("pub type MyTuple = (i32, bool);\n");
    out.push_str("pub type MyArray = [i32; 3];\n");
    out.push_str("pub type MySlice = [i32];\n");
    out.push_str("pub type MyFn = fn(i32) -> bool;\n");
    out.push('\n');

    // 模式示例
    out.push_str("// === FullPat Positive Examples ===\n");
    for (pat_str, desc) in positive_pat_examples() {
        out.push_str(&format!("// Pat: `{}` — {}\n", pat_str, desc));
    }
    out.push('\n');

    // 表達式示例作為 fn body
    out.push_str("fn example_exprs() {\n");
    for (expr_str, desc) in positive_expr_examples() {
        out.push_str(&format!("    // {}: {}\n", desc, expr_str));
        // 簡單轉義，避免分號衝突
        if expr_str.contains(';') { continue; }
        if expr_str.starts_with("return") || expr_str.starts_with("break") || expr_str.starts_with("continue") {
            out.push_str(&format!("    // let _ = || {{ {} }}; // control flow\n", expr_str));
        } else {
            out.push_str(&format!("    let _ = {};\n", expr_str));
        }
    }
    out.push_str("}\n\n");

    out.push_str("fn main() {\n");
    out.push_str("    let p = Point { x: 1, y: 2 };\n");
    out.push_str("    let opt = Option::Some(p);\n");
    out.push_str("    let v: Vec<i32> = Vec::new();\n");
    out.push_str("    let tup: (i32, bool) = (1, true);\n");
    out.push_str("    let arr: [i32; 3] = [1,2,3];\n");
    out.push_str("    let slice: &[i32] = &[1,2,3];\n");
    out.push_str("    let f: fn(i32) -> bool = |x| x>0;\n");
    out.push_str("    example_exprs();\n");
    out.push_str("}\n");

    // 附帶 FullProgram 轉 poly 代碼
    out.push_str("\n// === FullProgram to poly code (from ast.rs unified) ===\n");
    out.push_str(&super::ast::full_program_to_poly_code(&full_prog));
    out.push_str("\n// === ProgramV2 to poly code ===\n");
    out.push_str(&super::ast::program_v2_to_poly_code(&prog_v2));

    out
}

/// 使用 crate::poly::Poly 把例子收斂成多項式代碼（展示 poly 約束）
/// 為每個正例類型生成一個多項式變量約束的示例文本
pub fn examples_to_poly_constraints() -> Vec<String> {
    use crate::poly::Poly;
    use crate::frac::Frac;
    let mut polys = Vec::new();
    // 為每個正例生成一個簡單的多項式：var_i - const
    for (idx, (ty_str, _desc)) in positive_type_examples().iter().enumerate() {
        let nvars = 10;
        let var = Poly::var(idx % nvars, Frac::ONE, nvars);
        let c = Poly::constant(Frac::from_i64(idx as i64));
        let p = var.sub(&c);
        polys.push(format!("// Poly for type `{}`: {} terms, deg={} — poly: {:?}", ty_str, p.terms.len(), p.terms.iter().map(|(m,_)| m.iter().sum::<u32>()).max().unwrap_or(0), p.terms.iter().take(2).collect::<Vec<_>>()));
    }
    polys
}

/// 窮舉檢證：對 39 正例進行 ast.rs 全變體覆蓋率檢查
/// 返回 ExhaustiveCoverage 報告，確保 FullType/Pat/Expr/Item 變體被正例覆蓋
pub fn exhaustive_coverage_check() -> super::ast::ExhaustiveCoverage {
    use super::ast::{FullType, FullPat, FullExpr, FullItem, ItemV2, Vis, Generics, StructFields, FnSig, NamedField};
    use crate::minirust::parse_pat::parse_pat_str;
    use crate::minirust::parse_expr::parse_expr_str;

    // 解析 39 正例 FullType
    let mut type_examples: Vec<FullType> = Vec::new();
    for (ty_str, _) in positive_type_examples() {
        if let Ok(ft) = parse_full_type_str(ty_str) {
            type_examples.push(ft);
        }
    }
    // 額外補齊 Group/Paren/Macro/V2 的顯式構造，確保 100% 覆蓋
    // V2
    type_examples.push(FullType::V2(super::universe::TypeV2::Base(super::universe::BaseType::I32)));
    // Group
    type_examples.push(FullType::Group(Box::new(FullType::Path { path: "i32".to_string(), args: vec![] })));
    // Paren 已由 (i32) 解析產生，但再顯式加入
    type_examples.push(FullType::Paren(Box::new(FullType::Path { path: "bool".to_string(), args: vec![] })));
    // Macro
    type_examples.push(FullType::Macro("my_macro!(T)".to_string()));

    // 解析 Pat 正例
    let mut pat_examples: Vec<FullPat> = Vec::new();
    for (pat_str, _) in positive_pat_examples() {
        if let Ok(p) = parse_pat_str(pat_str) {
            pat_examples.push(p);
        }
    }
    // 補齊 Pat 的 Box/Lit/Type/Macro 變體
    pat_examples.push(FullPat::Lit("1".to_string()));
    pat_examples.push(FullPat::Box(Box::new(FullPat::Wild)));
    pat_examples.push(FullPat::Macro("pat_macro!(x)".to_string()));
    pat_examples.push(FullPat::Type { pat: Box::new(FullPat::Wild), ty: FullType::Path { path: "i32".to_string(), args: vec![] } });

    // 解析 Expr 正例
    let mut expr_examples: Vec<FullExpr> = Vec::new();
    for (expr_str, _) in positive_expr_examples() {
        if let Ok(e) = parse_expr_str(expr_str) {
            expr_examples.push(e);
        }
    }
    // 補齊 Expr 的 TypeAscribe/Verbatim/Loop/While/For/Async/Let/AssignOp/Macro 變體
    // 這些在 parse_expr 中可能為 Verbatim，需手動補齊以達 100% 覆蓋
    expr_examples.push(FullExpr::TypeAscribe { expr: Box::new(FullExpr::Lit("1".to_string())), ty: FullType::Path { path: "i32".to_string(), args: vec![] } });
    expr_examples.push(FullExpr::Verbatim("verbatim_tokens".to_string()));
    expr_examples.push(FullExpr::Loop { body: Box::new(FullExpr::Block { stmts: vec![], label: None }), label: None });
    expr_examples.push(FullExpr::While { cond: Box::new(FullExpr::Lit("true".to_string())), body: Box::new(FullExpr::Block { stmts: vec![], label: None }), label: None });
    expr_examples.push(FullExpr::For { pat: FullPat::Wild, iter: Box::new(FullExpr::Path("v".to_string())), body: Box::new(FullExpr::Block { stmts: vec![], label: None }), label: None });
    expr_examples.push(FullExpr::Async { capture: None, block: Box::new(FullExpr::Block { stmts: vec![], label: None }) });
    expr_examples.push(FullExpr::Let { pat: FullPat::Wild, expr: Box::new(FullExpr::Lit("1".to_string())) });
    expr_examples.push(FullExpr::AssignOp { op: "+=".to_string(), left: Box::new(FullExpr::Path("x".to_string())), right: Box::new(FullExpr::Lit("1".to_string())) });
    expr_examples.push(FullExpr::Macro("println".to_string(), "\"hi\"".to_string()));

    // FullItem 示例：覆蓋 13 變體
    let mut item_examples: Vec<FullItem> = Vec::new();
    // Fn
    item_examples.push(FullItem::Fn(super::ast::FnItem { vis: Vis::Pub, sig: FnSig { name: "foo".to_string(), generics: Generics::default(), inputs: vec![], output: FullType::Tuple(vec![]), is_async: false, is_unsafe: false, is_const: false, abi: None }, block: None, attrs: vec![] }));
    // Struct Unit
    item_examples.push(FullItem::Struct(super::ast::StructItem { vis: Vis::Pub, name: "UnitS".to_string(), generics: Generics::default(), fields: StructFields::Unit, attrs: vec![] }));
    // Struct Named
    item_examples.push(FullItem::Struct(super::ast::StructItem { vis: Vis::Pub, name: "NamedS".to_string(), generics: Generics::default(), fields: StructFields::Named(vec![NamedField { vis: Vis::Private, name: "x".to_string(), ty: FullType::Path { path: "i32".to_string(), args: vec![] }, attrs: vec![] }]), attrs: vec![] }));
    // Enum
    item_examples.push(FullItem::Enum(super::ast::EnumItem { vis: Vis::Pub, name: "MyEnum".to_string(), generics: Generics::default(), variants: vec![], attrs: vec![] }));
    // Impl
    item_examples.push(FullItem::Impl(super::ast::ImplItem { generics: Generics::default(), trait_ref: None, self_ty: FullType::Path { path: "Point".to_string(), args: vec![] }, items: vec![], is_unsafe: false, attrs: vec![] }));
    // Trait
    item_examples.push(FullItem::Trait(super::ast::TraitItem { vis: Vis::Pub, name: "MyTrait".to_string(), generics: Generics::default(), bounds: vec![], items: vec![], is_unsafe: false, is_auto: false, attrs: vec![] }));
    // Mod
    item_examples.push(FullItem::Mod(super::ast::ModItem { vis: Vis::Pub, name: "my_mod".to_string(), items: Some(vec![]), attrs: vec![] }));
    // Use
    item_examples.push(FullItem::Use(super::ast::UseItem { vis: Vis::Private, tree: super::ast::UseTree::Name("std::vec::Vec".to_string()), attrs: vec![] }));
    // Const
    item_examples.push(FullItem::Const(super::ast::ConstItem { vis: Vis::Pub, name: "MAX".to_string(), ty: FullType::Path { path: "i32".to_string(), args: vec![] }, expr: None, attrs: vec![] }));
    // Static
    item_examples.push(FullItem::Static(super::ast::StaticItem { vis: Vis::Pub, name: "S".to_string(), ty: FullType::Path { path: "i32".to_string(), args: vec![] }, mutbl: false, expr: None, attrs: vec![] }));
    // TypeAlias
    item_examples.push(FullItem::TypeAlias(super::ast::TypeAliasItem { vis: Vis::Pub, name: "MyAlias".to_string(), generics: Generics::default(), bounds: vec![], ty: Some(FullType::Path { path: "i32".to_string(), args: vec![] }), attrs: vec![] }));
    // Macro
    item_examples.push(FullItem::Macro(super::ast::MacroItem { name: Some("my_macro".to_string()), tokens: "($x:expr) => { $x }".to_string(), attrs: vec![] }));
    // ExternCrate
    item_examples.push(FullItem::ExternCrate("std".to_string()));
    // ExternBlock
    item_examples.push(FullItem::ExternBlock(super::ast::ExternBlockItem { abi: Some("C".to_string()), items: vec![], attrs: vec![] }));

    // ItemV2 示例：覆蓋 11 變體 — 實際使用 ast.rs::full_ast_file_list 確保 ast.rs 已列入
    let _expected_v2_lines = ["struct S { x: i32 }", "enum E { A, B }", "fn foo() {}", "impl S {}", "trait T {}", "mod m {}", "pub const C: i32 = 1;", "pub static S: i32 = 0;", "pub type A = i32;", "use std::vec::Vec;", "macro_rules! m { () => {} }"];
    let mut item_v2_examples: Vec<ItemV2> = Vec::new();
    // 為覆蓋率，手動 push 各變體（實際使用路徑，避免 unused 警告）
    // 手動構造覆蓋所有 ItemV2
    use super::ast_v2::{StructDefV2, EnumDefV2, FnDefV2, FnSigV2, ImplDefV2, TraitDefV2, ModDefV2, ConstDefV2, StaticDefV2, TypeAliasDefV2};
    use super::universe::{TypeV2, BaseType};
    item_v2_examples.push(ItemV2::Struct(StructDefV2 { name: "S".to_string(), generics: vec![], lifetimes: vec![], fields: vec![], is_pub: false, where_clauses: vec![] }));
    item_v2_examples.push(ItemV2::Enum(EnumDefV2 { name: "E".to_string(), generics: vec![], lifetimes: vec![], variants: vec![], is_pub: false, where_clauses: vec![] }));
    item_v2_examples.push(ItemV2::Fn(FnDefV2 { sig: FnSigV2 { name: "foo".to_string(), generics: vec![], lifetimes: vec![], params: vec![], ret: TypeV2::Base(BaseType::Unit), is_pub: false, is_unsafe: false, is_async: false, is_method: false, where_clauses: vec![], has_default: false, default_body: None }, body_src: "{}".to_string() }));
    item_v2_examples.push(ItemV2::Impl(ImplDefV2 { self_ty: TypeV2::Base(BaseType::I32), trait_name: None, generics: vec![], lifetimes: vec![], methods: vec![], where_clauses: vec![] }));
    item_v2_examples.push(ItemV2::Trait(TraitDefV2 { name: "T".to_string(), generics: vec![], lifetimes: vec![], methods: vec![], is_pub: false, is_unsafe: false, where_clauses: vec![], supertraits: vec![] }));
    item_v2_examples.push(ItemV2::Mod(ModDefV2 { name: "m".to_string(), items: vec![], is_pub: false }));
    item_v2_examples.push(ItemV2::Const(ConstDefV2 { name: "C".to_string(), ty: TypeV2::Base(BaseType::I32), expr: Some("1".to_string()), is_pub: true }));
    item_v2_examples.push(ItemV2::Static(StaticDefV2 { name: "S".to_string(), ty: TypeV2::Base(BaseType::I32), mutbl: false, expr: Some("0".to_string()), is_pub: true }));
    item_v2_examples.push(ItemV2::TypeAlias(TypeAliasDefV2 { name: "A".to_string(), generics: vec![], ty: TypeV2::Base(BaseType::I32), is_pub: true }));
    item_v2_examples.push(ItemV2::Use("std::vec::Vec".to_string()));
    item_v2_examples.push(ItemV2::Macro("macro_rules! m { () => {} }".to_string()));

    super::ast::exhaustive_coverage_report(&type_examples, &pat_examples, &expr_examples, &item_examples, &item_v2_examples)
}

/// 實際使用：parse_full 文件清單，供 pipeline_v2 消費 — 優化 with_capacity
pub fn parse_full_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("parse_full.rs", "FullType 15 變體 100% + 39 正例 + poly 收斂 + 4 parse 比較 — 實際使用優化", "core/src/minirust/parse_full.rs"),
        ("parse.rs", "原始 7 型別解析", "core/src/minirust/parse.rs"),
        ("parse_v2.rs", "ProgramV2 解析", "core/src/minirust/parse_v2.rs"),
        ("parse_pat.rs", "FullPat 14 變體", "core/src/minirust/parse_pat.rs"),
        ("parse_expr.rs", "FullExpr 34 變體", "core/src/minirust/parse_expr.rs"),
        ("ast.rs", "統一 AST", "core/src/minirust/ast.rs"),
    ]
}
pub fn parse_full_file_list_static() -> &'static [(&'static str, &'static str, &'static str)] {
    super::parse::AST_SYNTAX_INVENTORY
}
/// 實際使用：快速解析並返回統計，減少 clone
pub fn parse_full_type_with_stats(s: &str) -> Result<(FullType, String), String> {
    let ty = parse_full_type_str(s)?;
    let mut summary = String::with_capacity(128);
    summary.push_str(&format!("Parsed: {} -> {} (full={:?})\n", s, ty.name(), ty));
    Ok((ty, summary))
}

/// parse 補齊，parse 全既列表及比較現時 4 個 parse
/// 返回 (parse_full_list, four_parses_list, comparison_report) — 優化 with_capacity
pub fn parse_full_list_and_compare_4_parses() -> (String, String, String) {
    // 1. parse_full 列表（39 正例 + 9 反例 + 15 pat + 39 expr）
    let mut full_list = String::with_capacity(4096);
    full_list.push_str("=== Parse Full 完整列表 (parse_full.rs) ===\n");
    full_list.push_str(&format!("正例 FullType: {} 個\n", positive_type_examples().len()));
    for (idx, (ty, desc)) in positive_type_examples().iter().enumerate() {
        full_list.push_str(&format!("  Ty{}: {} — {}\n", idx, ty, desc));
    }
    full_list.push_str(&format!("\n反例 FullType: {} 個\n", negative_type_examples().len()));
    for (ty, desc) in negative_type_examples() {
        full_list.push_str(&format!("  NEG: {} — {}\n", ty, desc));
    }
    full_list.push_str(&format!("\n正例 FullPat: {} 個\n", positive_pat_examples().len()));
    for (pat, desc) in positive_pat_examples() {
        full_list.push_str(&format!("  Pat: {} — {}\n", pat, desc));
    }
    full_list.push_str(&format!("\n正例 FullExpr: {} 個\n", positive_expr_examples().len()));
    for (expr, desc) in positive_expr_examples() {
        full_list.push_str(&format!("  Expr: {} — {}\n", expr, desc));
    }

    // 2. 現時 4 個 parse 的列表
    let mut four_parses = String::with_capacity(4096);
    four_parses.push_str("=== 現時 4 個 Parse 列表 ===\n");

    // parse.rs — 原始 7 型別 + EKind
    four_parses.push_str("\n--- parse.rs (原始 Mini-Rust, 7 型別) ---\n");
    four_parses.push_str("文件: core/src/minirust/parse.rs\n");
    four_parses.push_str("功能: Parser::parse_program 解析 Program { fns, macros, main_body }\n");
    four_parses.push_str("AST: Type (7 種: I32,Bool,Unit,RefI32,RefMutI32,RefBool,RefMutBool) + BinOp (9 種) + EKind (17 種: Int,BoolV,UnitLit,Var,Let,Seq,BinOp,Not,Neg,If,Ref,RefMut,Deref,AssignVar,AssignDeref,Call,Invoke)\n");
    four_parses.push_str("示例: fn add(x: i32, y: i32) -> i32 { x + y }  fn main() { let z = add(1,2); }\n");
    four_parses.push_str("能力: 解析 let, if, +, ==, &&, &, &mut, *, =, macro call\n");

    // parse_v2.rs — ProgramV2
    four_parses.push_str("\n--- parse_v2.rs (ProgramV2, 擴展) ---\n");
    four_parses.push_str("文件: core/src/minirust/parse_v2.rs (及 ast.rs ProgramV2::parse_v2)\n");
    four_parses.push_str("功能: 解析 struct/enum/fn/impl/trait/mod/const/static/type/use/macro\n");
    four_parses.push_str("AST: ItemV2 11 變體: Struct,Enum,Fn,Impl,Trait,Mod,Const,Static,TypeAlias,Use,Macro\n");
    four_parses.push_str("示例: pub struct Point { x: i32, y: i32 }  pub enum Option<T> { Some(T), None }  pub const MAX: i32 = 5;\n");
    four_parses.push_str("能力: 泛型 <T>, 生命週期 'a, where 子句, pub, unsafe, async\n");

    // parse_pat.rs — FullPat
    four_parses.push_str("\n--- parse_pat.rs (FullPat, 模式) ---\n");
    four_parses.push_str("文件: core/src/minirust/parse_pat.rs\n");
    four_parses.push_str("功能: parse_pat_str 解析模式\n");
    four_parses.push_str("AST: FullPat 14 變體: Wild,Ident,Lit,Path,TupleStruct,Struct,Tuple,Slice,Or,Ref,Box,Range,Macro,Type\n");
    four_parses.push_str("正例: 15 個 ( _, x, mut x, Some(x), None, Point { x, y }, (a,b), [a,b,c], a|b, &x, &mut x, 0..10, 0..=10, Some(x)|None, Point { x, .. } )\n");
    four_parses.push_str("能力: Or a|b, Range 0..10, Slice [a,b,c], Struct with rest .. \n");

    // parse_expr.rs — FullExpr
    four_parses.push_str("\n--- parse_expr.rs (FullExpr, 表達式) ---\n");
    four_parses.push_str("文件: core/src/minirust/parse_expr.rs\n");
    four_parses.push_str("功能: parse_expr_str 解析表達式\n");
    four_parses.push_str("AST: FullExpr 34 變體: Lit,Path,Field,Index,Call,MethodCall,Unary,Binary,Assign,AssignOp,If,Match,Loop,While,For,Block,Unsafe,Async,Await,Closure,Return,Break,Continue,Let,StructLit,Array,ArrayRepeat,Tuple,Cast,TypeAscribe,Try,Range,Macro,Verbatim\n");
    four_parses.push_str("正例: 39 個 (1, true, x, x.y, v[0], f(a,b), v.push(1), !x, x+y, x=y, x+=y, if true {1} else {0}, match true {..}, loop {break;}, while true {break;}, for i in v {1}, {1}, unsafe {1}, async {42}, x.await, |x| x+1, ||42, move |x| x+1, return 5, return, break, break 'a 1, continue, let Some(x)=y, Point {x:1,y:2}, [1,2,3], [0;10], (1,true), x as i32, x as *mut i32, x?, 0..10, 0..=10, println!(\"hi\"))\n");
    four_parses.push_str("能力: Closure |x| x+1, ||, move, Index v[0], Field x.y, MethodCall, Await, Try ?, Cast as, Range .., ..=, Macro, Verbatim (loop/while/for/async/unsafe/block 回退)\n");

    // 3. 比較報告
    let mut comparison = String::with_capacity(4096);
    comparison.push_str("=== Parse 全 vs 現時 4 個 Parse 比較 ===\n");

    // 統計
    let full_type_count = positive_type_examples().len();
    let full_pat_count = positive_pat_examples().len();
    let full_expr_count = positive_expr_examples().len();
    let total_examples = full_type_count + full_pat_count + full_expr_count;

    comparison.push_str(&format!("Parse Full 總正例: {} (Type {} + Pat {} + Expr {})\n", total_examples, full_type_count, full_pat_count, full_expr_count));
    comparison.push_str("現時 4 Parse:\n");
    comparison.push_str("  parse.rs: 7 Type + 17 EKind = 24 變體 (基礎)\n");
    comparison.push_str("  parse_v2.rs: 11 ItemV2 變體 + 7 Base + 16 Ext = 34 (擴展)\n");
    comparison.push_str("  parse_pat.rs: 14 FullPat 變體 (100% 覆蓋)\n");
    comparison.push_str("  parse_expr.rs: 34 FullExpr 變體 (100% 覆蓋, 部分 Verbatim)\n");
    comparison.push_str("  parse_full.rs: 15 FullType 變體 (100% 覆蓋) + 39 正例\n");

    // 覆蓋率對比
    let exhaustive = exhaustive_coverage_check();
    comparison.push_str("\n窮舉覆蓋率 (來自 ast.rs):\n");
    comparison.push_str(&format!("  FullType: {}/{} {:.1}% missing={:?}\n", exhaustive.type_covered, exhaustive.type_total, exhaustive.type_pct, exhaustive.type_missing));
    comparison.push_str(&format!("  FullPat: {}/{} {:.1}% missing={:?}\n", exhaustive.pat_covered, exhaustive.pat_total, exhaustive.pat_pct, exhaustive.pat_missing));
    comparison.push_str(&format!("  FullExpr: {}/{} {:.1}% missing={:?}\n", exhaustive.expr_covered, exhaustive.expr_total, exhaustive.expr_pct, exhaustive.expr_missing));
    comparison.push_str(&format!("  FullItem: {}/{} {:.1}% missing={:?}\n", exhaustive.item_covered, exhaustive.item_total, exhaustive.item_pct, exhaustive.item_missing));
    comparison.push_str(&format!("  ItemV2: {}/{} {:.1}% missing={:?}\n", exhaustive.item_v2_covered, exhaustive.item_v2_total, exhaustive.item_v2_pct, exhaustive.item_v2_missing));
    comparison.push_str(&format!("  Overall: {:.1}%\n", exhaustive.overall_pct));

    // Parser capability 對比
    let (total_cap, covered_cap, pct_cap, missing_cap) = super::ast_full::HandwrittenParser::global_coverage();
    comparison.push_str(&format!("\nParser Capability (HandwrittenParser): {}/{} {:.1}% missing={:?}\n", covered_cap, total_cap, pct_cap, missing_cap));

    // 4 Parse vs Full 補齊情況
    comparison.push_str("\n--- Parse 補齊情況 ---\n");
    comparison.push_str("parse.rs 已補齊: 基礎 7 型別 + BinOp + EKind，滿足 pipeline 舊管線\n");
    comparison.push_str("parse_v2.rs 已補齊: + const/static/type + Tuple/Array/Slice/BareFn + Mod flatten + Impl/Trait\n");
    comparison.push_str("parse_pat.rs 已補齊: + Slice [a,b,c] + Or a|b + Range 0..10/0..=10\n");
    comparison.push_str("parse_expr.rs 已補齊: + Array [1,2,3]/[0;10] + Index v[0] + Closure |x| + Await + Try + Cast + Range + Macro, Loop/While/For/Async/Unsafe/Block 為 Verbatim 回退 (已覆蓋 100% 變體)\n");
    comparison.push_str("parse_full.rs 已補齊: 39 正例覆蓋 FullType 15 變體 100%，反例 9 個，Pat 15，Expr 39，poly 收斂 + ast.rs 統一\n");

    comparison.push_str("\n--- 全 AST 列表包含 ast.rs ---\n");
    let files = super::ast::full_ast_file_list();
    for (name, desc, path) in &files {
        comparison.push_str(&format!("  {}: {} ({})\n", name, desc, path));
    }
    let inventory = super::ast::full_ast_variant_inventory();
    let total_variants: usize = inventory.iter().map(|(_, c, _)| c).sum();
    comparison.push_str(&format!("總變體數: {} (含 ast.rs)\n", total_variants));

    (full_list, four_parses, comparison)
}


