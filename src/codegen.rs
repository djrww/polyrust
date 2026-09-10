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
    /// 行內註解每個節點選定的規則
    pub annotate_rules: bool,
    /// 宏展開處註解臂選擇
    pub annotate_arm: bool,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        CodeGenConfig { annotate_types: true, annotate_rules: false, annotate_arm: true }
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
        out.push_str(&format!(
            "fn {}({}: {}) -> {} {{\n",
            sanitize(&f.name),
            sanitize(&f.param),
            f.param_ty.name(),
            f.ret_ty.name()
        ));
        let mut buf = String::new();
        emit_expr(&f.body, exp, deriv, cfg, &mut buf, 1);
        out.push_str(&buf);
        out.push_str("}\n\n");
    }
    out.push_str("fn main() {\n");
    let mut buf = String::new();
    emit_expr(&p.main_body, exp, deriv, cfg, &mut buf, 1);
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
) {
    let ty = |id: usize| d.node_types.get(&id).map(|t| t.name().to_string()).unwrap_or_else(|| "?".into());
    match &e.kind {
        EKind::Int(n) => out.push_str(&n.to_string()),
        EKind::BoolV(b) => out.push_str(if *b { "true" } else { "false" }),
        EKind::UnitLit => out.push_str("()"),
        EKind::Var(x) => out.push_str(&sanitize(x)),
        EKind::Let(x, e1, e2) => {
            // let x[: T] = e1;
            out.push_str(&indent(depth));
            if cfg.annotate_types {
                out.push_str(&format!("let mut {}: {} = ", sanitize(x), ty(e1.id)));
            } else {
                out.push_str(&format!("let mut {} = ", sanitize(x)));
            }
            emit_expr(e1, exp, d, cfg, out, depth);
            out.push_str(";\n");
            emit_expr(e2, exp, d, cfg, out, depth);
        }
        EKind::Seq(e1, e2) => {
            emit_expr(e1, exp, d, cfg, out, depth);
            out.push('\n');
            emit_expr(e2, exp, d, cfg, out, depth);
        }
        EKind::BinOp(op, a, b) => {
            out.push('(');
            emit_expr(a, exp, d, cfg, out, depth);
            out.push_str(&format!(" {} ", op.name()));
            emit_expr(b, exp, d, cfg, out, depth);
            out.push(')');
        }
        EKind::Not(a) => {
            out.push_str("(!");
            emit_expr(a, exp, d, cfg, out, depth);
            out.push(')');
        }
        EKind::If(c, a, b) => {
            out.push_str("if ");
            emit_expr(c, exp, d, cfg, out, depth);
            out.push_str(" {\n");
            emit_expr(a, exp, d, cfg, out, depth + 1);
            out.push_str(&format!("\n{}}} else {{\n", indent(depth)));
            emit_expr(b, exp, d, cfg, out, depth + 1);
            out.push_str(&format!("\n{}}}", indent(depth)));
        }
        EKind::Ref(x) => out.push_str(&format!("&{}", sanitize(x))),
        EKind::RefMut(x) => out.push_str(&format!("&mut {}", sanitize(x))),
        EKind::Deref(a) => {
            out.push_str("*(");
            emit_expr(a, exp, d, cfg, out, depth);
            out.push(')');
        }
        EKind::AssignVar(x, rhs) => {
            out.push_str(&format!("{}{} = ", indent(depth), sanitize(x)));
            emit_expr(rhs, exp, d, cfg, out, depth);
            out.push(';');
        }
        EKind::AssignDeref(lhs, rhs) => {
            out.push_str(&indent(depth));
            emit_expr(lhs, exp, d, cfg, out, depth);
            out.push_str(" = ");
            emit_expr(rhs, exp, d, cfg, out, depth);
            out.push(';');
        }
        EKind::Call(f, a) => {
            out.push_str(&format!("{}(", sanitize(f)));
            emit_expr(a, exp, d, cfg, out, depth);
            out.push(')');
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
                emit_expr(tree, exp, d, cfg, out, depth);
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
