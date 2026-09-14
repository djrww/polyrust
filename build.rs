//! polyrust 的建置腳本：把 Lean 4 形式化（`lean/Polyrust`）編譯成靜態庫 `.a`，
//! 並在連結階段「編譯嵌入」到 polyrust 二進位檔內。
//!
//! 流程：
//!   1. 找 `lean/.lake/build/lib/libpolyrust_x2dformal_Polyrust.a`（由 lake 產出）。
//!   2. 找不到時，若系統有 `lake`（elan），就自動跑 `lake build Polyrust:static`。
//!   3. 找到 `.a` 且定位到 Lean 工具鏈後，把 `.a` 與 Lean 執行期靜態庫
//!      （libleancpp / libLean / libStd / libInit / libleanrt + libc++/libc++abi/
//!      libunwind/libgmp/libuv/libssl/libcrypto）一起以「靜態」方式連結進二進位檔，
//!      並設定 `cfg(has_lean_embed)`，讓 Rust 側可呼叫 Lean 初始化入口。
//!   4. 兩者皆缺時（例如無 Lean 的純 Rust CI / 交叉編譯平台），優雅跳過：
//!      仍可編譯 polyrust，只是不內嵌形式化庫（`cfg(has_lean_embed)` 未設定）。
//!
//! 說明：最終二進位只依賴 glibc（`ldd` 不含 lean/gmp/uv/ssl/crypto/libc++），
//! 其餘全部靜態嵌入，符合「排除執行期無需要的檔案、最優化執行期」的目標。

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// polyrust-formal 靜態庫路徑。
fn static_lib_path(manifest: &Path) -> PathBuf {
    manifest.join("lean/.lake/build/lib/libpolyrust_x2dformal_Polyrust.a")
}

/// 嘗試用 lake 建出靜態庫（需要 elan/lake 在 PATH 上）。
fn try_lake_build(manifest: &Path) -> bool {
    let lean_dir = manifest.join("lean");
    if !lean_dir.join("lakefile.toml").exists() {
        return false;
    }
    println!("cargo:warning=未找到 Lean 靜態庫，嘗試 `lake build Polyrust:static` …");
    let status = Command::new("lake")
        .args(["build", "Polyrust:static", "Polyrust:shared"])
        .current_dir(&lean_dir)
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

fn main() {
    // 告知 cargo 我們自訂的 cfg 名稱，消除 `unexpected cfg` 警告。
    println!("cargo::rustc-check-cfg=cfg(has_lean_embed)");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // 讓 lakefile / Lean 原始碼 / 靜態庫變動時觸發重編。
    println!("cargo:rerun-if-changed=lean/lakefile.toml");
    println!("cargo:rerun-if-changed=lean/lean-toolchain");
    println!("cargo:rerun-if-changed=lean/Polyrust");
    println!("cargo:rerun-if-changed=lean/Polyrust.lean");
    println!("cargo:rerun-if-changed=lean/.lake/build/lib");

    let lib = static_lib_path(&manifest);
    let lib_exists = lib.is_file();

    if !lib_exists && !try_lake_build(&manifest) {
        println!("cargo:warning=未內嵌 Lean 形式化庫（.a 不存在且無法建置）；polyrust 仍可運作");
        return;
    }

    if !lib.is_file() {
        println!("cargo:warning=Lean 靜態庫仍不存在，略過內嵌");
        return;
    }

    let Some(tc) = find_toolchain_prefix() else {
        println!("cargo:warning=找到 .a 但無法定位 Lean 工具鏈，略過內嵌");
        return;
    };

    let lib_lean = tc.join("lib/lean");
    let lib_root = tc.join("lib");

    // ── 搜尋路徑（順序無關） ──────────────────────────────────────────
    println!("cargo:rustc-link-search=native={}", lib_lean.display()); // libleancpp/libLean/libStd/libInit/libleanrt
    println!("cargo:rustc-link-search=native={}", lib_root.display()); // libc++/libc++abi/libunwind/libgmp/libuv/libssl/libcrypto

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

    // 啟用 Rust 側的內嵌入口。
    println!("cargo:rustc-cfg=has_lean_embed");
}
