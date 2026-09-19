//! Chalk/rustc 判決橋：外部判決產物（JSON artifact）→ [`crate::certify::OracleBits`]。
//!
//! # 定位（見 docs/THEOREMS.md §6b「認證路徑」）
//! 過渡到真 rustc 後，rustc + Chalk 負責判決（推斷型別、宏選臂、借用活性），
//! polyrust 只做**認證**：由判決產物重建位元賦值 σ，再對約束系統逐多項式
//! 直接求值驗證（多項式時間，與 2ⁿ 無關）。本模組是判決產物的**輸入接口**：
//!
//! ```json
//! {
//!   "format": "polyrust-oracle/1",
//!   "body": "main",
//!   "node_types":  [ { "node": 1, "chosen": 0 } ],
//!   "arm_choices": [ { "invoke": 2, "chosen": 1 } ],
//!   "borrows":     [ { "node": 3, "live": true } ]
//! }
//! ```
//!
//! 位元位置對應關係由 [`BitLayout`]（從約束系統 [`System`] 提取）定義；
//! `chosen` 是候選列表中的索引。未知節點/越界索引一律報錯（防錯字靜默通過）。
//!
//! # 本地閉環
//! 管線自身就是第一個「rustc」：σ → 判決（node_types/arm_choice）→ artifact JSON
//! → 本橋 → OracleBits → [`crate::certify::certify_from_oracle`] → certified。
//! 測試鎖定此閉環（含竄改拒絕），未來接真 Chalk 只需換 artifact 生產端。

use crate::certify::OracleBits;
use crate::json::J;
use crate::minirust::constraints::System;
use std::collections::BTreeMap;

// ── 極簡 JSON 解析器（零依賴；只供本橋讀 artifact）──

struct P<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn new(s: &'a str) -> P<'a> {
        P { b: s.as_bytes(), i: 0 }
    }
    fn ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }
    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("位置 {}：期望 '{}'，得到 {:?}", self.i, c as char, self.peek().map(|x| x as char)))
        }
    }
    fn ident(&mut self, word: &str) -> Result<(), String> {
        self.ws();
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(())
        } else {
            Err(format!("位置 {}：期望 {}", self.i, word))
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err("字串未閉合".into()),
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.i += 1;
                    match self.peek() {
                        Some(b'n') => out.push('\n'),
                        Some(b't') => out.push('\t'),
                        Some(b'r') => out.push('\r'),
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'/') => out.push('/'),
                        Some(b'u') => {
                            if self.i + 4 >= self.b.len() {
                                return Err("\\u 轉義不完整".into());
                            }
                            let hex = std::str::from_utf8(&self.b[self.i + 1..self.i + 5])
                                .map_err(|_| "非法 UTF-8".to_string())?;
                            let cp = u32::from_str_radix(hex, 16)
                                .map_err(|_| format!("非法 \\u 轉義 \\u{}", hex))?;
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                            self.i += 4;
                        }
                        other => return Err(format!("未知轉義 {:?}", other.map(|x| x as char))),
                    }
                    self.i += 1;
                }
                Some(_) => {
                    // 逐位元組收集 UTF-8（多位元組字元原樣拷貝）
                    let start = self.i;
                    while self.i < self.b.len() && self.b[self.i] != b'"' && self.b[self.i] != b'\\' {
                        self.i += 1;
                    }
                    let chunk = std::str::from_utf8(&self.b[start..self.i])
                        .map_err(|_| "非法 UTF-8".to_string())?;
                    out.push_str(chunk);
                }
            }
        }
    }
    fn number(&mut self) -> Result<i64, String> {
        self.ws();
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.i += 1;
        }
        if start == self.i {
            return Err(format!("位置 {}：期望數字", start));
        }
        std::str::from_utf8(&self.b[start..self.i])
            .unwrap()
            .parse::<i64>()
            .map_err(|e| format!("數字溢出/非法：{}", e))
    }
    fn value(&mut self) -> Result<J, String> {
        self.ws();
        match self.peek() {
            Some(b'{') => {
                self.i += 1;
                let mut pairs: Vec<(String, J)> = Vec::new();
                self.ws();
                if self.peek() == Some(b'}') {
                    self.i += 1;
                    return Ok(J::Obj(pairs));
                }
                loop {
                    let k = self.string()?;
                    self.eat(b':')?;
                    let v = self.value()?;
                    pairs.push((k, v));
                    self.ws();
                    match self.peek() {
                        Some(b',') => {
                            self.i += 1;
                        }
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(J::Obj(pairs));
                        }
                        _ => return Err(format!("位置 {}：期望 ',' 或 '}}'", self.i)),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.peek() == Some(b']') {
                    self.i += 1;
                    return Ok(J::Arr(items));
                }
                loop {
                    items.push(self.value()?);
                    self.ws();
                    match self.peek() {
                        Some(b',') => {
                            self.i += 1;
                        }
                        Some(b']') => {
                            self.i += 1;
                            return Ok(J::Arr(items));
                        }
                        _ => return Err(format!("位置 {}：期望 ',' 或 ']'", self.i)),
                    }
                }
            }
            Some(b'"') => Ok(J::Str(self.string()?)),
            Some(b't') => {
                self.ident("true")?;
                Ok(J::Bool(true))
            }
            Some(b'f') => {
                self.ident("false")?;
                Ok(J::Bool(false))
            }
            Some(b'n') => {
                self.ident("null")?;
                Ok(J::Null)
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => Ok(J::Int(self.number()?)),
            _ => Err(format!("位置 {}：非預期字元 {:?}", self.i, self.peek().map(|x| x as char))),
        }
    }
}

/// 解析 JSON 文本為 [`J`]。
pub fn parse_json(text: &str) -> Result<J, String> {
    let mut p = P::new(text);
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("位置 {}：JSON 之後有多餘內容", p.i));
    }
    Ok(v)
}

