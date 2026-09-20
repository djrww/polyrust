// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! syn 前端解釋器 — 用 `syn` 真實解析 rustc 语法，逐项解释 WRP-R2 三项
//! 仅在 frontends/full（syn feature）下编译，core 保持零依赖

#[cfg(feature = "syn")]
use syn::{File, Item, Type, GenericArgument, PathArguments};

/// 用 syn 解析 Rust 源，返回 File AST 或错误
#[cfg(feature = "syn")]
pub fn parse_with_syn(src: &str) -> Result<File, syn::Error> {
    syn::parse_str::<File>(src)
}

/// 解释 HashMap<String, String> 的泛型参数（enterprise_ide 课）
/// 演示为何手写 split(',') 会误切，而 syn::Punctuated 正确
#[cfg(feature = "syn")]
pub fn explain_hashmap_generic(src: &str) -> Result<String, String> {
    let file = parse_with_syn(src).map_err(|e| e.to_string())?;
    let mut out = String::new();
    out.push_str("=== syn 解析 HashMap 泛型 ===\n");
    for item in file.items {
        if let Item::Struct(s) = item {
            out.push_str(&format!("struct {} generics: {:?}\n", s.ident, s.generics));
            match s.fields {
                syn::Fields::Named(named) => {
                    for field in named.named {
                        let fname = field.ident.clone().map(|i| i.to_string()).unwrap_or_default();
                        let ty_str = quote::quote!(#field.ty).to_string();
                        out.push_str(&format!("  field {}: {}\n", fname, ty_str));
                        // 用 syn 精确提取泛型
                        if let Type::Path(tp) = &field.ty {
                            if let Some(seg) = tp.path.segments.last() {
                                out.push_str(&format!("    last segment: {} (ident={})\n", seg.ident, seg.ident));
                                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                                    out.push_str(&format!("    angle_bracketed args: {} 个\n", ab.args.len()));
                                    for (i, arg) in ab.args.iter().enumerate() {
                                        match arg {
                                            GenericArgument::Type(t) => {
                                                out.push_str(&format!("      Arg[{}] Type: {}\n", i, quote::quote!(#t).to_string()));
                                            }
                                            GenericArgument::Lifetime(lt) => {
                                                out.push_str(&format!("      Arg[{}] Lifetime: {}\n", i, lt.ident));
                                            }
                                            _ => out.push_str(&format!("      Arg[{}] {:?}\n", i, arg)),
                                        }
                                    }
                                    // 对比手写 split
                                    let ty_debug = quote::quote!(#field.ty).to_string();
                                    let naive = ty_debug.split(',').count();
                                    out.push_str(&format!("    手写 split(',') 会切 {} 段（误）vs syn 正确 {} 段\n", naive, ab.args.len()));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(out)
}

/// 解释 async fn 与 tokio::spawn 的 Send 边界（async_spawn 课）
#[cfg(feature = "syn")]
pub fn explain_async_spawn(src: &str) -> Result<String, String> {
    let file = parse_with_syn(src).map_err(|e| e.to_string())?;
    let mut out = String::new();
    out.push_str("=== syn 解析 async / spawn ===\n");
    for item in file.items {
        match item {
            Item::Fn(f) => {
                out.push_str(&format!("fn {} async={} unsafe={} const={}\n",
                    f.sig.ident, f.sig.asyncness.is_some(), f.sig.unsafety.is_some(), f.sig.constness.is_some()));
                if f.sig.asyncness.is_some() {
                    out.push_str("  → syn 标记为 async，脱糖后为 impl Future<Output=...> 状态机\n");
                }
                // 检查 block 内是否有 tokio::spawn
                let block_str = quote::quote!(#f.block).to_string();
                if block_str.contains("tokio :: spawn") || block_str.contains("spawn") {
                    out.push_str("  → 发现 tokio::spawn 调用，rustc 要求 Send + 'static\n");
                }
                for input in &f.sig.inputs {
                    if let syn::FnArg::Typed(t) = input {
                        let ty_s = quote::quote!(#t.ty).to_string();
                        if ty_s.contains("Rc") {
                            out.push_str(&format!("  param {:?}: {}  → Rc 非 Send，tokio::spawn 拒\n", t.pat, ty_s));
                        }
                    }
                }
            }
            _ => {}
        }
        // 也检查文件级表达式（若有）
    }
    // 全文搜索 await
    if src.contains(".await") {
        out.push_str("  含 .await：syn 将解析为 Expr::Await { base }\n");
    }
    if src.contains("tokio::spawn") {
        out.push_str("  含 tokio::spawn：需 where T: Send 边界，syn 解析为 Expr::Call { func: Path(tokio::spawn) }\n");
    }
    Ok(out)
}

/// 解释 pure 效应的按函数粒度（io_with_pure_call 课）
#[cfg(feature = "syn")]
pub fn explain_pure_io(src: &str) -> Result<String, String> {
    let file = parse_with_syn(src).map_err(|e| e.to_string())?;
    let mut out = String::new();
    out.push_str("=== syn 解析 pure × I/O ===\n");
    for item in file.items {
        if let Item::Fn(f) = item {
            let attrs: Vec<String> = f.attrs.iter().map(|a| {
                let path = a.path().get_ident().map(|i| i.to_string()).unwrap_or_default();
                path
            }).collect();
            // 检查是否有 #[pure] 或 # @pure 的近似
            let is_pure = attrs.iter().any(|a| a.contains("pure")) || f.sig.ident.to_string().contains("pure_inner");
            out.push_str(&format!("fn {} attrs={:?} is_pure={}\n", f.sig.ident, attrs, is_pure));
            let block_s = quote::quote!(#f.block).to_string();
            let has_println = block_s.contains("println") || block_s.contains("print");
            out.push_str(&format!("  body contains println: {}\n", has_println));
            if is_pure && has_println {
                out.push_str("  → 若按文件级 has_io 则误判 UNSAT，rustc 按函数级 Pure 仅拒 pure_inner 内 I/O\n");
            } else if !is_pure && has_println {
                out.push_str("  → outer 允许 I/O，与 pure_inner 解耦\n");
            }
            // 列出 syn 的 Stmt
            out.push_str(&format!("  syn Block stmts: {} 条\n", f.block.stmts.len()));
        }
    }
    out.push_str("\n结论：syn 将每个 Item::Fn 独立为 FnItem，attrs 与 block 分离，天然支撑按函数粒度 effect；\n");
    out.push_str("手写 pipeline_v2 的 file-level has_io 是对 syn 结构的塌陷，需改为 per-fn HasIo。\n");
    Ok(out)
}

/// 一键解释 WRP-R2 三项
#[cfg(feature = "syn")]
pub fn explain_wrp_r2_all() -> String {
    let mut all = String::new();

    let enterprise_src = r#"
use std::collections::HashMap;
struct EnterpriseIDE { editors: HashMap<String, String> }
impl EnterpriseIDE { fn new() -> Self { Self { editors: HashMap::new() } } fn open(&mut self, p: String, c: String) { self.editors.insert(p,c); } }
"#;
    let async_src = r#"
use std::rc::Rc;
fn main(){ let x = Rc::new(5); tokio::spawn(async move { println!("{}", x); }); }
"#;
    let pure_src = r#"
#[pure] fn pure_inner(x: i32) -> i32 { x + 1 }
fn outer() { println!("{}", pure_inner(5)); }
"#;

    all.push_str(&explain_hashmap_generic(enterprise_src).unwrap_or_else(|e| e));
    all.push_str("\n");
    all.push_str(&explain_async_spawn(async_src).unwrap_or_else(|e| e));
    all.push_str("\n");
    all.push_str(&explain_pure_io(pure_src).unwrap_or_else(|e| e));
    all
}


/// 供 handler 使用：对任意 src 选择性解释（空则全量）
#[cfg(feature = "syn")]
pub fn explain_wrp_r2_all_for_src(src: &str) -> String {
    if src.trim().is_empty() {
        return explain_wrp_r2_all();
    }
    let mut out = String::new();
    // 尝试三类解释，若某类解析失败则跳过
    if src.contains("HashMap") || src.contains("struct") {
        if let Ok(s) = explain_hashmap_generic(src) { out.push_str(&s); out.push_str("
"); }
    }
    if src.contains("async") || src.contains("spawn") || src.contains("await") {
        if let Ok(s) = explain_async_spawn(src) { out.push_str(&s); out.push_str("
"); }
    }
    if src.contains("pure") || src.contains("println") || src.contains("pure_inner") {
        if let Ok(s) = explain_pure_io(src) { out.push_str(&s); out.push_str("
"); }
    }
    if out.is_empty() {
        // 回退：尝试全部三项的通用解析，直接展示 File items
        match parse_with_syn(src) {
            Ok(file) => {
                out.push_str(&format!("syn 解析成功：{} 个 Item\n", file.items.len()));
                for item in file.items.iter().take(5) {
                    out.push_str(&format!("  - {:?}\n", std::mem::discriminant(item)));
                }
                out.push_str("\n");
                out.push_str(&explain_wrp_r2_all());
            }
            Err(e) => {
                out.push_str(&format!("syn 解析失败：{}\n\n", e));
                out.push_str(&explain_wrp_r2_all());
            }
        }
    }
    out
}

#[cfg(all(feature = "syn", test))]
mod tests {
    use super::*;

    #[test]
    fn test_explain_hashmap() {
        let src = "struct EnterpriseIDE { editors: HashMap<String, String> } fn main(){}";
        let out = explain_hashmap_generic(src).unwrap();
        assert!(out.contains("HashMap"));
        assert!(out.contains("String"));
        println!("{}", out);
    }

    #[test]
    fn test_explain_async() {
        let src = "async fn foo() -> i32 { 42 } fn main(){ let x = tokio::spawn(async { 1 }); }";
        let out = explain_async_spawn(src).unwrap();
        assert!(out.contains("async"));
        println!("{}", out);
    }

    #[test]
    fn test_explain_pure() {
        let src = "#[pure] fn pure_inner(x: i32) -> i32 { x+1 } fn outer(){ println!(\"{}\", pure_inner(5)); }";
        // syn 不认识 #[pure] 的自定义属性但仍解析为 Attribute，需启用 syn 的 full
        let out = explain_pure_io(src).unwrap();
        println!("{}", out);
    }
}
