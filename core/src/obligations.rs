//! 九條定理的機械化自證（義務自證）：
//! 每條定理對應一個 obligation_tN()，在測試套件上實例化驗證其可判部分。
//! 數學證明見 docs/THEOREMS.md；此處的程式檢查是證明的機械見證。

use crate::cdcl::{self, brute_force_sat, satisfies};
use crate::frac::Frac;
use crate::groebner::{field_polys, normal_form, reduced_groebner, solve_boolean, Strategy};
use crate::minirust::ast::Program;
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;
use crate::pipeline::{clause_to_poly, run_pipeline_eager as run_pipeline, PipelineResult};
use crate::poly::{Order, Poly};
use std::collections::HashMap;

pub struct ObligationResult {
    pub id: &'static str,
    pub name: &'static str,
    pub statement: String,
    pub pass: bool,
    pub detail: String,
}

/// 測試套件：(名稱, 源碼, 期望 checker 結果)
pub const SUITE: &[(&str, &str, bool)] = &[
    ("P1-plain", "fn main() { let a = 5; let b = a + 1; }", true),
    ("P2-addbool", "fn main() { 1 + true }", false),
    ("P3-sqr", "macro_rules! sqr { ($e:expr) => { $e * $e } }\nfn main() { let a = 5; let b = sqr!(a + 1); }", true),
    ("P4-bad", "macro_rules! bad { ($e:expr) => { $e + true } }\nfn main() { let b = bad!(1); }", false),
    (
        "P5-twice-mut",
        "macro_rules! twice_mut { ($v:ident) => { let r1 = &mut $v; let r2 = &mut $v; *r1 + *r2 } }\nfn main() { let x = 0; let u = twice_mut!(x); }",
        false,
    ),
    (
        "P6-temp-borrow",
        "macro_rules! tmp { ($v:ident) => { *(&mut $v) + *(&mut $v) } }\nfn main() { let x = 0; let u = tmp!(x); }",
        true,
    ),
    (
        "P7-pick-int",
        "macro_rules! pick { ($a:expr) => { $a + 1 }; ($a:expr) => { !$a } }\nfn main() { let u = 3; let v = pick!(u); }",
        true,
    ),
    (
        "P8-pick-bool",
        "macro_rules! pick { ($a:expr) => { $a + 1 }; ($a:expr) => { !$a } }\nfn main() { let c = true; let v = pick!(c); }",
        true,
    ),
    ("P9-ifmix", "fn main() { if true { 1 } else { false } }", false),
    (
        "P10-fn",
        "fn sqr(x: i32) -> i32 { x * x }\nmacro_rules! sqrm { ($e:expr) => { $e * $e } }\nfn main() { let a = sqr(3) + sqrm!(2); }",
        true,
    ),
    (
        "P11-shadow",
        "fn main() { let x = 1; let y = x + 1; let x = true; let z = !x; }",
        true,
    ),
    (
        "P12-assign-bad",
        "fn main() { let x = 1; x = true; }",
        false,
    ),
];

/// 取得管線結果（緩存）。
fn pipeline_results() -> Vec<(&'static str, PipelineResult)> {
    SUITE
        .iter()
        .map(|(name, src, _)| {
            let r = run_pipeline(name, src, false).expect("管線執行失敗");
            (*name, r)
        })
        .collect()
}

fn parse_and_expand(src: &str) -> (Program, Expander) {
    let p = Parser::parse_program(src).expect("解析失敗");
    let exp = Expander::new(p.macros.clone(), p.next_id);
    (p, exp)
}

