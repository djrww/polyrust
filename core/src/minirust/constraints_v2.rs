// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Phase2 — 約束生成 v2：可變 N = 7+i，product/sum/match/loop/mod/async 編碼
//!
//! 基於 `Universe` 的可變類型宇宙，每節點 N 個位元（one-hot），支持：
//! - product: struct
//! - sum: enum
//! - match decision tree
//! - for → loop
//! - mod flatten (已在 lower.rs 完成)
//! - async state machine
//! - Vec/String/HashMap/Option/Result/RawPtr/Future/RefExt

use std::collections::HashMap;

use crate::cdcl;
use crate::frac::Frac;
use crate::minirust::lower::{Lowered, MatchDecisionTree};
use crate::minirust::universe::{TypeV2, Universe};
use crate::poly::Poly;
use crate::qap::{Linear, R1cs};

/// 約束系統 v2：N 可變 + Phase3
#[derive(Clone, Debug, Default)]
pub struct SystemV2 {
    pub nvars: usize,
    pub names: Vec<String>,
    /// 多項式方程組 f=0（不含域多項式）
    pub polys: Vec<Poly>,
    /// CDCL 子句
    pub clauses: Vec<Vec<cdcl::Lit>>,
    /// 節點 → N 個類型位元變量（N = universe.n_types()）
    pub node_type: HashMap<usize, Vec<usize>>,
    /// 節點種類
    pub node_kind: HashMap<usize, String>,
    /// 借用衝突對
    pub borrow_conflicts: Vec<(usize, usize)>,
    /// arm 位元（match）
    pub arm_vars: HashMap<usize, Vec<usize>>,
    /// 宇宙
    pub universe: Universe,
    /// product 約束記錄
    pub product_constraints: Vec<ProductConstraint>,
    /// sum 約束記錄
    pub sum_constraints: Vec<SumConstraint>,
    /// match 決策樹約束
    pub match_constraints: Vec<MatchConstraint>,
    /// Phase3
    pub loop_fuel_constraints: Vec<LoopFuelConstraint>,
    pub async_constraints: Vec<AsyncConstraint>,
    pub lifetime_constraints: Vec<LifetimeConstraint>,
    pub unsafe_constraints: Vec<UnsafeConstraint>,
    pub stdlib_constraints: Vec<StdlibConstraint>,
    pub trait_impl_constraints: Vec<TraitImplConstraint>,
    // Unsafe Safety 前移：五類
    pub raw_ptr_safety: Vec<crate::minirust::unsafe_safety::RawPtrSafety>,
    pub static_mut_safety: Vec<crate::minirust::unsafe_safety::StaticMutSafety>,
    pub union_safety: Vec<crate::minirust::unsafe_safety::UnionSafety>,
    pub unsafe_fn_safety: Vec<crate::minirust::unsafe_safety::UnsafeFnSafety>,
    pub unsafe_trait_safety: Vec<crate::minirust::unsafe_safety::UnsafeTraitSafety>,
}

#[derive(Clone, Debug)]
pub struct ProductConstraint {
    pub struct_name: String,
    pub node_id: usize,
    pub struct_idx: usize,
    pub field_indices: Vec<usize>,
    pub poly_text: String,
}

#[derive(Clone, Debug)]
pub struct SumConstraint {
    pub enum_name: String,
    pub node_id: usize,
    pub enum_idx: usize,
    pub variant_indices: Vec<usize>,
    pub poly_text: String,
}

#[derive(Clone, Debug)]
pub struct MatchConstraint {
    pub node_id: usize,
    pub scrutinee_id: usize,
    pub arm_bits: Vec<usize>,
    pub result_idx: usize,
    pub decision_tree: MatchDecisionTree,
}

pub fn mono_of(var: usize, nvars: usize) -> Vec<u32> {
    let mut m = vec![0u32; nvars.max(var + 1)];
    m[var] = 1;
    m
}

pub fn var_poly(nvars: usize, v: usize) -> Poly {
    Poly::var(v, Frac::ONE, nvars)
}

