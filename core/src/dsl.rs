//! `.poly` DSL 載入器、函式庫 import、模板替換，與 LLM 接口契約。
//!
//! `.poly` 是 polyrust 的「可輸入」描述語言：內容即 Mini-Rust 子集源碼
//! （`fn` / `macro_rules!` / `fn main`），外加以 `#` 開頭的 metadata 行。
//!
//! 除了基礎 metadata（`@intent` / `@key: value`），還支援兩項 Phase 2 能力：
//!
//! - `@import: name`：內聯函式庫（內建 `std/` 或檔案系統 `.poly`）。
//! - `@set key = value` + `{{key}}` 佔位：模板實例化（描述 → 具體程式）。
//!
//! ```text
//! # @intent: 計算 (x + K) 的平方
//! # @import: basic
//! # @set init = 5
//! # @set k = 1
//!
//! fn main() {
//!     let x = {{init}};
//!     let y = sqr!(x + {{k}});
//! }
//! ```
//!
//! metadata 語法：`# @key: value`（`#` 前可含空白）；`# @set key = value` 是模板
//! 參數（`=` 分隔）；其餘 `#` 行是普通註釋。

use std::path::{Path, PathBuf};

/// 解析後的 `.poly` 描述。
#[derive(Clone, Debug, Default)]
pub struct PolySource {
    /// `@intent`（可多行，以換行合併）——LLM 標注意圖用，polyrust 不做語義處理。
    pub intent: Option<String>,
    /// 其餘 `@key` 的鍵值對（保留順序，供未來擴充）。
    pub metadata: Vec<(String, String)>,
    /// `@import` 的函式庫名稱（保留順序）。
    pub imports: Vec<String>,
    /// `@set key = value` 的模板參數（保留順序）。
    pub sets: Vec<(String, String)>,
    /// 剔除 metadata 後的純 Mini-Rust 源碼（可能含 `{{key}}` 佔位）。
    pub source: String,
}

/// 由 `.poly` 文本載入。`#` metadata 行會被剝離，其餘原樣保留。
pub fn load_poly(text: &str) -> Result<PolySource, String> {
    let mut intent_lines: Vec<String> = Vec::new();
    let mut metadata: Vec<(String, String)> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let mut sets: Vec<(String, String)> = Vec::new();
    let mut src_lines: Vec<&str> = Vec::new();

    for raw in text.lines() {
        let trimmed = raw.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            // metadata / 註釋行
            let rest = rest.trim();
            if let Some(kv) = rest.strip_prefix('@') {
                let kv = kv.trim();
                if let Some((k, v)) = kv.split_once(':') {
                    let k = k.trim();
                    let v = v.trim();
                    match k {
                        "intent" => intent_lines.push(v.to_string()),
                        "import" => imports.push(v.to_string()),
                        _ => metadata.push((k.to_string(), v.to_string())),
                    }
                } else if let Some(body) = kv.strip_prefix("set ") {
                    // `@set key = value`（`=` 分隔）
                    let body = body.trim();
                    if let Some((k, v)) = body.split_once('=') {
                        sets.push((k.trim().to_string(), v.trim().to_string()));
                    }
                    // 無 `=` 的 `@set` 視為無效，忽略
                }
                // 其餘 `# @key` 無冒號 → 視為註釋，忽略
            }
            // 否則為普通註釋，忽略
        } else {
            src_lines.push(raw);
        }
    }

    let intent = if intent_lines.is_empty() {
        None
    } else {
        Some(intent_lines.join("\n"))
    };

    Ok(PolySource {
        intent,
        metadata,
        imports,
        sets,
        source: src_lines.join("\n"),
    })
}

/// 模板替換：把 `{{key}}` 佔位換成 `sets` 裡對應的值。
/// 缺參數會報錯；多餘的 `@set`（無對應佔位）無害。
pub fn apply_sets(source: &str, sets: &[(String, String)]) -> Result<String, String> {
    let mut out = source.to_string();
    // 先偵測所有出現的佔位，確認都有對應參數
    let mut placeholders: Vec<String> = Vec::new();
    let mut rest = source;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let end = after
            .find("}}")
            .ok_or_else(|| "模板佔位 {{ 未閉合（缺 }}）".to_string())?;
        let key = &after[..end];
        placeholders.push(key.to_string());
        rest = &after[end + 2..];
    }
    for key in &placeholders {
        let key = key.trim();
        if !sets.iter().any(|(k, _)| k == key) {
            return Err(format!("模板參數 {{{{ {key} }}}} 未定義（缺 @set {key} = …）"));
        }
    }
    for (k, v) in sets {
        out = out.replace(&format!("{{{{{}}}}}", k), v);
    }
    Ok(out)
}

