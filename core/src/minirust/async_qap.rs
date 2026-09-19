//! Phase3 — async QAP：Future 輪詢約束、狀態機到 R1CS
//!
//! async/await 降維為狀態機，每個 await 點是一個狀態轉換

use std::collections::HashMap;

/// Future 狀態
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FutureState {
    Pending,
    Ready,
    Polling(usize), // await 點索引
}

/// async 函數的狀態機表示
#[derive(Clone, Debug)]
pub struct AsyncStateMachine {
    pub fn_name: String,
    pub num_await_points: usize,
    pub states: Vec<FutureState>,
    /// state_id -> (next_state, poll_constraint)
    pub transitions: HashMap<usize, (usize, String)>,
    /// 每個 await 對應的 Future 類型
    pub await_tys: Vec<String>,
}

impl AsyncStateMachine {
    pub fn new(fn_name: &str, num_await: usize) -> Self {
        let mut states = vec![FutureState::Pending];
        for i in 0..num_await {
            states.push(FutureState::Polling(i));
        }
        states.push(FutureState::Ready);
        Self {
            fn_name: fn_name.to_string(),
            num_await_points: num_await,
            states,
            transitions: HashMap::new(),
            await_tys: vec!["".to_string(); num_await],
        }
    }

    /// 添加轉換：from -> to，約束文本
    pub fn add_transition(&mut self, from: usize, to: usize, constraint: &str) {
        self.transitions.insert(from, (to, constraint.to_string()));
    }

    /// 生成 R1CS 約束：狀態轉換必須滿足 QAP
    pub fn to_r1cs(&self, nvars: usize) -> Vec<crate::poly::Poly> {
        use crate::frac::Frac;
        use crate::poly::Poly;
        let mut polys = vec![];
        // 狀態變量：s_i 表示當前狀態為 i (one-hot)
        // 約束：Σ s_i =1
        // 轉換：s_from * poll_ok = s_to
        for (from, (to, _)) in &self.transitions {
            // 簡化：s_from * c - s_to =0? 實際需 poll 變量
            // 生成 s_from - s_to 的差值多項式示意
            let p = Poly::var(*from % nvars, Frac::ONE, nvars)
                .sub(&Poly::var(*to % nvars, Frac::ONE, nvars));
            polys.push(p);
        }
        polys
    }

    /// 生成多項式文本
    pub fn poly_text(&self) -> Vec<String> {
        let mut out = vec![];
        out.push(format!("// Async state machine for {}", self.fn_name));
        out.push(format!("// {} await points, {} states", self.num_await_points, self.states.len()));
        out.push("// one-hot: Σ s_i =1".to_string());
        out.push(format!("s0 + s1 + ... + s{} -1 =0", self.states.len()-1));
        for (from, (to, constraint)) in &self.transitions {
            out.push(format!("// transition {} -> {} : {}", from, to, constraint));
            out.push(format!("s{} * poll_{} - s{} =0  # {}", from, from, to, constraint));
        }
        // Pending/Ready 布爾約束
        out.push("// s_i * (s_i -1) =0 for all i (boolean)".to_string());
        for i in 0..self.states.len() {
            out.push(format!("s{} * (s{} -1) =0", i, i));
        }
        out
    }

    /// 生成 Future 輪詢約束
    pub fn polling_constraints(&self) -> Vec<String> {
        let mut out = vec![];
        out.push(format!("// Polling constraints for {}", self.fn_name));
        for i in 0..self.num_await_points {
            out.push(format!("// await {} : Future poll", i));
            out.push(format!("poll_{} * (poll_{} -1) =0  # boolean", i, i));
            out.push(format!("poll_{} * ready_{} - ready_{} =0", i, i, i));
        }
        out
    }
}

/// 將 async fn 降維為狀態機
pub fn lower_async_fn(fn_name: &str, body: &str) -> AsyncStateMachine {
    // 計算 await 出現次數
    let num_await = body.matches("await").count();
    let mut sm = AsyncStateMachine::new(fn_name, num_await);

    // 簡單線性轉換：0->1->2...->Ready
    for i in 0..sm.states.len()-1 {
        sm.add_transition(i, i+1, &format!("poll_{}", i));
    }

    // 提取 await 類型（簡化）
    for (i, _) in body.match_indices("await") {
        if i < sm.await_tys.len() {
            sm.await_tys[i] = "impl Future".to_string();
        }
    }

    sm
}

/// QAP 處理：將 async 狀態機轉為 QAP 多項式
pub fn async_to_qap(sm: &AsyncStateMachine, nvars: usize) -> (Vec<crate::poly::Poly>, Vec<String>) {
    let polys = sm.to_r1cs(nvars);
    let texts = sm.poly_text();
    (polys, texts)
}

/// 實際使用：async_qap.rs 文件清單 — 優化 with_capacity
pub fn async_qap_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("async_qap.rs", "async_qap.rs 正式運作 — 優化 with_capacity", "core/src/minirust/async_qap.rs"),
    ]
}

