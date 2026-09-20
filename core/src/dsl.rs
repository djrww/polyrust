// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
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

    // ── Phase2 契約與配置 ──
    /// `@fuel`：迴圈有界展開次數，預設 None 表示使用預設 3
    pub fuel: Option<usize>,
    /// `@invariant`：迴圈不變量列表
    pub invariants: Vec<String>,
    /// `@pure`：是否純函數（None=未指定，Some(true)=純，Some(false)=允許效應）
    pub pure: Option<bool>,
    /// `@requires`：前置條件
    pub requires: Vec<String>,
    /// `@ensures`：後置條件
    pub ensures: Vec<String>,
    /// `@unsafe-allowed` 或 `@unsafe_allowed`：是否允許 unsafe
    pub unsafe_allowed: bool,
    /// `@lifetime`：生命週期 outlives 關係，如 `'a: 'b`
    pub lifetimes: Vec<String>,
    /// `@no-io`：禁止 I/O
    pub no_io: bool,
    /// `@qap` / `@cdcl` 等布爾開關（保留原始值）
    pub qap: Option<bool>,
    /// `@type-universe`：如 auto, 7, 7+3, 17
    pub type_universe: Option<String>,
    /// `@mode`：如 full, check, lower
    pub mode: Option<String>,
}

fn parse_bool_value(s: &str) -> Option<bool> {
    let t = s.trim().to_ascii_lowercase();
    match t.as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        "" => Some(true), // presence means true
        _ => None,
    }
}

/// 解析 `@key` 行：支持 `@key: value`, `@key = value`, `@key value`, `@key`
fn parse_at_key(kv: &str) -> (String, String) {
    let kv = kv.trim();
    if kv.is_empty() {
        return (String::new(), String::new());
    }
    // key 是首個 token，直到空白、冒號或等號
    let mut key_end = kv.len();
    for (i, c) in kv.char_indices() {
        if c.is_whitespace() || c == ':' || c == '=' {
            key_end = i;
            break;
        }
    }
    let k = kv[..key_end].trim().to_string();
    let mut rest = kv[key_end..].trim_start();
    // 跳過開頭的 : 或 = 以及空白，僅一層（保留 >= 中的 =）
    if rest.starts_with(':') || rest.starts_with('=') {
        // 若 rest 以 : 或 = 開頭，且下一個字符是空白或為單個分隔符，則視為分隔符
        // 但若是 >=, <=, ==, != 等，則保留
        // 簡單策略：若 rest 長度>=2 且第二個字符是 = 或 > < 等，且第一個是 > < = !，則不視為分隔
        // 否則視為分隔
        let first = rest.chars().next().unwrap();
        let second = rest.chars().nth(1);
        let is_comparison = matches!(first, '>' | '<' | '=' | '!') && matches!(second, Some('='));
        if !is_comparison {
            rest = rest[1..].trim_start();
            // 可能還有第二個分隔符，如 ": =" 或 "= :"
            if rest.starts_with(':') || rest.starts_with('=') {
                let second_first = rest.chars().next().unwrap();
                let second_second = rest.chars().nth(1);
                let second_is_comp = matches!(second_first, '>' | '<' | '=' | '!') && matches!(second_second, Some('='));
                if !second_is_comp {
                    rest = rest[1..].trim_start();
                }
            }
        }
    }
    (k, rest.to_string())
}

