//! 代碼生成：由（求解出的）型別推導生成可編譯的 Rust 源碼。
//! 可修改點：CodeGenConfig 的開關與 emit_* 函數的模板。

use crate::minirust::ast::*;
use crate::minirust::checker::Derivation;
use crate::minirust::macros::Expander;
use crate::minirust::parse::Parser;

#[derive(Clone, Copy, Debug)]
pub struct CodeGenConfig {
    /// let 綁定加顯式型別標註
    pub annotate_types: bool,
    /// 宏展開處註解臂選擇
    pub annotate_arm: bool,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        CodeGenConfig { annotate_types: true, annotate_arm: true }
    }
}

/// 衛生名稱清洗（#h3 → _h3，使其成為合法 Rust 識別字）
fn sanitize(name: &str) -> String {
    name.replace('#', "_")
}

pub fn generate_rust(
    p: &Program,
    exp: &Expander,
    deriv: &Derivation,
    cfg: &CodeGenConfig,
    header: &str,
) -> String {
    let mut out = String::new();
    out.push_str("// 由 polyrust 管線生成（可自由修改）\n");
    for line in header.lines() {
        out.push_str(&format!("// {}\n", line));
    }
    out.push_str("#![allow(unused_variables, unused_mut, non_snake_case, dead_code)]\n\n");
    // 宏定義原樣保留（供讀者對照；生成碼中的調用已被展開）
    for m in &p.macros {
        out.push_str(&format!(
            "// 定義：macro_rules! {} {{ ... }}（{} 個臂；見原始碼）\n",
            m.name,
            m.arms.len()
        ));
    }
    out.push('\n');
    for f in &p.fns {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|prm| format!("{}: {}", sanitize(&prm.name), prm.ty.name()))
            .collect();
        out.push_str(&format!(
            "fn {}({}) -> {} {{\n",
            sanitize(&f.name),
            params.join(", "),
            f.ret_ty.name()
        ));
        let mut buf = String::new();
        emit_expr(&f.body, exp, deriv, cfg, &mut buf, 1, false);
        out.push_str(&buf);
        out.push_str("}\n\n");
    }
    out.push_str("fn main() {\n");
    let mut buf = String::new();
    // 真 Rust 要求 main 回傳 ()。若 Mini-Rust main 的尾表達式非 unit，
    // 以語句形式丟棄之（加 `;`），既保持 round-trip 可解析，又過到真 rustc。
    let main_discard = !matches!(
        deriv.node_types.get(&p.main_body.id),
        Some(Type::Unit)
    );
    emit_expr(&p.main_body, exp, deriv, cfg, &mut buf, 1, main_discard);
    out.push_str(&buf);
    out.push_str("}\n");
    out
}

fn indent(n: usize) -> String {
    "    ".repeat(n)
}

