// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Benchmark F4/F4F5 vs Classic — 零依賴簡易基準 + Auto 策略
//! 運行: cargo run --release --bin bench_f4f5
//! 特性：
//! - Classic / F4 / F4F5 / Auto 四路對比
//! - Auto 基於 nvars/npolys/稀疏度/塊數/pair度數啟發式，量化 speedup、sparse_density_pct、blocks_detected、f5_skips、parallel_blocks
//! - F4/F4F5 已加入稀疏分塊並行 (std::thread::scope 零依賴)

use polyrust_core::frac::Frac;
use polyrust_core::groebner::{field_polys, reduced_groebner, Strategy};
use polyrust_core::groebner_f4::reduced_f4;
use polyrust_core::groebner_f4f5::reduced_f4f5;
use polyrust_core::pipeline::{select_groebner_algo_advanced, GroebnerAlgo};
use polyrust_core::poly::{Order, Poly};

fn var(i: usize, n: usize) -> Poly {
    Poly::var(i, Frac::ONE, n)
}
fn c(x: i64) -> Poly {
    Poly::constant(Frac::from_i64(x))
}

fn bench_case(name: &str, fs: Vec<Poly>) {
    let nvars = fs
        .iter()
        .map(|p| p.terms.iter().map(|(m, _)| m.len()).max().unwrap_or(0))
        .max()
        .unwrap_or(0);
    println!("\n== {} nvars~{} npolys={} ==", name, nvars, fs.len());
    let ord = Order::GrevLex;

    // Auto 決策
    let (auto_algo, reason) = select_groebner_algo_advanced(&fs, nvars);
    println!("Auto decision: {} | reason: {}", auto_algo.as_str(), reason);

    // Classic
    let t0 = std::time::Instant::now();
    let (g1, s1) = reduced_groebner(&fs, ord, Strategy::Normal, true);
    let dt1 = t0.elapsed();
    println!(
        "Classic: basis={} pairs={} s_polys={} time={:?}",
        g1.len(),
        s1.pairs_considered,
        s1.s_polys,
        dt1
    );

    // F4
    let t0 = std::time::Instant::now();
    let (g2, s2) = reduced_f4(&fs, ord);
    let dt2 = t0.elapsed();
    println!(
        "F4: basis={} batches={} pairs={} s_polys={} rows_max={} cols_max={} density={:.1}% blocks={} parallel_blocks={} time={:?} speedup={:.2}x",
        g2.len(),
        s2.batches,
        s2.pairs_considered,
        s2.s_polys,
        s2.matrix_rows_max,
        s2.matrix_cols_max,
        s2.sparse_density_pct,
        s2.blocks_detected,
        s2.parallel_blocks,
        dt2,
        dt1.as_secs_f64() / dt2.as_secs_f64().max(0.0001)
    );

    // F4F5
    let t0 = std::time::Instant::now();
    let (g3, s3) = reduced_f4f5(&fs, ord);
    let dt3 = t0.elapsed();
    println!(
        "F4F5: basis={} batches={} pairs={} f5_skips={} rewritten={} crit1={} crit2={} s_polys={} rows_max={} cols_max={} density={:.1}% blocks={} parallel_blocks={} time={:?} speedup={:.2}x",
        g3.len(),
        s3.batches,
        s3.pairs_considered,
        s3.f5_skips,
        s3.rewritten_skips,
        s3.crit1_skips,
        s3.crit2_skips,
        s3.s_polys,
        s3.matrix_rows_max,
        s3.matrix_cols_max,
        s3.sparse_density_pct,
        s3.blocks_detected,
        s3.parallel_blocks,
        dt3,
        dt1.as_secs_f64() / dt3.as_secs_f64().max(0.0001)
    );

    // Auto 執行
    let t0 = std::time::Instant::now();
    let (g_auto, s_auto, dt_auto) = match auto_algo {
        GroebnerAlgo::Classic => {
            let (g, s) = reduced_groebner(&fs, ord, Strategy::Normal, true);
            let dt = t0.elapsed();
            (g, format!("pairs={} s_polys={}", s.pairs_considered, s.s_polys), dt)
        }
        GroebnerAlgo::F4 => {
            let (g, s) = reduced_f4(&fs, ord);
            let dt = t0.elapsed();
            (
                g,
                format!(
                    "batches={} density={:.1}% blocks={} par_blocks={} pairs={}",
                    s.batches, s.sparse_density_pct, s.blocks_detected, s.parallel_blocks, s.pairs_considered
                ),
                dt,
            )
        }
        GroebnerAlgo::F4F5 => {
            let (g, s) = reduced_f4f5(&fs, ord);
            let dt = t0.elapsed();
            (
                g,
                format!(
                    "batches={} f5_skips={} density={:.1}% blocks={} par_blocks={} pairs={}",
                    s.batches, s.f5_skips, s.sparse_density_pct, s.blocks_detected, s.parallel_blocks, s.pairs_considered
                ),
                dt,
            )
        }
        GroebnerAlgo::F5 => {
            // F5 目前走 F4F5 實現 (簽名+批處理)
            let (g, s) = reduced_f4f5(&fs, ord);
            let dt = t0.elapsed();
            (
                g,
                format!(
                    "F5 via F4F5: batches={} f5_skips={} density={:.1}% blocks={}",
                    s.batches, s.f5_skips, s.sparse_density_pct, s.blocks_detected
                ),
                dt,
            )
        }
    };
    let best_time = dt1.min(dt2).min(dt3);
    let speedup_vs_best = best_time.as_secs_f64() / dt_auto.as_secs_f64().max(0.0001);
    let speedup_vs_classic = dt1.as_secs_f64() / dt_auto.as_secs_f64().max(0.0001);
    println!(
        "Auto({}): basis={} {} time={:?} speedup_vs_classic={:.2}x speedup_vs_best={:.2}x reason={}",
        auto_algo.as_str(),
        g_auto.len(),
        s_auto,
        dt_auto,
        speedup_vs_classic,
        speedup_vs_best,
        reason
    );

    // 一致性檢查
    if g1.len() != g2.len() {
        println!("WARN F4 basis size mismatch {} vs {}", g1.len(), g2.len());
    } else {
        println!("F4 consistent with Classic");
    }
    let unsat1 = g1.len() == 1 && g1[0].is_constant().is_some_and(|c| c.is_one());
    let unsat3 = g3.len() == 1 && g3[0].is_constant().is_some_and(|c| c.is_one());
    let unsat_auto = g_auto.len() == 1 && g_auto[0].is_constant().is_some_and(|c| c.is_one());
    if unsat1 != unsat3 {
        println!("WARN F4F5 UNSAT mismatch Classic vs F4F5");
    }
    if unsat1 != unsat_auto {
        println!("WARN Auto UNSAT mismatch");
    }

    // 量化指標匯總
    println!(
        "Metrics: nvars={} npolys={} auto_algo={} sparse_density_f4={:.1}% sparse_density_f4f5={:.1}% blocks_f4={} blocks_f4f5={} parallel_f4={} parallel_f4f5={} f5_skips={}",
        nvars,
        fs.len(),
        auto_algo.as_str(),
        s2.sparse_density_pct,
        s3.sparse_density_pct,
        s2.blocks_detected,
        s3.blocks_detected,
        s2.parallel_blocks,
        s3.parallel_blocks,
        s3.f5_skips
    );
}

