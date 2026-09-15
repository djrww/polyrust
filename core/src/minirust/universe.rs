//! Phase1 — 型別宇宙 7+i：從固定 7 種到可擴展 N = 7 + i
//!
//! v0.1.5 的 `Type` 是平坦枚舉 7 種（i32, bool, (), &i32, &mut i32, &bool, &mut bool）。
//! v0.2 Phase1 將其擴展為 `TypeV2`，保留 7 基底 + i 擴展（Vec, String, HashMap, Struct, Enum, RawPtr, Future, Option, Result 等）。
//!
//! 本模組零第三方依賴，只用 std，符合 core 承諾。

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
    /// Vec<T>
    Vec(Box<TypeV2>),
    /// String（獨立，非 Vec<u8> 糖）
    String,
    /// HashMap<K,V>
    HashMap(Box<TypeV2>, Box<TypeV2>),
    /// 具名 struct（字段在 Universe 中另存）
    Struct { name: String, args: Vec<TypeV2> },
    /// 具名 enum（變體在 Universe 中另存）
    Enum { name: String, args: Vec<TypeV2> },
    /// 裸指針 *mut T / *const T
    RawPtr { mutbl: bool, inner: Box<TypeV2> },
    /// Future<Output=T>
    Future(Box<TypeV2>),
    /// Option<T>
    Option(Box<TypeV2>),
    /// Result<T,E>
    Result(Box<TypeV2>, Box<TypeV2>),
    /// 引用帶生命週期 'a（Phase1 僅語法，語義在後續 Phase）
    RefExt { mutbl: bool, inner: Box<TypeV2>, lifetime: Option<String> },
    /// 泛型參數 T
    GenericParam(String),
    /// 關聯類型 <T as Trait>::Assoc
    Assoc { self_ty: Box<TypeV2>, trait_name: String, assoc: String },
    /// Never !
    Never,
    /// 推斷類型 _
    Inferred,
    /// 元組 (T1, T2, ...)
    Tuple(Vec<TypeV2>),
    /// 數組 [T; N] 或 [T]
    Array { elem: Box<TypeV2>, len: Option<String> },
    /// 切片 [T]
    Slice(Box<TypeV2>),
    /// 函數指針 fn(T1, T2) -> Ret
    BareFn { params: Vec<TypeV2>, ret: Box<TypeV2> },
    /// trait object dyn Trait + Send
    TraitObject { bounds: Vec<String> },
    /// impl Trait
    ImplTrait { bounds: Vec<String> },
}

