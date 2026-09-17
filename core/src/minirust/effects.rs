//! Phase3 — 效應系統：unsafe 上下文位元、I/O、pure、no-io、raw ptr + 5類 unsafe 前移
//! 核心思想：每個節點帶 eff 位元：in_unsafe: bool, has_io: bool, is_pure: bool
//! 新增：unsafe_fn_defs, unsafe_trait_defs, static_mut_defs, union_defs, thread_unsafe

use crate::minirust::ast::*;
use std::collections::HashSet;

/// 效應上下文
#[derive(Clone, Debug, Default)]
pub struct EffectContext {
    /// 是否在 unsafe 塊內
    pub in_unsafe: bool,
    /// 允許 unsafe（來自 # @unsafe-allowed）
    pub unsafe_allowed: bool,
    /// 是否有 I/O（print、file、net）
    pub has_io: bool,
    /// 是否 pure（來自 # @pure）
    pub pure: Option<bool>,
    /// 禁止 I/O（來自 # @no-io）
    pub no_io: bool,
    /// 記錄所有 unsafe 使用點
    pub unsafe_usages: Vec<usize>,
    /// 記錄所有 I/O 使用點
    pub io_usages: Vec<usize>,
    /// raw ptr 操作節點
    pub raw_ptr_ops: Vec<usize>,
    // ── 新增：五類 unsafe 前移 ──
    /// unsafe fn 定義集合
    pub unsafe_fn_defs: HashSet<String>,
    /// unsafe trait 定義集合
    pub unsafe_trait_defs: HashSet<String>,
    /// unsafe fn 調用點
    pub unsafe_fn_calls: Vec<usize>,
    /// static mut 定義
    pub static_mut_defs: HashSet<String>,
    /// static mut 訪問點
    pub static_mut_accesses: Vec<usize>,
    /// union 定義
    pub union_defs: HashSet<String>,
    /// union 訪問點
    pub union_accesses: Vec<usize>,
    /// unsafe trait impl 點
    pub unsafe_trait_impls: Vec<usize>,
    /// 多線程下不安全訪問（static mut 在 thread/async）
    pub thread_unsafe_usages: Vec<usize>,
    /// 是否在 threaded 上下文（thread::spawn / async）
    pub in_threaded: bool,
    /// 是否在 async 上下文
    pub in_async: bool,
}

impl EffectContext {
    pub fn from_poly_source(src: &crate::dsl::PolySource) -> Self {
        Self {
            in_unsafe: false,
            unsafe_allowed: src.unsafe_allowed,
            has_io: false,
            pure: src.pure,
            no_io: src.no_io,
            unsafe_usages: vec![],
            io_usages: vec![],
            raw_ptr_ops: vec![],
            unsafe_fn_defs: HashSet::new(),
            unsafe_trait_defs: HashSet::new(),
            unsafe_fn_calls: vec![],
            static_mut_defs: HashSet::new(),
            static_mut_accesses: vec![],
            union_defs: HashSet::new(),
            union_accesses: vec![],
            unsafe_trait_impls: vec![],
            thread_unsafe_usages: vec![],
            in_threaded: false,
            in_async: false,
        }
    }

    /// 檢查 unsafe gate
    pub fn check_unsafe_gate(&self) -> Result<(), String> {
        if !self.unsafe_usages.is_empty() && !self.unsafe_allowed && !self.in_unsafe {
            return Err(format!(
                "unsafe usage detected at nodes {:?} but # @unsafe-allowed not set and not in unsafe block",
                self.unsafe_usages
            ));
        }
        Ok(())
    }

    /// 檢查 no-io
    pub fn check_no_io(&self) -> Result<(), String> {
        if self.no_io && self.has_io {
            return Err(format!(
                "I/O usage detected at nodes {:?} but # @no-io set",
                self.io_usages
            ));
        }
        Ok(())
    }

    /// 檢查 pure
    pub fn check_pure(&self) -> Result<(), String> {
        if self.pure == Some(true) && self.has_io {
            return Err(format!(
                "pure function has I/O at nodes {:?}",
                self.io_usages
            ));
        }
        Ok(())
    }

    /// 檢查 unsafe fn 調用必須在 unsafe 塊
    pub fn check_unsafe_fn_gate(&self) -> Result<(), String> {
        if !self.unsafe_fn_calls.is_empty() && !self.unsafe_allowed && !self.in_unsafe {
            // 若調用點未在 unsafe 塊，且未全局允許，則報錯
            // 注意：調用點本身可能在 unsafe 塊內，但我們在 walk 時已根據 in_unsafe 判斷是否 push unsafe_usages
            // 這裡檢查 unsafe_fn_calls 是否有對應的 unsafe_usages
            // 簡化：若 unsafe_fn_calls 非空且 unsafe_usages 包含這些節點，則報錯
            let has_unprotected = self.unsafe_fn_calls.iter().any(|id| self.unsafe_usages.contains(id));
            if has_unprotected {
                return Err(format!(
                    "unsafe fn call at nodes {:?} requires unsafe block",
                    self.unsafe_fn_calls
                ));
            }
        }
        Ok(())
    }

