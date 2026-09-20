// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! PolyIR — Charon LLBC 同 polyrust 下半段（GB／QAP）之間嘅自家 IR（C1 骨架）。
//!
//! 定位（藍圖 §3 分層）：
//! ```text
//!   .rs ──charon──▶ LLBC ──charon_llbc.rs parse──▶ PolyIR ──lowering──▶ 多項式系統
//!        （C2: int/bool 直線值軌跡；C3: ADT 聚合/判別式/投影；後續: 分支/loop/calls）
//! ```
//!
//! C1 只定 **結構 lift**：decl 清單 + body 可用性投影，語義 lowering 係 C2 起嘅貨。
//! 呢一層嘅硬承諾（對下游）：
//! - `PolyFun.body_kind == Structured` ⇒　C2 預期可降；`Error` ⇒　判定呈 UNKNOWN(missing_decl)，
//!   口徑同 C0 harness 對齊；`Missing` ⇒ 同 Error（聲明保留、body 缺失）。
//! - 任何 schema 漂移喺 parse 階段已爆（charon_llbc.rs hard error），PolyModule 唔會見到半形狀數據。

use crate::charon_llbc::{BodyKind, FunDeclRef, LlbcRoot, Value};
use std::collections::BTreeMap;

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
        self.funs
            .iter()
            .filter(|f| f.body_kind == BodyKind::Structured)
    }
    /// 預計要報 UNKNOWN(missing_decl) 嘅 fun 數。
    pub fn missing_body_count(&self) -> usize {
        self.funs
            .iter()
            .filter(|f| f.body_kind != BodyKind::Structured)
            .count()
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

/// 槽位部件：整槽／tuple-adt 欄位／enum 判別值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Part {
    /// local 本身（int 值、ADT 不透明整槽）
    Whole,
    /// 欄位投影（checked tuple 值/旗標、ADT 欄位）
    Field(u8),
    /// enum 判別值（`Discriminant` 產物／enum 參數之判別值軸）
    Disc,
}

/// 值槽：(local index, part)。
/// checked 運算（如 `MulChecked`）喺 LLBC 產生 tuple local；後續以
/// `Field 0`（值）／`Field 1`（溢出旗標）投影訪問——展開成兩個槽。
/// enum 參數以 `Disc` 槽（判別值）＋ `Field(k)` 槽（欄位）表達。
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot {
    pub local: usize,
    pub part: Part,
}

