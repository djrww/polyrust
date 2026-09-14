//! 內嵌的 Lean 4 形式化（`lean/Polyrust`）之執行期橋接。
//!
//! 建置時（`build.rs`）把 `lean/.lake/build/lib/libpolyrust_x2dformal_Polyrust.a`
//! 靜態連結進本二進位檔，並設定 `cfg(has_lean_embed)`。此模組在該 cfg 開啟時
//! 對外提供 Lean 執行期的初始化入口；未內嵌時退化成安全的 no-op。

// Lean 執行期初始化入口（libleanrt，C ABI）。
#[cfg(has_lean_embed)]
extern "C" {
    // 初始化 Lean 執行期（task manager 等）。未在公開標頭檔，故手動宣告。
    fn lean_initialize_runtime_module();
    // 標記初始化階段結束（凍結持久物件）。
    fn lean_io_mark_end_initialization();
    // polyrust-formal 全模組初始化：註冊 `Polyrust.*` 的每一條定理宣告。
    fn initialize_polyrust_x2dformal_Polyrust();
}

/// 是否在編譯期內嵌了 Lean 形式化庫。
#[inline]
pub const fn embedded() -> bool {
    cfg!(has_lean_embed)
}

/// 初始化內嵌的 Lean 執行期並載入 polyrust-formal 全部定理。
///
/// 未內嵌時為 no-op。回傳 `Ok(())` 表示成功載入；本程式目前不會在載入後
/// 呼叫任何具體定理的計算入口（那些是宣告，非可執行函式），故僅做載入自檢。
#[cfg(has_lean_embed)]
pub fn init() -> Result<(), &'static str> {
    // 注意：這些 C 入口不回傳錯誤碼；若執行期初始化失敗會在內部 panic/abort。
    // 這裡的回傳值僅表達「已觸發載入」而非「載入結果」。
    unsafe {
        lean_initialize_runtime_module();
        initialize_polyrust_x2dformal_Polyrust();
        lean_io_mark_end_initialization();
    }
    Ok(())
}

#[cfg(not(has_lean_embed))]
pub fn init() -> Result<(), &'static str> {
    Err("未內嵌 Lean 形式化庫（建置時未偵測到 .a）")
}

/// 啟動時初始化內嵌庫，並回傳一行狀態供報告顯示。
pub fn startup_report() -> String {
    if !embedded() {
        return "Lean 4 形式化庫未內嵌（無 Lean 工具鏈，純 Rust 建置）".to_string();
    }
    match init() {
        Ok(()) => "Lean 4 形式化庫（Polyrust.*，14 模組）已靜態嵌入並載入 ✓".to_string(),
        Err(e) => format!("Lean 內嵌載入失敗：{}", e),
    }
}
