//! syn → ast_full 橋接：前端完整 Rust 語法解析
//! 依賴 syn = { features = ["full"] }，僅前端可用，core 保持零依賴

use polyrust_core::minirust::ast_full::*;
use syn::{File, Item, Type, Pat, Expr, Stmt, Visibility, Attribute, GenericParam as SynGenericParam, WherePredicate};

pub fn parse_with_syn(src: &str) -> Result<FullProgram, String> {
    let file: File = syn::parse_str(src).map_err(|e| format!("syn parse error: {}", e))?;
    Ok(file_to_full_program(file))
}

pub fn file_to_full_program(file: File) -> FullProgram {
    let mut prog = FullProgram::default();
    for attr in file.attrs {
        prog.attrs.push(syn_attr_to_attr(attr));
    }
    for item in file.items {
        if let Some(full_item) = syn_item_to_full_item(item) {
            // 檢測 main
            if let FullItem::Fn(f) = &full_item {
                if f.sig.name == "main" {
                    prog.main = Some(f.clone());
                    continue;
                }
            }
            prog.items.push(full_item);
        }
    }
    prog
}

fn syn_vis_to_vis(vis: Visibility) -> Vis {
    match vis {
        Visibility::Public(_) => Vis::Pub,
        Visibility::Restricted(r) => {
            let path_str = r.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
            match path_str.as_str() {
                "crate" => Vis::PubCrate,
                "super" => Vis::PubSuper,
                "self" => Vis::PubSelf,
                _ => Vis::PubIn(format!("{:?}", r.path)),
            }
        }
        Visibility::Inherited => Vis::Private,
    }
}

fn syn_attr_to_attr(attr: Attribute) -> Attr {
    let name = attr.path().get_ident().map(|i| i.to_string()).unwrap_or_else(|| format!("{:?}", attr.path()));
    let args = match &attr.meta {
        syn::Meta::List(list) => Some(format!("{:?}", list.tokens)),
        syn::Meta::NameValue(nv) => Some(format!("{:?}", nv.value)),
        _ => None,
    };
    Attr { name, args, is_inner: false }
}


fn member_to_string(m: syn::Member) -> String {
    match m {
        syn::Member::Named(ident) => ident.to_string(),
        syn::Member::Unnamed(idx) => idx.index.to_string(),
    }
}


fn syn_generics_to_generics(generics: syn::Generics) -> Generics {
    let mut params = vec![];
    for param in generics.params {
        match param {
            SynGenericParam::Type(t) => {
                let name = t.ident.to_string();
                let bounds = t.bounds.iter().map(|b| TypeBound::Trait(format!("{:?}", b))).collect();
                params.push(GenericParam::Type { name, bounds, default: t.default.map(|ty| syn_type_to_full_type(ty)) });
            }
            SynGenericParam::Lifetime(lt) => {
                params.push(GenericParam::Lifetime(Lifetime { name: format!("'{}", lt.lifetime.ident) }));
            }
            SynGenericParam::Const(c) => {
                params.push(GenericParam::Const { name: c.ident.to_string(), ty: syn_type_to_full_type(c.ty), default: c.default.map(|e| format!("{:?}", e)) });
            }
        }
    }
    let mut where_clauses = vec![];
    if let Some(where_clause) = generics.where_clause {
        for pred in where_clause.predicates {
            match pred {
                WherePredicate::Type(t) => {
                    let subject = format!("{:?}", t.bounded_ty);
                    let bounds = t.bounds.iter().map(|b| TypeBound::Trait(format!("{:?}", b))).collect();
                    where_clauses.push(WhereClause { subject, bounds });
                }
                WherePredicate::Lifetime(lt) => {
                    let subject = format!("'{}", lt.lifetime.ident);
                    let bounds = lt.bounds.iter().map(|b| TypeBound::Lifetime(format!("'{}", b.ident))).collect();
                    where_clauses.push(WhereClause { subject, bounds });
                }
                _ => {}
            }
        }
    }
    Generics { params, where_clauses }
}