impl Slot {
    fn whole(local: usize) -> Slot {
        Slot {
            local,
            part: Part::Whole,
        }
    }
    fn disc(local: usize) -> Slot {
        Slot {
            local,
            part: Part::Disc,
        }
    }
    pub fn field(local: usize, k: u8) -> Slot {
        Slot {
            local,
            part: Part::Field(k),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PirBinOp {
    Add,
    Sub,
    Mul,
    /// 比較（C4）：結果 ∈ {0,1}；語義由 concretize 模擬（i64 口徑）承擔，
    /// 編碼側經見證模式記錄為恆等式 dst = 模擬值。比較必須被 Switch 消費。
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
}

impl PirBinOp {
    pub fn name(self) -> &'static str {
        match self {
            PirBinOp::Add => "add",
            PirBinOp::Sub => "sub",
            PirBinOp::Mul => "mul",
            PirBinOp::Gt => "gt",
            PirBinOp::Lt => "lt",
            PirBinOp::Ge => "ge",
            PirBinOp::Le => "le",
            PirBinOp::Eq => "eq",
            PirBinOp::Ne => "ne",
        }
    }
    /// 比較判別（模擬用，i64 口徑與 rustc 一致）。
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            PirBinOp::Gt | PirBinOp::Lt | PirBinOp::Ge | PirBinOp::Le | PirBinOp::Eq | PirBinOp::Ne
        )
    }
    pub fn compare(self, x: i64, y: i64) -> bool {
        match self {
            PirBinOp::Gt => x > y,
            PirBinOp::Lt => x < y,
            PirBinOp::Ge => x >= y,
            PirBinOp::Le => x <= y,
            PirBinOp::Eq => x == y,
            PirBinOp::Ne => x != y,
            _ => unreachable!("compare 只對比較運算"),
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
    BinOp {
        op: PirBinOp,
        a: PirOperand,
        b: PirOperand,
        checked: bool,
    },
    /// Assert(flag == false)：checked 運算嘅非溢出假設，編成 flag = 0 等式。
    AssertFlagZero { flag: Slot },
    /// dst := 判別值(of)（C4 動態形態：of 為 enum 參數）。靜態已建構者
    /// 直接摺成 Const；此 variant 只喺見證模式編碼（恆等式 dst = 模擬值）。
    Discriminant { of: Slot },
    /// dst := callee(args…)（C5：本地 crate 純函數調用）。語義由
    /// concretize 模擬（callee body 遞迴模擬，深限）承擔；編碼側見證模式
    /// 恆等化 dst = 模擬 ret。`callee` = 函數名（def_id 對位後之最尾段）。
    Call {
        callee: String,
        args: Vec<PirOperand>,
    },
    /// dst := ADT 聚合建構（C3：struct／tuple／enum variant）。欄位逐一寫入
    /// `dst.local` 嘅 field 槽（域 = 運算元域，恆等式 = dst_f = src）；
    /// `disc = Some(c)` 表示 enum variant 建構，判別值 c 已由 type_decls 解析，
    /// 供後續 `Discriminant` rvalue 常數傳播。dst 整槽 = 不透明（ADT 值唔直接入算術）。
    Aggregate {
        fields: Vec<PirOperand>,
        disc: Option<i64>,
    },
}

/// enum 參數資訊（C4）：由 signature.inputs 嘅 ty（Adt.id）＋ type_decls 解析。
#[derive(Debug, Clone)]
pub struct EnumParam {
    /// 參數序號（0-based；對應 local = arg + 1）
    pub arg: usize,
    pub local: usize,
    /// 各 variant：(判別值, 欄位數)。C4 只支援 ≤1 欄位之 variant。
    pub variants: Vec<(i64, u8)>,
}

/// 一條執行路徑（C4）：guard = (scrutinee, value) 等值條件；fallback =
/// scrutinee 值唔中任何 branch 常數時行呢條。純直線 body 只有一條
/// guard = None 嘅 path（同 C2/C3 stmts 等價）。
#[derive(Debug, Clone)]
pub struct PirPath {
    pub scrutinee: Option<Slot>,
    pub value: Option<i64>,
    pub fallback: bool,
    pub stmts: Vec<PirStmt>,
}

/// 一個函數嘅值軌跡（C4：多路徑）。
#[derive(Debug, Clone)]
pub struct PirBody {
    pub fun_name: String,
    pub arg_count: usize,
    pub paths: Vec<PirPath>,
    /// 是否存在 Assert(flag==false)（溢出旗標被斷言為零）。
    pub overflow_asserted: bool,
    /// 返回值槽（LLBC 慣例：local 0 為 return place）。
    pub ret_slot: Option<Slot>,
    /// enum 參數（判別值/欄位軸；C4 組合枚舉用）。
    pub enum_params: Vec<EnumParam>,
}

/// 語義 lowering 判決：支援（值軌跡）／精確降級（附原因）／body 缺失。
#[derive(Debug, Clone)]
pub enum PirVerdict {
    ValueTrace(PirBody),
    /// C2 唔支援嘅形態——附精確原因，絕不猜測語義。
    Unknown {
        reason: String,
    },
    /// body 缺失（Charon `Error`／`Missing` variant）。
    NoBody {
        kind: BodyKind,
    },
}

/// 由 raw fun_decl JSON 語義 lowering（直線值軌跡；其餘 → Unknown{reason}）。
/// 唔帶 type_decls：enum Aggregate 嘅判別值解析會如實降級。
pub fn lower_fun(fun: &FunDeclRef) -> PirVerdict {
    lower_fun_in(&Value::Null, &[], fun)
}

/// 同上，另帶 root 嘅 type_decls（C3：enum Aggregate 判別值解析需要）。
pub fn lower_fun_in(types: &Value, funs: &[FunDeclRef], fun: &FunDeclRef) -> PirVerdict {
    match lower_fun_inner(types, funs, fun) {
        Ok(body) => PirVerdict::ValueTrace(body),
        Err(reason) => PirVerdict::Unknown { reason },
    }
}

fn lower_fun_inner(
    types: &Value,
    funs: &[FunDeclRef],
    fun: &FunDeclRef,
) -> Result<PirBody, String> {
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
        // C5 實證：opaque／內建函數 body = "Opaque" 等純字串 variant
        Value::Str(tag) => {
            return fail(format!("{name}: body={tag}（缺失 body，唔可模擬）"))
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
        return fail(format!(
            "{name}: locals 形狀異常 n={n_locals} argc={arg_count}"
        ));
    }

    // C4：enum 參數解析（signature.inputs ty = Adt → type_decls 查 Enum）
    let enum_params = parse_enum_params(types, fun, arg_count, &name)?;

    // statements
    let stmts_val = structured
        .get("body")
        .and_then(|b| b.get("statements"))
        .and_then(|v| v.as_arr())
        .ok_or_else(|| format!("{name}: body.statements 缺失"))?;

    let mut ctx = LowerCtx {
        name: name.clone(),
        disc_of: BTreeMap::new(),
        // enum param 之判別值軸：local → variants
        disc_domain: enum_params
            .iter()
            .map(|e| (e.local, e.variants.iter().map(|v| v.0).collect::<Vec<_>>()))
            .collect(),
        disc_dst_of: BTreeMap::new(),
        // C5：本地可調用對位表（def_id → 名；只收 Structured body——
        // Opaque/Builtin fun 唔可模擬）
        callee_names: funs
            .iter()
            .filter(|f| f.body_kind == BodyKind::Structured)
            .map(|f| (f.def_id, f.name.clone()))
            .collect(),
        cmp_dst: std::collections::BTreeSet::new(),
        overflow_asserted: false,
        ret_slot: None,
    };
    let mut paths: Vec<PirPath> = Vec::new();
    let mut probe = Vec::new();
    let end = lower_seq(&mut ctx, types, stmts_val, 0, &mut probe, &mut paths, true)?;
    let _ = end;
    if paths.is_empty() {
        // 純直線（無動態 switch）：主流程單 path
        let mut stmts = Vec::new();
        let end2 = lower_seq(&mut ctx, types, stmts_val, 0, &mut stmts, &mut paths, false)?;
        let _ = end2;
        paths.insert(
            0,
            PirPath {
                scrutinee: None,
                value: None,
                fallback: false,
                stmts,
            },
        );
    }
    if ctx.ret_slot.is_none() {
        return fail(format!("{name}: body 無 Return 亦無對 local0 賦值"));
    }

    Ok(PirBody {
        fun_name: name,
        arg_count,
        paths,
        overflow_asserted: ctx.overflow_asserted,
        ret_slot: ctx.ret_slot,
        enum_params,
    })
}

/// lowering 上下文（跨語句狀態）。
struct LowerCtx {
    name: String,
    /// 已知判別值（local → 值）：靜態已建構 enum
    disc_of: BTreeMap<usize, i64>,
    /// enum 參數判別值軸（local → 各 variant 判別值）
    disc_domain: BTreeMap<usize, Vec<i64>>,
    /// Discriminant(enum param) 產物（dst local → enum param local）
    disc_dst_of: BTreeMap<usize, usize>,
    /// C5：本地可調用對位表（def_id → 名；Structured body only）
    callee_names: BTreeMap<i64, String>,
    /// 比較結果槽（Switch scrutinee 域 = {0,1} 之依據）
    cmp_dst: std::collections::BTreeSet<Slot>,
    overflow_asserted: bool,
    ret_slot: Option<Slot>,
}

/// 序列 lowering 結局：Return 終止／FallThrough 流盡。
#[derive(PartialEq)]
enum SeqEnd {
    Returned,
    FallThrough,
}

/// 語句序列 lowering（C4：遇動態 Switch 進行路徑分裂，paths 收各臂完整軌跡）。
/// `split` = true 時允許路徑分裂（頂層調用）；false = 純直線（分裂 → 降級，
/// 供單 path 二次 lowering）。
fn lower_seq(
    ctx: &mut LowerCtx,
    types: &Value,
    stmts_val: &[Value],
    start: usize,
    out: &mut Vec<PirStmt>,
    paths: &mut Vec<PirPath>,
    split: bool,
) -> Result<SeqEnd, String> {
    let name = ctx.name.clone();
    let fail = |reason: String| -> Result<SeqEnd, String> { Err(reason) };
    let mut i = start;
    while i < stmts_val.len() {
        let s = &stmts_val[i];
        let kind = s
            .get("kind")
            .ok_or_else(|| format!("{name}: stmt[{i}].kind 缺失"))?;
        if let Some(tag) = kind.as_str() {
            if tag == "Return" {
                return Ok(SeqEnd::Returned);
            }
            return fail(format!("{name}: stmt[{i}] 非支援語句 `{tag}`"));
        }
        let (variant, payload) = match kind {
            Value::Obj(pairs) if pairs.len() == 1 => (pairs[0].0.as_str(), &pairs[0].1),
            _ => return fail(format!("{name}: stmt[{i}].kind 非單鍵 variant")),
        };
        match variant {
            "StorageLive" | "StorageDead" | "Borrowck" | "Drop" | "Nop" => {}
            "Abort" => {
                // unreachable 臂（exhaustive match fallback）：唔會有組合到達；
                // 若 lowering 到此 = 組合軸未覆蓋，唔生成語句（呼叫方已按值分派）
            }
            "Assign" => {
                let pair = payload
                    .as_arr()
                    .ok_or_else(|| format!("{name}: stmt[{i}].Assign 非陣列"))?;
                if pair.len() != 2 {
                    return fail(format!("{name}: stmt[{i}].Assign 長度 {}", pair.len()));
                }
                let dst = parse_place(&name, i, &pair[0])?;
                if dst.local == 0 && dst.part == Part::Whole && ctx.ret_slot.is_none() {
                    ctx.ret_slot = Some(dst.clone());
                }
                let stmt_kind =
                    parse_rvalue(types, &ctx.disc_of, &ctx.disc_domain, &name, i, &pair[1])?;
                if let PirStmtKind::BinOp { checked: true, .. } = stmt_kind {
                    if dst.part != Part::Whole {
                        return fail(format!("{name}: stmt[{i}] checked 運算 dst 非整槽"));
                    }
                }
                match &stmt_kind {
                    PirStmtKind::Aggregate { disc, .. } => {
                        if dst.part != Part::Whole {
                            return fail(format!("{name}: stmt[{i}] Aggregate dst 非整槽"));
                        }
                        if let Some(c) = disc {
                            ctx.disc_of.insert(dst.local, *c);
                        }
                    }
                    PirStmtKind::Copy { src } => {
                        if src.part == Part::Whole && dst.part == Part::Whole {
                            if let Some(&c) = ctx.disc_of.get(&src.local) {
                                ctx.disc_of.insert(dst.local, c);
                            }
                        }
                    }
                    PirStmtKind::BinOp { op, .. } => {
                        if op.is_comparison() {
                            if dst.part != Part::Whole {
                                return fail(format!("{name}: stmt[{i}] 比較 dst 非整槽"));
                            }
                            ctx.cmp_dst.insert(dst.clone());
                        }
                    }
                    PirStmtKind::Discriminant { of } => {
                        if dst.part != Part::Whole {
                            return fail(format!("{name}: stmt[{i}] Discriminant dst 非整槽"));
                        }
                        // 靜態已建構：摺 Const；enum 參數：記判別值軸產物
                        if let Some(&c) = ctx.disc_of.get(&of.local) {
                            out.push(PirStmt {
                                dst: dst.clone(),
                                kind: PirStmtKind::Const { val: c },
                            });
                            i += 1;
                            continue;
                        }
                        if ctx.disc_domain.contains_key(&of.local) {
                            ctx.disc_dst_of.insert(dst.local, of.local);
                        }
                        // 其餘（未知 enum）→ 照推，由 Switch 域偵測/encode 降級
                    }
                    _ => {}
                }
                out.push(PirStmt {
                    dst,
                    kind: stmt_kind,
                });
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
                if slot.part != Part::Field(1) {
                    return fail(format!(
                        "{name}: stmt[{i}] Assert 條件非溢出旗標（field1）投影"
                    ));
                }
                ctx.overflow_asserted = true;
                out.push(PirStmt {
                    dst: slot.clone(),
                    kind: PirStmtKind::AssertFlagZero { flag: slot },
                });
            }
            "Switch" => {
                // ---- 路徑分裂（C4 核心）----
                let data = payload
                    .get("data")
                    .ok_or_else(|| format!("{name}: stmt[{i}] Switch 缺 data"))?;
                let scrutinee = parse_operand_place(
                    &name,
                    i,
                    data.get("scrutinee")
                        .ok_or_else(|| format!("{name}: stmt[{i}] Switch 缺 scrutinee"))?,
                )?;
                if scrutinee.part != Part::Whole {
                    return fail(format!("{name}: stmt[{i}] Switch scrutinee 非整槽"));
                }
                let branches = data
                    .get("branches")
                    .and_then(|v| v.as_arr())
                    .ok_or_else(|| format!("{name}: stmt[{i}] Switch branches 缺失"))?;
                let fallback_idx = data
                    .get("fallback")
                    .and_then(|v| v.as_num())
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(|| format!("{name}: stmt[{i}] Switch fallback 非數字"))?;
                let mut branch_arms: Vec<(i64, usize)> = Vec::new();
                for b in branches {
                    let ba = b
                        .as_arr()
                        .ok_or_else(|| format!("{name}: stmt[{i}] branch 非陣列"))?;
                    if ba.len() != 2 {
                        return fail(format!("{name}: stmt[{i}] branch 長度 {}", ba.len()));
                    }
                    let c = parse_const(&name, i, &ba[0])?;
                    let idx = ba[1]
                        .as_num()
                        .and_then(|s| s.parse::<usize>().ok())
                        .ok_or_else(|| format!("{name}: stmt[{i}] branch 臂索引非數字"))?;
                    branch_arms.push((c, idx));
                }
                let arms = payload
                    .get("branches")
                    .and_then(|v| v.as_arr())
                    .ok_or_else(|| format!("{name}: stmt[{i}] Switch 臂陣列缺失"))?;

                // scrutinee 域可知性：靜態判別值 > enum 參數軸 > 比較 {0,1}
                let domain: Vec<i64> = if let Some(&c) = ctx.disc_of.get(&scrutinee.local) {
                    vec![c]
                } else if let Some(&param) = ctx.disc_dst_of.get(&scrutinee.local) {
                    ctx.disc_domain
                        .get(&param)
                        .cloned()
                        .ok_or_else(|| format!("{name}: stmt[{i}] 判別值軸缺失"))?
                } else if ctx.cmp_dst.contains(&scrutinee) {
                    vec![0, 1]
                } else {
                    return fail(format!(
                        "{name}: stmt[{i}] Switch scrutinee 域未知（判別值域/比較結果以外屬後續 slice）"
                    ));
                };

                if domain.len() == 1 && ctx.disc_of.contains_key(&scrutinee.local) {
                    // 靜態選臂：內聯該臂，續主流程
                    let c = domain[0];
                    let arm_idx = branch_arms
                        .iter()
                        .find(|(v, _)| *v == c)
                        .map(|(_, idx)| *idx)
                        .unwrap_or(fallback_idx);
                    let arm_stmts = arms
                        .get(arm_idx)
                        .and_then(|a| a.get("statements"))
                        .and_then(|v| v.as_arr())
                        .ok_or_else(|| format!("{name}: stmt[{i}] 臂 {arm_idx} 缺失"))?;
                    let end = lower_seq(ctx, types, arm_stmts, 0, out, paths, false)?;
                    if end == SeqEnd::FallThrough {
                        i += 1;
                        continue;
                    }
                    return Ok(SeqEnd::Returned);
                }

                if !split {
                    return fail(format!(
                        "{name}: stmt[{i}] 巢狀動態 Switch 唔支援（C4 只支援單層）"
                    ));
                }

                // 動態：每個可能值一條 path（fallback 值集 = 域 − branch 常數）
                let covered: std::collections::BTreeSet<i64> =
                    branch_arms.iter().map(|(v, _)| *v).collect();
                let mut values_with_arm: Vec<(i64, usize)> = domain
                    .iter()
                    .map(|&v| match branch_arms.iter().find(|(bv, _)| *bv == v) {
                        Some((_, idx)) => (v, *idx),
                        None => (v, fallback_idx),
                    })
                    .collect();
                let fallback_covered = domain.iter().any(|v| !covered.contains(v));
                if !fallback_covered {
                    // 域全覆蓋：fallback 臂 unreachable（abort）——不生成 path
                    values_with_arm.retain(|(_, idx)| *idx != fallback_idx);
                }
                for (v, arm_idx) in values_with_arm {
                    let is_fallback = !covered.contains(&v);
                    let arm_stmts = arms
                        .get(arm_idx)
                        .and_then(|a| a.get("statements"))
                        .and_then(|v| v.as_arr())
                        .ok_or_else(|| format!("{name}: stmt[{i}] 臂 {arm_idx} 缺失"))?;
                    // 每 path 之判別值狀態由 switch 前狀態分叉（互不污染）
                    let base_disc = ctx.disc_of.clone();
                    // guard 值本身係一個已知判別值（scrutinee 為 enum param 產物時）
                    if ctx.disc_dst_of.contains_key(&scrutinee.local) {
                        ctx.disc_of.insert(scrutinee.local, v);
                    }
                    let mut path_stmts = out.clone();
                    let mut sub_paths = Vec::new();
                    let end = lower_seq(
                        ctx,
                        types,
                        arm_stmts,
                        0,
                        &mut path_stmts,
                        &mut sub_paths,
                        false,
                    )?;
                    if end == SeqEnd::FallThrough {
                        // 續 post-switch 主流程
                        let end2 = lower_seq(
                            ctx,
                            types,
                            stmts_val,
                            i + 1,
                            &mut path_stmts,
                            &mut sub_paths,
                            false,
                        )?;
                        let _ = end2;
                    }
                    ctx.disc_of = base_disc;
                    paths.push(PirPath {
                        scrutinee: Some(scrutinee.clone()),
                        value: Some(v),
                        fallback: is_fallback,
                        stmts: path_stmts,
                    });
                }
                // 主流程到此終止（各 path 已含 post）
                return Ok(SeqEnd::Returned);
            }
            "Call" => {
                // C5：本地純函數調用。實證形態（charon 0.1.265）：
                // {"Call": {"call": {"func": {"Regular": {"kind": {"Fun": id}, …}},
                //           "args": [operand…], "dest": place}, "on_unwind": …}}
                let call = payload
                    .get("call")
                    .ok_or_else(|| format!("{name}: stmt[{i}] Call 缺 call"))?;
                let func = call
                    .get("func")
                    .ok_or_else(|| format!("{name}: stmt[{i}] Call 缺 func"))?;
                let fun_id = func
                    .get("Regular")
                    .and_then(|r| r.get("kind"))
                    .and_then(|k| k.get("Fun"))
                    .and_then(|v| v.as_num())
                    .and_then(|s| s.parse::<i64>().ok());
                let Some(fun_id) = fun_id else {
                    let kind_txt = func
                        .get("Regular")
                        .and_then(|r| r.get("kind"))
                        .map(|k| {
                            k.as_str().map(|s| s.to_string()).unwrap_or_else(|| {
                                if k.get("Builtin").is_some() {
                                    "Builtin".to_string()
                                } else {
                                    "未知".to_string()
                                }
                            })
                        })
                        .unwrap_or_else(|| "非 Regular".to_string());
                    return fail(format!(
                        "{name}: stmt[{i}] Call callee `{kind_txt}` 唔支援（只做本地函數）"
                    ));
                };
                let callee = ctx.callee_names.get(&fun_id).cloned().ok_or_else(|| {
                    format!(
                        "{name}: stmt[{i}] Call def_id={fun_id} 唔係本地可模擬函數（Opaque／缺失 body）"
                    )
                })?;
                if callee == name {
                    return fail(format!(
                        "{name}: stmt[{i}] 遞迴呼叫唔支援（不動點屬後續 slice）"
                    ));
                }
                let arg_vals = call
                    .get("args")
                    .and_then(|v| v.as_arr())
                    .ok_or_else(|| format!("{name}: stmt[{i}] Call args 缺失"))?;
                let mut args = Vec::with_capacity(arg_vals.len());
                for a in arg_vals {
                    args.push(parse_operand(&name, i, a)?);
                }
                let dst = parse_place(
                    &name,
                    i,
                    call.get("dest")
                        .ok_or_else(|| format!("{name}: stmt[{i}] Call 缺 dest"))?,
                )?;
                if dst.part != Part::Whole {
                    return fail(format!("{name}: stmt[{i}] Call dest 非整槽"));
                }
                if dst.local == 0 && ctx.ret_slot.is_none() {
                    ctx.ret_slot = Some(dst.clone());
                }
                out.push(PirStmt {
                    dst,
                    kind: PirStmtKind::Call { callee, args },
                });
            }
            "Loop" => {
                return fail(format!(
                    "{name}: stmt[{i}] Loop 唔支援（invariant 路線屬後續 slice）"
                ))
            }
            other => return fail(format!("{name}: stmt[{i}] 未知語句 `{other}`")),
        }
        i += 1;
    }
    Ok(SeqEnd::FallThrough)
}

/// C4：enum 參數解析（signature.inputs[i].ty = Adt → type_decls Enum）。
/// 唔支援：>1 欄位之 variant（組合軸語法未定義）。
fn parse_enum_params(
    types: &Value,
    fun: &FunDeclRef,
    arg_count: usize,
    name: &str,
) -> Result<Vec<EnumParam>, String> {
    let mut out = Vec::new();
    let inputs = fun
        .raw
        .get("signature")
        .and_then(|s| s.get("inputs"))
        .and_then(|v| v.as_arr())
        .ok_or_else(|| format!("{name}: signature.inputs 缺失"))?;
    for (idx, inp) in inputs.iter().enumerate() {
        if idx >= arg_count {
            break;
        }
        let ty_val = inp
            .get("Value")
            .and_then(|v| v.as_arr())
            .and_then(|a| a.get(1))
            .or_else(|| inp.get("ty"));
        // ty 可能多層 {"Value": [_, ty]} 包裝（實測 Option param 直接 {"Adt":…}）——
        // 迴圈拆到非 Arr 為止
        let mut tv = ty_val;
        while let Some(Value::Arr(a)) = tv {
            tv = a.get(1);
        }
        let adt_id = tv
            .and_then(|t| t.get("Adt"))
            .and_then(|a| a.get("id"))
            .and_then(|v| v.as_num())
            .and_then(|s| s.parse::<i64>().ok());
        let Some(adt_id) = adt_id else { continue };
        let tarr = types
            .as_arr()
            .ok_or_else(|| format!("{name}: type_decls 缺失（enum 參數解析需要）"))?;
        let td = tarr
            .iter()
            .find(|t| {
                t.get("def_id")
                    .and_then(|v| v.as_num())
                    .and_then(|s| s.parse::<i64>().ok())
                    == Some(adt_id)
            })
            .ok_or_else(|| format!("{name}: type_decls 無 def_id={adt_id}"))?;
        let variants = td
            .get("kind")
            .and_then(|k| k.get("Enum"))
            .and_then(|e| e.as_arr())
            .ok_or_else(|| format!("{name}: type #{adt_id} 非 Enum"))?;
        let mut vs = Vec::new();
        for v in variants {
            let disc = v
                .get("discriminant")
                .ok_or_else(|| format!("{name}: variant 缺 discriminant"))?;
            let (tag, pair) = match disc {
                Value::Obj(pp) if pp.len() == 1 => (pp[0].0.as_str(), &pp[0].1),
                _ => return Err(format!("{name}: discriminant 非單鍵 variant")),
            };
            let arr = pair
                .as_arr()
                .ok_or_else(|| format!("{name}: discriminant.{tag} 非陣列"))?;
            let raw = arr
                .get(1)
                .and_then(|x| x.as_str().or_else(|| x.as_num()))
                .ok_or_else(|| format!("{name}: discriminant 值非數字"))?;
            let val: i64 = raw
                .parse()
                .map_err(|_| format!("{name}: discriminant `{raw}` 非 i64"))?;
            let nfields = v
                .get("fields")
                .and_then(|f| f.as_arr())
                .map(|a| a.len())
                .unwrap_or(0);
            vs.push((val, nfields as u8));
        }
        if vs.iter().any(|(_, nf)| *nf > 1) {
            return Err(format!(
                "{name}: 參數 {idx} 係多欄位 enum variant（C4 只支援 ≤1 欄位；組合軸屬後續 slice）"
            ));
        }
        out.push(EnumParam {
            arg: idx,
            local: idx + 1,
            variants: vs,
        });
    }
    Ok(out)
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
                    return Err(format!(
                        "{fname}: stmt[{i}] Projection 長度 {}",
                        parts.len()
                    ));
                }
                // 基址必須係整 local
                let base = parse_place(fname, i, &parts[0])?;
                if base.part != Part::Whole {
                    return Err(format!("{fname}: stmt[{i}] 巢狀投影唔支援"));
                }
                match &parts[1] {
                    Value::Str(s) if s == "Deref" => {
                        // C3：Deref 透明——值軌跡直線世界下 ref = alias，
                        // 讀寫經 deref 都映射到被指者同一槽。多別名借用衝突
                        // 偵測屬 M3 代數 borrowck。
                        Ok(base)
                    }
                    Value::Obj(p2) if p2.len() == 1 => match (p2[0].0.as_str(), &p2[0].1) {
                        ("Field", f) => {
                            // Field = [variant_id | null, field_idx]：variant 限定符
                            // 唔影響槽位（同一 local 嘅 field 空間共享；執行時只有
                            // 一個 variant 活躍），只取 field_idx。
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
                            "{fname}: stmt[{i}] 投影 `{other}` 唔支援（支援 Field／Deref）"
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
fn parse_operand(
    fname: &str,
    i: usize,
    v: &crate::charon_llbc::Value,
) -> Result<PirOperand, String> {
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
            // {"Value": {...}} 包裝（Switch scrutinee 慣例）：拆開再認
            ("Value", inner) => parse_operand_place(fname, i, inner),
            (other, _) => Err(format!("{fname}: stmt[{i}] operand `{other}` 唔係位置")),
        },
        _ => Err(format!("{fname}: stmt[{i}] operand 非單鍵 variant")),
    }
}

/// rvalue → PirStmtKind。C4：Discriminant 認 enum 參數（動態判別值語句）。
fn parse_rvalue(
    types: &Value,
    disc_of: &BTreeMap<usize, i64>,
    disc_domain: &BTreeMap<usize, Vec<i64>>,
    fname: &str,
    i: usize,
    v: &Value,
) -> Result<PirStmtKind, String> {
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
                    Value::Obj(op) if op.len() == 1 => match (op[0].0.as_str(), &op[0].1) {
                        ("Copy" | "Move", place) => {
                            let src = parse_place(fname, i, place)?;
                            Ok(PirStmtKind::Copy { src })
                        }
                        ("Const", cval) => {
                            let val = parse_const(fname, i, cval)?;
                            Ok(PirStmtKind::Const { val })
                        }
                        (other, _) => Err(format!("{fname}: stmt[{i}] operand `{other}` 唔支援")),
                    },
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
                    // C4：比較（結果 {0,1}；語義由 concretize 模擬承擔）
                    "Gt" => (PirBinOp::Gt, false),
                    "Lt" => (PirBinOp::Lt, false),
                    "Ge" => (PirBinOp::Ge, false),
                    "Le" => (PirBinOp::Le, false),
                    "Eq" => (PirBinOp::Eq, false),
                    "Ne" => (PirBinOp::Ne, false),
                    other => {
                        return Err(format!(
                            "{fname}: stmt[{i}] BinaryOp `{other}` 唔支援（加減乘/checked/比較）"
                        ))
                    }
                };
                let a = parse_operand(fname, i, &arr[1])?;
                let b = parse_operand(fname, i, &arr[2])?;
                Ok(PirStmtKind::BinOp { op, a, b, checked })
            }
            ("Discriminant", place) => {
                // dst := 判別值(of)。C3 支援常數傳播：of 係已建構 enum（Aggregate
                // 建構或其整槽 copy）→ 直接摺成 Const；輸入 enum 嘅判別式域推導
                // 要配合分支編碼（後續 slice）一齊做，而家如實降級。
                let of = parse_place(fname, i, place)?;
                if of.part != Part::Whole {
                    return Err(format!("{fname}: stmt[{i}] Discriminant of 非整槽"));
                }
                if disc_of.get(&of.local).is_some() {
                    return Ok(PirStmtKind::Const {
                        val: disc_of[&of.local],
                    });
                }
                if disc_domain.contains_key(&of.local) {
                    // C4：enum 參數之判別值（動態；見證模式恆等化）
                    return Ok(PirStmtKind::Discriminant {
                        of: Slot::disc(of.local),
                    });
                }
                Err(format!(
                    "{fname}: stmt[{i}] 判別值未知（of 非已建構 enum／enum 參數）"
                ))
            }
            ("Aggregate", payload) => {
                // dst := Aggregate([adt, variant], [operands])——struct／tuple／
                // enum variant 建構。欄位逐一落 dst 嘅 field 槽（encode 展開）。
                let arr = payload
                    .as_arr()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Aggregate 非陣列"))?;
                if arr.len() != 2 {
                    return Err(format!("{fname}: stmt[{i}] Aggregate 長度 {}", arr.len()));
                }
                let ops = arr[1]
                    .as_arr()
                    .ok_or_else(|| format!("{fname}: stmt[{i}] Aggregate operands 非陣列"))?;
                let mut fields = Vec::with_capacity(ops.len());
                for o in ops {
                    fields.push(parse_operand(fname, i, o)?);
                }
                let disc = resolve_adt_disc(types, fname, i, &arr[0])?;
                Ok(PirStmtKind::Aggregate { fields, disc })
            }
            (other, _) => Err(format!("{fname}: stmt[{i}] rvalue `{other}` 唔支援")),
        },
        _ => Err(format!("{fname}: stmt[{i}] rvalue 非單鍵 variant")),
    }
}

