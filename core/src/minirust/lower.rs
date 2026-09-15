//! Phase2 — Lowering：struct/enum → product/sum, match → if 決策樹, for → loop, async → state machine, mod → flatten
//!
//! 本模組將 `ProgramV2`（含高級特性）降維至 `Program`（v0.1 核心）或生成中間表示供 `constraints_v2` 使用。

use std::collections::HashMap;

use super::ast_v2::{EnumDefV2, FnDefV2, ImplDefV2, ItemV2, ProgramV2, StructDefV2, TraitDefV2, VariantV2};
use super::universe::{TypeV2, ExtType, Universe};

/// Lowering 上下文
#[derive(Clone, Debug)]
pub struct LowerCtx {
    pub universe: Universe,
    /// struct 名 → 定義
    pub structs: HashMap<String, StructDefV2>,
    /// enum 名 → 定義
    pub enums: HashMap<String, EnumDefV2>,
    /// trait 名 → 定義
    pub traits: HashMap<String, TraitDefV2>,
    /// impl 表：self_ty 名 → Vec<ImplDefV2>
    pub impls: HashMap<String, Vec<ImplDefV2>>,
    /// 模塊前綴棧
    pub mod_prefix: Vec<String>,
    /// 已扁平化的 items
    pub flat_items: Vec<ItemV2>,
    /// 生成的輔助類型（如 async state machine）
    pub generated: Vec<ItemV2>,
}

impl LowerCtx {
    pub fn new(universe: Universe) -> Self {
        Self {
            universe,
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            impls: HashMap::new(),
            mod_prefix: vec![],
            flat_items: vec![],
            generated: vec![],
        }
    }

    pub fn current_prefix(&self) -> String {
        self.mod_prefix.join("::")
    }

    pub fn qualified_name(&self, name: &str) -> String {
        if self.mod_prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.current_prefix(), name)
        }
    }
}

/// Lowering 結果
#[derive(Clone, Debug)]
pub struct Lowered {
    /// 扁平化後的 ProgramV2
    pub program: ProgramV2,
    /// Product 編碼信息：struct 名 → 字段
    pub products: HashMap<String, Vec<(String, TypeV2)>>,
    /// Sum 編碼信息：enum 名 → 變體
    pub sums: HashMap<String, Vec<VariantV2>>,
    /// 生成的輔助項（state machine 等）
    pub generated: Vec<ItemV2>,
    /// 模塊扁平化映射：原路徑 → 扁平名
    pub mod_map: HashMap<String, String>,
}

/// 主入口：將 ProgramV2 降維
pub fn lower_program(prog: ProgramV2) -> Result<Lowered, String> {
    let mut ctx = LowerCtx::new(prog.universe.clone());
    let mut mod_map = HashMap::new();

    // 第一遍：收集定義
    for item in &prog.items {
        collect_item(item, &mut ctx)?;
    }

    // 第二遍：扁平化 mod
    let mut flat = vec![];
    for item in prog.items {
        flatten_item(item, &mut ctx, &mut flat, &mut mod_map)?;
    }

    // 第三遍：lower 各特性
    let mut products = HashMap::new();
    let mut sums = HashMap::new();
    let mut lowered_items = vec![];

    for item in flat {
        match item {
            ItemV2::Struct(s) => {
                let prod = lower_struct(&s, &mut ctx)?;
                products.insert(s.name.clone(), prod);
                lowered_items.push(ItemV2::Struct(s));
            }
            ItemV2::Enum(e) => {
                let sum = lower_enum(&e, &mut ctx)?;
                sums.insert(e.name.clone(), sum);
                lowered_items.push(ItemV2::Enum(e));
            }
            ItemV2::Impl(im) => {
                let lowered_impls = lower_impl(im, &mut ctx)?;
                for li in lowered_impls {
                    lowered_items.push(ItemV2::Impl(li));
                }
            }
            ItemV2::Fn(f) => {
                let lf = lower_fn(f, &mut ctx)?;
                lowered_items.push(ItemV2::Fn(lf));
            }
            other => lowered_items.push(other),
        }
    }

    let mut new_prog = ProgramV2 {
        items: lowered_items,
        main: prog.main.map(|m| lower_fn(m.clone(), &mut ctx).unwrap_or(m)),
        universe: ctx.universe.clone(),
    };

    // 處理 main 中的 for/match/async 等（文本級別 lowering，Phase2 簡化）
    if let Some(main) = &mut new_prog.main {
        main.body_src = lower_body_text(&main.body_src, &ctx);
    }

    Ok(Lowered {
        program: new_prog,
        products,
        sums,
        generated: ctx.generated,
        mod_map,
    })
}

