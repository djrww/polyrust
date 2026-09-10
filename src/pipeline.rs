//! 管線：解析 → 宏展開 → 代數約束 → CDCL(T) 迴圈（子句 + Gröbner 理論）→
//! Buchberger 化簡與判定 → 布爾求解（見證 σ）→ QAP 驗證 → 代碼生成。

use crate::cdcl::{self, CdclStats};
use crate::codegen::{roundtrip_check, CodeGenConfig};
use crate::frac::Frac;
use crate::fp::Fp;
use crate::groebner::{field_polys, reduced_groebner, solve_boolean, GroebnerStats, Strategy};
use crate::minirust::ast::{Program, Type};
use crate::minirust::checker::{check_program, Derivation};
use crate::minirust::constraints::{gen_constraints, to_r1cs, System};
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;
use crate::poly::{Order, Poly};
use crate::qap::{qap_from_r1cs, Qap, R1cs};
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Debug, Default)]
pub struct PipelineResult {
    pub name: String,
    pub n_vars: usize,
    pub n_polys: usize,          // 生成元數（含域多項式與子句多項式）
    pub n_clauses: usize,
    pub cdcl_rounds: usize,
    pub cdcl_stats: CdclStats,
    pub learned_clauses: Vec<Vec<cdcl::Lit>>,
    pub gb_stats: GroebnerStats,
    pub reduced_basis: Vec<Poly>,
    pub is_unsat: bool,
    pub sigma: Option<Vec<Frac>>,
    pub node_types: HashMap<usize, Type>,
    pub arm_choice: HashMap<usize, usize>,
    pub checker_ok: bool,
    pub checker_msg: String,
    pub agrees: bool,
    pub r1cs_constraints: usize,
    pub r1cs_wires: usize,
    pub qap_max_degree: usize,
    pub qap_verified: Option<bool>,
    pub qap_tamper_rejected: Option<bool>,
    pub generated_code: Option<String>,
    pub generated_file: Option<String>,
    pub rustc_compiles: Option<bool>,
    pub expansion_log: Vec<String>,
}

/// 子句 → 多項式：C = ℓ₁∨…∨ℓ_k ⇔ ∏(¬ℓᵢ 的真值指示) = 0
pub fn clause_to_poly(clause: &[cdcl::Lit], nvars: usize) -> Poly {
    let mut p = Poly::constant(Frac::ONE);
    for &l in clause {
        let v = cdcl::lit_var(l) as usize;
        let factor = if cdcl::lit_positive(l) {
            // ℓ = x：取 1−x
            Poly::constant(Frac::ONE).sub(&Poly::var(v, Frac::ONE, nvars))
        } else {
            Poly::var(v, Frac::ONE, nvars)
        };
        p = p.mul(&factor);
    }
    p
}

