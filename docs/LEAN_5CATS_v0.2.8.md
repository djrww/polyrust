# v0.2.8 Lean 五类引理 — 完成度报告

> 分支 `v0.2.8` | Lean 4.33.1 | `lake build` 48 jobs 綠 | 零 sorry / 零公理

| 类 | 文件 | 定位 | 定理数* | 核心引理示例 | 状态 |
|---|---|---|---|---|---|
| **補全（雙向完備）** | `lean/Polyrust/Completion.lean` (573行) | 单向⇒双向 `↔`：clauseSat↔polyZero、borrow_sat↔clean、typable↔root、parse/gen、one-hot、field poly、F4矩阵↔S归约、F5跳过↔零归约、Fp嵌入 | ~70 | `clauseSat_iff_polyZero`、`borrow_sat_iff_clean_completion`、`typable_iff_root_completion`、`oneHot_iff_exists_unique`、`field_poly_iff_bool`、`f4_ideal_invariant_iff`、`f5_skip_iff_zero` | ✅ 完成 |
| **新增（鐵律核心命題）** | `lean/Polyrust/IronLaw.lean` (425行) + `RustcAlign.lean` (99行) | 系统不可违铁律：one-hot排他、x²−x=0、借用互斥、lifetime无环、unsafe边界前移、watch保义、子句对偶、F4/F5不变、Emit IR对应 | ~90 | `iron_tycheck_exclusive_2`、`iron_field_poly_bit`、`iron_borrow_clash_unsat`、`iron_static_outlives_all`、`iron_all_unsafe_safe_example_no_ub`、`iron_emit_text_no_runtime_ub`、`rustcGatedSound`、`noFalseCertified` | ✅ 完成 |
| **衍生（由核心推出）** | `lean/Polyrust/Derived.lean` (404行) | 由铁律直接推的结论：pair/sum可定型、borrow 1∈理想、isMonoAt、watch多步、clause存在、F4块稀疏、F5零消除 | ~60 | `derived_typable_pair_iff`、`derived_borrow_clash_one_mem`、`derived_isMonoAt_of_root`、`derived_watchMoves_preserve_sat`、`derived_f4_block_diagonal_independent`、`derived_f5_zero_85` | ✅ 完成 |
| **次要（支撐性）** | `lean/Polyrust/Minor.lean` (425行) | 底层组合/位元支撑：bit运算、Lit、MonoExp、supportLe、List求和、field poly、one-hot基础、F4稀疏行、FNV哈希、签名比较 | ~80 | `minor_bit_mul_self`、`minor_dividesM_trans`、`minor_listSum_nonneg`、`minor_field_poly_bit`、`minor_f4_sparse_row_zero`、`minor_sigLT_trans` | ✅ 完成 |
| **迭代（增強）** | `lean/Polyrust/IncrementalIteration.lean` (726行) | 解析/生成/检查/借用/F4/F5的迭代收敛：parseFuel单调、gen长度、borrow子集单调、F4三批叠加、F5签名传递、fuel收敛、端到端t9+borrow+f4f5 | ~70 | `parseFuel_mono`、`sizeT_add_le`、`borrowSystem_pairs_mono`、`f4_ideal_invariant_iter3`、`f5_criterion_mono`、`fuel_iter_mono`、`incremental_iteration_complete` | ✅ 完成 |

\* 含 V3 Auto 反馈三层（code→V3_auto / 4-example→Poly / Auto反馈链）每类追加 10+ 深度引理

## 验证

```bash
cd lean && lake build            # 48 jobs 綠，僅 linter 警告
```

## RSAP R3 对准

- 新文件 `RustcAlign.lean` 归入**新增**类（铁律核心：对准判定三值与rustc四值、10定理）
- `TCB.md`、`RSAP_RESULT.md`、`RUSTC_SEMANTIC_ALIGNMENT_PLAN.md` 入册

## 发布物

- 分支 `v0.2.8` 从 `v0.2.6` + `e24f0d5 (item12+where/GAT)` + RSAP R3 构建
- 标籤 `v0.2.8` 指向本提交
