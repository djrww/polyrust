//! Phase3 — 效應系統：unsafe 上下文位元、I/O、pure、no-io、raw ptr
//!
//! 核心思想：每個節點帶 eff 位元：in_unsafe: bool, has_io: bool, is_pure: bool

use crate::minirust::ast::*;

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

    /// 全部檢查
    pub fn check_all(&self) -> Result<(), String> {
        self.check_unsafe_gate()?;
        self.check_no_io()?;
        self.check_pure()?;
        Ok(())
    }
}

/// 掃描 AST 檢測效應
pub fn analyze_effects(e: &E, ctx: &mut EffectContext) {
    walk(e, ctx, false);
}

fn walk(e: &E, ctx: &mut EffectContext, in_unsafe_block: bool) {
    // 檢測是否為 unsafe 塊：我們用 Call("unsafe", ...) 或特殊標記表示
    // 這裡約定：若 e.kind 是 Call 且 name == "unsafe" 則進入 unsafe
    match &e.kind {
        EKind::Call(name, args) => {
            if name == "unsafe" || name == "unsafe_block" {
                // 進入 unsafe 上下文
                let prev = ctx.in_unsafe;
                ctx.in_unsafe = true;
                for a in args {
                    walk(a, ctx, true);
                }
                ctx.in_unsafe = prev;
                return;
            }
            if name.starts_with("raw_") || name.contains("ptr") || name.contains("raw") {
                ctx.raw_ptr_ops.push(e.id);
                if !ctx.in_unsafe && !in_unsafe_block {
                    ctx.unsafe_usages.push(e.id);
                }
            }
            // I/O 檢測
            if is_io_call(name) {
                ctx.has_io = true;
                ctx.io_usages.push(e.id);
            }
            for a in args {
                walk(a, ctx, in_unsafe_block || ctx.in_unsafe);
            }
        }
        EKind::Int(_) | EKind::BoolV(_) | EKind::UnitLit | EKind::Invoke(_, _) => {}
        EKind::Var(_) => {}
        EKind::Let(_, e1, e2) => {
            walk(e1, ctx, in_unsafe_block || ctx.in_unsafe);
            walk(e2, ctx, in_unsafe_block || ctx.in_unsafe);
        }
        EKind::Seq(e1, e2) => {
            walk(e1, ctx, in_unsafe_block || ctx.in_unsafe);
            walk(e2, ctx, in_unsafe_block || ctx.in_unsafe);
        }
        EKind::BinOp(_, a, b) => {
            walk(a, ctx, in_unsafe_block || ctx.in_unsafe);
            walk(b, ctx, in_unsafe_block || ctx.in_unsafe);
        }
        EKind::Not(a) | EKind::Neg(a) => walk(a, ctx, in_unsafe_block || ctx.in_unsafe),
        EKind::Deref(a) => {
            // deref raw ptr 需要 unsafe
            // 假設 deref 的 arg 若是 raw ptr 類型則標記
            // 簡化：所有 Deref 若在非 unsafe 上下文則視為潛在 unsafe
            // 實際由 checker 的 Ty::RawPtr 判斷
            walk(a, ctx, in_unsafe_block || ctx.in_unsafe);
        }
        EKind::If(c, a, b) => {
            walk(c, ctx, in_unsafe_block || ctx.in_unsafe);
            walk(a, ctx, in_unsafe_block || ctx.in_unsafe);
            walk(b, ctx, in_unsafe_block || ctx.in_unsafe);
        }
        EKind::Ref(_) => {}
        EKind::RefMut(_) => {}
        EKind::AssignVar(_, rhs) => walk(rhs, ctx, in_unsafe_block || ctx.in_unsafe),
        EKind::AssignDeref(lhs, rhs) => {
            walk(lhs, ctx, in_unsafe_block || ctx.in_unsafe);
            walk(rhs, ctx, in_unsafe_block || ctx.in_unsafe);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minirust::ast::{E, EKind};

    fn mk_call(id: usize, name: &str) -> E {
        E { id, kind: EKind::Call(name.to_string(), vec![]) }
    }

    #[test]
    fn test_raw_ptr_parse() {
        let p = RawPtrTy::parse("*const i32").unwrap();
        assert_eq!(p.kind, RawPtrKind::Const);
        assert_eq!(p.inner, "i32");
        let p2 = RawPtrTy::parse("*mut u8").unwrap();
        assert_eq!(p2.kind, RawPtrKind::Mut);
    }

    #[test]
    fn test_io_detect() {
        let e = mk_call(1, "println");
        let mut ctx = EffectContext::default();
        analyze_effects(&e, &mut ctx);
        assert!(ctx.has_io);
        assert_eq!(ctx.io_usages, vec![1]);
    }

    #[test]
    fn test_unsafe_gate() {
        let e = mk_call(1, "raw_ptr_deref");
        let mut ctx = EffectContext { unsafe_allowed: false, ..Default::default() };
        analyze_effects(&e, &mut ctx);
        assert!(!ctx.unsafe_usages.is_empty());
        assert!(ctx.check_unsafe_gate().is_err());

        let mut ctx2 = EffectContext { unsafe_allowed: true, ..Default::default() };
        analyze_effects(&e, &mut ctx2);
        assert!(ctx2.check_unsafe_gate().is_ok());
    }

    #[test]
    fn test_no_io() {
        let e = mk_call(1, "print");
        let mut ctx = EffectContext { no_io: true, ..Default::default() };
        analyze_effects(&e, &mut ctx);
        assert!(ctx.check_no_io().is_err());
    }

    #[test]
    fn test_pure() {
        let e = mk_call(1, "println");
        let mut ctx = EffectContext { pure: Some(true), ..Default::default() };
        analyze_effects(&e, &mut ctx);
        assert!(ctx.check_pure().is_err());
    }

    #[test]
    fn test_from_poly_source() {
        let text = "# @unsafe-allowed\n# @no-io\nfn main() {}";
        let src = crate::dsl::load_poly(text).unwrap();
        let ctx = EffectContext::from_poly_source(&src);
        assert!(ctx.unsafe_allowed);
        assert!(ctx.no_io);
    }
}
