// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! LLBC body → 多項式約束 lowering（C2 核心）。
//!
//! ## 語義表（邊界全部白紙黑字，唔畀靜默近似）
//!
//! | LLBC 片段 | lowering | 性質 |
//! |---|---|---|
//! | `+ - *`（含 Checked 版） | 精確（𝔽_p，p=2⁶¹−1） | 精確 |
//! | `Neg / Not` | 精確（0−x / 1−x） | 精確 |
//! | `Eq/Ne/Lt/Le/Gt/Ge` 比較 | 新鮮 bool 抽象變數（v²−v=0 鎖定） | **抽象**（marker `cmp-abstraction`） |
//! | `Div/Rem`、位運算 | 新鮮自由變數 | **抽象**（`divrem|bitop-abstraction`） |
//! | `Assert(*)`（overflow 等） | 記 marker 不加約束（panic-free 放寬） | **放寬**（`assert-abstracted`）→ C4 |
//! | `Switch`（bool/int-const 模式） | 路徑分裂 + 守衛 `scrut−c=0` + continuation | 精確（continuation 免 φ 合流）；多分支 fallback 守衛放寬（marker） |
//! | `Call` 同 crate | inline（fuel=2，C4 預設對齊），用盡 → dest 自由變數 | **有界**（`call-inline-fuel`） |
//! | `Loop` | body 行 1 次出loop；Break(0)/Continue(0)/行到尾 都記為 exit | **有界**（`loop-unroll-1`） |
//! | Storage/Borrowck/Nop/Drop 系 | no-op | — |
//!
//! 判定：每條最終路徑獨立 GB——理想含非零常數 ⇒ 該路徑不可行；全路徑不可行 ⇒ UNSAT；否則 SAT。
//! 抽象只朝 SAT 方向放寬（C2 語料 expected 全 SAT；UNSAT 判定由設計層西 contribution：bad.rs 走 rustc reject，
//! 係 v4 端ㇳ端 UNSAT 的第一類來源——C4 panic-freedom 接真 Assert 後先有語義級 UNSAT）。

use crate::charon_llbc::LlbcRoot;
use crate::frac::Frac;
use crate::llbc_body::*;
use crate::pipeline::{reduced_groebner_with_algo, select_groebner_algo_auto, GroebnerAlgo};
use crate::poly::{Order, Poly};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum V4Verdict {
    Sat,
    Unsat,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub struct V4Outcome {
    pub fun_name: String,
    pub verdict: V4Verdict,
    pub nvars: usize,
    pub npolys: usize,
    pub paths: usize,
    pub bounded_markers: Vec<String>,
    pub gb_stats: String,
}

#[derive(Clone)]
struct PathState {
    ssa: HashMap<(usize, Option<usize>), usize>,
    polys: Vec<Poly>,
}

type LExit<'s> = Option<&'s mut Vec<PathState>>;

struct Lower<'m> {
    nvars: usize,
    ty_of_var: Vec<ScalarTy>,
    local_tys: &'m [ScalarTy],
    markers: Vec<String>,
    funs: &'m [crate::charon_llbc::FunDeclRef],
    type_decls: &'m [crate::charon_llbc::Value],
    spawned: usize,
    final_states: Vec<PathState>,
}

const MAX_PATHS: usize = 128;
const CALL_FUEL: usize = 2;

impl<'m> Lower<'m> {
    fn fresh_var(&mut self, ty: ScalarTy) -> usize {
        let idx = self.nvars;
        self.nvars += 1;
        self.ty_of_var.push(ty);
        idx
    }
    fn var_poly(&self, idx: usize) -> Poly {
        Poly::var(idx, Frac::ONE, self.nvars)
    }
    fn const_poly(&self, v: i128) -> Poly {
        Poly::constant(Frac::from_i64(v as i64))
    }
    fn mark(&mut self, m: &str) {
        if !self.markers.iter().any(|x| x == m) {
            self.markers.push(m.to_string());
        }
    }
    fn clamp_polys(&self, out: &mut Vec<Poly>) {
        for (i, t) in self.ty_of_var.iter().enumerate() {
            if *t == ScalarTy::Bool {
                let v = Poly::var(i, Frac::ONE, self.nvars);
                out.push(v.mul(&v).sub(&v));
            }
        }
    }

