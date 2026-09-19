//! 定理 T10（組合性）：模塊化編碼與組合終止界。
//!
//! 動機（見 docs/THEOREMS.md §6b）：過渡到真 rustc（MIR + Chalk）後，單一
//! 系統的位元變量數 n 會急劇膨脹，T4 的 2ⁿ 界雖然仍然成立，但已無實用意義。
//! 解法不是改進界，而是**改組件粒度**：跟隨編碼的天然邊界（每個函數體、
//! 每條 obligation）把系統切成變量集兩兩不相交的子系統，各自獨立求解。
//!
//! 本模組提供：
//! * [`components_of`] —— 以變量共現（同一多項式內出現）做並查集分解；
//! * [`run_decomposed`] —— 逐組件跑 Buchberger（各帶自己的域多項式），
//!   再驗證「各組件基之並 = 全系統的 Gröbner 基」；
//! * 組合界 **Σᵢ 2^{nᵢ} ≤ 2^{Σᵢ nᵢ}**（每個組件各自的 T4 界之和），
//!   只要組件位元數有常數上界，總成本即線性於程式規模。
//!
//! 「並基仍是 Gröbner 基」的代數依據：跨組件配對的首項使用不相交變量
//! ⇒ 互素 ⇒ S-多項式由第一準則（定理 T5(a)）必然歸零。本模組不依賴此論證，
//! 而是**逐對直接計算驗證**（`verify_union_basis`），論證與驗證雙軌並行。

use crate::frac::Frac;
use crate::groebner::{buchberger, field_polys, normal_form, reduce_to_reduced_gb, GroebnerStats, Strategy};
use crate::poly::{spoly, Order, Poly};
use std::collections::HashMap;

/// 2 的冪（u128 安全；n ≥ 127 時回傳 None）。
pub fn pow2_u128(n: usize) -> Option<u128> {
    if n >= 127 {
        None
    } else {
        Some(1u128 << n)
    }
}

/// 一個多項式的變量支撐（出現過非零指數的變量）。
pub fn support(p: &Poly) -> Vec<usize> {
    let mut seen: Vec<usize> = Vec::new();
    for (m, _) in &p.terms {
        for (i, &e) in m.iter().enumerate() {
            if e > 0 && !seen.contains(&i) {
                seen.push(i);
            }
        }
    }
    seen.sort_unstable();
    seen
}