fn syn_type_to_full_type(ty: Type) -> FullType {
    match ty {
        Type::Path(tp) => {
            let path_str = tp.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
            let mut args = vec![];
            if let Some(last) = tp.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(ab) = &last.arguments {
                    for arg in &ab.args {
                        if let syn::GenericArgument::Type(t) = arg {
                            args.push(syn_type_to_full_type(t.clone()));
                        }
                    }
                }
            }
            // 特殊處理
            match path_str.as_str() {
                "String" => FullType::V2(polyrust_core::minirust::universe::TypeV2::Ext(polyrust_core::minirust::universe::ExtType::String)),
                _ => FullType::Path { path: path_str, args },
            }
        }
        Type::Tuple(tup) => {
            FullType::Tuple(tup.elems.into_iter().map(syn_type_to_full_type).collect())
        }
        Type::Array(arr) => {
            let elem = Box::new(syn_type_to_full_type(*arr.elem));
            let len = Some(format!("{:?}", arr.len));
            FullType::Array { elem, len }
        }
        Type::Slice(slice) => {
            FullType::Slice(Box::new(syn_type_to_full_type(*slice.elem)))
        }
        Type::Ptr(ptr) => {
            FullType::Ptr { mutbl: ptr.mutability.is_some(), inner: Box::new(syn_type_to_full_type(*ptr.elem)) }
        }
        Type::Reference(r) => {
            FullType::Ref {
                mutbl: r.mutability.is_some(),
                lifetime: r.lifetime.map(|lt| format!("'{}", lt.ident)),
                inner: Box::new(syn_type_to_full_type(*r.elem)),
            }
        }
        Type::BareFn(bf) => {
            let params = bf.inputs.into_iter().map(|i| syn_type_to_full_type(i.ty)).collect();
            let ret = Box::new(match bf.output {
                syn::ReturnType::Default => FullType::Tuple(vec![]),
                syn::ReturnType::Type(_, ty) => syn_type_to_full_type(*ty),
            });
            FullType::BareFn { params, ret, is_unsafe: bf.unsafety.is_some(), is_async: false }
        }
        Type::Never(_) => FullType::Never,
        Type::Infer(_) => FullType::Inferred,
        Type::TraitObject(to) => {
            let bounds = to.bounds.iter().map(|b| TypeBound::Trait(format!("{:?}", b))).collect();
            FullType::TraitObject { bounds, dyn_token: to.dyn_token.is_some() }
        }
        Type::ImplTrait(it) => {
            let bounds = it.bounds.iter().map(|b| TypeBound::Trait(format!("{:?}", b))).collect();
            FullType::ImplTrait { bounds }
        }
        Type::Paren(p) => FullType::Paren(Box::new(syn_type_to_full_type(*p.elem))),
        Type::Group(g) => FullType::Group(Box::new(syn_type_to_full_type(*g.elem))),
        _ => FullType::Macro(format!("{:?}", ty)),
    }
}

