// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! LLBC Structured body 類型化（C2 lowering 嘅輸入形狀）。
//!
//! 同 `charon_llbc.rs` 同一戒律：**未見過嘅形狀 → hard error**。
//! 白名單係 C0 152 例 spike + 17 fixtures 實測覆蓋嘅全部 C2 相關變體；
//! Charon 升 pin 出新形狀 → 測試即刻爆，唔准半截語義溜過。

use crate::charon_llbc::{LlbcError, Value};
use std::collections::HashMap;

/// Const 值表：`{"Const":{"Value":[id,...]}}` 定點 → `{"Const":{"Deduplicated":id}}` 引用。
/// Charon 首跑法證：while_loop/fact 常量去重，無表解析必死。
pub type ConstTable = HashMap<usize, ConstLit>;

/// 掃 body JSON 收集所有 Const Value 定點（喺 parse 前先過一次）。
pub fn collect_consts(root: &Value, out: &mut ConstTable) {
    match root {
        Value::Obj(fields) => {
            if let Some(vc) = root.get("Const") {
                if let Some(arr) = vc.get("Value").and_then(|x| x.as_arr()) {
                    if arr.len() == 2 {
                        if let Some(id) = arr[0].as_num().and_then(|n| n.parse::<usize>().ok()) {
                            if let Ok(lit) = parse_const_inner(&arr[1], "collect") {
                                out.insert(id, lit);
                            }
                        }
                    }
                }
            }
            for (_, v) in fields {
                collect_consts(v, out);
            }
        }
        Value::Arr(items) => {
            for it in items {
                collect_consts(it, out);
            }
        }
        _ => {}
    }
}

fn err<T>(ctx: &str, msg: &str) -> Result<T, LlbcError> {
    Err(LlbcError { offset: 0, msg: format!("{ctx}: {msg}") })
}

/// local 引用。C2 支援淨 Local + 單 Field 投影（checked-op 嘅 (value, overflow) 二元組
/// 就用 `Projection[.. Field:null? idx]`——實測喺 sqr/pow 嘅 Assert cond 出現）。
/// Deref/Index/其他投影 → hard error（C4）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Place {
    pub local: usize,
    /// `Some(idx)` = `.idx` field 投影（e.g. checked 結果 .1 = overflow flag）。
    pub proj: Option<usize>,
}
impl Place {
    pub fn plain(local: usize) -> Self {
        Place { local, proj: None }
    }
}

/// 函數內型別：bool / 有符 / 無符整數（寬度留檔；域运算用 𝔽_p 近似）。其他型 → 上層當 opaque。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarTy {
    Bool,
    Signed(u32),
    Unsigned(u32),
    Opaque,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Move(Place),
    Copy(Place),
    /// (value, ty) —— 整數字面量（i128 範圍內）或 bool。
    Const(ConstLit),
}

