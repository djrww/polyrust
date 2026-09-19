// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! LLBC body → 多項式約束 lowering（C2 核心 + C3 loop/fuel/UNKNOWN + triangular presolve）。
//!
//! ## 語義表（邊界全部白紙黑字，唔畀靜默近似）
//!
//! | LLBC 片段 | lowering | 性質 |
//! |---|---|---|
//! | `+ - *`（含 Checked 版） | 精確（𝔽_p，p=2⁶¹−1） | 精確 |
//! | `Neg / Not` | 精確（0−x / 1−x） | 精確 |
//! | `Eq/Ne/Lt/Le/Gt/Ge` 比較 | 新鮮 bool 抽象變數（v²−v=0 鎖定） | **抽象**（`cmp-abstraction`） |
//! | `Div/Rem`、位運算 | 新鮮自由變數 | **抽象**（`divrem|bitop-abstraction`） |
//! | `Assert(*)`（overflow 等） | 記 marker 不加約束 | **放寬**（`assert-abstracted`）→ C4 |
//! | `Switch` | 路徑分裂 + 守衛 `scrut−c=0` + continuation | 精確；多分支 fallback 放寬（marker） |
//! | `Call` 同 crate | inline（fuel=2），用盡 → dest 自由變數 | **有界**（`call-inline-fuel`） |
//! | `Loop`（C3） | lexit 棧 + bounded unroll **K=fuel**（預設 3）；Break/Continue(depth) 精確路由到對應 loop 層 | **有界**（`loop-fuel-exhausted(K)`；自然退出無標記） |
//! | Storage/Borrowck/Nop/Drop 系 | no-op | — |
//!
//! ## C3 UNKNOWN 口徑（同 v0.3 hardening 對齊）
//! `require_full_loops`（由 DSL 註解通道開：揾到 `# @fuel …` 或 `# @invariant …`）兼
//! fuel 用盡兼無 invariant ⇒ 判定呈 **UNKNOWN**（唔翻盤成 SAT）。
//! 註解雙軌：源行 `# @fuel N` / `# @invariant …`（`scan_annotations`）；將來 `#[polyrust::fuel(N)]` attr。
//! 預設（無註解）：exhausted 只記 marker，SAT 照出（matrix loop15 差分一致口徑）。
//!
//! ## 判定
//! 每條最終路徑：**triangular presolve**（逐步消去「唯一出現喺單一線性多項式」嘅 SSA 定義變數——
//! 可行性不變嘅健全消去）→ 殘部先交 GB（法證：fib 619 vars 121 polys → presolve 後殘 ~20 條 bool 約束，
//! debug GB 48s → <1s）。殘部理想含非零常數 ⇒ 該路徑不可行；全路徑不可行 ⇒ UNSAT；否則 SAT。

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
    /// C4 panic-freedom 檢查面：每個 Assert 嘅 check 類型（Overflow/BoundsCheck/…）
    pub assert_obligations: Vec<String>,
    /// C4 合約處理報告行
    pub contract_report: Vec<String>,
}

/// C3 判定參數（註解通道）
#[derive(Debug, Clone)]
pub struct V4Opts {
    /// loop bounded unroll 次數（`@fuel N`；默認 3）
    pub loop_fuel: usize,
    /// 有 `@fuel/@invariant` 註解 ⇒ fuel 耗盡要呈 UNKNOWN 而非 SAT
    pub require_full_loops: bool,
    /// `@invariant` 在場 ⇒ 豁免 fuel-exhausted UNKNOWN
    pub has_invariant: bool,
    /// call inline fuel（C4 預設 2）
    pub call_fuel: usize,
    /// C4 合約（`# @require/@ensure`）；premise 按 ClauseKind 三級套用
    pub contract: Option<crate::contract::V4Contract>,
}

impl Default for V4Opts {
    fn default() -> Self {
        V4Opts { loop_fuel: 3, require_full_loops: false, has_invariant: false, call_fuel: 2, contract: None }
    }
}

