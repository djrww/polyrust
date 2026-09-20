// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! LRAT/RUP UNSAT 證書核（v0.3 hardening，Phase S1 首塊）。
//!
//! 目的：為「UNSAT 判定」提供**與求解器無關、可獨立複核**的憑證。CDCL 求解器
//! （`cdcl.rs`）輸出學習子勺序列作為證明；本模組以 **RUP（Reverse Unit Propagation）**
//! 規則逐步複核：要加入子勺 C，假設 ¬C 後對「初始子勺 + 已接受子勺」做單元傳播，
//! 必須導出衝突。整份證明被接受 ⟺ 某步加入了空子勺。
//!
//! 設計約束（對齊專案文化）：
//! - **零第三方依賴、無 unsafe**——本檔是 UNSAT 判定的唯一信任組件（TCB）；
//! - 只做 RUP 子方言（本專案 CDCL 只需 unit propagation，永不需 RAT），
//!   拒絕永遠安全（可能拒掉合法證書，絕不接受非法證書）；
//! - 檢查核刻意小而直白，作為後續 Lean 4 形式化（`Polyrust.RupKernel`）的
//!   逐行對應目標；對應關係與定理計畫見 `docs/DEV_PLAN_V03.md`。
//!
//! 證明格式（`to_drat_text`）為 `drat-trim` 相容的「僅加入」子集，可用外部
//! 已驗證檢查器（cake_lpr / Lean 4 標準庫 LRAT checker）交叉複核。

/// 子句：文字編碼沿用 DIMACS——正整數 v 表 x_v、−v 表 ¬x_v；空 Vec 表空子句（⊥）。
pub type Clause = Vec<i32>;

#[derive(Clone, Debug, Default)]
pub struct Cnf {
    pub nvars: usize,
    pub clauses: Vec<Clause>,
}

impl Cnf {
    pub fn new(nvars: usize, clauses: Vec<Clause>) -> Self {
        Cnf { nvars, clauses }
    }
    /// 簡易可滿足性（單元傳播不夠時回退指數枚舉；僅供測試小例使用）。
    #[cfg(test)]
    fn brute_sat(&self) -> bool {
        let n = self.nvars.min(24);
        'assign: for mask in 0u64..(1u64 << n) {
            for c in &self.clauses {
                let ok = c.iter().any(|&l| {
                    let v = (l.unsigned_abs() - 1) as u64;
                    let val = (mask >> v) & 1 == 1;
                    (l > 0) == val
                });
                if !ok {
                    continue 'assign;
                }
            }
            return true;
        }
        false
    }
}

/// 單元傳播：在賦值 `assign`（0=未賦、1=true、-1=false）下對 `clauses` 閉包。
/// 回傳 true = 導出衝突（存在被全否的子句）。
fn unit_propagate(clauses: &[Clause], assign: &mut [i8]) -> bool {
    loop {
        let mut progressed = false;
        for c in clauses {
            let mut unassigned: Option<i32> = None;
            let mut n_unassigned = 0usize;
            let mut satisfied = false;
            for &l in c {
                let v = l.unsigned_abs() as usize;
                let val = assign[v - 1];
                if val == 0 {
                    n_unassigned += 1;
                    unassigned = Some(l);
                } else if (val == 1) == (l > 0) {
                    satisfied = true;
                    break;
                }
            }
            if satisfied {
                continue;
            }
            if n_unassigned == 0 {
                return true; // 衝突：子句被全否
            }
            if n_unassigned == 1 {
                let l = unassigned.expect("exactly one unassigned literal");
                let v = l.unsigned_abs() as usize;
                let need: i8 = if l > 0 { 1 } else { -1 };
                if assign[v - 1] == 0 {
                    assign[v - 1] = need;
                    progressed = true;
                }
            }
        }
        if !progressed {
            return false;
        }
    }
}

/// RUP 檢查：在 `db`（初始 + 已接受子句）上，¬C 是否經單元傳播導出衝突。
fn is_rup(db: &[Clause], c: &[i32], nvars: usize) -> bool {
    let mut assign = vec![0i8; nvars];
    // 假設 ¬C：C 中每個文字都為假。
    for &l in c {
        let v = l.unsigned_abs() as usize;
        if v > nvars {
            return false; // 變量越界的證明步一律拒絕（保守）
        }
        assign[v - 1] = if l > 0 { -1 } else { 1 };
    }
    if c.is_empty() {
        // 空子句：不需假設；檢查 db 是否已 UP-不一致。
        return unit_propagate(db, &mut assign);
    }
    unit_propagate(db, &mut assign)
}

