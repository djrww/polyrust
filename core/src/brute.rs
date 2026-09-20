// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! @brute 對照常態化（仿 `rlzl`）：**暴力法 ⟺ 代數法，逐位元比對**。
//!
//! `rlzl` 的哲學：昂貴/精巧的實作必須時刻接受一個**獨立、簡單、直接**的
//! 暴力實作監督；兩者逐位元不一致即失敗。本模組把這條哲學常態化為
//! 每次 `cargo test` 都跑的駐場測試（另見 `polyrust brute` CLI）：
//!
//! 1. **子句層**：對大量隨機＋結構化子句集（臂互斥、鳩巢、恰好一），
//!    斷言 `cdcl::Solver` ⟺ `brute_force_sat`；SAT 時模型必須真滿足全部
//!    子句；每條**學習子句**必須被原子句集的全部模型蘊涵。
//! 2. **約束層**：對小程序的約束系統做**結構化暴力枚舉**——
//!    利用編碼規格（每節點型別位 one-hot）把搜索空間從 2^變量數
//!    壓到 7^節點數 × 2^布爾位，然後**直接逐多項式求值**。
//!    斷言：暴力可解 ⟺ Gröbner 基判定 ⟺ `solve_boolean` ⟺ 獨立檢查器。
//!    暴力側不共用任何求解器代碼（無傳播、無迴圈、純枚舉求值），
//!    保證對照的獨立性。
//! 3. **見證逐位元驗證**：`solve_boolean` 回傳的 σ 必須令合併系統
//!    （約束 + 域多項式 + 子句多項式）**逐條求值歸零**——任何規模都做，
//!    不受暴力枚舉的規模上限限制。

use crate::cdcl::{self, brute_force_sat, satisfies, Lit, Solver};
use crate::frac::Frac;
use crate::groebner::{field_polys, reduced_groebner, solve_boolean, Strategy};
use crate::minirust::ast::N_TYPES;
use crate::minirust::checker::check_program;
use crate::minirust::constraints::{gen_constraints, System};
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;
use crate::pipeline::clause_to_poly;
use crate::poly::{Order, Poly};

// ─────────────────────────────────────────────────────────────────────────────
// §1 決定性偽隨機（零依賴：xorshift64）
// ─────────────────────────────────────────────────────────────────────────────

/// xorshift64：離線、決定性、可重現（種子寫死在測試裡）。
pub struct XorShift(pub u64);