/// Aggregate[0] 嘅 ADT 資訊 → 判別值：enum variant → Some(判別值)、
/// struct（variant = Null）／tuple（adt = Null）→ None。
/// 判別值由 type_decls 解析（variant id ≠ 判別值，鐵律：絕不猜測）。
fn resolve_adt_disc(
    types: &Value,
    fname: &str,
    i: usize,
    adt_part: &Value,
) -> Result<Option<i64>, String> {
    if matches!(adt_part, Value::Null) {
        return Ok(None); // tuple／陣列聚合：無判別式
    }
    let inner = match adt_part {
        Value::Obj(p) if p.len() == 1 && p[0].0 == "Adt" => p[0]
            .1
            .as_arr()
            .ok_or_else(|| format!("{fname}: stmt[{i}] Aggregate.Adt 非陣列"))?,
        _ => return Err(format!("{fname}: stmt[{i}] Aggregate[0] 非 Adt／Null")),
    };
    let adt_ref = inner
        .first()
        .ok_or_else(|| format!("{fname}: stmt[{i}] Aggregate.Adt 空"))?;
    let adt_id = adt_ref
        .get("id")
        .and_then(|v| v.as_num())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| format!("{fname}: stmt[{i}] Adt.id 非數字"))?;
    let variant_id = match inner.get(1) {
        None | Some(Value::Null) => return Ok(None), // struct：單 variant 無判別值
        Some(v) => v
            .as_num()
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| format!("{fname}: stmt[{i}] Adt variant id 非數字"))?,
    };
    // type_decls 逐 def_id 對位（null 佔位項跳過）
    let tarr = types.as_arr().ok_or_else(|| {
        format!("{fname}: stmt[{i}] type_decls 缺失（enum Aggregate 判別值需要）")
    })?;
    let td = tarr
        .iter()
        .find(|t| {
            t.get("def_id")
                .and_then(|v| v.as_num())
                .and_then(|s| s.parse::<i64>().ok())
                == Some(adt_id)
        })
        .ok_or_else(|| format!("{fname}: stmt[{i}] type_decls 無 def_id={adt_id}"))?;
    let variants = td
        .get("kind")
        .and_then(|k| k.get("Enum"))
        .and_then(|e| e.as_arr())
        .ok_or_else(|| {
            format!("{fname}: stmt[{i}] type #{adt_id} 非 Enum（C3 只對 enum 記判別值）")
        })?;
    let vd = variants
        .iter()
        .find(|v| {
            v.get("id")
                .and_then(|x| x.as_num())
                .and_then(|s| s.parse::<i64>().ok())
                == Some(variant_id)
        })
        .ok_or_else(|| format!("{fname}: stmt[{i}] Enum #{adt_id} 無 variant {variant_id}"))?;
    let disc = vd
        .get("discriminant")
        .ok_or_else(|| format!("{fname}: stmt[{i}] variant 缺 discriminant"))?;
    // discriminant = {Signed|Unsigned: [ty, val]}——值係字串 token，同 parse_const 口徑
    let (tag, pair) = match disc {
        Value::Obj(p) if p.len() == 1 => (p[0].0.as_str(), &p[0].1),
        _ => return Err(format!("{fname}: stmt[{i}] discriminant 非單鍵 variant")),
    };
    let arr = pair
        .as_arr()
        .ok_or_else(|| format!("{fname}: stmt[{i}] discriminant.{tag} 非陣列"))?;
    let raw = arr
        .get(1)
        .and_then(|v| v.as_str().or_else(|| v.as_num()))
        .ok_or_else(|| format!("{fname}: stmt[{i}] discriminant 值非數字"))?;
    let val: i64 = raw
        .parse()
        .map_err(|_| format!("{fname}: stmt[{i}] discriminant `{raw}` 非 i64"))?;
    match tag {
        "Signed" => Ok(Some(val)),
        "Unsigned" if val >= 0 => Ok(Some(val)),
        "Unsigned" => Err(format!(
            "{fname}: stmt[{i}] Unsigned discriminant 為負 `{raw}`"
        )),
        other => Err(format!("{fname}: stmt[{i}] discriminant `{other}` 唔支援")),
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
                    return Err(format!(
                        "{fname}: stmt[{i}] Integer.{tag} 長度 {}",
                        arr.len()
                    ));
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
            (other, _) => Err(format!(
                "{fname}: stmt[{i}] 常數 `{other}` 唔支援（只做整數/布林）"
            )),
        },
        _ => Err(format!("{fname}: stmt[{i}] 常數非單鍵 variant")),
    }
}