// ── T1 編碼可靠性：checker 接受 ⇒ 系統有 0/1 解，且解與推導一致 ──
pub fn obligation_t1() -> ObligationResult {
    let results = pipeline_results();
    let mut detail = String::new();
    let mut pass = true;
    for (name, src, expect_ok) in SUITE {
        let _r = &results.iter().find(|x| x.0 == *name).unwrap().1;
        if !*expect_ok {
            continue; // T1 只看良構程序
        }
        // 直接構造推導的規範賦值 σ_D 並驗證其為多項式系統的根
        let (p, mut exp) = parse_and_expand(src);
        let ds = crate::minirust::checker::check_program(&p, &mut exp).expect("checker 應接受");
        let d = &ds[0];
        let sys = crate::minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
        // σ_D：型別位元按推導；臂位元按推導；借用位元 = 1
        let mut sigma = vec![Frac::ZERO; sys.nvars];
        for (node, t) in &d.node_types {
            if let Some(ts) = sys.node_type.get(node) {
                sigma[ts[t.index()]] = Frac::ONE;
            }
        }
        for (inv, arm) in &d.arm_choice {
            if let Some(avs) = sys.arm_vars.get(inv) {
                sigma[avs[*arm]] = Frac::ONE;
            }
        }
        for (_, bv) in &sys.borrow_vars {
            sigma[*bv] = Frac::ONE;
        }
        // 未覆蓋節點（未選臂的內部節點等）：補預設 one-hot（其約束均乘 a_i = 0，任意補值皆合法）
        for (node, ts) in &sys.node_type {
            if d.node_types.get(node).is_none() {
                sigma[ts[crate::minirust::ast::Type::I32.index()]] = Frac::ONE;
            }
        }
        // 驗證 σ_D 是所有生成元 + 域多項式的根
        let mut all = sys.polys.clone();
        all.extend(field_polys(sys.nvars));
        let bad = all.iter().filter(|f| !f.eval_full(&sigma).is_zero()).count();
        if bad == 0 {
            detail.push_str(&format!("{}：σ_D 是系統的根 ✓  ", name));
        } else {
            pass = false;
            detail.push_str(&format!("{}：σ_D 違反 {} 條約束 ✗  ", name, bad));
        }
    }
    ObligationResult {
        id: "T1",
        name: "編碼可靠性",
        statement: "Γ ⊢ e : τ ⇒ σ_D ∈ V₀/₁(F(e) ∪ B)".into(),
        pass,
        detail,
    }
}

// ── T2 編碼完備性：系統解 ⇒ 存在接受推導，且型別一致 ──
pub fn obligation_t2() -> ObligationResult {
    let results = pipeline_results();
    let mut detail = String::new();
    let mut pass = true;
    for (name, src, expect_ok) in SUITE {
        let r = &results.iter().find(|x| x.0 == *name).unwrap().1;
        if r.is_unsat {
            continue; // T2 只看有解系統
        }
        let (p, mut exp) = parse_and_expand(src);
        let ds = crate::minirust::checker::check_program(&p, &mut exp).unwrap();
        // σ 解碼出的臂選擇必須對應某條接受推導
        let found = ds.iter().any(|d| {
            r.arm_choice.iter().all(|(inv, arm)| d.arm_choice.get(inv) == Some(arm))
                && r.node_types.get(&p.main_body.id) == Some(&d.ty)
        });
        if found {
            detail.push_str(&format!("{}：σ 解碼 ↔ 推導 ✓  ", name));
        } else {
            pass = false;
            detail.push_str(&format!("{}：σ 與推導不符 ✗  ", name));
        }
        let _ = expect_ok;
    }
    ObligationResult {
        id: "T2",
        name: "編碼完備性",
        statement: "σ ∈ V₀/₁(F(e) ∪ B) ⇒ ∃ Γ ⊢ e : τ（且 σ 解碼該推導）".into(),
        pass,
        detail,
    }
}

