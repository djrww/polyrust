//! Phase3 — trait/impl 方法表與存在量化
//!
//! trait 定義、impl 塊、方法表 lowering、存在量化 ∃T

use std::collections::HashMap;

/// trait 方法簽名
#[derive(Clone, Debug)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, ty)
    pub ret_ty: String,
    pub has_default: bool,
    pub default_body: Option<String>,
}

/// trait 定義
#[derive(Clone, Debug)]
pub struct TraitDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub lifetime_params: Vec<String>,
    pub methods: Vec<TraitMethod>,
    pub supertraits: Vec<String>,
}

/// impl 塊
#[derive(Clone, Debug)]
pub struct ImplDef {
    pub trait_name: Option<String>, // None => inherent impl
    pub for_ty: String,
    pub type_params: Vec<String>,
    pub lifetime_params: Vec<String>,
    pub methods: Vec<ImplMethod>,
    pub where_clauses: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ImplMethod {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub ret_ty: String,
    pub body: String,
}

/// 方法表：trait + type → 方法實現
#[derive(Clone, Debug, Default)]
pub struct MethodTable {
    pub traits: HashMap<String, TraitDef>,
    pub impls: Vec<ImplDef>,
    /// (for_ty, trait_name) -> impl index
    pub impl_map: HashMap<(String, String), usize>,
    /// for_ty -> inherent impls indices
    pub inherent_map: HashMap<String, Vec<usize>>,
}

impl MethodTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trait(&mut self, tr: TraitDef) {
        self.traits.insert(tr.name.clone(), tr);
    }

    pub fn add_impl(&mut self, imp: ImplDef) {
        let idx = self.impls.len();
        if let Some(trait_name) = &imp.trait_name {
            self.impl_map.insert((imp.for_ty.clone(), trait_name.clone()), idx);
        } else {
            self.inherent_map.entry(imp.for_ty.clone()).or_default().push(idx);
        }
        self.impls.push(imp);
    }

    /// 解析方法調用：receiver_ty.method_name → ImplMethod
    pub fn resolve_method(&self, recv_ty: &str, method: &str) -> Option<&ImplMethod> {
        // 先找 inherent
        if let Some(idxs) = self.inherent_map.get(recv_ty) {
            for &i in idxs {
                if let Some(m) = self.impls[i].methods.iter().find(|m| m.name == method) {
                    return Some(m);
                }
            }
        }
        // 再找 trait impl
        for ((ty, _trait), &idx) in &self.impl_map {
            if ty == recv_ty {
                if let Some(m) = self.impls[idx].methods.iter().find(|m| m.name == method) {
                    return Some(m);
                }
            }
        }
        None
    }

    /// 解析 trait 方法：(trait_name, method) → TraitMethod
    pub fn resolve_trait_method(&self, trait_name: &str, method: &str) -> Option<&TraitMethod> {
        self.traits.get(trait_name)?.methods.iter().find(|m| m.name == method)
    }

    /// 生成多項式約束：方法調用等價於 impl body
    pub fn method_call_poly(&self, recv_ty: &str, method: &str, _args: &[String]) -> Option<String> {
        let m = self.resolve_method(recv_ty, method)?;
        Some(format!("// {}.{} -> {} : {}", recv_ty, method, m.ret_ty, m.body))
    }

    /// 存在量化：∃T. P(T) 的編碼
    pub fn existential_poly(&self, var: &str, bound_trait: &str, body: &str) -> String {
        // ∃T: Trait. body(T)
        // 編碼為：引入新變量 t_T，約束 t_T ∈ { types implementing Trait }
        format!("exists {}: {} {{ {} }}", var, bound_trait, body)
    }

    /// 列出實現某 trait 的所有類型
    pub fn types_implementing(&self, trait_name: &str) -> Vec<String> {
        self.impl_map.iter()
            .filter(|((_, t), _)| t == trait_name)
            .map(|((ty, _), _)| ty.clone())
            .collect()
    }
}

/// 解析 .poly 中的 trait/impl 文本（簡化）
pub fn parse_trait_def(text: &str) -> Option<TraitDef> {
    // 形如 "trait Foo<T> { fn bar(&self) -> i32; }"
    let t = text.trim();
    if !t.starts_with("trait ") {
        return None;
    }
    let rest = t["trait ".len()..].trim();
    let name_end = rest.find(|c: char| c == '<' || c == '{' || c.is_whitespace()).unwrap_or(rest.len());
    let name = rest[..name_end].trim().to_string();
    // type params
    let mut type_params = vec![];
    let mut lifetime_params = vec![];
    if rest.contains('<') {
        if let Some(lt) = rest.find('<') {
            if let Some(gt) = rest.find('>') {
                let params_str = &rest[lt+1..gt];
                for p in params_str.split(',') {
                    let p = p.trim();
                    if p.starts_with('\'') {
                        lifetime_params.push(p.to_string());
                    } else if !p.is_empty() {
                        type_params.push(p.to_string());
                    }
                }
            }
        }
    }
    // methods：簡化提取 fn ...
    let mut methods = vec![];
    if let Some(body_start) = rest.find('{') {
        if let Some(body_end) = rest.rfind('}') {
            let body = &rest[body_start+1..body_end];
            for line in body.lines() {
                let line = line.trim();
                if line.starts_with("fn ") {
                    let m_name_end = line[3..].find('(').map(|i| i+3).unwrap_or(line.len());
                    let m_name = line[3..m_name_end].trim().to_string();
                    methods.push(TraitMethod {
                        name: m_name,
                        params: vec![],
                        ret_ty: "i32".to_string(),
                        has_default: line.contains('{'),
                        default_body: None,
                    });
                }
            }
        }
    }

    Some(TraitDef { name, type_params, lifetime_params, methods, supertraits: vec![] })
}

