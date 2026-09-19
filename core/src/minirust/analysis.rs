//! 借用活性分析（簡化的直線碼 liveness）：
//! 每個 &mut 借用有存活區間 [建立點, 最後使用點+1)；
//! 同一變量的兩個借用區間重疊 ⇒ 衝突；對仍被借用變量直接賦值 ⇒ 衝突。
//! 檢查器（ground truth）與代數約束生成共用本分析，保證兩側語義一致。

use crate::minirust::ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct BorrowInfo {
    pub node: usize,      // RefMut 節點 id
    pub var_def: usize,   // 被借用變量的綁定節點 id
    pub start: u32,
    pub end: u32,
    #[allow(dead_code)] // 對應 Lean BorrowInfo.binder，暫未讀取
    pub binder: Option<String>, // let r = &mut x 的綁定名（暫時借用 = None）
}

#[derive(Clone, Debug, Default)]
pub struct BorrowAnalysis {
    #[allow(dead_code)] // 對應 Lean BorrowAnalysis.borrows，暫未讀取
    pub borrows: Vec<BorrowInfo>,
    /// 存活區間重疊的借用對（同 var_def）
    pub conflicts: Vec<(usize, usize)>,
    /// (借用節點, 賦值節點)：對活躍借用中的變量直接賦值
    pub assign_conflicts: Vec<(usize, usize)>,
}

impl BorrowAnalysis {
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty() && self.assign_conflicts.is_empty()
    }
}

struct Walker {
    point: u32,
    /// (refmut node, var_def, start)
    borrows: Vec<(usize, usize, u32)>,
    /// refmut node → let 綁定名
    binder_name: HashMap<usize, String>,
    /// refmut node → 最後使用點
    last_use: HashMap<usize, u32>,
    /// (assign node, var_def, point)
    assigns: Vec<(usize, usize, u32)>,
    /// 所有 RefMut 節點 id
    refmut_nodes: HashSet<usize>,
}

/// 分析一棵樹。ext_env：外部作用域 name → 綁定節點 id
/// （arm 展開樹引用外部變量時，def 節點 id 由此查得）。
pub fn analyze(e: &E, ext_env: &HashMap<String, usize>) -> BorrowAnalysis {
    let mut w = Walker {
        point: 0,
        borrows: vec![],
        binder_name: HashMap::new(),
        last_use: HashMap::new(),
        assigns: vec![],
        refmut_nodes: HashSet::new(),
    };
    let mut env = ext_env.clone();
    walk(e, &mut env, &mut w);

    let mut borrows: Vec<BorrowInfo> = w
        .borrows
        .iter()
        .map(|&(node, var_def, start)| {
            let end = w.last_use.get(&node).map_or(start + 1, |u| u + 1);
            BorrowInfo { node, var_def, start, end, binder: w.binder_name.get(&node).cloned() }
        })
        .collect();
    borrows.sort_by_key(|b| b.start);

    // 借用衝突：同 var_def 且區間重疊
    let mut conflicts = vec![];
    for i in 0..borrows.len() {
        for j in (i + 1)..borrows.len() {
            let (a, b) = (&borrows[i], &borrows[j]);
            if a.var_def == b.var_def && a.start < b.end && b.start < a.end {
                conflicts.push((a.node, b.node));
            }
        }
    }
    // 賦值衝突：對活躍借用中的變量賦值
    let mut assign_conflicts = vec![];
    for &(anode, adef, apt) in &w.assigns {
        for b in &borrows {
            if b.var_def == adef && b.start <= apt && apt < b.end {
                assign_conflicts.push((b.node, anode));
            }
        }
    }
    BorrowAnalysis { borrows, conflicts, assign_conflicts }
}

fn walk(e: &E, env: &mut HashMap<String, usize>, w: &mut Walker) {
    w.point += 1;
    let my_point = w.point;
    match &e.kind {
        EKind::Int(_) | EKind::BoolV(_) | EKind::UnitLit | EKind::Invoke(_, _) => {}
        EKind::Var(name) => {
            // 使用了借用綁定（let r = &mut x 後的 r）⇒ 更新該借用的最後使用點
            if let Some(&def) = env.get(name) {
                if w.refmut_nodes.contains(&def) {
                    w.last_use.insert(def, my_point);
                }
            }
        }
        EKind::Let(name, e1, e2) => {
            walk(e1, env, w);
            // e1 根為 RefMut ⇒ name 是借用綁定名（衛生改名保證名不衝突）
            if let EKind::RefMut(_) = &e1.kind {
                w.binder_name.insert(e1.id, name.clone());
            }
            env.insert(name.clone(), e1.id);
            walk(e2, env, w);
            env.remove(name);
        }
        EKind::Seq(e1, e2) => {
            walk(e1, env, w);
            walk(e2, env, w);
        }
        EKind::BinOp(_, a, b) => {
            walk(a, env, w);
            walk(b, env, w);
        }
        EKind::Not(a) | EKind::Neg(a) | EKind::Deref(a) => walk(a, env, w),
        EKind::If(c, a, b) => {
            walk(c, env, w);
            walk(a, env, w);
            walk(b, env, w);
        }
        EKind::Ref(_) => {}
        EKind::RefMut(name) => {
            let var_def = env.get(name).copied().unwrap_or(usize::MAX);
            w.borrows.push((e.id, var_def, my_point));
            w.refmut_nodes.insert(e.id);
        }
        EKind::AssignVar(name, rhs) => {
            let var_def = env.get(name).copied().unwrap_or(usize::MAX);
            w.assigns.push((e.id, var_def, my_point));
            walk(rhs, env, w);
        }
        EKind::AssignDeref(lhs, rhs) => {
            walk(lhs, env, w);
            walk(rhs, env, w);
        }
        EKind::Call(_, args) => {
            for a in args {
                walk(a, env, w);
            }
        }
    }
}

/// 實際使用：analysis.rs 文件清單 — 優化 with_capacity
pub fn analysis_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("analysis.rs", "analysis.rs 正式運作 — 優化 with_capacity", "core/src/minirust/analysis.rs"),
    ]
}