fn obj_get<'j>(o: &'j J, key: &str) -> Result<&'j J, String> {
    match o {
        J::Obj(pairs) => pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("缺欄位 \"{}\"", key)),
        _ => Err("期望物件".into()),
    }
}

fn as_u64(j: &J, what: &str) -> Result<u64, String> {
    match j {
        J::Int(n) if *n >= 0 => Ok(*n as u64),
        _ => Err(format!("{}：期望非負整數", what)),
    }
}

fn as_arr<'j>(j: &'j J, what: &str) -> Result<&'j Vec<J>, String> {
    match j {
        J::Arr(a) => Ok(a),
        _ => Err(format!("{}：期望陣列", what)),
    }
}

// ── Artifact 模型 ──

/// Chalk/rustc 判決產物（型別/臂/借用的「選擇」）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChalkArtifact {
    pub body: String,
    /// (節點 id, 候選索引)
    pub node_types: Vec<(u64, usize)>,
    /// (invoke 節點 id, 臂索引)
    pub arm_choices: Vec<(u64, usize)>,
    /// (借用節點 id, 是否存活)
    pub borrows: Vec<(u64, bool)>,
}

/// 解析 polyrust-oracle/1 格式的 artifact。
pub fn parse_artifact(text: &str) -> Result<ChalkArtifact, String> {
    let v = parse_json(text)?;
    let fmt = match obj_get(&v, "format")? {
        J::Str(s) => s.clone(),
        _ => return Err("format：期望字串".into()),
    };
    if fmt != "polyrust-oracle/1" {
        return Err(format!("不支援的格式：{}", fmt));
    }
    let body = match obj_get(&v, "body") {
        Ok(J::Str(s)) => s.clone(),
        Ok(_) => return Err("body：期望字串".into()),
        Err(_) => "main".to_string(),
    };
    let mut art = ChalkArtifact { body, ..Default::default() };
    for item in as_arr(obj_get(&v, "node_types")?, "node_types")? {
        let node = as_u64(obj_get(item, "node")?, "node_types.node")?;
        let chosen = as_u64(obj_get(item, "chosen")?, "node_types.chosen")? as usize;
        art.node_types.push((node, chosen));
    }
    for item in as_arr(obj_get(&v, "arm_choices")?, "arm_choices")? {
        let invoke = as_u64(obj_get(item, "invoke")?, "arm_choices.invoke")?;
        let chosen = as_u64(obj_get(item, "chosen")?, "arm_choices.chosen")? as usize;
        art.arm_choices.push((invoke, chosen));
    }
    // borrows 為可選欄位（向後相容 v0 舊 artifact）
    match obj_get(&v, "borrows") {
        Ok(J::Arr(items)) => {
            for item in items {
                let node = as_u64(obj_get(item, "node")?, "borrows.node")?;
                let live = match obj_get(item, "live")? {
                    J::Bool(b) => *b,
                    _ => return Err("borrows.live：期望布林".into()),
                };
                art.borrows.push((node, live));
            }
        }
        Ok(_) => return Err("borrows：期望陣列".into()),
        Err(_) => {}
    }
    Ok(art)
}