    fn operand_poly(&mut self, st: &mut PathState, op: &Operand) -> Poly {
        match op {
            Operand::Const(ConstLit::Bool(b)) => self.const_poly(if *b { 1 } else { 0 }),
            Operand::Const(ConstLit::Int(v, _, _)) => self.const_poly(*v),
            Operand::Move(p) | Operand::Copy(p) => {
                let key = (p.local, p.proj);
                let idx = match st.ssa.get(&key) {
                    Some(i) => *i,
                    None => {
                        let ty = self.local_tys.get(p.local).copied().unwrap_or(ScalarTy::Opaque);
                        let v = self.fresh_var(ty);
                        st.ssa.insert(key, v);
                        v
                    }
                };
                self.var_poly(idx)
            }
        }
    }

    fn assign(&mut self, st: &mut PathState, dst: Place, expr: Poly) {
        let ty = self.local_tys.get(dst.local).copied().unwrap_or(ScalarTy::Opaque);
        let nv = self.fresh_var(ty);
        let eq = self.var_poly(nv).sub(&expr);
        st.polys.push(eq);
        st.ssa.insert((dst.local, dst.proj), nv);
    }

    fn lower_rvalue(&mut self, st: &mut PathState, rv: &RValue) -> Result<Poly, String> {
        use BinOpName::*;
        use RValue::*;
        Ok(match rv {
            Use(op) => self.operand_poly(st, op),
            UnaryOp(UnOpName::Neg, op) => {
                let p = self.operand_poly(st, op);
                self.const_poly(0).sub(&p)
            }
            UnaryOp(UnOpName::Not, op) => {
                let p = self.operand_poly(st, op);
                self.const_poly(1).sub(&p)
            }
            BinaryOp(op, l, r) => match op {
                Add | AddChecked => {
                    let a = self.operand_poly(st, l);
                    let b = self.operand_poly(st, r);
                    a.add(&b)
                }
                Sub | SubChecked => {
                    let a = self.operand_poly(st, l);
                    let b = self.operand_poly(st, r);
                    a.sub(&b)
                }
                Mul | MulChecked => {
                    let a = self.operand_poly(st, l);
                    let b = self.operand_poly(st, r);
                    a.mul(&b)
                }
                Eq | Ne | Lt | Le | Gt | Ge => {
                    let _ = (l, r);
                    self.mark("cmp-abstraction");
                    let v = self.fresh_var(ScalarTy::Bool);
                    self.var_poly(v)
                }
                Div | Rem => {
                    let _ = (l, r);
                    self.mark("divrem-abstraction");
                    let v = self.fresh_var(ScalarTy::Opaque);
                    self.var_poly(v)
                }
                BitAnd | BitOr | BitXor | Shl | Shr => {
                    let _ = (l, r);
                    self.mark("bitop-abstraction");
                    let v = self.fresh_var(ScalarTy::Opaque);
                    self.var_poly(v)
                }
            },
            Discriminant(_) => {
                self.mark("discriminant-abstraction");
                let v = self.fresh_var(ScalarTy::Signed(64));
                self.var_poly(v)
            }
            AggregateOpaque(_) => {
                self.mark("aggregate-abstraction");
                let v = self.fresh_var(ScalarTy::Opaque);
                self.var_poly(v)
            }
        })
    }

    fn spawn(&mut self, parent: &PathState) -> Result<PathState, String> {
        self.spawned += 1;
        if self.spawned > MAX_PATHS {
            return Err(format!("path-explosion > {MAX_PATHS}"));
        }
        Ok(parent.clone())
    }

