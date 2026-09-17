//! Unsafe Safety 前移：把執行期才驗到既 unsafe bug，推前到靜態開發階段鎖定
//! 核心思想：每個 unsafe 操作必須附帶 safety 證明，編碼為多項式約束，Lean 中證明約束 → 執行期無 UB

use std::collections::HashMap;

/// 裸指針安全證明
#[derive(Clone, Debug)]
pub struct RawPtrSafety {
    pub node_id: usize,
    pub ptr_var: usize,           // 指針變量
    pub is_non_null_var: usize,   // 非空證明
    pub is_aligned_var: usize,    // 對齊證明
    pub in_bounds_var: usize,     // 邊界內證明
    pub not_dangling_var: usize,  // 非懸垂證明
    pub valid_var: usize,         // 總有效性 = non_null ∧ aligned ∧ in_bounds ∧ not_dangling
    pub deref_var: usize,         // 解引用操作
    pub poly_texts: Vec<String>,
}

/// static mut 安全證明
#[derive(Clone, Debug)]
pub struct StaticMutSafety {
    pub node_id: usize,
    pub access_var: usize,        // 訪問操作
    pub in_unsafe_var: usize,     // 在 unsafe 塊
    pub exclusive_var: usize,     // 獨佔訪問證明
    pub mutex_protected_var: usize, // Mutex 保護證明
    pub single_threaded_var: usize, // 單線程證明
    pub safe_var: usize,          // 總安全 = in_unsafe ∧ (exclusive ∨ mutex ∨ single_threaded)
    pub poly_texts: Vec<String>,
}

/// union 安全證明
#[derive(Clone, Debug)]
pub struct UnionSafety {
    pub node_id: usize,
    pub union_var: usize,         // union 變量
    pub active_tag_var: usize,    // 當前活躍字段 tag
    pub accessed_tag_var: usize,  // 訪問的字段 tag
    pub tag_match_var: usize,     // tag 匹配證明
    pub in_unsafe_var: usize,
    pub safe_var: usize,          // safe = in_unsafe ∧ tag_match
    pub poly_texts: Vec<String>,
}

/// unsafe fn 安全證明
#[derive(Clone, Debug)]
pub struct UnsafeFnSafety {
    pub node_id: usize,
    pub fn_name: String,
    pub call_var: usize,          // 調用操作
    pub in_unsafe_var: usize,
    pub precond_var: usize,       // 前置條件滿足
    pub safe_var: usize,          // safe = in_unsafe ∧ precond
    pub poly_texts: Vec<String>,
}

/// Unsafe Trait 實現安全
#[derive(Clone, Debug)]
pub struct UnsafeTraitSafety {
    pub node_id: usize,
    pub trait_name: String,
    pub impl_var: usize,
    pub is_unsafe_impl_var: usize,
    pub invariant_var: usize,     // 不變量滿足
    pub safe_var: usize,
    pub poly_texts: Vec<String>,
}

use crate::minirust::constraints_v2::{SystemV2, var_poly, emit, mono_of};
use crate::poly::Poly;
use crate::frac::Frac;