/// 變量共現分解（並查集）：同一多項式內出現的變量歸同一組件。
/// 回傳：各組件的（全局）變量索引表（排序）＋未出現在任何約束中的變量數。
pub fn components_of(gens: &[Poly], nvars: usize) -> (Vec<Vec<usize>>, usize) {
    let mut parent: Vec<usize> = (0..nvars).collect();
    fn find(p: &mut Vec<usize>, mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    for g in gens {
        let s = support(g);
        if s.len() >= 2 {
            let r0 = find(&mut parent, s[0]);
            for &v in &s[1..] {
                let rv = find(&mut parent, v);
                parent[rv] = r0;
            }
        }
    }
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut unused = 0usize;
    let mut appears = vec![false; nvars];
    for g in gens {
        for v in support(g) {
            appears[v] = true;
        }
    }
    for v in 0..nvars {
        if !appears[v] {
            unused += 1;
            continue;
        }
        let r = find(&mut parent, v);
        groups.entry(r).or_default().push(v);
    }
    let mut comps: Vec<Vec<usize>> = groups.into_values().collect();
    for c in comps.iter_mut() {
        c.sort_unstable();
    }
    comps.sort_by_key(|c| c[0]);
    (comps, unused)
}

fn remap_poly(p: &Poly, map: &HashMap<usize, usize>, nvars_local: usize) -> Poly {
    let terms: Vec<(Vec<u32>, Frac)> = p
        .terms
        .iter()
        .map(|(m, c)| {
            let mut m2 = vec![0u32; nvars_local];
            for (i, &e) in m.iter().enumerate() {
                if e != 0 {
                    if let Some(&j) = map.get(&i) {
                        m2[j] = e;
                    } else {
                        // 不應發生：調用方須保證支撐 ⊆ 組件
                        panic!("remap_poly：變量 {} 不在組件映射內", i);
                    }
                }
            }
            (m2, *c)
        })
        .collect();
    Poly::from_terms(terms)
}

fn unmap_poly(p: &Poly, vars: &[usize], nvars_global: usize) -> Poly {
    let terms: Vec<(Vec<u32>, Frac)> = p
        .terms
        .iter()
        .map(|(m, c)| {
            let mut m2 = vec![0u32; nvars_global];
            for (j, &e) in m.iter().enumerate() {
                if e != 0 {
                    m2[vars[j]] = e;
                }
            }
            (m2, *c)
        })
        .collect();
    Poly::from_terms(terms)
}

/// 單個組件的 Gröbner 結果。
#[derive(Clone)]
pub struct ComponentGb {
    /// 組件的全局變量索引。
    pub vars: Vec<usize>,
    /// 組件內變量數（局部）。
    pub nvars_local: usize,
    /// 送入的生成元數（含域多項式）。
    pub n_gens: usize,
    /// Buchberger 統計（`basis_adds` = 基擴充次數）。
    pub stats: GroebnerStats,
    /// 約化基（已映回全局變量索引）。
    pub basis: Vec<Poly>,
    /// 1 ∈ G（此組件不可解）。
    pub is_unsat: bool,
}

/// 組合求解報告。
#[derive(Clone)]
pub struct DecompReport {
    pub components: Vec<ComponentGb>,
    /// 未出現在任何約束中的變量數（不貢獻任何擴充）。
    pub unused_vars: usize,
    /// 各組件基之並（全局索引）。
    pub union_basis: Vec<Poly>,
    /// 並基是否經逐對 S-多項式驗證為全系統的 Gröbner 基。
    pub union_basis_is_gb: bool,
    /// 組合界 Σᵢ 2^{nᵢ}（None = 溢出）。
    pub bound_decomposed: Option<u128>,
    /// 逐組件實際總擴充次數。
    pub total_extensions: usize,
    /// 全系統位元數（naive T4 界 = 2^naive_bits）。
    pub naive_bits: usize,
    /// 任一組件 UNSAT ⇒ 全系統 UNSAT（理想含 1 的組件已使整體含 1）。
    pub any_unsat: bool,
}

/// 驗證一組多項式是否構成 Gröbner 基（Buchberger 準則的直接判定：
/// 所有配對的 S-多項式對基歸約為零）。
pub fn verify_union_basis(basis: &[Poly], ord: Order) -> (bool, usize) {
    let mut checked = 0usize;
    for i in 0..basis.len() {
        for j in (i + 1)..basis.len() {
            let s = spoly(&basis[i], &basis[j], ord);
            let r = normal_form(&s, basis, ord);
            checked += 1;
            if !r.is_zero() {
                return (false, checked);
            }
        }
    }
    (true, checked)
}

/// 值域模式：組件內每個變量的「域約束」由誰提供。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DomainMode {
    /// 經典布爾：自動加 xᵢ² − xᵢ（T4/T10、管線位元系統）。
    Boolean,
    /// 調用方已在 gens 內提供域約束（如消失多項式 ∏(x−v)，MIR 前端）；
    /// 不另加域多項式。`bound_decomposed` 由此為 None（調用方可自行以
    /// `vanishing::standard_bound_product` 計 ∏kᵢ）。
    Provided,
}