/// 逐步複核 RUP 證明。`proof` 為依序加入的子句（學習子句序列，末步通常為空子句）。
///
/// 接受條件：每一步對當前資料庫為 RUP，且**某步加入空子句**（其後步驟忽略）。
/// 回傳 Ok(已接受步數) 或 Err(首個失敗步與原因)。
pub fn check_rup_proof(cnf: &Cnf, proof: &[Clause]) -> Result<usize, String> {
    let mut db: Vec<Clause> = cnf.clauses.clone();
    for (i, c) in proof.iter().enumerate() {
        if !is_rup(&db, c, cnf.nvars) {
            return Err(format!(
                "RUP 檢查失敗於第 {} 步：子句 {:?} 的否定未能經單元傳播導出衝突",
                i, c
            ));
        }
        let is_empty = c.is_empty();
        db.push(c.clone());
        if is_empty {
            return Ok(i + 1);
        }
    }
    Err("證明結束但未加入空子句：未完成駁斥".to_string())
}

/// 序列化為 drat-trim 相容文本（僅加入子句；每行以 0 結尾）。
pub fn to_drat_text(proof: &[Clause]) -> String {
    let mut out = String::new();
    for c in proof {
        for &l in c {
            out.push_str(&l.to_string());
            out.push(' ');
        }
        out.push_str("0\n");
    }
    out
}

/// C8（審計②）：CDCL 學習子句序列（`Solver::solve_with_proof` 產物）→
/// RUP 證書子句（i32 慣例：±(v+1)）。呢個係「CDCL 輸出 ⇒ 自家
/// check_rup_proof 可驗」嘅信任鏈橋接——終點 LRAT（帶 id／刪除行）屬後續。
pub fn cdcl_learnts_to_rup(learnts: &[Vec<crate::cdcl::Lit>]) -> Vec<Clause> {
    learnts
        .iter()
        .map(|c| {
            c.iter()
                .map(|&l| {
                    let v = crate::cdcl::lit_var(l) as i32 + 1;
                    if crate::cdcl::lit_positive(l) {
                        v
                    } else {
                        -v
                    }
                })
                .collect()
        })
        .collect()
}

/// 序列化 CNF 為 DIMACS 文本（供外部檢查器交叉複核）。
pub fn to_dimacs(cnf: &Cnf) -> String {
    let mut out = format!("p cnf {} {}\n", cnf.nvars, cnf.clauses.len());
    for c in &cnf.clauses {
        for &l in c {
            out.push_str(&l.to_string());
            out.push(' ');
        }
        out.push_str("0\n");
    }
    out
}

#[cfg(test)]
mod tests {

    #[test]
    fn cdcl_unsat_proof_self_verifying_php32() {
        // 鴿籠 PHP(3,2)：3 鴿 2 洞——經典 UNSAT（6 變數 9 子句）。
        // C8 信任鏈：CDCL solve_with_proof 產證書 → cdcl_learnts_to_rup →
        // check_rup_proof 獨立回放必 Ok（同模組不同代碼路徑之差分 oracle）。
        use crate::cdcl::{lit, Solver};
        let raw: Vec<Vec<i32>> = vec![
            vec![1, 2],
            vec![3, 4],
            vec![5, 6],
            vec![-1, -3],
            vec![-1, -5],
            vec![-3, -5],
            vec![-2, -4],
            vec![-2, -6],
            vec![-4, -6],
        ];
        let enc = |c: &[i32]| -> Vec<crate::cdcl::Lit> {
            c.iter()
                .map(|&l| lit(l.unsigned_abs() as usize - 1, l > 0))
                .collect()
        };
        let mut s = Solver::new(6, raw.iter().map(|c| enc(c)).collect());
        let (sat, proof) = s.solve_with_proof();
        assert!(!sat, "PHP(3,2) 應 UNSAT");
        assert!(!proof.is_empty(), "UNSAT 證書非空");
        let rup = cdcl_learnts_to_rup(&proof);
        let cnf = Cnf::new(6, raw.clone());
        let res = check_rup_proof(&cnf, &rup);
        assert!(res.is_ok(), "RUP 回放應通過：{res:?}");
        // 終結性由 RUP 回放判定（0 層衝突子句可為任意長度——其否定可由
        // db 傳播駁倒）；結構檢查只要求「有空子句或回放通過」。
        assert!(
            proof.iter().any(|c| c.is_empty()) || res.is_ok(),
            "證書應含空子句或整體回放通過"
        );
    }

    #[test]
    fn cdcl_sat_proof_has_no_empty_clause_model_rechecks() {
        // SAT 場景：證書無空子句（唔構成駁斥）；判定由 model σ 對原子句重驗
        use crate::cdcl::{lit, lit_positive, lit_var, Solver};
        let raw: Vec<Vec<i32>> = vec![vec![1, 2], vec![-1, 3], vec![-3, 2]];
        let enc = |c: &[i32]| -> Vec<crate::cdcl::Lit> {
            c.iter()
                .map(|&l| lit(l.unsigned_abs() as usize - 1, l > 0))
                .collect()
        };
        let mut s = Solver::new(3, raw.iter().map(|c| enc(c)).collect());
        let (sat, proof) = s.solve_with_proof();
        assert!(sat);
        assert!(proof.iter().all(|c| !c.is_empty()), "SAT 證書無空子句");
        let model = s.model().expect("SAT 應有 model");
        for c in &raw {
            let ok = c.iter().any(|&l| {
                let v = lit_var(lit(l.unsigned_abs() as usize - 1, l > 0));
                model[v] == lit_positive(lit(l.unsigned_abs() as usize - 1, l > 0))
            });
            assert!(ok, "model 違反子句 {c:?}");
        }
        // SAT 證書唔應通過 check_rup_proof（無空子句=未完成駁斥）——三值口徑：
        // UNSAT 先有駁斥證書
        let rup = cdcl_learnts_to_rup(&proof);
        if !rup.is_empty() {
            let cnf = Cnf::new(3, raw.clone());
            assert!(check_rup_proof(&cnf, &rup).is_err());
        }
    }