// ── 位元布局與轉換 ──

/// 約束系統的位元布局：語義選擇 → 位元變量索引。
#[derive(Clone, Debug, Default)]
pub struct BitLayout {
    pub nvars: usize,
    /// 節點 → 各候選型別的位元變量（與 `System::node_type` 對應）。
    pub node_type: BTreeMap<u64, Vec<usize>>,
    /// invoke 節點 → 各臂的位元變量（與 `System::arm_vars` 對應）。
    pub arm: BTreeMap<u64, Vec<usize>>,
    /// 借用節點 → 活性位元（與 `System::borrow_vars` 對應）。
    pub borrow: BTreeMap<u64, usize>,
}

/// 從約束系統提取布局。
pub fn layout_of(sys: &System) -> BitLayout {
    BitLayout {
        nvars: sys.nvars,
        node_type: sys
            .node_type
            .iter()
            .map(|(&k, ts)| (k as u64, ts.to_vec()))
            .collect(),
        arm: sys
            .arm_vars
            .iter()
            .map(|(&k, vs)| (k as u64, vs.clone()))
            .collect(),
        borrow: sys
            .borrow_vars
            .iter()
            .map(|(&k, &v)| (k as u64, v))
            .collect(),
    }
}

/// artifact + 布局 → OracleBits（未指定的位元留 None，由 one-hot 補全處理）。
/// 未知節點或越界索引一律 Err（防錯字靜默通過——認證路徑寧拒勿收）。
pub fn oracle_bits(art: &ChalkArtifact, layout: &BitLayout) -> Result<OracleBits, String> {
    let mut bits: Vec<Option<bool>> = vec![None; layout.nvars];
    let mut one_hot_groups: Vec<Vec<usize>> = Vec::new();
    for (node, chosen) in &art.node_types {
        let vars = layout
            .node_type
            .get(node)
            .ok_or_else(|| format!("未知型別節點 {}（不在布局內）", node))?;
        if *chosen >= vars.len() {
            return Err(format!(
                "節點 {}：候選索引 {} 越界（共 {} 個候選）",
                node,
                chosen,
                vars.len()
            ));
        }
        for (i, &v) in vars.iter().enumerate() {
            if v >= layout.nvars {
                return Err(format!("布局位元 {} 越界（nvars={}）", v, layout.nvars));
            }
            bits[v] = Some(i == *chosen);
        }
        one_hot_groups.push(vars.clone());
    }
    for (invoke, chosen) in &art.arm_choices {
        let vars = layout
            .arm
            .get(invoke)
            .ok_or_else(|| format!("未知臂節點 {}（不在布局內）", invoke))?;
        if *chosen >= vars.len() {
            return Err(format!(
                "臂 {}：索引 {} 越界（共 {} 臂）",
                invoke,
                chosen,
                vars.len()
            ));
        }
        for (i, &v) in vars.iter().enumerate() {
            if v >= layout.nvars {
                return Err(format!("布局位元 {} 越界（nvars={}）", v, layout.nvars));
            }
            bits[v] = Some(i == *chosen);
        }
        one_hot_groups.push(vars.clone());
    }
    for (node, live) in &art.borrows {
        let v = *layout
            .borrow
            .get(node)
            .ok_or_else(|| format!("未知借用節點 {}（不在布局內）", node))?;
        if v >= layout.nvars {
            return Err(format!("布局位元 {} 越界（nvars={}）", v, layout.nvars));
        }
        bits[v] = Some(*live);
    }
    Ok(OracleBits { bits, one_hot_groups })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certify::certify_from_oracle;
    use crate::frac::Frac;
    use crate::groebner::{field_polys, solve_boolean};
    use crate::json::J;
    use crate::minirust::constraints::gen_constraints;
    use crate::minirust::macros::Expander;
    use crate::minirust::parse::Parser;

    const P7: &str = "macro_rules! pick { ($a:expr) => { $a + 1 }; ($a:expr) => { !$a } }\nfn main() { let u = 3; let v = pick!(u); }";
    const P11: &str = "fn main() { let x = 1; let y = x + 1; let x = true; let z = !x; }";
    const P6: &str = "macro_rules! tmp { ($v:ident) => { *(&mut $v) + *(&mut $v) } }\nfn main() { let x = 0; let u = tmp!(x); }";

    /// 由 σ 解碼判決（模擬 rustc/Chalk 的輸出端）。
    fn artifact_from_sigma(sys: &System, sigma: &[Frac], body: &str) -> (ChalkArtifact, String) {
        let mut art = ChalkArtifact { body: body.into(), ..Default::default() };
        for (node, ts) in &sys.node_type {
            let chosen = ts.iter().position(|&v| sigma[v].is_one()).unwrap_or(0);
            art.node_types.push((*node as u64, chosen));
        }
        for (inv, avs) in &sys.arm_vars {
            let chosen = avs.iter().position(|&v| sigma[v].is_one()).unwrap_or(0);
            art.arm_choices.push((*inv as u64, chosen));
        }
        for (node, &v) in &sys.borrow_vars {
            art.borrows.push((*node as u64, sigma[v].is_one()));
        }
        let j = J::obj(vec![
            ("format", J::s("polyrust-oracle/1")),
            ("body", J::s(body)),
            (
                "node_types",
                J::Arr(
                    art.node_types
                        .iter()
                        .map(|(n, c)| J::obj(vec![("node", J::Int(*n as i64)), ("chosen", J::Int(*c as i64))]))
                        .collect(),
                ),
            ),
            (
                "arm_choices",
                J::Arr(
                    art.arm_choices
                        .iter()
                        .map(|(n, c)| J::obj(vec![("invoke", J::Int(*n as i64)), ("chosen", J::Int(*c as i64))]))
                        .collect(),
                ),
            ),
            (
                "borrows",
                J::Arr(
                    art.borrows
                        .iter()
                        .map(|(n, l)| J::obj(vec![("node", J::Int(*n as i64)), ("live", J::Bool(*l))]))
                        .collect(),
                ),
            ),
        ]);
        (art.clone(), j.to_string())
    }

    fn system_and_sigma(src: &str) -> (crate::minirust::constraints::System, Vec<Frac>) {
        let p = Parser::parse_program(src).expect("解析失敗");
        let mut exp = Expander::new(p.macros.clone(), p.next_id);
        let sys = gen_constraints(&p, &mut exp).expect("約束生成失敗");
        let mut merged = sys.polys.clone();
        merged.extend(field_polys(sys.nvars));
        let sigma = solve_boolean(&merged, sys.nvars).expect("可解系統");
        (sys, sigma)
    }

    #[test]
    fn json_parser_basics() {
        let j = parse_json(r#"{"a": [1, -2, true, false, null], "b": {"c": "hi\nthere"}}"#).unwrap();
        assert!(matches!(obj_get(&j, "a").unwrap(), J::Arr(_)));
        match obj_get(&j, "b").unwrap() {
            J::Obj(pairs) => assert_eq!(pairs[0].0, "c"),
            _ => panic!(),
        }
        assert!(parse_json("{\"a\":}").is_err());
        assert!(parse_json("[1,2").is_err());
        assert!(parse_json("{} 額外").is_err());
        assert!(parse_json("nul").is_err());
        // 中文與跳脫
        let j2 = parse_json(r#"{"s": "指定「x」\u0041"}"#).unwrap();
        match obj_get(&j2, "s").unwrap() {
            J::Str(s) => assert!(s.contains("指定") && s.contains('A')),
            _ => panic!(),
        }
    }

    #[test]
    fn artifact_round_trip_certifies_p7() {
        let (sys, sigma) = system_and_sigma(P7);
        assert!(!sys.arm_vars.is_empty(), "P7 必須含宏選臂");
        let (_art, json_text) = artifact_from_sigma(&sys, &sigma, "main");
        let parsed = parse_artifact(&json_text).expect("artifact 解析失敗");
        let layout = layout_of(&sys);
        let oracle = oracle_bits(&parsed, &layout).expect("位元轉換失敗");
        let rep = certify_from_oracle(&sys.polys, sys.nvars, &oracle);
        assert!(
            rep.certified,
            "由 σ 編碼的 artifact 必須通過認證: {:?}",
            (rep.first_bad, rep.oracle_invalid)
        );
    }

    #[test]
    fn artifact_invalid_one_hot_oracle_rejected() {
        // 竄改偵測的 deterministic 形式：同一 one-hot 組內兩個 1 ⇒
        // complete() 直接判 oracle 無效（認證路徑寧拒勿收）。
        // （臂翻轉的語義拒絕由 P11 型別翻轉測試鎖定：無宏樣本全部規則無條件。）
        let (sys, sigma) = system_and_sigma(P7);
        let (art, _) = artifact_from_sigma(&sys, &sigma, "main");
        let layout = layout_of(&sys);
        let mut oracle = oracle_bits(&art, &layout).unwrap();
        let group = layout.arm.values().next().unwrap().clone();
        assert!(group.len() >= 1);
        // 把組內全部位元設 1（若單臂組則找型別組）
        let g = if group.len() >= 2 { group } else { layout.node_type.values().next().unwrap().clone() };
        assert!(g.len() >= 2);
        for &v in &g {
            oracle.bits[v] = Some(true);
        }
        let rep = certify_from_oracle(&sys.polys, sys.nvars, &oracle);
        assert!(!rep.certified, "one-hot 破壞必須被拒絕");
        assert!(rep.oracle_invalid.is_some(), "應標記 oracle 無效: {:?}", rep.oracle_invalid);
    }

    #[test]
    fn artifact_tampered_type_rejected_p11() {
        // P11（shadow，可解、無宏）：全部節點規則都是無條件（ctx=1）耦合，
        // 因此翻轉任一節點的型別選擇必違反某條規則方程 —— deterministic 拒絕。
        let (sys, sigma) = system_and_sigma(P11);
        let (mut art, _) = artifact_from_sigma(&sys, &sigma, "main");
        assert!(!art.node_types.is_empty());
        let (node, c) = art.node_types[0];
        let n = sys.node_type.values().next().unwrap().len();
        art.node_types[0] = (node, (c + 1) % n);
        let layout = layout_of(&sys);
        let oracle = oracle_bits(&art, &layout).unwrap();
        let rep = certify_from_oracle(&sys.polys, sys.nvars, &oracle);
        assert!(!rep.certified, "竄改型別選擇必須被認證拒絕");
        assert!(rep.first_bad.is_some(), "應回報違反的規則方程");
    }

    #[test]
    fn artifact_rejects_unknown_node_and_out_of_range() {
        let (sys, sigma) = system_and_sigma(P7);
        let (mut art, _) = artifact_from_sigma(&sys, &sigma, "main");
        let layout = layout_of(&sys);
        // 未知節點
        art.node_types.push((99_999, 0));
        assert!(oracle_bits(&art, &layout).is_err());
        // 越界索引
        let (mut art2, _) = artifact_from_sigma(&sys, &sigma, "main");
        art2.node_types[0] = (art2.node_types[0].0, 77);
        assert!(oracle_bits(&art2, &layout).is_err());
        // 錯誤格式
        assert!(parse_artifact(r#"{"format":"polyrust-oracle/9"}"#).is_err());
        assert!(parse_artifact("不是 JSON").is_err());
    }

    #[test]
    fn artifact_borrows_optional_and_mapped() {
        // P6（temp-borrow，可解）含 RefMut 借用位元
        let (sys, sigma) = system_and_sigma(P6);
        let (_art, json_text) = artifact_from_sigma(&sys, &sigma, "main");
        let parsed = parse_artifact(&json_text).expect("解析失敗");
        assert!(!parsed.borrows.is_empty(), "P6 應含借用節點");
        let layout = layout_of(&sys);
        let oracle = oracle_bits(&parsed, &layout).expect("轉換失敗");
        let rep = certify_from_oracle(&sys.polys, sys.nvars, &oracle);
        assert!(rep.certified, "含借用位元的閉環必須通過: {:?}", rep.oracle_invalid);
    }
}
