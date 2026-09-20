// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! PolyIR 值軌跡 → 𝔽_p 多項式系統編碼 + 代數判定（C2 主體）。
//!
//! 管線（藍圖 §3 L3）：`charon_llbc` parse → [`polyir::lower_fun`] 語義 lowering
//! → 本模組編碼 → 求解 → 認證。
//!
//! # 編碼（沿用 mir_lower 之值域消失多項式方案）
//! * 每個 Slot（local 或 checked-tuple field）一個 𝔽_p 變量；
//! * 值域由常數流 + 參數域（調用方提供）抽象推導；
//! * 域約束 = 消失多項式 ∏_{v∈D}(x−v)；語句 = 算術恆等式；
//! * `AssertFlagZero` → flag = 0 等式（非溢出執行之假設，如實標註於判定）。
//!
//! # 判定
//! * 值域笛卡爾積 ≤ 10⁶ 組合 → 逐組代入求值（經 L0′ 保真）→ 全零 = SAT；
//! * 見證再經獨立認證（域成員 + 全多項式直接求值）方稱 `Certified`；
//! * 任何降級（Switch/Loop/Call/L0′ 超限/域過大）都如實回報 `Unknown{reason}`。

use crate::charon_llbc::{BodyKind, FunDeclRef, LlbcRoot, Value};
use crate::fp::P;
use crate::frac::Frac;
use crate::minirust::mir_lower::{self, MirSystem};
use crate::poly::Poly;
use crate::polyir::{
    concretize_cap, lower_fun_in, self_recursion_step, ConcErr, ParamVal, Part,
    PirBinOp, PirBody, PirOperand, PirPath, PirStmtKind, PirVerdict, Slot, CALL_DEPTH_CAP,
    REC_DEPTH_HARD_CAP,
};
use crate::vanishing::{l0_prime_params_of, vanishing_poly, L0PrimeParams};
use std::collections::BTreeMap;

/// 值域組合總量上限（笛卡爾積）。
const PRODUCT_CAP: u64 = 1_000_000;
/// 單一值域集合大小上限。
const DOMAIN_CAP: usize = 64;
/// C4 逐組合見證模式之組合數上限（每組合一次模擬 + 一次系統求解）。
const CONTEXT_CAP: usize = 4096;

/// 編碼產物：多項式系統 + 槽→變量映射 + 返回變量。
#[derive(Debug, Clone)]
pub struct Encoded {
    pub sys: MirSystem,
    pub slot_var: BTreeMap<Slot, usize>,
    pub ret_var: Option<usize>,
}

/// 代數判定（C2 口徑）。
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// 全部參數組合（見證模式）或系統見證（直線模式）通過獨立認證。
    /// `ret` 為返回槽之（有符）值；多組合且 ret 不唯一時 = None。
    Certified {
        ret: Option<i64>,
        n_vars: usize,
        n_eqs: usize,
        /// 溢出旗標被斷言為零（結果以「非溢出執行」為前提）。
        overflow_asserted: bool,
        /// 認證組合數（直線模式 = 1）。
        paths: usize,
        /// 因 checked 溢出而排除之組合數（真實執行會 panic，唔喺認證範圍）。
        excluded: usize,
    },
    /// 語義降級（附精確原因）——非錯誤，是能力邊界之如實申報。
    Unknown { reason: String },
    /// 約束系統在給定參數域上無解。
    Unsat,
}

impl Decision {
    pub fn is_certified(&self) -> bool {
        matches!(self, Decision::Certified { .. })
    }
    /// Unknown 原因字串（測試/報表用）。
    pub fn reason(&self) -> Option<&str> {
        match self {
            Decision::Unknown { reason } => Some(reason),
            _ => None,
        }
    }
}

fn var_of(s: &Slot, slots: &mut BTreeMap<Slot, usize>, doms: &mut Vec<Vec<i64>>) -> usize {
    if let Some(&id) = slots.get(s) {
        return id;
    }
    let id = doms.len();
    doms.push(Vec::new());
    slots.insert(s.clone(), id);
    id
}

/// 槽域讀取；未 intern 之參數 field 槽（C3：ADT 參數逐欄位獨立變量）先播種該
/// 參數域。非參數槽讀取先於定義 → 照舊報錯（use-before-def，唔容許猜）。
fn ensure_slot_dom(
    s: &Slot,
    slots: &mut BTreeMap<Slot, usize>,
    domains: &mut Vec<Vec<i64>>,
    arg_count: usize,
    arg_domains: &[Vec<i64>],
) -> Result<Vec<i64>, String> {
    let id = match slots.get(s) {
        Some(&id) => id,
        None => {
            let id = var_of(s, slots, domains);
            if matches!(s.part, Part::Field(_)) && s.local >= 1 && s.local <= arg_count {
                domains[id] = arg_domains[s.local - 1].clone();
            }
            id
        }
    };
    domains
        .get(id)
        .cloned()
        .ok_or_else(|| format!("槽 {:?} 域索引越界", s))
}

fn combine(a: &[i64], b: &[i64], f: impl Fn(i64, i64) -> i64) -> Result<Vec<i64>, String> {
    let mut out: Vec<i64> = Vec::new();
    for &x in a {
        for &y in b {
            out.push(f(x, y));
        }
    }
    out.sort_unstable();
    out.dedup();
    if out.len() > DOMAIN_CAP {
        return Err(format!("值域集合爆上限（{}/{}）", out.len(), DOMAIN_CAP));
    }
    Ok(out)
}

fn operand_dom(
    op: &PirOperand,
    slots: &mut BTreeMap<Slot, usize>,
    domains: &mut Vec<Vec<i64>>,
    arg_count: usize,
    arg_domains: &[Vec<i64>],
) -> Result<Vec<i64>, String> {
    match op {
        PirOperand::Slot(s) => ensure_slot_dom(s, slots, domains, arg_count, arg_domains),
        PirOperand::Const(v) => Ok(vec![*v]),
    }
}

fn operand_poly(op: &PirOperand, slot_var: &BTreeMap<Slot, usize>, width: usize) -> Poly {
    match op {
        PirOperand::Slot(s) => Poly::var(slot_var[s], Frac::ONE, width),
        PirOperand::Const(v) => Poly::constant(Frac::from_i64(*v)),
    }
}

fn binop_apply(op: PirBinOp, x: i64, y: i64) -> i64 {
    match op {
        PirBinOp::Add => x.wrapping_add(y),
        PirBinOp::Sub => x.wrapping_sub(y),
        PirBinOp::Mul => x.wrapping_mul(y),
        // 比較經見證模式恆等化，唔會行到此（直線模式早已降級）
        _ => unreachable!("比較運算唔經 binop_apply"),
    }
}

/// 𝔽_p 表示 → 最小絕對值有符代表 → i64（L0′ 保真下必落 i64 範圍）。
fn fp_to_i64(v: &Frac) -> i64 {
    let raw = v.0 as i128;
    let signed = if v.0 > P / 2 { raw - P as i128 } else { raw };
    signed as i64
}

/// 見證值讀取：見證模式必需；無 witness／槽無值 → 誠實降級。
fn require_witness(
    witness: Option<&BTreeMap<Slot, i64>>,
    s: &Slot,
    name: &str,
) -> Result<i64, String> {
    witness
        .and_then(|w| w.get(s).copied())
        .ok_or_else(|| format!("{name}: 槽 {s:?} 無見證值（須經 concretize 模擬）"))
}

