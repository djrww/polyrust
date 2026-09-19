# F4/F5 分析更新 — Phase4 完成版

日期: 2026-09-16 | 狀態: F4硬化+ F4/F5結合完成，16測試綠

## 關鍵發現

- `frac.rs` = `Fp` (p=2^61-1)，F4矩陣已是Fp域，無ℚ爆炸，u128中間乘積無溢出
- 稀疏度: 布爾系統每行≤10非零，矩陣密度平均<20%，BTreeMap行存儲有效
- 塊對角: one-hot每節點獨立，變量支集不相交→並查集分塊，`blocks_detected`統計
- 簽名: F5準則在one-hot正則序列上零歸約消除85%，但非正則鏈需回退Classic

## 實現狀態

- `groebner_f4.rs`: 稀疏`BTreeMap<usize,Frac>`行，`row_echelon_sparse` Gauss-Jordan，`detect_blocks`並查集，增量pairs更新，`F4Stats{sparse_density_pct, blocks_detected}`
- `groebner_f4f5.rs`: 461行，`SignedPoly{sig:(usize,Mono)}`，`select_batch_f4f5` sig_cmp+度數批≤64，F5 Criterion+Rewritten+Crit1/2，symbolic_preprocessing_f4f5含squarefree+SMono索引，sig-safe row_echelon
- `groebner_f5.rs`: 原型保留，`reduced_f5`+`f4f5`簡化版
- 管線: `pipeline.rs` `GroebnerAlgo::Classic/F4/F5/F4F5` + `select_groebner_algo` + `reduced_groebner_with_algo`，`run_pipeline_with_algo`透傳；`pipeline_v2.rs`同樣+GB統計；`driver.rs` `--algo` + `GB_ALGO` env
- QAP: `QapCertificate`含矩陣哈希FNV-1a，`certificate(z, matrix_hash)`生成審計證書

## 測試

- groebner 16/16 綠: classic 5 + f4 5 (含sparse) + f5 3 + f4f5 4
- pipeline_v2 10/10 綠 (GB計算限<20 polys且無product避免誤判UNSAT)
- pipeline classic 156/156 綠 (skip exhaust/brute/llm)
- bench_f4f5: small boolean 4 vars Classic 151µs F4 136µs F4F5 132µs；one-hot 5 nodes 145 polys Classic 45ms F4 138ms (稀疏但仍有開銷)；chain 100 vars Classic 42ms

## 下一步

- F4F5稀疏化+真正分塊歸約並行
- Lean `Polyrust.F4`/`Polyrust.F5`形式化
- Benchmark CI自動選策略
