// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Phase1 — 型別宇宙 7+i：從固定 7 種到可擴展 N = 7 + i
//! v0.1.5 的 `Type` 是平坦枚舉 7 種（i32, bool, (), &i32, &mut i32, &bool, &mut bool）。
//! v0.2 Phase1 將其擴展為 `TypeV2`，保留 7 基底 + i 擴展（Vec, String, HashMap, Struct, Enum, RawPtr, Future, Option, Result 等）。
//! 本模組零第三方依賴，只用 std，符合 core 承諾。
//! 優化版：with_capacity、減少 clone、實際使用 API 供 pipeline_v2 消費

use std::collections::HashMap;

/// 基底 7 種（與原 Type 一一對應）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BaseType {
    I32,
    Bool,
    Unit,
    RefI32,
    RefMutI32,
    RefBool,
    RefMutBool,
}

pub const BASE_TYPES: [BaseType; 7] = [
    BaseType::I32,
    BaseType::Bool,
    BaseType::Unit,
    BaseType::RefI32,
    BaseType::RefMutI32,
    BaseType::RefBool,
    BaseType::RefMutBool,
];

impl BaseType {
    pub fn name(&self) -> &'static str {
        match self {
            BaseType::I32 => "i32",
            BaseType::Bool => "bool",
            BaseType::Unit => "()",
            BaseType::RefI32 => "&i32",
            BaseType::RefMutI32 => "&mut i32",
            BaseType::RefBool => "&bool",
            BaseType::RefMutBool => "&mut bool",
        }
    }
    pub fn index(&self) -> usize {
        BASE_TYPES.iter().position(|t| t == self).unwrap()
    }
}

/// 擴展型別（i 部分）— 每個變體帶結構信息，供統一與編碼 — Path C 擴展 Tuple/Array/Slice/BareFn/TraitObject/ImplTrait
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ExtType {
    Vec(Box<TypeV2>),
    String,
    HashMap(Box<TypeV2>, Box<TypeV2>),
    Struct { name: String, args: Vec<TypeV2> },
    Enum { name: String, args: Vec<TypeV2> },
    RawPtr { mutbl: bool, inner: Box<TypeV2> },
    Future(Box<TypeV2>),
    Option(Box<TypeV2>),
    Result(Box<TypeV2>, Box<TypeV2>),
    RefExt { mutbl: bool, inner: Box<TypeV2>, lifetime: Option<String> },
    GenericParam(String),
    Assoc { self_ty: Box<TypeV2>, trait_name: String, assoc: String },
    Never,
    Inferred,
    Tuple(Vec<TypeV2>),
    Array { elem: Box<TypeV2>, len: Option<String> },
    Slice(Box<TypeV2>),
    BareFn { params: Vec<TypeV2>, ret: Box<TypeV2> },
    TraitObject { bounds: Vec<String> },
    ImplTrait { bounds: Vec<String> },
}

