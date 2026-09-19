use crate::cdcl::{self, CdclStats};
use crate::codegen::{roundtrip_check, CodeGenConfig};
use crate::frac::Frac;
use crate::fp::Fp;
use crate::groebner::{field_polys, reduced_groebner, solve_boolean, GroebnerStats, Strategy};
use crate::groebner_f4::reduced_f4;
use crate::groebner_f5::reduced_f5;
use crate::groebner_f4f5::reduced_f4f5;
use crate::minirust::ast::{Program, Type};
use crate::minirust::checker::{check_program, Derivation};
use crate::minirust::constraints::{gen_constraints, to_r1cs, System};
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;
use crate::poly::{Order, Poly};
use crate::qap::{qap_from_r1cs, Qap, R1cs};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroebnerAlgo {
    Classic,
    F4,
    F5,
    F4F5,
}

impl GroebnerAlgo {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "f4" => GroebnerAlgo::F4,
            "f5" => GroebnerAlgo::F5,
            "f4f5" | "f4/f5" | "f4_f5" => GroebnerAlgo::F4F5,
            _ => GroebnerAlgo::Classic,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            GroebnerAlgo::Classic => "classic",
            GroebnerAlgo::F4 => "f4",
            GroebnerAlgo::F5 => "f5",
            GroebnerAlgo::F4F5 => "f4f5",
        }
    }
}

/// 估算稀疏度：平均項數 / nvars
fn estimate_sparsity(fs: &[Poly], nvars: usize) -> f64 {
    if fs.is_empty() || nvars == 0 {
        return 0.0;
    }
    let total_terms: usize = fs.iter().map(|p| p.terms.len()).sum();
    let avg_terms = total_terms as f64 / fs.len() as f64;
    // 歸一化：若平均項數遠小於 nvars，認為稀疏
    (avg_terms / (nvars as f64).max(1.0) * 100.0).min(100.0)
}

/// 估算塊數：按變量支集並查集快速估算
fn estimate_blocks(fs: &[Poly]) -> usize {
    use std::collections::{BTreeSet, HashMap};
    if fs.len() <= 1 {
        return fs.len();
    }
    let mut var_sets: Vec<BTreeSet<usize>> = Vec::with_capacity(fs.len());
    for p in fs {
        let mut vs = BTreeSet::new();
        for (m, _) in &p.terms {
            for (vi, &e) in m.iter().enumerate() {
                if e > 0 {
                    vs.insert(vi);
                }
            }
        }
        var_sets.push(vs);
    }
    let n = fs.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }
    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[rb] = ra;
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if !var_sets[i].is_disjoint(&var_sets[j]) {
                union(&mut parent, i, j);
            }
        }
    }
    let mut groups: HashMap<usize, usize> = HashMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        *groups.entry(r).or_default() += 1;
    }
    groups.len()
}

pub fn select_groebner_algo(nvars: usize, npolys: usize) -> GroebnerAlgo {
    // 兼容舊接口：基於 nvars/npolys 的快速啟發式
    if nvars > 200 && npolys > 500 {
        GroebnerAlgo::F4
    } else if npolys > 100 {
        GroebnerAlgo::F4F5
    } else if nvars > 50 {
        GroebnerAlgo::F4
    } else {
        GroebnerAlgo::Classic
    }
}

