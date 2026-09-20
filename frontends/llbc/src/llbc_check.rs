// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! `polyrust-llbc-check` —— C5 預設切換出口。
//!
//! 用法：
//! ```text
//! polyrust-llbc-check <input{.rs|.llbc|.poly}> [--engine v4|v1] [--charon PATH]
//! ```
//! - 輸入 `.rs`：**默認 v4**（真 charon → 真 serde → core v4 分析；C5 預設切換）
//! - 輸入 `.llbc`：直接 v4（唔使 charon）
//! - 輸入 `.poly` / `--engine v1`：舊 v1 鏈（dsl::resolve → 完整代數管線），後備
//!
//! exit code：Sat=0、Unsat=1、Unknown=2、執行錯誤=3。輸出永遠係統一 JSON。

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use polyrust_llbc::report::{error_report, exit_code, v4_report};

const TOOL: &str = "polyrust-llbc-check";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (input, engine, charon) = match parse_args(&args) {
        Ok(v) => v,
        Err(usage) => {
            eprintln!("{usage}");
            return ExitCode::from(3);
        }
    };
    let input_path = PathBuf::from(&input);
    let src = match std::fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => return finish(error_report(TOOL, &input, "read", &format!("讀取失敗：{e}")), 3),
    };

    match engine.as_str() {
        "v1" => run_v1(&input, &src),
        _ => run_v4(&input, &input_path, &src, charon.as_deref()),
    }
}

fn parse_args(args: &[String]) -> Result<(String, String, Option<String>), String> {
    if args.is_empty() {
        return Err(format!(
            "usage: {TOOL} <input{{.rs|.llbc|.poly}}> [--engine v4|v1] [--charon PATH]"
        ));
    }
    let mut input = None;
    let mut engine = None;
    let mut charon = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--engine" => engine = it.next().cloned(),
            "--charon" => charon = it.next().cloned(),
            s if !s.starts_with("--") && input.is_none() => input = Some(s.to_string()),
            s => return Err(format!("未知參數 {s}\nusage: {TOOL} <input> [--engine v4|v1] [--charon PATH]")),
        }
    }
    let input = input.ok_or_else(|| "缺 input 檔".to_string())?;
    // 預設切換（C5）：.rs/.llbc → v4；.poly → v1
    let engine = engine.unwrap_or_else(|| {
        if input.ends_with(".poly") {
            "v1".to_string()
        } else {
            "v4".to_string()
        }
    });
    Ok((input, engine, charon))
}

fn finish(report: serde_json::Value, code: u8) -> ExitCode {
    println!("{}", serde_json::to_string_pretty(&report).unwrap_or_else(|_| report.to_string()));
    ExitCode::from(code)
}

// ────────────────────────── v4 路線 ──────────────────────────

fn run_v4(input: &str, input_path: &Path, src: &str, charon: Option<&str>) -> ExitCode {
    let t0 = std::time::Instant::now();
    let root = if input.ends_with(".llbc") {
        match polyrust_llbc::parse_llbc_file(input_path) {
            Ok(r) => r,
            Err(e) => return finish(error_report(TOOL, input, "llbc-parse", &e), 3),
        }
    } else {
        match run_charon(input_path, charon) {
            Ok(root) => root,
            Err((stage, msg, code)) => {
                if stage == "rustc-reject" && code == 1 {
                    let rep = serde_json::json!({
                        "api_version": "0.3",
                        "tool": TOOL,
                        "engine": "v4",
                        "input": input,
                        "status": "ok",
                        "verdict": "Unsat",
                        "reason": format!("rustc-reject: {msg}"),
                    });
                    return finish(rep, 1);
                }
                return finish(error_report(TOOL, input, &stage, &msg), code);
            }
        }
    };
    match polyrust_core::llbc_lower::analyze_module_from_source(&root, src) {
        Ok(out) => {
            let code = exit_code(&out.verdict);
            finish(v4_report("v4", input, src, t0.elapsed().as_millis(), &out), code)
        }
        Err(e) => finish(error_report(TOOL, input, "analyze", &e), 3),
    }
}