// ---------------------------------------------------------------------------
// C4：concretize——參數組合 → 單路徑模擬（真語義 i64 口徑）+ 見證槽值表
// ---------------------------------------------------------------------------

/// 參數值（組合一點）。
#[derive(Debug, Clone, PartialEq)]
pub enum ParamVal {
    Int(i64),
    /// enum 參數：判別值 + 欄位值（C4 只支援 ≤1 欄位）
    Enum {
        disc: i64,
        field: Option<i64>,
    },
}

/// concretize 結果：攤平語句（guard 已消費）+ 見證槽值表 + 模擬 ret。
#[derive(Debug, Clone)]
pub struct ConcOut {
    pub path_idx: usize,
    pub values: BTreeMap<Slot, i64>,
    pub ret: Option<i64>,
}

/// concretize 錯誤：溢出（組合唔喺認證範圍）／其他降級原因。
#[derive(Debug, Clone)]
pub enum ConcErr {
    /// checked 運算溢出／溢出旗標非零：真實執行會 panic，組合排除。
    Overflow,
    Reason(String),
}

/// 參數組合 → 路徑選擇 + 全程模擬。路徑選擇：guard 值查表（enum 判別值軸
/// ／比較 {0,1}），fallback path 接收唔中任何 branch 常數嘅值。
pub fn concretize(
    body: &PirBody,
    params: &[ParamVal],
    callees: &BTreeMap<String, PirBody>,
) -> Result<ConcOut, ConcErr> {
    if params.len() != body.arg_count {
        return Err(ConcErr::Reason(format!(
            "參數值數量 {} ≠ arg_count {}",
            params.len(),
            body.arg_count
        )));
    }

    // 參數預填（每 path 相同 → 只建一次）
    let mut init: BTreeMap<Slot, i64> = BTreeMap::new();
    for (k, pv) in params.iter().enumerate() {
        match pv {
            ParamVal::Int(v) => {
                init.insert(Slot::whole(k + 1), *v);
            }
            ParamVal::Enum { disc, field } => {
                init.insert(Slot::disc(k + 1), *disc);
                if let Some(fv) = field {
                    init.insert(Slot::field(k + 1, 0), *fv);
                }
            }
        }
    }
    simulate_body(body, &init, callees, 0)
}