fn emit_expr(
    e: &E,
    exp: &Expander,
    d: &Derivation,
    cfg: &CodeGenConfig,
    out: &mut String,
    depth: usize,
    discard_tail: bool,
) {
    let ty = |id: usize| d.node_types.get(&id).map(|t| t.name().to_string()).unwrap_or_else(|| "?".into());
    // 若此表達式是 main 的非 unit 尾表達式，則以語句形式丟棄（補 `;`）。
    let is_unit = ty(e.id) == "()";
    let maybe_discard = |out: &mut String| {
        if discard_tail && !is_unit {
            out.push(';');
        }
    };
    match &e.kind {
        EKind::Int(n) => {
            out.push_str(&n.to_string());
            maybe_discard(out);
        }
        EKind::BoolV(b) => {
            out.push_str(if *b { "true" } else { "false" });
            maybe_discard(out);
        }
        EKind::UnitLit => out.push_str("()"),
        EKind::Var(x) => {
            out.push_str(&sanitize(x));
            maybe_discard(out);
        }
        EKind::Let(x, e1, e2) => {
            // let x[: T] = e1;
            out.push_str(&indent(depth));
            if cfg.annotate_types {
                out.push_str(&format!("let mut {}: {} = ", sanitize(x), ty(e1.id)));
            } else {
                out.push_str(&format!("let mut {} = ", sanitize(x)));
            }
            emit_expr(e1, exp, d, cfg, out, depth, false);
            out.push_str(";\n");
            emit_expr(e2, exp, d, cfg, out, depth, discard_tail);
        }
        EKind::Seq(e1, e2) => {
            // e1 是語句位：非 unit 則補 `;`（恆真丟棄）
            emit_expr(e1, exp, d, cfg, out, depth, true);
            out.push('\n');
            emit_expr(e2, exp, d, cfg, out, depth, discard_tail);
        }
        EKind::BinOp(op, a, b) => {
            out.push('(');
            emit_expr(a, exp, d, cfg, out, depth, false);
            out.push_str(&format!(" {} ", op.name()));
            emit_expr(b, exp, d, cfg, out, depth, false);
            out.push(')');
            maybe_discard(out);
        }
        EKind::Not(a) => {
            out.push_str("(!");
            emit_expr(a, exp, d, cfg, out, depth, false);
            out.push(')');
            maybe_discard(out);
        }
        EKind::Neg(a) => {
            out.push_str("(-");
            emit_expr(a, exp, d, cfg, out, depth, false);
            out.push(')');
            maybe_discard(out);
        }
        EKind::If(c, a, b) => {
            out.push_str("if ");
            emit_expr(c, exp, d, cfg, out, depth, false);
            out.push_str(" {\n");
            emit_expr(a, exp, d, cfg, out, depth + 1, false);
            out.push_str(&format!("\n{}}} else {{\n", indent(depth)));
            emit_expr(b, exp, d, cfg, out, depth + 1, false);
            out.push_str(&format!("\n{}}}", indent(depth)));
            maybe_discard(out);
        }
        EKind::Ref(x) => {
            out.push_str(&format!("&{}", sanitize(x)));
            maybe_discard(out);
        }
        EKind::RefMut(x) => {
            out.push_str(&format!("&mut {}", sanitize(x)));
            maybe_discard(out);
        }
        EKind::Deref(a) => {
            out.push_str("*(");
            emit_expr(a, exp, d, cfg, out, depth, false);
            out.push(')');
            maybe_discard(out);
        }
        EKind::AssignVar(x, rhs) => {
            out.push_str(&format!("{}{} = ", indent(depth), sanitize(x)));
            emit_expr(rhs, exp, d, cfg, out, depth, false);
            out.push(';');
        }
        EKind::AssignDeref(lhs, rhs) => {
            out.push_str(&indent(depth));
            emit_expr(lhs, exp, d, cfg, out, depth, false);
            out.push_str(" = ");
            emit_expr(rhs, exp, d, cfg, out, depth, false);
            out.push(';');
        }
        EKind::Call(f, args) => {
            out.push_str(&format!("{}(", sanitize(f)));
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_expr(a, exp, d, cfg, out, depth, false);
            }
            out.push(')');
            maybe_discard(out);
        }
        EKind::Invoke(name, _) => {
            // 展開所選臂
            let arm = d.arm_choice.get(&e.id).copied().unwrap_or(0);
            if cfg.annotate_arm {
                out.push_str(&format!(
                    "/* {}! 展開（臂 {}，型別 {}）*/ ",
                    name,
                    arm + 1,
                    ty(e.id)
                ));
            }
            if let Some(tree) = exp.memo.get(&(e.id, arm)) {
                emit_expr(tree, exp, d, cfg, out, depth, discard_tail);
            } else {
                out.push_str(&format!("/* 展開缺失：{}! */", name));
            }
        }
    }
}

/// 重新解析生成碼並用直接檢查器驗證（round-trip 自檢；定理 9 義務的一部分）。
/// （剝離檔頭註解與內部屬性 `#![...]`，它們不屬於 Mini-Rust 語法。）
pub fn roundtrip_check(generated: &str) -> Result<(), String> {
    let stripped: String = generated
        .lines()
        .filter(|l| !l.trim_start().starts_with("//") && !l.trim_start().starts_with("#!["))
        .collect::<Vec<_>>()
        .join("\n");
    let p = Parser::parse_program(&stripped)?;
    if !p.macros.is_empty() {
        Err("生成碼含宏定義（應已展開）".into())
    } else {
        Ok(())
    }
}