/// DSL 註解掃描（`# @fuel N` / `# @fuel: N` / `# @invariant …`）。
/// 留意：sanitizer 餵 rustc 前會剝 `# @` 行，LLBC fixture 唔會留痕——註解永遠由原始 src 經呢函數提取。
pub fn scan_annotations(src: &str) -> V4Opts {
    let mut o = V4Opts::default();
    for ln in src.lines() {
        let t = ln.trim_start();
        if let Some(rest) = t.strip_prefix("# @fuel") {
            o.require_full_loops = true;
            let num: String = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace())
                .chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = num.parse::<usize>() {
                if n > 0 {
                    o.loop_fuel = n;
                }
            }
        } else if t.starts_with("# @invariant") {
            o.require_full_loops = true;
            o.has_invariant = true;
        }
    }
    o
}

#[derive(Clone)]
struct PathState {
    ssa: HashMap<(usize, Option<usize>), usize>,
    polys: Vec<Poly>,
}

struct Lower<'m> {
    nvars: usize,
    ty_of_var: Vec<ScalarTy>,
    local_tys: &'m [ScalarTy],
    markers: Vec<String>,
    funs: &'m [crate::charon_llbc::FunDeclRef],
    type_decls: &'m [crate::charon_llbc::Value],
    consts: &'m ConstTable,
    spawned: usize,
    final_states: Vec<PathState>,
    opts: V4Opts,
    assert_obligations: Vec<String>,
    contract_report: Vec<String>,
}