impl ExtType {
    pub fn name(&self) -> String {
        match self {
            ExtType::Vec(t) => {
                let inner = t.name();
                let mut s = String::with_capacity(5 + inner.len());
                s.push_str("Vec<");
                s.push_str(&inner);
                s.push('>');
                s
            }
            ExtType::String => "String".to_string(),
            ExtType::HashMap(k, v) => {
                let kn = k.name();
                let vn = v.name();
                let mut s = String::with_capacity(9 + kn.len() + vn.len() + 1);
                s.push_str("HashMap<");
                s.push_str(&kn);
                s.push(',');
                s.push_str(&vn);
                s.push('>');
                s
            }
            ExtType::Struct { name, args } => {
                if args.is_empty() { name.clone() } else {
                    let args_names: Vec<String> = args.iter().map(|a| a.name()).collect();
                    let total: usize = args_names.iter().map(|a| a.len()).sum::<usize>() + args_names.len();
                    let mut s = String::with_capacity(name.len() + 2 + total);
                    s.push_str(name);
                    s.push('<');
                    s.push_str(&args_names.join(","));
                    s.push('>');
                    s
                }
            }
            ExtType::Enum { name, args } => {
                if args.is_empty() { name.clone() } else {
                    let args_names: Vec<String> = args.iter().map(|a| a.name()).collect();
                    let total: usize = args_names.iter().map(|a| a.len()).sum::<usize>() + args_names.len();
                    let mut s = String::with_capacity(name.len() + 2 + total);
                    s.push_str(name);
                    s.push('<');
                    s.push_str(&args_names.join(","));
                    s.push('>');
                    s
                }
            }
            ExtType::RawPtr { mutbl, inner } => {
                let inner_n = inner.name();
                if *mutbl {
                    let mut s = String::with_capacity(5 + inner_n.len());
                    s.push_str("*mut ");
                    s.push_str(&inner_n);
                    s
                } else {
                    let mut s = String::with_capacity(7 + inner_n.len());
                    s.push_str("*const ");
                    s.push_str(&inner_n);
                    s
                }
            }
            ExtType::Future(t) => {
                let inner = t.name();
                let mut s = String::with_capacity(8 + inner.len());
                s.push_str("Future<");
                s.push_str(&inner);
                s.push('>');
                s
            }
            ExtType::Option(t) => {
                let inner = t.name();
                let mut s = String::with_capacity(8 + inner.len());
                s.push_str("Option<");
                s.push_str(&inner);
                s.push('>');
                s
            }
            ExtType::Result(t, e) => {
                let tn = t.name();
                let en = e.name();
                let mut s = String::with_capacity(8 + tn.len() + en.len() + 1);
                s.push_str("Result<");
                s.push_str(&tn);
                s.push(',');
                s.push_str(&en);
                s.push('>');
                s
            }
            ExtType::RefExt { mutbl, inner, lifetime } => {
                let inner_n = inner.name();
                let lt = lifetime.as_deref().unwrap_or("");
                let mut s = String::with_capacity(1 + lt.len() + 4 + inner_n.len() + 2);
                s.push('&');
                if !lt.is_empty() {
                    s.push_str(lt);
                    s.push(' ');
                }
                if *mutbl {
                    s.push_str("mut ");
                }
                s.push_str(&inner_n);
                s
            }
            ExtType::GenericParam(s) => s.clone(),
            ExtType::Assoc { self_ty, trait_name, assoc } => {
                let self_n = self_ty.name();
                let mut s = String::with_capacity(6 + self_n.len() + trait_name.len() + assoc.len());
                s.push('<');
                s.push_str(&self_n);
                s.push_str(" as ");
                s.push_str(trait_name);
                s.push_str(">::");
                s.push_str(assoc);
                s
            }
            ExtType::Never => "!".to_string(),
            ExtType::Inferred => "_".to_string(),
            ExtType::Tuple(ts) => {
                if ts.is_empty() { "()".to_string() } else {
                    let parts: Vec<String> = ts.iter().map(|t| t.name()).collect();
                    let total: usize = parts.iter().map(|p| p.len()).sum::<usize>() + parts.len()*2;
                    let mut s = String::with_capacity(total);
                    s.push('(');
                    s.push_str(&parts.join(", "));
                    s.push(')');
                    s
                }
            }
            ExtType::Array { elem, len } => {
                let elem_n = elem.name();
                if let Some(l) = len {
                    let mut s = String::with_capacity(3 + elem_n.len() + l.len());
                    s.push('[');
                    s.push_str(&elem_n);
                    s.push_str("; ");
                    s.push_str(l);
                    s.push(']');
                    s
                } else {
                    let mut s = String::with_capacity(2 + elem_n.len());
                    s.push('[');
                    s.push_str(&elem_n);
                    s.push(']');
                    s
                }
            }
            ExtType::Slice(elem) => {
                let elem_n = elem.name();
                let mut s = String::with_capacity(2 + elem_n.len());
                s.push('[');
                s.push_str(&elem_n);
                s.push(']');
                s
            }
            ExtType::BareFn { params, ret } => {
                let ret_n = ret.name();
                let param_names: Vec<String> = params.iter().map(|p| p.name()).collect();
                let total: usize = param_names.iter().map(|p| p.len()).sum::<usize>() + ret_n.len() + 10;
                let mut s = String::with_capacity(total);
                s.push_str("fn(");
                s.push_str(&param_names.join(", "));
                s.push_str(")->");
                s.push_str(&ret_n);
                s
            }
            ExtType::TraitObject { bounds } => {
                let mut s = String::with_capacity(4 + bounds.join(" + ").len());
                s.push_str("dyn ");
                s.push_str(&bounds.join(" + "));
                s
            }
            ExtType::ImplTrait { bounds } => {
                let mut s = String::with_capacity(5 + bounds.join(" + ").len());
                s.push_str("impl ");
                s.push_str(&bounds.join(" + "));
                s
            }
        }
    }
    pub fn is_container(&self) -> bool {
        matches!(self, ExtType::Vec(_) | ExtType::HashMap(_, _) | ExtType::Option(_) | ExtType::Result(_, _))
    }
    pub fn variant_name(&self) -> &'static str {
        match self {
            ExtType::Vec(_) => "Vec",
            ExtType::String => "String",
            ExtType::HashMap(_, _) => "HashMap",
            ExtType::Struct { .. } => "Struct",
            ExtType::Enum { .. } => "Enum",
            ExtType::RawPtr { .. } => "RawPtr",
            ExtType::Future(_) => "Future",
            ExtType::Option(_) => "Option",
            ExtType::Result(_, _) => "Result",
            ExtType::RefExt { .. } => "RefExt",
            ExtType::GenericParam(_) => "GenericParam",
            ExtType::Assoc { .. } => "Assoc",
            ExtType::Never => "Never",
            ExtType::Inferred => "Inferred",
            ExtType::Tuple(_) => "Tuple",
            ExtType::Array { .. } => "Array",
            ExtType::Slice(_) => "Slice",
            ExtType::BareFn { .. } => "BareFn",
            ExtType::TraitObject { .. } => "TraitObject",
            ExtType::ImplTrait { .. } => "ImplTrait",
        }
    }
}