impl XorShift {
    pub fn new(seed: u64) -> Self {
        XorShift(if seed == 0 { 0x9E3779B97F4A7C15 } else { seed })
    }
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// [0, n) 均勻（n > 0）
    pub fn range(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §2 子句層對照：CDCL ⟺ 暴力枚舉
// ─────────────────────────────────────────────────────────────────────────────

/// 結構化子句集（真實管線產生的子句形態）：
/// - `exactly_one(n)`：臂 one-hot（至少一 + 兩兩互斥）——宏臂子句的原型
/// - `php(p, h)`：鳩巢原理 p 鴿 h 籠（p > h ⇒ UNSAT，需要衝突學習）
fn structured_clause_sets() -> Vec<(usize, Vec<Vec<Lit>>)> {
    let mut out = Vec::with_capacity(8);
    for n in 2..=6 {
        let mut cls: Vec<Vec<Lit>> = vec![(0..n).map(|v| cdcl::lit(v, true)).collect()];
        for i in 0..n {
            for j in (i + 1)..n {
                cls.push(vec![cdcl::lit(i, false), cdcl::lit(j, false)]);
            }
        }
        out.push((n, cls));
    }
    for (p, h) in [(3, 2), (4, 3)] {
        let nv = p * h;
        let mut cls = Vec::new();
        for i in 0..p {
            cls.push((0..h).map(|j| cdcl::lit(i * h + j, true)).collect());
        }
        for j in 0..h {
            for i1 in 0..p {
                for i2 in (i1 + 1)..p {
                    cls.push(vec![
                        cdcl::lit(i1 * h + j, false),
                        cdcl::lit(i2 * h + j, false),
                    ]);
                }
            }
        }
        out.push((nv, cls));
    }
    out
}

/// 隨機 k-SAT 子句集（決定性種子）。
fn random_clause_sets(rng: &mut XorShift, count: usize) -> Vec<(usize, Vec<Vec<Lit>>)> {
    let mut out = Vec::with_capacity(8);
    for _ in 0..count {
        let nv = 3 + rng.range(8); // 3..=10 變量（≤20 供暴力枚舉）
        let nc = 1 + rng.range(14); // 1..=14 子句
        out.push(one_random_set(rng, nv, nc));
    }
    out
}

/// 硬 3-SAT：**純寬 3** 子句、子句/變量比 ≈ 3.0（變量 9..=12）——
/// 多數可解但需要大量衝突學習，令**學習子句蘊涵驗證**（須有模型）
/// 有足量樣本。純寬 3 至關重要：混合窄寬度實例太易，無法觸發
/// 真正的衝突驅動學習路徑（v0.1.x 的雙監視文字交換缺陷正是由
/// 純 3-SAT 揭露）。
fn hard_3sat_sets(rng: &mut XorShift, count: usize) -> Vec<(usize, Vec<Vec<Lit>>)> {
    let mut out = Vec::with_capacity(8);
    for _ in 0..count {
        let nv = 9 + rng.range(4); // 9..=12
        let nc = nv * 3; // 可解但困難 ⇒ 有模型可驗證學習子句蘊涵
        let mut cls = Vec::new();
        for _ in 0..nc {
            let mut seen = vec![false; nv];
            let mut c = Vec::new();
            while c.len() < 3 {
                let v = rng.range(nv);
                if seen[v] {
                    continue;
                }
                seen[v] = true;
                c.push(cdcl::lit(v, rng.range(2) == 0));
            }
            cls.push(c);
        }
        out.push((nv, cls));
    }
    out
}

fn one_random_set(rng: &mut XorShift, nv: usize, nc: usize) -> (usize, Vec<Vec<Lit>>) {
    let mut cls = Vec::new();
    for _ in 0..nc {
        let k = 1 + rng.range(3.min(nv)); // 寬 1..=3
        let mut seen = vec![false; nv];
        let mut c = Vec::new();
        while c.len() < k {
            let v = rng.range(nv);
            if seen[v] {
                continue;
            }
            seen[v] = true;
            c.push(cdcl::lit(v, rng.range(2) == 0));
        }
        cls.push(c);
    }
    (nv, cls)
}

#[derive(Clone, Debug, Default)]
pub struct ClauseReport {
    pub sets: u64,
    pub sat_sets: u64,
    pub learned_checked: u64,
    pub mismatches: Vec<String>,
}

/// 子句層對照：對每個子句集斷言
/// (a) CDCL 判定 ⟺ 暴力枚舉判定；
/// (b) CDCL 模型必須真滿足全部子句；
/// (c) 每條學習子句被原子句集的**全部模型**蘊涵（變量 ≤12 時逐模型驗證）。
pub fn cross_check_clauses(random_sets: usize, seed: u64) -> ClauseReport {
    let mut rep = ClauseReport::default();
    let mut rng = XorShift::new(seed);
    let mut sets = structured_clause_sets();
    sets.extend(random_clause_sets(&mut rng, random_sets));
    sets.extend(hard_3sat_sets(&mut rng, random_sets / 5)); // 硬樣本：強迫子句學習

    for (idx, (nv, cls)) in sets.iter().enumerate() {
        rep.sets += 1;
        let brute = brute_force_sat(*nv, cls);
        let mut solver = Solver::new(*nv, cls.clone());
        let cdcl_sat = solver.solve();
        if cdcl_sat != brute.is_some() {
            rep.mismatches.push(format!(
                "[{}] CDCL={} 暴力={}（{} 變量 / {} 子句）",
                idx,
                cdcl_sat,
                brute.is_some(),
                nv,
                cls.len()
            ));
            continue;
        }
        if cdcl_sat {
            rep.sat_sets += 1;
            let model = solver.model().expect("SAT 必有模型");
            if !satisfies(&model, cls) {
                rep.mismatches
                    .push(format!("[{}] CDCL 模型不滿足原子句集", idx));
            }
        }
        // 學習子句蘊涵驗證：暴力遍歷全部滿足賦值
        if *nv <= 12 {
            let learned = solver.learned_clauses();
            for mask in 0u64..(1u64 << nv) {
                let a: Vec<bool> = (0..*nv).map(|i| (mask >> i) & 1 == 1).collect();
                if satisfies(&a, cls) {
                    for lc in &learned {
                        rep.learned_checked += 1;
                        if !satisfies(&a, std::slice::from_ref(lc)) {
                            rep.mismatches.push(format!(
                                "[{}] 學習子句 {:?} 不被模型 {:?} 蘊涵",
                                idx, lc, a
                            ));
                        }
                    }
                }
            }
        }
    }
    rep
}

// ─────────────────────────────────────────────────────────────────────────────
// §3 約束層結構化暴力枚舉
// ─────────────────────────────────────────────────────────────────────────────

/// 結構化暴力枚舉的上限（超出即跳過搜索，僅做逐位元見證驗證）。
const BRUTE_MAX_COMBOS: u64 = 300_000;
const BRUTE_MAX_NODES: usize = 7; // 7^8 > 5M，超出跳過

/// 合併系統（約束 + 域多項式 + 子句多項式）——與管線 S6 同一份。
pub fn merged_system(sys: &System) -> Vec<Poly> {
    let mut merged = sys.polys.clone();
    merged.extend(field_polys(sys.nvars));
    for c in &sys.clauses {
        merged.push(clause_to_poly(c, sys.nvars));
    }
    merged
}

/// 見證逐位元驗證：σ 必須令每條多項式求值歸零。
pub fn verify_witness_bitwise(merged: &[Poly], sigma: &[Frac]) -> Result<(), String> {
    for (i, p) in merged.iter().enumerate() {
        let v = p.eval_full(sigma);
        if !v.is_zero() {
            return Err(format!("約束 [{}] 違反：求值 = {}", i, v));
        }
    }
    Ok(())
}

/// 結構化暴力枚舉：利用 one-hot 規格（每節點恰好一個型別）把空間壓到
/// 7^節點 × 2^(臂位+借用位+其餘位)，直接逐多項式求值。
/// 回傳 `(可解, 見證數【截斷至 1024】)`；`None` = 規模超上限（跳過）。
pub fn brute_system(sys: &System, merged: &[Poly]) -> Option<(bool, u64)> {
    // 節點型別組（排序以決定性）
    let mut nodes: Vec<(usize, [usize; N_TYPES])> =
        sys.node_type.iter().map(|(k, v)| (*k, *v)).collect();
    nodes.sort_by_key(|(k, _)| *k);
    if nodes.len() > BRUTE_MAX_NODES {
        return None;
    }
    // 布爾位：臂位 + 借用位 + 其餘未分組變量
    let mut grouped: Vec<bool> = vec![false; sys.nvars];
    for (_, ts) in &nodes {
        for &v in ts.iter() {
            grouped[v] = true;
        }
    }
    let mut bool_vars: Vec<usize> = Vec::new();
    for avs in sys.arm_vars.values() {
        for &v in avs {
            if !grouped[v] {
                grouped[v] = true;
                bool_vars.push(v);
            }
        }
    }
    for &v in sys.borrow_vars.values() {
        if !grouped[v] {
            grouped[v] = true;
            bool_vars.push(v);
        }
    }
    for v in 0..sys.nvars {
        if !grouped[v] {
            bool_vars.push(v);
        }
    }
    if bool_vars.len() > 20 {
        return None;
    }
    let combos = 7u64.pow(nodes.len() as u32) * (1u64 << bool_vars.len());
    if combos > BRUTE_MAX_COMBOS {
        return None;
    }

    // 預處理：每條多項式（略過零多項式）
    let polys: Vec<&Poly> = merged.iter().filter(|p| !p.is_zero()).collect();
    let bool_n = bool_vars.len();
    let mut sat = false;
    let mut count = 0u64;

    // 遞迴枚舉節點型別；葉層遍歷布爾位掩碼
    let mut ty: Vec<usize> = vec![0; nodes.len()];
    fn rec(
        depth: usize,
        ty: &mut Vec<usize>,
        nodes: &[(usize, [usize; N_TYPES])],
        bool_n: usize,
        bool_vars: &[usize],
        polys: &[&Poly],
        nvars: usize,
        sat: &mut bool,
        count: &mut u64,
    ) {
        if depth == nodes.len() {
            let mut sigma = vec![Frac::ZERO; nvars];
            for (di, (_, ts)) in nodes.iter().enumerate() {
                sigma[ts[ty[di]]] = Frac::ONE;
            }
            for mask in 0u64..(1u64 << bool_n) {
                for (bi, &v) in bool_vars.iter().enumerate() {
                    sigma[v] = if (mask >> bi) & 1 == 1 { Frac::ONE } else { Frac::ZERO };
                }
                if polys.iter().all(|p| p.eval_full(&sigma).is_zero()) {
                    *sat = true;
                    *count = (*count).saturating_add(1).min(1024);
                }
            }
            return;
        }
        for t in 0..N_TYPES {
            ty[depth] = t;
            rec(
                depth + 1,
                ty,
                nodes,
                bool_n,
                bool_vars,
                polys,
                nvars,
                sat,
                count,
            );
        }
    }
    rec(
        0,
        &mut ty,
        &nodes,
        bool_n,
        &bool_vars,
        &polys,
        sys.nvars,
        &mut sat,
        &mut count,
    );
    Some((sat, count))
}

// ─────────────────────────────────────────────────────────────────────────────
// §4 約束層對照驅動
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct BruteReport {
    pub programs: u64,
    pub brute_searched: u64,  // 做了結構化暴力搜索的程序數
    pub witnesses_checked: u64, // 逐位元見證驗證次數
    pub sat: u64,
    pub unsat: u64,
    pub mismatches: Vec<String>,
}

/// 單個程序的約束層對照：
/// 暴力枚舉 ⟺ Gröbner 基判定 ⟺ `solve_boolean` ⟺ 獨立檢查器；
/// SAT 時見證逐位元驗證 + one-hot 解碼健全性。
pub fn cross_check_program(src: &str, rep: &mut BruteReport) {
    rep.programs += 1;
    let p = match Parser::parse_program(src) {
        Ok(p) => p,
        Err(e) => {
            rep.mismatches.push(format!("解析失敗：{}\n程式：{}", e, src));
            return;
        }
    };
    let mut exp_c = Expander::new(p.macros.clone(), p.next_id);
    let checker_ok = check_program(&p, &mut exp_c).is_ok();
    let mut exp_g = Expander::new(p.macros.clone(), p.next_id);
    let sys = match gen_constraints(&p, &mut exp_g) {
        Ok(s) => s,
        Err(e) => {
            rep.mismatches
                .push(format!("約束生成失敗：{}\n程式：{}", e, src));
            return;
        }
    };
    let merged = merged_system(&sys);

    // ── 代數側判定 1：Gröbner 基（管線 S6 的判定核心）──
    let (red, _) = reduced_groebner(&merged, Order::GrevLex, Strategy::Normal, true);
    let gb_sat = !(red.len() == 1 && red[0].is_constant().is_some_and(|c| c.is_one()));

    // ── 代數側判定 2：布爾求解見證 ──
    let sigma = solve_boolean(&merged, sys.nvars);
    let solve_sat = sigma.is_some();

    // ── 暴力側（結構化枚舉；規模超限則跳過搜索，但仍做見證驗證）──
    let brute = brute_system(&sys, &merged);
    if let Some((brute_sat, _witnesses)) = brute {
        rep.brute_searched += 1;
        if brute_sat != gb_sat {
            rep.mismatches.push(format!(
                "暴力={} Gröbner={} 不一致\n程式：{}",
                brute_sat, gb_sat, src
            ));
        }
        if brute_sat != solve_sat {
            rep.mismatches.push(format!(
                "暴力={} solve_boolean={} 不一致\n程式：{}",
                brute_sat, solve_sat, src
            ));
        }
        if brute_sat != checker_ok {
            rep.mismatches.push(format!(
                "暴力={} 檢查器={} 不一致（T6 等價違反）\n程式：{}",
                brute_sat, checker_ok, src
            ));
        }
    }

    // ── 四方判定必須一致（即使暴力跳過，三方也要一致）──
    if gb_sat != solve_sat || gb_sat != checker_ok {
        rep.mismatches.push(format!(
            "判定不一致：Gröbner={} solve_boolean={} 檢查器={}\n程式：{}",
            gb_sat, solve_sat, checker_ok, src
        ));
    }

    if gb_sat {
        rep.sat += 1;
    } else {
        rep.unsat += 1;
    }

    // ── 見證逐位元驗證（任何規模）──
    if let Some(s) = &sigma {
        rep.witnesses_checked += 1;
        if let Err(e) = verify_witness_bitwise(&merged, s) {
            rep.mismatches.push(format!("{}（見證逐位元驗證失敗）\n程式：{}", e, src));
        }
        // one-hot 解碼健全性：每個節點恰好一個型別位為 1
        for (node, ts) in &sys.node_type {
            let ones = ts.iter().filter(|&&v| s[v].is_one()).count();
            if ones != 1 {
                rep.mismatches.push(format!(
                    "節點 {} 型別位 one-hot 破產（{} 個 1）\n程式：{}",
                    node, ones, src
                ));
            }
        }
    }
}

/// 約束層對照：表達式空間 ≤ `max_size` 全部程序 + 定置宏/函式樣本。
pub fn run_brute_cross(max_size: usize) -> BruteReport {
    let mut rep = BruteReport::default();
    for size in 1..=max_size {
        let mut exprs = Vec::new();
        crate::exhaust::gen_seq(size, 0, (0, 0), &mut exprs, 100_000);
        for e in &exprs {
            let mut env: Vec<String> = Vec::new();
            let mut next_name = 0usize;
            let body = crate::exhaust::render(e, &mut env, &mut next_name);
            let src = format!("fn main() {{\n{}\n}}", body);
            cross_check_program(&src, &mut rep);
        }
    }
    // 定置樣本：宏臂選擇 / 借用 / fn（見證逐位元驗證不受規模上限影響）
    const EXTRA: &[&str] = &[
        "macro_rules! sqr { ($e:expr) => { $e * $e } }\nfn main() { let a = sqr!(2); }",
        "macro_rules! pick { ($a:expr) => { $a + 1 }; ($a:expr) => { !$a } }\nfn main() { let u = 3; let v = pick!(u); }",
        "macro_rules! twice_mut { ($v:ident) => { let r1 = &mut $v; let r2 = &mut $v; *r1 + *r2 } }\nfn main() { let x = 0; let u = twice_mut!(x); }",
        "fn add(a: i32, b: i32) -> i32 { a + b }\nfn main() { let s = add(1, 2); }",
        "fn main() { let x = 0; let r = &mut x; *r = 1; }",
        "fn main() { let u = nosuch!(1); }",
    ];
    for src in EXTRA {
        cross_check_program(src, &mut rep);
    }
    rep
}

// ─────────────────────────────────────────────────────────────────────────────
// §5 駐場測試（每次 `cargo test` 必跑）
// ─────────────────────────────────────────────────────────────────────────────

/// 實際使用：brute.rs 文件清單 — 優化 with_capacity
pub fn brute_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("brute.rs", "brute.rs 正式運作 — 優化 with_capacity", "core/src/brute.rs"),
    ]
}