    /// 檢查 static mut 在多線程下
    pub fn check_static_mut_thread_safety(&self) -> Result<(), String> {
        if !self.thread_unsafe_usages.is_empty() {
            return Err(format!(
                "static mut access in threaded/async context at nodes {:?} may cause data race, requires Mutex or single-threaded guarantee",
                self.thread_unsafe_usages
            ));
        }
        Ok(())
    }

    /// 檢查 union 訪問
    pub fn check_union_gate(&self) -> Result<(), String> {
        if !self.union_accesses.is_empty() && !self.unsafe_allowed && !self.in_unsafe {
            let has_unprotected = self.union_accesses.iter().any(|id| self.unsafe_usages.contains(id));
            if has_unprotected {
                return Err(format!(
                    "union field access at nodes {:?} requires unsafe block",
                    self.union_accesses
                ));
            }
        }
        Ok(())
    }

    /// 全部檢查
    pub fn check_all(&self) -> Result<(), String> {
        self.check_unsafe_gate()?;
        self.check_no_io()?;
        self.check_pure()?;
        self.check_unsafe_fn_gate()?;
        self.check_static_mut_thread_safety()?;
        self.check_union_gate()?;
        Ok(())
    }

    /// 註冊 unsafe fn 定義
    pub fn register_unsafe_fn(&mut self, name: String) {
        self.unsafe_fn_defs.insert(name);
    }

    /// 註冊 unsafe trait 定義
    pub fn register_unsafe_trait(&mut self, name: String) {
        self.unsafe_trait_defs.insert(name);
    }

    /// 註冊 static mut 定義
    pub fn register_static_mut(&mut self, name: String) {
        self.static_mut_defs.insert(name);
    }

    /// 註冊 union 定義
    pub fn register_union(&mut self, name: String) {
        self.union_defs.insert(name);
    }
}

/// 掃描 AST 檢測效應
pub fn analyze_effects(e: &E, ctx: &mut EffectContext) {
    walk(e, ctx, false, false, false);
}