impl ExtType {
    pub fn name(&self) -> String {
        match self {
            ExtType::Vec(t) => format!("Vec<{}>", t.name()),
            ExtType::String => "String".to_string(),
            ExtType::HashMap(k, v) => format!("HashMap<{},{}>", k.name(), v.name()),
            ExtType::Struct { name, args } => {
                if args.is_empty() { name.clone() } else {
                    format!("{}<{}>", name, args.iter().map(|a| a.name()).collect::<Vec<_>>().join(","))
                }
            }
            ExtType::Enum { name, args } => {
                if args.is_empty() { name.clone() } else {
                    format!("{}<{}>", name, args.iter().map(|a| a.name()).collect::<Vec<_>>().join(","))
                }
            }
            ExtType::RawPtr { mutbl, inner } => {
                if *mutbl { format!("*mut {}", inner.name()) } else { format!("*const {}", inner.name()) }
            }
            ExtType::Future(t) => format!("Future<{}>", t.name()),
            ExtType::Option(t) => format!("Option<{}>", t.name()),
            ExtType::Result(t, e) => format!("Result<{},{}>", t.name(), e.name()),
            ExtType::RefExt { mutbl, inner, lifetime } => {
                let lt = lifetime.as_deref().unwrap_or("");
                let lt_str = if lt.is_empty() { "".to_string() } else { format!("{} ", lt) };
                if *mutbl { format!("&{}{}mut {}", lt_str, "", inner.name()) } else { format!("&{}{}", lt_str, inner.name()) }
            }
            ExtType::GenericParam(s) => s.clone(),
            ExtType::Assoc { self_ty, trait_name, assoc } => format!("<{} as {}>::{}", self_ty.name(), trait_name, assoc),
            ExtType::Never => "!".to_string(),
            ExtType::Inferred => "_".to_string(),
            ExtType::Tuple(ts) => format!("({})", ts.iter().map(|t| t.name()).collect::<Vec<_>>().join(", ")),
            ExtType::Array { elem, len } => {
                if let Some(l) = len { format!("[{}; {}]", elem.name(), l) } else { format!("[{}]", elem.name()) }
            }
            ExtType::Slice(elem) => format!("[{}]", elem.name()),
            ExtType::BareFn { params, ret } => format!("fn({})->{}", params.iter().map(|p| p.name()).collect::<Vec<_>>().join(", "), ret.name()),
            ExtType::TraitObject { bounds } => format!("dyn {}", bounds.join(" + ")),
            ExtType::ImplTrait { bounds } => format!("impl {}", bounds.join(" + ")),
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

    /// 是否為引用（包含帶生命週期的）
    pub fn is_ref(&self) -> bool {
        match self {
            TypeV2::Base(BaseType::RefI32) | TypeV2::Base(BaseType::RefMutI32)
            | TypeV2::Base(BaseType::RefBool) | TypeV2::Base(BaseType::RefMutBool) => true,
            TypeV2::Ext(ExtType::RefExt { .. }) => true,
            _ => false,
        }
    }

    /// 是否為可變引用
    pub fn is_mut_ref(&self) -> bool {
        match self {
            TypeV2::Base(BaseType::RefMutI32) | TypeV2::Base(BaseType::RefMutBool) => true,
            TypeV2::Ext(ExtType::RefExt { mutbl: true, .. }) => true,
            _ => false,
        }
    }
}

/// 型別宇宙：收集程序中出現的所有 TypeV2，去重，賦予緊湊索引
/// N = 7 + i，其中 i = 擴展類型數
#[derive(Clone, Debug)]
pub struct Universe {
    /// 所有類型，按發現順序，0..N-1
    pub types: Vec<TypeV2>,
    /// 類型 → 索引
    pub index_map: HashMap<TypeV2, usize>,
    /// 基底 7 種的索引（固定前 7）
    pub base_indices: [usize; 7],
    /// 擴展類型的分組
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
    /// 從基底 7 構造空宇宙
    pub fn new() -> Self {
        let mut types = Vec::with_capacity(16);
        let mut index_map = HashMap::new();
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

    /// 插入一個類型，若已存在則返回現有索引，否則新增
    pub fn insert(&mut self, ty: TypeV2) -> usize {
        if let Some(&idx) = self.index_map.get(&ty) {
            return idx;
        }
        let idx = self.types.len();
        // 更新計數
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

    /// 批量插入並閉包（子類型、泛型實參）— Path C 擴展 Tuple/Array/Slice/BareFn
    pub fn insert_closure(&mut self, ty: TypeV2) -> usize {
        let idx = self.insert(ty.clone());
        // 遞歸插入子類型
        match ty {
            TypeV2::Ext(ExtType::Vec(inner)) => {
                self.insert_closure((*inner).clone());
            }
            TypeV2::Ext(ExtType::HashMap(k, v)) => {
                self.insert_closure((*k).clone());
                self.insert_closure((*v).clone());
            }
            TypeV2::Ext(ExtType::Struct { args, .. }) | TypeV2::Ext(ExtType::Enum { args, .. }) => {
                for a in args {
                    self.insert_closure(a);
                }
            }
            TypeV2::Ext(ExtType::RawPtr { inner, .. }) => {
                self.insert_closure((*inner).clone());
            }
            TypeV2::Ext(ExtType::Future(inner)) | TypeV2::Ext(ExtType::Option(inner)) | TypeV2::Ext(ExtType::Slice(inner)) => {
                self.insert_closure((*inner).clone());
            }
            TypeV2::Ext(ExtType::Result(t, e)) => {
                self.insert_closure((*t).clone());
                self.insert_closure((*e).clone());
            }
            TypeV2::Ext(ExtType::RefExt { inner, .. }) => {
                self.insert_closure((*inner).clone());
            }
            TypeV2::Ext(ExtType::Assoc { self_ty, .. }) => {
                self.insert_closure((*self_ty).clone());
            }
            TypeV2::Ext(ExtType::Tuple(ts)) => {
                for t in ts { self.insert_closure(t); }
            }
            TypeV2::Ext(ExtType::Array { elem, .. }) => {
                self.insert_closure((*elem).clone());
            }
            TypeV2::Ext(ExtType::BareFn { params, ret }) => {
                for p in params { self.insert_closure(p); }
                self.insert_closure((*ret).clone());
            }
            _ => {}
        }
        idx
    }

    pub fn n_types(&self) -> usize {
        self.types.len()
    }

    pub fn n_base(&self) -> usize {
        7
    }

    pub fn n_ext(&self) -> usize {
        self.n_types() - 7
    }

    /// one-hot 編碼的變量數：每個節點 N 個位元
    pub fn bits_per_node(&self) -> usize {
        self.n_types()
    }

    /// 獲取類型的索引
    pub fn index_of(&self, ty: &TypeV2) -> Option<usize> {
        self.index_map.get(ty).copied()
    }

    /// 從索引獲取類型
    pub fn type_of(&self, idx: usize) -> Option<&TypeV2> {
        self.types.get(idx)
    }

    /// 判斷是否為 base
    pub fn is_base_idx(&self, idx: usize) -> bool {
        idx < 7
    }

    /// 生成 one-hot 多項式文本（用於調試/文檔）
    pub fn one_hot_poly_text(&self, node_id: usize) -> String {
        let terms: Vec<String> = (0..self.n_types())
            .map(|i| format!("t{}_{}", node_id, i))
            .collect();
        format!("{} - 1 = 0  # Σ t =1, N={} (7+{})", terms.join(" + "), self.n_types(), self.n_ext())
    }

    /// 生成域多項式文本
    pub fn field_polys_text(&self, node_id: usize) -> Vec<String> {
        (0..self.n_types())
            .map(|i| format!("t{}_{}^2 - t{}_{} = 0  # bool", node_id, i, node_id, i))
            .collect()
    }

    /// 打印宇宙
    pub fn display(&self) -> String {
        let mut s = String::new();
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
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
}

/// 從舊 Type 轉換到 TypeV2::Base
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
    fn from(b: BaseType) -> Self {
        TypeV2::Base(b)
    }
}

/// 解析一個類型字符串為 TypeV2（Phase1 簡化解析，支持 Vec<T>, Option<T>, Result<T,E>, HashMap<K,V>, String, *mut T, *const T, &T, &mut T, &'a T）
pub fn parse_type_v2(s: &str) -> Result<TypeV2, String> {
    let s = s.trim();
    // base
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
    // Vec<T>
    if s.starts_with("Vec<") && s.ends_with('>') {
        let inner = &s[4..s.len()-1];
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::Vec(Box::new(inner_ty))));
    }
    // Option<T>
    if s.starts_with("Option<") && s.ends_with('>') {
        let inner = &s[7..s.len()-1];
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::Option(Box::new(inner_ty))));
    }
    // Result<T,E>
    if s.starts_with("Result<") && s.ends_with('>') {
        let inner = &s[7..s.len()-1];
        // 簡單 split 最外層逗號
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
    // HashMap<K,V>
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
    // *mut T / *const T
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
    // &mut T / &T / &'a T / &'a mut T
    if s.starts_with('&') {
        let rest = s[1..].trim();
        // 檢查 lifetime
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
    // Future<T>
    if s.starts_with("Future<") && s.ends_with('>') {
        let inner = &s[7..s.len()-1];
        let inner_ty = parse_type_v2(inner)?;
        return Ok(TypeV2::Ext(ExtType::Future(Box::new(inner_ty))));
    }
    // 泛型參數優先：單大寫字母 T,U,V,K,E 等，或常見泛型名
    if s.len() <= 3 && s.chars().all(|c| c.is_alphabetic()) {
        // 若全大寫單字母或首字母大寫且長度<=3，優先視為泛型參數
        // 但需排除已知類型 String, Point 等？Point 長度 5，已排除
        // 為避免誤判，僅當 s 為單字符大寫或常見泛型名時視為 GenericParam
        let is_generic = match s {
            "T" | "U" | "V" | "K" | "E" | "W" | "A" | "B" | "C" | "D" | "F" | "G" | "H" | "I" | "J" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" => true,
            _ if s.len() == 1 && s.chars().next().unwrap().is_uppercase() => true,
            _ => false,
        };
        if is_generic {
            return Ok(TypeV2::Ext(ExtType::GenericParam(s.to_string())));
        }
    }
    // Struct/Enum 泛型無參或有參：Name 或 Name<...>
    // 若首字母大寫，視為具名類型
    if s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        if let Some(idx) = s.find('<') {
            if s.ends_with('>') {
                let name = s[..idx].trim().to_string();
                let args_str = &s[idx+1..s.len()-1];
                // 簡單逗號分割（不處理嵌套，Phase1 簡化）
                let args: Result<Vec<TypeV2>, String> = args_str.split(',')
                    .map(|a| parse_type_v2(a.trim()))
                    .collect();
                let args = args?;
                // 默認當 struct
                return Ok(TypeV2::Ext(ExtType::Struct { name, args }));
            }
        } else {
            // 無泛型參數的具名類型，視為 struct
            return Ok(TypeV2::Ext(ExtType::Struct { name: s.to_string(), args: vec![] }));
        }
    }
    // Path C 新增：Tuple (T1, T2, ...) — 平衡括號解析
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len()-1].trim();
        if inner.is_empty() {
            return Ok(TypeV2::Base(BaseType::Unit));
        }
        // 分割逗號，考慮嵌套 < > ( ) [ ]
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
        let tys: Result<Vec<TypeV2>, String> = parts.into_iter().filter(|p| !p.is_empty()).map(|p| parse_type_v2(p)).collect();
        return Ok(TypeV2::Ext(ExtType::Tuple(tys?)));
    }

    // Path C 新增：Array [T; N] 或 [T; 3] 或 [T]
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len()-1].trim();
        if inner.contains(';') {
            // [T; N]
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
            // [T] 視為 Slice 或 Array 無長度
            if !inner.is_empty() {
                let elem = parse_type_v2(inner)?;
                // 默認當 Slice，若需 Array 可後續區分
                return Ok(TypeV2::Ext(ExtType::Slice(Box::new(elem))));
            }
        }
    }

    // Path C 新增：BareFn fn(T1, T2) -> Ret
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

    // Path C 新增：Never !, Inferred _
    if s == "!" {
        return Ok(TypeV2::Ext(ExtType::Never));
    }
    if s == "_" {
        return Ok(TypeV2::Ext(ExtType::Inferred));
    }

    // Path C 新增：dyn Trait + Send, impl Trait
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

    // 泛型參數（多字符小寫或全大寫短名）
    if s.len() <= 3 && s.chars().all(|c| c.is_alphabetic()) {
        return Ok(TypeV2::Ext(ExtType::GenericParam(s.to_string())));
    }

    Err(format!("無法解析類型: {}", s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_new() {
        let u = Universe::new();
        assert_eq!(u.n_types(), 7);
        assert_eq!(u.n_ext(), 0);
        assert_eq!(u.types[0], TypeV2::Base(BaseType::I32));
    }

    #[test]
    fn test_parse_base() {
        assert_eq!(parse_type_v2("i32").unwrap(), TypeV2::Base(BaseType::I32));
        assert_eq!(parse_type_v2("bool").unwrap(), TypeV2::Base(BaseType::Bool));
        assert_eq!(parse_type_v2("String").unwrap(), TypeV2::Ext(ExtType::String));
    }

    #[test]
    fn test_parse_vec() {
        let ty = parse_type_v2("Vec<i32>").unwrap();
        match ty {
            TypeV2::Ext(ExtType::Vec(inner)) => assert_eq!(*inner, TypeV2::Base(BaseType::I32)),
            _ => panic!("wrong"),
        }
    }

    #[test]
    fn test_parse_hashmap() {
        let ty = parse_type_v2("HashMap<String,i32>").unwrap();
        match ty {
            TypeV2::Ext(ExtType::HashMap(k, v)) => {
                assert_eq!(*k, TypeV2::Ext(ExtType::String));
                assert_eq!(*v, TypeV2::Base(BaseType::I32));
            }
            _ => panic!("wrong"),
        }
    }

    #[test]
    fn test_universe_ext() {
        let mut u = Universe::new();
        let ty1 = parse_type_v2("Vec<i32>").unwrap();
        let ty2 = parse_type_v2("String").unwrap();
        let ty3 = parse_type_v2("HashMap<String,i32>").unwrap();
        u.insert_closure(ty1);
        u.insert_closure(ty2);
        u.insert_closure(ty3);
        assert_eq!(u.n_types(), 7 + 3); // Vec<i32>, String, HashMap<String,i32> (+ i32 已存在)
        // String 已存在，不重複
        assert!(u.n_ext() >= 2);
        println!("{}", u.display());
    }

    #[test]
    fn test_one_hot_text() {
        let mut u = Universe::new();
        u.insert_closure(parse_type_v2("Vec<i32>").unwrap());
        let text = u.one_hot_poly_text(0);
        assert!(text.contains("t0_"));
        assert!(text.contains("7+"));
    }

    #[test]
    fn test_parse_complex() {
        let cases = vec![
            "Option<i32>",
            "Result<i32,bool>",
            "*mut i32",
            "*const bool",
            "&i32",
            "&mut i32",
            "&'a i32",
            "Future<i32>",
            "Point",
            "Vec<String>",
        ];
        for c in cases {
            let ty = parse_type_v2(c);
            assert!(ty.is_ok(), "failed to parse {}: {:?}", c, ty.err());
        }
    }
}
