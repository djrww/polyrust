// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
// 共用建置邏輯（由各 crate 的 `build.rs` 以 `include!` 引入；非獨立編譯單元）。
//
// 職責：定位倉庫根的 `lean/`、確保靜態庫 `.a` 存在（缺則 `lake build`），
// 並輸出把 Lean 形式化庫靜態嵌入的連結參數。
//
// 背景：`cargo:rustc-link-arg` 只對「發出它的 crate 自己的目標」生效，
// 不會傳播到下游依賴者。因此凡是要連結 `polyrust_core`（其 `formal`
// 模組引用 Lean 執行期符號）的**二進位** crate，其 `build.rs` 都要
// include 本檔並呼叫 `emit_lean_links()`——單一事實來源，避免邏輯漂移。
//
// 回傳 `true` 表示本次有嵌入（`.a` 與工具鏈皆就位）；`false` 表示降級
// （無 Lean 環境，仍可編譯，`cfg(has_lean_embed)` 由 core 自行決定不設定）。

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 從当前 crate 的 manifest 目錄向上找 `lean/lakefile.toml`（倉庫根）。
fn find_lean_dir() -> Option<PathBuf> {
    let mut d = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    for _ in 0..4 {
        if d.join("lean/lakefile.toml").exists() {
            return Some(d.join("lean"));
        }
        if !d.pop() {
            break;
        }
    }
    None
}

/// Lean 工具鏈根目錄（含 `lib/lean/*.a` 與 `lib/*.a`）。
fn find_toolchain_prefix() -> Option<PathBuf> {
    // 1) lake env 已設定的 LEAN_SYSROOT
    if let Ok(p) = env::var("LEAN_SYSROOT") {
        let p = PathBuf::from(p);
        if p.join("lib/lean").is_dir() {
            return Some(p);
        }
    }
    // 2) PATH 上的 `lean --print-prefix`
    if let Ok(out) = Command::new("lean").arg("--print-prefix").output() {
        if out.status.success() {
            let p = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
            if p.join("lib/lean").is_dir() {
                return Some(p);
            }
        }
    }
    // 3) elan 家目錄下的工具鏈（ELAN_HOME 或 ~/.elan）
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(h) = env::var("ELAN_HOME") {
        roots.push(PathBuf::from(h));
    }
    if let Ok(home) = env::var("HOME") {
        roots.push(PathBuf::from(home).join(".elan"));
    }
    for root in roots {
        let tc = root.join("toolchains");
        if let Ok(entries) = std::fs::read_dir(&tc) {
            for e in entries.flatten() {
                let p = e.path();
                if p.join("lib/lean").is_dir() {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// 嘗試用 lake 建出靜態庫（需要 elan/lake 在 PATH 上）。
fn try_lake_build(lean_dir: &Path) -> bool {
    println!("cargo:warning=未找到 Lean 靜態庫，嘗試 `lake build Polyrust:static` …");
    let status = Command::new("lake")
        .args(["build", "Polyrust:static", "Polyrust:shared"])
        .current_dir(lean_dir)
        .status();
    match status {
        Ok(s) if s.success() => true,
        Ok(s) => {
            println!("cargo:warning=lake build 失敗（exit {:?}），略過 Lean 內嵌", s.code());
            false
        }
        Err(_) => {
            println!("cargo:warning=找不到 lake（Lean 工具鏈未安裝），略過 Lean 內嵌");
            false
        }
    }
}

/// 輸出 Lean 靜態嵌入的連結參數。詳見檔首說明。
fn emit_lean_links() -> bool {
    let Some(lean_dir) = find_lean_dir() else {
        return false;
    };

    // 讓 lakefile / Lean 原始碼 / 靜態庫變動時觸發重編。
    println!("cargo:rerun-if-changed={}", lean_dir.join("lakefile.toml").display());
    println!("cargo:rerun-if-changed={}", lean_dir.join("lean-toolchain").display());
    println!("cargo:rerun-if-changed={}", lean_dir.join("Polyrust").display());
    println!("cargo:rerun-if-changed={}", lean_dir.join("Polyrust.lean").display());
    println!("cargo:rerun-if-changed={}", lean_dir.join(".lake/build/lib").display());

    let lib = lean_dir.join(".lake/build/lib/libpolyrust_x2dformal_Polyrust.a");
    if !lib.is_file() && !try_lake_build(&lean_dir) {
        println!("cargo:warning=未內嵌 Lean 形式化庫（.a 不存在且無法建置）；仍可編譯運作");
        return false;
    }
    if !lib.is_file() {
        println!("cargo:warning=Lean 靜態庫仍不存在，略過內嵌");
        return false;
    }
    let Some(tc) = find_toolchain_prefix() else {
        println!("cargo:warning=找到 .a 但無法定位 Lean 工具鏈，略過內嵌");
        return false;
    };

    let lib_lean = tc.join("lib/lean");
    let lib_root = tc.join("lib");

    // ── 搜尋路徑（順序無關） ──────────────────────────────────────────
    println!("cargo:rustc-link-search=native={}", lib_lean.display());
    println!("cargo:rustc-link-search=native={}", lib_root.display());

    // ── 連結參數（順序重要，全部用 link-arg 保證順序） ─────────────────
    // 1) 形式化庫（以絕對路徑指定 .a，避免 lake 同時產出的 .so 被優先挑中）
    //    + Lean 執行期（成組以解循環依賴）
    println!("cargo:rustc-link-arg=-Wl,--start-group");
    println!("cargo:rustc-link-arg={}", lib.display());
    println!("cargo:rustc-link-arg=-Wl,-lleancpp");
    println!("cargo:rustc-link-arg=-Wl,-lLean");
    println!("cargo:rustc-link-arg=-Wl,-lStd");
    println!("cargo:rustc-link-arg=-Wl,-lInit");
    println!("cargo:rustc-link-arg=-Wl,-lleanrt");
    println!("cargo:rustc-link-arg=-Wl,--end-group");
    // 2) C++ 執行期與 unwinder 必須「靜態」連結（工具鏈同時有 .a 與 .so）
    println!("cargo:rustc-link-arg=-Wl,-Bstatic");
    println!("cargo:rustc-link-arg=-Wl,-lc++");
    println!("cargo:rustc-link-arg=-Wl,-lc++abi");
    println!("cargo:rustc-link-arg=-Wl,-lunwind");
    println!("cargo:rustc-link-arg=-Wl,-Bdynamic");
    // 3) 其餘執行期（僅 .a 存在，自然靜態）+ glibc 系統庫
    println!("cargo:rustc-link-arg=-Wl,-lgmp");
    println!("cargo:rustc-link-arg=-Wl,-luv");
    println!("cargo:rustc-link-arg=-Wl,-lssl");
    println!("cargo:rustc-link-arg=-Wl,-lcrypto");
    println!("cargo:rustc-link-arg=-Wl,-lpthread");
    println!("cargo:rustc-link-arg=-Wl,-ldl");
    println!("cargo:rustc-link-arg=-Wl,-lrt");
    println!("cargo:rustc-link-arg=-Wl,-lm");
    // 4) 去未使用區段，縮小體積（與最優化執行期目標一致）
    println!("cargo:rustc-link-arg=-Wl,--gc-sections");

    true
}