/// 完整管線。
pub fn run_pipeline(name: &str, source: &str, do_codegen: bool) -> Result<PipelineResult, String> {
    let mut res = PipelineResult { name: name.to_string(), ..Default::default() };
    let mut t0 = std::time::Instant::now();
    macro_rules! stage { ($m:expr) => {
        if std::env::var("PL_DBG").is_ok() {
            eprintln!("[{}] {:?} {}", name, t0.elapsed(), $m);
            t0 = std::time::Instant::now();
        }
    } }

    // ── S1 解析 ──
    let p: Program = Parser::parse_program(source)?;

    // ── S2 宏展開（記錄各臂展開文本）──
    let mut exp = Expander::new(p.macros.clone(), p.next_id);

    // ── S3 ground truth（直接檢查器）──
    let checker: Result<Vec<Derivation>, String> = check_program(&p, &mut exp);
    match &checker {
        Ok(ds) => {
            res.checker_ok = true;
            res.checker_msg = format!(
                "接受（{} 條推導；臂選擇 {:?}；main 型別 {}）",
                ds.len(),
                ds.first().map(|d| d.arm_choice.clone()).unwrap_or_default(),
                ds.first().map(|d| d.ty.name()).unwrap_or("?")
            );
        }
        Err(e) => {
            res.checker_ok = false;
            res.checker_msg = format!("拒絕：{}", e);
        }
    }

    // 展開記錄
    for ((node, arm), text) in &exp.memo_text {
        res.expansion_log.push(format!("invoke#{} 臂{} → {}", node, arm + 1, text));
    }

    stage!("S3 checker");
    // ── S4 代數約束 ──
    let sys: System = gen_constraints(&p, &mut exp)?;
    stage!("S4 constraints");
    res.n_vars = sys.nvars;

    // ── S5 CDCL(T) 迴圈 ──
    // 子句變量（arm bits + borrow bits）壓縮編號
    let mut clause_vars: Vec<usize> = vec![];
    for c in &sys.clauses {
        for &l in c {
            clause_vars.push(cdcl::lit_var(l) as usize);
        }
    }
    clause_vars.sort();
    clause_vars.dedup();
    let compact: HashMap<usize, usize> =
        clause_vars.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    // 子句一律以「系統變量索引」保存；餵 CDCL 時才映射為緊湊索引
    let mut clauses_sys: Vec<Vec<cdcl::Lit>> = sys.clauses.clone();

    let mut cdcl_stats_acc = CdclStats::default();
    let mut learned_total: Vec<Vec<cdcl::Lit>> = vec![];
    let mut sat_model: Option<Vec<bool>> = None;
    let mut rounds = 0usize;
    loop {
        rounds += 1;
        assert!(rounds < 200, "CDCL(T) 迴圈不收斂");
        stage!("S5 round start");
        let compact_clauses: Vec<Vec<cdcl::Lit>> = clauses_sys
            .iter()
            .map(|c| {
                c.iter()
                    .map(|&l| cdcl::lit(compact[&(cdcl::lit_var(l) as usize)], cdcl::lit_positive(l)))
                    .collect()
            })
            .collect();
        let mut solver = cdcl::Solver::new(clause_vars.len(), compact_clauses);
        let ok = solver.solve();
        stage!("S5 cdcl solve");
        cdcl_stats_acc.decisions += solver.stats().decisions;
        cdcl_stats_acc.propagations += solver.stats().propagations;
        cdcl_stats_acc.conflicts += solver.stats().conflicts;
        cdcl_stats_acc.learned += solver.stats().learned;
        for lc in solver.learned_clauses() {
            if !learned_total.contains(&lc) {
                learned_total.push(lc.clone());
            }
        }
        if !ok {
            break; // 子句層 UNSAT ⇒ 整體 UNSAT（見定理 9）
        }
        let model = solver.model().unwrap();
        // 理論檢查：把 arm/borrow 值代入多項式系統
        let mut subst = sys.polys.clone();
        for (vi, &cv) in clause_vars.iter().enumerate() {
            let val = Frac::from_i64(model[vi] as i64);
            subst = subst.iter().map(|f| f.subst_var(cv, &val)).collect();
        }
        let mut with_field = subst;
        with_field.extend(field_polys(sys.nvars));
        stage!("S5 subst done");
        let (g, _) = reduced_groebner(&with_field, Order::GrevLex, Strategy::Normal, true);
        stage!("S5 theory GB");
        let unsat = g.len() == 1 && g[0].is_constant().map_or(false, |c| c.is_one());
        if unsat {
            // 學習子句：¬(當前模型)（轉回系統變量索引）
            let lc: Vec<cdcl::Lit> = clause_vars
                .iter()
                .copied()
                .enumerate()
                .map(|(vi, sysv)| cdcl::lit(sysv, !model[vi]))
                .collect();
            if !clauses_sys.contains(&lc) {
                clauses_sys.push(lc);
            }
            continue;
        }
        sat_model = Some(model);
        break;
    }
    res.cdcl_rounds = rounds;
    res.cdcl_stats = cdcl_stats_acc;
    res.learned_clauses = learned_total.clone();
    res.n_clauses = clauses_sys.len();

    // ── S6 Buchberger 化簡與判定（合併系統 = 約束 + 域多項式 + 子句多項式）──
    let mut merged: Vec<Poly> = sys.polys.clone();
    merged.extend(field_polys(sys.nvars));
    for c in &clauses_sys {
        merged.push(clause_to_poly(c, sys.nvars));
    }
    res.n_polys = merged.len();
    stage!("S6 merge");
    let (red, gstats) = reduced_groebner(&merged, Order::GrevLex, Strategy::Normal, true);
    stage!("S6 GB");
    res.gb_stats = gstats;
    res.reduced_basis = red.clone();
    res.is_unsat = red.len() == 1 && red[0].is_constant().map_or(false, |c| c.is_one());

    // ── S7 見證 σ 與型別解碼 ──
    if !res.is_unsat {
        stage!("S7 start");
        let sigma = solve_boolean(&merged, sys.nvars)
            .ok_or("Gröbner 基非 {1} 但布爾求解失敗（內部不一致）")?;
        stage!("S7 solve sigma");
        // 逐節點解碼型別
        for (node, ts) in &sys.node_type {
            let mut found = None;
            for (ti, &v) in ts.iter().enumerate() {
                if sigma[v].is_one() {
                    found = Some(Type::from_index(ti));
                }
            }
            if let Some(t) = found {
                res.node_types.insert(*node, t);
            }
        }
        for (inv, avs) in &sys.arm_vars {
            for (k, &v) in avs.iter().enumerate() {
                if sigma[v].is_one() {
                    res.arm_choice.insert(*inv, k);
                }
            }
        }
        res.sigma = Some(sigma.clone());

        // ── S8 QAP 驗證 ──
        stage!("S8 r1cs start");
        let r1cs: R1cs = to_r1cs(sys.nvars, &merged);
        stage!("S8 r1cs");
        res.r1cs_constraints = r1cs.constraints.len();
        res.r1cs_wires = r1cs.n_wires;
        let z: Vec<Fp> = r1cs.witness(
            &sigma.iter().map(|f| if f.is_one() { Fp::one() } else { Fp::zero() }).collect::<Vec<_>>(),
        );
        stage!("S8 qap build");
        let qap: Qap = qap_from_r1cs(&r1cs);
        stage!("S8 qap");
        res.qap_max_degree = qap.max_wire_degree();
        res.qap_verified = Some(qap.verify(&z));
        // 竄改見證：翻轉第一個型別位元 ⇒ QAP 必須拒絕
        let mut z_bad = z.clone();
        if sys.nvars > 0 {
            // 找一個值為 1 的變量位元翻轉
            if let Some(i) = sigma.iter().position(|f| f.is_one()) {
                z_bad[i + 1] = z_bad[i + 1] + Fp::one(); // 1 → 2（非法）
            }
        }
        res.qap_tamper_rejected = Some(!qap.verify(&z_bad));

        // ── S9 代碼生成 ──
        if do_codegen {
            // 從 σ 構造推導
            let deriv = Derivation {
                ty: *res.node_types.get(&p.main_body.id).unwrap_or(&Type::Unit),
                node_types: res.node_types.clone(),
                arm_choice: res.arm_choice.clone(),
            };
            let cfg = CodeGenConfig::default();
            let header = format!(
                "程序 {} | 管線判定：SAT | 變量 {} | 約束 {} | Gröbner 基 {} | QAP {}",
                name,
                sys.nvars,
                merged.len(),
                red.len(),
                if res.qap_verified == Some(true) { "通過" } else { "未通過" }
            );
            let code = crate::codegen::generate_rust(&p, &exp, &deriv, &cfg, &header);
            let path = format!("output/generated/{}.rs", name);
            std::fs::create_dir_all("output/generated").ok();
            std::fs::write(&path, &code).map_err(|e| e.to_string())?;
            roundtrip_check(&code)?;
            res.generated_code = Some(code.clone());
            res.generated_file = Some(path.clone());
            // rustc 編譯驗證（若可用）
            let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
            let st = std::process::Command::new(&rustc)
                .args([
                    "--edition",
                    "2021",
                    "--crate-type",
                    "bin",
                    "-o",
                    "/tmp/polyrust_gen",
                    &path,
                ])
                .output();
            res.rustc_compiles = st.ok().map(|o| o.status.success());
            let _ = std::fs::remove_file("/tmp/polyrust_gen");
        }
    }

    // ── 對照 ──
    res.agrees = res.checker_ok != res.is_unsat;
    Ok(res)
}

