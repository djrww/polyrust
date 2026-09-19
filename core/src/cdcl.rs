// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! CDCL（Conflict-Driven Clause Learning）SAT 求解器。
//! 雙監視文字傳播、VSIDS 活動度、1-UIP 衝突分析、相位保存。
//! 用於：宏臂選擇、借用互斥等布爾側條件；學習子句可轉成多項式後
//! 由 Gröbner 基代數驗證（定理 3 的機械自證）。

/// 文字編碼：變量 v 的正文字 = 2v，負文字 = 2v+1。
pub type Lit = u32;

pub fn lit(v: usize, positive: bool) -> Lit {
    2 * v as u32 + if positive { 0 } else { 1 }
}
pub fn lit_var(l: Lit) -> usize {
    (l / 2) as usize
}
pub fn lit_positive(l: Lit) -> bool {
    l % 2 == 0
}
pub fn lit_neg(l: Lit) -> Lit {
    l ^ 1
}

#[derive(Clone, Debug, Default)]
pub struct CdclStats {
    pub decisions: usize,
    pub propagations: usize,
    pub conflicts: usize,
    pub learned: usize,
}

pub struct Solver {
    n_vars: usize,
    clauses: Vec<Vec<Lit>>,
    n_orig: usize,
    watches: Vec<Vec<usize>>, // 每文字 → 監視的子句索引
    values: Vec<Option<bool>>,
    level: Vec<u32>,
    reason: Vec<Option<usize>>,
    trail: Vec<usize>,
    trail_lim: Vec<usize>,
    qhead: usize,
    activity: Vec<f64>,
    var_inc: f64,
    phase: Vec<bool>,
    stats: CdclStats,
    ok: bool,
    model: Option<Vec<bool>>,
}

impl Solver {
    pub fn new(n_vars: usize, clauses: Vec<Vec<Lit>>) -> Solver {
        let mut s = Solver {
            n_vars,
            clauses: vec![],
            n_orig: 0,
            watches: vec![vec![]; 2 * n_vars.max(1)],
            values: vec![None; n_vars],
            level: vec![0; n_vars],
            reason: vec![None; n_vars],
            trail: vec![],
            trail_lim: vec![],
            qhead: 0,
            activity: vec![0.0; n_vars],
            var_inc: 1.0,
            phase: vec![false; n_vars],
            stats: CdclStats::default(),
            ok: true,
            model: None,
        };
        for clause in clauses {
            s.add_clause_raw(clause);
        }
        s.n_orig = s.clauses.len();
        s
    }

    /// 加入子句。回傳存入子句表的索引（單元/恆真/空子句回傳 None）。
    pub fn add_clause_raw(&mut self, clause: Vec<Lit>) -> Option<usize> {
        let mut c: Vec<Lit> = Vec::new();
        for &l in &clause {
            if !c.contains(&l) {
                c.push(l);
            }
        }
        for &l in &c {
            if c.contains(&lit_neg(l)) {
                return None; // 恆真
            }
        }
        for &l in &c {
            assert!((l as usize) < self.watches.len(), "文字 {} 超出變量範圍", l);
        }
        if c.is_empty() {
            self.ok = false;
            return None;
        }
        if c.len() == 1 {
            let l = c[0];
            match self.value_of(l) {
                Some(false) => self.ok = false,
                Some(true) => {}
                None => self.enqueue(l, None),
            }
            return None;
        }
        let idx = self.clauses.len();
        self.clauses.push(c);
        self.watches[self.clauses[idx][0] as usize].push(idx);
        self.watches[self.clauses[idx][1] as usize].push(idx);
        Some(idx)
    }

    fn value_of(&self, l: Lit) -> Option<bool> {
        let v = self.values[lit_var(l)];
        if lit_positive(l) {
            v
        } else {
            v.map(|b| !b)
        }
    }

    fn current_level(&self) -> u32 {
        self.trail_lim.len() as u32
    }

    fn enqueue(&mut self, l: Lit, reason: Option<usize>) {
        let v = lit_var(l);
        debug_assert!(self.values[v].is_none());
        let val = lit_positive(l);
        self.values[v] = Some(val);
        self.level[v] = self.current_level();
        self.reason[v] = reason;
        self.phase[v] = val;
        self.trail.push(v);
    }