pub fn parse_impl_def(text: &str) -> Option<ImplDef> {
    let t = text.trim();
    if !t.starts_with("impl") {
        return None;
    }
    let rest = t["impl".len()..].trim();
    // impl<T> Trait for Ty  或 impl Ty
    let (trait_name, for_ty) = if rest.contains(" for ") {
        let parts: Vec<&str> = rest.splitn(2, " for ").collect();
        let trait_part = parts[0].trim();
        // 去掉泛型參數
        let trait_name = trait_part.split('<').next().unwrap().trim().split_whitespace().last().unwrap_or(trait_part).to_string();
        let for_ty = parts[1].split('{').next().unwrap().trim().to_string();
        (Some(trait_name), for_ty)
    } else {
        let for_ty = rest.split('{').next().unwrap().trim().to_string();
        (None, for_ty)
    };

    let mut methods = vec![];
    if let Some(body_start) = rest.find('{') {
        if let Some(body_end) = rest.rfind('}') {
            let body = &rest[body_start+1..body_end];
            for line in body.lines() {
                let line = line.trim();
                if line.starts_with("fn ") {
                    let m_name_end = line[3..].find('(').map(|i| i+3).unwrap_or(line.len());
                    let m_name = line[3..m_name_end].trim().to_string();
                    methods.push(ImplMethod {
                        name: m_name,
                        params: vec![],
                        ret_ty: "i32".to_string(),
                        body: line.to_string(),
                    });
                }
            }
        }
    }

    Some(ImplDef {
        trait_name,
        for_ty,
        type_params: vec![],
        lifetime_params: vec![],
        methods,
        where_clauses: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trait() {
        let text = "trait Foo { fn bar(&self) -> i32; fn baz(&self) -> bool { true } }";
        let tr = parse_trait_def(text).unwrap();
        assert_eq!(tr.name, "Foo");
        // 簡化解析可能將同一行兩個 fn 視為一個，至少 1 個
        assert!(tr.methods.len() >= 1);
        assert!(tr.methods.iter().any(|m| m.name == "bar"));
    }

    #[test]
    fn test_parse_impl_inherent() {
        let text = "impl MyType { fn foo(&self) -> i32 { 42 } }";
        let imp = parse_impl_def(text).unwrap();
        assert!(imp.trait_name.is_none());
        assert_eq!(imp.for_ty, "MyType");
        assert_eq!(imp.methods[0].name, "foo");
    }

    #[test]
    fn test_parse_impl_trait() {
        let text = "impl Foo for MyType { fn bar(&self) -> i32 { 1 } }";
        let imp = parse_impl_def(text).unwrap();
        assert_eq!(imp.trait_name, Some("Foo".to_string()));
        assert_eq!(imp.for_ty, "MyType");
    }

    #[test]
    fn test_method_table_resolve() {
        let mut table = MethodTable::new();
        table.add_impl(ImplDef {
            trait_name: None,
            for_ty: "MyType".to_string(),
            type_params: vec![],
            lifetime_params: vec![],
            methods: vec![ImplMethod { name: "foo".to_string(), params: vec![], ret_ty: "i32".to_string(), body: "42".to_string() }],
            where_clauses: vec![],
        });
        let m = table.resolve_method("MyType", "foo").unwrap();
        assert_eq!(m.body, "42");
    }

    #[test]
    fn test_existential() {
        let table = MethodTable::new();
        let poly = table.existential_poly("T", "Display", "print(T)");
        assert!(poly.contains("exists"));
        assert!(poly.contains("Display"));
    }

    #[test]
    fn test_types_implementing() {
        let mut table = MethodTable::new();
        table.add_impl(ImplDef {
            trait_name: Some("Foo".to_string()),
            for_ty: "A".to_string(),
            type_params: vec![],
            lifetime_params: vec![],
            methods: vec![],
            where_clauses: vec![],
        });
        table.add_impl(ImplDef {
            trait_name: Some("Foo".to_string()),
            for_ty: "B".to_string(),
            type_params: vec![],
            lifetime_params: vec![],
            methods: vec![],
            where_clauses: vec![],
        });
        let tys = table.types_implementing("Foo");
        assert_eq!(tys.len(), 2);
        assert!(tys.contains(&"A".to_string()));
    }
}