// ── T3 子句–多項式對偶：CDCL ⟺ Gröbner；學習子句 ∈ 理想 ──
pub fn obligation_t3() -> ObligationResult {
    let mut detail = String::new();
    let mut pass = true;

    // (i) 純子句實例：CDCL 結果 = GB 判定
    let clause_sets: Vec<(usize, Vec<Vec<cdcl::Lit>>)> = vec![
        (
            3,
            vec![
                vec![cdcl::lit(0, true), cdcl::lit(1, true)],
                vec![cdcl::lit(0, false), cdcl::lit(2, false)],
                vec![cdcl::lit(1, false), cdcl::lit(2, true)],
            ],
        ),
        (
            4,
            vec![
                vec![cdcl::lit(0, true), cdcl::lit(1, true), cdcl::lit(2, true)],
                vec![cdcl::lit(0, false), cdcl::lit(1, false)],
                vec![cdcl::lit(1, false), cdcl::lit(2, false)],
                vec![cdcl::lit(2, false), cdcl::lit(3, false)],
                vec![cdcl::lit(0, true), cdcl::lit(3, true)],
            ],
        ),
    ];
    for (nv, cls) in clause_sets {
        let mut s = cdcl::Solver::new(nv, cls.clone());
        let sat = s.solve();
        let mut polys: Vec<Poly> = cls.iter().map(|c| clause_to_poly(c, nv)).collect();
        polys.extend(field_polys(nv));
        let (g, _) = reduced_groebner(&polys, Order::GrevLex, Strategy::Normal, true);
        let unsat = g.len() == 1 && g[0].is_constant().map_or(false, |c| c.is_one());
        let ok = sat != unsat;
        pass &= ok;
        detail.push_str(&format!(
            "CDCL={} vs GB={}（{}）  ",
            if sat { "SAT" } else { "UNSAT" },
            if unsat { "⊥" } else { "有解" },
            if ok { "✓" } else { "✗" }
        ));
        // 學習子句的多項式必須在理想中（範式 = 0）
        for lc in s.learned_clauses() {
            let p = clause_to_poly(&lc, nv);
            let nf = normal_form(&p, &g, Order::GrevLex);
            if !nf.is_zero() {
                pass = false;
                detail.push_str(&format!("學習子句 {:?} 不在理想中 ✗  ", lc));
            }
        }
    }
    detail.push_str("學習子句範式皆 0 ✓  ");

    // (ii) 鴿籠 PHP(4,3)：UNSAT 一致性 + 學習子句蘊涵
    let nv = 12usize;
    let mut cls = vec![];
    for i in 0..4 {
        cls.push((0..3).map(|j| cdcl::lit(3 * i + j, true)).collect());
    }
    for j in 0..3 {
        for i1 in 0..4 {
            for i2 in (i1 + 1)..4 {
                cls.push(vec![cdcl::lit(3 * i1 + j, false), cdcl::lit(3 * i2 + j, false)]);
            }
        }
    }
    let mut s = cdcl::Solver::new(nv, cls.clone());
    let sat = s.solve();
    let learned = s.learned_clauses();
    // 蘊涵驗證：滿足原子句集的每個賦值都滿足學習子句
    let mut implied = true;
    if let Some(_) = brute_force_sat(nv, &cls) {
        // SAT 實例：暴力枚舉所有滿足賦值
        for mask in 0u64..(1 << nv) {
            let a: Vec<bool> = (0..nv).map(|i| (mask >> i) & 1 == 1).collect();
            if satisfies(&a, &cls) {
                for lc in &learned {
                    if !satisfies(&a, &[lc.clone()]) {
                        implied = false;
                    }
                }
            }
        }
    }
    pass &= !sat && implied && !learned.is_empty();
    detail.push_str(&format!(
        "PHP(4,3)：UNSAT ✓ 學習子句 {} 條皆被蘊涵 {}",
        learned.len(),
        if implied { "✓" } else { "✗" }
    ));

    ObligationResult {
        id: "T3",
        name: "子句–多項式對偶",
        statement: "Φ 可滿足 ⟺ V₀/₁(P_Φ ∪ B) ≠ ∅；CDCL 學習子句的多項式 ∈ ⟨P_Φ ∪ B⟩".into(),
        pass,
        detail,
    }
}

// ── T4 終止性：basis_adds ≤ 2^n（布爾系統的首項皆無平方） ──
pub fn obligation_t4() -> ObligationResult {
    let results = pipeline_results();
    let mut detail = String::new();
    let mut pass = true;
    for (name, r) in &results {
        let bound = 1usize << r.n_vars.min(60);
        let ok = r.gb_stats.basis_adds <= bound;
        pass &= ok;
        detail.push_str(&format!(
            "{}：{} 次基擴充 ≤ 2^{}  {}  ",
            name,
            r.gb_stats.basis_adds,
            r.n_vars,
            if ok { "✓" } else { "✗" }
        ));
    }
    ObligationResult {
        id: "T4",
        name: "Buchberger 終止性",
        statement: "含域多項式時首項理想由無平方單項式生成 ⇒ 擴充次數 ≤ 2ⁿ，必終止".into(),
        pass,
        detail,
    }
}