    /// 單元傳播。回傳衝突子句索引。
    fn propagate(&mut self) -> Option<usize> {
        while self.qhead < self.trail.len() {
            let v = self.trail[self.qhead];
            self.qhead += 1;
            let true_lit = lit(v, self.values[v].unwrap());
            let falsified = lit_neg(true_lit);
            let list = self.watches[falsified as usize].clone();
            let mut keep: Vec<usize> = vec![];
            let mut i = 0;
            while i < list.len() {
                let ci = list[i];
                i += 1;
                let c = self.clauses[ci].clone();
                debug_assert!(
                    c[0] == falsified || c[1] == falsified,
                    "陳舊監視：子句 {:?} 在 watches[{}] 但首兩文字均非之",
                    c,
                    falsified
                );
                let other = if c[0] == falsified { c[1] } else { c[0] };
                if self.value_of(other) == Some(true) {
                    keep.push(ci);
                    continue; // 子句已滿足
                }
                // 尋找新監視文字
                let mut moved = false;
                for (k, &l) in c.iter().enumerate().skip(2) {
                    if self.value_of(l) != Some(false) {
                        let mut c2 = c.clone();
                        // 交換（非覆寫）：被否證的文字換到 l 的原位置，
                        // 子句多重集不變——否則會丟失文字、子句變強，
                        // 回溯後造成不可靠剪枝（SAT 誤報 UNSAT）。
                        if c2[0] == falsified {
                            c2[0] = l;
                        } else {
                            c2[1] = l;
                        }
                        c2[k] = falsified;
                        self.clauses[ci] = c2;
                        self.watches[l as usize].push(ci);
                        moved = true;
                        break;
                    }
                }
                if moved {
                    continue;
                }
                keep.push(ci);
                // 單元或衝突
                match self.value_of(other) {
                    Some(false) => {
                        // 衝突
                        while i < list.len() {
                            keep.push(list[i]);
                            i += 1;
                        }
                        self.watches[falsified as usize] = keep;
                        self.stats.propagations += 1;
                        return Some(ci);
                    }
                    Some(true) => unreachable!(),
                    None => {
                        self.enqueue(other, Some(ci));
                        self.stats.propagations += 1;
                    }
                }
            }
            self.watches[falsified as usize] = keep;
        }
        None
    }

    fn bump(&mut self, v: usize) {
        self.activity[v] += self.var_inc;
        if self.activity[v] > 1e100 {
            for a in self.activity.iter_mut() {
                *a *= 1e-100;
            }
            self.var_inc *= 1e-100;
        }
    }

    /// 1-UIP 衝突分析（MiniSat 式）。回傳 (學習子句, 回跳層級)。
    fn analyze(&mut self, confl: usize) -> (Vec<Lit>, u32) {
        let mut learnt: Vec<Lit> = vec![];
        let mut seen = vec![false; self.n_vars];
        let mut path_c = 0usize;
        let mut p: Option<usize> = None;
        let mut cursor = self.trail.len();
        let mut cur = Some(confl);
        loop {
            let clause = match cur.take() {
                Some(ci) => {
                    if p.is_none() {
                        debug_assert!(
                            self.clauses[ci].iter().all(|&l| self.value_of(l) == Some(false)),
                            "衝突子句非全假：{:?}",
                            self.clauses[ci]
                        );
                    } else {
                        let pv = p.unwrap();
                        let implied = lit(pv, self.values[pv].unwrap());
                        debug_assert!(
                            self.clauses[ci]
                                .iter()
                                .all(|&l| l == implied || self.value_of(l) == Some(false)),
                            "原因子句非（蘊含文字真＋其餘全假）：{:?} implied={}",
                            self.clauses[ci],
                            implied
                        );
                    }
                    self.clauses[ci].clone()
                }
                None => {
                    let v = p.expect("原因鏈斷裂");
                    let ci = self.reason[v].expect("當前層變量無原因");
                    let implied = lit(v, self.values[v].unwrap());
                    debug_assert_eq!(
                        self.value_of(implied),
                        Some(true),
                        "原因子句蘊含文字應為真：v={} ci={}",
                        v,
                        ci
                    );
                    debug_assert!(
                        self.clauses[ci]
                            .iter()
                            .all(|&l| l == implied || self.value_of(l) == Some(false)),
                        "原因子句除蘊含文字外應全假：{:?} implied={} v={}",
                        self.clauses[ci],
                        implied,
                        v
                    );
                    self.clauses[ci].clone()
                }
            };
            // 跳過被蘊含文字（原因子句中唯一為真的文字）
            let skip = p.map(|v| lit(v, self.values[v].unwrap()));
            for &l in &clause {
                if Some(l) == skip {
                    continue;
                }
                let v = lit_var(l);
                if !seen[v] && self.level[v] > 0 {
                    seen[v] = true;
                    self.bump(v);
                    if self.level[v] >= self.current_level() {
                        path_c += 1;
                    } else {
                        learnt.push(l);
                    }
                }
            }
            // 從 trail 尾往回找下一個 seen 的變量
            loop {
                cursor -= 1;
                if seen[self.trail[cursor]] {
                    break;
                }
            }
            let v = self.trail[cursor];
            seen[v] = false;
            path_c -= 1;
            p = Some(v);
            if path_c == 0 {
                break;
            }
            cur = self.reason[v];
        }
        let uip = p.expect("UIP 缺失");
        // 學習子句 = [¬UIP, 其餘較低層文字...]
        let mut out = vec![lit_neg_of_var(uip, self)];
        out.extend(learnt);
        if out.len() <= 1 {
            return (out, 0);
        }
        // 回跳層 = 其餘文字的最大層（把它換到索引 1，成為第二監視文字）
        let mut max_lvl = 0u32;
        let mut max_i = 1usize;
        for i in 1..out.len() {
            let l = self.level[lit_var(out[i])];
            if l > max_lvl {
                max_lvl = l;
                max_i = i;
            }
        }
        out.swap(1, max_i);
        (out, max_lvl)
    }

