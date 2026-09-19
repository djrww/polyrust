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

use crate::charon_llbc::{FunDeclRef, LlbcRoot};
use crate::fp::P;
use crate::frac::Frac;
use crate::minirust::mir_lower::{self, MirSystem};
use crate::poly::Poly;
use crate::polyir::{lower_fun, PirBinOp, PirBody, PirOperand, PirStmtKind, PirVerdict, Slot};
use crate::vanishing::{l0_prime_params_of, vanishing_poly, L0PrimeParams};
use std::collections::BTreeMap;

/// 值域組合總量上限（笛卡爾積）。
const PRODUCT_CAP: u64 = 1_000_000;
/// 單一值域集合大小上限。
const DOMAIN_CAP: usize = 64;

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
    /// SAT 且見證通過獨立認證。`ret` 為返回槽之（有符）值。
    Certified {
        ret: Option<i64>,
        n_vars: usize,
        n_eqs: usize,
        /// 溢出旗標被斷言為零（結果以「非溢出執行」為前提）。
        overflow_asserted: bool,
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

fn get_dom(domains: &[Vec<i64>], s: &Slot, slot_var: &BTreeMap<Slot, usize>) -> Result<Vec<i64>, String> {
    let id = slot_var
        .get(s)
        .ok_or_else(|| format!("槽 {:?} 域未定義（使用先於定義？）", s))?;
    domains
        .get(*id)
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
    domains: &[Vec<i64>],
    op: &PirOperand,
    slot_var: &BTreeMap<Slot, usize>,
) -> Result<Vec<i64>, String> {
    match op {
        PirOperand::Slot(s) => get_dom(domains, s, slot_var),
        PirOperand::Const(v) => Ok(vec![*v]),
    }
}

fn operand_poly(op: &PirOperand, slot_var: &BTreeMap<Slot, usize>, width: usize) -> Poly {
    match op {
        PirOperand::Slot(s) => {
            Poly::var(slot_var[s], Frac::ONE, width)
        }
        PirOperand::Const(v) => Poly::constant(Frac::from_i64(*v)),
    }
}

fn binop_apply(op: PirBinOp, x: i64, y: i64) -> i64 {
    match op {
        PirBinOp::Add => x.wrapping_add(y),
        PirBinOp::Sub => x.wrapping_sub(y),
        PirBinOp::Mul => x.wrapping_mul(y),
    }
}

/// 𝔽_p 表示 → 最小絕對值有符代表 → i64（L0′ 保真下必落 i64 範圍）。
fn fp_to_i64(v: &Frac) -> i64 {
    let raw = v.0 as i128;
    let signed = if (v.0 as u64) > P / 2 { raw - P as i128 } else { raw };
    signed as i64
}

/// PirBody → 多項式系統（值域 + 恆等式 + L0′ 門檻）。
pub fn encode_body(b: &PirBody, arg_domains: &[Vec<i64>]) -> Result<Encoded, String> {
    if arg_domains.len() != b.arg_count {
        return Err(format!(
            "參數域數量 {} ≠ arg_count {}",
            arg_domains.len(),
            b.arg_count
        ));
    }
    for (i, d) in arg_domains.iter().enumerate() {
        if d.is_empty() {
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
        let s = Slot { local: i + 1, field: None };
        let id = var_of(&s, &mut slot_var, &mut domains);
        domains[id] = d.clone();
    }

    let mut eqs: Vec<Poly> = Vec::new();
    let mut ret_var: Option<usize> = None;
    // Poly::var 需要上界；槽數 ≤ 參數 + 每語句產生數（寬鬆上界即可）
    let width = b.arg_count + b.stmts.len() * 2 + 8;

    for st in &b.stmts {
        match &st.kind {
            PirStmtKind::Const { val } => {
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = vec![*val];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::constant(Frac::from_i64(*val))));
                if st.dst.local == 0 && st.dst.field.is_none() {
                    ret_var = Some(d);
                }
            }
            PirStmtKind::Copy { src } => {
                let sd = get_dom(&domains, src, &slot_var)?;
                let d = var_of(&st.dst, &mut slot_var, &mut domains);
                domains[d] = sd;
                let s = slot_var[src];
                eqs.push(Poly::var(d, Frac::ONE, width).sub(&Poly::var(s, Frac::ONE, width)));
                if st.dst.local == 0 && st.dst.field.is_none() {
                    ret_var = Some(d);
                }
            }
            PirStmtKind::BinOp { op, a, b: bop, checked } => {
                let da = operand_dom(&domains, a, &slot_var)?;
                let db = operand_dom(&domains, bop, &slot_var)?;
                let dom = combine(&da, &db, |x, y| binop_apply(*op, x, y))?;
                let (pa, pb) = (operand_poly(a, &slot_var, width), operand_poly(bop, &slot_var, width));
                let rhs = match op {
                    PirBinOp::Add => pa.add(&pb),
                    PirBinOp::Sub => pa.sub(&pb),
                    PirBinOp::Mul => pa.mul(&pb),
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
                    if st.dst.local == 0 && st.dst.field.is_none() {
                        ret_var = Some(d);
                    }
                }
            }
            PirStmtKind::AssertFlagZero { flag } => {
                let id = var_of(flag, &mut slot_var, &mut domains);
                eqs.push(Poly::var(id, Frac::ONE, width));
            }
        }
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
    let v_max = domains.iter().flatten().map(|v| v.unsigned_abs()).max().unwrap_or(0);
    let mut l0 = L0PrimeParams { n_terms: 0, max_abs_coeff: 0, max_abs_value: v_max, degree: 0 };
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
pub fn decide_fun(fun: &FunDeclRef, arg_domains: &[Vec<i64>]) -> Decision {
    match lower_fun(fun) {
        PirVerdict::NoBody { kind } => Decision::Unknown {
            reason: format!("body 缺失（{:?}）", kind),
        },
        PirVerdict::Unknown { reason } => Decision::Unknown { reason },
        PirVerdict::ValueTrace(body) => decide_body(&body, arg_domains),
    }
}

/// 對已 lowering 之 body 做判定。
pub fn decide_body(body: &PirBody, arg_domains: &[Vec<i64>]) -> Decision {
    let enc = match encode_body(body, arg_domains) {
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
    }
}

/// 在 crate root 內按名稱找函數並判定（name = path 最尾段）。
pub fn decide_entry(root: &LlbcRoot, name: &str, arg_domains: &[Vec<i64>]) -> Option<Decision> {
    root.funs
        .iter()
        .find(|f| f.name == name)
        .map(|f| decide_fun(f, arg_domains))
}

/// 模組總覽：全部函數逐一報 lowering 判決（供 CLI `pir` 總表）。
pub fn survey(root: &LlbcRoot) -> Vec<(String, String)> {
    root.funs
        .iter()
        .map(|f| {
            let line = match lower_fun(f) {
                PirVerdict::ValueTrace(b) => format!(
                    "ValueTrace(stmts={} argc={} overflow_asserted={})",
                    b.stmts.len(),
                    b.arg_count,
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
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("fixture {name}: {e}"));
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
                Decision::Certified { ret, overflow_asserted, n_eqs, .. } => {
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
            Decision::Certified { ret, overflow_asserted, .. } => {
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
                assert!(ret == Some(4) || ret == Some(9) || ret == Some(16), "ret={ret:?}");
            }
            other => panic!("多值域應 Certified，得到 {:?}", other),
        }
    }

    #[test]
    fn unsupported_shapes_degrade_with_reasons() {
        let cases: [(&str, &str, &str); 8] = [
            ("max", "max", "Gt"),
            ("while_loop", "while_sum", "唔支援"),
            ("match_option", "match_option", "唔支援"),
            ("enum_option", "unwrap_or", "唔支援"),
            ("phase3__loop_sat", "main", "唔支援"),
            ("struct_point", "len", "Unknown"),
            ("borrow_immut", "borrow_immut", "投影"),
            ("async_simple", "async_add", "缺失 body"),
        ];
        for (file, fun, needle) in cases {
            let root = load(file);
            let d = decide_entry(&root, fun, &[])
                .unwrap_or_else(|| panic!("{file}: fun {fun} 不存在"));
            assert!(!d.is_certified(), "{file}:{fun} 不應 Certified");
            let hit = match &d {
                Decision::Unknown { reason } => reason.contains(needle) || needle == "Unknown",
                _ => false,
            };
            assert!(hit, "{file}:{fun} → {:?}（期望含 `{needle}`）", d.reason());
        }
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
        assert!(d.reason().map_or(false, |r| r.contains("L0")), "得到 {:?}", d.reason());
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
