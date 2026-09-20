// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! LLBC **typed schema 層**（C1 起嘅法證白名單；2026-09-20 serde 遷移）。
//!
//! 設計戒律：
//! 1. **零依賴**：core `[dependencies]` 為空。「字節→JSON 樹」已移交
//!    `frontends/llbc`（真 serde_json）；呢度只剩 [`Value`] 記憶體樹
//!    同「樹→typed LLBC」嘅 schema 行走——邊界：JSON **語法**歸 serde，
//!    LLBC **schema** 歸呢度。
//! 2. **唔畀靜默跳過**：schema 漂移（多 key、少 key、未知 enum variant、型別唔對）一律
//!    `LlbcError`，由調用方決定點處理；walker 自己永遠唔會折叠未知形狀。
//! 3. **pin 對象**：Charon `ca501af6`（見 `charon.pin`）出嘅 LLBC 結構
//!    `{charon_version, translated{13 keys}, has_errors}`；fixture snapshot 入倉
//!    （`core/tests/charon_fixtures/*.llbc`），升 pin 前要行 `scripts/c0_spike.py` 重做 fixtures。

use std::collections::BTreeMap;
use std::fmt;

/// JSON 值（保序 object；number 存原文 token，round-trip 比較唔受浮點化影響）。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_num(&self) -> Option<&str> {
        match self {
            Value::Num(n) => Some(n),
            _ => None,
        }
    }
    /// Canonical 序列化（object 保序）；round-trip 測試係 parse(dump(v)) == v。
    pub fn dump(&self) -> String {
        let mut out = String::new();
        self.dump_into(&mut out);
        out
    }
    fn dump_into(&self, out: &mut String) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(true) => out.push_str("true"),
            Value::Bool(false) => out.push_str("false"),
            Value::Num(n) => out.push_str(n),
            Value::Str(s) => dump_str(s, out),
            Value::Arr(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.dump_into(out);
                }
                out.push(']');
            }
            Value::Obj(o) => {
                out.push('{');
                for (i, (k, v)) in o.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    dump_str(k, out);
                    out.push(':');
                    v.dump_into(out);
                }
                out.push('}');
            }
        }
    }
}

fn dump_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// parser 錯誤——永遠帶 byte offset，調試直接跳位。
#[derive(Debug, Clone, PartialEq)]
pub struct LlbcError {
    pub offset: usize,
    pub msg: String,
}

impl fmt::Display for LlbcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LLBC schema error @byte {}: {}", self.offset, self.msg)
    }
}
impl std::error::Error for LlbcError {}

// ---------------------------------------------------------------------------
// 模式化 LLBC root / translated / fun decl（pinned schema，漂移即 hard error）
// ---------------------------------------------------------------------------

/// fun body 變體白名單：Pin 版本實測只有 `Structured | Error`；
/// 新變體（Charon 升級引入）→ `LlbcError`，唔畀靜默當 Missing 處理（語義漂移防線）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
    Structured,
    Error,
    Missing,
    /// 外部/trait 聲明無 body（string 變體）
    Opaque,
    /// 編譯器 intrinsic（derive 合成；無 LLBC body）
    Intrinsic,
}

/// 抽一個 fun_decl 嘅最細可用投影（body 內容屬 C2/C3 活動範圍，呢度保留原 raw JSON）。
#[derive(Debug, Clone)]
pub struct FunDeclRef {
    pub def_id: i64,
    /// item_meta.name 最尾 segment（通常即函數名；派生/impl item 例外）
    pub name: String,
    pub full_path: Vec<String>,
    pub body_kind: BodyKind,
    pub raw: Value,
}

/// root 容許嘅 key（pinned）——多一個少一個都報錯。
const ROOT_KEYS: [&str; 3] = ["charon_version", "translated", "has_errors"];
/// translated 容許嘅 key（實測 13 個，fixtures 全一致）。
const TRANSLATED_KEYS: [&str; 13] = [
    "assoc_item_names",
    "crate_name",
    "files",
    "fun_decls",
    "global_decls",
    "item_names",
    "options",
    "ordered_decls",
    "short_names",
    "target_information",
    "trait_decls",
    "trait_impls",
    "type_decls",
];