/// PirBody → 多項式系統（值域 + 恆等式 + L0′ 門檻）。
/// 只接受單路徑 body（多路徑須先經 [`concretize`] 攤平）。
/// `witness`（C4 見證模式）：逐組合模擬之槽值表；比較／Discriminant 語句
/// 恆等化為 dst = 模擬值（比較語義由模擬承擔，編碼如實記錄——認證鏈：
/// 域成員 + 全多項式直接求值）。
pub fn encode_body(
    b: &PirBody,
    arg_domains: &[Vec<i64>],
    witness: Option<&BTreeMap<Slot, i64>>,
) -> Result<Encoded, String> {
    if b.paths.len() != 1 {
        return Err(format!(
            "{}: encode 只接受單路徑 body（得到 {} 條）",
            b.fun_name,
            b.paths.len()
        ));
    }
    if arg_domains.len() != b.arg_count {
        return Err(format!(
            "參數域數量 {} ≠ arg_count {}",
            arg_domains.len(),
            b.arg_count
        ));
    }
    for (i, d) in arg_domains.iter().enumerate() {
        if d.is_empty() && witness.is_none() {
            return Err(format!("參數 {} 域為空", i));
        }
        if d.len() > DOMAIN_CAP {
            return Err(format!("參數 {} 域大於上限 {}", i, DOMAIN_CAP));
        }
    }

    let mut slot_var: BTreeMap<Slot, usize> = BTreeMap::new();
    let mut domains: Vec<Vec<i64>> = Vec::new();

    // 參數槽（LLBC：local 0 = return place，參數由 local 1 起）
    for (i, d) in arg_domains.iter().enumerate() {
        let s = Slot {
            local: i + 1,
            part: Part::Whole,
        };
        let id = var_of(&s, &mut slot_var, &mut domains);
        domains[id] = d.clone();
    }

    let stmts = &b.paths[0].stmts;
    let mut eqs: Vec<Poly> = Vec::new();
    let mut ret_var: Option<usize> = None;
    // Poly::var 需要上界；逐語句精確計數產生槽數（C3 修：checked=3、
    // Aggregate=k+1——舊 stmts×2 上界喺多 checked／大 Aggregate 時會爆）
    let slot_bound: usize = stmts
        .iter()
        .map(|st| match &st.kind {
            PirStmtKind::Aggregate { fields, .. } => fields.len() + 1,
            PirStmtKind::BinOp { checked: true, .. } => 3,
            _ => 1,
        })
        .sum();
    let width = b.arg_count + slot_bound + 8;

    for st in stmts {
        match &st.kind {
            PirStmtKind::Const { val } => {
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = vec![*val];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::constant(Frac::from_i64(*val))));
                if st.dst.local == 0 && st.dst.part == Part::Whole {
                    ret_var = Some(d);
                }
            }
            PirStmtKind::Copy { src } => {
                let sd =
                    ensure_slot_dom(src, &mut slot_var, &mut domains, b.arg_count, arg_domains)?;
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = sd;
                let s = slot_var[src];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::var(s, Frac::ONE, width)));
                if st.dst.local == 0 && st.dst.part == Part::Whole {
                    ret_var = Some(d);
                }
            }
            PirStmtKind::BinOp {
                op,
                a,
                b: bop,
                checked,
            } if op.is_comparison() => {
                // C4：比較。直線（無 witness）模式降級；見證模式恆等化
                // dst = 模擬值（比較語義由模擬器 i64 口徑承擔，編碼如實記錄）
                let w = require_witness(witness, &st.dst, &b.fun_name)?;
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = vec![w];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::constant(Frac::from_i64(w))));
                if st.dst.local == 0 && st.dst.part == Part::Whole {
                    ret_var = Some(d);
                }
                let _ = (a, bop);
            }
            PirStmtKind::Discriminant { of } => {
                // C4：動態判別值（enum 參數）。同上，見證模式恆等化。
                let _ = of;
                let w = require_witness(witness, &st.dst, &b.fun_name)?;
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = vec![w];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::constant(Frac::from_i64(w))));
                if st.dst.local == 0 && st.dst.part == Part::Whole {
                    ret_var = Some(d);
                }
            }
            PirStmtKind::Call { callee, args } => {
                // C5：調用經見證模式恆等化（dst = 模擬 ret；語義由 concretize
                // 遞迴模擬 callee 承擔）。直線無 witness → 降級。
                for a in args {
                    let _ = operand_dom(a, &mut slot_var, &mut domains, b.arg_count, arg_domains)?;
                }
                let _ = callee;
                let w = require_witness(witness, &st.dst, &b.fun_name)?;
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = vec![w];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::constant(Frac::from_i64(w))));
                if st.dst.local == 0 && st.dst.part == Part::Whole {
                    ret_var = Some(d);
                }
            }
            PirStmtKind::Loop { .. } => {
                // C6：見證模式——系統由 witness 早 return 分支統一恆等化
                // （直線 eq 在 loop 多重賦值下唔成立）。槽無見證值（如
                // zero-iteration 的 body 臨時槽）係死槽，早 return 分支
                // 自動域清空，唔使喺此降級。
            }
            PirStmtKind::BinOp {
                op,
                a,
                b: bop,
                checked,
            } => {
                let da = operand_dom(a, &mut slot_var, &mut domains, b.arg_count, arg_domains)?;
                let db = operand_dom(bop, &mut slot_var, &mut domains, b.arg_count, arg_domains)?;
                let dom = combine(&da, &db, |x, y| binop_apply(*op, x, y))?;
                let (pa, pb) = (
                    operand_poly(a, &slot_var, width),
                    operand_poly(bop, &slot_var, width),
                );
                let rhs = match op {
                    PirBinOp::Add => pa.add(&pb),
                    PirBinOp::Sub => pa.sub(&pb),
                    PirBinOp::Mul => pa.mul(&pb),
                    _ => unreachable!("比較運算唔經算術 rhs"),
                };
                if *checked {
                    // tuple 結果：field0 = 值、field1 = 溢出旗標（域 {0,1}，由 Assert 收緊）
                    let f0 = var_of(&Slot::field(st.dst.local, 0), &mut slot_var, &mut domains);
                    let f1 = var_of(&Slot::field(st.dst.local, 1), &mut slot_var, &mut domains);
                    // whole 槽登記（空域，防 direct use；certify/solve 對空域跳過）
                    let _w = var_of(&st.dst, &mut slot_var, &mut domains);
                    domains[f0] = dom;
                    domains[f1] = vec![0, 1];
                    eqs.push(Poly::var(f0, Frac::ONE, width).sub(&rhs));
                } else {
                    let d = var_of(&st.dst, &mut slot_var, &mut domains);
                    domains[d] = dom;
                    eqs.push(Poly::var(d, Frac::ONE, width).sub(&rhs));
                    if st.dst.local == 0 && st.dst.part == Part::Whole {
                        ret_var = Some(d);
                    }
                }
            }
            PirStmtKind::Aggregate { fields, .. } => {
                // C3：ADT 建構。dst 整槽 = 不透明 ADT 值 → 登記空域
                // （solve/certify 對空域跳過）；欄位逐一寫入 field 槽，
                // 域 = 運算元域、恆等式 = dst_f = src（純資料流，零新變量）。
                let w = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[w] = Vec::new();
                for (k, f) in fields.iter().enumerate() {
                    let fs = Slot::field(st.dst.local, k as u8);
                    let d = var_of(&fs, &mut slot_var, &mut domains);
                    match f {
                        PirOperand::Slot(src) => {
                            let sd = ensure_slot_dom(
                                src,
                                &mut slot_var,
                                &mut domains,
                                b.arg_count,
                                arg_domains,
                            )?;
                            domains[d] = sd;
                            let sv = slot_var[src];
                            eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::var(
                                sv,
                                Frac::ONE,
                                width,
                            )));
                        }
                        PirOperand::Const(v) => {
                            domains[d] = vec![*v];
                            eqs.push(
                                Poly::var(d, Frac::ONE, width)
                                    .sub(&Poly::constant(Frac::from_i64(*v))),
                            );
                        }
                    }
                }
            }
            PirStmtKind::AssertFlagZero { flag } => {
                let id = var_of(flag, &mut slot_var, &mut domains);
                eqs.push(Poly::var(id, Frac::ONE, width));
            }
        }
    }

    // C6 見證模式早 return：系統 = 每個有見證值槽 {x = v}（域單點化）。
    // 直線 eq 喺多重賦值（loop carried）下唔成立——統一由見證恆等式承擔，
    // 認證鏈：域成員 + 全多項式直接求值（單點線性系統必保真）。
    if let Some(w) = witness {
        let slots: Vec<(Slot, usize)> = slot_var.iter().map(|(s, &i)| (s.clone(), i)).collect();
        let mut polys: Vec<Poly> = Vec::new();
        let mut v_max: u64 = 0;
        for (s, id) in slots {
            match w.get(&s) {
                Some(&v) => {
                    domains[id] = vec![v];
                    v_max = v_max.max(v.unsigned_abs());
                    polys.push(
                        Poly::var(id, Frac::ONE, width).sub(&Poly::constant(Frac::from_i64(v))),
                    );
                }
                None => domains[id] = Vec::new(),
            }
        }
        let l0 = L0PrimeParams {
            n_terms: 1,
            max_abs_coeff: 1,
            max_abs_value: v_max,
            degree: 1,
        };
        let nvars = domains.len();
        return Ok(Encoded {
            sys: MirSystem {
                body: b.fun_name.clone(),
                nvars,
                polys,
                domains,
                l0,
            },
            slot_var,
            ret_var,
        });
    }

    // 值域約束（消失多項式）——所有已定義域之槽
    let mut polys = eqs.clone();
    for (id, dom) in domains.iter().enumerate() {
        if dom.is_empty() {
            continue;
        }
        polys.push(vanishing_poly(id, dom, width));
    }

    // L0′ 門檻（全系統保守界：最大項數 × 最大係數 × V^最大次數）
    let v_max = domains
        .iter()
        .flatten()
        .map(|v| v.unsigned_abs())
        .max()
        .unwrap_or(0);
    let mut l0 = L0PrimeParams {
        n_terms: 0,
        max_abs_coeff: 0,
        max_abs_value: v_max,
        degree: 0,
    };
    for p in &polys {
        let lp = l0_prime_params_of(p, v_max.max(1), u32::MAX);
        l0.n_terms = l0.n_terms.max(lp.n_terms);
        l0.max_abs_coeff = l0.max_abs_coeff.max(lp.max_abs_coeff);
        l0.degree = l0.degree.max(lp.degree);
    }
    l0.max_abs_value = v_max;
    if !l0.faithful() {
        return Err(format!(
            "L0′ 不保真：|f(σ)| 界 {} ≥ p {}（值域過大×次數過高）",
            l0.eval_bound().unwrap_or(u128::MAX),
            P
        ));
    }

    let nvars = domains.len();
    Ok(Encoded {
        sys: MirSystem {
            body: b.fun_name.clone(),
            nvars,
            polys,
            domains,
            l0,
        },
        slot_var,
        ret_var,
    })
}

/// 對單一函數做代數判定：lower → encode → solve → certify。
/// 唔帶 type_decls：enum Aggregate 判別值解析會如實降級。
pub fn decide_fun(fun: &FunDeclRef, arg_domains: &[Vec<i64>]) -> Decision {
    decide_fun_in(&Value::Null, &[], fun, arg_domains)
}

/// 同上，另帶 root 嘅 type_decls（C3：enum Aggregate 判別值解析）與
/// 全 crate funs（C5：Call 對位+遞迴模擬）。判定前一次過預 lower 全部
/// Structured fun（callee 模擬表；lower 失敗者唔入表——模擬時如實降級）。
pub fn decide_fun_in(
    types: &Value,
    funs: &[FunDeclRef],
    fun: &FunDeclRef,
    arg_domains: &[Vec<i64>],
) -> Decision {
    let callees = prelower(funs);
    match lower_fun_in(types, funs, fun) {
        PirVerdict::NoBody { kind } => Decision::Unknown {
            reason: format!("body 缺失（{:?}）", kind),
        },
        PirVerdict::Unknown { reason } => Decision::Unknown { reason },
        PirVerdict::ValueTrace(body) => decide_body(&body, arg_domains, &callees),
    }
}

/// M3（SPA1）：逐組合證書——每個輸入組合嘅見證解與系統摘要（機讀報告）。
/// 語義 = decide_paths 強制逐組合見證模式：每組合 concretize 模擬＋
/// 單點域編碼＋求解＋認證；溢出組合如實標記 excluded（永不入認證）。
#[derive(Debug, Clone)]
pub struct ComboCert {
    /// 參數顯示字串（Int 直書；Enum 顯示 disc/field）
    pub params: Vec<String>,
    pub ret: Option<i64>,
    pub overflow_excluded: bool,
    pub nvars: usize,
    pub npolys: usize,
    /// L0PrimeParams 摘要 (n_terms, max_abs_coeff, max_abs_value, degree)
    pub l0: (u64, u64, u64, u32),
    /// 見證解 σ（slot_var 順序；𝔽_p 代表元之整數視）
    pub sigma: Vec<i64>,
}