/// 由 `.poly` 文本載入。`#` metadata 行會被剝離，其餘原樣保留。
pub fn load_poly(text: &str) -> Result<PolySource, String> {
    let mut intent_lines: Vec<String> = Vec::with_capacity(2);
    let mut metadata: Vec<(String, String)> = Vec::with_capacity(8);
    let mut imports: Vec<String> = Vec::with_capacity(4);
    let mut sets: Vec<(String, String)> = Vec::with_capacity(4);
    let mut src_lines: Vec<&str> = Vec::with_capacity(text.lines().count());

    // Phase2 契約 — 優化 with_capacity
    let mut fuel: Option<usize> = None;
    let mut invariants: Vec<String> = Vec::with_capacity(2);
    let mut pure: Option<bool> = None;
    let mut requires: Vec<String> = Vec::with_capacity(2);
    let mut ensures: Vec<String> = Vec::with_capacity(2);
    let mut unsafe_allowed = false;
    let mut lifetimes: Vec<String> = Vec::with_capacity(2);
    let mut no_io = false;
    let mut qap: Option<bool> = None;
    let mut type_universe: Option<String> = None;
    let mut mode: Option<String> = None;

    for raw in text.lines() {
        let trimmed = raw.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            // metadata / 註釋行
            let rest = rest.trim();
            if let Some(kv) = rest.strip_prefix('@') {
                let kv = kv.trim();
                // @set 特殊處理
                if kv.starts_with("set ") || kv.starts_with("set:") || kv.starts_with("set=") {
                    let body = kv.strip_prefix("set").unwrap().trim();
                    let body = body.trim_start_matches(|c| c == ':' || c == '=').trim();
                    if let Some((k, v)) = body.split_once('=') {
                        sets.push((k.trim().to_string(), v.trim().to_string()));
                    } else if let Some((k, v)) = body.split_once(':') {
                        sets.push((k.trim().to_string(), v.trim().to_string()));
                    } else if !body.is_empty() {
                        // 無分隔，視為 key 無 value？忽略
                    }
                    continue;
                }

                let (k, v) = parse_at_key(kv);
                let k_lower = k.to_ascii_lowercase();
                match k_lower.as_str() {
                    "intent" => {
                        if !v.is_empty() { intent_lines.push(v.clone()); }
                        // intent 不入 metadata，保持舊行為兼容
                    }
                    "import" | "imports" => {
                        if !v.is_empty() {
                            for part in v.split(',') {
                                let name = part.trim();
                                if !name.is_empty() {
                                    imports.push(name.to_string());
                                }
                            }
                        }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "fuel" => {
                        if let Ok(n) = v.trim().parse::<usize>() {
                            fuel = Some(n);
                        } else if let Some(first) = v.split_whitespace().next() {
                            if let Ok(n) = first.parse::<usize>() {
                                fuel = Some(n);
                            }
                        }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "invariant" | "invariants" => {
                        if !v.is_empty() { invariants.push(v.clone()); }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "pure" => {
                        if let Some(b) = parse_bool_value(&v) {
                            pure = Some(b);
                        } else {
                            pure = Some(true);
                        }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "requires" | "require" | "pre" => {
                        if !v.is_empty() { requires.push(v.clone()); }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "ensures" | "ensure" | "post" => {
                        if !v.is_empty() { ensures.push(v.clone()); }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "unsafe-allowed" | "unsafe_allowed" | "unsafe" => {
                        if let Some(b) = parse_bool_value(&v) {
                            unsafe_allowed = b;
                        } else {
                            unsafe_allowed = true;
                        }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "lifetime" | "lifetimes" | "outlives" => {
                        if !v.is_empty() { lifetimes.push(v.clone()); }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "no-io" | "no_io" | "noio" => {
                        if let Some(b) = parse_bool_value(&v) {
                            no_io = b;
                        } else {
                            no_io = true;
                        }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "qap" => {
                        if let Some(b) = parse_bool_value(&v) {
                            qap = Some(b);
                        }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "type-universe" | "type_universe" | "types" => {
                        if !v.is_empty() { type_universe = Some(v.clone()); }
                        metadata.push((k.clone(), v.clone()));
                    }
                    "mode" => {
                        if !v.is_empty() { mode = Some(v.clone()); }
                        metadata.push((k.clone(), v.clone()));
                    }
                    _ => {
                        metadata.push((k.clone(), v.clone()));
                    }
                }
            } else if rest.starts_with('[') || rest.starts_with("![") {
                // Rust 屬性如 #[pure] / #![allow]——保留為源碼，非 DSL 註釋
                src_lines.push(raw);
            } else {
                // 普通註釋，忽略
            }
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
        fuel,
        invariants,
        pure,
        requires,
        ensures,
        unsafe_allowed,
        lifetimes,
        no_io,
        qap,
        type_universe,
        mode,
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
    ("vec", include_str!("../std/vec.poly")),
    ("string", include_str!("../std/string.poly")),
    ("hashmap", include_str!("../std/hashmap.poly")),
    ("option", "enum Option<T> { Some(T), None }"),
    ("result", "enum Result<T,E> { Ok(T), Err(E) }"),
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
        fuel: poly.fuel,
        invariants: poly.invariants.clone(),
        pure: poly.pure,
        requires: poly.requires.clone(),
        ensures: poly.ensures.clone(),
        unsafe_allowed: poly.unsafe_allowed,
        lifetimes: poly.lifetimes.clone(),
        no_io: poly.no_io,
        qap: poly.qap,
        type_universe: poly.type_universe.clone(),
        mode: poly.mode.clone(),
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

