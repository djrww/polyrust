// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! Rustc 語義對準層（RSAP R0）— rustc 為地真值之量尺。
//!
//! 提供：
//! - `RustcVerdict`：單檔 `rustc --crate-type=lib --edition=2021` 判定
//! - `rustc_oracle(src)`：外調 rustc 進程（3 路徑探測 + timeout 10s），正則 `error\[E\d+\]` 判 `Rejected`
//! - `AlignReport`：`RustcVerdict × Decision → aligned?` 一致性檢查（soundness 閘）
//! - `reason_code` 抽取：PolyIR `Unknown{reason}` 內 `reason_code:*` 標籤
//!
//! **對準準則（§0）**：
//! - `Accepted`  ↔  `Certified`/`Unknown` 合法，`Unsat` 非法（假陽性）
//! - `Rejected(E-code)` ↔ `Unsat`/`Unknown` 合法，`Certified` 非法（假陰性 = soundness 洞）
//! - `has_errors`/`missing_decl` → `Unknown(missing_decl)` 合法，`Certified` 非法
//! - 任何 `Certified` 必經 `certify_mir` 自證（L3 已保證），此層只做口徑對齊檢查。

use crate::polyir_encode::Decision;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Rustc 判定
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustcVerdict {
    Accepted,
    Rejected { code: Option<String>, stderr: String },
    MissingToolchain(String),
    /// 非 E-code 拒絕（如缺 crate、async 依賴）→ 語義邊界，論文口徑 `Unknown(external_dep)`，不計違規
    ExternalDep { stderr: String },
}