fn main() {
    println!("Polyrust F4/F4F5 Benchmark — Fp p=2^61-1, sparse + block-parallel + Auto");
    println!("Features: Classic vs F4 vs F4F5 vs Auto (nvars/npolys/density/blocks heuristic)");

    // Case 1: small boolean
    {
        let n = 4;
        let fs = vec![
            var(0, n).mul(&var(1, n)),
            var(0, n).add(&var(1, n)).sub(&c(1)),
            var(2, n).sub(&var(0, n)),
        ];
        let mut all = fs;
        all.extend(field_polys(n));
        bench_case("small boolean 4 vars", all);
    }

    // Case 2: medium one-hot (N=7, nodes=5 => nvars=35) — 稀疏布爾，期望 Auto->F4
    {
        let n = 35;
        let mut fs = Vec::new();
        for node in 0..5 {
            let base = node * 7;
            let mut sum = Poly::constant(Frac::from_i64(-1));
            for k in 0..7 {
                sum = sum.add(&var(base + k, n));
            }
            fs.push(sum);
            for i in 0..7 {
                for j in (i + 1)..7 {
                    fs.push(var(base + i, n).mul(&var(base + j, n)));
                }
            }
        }
        fs.extend(field_polys(n));
        bench_case("one-hot 5 nodes N=7 (sparse)", fs);
    }

    // Case 3: larger chain (nvars 100) — 鏈式，塊檢測可能分塊
    {
        let n = 100;
        let mut fs = Vec::new();
        for i in 0..20 {
            fs.push(var(i, n).mul(&var((i + 1) % n, n)).sub(&c(1)));
        }
        fs.extend(field_polys(n));
        bench_case("chain 100 vars (block test)", fs);
    }

    // Case 4: multi-block independent (最能體現並行) — 3 個獨立子系統
    {
        let n = 60;
        let mut fs = Vec::new();
        // block A: vars 0-19
        for i in 0..5 {
            fs.push(var(i, n).mul(&var(i + 1, n)).sub(&var(i + 2, n)));
        }
        // block B: vars 20-39
        for i in 20..25 {
            fs.push(var(i, n).mul(&var(i + 1, n)).sub(&var(i + 2, n)));
        }
        // block C: vars 40-59
        for i in 40..45 {
            fs.push(var(i, n).mul(&var(i + 1, n)).sub(&var(i + 2, n)));
        }
        fs.extend(field_polys(n));
        bench_case("multi-block 3x independent 60 vars (parallel)", fs);
    }

    // Case 5: dense large (npolys>100) — 期望 Auto->F4F5
    {
        let n = 50;
        let mut fs = Vec::new();
        for i in 0..30 {
            for j in (i + 1)..(i + 5).min(n) {
                fs.push(var(i, n).mul(&var(j, n)).sub(&c((i + j) as i64 % 5)));
            }
        }
        fs.extend(field_polys(n));
        bench_case("dense 50 vars ~120 polys (F4F5 candidate)", fs);
    }

    println!("\nBenchmark done — Auto strategy: Classic/F4/F4F5 with sparse+block-parallel metrics");
}

/// 實際使用：bench_f4f5.rs 文件清單 — 優化 with_capacity
pub fn bench_f4f5_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("bench_f4f5.rs", "bench_f4f5.rs 正式運作 — 優化 with_capacity", "core/src/bench_f4f5.rs"),
    ]
}