    /// 主循環：由 stmts[from..] 一路降到路徑終結。
    /// - 終結去 final_states；Break(0)/Continue(0) 去 lexit（loop 出邊）；
    /// - Switch/Call/Loop 自己遞歸接管後續接續（continuation-passing，免 φ）。
    fn cont(
        &mut self,
        st: PathState,
        stmts: &[Statement],
        from: usize,
        call_depth: usize,
        lexit: &mut LExit,
    ) -> Result<(), String> {
        let mut st = st;
        let mut i = from;
        while i < stmts.len() {
            match &stmts[i].kind {
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_)
                | StmtKind::BorrowckOpaque(_) | StmtKind::NopLike(_) => {}
                StmtKind::UnwindResume => return Ok(()), // unwind 支路唔屬正常語義路徑
                StmtKind::Return => {
                    self.final_states.push(st);
                    return Ok(());
                }
                StmtKind::Assign(dst, rv) => {
                    let expr = self.lower_rvalue(&mut st, rv)?;
                    self.assign(&mut st, *dst, expr);
                }
                StmtKind::Assert(_) => self.mark("assert-abstracted"),
                StmtKind::Break(0) | StmtKind::Continue(0) => {
                    if let Some(buf) = lexit.as_deref_mut() {
                        buf.push(st);
                    } else {
                        // 頂層無 loop：理論上 structured LLBC 唔會出現；當路徑終結
                        self.final_states.push(st);
                    }
                    return Ok(());
                }
                StmtKind::Break(d) | StmtKind::Continue(d) => {
                    return Err(format!("nested loop control depth {d}（C3）"));
                }
                StmtKind::Call(ci) => {
                    if call_depth >= CALL_FUEL {
                        self.mark("call-inline-fuel");
                        let ty = self.local_tys.get(ci.dest.local).copied().unwrap_or(ScalarTy::Opaque);
                        let v = self.fresh_var(ty);
                        st.ssa.insert((ci.dest.local, ci.dest.proj), v);
                        i += 1;
                        continue;
                    }
                    let callee = self.funs.iter().find(|f| f.def_id == ci.fun_id as i64)
                        .ok_or_else(|| format!("call target fun_id {} not local", ci.fun_id))?;
                    if callee.body_kind != crate::charon_llbc::BodyKind::Structured {
                        return Err("call target body missing".into());
                    }
                    let cbody = parse_fun_body(&callee.raw, self.type_decls)
                        .map_err(|e| format!("callee parse: {e}"))?;
                    let mut cst = self.spawn(&st)?;
                    for (pi, arg) in ci.args.iter().enumerate() {
                        let av = self.operand_poly(&mut st, arg);
                        let ty = cbody.param_tys.get(pi).copied().unwrap_or(ScalarTy::Opaque);
                        let nv = self.fresh_var(ty);
                        cst.polys.push(self.var_poly(nv).sub(&av));
                        cst.ssa.insert((pi + 1, None), nv);
                    }
                    let saved = self.final_states.len();
                    let mut noexit: LExit = None;
                    self.cont(cst, &cbody.top.statements, 0, call_depth + 1, &mut noexit)?;
                    if noexit.is_some() && !noexit.as_ref().unwrap().is_empty() {
                        return Err("loop inside inlined call（C3）".into());
                    }
                    let finals = self.final_states.split_off(saved);
                    if finals.is_empty() {
                        return Ok(()); // callee 無正常返回 → caller 路徑死
                    }
                    for mut fs in finals {
                        let ret_idx = fs.ssa.get(&(0, None)).copied();
                        let dty = self.local_tys.get(ci.dest.local).copied().unwrap_or(ScalarTy::Opaque);
                        let nv = self.fresh_var(dty);
                        if let Some(ri) = ret_idx {
                            fs.polys.push(self.var_poly(nv).sub(&self.var_poly(ri)));
                        }
                        fs.ssa.insert((ci.dest.local, ci.dest.proj), nv);
                        self.cont(fs, stmts, i + 1, call_depth, lexit)?;
                    }
                    return Ok(());
                }
                StmtKind::Switch(sw) => {
                    let scrut = self.operand_poly(&mut st, &sw.scrutinee);
                    let mut next: Vec<PathState> = Vec::new();
                    for (pat, bid) in &sw.arms {
                        let blk = sw.blocks.get(*bid).ok_or("switch arm block id out of range")?;
                        let mut ast = self.spawn(&st)?;
                        let c = match pat {
                            ConstLit::Bool(b) => i128::from(*b as u8),
                            ConstLit::Int(v, _, _) => *v,
                        };
                        ast.polys.push(scrut.sub(&self.const_poly(c)));
                        self.into_arm(ast, blk, call_depth, lexit, &mut next)?;
                    }
                    if let Some(fb) = sw.blocks.get(sw.fallback) {
                        let mut fst = self.spawn(&st)?;
                        if sw.arms.len() == 1 {
                            if let ConstLit::Bool(b) = &sw.arms[0].0 {
                                fst.polys.push(scrut.sub(&self.const_poly(1 - i128::from(*b as u8))));
                            } else {
                                self.mark("switch-fallback-abstraction");
                            }
                        } else if !sw.arms.is_empty() {
                            self.mark("switch-fallback-abstraction");
                        }
                        self.into_arm(fst, fb, call_depth, lexit, &mut next)?;
                    }
                    let mut reborrow: LExit = lexit.as_deref_mut();
                    for ns in next {
                        self.cont(ns, stmts, i + 1, call_depth, &mut reborrow)?;
                    }
                    return Ok(());
                }
                StmtKind::Loop(lblk) => {
                    self.mark("loop-unroll-1");
                    let mut body_end: Vec<PathState> = Vec::new();
                    let mut broke: Vec<PathState> = Vec::new();
                    {
                        let mut lex: LExit = Some(&mut broke);
                        self.into_arm(st, lblk, call_depth, &mut lex, &mut body_end)?;
                    }
                    // body 行完（自然結尾）+ Break/Continue → 全部出 loop（有界）
                    let mut reborrow: LExit = lexit.as_deref_mut();
                    for ns in body_end.into_iter().chain(broke.into_iter()) {
                        self.cont(ns, stmts, i + 1, call_depth, &mut reborrow)?;
                    }
                    return Ok(());
                }
            }
            i += 1;
        }
        // block 走完無 Return：視為正常 path-end（fn 無 return expr / unit 返回）
        self.final_states.push(st);
        Ok(())
    }

    /// 進入一個 subtitle block（switch arm / loop body）：
    /// 路徑中途終結（Return/Break→lexit）更正確；自然行完 → next（交由上層續行）。
    fn into_arm(
        &mut self,
        st: PathState,
        blk: &Block,
        call_depth: usize,
        lexit: &mut LExit,
        next: &mut Vec<PathState>,
    ) -> Result<(), String> {
        let saved = self.final_states.len();
        self.cont(st, &blk.statements, 0, call_depth, lexit)?;
        next.extend(self.final_states.split_off(saved));
        Ok(())
    }
}

