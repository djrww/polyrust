//! macro_rules! 展開器：token 層匹配、衛生轉錄（模板引入的識別字改名）、
//! 結果重新解析為 AST（新節點 id）。多臂宏的每個臂都是一個「選擇點」，
//! 由約束生成賦予臂位元 a_i（代數）與臂互斥子句（CDCL）。

use crate::minirust::ast::*;
use crate::minirust::lexer::Tok;
use crate::minirust::parse::parse_tokens_as_seq;
use std::collections::HashMap;

pub struct Expander {
    pub macros: Vec<MacroDef>,
    pub next_id: usize,
    hygiene_counter: usize,
    /// (invoke 節點 id, 臂索引) → 展開樹（同一調用在檢查與約束生成間共享，
    /// 保證兩側節點 id 一致——定理 1/2 對照的關鍵）
    pub memo: HashMap<(usize, usize), E>,
    /// 展開文本（報告用）
    pub memo_text: HashMap<(usize, usize), String>,
}

/// 匹配綁定：名字 → (捕獲的 token, 是否表達式捕獲)
type Bindings = HashMap<String, (Vec<Tok>, bool)>;

impl Expander {
    pub fn new(macros: Vec<MacroDef>, next_id: usize) -> Expander {
        Expander { macros, next_id, hygiene_counter: 0, memo: HashMap::new(), memo_text: HashMap::new() }
    }

    pub fn find_macro(&self, name: &str) -> Option<usize> {
        self.macros.iter().position(|m| m.name == name)
    }

    /// 語法匹配的臂索引（Rust 語義按序嘗試；本管線把全部匹配臂交給型別導向選擇）
    pub fn matching_arms(&self, mac: usize, toks: &[Tok]) -> Vec<usize> {
        self.macros[mac]
            .arms
            .iter()
            .enumerate()
            .filter(|(_, arm)| match_patterns(&arm.matcher, toks).is_some())
            .map(|(i, _)| i)
            .collect()
    }

    /// 展開特定臂（memo 化）。
    pub fn expand_arm(
        &mut self,
        invoke_node: usize,
        mac: usize,
        arm: usize,
        toks: &[Tok],
    ) -> Result<E, String> {
        if let Some(e) = self.memo.get(&(invoke_node, arm)) {
            return Ok(e.clone());
        }
        let arm_def = self.macros[mac].arms[arm].clone();
        let bindings = match_patterns(&arm_def.matcher, toks)
            .ok_or_else(|| format!("臂 {} 語法不匹配", arm))?;
        self.hygiene_counter += 1;
        let out_toks = self.transcribe(&arm_def.template, &bindings);
        let text = out_toks.iter().map(|t| t.show()).collect::<Vec<_>>().join(" ");
        let (e, next) = parse_tokens_as_seq(&out_toks, self.next_id)?;
        self.next_id = next;
        self.memo.insert((invoke_node, arm), e.clone());
        self.memo_text.insert((invoke_node, arm), text);
        Ok(e)
    }
}

/// 模式序列匹配：matcher 對 token 流。回傳綁定。
fn match_patterns(matcher: &[MPat], toks: &[Tok]) -> Option<Bindings> {
    let mut binds: Bindings = HashMap::new();
    let mut i = 0usize; // matcher 指標
    let mut j = 0usize; // token 指標
    while i < matcher.len() {
        match &matcher[i] {
            MPat::Tok(t) => {
                if j < toks.len() && &toks[j] == t {
                    j += 1;
                    i += 1;
                } else {
                    return None;
                }
            }
            MPat::Hole(name, MSpec::Ident) => {
                if j < toks.len() {
                    if let Tok::Ident(_) = &toks[j] {
                        binds.insert(name.clone(), (vec![toks[j].clone()], false));
                        j += 1;
                        i += 1;
                        continue;
                    }
                }
                return None;
            }
            MPat::Hole(name, MSpec::Expr) => {
                // 表達式捕獲：吃到「下一個分隔符模式」的頂層出現，或流末尾。
                let sep = matcher.get(i + 1).and_then(|p| match p {
                    MPat::Tok(t) if !matches!(t, Tok::Ident(_) | Tok::Int(_)) => Some(t.clone()),
                    _ => None,
                });
                let start = j;
                let mut depth = 0usize;
                while j < toks.len() {
                    if depth == 0 {
                        if let Some(sep) = &sep {
                            if &toks[j] == sep {
                                break;
                            }
                        }
                    }
                    match &toks[j] {
                        Tok::LParen | Tok::LBrace => depth += 1,
                        Tok::RParen | Tok::RBrace => {
                            if depth == 0 {
                                break; // 不應出現的右括號
                            }
                            depth -= 1;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if j == start {
                    return None; // 空表達式
                }
                binds.insert(name.clone(), (toks[start..j].to_vec(), true));
                i += 1;
            }
            MPat::Group(pats) => {
                // 嵌套括號組
                if j >= toks.len() || toks[j] != Tok::LParen {
                    return None;
                }
                let mut depth = 1usize;
                let mut k = j + 1;
                while k < toks.len() && depth > 0 {
                    match toks[k] {
                        Tok::LParen => depth += 1,
                        Tok::RParen => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 {
                        break;
                    }
                    k += 1;
                }
                if depth != 0 {
                    return None;
                }
                let inner = match_patterns(pats, &toks[j + 1..k])?;
                binds.extend(inner);
                j = k + 1;
                i += 1;
            }
        }
    }
    if j == toks.len() {
        Some(binds)
    } else {
        None
    }
}

impl Expander {
    /// 轉錄模板：$name 代換綁定；模板引入的識別字衛生改名 name#hN。
    fn transcribe(&mut self, template: &[TTmpl], bindings: &Bindings) -> Vec<Tok> {
        let mut renames: HashMap<String, String> = HashMap::new();
        let mut out = vec![];
        self.transcribe_into(template, bindings, &mut renames, &mut out);
        out
    }

    fn transcribe_into(
        &mut self,
        template: &[TTmpl],
        bindings: &Bindings,
        renames: &mut HashMap<String, String>,
        out: &mut Vec<Tok>,
    ) {
        for t in template {
            match t {
                TTmpl::Ref(name) => {
                    if let Some((toks, _)) = bindings.get(name) {
                        // 表達式捕獲以括號保護（宏語義：$e:expr 是不可分的語法單元）
                        let is_expr = bindings.get(name).map_or(false, |(_, e)| *e);
                        if is_expr && toks.len() > 1 {
                            out.push(Tok::LParen);
                            out.extend(toks.iter().cloned());
                            out.push(Tok::RParen);
                        } else {
                            out.extend(toks.iter().cloned());
                        }
                    } else {
                        out.push(Tok::Ident(name.clone())); // 未綁定（不該發生）
                    }
                }
                TTmpl::Tok(t) => match t {
                    Tok::Ident(s) => {
                        let renamed = renames
                            .entry(s.clone())
                            .or_insert_with(|| format!("{}#h{}", s, self.hygiene_counter))
                            .clone();
                        out.push(Tok::Ident(renamed));
                    }
                    other => out.push(other.clone()),
                },
                TTmpl::Group(br, inner) => {
                    out.push(br.open());
                    self.transcribe_into(inner, bindings, renames, out);
                    out.push(br.close());
                }
            }
        }
    }
}

/// 實際使用：macros.rs 文件清單 — 優化 with_capacity
pub fn macros_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("macros.rs", "macros.rs 正式運作 — 優化 with_capacity", "core/src/minirust/macros.rs"),
    ]
}

