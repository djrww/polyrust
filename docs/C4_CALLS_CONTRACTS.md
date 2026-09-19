# C4 Calls + Contracts + Panic-freedom 記檔（2026-09-20）

C3 完咗 loop/fuel，C4 攻最後一個 C 級：**函數調用語義、`@require/@ensure` 合約、panic-freedom 檢查面**。
驗收：struct/impl/trait 案例差分推進 + commercial 10 案例 ≥6 可判（含 enterprise_ide false-UNSAT 修復）。

## 1. 合約模組 `core/src/contract.rs`

LLBC 來源碼註解雙軌：

```
# @require x == 3        → ClauseKind::ExactEq   → premise：x − 3 = 0（參數多項式直接入關）
# @require n >= 0        → ClauseKind::CmpAbstract → 新鮮 bool b，premise：b = 1（保守：只鎖「條件成立」事實）
# @ensure result == x*x  → 只收集報告，唔 enforce（事後性質，C4 層位唔夠資訊化做後置約束）
# @opaque 樣式           → ClauseKind::Opaque    → marker only
```

**安全網**：ExactEq 嘅 ident 必須喺函數參數名單（LLBC locals 1..=arg_count 嘅 `name`），
否則降級 Opaque——唔畀隨便一個 ident 就入 premise。測試 6/6 綠（單元覆蓋三級 + 冒號軌兼容 + ensure 收集）。

## 2. 參數 eager 綁定（llbc_lower.rs）

之前 v4 嘅參數係 lazy——遇唔用唔綁。合約 premise 要直接指到參數變數，
故**分析入口提前建變數**：locals 1..=arg_count 全部 eager fresh + ssa 插入 + param_var 名→變數表，
再將 ClauseKind::ExactEq 嘅 `var − c = 0` 入 path constraint。`V4Outcome` 加：

- `assert_obligations: Vec<String>`——程式路徑上真實觸及嘅 assert 種類（dedup），
  panic-freedom 檢查面：SAT 且 obligations 非空 ⇒ 呢啲 panic 位「可到達但未違反」；
- `contract_report: Vec<String>`——require/ensure 逐條記錄。

## 3. Assert 處理（延續 C3 補丁點）

`Assert{cond}` 落入 obligations（Overflow / BoundCheck / …）。**enum_list** 實測
`asserts=["Overflow"]` + paths=4——assert 檢查面真係工作，唔係空轉。

## 4. 法證新增 LLBC 形狀（C2 規則：實測過先入白名單）

| 形狀 | 出處 | 處理 |
|---|---|---|
| body variant `Intrinsic` | enum_color 嘅 derive(PartialEq) 合成 body | `BodyKind::Intrinsic`（missing 類，唔畀靜默語義漂移跟 Structured parse） |
| statement `PlaceMention` | struct_with_enum（fake-read/雷區標記） | NopLike |

## 5. 實測結果

### struct/enum 15/15 SAT（`struct_enum_fifteen_differential`）

struct_point → enum_option 全 SAT；enum_list vars=51 paths=4（switch 分支 × inline fuel），
enum_with_data 同有 assert=["Overflow"]；marker 面集中
aggregate-abstraction / discriminant-abstraction / switch-fallback-abstraction / ref-abstraction /
call-external / cmp-abstraction / call-inline-fuel / assert-abstracted——全部有語義表對應。

### commercial 10/10 全 SAT（≥6 驗收達成 ×1.67）

| 案例 | vars | 判決 |
|---|---|---|
| password_gen | 8 | SAT |
| invoice_engine | … | SAT |
| rate_limiter | … | SAT |
| cache_shard | … | SAT |
| audit_log | … | SAT |
| workflow_dag | … | SAT |
| token_vesting | … | SAT |
| config_merge | … | SAT |
| **enterprise_ide** | **3** | **SAT（舊 false-UNSAT 修復 ✓）** |
| self_evolving | 24 | SAT |

全鏈冇外部 crate 依賴（Charon 單檔 accept），markers 集中
call-external / ref-abstraction / aggregate-abstraction / scalar-const-abstraction / cast-passthrough。

### 合約端到端

- `contract_premise_end_to_end`：sqr + `@require x == 3` → report=["require ExactEq: x == 3", "ensure 收集…result == x*x"]，obligations=["Overflow"]，SAT ✓
- `contract_exact_eq_infeasible_detects_unsat`：per-path 矛盾（x==3 前提 vs x==7 事實）→ GB 層逐路徑克 Sigma，UNSAT ✓（呢個就係「ExactEq 真係入咗 premise」嘅硬證據）

## 6. 全量

- `cargo test --lib`：**128/128 綠，71.6s**（慢頭係既有 GB 重測，唔係 C4 新增）
- license-scan：見 commit

## 7. 殘尾（C5/日後）

- ensure 只報告唔 enforce——要真 enforce 得動 SMT 後置（v4 承諾嘅 GB-only 界線）
- CmpAbstract 目前 bool 鎖定，語義弱（只保「成立」）——可升級做半區界多項式
- trait 動態分派（dyn）Charon 唔支援 ⇒ 照舊 missing 入檔
