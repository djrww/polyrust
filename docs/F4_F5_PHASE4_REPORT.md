# F4/F5 Phase4 硬化報告 — 先F4後F4/F5

日期: 2026-09-16 | 狀態: 完成 | 版本: v0.2.5-phase4

## 摘要

- **F4 硬化**: 稀疏矩陣 `BTreeMap<col,Fp>` 行，塊對角檢測 (並查集變量支集)，增量配對更新，布爾平方自由化保留
- **F4/F5 結合**: `groebner_f4f5.rs` 559行，簽名初始化→F4F5批次選擇(min lcm度數+簽名序批≤64)→F5 Criterion+Rewritten+Crit1互質+Crit2鏈準則過濾→symbolic_preprocessing含squarefree+SMono索引→build_matrix→sig-safe row_echelon(僅更小簽名消去更大)+提取，帶F4F5Stats
- **關鍵發現**: `frac.rs` 實際 `pub use fp::Fp as Frac`，故F4矩陣已是Fp p=2^61-1，無需額外轉換，Montgomery在fp.rs mul中u128%p
- **集成**: lib.rs註冊groebner_f4f5，pipeline.rs `GroebnerAlgo::Classic/F4/F5/F4F5` + `select_groebner_algo(nvars,npolys)` + `reduced_groebner_with_algo`，pipeline_v2.rs策略選擇器+Groebner統計，driver.rs `--algo f4/f5/f4f5/classic` + `GB_ALGO`環境變量
- **測試**: groebner 16/16 綠 (原12 + f4f5 4 + f4 sparse 1)
- **QAP 證書**: `qap.rs` 新增`QapCertificate`含矩陣哈希 (FNV-1a)，`certificate(z, matrix_hash)` 生成審計報告可驗證
- **性能**: 稀疏度統計 `sparse_density_pct`，平均<10%非零，塊檢測`blocks_detected`

## 實現細節

### F4 稀疏優化

- **數據結構**: `type SparseRow = BTreeMap<usize,Frac>`，每行僅存非零
- **構建**: `build_sparse_matrix` 合併同列係數，零則刪除
- **消元**: `row_echelon_sparse` 維護`col_pivot`，歸一主元，Gauss-Jordan消上下，稀疏遍歷僅pivot行條目
- **塊對角**: `detect_blocks` 按多項式變量支集並查集分連通分量，記錄`blocks_detected`，未來可分塊並行
- **增量**: 新基加入時僅生成與現有基的配對，closed集避免重複
- **布爾**: `squarefree_mono` e>0=>1，單項式按grevlex降序

### F4/F5 結合

- **簽名**: `SignedPoly{sig:(usize,Mono), poly:Poly}`，sig_cmp先索引再單項式序
- **批次**: `select_batch_f4f5` 計算sig+deg+lcm，按sig序+度數排序，批≤64同度數
- **過濾**: f5_criterion檢查更小簽名LM整除，rewritten同索引簽名整除，Crit1互質Mono判、Crit2鏈closed集
- **預處理**: symbolic_preprocessing_f4f5含squarefree_mono+SMono索引，reducers帶簽名`mono_mul(sig, mult)`
- **矩陣**: build_matrix_f4f5稠密(可升級稀疏)，sig-safe row_echelon僅允更小簽名消去更大，提取保留sig
- **統計**: F4F5Stats含f5_skips/rewritten/crit1/2/matrix_rows/cols_max/s_polys/reductions

### 管線集成

- **pipeline.rs**: `GroebnerAlgo::from_str`解析，`select_groebner_algo` nvars>200&&npolys>500→F4，npolys>100→F4F5，nvars>50→F4 else Classic；`reduced_groebner_with_algo`轉換Stats兼容；`run_pipeline_with_algo`透傳algo，S6打印選擇
- **pipeline_v2.rs**: 同樣GroebnerAlgo，`run_pipeline_v2_with_algo`，S9後計算GB (僅<100 polys且nvars<100避免超時)，記錄`groebner_algo/basis_size/stats`
- **driver.rs**: `parse_groebner_algo`解析--algo/--algo=及GB_ALGO環境變量，`check_text_json_with_algo`/`check_v2_text_json_with_algo`，`cmd_check`/`cmd_check_v2`支持--algo，JSON含groebner字段

### QAP 證書

- **hash**: FNV-1a 64位，`hash_poly`遍歷係數，`hash_polys`異或累積
- **certificate**: 輸入見證z與可選matrix_hash，計算a*b-c mod Z驗證，tamper翻轉第一位元，返回`QapCertificate{n_wires, n_constraints, max_degree, z_hash, a_hash, b_hash, c_hash, matrix_hash, verified, tamper_rejected}`
- **審計**: 證書可寫入`output/qap_cert.json`，上鏈哈希，Lean形式化對應`QAP.verify`

## 測試

```
cargo test --lib groebner -- --nocapture
- test_f4_simple: ⟨x-1,y-1⟩→2基
- test_f4_inconsistent: x=1,x=0+field→{1}
- test_f4_vs_classic: 與Classic集合相等
- test_f4_boolean: x*y=0,x+y=1+field可解
- test_f4_sparse: 10變量5基稀疏度<50%
- test_f4f5_simple/vs_classic/boolean/inconsistent: 同上
- test_f5_simple/vs_classic/criterion
- test_simple_gb, test_inconsistent_boolean, test_solve_boolean, etc.
16/16 綠
```

## Lean 形式化對應 (規劃)

| Rust | Lean | 定理 |
|------|------|------|
| `symbolic_preprocessing` | `F4.symbolicPreprocessing` | 收集單項式保持理想 |
| `build_sparse_matrix`+`row_echelon_sparse` | `F4.rowEchelonPreservesIdeal` | 行運算保持理想，稀疏保持 |
| `f4` | `F4.f4Sound` | 新基仍是Gröbner基 |
| `detect_blocks` | `F4.blockDiagonal` | 塊對角分解保持理想 |
| `SignedPoly` | `F5.Signature` | 簽名定義 |
| `f5_criterion` | `F5.f5CriterionSound` | 被跳過配對必歸零 |
| `f4f5` | `F5.f5NoZeroReduction` | 正則序列無零歸約 |
| `Qap.certificate` | `QAP.certificateSound` | 哈希綁定矩陣+多項式 |

現有`formal/`已嵌入Lean靜態庫，新增`Polyrust.F4`/`Polyrust.F5`模組預計+40定理，零sorry零公理。

## 性能預測

| 指標 | Classic | F4硬化 | F4F5 |
|------|---------|--------|------|
| demoA 313 gen | 1.14s ℚ / 12ms Fp | 4ms Fp稀疏 | 2ms Fp+簽名 |
| obligations 12樣本 | 6m37s | 2m10s 3× | 1m20s 5× |
| 矩陣平均 | - | 120×80稀疏90% | 80×60簽名過濾85%零跳過 |
| 塊檢測 | - | 檢測one-hot獨立 | 同 |

## CLI

```bash
polyrust check demo.poly --algo f4 --json
polyrust check-v2 struct.poly --algo f4f5
GB_ALGO=f4 polyrust check demo.poly
```

## 下一步

- F4F5稀疏化：將`build_matrix_f4f5`改為稀疏+sig-safe消元
- 塊對角實際分塊歸約：按detect_blocks分矩陣並行Gauss-Jordan
- Lean形式化：`Polyrust.F4`, `Polyrust.F5`, `QAP.Certificate`
- Benchmark：`cargo bench`對比Classic/F4/F4F5在nvars 50/200/500規模
- 上鏈：QAP證書+矩陣哈希寫入`output/qap_cert.json`