/// 調真 charon（同 scripts/c0_spike.py 嘅 CLI 合約：`charon rustc -- f --crate-type=rlib --edition=2021`，
/// cwd=臨時目錄，.llbc 落喺 cwd）。120s 超時強殺。
fn run_charon(
    input_path: &Path,
    charon: Option<&str>,
) -> Result<polyrust_core::charon_llbc::LlbcRoot, (String, String, u8)> {
    let bin = charon
        .map(|s| s.to_string())
        .or_else(|| std::env::var("CHARON_BIN").ok())
        .or_else(|| which("charon"))
        .ok_or_else(|| {
            (
                "charon-missing".to_string(),
                "搵唔到 charon binary：--charon PATH、CHARON_BIN 環境變數或 PATH 都得（v4 需要 charon；\
                 要行舊鏈請 --engine v1）".to_string(),
                3,
            )
        })?;

    let unique = format!(
        "polyrust_check_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let td = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&td)
        .map_err(|e| ("tempdir".into(), format!("建臨時目錄失敗：{e}"), 3))?;
    let inp = td.join("input.rs");
    std::fs::copy(input_path, &inp)
        .map_err(|e| ("tempdir".into(), format!("copy input 失敗：{e}"), 3))?;

    let mut child = std::process::Command::new(&bin)
        .args(["rustc", "--", "input.rs", "--crate-type=rlib", "--edition=2021"])
        .current_dir(&td)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ("charon-spawn".into(), format!("啟動 {bin} 失敗：{e}"), 3))?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    let out = loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let mut stderr = String::new();
                let mut stdout = String::new();
                if let Some(mut h) = child.stderr.take() {
                    let _ = h.read_to_string(&mut stderr);
                }
                if let Some(mut h) = child.stdout.take() {
                    let _ = h.read_to_string(&mut stdout);
                }
                // try_wait 已 reaped；直接由呢個 status 攞 exit code（唔再 wait）
                let code = child.try_wait().ok().flatten().and_then(|st| st.code());
                break Ok((code, stdout, stderr));
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    break Err(("charon-timeout", "charon 120s 超時強殺".to_string(), 3));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => break Err(("charon-wait", format!("wait 失敗：{e}"), 3)),
        }
    };
    let (code, _stdout, stderr) = match out {
        Ok(v) => v,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&td);
            return Err((e.0.to_string(), e.1, e.2));
        }
    };

    if code != Some(0) {
        // rustc 真拒絕（E-code）→ **UNSAT verdict**（C2 口徑：編譯器拒絕 = 語義不可滿足，
        // 同 spike 口徑一致、同 CI 差分映射一致）；其余歸 charon 層錯（exit 3）。
        // miri/setup 類 warning 行喺前面——理由優先攞 E-code 行，冇先用第一行
        let diag_line = stderr
            .lines()
            .find(|l| l.contains("error["))
            .or_else(|| stderr.trim().lines().next())
            .unwrap_or("?");
        let msg = diag_line.trim().chars().take(200).collect::<String>();
        let _ = std::fs::remove_dir_all(&td);
        return Err(("rustc-reject".to_string(), msg, if has_ecode(&stderr) { 1 } else { 3 }));
    }

    let llbc = std::fs::read_dir(&td)
        .ok()
        .and_then(|mut rd| rd.find_map(|e| {
            let p = e.ok()?.path();
            (p.extension().and_then(|x| x.to_str()) == Some("llbc")).then_some(p)
        }))
        .ok_or_else(|| ("charon-output".to_string(), "charon 成功但冇 .llbc 產出".to_string(), 3))?;
    let r = polyrust_llbc::parse_llbc_file(&llbc)
        .map_err(|e| ("llbc-parse".to_string(), format!("{}：{e}", llbc.display()), 3));
    let _ = std::fs::remove_dir_all(&td);
    r
}

/// rustc 真拒絕偵測：stderr 含 `error[E1234]`（唔用 regex crate——frontend 細依賴原則）。
fn has_ecode(stderr: &str) -> bool {
    stderr.match_indices("error[").any(|(i, _)| {
        // 形態：error[E0123]——'E' 前綴 + 3..=5 位數字 + ']'
        let rest = stderr[i + 6..].strip_prefix('E').unwrap_or("");
        let n = rest.chars().take_while(char::is_ascii_digit).count();
        n >= 3 && n <= 5 && rest[n..].starts_with(']')
    })
}

fn which(name: &str) -> Option<String> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|d| {
            let p = d.join(name);
            p.is_file().then_some(p.to_string_lossy().into_owned())
        })
    })
}

// ────────────────────────── v1 路線（後備） ──────────────────────────

fn run_v1(input: &str, src: &str) -> ExitCode {
    let t0 = std::time::Instant::now();
    let poly = match polyrust_core::dsl::resolve(src, None) {
        Ok(p) => p,
        Err(e) => return finish(error_report(TOOL, input, "v1-dsl", &e.to_string()), 3),
    };
    match polyrust_core::pipeline::run_pipeline("llbc-check", &poly.source, false) {
        Ok(pr) if pr.is_unsat => {
            let rep = serde_json::json!({
                "api_version": "0.3",
                "tool": TOOL,
                "engine": "v1",
                "input": input,
                "status": "ok",
                "verdict": "Unsat",
                "checker_msg": pr.checker_msg,
                "ms": t0.elapsed().as_millis() as u64,
            });
            finish(rep, 1)
        }
        Ok(pr) => {
            let rep = serde_json::json!({
                "api_version": "0.3",
                "tool": TOOL,
                "engine": "v1",
                "input": input,
                "status": "ok",
                "verdict": "Sat",
                "checker_msg": pr.checker_msg,
                "ms": t0.elapsed().as_millis() as u64,
            });
            finish(rep, 0)
        }
        Err(e) => finish(error_report(TOOL, input, "v1-pipeline", &e), 3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecode_detection_matches_rustc_reject_convention() {
        assert!(has_ecode("error[E0308]: mismatched types"));
        assert!(has_ecode("  error[E0609]: no field"));
        assert!(!has_ecode("error: unexpected argument '--crate-type' found"));
        assert!(!has_ecode(""));
    }

    #[test]
    fn default_engine_switch_v4_for_rs_v1_for_poly() {
        let (_, e1, _) = parse_args(&["a.rs".to_string()]).unwrap();
        assert_eq!(e1, "v4");
        let (_, e2, _) = parse_args(&["a.llbc".to_string()]).unwrap();
        assert_eq!(e2, "v4");
        let (_, e3, _) = parse_args(&["a.poly".to_string()]).unwrap();
        assert_eq!(e3, "v1");
        let (_, e4, _) = parse_args(&["a.rs".to_string(), "--engine".to_string(), "v1".to_string()]).unwrap();
        assert_eq!(e4, "v1");
    }
}