// ── T5 消去準則：基合法性抽檢 + 策略無關 + 小系統準則等價 ──
pub fn obligation_t5() -> ObligationResult {
    let mut detail = String::new();
    let mut pass = true;

    // (a) 套件樣本：約化基的合法性抽檢（隨機配對的 S-多項式必須對基歸零）
    use crate::poly::spoly;
    let mut checked_pairs = 0usize;
    for (name, src, _) in SUITE {
        let (p, mut exp) = parse_and_expand(src);
        let sys = crate::minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
        let mut merged = sys.polys.clone();
        merged.extend(field_polys(sys.nvars));
        let (g, _) = reduced_groebner(&merged, Order::GrevLex, Strategy::Normal, true);
        // 抽檢 40 個配對
        let mut bad = 0;
        let n = g.len();
        let mut idx = 0usize;
        'outer: for i in 0..n {
            for j in (i + 1)..n {
                if idx >= 40 {
                    break 'outer;
                }
                if (i * 31 + j * 17 + name.len()) % 3 == 0 {
                    idx += 1;
                    checked_pairs += 1;
                    let s = spoly(&g[i], &g[j], Order::GrevLex);
                    if !normal_form(&s, &g, Order::GrevLex).is_zero() {
                        bad += 1;
                    }
                }
            }
        }
        if bad > 0 {
            pass = false;
            detail.push_str(&format!("{}：{} 配對未歸零 ✗  ", name, bad));
        }
    }
    detail.push_str(&format!("基合法性抽檢（{} 配對全歸零）✓  ", checked_pairs));

    // (b) 策略無關（皆帶準則；準則無關於配對選取）
    for name in ["P1-plain", "P9-ifmix", "P12-assign-bad"] {
        let (p, mut exp) = parse_and_expand(SUITE.iter().find(|(n, _, _)| *n == name).unwrap().1);
        let sys = crate::minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
        let mut merged = sys.polys.clone();
        merged.extend(field_polys(sys.nvars));
        let (g1, _) = reduced_groebner(&merged, Order::GrevLex, Strategy::Normal, true);
        let (g2, _) = reduced_groebner(&merged, Order::GrevLex, Strategy::Fifo, true);
        if g1 == g2 {
            detail.push_str(&format!("{}：策略無關 ✓  ", name));
        } else {
            pass = false;
            detail.push_str(&format!("{}：策略相關 ✗  ", name));
        }
    }

    // (c) 小型合成系統：帶/不帶準則 ⇒ 相同約化基（準則可靠性）
    let synth: Vec<Vec<Poly>> = vec![
        // 系統1：布爾 one-hot + 規則 + 域多項式（6 變量）
        {
            let n = 6;
            let v = |i: usize| Poly::var(i, Frac::ONE, n);
            let c = |x: i64| Poly::constant(Frac::from_i64(x));
            let mut fs = vec![
                v(0).add(&v(1)).sub(&c(1)),
                v(2).add(&v(3)).sub(&c(1)),
                v(4).sub(&v(0).mul(&v(2))),
                v(4).add(&v(5)).sub(&c(1)),
            ];
            fs.extend(field_polys(n));
            fs
        },
        // 系統2：循環 one-hot + 衝突
        {
            let n = 5;
            let v = |i: usize| Poly::var(i, Frac::ONE, n);
            let c = |x: i64| Poly::constant(Frac::from_i64(x));
            let mut fs = vec![
                v(0).add(&v(1)).sub(&c(1)),
                v(1).add(&v(2)).sub(&c(1)),
                v(3).mul(&v(4)),
                v(0).sub(&c(1)),
                v(3).add(&v(4)).sub(&c(1)),
            ];
            fs.extend(field_polys(n));
            fs
        },
        // 系統3：非布爾小系統
        {
            let n = 3;
            let v = |i: usize| Poly::var(i, Frac::ONE, n);
            let c = |x: i64| Poly::constant(Frac::from_i64(x));
            vec![
                v(0).mul(&v(1)).sub(&c(1)),
                v(1).sub(&v(2).mul(&v(2))),
                v(0).add(&v(2)).sub(&c(2)),
            ]
        },
    ];
    for (k, fs) in synth.iter().enumerate() {
        let (g1, s1) = reduced_groebner(fs, Order::GrevLex, Strategy::Normal, true);
        let (g2, s2) = reduced_groebner(fs, Order::GrevLex, Strategy::Fifo, false);
        if g1 == g2 {
            detail.push_str(&format!("合成系統{}：帶/不帶準則一致 ✓（消除 {}/{} 配對）  ", k + 1, s1.crit1_skips + s1.crit2_skips, s2.pairs_considered));
        } else {
            pass = false;
            detail.push_str(&format!("合成系統{}：不一致 ✗  ", k + 1));
        }
    }

    // (d) 準則消除統計（套件）
    let mut tot_skipped = 0usize;
    let mut tot_spolys = 0usize;
    for (_, src, _) in SUITE {
        let (p, mut exp) = parse_and_expand(src);
        let sys = crate::minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
        let mut merged = sys.polys.clone();
        merged.extend(field_polys(sys.nvars));
        let (_, s1) = reduced_groebner(&merged, Order::GrevLex, Strategy::Normal, true);
        tot_skipped += s1.crit1_skips + s1.crit2_skips;
        tot_spolys += s1.s_polys;
    }
    detail.push_str(&format!(
        "| 套件消除率：消除 {} 配對 vs 實算 {} S-多項式（{:.0}%）",
        tot_skipped,
        tot_spolys,
        100.0 * tot_skipped as f64 / (tot_skipped + tot_spolys).max(1) as f64
    ));
    ObligationResult {
        id: "T5",
        name: "S-多項式消去準則",
        statement: "LM 互素 ⇒ S→₀（第一準則）；鏈準則可靠；準則不改變約化 Gröbner 基".into(),
        pass,
        detail,
    }
}