/// 組合求解：分解 → 逐組件 Buchberger（按 DomainMode 加域約束）→ 並基驗證。
pub fn run_decomposed_mode(
    gens: &[Poly],
    nvars: usize,
    ord: Order,
    strat: Strategy,
    mode: DomainMode,
) -> DecompReport {
    let (comps, unused) = components_of(gens, nvars);
    let mut components: Vec<ComponentGb> = Vec::with_capacity(comps.len());
    let mut union_basis: Vec<Poly> = Vec::new();

    for vars in comps {
        let n_local = vars.len();
        let map: HashMap<usize, usize> = vars.iter().enumerate().map(|(j, &v)| (v, j)).collect();
        // 收集屬於本組件的生成元（支撐 ⊆ 組件 ⇒ 全局變量 ∈ vars）
        let local_gens: Vec<Poly> = gens
            .iter()
            .filter(|g| support(g).iter().all(|v| map.contains_key(v)))
            .map(|g| remap_poly(g, &map, n_local))
            .collect();
        let mut sys = local_gens.clone();
        if mode == DomainMode::Boolean {
            sys.extend(field_polys(n_local));
        }
        let n_gens = sys.len();
        let (g, stats) = buchberger(&sys, ord, strat, true);
        let red = reduce_to_reduced_gb(g, ord);
        let is_unsat = red.len() == 1 && red[0].is_constant() == Some(Frac::ONE);
        let basis_global: Vec<Poly> = red.iter().map(|p| unmap_poly(p, &vars, nvars)).collect();
        union_basis.extend(basis_global.clone());
        components.push(ComponentGb {
            nvars_local: n_local,
            vars,
            n_gens,
            stats,
            basis: basis_global,
            is_unsat,
        });
    }

    let (union_basis_is_gb, _pairs) = verify_union_basis(&union_basis, ord);

    let mut bound_decomposed: Option<u128> = Some(0);
    let mut total_extensions = 0usize;
    let mut any_unsat = false;
    for c in &components {
        total_extensions += c.stats.basis_adds;
        any_unsat |= c.is_unsat;
        bound_decomposed = if mode == DomainMode::Boolean {
            match (bound_decomposed, pow2_u128(c.nvars_local)) {
                (Some(acc), Some(b)) => acc.checked_add(b),
                _ => None,
            }
        } else {
            None // Provided：界 = ∏kᵢ（值域大小積），由調用方計算
        };
    }

    DecompReport {
        naive_bits: nvars,
        components,
        unused_vars: unused,
        union_basis,
        union_basis_is_gb,
        bound_decomposed,
        total_extensions,
        any_unsat,
    }
}

/// 組合求解（布爾域模式；管線位元系統的預設入口）。
pub fn run_decomposed(gens: &[Poly], nvars: usize, ord: Order, strat: Strategy) -> DecompReport {
    run_decomposed_mode(gens, nvars, ord, strat, DomainMode::Boolean)
}