/// 內建 std 函式庫（編譯期嵌入，無需檔案系統）。
pub const BUILTIN_STD: &[(&str, &str)] = &[
    ("basic", include_str!("../std/basic.poly")),
    ("math", include_str!("../std/math.poly")),
    ("bool", include_str!("../std/bool.poly")),
];

/// 解析 `.poly`：載入 metadata、內聯 `@import` 函式庫、套用 `@set` 模板替換。
///
/// - `base`：來源檔案所在目錄（供相對 import 解析）；`None` 表示無來源檔（stdin/web）。
/// - 回傳最終 `PolySource`（`source` 已含函式庫定義、已替換佔位）。
pub fn resolve(text: &str, base: Option<&Path>) -> Result<PolySource, String> {
    let poly = load_poly(text)?;

    // 收集函式庫源碼（遞迴，去重，偵測循環）
    let mut lib_sources: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for name in &poly.imports {
        collect_import(name, base, &mut seen, &mut lib_sources)?;
    }

    // 套用模板替換（只作用於使用者源碼；函式庫是固定文本）
    let user_src = apply_sets(&poly.source, &poly.sets)?;

    let mut final_source = lib_sources.join("\n");
    if !final_source.is_empty() {
        final_source.push('\n');
    }
    final_source.push_str(&user_src);

    Ok(PolySource {
        intent: poly.intent.clone(),
        metadata: poly.metadata.clone(),
        imports: poly.imports.clone(),
        sets: poly.sets.clone(),
        source: final_source,
    })
}

/// 遞迴收集某個 import 的函式庫源碼（去重 + 循環偵測）。
fn collect_import(
    name: &str,
    base: Option<&Path>,
    seen: &mut Vec<String>,
    out: &mut Vec<String>,
) -> Result<(), String> {
    if seen.iter().any(|s| s == name) {
        return Err(format!("函式庫 import 循環：{}", name));
    }
    seen.push(name.to_string());

    let text = find_library(name, base)?;
    let lib = load_poly(&text)?;
    // 函式庫自身也可能 import（遞迴）
    for sub in &lib.imports {
        collect_import(sub, base, seen, out)?;
    }
    // 只取 fn/macro 定義，剝除 `fn main`（函式庫不提供 main）
    let defs = strip_main(&lib.source);
    out.push(defs);

    seen.pop();
    Ok(())
}

/// 剝除源碼中的 `fn main() { … }`，只留 fn/macro 定義。
fn strip_main(source: &str) -> String {
    let mut out_lines: Vec<&str> = Vec::new();
    let mut depth = 0i32;
    let mut in_main = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !in_main {
            if trimmed.starts_with("fn main") {
                in_main = true;
                // 統計本行的大括號深度
                depth = brace_delta(line);
                if depth <= 0 {
                    // 單行 `fn main() { body }`，直接跳過
                    in_main = false;
                    continue;
                }
                continue;
            }
            out_lines.push(line);
        } else {
            depth += brace_delta(line);
            if depth <= 0 {
                in_main = false;
            }
        }
    }
    out_lines.join("\n")
}

fn brace_delta(line: &str) -> i32 {
    let mut d = 0i32;
    for c in line.chars() {
        match c {
            '{' => d += 1,
            '}' => d -= 1,
            _ => {}
        }
    }
    d
}