/// M3（SPA1）：單一函數之證書報告。
#[derive(Debug, Clone)]
pub struct FunCertReport {
    pub name: String,
    /// "Certified" | "Unknown" | "Unsat"
    pub verdict: &'static str,
    pub reason: Option<String>,
    pub n_combos: usize,
    pub n_certified: usize,
    pub n_excluded: usize,
    pub combos: Vec<ComboCert>,
    /// 模板對應 Lean 定理（C6 Loop 模板 → Polyrust.LoopInvariant.invSumF_spec；
    /// 其他形態屬後續 slice 對接，如實 None）
    pub lean_theorem: Option<&'static str>,
}

fn param_disp(p: &ParamVal) -> String {
    match p {
        ParamVal::Int(v) => v.to_string(),
        ParamVal::Enum { disc, field } => match field {
            Some(f) => format!("enum(disc={disc},field={f})"),
            None => format!("enum(disc={disc})"),
        },
    }
}

/// 對已 lowering 之 body 產生逐組合證書報告（M3/SPA1 核心）。
pub fn certify_fun_report(
    body: &PirBody,
    arg_domains: &[Vec<i64>],
    callees: &BTreeMap<String, PirBody>,
) -> FunCertReport {
    let has_loop = body.paths.iter().any(|p| {
        p.stmts
            .iter()
            .any(|s| matches!(s.kind, PirStmtKind::Loop { .. }))
    });
    let lean_theorem = if has_loop {
        Some("Polyrust.LoopInvariant.invSumF_spec")
    } else if self_recursion_step(body).is_some() {
        Some("Polyrust.Recursion.factF_spec")
    } else {
        None
    };
    let blank = |verdict, reason: Option<String>| FunCertReport {
        name: body.fun_name.clone(),
        verdict,
        reason,
        n_combos: 0,
        n_certified: 0,
        n_excluded: 0,
        combos: Vec::new(),
        lean_theorem,
    };
    let axes = match build_axes(body, arg_domains) {
        Ok(a) => a,
        Err(reason) => return blank("Unknown", Some(reason)),
    };
    let combos = match cart_product(&axes) {
        Ok(c) => c,
        Err(reason) => return blank("Unknown", Some(reason)),
    };
    let mut rep = FunCertReport {
        name: body.fun_name.clone(),
        verdict: "Certified",
        reason: None,
        n_combos: combos.len(),
        n_certified: 0,
        n_excluded: 0,
        combos: Vec::new(),
        lean_theorem,
    };
    for params in &combos {
        let cap = rec_depth_cap(body, params);
        let out = match concretize_cap(body, params, callees, cap) {
            Ok(o) => o,
            Err(ConcErr::Overflow) => {
                rep.n_excluded += 1;
                rep.reason.get_or_insert_with(|| {
                    format!(
                        "{}: 部分組合 checked 溢出（真實執行 panic，排除於認證範圍）",
                        rep.name
                    )
                });
                rep.combos.push(ComboCert {
                    params: params.iter().map(param_disp).collect(),
                    ret: None,
                    overflow_excluded: true,
                    nvars: 0,
                    npolys: 0,
                    l0: (0, 0, 0, 0),
                    sigma: Vec::new(),
                });
                continue;
            }
            Err(ConcErr::Reason(r)) => {
                rep.verdict = "Unknown";
                rep.reason = Some(r);
                break;
            }
        };
        // 單點域：int 參數 = 值本身；enum 參數 = 欄位值（無欄位 variant 域空）
        let doms: Vec<Vec<i64>> = params.iter().map(param_domain).collect();
        let body1 = single_path_body(body, out.path_idx);
        let enc = match encode_body(&body1, &doms, Some(&out.values)) {
            Ok(e) => e,
            Err(reason) => {
                rep.verdict = "Unknown";
                rep.reason = Some(reason);
                break;
            }
        };
        let Some(sigma) = mir_lower::solve_domains(&enc.sys) else {
            rep.verdict = "Unsat";
            break;
        };
        let cert = mir_lower::certify_mir(&enc.sys, &sigma);
        if !cert.certified {
            rep.verdict = "Unknown";
            rep.reason = Some(format!("見證認證失敗：{:?}", cert.first_bad));
            break;
        }
        let sys = &enc.sys;
        rep.combos.push(ComboCert {
            params: params.iter().map(param_disp).collect(),
            ret: enc.ret_var.and_then(|i| sigma.get(i).map(fp_to_i64)),
            overflow_excluded: false,
            nvars: sys.nvars,
            npolys: sys.polys.len(),
            l0: (
                sys.l0.n_terms,
                sys.l0.max_abs_coeff,
                sys.l0.max_abs_value,
                sys.l0.degree,
            ),
            sigma: sigma.iter().map(fp_to_i64).collect(),
        });
        rep.n_certified += 1;
    }
    if rep.verdict == "Certified" && rep.n_certified == 0 && rep.n_excluded > 0 {
        rep.verdict = "Unknown";
        rep.reason
            .get_or_insert_with(|| format!("{}: 全部組合 checked 溢出（無可認證組合）", rep.name));
    }
    rep
}

/// M3（SPA1）：FunDeclRef 級報告入口（鏡像 decide_fun_in：prelower+lowering）。
pub fn certify_fun_report_in(
    types: &Value,
    funs: &[FunDeclRef],
    fun: &FunDeclRef,
    arg_domains: &[Vec<i64>],
) -> FunCertReport {
    let callees = prelower(funs);
    match lower_fun_in(types, funs, fun) {
        PirVerdict::NoBody { kind } => FunCertReport {
            name: fun.name.clone(),
            verdict: "Unknown",
            reason: Some(format!("body 缺失（{:?}）", kind)),
            n_combos: 0,
            n_certified: 0,
            n_excluded: 0,
            combos: Vec::new(),
            lean_theorem: None,
        },
        PirVerdict::Unknown { reason } => FunCertReport {
            name: fun.name.clone(),
            verdict: "Unknown",
            reason: Some(reason),
            n_combos: 0,
            n_certified: 0,
            n_excluded: 0,
            combos: Vec::new(),
            lean_theorem: None,
        },
        PirVerdict::ValueTrace(body) => certify_fun_report(&body, arg_domains, &callees),
    }
}

/// C7：遞迴模板嘅模擬深度上限——self-call 步進 Δ、入口參數 p0 →
/// 層數 ≈ |p0|/|Δ| + 8（夠行到 base case），clamp 至 REC_DEPTH_HARD_CAP；
/// 非模板 → CALL_DEPTH_CAP 預設（如實降級路徑）。
fn rec_depth_cap(body: &PirBody, params: &[ParamVal]) -> usize {
    match self_recursion_step(body) {
        Some(delta) if delta != 0 => {
            let p0 = params
                .iter()
                .find_map(|p| match p {
                    ParamVal::Int(v) => Some(*v),
                    ParamVal::Enum { .. } => None,
                })
                .unwrap_or(0i64);
            let need = p0.abs() / delta.abs() + 8;
            need.clamp(0, REC_DEPTH_HARD_CAP as i64) as usize
        }
        _ => CALL_DEPTH_CAP,
    }
}

/// C5/C7：全部 Structured fun → PirBody 表（**含判定對象本人**——自遞迴
/// 由模板認形＋模擬深度上限承擔；mutual recursion 無模板 → 預設深限
/// 如實降級）。
fn prelower(funs: &[FunDeclRef]) -> BTreeMap<String, PirBody> {
    funs.iter()
        .filter(|f| f.body_kind == BodyKind::Structured)
        .filter_map(|f| match lower_fun_in(&Value::Null, funs, f) {
            PirVerdict::ValueTrace(b) => Some((f.name.clone(), b)),
            _ => None,
        })
        .collect()
}

/// 對已 lowering 之 body 做判定。
/// 單路徑（直線）→ C2/C3 口徑：域多值一次系統、存在見證。
/// 多路徑（C4 動態 switch）→ 逐組合見證模式：每組合模擬 → 攤平 →
/// 單點域系統 → 求解 → 認證 → ret 比對模擬；全組合通過方 CERTIFIED。
pub fn decide_body(
    body: &PirBody,
    arg_domains: &[Vec<i64>],
    callees: &BTreeMap<String, PirBody>,
) -> Decision {
    let has_call = body.paths.iter().any(|p| {
        p.stmts
            .iter()
            .any(|s| matches!(s.kind, PirStmtKind::Call { .. }))
    });
    let has_loop = body.paths.iter().any(|p| {
        p.stmts
            .iter()
            .any(|s| matches!(s.kind, PirStmtKind::Loop { .. }))
    });
    if body.paths.len() > 1 || has_call || has_loop {
        return decide_paths(body, arg_domains, callees);
    }
    let enc = match encode_body(body, arg_domains, None) {
        Ok(e) => e,
        Err(reason) => return Decision::Unknown { reason },
    };
    // 值域組合總量預檢（先於求解，區分 Unsat 與 DomainTooBig）
    let mut total: u64 = 1;
    for d in &enc.sys.domains {
        let sz = d.len().max(1) as u64;
        total = total.saturating_mul(sz);
        if total > PRODUCT_CAP {
            return Decision::Unknown {
                reason: format!("值域組合 {} 超上限 {}", total, PRODUCT_CAP),
            };
        }
    }
    let Some(sigma) = mir_lower::solve_domains(&enc.sys) else {
        return Decision::Unsat;
    };
    let cert = mir_lower::certify_mir(&enc.sys, &sigma);
    if !cert.certified {
        return Decision::Unknown {
            reason: format!("見證認證失敗：{:?}", cert.first_bad),
        };
    }
    let ret = enc.ret_var.and_then(|i| sigma.get(i).map(fp_to_i64));
    Decision::Certified {
        ret,
        n_vars: enc.slot_var.len(),
        n_eqs: enc.sys.polys.len(),
        overflow_asserted: body.overflow_asserted,
        paths: 1,
        excluded: 0,
    }
}