/// 生成裸指針安全約束：valid = non_null ∧ aligned ∧ in_bounds ∧ not_dangling
/// 並且 deref 需要 valid
pub fn gen_raw_ptr_safety(sys: &mut SystemV2, node_id: usize) -> RawPtrSafety {
    let ptr_var = sys.nvars; sys.names.push(format!("ptr_{}", node_id)); sys.nvars+=1;
    let non_null = sys.nvars; sys.names.push(format!("non_null_{}", node_id)); sys.nvars+=1;
    let aligned = sys.nvars; sys.names.push(format!("aligned_{}", node_id)); sys.nvars+=1;
    let in_bounds = sys.nvars; sys.names.push(format!("in_bounds_{}", node_id)); sys.nvars+=1;
    let not_dangling = sys.nvars; sys.names.push(format!("not_dangling_{}", node_id)); sys.nvars+=1;
    let valid = sys.nvars; sys.names.push(format!("valid_ptr_{}", node_id)); sys.nvars+=1;
    let deref = sys.nvars; sys.names.push(format!("deref_{}", node_id)); sys.nvars+=1;
    let in_unsafe = sys.nvars; sys.names.push(format!("in_unsafe_raw_{}", node_id)); sys.nvars+=1;

    let mut texts = vec![];

    // boolean 約束：每個 safety 位元都是 0/1
    for &v in &[non_null, aligned, in_bounds, not_dangling, valid, deref, in_unsafe] {
        let poly = var_poly(sys.nvars, v).mul(&var_poly(sys.nvars, v).sub(&Poly::constant(Frac::ONE)));
        emit(sys, poly);
        texts.push(format!("t_{}*(t_{}-1)=0 # bool", v, v));
    }

    // valid = non_null ∧ aligned ∧ in_bounds ∧ not_dangling
    // 編碼：valid = non_null * aligned * in_bounds * not_dangling
    // 簡化：valid - non_null*aligned =0, valid - in_bounds*not_dangling=0 等
    // 這裡用乘積：valid - non_null*aligned*in_bounds*not_dangling =0
    let mut prod_terms = vec![(mono_of(non_null, sys.nvars), Frac::ONE)];
    // 為簡化，我們生成：valid - non_null =0 且 valid - aligned =0 等，實際應為合取，暫用多個約束近似
    // 約束1: valid - non_null*aligned =0
    let poly1 = var_poly(sys.nvars, valid).sub(&var_poly(sys.nvars, non_null).mul(&var_poly(sys.nvars, aligned)));
    emit(sys, poly1); texts.push(format!("valid_{} - non_null_{}*aligned_{}=0", node_id, node_id, node_id));
    // 約束2: valid - in_bounds*not_dangling=0
    let poly2 = var_poly(sys.nvars, valid).sub(&var_poly(sys.nvars, in_bounds).mul(&var_poly(sys.nvars, not_dangling)));
    emit(sys, poly2); texts.push(format!("valid_{} - in_bounds_{}*not_dangling_{}=0", node_id, node_id, node_id));

    // deref 需要 valid 且 in_unsafe
    // (1 - valid)*deref =0
    let poly3 = Poly::constant(Frac::ONE).sub(&var_poly(sys.nvars, valid)).mul(&var_poly(sys.nvars, deref));
    emit(sys, poly3); texts.push(format!("(1-valid_{})*deref_{}=0 # deref requires valid", node_id, node_id));
    // (1 - in_unsafe)*deref =0
    let poly4 = Poly::constant(Frac::ONE).sub(&var_poly(sys.nvars, in_unsafe)).mul(&var_poly(sys.nvars, deref));
    emit(sys, poly4); texts.push(format!("(1-in_unsafe_raw_{})*deref_{}=0 # deref requires unsafe", node_id, node_id));

    RawPtrSafety {
        node_id,
        ptr_var,
        is_non_null_var: non_null,
        is_aligned_var: aligned,
        in_bounds_var: in_bounds,
        not_dangling_var: not_dangling,
        valid_var: valid,
        deref_var: deref,
        poly_texts: texts,
    }
}