/// 自動策略選擇高級版：返回 (算法, 決策理由)
pub fn select_groebner_algo_advanced(fs: &[Poly], nvars: usize) -> (GroebnerAlgo, String) {
    let npolys = fs.len();
    let sparsity = estimate_sparsity(fs, nvars);
    let blocks = estimate_blocks(fs);
    let avg_terms = if npolys > 0 {
        fs.iter().map(|p| p.terms.len()).sum::<usize>() as f64 / npolys as f64
    } else {
        0.0
    };
    // 估算 pair 度數重疊
    let overlap_ratio = if npolys > 1 { blocks as f64 / npolys as f64 } else { 1.0 };
    // 決策鏈
    let (algo, reason) = if blocks > 1 && npolys > 20 {
        if blocks >= 3 {
            (GroebnerAlgo::F4, format!("blocks={}>=3 multi-block parallel, npolys={}, avg_terms={:.1}", blocks, npolys, avg_terms))
        } else if avg_terms < 8.0 {
            (GroebnerAlgo::F4, format!("blocks={} sparse avg_terms={:.1} -> F4 sparse+parallel", blocks, avg_terms))
        } else {
            (GroebnerAlgo::F4, format!("blocks={} >1 npolys={} -> F4 block-diagonal", blocks, npolys))
        }
    } else if sparsity < 20.0 && avg_terms < 10.0 && nvars > 30 {
        (GroebnerAlgo::F4, format!("sparse density={:.1}% avg_terms={:.1} nvars={} -> F4 sparse", sparsity, avg_terms, nvars))
    } else if npolys > 100 && nvars < 100 {
        (GroebnerAlgo::F4F5, format!("npolys={} nvars={} dense -> F4F5 sig-filter", npolys, nvars))
    } else if nvars > 200 {
        (GroebnerAlgo::F4, format!("nvars={} huge -> F4 batch", nvars))
    } else if npolys > 50 || nvars > 50 {
        (GroebnerAlgo::F4, format!("medium nvars={} npolys={} -> F4", nvars, npolys))
    } else {
        (GroebnerAlgo::Classic, format!("small nvars={} npolys={} overlap_ratio={:.2} -> Classic stable", nvars, npolys, overlap_ratio))
    };
    (algo, reason)
}

/// 自動策略選擇：基於稀疏化+分塊並行特徵
/// 輸入完整多項式集合，綜合 nvars、npolys、稀疏度、塊數
pub fn select_groebner_algo_auto(fs: &[Poly], nvars: usize) -> GroebnerAlgo {
    let npolys = fs.len();
    let sparsity = estimate_sparsity(fs, nvars);
    let blocks = estimate_blocks(fs);
    let avg_terms = if npolys > 0 {
        fs.iter().map(|p| p.terms.len()).sum::<usize>() as f64 / npolys as f64
    } else {
        0.0
    };

    // 啟發式規則（按優先級）
    // 1. 多塊獨立系統 → F4 分塊並行最優
    if blocks > 1 && npolys > 20 {
        // 若塊數 >=3 且平均項數小，F4 並行收益大
        if blocks >= 3 || avg_terms < 8.0 {
            return GroebnerAlgo::F4;
        }
    }
    // 2. 極稀疏布爾系統（密度 <20%）→ F4 稀疏矩陣
    if sparsity < 20.0 && nvars > 30 {
        return GroebnerAlgo::F4;
    }
    // 3. 稠密大系統且多對 → F4F5 簽名過濾
    if npolys > 100 && avg_terms > 5.0 {
        return GroebnerAlgo::F4F5;
    }
    // 4. 超大變量 → F4
    if nvars > 200 && npolys > 500 {
        return GroebnerAlgo::F4;
    }
    // 5. 中等規模 → F4
    if nvars > 50 || npolys > 50 {
        return GroebnerAlgo::F4;
    }
    // 6. 小規模 → Classic 穩定
    GroebnerAlgo::Classic
}

/// Gröbner 基計算模式（v0.3 hardening 新增）。
///
/// * `Eager`：CDCL(T) 迴圈後**再算一次全量規約 Gröbner 基**——證書/審計/證據用途
///   （obligations、demo、exhaust 走此路，輸出與 v0.2.2 逐位一致）。
/// * `Lazy`：跳過最終全基，判定直接採 CDCL(T) 迴圈結果。正確性由現有定理保證：
///   - SAT 側：迴圈最後一輪理論檢查已給出布林根 ⇒ 1∉G（T6），判定 SAT；
///   - UNSAT 側：CDCL 判定子句骨不可滿足 ⇒ 子句多項式自身生成單位理想（T3 對偶），判定 UNSAT。
///   `reduced_basis` 留空、`gb_stats` 歸零，並置 `gb_basis_deferred=true`。
///
/// 動機（2026-09-19 實測，`PL_DBG=1`）：sqr.poly（170 vars）中 S6 全基獨佔 21.5s/23.4s（92%），
/// 跳過後 check 降至 ~2s，判定與 QAP/codegen/rustc 結果不變。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GbBasisMode {
    Eager,
    Lazy,
}