#[derive(Debug, Clone)]
pub enum ConstLit {
    Int(i128, bool /*signed*/, u32 /*bits*/),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum RValue {
    Use(Operand),
    /// (op, lhs, rhs)；Checked 系列會伴隨 Assert Overflow。
    BinaryOp(BinOpName, Operand, Operand),
    UnaryOp(UnOpName, Operand),
    /// enum 判別值（C2 以 fresh 抽象表示）。
    Discriminant(Place),
    /// struct/tuple 初始化等（C4 語義；C2 保留 raw 作 no-op 抽象）。
    AggregateOpaque(Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpName {
    Add, Sub, Mul, AddChecked, SubChecked, MulChecked,
    Div, Rem,
    Eq, Ne, Lt, Le, Gt, Ge,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOpName {
    Neg, Not,
}

#[derive(Debug, Clone)]
pub struct AssertInfo {
    /// cond operand；expected = 期望 cond 取咩值先至唔 panic（Overflow 實測 expected=False 即「唔准 overflow」）。
    pub cond: Operand,
    pub expected: bool,
    pub check: String,
}

#[derive(Debug, Clone)]
pub struct CallInfo {
    /// 同 crate fun_decl id（Regular Fun）。外部調用 → 上層硬錯/UNKNOWN。
    pub fun_id: usize,
    pub args: Vec<Operand>,
    pub dest: Place,
}

/// Switch 分支：分支模式 → block index（wit `SwitchData.fallback` 作不然）。
#[derive(Debug, Clone)]
pub struct SwitchInfo {
    pub scrutinee: Operand,
    /// [(pattern const, block_id)] + fallback block id
    pub arms: Vec<(ConstLit, usize)>,
    pub fallback: usize,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    StorageLive(usize),
    StorageDead(usize),
    Assign(Place, RValue),
    Assert(AssertInfo),
    Call(CallInfo),
    Switch(SwitchInfo),
    Loop(Block),
    BorrowckOpaque(Value),
    /// depth payload（depth-0 目標 block/loop；C2 以桶 fuel 處理）。
    Break(usize),
    Continue(usize),
    Return,
    UnwindResume,
    NopLike(String),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub id: i64,
    pub kind: StmtKind,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: i64,
    pub statements: Vec<Statement>,
}

/// Structured body = locals（arg0 = return slot，其後 1..=arg_count 參數）+ 頂層 Block。
#[derive(Debug, Clone)]
pub struct FunBody {
    pub arg_count: usize,
    pub n_locals: usize,
    pub ret_ty: ScalarTy,
    pub param_tys: Vec<ScalarTy>,
    pub local_tys: Vec<ScalarTy>,
    pub top: Block,
}

// ---------------------------------------------------------------------------
// 解析（Value → typed；硬錯白名單）
// ---------------------------------------------------------------------------

fn parse_place(v: &Value, ctx: &str) -> Result<Place, LlbcError> {
    let kind = v.get("kind")
        .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: place.kind missing") })?;
    if let Some(l) = kind.get("Local").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
        return Ok(Place::plain(l));
    }
    if let Some(Value::Arr(pj)) = kind.get("Projection") {
        // {"Projection":[base_place, [elem...]]}
        let base = pj.first().ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: projection base missing") })?;
        let base = parse_place(base, &format!("{ctx}.proj"))?;
        if base.proj.is_some() {
            return err(ctx, "nested projection（C4）");
        }
        // elems 係單個 elem 物件（{"Field":[_,i]}）或 array
        let elems: Vec<Value> = match pj.get(1) {
            Some(Value::Arr(a)) => a.clone(),
            Some(single @ Value::Obj(_)) => vec![single.clone()],
            _ => Vec::new(),
        };
        if elems.len() == 1 {
            if let Some(idx) = elems[0].get("Field").and_then(|f| f.as_arr()).and_then(|a| a.get(1)).and_then(|n| n.as_num()).and_then(|n| n.parse::<usize>().ok()) {
                return Ok(Place { local: base.local, proj: Some(idx) });
            }
        }
        return err(ctx, &format!("unsupported projection elems（C4）`{}`", kind.dump().chars().take(120).collect::<String>()));
    }
    err(ctx, &format!("unsupported place kind `{}`（C4 先支援）", kind.dump().chars().take(160).collect::<String>()))
}

fn parse_operand(v: &Value, ctx: &str, consts: &ConstTable) -> Result<Operand, LlbcError> {
    if let Some(p) = v.get("Move") {
        return parse_place(p, &format!("{ctx}.Move")).map(Operand::Move);
    }
    if let Some(p) = v.get("Copy") {
        return parse_place(p, &format!("{ctx}.Copy")).map(Operand::Copy);
    }
    if let Some(c) = v.get("Const") {
        return parse_const(c, &format!("{ctx}.Const"), consts).map(Operand::Const);
    }
    err(ctx, &format!("unsupported operand shape `{}`", v.dump().chars().take(160).collect::<String>()))
}

fn parse_const(v: &Value, ctx: &str, consts: &ConstTable) -> Result<ConstLit, LlbcError> {
    // {"Value":[id,[lit, tyref]]} 定點 | {"Deduplicated":id} 引用 | 直接 [lit, tyref]
    if let Some(id) = v.get("Deduplicated").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
        return consts
            .get(&id)
            .cloned()
            .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: const Deduplicated:{id} 未見定點") });
    }
    let core = match v.get("Value").and_then(|x| x.as_arr()) {
        Some(arr) if arr.len() == 2 => arr[1].clone(),
        _ => v.clone(),
    };
    parse_const_inner(&core, ctx)
}

fn parse_const_inner(core: &Value, ctx: &str) -> Result<ConstLit, LlbcError> {
    let arr = core.as_arr().ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: const not array") })?;
    let lit = arr.first().ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: const lit missing") })?;
    if let Some(b) = lit.get("Bool").and_then(|b| b.as_bool()) {
        return Ok(ConstLit::Bool(b));
    }
    if let Some(int) = lit.get("Integer") {
        // {"Integer":{"Signed":["I32","0"]}} / {"Unsigned":["Usize","42"]}
        for (signed, key) in [(true, "Signed"), (false, "Unsigned")] {
            if let Some(pair) = int.get(key).and_then(|p| p.as_arr()) {
                let bits = pair
                    .first()
                    .and_then(|w| w.as_str())
                    .and_then(|w| w.trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u32>().ok())
                    .unwrap_or(64);
                let sval = pair.get(1).and_then(|v| v.as_str()).unwrap_or("0");
                let val: i128 = sval
                    .parse()
                    .map_err(|_| LlbcError { offset: 0, msg: format!("{ctx}: int literal `{sval}` not i128") })?;
                return Ok(ConstLit::Int(val, signed, bits));
            }
        }
    }
    err(ctx, &format!("unsupported const literal `{}`", lit.dump().chars().take(120).collect::<String>()))
}

/// op token 正規化：plain string（"Gt"）或 object variant（{"Rem":"UB"}）。
/// Charon 法證：Rem 帶 UB payload；統一攞 key 名。（payload 檢查級別屬 C4）
fn op_token_name(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Value::Obj(o) = v {
        if o.len() == 1 {
            return Some(o[0].0.clone());
        }
    }
    None
}

fn parse_binop(s: &str) -> Option<BinOpName> {
    use BinOpName::*;
    Some(match s {
        "Add" => Add, "Sub" => Sub, "Mul" => Mul,
        "AddChecked" => AddChecked, "SubChecked" => SubChecked, "MulChecked" => MulChecked,
        "Div" => Div, "Rem" => Rem,
        "Eq" => Eq, "Ne" => Ne, "Lt" => Lt, "Le" => Le, "Gt" => Gt, "Ge" => Ge,
        "BitAnd" => BitAnd, "BitOr" => BitOr, "BitXor" => BitXor,
        "Shl" => Shl, "Shr" => Shr,
        _ => return None,
    })
}

fn parse_rvalue(v: &Value, ctx: &str, consts: &ConstTable) -> Result<RValue, LlbcError> {
    if let Some(u) = v.get("Use") {
        let op = match u {
            Value::Arr(a) => a.first().ok_or(u),
            other => Ok(other),
        }.map_err(|m| LlbcError { offset: 0, msg: format!("{ctx}.Use: malformed {:?}", m) })?;
        return parse_operand(op, &format!("{ctx}.Use"), consts).map(RValue::Use);
    }
    if let Some(Value::Arr(a)) = v.get("BinaryOp") {
        if a.len() == 3 {
            let opn = op_token_name(&a[0]).as_deref().and_then(parse_binop)
                .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.BinaryOp: unknown op `{}`", a[0].dump()) })?;
            let l = parse_operand(&a[1], &format!("{ctx}.BinaryOp.l"), consts)?;
            let r = parse_operand(&a[2], &format!("{ctx}.BinaryOp.r"), consts)?;
            return Ok(RValue::BinaryOp(opn, l, r));
        }
        return err(&format!("{ctx}.BinaryOp"), "expected [op, l, r]");
    }
    if let Some(Value::Arr(a)) = v.get("UnaryOp") {
        if a.len() == 2 {
            let opn = match op_token_name(&a[0]).as_deref() {
                Some("Neg") => UnOpName::Neg,
                Some("Not") => UnOpName::Not,
                other => return err(&format!("{ctx}.UnaryOp"), &format!("unknown op `{other:?}`")),
            };
            return parse_operand(&a[1], &format!("{ctx}.UnaryOp.0"), consts).map(|o| RValue::UnaryOp(opn, o));
        }
    }
    if let Some(d) = v.get("Discriminant") {
        let ph = if let Some(inner) = d.get("Move").or_else(|| d.get("Copy")) { inner } else { d };
        return parse_place(ph, &format!("{ctx}.Discriminant")).map(RValue::Discriminant);
    }
    if let Some(agg) = v.get("Aggregate") {
        return Ok(RValue::AggregateOpaque(agg.clone()));
    }
    err(ctx, &format!("unsupported rvalue `{}`", v.dump().chars().take(160).collect::<String>()))
}

fn unwrap_value_oper(v: &Value, ctx: &str, consts: &ConstTable) -> Result<Operand, LlbcError> {
    // scrutinee: {"Value": operand} 或 直接 operand
    if let Some(inner) = v.get("Value") {
        return parse_operand(inner, ctx, consts);
    }
    parse_operand(v, ctx, consts)
}

fn parse_stmt_kind(v: &Value, ctx: &str, consts: &ConstTable) -> Result<StmtKind, LlbcError> {
    if let Some(s) = v.as_str() {
        return Ok(match s {
            "Return" => StmtKind::Return,
            "UnwindResume" => StmtKind::UnwindResume,
            // 已知 no-op 白名單；其他未知 unit variant → 落下面 hard error（唔畀靜默吞咗）
            "Nop" | "FakeRead" | "Drop" | "UnwindContinue" => StmtKind::NopLike(s.to_string()),
            other => return Err(LlbcError {
                offset: 0,
                msg: format!("{ctx}: unknown unit statement `{other}`（schema 漂移 → 硬錯）"),
            }),
        });
    }
    if let Some(n) = v.get("StorageLive").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
        return Ok(StmtKind::StorageLive(n));
    }
    if let Some(n) = v.get("StorageDead").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
        return Ok(StmtKind::StorageDead(n));
    }
    if let Some(Value::Arr(a)) = v.get("Assign") {
        if a.len() == 2 {
            let p = parse_place(&a[0], &format!("{ctx}.Assign.dst"))?;
            let r = parse_rvalue(&a[1], &format!("{ctx}.Assign.rv"), consts)?;
            return Ok(StmtKind::Assign(p, r));
        }
        return err(&format!("{ctx}.Assign"), "expected [place, rvalue]");
    }
    if let Some(a) = v.get("Assert") {
        let inner = a.get("assert").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Assert: missing assert") })?;
        let cond = unwrap_value_oper(inner.get("cond").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Assert: missing cond") })?, &format!("{ctx}.Assert.cond"), consts)?;
        let expected = inner.get("expected").and_then(|b| b.as_bool()).unwrap_or(true);
        let check = inner
            .get("check_kind")
            .map(|c| c.dump())
            .unwrap_or_else(|| "?".into());
        return Ok(StmtKind::Assert(AssertInfo { cond, expected, check }));
    }
    if let Some(c) = v.get("Call") {
        let call = c.get("call").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Call: missing call") })?;
        let func = call.get("func").and_then(|f| f.get("Regular")).ok_or_else(|| LlbcError {
            offset: 0, msg: format!("{ctx}.Call: non-Regular func（外部/trait）— C4 支援面"),
        })?;
        let fun_id = func
            .get("kind").and_then(|k| k.get("Fun")).and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok())
            .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Call: bad Fun id") })?;
        let args = call
            .get("args").and_then(|a| a.as_arr()).unwrap_or(&[])
            .iter().enumerate()
            .map(|(i, a)| parse_operand(a, &format!("{ctx}.Call.args[{i}]"), consts))
            .collect::<Result<Vec<_>, _>>()?;
        let dest = parse_place(call.get("dest").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Call: dest missing") })?, &format!("{ctx}.Call.dest"))?;
        return Ok(StmtKind::Call(CallInfo { fun_id, args, dest }));
    }
    if let Some(swv) = v.get("Switch") {
        let data = swv.get("data").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Switch: missing data") })?;
        let scrut = unwrap_value_oper(data.get("scrutinee").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Switch: scrutinee missing") })?, &format!("{ctx}.Switch.scrutinee"), consts)?;
        let mut arms = Vec::new();
        for (i, b) in data.get("branches").and_then(|x| x.as_arr()).unwrap_or(&[]).iter().enumerate() {
            let pair = b.as_arr().ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Switch.branches[{i}]: not pair") })?;
            let pat = pair.first().ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Switch.branches[{i}]: pat missing") })?;
            let bid = pair.get(1).and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok())
                .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Switch.branches[{i}]: bad block id") })?;
            arms.push((parse_const(pat, &format!("{ctx}.Switch.branches[{i}].pat"), consts)?, bid));
        }
        let fallback = data.get("fallback").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok())
            .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.Switch: fallback missing") })?;
        let blocks = swv
            .get("branches").and_then(|x| x.as_arr()).unwrap_or(&[])
            .iter().enumerate()
            .map(|(i, b)| parse_block_t(b, &format!("{ctx}.Switch.block[{i}]"), consts))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(StmtKind::Switch(SwitchInfo { scrutinee: scrut, arms, fallback, blocks }));
    }
    if let Some(l) = v.get("Loop") {
        return parse_block_t(l, &format!("{ctx}.Loop"), consts).map(StmtKind::Loop);
    }
    if let Some(bk) = v.get("Borrowck") {
        return Ok(StmtKind::BorrowckOpaque(bk.clone()));
    }
    if let Some(n) = v.get("Break").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
        return Ok(StmtKind::Break(n));
    }
    if let Some(n) = v.get("Continue").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
        return Ok(StmtKind::Continue(n));
    }
    err(ctx, &format!("unknown statement kind `{}`", v.dump().chars().take(160).collect::<String>()))
}

