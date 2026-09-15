//! 極簡 JSON 序列化（無外部依賴），供 `--json` 出口與 LLM 接口使用。
//!
//! 只支援本專案需要的型態：字串、布林、整數、Option、Vec、以及手寫物件。
//! 用 `J` 表示 JSON 值。

#[derive(Clone, Debug)]
pub enum J {
    Str(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Null,
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    pub fn s(v: &str) -> J {
        J::Str(v.to_string())
    }
    pub fn obj(pairs: Vec<(&str, J)>) -> J {
        J::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
    pub fn opt_bool(v: Option<bool>) -> J {
        match v {
            Some(b) => J::Bool(b),
            None => J::Null,
        }
    }
    pub fn opt_str(v: Option<&str>) -> J {
        match v {
            Some(s) => J::Str(s.to_string()),
            None => J::Null,
        }
    }
    fn write(&self, out: &mut String) {
        match self {
            J::Str(s) => {
                out.push('"');
                for c in s.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        '\n' => out.push_str("\\n"),
                        '\r' => out.push_str("\\r"),
                        '\t' => out.push_str("\\t"),
                        c if (c as u32) < 0x20 => {
                            out.push_str(&format!("\\u{:04x}", c as u32))
                        }
                        c => out.push(c),
                    }
                }
                out.push('"');
            }
            J::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            J::Int(n) => out.push_str(&n.to_string()),
            J::Float(f) => {
                // 避免科學記號與無窮/NaN（JSON 不接受）
                if f.is_finite() {
                    let s = format!("{}", f);
                    out.push_str(&s);
                } else {
                    out.push_str("null");
                }
            }
            J::Null => out.push_str("null"),
            J::Arr(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    item.write(out);
                }
                out.push(']');
            }
            J::Obj(pairs) => {
                out.push('{');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    J::Str(k.clone()).write(out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
}

impl std::fmt::Display for J {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        self.write(&mut out);
        f.write_str(&out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        assert_eq!(J::s("a\"b\\c\nd").to_string(), "\"a\\\"b\\\\c\\nd\"");
    }

    #[test]
    fn test_nested() {
        let v = J::obj(vec![
            ("name", J::s("x")),
            ("ok", J::Bool(true)),
            ("n", J::Int(42)),
            ("none", J::Null),
            ("list", J::Arr(vec![J::Int(1), J::Int(2)])),
        ]);
        assert_eq!(
            v.to_string(),
            "{\"name\":\"x\",\"ok\":true,\"n\":42,\"none\":null,\"list\":[1,2]}"
        );
    }
}