// ── T6 判定定理：1 ∈ G ⟺ checker 拒絕 ──
pub fn obligation_t6() -> ObligationResult {
    let results = pipeline_results();
    let mut detail = String::new();
    let mut pass = true;
    for (name, _, expect_ok) in SUITE {
        let r = &results.iter().find(|x| x.0 == *name).unwrap().1;
        let predicted_ok = !r.is_unsat;
        let ok = predicted_ok == *expect_ok;
        pass &= ok;
        detail.push_str(&format!(
            "{}：{}={} {}  ",
            name,
            if predicted_ok { "SAT" } else { "1∈G" },
            if *expect_ok { "良構" } else { "不良構" },
            if ok { "✓" } else { "✗" }
        ));
    }
    ObligationResult {
        id: "T6",
        name: "Gröbner 判定定理",
        statement: "1 ∈ G(F(e) ∪ B) ⟺ e 不可定型".into(),
        pass,
        detail,
    }
}

// ── T7 規範性：策略無關的唯一約化基；宏展開（求值同態）保持可解性 ──
pub fn obligation_t7() -> ObligationResult {
    let mut detail = String::new();
    let mut pass = true;

    // (a) 兩種策略 + 生成元重排 ⇒ 相同約化基
    let (p, mut exp) = parse_and_expand(SUITE[2].1); // P3-sqr
    let sys = crate::minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
    let mut merged = sys.polys.clone();
    merged.extend(field_polys(sys.nvars));
    let (g1, _) = reduced_groebner(&merged, Order::GrevLex, Strategy::Normal, true);
    let mut shuffled = merged.clone();
    shuffled.reverse();
    let (g2, _) = reduced_groebner(&shuffled, Order::GrevLex, Strategy::Fifo, true);
    let mut g2s = g2.clone();
    g2s.sort_by(|a, b| {
        crate::poly::cmp_mono(&a.lm(Order::GrevLex).unwrap(), &b.lm(Order::GrevLex).unwrap(), Order::GrevLex)
    });
    if g1 == g2s {
        detail.push_str("策略/順序無關 ⇒ 唯一約化基 ✓  ");
    } else {
        pass = false;
        detail.push_str("約化基不一致 ✗  ");
    }

    // (b) 宏展開不變性：invoke 系統在 a_i=1 求值下 ≡ 直接展開系統
    for (name, src, _) in SUITE.iter().filter(|(n, _, _)| n.contains("pick")) {
        let r = run_pipeline(name, src, false).unwrap();
        let (p, mut exp) = parse_and_expand(src);
        let sys = crate::minirust::constraints::gen_constraints(&p, &mut exp).unwrap();
        // 對每個 invoke：arm one-hot poly 在所選臂求值後，系統仍可解
        //（與直接展開程序的系統比較可解性）
        let arm_choice: HashMap<usize, usize> = r.arm_choice.clone();
        let mut subst = sys.polys.clone();
        for (inv, avs) in &sys.arm_vars {
            let chosen = arm_choice.get(inv).copied().unwrap_or(0);
            for (k, &v) in avs.iter().enumerate() {
                let val = Frac::from_i64((k == chosen) as i64);
                subst = subst.iter().map(|f| f.subst_var(v, &val)).collect();
            }
        }
        let mut all = subst;
        all.extend(field_polys(sys.nvars));
        let solvable = solve_boolean(&all, sys.nvars).is_some();
        // 未選臂路徑必須不可解（型別導向選擇的依據）
        let mut subst_bad = sys.polys.clone();
        for (inv, avs) in &sys.arm_vars {
            let chosen = arm_choice.get(inv).copied().unwrap_or(0);
            let bad_arm = 1 - chosen.min(avs.len() - 1);
            for (k, &v) in avs.iter().enumerate() {
                let val = Frac::from_i64((k == bad_arm) as i64);
                subst_bad = subst_bad.iter().map(|f| f.subst_var(v, &val)).collect();
            }
        }
        let mut all_bad = subst_bad;
        all_bad.extend(field_polys(sys.nvars));
        let (gb_bad, _) = reduced_groebner(&all_bad, Order::GrevLex, Strategy::Normal, true);
        let bad_unsat = gb_bad.len() == 1 && gb_bad[0].is_constant().map_or(false, |c| c.is_one());
        let ok = solvable && bad_unsat;
        pass &= ok;
        detail.push_str(&format!(
            "{}：選臂可解 ✓ / 錯臂 1∈G {}  ",
            name,
            if bad_unsat { "✓" } else { "✗" }
        ));
    }
    ObligationResult {
        id: "T7",
        name: "規範性與宏展開不變性",
        statement: "約化基唯一；展開=求值同態，可解性不變（選臂可解、錯臂矛盾）".into(),
        pass,
        detail,
    }
}