/// static mut 安全：safe = in_unsafe ∧ (exclusive ∨ mutex ∨ single_threaded)
pub fn gen_static_mut_safety(sys: &mut SystemV2, node_id: usize) -> StaticMutSafety {
    let access = sys.nvars; sys.names.push(format!("static_mut_access_{}", node_id)); sys.nvars+=1;
    let in_unsafe = sys.nvars; sys.names.push(format!("in_unsafe_static_{}", node_id)); sys.nvars+=1;
    let exclusive = sys.nvars; sys.names.push(format!("exclusive_{}", node_id)); sys.nvars+=1;
    let mutex_prot = sys.nvars; sys.names.push(format!("mutex_prot_{}", node_id)); sys.nvars+=1;
    let single_thread = sys.nvars; sys.names.push(format!("single_thread_{}", node_id)); sys.nvars+=1;
    let safe = sys.nvars; sys.names.push(format!("safe_static_{}", node_id)); sys.nvars+=1;

    let mut texts = vec![];
    for &v in &[access, in_unsafe, exclusive, mutex_prot, single_thread, safe] {
        let poly = var_poly(sys.nvars, v).mul(&var_poly(sys.nvars, v).sub(&Poly::constant(Frac::ONE)));
        emit(sys, poly);
    }

    // safe = in_unsafe * (exclusive + mutex + single - exclusive*mutex - ...)
    // 簡化：safe - in_unsafe*exclusive =0  OR  safe - in_unsafe*mutex =0  OR  safe - in_unsafe*single=0
    // 我們生成三個可能的證明路徑，任一滿足即可，實際用 OR 編碼較複雜，這裡近似為 safe <= in_unsafe 且 safe <= (exclusive+mutex+single)
    // 約束1: (1 - in_unsafe)*safe =0
    let p1 = Poly::constant(Frac::ONE).sub(&var_poly(sys.nvars, in_unsafe)).mul(&var_poly(sys.nvars, safe));
    emit(sys, p1); texts.push(format!("(1-in_unsafe_static_{})*safe_static_{}=0", node_id, node_id));
    // 約束2: safe - (exclusive + mutex + single) <=0 近似為 safe*(1 - exclusive)*(1 - mutex)*(1 - single)=0
    // 即 safe=1 則至少一個保護為1
    let one = Poly::constant(Frac::ONE);
    let prod = one.sub(&var_poly(sys.nvars, exclusive))
        .mul(&one.sub(&var_poly(sys.nvars, mutex_prot)))
        .mul(&one.sub(&var_poly(sys.nvars, single_thread)))
        .mul(&var_poly(sys.nvars, safe));
    emit(sys, prod); texts.push(format!("safe_static_{}*(1-exclusive)*(1-mutex)*(1-single)=0 # need one protection", node_id));

    // access 需要 safe
    let p2 = one.sub(&var_poly(sys.nvars, safe)).mul(&var_poly(sys.nvars, access));
    emit(sys, p2); texts.push(format!("(1-safe_static_{})*access_{}=0 # static mut access requires safe", node_id, node_id));

    StaticMutSafety {
        node_id,
        access_var: access,
        in_unsafe_var: in_unsafe,
        exclusive_var: exclusive,
        mutex_protected_var: mutex_prot,
        single_threaded_var: single_thread,
        safe_var: safe,
        poly_texts: texts,
    }
}

/// union 安全：tag 匹配
pub fn gen_union_safety(sys: &mut SystemV2, node_id: usize) -> UnionSafety {
    let union_var = sys.nvars; sys.names.push(format!("union_{}", node_id)); sys.nvars+=1;
    let active_tag = sys.nvars; sys.names.push(format!("active_tag_{}", node_id)); sys.nvars+=1;
    let accessed_tag = sys.nvars; sys.names.push(format!("accessed_tag_{}", node_id)); sys.nvars+=1;
    let tag_match = sys.nvars; sys.names.push(format!("tag_match_{}", node_id)); sys.nvars+=1;
    let in_unsafe = sys.nvars; sys.names.push(format!("in_unsafe_union_{}", node_id)); sys.nvars+=1;
    let safe = sys.nvars; sys.names.push(format!("safe_union_{}", node_id)); sys.nvars+=1;

    let mut texts = vec![];
    for &v in &[active_tag, accessed_tag, tag_match, in_unsafe, safe] {
        let poly = var_poly(sys.nvars, v).mul(&var_poly(sys.nvars, v).sub(&Poly::constant(Frac::ONE)));
        emit(sys, poly);
    }

    // tag_match = 1 iff active_tag == accessed_tag
    // 簡化：tag_match - (1 - (active - accessed)^2) =0  近似為 tag_match*(active - accessed)=0 且 (1-tag_match)*(1 - (active-accessed)^2)=0
    // 這裡簡化為：tag_match*(active - accessed)=0 表示若 tag_match=1 則 active==accessed
    let diff = var_poly(sys.nvars, active_tag).sub(&var_poly(sys.nvars, accessed_tag));
    let p1 = var_poly(sys.nvars, tag_match).mul(&diff.clone());
    emit(sys, p1); texts.push(format!("tag_match_{}*(active_tag_{}-accessed_tag_{})=0 # tag match implies equality", node_id, node_id, node_id));

    // safe = in_unsafe * tag_match
    let p2 = var_poly(sys.nvars, safe).sub(&var_poly(sys.nvars, in_unsafe).mul(&var_poly(sys.nvars, tag_match)));
    emit(sys, p2); texts.push(format!("safe_union_{} - in_unsafe_union_{}*tag_match_{}=0", node_id, node_id, node_id));

    UnionSafety {
        node_id,
        union_var,
        active_tag_var: active_tag,
        accessed_tag_var: accessed_tag,
        tag_match_var: tag_match,
        in_unsafe_var: in_unsafe,
        safe_var: safe,
        poly_texts: texts,
    }
}