pub fn parse_block(v: &Value, ctx: &str) -> Result<Block, LlbcError> {
    let consts = ConstTable::new();
    parse_block_t(v, ctx, &consts)
}

pub fn parse_block_t(v: &Value, ctx: &str, consts: &ConstTable) -> Result<Block, LlbcError> {
    let id = v.get("id").and_then(|x| x.as_num()).and_then(|n| n.parse::<i64>().ok())
        .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: block id missing") })?;
    let mut statements = Vec::new();
    for (i, s) in v.get("statements").and_then(|x| x.as_arr()).unwrap_or(&[]).iter().enumerate() {
        let sid = s.get("id").and_then(|x| x.as_num()).and_then(|n| n.parse::<i64>().ok()).unwrap_or(i as i64);
        let kind = s.get("kind").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}.statements[{i}]: kind missing") })?;
        statements.push(Statement { id: sid, kind: parse_stmt_kind(kind, &format!("{ctx}.statements[{i}/id{sid}]"), consts)? });
    }
    Ok(Block { id, statements })
}

fn parse_scalar_ty(v: &Value) -> ScalarTy {
    if let Some(sc) = v.get("Scalar") {
        if sc.get("Bool").is_some() {
            return ScalarTy::Bool;
        }
        if let Some(i) = sc.get("Integer") {
            for (signed, key) in [(true, "Signed"), (false, "Unsigned")] {
                if let Some(w) = i.get(key).and_then(|w| w.as_str()) {
                    let bits = w.trim_start_matches(|c: char| !c.is_ascii_digit()).parse::<u32>().unwrap_or(64);
                    return if signed { ScalarTy::Signed(bits) } else { ScalarTy::Unsigned(bits) };
                }
            }
        }
    }
    ScalarTy::Opaque
}