#[derive(Debug, Clone)]
pub struct LlbcRoot {
    pub charon_version: String,
    pub has_errors: bool,
    pub crate_name: String,
    pub funs: Vec<FunDeclRef>,
    pub n_type_decls: usize,
    pub n_global_decls: usize,
    pub n_trait_decls: usize,
    pub n_trait_impls: usize,
    /// translated 原片（type_decls/global_decls 等 C4 再逐類型化）。
    pub raw_translated: Value,
}

fn keyset_check(
    obj: &Value,
    allowed: &[&str],
    ctx: &str,
) -> Result<(), LlbcError> {
    let fields = match obj {
        Value::Obj(f) => f,
        _ => return Err(LlbcError { offset: 0, msg: format!("{ctx}: expected object") }),
    };
    let mut have: BTreeMap<&str, usize> = BTreeMap::new();
    for (k, _) in fields {
        if !allowed.contains(&k.as_str()) {
            return Err(LlbcError {
                offset: 0,
                msg: format!("{ctx}: unknown key `{k}` — schema drift? 升 Charon pin 前要更新白名單+fixtures"),
            });
        }
        *have.entry(k.as_str()).or_default() += 1;
    }
    for (k, n) in &have {
        if *n > 1 {
            return Err(LlbcError { offset: 0, msg: format!("{ctx}: duplicate key `{k}`") });
        }
    }
    if let Some(k) = allowed.iter().copied().find(|k| !have.contains_key(k)) {
        return Err(LlbcError { offset: 0, msg: format!("{ctx}: missing key `{k}`") });
    }
    Ok(())
}

fn num_i64(v: Option<&Value>, ctx: &str) -> Result<i64, LlbcError> {
    let raw = v.and_then(|v| v.as_num()).ok_or_else(|| LlbcError {
        offset: 0,
        msg: format!("{ctx}: expected number"),
    })?;
    raw.parse::<i64>()
        .map_err(|_| LlbcError { offset: 0, msg: format!("{ctx}: number `{raw}` not i64") })
}

/// item_meta.name = [{Ident:[seg,did]}|…] → (full_path, last_or_defid)
fn extract_name(v: Option<&Value>) -> (Vec<String>, String) {
    let mut path = Vec::new();
    if let Some(segs) = v.and_then(|v| v.as_arr()) {
        for seg in segs {
            if let Some(ident) = seg.get("Ident").and_then(|x| x.as_arr()) {
                if let Some(s) = ident.first().and_then(|x| x.as_str()) {
                    path.push(s.to_string());
                }
            }
        }
    }
    let last = path.last().cloned().unwrap_or_else(|| "?".to_string());
    (path, last)
}