fn walk(e: &E, ctx: &mut EffectContext, in_unsafe_block: bool, in_threaded: bool, in_async: bool) {
    let cur_unsafe = in_unsafe_block || ctx.in_unsafe;
    let cur_threaded = in_threaded || ctx.in_threaded;
    let cur_async = in_async || ctx.in_async;

    match &e.kind {
        EKind::Call(name, args) => {
            if name == "unsafe" || name == "unsafe_block" {
                let prev_unsafe = ctx.in_unsafe;
                let prev_threaded = ctx.in_threaded;
                let prev_async = ctx.in_async;
                ctx.in_unsafe = true;
                // unsafe 塊內保持 threaded/async 狀態
                for a in args {
                    walk(a, ctx, true, cur_threaded, cur_async);
                }
                ctx.in_unsafe = prev_unsafe;
                ctx.in_threaded = prev_threaded;
                ctx.in_async = prev_async;
                return;
            }

            // 檢測是否進入 threaded 上下文
            let mut next_threaded = cur_threaded;
            let mut next_async = cur_async;
            if name == "thread_spawn" || name.contains("spawn") || name == "thread::spawn" {
                next_threaded = true;
            }
            if name == "async_block" || name.contains("async") || name == "await" {
                next_async = true;
            }

            // raw ptr 操作
            if name.starts_with("raw_") || name.contains("ptr") || name.contains("raw") {
                ctx.raw_ptr_ops.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
            }

            // unsafe fn 調用檢測
            if ctx.unsafe_fn_defs.contains(name) {
                ctx.unsafe_fn_calls.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
            }
            // 即使未註冊，若名字以 unsafe_fn_ 開頭也視為 unsafe fn
            if name.starts_with("unsafe_fn_") || name.starts_with("my_unsafe") {
                ctx.unsafe_fn_calls.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
            }

            // static mut 訪問檢測
            if name.starts_with("static_mut_") || name.contains("static_mut") {
                ctx.static_mut_accesses.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
                if cur_threaded || cur_async {
                    ctx.thread_unsafe_usages.push(e.id);
                }
            }
            // 若調用名是已知的 static mut 定義
            if ctx.static_mut_defs.contains(name) {
                ctx.static_mut_accesses.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
                if cur_threaded || cur_async {
                    ctx.thread_unsafe_usages.push(e.id);
                }
            }

            // union 訪問檢測
            if name.starts_with("union_access") || name.contains("union") {
                ctx.union_accesses.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
            }
            if ctx.union_defs.iter().any(|u| name.contains(u)) && (name.contains("field") || name.contains("access")) {
                ctx.union_accesses.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
            }

            // unsafe trait impl 檢測
            if name.starts_with("unsafe_trait_impl") || (name.contains("impl") && ctx.unsafe_trait_defs.iter().any(|t| name.contains(t))) {
                ctx.unsafe_trait_impls.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
            }

            // I/O 檢測
            if is_io_call(name) {
                ctx.has_io = true;
                ctx.io_usages.push(e.id);
            }

            // 遞歸，傳遞 threaded/async 狀態
            for a in args {
                walk(a, ctx, cur_unsafe, next_threaded, next_async);
            }
        }
        EKind::Int(_) | EKind::BoolV(_) | EKind::UnitLit | EKind::Invoke(_, _) => {}
        EKind::Var(vname) => {
            // 檢查變量是否為 static mut
            if ctx.static_mut_defs.contains(vname) {
                ctx.static_mut_accesses.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
                if cur_threaded || cur_async {
                    ctx.thread_unsafe_usages.push(e.id);
                }
            }
            // union 類型變量訪問
            if ctx.union_defs.contains(vname) {
                // 訪問 union 變量本身不一定需要 unsafe，但字段訪問需要
                // 這裡保守不報，字段訪問由 Call 捕獲
            }
        }
        EKind::Let(_, e1, e2) => {
            walk(e1, ctx, cur_unsafe, cur_threaded, cur_async);
            walk(e2, ctx, cur_unsafe, cur_threaded, cur_async);
        }
        EKind::Seq(e1, e2) => {
            walk(e1, ctx, cur_unsafe, cur_threaded, cur_async);
            walk(e2, ctx, cur_unsafe, cur_threaded, cur_async);
        }
        EKind::BinOp(_, a, b) => {
            walk(a, ctx, cur_unsafe, cur_threaded, cur_async);
            walk(b, ctx, cur_unsafe, cur_threaded, cur_async);
        }
        EKind::Not(a) | EKind::Neg(a) => walk(a, ctx, cur_unsafe, cur_threaded, cur_async),
        EKind::Deref(a) => {
            walk(a, ctx, cur_unsafe, cur_threaded, cur_async);
        }
        EKind::If(c, a, b) => {
            walk(c, ctx, cur_unsafe, cur_threaded, cur_async);
            walk(a, ctx, cur_unsafe, cur_threaded, cur_async);
            walk(b, ctx, cur_unsafe, cur_threaded, cur_async);
        }
        EKind::Ref(_) => {}
        EKind::RefMut(_) => {}
        EKind::AssignVar(vname, rhs) => {
            if ctx.static_mut_defs.contains(vname) {
                ctx.static_mut_accesses.push(e.id);
                if !cur_unsafe {
                    ctx.unsafe_usages.push(e.id);
                }
                if cur_threaded || cur_async {
                    ctx.thread_unsafe_usages.push(e.id);
                }
            }
            walk(rhs, ctx, cur_unsafe, cur_threaded, cur_async);
        }
        EKind::AssignDeref(lhs, rhs) => {
            walk(lhs, ctx, cur_unsafe, cur_threaded, cur_async);
            walk(rhs, ctx, cur_unsafe, cur_threaded, cur_async);
        }
    }
}

fn is_io_call(name: &str) -> bool {
    matches!(name,
        "print" | "println" | "eprint" | "eprintln" |
        "read" | "read_line" | "read_to_string" |
        "write" | "write_all" |
        "open" | "File_open" | "File_create" |
        "TcpStream_connect" | "TcpListener_bind" |
        "std_fs_read" | "std_fs_write"
    ) || name.starts_with("io_") || name.contains("print")
}

/// raw ptr 類型檢查：*const T, *mut T
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RawPtrKind {
    Const,
    Mut,
}

#[derive(Clone, Debug)]
pub struct RawPtrTy {
    pub kind: RawPtrKind,
    pub inner: String,
}

impl RawPtrTy {
    pub fn parse(s: &str) -> Option<Self> {
        let t = s.trim();
        if t.starts_with("*const ") {
            Some(Self { kind: RawPtrKind::Const, inner: t["*const ".len()..].trim().to_string() })
        } else if t.starts_with("*mut ") {
            Some(Self { kind: RawPtrKind::Mut, inner: t["*mut ".len()..].trim().to_string() })
        } else {
            None
        }
    }

    pub fn is_raw_ptr(s: &str) -> bool {
        s.trim().starts_with("*const ") || s.trim().starts_with("*mut ")
    }

    /// 約束：raw ptr 解引用必須在 unsafe
    pub fn check_deref_allowed(&self, ctx: &EffectContext, node_id: usize) -> Result<(), String> {
        if !ctx.in_unsafe && !ctx.unsafe_allowed {
            return Err(format!(
                "raw ptr deref of *{:?} {} at node {} requires unsafe",
                self.kind, self.inner, node_id
            ));
        }
        Ok(())
    }
}

/// 實際使用：effects.rs 文件清單
pub fn effects_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("effects.rs", "effects.rs 5類 unsafe 前移 — unsafe_fn/static_mut/union/thread", "core/src/minirust/effects.rs"),
    ]
}