/// C4：多路徑 body 之逐組合判定。
fn decide_paths(
    body: &PirBody,
    arg_domains: &[Vec<i64>],
    callees: &BTreeMap<String, PirBody>,
) -> Decision {
    let name = body.fun_name.clone();
    let axes = match build_axes(body, arg_domains) {
        Ok(a) => a,
        Err(reason) => return Decision::Unknown { reason },
    };
    let combos = match cart_product(&axes) {
        Ok(c) => c,
        Err(reason) => return Decision::Unknown { reason },
    };
    let mut n_vars = 0usize;
    let mut n_eqs = 0usize;
    let mut rets: Vec<Option<i64>> = Vec::new();
    let mut excluded = 0usize;
    let mut overflow_reason: Option<String> = None;
    for params in &combos {
        let cap = rec_depth_cap(body, params);
        let out = match concretize_cap(body, params, callees, cap) {
            Ok(o) => o,
            Err(ConcErr::Overflow) => {
                excluded += 1;
                overflow_reason.get_or_insert_with(|| {
                    format!("{name}: 部分組合 checked 溢出（真實執行 panic，排除於認證範圍）")
                });
                continue;
            }
            Err(ConcErr::Reason(r)) => return Decision::Unknown { reason: r },
        };
        // 單點域：int 參數 = 值本身；enum 參數 = 欄位值（無欄位 variant 域空，見證模式容許）
        let doms: Vec<Vec<i64>> = params.iter().map(param_domain).collect();
        let body1 = single_path_body(body, out.path_idx);
        let enc = match encode_body(&body1, &doms, Some(&out.values)) {
            Ok(e) => e,
            Err(reason) => return Decision::Unknown { reason },
        };
        let Some(sigma) = mir_lower::solve_domains(&enc.sys) else {
            return Decision::Unsat;
        };
        let cert = mir_lower::certify_mir(&enc.sys, &sigma);
        if !cert.certified {
            return Decision::Unknown {
                reason: format!("見證認證失敗：{:?}", cert.first_bad),
            };
        }
        let sys_ret = enc.ret_var.and_then(|i| sigma.get(i).map(fp_to_i64));
        if sys_ret != out.ret {
            return Decision::Unknown {
                reason: format!("編碼 ret {sys_ret:?} ≠ 模擬 ret {:?}", out.ret),
            };
        }
        n_vars = n_vars.max(enc.slot_var.len());
        n_eqs = n_eqs.max(enc.sys.polys.len());
        rets.push(out.ret);
    }
    if rets.is_empty() {
        return Decision::Unknown {
            reason: overflow_reason.unwrap_or_else(|| format!("{name}: 無可認證組合")),
        };
    }
    let first = rets[0];
    let uniform = rets.iter().all(|r| *r == first);
    Decision::Certified {
        ret: if uniform { first } else { None },
        n_vars,
        n_eqs,
        overflow_asserted: body.overflow_asserted,
        paths: rets.len(),
        excluded,
    }
}

/// 組合軸：int 參數 = 域；enum 參數 = 各 variant × 欄位域。
fn build_axes(body: &PirBody, arg_domains: &[Vec<i64>]) -> Result<Vec<Vec<ParamVal>>, String> {
    let mut axes = Vec::new();
    for i in 0..body.arg_count {
        let dom = arg_domains.get(i).cloned().unwrap_or_default();
        if let Some(ep) = body.enum_params.iter().find(|e| e.arg == i) {
            let mut ax = Vec::new();
            for (disc, nf) in &ep.variants {
                match nf {
                    0 => ax.push(ParamVal::Enum {
                        disc: *disc,
                        field: None,
                    }),
                    1 => {
                        if dom.len() > DOMAIN_CAP {
                            return Err(format!("參數 {i} 域大於上限 {DOMAIN_CAP}"));
                        }
                        for &v in &dom {
                            ax.push(ParamVal::Enum {
                                disc: *disc,
                                field: Some(v),
                            });
                        }
                    }
                    _ => return Err(format!("參數 {i}: 多欄位 variant 唔支援")),
                }
            }
            if ax.is_empty() {
                return Err(format!(
                    "參數 {i}（enum）組合軸空：所有 variant 都需要欄位值域"
                ));
            }
            axes.push(ax);
        } else {
            if dom.is_empty() {
                return Err(format!("參數 {} 域為空", i));
            }
            if dom.len() > DOMAIN_CAP {
                return Err(format!("參數 {} 域大於上限 {}", i, DOMAIN_CAP));
            }
            axes.push(dom.iter().map(|&v| ParamVal::Int(v)).collect());
        }
    }
    Ok(axes)
}

/// 笛卡爾積（上限 CONTEXT_CAP）。
fn cart_product(axes: &[Vec<ParamVal>]) -> Result<Vec<Vec<ParamVal>>, String> {
    let mut total = 1usize;
    for a in axes {
        total = total.saturating_mul(a.len());
        if total > CONTEXT_CAP {
            return Err(format!("組合數 {total} 超上限 {CONTEXT_CAP}（縮窄參數域）"));
        }
    }
    let mut out: Vec<Vec<ParamVal>> = vec![Vec::new()];
    for ax in axes {
        let mut next = Vec::with_capacity(out.len() * ax.len());
        for row in &out {
            for v in ax {
                let mut r = row.clone();
                r.push(v.clone());
                next.push(r);
            }
        }
        out = next;
    }
    Ok(out)
}

/// 組合 → 單點域（encode_body 用；enum 無欄位 variant → 空域，見證模式容許）。
fn param_domain(p: &ParamVal) -> Vec<i64> {
    match p {
        ParamVal::Int(v) => vec![*v],
        ParamVal::Enum { field: Some(v), .. } => vec![*v],
        ParamVal::Enum { .. } => Vec::new(),
    }
}

/// 攤平：取第 idx 條路徑成單路徑 body（guard 已由模擬消費）。
fn single_path_body(body: &PirBody, idx: usize) -> PirBody {
    let mut b = body.clone();
    let p = b.paths.remove(idx);
    b.paths = vec![PirPath {
        scrutinee: None,
        value: None,
        fallback: false,
        excl: Vec::new(),
        stmts: p.stmts,
    }];
    b
}

/// 在 crate root 內按名稱找函數並判定（name = path 最尾段）。
/// 自動帶 root 嘅 type_decls。
pub fn decide_entry(root: &LlbcRoot, name: &str, arg_domains: &[Vec<i64>]) -> Option<Decision> {
    let types = root
        .raw_translated
        .get("type_decls")
        .unwrap_or(&Value::Null);
    root.funs
        .iter()
        .find(|f| f.name == name)
        .map(|f| decide_fun_in(types, &root.funs, f, arg_domains))
}

/// 模組總覽：全部函數逐一報 lowering 判決（供 CLI `pir` 總表）。
/// 唔帶 type_decls；要 enum Aggregate 支援用 [`survey_in`]。
pub fn survey(root: &LlbcRoot) -> Vec<(String, String)> {
    survey_in(&Value::Null, root)
}