/// 完整型別 V2：Base + Ext，N = 7 + i
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum TypeV2 {
    Base(BaseType),
    Ext(ExtType),
}

impl TypeV2 {
    pub fn name(&self) -> String {
        match self {
            TypeV2::Base(b) => b.name().to_string(),
            TypeV2::Ext(e) => e.name(),
        }
    }
    pub fn is_base(&self) -> bool {
        matches!(self, TypeV2::Base(_))
    }
    pub fn is_ext(&self) -> bool {
        !self.is_base()
    }
    pub fn is_ref(&self) -> bool {
        match self {
            TypeV2::Base(BaseType::RefI32) | TypeV2::Base(BaseType::RefMutI32)
            | TypeV2::Base(BaseType::RefBool) | TypeV2::Base(BaseType::RefMutBool) => true,
            TypeV2::Ext(ExtType::RefExt { .. }) => true,
            _ => false,
        }
    }
    pub fn is_mut_ref(&self) -> bool {
        match self {
            TypeV2::Base(BaseType::RefMutI32) | TypeV2::Base(BaseType::RefMutBool) => true,
            TypeV2::Ext(ExtType::RefExt { mutbl: true, .. }) => true,
            _ => false,
        }
    }
    /// 實際使用：快速獲取內部類型引用，避免 clone
    pub fn inner_types(&self) -> Vec<&TypeV2> {
        match self {
            TypeV2::Ext(ExtType::Vec(inner)) => vec![inner],
            TypeV2::Ext(ExtType::HashMap(k, v)) => vec![k, v],
            TypeV2::Ext(ExtType::RawPtr { inner, .. }) => vec![inner],
            TypeV2::Ext(ExtType::Future(inner)) => vec![inner],
            TypeV2::Ext(ExtType::Option(inner)) => vec![inner],
            TypeV2::Ext(ExtType::Slice(inner)) => vec![inner],
            TypeV2::Ext(ExtType::RefExt { inner, .. }) => vec![inner],
            _ => vec![],
        }
    }
}

/// 型別宇宙：收集程序中出現的所有 TypeV2，去重，賦予緊湊索引
#[derive(Clone, Debug)]
pub struct Universe {
    pub types: Vec<TypeV2>,
    pub index_map: HashMap<TypeV2, usize>,
    pub base_indices: [usize; 7],
    pub ext_counts: ExtCounts,
}

#[derive(Clone, Debug, Default)]
pub struct ExtCounts {
    pub vec: usize,
    pub string: usize,
    pub hashmap: usize,
    pub structs: usize,
    pub enums: usize,
    pub raw_ptr: usize,
    pub future: usize,
    pub option: usize,
    pub result: usize,
    pub ref_ext: usize,
    pub generic: usize,
    pub tuple: usize,
    pub array: usize,
    pub slice: usize,
    pub bare_fn: usize,
    pub trait_object: usize,
    pub impl_trait: usize,
    pub total: usize,
}