pub fn analyze_module(root: &LlbcRoot) -> Result<V4Outcome, String> {
    let type_decls: &[crate::charon_llbc::Value] = root
        .raw_translated
        .get("type_decls")
        .and_then(|t| t.as_arr())
        .unwrap_or(&[]);
    let fun = root
        .funs
        .iter()
        .find(|f| f.body_kind == crate::charon_llbc::BodyKind::Structured)
        .ok_or("no Structured fun in module")?;
    let body = parse_fun_body(&fun.raw, type_decls).map_err(|e| format!("body parse: {e}"))?;

    let mut lw = Lower {
        nvars: 0,
        ty_of_var: Vec::new(),
        local_tys: &body.local_tys,
        markers: Vec::new(),
        funs: &root.funs,
        type_decls,
        spawned: 1,
        final_states: Vec::new(),
    };
    lw.cont(PathState { ssa: HashMap::new(), polys: Vec::new() }, &body.top.statements, 0, 0, &mut None)?;

    if lw.final_states.is_empty() {
        return Ok(V4Outcome {
            fun_name: fun.name.clone(),
            verdict: V4Verdict::Unknown("no normal-termination path".into()),
            nvars: lw.nvars,
            npolys: 0,
            paths: 0,
            bounded_markers: lw.markers,
            gb_stats: "-".into(),
        });
    }

    // 短路：SAT 證明只需任一可行路徑 —— 第一條 SAT 即停（語義冇變，性能 10×+，
    // 法證：遞歸案例 debug-GB 慢，fib/pow 全路徑跑會超出測試預算）。
    let mut any_sat = false;
    let mut stats_txt = String::new();
    let mut total_polys = 0usize;
    for (pi, st) in lw.final_states.iter().enumerate() {
        let mut all = st.polys.clone();
        lw.clamp_polys(&mut all);
        total_polys += all.len();
        if all.is_empty() {
            // 無約束 = 自由解 = SAT（恒真），毋須 GB
            stats_txt.push_str(&format!("path{pi}: free "));
            any_sat = true;
            break;
        }
        let algo: GroebnerAlgo = select_groebner_algo_auto(&all, lw.nvars);
        let (gb, stats) = reduced_groebner_with_algo(&all, Order::GrevLex, algo);
        let infeasible = gb.iter().any(|g| matches!(g.is_constant(), Some(c) if c != Frac::ZERO));
        stats_txt.push_str(&format!("path{pi}: algo={:?} basis={} pairs={} ", algo, stats.basis_final, stats.pairs_considered));
        if !infeasible {
            any_sat = true;
            break; // 第一條可行路徑 → SAT，後面唔使查（disjunction 短路）
        }
    }
    Ok(V4Outcome {
        fun_name: fun.name.clone(),
        verdict: if any_sat { V4Verdict::Sat } else { V4Verdict::Unsat },
        nvars: lw.nvars,
        npolys: total_polys,
        paths: lw.final_states.len(),
        bounded_markers: lw.markers,
        gb_stats: stats_txt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verdict(fixture: &str) -> V4Outcome {
        let root = LlbcRoot::parse(fixture).unwrap();
        analyze_module(&root).unwrap()
    }

    #[test]
    fn sqr_sat() {
        let o = verdict(include_str!("../tests/charon_fixtures/sqr.llbc"));
        assert_eq!(o.verdict, V4Verdict::Sat, "{o:?}");
    }

    #[test]
    fn straight_and_cmp_categories_sat() {
        for f in [
            include_str!("../tests/charon_fixtures/add.llbc"),
            include_str!("../tests/charon_fixtures/is_even.llbc"),
        ] {
            assert_eq!(verdict(f).verdict, V4Verdict::Sat);
        }
    }

    #[test]
    fn switch_arms_sat() {
        for f in [
            include_str!("../tests/charon_fixtures/max.llbc"),
            include_str!("../tests/charon_fixtures/abs.llbc"),
        ] {
            assert_eq!(verdict(f).verdict, V4Verdict::Sat);
        }
    }

    #[test]
    fn recursion_inline_sat_with_marker() {
        for (n, f) in [
            ("fact", include_str!("../tests/charon_fixtures/fact.llbc")),
            ("fib", include_str!("../tests/charon_fixtures/fib.llbc")),
            ("pow", include_str!("../tests/charon_fixtures/pow.llbc")),
            ("gcd", include_str!("../tests/charon_fixtures/gcd.llbc")),
        ] {
            let o = verdict(f);
            assert_eq!(o.verdict, V4Verdict::Sat, "{n}: {o:?}");
            assert!(o.bounded_markers.iter().any(|m| m == "call-inline-fuel"), "{n}: {:?}", o.bounded_markers);
        }
    }

    #[test]
    fn sum_range_loop_sat_bounded() {
        let o = verdict(include_str!("../tests/charon_fixtures/sum_range.llbc"));
        assert_eq!(o.verdict, V4Verdict::Sat);
        assert!(o.bounded_markers.iter().any(|m| m == "loop-unroll-1"), "{:?}", o.bounded_markers);
    }

    #[test]
    fn while_loop_control_flow_no_misroute() {
        // loop 內 Switch arm 帶 Break：arm 續行唔准錯路由（詳見 lexit 通道設計）
        let o = verdict(include_str!("../tests/charon_fixtures/while_loop.llbc"));
        assert_eq!(o.verdict, V4Verdict::Sat, "{o:?}");
    }

    #[test]
    fn gb_detects_contradiction() {
        let p0 = Poly::var(0, Frac::ONE, 1).sub(&Poly::constant(Frac::ONE));
        let p1 = Poly::var(0, Frac::ONE, 1).sub(&Poly::constant(Frac::from_i64(2)));
        let (gb, _) = reduced_groebner_with_algo(&[p0, p1], Order::GrevLex, GroebnerAlgo::Classic);
        assert!(gb.iter().any(|g| matches!(g.is_constant(), Some(c) if c != Frac::ZERO)));
    }
}