fn syn_pat_to_full_pat(pat: Pat) -> FullPat {
    match pat {
        Pat::Wild(_) => FullPat::Wild,
        Pat::Ident(pi) => {
            FullPat::Ident {
                name: pi.ident.to_string(),
                mutbl: pi.mutability.is_some(),
                by_ref: pi.by_ref.is_some(),
                subpat: pi.subpat.map(|(_, p)| Box::new(syn_pat_to_full_pat(*p))),
            }
        }
        Pat::Lit(pl) => FullPat::Lit(format!("{:?}", pl.lit)),
        Pat::Path(pp) => FullPat::Path(pp.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")),
        Pat::TupleStruct(pts) => {
            FullPat::TupleStruct {
                path: pts.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::"),
                elems: pts.elems.into_iter().map(syn_pat_to_full_pat).collect(),
            }
        }
        Pat::Struct(ps) => {
            FullPat::Struct {
                path: ps.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::"),
                fields: ps.fields.into_iter().map(|f| (member_to_string(f.member), syn_pat_to_full_pat(*f.pat))).collect(),
                rest: ps.rest.is_some(),
            }
        }
        Pat::Tuple(pt) => FullPat::Tuple(pt.elems.into_iter().map(syn_pat_to_full_pat).collect()),
        Pat::Slice(ps) => FullPat::Slice(ps.elems.into_iter().map(syn_pat_to_full_pat).collect()),
        Pat::Or(po) => FullPat::Or(po.cases.into_iter().map(syn_pat_to_full_pat).collect()),
        Pat::Reference(pr) => FullPat::Ref { mutbl: pr.mutability.is_some(), inner: Box::new(syn_pat_to_full_pat(*pr.pat)) },
        Pat::Type(pt) => FullPat::Type { pat: Box::new(syn_pat_to_full_pat(*pt.pat)), ty: syn_type_to_full_type(*pt.ty) },
        Pat::Range(pr) => FullPat::Range { start: pr.start.map(|e| format!("{:?}", e)), end: pr.end.map(|e| format!("{:?}", e)), inclusive: matches!(pr.limits, syn::RangeLimits::Closed(_)) },
        Pat::Macro(pm) => FullPat::Macro(format!("{:?}", pm.mac.tokens)),
        _ => FullPat::Macro(format!("{:?}", pat)),
    }
}

fn syn_expr_to_full_expr(expr: Expr) -> FullExpr {
    match expr {
        Expr::Lit(el) => FullExpr::Lit(format!("{:?}", el.lit)),
        Expr::Path(ep) => FullExpr::Path(ep.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")),
        Expr::Field(ef) => FullExpr::Field { base: Box::new(syn_expr_to_full_expr(*ef.base)), field: member_to_string(ef.member) },
        Expr::Index(ei) => FullExpr::Index { base: Box::new(syn_expr_to_full_expr(*ei.expr)), index: Box::new(syn_expr_to_full_expr(*ei.index)) },
        Expr::Call(ec) => FullExpr::Call { func: Box::new(syn_expr_to_full_expr(*ec.func)), args: ec.args.into_iter().map(syn_expr_to_full_expr).collect() },
        Expr::MethodCall(emc) => FullExpr::MethodCall {
            receiver: Box::new(syn_expr_to_full_expr(*emc.receiver)),
            method: emc.method.to_string(),
            turbofish: emc.turbofish.as_ref().map(|tf| tf.args.iter().filter_map(|a| if let syn::GenericArgument::Type(t) = a { Some(syn_type_to_full_type(t.clone())) } else { None }).collect()).unwrap_or_default(),
            args: emc.args.into_iter().map(syn_expr_to_full_expr).collect(),
        },
        Expr::Unary(eu) => FullExpr::Unary { op: format!("{:?}", eu.op), expr: Box::new(syn_expr_to_full_expr(*eu.expr)) },
        Expr::Binary(eb) => FullExpr::Binary { op: format!("{:?}", eb.op), left: Box::new(syn_expr_to_full_expr(*eb.left)), right: Box::new(syn_expr_to_full_expr(*eb.right)) },
        Expr::Assign(ea) => FullExpr::Assign { left: Box::new(syn_expr_to_full_expr(*ea.left)), right: Box::new(syn_expr_to_full_expr(*ea.right)) },
        Expr::If(ei) => FullExpr::If {
            cond: Box::new(syn_expr_to_full_expr(*ei.cond)),
            then_branch: Box::new(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: ei.then_branch }))),
            else_branch: ei.else_branch.map(|(_, e)| Box::new(syn_expr_to_full_expr(*e))),
        },
        Expr::Match(em) => FullExpr::Match {
            scrutinee: Box::new(syn_expr_to_full_expr(*em.expr)),
            arms: em.arms.into_iter().map(|arm| MatchArm {
                pat: syn_pat_to_full_pat(arm.pat),
                guard: arm.guard.map(|(_, e)| syn_expr_to_full_expr(*e)),
                body: syn_expr_to_full_expr(*arm.body),
                comma: arm.comma.is_some(),
            }).collect(),
        },
        Expr::Loop(el) => FullExpr::Loop { body: Box::new(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: el.body }))), label: el.label.map(|l| l.name.ident.to_string()) },
        Expr::While(ew) => FullExpr::While { cond: Box::new(syn_expr_to_full_expr(*ew.cond)), body: Box::new(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: ew.body }))), label: ew.label.map(|l| l.name.ident.to_string()) },
        Expr::ForLoop(ef) => FullExpr::For { pat: syn_pat_to_full_pat(*ef.pat), iter: Box::new(syn_expr_to_full_expr(*ef.expr)), body: Box::new(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: ef.body }))), label: ef.label.map(|l| l.name.ident.to_string()) },
        Expr::Block(eb) => FullExpr::Block { stmts: eb.block.stmts.into_iter().filter_map(syn_stmt_to_full_stmt).collect(), label: eb.label.map(|l| l.name.ident.to_string()) },
        Expr::Unsafe(eu) => FullExpr::Unsafe(Box::new(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: eu.block })))),
        Expr::Async(ea) => FullExpr::Async { capture: ea.capture.map(|c| format!("{:?}", c)), block: Box::new(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: ea.block }))) },
        Expr::Await(ea) => FullExpr::Await { base: Box::new(syn_expr_to_full_expr(*ea.base)) },
        Expr::Closure(ec) => FullExpr::Closure {
            inputs: ec.inputs.into_iter().map(|i| match i {
                Pat::Ident(pi) => (pi.ident.to_string(), None),
                _ => (format!("{:?}", i), None),
            }).collect(),
            body: Box::new(syn_expr_to_full_expr(*ec.body)),
            is_async: ec.asyncness.is_some(),
            is_move: ec.movability.is_some(),
            is_mut: false,
        },
        Expr::Return(er) => FullExpr::Return(er.expr.map(|e| Box::new(syn_expr_to_full_expr(*e)))),
        Expr::Break(eb) => FullExpr::Break { label: eb.label.map(|l| l.ident.to_string()), expr: eb.expr.map(|e| Box::new(syn_expr_to_full_expr(*e))) },
        Expr::Continue(ec) => FullExpr::Continue(ec.label.map(|l| l.ident.to_string())),
        Expr::Let(el) => FullExpr::Let { pat: syn_pat_to_full_pat(*el.pat), expr: Box::new(syn_expr_to_full_expr(*el.expr)) },
        Expr::Struct(es) => FullExpr::StructLit {
            path: es.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::"),
            fields: es.fields.into_iter().map(|f| (member_to_string(f.member), syn_expr_to_full_expr(f.expr))).collect(),
            rest: es.rest.map(|r| Box::new(syn_expr_to_full_expr(*r))),
        },
        Expr::Array(ea) => FullExpr::Array(ea.elems.into_iter().map(syn_expr_to_full_expr).collect()),
        Expr::Repeat(er) => FullExpr::ArrayRepeat { elem: Box::new(syn_expr_to_full_expr(*er.expr)), len: Box::new(syn_expr_to_full_expr(*er.len)) },
        Expr::Tuple(et) => FullExpr::Tuple(et.elems.into_iter().map(syn_expr_to_full_expr).collect()),
        Expr::Cast(ec) => FullExpr::Cast { expr: Box::new(syn_expr_to_full_expr(*ec.expr)), ty: syn_type_to_full_type(*ec.ty) },
        Expr::Try(et) => FullExpr::Try(Box::new(syn_expr_to_full_expr(*et.expr))),
        Expr::Range(er) => FullExpr::Range { start: er.start.map(|e| Box::new(syn_expr_to_full_expr(*e))), end: er.end.map(|e| Box::new(syn_expr_to_full_expr(*e))), inclusive: matches!(er.limits, syn::RangeLimits::Closed(_)) },
        Expr::Macro(em) => FullExpr::Macro(em.mac.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::"), format!("{:?}", em.mac.tokens)),
        _ => FullExpr::Verbatim(format!("{:?}", expr)),
    }
}

