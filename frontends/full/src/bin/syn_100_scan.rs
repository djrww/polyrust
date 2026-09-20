//! syn(full)+visit 100 持续扫 — 98(v4) + 86(v1) groundtruth
//! 运行: cargo run -p polyrust-full --features syn --bin syn_100_scan -- --loop 5

#[cfg(feature = "syn")]
use polyrust_full::syn_100::continuous_scan_once;

fn main() {
    #[cfg(not(feature = "syn"))]
    {
        eprintln!("syn feature not enabled");
        return;
    }
    #[cfg(feature = "syn")]
    {
        let args: Vec<String> = std::env::args().collect();
        let loop_secs = args.iter().position(|a| a=="--loop").and_then(|i| args.get(i+1)).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        if loop_secs > 0 {
            eprintln!("[syn_100_scan] 持续扫 启动，每 {}s 一轮 (98+86 groundtruth)", loop_secs);
            loop {
                let out = continuous_scan_once();
                println!("{}", out);
                eprintln!("[syn_100_scan] {} {:?}", chrono_now(), "tick");
                std::thread::sleep(std::time::Duration::from_secs(loop_secs));
            }
        } else {
            println!("{}", continuous_scan_once());
        }
    }
}

#[cfg(feature = "syn")]
fn chrono_now() -> String {
    // 轻量时间戳，不引 chrono 依赖
    format!("{:?}", std::time::SystemTime::now())
}
