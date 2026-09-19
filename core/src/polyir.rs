// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! PolyIR — Charon LLBC 同 polyrust 下半段（GB／QAP）之間嘅自家 IR（C1 骨架）。
//!
//! 定位（藍圖 §3 分層）：
//! ```text
//!   .rs ──charon──▶ LLBC ──charon_llbc.rs parse──▶ PolyIR ──lowering──▶ 多項式系統
//!        （C2: int/bool 直線+分支；C3: loop/fuel；C4: calls/contracts）
//! ```
//!
//! C1 只定 **結構 lift**：decl 清單 + body 可用性投影，語義 lowering 係 C2 起嘅貨。
//! 呢一層嘅硬承諾（對下游）：
//! - `PolyFun.body_kind == Structured` ⇒　C2 預期可降；`Error` ⇒　判定呈 UNKNOWN(missing_decl)，
//!   口徑同 C0 harness 對齊；`Missing` ⇒ 同 Error（聲明保留、body 缺失）。
//! - 任何 schema 漂移喺 parse 階段已爆（charon_llbc.rs hard error），PolyModule 唔會見到半形狀數據。

use crate::charon_llbc::{BodyKind, FunDeclRef, LlbcRoot};

/// 一個函數單元喺 PolyIR 嘅投影。
#[derive(Debug, Clone)]
pub struct PolyFun {
    pub def_id: i64,
    pub name: String,
    pub path: Vec<String>,
    pub body_kind: BodyKind,
}

#[derive(Debug, Clone)]
pub struct PolyModule {
    pub crate_name: String,
    pub funs: Vec<PolyFun>,
    pub n_types: usize,
    pub n_globals: usize,
    pub n_trait_decls: usize,
    pub n_trait_impls: usize,
    /// Charon 已報告有 decl 抽唔到（= C0 嘅 ok_with_missing）。
    pub has_missing: bool,
}

impl PolyModule {
    /// 可以進入 C2 lowering 嘅 fun（Structured body）。
    pub fn lowerable_funs(&self) -> impl Iterator<Item = &PolyFun> {
        self.funs.iter().filter(|f| f.body_kind == BodyKind::Structured)
    }
    /// 預計要報 UNKNOWN(missing_decl) 嘅 fun 數。
    pub fn missing_body_count(&self) -> usize {
        self.funs.iter().filter(|f| f.body_kind != BodyKind::Structured).count()
    }
}

/// LLBC root → PolyIR 結構 lift（C1：純投影，唔做語義）。
pub fn lift(root: &LlbcRoot) -> PolyModule {
    PolyModule {
        crate_name: root.crate_name.clone(),
        funs: root
            .funs
            .iter()
            .map(|f| PolyFun {
                def_id: f.def_id,
                name: f.name.clone(),
                path: f.full_path.clone(),
                body_kind: f.body_kind,
            })
            .collect(),
        n_types: root.n_type_decls,
        n_globals: root.n_global_decls,
        n_trait_decls: root.n_trait_decls,
        n_trait_impls: root.n_trait_impls,
        has_missing: root.has_errors,
    }
}

// ---------------------------------------------------------------------------
// C2：語義 lowering——LLBC body → 值軌跡 IR（直線算術），其餘形態精確降級
// ---------------------------------------------------------------------------

/// 值槽：(local index, Option<tuple field>)。
/// checked 運算（如 `MulChecked`）喺 LLBC 產生 tuple local；後續以
/// `Field 0`（值）／`Field 1`（溢出旗標）投影訪問——展開成兩個槽。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot {
    pub local: usize,
    pub field: Option<u8>,
}