/// 尋找函式庫：先內建 std，再檔案系統（`std/<name>.poly`、`<base>/<name>.poly`、`<name>.poly`）。
fn find_library(name: &str, base: Option<&Path>) -> Result<String, String> {
    if let Some((_, content)) = BUILTIN_STD.iter().find(|(n, _)| *n == name) {
        return Ok(content.to_string());
    }
    // 檔案系統候選
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(b) = base {
        candidates.push(b.join(format!("{}.poly", name)));
    }
    candidates.push(PathBuf::from(format!("std/{}.poly", name)));
    candidates.push(PathBuf::from(format!("{}.poly", name)));
    for c in &candidates {
        if c.is_file() {
            return std::fs::read_to_string(c)
                .map_err(|e| format!("讀取函式庫 {} 失敗：{}", c.display(), e));
        }
    }
    Err(format!(
        "找不到函式庫：{}（內建 std 或檔案 std/{}.poly）",
        name, name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_poly_basic() {
        let text = "# @intent: 測試\n# @author: llm\n\nfn main() { let a = 5; }";
        let p = load_poly(text).unwrap();
        assert_eq!(p.intent.as_deref(), Some("測試"));
        assert_eq!(p.metadata, vec![("author".to_string(), "llm".to_string())]);
        assert!(p.source.contains("fn main()"));
        assert!(!p.source.contains('#'));
    }

    #[test]
    fn test_load_poly_multi_intent() {
        let text = "# @intent: 第一行\n# @intent: 第二行\nfn main() {}";
        let p = load_poly(text).unwrap();
        assert_eq!(p.intent.as_deref(), Some("第一行\n第二行"));
    }

    #[test]
    fn test_load_poly_plain_comment_ignored() {
        let text = "# 普通註釋\n# @intent: X\nfn main() {}";
        let p = load_poly(text).unwrap();
        assert_eq!(p.intent.as_deref(), Some("X"));
        assert!(p.source.starts_with("fn main()"));
    }

    #[test]
    fn test_load_poly_no_metadata() {
        let text = "fn sqr(x: i32) -> i32 { x * x }\nfn main() { let a = 1; }";
        let p = load_poly(text).unwrap();
        assert!(p.intent.is_none());
        assert!(p.metadata.is_empty());
        assert_eq!(p.source, text);
    }

    #[test]
    fn test_load_poly_import_and_set() {
        let text = "# @import: basic\n# @set init = 5\n# @set k = 1\nfn main() { let x = {{init}}; }";
        let p = load_poly(text).unwrap();
        assert_eq!(p.imports, vec!["basic".to_string()]);
        assert_eq!(p.sets, vec![("init".to_string(), "5".to_string()), ("k".to_string(), "1".to_string())]);
    }

    #[test]
    fn test_apply_sets() {
        let src = "let x = {{init}}; let y = sqr!(x + {{k}});";
        let sets = vec![("init".to_string(), "5".to_string()), ("k".to_string(), "1".to_string())];
        let out = apply_sets(src, &sets).unwrap();
        assert_eq!(out, "let x = 5; let y = sqr!(x + 1);");
    }

    #[test]
    fn test_apply_sets_missing() {
        let src = "let x = {{missing}};";
        let sets: Vec<(String, String)> = vec![];
        assert!(apply_sets(src, &sets).is_err());
    }

    #[test]
    fn test_strip_main() {
        let src = "fn sqr(x: i32) -> i32 { x * x }\nfn main() {\n    let a = 1;\n}\nfn other() -> i32 { 2 }";
        let out = strip_main(src);
        assert!(out.contains("fn sqr"));
        assert!(out.contains("fn other"));
        assert!(!out.contains("fn main"));
    }

    #[test]
    fn test_resolve_import() {
        let text = "# @import: basic\nfn main() { let x = sqr!(3); }";
        let p = resolve(text, None).unwrap();
        assert!(p.source.contains("macro_rules! sqr"));
        assert!(p.source.contains("fn main()"));
    }

    #[test]
    fn test_resolve_template() {
        let text = "# @import: basic\n# @set k = 2\nfn main() { let x = sqr!({{k}}); }";
        let p = resolve(text, None).unwrap();
        assert!(p.source.contains("sqr!(2)"));
        assert!(!p.source.contains("{{"));
    }

    #[test]
    fn test_resolve_cycle() {
        // 內建 std 無循環，這裡只測未知函式庫會報錯
        let text = "# @import: does_not_exist\nfn main() {}";
        assert!(resolve(text, None).is_err());
    }
}