impl LlbcRoot {
    /// 由 [`Value`] 樹行 typed schema 檢查。文本→樹請用
    /// `frontends/llbc`（真 serde_json）：`parse_llbc_text` 即係本函數加埋前端。
    pub fn from_value(root: &Value) -> Result<Self, LlbcError> {
        keyset_check(root, &ROOT_KEYS, "root")?;
        let charon_version = root
            .get("charon_version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlbcError { offset: 0, msg: "root.charon_version: expected string".into() })?
            .to_string();
        let has_errors = root
            .get("has_errors")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| LlbcError { offset: 0, msg: "root.has_errors: expected bool".into() })?;
        let translated = root
            .get("translated")
            .ok_or_else(|| LlbcError { offset: 0, msg: "root.translated missing".into() })?;
        keyset_check(translated, &TRANSLATED_KEYS, "translated")?;
        let crate_name = translated
            .get("crate_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlbcError { offset: 0, msg: "translated.crate_name: expected string".into() })?
            .to_string();
        let n_list = |key: &str| -> Result<usize, LlbcError> {
            translated
                .get(key)
                .and_then(|v| v.as_arr())
                .map(|a| a.len())
                .ok_or_else(|| LlbcError { offset: 0, msg: format!("translated.{key}: expected array") })
        };
        let fun_arr = translated
            .get("fun_decls")
            .and_then(|v| v.as_arr())
            .ok_or_else(|| LlbcError { offset: 0, msg: "translated.fun_decls: expected array".into() })?;
        let mut funs = Vec::with_capacity(fun_arr.len());
        for (i, f) in fun_arr.iter().enumerate() {
            let ctx = format!("fun_decls[{i}]");
            if matches!(f, Value::Null) {
                continue; // null 佔位（trait/外部聲明索引對齊用）— 實測喺 while_let fixtures
            }
            let def_id = num_i64(f.get("def_id"), &format!("{ctx}.def_id"))?;
            let (path, name) = extract_name(f.get("item_meta").and_then(|m| m.get("name")));
            let body_kind = match f.get("body") {
                Some(Value::Obj(b)) if b.len() == 1 => match b[0].0.as_str() {
                    "Structured" => BodyKind::Structured,
                    "Error" => BodyKind::Error,
                    "Missing" => BodyKind::Missing,
                    // C4 法證：derive 編譯器 intrinsic（copy/clone/discriminant 等合成 body 無 LLBC）
                    "Intrinsic" => BodyKind::Intrinsic,
                    other => {
                        return Err(LlbcError {
                            offset: 0,
                            msg: format!("{ctx}.body: unknown variant `{other}` — 硬錯（唔畀靜默語義漂移）"),
                        })
                    }
                },
                // C3 法證：外部/trait method body 係 string "Opaque"（loop_return 嘅 Vec::iter 等）
                Some(Value::Str(s)) => match s.as_str() {
                    "Opaque" => BodyKind::Opaque,
                    "Missing" => BodyKind::Missing,
                    other => {
                        return Err(LlbcError {
                            offset: 0,
                            msg: format!("{ctx}.body: unknown string variant `{other}`"),
                        })
                    }
                },
                _ => {
                    return Err(LlbcError {
                        offset: 0,
                        msg: format!("{ctx}.body: expected 1-key object variant or string (Structured|Error|Missing|Opaque)"),
                    })
                }
            };
            funs.push(FunDeclRef { def_id, name, full_path: path, body_kind, raw: f.clone() });
        }
        Ok(LlbcRoot {
            charon_version,
            has_errors,
            crate_name,
            funs,
            n_type_decls: n_list("type_decls")?,
            n_global_decls: n_list("global_decls")?,
            n_trait_decls: n_list("trait_decls")?,
            n_trait_impls: n_list("trait_impls")?,
            raw_translated: translated.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// 測試：fixtures round-trip + schema 硬錯
// ---------------------------------------------------------------------------

/// 測試橋：core 測試經 serde_json（dev-dep）過真 parser，
/// 與 `frontends/llbc::from_serde_value` 逐行同源（bridge 太細，兩處複製成本低過
/// crate 循環；frontend parity 測試用真 fixtures 鎖住一致性）。
#[cfg(test)]
pub(crate) fn value_for_test(src: &str) -> Result<Value, String> {
    fn conv(v: &serde_json::Value) -> Value {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => Value::Num(n.to_string()),
            serde_json::Value::String(s) => Value::Str(s.clone()),
            serde_json::Value::Array(a) => Value::Arr(a.iter().map(conv).collect()),
            serde_json::Value::Object(m) => {
                Value::Obj(m.iter().map(|(k, v)| (k.clone(), conv(v))).collect())
            }
        }
    }
    let v: serde_json::Value = serde_json::from_str(src)
        .map_err(|e| format!("test JSON syntax @{}:{}: {e}", e.line(), e.column()))?;
    Ok(conv(&v))
}

/// 測試橋：文本 → typed root（= frontends/llbc::parse_llbc_text 嘅測試版本）。
#[cfg(test)]
pub(crate) fn root_for_test(src: &str) -> Result<LlbcRoot, LlbcError> {
    let v = value_for_test(src).map_err(|e| LlbcError { offset: 0, msg: e })?;
    LlbcRoot::from_value(&v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charon_llbc::{root_for_test, value_for_test};

    const FIXTURES: [(&str, &str); 10] = [
        ("sqr", include_str!("../tests/charon_fixtures/sqr.llbc")),
        ("add", include_str!("../tests/charon_fixtures/add.llbc")),
        ("max", include_str!("../tests/charon_fixtures/max.llbc")),
        ("while_loop", include_str!("../tests/charon_fixtures/while_loop.llbc")),
        ("match_option", include_str!("../tests/charon_fixtures/match_option.llbc")),
        ("borrow_immut", include_str!("../tests/charon_fixtures/borrow_immut.llbc")),
        ("enum_option", include_str!("../tests/charon_fixtures/enum_option.llbc")),
        ("async_simple", include_str!("../tests/charon_fixtures/async_simple.llbc")),
        ("phase3__loop_sat", include_str!("../tests/charon_fixtures/phase3__loop_sat.llbc")),
        ("struct_point", include_str!("../tests/charon_fixtures/struct_point.llbc")),
    ];

    #[test]
    fn fixtures_parse_and_shape() {
        for (name, src) in FIXTURES {
            let root = root_for_test(src).unwrap_or_else(|e| panic!("fixture {name}: {e}"));
            assert!(!root.funs.is_empty(), "fixture {name}: no fun_decls");
            assert!(!root.charon_version.is_empty(), "fixture {name}");
        }
    }

    #[test]
    fn fixtures_roundtrip() {
        for (name, src) in FIXTURES {
            let v = value_for_test(src).unwrap_or_else(|e| panic!("{name}: {e}"));
            let v2 = value_for_test(&v.dump()).unwrap();
            assert_eq!(v, v2, "fixture {name} round-trip mismatch");
        }
    }

    #[test]
    fn async_fixture_is_error_body() {
        // pinned：async desugar 部分 body 抽唔到 → has_errors + body=Error variant
        let root = root_for_test(FIXTURES.iter().find(|(n, _)| *n == "async_simple").unwrap().1).unwrap();
        assert!(root.has_errors);
        assert!(root.funs.iter().any(|f| f.body_kind == BodyKind::Error));
    }

    #[test]
    fn hard_error_on_unknown_root_key() {
        let v = value_for_test(r#"{"charon_version":"0.1","translated":{},"has_errors":false,"surprise":1}"#).unwrap();
        let e = LlbcRoot::from_value(&v).unwrap_err();
        assert!(e.msg.contains("unknown key"), "{}", e.msg);
    }

    #[test]
    fn hard_error_on_unknown_body_variant() {
        // 攞真 fixture 細改 body variant → 必須 hard error（唔准靜默當 Missing）
        let src = FIXTURES.iter().find(|(n, _)| *n == "max").unwrap().1
            .replacen(r#""Structured""#, r#""WatVariant""#, 1);
        let e = LlbcRoot::from_value(&value_for_test(&src).unwrap()).unwrap_err();
        assert!(e.msg.contains("unknown variant"), "{}", e.msg);
    }

    #[test]
    fn hard_error_on_missing_translated_key() {
        let src = FIXTURES.iter().find(|(n, _)| *n == "add").unwrap().1
            .replacen(r#""trait_impls""#, r#""trait_implz""#, 1);
        let e = LlbcRoot::from_value(&value_for_test(&src).unwrap()).unwrap_err();
        assert!(e.msg.contains("unknown key") || e.msg.contains("missing key"), "{}", e.msg);
    }

    #[test]
    fn json_number_and_unicode() {
        let v = value_for_test(r#"{"a":-12.5e3,"s":"x\u4e2d\u6587\n"}"#).unwrap();
        // serde_json 規範化數字原文（-12.5e3 → -12500.0）；LLBC number 全部係
        // id/整數字面量，唔受影響（num_i64 只係 schema 層讀整數位）。
        assert_eq!(v.get("a").unwrap().as_num(), Some("-12500.0"));
        assert_eq!(v.get("s").unwrap().as_str(), Some("x中文\n"));
        // 代理對（真 serde_json 解）
        let v2 = value_for_test(r#"["\ud83d\ude00"]"#).unwrap();
        assert_eq!(v2.as_arr().unwrap()[0].as_str(), Some("😀"));
    }
}