/// C5 調用鏈模擬深度上限（超限 = 遞迴／超深調用，如實降級）。
const CALL_DEPTH_CAP: usize = 4;

/// 核心：逐 path 嘗試模擬 + guard 驗證（scrutinee 槽值 == path.value；
/// fallback path 接收唔中任何 branch 常數嘅值）。guard 值由模擬確定。
/// 溢出唔立即傳——試晒所有 path 先報（避免誤判「真正行嘅 path」以外嘅溢出）。
fn simulate_body(
    body: &PirBody,
    init: &BTreeMap<Slot, i64>,
    callees: &BTreeMap<String, PirBody>,
    depth: usize,
) -> Result<ConcOut, ConcErr> {
    let name = body.fun_name.clone();
    let err = |m: String| ConcErr::Reason(m);
    let mut last: Option<ConcErr> = None;
    for (idx, path) in body.paths.iter().enumerate() {
        let mut values = init.clone();
        match simulate(
            &name,
            &path.stmts,
            &mut values,
            body.ret_slot.as_ref(),
            callees,
            depth,
        ) {
            Ok(ret) => {
                // guard 驗證
                if let (Some(s), Some(v)) = (&path.scrutinee, path.value) {
                    match values.get(s) {
                        Some(&actual) if actual == v => {}
                        Some(_) if path.fallback => {}
                        _ => {
                            last =
                                Some(err(format!("path[{idx}] guard 驗證失敗（唔係呢條 path）")));
                            continue;
                        }
                    }
                }
                return Ok(ConcOut {
                    path_idx: idx,
                    values,
                    ret,
                });
            }
            Err(ConcErr::Overflow) => {
                // 記住但試下一 path：真正行嘅 path 未必係呢條
                last = Some(ConcErr::Overflow);
                continue;
            }
            Err(e) => {
                last = Some(e);
                continue;
            }
        }
    }
    Err(last.unwrap_or_else(|| ConcErr::Reason("無路徑可模擬".to_string())))
}