pub fn reduced_groebner_with_algo(fs: &[Poly], ord: Order, algo: GroebnerAlgo) -> (Vec<Poly>, GroebnerStats) {
    match algo {
        GroebnerAlgo::Classic => reduced_groebner(fs, ord, Strategy::Normal, true),
        GroebnerAlgo::F4 => {
            let (g, f4stats) = reduced_f4(fs, ord);
            let stats = GroebnerStats {
                generators: f4stats.generators,
                pairs_considered: f4stats.pairs_considered,
                crit1_skips: f4stats.crit1_skips,
                crit2_skips: f4stats.crit2_skips,
                s_polys: f4stats.s_polys,
                reductions_to_zero: f4stats.reductions_to_zero,
                basis_adds: f4stats.basis_adds,
                basis_final: f4stats.basis_final,
            };
            (g, stats)
        }
        GroebnerAlgo::F5 => {
            let (g, f5stats) = reduced_f5(fs, ord);
            let stats = GroebnerStats {
                generators: f5stats.generators,
                pairs_considered: f5stats.pairs_considered,
                crit1_skips: f5stats.crit1_skips + f5stats.f5_criterion_skips,
                crit2_skips: f5stats.crit2_skips + f5stats.rewritten_skips,
                s_polys: f5stats.s_polys,
                reductions_to_zero: f5stats.reductions_to_zero,
                basis_adds: f5stats.basis_adds,
                basis_final: f5stats.basis_final,
            };
            (g, stats)
        }
        GroebnerAlgo::F4F5 => {
            let (g, f4f5stats) = reduced_f4f5(fs, ord);
            let stats = GroebnerStats {
                generators: f4f5stats.generators,
                pairs_considered: f4f5stats.pairs_considered,
                crit1_skips: f4f5stats.crit1_skips + f4f5stats.f5_skips,
                crit2_skips: f4f5stats.crit2_skips + f4f5stats.rewritten_skips,
                s_polys: f4f5stats.s_polys,
                reductions_to_zero: f4f5stats.reductions_to_zero,
                basis_adds: f4f5stats.basis_adds,
                basis_final: f4f5stats.basis_final,
            };
            (g, stats)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PipelineResult {
    pub n_vars: usize,
    pub n_polys: usize,
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
    /// v0.3 hardening：Lazy 模式下為 true——最終全量規約基被推遲（未計算），
    /// 判定採 CDCL(T) 迴圈結果（見 GbBasisMode）。需要基與統計時用 `run_pipeline_eager`。
    pub gb_basis_deferred: bool,
}

pub fn clause_to_poly(clause: &[cdcl::Lit], nvars: usize) -> Poly {
    let mut p = Poly::constant(Frac::ONE);
    for &l in clause {
        let v = cdcl::lit_var(l) as usize;
        let factor = if cdcl::lit_positive(l) {
            Poly::constant(Frac::ONE).sub(&Poly::var(v, Frac::ONE, nvars))
        } else {
            Poly::var(v, Frac::ONE, nvars)
        };
        p = p.mul(&factor);
    }
    p
}

pub fn run_pipeline_with_algo(name: &str, source: &str, do_codegen: bool, algo: Option<GroebnerAlgo>) -> Result<PipelineResult, String> {
    // 相容包裝：顯式指定算法 = 證據/審計意圖 ⇒ 走 Eager（與 v0.2.2 行為一致）。
    run_pipeline_with_mode(name, source, do_codegen, algo, GbBasisMode::Eager)
}

#[allow(unused_assignments)]
pub fn run_pipeline_with_mode(name: &str, source: &str, do_codegen: bool, algo: Option<GroebnerAlgo>, gb_mode: GbBasisMode) -> Result<PipelineResult, String> {
    let mut res = PipelineResult::default();
    let mut t0 = std::time::Instant::now();
    macro_rules! stage { ($m:expr) => {
        if std::env::var("PL_DBG").is_ok() {
            eprintln!("[{}] {:?} {}", name, t0.elapsed(), $m);
            t0 = std::time::Instant::now();
        }
    } }

    let p: Program = Parser::parse_program(source)?;
    let mut exp = Expander::new(p.macros.clone(), p.next_id);
    let checker: Result<Vec<Derivation>, String> = check_program(&p, &mut exp);
    match &checker {
        Ok(ds) => {
            res.checker_ok = true;
            res.checker_msg = format!("accept {} derivations arm {:?} main {}", ds.len(), ds.first().map(|d| d.arm_choice.clone()).unwrap_or_default(), ds.first().map(|d| d.ty.name()).unwrap_or("?"));
        }
        Err(e) => {
            res.checker_ok = false;
            res.checker_msg = format!("reject: {}", e);
        }
    }
    for ((node, arm), text) in &exp.memo_text {
        res.expansion_log.push(format!("invoke#{} arm{} -> {}", node, arm + 1, text));
    }
    stage!("S3 checker");
    let sys: System = gen_constraints(&p, &mut exp)?;
    stage!("S4 constraints");
    res.n_vars = sys.nvars;

    let mut clause_vars: Vec<usize> = vec![];
    for c in &sys.clauses {
        for &l in c { clause_vars.push(cdcl::lit_var(l) as usize); }
    }
    clause_vars.sort(); clause_vars.dedup();
    let compact: HashMap<usize, usize> = clause_vars.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let mut clauses_sys: Vec<Vec<cdcl::Lit>> = sys.clauses.clone();

    let mut cdcl_stats_acc = CdclStats::default();
    let mut learned_total: Vec<Vec<cdcl::Lit>> = vec![];
    let mut rounds = 0usize;
    let mut cdcl_failed = false; // v0.3 hardening：Lazy 模式判定所需的 CDCL 骨架失敗標記
    loop {
        rounds += 1;
        if rounds >= 200 { return Err("CDCL(T) 200 rounds not converge".to_string()); }
        stage!("S5 round start");
        let compact_clauses: Vec<Vec<cdcl::Lit>> = clauses_sys.iter().map(|c| {
            c.iter().map(|&l| cdcl::lit(compact[&(cdcl::lit_var(l) as usize)], cdcl::lit_positive(l))).collect()
        }).collect();
        let mut solver = cdcl::Solver::new(clause_vars.len(), compact_clauses);
        let ok = solver.solve();
        stage!("S5 cdcl solve");
        cdcl_stats_acc.decisions += solver.stats().decisions;
        cdcl_stats_acc.propagations += solver.stats().propagations;
        cdcl_stats_acc.conflicts += solver.stats().conflicts;
        cdcl_stats_acc.learned += solver.stats().learned;
        for lc in solver.learned_clauses() { if !learned_total.contains(&lc) { learned_total.push(lc.clone()); } }
        if !ok { cdcl_failed = true; break; }
        let model = solver.model().unwrap();
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
            let lc: Vec<cdcl::Lit> = clause_vars.iter().copied().enumerate().map(|(vi, sysv)| cdcl::lit(sysv, !model[vi])).collect();
            if !clauses_sys.contains(&lc) { clauses_sys.push(lc); }
            continue;
        }
        break;
    }
    res.cdcl_rounds = rounds;
    res.cdcl_stats = cdcl_stats_acc;
    res.learned_clauses = learned_total.clone();
    res.n_clauses = clauses_sys.len();

    let mut merged: Vec<Poly> = sys.polys.clone();
    merged.extend(field_polys(sys.nvars));
    for c in &clauses_sys { merged.push(clause_to_poly(c, sys.nvars)); }
    res.n_polys = merged.len();
    stage!("S6 merge");
    let chosen_algo = algo.unwrap_or_else(|| select_groebner_algo_auto(&merged, sys.nvars));
    if std::env::var("GB_ALGO").is_ok() || std::env::var("PL_DBG").is_ok() {
        let sparsity = estimate_sparsity(&merged, sys.nvars);
        let blocks = estimate_blocks(&merged);
        eprintln!("[{}] Groebner algo: {} (nvars={}, npolys={}, sparsity={:.1}%, blocks={})", name, chosen_algo.as_str(), sys.nvars, merged.len(), sparsity, blocks);
    }
    let (red, gstats) = match gb_mode {
        GbBasisMode::Eager => {
            let r = reduced_groebner_with_algo(&merged, Order::GrevLex, chosen_algo);
            stage!("S6 GB");
            r
        }
        GbBasisMode::Lazy => {
            // v0.3 hardening：跳過最終全量規約基（SAT 實測省 ~92% 總耗時）。
            // 判定：CDCL(T) 迴圈結果——cdcl_failed=true ⇒ 子句多項式生成 1（T3）；否則
            // 最後一輪理論檢查已保證存在布林根 ⇒ 1∉G（T6）。基與統計留待 Eager 模式補算。
            res.gb_basis_deferred = true;
            (
                Vec::new(),
                GroebnerStats {
                    generators: 0,
                    pairs_considered: 0,
                    crit1_skips: 0,
                    crit2_skips: 0,
                    s_polys: 0,
                    reductions_to_zero: 0,
                    basis_adds: 0,
                    basis_final: 0,
                },
            )
        }
    };
    res.gb_stats = gstats;
    res.reduced_basis = red.clone();
    res.is_unsat = match gb_mode {
        GbBasisMode::Eager => red.len() == 1 && red[0].is_constant().map_or(false, |c| c.is_one()),
        GbBasisMode::Lazy => cdcl_failed,
    };

    if !res.is_unsat {
        stage!("S7 start");
        let sigma = solve_boolean(&merged, sys.nvars).ok_or("Groebner not {1} but solve failed")?;
        stage!("S7 solve sigma");
        for (node, ts) in &sys.node_type {
            let mut found = None;
            for (ti, &v) in ts.iter().enumerate() { if sigma[v].is_one() { found = Some(Type::from_index(ti)); } }
            if let Some(t) = found { res.node_types.insert(*node, t); }
        }
        for (inv, avs) in &sys.arm_vars {
            for (k, &v) in avs.iter().enumerate() { if sigma[v].is_one() { res.arm_choice.insert(*inv, k); } }
        }
        res.sigma = Some(sigma.clone());

        stage!("S8 r1cs start");
        let r1cs: R1cs = to_r1cs(sys.nvars, &merged);
        stage!("S8 r1cs");
        res.r1cs_constraints = r1cs.constraints.len();
        res.r1cs_wires = r1cs.n_wires;
        let z: Vec<Fp> = r1cs.witness(&sigma.iter().map(|f| if f.is_one() { Fp::one() } else { Fp::zero() }).collect::<Vec<_>>());
        stage!("S8 qap build");
        let qap: Qap = qap_from_r1cs(&r1cs);
        stage!("S8 qap");
        res.qap_max_degree = qap.max_wire_degree();
        res.qap_verified = Some(qap.verify(&z));
        let mut z_bad = z.clone();
        if sys.nvars > 0 {
            if let Some(i) = sigma.iter().position(|f| f.is_one()) { z_bad[i+1] = z_bad[i+1] + Fp::one(); }
        }
        res.qap_tamper_rejected = Some(!qap.verify(&z_bad));

        if do_codegen {
            let deriv = Derivation {
                ty: *res.node_types.get(&p.main_body.id).unwrap_or(&Type::Unit),
                node_types: res.node_types.clone(),
                arm_choice: res.arm_choice.clone(),
            };
            let cfg = CodeGenConfig::default();
            let basis_desc = if res.gb_basis_deferred { "skipped(lazy)".to_string() } else { red.len().to_string() };
            let header = format!("program {} SAT vars {} constraints {} basis {} qap {}", name, sys.nvars, merged.len(), basis_desc, if res.qap_verified==Some(true){"pass"}else{"fail"});
            let code = crate::codegen::generate_rust(&p, &exp, &deriv, &cfg, &header);
            let path = format!("output/generated/{}.rs", name);
            std::fs::create_dir_all("output/generated").ok();
            std::fs::write(&path, &code).map_err(|e| e.to_string())?;
            roundtrip_check(&code)?;
            res.generated_code = Some(code.clone());
            res.generated_file = Some(path.clone());
            let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
            let st = std::process::Command::new(&rustc).args(["--edition","2021","--crate-type","bin","-o","/tmp/polyrust_gen",&path]).output();
            res.rustc_compiles = st.ok().map(|o| o.status.success());
            let _ = std::fs::remove_file("/tmp/polyrust_gen");
        }
    }
    res.agrees = res.checker_ok != res.is_unsat;
    Ok(res)
}

pub fn run_pipeline(name: &str, source: &str, do_codegen: bool) -> Result<PipelineResult, String> {
    // v0.3 hardening：常用入口默認 **Lazy**（判定不變，SAT 實測快 ~12×）。
    // 需要全量規約基（證書/審計/obligations/demo/證據）請用 `run_pipeline_eager`。
    run_pipeline_with_mode(name, source, do_codegen, None, GbBasisMode::Lazy)
}

/// 全量規約基模式（與 v0.2.2 行為逐位一致）：obligations/demo/exhaust/證據檔專用。
pub fn run_pipeline_eager(name: &str, source: &str, do_codegen: bool) -> Result<PipelineResult, String> {
    run_pipeline_with_mode(name, source, do_codegen, None, GbBasisMode::Eager)
}

/// 實際使用：pipeline.rs 文件清單 — 優化 with_capacity
pub fn pipeline_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("pipeline.rs", "pipeline.rs 正式運作 — 優化 with_capacity", "core/src/pipeline.rs"),
    ]
}