impl Universe {
    pub fn new() -> Self {
        let mut types = Vec::with_capacity(16);
        let mut index_map = HashMap::with_capacity(16);
        let mut base_indices = [0usize; 7];
        for (i, bt) in BASE_TYPES.iter().enumerate() {
            let ty = TypeV2::Base(*bt);
            base_indices[i] = types.len();
            index_map.insert(ty.clone(), types.len());
            types.push(ty);
        }
        Universe {
            types,
            index_map,
            base_indices,
            ext_counts: ExtCounts::default(),
        }
    }
    pub fn insert(&mut self, ty: TypeV2) -> usize {
        if let Some(&idx) = self.index_map.get(&ty) {
            return idx;
        }
        let idx = self.types.len();
        if let TypeV2::Ext(ext) = &ty {
            match ext {
                ExtType::Vec(_) => self.ext_counts.vec += 1,
                ExtType::String => self.ext_counts.string += 1,
                ExtType::HashMap(_, _) => self.ext_counts.hashmap += 1,
                ExtType::Struct { .. } => self.ext_counts.structs += 1,
                ExtType::Enum { .. } => self.ext_counts.enums += 1,
                ExtType::RawPtr { .. } => self.ext_counts.raw_ptr += 1,
                ExtType::Future(_) => self.ext_counts.future += 1,
                ExtType::Option(_) => self.ext_counts.option += 1,
                ExtType::Result(_, _) => self.ext_counts.result += 1,
                ExtType::RefExt { .. } => self.ext_counts.ref_ext += 1,
                ExtType::GenericParam(_) => self.ext_counts.generic += 1,
                ExtType::Tuple(_) => self.ext_counts.tuple += 1,
                ExtType::Array { .. } => self.ext_counts.array += 1,
                ExtType::Slice(_) => self.ext_counts.slice += 1,
                ExtType::BareFn { .. } => self.ext_counts.bare_fn += 1,
                ExtType::TraitObject { .. } => self.ext_counts.trait_object += 1,
                ExtType::ImplTrait { .. } => self.ext_counts.impl_trait += 1,
                _ => {}
            }
            self.ext_counts.total += 1;
        }
        self.index_map.insert(ty.clone(), idx);
        self.types.push(ty);
        idx
    }
    pub fn insert_closure(&mut self, ty: TypeV2) -> usize {
        let idx = self.insert(ty.clone());
        match ty {
            TypeV2::Ext(ExtType::Vec(inner)) => { self.insert_closure((*inner).clone()); }
            TypeV2::Ext(ExtType::HashMap(k, v)) => {
                self.insert_closure((*k).clone());
                self.insert_closure((*v).clone());
            }
            TypeV2::Ext(ExtType::Struct { args, .. }) | TypeV2::Ext(ExtType::Enum { args, .. }) => {
                for a in args { self.insert_closure(a); }
            }
            TypeV2::Ext(ExtType::RawPtr { inner, .. }) => { self.insert_closure((*inner).clone()); }
            TypeV2::Ext(ExtType::Future(inner)) | TypeV2::Ext(ExtType::Option(inner)) | TypeV2::Ext(ExtType::Slice(inner)) => {
                self.insert_closure((*inner).clone());
            }
            TypeV2::Ext(ExtType::Result(t, e)) => {
                self.insert_closure((*t).clone());
                self.insert_closure((*e).clone());
            }
            TypeV2::Ext(ExtType::RefExt { inner, .. }) => { self.insert_closure((*inner).clone()); }
            TypeV2::Ext(ExtType::Assoc { self_ty, .. }) => { self.insert_closure((*self_ty).clone()); }
            TypeV2::Ext(ExtType::Tuple(ts)) => { for t in ts { self.insert_closure(t); } }
            TypeV2::Ext(ExtType::Array { elem, .. }) => { self.insert_closure((*elem).clone()); }
            TypeV2::Ext(ExtType::BareFn { params, ret }) => {
                for p in params { self.insert_closure(p); }
                self.insert_closure((*ret).clone());
            }
            _ => {}
        }
        idx
    }
    pub fn n_types(&self) -> usize { self.types.len() }
    pub fn n_base(&self) -> usize { 7 }
    pub fn n_ext(&self) -> usize { self.n_types() - 7 }
    pub fn bits_per_node(&self) -> usize { self.n_types() }
    pub fn index_of(&self, ty: &TypeV2) -> Option<usize> { self.index_map.get(ty).copied() }
    pub fn type_of(&self, idx: usize) -> Option<&TypeV2> { self.types.get(idx) }
    pub fn is_base_idx(&self, idx: usize) -> bool { idx < 7 }
    pub fn types(&self) -> &[TypeV2] { &self.types }
    pub fn one_hot_poly_text(&self, node_id: usize) -> String {
        let mut out = String::with_capacity(self.n_types()*8 + 32);
        for i in 0..self.n_types() {
            if i>0 { out.push_str(" + "); }
            out.push_str(&format!("t{}_{}", node_id, i));
        }
        out.push_str(&format!(" - 1 = 0  # Σ t =1, N={} (7+{})", self.n_types(), self.n_ext()));
        out
    }
    pub fn field_polys_text(&self, node_id: usize) -> Vec<String> {
        let mut v = Vec::with_capacity(self.n_types());
        for i in 0..self.n_types() {
            v.push(format!("t{}_{}^2 - t{}_{} = 0  # bool", node_id, i, node_id, i));
        }
        v
    }
    pub fn display(&self) -> String {
        let mut s = String::with_capacity(512 + self.n_types()*24);
        s.push_str(&format!("Universe N={} = 7 + {}\n", self.n_types(), self.n_ext()));
        s.push_str(&format!("  base: {} types\n", self.n_base()));
        for (i, bt) in BASE_TYPES.iter().enumerate() {
            s.push_str(&format!("    [{}] {}\n", self.base_indices[i], bt.name()));
        }
        s.push_str(&format!("  ext: {} types (vec={}, string={}, hashmap={}, struct={}, enum={}, raw_ptr={}, future={}, option={}, result={}, ref_ext={}, generic={}, tuple={}, array={}, slice={}, bare_fn={}, trait_obj={}, impl_trait={})\n",
            self.ext_counts.total,
            self.ext_counts.vec,
            self.ext_counts.string,
            self.ext_counts.hashmap,
            self.ext_counts.structs,
            self.ext_counts.enums,
            self.ext_counts.raw_ptr,
            self.ext_counts.future,
            self.ext_counts.option,
            self.ext_counts.result,
            self.ext_counts.ref_ext,
            self.ext_counts.generic,
            self.ext_counts.tuple,
            self.ext_counts.array,
            self.ext_counts.slice,
            self.ext_counts.bare_fn,
            self.ext_counts.trait_object,
            self.ext_counts.impl_trait
        ));
        for (i, ty) in self.types.iter().enumerate().skip(7) {
            s.push_str(&format!("    [{}] {}\n", i, ty.name()));
        }
        s
    }
    /// 實際使用：供 pipeline_v2 消費的摘要，with_capacity 優化
    pub fn summary_for_pipeline(&self) -> String {
        let mut s = String::with_capacity(256);
        s.push_str(&format!("Universe N={} base=7 ext={} vec={} string={} hashmap={} tuple={} array={} bare_fn={}\n",
            self.n_types(), self.n_ext(), self.ext_counts.vec, self.ext_counts.string, self.ext_counts.hashmap,
            self.ext_counts.tuple, self.ext_counts.array, self.ext_counts.bare_fn));
        s
    }
    /// 實際使用：判斷是否包含特定擴展類型
    pub fn contains_ext(&self, variant: &str) -> bool {
        self.types.iter().skip(7).any(|t| match t {
            TypeV2::Ext(e) => e.variant_name() == variant,
            _ => false,
        })
    }
}