/// 直線語句模擬（真語義；C2 口徑：checked 溢出 → Overflow、旗標非零 → Overflow）。
/// C5：Call 遞迴模擬（callee 經 callees lookup，深限 CALL_DEPTH_CAP）。
#[allow(clippy::too_many_arguments)]
fn simulate(
    name: &str,
    stmts: &[PirStmt],
    values: &mut BTreeMap<Slot, i64>,
    ret_slot: Option<&Slot>,
    callees: &BTreeMap<String, PirBody>,
    depth: usize,
) -> Result<Option<i64>, ConcErr> {
    for st in stmts {
        match &st.kind {
            PirStmtKind::Const { val } => {
                values.insert(st.dst.clone(), *val);
            }
            PirStmtKind::Copy { src } => {
                let v = values
                    .get(src)
                    .ok_or_else(|| ConcErr::Reason(format!("{name}: 模擬讀取未定義槽 {src:?}")))?;
                values.insert(st.dst.clone(), *v);
            }
            PirStmtKind::BinOp { op, a, b, checked } => {
                let x = slot_or_const(a, values);
                let y = slot_or_const(b, values);
                let (x, y) = match (x, y) {
                    (Some(x), Some(y)) => (x, y),
                    _ => return Err(ConcErr::Reason(format!("{name}: 模擬讀取未定義運算元"))),
                };
                if op.is_comparison() {
                    values.insert(st.dst.clone(), if op.compare(x, y) { 1 } else { 0 });
                } else {
                    let (v, of) = match op {
                        PirBinOp::Add => (x.wrapping_add(y), x.checked_add(y).is_none()),
                        PirBinOp::Sub => (x.wrapping_sub(y), x.checked_sub(y).is_none()),
                        PirBinOp::Mul => (x.wrapping_mul(y), x.checked_mul(y).is_none()),
                        _ => unreachable!("非比較非算術"),
                    };
                    if *checked {
                        values.insert(Slot::field(st.dst.local, 0), v);
                        values.insert(Slot::field(st.dst.local, 1), if of { 1 } else { 0 });
                        if of {
                            return Err(ConcErr::Overflow);
                        }
                    } else {
                        values.insert(st.dst.clone(), v);
                    }
                }
            }
            PirStmtKind::AssertFlagZero { flag } => {
                let f = values
                    .get(flag)
                    .ok_or_else(|| ConcErr::Reason(format!("{name}: 模擬讀取未定義旗標")))?;
                if *f != 0 {
                    return Err(ConcErr::Overflow);
                }
            }
            PirStmtKind::Discriminant { of } => {
                let d = values
                    .get(of)
                    .ok_or_else(|| ConcErr::Reason(format!("{name}: 模擬讀取未定義判別值")))?;
                values.insert(st.dst.clone(), *d);
            }
            PirStmtKind::Call { callee, args } => {
                // C5：本地純函數調用——遞迴模擬 callee body（值語義），
                // ret 寫入 dst 槽。enum 值傳參／ADT ret 屬後續 slice。
                if depth >= CALL_DEPTH_CAP {
                    return Err(ConcErr::Reason(format!(
                        "{name}: 調用鏈深度超限 {CALL_DEPTH_CAP}（遞迴／超深調用）"
                    )));
                }
                let cb = callees.get(callee).ok_or_else(|| {
                    ConcErr::Reason(format!("{name}: callee `{callee}` 唔可模擬（body 缺失）"))
                })?;
                if !cb.enum_params.is_empty() {
                    return Err(ConcErr::Reason(format!(
                        "{name}: callee `{callee}` 帶 enum 參數（enum 值傳參屬後續 slice）"
                    )));
                }
                if args.len() != cb.arg_count {
                    return Err(ConcErr::Reason(format!(
                        "{name}: 實參數 {} ≠ callee `{callee}` 參數數 {}",
                        args.len(),
                        cb.arg_count
                    )));
                }
                let mut init: BTreeMap<Slot, i64> = BTreeMap::new();
                for (k, a) in args.iter().enumerate() {
                    let v = slot_or_const(a, values).ok_or_else(|| {
                        ConcErr::Reason(format!(
                            "{name}: 實參 {k} 無模擬值（enum 值傳參屬後續 slice）"
                        ))
                    })?;
                    init.insert(Slot::whole(k + 1), v);
                }
                let out = simulate_body(cb, &init, callees, depth + 1)?;
                let rv = out.ret.ok_or_else(|| {
                    ConcErr::Reason(format!(
                        "{name}: callee `{callee}` 無模擬 ret 值（ADT ret 屬後續 slice）"
                    ))
                })?;
                values.insert(st.dst.clone(), rv);
            }
            PirStmtKind::Aggregate { fields, .. } => {
                for (k, f) in fields.iter().enumerate() {
                    let v = match f {
                        PirOperand::Const(c) => *c,
                        PirOperand::Slot(s) => *values.get(s).ok_or_else(|| {
                            ConcErr::Reason(format!("{name}: 模擬讀取未定義欄位"))
                        })?,
                    };
                    values.insert(Slot::field(st.dst.local, k as u8), v);
                }
                // 整槽不透明（不記值；欄位槽承擔全部語義）
            }
        }
    }
    Ok(ret_slot.and_then(|s| values.get(s).copied()))
}

fn slot_or_const(op: &PirOperand, values: &BTreeMap<Slot, i64>) -> Option<i64> {
    match op {
        PirOperand::Const(v) => Some(*v),
        PirOperand::Slot(s) => values.get(s).copied(),
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
        let root =
            LlbcRoot::parse(include_str!("../tests/charon_fixtures/async_simple.llbc")).unwrap();
        let m = lift(&root);
        assert!(m.has_missing);
        assert_eq!(m.missing_body_count(), 1, "async body=Error → 1 missing");
        assert_eq!(m.lowerable_funs().count(), 0);
    }
}