/// 顯示多項式（帶變量名）。
pub fn show_polys(polys: &[Poly], names: &[String], limit: usize) -> String {
    let mut s = String::new();
    for (i, f) in polys.iter().take(limit).enumerate() {
        s.push_str(&format!("  [{:>3}] {} = 0\n", i, f.display(names)));
    }
    if polys.len() > limit {
        s.push_str(&format!("  …（共 {} 條）\n", polys.len()));
    }
    s
}

/// 基中非零非常數元素（顯示用）。
pub fn basis_summary(basis: &[Poly], names: &[String], limit: usize) -> String {
    if basis.len() == 1 && basis[0].is_constant().map_or(false, |c| c.is_one()) {
        return "  { 1 }（系統矛盾）".to_string();
    }
    let mut s = String::new();
    for (i, g) in basis.iter().take(limit).enumerate() {
        s.push_str(&format!("  [{:>3}] {}\n", i, g.display(names)));
    }
    if basis.len() > limit {
        s.push_str(&format!("  …（共 {} 條）\n", basis.len()));
    }
    s
}

/// 借用統計
pub fn used_var_count(polys: &[Poly]) -> usize {
    let mut s = BTreeSet::new();
    for f in polys {
        for v in f.vars() {
            s.insert(v);
        }
    }
    s.len()
}
