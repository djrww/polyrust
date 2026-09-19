// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! std-only LLBC JSON 子集 parser（C1 里程碑）。
//!
//! 設計戒律（藍圖 C1 驗收口徑）：
//! 1. **零依賴**：唔用 serde——core 保持 wasm/審計友好，JSON 子集自家實作。
//! 2. **唔畀靜默跳過**：schema 漂移（多 key、少 key、未知 enum variant、型別唔對）一律
//!    `LlbcError`，由調用方決定點處理；parser 自己永遠唔會折叠未知形狀。
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

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

pub fn parse_json(src: &str) -> Result<Value, LlbcError> {
    let mut p = Parser { src: src.as_bytes(), pos: 0 };
    let v = p.value()?;
    p.skip_ws();
    if p.pos != p.src.len() {
        return Err(p.err("trailing bytes after JSON value"));
    }
    Ok(v)
}

impl<'a> Parser<'a> {
    fn err(&self, msg: &str) -> LlbcError {
        LlbcError { offset: self.pos, msg: msg.to_string() }
    }
    fn skip_ws(&mut self) {
        while let Some(&b) = self.src.get(self.pos) {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }
    fn eat(&mut self, b: u8, what: &str) -> Result<(), LlbcError> {
        self.skip_ws();
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(&format!("expected `{}` ({})", b as char, what)))
        }
    }
    fn value(&mut self) -> Result<Value, LlbcError> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Value::Str),
            Some(b't') => self.lit("true", Value::Bool(true)),
            Some(b'f') => self.lit("false", Value::Bool(false)),
            Some(b'n') => self.lit("null", Value::Null),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            Some(c) => {
                self.pos += 1;
                Err(self.err(&format!("unexpected byte 0x{:02x} starting JSON value", c)))
            }
            None => Err(self.err("unexpected end of input")),
        }
    }
    fn lit(&mut self, word: &str, v: Value) -> Result<Value, LlbcError> {
        if self.src[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(v)
        } else {
            Err(self.err(&format!("bad literal (want {})", word)))
        }
    }
    fn number(&mut self) -> Result<Value, LlbcError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        let mut saw = false;
        while let Some(c) = self.peek() {
            match c {
                b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-' => {
                    saw = saw || c.is_ascii_digit();
                    self.pos += 1;
                }
                _ => break,
            }
        }
        if !saw {
            return Err(self.err("number without digits"));
        }
        Ok(Value::Num(
            std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("").to_string(),
        ))
    }
    fn string(&mut self) -> Result<String, LlbcError> {
        self.skip_ws();
        if self.peek() != Some(b'"') {
            return Err(self.err("expected string"));
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            let c = self.peek().ok_or_else(|| self.err("unterminated string"))?;
            self.pos += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let e = self.peek().ok_or_else(|| self.err("unterminated escape"))?;
                    self.pos += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.hex4()?;
                            if (0xd800..0xdc00).contains(&cp) {
                                // surrogate pair
                                if self.peek() == Some(b'\\') {
                                    self.pos += 1;
                                    if self.peek() == Some(b'u') {
                                        self.pos += 1;
                                        let lo = self.hex4()?;
                                        let c = 0x10000
                                            + ((cp - 0xd800) << 10)
                                            + (lo.wrapping_sub(0xdc00) & 0x3ff);
                                        out.push(char::from_u32(c).unwrap_or('\u{fffd}'));
                                    } else {
                                        out.push('\u{fffd}');
                                    }
                                } else {
                                    out.push('\u{fffd}');
                                }
                            } else {
                                out.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
                            }
                        }
                        _ => return Err(self.err("bad escape char")),
                    }
                }
                c if c < 0x80 => out.push(c as char),
                c => {
                    // UTF-8 continuation (2/3/4 bytes)
                    let n = match c {
                        0xc0..=0xdf => 1,
                        0xe0..=0xef => 2,
                        0xf0..=0xf7 => 3,
                        _ => return Err(self.err("bad UTF-8 lead byte")),
                    };
                    let start = self.pos - 1;
                    self.pos = (self.pos + n).min(self.src.len());
                    out.push_str(std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("\u{fffd}"));
                }
            }
        }
    }
    fn hex4(&mut self) -> Result<u32, LlbcError> {
        let mut v = 0u32;
        for _ in 0..4 {
            let c = self.peek().ok_or_else(|| self.err("short \\u escape"))?;
            self.pos += 1;
            v = v * 16
                + match c {
                    b'0'..=b'9' => (c - b'0') as u32,
                    b'a'..=b'f' => (c - b'a' + 10) as u32,
                    b'A'..=b'F' => (c - b'A' + 10) as u32,
                    _ => return Err(self.err("bad hex in \\u escape")),
                };
        }
        Ok(v)
    }
    fn array(&mut self) -> Result<Value, LlbcError> {
        self.pos += 1; // '['
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Value::Arr(out));
        }
        loop {
            out.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Value::Arr(out));
                }
                _ => return Err(self.err("expected `,` or `]` in array")),
            }
        }
    }
    fn object(&mut self) -> Result<Value, LlbcError> {
        self.pos += 1; // '{'
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Value::Obj(out));
        }
        loop {
            self.skip_ws();
            let k = self.string()?;
            self.eat(b':', "object colon")?;
            let v = self.value()?;
            out.push((k, v));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Value::Obj(out));
                }
                _ => return Err(self.err("expected `,` or `}` in object")),
            }
        }
    }
}

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
    for k in allowed.iter().copied().filter(|k| !have.contains_key(k)) {
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
    pub fn parse(src: &str) -> Result<Self, LlbcError> {
        let root = parse_json(src)?;
        keyset_check(&root, &ROOT_KEYS, "root")?;
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
            let def_id = num_i64(f.get("def_id"), &format!("{ctx}.def_id"))?;
            let (path, name) = extract_name(f.get("item_meta").and_then(|m| m.get("name")));
            let body_kind = match f.get("body") {
                Some(Value::Obj(b)) if b.len() == 1 => match b[0].0.as_str() {
                    "Structured" => BodyKind::Structured,
                    "Error" => BodyKind::Error,
                    "Missing" => BodyKind::Missing,
                    other => {
                        return Err(LlbcError {
                            offset: 0,
                            msg: format!("{ctx}.body: unknown variant `{other}` — 硬錯（唔畀靜默語義漂移）"),
                        })
                    }
                },
                _ => {
                    return Err(LlbcError {
                        offset: 0,
                        msg: format!("{ctx}.body: expected 1-key object variant (Structured|Error|Missing)"),
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: [(&str, &str); 11] = [
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
        ("io_pure", include_str!("../tests/charon_fixtures/io_pure.llbc")),
    ];

    #[test]
    fn fixtures_parse_and_shape() {
        for (name, src) in FIXTURES {
            let root = LlbcRoot::parse(src).unwrap_or_else(|e| panic!("fixture {name}: {e}"));
            assert!(!root.funs.is_empty(), "fixture {name}: no fun_decls");
            assert_eq!(root.charon_version.is_empty(), false, "fixture {name}");
        }
    }

    #[test]
    fn fixtures_roundtrip() {
        for (name, src) in FIXTURES {
            let v = parse_json(src).unwrap_or_else(|e| panic!("{name}: {e}"));
            let v2 = parse_json(&v.dump()).unwrap();
            assert_eq!(v, v2, "fixture {name} round-trip mismatch");
        }
    }

    #[test]
    fn async_fixture_is_error_body() {
        // pinned：async desugar 部分 body 抽唔到 → has_errors + body=Error variant
        let root = LlbcRoot::parse(FIXTURES.iter().find(|(n, _)| *n == "async_simple").unwrap().1).unwrap();
        assert!(root.has_errors);
        assert!(root.funs.iter().any(|f| f.body_kind == BodyKind::Error));
    }

    #[test]
    fn hard_error_on_unknown_root_key() {
        let bad = r#"{"charon_version":"0.1","translated":{},"has_errors":false,"surprise":1}"#;
        let e = LlbcRoot::parse(bad).unwrap_err();
        assert!(e.msg.contains("unknown key"), "{}", e.msg);
    }

    #[test]
    fn hard_error_on_unknown_body_variant() {
        // 攞真 fixture 細改 body variant → 必須 hard error（唔准靜默當 Missing）
        let src = FIXTURES.iter().find(|(n, _)| *n == "max").unwrap().1
            .replacen(r#""Structured""#, r#""WatVariant""#, 1);
        let e = LlbcRoot::parse(&src).unwrap_err();
        assert!(e.msg.contains("unknown variant"), "{}", e.msg);
    }

    #[test]
    fn hard_error_on_missing_translated_key() {
        let src = FIXTURES.iter().find(|(n, _)| *n == "add").unwrap().1
            .replacen(r#""trait_impls""#, r#""trait_implz""#, 1);
        let e = LlbcRoot::parse(&src).unwrap_err();
        assert!(e.msg.contains("unknown key") || e.msg.contains("missing key"), "{}", e.msg);
    }

    #[test]
    fn json_number_and_unicode() {
        let v = parse_json(r#"{"a":-12.5e3,"s":"x\u4e2d\u6587\n"}"#).unwrap();
        assert_eq!(v.get("a").unwrap().as_num(), Some("-12.5e3"));
        assert_eq!(v.get("s").unwrap().as_str(), Some("x中文\n"));
        // 代理對
        let v2 = parse_json(r#"["\ud83d\ude00"]"#).unwrap();
        assert_eq!(v2.as_arr().unwrap()[0].as_str(), Some("😀"));
    }
}