/// 對照組：不分組，直接對全系統跑 Buchberger（T4 的 2ⁿ 界直接適用）。
pub fn run_naive(gens: &[Poly], nvars: usize, ord: Order, strat: Strategy) -> GroebnerStats {
    let mut sys: Vec<Poly> = gens.to_vec();
    sys.extend(field_polys(nvars));
    let (_g, stats) = buchberger(&sys, ord, strat, true);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poly::Order;

    fn x(i: usize, n: usize) -> Poly {
        Poly::var(i, Frac::ONE, n)
    }

    /// 兩個完全不相交的布爾子系統（各 4 變量）。
    fn disjoint_system(n: usize) -> Vec<Poly> {
        let mut gs = Vec::new();
        // 組件 A（變量 0..4）：x2 = x0·x1，x3 = 1 − x2（反相）
        gs.push(x(0, n).mul(&x(1, n)).sub(&x(2, n)));
        gs.push(x(2, n).add(&x(3, n)).sub(&Poly::constant(Frac::ONE)));
        // 組件 B（變量 4..8）：x6 = x4 + x5（mod 布爾意義下由域多項式約束），x7 = x4·x6
        gs.push(x(4, n).add(&x(5, n)).sub(&x(6, n)));
        gs.push(x(4, n).mul(&x(6, n)).sub(&x(7, n)));
        gs
    }

    #[test]
    fn t10_decomposition_finds_two_components() {
        let gs = disjoint_system(8);
        let (comps, unused) = components_of(&gs, 8);
        assert_eq!(comps.len(), 2, "應分解為 2 個組件");
        assert_eq!(comps[0], vec![0, 1, 2, 3]);
        assert_eq!(comps[1], vec![4, 5, 6, 7]);
        assert_eq!(unused, 0);
    }

    #[test]
    fn t10_union_basis_is_groebner_basis() {
        let gs = disjoint_system(8);
        let rep = run_decomposed(&gs, 8, Order::GrevLex, Strategy::Normal);
        assert!(
            rep.union_basis_is_gb,
            "各組件約化基之並必須經逐對 S-多項式驗證為全系統 Gröbner 基"
        );
        assert!(!rep.any_unsat, "可解系統不應有 UNSAT 組件");
    }

    #[test]
    fn t10_extensions_respect_composition_bound() {
        let gs = disjoint_system(8);
        let rep = run_decomposed(&gs, 8, Order::GrevLex, Strategy::Normal);
        // 組合界：Σ 2^{nᵢ} = 2⁴ + 2⁴ = 32
        assert_eq!(rep.bound_decomposed, Some(32));
        // 實際擴充必須 ≤ 組合界
        assert!(
            (rep.total_extensions as u128) <= rep.bound_decomposed.unwrap(),
            "實際擴充 {} 超過組合界 {:?}",
            rep.total_extensions,
            rep.bound_decomposed
        );
        // 每個組件各自的 T4 界：2⁴ = 16
        for c in &rep.components {
            assert!(
                c.stats.basis_adds <= 1 << c.nvars_local,
                "組件 {:?} 擴充 {} 超過其局部界 2^{}",
                c.vars,
                c.stats.basis_adds,
                c.nvars_local
            );
        }
        // 組合界嚴格小於 naive 界 2⁸ = 256（模塊化收益）
        assert!(rep.bound_decomposed.unwrap() < (1u128 << rep.naive_bits));
    }

    #[test]
    fn t10_unsat_localized_to_one_component() {
        let n = 8;
        let mut gs = disjoint_system(n);
        // 在組件 B 內注入矛盾：x4 = 1 ∧ x4 = 0
        gs.push(x(4, n).sub(&Poly::constant(Frac::ONE)));
        gs.push(x(4, n).clone());
        let rep = run_decomposed(&gs, n, Order::GrevLex, Strategy::Normal);
        assert!(rep.any_unsat, "注入矛盾的系統必須判出 UNSAT");
        // UNSAT 必須局限於含矛盾的組件（變量 4..8），組件 A 不受污染
        for c in &rep.components {
            if c.vars == vec![0, 1, 2, 3] {
                assert!(!c.is_unsat, "無關組件不應被誤判 UNSAT");
            }
        }
        assert!(rep.union_basis_is_gb, "含 1 的並基仍是 Gröbner 基");
    }

    #[test]
    fn t10_unused_variables_cost_nothing() {
        // 10 個變量，但只有前 4 個受約束：未用變量不貢獻擴充、不進組合界
        let n = 10;
        let gs = vec![
            x(0, n).mul(&x(1, n)).sub(&x(2, n)),
            x(2, n).add(&x(3, n)).sub(&Poly::constant(Frac::ONE)),
        ];
        let rep = run_decomposed(&gs, n, Order::GrevLex, Strategy::Normal);
        assert_eq!(rep.unused_vars, 6);
        assert_eq!(rep.components.len(), 1);
        // 組合界 = 2⁴ = 16，而非 2¹⁰
        assert_eq!(rep.bound_decomposed, Some(16));
    }

    #[test]
    fn t10_naive_run_still_bounded_by_2n() {
        let gs = disjoint_system(8);
        let stats = run_naive(&gs, 8, Order::GrevLex, Strategy::Normal);
        // T4：全系統擴充 ≤ 2⁸ = 256（此處只是界存在性的冒煙檢查）
        assert!(stats.basis_adds <= 256);
    }
}