impl Default for Universe {
    fn default() -> Self { Self::new() }
}

impl From<crate::minirust::ast::Type> for TypeV2 {
    fn from(t: crate::minirust::ast::Type) -> Self {
        use crate::minirust::ast::Type as Old;
        match t {
            Old::I32 => TypeV2::Base(BaseType::I32),
            Old::Bool => TypeV2::Base(BaseType::Bool),
            Old::Unit => TypeV2::Base(BaseType::Unit),
            Old::RefI32 => TypeV2::Base(BaseType::RefI32),
            Old::RefMutI32 => TypeV2::Base(BaseType::RefMutI32),
            Old::RefBool => TypeV2::Base(BaseType::RefBool),
            Old::RefMutBool => TypeV2::Base(BaseType::RefMutBool),
        }
    }
}

impl From<BaseType> for TypeV2 {
    fn from(b: BaseType) -> Self { TypeV2::Base(b) }
}

pub fn parse_type_v2(s: &str) -> Result<TypeV2, String> {
    let s = s.trim();
    match s {
        "i32" => return Ok(TypeV2::Base(BaseType::I32)),
        "bool" => return Ok(TypeV2::Base(BaseType::Bool)),
        "()" => return Ok(TypeV2::Base(BaseType::Unit)),
        "&i32" => return Ok(TypeV2::Base(BaseType::RefI32)),
        "&mut i32" => return Ok(TypeV2::Base(BaseType::RefMutI32)),
        "&bool" => return Ok(TypeV2::Base(BaseType::RefBool)),
        "&mut bool" => return Ok(TypeV2::Base(BaseType::RefMutBool)),
        "String" => return Ok(TypeV2::Ext(ExtType::String)),
        _ => {}
    }
    if s.starts_with("Vec<") && s.ends_with('>') {
        let inner = &s[4..s.len()-1];
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::Vec(Box::new(inner_ty))));
    }
    if s.starts_with("Option<") && s.ends_with('>') {
        let inner = &s[7..s.len()-1];
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::Option(Box::new(inner_ty))));
    }
    if s.starts_with("Result<") && s.ends_with('>') {
        let inner = &s[7..s.len()-1];
        let mut depth = 0;
        let mut split = None;
        for (i, c) in inner.chars().enumerate() {
            match c {
                '<' => depth+=1,
                '>' => depth-=1,
                ',' if depth==0 => { split = Some(i); break; }
                _ => {}
            }
        }
        if let Some(idx) = split {
            let t_str = inner[..idx].trim();
            let e_str = inner[idx+1..].trim();
            let t_ty = parse_type_v2(t_str)?;
            let e_ty = parse_type_v2(e_str)?;
            return Ok(TypeV2::Ext(ExtType::Result(Box::new(t_ty), Box::new(e_ty))));
        }
    }
    if s.starts_with("HashMap<") && s.ends_with('>') {
        let inner = &s[8..s.len()-1];
        let mut depth = 0;
        let mut split = None;
        for (i, c) in inner.chars().enumerate() {
            match c {
                '<' => depth+=1,
                '>' => depth-=1,
                ',' if depth==0 => { split = Some(i); break; }
                _ => {}
            }
        }
        if let Some(idx) = split {
            let k_str = inner[..idx].trim();
            let v_str = inner[idx+1..].trim();
            let k_ty = parse_type_v2(k_str)?;
            let v_ty = parse_type_v2(v_str)?;
            return Ok(TypeV2::Ext(ExtType::HashMap(Box::new(k_ty), Box::new(v_ty))));
        }
    }
    if s.starts_with("*mut ") {
        let inner = s[5..].trim();
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::RawPtr { mutbl: true, inner: Box::new(inner_ty) }));
    }
    if s.starts_with("*const ") {
        let inner = s[7..].trim();
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::RawPtr { mutbl: false, inner: Box::new(inner_ty) }));
    }
    if s.starts_with('&') {
        let rest = s[1..].trim();
        let (lt, rest) = if rest.starts_with('\'') {
            let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
            let lt = rest[..end].trim();
            let r = rest[end..].trim();
            (Some(lt.to_string()), r)
        } else {
            (None, rest)
        };
        if rest.starts_with("mut ") {
            let inner = rest[4..].trim();
            let inner_ty = parse_type_v2(inner)?;
            return Ok(TypeV2::Ext(ExtType::RefExt { mutbl: true, inner: Box::new(inner_ty), lifetime: lt }));
        } else {
            let inner_ty = parse_type_v2(rest)?;
            return Ok(TypeV2::Ext(ExtType::RefExt { mutbl: false, inner: Box::new(inner_ty), lifetime: lt }));
        }
    }
    if s.starts_with("Future<") && s.ends_with('>') {
        let inner = &s[7..s.len()-1];
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::Future(Box::new(inner_ty))));
    }
    if s.len() <= 3 && s.chars().all(|c| c.is_alphabetic()) {
        let is_generic = match s {
            "T" | "U" | "V" | "K" | "E" | "W" | "A" | "B" | "C" | "D" | "F" | "G" | "H" | "I" | "J" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" => true,
            _ if s.len() == 1 && s.chars().next().unwrap().is_uppercase() => true,
            _ => false,
        };
        if is_generic {
            return Ok(TypeV2::Ext(ExtType::GenericParam(s.to_string())));
        }
    }
    if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        if let Some(idx) = s.find('<') {
            if s.ends_with('>') {
                let name = s[..idx].trim().to_string();
                let args_str = &s[idx+1..s.len()-1];
                let args: Result<Vec<TypeV2>, String> = args_str.split(',')
                    .map(|a| parse_type_v2(a.trim()))
                    .collect();
                let args = args?;
                return Ok(TypeV2::Ext(ExtType::Struct { name, args }));
            }
        } else {
            return Ok(TypeV2::Ext(ExtType::Struct { name: s.to_string(), args: vec![] }));
        }
    }
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len()-1].trim();
        if inner.is_empty() {
            return Ok(TypeV2::Base(BaseType::Unit));
        }
        let mut parts = vec![];
        let mut depth_angle = 0;
        let mut depth_paren = 0;
        let mut depth_brack = 0;
        let mut start = 0;
        for (i, c) in inner.char_indices() {
            match c {
                '<' => depth_angle += 1,
                '>' => depth_angle -= 1,
                '(' => depth_paren += 1,
                ')' => depth_paren -= 1,
                '[' => depth_brack += 1,
                ']' => depth_brack -= 1,
                ',' if depth_angle==0 && depth_paren==0 && depth_brack==0 => {
                    parts.push(inner[start..i].trim());
                    start = i+1;
                }
                _ => {}
            }
        }
        parts.push(inner[start..].trim());
        let tys: Result<Vec<TypeV2>, String> = parts.into_iter().filter(|p| !p.is_empty()).map(parse_type_v2).collect();
        return Ok(TypeV2::Ext(ExtType::Tuple(tys?)));
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len()-1].trim();
        if inner.contains(';') {
            let mut depth = 0;
            let mut split = None;
            for (i, c) in inner.char_indices() {
                match c {
                    '<' => depth+=1,
                    '>' => depth-=1,
                    '(' => depth+=1,
                    ')' => depth-=1,
                    ';' if depth==0 => { split = Some(i); break; }
                    _ => {}
                }
            }
            if let Some(idx) = split {
                let elem_str = inner[..idx].trim();
                let len_str = inner[idx+1..].trim().to_string();
                let elem = parse_type_v2(elem_str)?;
                return Ok(TypeV2::Ext(ExtType::Array { elem: Box::new(elem), len: Some(len_str) }));
            }
        } else {
            if !inner.is_empty() {
                let elem = parse_type_v2(inner)?;
                return Ok(TypeV2::Ext(ExtType::Slice(Box::new(elem))));
            }
        }
    }
    if s.starts_with("fn(") || s.starts_with("fn (") {
        let paren_start = s.find('(').unwrap();
        let paren_end = s.find(')').ok_or_else(|| format!("無法解析類型: {}", s))?;
        let params_str = &s[paren_start+1..paren_end];
        let ret_str = if let Some(arrow) = s[paren_end..].find("->") {
            s[paren_end+arrow+2..].trim()
        } else {
            "()"
        };
        let params: Vec<TypeV2> = if params_str.trim().is_empty() { vec![] } else {
            params_str.split(',').map(|p| parse_type_v2(p.trim()).unwrap_or(TypeV2::Base(BaseType::I32))).collect()
        };
        let ret = parse_type_v2(ret_str).unwrap_or(TypeV2::Base(BaseType::Unit));
        return Ok(TypeV2::Ext(ExtType::BareFn { params, ret: Box::new(ret) }));
    }
    if s == "!" {
        return Ok(TypeV2::Ext(ExtType::Never));
    }
    if s == "_" {
        return Ok(TypeV2::Ext(ExtType::Inferred));
    }
    if s.starts_with("dyn ") {
        let bounds_str = s[4..].trim();
        let bounds = bounds_str.split('+').map(|b| b.trim().to_string()).collect();
        return Ok(TypeV2::Ext(ExtType::TraitObject { bounds }));
    }
    if s.starts_with("impl ") {
        let bounds_str = s[5..].trim();
        let bounds = bounds_str.split('+').map(|b| b.trim().to_string()).collect();
        return Ok(TypeV2::Ext(ExtType::ImplTrait { bounds }));
    }
    if s.len() <= 3 && s.chars().all(|c| c.is_alphabetic()) {
        return Ok(TypeV2::Ext(ExtType::GenericParam(s.to_string())));
    }
    Err(format!("無法解析類型: {}", s))
}

/// 實際使用：universe.rs 文件清單 — 優化 with_capacity
pub fn universe_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("universe.rs", "universe.rs 正式運作 — 優化 with_capacity", "core/src/minirust/universe.rs"),
    ]
}

