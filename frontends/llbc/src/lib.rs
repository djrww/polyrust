// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! LLBC 前端：**真 serde_json 文本解析** → core 零依賴 [`Value`] 樹。
//!
//! 職責分界（架構決策 2026-09-20）：
//! - 呢個 crate 擁有「字節 → JSON 樹」嘅全部職責，用業界標準 serde_json
//!   （代理對、轉義、數字邊界、巢深限制全部由 serde_json 保證）。
//! - `polyrust-core` 唔再自寫 JSON lexer：core 只持有 [`Value`] 記憶體樹
//!   同「樹 → typed LLBC」嘅**法證白名單 schema 行走**（`LlbcRoot::from_value`，
//!   C2 起嘅硬錯防線，唔屬於 JSON parsing，永不搬走）。
//! - core `[dependencies]` 維持為空；serde 全部喺 frontends/*。
//!
//! 主入口：
//! - [`parse_json_value`]：LLBC 文本 → [`Value`] 樹
//! - [`parse_llbc_text`] / [`parse_llbc_file`]：直達 typed [`LlbcRoot`]

use polyrust_core::charon_llbc::{LlbcError, LlbcRoot, Value};

/// serde_json 值 → core 零依賴 [`Value`] 樹。
///
/// 數字以 serde_json 規範化後嘅原文保存（整數完全冇損；LLBC 全部 number
/// 都係 id/整數字面量，唔會出現浮點 token）。物件成員**保序**，同 Charon
/// 輸出一致，下游 round-trip / keyset 檢查唔受影響。
pub fn from_serde_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => Value::Num(n.to_string()),
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Array(a) => Value::Arr(a.iter().map(from_serde_value).collect()),
        serde_json::Value::Object(m) => {
            Value::Obj(m.iter().map(|(k, v)| (k.clone(), from_serde_value(v))).collect())
        }
    }
}

/// LLBC JSON 文本 → core [`Value`] 樹（真 serde_json 解析）。
///
/// 語法層錯誤（壞 JSON）以 serde_json 自帶 line/column 報錯，包入
/// [`LlbcError`] 維持統一錯誤型。
pub fn parse_json_value(src: &str) -> Result<Value, LlbcError> {
    let v: serde_json::Value =
        serde_json::from_str(src).map_err(|e| LlbcError {
            offset: 0,
            msg: format!("JSON 語法錯（serde_json @{}:{}）: {e}", e.line(), e.column()),
        })?;
    Ok(from_serde_value(&v))
}

/// LLBC JSON 文本 → typed [`LlbcRoot`]（前端總入口）。
pub fn parse_llbc_text(src: &str) -> Result<LlbcRoot, LlbcError> {
    let v = parse_json_value(src)?;
    LlbcRoot::from_value(&v)
}

/// LLBC JSON 檔 → typed [`LlbcRoot`]。
pub fn parse_llbc_file(path: &std::path::Path) -> Result<LlbcRoot, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("讀取 {} 失敗：{e}", path.display()))?;
    parse_llbc_text(&src).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真 fixture（真 charon 對真 rustc nightly 輸出）行全鏈。
    #[test]
    fn real_fixture_round_trip_through_serde() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../core/tests/charon_fixtures/sqr.llbc");
        let root = parse_llbc_file(&path).expect("sqr fixture parse");
        // charon 對 stdin 源檔編譯，crate_name 固定叫 "input"（法證實測值）
        assert_eq!(root.crate_name, "input");
        assert!(root.funs.iter().any(|f| f.name == "sqr"));
    }

    #[test]
    fn syntax_error_reports_line_column() {
        let e = parse_llbc_text("{ not json").unwrap_err();
        assert!(e.msg.contains("serde_json"), "{e}");
    }

    #[test]
    fn number_normalization_preserves_i64_extraction() {
        // LLBC 只用整數：1e3 呢類寫法 Charon 唔會出，但 serde 規範化後仍然可 parse_i64
        let v = parse_json_value(r#"{"a":42,"b":-7}"#).unwrap();
        assert_eq!(v.get("a").unwrap().as_num(), Some("42"));
        assert_eq!(v.get("b").unwrap().as_num(), Some("-7"));
    }

    #[test]
    fn unicode_and_surrogate_pairs_from_real_serde() {
        let v = parse_json_value(r#"{"s":"x中文\n","e":"😀"}"#).unwrap();
        assert_eq!(v.get("s").unwrap().as_str(), Some("x中文\n"));
        assert_eq!(v.get("e").unwrap().as_str(), Some("😀"));
    }
}
