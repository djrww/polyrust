// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! syn 前端：真 Rust 解析 -> ProgramV2
//! 保持 core 零依賴，syn 僅在 frontend
//! 混合路線：DSL metadata 仍用 # @key 解析，body 用 syn

use polyrust_core::minirust::ast_v2::{ProgramV2, ItemV2, StructDefV2, EnumDefV2, VariantV2, FnDefV2, FnSigV2, ImplDefV2, TraitDefV2, ModDefV2};
use polyrust_core::minirust::universe::{TypeV2, BaseType, ExtType, parse_type_v2};
use syn::{Item, File, Type, Fields, GenericParam, Visibility};

fn syn_type_to_typev2(ty: &Type) -> TypeV2 {
    // 簡化映射，盡量保留原文本
    let s = quote::quote!(#ty).to_string();
    // 清理空格
    let s = s.replace(" ", "");
    // 常見映射
    if let Ok(t) = parse_type_v2(&s) {
        return t;
    }
    // fallback: 嘗試原始文本
    let raw = s.clone();
    // 啟發：Vec<...>, Option<...>, Result<...>, HashMap<...>, &..., *mut..., *const...
    if raw.starts_with("Vec<") {
        if let Ok(t) = parse_type_v2(&raw) { return t; }
        // 提取內部
        let inner = raw.trim_start_matches("Vec<").trim_end_matches(">");
        let inner_ty = syn_type_to_typev2(&syn::parse_str::<Type>(inner).unwrap_or(Type::Verbatim(Default::default())));
        return TypeV2::Ext(ExtType::Vec(Box::new(inner_ty)));
    }
    if raw == "String" {
        return TypeV2::Ext(ExtType::String);
    }
    if raw.starts_with("HashMap<") {
        if let Ok(t) = parse_type_v2(&raw) { return t; }
        return TypeV2::Ext(ExtType::HashMap(Box::new(TypeV2::Base(BaseType::I32)), Box::new(TypeV2::Base(BaseType::I32))));
    }
    if raw.starts_with("Option<") {
        let inner = raw.trim_start_matches("Option<").trim_end_matches(">");
        let inner_ty = syn::parse_str::<Type>(inner).map(|t| syn_type_to_typev2(&t)).unwrap_or(TypeV2::Base(BaseType::I32));
        return TypeV2::Ext(ExtType::Option(Box::new(inner_ty)));
    }
    if raw.starts_with("Result<") {
        return TypeV2::Ext(ExtType::Result(Box::new(TypeV2::Base(BaseType::I32)), Box::new(TypeV2::Ext(ExtType::String))));
    }
    if raw.starts_with("&'") {
        // &'a mut T / &'a T
        let is_mut = raw.contains("mut");
        let mut lifetime = None;
        // 提取 lifetime
        if let Some(start) = raw.find('\'') {
            let rest = &raw[start..];
            let end = rest.find([' ', '&']).unwrap_or(rest.len());
            let lt = rest[..end].trim().trim_end_matches([',', '>']).to_string();
            // 確保以 ' 開頭
            if lt.starts_with('\'') {
                lifetime = Some(lt);
            }
        }
        // 內部類型：取最後一個標識
        let inner_str = if raw.contains("i32") { "i32" } else if raw.contains("bool") { "bool" } else if raw.contains("String") { "String" } else { "i32" };
        let inner = parse_type_v2(inner_str).unwrap_or(TypeV2::Base(BaseType::I32));
        return TypeV2::Ext(ExtType::RefExt { mutbl: is_mut, inner: Box::new(inner), lifetime });
    }
    if raw.starts_with("&mut") {
        let inner_str = if raw.contains("i32") { "i32" } else { "i32" };
        let inner = parse_type_v2(inner_str).unwrap_or(TypeV2::Base(BaseType::I32));
        return TypeV2::Ext(ExtType::RefExt { mutbl: true, inner: Box::new(inner), lifetime: None });
    }
    if raw.starts_with("&") {
        let inner_str = if raw.contains("i32") { "i32" } else if raw.contains("str") { "String" } else { "i32" };
        let inner = parse_type_v2(inner_str).unwrap_or(TypeV2::Base(BaseType::I32));
        return TypeV2::Ext(ExtType::RefExt { mutbl: false, inner: Box::new(inner), lifetime: None });
    }
    if raw.starts_with("*mut") {
        let inner_str = raw.trim_start_matches("*mut").trim();
        let inner = parse_type_v2(inner_str).unwrap_or(TypeV2::Base(BaseType::I32));
        return TypeV2::Ext(ExtType::RawPtr { mutbl: true, inner: Box::new(inner) });
    }
    if raw.starts_with("*const") {
        let inner_str = raw.trim_start_matches("*const").trim();
        let inner = parse_type_v2(inner_str).unwrap_or(TypeV2::Base(BaseType::I32));
        return TypeV2::Ext(ExtType::RawPtr { mutbl: false, inner: Box::new(inner) });
    }
    // Struct/Enum 名
    if raw.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        let name = raw.split('<').next().unwrap_or(&raw).split("::").last().unwrap_or(&raw).to_string();
        // 判斷是否已知泛型容器，否則視為 struct
        if ["Point", "User", "MyType", "Foo"].contains(&name.as_str()) || name.len() > 1 {
            return TypeV2::Ext(ExtType::Struct { name, args: vec![] });
        }
        return TypeV2::Ext(ExtType::GenericParam(name));
    }
    TypeV2::Base(BaseType::I32)
}