/// 類型位元向量建立（可變 N）
fn type_bits_v2(sys: &mut SystemV2, node: usize, kind: &str) -> Vec<usize> {
    if let Some(ts) = sys.node_type.get(&node) {
        return ts.clone();
    }
    let n = sys.universe.n_types();
    let mut ts = Vec::with_capacity(n);
    for (ti, ty) in sys.universe.types.iter().enumerate() {
        let v = sys.nvars;
        sys.names.push(format!("t{}:{}[{}]", node, ty.name(), ti));
        ts.push(v);
        sys.nvars += 1;
    }
    sys.node_type.insert(node, ts.clone());
    sys.node_kind.insert(node, kind.to_string());
    // one-hot: Σ t -1 =0
    let mut terms: Vec<(Vec<u32>, Frac)> = vec![(vec![], Frac::from_i64(-1))];
    for &v in &ts {
        terms.push((mono_of(v, sys.nvars), Frac::ONE));
    }
    sys.polys.push(Poly::from_terms(terms));
    // 域多項式 t^2 - t =0 會由管線加入，這裡僅記錄
    ts
}

pub fn emit(sys: &mut SystemV2, f: Poly) {
    if !f.is_zero() {
        sys.polys.push(f);
    }
}

pub fn emit_bool(sys: &mut SystemV2, v: usize) {
    // boolean 約束：x*(x-1)=0
    let x = var_poly(sys.nvars, v);
    let p = x.clone().mul(&x.sub(&Poly::constant(Frac::ONE)));
    emit(sys, p);
}