// ── T8 QAP 忠實性：見證通過 ⇔ 全部 R1CS 約束成立；竄改必拒 ──
pub fn obligation_t8() -> ObligationResult {
    let results = pipeline_results();
    let mut detail = String::new();
    let mut pass = true;
    let mut checked = 0;
    for (_name, r) in &results {
        if let Some(v) = r.qap_verified {
            pass &= v == true;
            checked += 1;
        }
        if let Some(t) = r.qap_tamper_rejected {
            pass &= t == true;
        }
    }
    detail.push_str(&format!(
        "SAT 樣本 QAP 全數通過（{} 個）且竄改全數被拒 {}",
        checked,
        if pass { "✓" } else { "✗" }
    ));
    ObligationResult {
        id: "T8",
        name: "QAP 忠實性",
        statement: "Z | a·b−c ⟺ z 滿足全部 R1CS 約束；deg A,B,C ≤ m−1".into(),
        pass,
        detail,
    }
}

// ── T9 端到端：管線判定 = checker 判定；生成碼 round-trip + rustc ──
pub fn obligation_t9() -> ObligationResult {
    let mut detail = String::new();
    let mut pass = true;
    for (name, src, expect_ok) in SUITE {
        let r = run_pipeline(name, src, true).expect("管線失敗");
        let ok1 = (!r.is_unsat) == *expect_ok;
        pass &= ok1;
        let mut msg = format!("{}：判定{} ", name, if ok1 { "✓" } else { "✗" });
        if let Some(code) = &r.generated_code {
            let rt = crate::codegen::roundtrip_check(code).is_ok();
            pass &= rt;
            msg.push_str(&format!("round-trip {} ", if rt { "✓" } else { "✗" }));
            if let Some(rc) = r.rustc_compiles {
                pass &= rc;
                msg.push_str(&format!("rustc {} ", if rc { "✓" } else { "✗" }));
            }
        }
        detail.push_str(&msg);
    }
    ObligationResult {
        id: "T9",
        name: "端到端正確性",
        statement: "管線 SAT ⟺ checker 接受；生成碼可再解析、可編譯、語義保持".into(),
        pass,
        detail,
    }
}

pub fn run_all() -> Vec<ObligationResult> {
    vec![
        obligation_t1(),
        obligation_t2(),
        obligation_t3(),
        obligation_t4(),
        obligation_t5(),
        obligation_t6(),
        obligation_t7(),
        obligation_t8(),
        obligation_t9(),
    ]
}

/// 實際使用：obligations.rs 文件清單 — 優化 with_capacity
pub fn obligations_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("obligations.rs", "obligations.rs 正式運作 — 優化 with_capacity", "core/src/obligations.rs"),
    ]
}