/// 解析 fun body（須已通過 charon_llbc 白名單）。兜手處理 type ref：decode inline scalar；
/// `Deduplicated:n` → 查 type_decls[n].kind。
pub fn parse_fun_body(fun_raw: &Value, type_decls: &[Value]) -> Result<FunBody, LlbcError> {
    let ctx = fun_raw
        .get("item_meta").and_then(|m| m.get("name"))
        .map(|_| "fun").unwrap_or("fun");
    let body = fun_raw.get("body").and_then(|b| b.get("Structured"))
        .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: body not Structured") })?;
    let resolve = |tref: &Value| -> ScalarTy {
        if let Some(n) = tref.get("Deduplicated").and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok()) {
            if let Some(td) = type_decls.get(n) {
                if let Some(k) = td.get("kind") {
                    return parse_scalar_ty(k);
                }
            }
            return ScalarTy::Opaque;
        }
        tref.get("Value").and_then(|x| x.as_arr()).and_then(|a| a.get(1)).map(parse_scalar_ty).unwrap_or(ScalarTy::Opaque)
    };
    let locals_arr = body.get("locals").and_then(|l| l.get("locals")).and_then(|x| x.as_arr())
        .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: locals missing") })?;
    let arg_count = body.get("locals").and_then(|l| l.get("arg_count")).and_then(|x| x.as_num()).and_then(|n| n.parse::<usize>().ok())
        .ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: arg_count missing") })?;
    let mut local_tys = Vec::with_capacity(locals_arr.len());
    for l in locals_arr {
        let tref = l.get("ty").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: local ty missing") })?;
        local_tys.push(resolve(tref));
    }
    // 準確 scalar 訊息取自 signature（inline literal）；locals 表 Deduplicated 內建型 → Opaque（預設唔鎖定）
    let mut ret_ty = local_tys.first().copied().unwrap_or(ScalarTy::Opaque);
    let mut param_tys: Vec<ScalarTy> = local_tys.iter().skip(1).take(arg_count).copied().collect();
    if let Some(sig) = fun_raw.get("signature") {
        if let Some(out) = sig.get("output") {
            let r = resolve(out);
            if r != ScalarTy::Opaque {
                ret_ty = r;
            }
        }
        if let Some(ins) = sig.get("inputs").and_then(|x| x.as_arr()) {
            for (i, tref) in ins.iter().enumerate().take(param_tys.len()) {
                let t = resolve(tref);
                if t != ScalarTy::Opaque {
                    param_tys[i] = t;
                }
            }
        }
    }
    let mut consts = ConstTable::new();
    collect_consts(body, &mut consts);
    let top = parse_block_t(body.get("body").ok_or_else(|| LlbcError { offset: 0, msg: format!("{ctx}: body.body missing") })?, ctx, &consts)?;
    Ok(FunBody {
        arg_count,
        n_locals: locals_arr.len(),
        ret_ty,
        param_tys,
        local_tys,
        top,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charon_llbc::LlbcRoot;

    fn body_of(fixture: &str) -> FunBody {
        let root = LlbcRoot::parse(fixture).unwrap();
        let fun = root.funs.iter().find(|f| f.body_kind == crate::charon_llbc::BodyKind::Structured).unwrap();
        parse_fun_body(
            &fun.raw,
            root.raw_translated.get("type_decls").and_then(|t| t.as_arr()).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn parse_sqr_body() {
        let b = body_of(include_str!("../tests/charon_fixtures/sqr.llbc"));
        assert_eq!(b.arg_count, 1);
        assert!(b.n_locals >= 2);
        assert_eq!(b.param_tys[0], ScalarTy::Signed(32));
        assert!(b.top.statements.iter().any(|s| matches!(s.kind, StmtKind::Assign(_, RValue::BinaryOp(BinOpName::MulChecked, _, _)))));
        assert!(b.top.statements.iter().any(|s| matches!(s.kind, StmtKind::Return)));
        assert!(b.top.statements.iter().any(|s| matches!(s.kind, StmtKind::Assert(_))));
    }

    #[test]
    fn parse_max_switch_and_consts() {
        let b = body_of(include_str!("../tests/charon_fixtures/max.llbc"));
        let sw = b.top.statements.iter().find_map(|s| match &s.kind {
            StmtKind::Switch(si) => Some(si),
            _ => None,
        }).expect("max 必须有 Switch");
        assert_eq!(sw.blocks.len(), 2);
        assert!(!sw.arms.is_empty());
    }

    #[test]
    fn parse_while_loop_kind() {
        let b = body_of(include_str!("../tests/charon_fixtures/while_loop.llbc"));
        assert!(b.top.statements.iter().any(|s| matches!(s.kind, StmtKind::Loop(_))));
    }

    fn stmts_deep<'a>(b: &'a Block, out: &mut Vec<&'a StmtKind>) {
        for s in &b.statements {
            out.push(&s.kind);
            match &s.kind {
                StmtKind::Switch(sw) => sw.blocks.iter().for_each(|b2| stmts_deep(b2, out)),
                StmtKind::Loop(l) => stmts_deep(l, out),
                _ => {}
            }
        }
    }

    #[test]
    fn parse_fact_call_and_switch() {
        let b = body_of(include_str!("../tests/charon_fixtures/fact.llbc"));
        let mut kinds = Vec::new();
        stmts_deep(&b.top, &mut kinds);
        assert!(kinds.iter().any(|s| matches!(s, StmtKind::Call(_))), "fact: no Call anywhere");
        assert!(kinds.iter().any(|s| matches!(s, StmtKind::Switch(_))), "fact: no Switch anywhere");
    }

    #[test]
    fn unknown_stmt_kind_hard_errors() {
        let bad = r#"{"useless":0}"#;
        let v = crate::charon_llbc::parse_json(bad).unwrap();
        assert!(parse_stmt_kind(&v, "t", &ConstTable::new()).is_err());
    }
}