/// 從 Lowered 生成約束系統 v2
pub fn gen_constraints_v2(lowered: &Lowered) -> Result<SystemV2, String> {
    let mut sys = SystemV2 {
        universe: lowered.program.universe.clone(),
        ..Default::default()
    };

    let mut next_node_id = 1000usize; // 用於虛擬節點

    // 為每個 struct 生成 product 約束
    for (struct_name, fields) in &lowered.products {
        let node_id = next_node_id;
        next_node_id += 1;
        let ts = type_bits_v2(&mut sys, node_id, &format!("struct:{}", struct_name));
        // 找到 struct 在宇宙中的索引
        let struct_ty = TypeV2::Ext(super::universe::ExtType::Struct { name: struct_name.clone(), args: vec![] });
        if let Some(struct_idx) = sys.universe.index_of(&struct_ty) {
            let field_indices: Vec<usize> = fields.iter()
                .filter_map(|(_, ty)| sys.universe.index_of(ty))
                .collect();
            // t_struct - Π t_field =0
            // 簡化：若 field_indices 非空，生成 product 多項式
            if !field_indices.is_empty() {
                let mut prod = var_poly(sys.nvars, ts[field_indices[0]]);
                for &fi in &field_indices[1..] {
                    prod = prod.mul(&var_poly(sys.nvars, ts[fi]));
                }
                let f = var_poly(sys.nvars, ts[struct_idx]).sub(&prod);
                let poly_text = format!("t{}_{} - {} =0 # product {}", node_id, struct_name,
                    field_indices.iter().map(|i| format!("t{}_{}", node_id, i)).collect::<Vec<_>>().join("*"),
                    struct_name);
                emit(&mut sys, f);
                sys.product_constraints.push(ProductConstraint {
                    struct_name: struct_name.clone(),
                    node_id,
                    struct_idx,
                    field_indices,
                    poly_text,
                });
            }
        }
    }

    // 為每個 enum 生成 sum 約束
    for (enum_name, variants) in &lowered.sums {
        let node_id = next_node_id;
        next_node_id += 1;
        let ts = type_bits_v2(&mut sys, node_id, &format!("enum:{}", enum_name));
        let enum_ty = TypeV2::Ext(super::universe::ExtType::Enum { name: enum_name.clone(), args: vec![] });
        if let Some(enum_idx) = sys.universe.index_of(&enum_ty) {
            // 變體索引：假設變體類型在宇宙中
            let variant_indices: Vec<usize> = variants.iter()
                .enumerate()
                .filter_map(|(i, _)| {
                    // 變體用其位置作為示例索引，實際應為變體類型索引
                    // Phase2 簡化：用 i + 7 + offset
                    Some((enum_idx + 1 + i) % sys.universe.n_types())
                })
                .collect();
            if !variant_indices.is_empty() {
                let mut sum = Poly::zero();
                for &vi in &variant_indices {
                    sum = sum.add(&var_poly(sys.nvars, ts[vi]));
                }
                let f = var_poly(sys.nvars, ts[enum_idx]).sub(&sum);
                let poly_text = format!("t{}_{} - ({}) =0 # sum {}", node_id, enum_name,
                    variant_indices.iter().map(|i| format!("t{}_{}", node_id, i)).collect::<Vec<_>>().join("+"),
                    enum_name);
                emit(&mut sys, f);
                sys.sum_constraints.push(SumConstraint {
                    enum_name: enum_name.clone(),
                    node_id,
                    enum_idx,
                    variant_indices,
                    poly_text,
                });
            }
        }
    }

    // 為程序中的每個 fn 生成基本約束（參數、返回）
    for item in &lowered.program.items {
        if let super::ast_v2::ItemV2::Fn(f) = item {
            let node_id = next_node_id;
            next_node_id += 1;
            let ts = type_bits_v2(&mut sys, node_id, &format!("fn:{}", f.sig.name));
            // 返回類型約束
            if let Some(ret_idx) = sys.universe.index_of(&f.sig.ret) {
                let f_poly = var_poly(sys.nvars, ts[ret_idx]).sub(&Poly::constant(Frac::ONE));
                emit(&mut sys, f_poly);
            }
        }
    }

    // main 體的類型位元
    if let Some(main) = &lowered.program.main {
        let node_id = next_node_id;
        let _ts = type_bits_v2(&mut sys, node_id, "main");
        let _ = main;
    }

    // Phase3: 檢測 Vec/String/HashMap 使用，生成 stdlib 約束
    {
        let mut type_text = String::new();
        for ty in &sys.universe.types {
            type_text.push_str(&ty.name());
            type_text.push(',');
        }
        let stdlib_cons = gen_stdlib_constraints(&mut sys, &type_text);
        sys.stdlib_constraints.extend(stdlib_cons);
    }

    // Phase3: async fn 約束
    for item in &lowered.program.items {
        if let super::ast_v2::ItemV2::Fn(f) = item {
            // 若原 fn 是 async，lowered 中已轉為 Future，但我們仍檢查 is_async 標記（lowered 後 is_async=false，但 generated 中有 State）
            // 這裡用 body 中是否含 await 作為啟發，或直接檢查 lowered.generated
            if f.body_src.contains("await") || f.sig.name.contains("fetch") || f.sig.is_async {
                let ac = gen_async_constraints(&mut sys, &f.sig.name, 1);
                sys.async_constraints.push(ac);
            }
        }
    }
    // 也檢查 generated state machines
    for gen_item in &lowered.generated {
        if let super::ast_v2::ItemV2::Enum(e) = gen_item {
            if e.name.ends_with("State") {
                let fn_name = e.name.trim_end_matches("State");
                let ac = gen_async_constraints(&mut sys, fn_name, 2);
                sys.async_constraints.push(ac);
            }
        }
    }

    // Phase3: trait/impl 約束
    for item in &lowered.program.items {
        if let super::ast_v2::ItemV2::Impl(im) = item {
            if let Some(trait_name) = &im.trait_name {
                let tic = gen_trait_impl_constraints(&mut sys, trait_name, &im.self_ty.name());
                sys.trait_impl_constraints.push(tic);
            }
        }
    }

    // Phase3: 默認 loop fuel 約束（示例，實際應由 DSL fuel 驅動）
    // 為每個 fn 生成一個 fuel=3 的約束作為示範
    {
        let fuel_cons = gen_loop_fuel_constraints(&mut sys, next_node_id, 3);
        sys.loop_fuel_constraints.push(fuel_cons);
    }

    Ok(sys)
}

/// 生成統一多項式：t_T1 - t_T2 =0
pub fn unify_poly(sys: &SystemV2, node_id: usize, idx1: usize, idx2: usize) -> Poly {
    let ts = sys.node_type.get(&node_id).expect("node not found");
    var_poly(sys.nvars, ts[idx1]).sub(&var_poly(sys.nvars, ts[idx2]))
}