/// unsafe fn 安全：precond
pub fn gen_unsafe_fn_safety(sys: &mut SystemV2, node_id: usize, fn_name: &str) -> UnsafeFnSafety {
    let call = sys.nvars; sys.names.push(format!("call_unsafe_fn_{}_{}", fn_name, node_id)); sys.nvars+=1;
    let in_unsafe = sys.nvars; sys.names.push(format!("in_unsafe_fn_{}_{}", fn_name, node_id)); sys.nvars+=1;
    let precond = sys.nvars; sys.names.push(format!("precond_{}_{}", fn_name, node_id)); sys.nvars+=1;
    let safe = sys.nvars; sys.names.push(format!("safe_fn_{}_{}", fn_name, node_id)); sys.nvars+=1;

    let mut texts = vec![];
    for &v in &[call, in_unsafe, precond, safe] {
        let poly = var_poly(sys.nvars, v).mul(&var_poly(sys.nvars, v).sub(&Poly::constant(Frac::ONE)));
        emit(sys, poly);
    }

    // safe = in_unsafe * precond
    let p1 = var_poly(sys.nvars, safe).sub(&var_poly(sys.nvars, in_unsafe).mul(&var_poly(sys.nvars, precond)));
    emit(sys, p1); texts.push(format!("safe_fn_{}_{} - in_unsafe_fn_{}_{}*precond_{}_{}=0", fn_name, node_id, fn_name, node_id, fn_name, node_id));

    // call 需要 safe
    let p2 = Poly::constant(Frac::ONE).sub(&var_poly(sys.nvars, safe)).mul(&var_poly(sys.nvars, call));
    emit(sys, p2); texts.push(format!("(1-safe_fn_{}_{})*call_{}_{}=0 # unsafe fn call requires safe", fn_name, node_id, fn_name, node_id));

    UnsafeFnSafety {
        node_id,
        fn_name: fn_name.to_string(),
        call_var: call,
        in_unsafe_var: in_unsafe,
        precond_var: precond,
        safe_var: safe,
        poly_texts: texts,
    }
}

/// Unsafe Trait 安全
pub fn gen_unsafe_trait_safety(sys: &mut SystemV2, node_id: usize, trait_name: &str) -> UnsafeTraitSafety {
    let impl_var = sys.nvars; sys.names.push(format!("impl_{}_{}", trait_name, node_id)); sys.nvars+=1;
    let is_unsafe_impl = sys.nvars; sys.names.push(format!("is_unsafe_impl_{}_{}", trait_name, node_id)); sys.nvars+=1;
    let invariant = sys.nvars; sys.names.push(format!("invariant_{}_{}", trait_name, node_id)); sys.nvars+=1;
    let safe = sys.nvars; sys.names.push(format!("safe_trait_{}_{}", trait_name, node_id)); sys.nvars+=1;

    let mut texts = vec![];
    for &v in &[impl_var, is_unsafe_impl, invariant, safe] {
        let poly = var_poly(sys.nvars, v).mul(&var_poly(sys.nvars, v).sub(&Poly::constant(Frac::ONE)));
        emit(sys, poly);
    }

    // safe = is_unsafe_impl * invariant
    let p1 = var_poly(sys.nvars, safe).sub(&var_poly(sys.nvars, is_unsafe_impl).mul(&var_poly(sys.nvars, invariant)));
    emit(sys, p1); texts.push(format!("safe_trait_{}_{} - is_unsafe_impl_{}_{}*invariant_{}_{}=0", trait_name, node_id, trait_name, node_id, trait_name, node_id));

    // impl 需要 safe
    let p2 = Poly::constant(Frac::ONE).sub(&var_poly(sys.nvars, safe)).mul(&var_poly(sys.nvars, impl_var));
    emit(sys, p2); texts.push(format!("(1-safe_trait_{}_{})*impl_{}_{}=0 # unsafe trait impl requires safe", trait_name, node_id, trait_name, node_id));

    UnsafeTraitSafety {
        node_id,
        trait_name: trait_name.to_string(),
        impl_var,
        is_unsafe_impl_var: is_unsafe_impl,
        invariant_var: invariant,
        safe_var: safe,
        poly_texts: texts,
    }
}

pub fn unsafe_safety_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("unsafe_safety.rs", "unsafe 前移：裸指針/static mut/union/unsafe fn/unsafe trait 多項式約束", "core/src/minirust/unsafe_safety.rs"),
    ]
}