impl Slot {
    fn whole(local: usize) -> Slot {
        Slot { local, field: None }
    }
    pub fn field(local: usize, k: u8) -> Slot {
        Slot { local, field: Some(k) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PirBinOp {
    Add,
    Sub,
    Mul,
}

impl PirBinOp {
    pub fn name(self) -> &'static str {
        match self {
            PirBinOp::Add => "add",
            PirBinOp::Sub => "sub",
            PirBinOp::Mul => "mul",
        }
    }
}

/// BinOp 運算元：槽（Copy/Move）或常數（Const → 折入恆等式）。
#[derive(Debug, Clone)]
pub enum PirOperand {
    Slot(Slot),
    Const(i64),
}

#[derive(Debug, Clone)]
pub struct PirStmt {
    pub dst: Slot,
    pub kind: PirStmtKind,
}

#[derive(Debug, Clone)]
pub enum PirStmtKind {
    /// dst := src（Use + Copy/Move）
    Copy { src: Slot },
    /// dst := 常數（Use + Const Integer）
    Const { val: i64 },
    /// dst := a op b；`checked = true` 表示 LLBC `*Checked`（tuple 結果：
    /// field0 = 值、field1 = 溢出旗標），dst 槽會被展開成兩個 field 槽。
    BinOp { op: PirBinOp, a: PirOperand, b: PirOperand, checked: bool },
    /// Assert(flag == false)：checked 運算嘅非溢出假設，編成 flag = 0 等式。
    AssertFlagZero { flag: Slot },
}

/// 一個函數嘅直線值軌跡（C2 支援形態）。
#[derive(Debug, Clone)]
pub struct PirBody {
    pub fun_name: String,
    pub arg_count: usize,
    pub stmts: Vec<PirStmt>,
    /// 是否存在 Assert(flag==false)（溢出旗標被斷言為零）。
    pub overflow_asserted: bool,
    /// 返回值槽（LLBC 慣例：local 0 為 return place）。
    pub ret_slot: Option<Slot>,
}

/// 語義 lowering 判決：支援（值軌跡）／精確降級（附原因）／body 缺失。
#[derive(Debug, Clone)]
pub enum PirVerdict {
    ValueTrace(PirBody),
    /// C2 唔支援嘅形態——附精確原因，絕不猜測語義。
    Unknown { reason: String },
    /// body 缺失（Charon `Error`／`Missing` variant）。
    NoBody { kind: BodyKind },
}

/// 由 raw fun_decl JSON 語義 lowering（C2：直線值軌跡；其餘 → Unknown{reason}）。
pub fn lower_fun(fun: &FunDeclRef) -> PirVerdict {
    match lower_fun_inner(fun) {
        Ok(body) => PirVerdict::ValueTrace(body),
        Err(reason) => PirVerdict::Unknown { reason },
    }
}

fn lower_fun_inner(fun: &FunDeclRef) -> Result<PirBody, String> {
    use crate::charon_llbc::Value;
    let name = fun.name.clone();
    let fail = |reason: String| -> Result<PirBody, String> { Err(reason) };

    // body variant
    let body_val = fun
        .raw
        .get("body")
        .ok_or_else(|| format!("{name}: body 缺失"))?;
    let structured = match body_val {
        Value::Obj(pairs) if pairs.len() == 1 => {
            if pairs[0].0 == "Structured" {
                &pairs[0].1
            } else {
                // Error/Missing 同判：聲明在、body 缺
                return fail(format!("{name}: body={}（缺失 body）", pairs[0].0));
            }
        }
        _ => return fail(format!("{name}: body 非單鍵 variant")),
    };

    // locals / arg_count
    let locals_obj = structured
        .get("locals")
        .ok_or_else(|| format!("{name}: locals 缺失"))?;
    let arg_count = locals_obj
        .get("arg_count")
        .and_then(|v| v.as_num())
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| format!("{name}: locals.arg_count 非數字"))?;
    let n_locals = locals_obj
        .get("locals")
        .and_then(|v| v.as_arr())
        .map(|a| a.len())
        .ok_or_else(|| format!("{name}: locals.locals 非陣列"))?;
    if n_locals == 0 || arg_count >= n_locals {
        return fail(format!("{name}: locals 形狀異常 n={n_locals} argc={arg_count}"));
    }

    // statements
    let stmts_val = structured
        .get("body")
        .and_then(|b| b.get("statements"))
        .and_then(|v| v.as_arr())
        .ok_or_else(|| format!("{name}: body.statements 缺失"))?;

    let mut out: Vec<PirStmt> = Vec::with_capacity(stmts_val.len());
    let mut overflow_asserted = false;
    let mut ret_slot: Option<Slot> = None;
    let mut returned = false;

    for (i, s) in stmts_val.iter().enumerate() {
        let kind = s
            .get("kind")
            .ok_or_else(|| format!("{name}: stmt[{i}].kind 缺失"))?;
        if let Some(tag) = kind.as_str() {
            if tag == "Return" {
                returned = true;
                break;
            }
            return fail(format!("{name}: stmt[{i}] 非支援語句 `{tag}`"));
        }
        let (variant, payload) = match kind {
            Value::Obj(pairs) if pairs.len() == 1 => (pairs[0].0.as_str(), &pairs[0].1),
            _ => return fail(format!("{name}: stmt[{i}].kind 非單鍵 variant")),
        };
        match variant {
            "StorageLive" | "StorageDead" | "Borrowck" | "Drop" | "Nop" => {
                // 存儲管理／清理標記：對值軌跡無影響，跳過
            }
            "Assign" => {
                let pair = payload
                    .as_arr()
                    .ok_or_else(|| format!("{name}: stmt[{i}].Assign 非陣列"))?;
                if pair.len() != 2 {
                    return fail(format!("{name}: stmt[{i}].Assign 長度 {}", pair.len()));
                }
                let dst = parse_place(&name, i, &pair[0])?;
                // ret slot 偵測：LLBC local 0 = return place
                if dst.local == 0 && dst.field.is_none() && ret_slot.is_none() {
                    ret_slot = Some(dst.clone());
                }
                let stmt_kind = parse_rvalue(&name, i, &pair[1])?;
                if let PirStmtKind::BinOp { checked: true, .. } = stmt_kind {
                    // checked 運算嘅 dst 係 tuple：展開為 (n,0)/(n,1)，whole 槽不可用
                    if dst.field.is_some() {
                        return fail(format!("{name}: stmt[{i}] checked 運算 dst 非整槽"));
                    }
                }
                out.push(PirStmt { dst, kind: stmt_kind });
            }
            "Assert" => {
                let inner = payload
                    .get("assert")
                    .ok_or_else(|| format!("{name}: stmt[{i}].Assert 缺 assert"))?;
                let cond = inner
                    .get("cond")
                    .ok_or_else(|| format!("{name}: stmt[{i}].Assert 缺 cond"))?;
                let expected = inner
                    .get("expected")
                    .and_then(|v| v.as_bool())
                    .ok_or_else(|| format!("{name}: stmt[{i}].Assert 缺 expected"))?;
                let slot = parse_operand_place(&name, i, cond)?;
                if expected {
                    return fail(format!("{name}: stmt[{i}] Assert(expected=true) 唔支援"));
                }
                if slot.field != Some(1) {
                    return fail(format!(
                        "{name}: stmt[{i}] Assert 條件非溢出旗標（field1）投影"
                    ));
                }
                overflow_asserted = true;
                out.push(PirStmt { dst: slot.clone(), kind: PirStmtKind::AssertFlagZero { flag: slot } });
            }
            "Switch" => return fail(format!("{name}: stmt[{i}] Switch（C3 分支編碼）")),
            "Loop" => return fail(format!("{name}: stmt[{i}] Loop（C3 invariant 路線）")),
            "Call" => return fail(format!("{name}: stmt[{i}] Call（C4 函數契約）")),
            other => return fail(format!("{name}: stmt[{i}] 未知語句 `{other}`")),
        }
    }
    if !returned && ret_slot.is_none() {
        return fail(format!("{name}: body 無 Return 亦無對 local0 賦值"));
    }

    let arg_slots: Vec<Slot> = (1..=arg_count).map(Slot::whole).collect();
    Ok(PirBody { fun_name: name, arg_count, stmts: out, overflow_asserted, ret_slot })
}

/// place → Slot：Local(n) ｜ Projection[Local(n), Field[_,k]]。
fn parse_place(fname: &str, i: usize, v: &crate::charon_llbc::Value) -> Result<Slot, String> {
    use crate::charon_llbc::Value;
    let kind = v
        .get("kind")
        .ok_or_else(|| format!("{fname}: stmt[{i}] place 缺 kind"))?;
    match kind {
        Value::Obj(pairs) if pairs.len() == 1 => match (pairs[0].0.as_str(), &pairs[0].1) {
            ("Local", n) => {
                let n = n
                    .as_num()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Local 索引非數字"))?;
                Ok(Slot::whole(n))
            }
            ("Projection", proj) => {
                let parts = proj
                    .as_arr()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Projection 非陣列"))?;
                if parts.len() != 2 {
                    return Err(format!("{fname}: stmt[{i}] Projection 長度 {}", parts.len()));
                }
                // 基址必須係整 local
                let base = parse_place(fname, i, &parts[0])?;
                if base.field.is_some() {
                    return Err(format!("{fname}: stmt[{i}] 巢狀投影唔支援"));
                }
                match &parts[1] {
                    Value::Obj(p2) if p2.len() == 1 => match (p2[0].0.as_str(), &p2[0].1) {
                        ("Field", f) => {
                            let fa = f
                                .as_arr()
                                .ok_or_else(|| format!("{fname}: stmt[{i}] Field 非陣列"))?;
                            if fa.len() != 2 {
                                return Err(format!("{fname}: stmt[{i}] Field 長度 {}", fa.len()));
                            }
                            let k = fa[1]
                                .as_num()
                                .and_then(|s| s.parse::<u8>().ok())
                                .ok_or_else(|| format!("{fname}: stmt[{i}] Field 索引非數字"))?;
                            Ok(Slot::field(base.local, k))
                        }
                        (other, _) => Err(format!(
                            "{fname}: stmt[{i}] 投影 `{other}` 唔支援（只支援 checked-tuple Field）"
                        )),
                    },
                    _ => Err(format!("{fname}: stmt[{i}] 投影非單鍵 variant")),
                }
            }
            (other, _) => Err(format!("{fname}: stmt[{i}] place `{other}` 唔支援")),
        },
        _ => Err(format!("{fname}: stmt[{i}] place 非物件")),
    }
}

/// operand → PirOperand（Copy/Move → 槽；Const → 常數）。
fn parse_operand(fname: &str, i: usize, v: &crate::charon_llbc::Value) -> Result<PirOperand, String> {
    use crate::charon_llbc::Value;
    match v {
        Value::Obj(pairs) if pairs.len() == 1 => match (pairs[0].0.as_str(), &pairs[0].1) {
            ("Copy" | "Move", place) => Ok(PirOperand::Slot(parse_place(fname, i, place)?)),
            ("Const", cval) => Ok(PirOperand::Const(parse_const(fname, i, cval)?)),
            (other, _) => Err(format!("{fname}: stmt[{i}] operand `{other}` 唔支援")),
        },
        _ => Err(format!("{fname}: stmt[{i}] operand 非單鍵 variant")),
    }
}

/// operand → 其 place 嘅 Slot（Copy/Move）。
fn parse_operand_place(
    fname: &str,
    i: usize,
    v: &crate::charon_llbc::Value,
) -> Result<Slot, String> {
    use crate::charon_llbc::Value;
    match v {
        Value::Obj(pairs) if pairs.len() == 1 => match (pairs[0].0.as_str(), &pairs[0].1) {
            ("Copy" | "Move", place) => parse_place(fname, i, place),
            (other, _) => Err(format!("{fname}: stmt[{i}] operand `{other}` 唔係位置")),
        },
        _ => Err(format!("{fname}: stmt[{i}] operand 非單鍵 variant")),
    }
}

/// operand → PirStmtKind 用（Copy/Const）；BinaryOp 展開。
fn parse_rvalue(
    fname: &str,
    i: usize,
    v: &crate::charon_llbc::Value,
) -> Result<PirStmtKind, String> {
    use crate::charon_llbc::Value;
    match v {
        Value::Obj(pairs) if pairs.len() == 1 => match (pairs[0].0.as_str(), &pairs[0].1) {
            ("Use", payload) => {
                let arr = payload
                    .as_arr()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Use 非陣列"))?;
                if arr.is_empty() {
                    return Err(format!("{fname}: stmt[{i}] Use 空陣列"));
                }
                match &arr[0] {
                    Value::Obj(op) if op.len() == 1 => {
                        match (op[0].0.as_str(), &op[0].1) {
                            ("Copy" | "Move", place) => {
                                let src = parse_place(fname, i, place)?;
                                Ok(PirStmtKind::Copy { src })
                            }
                            ("Const", cval) => {
                                let val = parse_const(fname, i, cval)?;
                                Ok(PirStmtKind::Const { val })
                            }
                            (other, _) => Err(format!("{fname}: stmt[{i}] operand `{other}` 唔支援")),
                        }
                    }
                    _ => Err(format!("{fname}: stmt[{i}] operand 非單鍵 variant")),
                }
            }
            ("BinaryOp", payload) => {
                let arr = payload
                    .as_arr()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] BinaryOp 非陣列"))?;
                if arr.len() != 3 {
                    return Err(format!("{fname}: stmt[{i}] BinaryOp 長度 {}", arr.len()));
                }
                let op_name = arr[0]
                    .as_str()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] BinaryOp 運算符非字串"))?;
                let (op, checked) = match op_name {
                    "Add" => (PirBinOp::Add, false),
                    "Sub" => (PirBinOp::Sub, false),
                    "Mul" => (PirBinOp::Mul, false),
                    "AddChecked" => (PirBinOp::Add, true),
                    "SubChecked" => (PirBinOp::Sub, true),
                    "MulChecked" => (PirBinOp::Mul, true),
                    other => {
                        return Err(format!(
                            "{fname}: stmt[{i}] BinaryOp `{other}` 唔支援（C2 只做加減乘）"
                        ))
                    }
                };
                let a = parse_operand(fname, i, &arr[1])?;
                let b = parse_operand(fname, i, &arr[2])?;
                Ok(PirStmtKind::BinOp { op, a, b, checked })
            }
            (other, _) => Err(format!("{fname}: stmt[{i}] rvalue `{other}` 唔支援")),
        },
        _ => Err(format!("{fname}: stmt[{i}] rvalue 非單鍵 variant")),
    }
}