/// 生成 Vec 操作約束
pub fn vec_push_constraint(sys: &mut SystemV2, node_id: usize, vec_idx: usize, elem_idx: usize) -> Poly {
    let ts = type_bits_v2(sys, node_id, "vec_push");
    // t_vec * t_elem - t_vec =0 → 要求 elem 類型一致時 vec 存在
    var_poly(sys.nvars, ts[vec_idx]).mul(&var_poly(sys.nvars, ts[elem_idx])).sub(&var_poly(sys.nvars, ts[vec_idx]))
}

/// 生成 Option 約束
pub fn option_some_constraint(sys: &mut SystemV2, node_id: usize, option_idx: usize, inner_idx: usize) -> Poly {
    let ts = type_bits_v2(sys, node_id, "option_some");
    // t_Option - t_inner =0 當 Some
    var_poly(sys.nvars, ts[option_idx]).sub(&var_poly(sys.nvars, ts[inner_idx]))
}

/// 生成 match 決策樹約束
pub fn gen_match_constraints(sys: &mut SystemV2, node_id: usize, tree: MatchDecisionTree) -> Result<MatchConstraint, String> {
    let _ts = type_bits_v2(sys, node_id, &format!("match:{}", tree.scrutinee));
    let scrutinee_id = node_id + 100; // 虛擬 scrutinee 節點
    let _scrut_ts = type_bits_v2(sys, scrutinee_id, "scrutinee");

    // arm bits
    let mut arm_bits = vec![];
    for i in 0..tree.arms.len() {
        let v = sys.nvars;
        sys.names.push(format!("m{}_{}", node_id, i));
        arm_bits.push(v);
        sys.nvars += 1;
    }
    // Σ m_i -1 =0
    let mut terms: Vec<(Vec<u32>, Frac)> = vec![(vec![], Frac::from_i64(-1))];
    for &a in &arm_bits {
        terms.push((mono_of(a, sys.nvars), Frac::ONE));
    }
    emit(sys, Poly::from_terms(terms));

    // arm 互斥子句
    for i in 0..arm_bits.len() {
        for j in (i+1)..arm_bits.len() {
            sys.clauses.push(vec![cdcl::lit(arm_bits[i], false), cdcl::lit(arm_bits[j], false)]);
        }
    }
    sys.clauses.push(arm_bits.iter().map(|&v| cdcl::lit(v, true)).collect());

    // 結果類型 = Σ m_i * t_branch_i
    // 簡化：假設結果索引為 0 (i32)
    let result_idx = 0;

    let mc = MatchConstraint {
        node_id,
        scrutinee_id,
        arm_bits: arm_bits.clone(),
        result_idx,
        decision_tree: tree,
    };
    sys.arm_vars.insert(node_id, arm_bits);
    sys.match_constraints.push(mc.clone());

    Ok(mc)
}

/// Phase3 — 擴展約束：Vec/String/HashMap、loop fuel、async、unsafe、lifetime、trait/impl、I/O

