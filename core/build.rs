// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! polyrust-core 的建置腳本：把 Lean 4 形式化（`lean/Polyrust`）編譯成靜態庫
//! `.a`，並在連結階段「編譯嵌入」到凡依賴本 crate 的二進位檔內。
//!
//! 實際的「定位 .a／lake build／輸出連結參數」邏輯集中在
//! `scripts/build_lean_embed.rs`（凡要連結本核心的二進位都 include 它——
//! 因為 `cargo:rustc-link-arg` 不會跨 crate 傳播）。本檔只額外負責：
//! 在成功嵌入時為 **core** 設定 `cfg(has_lean_embed)`，啟用 `formal.rs`
//! 的 Lean 執行期入口。

include!("../scripts/build_lean_embed.rs");

fn main() {
    // 告知 cargo 我們自訂的 cfg 名稱，消除 `unexpected cfg` 警告。
    println!("cargo::rustc-check-cfg=cfg(has_lean_embed)");

    let embedded = emit_lean_links();
    if embedded {
        // 啟用 Rust 側（core::formal）的內嵌入口。
        println!("cargo:rustc-cfg=has_lean_embed");
    }
}