fn generics_to_strings(generics: &syn::Generics) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut gen = vec![];
    let mut lifetimes = vec![];
    let mut where_clauses = vec![];
    for param in &generics.params {
        match param {
            GenericParam::Type(t) => gen.push(t.ident.to_string()),
            GenericParam::Lifetime(lt) => lifetimes.push(format!("'{}", lt.lifetime.ident)),
            GenericParam::Const(c) => gen.push(c.ident.to_string()),
        }
    }
    if let Some(where_clause) = &generics.where_clause {
        for pred in &where_clause.predicates {
            where_clauses.push(quote::quote!(#pred).to_string());
        }
    }
    (gen, lifetimes, where_clauses)
}

fn parse_struct_item(item: &syn::ItemStruct) -> StructDefV2 {
    let name = item.ident.to_string();
    let is_pub = matches!(item.vis, Visibility::Public(_));
    let (generics, lifetimes, where_clauses) = generics_to_strings(&item.generics);
    let mut fields = vec![];
    match &item.fields {
        Fields::Named(named) => {
            for f in &named.named {
                if let Some(ident) = &f.ident {
                    let fname = ident.to_string();
                    let fty = syn_type_to_typev2(&f.ty);
                    fields.push((fname, fty));
                }
            }
        }
        Fields::Unnamed(unnamed) => {
            for (i, f) in unnamed.unnamed.iter().enumerate() {
                let fname = format!("_{}", i);
                let fty = syn_type_to_typev2(&f.ty);
                fields.push((fname, fty));
            }
        }
        Fields::Unit => {}
    }
    StructDefV2 { name, generics, lifetimes, fields, is_pub, where_clauses }
}

fn parse_enum_item(item: &syn::ItemEnum) -> EnumDefV2 {
    let name = item.ident.to_string();
    let is_pub = matches!(item.vis, Visibility::Public(_));
    let (generics, lifetimes, where_clauses) = generics_to_strings(&item.generics);
    let mut variants = vec![];
    for v in &item.variants {
        let vname = v.ident.to_string();
        let mut fields = vec![];
        match &v.fields {
            Fields::Named(named) => {
                for f in &named.named {
                    fields.push(syn_type_to_typev2(&f.ty));
                }
            }
            Fields::Unnamed(unnamed) => {
                for f in &unnamed.unnamed {
                    fields.push(syn_type_to_typev2(&f.ty));
                }
            }
            Fields::Unit => {}
        }
        variants.push(VariantV2 { name: vname, fields, discriminant: None });
    }
    EnumDefV2 { name, generics, lifetimes, variants, is_pub, where_clauses }
}

fn parse_fn_sig(sig: &syn::Signature) -> FnSigV2 {
    let name = sig.ident.to_string();
    let is_pub = false; // 簡化
    let is_async = sig.asyncness.is_some();
    let is_unsafe = sig.unsafety.is_some();
    let (generics, lifetimes, where_clauses) = generics_to_strings(&sig.generics);
    let mut params = vec![];
    for input in &sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => {
                params.push(("self".to_string(), TypeV2::Ext(ExtType::GenericParam("Self".to_string()))));
            }
            syn::FnArg::Typed(pat_ty) => {
                let pname = quote::quote!(#pat_ty.pat).to_string();
                let pty = syn_type_to_typev2(&pat_ty.ty);
                params.push((pname, pty));
            }
        }
    }
    let ret = match &sig.output {
        syn::ReturnType::Default => TypeV2::Base(BaseType::Unit),
        syn::ReturnType::Type(_, ty) => syn_type_to_typev2(ty),
    };
    FnSigV2 {
        name,
        generics,
        lifetimes,
        params,
        ret,
        is_pub,
        is_unsafe,
        is_async,
        is_method: false,
        where_clauses,
        has_default: false,
        default_body: None,
    }
}