/// Const 字面值：Integer {Signed|Unsigned}；唔支援則精確報錯。
fn parse_const(fname: &str, i: usize, cval: &crate::charon_llbc::Value) -> Result<i64, String> {
    use crate::charon_llbc::Value;
    // Const = { Value: [id, [literal, ty]] }（或直接 literal，容錯兩種）
    let lit = cval
        .get("Value")
        .and_then(|v| v.as_arr())
        .and_then(|a| a.get(1))
        .and_then(|pair| pair.as_arr())
        .and_then(|a| a.first())
        .unwrap_or(cval);
    match lit {
        Value::Obj(pairs) if pairs.len() == 1 => match (pairs[0].0.as_str(), &pairs[0].1) {
            ("Integer", iv) => {
                // Integer = { Signed: [ty, val] | Unsigned: [ty, val] }
                let (tag, pair) = match iv {
                    Value::Obj(p2) if p2.len() == 1 => (p2[0].0.as_str(), &p2[0].1),
                    _ => return Err(format!("{fname}: stmt[{i}] Integer 非單鍵 variant")),
                };
                let arr = pair
                    .as_arr()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Integer.{tag} 非陣列"))?;
                if arr.len() != 2 {
                    return Err(format!("{fname}: stmt[{i}] Integer.{tag} 長度 {}", arr.len()));
                }
                // 值係字串 token（如 "0"）；容錯 Str/Num 兩種
                let raw = arr[1]
                    .as_str()
                    .or_else(|| arr[1].as_num())
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Integer 值非數字"))?;
                let v: i64 = raw
                    .parse()
                    .map_err(|_| format!("{fname}: stmt[{i}] Integer `{raw}` 非 i64"))?;
                match tag {
                    "Signed" => Ok(v),
                    "Unsigned" => {
                        if v < 0 {
                            Err(format!("{fname}: stmt[{i}] Unsigned 值為負 `{raw}`"))
                        } else {
                            Ok(v)
                        }
                    }
                    other => Err(format!("{fname}: stmt[{i}] Integer `{other}` 唔支援")),
                }
            }
            ("Bool", b) => {
                let b = b
                    .as_bool()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Bool 非布林"))?;
                Ok(if b { 1 } else { 0 })
            }
            (other, _) => Err(format!("{fname}: stmt[{i}] 常數 `{other}` 唔支援（只做整數/布林）")),
        },
        _ => Err(format!("{fname}: stmt[{i}] 常數非單鍵 variant")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charon_llbc::LlbcRoot;

    #[test]
    fn lift_sqr_fixture() {
        let root = LlbcRoot::parse(include_str!("../tests/charon_fixtures/sqr.llbc")).unwrap();
        let m = lift(&root);
        assert_eq!(m.crate_name, "input");
        assert_eq!(m.funs.len(), 1);
        assert!(m.funs[0].path.contains(&"sqr".to_string()));
        assert_eq!(m.lowerable_funs().count(), 1);
        assert_eq!(m.missing_body_count(), 0);
        assert!(!m.has_missing);
    }

    #[test]
    fn lift_async_fixture_marks_missing() {
        let root = LlbcRoot::parse(include_str!("../tests/charon_fixtures/async_simple.llbc")).unwrap();
        let m = lift(&root);
        assert!(m.has_missing);
        assert_eq!(m.missing_body_count(), 1, "async body=Error → 1 missing");
        assert_eq!(m.lowerable_funs().count(), 0);
    }
}
