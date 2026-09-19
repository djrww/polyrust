// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! polyrust-full 的建置腳本：
//! 本二進位連結 `polyrust_core`，其 `formal` 模組引用 Lean 執行期符號；
//! `cargo:rustc-link-arg` 不跨 crate 傳播，故在此重新發出同一組連結參數
//! （邏輯集中在 `scripts/build_lean_embed.rs`，與 core 同源）。

include!("../../scripts/build_lean_embed.rs");

fn main() {
    let _ = emit_lean_links();
}