fn syn_stmt_to_full_stmt(stmt: Stmt) -> Option<FullStmt> {
    match stmt {
        Stmt::Local(local) => {
            Some(FullStmt::Local {
                pat: syn_pat_to_full_pat(local.pat),
                ty: None,
                init: local.init.map(|i| syn_expr_to_full_expr(*i.expr)),
                attrs: local.attrs.into_iter().map(syn_attr_to_attr).collect(),
            })
        }
        Stmt::Item(item) => syn_item_to_full_item(item).map(FullStmt::Item),
        Stmt::Expr(expr, _) => Some(FullStmt::Expr(syn_expr_to_full_expr(expr))),
        Stmt::Macro(m) => Some(FullStmt::Macro(format!("{:?}", m.mac.tokens))),
    }
}

fn syn_item_to_full_item(item: Item) -> Option<FullItem> {
    match item {
        Item::Fn(f) => {
            let vis = syn_vis_to_vis(f.vis);
            let attrs = f.attrs.into_iter().map(syn_attr_to_attr).collect();
            let sig = FnSig {
                name: f.sig.ident.to_string(),
                generics: syn_generics_to_generics(f.sig.generics),
                inputs: f.sig.inputs.into_iter().filter_map(|input| match input {
                    syn::FnArg::Receiver(r) => Some(FnInput::Receiver { mutbl: r.mutability.is_some(), reference: r.reference.is_some(), lifetime: r.reference.as_ref().and_then(|(_, lt)| lt.as_ref()).map(|lt| format!("'{}", lt.ident)) }),
                    syn::FnArg::Typed(t) => Some(FnInput::Typed { pat: syn_pat_to_full_pat(*t.pat), ty: syn_type_to_full_type(*t.ty) }),
                }).collect(),
                output: match f.sig.output {
                    syn::ReturnType::Default => FullType::Tuple(vec![]),
                    syn::ReturnType::Type(_, ty) => syn_type_to_full_type(*ty),
                },
                is_async: f.sig.asyncness.is_some(),
                is_unsafe: f.sig.unsafety.is_some(),
                is_const: f.sig.constness.is_some(),
                abi: f.sig.abi.map(|abi| format!("{:?}", abi.name)),
            };
            let block = Some(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: *f.block })));
            Some(FullItem::Fn(FnItem { vis, sig, block, attrs }))
        }
        Item::Struct(s) => {
            let vis = syn_vis_to_vis(s.vis);
            let attrs = s.attrs.into_iter().map(syn_attr_to_attr).collect();
            let generics = syn_generics_to_generics(s.generics);
            let fields = match s.fields {
                syn::Fields::Named(named) => {
                    StructFields::Named(named.named.into_iter().map(|f| NamedField {
                        vis: syn_vis_to_vis(f.vis),
                        name: f.ident.map(|i| i.to_string()).unwrap_or_default(),
                        ty: syn_type_to_full_type(f.ty),
                        attrs: f.attrs.into_iter().map(syn_attr_to_attr).collect(),
                    }).collect())
                }
                syn::Fields::Unnamed(unnamed) => {
                    StructFields::Tuple(unnamed.unnamed.into_iter().map(|f| (syn_vis_to_vis(f.vis), syn_type_to_full_type(f.ty))).collect())
                }
                syn::Fields::Unit => StructFields::Unit,
            };
            Some(FullItem::Struct(StructItem { vis, name: s.ident.to_string(), generics, fields, attrs }))
        }
        Item::Enum(e) => {
            let vis = syn_vis_to_vis(e.vis);
            let attrs = e.attrs.into_iter().map(syn_attr_to_attr).collect();
            let generics = syn_generics_to_generics(e.generics);
            let variants = e.variants.into_iter().map(|v| EnumVariant {
                name: v.ident.to_string(),
                fields: match v.fields {
                    syn::Fields::Named(named) => StructFields::Named(named.named.into_iter().map(|f| NamedField {
                        vis: syn_vis_to_vis(f.vis),
                        name: f.ident.map(|i| i.to_string()).unwrap_or_default(),
                        ty: syn_type_to_full_type(f.ty),
                        attrs: f.attrs.into_iter().map(syn_attr_to_attr).collect(),
                    }).collect()),
                    syn::Fields::Unnamed(unnamed) => StructFields::Tuple(unnamed.unnamed.into_iter().map(|f| (syn_vis_to_vis(f.vis), syn_type_to_full_type(f.ty))).collect()),
                    syn::Fields::Unit => StructFields::Unit,
                },
                discriminant: v.discriminant.map(|(_, e)| syn_expr_to_full_expr(e)),
                attrs: v.attrs.into_iter().map(syn_attr_to_attr).collect(),
            }).collect();
            Some(FullItem::Enum(EnumItem { vis, name: e.ident.to_string(), generics, variants, attrs }))
        }
        Item::Impl(i) => {
            let generics = syn_generics_to_generics(i.generics);
            let trait_ref = i.trait_.map(|(bang, path, _)| (bang.is_some(), path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::"), vec![]));
            let self_ty = syn_type_to_full_type(*i.self_ty);
            let items = i.items.into_iter().filter_map(|ii| match ii {
                syn::ImplItem::Fn(f) => {
                    let vis = syn_vis_to_vis(f.vis);
                    let attrs = f.attrs.into_iter().map(syn_attr_to_attr).collect();
                    let sig = FnSig {
                        name: f.sig.ident.to_string(),
                        generics: syn_generics_to_generics(f.sig.generics),
                        inputs: f.sig.inputs.into_iter().filter_map(|input| match input {
                            syn::FnArg::Receiver(r) => Some(FnInput::Receiver { mutbl: r.mutability.is_some(), reference: r.reference.is_some(), lifetime: None }),
                            syn::FnArg::Typed(t) => Some(FnInput::Typed { pat: syn_pat_to_full_pat(*t.pat), ty: syn_type_to_full_type(*t.ty) }),
                        }).collect(),
                        output: match f.sig.output {
                            syn::ReturnType::Default => FullType::Tuple(vec![]),
                            syn::ReturnType::Type(_, ty) => syn_type_to_full_type(*ty),
                        },
                        is_async: f.sig.asyncness.is_some(),
                        is_unsafe: f.sig.unsafety.is_some(),
                        is_const: f.sig.constness.is_some(),
                        abi: None,
                    };
                    let block = Some(syn_expr_to_full_expr(Expr::Block(syn::ExprBlock { attrs: vec![], label: None, block: f.block })));
                    Some(FullItem::Fn(FnItem { vis, sig, block, attrs }))
                }
                _ => None,
            }).collect();
            Some(FullItem::Impl(ImplItem { generics, trait_ref, self_ty, items, is_unsafe: i.unsafety.is_some(), attrs: i.attrs.into_iter().map(syn_attr_to_attr).collect() }))
        }
        Item::Mod(m) => {
            let vis = syn_vis_to_vis(m.vis);
            let attrs = m.attrs.into_iter().map(syn_attr_to_attr).collect();
            let items = m.content.map(|(_, items)| items.into_iter().filter_map(syn_item_to_full_item).collect());
            Some(FullItem::Mod(ModItem { vis, name: m.ident.to_string(), items, attrs }))
        }
        Item::Use(u) => {
            let vis = syn_vis_to_vis(u.vis);
            let attrs = u.attrs.into_iter().map(syn_attr_to_attr).collect();
            // 簡化：tree 存文本
            let tree = UseTree::Name(format!("{:?}", u.tree));
            Some(FullItem::Use(UseItem { vis, tree, attrs }))
        }
        Item::Const(c) => {
            let vis = syn_vis_to_vis(c.vis);
            let attrs = c.attrs.into_iter().map(syn_attr_to_attr).collect();
            Some(FullItem::Const(ConstItem { vis, name: c.ident.to_string(), ty: syn_type_to_full_type(*c.ty), expr: Some(syn_expr_to_full_expr(*c.expr)), attrs }))
        }
        Item::Static(s) => {
            let vis = syn_vis_to_vis(s.vis);
            let attrs = s.attrs.into_iter().map(syn_attr_to_attr).collect();
            Some(FullItem::Static(StaticItem { vis, name: s.ident.to_string(), ty: syn_type_to_full_type(*s.ty), mutbl: matches!(s.mutability, syn::StaticMutability::Mut(_)), expr: Some(syn_expr_to_full_expr(*s.expr)), attrs }))
        }
        Item::Type(t) => {
            let vis = syn_vis_to_vis(t.vis);
            let attrs = t.attrs.into_iter().map(syn_attr_to_attr).collect();
            Some(FullItem::TypeAlias(TypeAliasItem { vis, name: t.ident.to_string(), generics: syn_generics_to_generics(t.generics), bounds: vec![], ty: Some(syn_type_to_full_type(*t.ty)), attrs }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_bridge_struct() {
        let src = r#"
            pub struct Point { pub x: i32, pub y: i32 }
            fn main() {}
        "#;
        let prog = parse_with_syn(src).unwrap();
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            FullItem::Struct(s) => assert_eq!(s.name, "Point"),
            _ => panic!("not struct"),
        }
    }

    #[test]
    fn test_syn_bridge_full() {
        let src = r#"
            use std::collections::HashMap;
            pub mod geometry {
                pub struct Point { pub x: i32, pub y: i32 }
                pub enum Option<T> { Some(T), None }
                pub fn new(x: i32) -> Point { Point { x, y: 0 } }
            }
            async fn fetch() -> i32 { 42 }
            fn main() {
                let v: Vec<i32> = Vec::new();
                let x = fetch().await;
                let y = match Some(5) { Some(v) => v, None => 0 };
            }
        "#;
        let prog = parse_with_syn(src).unwrap();
        println!("items: {}", prog.items.len());
        assert!(prog.items.len() >= 3);
    }
}