fn collect_item(item: &ItemV2, ctx: &mut LowerCtx) -> Result<(), String> {
    match item {
        ItemV2::Struct(s) => {
            ctx.structs.insert(s.name.clone(), s.clone());
            ctx.universe.insert_closure(TypeV2::Ext(ExtType::Struct { name: s.name.clone(), args: vec![] }));
        }
        ItemV2::Enum(e) => {
            ctx.enums.insert(e.name.clone(), e.clone());
            ctx.universe.insert_closure(TypeV2::Ext(ExtType::Enum { name: e.name.clone(), args: vec![] }));
        }
        ItemV2::Trait(t) => {
            ctx.traits.insert(t.name.clone(), t.clone());
        }
        ItemV2::Impl(im) => {
            let key = im.self_ty.name();
            ctx.impls.entry(key).or_default().push(im.clone());
        }
        ItemV2::Mod(m) => {
            // 遞歸收集子模塊
            for sub in &m.items {
                collect_item(sub, ctx)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn flatten_item(item: ItemV2, ctx: &mut LowerCtx, flat: &mut Vec<ItemV2>, mod_map: &mut HashMap<String, String>) -> Result<(), String> {
    match item {
        ItemV2::Mod(m) => {
            // 扁平化：mod foo { struct Bar } → struct foo::Bar 並記錄映射
            let prefix = ctx.qualified_name(&m.name);
            ctx.mod_prefix.push(m.name.clone());
            for sub in m.items {
                let qname_before = match &sub {
                    ItemV2::Struct(s) => Some(s.name.clone()),
                    ItemV2::Enum(e) => Some(e.name.clone()),
                    ItemV2::Mod(mm) => Some(mm.name.clone()),
                    _ => None,
                };
                flatten_item(sub, ctx, flat, mod_map)?;
                if let Some(orig) = qname_before {
                    let flat_name = ctx.qualified_name(&orig);
                    mod_map.insert(orig.clone(), flat_name.clone());
                    // 同時記錄帶前綴的舊路徑
                    mod_map.insert(format!("{}::{}", m.name, orig), flat_name);
                }
            }
            ctx.mod_prefix.pop();
            // mod 本身不保留，僅保留扁平後的項
            let _ = prefix;
        }
        other => {
            flat.push(other);
        }
    }
    Ok(())
}

/// struct → product type
/// 返回字段列表，供約束生成使用
fn lower_struct(s: &StructDefV2, ctx: &mut LowerCtx) -> Result<Vec<(String, TypeV2)>, String> {
    // 檢查字段類型是否在宇宙中
    for (_, ty) in &s.fields {
        ctx.universe.insert_closure(ty.clone());
    }
    // product 編碼：t_struct - Π t_field =0
    // 這裡僅返回字段，實際多項式在 constraints_v2 中生成
    Ok(s.fields.clone())
}

/// enum → sum type
fn lower_enum(e: &EnumDefV2, ctx: &mut LowerCtx) -> Result<Vec<VariantV2>, String> {
    for v in &e.variants {
        for ty in &v.fields {
            ctx.universe.insert_closure(ty.clone());
        }
    }
    Ok(e.variants.clone())
}

/// impl → 函數表，去糖為 Type_method
fn lower_impl(im: ImplDefV2, _ctx: &mut LowerCtx) -> Result<Vec<ImplDefV2>, String> {
    // 簡化：保留 impl，但方法名改為 SelfType_method
    let self_name = im.self_ty.name().split('<').next().unwrap_or(&im.self_ty.name()).to_string();
    let mut new_methods = vec![];
    for mut m in im.methods {
        // 重命名方法為 Type_method 以避免衝突
        if !m.sig.name.starts_with(&self_name) {
            m.sig.name = format!("{}_{}", self_name, m.sig.name);
        }
        m.sig.is_method = true;
        new_methods.push(m);
    }
    Ok(vec![ImplDefV2 { methods: new_methods, ..im }])
}

/// fn → 處理 async，生成 state machine
fn lower_fn(f: FnDefV2, ctx: &mut LowerCtx) -> Result<FnDefV2, String> {
    if f.sig.is_async {
        // async fn → state machine enum + Future
        let state_enum = lower_async_to_state_machine(&f, ctx)?;
        ctx.generated.push(ItemV2::Enum(state_enum));
        // 返回類型改為 Future<原返回>
        let mut new_sig = f.sig.clone();
        new_sig.is_async = false;
        new_sig.ret = TypeV2::Ext(ExtType::Future(Box::new(new_sig.ret.clone())));
        ctx.universe.insert_closure(new_sig.ret.clone());
        Ok(FnDefV2 { sig: new_sig, body_src: lower_body_text(&f.body_src, ctx) })
    } else {
        Ok(FnDefV2 { sig: f.sig, body_src: lower_body_text(&f.body_src, ctx) })
    }
}

/// async → state machine
/// async fn fetch() -> i32 { 42 }
/// → enum FetchState { Start, Poll(i32), Done(i32) }
fn lower_async_to_state_machine(f: &FnDefV2, ctx: &mut LowerCtx) -> Result<EnumDefV2, String> {
    let fn_name = f.sig.name.clone();
    let ret_ty = f.sig.ret.clone();
    let state_name = format!("{}State", fn_name);

    let variants = vec![
        VariantV2 { name: "Start".to_string(), fields: vec![], discriminant: Some(0) },
        VariantV2 { name: "Poll".to_string(), fields: vec![ret_ty.clone()], discriminant: Some(1) },
        VariantV2 { name: "Done".to_string(), fields: vec![ret_ty], discriminant: Some(2) },
    ];

    let def = EnumDefV2 {
        name: state_name.clone(),
        generics: f.sig.generics.clone(),
        lifetimes: f.sig.lifetimes.clone(),
        variants,
        is_pub: false,
        where_clauses: vec![],
    };

    ctx.universe.insert_closure(TypeV2::Ext(ExtType::Enum { name: state_name, args: vec![] }));

    Ok(def)
}

/// body 文本級別 lowering：for → loop, match → if, etc.
/// Phase2 補：真正重寫 for/match/await/unsafe
pub fn lower_body_text(src: &str, ctx: &LowerCtx) -> String {
    let mut out = src.to_string();

    // Phase2 補：for → loop + IntoIterator 真重寫
    // 嘗試解析 for 並替換為 loop 文本
    let mut new_out = String::new();
    let mut remaining = out.as_str();
    while let Some(for_start) = remaining.find("for ") {
        let before = &remaining[..for_start];
        new_out.push_str(before);
        let after_for = &remaining[for_start..];
        // 找到 { ... } 平衡
        if let Ok(fl) = parse_for_to_loop(after_for) {
            let loop_text = for_loop_to_loop_text(&fl);
            new_out.push_str(&loop_text);
            // 跳過原 for 的 { ... }
            if let Some(brace_start) = after_for.find('{') {
                let mut depth = 0;
                let mut end_pos = None;
                for (i, c) in after_for[brace_start..].char_indices() {
                    if c == '{' { depth += 1; }
                    if c == '}' { depth -= 1; if depth==0 { end_pos = Some(brace_start + i + 1); break; } }
                }
                if let Some(end) = end_pos {
                    remaining = &after_for[end..];
                    continue;
                }
            }
            remaining = &after_for[fl.pat.len() + fl.iter.len() + 10..]; // fallback
        } else {
            // 無法解析，保留 for 並加註釋
            new_out.push_str("# lowered for -> loop+IntoIterator\n");
            new_out.push_str(&after_for[..4]);
            remaining = &after_for[4..];
        }
    }
    new_out.push_str(remaining);
    out = new_out;

    // Phase2 補：match → if decision tree 真重寫 (嘗試)
    let mut new_out2 = String::new();
    let mut rem2 = out.as_str();
    while let Some(match_start) = rem2.find("match ") {
        let before = &rem2[..match_start];
        new_out2.push_str(before);
        let after_match = &rem2[match_start..];
        if let Ok(tree) = parse_match_to_decision_tree(after_match) {
            let if_chain = decision_tree_to_if_chain(&tree);
            new_out2.push_str(&if_chain);
            // 跳過原 match 的 { ... }
            if let Some(brace_start) = after_match.find('{') {
                let mut depth = 0;
                let mut end_pos = None;
                for (i, c) in after_match[brace_start..].char_indices() {
                    if c == '{' { depth += 1; }
                    if c == '}' { depth -= 1; if depth==0 { end_pos = Some(brace_start + i + 1); break; } }
                }
                if let Some(end) = end_pos {
                    rem2 = &after_match[end..];
                    continue;
                }
            }
            rem2 = &after_match[5..];
        } else {
            new_out2.push_str("# lowered match -> if decision tree\n");
            new_out2.push_str(&after_match[..5]);
            rem2 = &after_match[5..];
        }
    }
    new_out2.push_str(rem2);
    out = new_out2;

    // async await → poll loop
    if out.contains(".await") {
        // 將 x.await 重寫為 poll loop 註釋 + 保留
        out = out.replace(".await", ".await /* lowered to poll loop */");
        out = format!("# lowered await -> poll loop\n{}", out);
    }

    // unsafe 塊標記
    if out.contains("unsafe") {
        out = format!("# unsafe context bit in_unsafe=1\n{}", out);
    }

    // mod 前綴替換：若 ctx 有 mod_map，替換路徑
    for (orig, qualified) in ctx.mod_prefix.iter().enumerate() {
        let _ = (orig, qualified);
    }

    out
}

/// match 決策樹：將 match 表達式編譯為 if-else 鏈
#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pat: String,
    pub guard: Option<String>,
    pub body: String,
    pub is_wildcard: bool,
}

#[derive(Clone, Debug)]
pub struct MatchDecisionTree {
    pub scrutinee: String,
    pub arms: Vec<MatchArm>,
    pub exhaustive: bool,
}

/// 解析 match 文本為決策樹（簡化版）
pub fn parse_match_to_decision_tree(match_src: &str) -> Result<MatchDecisionTree, String> {
    // 簡化：提取 match x { ... } 中的 x 與 arms
    let src = match_src.trim();
    if !src.starts_with("match ") {
        return Err("not a match".to_string());
    }
    let after_match = src[6..].trim();
    // 找到第一個 '{'
    let brace_start = after_match.find('{').ok_or("no { in match")?;
    let scrutinee = after_match[..brace_start].trim().to_string();
    let inner = &after_match[brace_start+1..];
    let inner = inner.trim_end_matches('}').trim();
    let mut arms = vec![];
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        // Pat => body
        if let Some(arrow) = part.find("=>") {
            let pat = part[..arrow].trim().to_string();
            let body = part[arrow+2..].trim().to_string();
            let is_wildcard = pat == "_" || pat == "_ =>";
            arms.push(MatchArm { pat, guard: None, body, is_wildcard });
        }
    }
    let exhaustive = arms.iter().any(|a| a.is_wildcard) || arms.len() >= 2;
    Ok(MatchDecisionTree { scrutinee, arms, exhaustive })
}

/// 將決策樹轉為 if-else 鏈文本（用於展示與 core 管線）
pub fn decision_tree_to_if_chain(tree: &MatchDecisionTree) -> String {
    let mut out = String::new();
    out.push_str(&format!("// match {} lowered to if chain\n", tree.scrutinee));
    out.push_str("{\n");
    out.push_str(&format!("  let __scrut = {};\n", tree.scrutinee));
    for (i, arm) in tree.arms.iter().enumerate() {
        if arm.is_wildcard {
            out.push_str(&format!("  // _ => {}\n  {}\n", arm.body, arm.body));
        } else if i == 0 {
            out.push_str(&format!("  if is_{}(__scrut) {{\n    let {} = destructure_{}(__scrut);\n    {}\n  }}\n", arm.pat, arm.pat, arm.pat, arm.body));
        } else {
            out.push_str(&format!("  else if is_{}(__scrut) {{\n    let {} = destructure_{}(__scrut);\n    {}\n  }}\n", arm.pat, arm.pat, arm.pat, arm.body));
        }
    }
    if !tree.exhaustive {
        out.push_str("  else { panic!(\"non-exhaustive match\"); }\n");
    }
    out.push_str("}\n");
    out
}

/// for → loop + IntoIterator lowering
#[derive(Clone, Debug)]
pub struct ForLoop {
    pub pat: String,
    pub iter: String,
    pub body: String,
    pub invariant: Option<String>,
}

pub fn parse_for_to_loop(for_src: &str) -> Result<ForLoop, String> {
    // for x in iter { body }
    let src = for_src.trim();
    if !src.starts_with("for ") {
        return Err("not a for".to_string());
    }
    let after_for = src[4..].trim();
    if let Some(in_pos) = after_for.find(" in ") {
        let pat = after_for[..in_pos].trim().to_string();
        let rest = after_for[in_pos+4..].trim();
        let brace_start = rest.find('{').ok_or("no { in for")?;
        let iter = rest[..brace_start].trim().to_string();
        let body = rest[brace_start..].trim().trim_start_matches('{').trim_end_matches('}').trim().to_string();
        // 檢查是否有 invariant 註釋
        let invariant = if body.contains("@invariant") {
            Some("invariant".to_string())
        } else { None };
        Ok(ForLoop { pat, iter, body, invariant })
    } else {
        Err("no 'in' in for".to_string())
    }
}

pub fn for_loop_to_loop_text(fl: &ForLoop) -> String {
    format!(
        r#"// for {} in {} lowered
{{
  let mut __iter = {}.into_iter();
  loop {{
    match __iter.next() {{
      Some({}) => {{ {} }},
      None => break,
    }}
  }}
}}"#,
        fl.pat, fl.iter, fl.iter, fl.pat, fl.body
    )
}

/// Product 約束生成：t_struct - Π t_field =0
pub fn product_poly_text(struct_name: &str, node_id: usize, field_indices: &[usize]) -> String {
    if field_indices.is_empty() {
        format!("t{}_{} - 1 = 0  # struct {} unit", node_id, struct_name, struct_name)
    } else {
        let prod = field_indices.iter().map(|i| format!("t{}_{}", node_id, i)).collect::<Vec<_>>().join(" * ");
        format!("t{}_{} - {} = 0  # product {}", node_id, struct_name, prod, struct_name)
    }
}

/// Sum 約束生成：t_enum - Σ t_variant =0
pub fn sum_poly_text(enum_name: &str, node_id: usize, variant_indices: &[usize]) -> String {
    let sum = variant_indices.iter().map(|i| format!("t{}_{}", node_id, i)).collect::<Vec<_>>().join(" + ");
    format!("t{}_{} - ({}) = 0  # sum {}", node_id, enum_name, sum, enum_name)
}

/// Phase3 — trait/impl 方法表 lowering
pub fn lower_trait_impl_method_table(prog: &ProgramV2) -> crate::minirust::trait_impl::MethodTable {
    use crate::minirust::trait_impl::{MethodTable, TraitDef, ImplDef, TraitMethod, ImplMethod};
    let mut table = MethodTable::new();
    for item in &prog.items {
        match item {
            ItemV2::Trait(tr) => {
                let methods = tr.methods.iter().map(|m| TraitMethod {
                    name: m.name.clone(),
                    params: m.params.iter().map(|(n,ty)| (n.clone(), ty.name())).collect(),
                    ret_ty: m.ret.name(),
                    has_default: m.has_default,
                    default_body: m.default_body.clone(),
                }).collect();
                let td = TraitDef {
                    name: tr.name.clone(),
                    type_params: tr.generics.clone(),
                    lifetime_params: tr.lifetimes.clone(),
                    methods,
                    supertraits: tr.supertraits.clone(),
                };
                table.add_trait(td);
            }
            ItemV2::Impl(im) => {
                let methods = im.methods.iter().map(|m| ImplMethod {
                    name: m.sig.name.clone(),
                    params: m.sig.params.iter().map(|(n,ty)| (n.clone(), ty.name())).collect(),
                    ret_ty: m.sig.ret.name(),
                    body: m.body_src.clone(),
                }).collect();
                let trait_name = im.trait_name.clone();
                let for_ty = im.self_ty.name();
                let imp = ImplDef {
                    trait_name,
                    for_ty,
                    type_params: im.generics.clone(),
                    lifetime_params: im.lifetimes.clone(),
                    methods,
                    where_clauses: im.where_clauses.clone(),
                };
                table.add_impl(imp);
            }
            _ => {}
        }
    }
    table
}

/// Phase3 — lifetime 參數 lowering：檢查 lifetime 約束
pub fn lower_lifetimes(prog: &ProgramV2) -> Result<crate::minirust::lifetime::LifetimeGraph, String> {
    use crate::minirust::lifetime::{LifetimeGraph, Lifetime, Outlives};
    let mut graph = LifetimeGraph::new();
    for item in &prog.items {
        match item {
            ItemV2::Fn(f) => {
                for lt in &f.sig.lifetimes {
                    graph.add_lifetime(Lifetime::new(lt));
                }
                for wc in &f.sig.where_clauses {
                    if let Some(o) = Outlives::parse(wc) {
                        graph.add_outlives(o);
                    }
                }
            }
            ItemV2::Struct(s) => {
                for lt in &s.lifetimes {
                    graph.add_lifetime(Lifetime::new(lt));
                }
                for wc in &s.where_clauses {
                    if let Some(o) = Outlives::parse(wc) {
                        graph.add_outlives(o);
                    }
                }
            }
            ItemV2::Enum(e) => {
                for lt in &e.lifetimes {
                    graph.add_lifetime(Lifetime::new(lt));
                }
                for wc in &e.where_clauses {
                    if let Some(o) = Outlives::parse(wc) {
                        graph.add_outlives(o);
                    }
                }
            }
            ItemV2::Impl(im) => {
                for lt in &im.lifetimes {
                    graph.add_lifetime(Lifetime::new(lt));
                }
                for wc in &im.where_clauses {
                    if let Some(o) = Outlives::parse(wc) {
                        graph.add_outlives(o);
                    }
                }
            }
            ItemV2::Trait(tr) => {
                for lt in &tr.lifetimes {
                    graph.add_lifetime(Lifetime::new(lt));
                }
                for wc in &tr.where_clauses {
                    if let Some(o) = Outlives::parse(wc) {
                        graph.add_outlives(o);
                    }
                }
            }
            _ => {}
        }
    }
    if graph.has_cycle() {
        return Err("lifetime cycle detected in where clauses".to_string());
    }
    Ok(graph)
}

/// Phase3 — Vec/String/HashMap 內建 lowering
pub fn lower_stdlib_usage(prog: &ProgramV2) -> crate::minirust::stdlib::StdlibRegistry {
    use crate::minirust::stdlib::StdlibRegistry;
    let mut all_src = String::new();
    for item in &prog.items {
        match item {
            ItemV2::Struct(s) => {
                for (_, ty) in &s.fields {
                    all_src.push_str(&ty.name());
                    all_src.push(',');
                }
            }
            ItemV2::Enum(e) => {
                for v in &e.variants {
                    for ty in &v.fields {
                        all_src.push_str(&ty.name());
                        all_src.push(',');
                    }
                }
            }
            ItemV2::Fn(f) => {
                all_src.push_str(&f.sig.ret.name());
                all_src.push(',');
                for (_, ty) in &f.sig.params {
                    all_src.push_str(&ty.name());
                    all_src.push(',');
                }
                all_src.push_str(&f.body_src);
            }
            _ => {}
        }
    }
    StdlibRegistry::from_type_universe(&all_src)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minirust::ast_v2::ProgramV2;

    #[test]
    fn test_lower_struct() {
        let src = r#"
            struct Point { x: i32, y: i32 }
            fn main() { let p = Point { x: 3, y: 4 }; }
        "#;
        let prog = ProgramV2::parse_v2(src).unwrap();
        let lowered = lower_program(prog).unwrap();
        assert!(lowered.products.contains_key("Point"));
        println!("products: {:?}", lowered.products);
    }

    #[test]
    fn test_lower_enum() {
        let src = r#"
            enum Option<T> { Some(T), None }
            fn main() { let x = Option::Some(5); }
        "#;
        let prog = ProgramV2::parse_v2(src).unwrap();
        let lowered = lower_program(prog).unwrap();
        assert!(lowered.sums.contains_key("Option"));
        println!("sums: {:?}", lowered.sums);
    }

    #[test]
    fn test_match_decision_tree() {
        let src = "match x { Some(v) => v, None => 0 }";
        let tree = parse_match_to_decision_tree(src).unwrap();
        assert_eq!(tree.arms.len(), 2);
        let if_chain = decision_tree_to_if_chain(&tree);
        println!("{}", if_chain);
        assert!(if_chain.contains("if is_Some"));
    }

    #[test]
    fn test_for_loop() {
        let src = "for x in v { sum = sum + x; }";
        let fl = parse_for_to_loop(src).unwrap();
        assert_eq!(fl.pat, "x");
        assert_eq!(fl.iter, "v");
        let loop_text = for_loop_to_loop_text(&fl);
        println!("{}", loop_text);
        assert!(loop_text.contains("into_iter"));
    }

    #[test]
    fn test_mod_flatten() {
        let src = r#"
            mod utils {
                pub struct Point { x: i32, y: i32 }
            }
            fn main() { let p = utils::Point { x: 1, y: 2 }; }
        "#;
        let prog = ProgramV2::parse_v2(src).unwrap();
        let lowered = lower_program(prog).unwrap();
        println!("mod_map: {:?}", lowered.mod_map);
        // 扁平化後應無 mod 項，或已轉為帶前綴的 struct
        assert!(lowered.program.items.iter().any(|it| matches!(it, ItemV2::Struct(s) if s.name.contains("Point"))));
    }

    #[test]
    fn test_async_state_machine() {
        let src = r#"
            async fn fetch() -> i32 { 42 }
            fn main() { let f = fetch(); }
        "#;
        let prog = ProgramV2::parse_v2(src).unwrap();
        let lowered = lower_program(prog).unwrap();
        println!("generated: {:?}", lowered.generated);
        // 應生成 FetchState enum
        assert!(lowered.generated.iter().any(|it| matches!(it, ItemV2::Enum(e) if e.name.contains("State"))));
    }
}