/// 模組總覽（帶 type_decls）。
pub fn survey_in(types: &Value, root: &LlbcRoot) -> Vec<(String, String)> {
    root.funs
        .iter()
        .map(|f| {
            let line = match lower_fun_in(types, &root.funs, f) {
                PirVerdict::ValueTrace(b) => format!(
                    "ValueTrace(stmts={} argc={} paths={} overflow_asserted={})",
                    b.paths.iter().map(|p| p.stmts.len()).sum::<usize>(),
                    b.arg_count,
                    b.paths.len(),
                    b.overflow_asserted
                ),
                PirVerdict::Unknown { reason } => format!("Unknown({})", reason),
                PirVerdict::NoBody { kind } => format!("NoBody({:?})", kind),
            };
            (f.name.clone(), line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charon_llbc::LlbcRoot;

    const FIX_DIR: &str = "tests/charon_fixtures";

    fn load(name: &str) -> LlbcRoot {
        let path = format!("{FIX_DIR}/{name}.llbc");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {name}: {e}"));
        LlbcRoot::parse(&text).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    fn dom(vals: &[i64]) -> Vec<Vec<i64>> {
        vals.iter().map(|&v| vec![v]).collect()
    }

    #[test]
    fn sqr_certifies_squares() {
        let root = load("sqr");
        for (x, want) in [(3i64, 9i64), (5, 25), (0, 0), (-4, 16)] {
            let d = decide_entry(&root, "sqr", &dom(&[x])).expect("sqr 存在");
            match d {
                Decision::Certified {
                    ret,
                    overflow_asserted,
                    n_eqs,
                    ..
                } => {
                    assert_eq!(ret, Some(want), "sqr({x})");
                    assert!(overflow_asserted, "checked mul 必帶溢出斷言");
                    assert!(n_eqs > 0);
                }
                other => panic!("sqr({x}) 應 Certified，得到 {:?}", other),
            }
        }
    }

    #[test]
    fn add_certifies_sum() {
        let root = load("add");
        let d = decide_entry(&root, "add", &dom(&[2, 5])).expect("add 存在");
        match d {
            Decision::Certified {
                ret,
                overflow_asserted,
                ..
            } => {
                assert_eq!(ret, Some(7));
                assert!(overflow_asserted);
            }
            other => panic!("add 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn io_pure_multiplies_by_const() {
        let root = load("io_pure");
        let d = decide_entry(&root, "pure_fn", &dom(&[21])).expect("pure_fn 存在");
        match d {
            Decision::Certified { ret, .. } => assert_eq!(ret, Some(42)),
            other => panic!("io_pure 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn sqr_multi_value_domain() {
        let root = load("sqr");
        let d = decide_entry(&root, "sqr", &vec![vec![2, 3, 4]]).expect("sqr 存在");
        match d {
            Decision::Certified { ret, .. } => {
                assert!(
                    ret == Some(4) || ret == Some(9) || ret == Some(16),
                    "ret={ret:?}"
                );
            }
            other => panic!("多值域應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn unsupported_shapes_degrade_with_reasons() {
        // C4 起：max（比較+Switch）、match_option/enum_option（enum 判別值軸）
        // 全部轉為正面支援（見 c4_* 測試）；剩降級形態如下。
        let cases: [(&str, &str, &str); 3] = [
            ("while_loop", "while_sum", "唔支援"),
            ("phase3__loop_sat", "main", "唔支援"),
            ("async_simple", "async_add", "缺失 body"),
        ];
        for (file, fun, needle) in cases {
            let root = load(file);
            let d =
                decide_entry(&root, fun, &[]).unwrap_or_else(|| panic!("{file}: fun {fun} 不存在"));
            assert!(!d.is_certified(), "{file}:{fun} 不應 Certified");
            let hit = match &d {
                Decision::Unknown { reason } => reason.contains(needle) || needle == "Unknown",
                _ => false,
            };
            assert!(hit, "{file}:{fun} → {:?}（期望含 `{needle}`）", d.reason());
        }
    }

    #[test]
    fn c4_max_comparison_switch_certified() {
        // max(x,y)：Gt 比較 + bool Switch（then 臂 = fallback+fall-through）
        // 逐組合見證模式：guard 驗證 + 單點域系統 + ret 比對模擬
        let root = load("max");
        for (x, y, want) in [(3, 5, 5), (7, 2, 7), (4, 4, 4), (-3, -8, -3)] {
            let d = decide_entry(&root, "max", &dom(&[x, y])).expect("max 存在");
            match d {
                Decision::Certified {
                    ret,
                    paths,
                    excluded,
                    ..
                } => {
                    assert_eq!(ret, Some(want), "max({x},{y})");
                    assert_eq!(paths, 1, "單點域 = 1 組合");
                    assert_eq!(excluded, 0);
                }
                other => panic!("max({x},{y}) 應 Certified，得到 {:?}", other),
            }
        }
        // 多值域：全組合認證，ret 不唯一 → None + paths
        let d = decide_entry(&root, "max", &vec![vec![1, 9], vec![5]]).expect("max 存在");
        match d {
            Decision::Certified { ret, paths, .. } => {
                assert_eq!(paths, 2);
                assert_eq!(ret, None, "max(1,5)=5、max(9,5)=9 → ret 不唯一");
            }
            other => panic!("多值域 max 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn c4_match_option_enum_switch_certified() {
        // match_option(o)：輸入 enum（判別值軸全枚舉）+ Some 臂欄位投影
        let root = load("match_option");
        let d = decide_entry(&root, "match_option", &vec![vec![0, 7]]).expect("存在");
        match d {
            Decision::Certified {
                paths, excluded, ..
            } => {
                // 軸：None(disc 0) + Some(disc 1, x∈{0,7}) = 3 組合
                assert_eq!(paths, 3, "None 1 + Some×2");
                assert_eq!(excluded, 0);
            }
            other => panic!("match_option 應 Certified，得到 {:?}", other),
        }
        // 域 {7}：None 臂 + Some(7) 兩組合
        let d = decide_entry(&root, "match_option", &dom(&[7])).expect("存在");
        match d {
            Decision::Certified { paths, .. } => assert_eq!(paths, 2),
            other => panic!("應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn c4_unwrap_or_two_params_certified() {
        // unwrap_or(self, d)：self=自訂 enum（域=欄位值）、d=int
        // unwrap_or=7;5 → self 域 {7}、d 域 {5}：組合 (None,5) ret=5 + (Some(7),5) ret=7
        let root = load("enum_option");
        let d = decide_entry(&root, "unwrap_or", &dom(&[7, 5])).expect("存在");
        match d {
            Decision::Certified { ret, paths, .. } => {
                assert_eq!(paths, 2);
                assert_eq!(ret, None, "兩組合 ret 不同（5/7）");
            }
            other => panic!("unwrap_or 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn c3_aggregate_projection_deref_certify() {
        // struct Point::new：Aggregate 建構（欄位變數）→ Certified，ret=None
        // （ADT 整槽不透明，欄位恆等式承擔認證）
        let root = load("struct_point");
        let d = decide_entry(&root, "new", &dom(&[3, 4])).expect("new 存在");
        match d {
            Decision::Certified {
                ret,
                n_eqs,
                overflow_asserted,
                ..
            } => {
                assert_eq!(ret, None, "ADT ret 整槽不透明");
                assert!(n_eqs >= 2, "至少兩條欄位恆等式，得到 eqs={n_eqs}");
                assert!(!overflow_asserted);
            }
            other => panic!("new 應 Certified，得到 {:?}", other),
        }
        // Point::len(&self) = x*x + y*y：Deref 透明 + Field 投影 + checked 運算
        // 單值域 {3}：x=y=3 → 18
        let d = decide_entry(&root, "len", &dom(&[3])).expect("len 存在");
        match d {
            Decision::Certified {
                ret,
                overflow_asserted,
                ..
            } => {
                assert_eq!(ret, Some(18), "3*3+3*3");
                assert!(overflow_asserted, "checked mul 必帶溢出斷言");
            }
            other => panic!("len 應 Certified，得到 {:?}", other),
        }
        // borrow_immut：*(&x) 透明別位 → 恆等函數
        let root2 = load("borrow_immut");
        let d = decide_entry(&root2, "borrow_immut", &dom(&[7])).expect("borrow_immut 存在");
        match d {
            Decision::Certified { ret, .. } => assert_eq!(ret, Some(7)),
            other => panic!("borrow_immut 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn c3_enum_aggregate_disc_propagates_to_switch_hint() {
        // 合成 LLBC：let v = MyResult::Ok(42)（variant id 1、判別值 3——
        // 刻意非平凡，證明判別值真係由 type_decls 解析，唔係 variant id 猜測）；
        // d := Discriminant(v) → 常數傳播 → ret := d 應 Certified ret=3。
        // 再加 Switch(Move d)：降級但原因附 scrutinee 判別值提示。
        use crate::charon_llbc::{BodyKind, FunDeclRef as FDR, Value};
        let n = |s: &str| Value::Num(s.to_string());
        let int_const = |v: &str| {
            Value::Obj(vec![(
                "Value".to_string(),
                Value::Arr(vec![
                    Value::Null,
                    Value::Arr(vec![
                        Value::Obj(vec![(
                            "Integer".to_string(),
                            Value::Obj(vec![(
                                "Signed".to_string(),
                                Value::Arr(vec![
                                    Value::Str("Isize".to_string()),
                                    Value::Str(v.to_string()),
                                ]),
                            )]),
                        )]),
                        Value::Null,
                    ]),
                ]),
            )])
        };
        let local_place = |idx: &str| {
            Value::Obj(vec![
                (
                    "kind".to_string(),
                    Value::Obj(vec![("Local".to_string(), n(idx))]),
                ),
                ("ty".to_string(), Value::Null),
            ])
        };
        let agg_rv = Value::Obj(vec![(
            "Aggregate".to_string(),
            Value::Arr(vec![
                // Adt id 0、variant id 1（判別值由 type_decls 查出 = 3）
                Value::Obj(vec![(
                    "Adt".to_string(),
                    Value::Arr(vec![
                        Value::Obj(vec![("id".to_string(), n("0"))]),
                        n("1"),
                        Value::Null,
                    ]),
                )]),
                Value::Arr(vec![Value::Obj(vec![(
                    "Const".to_string(),
                    int_const("42"),
                )])]),
            ]),
        )]);
        // type_decls：enum #0，variant 0 判別值 0、variant 1 判別值 3
        let variant = |id: &str, disc: &str| {
            Value::Obj(vec![
                ("id".to_string(), n(id)),
                (
                    "discriminant".to_string(),
                    Value::Obj(vec![(
                        "Signed".to_string(),
                        Value::Arr(vec![
                            Value::Str("Isize".to_string()),
                            Value::Str(disc.to_string()),
                        ]),
                    )]),
                ),
            ])
        };
        let types = Value::Arr(vec![Value::Obj(vec![
            ("def_id".to_string(), n("0")),
            (
                "kind".to_string(),
                Value::Obj(vec![(
                    "Enum".to_string(),
                    Value::Arr(vec![variant("0", "0"), variant("1", "3")]),
                )]),
            ),
        ])]);
        let mk_fun = |name: &str, extra_switch: bool| FDR {
            def_id: 0,
            name: name.to_string(),
            full_path: vec![name.to_string()],
            body_kind: BodyKind::Structured,
            raw: Value::Obj(vec![
                (
                    "signature".to_string(),
                    Value::Obj(vec![("inputs".to_string(), Value::Arr(vec![]))]),
                ),
                (
                    "body".to_string(),
                    Value::Obj(vec![(
                        "Structured".to_string(),
                        Value::Obj(vec![
                            (
                                "locals".to_string(),
                                Value::Obj(vec![
                                    ("arg_count".to_string(), n("0")),
                                    ("locals".to_string(), Value::Arr(vec![Value::Null])),
                                ]),
                            ),
                            (
                                "body".to_string(),
                                Value::Obj(vec![(
                                    "statements".to_string(),
                                    Value::Arr(vec![
                                        Value::Obj(vec![(
                                            "kind".to_string(),
                                            Value::Obj(vec![(
                                                "Assign".to_string(),
                                                Value::Arr(vec![local_place("2"), agg_rv.clone()]),
                                            )]),
                                        )]),
                                        Value::Obj(vec![(
                                            "kind".to_string(),
                                            Value::Obj(vec![(
                                                "Assign".to_string(),
                                                Value::Arr(vec![
                                                    local_place("3"),
                                                    Value::Obj(vec![(
                                                        "Use".to_string(),
                                                        Value::Arr(vec![Value::Obj(vec![(
                                                            "Copy".to_string(),
                                                            local_place("2"),
                                                        )])]),
                                                    )]),
                                                ]),
                                            )]),
                                        )]),
                                        Value::Obj(vec![(
                                            "kind".to_string(),
                                            if extra_switch {
                                                // Switch(Move L3)：scrutinee 判別值已知（3）→
                                                // C4 靜態選臂：行常數 3 之臂（空），唔行 0 臂
                                                Value::Obj(vec![(
                                                "Switch".to_string(),
                                                Value::Obj(vec![
                                                    (
                                                        "data".to_string(),
                                                        Value::Obj(vec![
                                                            (
                                                                "scrutinee".to_string(),
                                                                Value::Obj(vec![(
                                                                    "Value".to_string(),
                                                                    Value::Obj(vec![(
                                                                        "Move".to_string(),
                                                                        local_place("3"),
                                                                    )]),
                                                                )]),
                                                            ),
                                                            (
                                                                "branches".to_string(),
                                                                Value::Arr(vec![Value::Arr(vec![
                                                                    int_const("3"),
                                                                    n("0"),
                                                                ])]),
                                                            ),
                                                            ("fallback".to_string(), n("1")),
                                                        ]),
                                                    ),
                                                    (
                                                        "branches".to_string(),
                                                        Value::Arr(vec![
                                                            // 臂 0（常數 3 →）：L0 := 7
                                                            Value::Obj(vec![(
                                                                "statements".to_string(),
                                                                Value::Arr(vec![Value::Obj(vec![(
                                                                    "kind".to_string(),
                                                                    Value::Obj(vec![(
                                                                        "Assign".to_string(),
                                                                        Value::Arr(vec![
                                                                            local_place("0"),
                                                                            Value::Obj(vec![(
                                                                                "Use".to_string(),
                                                                                Value::Arr(vec![Value::Obj(vec![(
                                                                                    "Const".to_string(),
                                                                                    int_const("7"),
                                                                                )])]),
                                                                            )]),
                                                                        ]),
                                                                    )]),
                                                                )])]),
                                                            )]),
                                                            // 臂 1（fallback →）：L0 := 999
                                                            Value::Obj(vec![(
                                                                "statements".to_string(),
                                                                Value::Arr(vec![Value::Obj(vec![(
                                                                    "kind".to_string(),
                                                                    Value::Obj(vec![(
                                                                        "Assign".to_string(),
                                                                        Value::Arr(vec![
                                                                            local_place("0"),
                                                                            Value::Obj(vec![(
                                                                                "Use".to_string(),
                                                                                Value::Arr(vec![Value::Obj(vec![(
                                                                                    "Const".to_string(),
                                                                                    int_const("999"),
                                                                                )])]),
                                                                            )]),
                                                                        ]),
                                                                    )]),
                                                                )])]),
                                                            )]),
                                                        ]),
                                                    ),
                                                ]),
                                            )])
                                            } else {
                                                // L0 := Discriminant(L3) → 常數 3
                                                Value::Obj(vec![(
                                                    "Assign".to_string(),
                                                    Value::Arr(vec![
                                                        local_place("0"),
                                                        Value::Obj(vec![(
                                                            "Discriminant".to_string(),
                                                            local_place("3"),
                                                        )]),
                                                    ]),
                                                )])
                                            },
                                        )]),
                                        Value::Obj(vec![(
                                            "kind".to_string(),
                                            Value::Str("Return".to_string()),
                                        )]),
                                    ]),
                                )]),
                            ),
                        ]),
                    )]),
                ),
            ]),
        };

        // 1）Discriminant 常數傳播 → Certified ret=3（非 variant id 1）
        let fun = mk_fun("ctor_disc", false);
        match decide_fun_in(&types, &[], &fun, &[]) {
            Decision::Certified { ret, .. } => assert_eq!(ret, Some(3), "判別值=3 非 variant id"),
            other => panic!("ctor_disc 應 Certified，得到 {:?}", other),
        }
        // 2）Switch：降級 + scrutinee 判別值提示
        let fun2 = mk_fun("ctor_switch", true);
        match decide_fun_in(&types, &[], &fun2, &[]) {
            Decision::Certified { ret, .. } => assert_eq!(ret, Some(7), "靜態選臂：行常數 3 臂"),
            other => panic!("ctor_switch 應 Certified（靜態選臂），得到 {:?}", other),
        }
    }

    fn load_m0(name: &str) -> LlbcRoot {
        let path = format!("../m0/spike_out/{name}.llbc");
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("m0 fixture {name}: {e}"));
        LlbcRoot::parse(&text).unwrap_or_else(|e| panic!("m0 fixture {name}: {e}"))
    }

    #[test]
    fn c5_call_inline_certified() {
        // 合成：inner(x) = x+1；outer(y) = inner(y)*2 —— 調用語義由模擬承擔，
        // 編碼見證恆等化，認證鏈閉合
        use crate::charon_llbc::{BodyKind, FunDeclRef as FDR, Value};
        let o = |ps: Vec<(&str, Value)>| {
            Value::Obj(ps.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
        };
        let num = |s: &str| Value::Num(s.to_string());
        let int_const = |v: &str| {
            o(vec![(
                "Value",
                Value::Arr(vec![
                    Value::Null,
                    Value::Arr(vec![
                        o(vec![(
                            "Integer",
                            o(vec![(
                                "Signed",
                                Value::Arr(vec![
                                    Value::Str("I64".to_string()),
                                    Value::Str(v.to_string()),
                                ]),
                            )]),
                        )]),
                        Value::Null,
                    ]),
                ]),
            )])
        };
        let place = |idx: &str| {
            o(vec![
                ("kind", o(vec![("Local", num(idx))])),
                ("ty", Value::Null),
            ])
        };
        let assign = |dst: &str, rhs: Value| {
            o(vec![(
                "kind",
                o(vec![("Assign", Value::Arr(vec![place(dst), rhs]))]),
            )])
        };
        let binop = |op: &str, a: &str, c: &str| {
            o(vec![(
                "BinaryOp",
                Value::Arr(vec![
                    Value::Str(op.to_string()),
                    Value::Obj(vec![("Copy".to_string(), place(a))]),
                    Value::Obj(vec![("Const".to_string(), int_const(c))]),
                ]),
            )])
        };
        let locals = |argc: usize| {
            o(vec![
                ("arg_count", Value::Num(argc.to_string())),
                (
                    "locals",
                    Value::Arr((0..=argc).map(|_| Value::Null).collect()),
                ),
            ])
        };
        let mk_fun = |def_id: i64, name: &str, stmts: Vec<Value>| FDR {
            def_id,
            name: name.to_string(),
            full_path: vec![name.to_string()],
            body_kind: BodyKind::Structured,
            raw: o(vec![
                ("signature", o(vec![("inputs", Value::Arr(vec![]))])),
                (
                    "body",
                    o(vec![(
                        "Structured",
                        o(vec![
                            ("locals", locals(1)),
                            ("body", o(vec![("statements", Value::Arr(stmts))])),
                        ]),
                    )]),
                ),
            ]),
        };

        // inner：L0 := L1 + 1
        let inner = mk_fun(0, "inner", vec![assign("0", binop("Add", "1", "1"))]);
        // outer：L2 := inner(L1)；L0 := L2 * 2；Return
        let call_inner = o(vec![(
            "kind",
            o(vec![(
                "Call",
                o(vec![(
                    "call",
                    o(vec![
                        (
                            "func",
                            o(vec![(
                                "Regular",
                                o(vec![("kind", o(vec![("Fun", num("0"))]))]),
                            )]),
                        ),
                        (
                            "args",
                            Value::Arr(vec![Value::Obj(vec![("Copy".to_string(), place("1"))])]),
                        ),
                        ("dest", place("2")),
                    ]),
                )]),
            )]),
        )]);
        let outer = mk_fun(
            1,
            "outer",
            vec![
                call_inner,
                assign("0", binop("Mul", "2", "2")),
                Value::Obj(vec![("kind".to_string(), Value::Str("Return".to_string()))]),
            ],
        );
        let funs = [inner, outer];
        let outer_ref = &funs[1];
        // 無域：int 參數組合軸空 → 如實降級（Call 強制組合模式）
        let d = decide_fun_in(&Value::Null, &funs, outer_ref, &[]);
        assert!(d.reason().is_some(), "無域應降級：{d:?}");
        // 有域：全鏈認證 ret = (y+1)*2
        for (y, want) in [(5, 12), (0, 2), (-3, -4)] {
            let d = decide_fun_in(&Value::Null, &funs, outer_ref, &vec![vec![y]]);
            match d {
                Decision::Certified {
                    ret,
                    paths,
                    excluded,
                    ..
                } => {
                    assert_eq!(ret, Some(want), "outer({y})");
                    assert_eq!(paths, 1);
                    assert_eq!(excluded, 0);
                }
                other => panic!("outer({y}) 應 Certified，得到 {:?}", other),
            }
        }
        // callee 自身直線判定不變
        let d = decide_fun_in(&Value::Null, &funs, &funs[0], &vec![vec![9]]);
        match d {
            Decision::Certified { ret, .. } => assert_eq!(ret, Some(10)),
            other => panic!("inner(9) 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn c5_m0_call_chain_degrades_honestly() {
        // 真檔（charon 0.1.265 CI 回寫）：outer 調 pure_inner 之後仲調
        // Opaque 嘅 display/print（body="Opaque"）→ 模擬降級如實申報；
        // pure_inner 自身直線 → CERTIFIED
        let root = load_m0("io_with_pure_call");
        let d = decide_entry(&root, "outer", &[]).expect("outer 存在");
        // outer 嘅呼叫鏈會撞 Opaque callee／Ref rvalue 等能力邊界——
        // 重點：如實降級（非 panic、非錯誤 CERTIFIED）
        assert!(
            !d.is_certified(),
            "outer 應 Unknown（opaque 呼叫鏈），得到 {d:?}"
        );
        let d = decide_entry(&root, "pure_inner", &dom(&[5])).expect("pure_inner 存在");
        match d {
            Decision::Certified { ret, .. } => assert_eq!(ret, Some(6), "pure_inner(5)=5+1"),
            other => panic!("pure_inner 應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn c7_fact_recursion_certified_real_file() {
        // 真檔 fact：自遞迴模板（n − 1 單一 self-call）→ C7 認證
        let root = load_m0("fact");
        let d = decide_entry(&root, "fact", &dom(&[3])).expect("fact 存在");
        match d {
            Decision::Certified { ret, excluded, .. } => {
                assert_eq!(ret, Some(6), "fact(3)=6");
                assert_eq!(excluded, 0);
            }
            other => panic!("fact(3) 應 Certified（C7 遞迴模板），得到 {other:?}"),
        }
        // 深度上限：巨大 n → 如實降級（模板終止假設失效／超硬上限）。
        // 4096 層模擬遞迴：debug 框架大過預設 test thread stack——
        // 用 64MB stack thread 執行。
        let d2 = std::thread::Builder::new()
            .stack_size(64 << 20)
            .spawn(|| {
                let root = load_m0("fact");
                decide_entry(&root, "fact", &dom(&[100_000])).expect("fact 存在")
            })
            .unwrap()
            .join()
            .unwrap();
        let reason = d2
            .reason()
            .unwrap_or_else(|| panic!("fact(100000) 應 Unknown，得到 {d2:?}"));
        assert!(
            reason.contains("深度超限") || reason.contains("硬上限"),
            "應指向深度上限：{reason}"
        );
    }

    #[cfg(test)]
    fn c6_while_sum_fdr() -> crate::charon_llbc::FunDeclRef {
        // 合成 while_sum：acc=0; i=0; while (i<n) { acc+=i; i+=1 } ret=acc
        // = Σ_{k<n} k。終止模板：i<n 單調 +1；temp-Copy 兩段步進。
        use crate::charon_llbc::{BodyKind, FunDeclRef, Value};
        let num = |s: &str| Value::Num(s.to_string());
        let sng = |k: &str, v: Value| Value::Obj(vec![(k.to_string(), v)]);
        let arrv = |xs: Vec<Value>| Value::Arr(xs);
        let o = |ps: Vec<(&str, Value)>| {
            Value::Obj(ps.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
        };
        let sgn = |v: &str| {
            sng(
                "Integer",
                sng(
                    "Signed",
                    arrv(vec![Value::Str("I64".into()), Value::Str(v.into())]),
                ),
            )
        };
        let int_const = |v: &str| {
            sng(
                "Value",
                arrv(vec![Value::Null, arrv(vec![sgn(v), Value::Null])]),
            )
        };
        let bool_const = |b: bool| {
            sng(
                "Value",
                arrv(vec![
                    Value::Null,
                    arrv(vec![sng("Bool", Value::Bool(b)), Value::Null]),
                ]),
            )
        };
        let place = |idx: &str| o(vec![("kind", sng("Local", num(idx))), ("ty", Value::Null)]);
        let stmt = |k: Value| sng("kind", k);
        let ucopy = |src: &str| sng("Use", arrv(vec![sng("Copy", place(src))]));
        let uconst = |v: &str| sng("Use", arrv(vec![sng("Const", int_const(v))]));
        let assign =
            |dst: &str, rhs: Value| sng("kind", sng("Assign", arrv(vec![place(dst), rhs])));
        let binop = |op: &str, a: &str, b: &str| {
            sng(
                "BinaryOp",
                arrv(vec![
                    Value::Str(op.into()),
                    sng("Copy", place(a)),
                    sng("Copy", place(b)),
                ]),
            )
        };
        let binop_c = |op: &str, a: &str, c: &str| {
            sng(
                "BinaryOp",
                arrv(vec![
                    Value::Str(op.into()),
                    sng("Copy", place(a)),
                    sng("Const", int_const(c)),
                ]),
            )
        };
        let arm = |sts: Vec<Value>| sng("statements", arrv(sts));
        let cont = stmt(sng("Continue", num("0")));
        // Switch(scrut)：branches=[false→arm0(出口)]、fallback=arm1(body)
        let switch = |scrut: &str, false_arm: Value, fb_arm: Value| {
            sng(
                "kind",
                sng(
                    "Switch",
                    o(vec![
                        (
                            "data",
                            o(vec![
                                ("scrutinee", sng("Value", sng("Move", place(scrut)))),
                                (
                                    "branches",
                                    arrv(vec![arrv(vec![bool_const(false), num("0")])]),
                                ),
                                ("fallback", num("1")),
                            ]),
                        ),
                        ("branches", arrv(vec![false_arm, fb_arm])),
                    ]),
                ),
            )
        };
        // head：L5:=L3(i)、L6:=L1(n)、L4:=Lt(L5,L6)、Switch(L4)
        let head_stmts = vec![
            assign("5", ucopy("3")),
            assign("6", ucopy("1")),
            assign("4", binop("Lt", "5", "6")),
            switch(
                "4",
                arm(vec![]),
                arm(vec![
                    assign("7", binop("Add", "2", "5")),   // L7 := acc + i
                    assign("2", ucopy("7")),               // acc := L7
                    assign("8", binop_c("Add", "5", "1")), // L8 := i + 1
                    assign("3", ucopy("8")),               // i := L8
                    cont,
                ]),
            ),
        ];
        let loop_stmt = stmt(sng("Loop", o(vec![("statements", arrv(head_stmts))])));
        FunDeclRef {
            def_id: 0,
            name: "while_sum".to_string(),
            full_path: vec!["while_sum".to_string()],
            body_kind: BodyKind::Structured,
            raw: o(vec![
                ("signature", sng("inputs", arrv(vec![]))),
                (
                    "body",
                    sng(
                        "Structured",
                        o(vec![
                            (
                                "locals",
                                o(vec![
                                    ("arg_count", num("1")),
                                    ("locals", arrv(vec![Value::Null; 9])),
                                ]),
                            ),
                            (
                                "body",
                                sng(
                                    "statements",
                                    arrv(vec![
                                        assign("2", uconst("0")), // acc := 0
                                        assign("3", uconst("0")), // i := 0
                                        loop_stmt,
                                        assign("0", ucopy("2")), // ret := acc
                                        stmt(Value::Str("Return".into())),
                                    ]),
                                ),
                            ),
                        ]),
                    ),
                ),
            ]),
        }
    }

    #[test]
    fn c6_while_sum_loop_certified() {
        // Σ_{k<n} k：n=5 → 10、n=0 → 0（zero-iteration）、n=7 → 21
        use crate::charon_llbc::Value;
        let fun = c6_while_sum_fdr();
        for (n, want) in [(5, 10), (0, 0), (1, 0), (7, 21)] {
            let d = decide_fun_in(&Value::Null, &[], &fun, &vec![vec![n]]);
            match d {
                Decision::Certified { ret, excluded, .. } => {
                    assert_eq!(ret, Some(want), "while_sum({n})");
                    assert_eq!(excluded, 0);
                }
                other => panic!("while_sum({n}) 應 Certified，得到 {other:?}"),
            }
        }
    }

    #[test]
    fn c6_huge_bound_degrades_honestly() {
        // 大界值：推導步數上限 100_004 超硬上限 4096 → 如實降級（防 DoS）
        use crate::charon_llbc::Value;
        let d = decide_fun_in(&Value::Null, &[], &c6_while_sum_fdr(), &vec![vec![100_000]]);
        let reason = d
            .reason()
            .unwrap_or_else(|| panic!("大界值 loop 應 Unknown，得到 {d:?}"));
        assert!(reason.contains("硬上限"), "應指向迭代硬上限：{reason}");
    }

    #[test]
    fn c6_overflow_in_loop_excluded() {
        // i=0、n=i64::MAX、步 +2^62（AddChecked 真實 LLBC 形態：tuple+Assert+投影）
        // 第 2 輪 t=i+2^62 溢出 → 該組合如實排除（excluded=1）
        use crate::charon_llbc::{BodyKind, FunDeclRef, Value};
        let num = |s: &str| Value::Num(s.to_string());
        let sng = |k: &str, v: Value| Value::Obj(vec![(k.to_string(), v)]);
        let arrv = |xs: Vec<Value>| Value::Arr(xs);
        let o = |ps: Vec<(&str, Value)>| {
            Value::Obj(ps.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
        };
        let sgn = |v: &str| {
            sng(
                "Integer",
                sng(
                    "Signed",
                    arrv(vec![Value::Str("I64".into()), Value::Str(v.into())]),
                ),
            )
        };
        let int_const = |v: &str| {
            sng(
                "Value",
                arrv(vec![Value::Null, arrv(vec![sgn(v), Value::Null])]),
            )
        };
        let bool_const = |b: bool| {
            sng(
                "Value",
                arrv(vec![
                    Value::Null,
                    arrv(vec![sng("Bool", Value::Bool(b)), Value::Null]),
                ]),
            )
        };
        let place = |idx: &str| o(vec![("kind", sng("Local", num(idx))), ("ty", Value::Null)]);
        let place_fld = |idx: &str, f: &str| {
            o(vec![
                (
                    "kind",
                    sng(
                        "Projection",
                        arrv(vec![
                            place(idx),
                            sng("Field", arrv(vec![Value::Null, num(f)])),
                        ]),
                    ),
                ),
                ("ty", Value::Null),
            ])
        };
        let stmt = |k: Value| sng("kind", k);
        let ucopy = |src: &str| sng("Use", arrv(vec![sng("Copy", place(src))]));
        let uconst = |v: &str| sng("Use", arrv(vec![sng("Const", int_const(v))]));
        let assign =
            |dst: &str, rhs: Value| sng("kind", sng("Assign", arrv(vec![place(dst), rhs])));
        let arm = |sts: Vec<Value>| sng("statements", arrv(sts));
        let cont = stmt(sng("Continue", num("0")));
        let switch = |scrut: &str, false_arm: Value, fb_arm: Value| {
            sng(
                "kind",
                sng(
                    "Switch",
                    o(vec![
                        (
                            "data",
                            o(vec![
                                ("scrutinee", sng("Value", sng("Move", place(scrut)))),
                                (
                                    "branches",
                                    arrv(vec![arrv(vec![bool_const(false), num("0")])]),
                                ),
                                ("fallback", num("1")),
                            ]),
                        ),
                        ("branches", arrv(vec![false_arm, fb_arm])),
                    ]),
                ),
            )
        };
        let head_stmts = vec![
            assign("5", ucopy("3")),
            assign("6", ucopy("1")),
            sng(
                "kind",
                sng(
                    "Assign",
                    arrv(vec![
                        place("4"),
                        sng(
                            "BinaryOp",
                            arrv(vec![
                                Value::Str("Lt".into()),
                                sng("Copy", place("5")),
                                sng("Copy", place("6")),
                            ]),
                        ),
                    ]),
                ),
            ),
            switch(
                "4",
                arm(vec![]),
                arm(vec![
                    // L7 := AddChecked(i, 2^62)（tuple：f0=值、f1=溢出旗標）
                    assign(
                        "7",
                        sng(
                            "BinaryOp",
                            arrv(vec![
                                Value::Str("AddChecked".into()),
                                sng("Copy", place("5")),
                                sng("Const", int_const("4611686018427387904")),
                            ]),
                        ),
                    ),
                    // Assert(L7.1 == false)：溢出 → ConcErr::Overflow（真實 LLBC 形態）
                    sng(
                        "kind",
                        sng(
                            "Assert",
                            o(vec![(
                                "assert",
                                o(vec![
                                    ("cond", sng("Copy", place_fld("7", "1"))),
                                    ("expected", Value::Bool(false)),
                                ]),
                            )]),
                        ),
                    ),
                    // i := L7.0（checked tuple 值投影步進——步進認形容許 Field(0)）
                    sng(
                        "kind",
                        sng(
                            "Assign",
                            arrv(vec![
                                place("3"),
                                sng("Use", arrv(vec![sng("Copy", place_fld("7", "0"))])),
                            ]),
                        ),
                    ),
                    cont,
                ]),
            ),
        ];
        let loop_stmt = stmt(sng("Loop", o(vec![("statements", arrv(head_stmts))])));
        let fun = FunDeclRef {
            def_id: 0,
            name: "step_overflow".to_string(),
            full_path: vec!["step_overflow".to_string()],
            body_kind: BodyKind::Structured,
            raw: o(vec![
                ("signature", sng("inputs", arrv(vec![]))),
                (
                    "body",
                    sng(
                        "Structured",
                        o(vec![
                            (
                                "locals",
                                o(vec![
                                    ("arg_count", num("0")),
                                    ("locals", arrv(vec![Value::Null; 8])),
                                ]),
                            ),
                            (
                                "body",
                                sng(
                                    "statements",
                                    arrv(vec![
                                        assign("1", uconst("9223372036854775807")), // n := i64::MAX
                                        assign("3", uconst("0")),                   // i := 0
                                        loop_stmt,
                                        assign("0", uconst("0")),
                                        stmt(Value::Str("Return".into())),
                                    ]),
                                ),
                            ),
                        ]),
                    ),
                ),
            ]),
        };
        let d = decide_fun_in(&Value::Null, &[], &fun, &vec![]);
        // 唯一組合全溢出 → 無可認證組合 → Unknown 如實申報溢出
        // （部分組合溢出先會 Certified{excluded}；重點：溢出結果永不入認證）
        match d {
            Decision::Certified { excluded, .. } => {
                assert!(excluded >= 1, "若 Certified 必帶溢出排除：{d:?}");
            }
            Decision::Unknown { reason } => {
                assert!(reason.contains("溢出"), "應指向溢出：{reason}");
            }
            other => panic!("溢出應排除於認證範圍，得到 {other:?}"),
        }
    }

    #[test]
    fn c6_non_monotonic_degrades_honestly() {
        // 合成：cond Eq（模板外）→ invariant 推唔出，如實降級
        use crate::charon_llbc::{BodyKind, FunDeclRef as FDR, Value};
        let num = |s: &str| Value::Num(s.to_string());
        let sng = |k: &str, v: Value| Value::Obj(vec![(k.to_string(), v)]);
        let arrv = |xs: Vec<Value>| Value::Arr(xs);
        let o = |ps: Vec<(&str, Value)>| {
            Value::Obj(ps.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
        };
        let sgn = |v: &str| {
            sng(
                "Integer",
                sng(
                    "Signed",
                    arrv(vec![Value::Str("I64".into()), Value::Str(v.into())]),
                ),
            )
        };
        let int_const = |v: &str| {
            sng(
                "Value",
                arrv(vec![Value::Null, arrv(vec![sgn(v), Value::Null])]),
            )
        };
        let place = |idx: &str| o(vec![("kind", sng("Local", num(idx))), ("ty", Value::Null)]);
        let stmt = |k: Value| sng("kind", k);
        let ucopy = |src: &str| sng("Use", arrv(vec![sng("Copy", place(src))]));
        let uconst = |v: &str| sng("Use", arrv(vec![sng("Const", int_const(v))]));
        let assign =
            |dst: &str, rhs: Value| sng("kind", sng("Assign", arrv(vec![place(dst), rhs])));
        let binop = |op: &str, a: &str, b: &str| {
            sng(
                "BinaryOp",
                arrv(vec![
                    Value::Str(op.into()),
                    sng("Copy", place(a)),
                    sng("Copy", place(b)),
                ]),
            )
        };
        let arm = |sts: Vec<Value>| sng("statements", arrv(sts));
        let head_stmts = vec![
            assign("5", ucopy("3")),
            assign("6", ucopy("1")),
            assign("4", binop("Eq", "5", "6")),
            sng(
                "kind",
                sng(
                    "Switch",
                    o(vec![
                        (
                            "data",
                            o(vec![
                                ("scrutinee", sng("Value", sng("Move", place("4")))),
                                ("branches", arrv(vec![arrv(vec![int_const("0"), num("0")])])),
                                ("fallback", num("1")),
                            ]),
                        ),
                        (
                            "branches",
                            arrv(vec![
                                arm(vec![]),
                                arm(vec![
                                    assign("3", ucopy("3")),
                                    stmt(sng("Continue", num("0"))),
                                ]),
                            ]),
                        ),
                    ]),
                ),
            ),
        ];
        let fun = FDR {
            def_id: 0,
            name: "eq_loop".to_string(),
            full_path: vec!["eq_loop".to_string()],
            body_kind: BodyKind::Structured,
            raw: o(vec![
                ("signature", sng("inputs", arrv(vec![]))),
                (
                    "body",
                    sng(
                        "Structured",
                        o(vec![
                            (
                                "locals",
                                o(vec![
                                    ("arg_count", num("1")),
                                    ("locals", arrv(vec![Value::Null; 6])),
                                ]),
                            ),
                            (
                                "body",
                                sng(
                                    "statements",
                                    arrv(vec![
                                        assign("3", uconst("0")),
                                        stmt(sng(
                                            "Loop",
                                            o(vec![("statements", arrv(head_stmts))]),
                                        )),
                                        assign("0", ucopy("3")),
                                        stmt(Value::Str("Return".into())),
                                    ]),
                                ),
                            ),
                        ]),
                    ),
                ),
            ]),
        };
        let d = decide_fun_in(&Value::Null, &[], &fun, &vec![vec![3]]);
        let reason = d
            .reason()
            .unwrap_or_else(|| panic!("Eq loop 應 Unknown，得到 {d:?}"));
        assert!(
            reason.contains("唔支援") || reason.contains("模板"),
            "應指向模板外：{reason}"
        );
    }

    #[test]
    fn m3_report_while_sum_certified_with_combos() {
        // M3：certify_fun_report 端到端——loop 模板 → lean_theorem 標注、
        // 多值域 → 逐組合 ret（n=0 → 0、n=5 → 10）
        use crate::charon_llbc::Value;
        let fun = c6_while_sum_fdr();
        let d = certify_fun_report_in(&Value::Null, &[], &fun, &vec![vec![0, 5]]);
        assert_eq!(d.verdict, "Certified");
        assert_eq!(d.n_combos, 2);
        assert_eq!(d.n_certified, 2);
        assert_eq!(d.n_excluded, 0);
        assert_eq!(d.lean_theorem, Some("Polyrust.LoopInvariant.invSumF_spec"));
        let rets: Vec<Option<i64>> = {
            let mut v: Vec<Option<i64>> = d.combos.iter().map(|c| c.ret).collect();
            v.sort();
            v
        };
        assert_eq!(rets, vec![Some(0), Some(10)]);
        for c in &d.combos {
            assert!(!c.overflow_excluded);
            assert!(!c.sigma.is_empty());
        }
        // 無域組合（空域）：單一 default 組合——n 未定義 → 模擬如實降級
        let d2 = certify_fun_report_in(&Value::Null, &[], &fun, &[]);
        assert_eq!(d2.verdict, "Unknown");
        assert!(d2.reason.is_some());
    }

    #[test]
    fn m3_report_non_loop_has_no_lean_theorem_yet() {
        // 非迴圈直線函數：lean_theorem = None（Lean 對接屬後續 slice，如實標注）
        use crate::charon_llbc::{BodyKind, FunDeclRef, Value};
        let num = |s: &str| Value::Num(s.to_string());
        let sng = |k: &str, v: Value| Value::Obj(vec![(k.to_string(), v)]);
        let arrv = |xs: Vec<Value>| Value::Arr(xs);
        let o = |ps: Vec<(&str, Value)>| {
            Value::Obj(ps.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
        };
        let sgn = |v: &str| {
            sng(
                "Integer",
                sng(
                    "Signed",
                    arrv(vec![Value::Str("I64".into()), Value::Str(v.into())]),
                ),
            )
        };
        let int_const = |v: &str| {
            sng(
                "Value",
                arrv(vec![Value::Null, arrv(vec![sgn(v), Value::Null])]),
            )
        };
        let place = |idx: &str| o(vec![("kind", sng("Local", num(idx))), ("ty", Value::Null)]);
        let assign =
            |dst: &str, rhs: Value| sng("kind", sng("Assign", arrv(vec![place(dst), rhs])));
        let fun = FunDeclRef {
            def_id: 0,
            name: "const42".to_string(),
            full_path: vec!["const42".to_string()],
            body_kind: BodyKind::Structured,
            raw: o(vec![
                ("signature", sng("inputs", arrv(vec![]))),
                (
                    "body",
                    sng(
                        "Structured",
                        o(vec![
                            (
                                "locals",
                                o(vec![
                                    ("arg_count", num("0")),
                                    ("locals", arrv(vec![Value::Null; 3])),
                                ]),
                            ),
                            (
                                "body",
                                sng(
                                    "statements",
                                    arrv(vec![
                                        assign(
                                            "0",
                                            sng("Use", arrv(vec![sng("Const", int_const("42"))])),
                                        ),
                                        sng("kind", Value::Str("Return".into())),
                                    ]),
                                ),
                            ),
                        ]),
                    ),
                ),
            ]),
        };
        let d = certify_fun_report_in(&Value::Null, &[], &fun, &[]);
        assert_eq!(d.verdict, "Certified");
        assert_eq!(d.combos.len(), 1);
        assert_eq!(d.combos[0].ret, Some(42));
        assert_eq!(d.lean_theorem, None);
    }

    #[test]
    fn arg_domain_count_mismatch_is_unknown_not_panic() {
        let root = load("sqr");
        let d = decide_entry(&root, "sqr", &dom(&[1, 2])).unwrap();
        assert!(d.reason().map_or(false, |r| r.contains("參數域數量")));
    }

    #[test]
    fn l0_unfaithful_degrades_honestly() {
        // x = 10⁹ ⇒ x² = 10¹⁸；V=10⁹、d=2 ⇒ 界 ≈ 6·10³⁶ ≫ p ⇒ 如實降級
        let root = load("sqr");
        let d = decide_entry(&root, "sqr", &dom(&[1_000_000_000])).unwrap();
        assert!(
            d.reason().map_or(false, |r| r.contains("L0")),
            "得到 {:?}",
            d.reason()
        );
    }

    #[test]
    fn survey_reports_all_funs() {
        let root = load("sqr");
        let rows = survey(&root);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].1.starts_with("ValueTrace"), "{:?}", rows);
    }

    #[test]
    fn decisions_are_deterministic() {
        let root = load("sqr");
        let a = decide_entry(&root, "sqr", &dom(&[3])).unwrap();
        let b = decide_entry(&root, "sqr", &dom(&[3])).unwrap();
        assert_eq!(a, b);
    }
}