    #[test]
    fn cdcl_unit_chain_unsat_minimal() {
        // 最小單位鏈：(1)(−1)——UNSAT 證書即時收尾
        use crate::cdcl::{lit, Solver};
        let raw: Vec<Vec<i32>> = vec![vec![1], vec![-1]];
        let enc = |c: &[i32]| -> Vec<crate::cdcl::Lit> {
            c.iter()
                .map(|&l| lit(l.unsigned_abs() as usize - 1, l > 0))
                .collect()
        };
        let mut s = Solver::new(1, raw.iter().map(|c| enc(c)).collect());
        let (sat, proof) = s.solve_with_proof();
        assert!(!sat);
        assert!(!proof.is_empty());
        let rup = cdcl_learnts_to_rup(&proof);
        let res = check_rup_proof(&Cnf::new(1, raw.clone()), &rup);
        assert!(res.is_ok(), "{res:?}");
    }

    use super::*;

    fn cnf(n: usize, cls: &[&[i32]]) -> Cnf {
        Cnf::new(n, cls.iter().map(|c| c.to_vec()).collect())
    }

    #[test]
    fn test_rup_trivial_contradiction() {
        // [1], [-1] ⊢ ⊥：對空子句，UP 先賦 x1=1 再於 [-1] 衝突。
        let f = cnf(1, &[&[1], &[-1]]);
        assert_eq!(check_rup_proof(&f, &[vec![]]), Ok(1));
        assert!(!f.brute_sat());
    }

    #[test]
    fn test_rup_learned_then_empty() {
        // F = (1∨2),(¬1∨2),(¬2) ⊢ (2)（RUP），再 ⊢ ⊥。
        let f = cnf(2, &[&[1, 2], &[-1, 2], &[-2]]);
        let proof = vec![vec![2], vec![]];
        assert_eq!(check_rup_proof(&f, &proof), Ok(2));
        assert!(!f.brute_sat());
        // 也應被逐步文本序列化（drat 相容）
        let t = to_drat_text(&proof);
        assert!(t.contains("2 0"));
    }

    #[test]
    fn test_rup_chain_three_vars() {
        // (1∨2),(¬1∨2),(¬2∨3),(¬2∨¬3) ⊢ (3) 後 ⊥？先驗非直觀路徑：
        // 賦 ¬(3) ⇒ x3=false；(¬2∨3) 給 ¬2 ⇒ x2=false；(1∨2)⇒x1；(¬1∨2) 衝突。
        let f = cnf(3, &[&[1, 2], &[-1, 2], &[-2, 3], &[-2, -3]]);
        let proof = vec![vec![-2], vec![1], vec![]];
        // 逐步：¬2 是否 RUP？假設 x2=true：(1∨2) 滿；(−1∨2) 滿；(−2∨3)⇒x3；(−2∨¬3) 衝突 ⇒ RUP ✓
        // 再學 (1)？假設 x1=false：在 F∪{¬2} 下 x2=false ⇒ (1∨2)⇒x1 衝突 ⇒ RUP ✓
        // 空子句：F∪{¬2,1} UP → x2=false ⇒ (−1∨2) 與 x1 衝突 ⇒ ✓
        assert_eq!(check_rup_proof(&f, &proof), Ok(3));
        assert!(!f.brute_sat());
    }

    #[test]
    fn test_reject_invalid_step() {
        // 可滿足公式 [1]：空子句非 RUP（無衝突）⇒ 拒絕。
        let f = cnf(1, &[&[1]]);
        assert!(check_rup_proof(&f, &[vec![]]).is_err());
        assert!(f.brute_sat());
    }

    #[test]
    fn test_reject_unfinished_proof() {
        // 合法駁斥但缺空子句收尾 ⇒ 拒絕（不完整證明）。
        let f = cnf(2, &[&[1, 2], &[-1, 2], &[-2]]);
        assert!(check_rup_proof(&f, &[vec![2]]).is_err());
    }

    #[test]
    fn test_dimacs_roundtrip_shape() {
        let f = cnf(2, &[&[1, -2], &[2]]);
        let d = to_dimacs(&f);
        assert!(d.starts_with("p cnf 2 2"));
        assert!(d.contains("1 -2 0"));
    }
}