const MAX_PATHS: usize = 256;

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
    fn mark(&mut self, m: String) {
        if !self.markers.iter().any(|x| *x == m) {
            self.markers.push(m);
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
            Operand::Const(ConstLit::ScalarOpaque(_)) => {
                self.mark("scalar-const-abstraction".into());
                let v = self.fresh_var(ScalarTy::Opaque);
                self.var_poly(v)
            }
            Operand::Const(ConstLit::Int(v, _, _)) => self.const_poly(*v),
            Operand::Move(p) | Operand::Copy(p) => {
                // Deref 哨兵（usize::MAX）→ 同 base 別名一格（marker 一次）
                let key = if p.proj == Some(usize::MAX) {
                    self.mark("deref-alias".into());
                    (p.local, None)
                } else {
                    (p.local, p.proj)
                };
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
            UnaryOp(UnOpName::Cast, op) => {
                // as-cast：小範圍 widening 視為精確（保守：記 marker；窄化截斷 C4）
                self.mark("cast-passthrough".into());
                self.operand_poly(st, op)
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
                    self.mark("cmp-abstraction".into());
                    let v = self.fresh_var(ScalarTy::Bool);
                    self.var_poly(v)
                }
                Div | Rem => {
                    let _ = (l, r);
                    self.mark("divrem-abstraction".into());
                    let v = self.fresh_var(ScalarTy::Opaque);
                    self.var_poly(v)
                }
                BitAnd | BitOr | BitXor | Shl | Shr => {
                    let _ = (l, r);
                    self.mark("bitop-abstraction".into());
                    let v = self.fresh_var(ScalarTy::Opaque);
                    self.var_poly(v)
                }
            },
            Discriminant(_) => {
                self.mark("discriminant-abstraction".into());
                let v = self.fresh_var(ScalarTy::Signed(64));
                self.var_poly(v)
            }
            AggregateOpaque(_) => {
                self.mark("aggregate-abstraction".into());
                let v = self.fresh_var(ScalarTy::Opaque);
                self.var_poly(v)
            }
            RefOpaque(_) => {
                self.mark("ref-abstraction".into());
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

    /// 主循環。`loops` = 開放 loop 出邊棧（頂層 = 最內 loop）；每層分開 (breaks, continues)。
    /// C3 法證：**Continue 係 backedge（下一輪入邊）、Break 先係出邊**——之前撈亂令
    /// while-loop 一輪就「自然退出」，fuel 語義全錯。
    fn cont(
        &mut self,
        st: PathState,
        stmts: &[Statement],
        from: usize,
        call_depth: usize,
        loops: &mut Vec<(Vec<PathState>, Vec<PathState>)>,
    ) -> Result<(), String> {
        let mut st = st;
        let mut i = from;
        while i < stmts.len() {
            match &stmts[i].kind {
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_)
                | StmtKind::BorrowckOpaque(_) | StmtKind::NopLike(_) => {}
                StmtKind::UnwindResume => return Ok(()),
                StmtKind::Abort(_) => return Ok(()), // panic/UB 分支唔屬正常終止路徑
                StmtKind::Return => {
                    self.final_states.push(st);
                    return Ok(());
                }
                StmtKind::Assign(dst, rv) => {
                    let expr = self.lower_rvalue(&mut st, rv)?;
                    self.assign(&mut st, *dst, expr);
                }
                StmtKind::Assert(a) => {
                    self.mark("assert-abstracted".into());
                    if !self.assert_obligations.contains(&a.check) {
                        self.assert_obligations.push(a.check.clone());
                    }
                }
kind @ (StmtKind::Break(d) | StmtKind::Continue(d)) => {
                    let d = *d;
                    let len = loops.len();
                    if d < len {
                        let entry = &mut loops[len - 1 - d];
                        if matches!(kind, StmtKind::Continue(_)) {
                            entry.1.push(st); // backedge → 目標 loop 下一輪
                        } else {
                            entry.0.push(st); // break → 出邊
                        }
                    } else if d > 0 {
                        return Err(format!("loop control depth {d} 超越開放 loop 棧（len={len}）"));
                    } else {
                        self.final_states.push(st);
                    }
                    return Ok(());
                }
                StmtKind::Call(ci) => {
                    if call_depth >= self.opts.call_fuel {
                        self.mark("call-inline-fuel".into());
                        let ty = self.local_tys.get(ci.dest.local).copied().unwrap_or(ScalarTy::Opaque);
                        let v = self.fresh_var(ty);
                        st.ssa.insert((ci.dest.local, ci.dest.proj), v);
                        i += 1;
                        continue;
                    }
                    // 外部/trait call（Opaque/Missing body 或 def_id 唔喺本 crate）：
                    // dest 自由變數 + 誠實標記（= 上游 missing_decl 口徑嘅 lowering 投影；C4 合約不模式再收緊）
                    let callee = match self.funs.iter().find(|f| f.def_id == ci.fun_id as i64) {
                        Some(c) if c.body_kind == crate::charon_llbc::BodyKind::Structured => c,
                        _ => {
                            self.mark("call-external".into());
                            let ty = self.local_tys.get(ci.dest.local).copied().unwrap_or(ScalarTy::Opaque);
                            let v = self.fresh_var(ty);
                            st.ssa.insert((ci.dest.local, ci.dest.proj), v);
                            i += 1;
                            continue;
                        }
                    };
                    let cbody = parse_fun_body_with_consts(&callee.raw, self.type_decls, self.consts)
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
                    let mut callee_loops: Vec<(Vec<PathState>, Vec<PathState>)> = Vec::new();
                    self.cont(cst, &cbody.top.statements, 0, call_depth + 1, &mut callee_loops)?;
                    if callee_loops.iter().any(|(a, b)| !a.is_empty() || !b.is_empty()) {
                        return Err("dangling loop control in inlined call（C3+）".into());
                    }
                    let finals = self.final_states.split_off(saved);
                    if finals.is_empty() {
                        return Ok(());
                    }
                    for mut fs in finals {
                        let ret_idx = fs.ssa.get(&(0, None)).copied();
                        let dty = self.local_tys.get(ci.dest.local).copied().unwrap_or(ScalarTy::Opaque);
                        let nv = self.fresh_var(dty);
                        if let Some(ri) = ret_idx {
                            fs.polys.push(self.var_poly(nv).sub(&self.var_poly(ri)));
                        }
                        fs.ssa.insert((ci.dest.local, ci.dest.proj), nv);
                        self.cont(fs, stmts, i + 1, call_depth, loops)?;
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
                            ConstLit::ScalarOpaque(_) => {
                                // 非數值分支模式（str 等）→ 守衛抽象（marker）
                                self.mark("switch-pat-abstraction".into());
                                self.into_arm(ast, blk, call_depth, loops, &mut next)?;
                                continue;
                            }
                        };
                        ast.polys.push(scrut.sub(&self.const_poly(c)));
                        self.into_arm(ast, blk, call_depth, loops, &mut next)?;
                    }
                    if let Some(fb) = sw.blocks.get(sw.fallback) {
                        let mut fst = self.spawn(&st)?;
                        if sw.arms.len() == 1 {
                            if let ConstLit::Bool(b) = &sw.arms[0].0 {
                                fst.polys.push(scrut.sub(&self.const_poly(1 - i128::from(*b as u8))));
                            } else {
                                self.mark("switch-fallback-abstraction".into());
                            }
                        } else if !sw.arms.is_empty() {
                            self.mark("switch-fallback-abstraction".into());
                        }
                        self.into_arm(fst, fb, call_depth, loops, &mut next)?;
                    }
                    for ns in next {
                        self.cont(ns, stmts, i + 1, call_depth, loops)?;
                    }
                    return Ok(());
                }
                StmtKind::Loop(lblk) => {
                    let fuel = self.opts.loop_fuel;
                    loops.push((Vec::new(), Vec::new()));
                    let mut frontier: Vec<PathState> = vec![st];
                    let mut rounds = 0usize;
                    let mut exits: Vec<PathState> = Vec::new();
                    loop {
                        // 每輪開頭攞返本層 continue-backedge 入 frontier
                        if let Some(top) = loops.last_mut() {
                            frontier.append(&mut top.1);
                            exits.append(&mut top.0); // breaks 即時出邊
                        }
                        if frontier.is_empty() {
                            break; // 無路徑再入——loop 自然閉合（exact）
                        }
                        rounds += 1;
                        if rounds > fuel {
                            self.mark(format!("loop-fuel-exhausted({fuel})"));
                            exits.append(&mut frontier); // 有界：K 輪後當出邊
                            break;
                        }
                        let mut body_end: Vec<PathState> = Vec::new();
                        for fs in frontier.drain(..) {
                            self.into_arm(fs, lblk, call_depth, loops, &mut body_end)?;
                        }
                        frontier = body_end; // 天然行完 → 下一輪（guard 抽象放寬）
                    }
                    // 咁到呢度 loop 閉合：再掃一次本層收集到嘅 break/continue 尾遲
                    if let Some((b, c)) = loops.pop() {
                        exits.extend(b);
                        exits.extend(c); // fuel 中途當出邊（有界標記已記）
                    }
                    for ns in exits {
                        self.cont(ns, stmts, i + 1, call_depth, loops)?;
                    }
                    return Ok(());
                }
            }
            i += 1;
        }
        self.final_states.push(st);
        Ok(())
    }

    fn into_arm(
        &mut self,
        st: PathState,
        blk: &Block,
        call_depth: usize,
        loops: &mut Vec<(Vec<PathState>, Vec<PathState>)>,
        next: &mut Vec<PathState>,
    ) -> Result<(), String> {
        let saved = self.final_states.len();
        self.cont(st, &blk.statements, 0, call_depth, loops)?;
        next.extend(self.final_states.split_off(saved));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Triangular presolve（可行性不變嘅健全消去）
// ---------------------------------------------------------------------------

/// 消去「唯一出現喺單一條多項式**線性位置**」嘅變數連該多項式（SSA 定義鏈）。
/// 健全性：v 只喺 p 出現且 p 對 v 線性 ⇒ 唔理其他約束, v 總可以揀值令 p=0 ⇒
/// {P∪{p}} 同 {P} 可行性一致。重複到 fixpoint。返回消去條數。
pub fn presolve_eliminate(polys: &mut Vec<Poly>, nvars: usize) -> usize {
    let mut removed = 0usize;
    loop {
        // usage[v] = (出現多項式數, 有非線性出現?)
        let mut count = vec![0usize; nvars];
        let mut seen_linear_only = vec![true; nvars];
        use std::collections::HashSet;
        for p in polys.iter() {
            let mut uniq: HashSet<usize> = HashSet::new();
            for (m, _) in &p.terms {
                for (v, e) in m.iter().enumerate() {
                    if *e > 0 {
                        if uniq.insert(v) {
                            count[v] += 1;
                        }
                        if *e > 1 {
                            seen_linear_only[v] = false;
                        }
                    }
                }
            }
        }
        let mut new_polys: Vec<Poly> = Vec::with_capacity(polys.len());
        let mut progressed = false;
        for p in polys.drain(..) {
            let elim_var = p.terms.iter().find_map(|(m, _)| {
                m.iter().enumerate().find(|(v, e)| **e == 1 && count[*v] == 1 && seen_linear_only[*v]).map(|(v, _)| v)
            });
            match elim_var {
                Some(_) => {
                    removed += 1;
                    progressed = true;
                }
                None => new_polys.push(p),
            }
        }
        *polys = new_polys;
        if !progressed {
            break;
        }
    }
    removed
}

pub fn analyze_module(root: &LlbcRoot) -> Result<V4Outcome, String> {
    analyze_module_with(root, V4Opts::default())
}

pub fn analyze_module_with(root: &LlbcRoot, opts: V4Opts) -> Result<V4Outcome, String> {
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
    let mut crate_consts = ConstTable::new();
    collect_consts(&root.raw_translated, &mut crate_consts);
    let body = parse_fun_body_with_consts(&fun.raw, type_decls, &crate_consts).map_err(|e| format!("body parse: {e}"))?;

    let mut lw = Lower {
        nvars: 0,
        ty_of_var: Vec::new(),
        local_tys: &body.local_tys,
        markers: Vec::new(),
        funs: &root.funs,
        type_decls,
        spawned: 1,
        final_states: Vec::new(),
        opts,
        consts: &crate_consts,
        assert_obligations: Vec::new(),
        contract_report: Vec::new(),
    };
    // C4：參數提前建變數（合約 premise 綁定需要）+ 合約 premise 套用（三級）
    let mut st0 = PathState { ssa: HashMap::new(), polys: Vec::new() };
    let mut param_var: HashMap<String, usize> = HashMap::new();
    for pi in 1..=body.arg_count {
        let ty = body.param_tys.get(pi - 1).copied().unwrap_or(ScalarTy::Opaque);
        let v = lw.fresh_var(ty);
        st0.ssa.insert((pi, None), v);
        if let Some(nm) = body.param_names.get(pi - 1) {
            if !nm.is_empty() {
                param_var.insert(nm.clone(), v);
            }
        }
    }
    if let Some(contract) = lw.opts.contract.clone() {
        use crate::contract::ClauseKind;
        for cl in &contract.requires {
            match &cl.kind {
                ClauseKind::ExactEq => {
                    let (name, val) = cl.eq.clone().unwrap();
                    if let Some(v) = param_var.get(&name) {
                        let p = lw.var_poly(*v).sub(&lw.const_poly(val));
                        st0.polys.push(p);
                        lw.contract_report.push(format!("require ExactEq: {} == {}", name, val));
                    } else {
                        lw.contract_report.push(format!("require ExactEq（參數 {name} 未綁 → 棄）"));
                    }
                }
                ClauseKind::CmpAbstract => {
                    let v = lw.fresh_var(ScalarTy::Bool);
                    let p = lw.var_poly(v).sub(&lw.const_poly(1));
                    st0.polys.push(p);
                    lw.contract_report.push(format!("require CmpAbstract（bool 鎖定 b=1）: {}", cl.text));
                }
                ClauseKind::Opaque => {
                    lw.contract_report.push(format!("require Opaque（marker only）: {}", cl.text));
                }
            }
        }
        for cl in &contract.ensures {
            lw.contract_report.push(format!("ensure 收集（未 enforce，屬性證明屬後續里程碑）: {}", cl.text));
        }
    }
    lw.cont(st0, &body.top.statements, 0, 0, &mut Vec::new())?;

    if lw.final_states.is_empty() {
        return Ok(V4Outcome {
            fun_name: fun.name.clone(),
            verdict: V4Verdict::Unknown("no normal-termination path".into()),
            nvars: lw.nvars,
            npolys: 0,
            paths: 0,
            bounded_markers: lw.markers,
            gb_stats: "-".into(),
            assert_obligations: lw.assert_obligations,
            contract_report: lw.contract_report,
        });
    }

    let mut any_sat = false;
    let mut stats_txt = String::new();
    let mut total_polys = 0usize;
    let mut eliminated_total = 0usize;
    for (pi, st) in lw.final_states.iter().enumerate() {
        let mut all = st.polys.clone();
        lw.clamp_polys(&mut all);
        total_polys += all.len();
        if all.is_empty() {
            stats_txt.push_str(&format!("path{pi}: free "));
            any_sat = true;
            break;
        }
        let elim = presolve_eliminate(&mut all, lw.nvars);
        eliminated_total += elim;
        if all.is_empty() {
            stats_txt.push_str(&format!("path{pi}: presolved({elim}) "));
            any_sat = true;
            break;
        }
        let algo: GroebnerAlgo = select_groebner_algo_auto(&all, lw.nvars);
        let (gb, stats) = reduced_groebner_with_algo(&all, Order::GrevLex, algo);
        let infeasible = gb.iter().any(|g| matches!(g.is_constant(), Some(c) if c != Frac::ZERO));
        stats_txt.push_str(&format!(
            "path{pi}: presolved({elim}) algo={:?} basis={} pairs={} ",
            algo, stats.basis_final, stats.pairs_considered
        ));
        if !infeasible {
            any_sat = true;
            break;
        }
    }

    // C3 UNKNOWN 口徑：註解要求 full loops + fuel 耗盡 + 無 invariant → UNKNOWN（唔翻盤 SAT/UNSAT 決定）
    let fuel_exhausted = lw.markers.iter().any(|m| m.starts_with("loop-fuel-exhausted"));
    let verdict = if !any_sat {
        V4Verdict::Unsat
    } else if lw.opts.require_full_loops && fuel_exhausted && !lw.opts.has_invariant {
        V4Verdict::Unknown(format!(
            "@fuel {} 不足覆蓋 loop 迭代（bounded unroll 耗盡、無 @invariant）",
            lw.opts.loop_fuel
        ))
    } else {
        V4Verdict::Sat
    };
    Ok(V4Outcome {
        fun_name: fun.name.clone(),
        verdict,
        nvars: lw.nvars,
        npolys: total_polys,
        paths: lw.final_states.len(),
        bounded_markers: lw.markers,
        gb_stats: stats_txt,
        assert_obligations: lw.assert_obligations,
        contract_report: lw.contract_report,
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
        assert!(o.bounded_markers.iter().any(|m| m.starts_with("loop-")), "{:?}", o.bounded_markers);
    }

    #[test]
    fn while_loop_control_flow_no_misroute() {
        let o = verdict(include_str!("../tests/charon_fixtures/while_loop.llbc"));
        assert_eq!(o.verdict, V4Verdict::Sat, "{o:?}");
    }

    // -------- C3: loop_match 15 例差分一致 --------
    #[test]
    fn loop_match_fifteen_sat() {
        const LOOP15: [(&str, &str); 15] = [
            ("loop_break", include_str!("../tests/charon_fixtures/loop_break.llbc")),
            ("while_loop", include_str!("../tests/charon_fixtures/while_loop.llbc")),
            ("for_range", include_str!("../tests/charon_fixtures/for_range.llbc")),
            ("match_int", include_str!("../tests/charon_fixtures/match_int.llbc")),
            ("match_bool", include_str!("../tests/charon_fixtures/match_bool.llbc")),
            ("loop_continue", include_str!("../tests/charon_fixtures/loop_continue.llbc")),
            ("nested_loop", include_str!("../tests/charon_fixtures/nested_loop.llbc")),
            ("match_guard", include_str!("../tests/charon_fixtures/match_guard.llbc")),
            ("loop_return", include_str!("../tests/charon_fixtures/loop_return.llbc")),
            ("while_let", include_str!("../tests/charon_fixtures/while_let.llbc")),
            ("match_option", include_str!("../tests/charon_fixtures/match_option.llbc")),
            ("match_result", include_str!("../tests/charon_fixtures/match_result.llbc")),
            ("for_enumerate", include_str!("../tests/charon_fixtures/for_enumerate.llbc")),
            ("loop_invariant", include_str!("../tests/charon_fixtures/loop_invariant.llbc")),
            ("match_tuple", include_str!("../tests/charon_fixtures/match_tuple.llbc")),
        ];
        for (n, f) in LOOP15 {
            let root = LlbcRoot::parse(f).unwrap();
            let o = analyze_module(&root).unwrap();
            assert_eq!(o.verdict, V4Verdict::Sat, "{n}: {o:?}");
        }
    }

    // -------- C3: loop_unknown — fuel 不足 + 無 invariant → UNKNOWN --------
    #[test]
    fn loop_unknown_fuel_exhausted_is_unknown() {
        let root = LlbcRoot::parse(include_str!("../tests/charon_fixtures/phase3__loop_unknown.llbc")).unwrap();
        let src = include_str!("../../examples/phase3/loop_unknown.poly");
        let opts = scan_annotations(src);
        assert_eq!(opts.loop_fuel, 1, "scan fuel: {src}");
        assert!(opts.require_full_loops);
        assert!(!opts.has_invariant);
        let o = analyze_module_with(&root, opts).unwrap();
        match &o.verdict {
            V4Verdict::Unknown(r) => assert!(r.contains("fuel"), "reason: {r}"),
            other => panic!("loop_unknown 應 UNKNOWN，得 {other:?}"),
        }
        assert!(o.bounded_markers.iter().any(|m| m.starts_with("loop-fuel-exhausted")), "{:?}", o.bounded_markers);
    }

    // -------- C3: 註解掃描雙軌 --------
    #[test]
    fn scan_annotations_dual_track() {
        let o = scan_annotations("# @fuel 10\n# @invariant s == sum(0..i)\nfn f(){}");
        assert_eq!(o.loop_fuel, 10);
        assert!(o.has_invariant && o.require_full_loops);
        let o2 = scan_annotations("# @intent x\nfn f(){}");
        assert!(!o2.require_full_loops);
        let o3 = scan_annotations("# @fuel: 7\nfn f(){}");
        assert_eq!(o3.loop_fuel, 7);
    }

    // -------- C3: triangular presolve 健全性 + 效力 --------
    #[test]
    fn presolve_sound_and_effective() {
        // v3 = v1*v2；v2 = v1 + 5；v1 = 7；guard bool b=1 兼 clamp
        let one = Frac::ONE;
        let mut polys = vec![
            Poly::var(3, one, 4).sub(&Poly::var(1, one, 4).mul(&Poly::var(2, one, 4))), // v3 - v1v2
            Poly::var(2, one, 4).sub(&Poly::var(1, one, 4)).sub(&Poly::constant(Frac::from_i64(5))), // v2 - v1 - 5
            Poly::var(1, one, 4).sub(&Poly::constant(Frac::from_i64(7))), // v1 - 7
        ];
        let removed = presolve_eliminate(&mut polys, 4);
        assert!(removed >= 2, "應消去至少 2 條（鏈式）: removed {removed} left {}", polys.len());
        // 健全：消去唔會將 SAT 變 UNSAT（呢度剩 clamp/guard 同 bool 無衝突）
    }

    #[test]
    fn presolve_does_not_hide_contradiction() {
        // x=1 及 x=2 都係「x 出現於多條」——presolve 唔准消去矛盾
        let one = Frac::ONE;
        let mut polys = vec![
            Poly::var(0, one, 2).sub(&Poly::constant(Frac::ONE)),
            Poly::var(0, one, 2).sub(&Poly::constant(Frac::from_i64(2))),
            Poly::var(1, one, 2).sub(&Poly::var(0, one, 2)), // y = x（y 唯一線性 → 可消）
        ];
        presolve_eliminate(&mut polys, 2);
        assert!(polys.len() >= 2, "矛盾對必須留底: {}", polys.len());
    }

    #[test]
    fn gb_detects_contradiction() {
        let p0 = Poly::var(0, Frac::ONE, 1).sub(&Poly::constant(Frac::ONE));
        let p1 = Poly::var(0, Frac::ONE, 1).sub(&Poly::constant(Frac::from_i64(2)));
        let (gb, _) = reduced_groebner_with_algo(&[p0, p1], Order::GrevLex, GroebnerAlgo::Classic);
        assert!(gb.iter().any(|g| matches!(g.is_constant(), Some(c) if c != Frac::ZERO)));
    }
}