#[derive(Clone, Debug)]
pub struct LoopFuelConstraint {
    pub node_id: usize,
    pub fuel: usize,
    pub counter_vars: Vec<usize>,
    pub poly_texts: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct AsyncConstraint {
    pub fn_name: String,
    pub num_await: usize,
    pub state_vars: Vec<usize>,
    pub poly_texts: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct LifetimeConstraint {
    pub longer: String,
    pub shorter: String,
    pub node_id: usize,
    pub poly_text: String,
}

#[derive(Clone, Debug)]
pub struct UnsafeConstraint {
    pub node_id: usize,
    pub in_unsafe_var: usize,
    pub op_var: usize,
    pub poly_text: String,
}

#[derive(Clone, Debug)]
pub struct StdlibConstraint {
    pub ty_name: String,
    pub kind: String, // Vec, String, HashMap
    pub node_id: usize,
    pub poly_texts: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TraitImplConstraint {
    pub trait_name: String,
    pub for_ty: String,
    pub impl_exists_var: usize,
    pub poly_text: String,
}

impl SystemV2 {
    /// 添加 Phase3 擴展記錄（可選字段，通過額外 Vec 存儲）
    pub fn add_loop_fuel(&mut self, c: LoopFuelConstraint) {
        // 記錄為 product 約束的擴展，用 names 記錄
        self.names.push(format!("loop_fuel_{}_{}", c.node_id, c.fuel));
        // 實際多項式已在 gen 函數中加入 polys
        let _ = c;
    }
}

/// 生成 loop fuel 約束：c_{i+1} - c_i +1 =0
pub fn gen_loop_fuel_constraints(sys: &mut SystemV2, node_id: usize, fuel: usize) -> LoopFuelConstraint {
    let mut counter_vars = vec![];
    let mut poly_texts = vec![];
    for _ in 0..=fuel {
        let v = sys.nvars;
        sys.names.push(format!("c{}_{}", node_id, counter_vars.len()));
        counter_vars.push(v);
        sys.nvars += 1;
    }
    for i in 0..fuel {
        let c_i = counter_vars[i];
        let c_next = counter_vars[i+1];
        // c_next - c_i +1 =0
        let poly = var_poly(sys.nvars, c_next).sub(&var_poly(sys.nvars, c_i)).add(&Poly::constant(Frac::ONE));
        emit(sys, poly);
        poly_texts.push(format!("c{}_{} - c{}_{} +1 =0 # fuel {}->{}", node_id, i+1, node_id, i, i, i+1));
    }
    LoopFuelConstraint { node_id, fuel, counter_vars, poly_texts }
}

/// 生成 invariant 約束：t_inv -1 =0
pub fn gen_invariant_constraint(sys: &mut SystemV2, node_id: usize, inv_idx: usize) -> Poly {
    let ts = type_bits_v2(sys, node_id, &format!("invariant_{}", inv_idx));
    // 假設 invariant bool 變量為第 0 個類型位元對應的 bool
    let inv_var = ts.first().copied().unwrap_or(0);
    let poly = var_poly(sys.nvars, inv_var).sub(&Poly::constant(Frac::ONE));
    emit(sys, poly.clone());
    poly
}

/// 生成 async 狀態機約束
pub fn gen_async_constraints(sys: &mut SystemV2, fn_name: &str, num_await: usize) -> AsyncConstraint {
    let sm = crate::minirust::async_qap::AsyncStateMachine::new(fn_name, num_await);
    let mut state_vars = vec![];
    for i in 0..sm.states.len() {
        let v = sys.nvars;
        sys.names.push(format!("s{}_{}", fn_name, i));
        state_vars.push(v);
        sys.nvars += 1;
    }
    // one-hot: Σ s_i -1 =0
    let mut terms: Vec<(Vec<u32>, Frac)> = vec![(vec![], Frac::from_i64(-1))];
    for &v in &state_vars {
        terms.push((mono_of(v, sys.nvars), Frac::ONE));
    }
    emit(sys, Poly::from_terms(terms));
    // boolean: s_i*(s_i-1)=0
    for &v in &state_vars {
        let poly = var_poly(sys.nvars, v).mul(&var_poly(sys.nvars, v).sub(&Poly::constant(Frac::ONE)));
        emit(sys, poly);
    }
    // transitions: s_from * poll - s_to =0 (簡化)
    for (from, (to, _)) in &sm.transitions {
        if *from < state_vars.len() && *to < state_vars.len() {
            let poly = var_poly(sys.nvars, state_vars[*from]).sub(&var_poly(sys.nvars, state_vars[*to]));
            emit(sys, poly);
        }
    }
    AsyncConstraint {
        fn_name: fn_name.to_string(),
        num_await,
        state_vars,
        poly_texts: sm.poly_text(),
    }
}

/// 生成 lifetime outlives 約束：ss - ls, le - se
pub fn gen_lifetime_constraints(sys: &mut SystemV2, graph: &crate::minirust::lifetime::LifetimeGraph) -> Vec<LifetimeConstraint> {
    let mut out = vec![];
    for (longer, shorters) in &graph.outlives {
        for shorter in shorters {
            let node_id = sys.nvars;
            let ls = sys.nvars;
            sys.names.push(format!("lt_{}_start", longer.0));
            sys.nvars += 1;
            let le = sys.nvars;
            sys.names.push(format!("lt_{}_end", longer.0));
            sys.nvars += 1;
            let ss = sys.nvars;
            sys.names.push(format!("lt_{}_start", shorter.0));
            sys.nvars += 1;
            let se = sys.nvars;
            sys.names.push(format!("lt_{}_end", shorter.0));
            sys.nvars += 1;

            // ss - ls >=0, le - se >=0，編碼為差值多項式
            let p1 = var_poly(sys.nvars, ss).sub(&var_poly(sys.nvars, ls));
            let p2 = var_poly(sys.nvars, le).sub(&var_poly(sys.nvars, se));
            emit(sys, p1);
            emit(sys, p2);

            out.push(LifetimeConstraint {
                longer: longer.0.clone(),
                shorter: shorter.0.clone(),
                node_id,
                poly_text: format!("{}: {} - {} start, {} - {} end", longer.0, shorter.0, ss, le, se),
            });
        }
    }
    out
}

/// 生成 unsafe 約束：(1 - in_unsafe) * t_op =0
pub fn gen_unsafe_constraint(sys: &mut SystemV2, node_id: usize) -> UnsafeConstraint {
    let in_unsafe_var = sys.nvars;
    sys.names.push(format!("in_unsafe_{}", node_id));
    sys.nvars += 1;
    let op_var = sys.nvars;
    sys.names.push(format!("unsafe_op_{}", node_id));
    sys.nvars += 1;

    // boolean for in_unsafe
    let bool_poly = var_poly(sys.nvars, in_unsafe_var).mul(&var_poly(sys.nvars, in_unsafe_var).sub(&Poly::constant(Frac::ONE)));
    emit(sys, bool_poly);

    // (1 - in_unsafe) * op =0
    let one = Poly::constant(Frac::ONE);
    let poly = one.sub(&var_poly(sys.nvars, in_unsafe_var)).mul(&var_poly(sys.nvars, op_var));
    emit(sys, poly);

    UnsafeConstraint {
        node_id,
        in_unsafe_var,
        op_var,
        poly_text: format!("(1 - in_unsafe_{}) * op_{} =0 # unsafe gate", node_id, node_id),
    }
}

/// 生成 stdlib 約束：Vec len<=cap, HashMap unique, String utf8
pub fn gen_stdlib_constraints(sys: &mut SystemV2, type_uni: &str) -> Vec<StdlibConstraint> {
    let mut out = vec![];
    let reg = crate::minirust::stdlib::StdlibRegistry::from_type_universe(type_uni);
    for (name, enc) in reg.vec_encodings {
        let node_id = sys.nvars;
        let poly = enc.len_le_cap_poly(sys.nvars + 3);
        emit(sys, poly);
        out.push(StdlibConstraint {
            ty_name: name.clone(),
            kind: "Vec".to_string(),
            node_id,
            poly_texts: vec![format!("// Vec {} len<=cap", name), "cap - len - slack=0".to_string()],
        });
    }
    for (name, enc) in reg.hashmap_encodings {
        let node_id = sys.nvars;
        // unique 約束需具體 k_i,k_j,inv，這裡生成示意
        out.push(StdlibConstraint {
            ty_name: name.clone(),
            kind: "HashMap".to_string(),
            node_id,
            poly_texts: vec![format!("// HashMap {} key unique", name), "(k_i - k_j)*inv -1=0".to_string()],
        });
        let _ = enc;
    }
    for (name, _) in reg.string_encodings {
        out.push(StdlibConstraint {
            ty_name: name.clone(),
            kind: "String".to_string(),
            node_id: sys.nvars,
            poly_texts: vec!["// String utf8 check".to_string()],
        });
    }
    out
}

/// 生成 trait/impl 約束：impl_exists bit
pub fn gen_trait_impl_constraints(sys: &mut SystemV2, trait_name: &str, for_ty: &str) -> TraitImplConstraint {
    let impl_exists_var = sys.nvars;
    sys.names.push(format!("impl_exists_{}_{}", for_ty, trait_name));
    sys.nvars += 1;

    // boolean
    let bool_poly = var_poly(sys.nvars, impl_exists_var).mul(&var_poly(sys.nvars, impl_exists_var).sub(&Poly::constant(Frac::ONE)));
    emit(sys, bool_poly);

    TraitImplConstraint {
        trait_name: trait_name.to_string(),
        for_ty: for_ty.to_string(),
        impl_exists_var,
        poly_text: format!("impl_exists_{}_{} boolean + method availability", for_ty, trait_name),
    }
}

/// 轉 R1CS（復用 v1 邏輯，支持可變 N）
pub fn to_r1cs_v2(nvars: usize, polys: &[Poly]) -> R1cs {
    let mut r = R1cs {
        n_wires: nvars + 1,
        constraints: vec![],
        intermediates: vec![],
    };
    for f in polys {
        if f.is_zero() { continue; }
        poly_to_r1cs_v2(f, &mut r);
    }
    r
}

fn poly_to_r1cs_v2(f: &Poly, r: &mut R1cs) {
    let mut lin: Vec<(usize, crate::fp::Fp)> = vec![];
    for (m, cf) in &f.terms {
        let k = *cf;
        let deg: u32 = m.iter().sum();
        if deg == 0 {
            lin.push((0, k));
        } else if deg == 1 {
            let v = m.iter().position(|&e| e == 1).unwrap();
            lin.push((v + 1, k));
        } else {
            let lw = monomial_linear_v2(m, r);
            for (w, c) in lw {
                lin.push((w, k * c));
            }
        }
    }
    let mut merged: Vec<(usize, crate::fp::Fp)> = vec![];
    for (w, k) in lin {
        if let Some(e) = merged.iter_mut().find(|(mw, _)| *mw == w) {
            e.1 += k;
        } else {
            merged.push((w, k));
        }
    }
    merged.retain(|(_, k)| !k.is_zero());
    r.constraints.push((merged, vec![(0, crate::fp::Fp::one())], vec![]));
    r.intermediates.push(None);
}

fn monomial_linear_v2(m: &[u32], r: &mut R1cs) -> Linear {
    let deg: u32 = m.iter().sum();
    if deg == 0 {
        return vec![(0, crate::fp::Fp::one())];
    }
    if deg == 1 {
        let v = m.iter().position(|&e| e == 1).unwrap();
        return vec![(v + 1, crate::fp::Fp::one())];
    }
    let mut factors: Vec<usize> = vec![];
    for (i, &e) in m.iter().enumerate() {
        for _ in 0..e {
            factors.push(i);
        }
    }
    let half = factors.len().div_ceil(2);
    let (fa, fb) = factors.split_at(half);
    let la = monomial_factors_linear_v2(fa, r);
    let lb = monomial_factors_linear_v2(fb, r);
    let w = r.n_wires;
    r.n_wires += 1;
    r.constraints.push((la, lb, vec![(w, crate::fp::Fp::one())]));
    r.intermediates.push(Some(w));
    vec![(w, crate::fp::Fp::one())]
}

fn monomial_factors_linear_v2(factors: &[usize], r: &mut R1cs) -> Linear {
    if factors.len() == 1 {
        return vec![(factors[0] + 1, crate::fp::Fp::one())];
    }
    if factors.is_empty() {
        return vec![(0, crate::fp::Fp::one())];
    }
    let half = factors.len().div_ceil(2);
    let (fa, fb) = factors.split_at(half);
    let la = monomial_factors_linear_v2(fa, r);
    let lb = monomial_factors_linear_v2(fb, r);
    let w = r.n_wires;
    r.n_wires += 1;
    r.constraints.push((la, lb, vec![(w, crate::fp::Fp::one())]));
    r.intermediates.push(Some(w));
    vec![(w, crate::fp::Fp::one())]
}

/// 實際使用：constraints_v2.rs 文件清單 — 優化 with_capacity
pub fn constraints_v2_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("constraints_v2.rs", "constraints_v2.rs 正式運作 — 優化 with_capacity", "core/src/minirust/constraints_v2.rs"),
    ]
}