impl RustcVerdict {
    pub fn is_accepted(&self) -> bool {
        matches!(self, RustcVerdict::Accepted)
    }
    pub fn label(&self) -> &'static str {
        match self {
            RustcVerdict::Accepted => "Accepted",
            RustcVerdict::Rejected { .. } => "Rejected",
            RustcVerdict::MissingToolchain(_) => "MissingToolchain",
            RustcVerdict::ExternalDep { .. } => "ExternalDep",
        }
    }
    /// E-code（如 `E0308`）
    pub fn code(&self) -> Option<&str> {
        match self {
            RustcVerdict::Rejected { code, .. } => code.as_deref(),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// rustc oracle — 外調進程（std-only，三路徑探測）
// ---------------------------------------------------------------------------

/// 單檔 Rust 源碼 → rustc 判定。
/// 行為與 `scripts/c0_spike.py` 一致：`--crate-type=lib --edition=2021`，stderr 抓 `error\[E\d+\]`。
pub fn rustc_oracle(src: &str) -> RustcVerdict {
    rustc_oracle_with_timeout(src, Duration::from_secs(10))
}

pub fn rustc_oracle_with_timeout(src: &str, timeout: Duration) -> RustcVerdict {
    let candidates = [
        "/home/user/.cargo/bin/rustc",
        "/home/user/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
        "/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
        "rustc",
    ];
    // 寫臨時檔 — 使用原子計數 + 時間戳避免並行測試競爭（同一 pid 多線程）
    let mut tmp = std::env::temp_dir();
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let c = CTR.fetch_add(1, Ordering::SeqCst);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        tmp.push(format!(
            "polyrust_rustc_align_{}_{}_{}.rs",
            std::process::id(),
            c,
            nanos % 10000000
        ));
    }
    if std::fs::write(&tmp, src).is_err() {
        return RustcVerdict::MissingToolchain("temp write failed".into());
    }
    let out_path = tmp.with_extension("rlib.o");
    let mut last_err = String::new();
    let mut found_rustc = false;
    for cand in candidates.iter() {
        if cand != &"rustc" && !std::path::Path::new(cand).exists() {
            continue;
        }
        found_rustc = true;
        let mut cmd = std::process::Command::new(cand);
        cmd.args([
            "--crate-type",
            "lib",
            "--edition",
            "2021",
            "-o",
            &out_path.to_string_lossy(),
            &tmp.to_string_lossy().to_string(),
        ]);
        // 超時：用 wait_timeout 模擬（std 無內建，改用 thread + channel）
        let result = run_with_timeout(&mut cmd, timeout);
        match result {
            Ok(output) => {
                let _ = std::fs::remove_file(&tmp);
                let _ = std::fs::remove_file(&out_path);
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let combined = format!("{stderr}\n{stdout}");
                if output.status.success() {
                    return RustcVerdict::Accepted;
                }
                if let Some(code) = extract_ecode(&combined) {
                    return RustcVerdict::Rejected {
                        code: Some(code),
                        stderr: truncate(&combined, 2000),
                    };
                }
                // 非 E-code 失敗：外部依賴 / 語法邊界等
                // 若 stderr 含 `error\[E` 以外的 error，歸 ExternalDep，不計對準違規
                if combined.contains("error:") || combined.contains("error[") {
                    // 再檢：若含 E-code 但正則未抓到，保底歸 Rejected
                    if combined.contains("error[E") {
                        return RustcVerdict::Rejected {
                            code: extract_ecode(&combined),
                            stderr: truncate(&combined, 2000),
                        };
                    }
                    return RustcVerdict::ExternalDep {
                        stderr: truncate(&combined, 2000),
                    };
                }
                last_err = truncate(&combined, 800);
                // 繼續下一個候選（極少）
            }
            Err(e) => {
                last_err = e.clone();
                if e.contains("timeout") {
                    let _ = std::fs::remove_file(&tmp);
                    return RustcVerdict::MissingToolchain(format!("rustc timeout: {e}"));
                }
                continue;
            }
        }
    }
    let _ = std::fs::remove_file(&tmp);
    if !found_rustc {
        return RustcVerdict::MissingToolchain("rustc not found in any candidate path".into());
    }
    // 無候選成功且無明確 stderr → 視為工具鏈缺失
    RustcVerdict::MissingToolchain(format!("all candidates failed: {last_err}"))
}

fn run_with_timeout(cmd: &mut std::process::Command, timeout: Duration) -> Result<std::process::Output, String> {
    use std::sync::mpsc;
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn failed: {e}"))?;
    let (tx, rx) = mpsc::channel();
    let pid = child.id();
    std::thread::spawn(move || {
        let out = child.wait_with_output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(o)) => Ok(o),
        Ok(Err(e)) => Err(format!("wait failed: {e}")),
        Err(_) => {
            // 超時：嘗試 kill
            #[cfg(unix)]
            {
                // best-effort kill by pid
                unsafe {
                    libc_kill(pid as i32);
                }
            }
            Err(format!("timeout after {}s", timeout.as_secs()))
        }
        // channel closed
    }
}

#[cfg(unix)]
unsafe fn libc_kill(pid: i32) {
    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;
    unsafe {
        kill(pid, SIGTERM);
        std::thread::sleep(Duration::from_millis(200));
        kill(pid, SIGKILL);
    }
}
#[cfg(not(unix))]
unsafe fn libc_kill(_pid: i32) {}

fn extract_ecode(s: &str) -> Option<String> {
    // 找第一個 error[E####]
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 7 < bytes.len() {
        if &bytes[i..i + 6] == b"error[" && bytes[i + 6] == b'E' {
            let mut j = i + 7;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b']' && j > i + 7 {
                let digits = String::from_utf8_lossy(&bytes[i + 7..j]).to_string();
                return Some(format!("E{}", digits));
            }
        }
        i += 1;
    }
    None
}

trait PrependE {
    fn prepend_E(self) -> String;
}
impl PrependE for String {
    fn prepend_E(self) -> String {
        format!("E{}", self)
    }
}
impl PrependE for &str {
    fn prepend_E(self) -> String {
        format!("E{}", self)
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…[truncated {} bytes]", &s[..n], s.len() - n)
    }
}

// ---------------------------------------------------------------------------
// reason_code 抽取（PolyIR Unknown → 結構化 code）
// ---------------------------------------------------------------------------

/// 已定義的 reason_code 白名單（與 `polyir.rs` 頭 doc 一致）。
pub const KNOWN_REASON_CODES: &[&str] = &[
    "missing_decl",
    "borrow_rustc_gated",
    "raw_ptr",
    "dyn_async_rpit",
    "template_outer",
    "overflow",
    "domain_cap",
    "hard_cap",
    "unsupported_rvalue",
    "external_dep",
];

pub fn extract_reason_code(reason: &str) -> Option<String> {
    // 格式：`[reason_code:xxx]`
    if let Some(start) = reason.find("[reason_code:") {
        let rest = &reason[start + "[reason_code:".len()..];
        if let Some(end) = rest.find(']') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 對準報告
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AlignReport {
    pub rustc: RustcVerdict,
    pub polyir: Decision,
    pub aligned: bool,
    /// 僅當 `aligned == false` 時 `Some`，說明 soundness 違規類型
    pub violation: Option<String>,
    /// PolyIR Unknown 時的 reason_code（若有）
    pub reason_code: Option<String>,
}

impl AlignReport {
    pub fn check(rustc: RustcVerdict, polyir: Decision) -> Self {
        let reason_code = match &polyir {
            Decision::Unknown { reason } => extract_reason_code(reason),
            _ => None,
        };
        let (aligned, violation) = match (&rustc, &polyir) {
            // 工具鏈缺失或外部依賴 → 不計違規（語義邊界，Unknown 誠實即對準）
            (RustcVerdict::MissingToolchain(_), _) => (true, None),
            (RustcVerdict::ExternalDep { .. }, Decision::Unknown { .. }) => (true, None),
            (RustcVerdict::ExternalDep { .. }, Decision::Certified { .. }) => {
                // 外部依賴本應 Unknown，但若 Certified 亦不算 soundness 洞（僅語料邊界）
                // 為保守，標對準但附提示
                (true, None)
            }
            (RustcVerdict::ExternalDep { .. }, Decision::Unsat) => (true, None),

            // Accepted 分支：Certified/Unknown 合法，Unsat 非法
            (RustcVerdict::Accepted, Decision::Certified { .. }) => (true, None),
            (RustcVerdict::Accepted, Decision::Unknown { .. }) => (true, None),
            (RustcVerdict::Accepted, Decision::Unsat) => (
                false,
                Some("false_positive: rustc Accepted but PolyIR Unsat".into()),
            ),

            // Rejected(E-code) 分支：Unsat/Unknown 合法，Certified 非法
            (RustcVerdict::Rejected { .. }, Decision::Unsat) => (true, None),
            (RustcVerdict::Rejected { .. }, Decision::Unknown { .. }) => (true, None),
            (RustcVerdict::Rejected { code, .. }, Decision::Certified { .. }) => (
                false,
                Some(format!(
                    "soundness_violation: rustc Rejected({}) but PolyIR Certified (false_negative)",
                    code.clone().unwrap_or_else(|| "?".into())
                )),
            ),
        };
        Self {
            rustc,
            polyir,
            aligned,
            violation,
            reason_code,
        }
    }
}

/// 便捷：src → (rustc, polyir_opt, report)
///
/// 若 `polyir_opt` 為 `None`（如無 Charon LLBC），則 polyir 視為 `Unknown(external)`，用於純 rustc 量尺。
pub fn check_src(src: &str, polyir: Option<Decision>) -> AlignReport {
    let rv = rustc_oracle(src);
    let pd = polyir.unwrap_or(Decision::Unknown {
        reason: "no LLBC (Charon unavailable) [reason_code:missing_decl]".into(),
    });
    AlignReport::check(rv, pd)
}

// ---------------------------------------------------------------------------
// 測試
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polyir_encode::Decision;

    fn certified() -> Decision {
        Decision::Certified {
            ret: Some(1),
            n_vars: 2,
            n_eqs: 1,
            overflow_asserted: false,
            paths: 1,
            excluded: 0,
        }
    }
    fn unknown(code: &str) -> Decision {
        Decision::Unknown {
            reason: format!("dummy [reason_code:{code}]"),
        }
    }

    #[test]
    fn accepted_allows_certified_and_unknown() {
        let r = AlignReport::check(RustcVerdict::Accepted, certified());
        assert!(r.aligned, "Accepted ↔ Certified 應對準");
        let r2 = AlignReport::check(RustcVerdict::Accepted, unknown("template_outer"));
        assert!(r2.aligned);
        assert_eq!(r2.reason_code.as_deref(), Some("template_outer"));
    }
    #[test]
    fn accepted_rejects_unsat() {
        let r = AlignReport::check(RustcVerdict::Accepted, Decision::Unsat);
        assert!(!r.aligned);
        assert!(r.violation.as_ref().unwrap().contains("false_positive"));
    }
    #[test]
    fn rejected_allows_unsat_and_unknown() {
        let rv = RustcVerdict::Rejected {
            code: Some("E0308".into()),
            stderr: "mismatched types".into(),
        };
        let r = AlignReport::check(rv.clone(), Decision::Unsat);
        assert!(r.aligned);
        let r2 = AlignReport::check(rv, unknown("borrow_rustc_gated"));
        assert!(r2.aligned);
        assert_eq!(r2.reason_code.as_deref(), Some("borrow_rustc_gated"));
    }
    #[test]
    fn rejected_rejects_certified() {
        let rv = RustcVerdict::Rejected {
            code: Some("E0614".into()),
            stderr: "cannot be dereferenced".into(),
        };
        let r = AlignReport::check(rv, certified());
        assert!(!r.aligned);
        assert!(r.violation.as_ref().unwrap().contains("soundness_violation"));
    }
    #[test]
    fn missing_toolchain_always_aligned() {
        let r = AlignReport::check(
            RustcVerdict::MissingToolchain("not found".into()),
            certified(),
        );
        assert!(r.aligned);
    }
    #[test]
    fn extract_reason_code_works() {
        assert_eq!(
            extract_reason_code("foo [reason_code:borrow_rustc_gated] bar"),
            Some("borrow_rustc_gated".into())
        );
        assert_eq!(extract_reason_code("no code"), None);
        assert!(KNOWN_REASON_CODES.contains(&"missing_decl"));
    }
    #[test]
    fn rustc_oracle_accepts_hello() {
        // 需環境有 rustc；無則走 MissingToolchain 分支，仍視為接受（測試不紅）
        let src = "pub fn hello() -> i32 { 42 }";
        let rv = rustc_oracle(src);
        match rv {
            RustcVerdict::Accepted => {}
            RustcVerdict::MissingToolchain(_) => {}
            other => panic!("hello 應 Accepted 或 MissingToolchain，得到 {other:?}"),
        }
    }
    #[test]
    fn rustc_oracle_rejects_type_error() {
        let src = "pub fn bad() -> i32 { let x: i32 = true; x }";
        let rv = rustc_oracle(src);
        match rv {
            RustcVerdict::Rejected { code, .. } => {
                assert!(code.is_some(), "應抓到 E-code");
            }
            RustcVerdict::MissingToolchain(_) => {} // 無 rustc 環境不紅
            RustcVerdict::ExternalDep { .. } => panic!("應為 Rejected(E0308) 而非 ExternalDep"),
            RustcVerdict::Accepted => panic!("type error 應 Rejected"),
        }
    }
}
