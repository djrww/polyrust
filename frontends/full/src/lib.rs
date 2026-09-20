//! polyrust-full 库 — 供 syn_100_scan 等 bin 复用
pub mod ir;
pub mod encoding;
pub mod api;
pub mod ty;
pub mod lower;
#[cfg(feature = "syn")]
pub mod syn_lower;
#[cfg(feature = "syn")]
pub mod syn_bridge;
#[cfg(feature = "syn")]
pub mod syn_explain;
#[cfg(feature = "syn")]
pub mod syn_visit;
#[cfg(feature = "syn")]
pub mod syn_100;
pub mod oracle;