fn parse_fn_item(item: &syn::ItemFn) -> FnDefV2 {
    let sig = parse_fn_sig(&item.sig);
    let body_src = quote::quote!(#item.block).to_string();
    FnDefV2 { sig, body_src }
}

fn parse_impl_item(item: &syn::ItemImpl) -> ImplDefV2 {
    let self_ty = syn_type_to_typev2(&item.self_ty);
    let trait_name = item.trait_.as_ref().map(|(_, path, _)| {
        quote::quote!(#path).to_string().replace(" ", "")
    });
    let (generics, lifetimes, where_clauses) = generics_to_strings(&item.generics);
    let mut methods = vec![];
    for impl_item in &item.items {
        if let syn::ImplItem::Fn(m) = impl_item {
            let mut sig = parse_fn_sig(&m.sig);
            sig.is_method = true;
            let body_src = quote::quote!(#m.block).to_string();
            methods.push(FnDefV2 { sig, body_src });
        }
    }
    ImplDefV2 { self_ty, trait_name, generics, lifetimes, methods, where_clauses }
}

fn parse_trait_item(item: &syn::ItemTrait) -> TraitDefV2 {
    let name = item.ident.to_string();
    let is_pub = matches!(item.vis, Visibility::Public(_));
    let is_unsafe = item.unsafety.is_some();
    let (generics, lifetimes, where_clauses) = generics_to_strings(&item.generics);
    let mut methods = vec![];
    let mut supertraits = vec![];
    for bound in &item.supertraits {
        supertraits.push(quote::quote!(#bound).to_string());
    }
    for trait_item in &item.items {
        if let syn::TraitItem::Fn(m) = trait_item {
            let sig = parse_fn_sig(&m.sig);
            methods.push(sig);
        }
    }
    TraitDefV2 { name, generics, lifetimes, methods, is_pub, is_unsafe, where_clauses, supertraits }
}

fn parse_mod_item(item: &syn::ItemMod) -> ModDefV2 {
    let name = item.ident.to_string();
    let is_pub = matches!(item.vis, Visibility::Public(_));
    let mut items = vec![];
    if let Some((_, content_items)) = &item.content {
        for inner in content_items {
            if let Some(v2) = syn_item_to_v2(inner) {
                items.extend(v2);
            }
        }
    }
    ModDefV2 { name, items, is_pub }
}

fn syn_item_to_v2(item: &Item) -> Option<Vec<ItemV2>> {
    match item {
        Item::Struct(s) => Some(vec![ItemV2::Struct(parse_struct_item(s))]),
        Item::Enum(e) => Some(vec![ItemV2::Enum(parse_enum_item(e))]),
        Item::Fn(f) => {
            let fd = parse_fn_item(f);
            if fd.sig.name == "main" {
                // main 單獨處理，調用方會收集
                Some(vec![ItemV2::Fn(fd)])
            } else {
                Some(vec![ItemV2::Fn(fd)])
            }
        }
        Item::Impl(i) => Some(vec![ItemV2::Impl(parse_impl_item(i))]),
        Item::Trait(t) => Some(vec![ItemV2::Trait(parse_trait_item(t))]),
        Item::Mod(m) => Some(vec![ItemV2::Mod(parse_mod_item(m))]),
        Item::Use(u) => Some(vec![ItemV2::Use(quote::quote!(#u).to_string())]),
        _ => None,
    }
}

/// 主入口：真 Rust 源碼 -> ProgramV2，失敗回退到原 parse_v2
pub fn parse_rust_to_program_v2(src: &str) -> Result<ProgramV2, String> {
    // 先嘗試 syn 解析
    let file: File = syn::parse_str(src).map_err(|e| format!("syn parse error: {}", e))?;
    let mut prog = ProgramV2::new();
    let mut main_fn: Option<FnDefV2> = None;

    for item in &file.items {
        if let Some(mut v2_items) = syn_item_to_v2(item) {
            for it in v2_items.drain(..) {
                match it {
                    ItemV2::Fn(f) if f.sig.name == "main" => {
                        main_fn = Some(f);
                    }
                    other => {
                        // 收集類型到宇宙
                        match &other {
                            ItemV2::Struct(s) => {
                                for (_, ty) in &s.fields {
                                    prog.universe.insert_closure(ty.clone());
                                }
                                prog.universe.insert_closure(TypeV2::Ext(ExtType::Struct { name: s.name.clone(), args: vec![] }));
                            }
                            ItemV2::Enum(e) => {
                                for v in &e.variants {
                                    for f in &v.fields {
                                        prog.universe.insert_closure(f.clone());
                                    }
                                }
                                prog.universe.insert_closure(TypeV2::Ext(ExtType::Enum { name: e.name.clone(), args: vec![] }));
                            }
                            ItemV2::Fn(f) => {
                                for (_, ty) in &f.sig.params {
                                    prog.universe.insert_closure(ty.clone());
                                }
                                prog.universe.insert_closure(f.sig.ret.clone());
                            }
                            ItemV2::Impl(im) => {
                                prog.universe.insert_closure(im.self_ty.clone());
                            }
                            _ => {}
                        }
                        prog.items.push(other);
                    }
                }
            }
        }
    }
    prog.main = main_fn;
    // 如果什麼都沒解析到，返回錯誤讓上層回退
    if prog.items.is_empty() && prog.main.is_none() {
        return Err("syn parsed empty".to_string());
    }
    Ok(prog)
}

/// 混合解析：先 DSL 載入 metadata，再用 syn 解析 body
pub fn parse_hybrid(src: &str) -> Result<(ProgramV2, polyrust_core::dsl::PolySource), String> {
    let poly_src = polyrust_core::dsl::load_poly(src).map_err(|e| format!("dsl load error: {}", e))?;
    // 嘗試 syn 解析 poly_src.source (已 resolve import 後的純 Rust)
    match parse_rust_to_program_v2(&poly_src.source) {
        Ok(prog) => Ok((prog, poly_src)),
        Err(e) => {
            // 回退到原 parse_v2
            let prog = ProgramV2::parse_v2(&poly_src.source).map_err(|e2| format!("both syn and v2 failed: syn={}, v2={}", e, e2))?;
            Ok((prog, poly_src))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_struct() {
        let src = r#"
            struct Point { x: i32, y: i32 }
            fn main() { let p = Point { x: 3, y: 4 }; }
        "#;
        let prog = parse_rust_to_program_v2(src).unwrap();
        assert!(prog.items.iter().any(|it| matches!(it, ItemV2::Struct(s) if s.name=="Point")));
        println!("{}", prog.display());
    }

    #[test]
    fn test_syn_enum_match() {
        let src = r#"
            enum Option<T> { Some(T), None }
            fn main() {
                let x = Option::Some(5);
                let y = match x { Some(v) => v, None => 0 };
            }
        "#;
        let prog = parse_rust_to_program_v2(src).unwrap();
        assert!(prog.universe.n_types() > 7);
    }

    #[test]
    fn test_syn_lifetime() {
        let src = r#"
            fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { x }
            fn main() {}
        "#;
        let prog = parse_rust_to_program_v2(src).unwrap();
        assert!(prog.items.iter().any(|it| matches!(it, ItemV2::Fn(f) if f.sig.lifetimes.contains(&"'a".to_string()))));
    }
}
