// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! rustc 语法学习原型 — WRP-R2 后置（2026-09-20）
//! 对应 docs/RUSTC_SYNTAX_STUDY.md §4，演示如何对齐 rustc 的泛型逗号解析。
//! 零依赖（std only），与 `syn` 的 AngleBracketedGenericArguments 对照。

/// 按顶层逗号切分 struct 字面量内部（`HashMap<String, String>` 内逗号不切）。
/// 与 `pipeline_v2::check_struct_field_types` 的 WRP-R2 修复同构。
pub fn split_fields_top_level(inner: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut angle_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    for ch in inner.chars() {
        match ch {
            '<' => {
                angle_depth += 1;
                cur.push(ch);
            }
            '>' => {
                if angle_depth > 0 {
                    angle_depth -= 1;
                }
                cur.push(ch);
            }
            '(' => {
                paren_depth += 1;
                cur.push(ch);
            }
            ')' => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
                cur.push(ch);
            }
            '{' => {
                brace_depth += 1;
                cur.push(ch);
            }
            '}' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
                cur.push(ch);
            }
            ',' if angle_depth == 0 && paren_depth == 0 && brace_depth == 0 => {
                parts.push(cur.clone());
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    parts
}

/// 检测类型字符串是否含泛型（`<T>`），用于对照 syn::Type::Path。
pub fn has_generic_args(ty: &str) -> bool {
    let t = ty.trim();
    t.contains('<') && t.contains('>')
}

/// 简单的泛型参数提取（顶层逗号切分 `<A, B>`），演示 syn::Punctuated 的行为。
pub fn generic_args(ty: &str) -> Option<Vec<String>> {
    let start = ty.find('<')?;
    let end = ty.rfind('>')?;
    if start >= end {
        return None;
    }
    let inner = &ty[start + 1..end];
    Some(split_fields_top_level(inner).into_iter().map(|s| s.trim().to_string()).collect())
}

/// 学习清单：rustc 语法七层的最小 self-check。
pub fn rustc_syntax_study_checklist() -> Vec<(&'static str, &'static str, bool)> {
    vec![
        ("Items", "Struct/Enum/Fn/Impl/Trait/Static/Union/Mod", true),
        ("Types", "HashMap<String, String> generic comma", true), // R2 已修 split
        ("Functions", "#[pure] per-fn vs file-level has_io", true), // R2 已修
        ("Lifetimes", "'a: 'b graph + cycle", true),
        ("Macros", "println! format_args desugar", false),
        ("Async", "async fn/Future + Send bound", true), // R2 已补 spawn
        ("Unsafe", "5 类 safety 前移", true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_hashmap_keeps_generic_comma() {
        let inner = "editors: HashMap<String, String>";
        let parts = split_fields_top_level(inner);
        assert_eq!(parts.len(), 1);
        assert!(parts[0].contains("HashMap<String, String>"));
    }

    #[test]
    fn split_multi_fields() {
        let inner = "x: i32, y: HashMap<String, Vec<i32>>, z: bool";
        let parts = split_fields_top_level(inner);
        assert_eq!(parts.len(), 3);
        assert!(parts[1].contains("HashMap"));
    }

    #[test]
    fn generic_args_hashmap() {
        let args = generic_args("HashMap<String, String>").unwrap();
        assert_eq!(args, vec!["String", "String"]);
        let args2 = generic_args("HashMap<String, Vec<i32>>").unwrap();
        assert_eq!(args2[0], "String");
        assert_eq!(args2[1], "Vec<i32>");
    }

    #[test]
    fn has_generic() {
        assert!(has_generic_args("HashMap<String, String>"));
        assert!(!has_generic_args("i32"));
    }
}