    fn cancel_until(&mut self, lvl: u32) {
        while self.current_level() > lvl {
            let lim = *self.trail_lim.last().unwrap();
            while self.trail.len() > lim {
                let v = self.trail.pop().unwrap();
                self.values[v] = None;
                self.reason[v] = None;
                self.level[v] = 0;
            }
            self.trail_lim.pop();
        }
        self.qhead = self.trail.len();
    }

    fn decide(&mut self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_act = f64::NEG_INFINITY;
        for v in 0..self.n_vars {
            if self.values[v].is_none() && self.activity[v] > best_act {
                best_act = self.activity[v];
                best = Some(v);
            }
        }
        best
    }

    pub fn solve(&mut self) -> bool {
        if !self.ok {
            return false;
        }
        loop {
            match self.propagate() {
                Some(confl) => {
                    debug_assert!(
                        self.clauses[confl].iter().all(|&l| self.value_of(l) == Some(false)),
                        "衝突子句並非全假：{:?} 賦值={:?}",
                        self.clauses[confl],
                        self.values
                    );
                    self.stats.conflicts += 1;
                    if self.current_level() == 0 {
                        return false;
                    }
                    let (learnt, lvl) = self.analyze(confl);
                    debug_assert!(
                        learnt.iter().skip(1).all(|&l| self.value_of(l) == Some(false)),
                        "學習子句尾文字並非全假：{:?} 賦值={:?} 層級={:?}",
                        learnt,
                        self.values,
                        self.level
                    );
                    self.var_inc /= 0.95;
                    self.cancel_until(lvl);
                    if learnt.is_empty() {
                        return false;
                    }
                    self.stats.learned += 1;
                    let ci = self.add_clause_raw(learnt.clone());
                    if !self.ok {
                        return false;
                    }
                    if let Some(ci) = ci {
                        // 斷言 ¬UIP（回跳後必未賦值；不變量違背只在除錯建置報錯）
                        debug_assert!(
                            self.value_of(learnt[0]).is_none(),
                            "學習子句首文字回跳後應未賦值：{:?} lvl={}",
                            learnt,
                            lvl
                        );
                        self.enqueue(learnt[0], Some(ci));
                    }
                    // learnt 長度 1：add_clause_raw 已在 0 層入列
                }
                None => match self.decide() {
                    None => {
                        self.model = Some(
                            self.values.iter().map(|v| v.expect("SAT 但有未賦值變量")).collect(),
                        );
                        return true;
                    }
                    Some(v) => {
                        self.stats.decisions += 1;
                        self.trail_lim.push(self.trail.len());
                        self.enqueue(lit(v, self.phase[v]), None);
                    }
                },
            }
        }
    }

    pub fn model(&self) -> Option<Vec<bool>> {
        self.model.clone()
    }

    pub fn stats(&self) -> &CdclStats {
        &self.stats
    }

    /// 學習子句（問題子句之後加入者）。
    pub fn learned_clauses(&self) -> Vec<Vec<Lit>> {
        self.clauses[self.n_orig..].to_vec()
    }

}

fn lit_neg_of_var(v: usize, s: &Solver) -> Lit {
    // ¬(v 的當前賦值文字)
    lit(v, !s.values[v].unwrap())
}

/// 對小實例暴力檢查子句集可滿足性（義務自證的 oracle）。
pub fn brute_force_sat(n_vars: usize, clauses: &[Vec<Lit>]) -> Option<Vec<bool>> {
    assert!(n_vars <= 20, "暴力枚舉僅用於 ≤20 變量");
    for mask in 0u64..(1u64 << n_vars) {
        let assign: Vec<bool> = (0..n_vars).map(|i| (mask >> i) & 1 == 1).collect();
        let sat = clauses.iter().all(|c| {
            c.iter().any(|&l| {
                let v = lit_var(l);
                assign[v] == lit_positive(l)
            })
        });
        if sat {
            return Some(assign);
        }
    }
    None
}

/// 賦值是否滿足子句集。
pub fn satisfies(assign: &[bool], clauses: &[Vec<Lit>]) -> bool {
    clauses.iter().all(|c| {
        c.iter().any(|&l| assign[lit_var(l)] == lit_positive(l))
    })
}

/// 實際使用：cdcl.rs 文件清單 — 優化 with_capacity
pub fn cdcl_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("cdcl.rs", "cdcl.rs 正式運作 — 優化 with_capacity", "core/src/cdcl.rs"),
    ]
}

