// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! polyrust-full —— 全特性前端
//! 支援 struct/enum/impl/trait、Vec/String/HashMap、loop/match、mod、async、I/O、unsafe、lifetime
//!
//! 端點:
//! GET  /              → 內嵌 UI（9 特性分頁）
//! GET  /health
//! POST /api/v2/check  → v0.2 驗證（JSON {source}）
//! POST /api/v2/lower  → 僅降維
//! POST /api/check     → 兼容 v0.1 core API
//! POST /api/expand    → 兼容 v0.1

mod ir;
mod encoding;
mod api;
mod ty;
mod lower;
#[cfg(feature = "syn")]
mod syn_lower;
#[cfg(feature = "syn")]
mod syn_bridge;
#[cfg(feature = "syn")]
mod syn_explain;
#[cfg(feature = "syn")]
mod syn_visit;
#[cfg(feature = "syn")]
mod syn_100;
mod oracle;

use axum::{routing::{get, post}, Router, Json, http::StatusCode, response::Html};
use polyrust_core::{formal};
use serde_json::Value;
#[cfg(feature = "cors")]
use tower_http::cors::CorsLayer;


#[cfg(feature = "syn")]
async fn syn_explain_handler(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    let src = payload.get("source").and_then(|v| v.as_str()).unwrap_or("");
    let result = syn_explain::explain_wrp_r2_all_for_src(src);
    (StatusCode::OK, Json(serde_json::json!({
        "ok": true,
        "explain": result,
        "syn_used": true
    })))
}
#[cfg(not(feature = "syn"))]
async fn syn_explain_handler(Json(_payload): Json<Value>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(serde_json::json!({"ok": false, "error": "syn feature not enabled"})))
}

#[cfg(feature = "syn")]
async fn syn_visit_handler(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    let src = payload.get("source").and_then(|v| v.as_str()).unwrap_or("");
    let file_pure = src.contains("# @pure") || src.contains("#[pure]");
    let report = syn_visit::analyze_with_visit(src, file_pure).map(|r| format!("{:#?}", r)).unwrap_or_else(|e| e);
    let decide = syn_visit::visit_decide(src).map(|b| if b { "UNSAT".to_string() } else { "SAT".to_string() }).unwrap_or_else(|e| e);
    let align = syn_visit::visit_align_100();
    let demo = syn_visit::demo_proc_macro2_visit(src).unwrap_or_else(|e| e);
    (StatusCode::OK, Json(serde_json::json!({
        "ok": true,
        "decide": decide,
        "report": report,
        "align": format!("{}/{}", align.0, align.1),
        "demo": demo,
        "syn_used": true,
        "proc_macro2_used": true
    })))
}
#[cfg(not(feature = "syn"))]
async fn syn_visit_handler(Json(_payload): Json<Value>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(serde_json::json!({"ok": false, "error": "syn feature not enabled"})))
}

#[cfg(feature = "syn")]
async fn syn_100_handler(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    let src = payload.get("source").and_then(|v| v.as_str()).unwrap_or("");
    if src.trim().is_empty() {
        let once = syn_100::continuous_scan_once();
        return (StatusCode::OK, Json(serde_json::json!({"ok": true, "scan": once, "syn_full": true, "visit": true, "proc_macro2": true})));
    }
    let gt = syn_100::grab_groundtruth(src).map(|g| format!("{:#?}", g)).unwrap_or_else(|e| e);
    (StatusCode::OK, Json(serde_json::json!({"ok": true, "groundtruth": gt})))
}
#[cfg(not(feature = "syn"))]
async fn syn_100_handler(Json(_payload): Json<Value>) -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(serde_json::json!({"ok": false, "error": "syn feature not enabled"})))
}

async fn health() -> Json<Value> {
    Json(serde_json::json!({
        "ok": true,
        "crate": "polyrust-full",
        "version": "0.2.0-proto",
        "core": "polyrust-core",
        "lean_embedded": formal::embedded(),
        "features": ["struct","enum","impl","trait","Vec","String","HashMap","loop","match","mod","async","io","unsafe","lifetime"],
        "api": ["/api/v2/check","/api/v2/lower","/api/check","/api/expand","/health"]
    }))
}

// 兼容 v0.1 的 /api/check
async fn check_v1(body: String) -> (StatusCode, Json<Value>) {
    let (j, _ok) = polyrust_core::driver::check_text_json("web", &body, None);
    let v: Value = serde_json::from_str(&j.to_string()).unwrap_or_else(|_| serde_json::json!({"status":"error"}));
    (StatusCode::OK, Json(v))
}

async fn expand_v1(body: String) -> (StatusCode, Json<Value>) {
    let (j, _ok) = polyrust_core::driver::expand_text_json("web", &body, None);
    let v: Value = serde_json::from_str(&j.to_string()).unwrap_or_else(|_| serde_json::json!({"status":"error"}));
    (StatusCode::OK, Json(v))
}

const PAGE: &str = r##"<!DOCTYPE html>
<html lang="zh-Hant">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>polyrust-full v0.2 — 全 Rust 特性驗證</title>
<style>
:root{--bg:#0e1117;--panel:#161b26;--panel2:#1c2333;--border:#2a3348;--fg:#d7deea;--dim:#8591a8;--accent:#4ea1ff;--ok:#34d399;--bad:#f87171;--warn:#fbbf24;--mono:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
*{box-sizing:border-box}body{margin:0;font-family:system-ui,-apple-system,Segoe UI,sans-serif;background:var(--bg);color:var(--fg);line-height:1.5}
header{padding:16px 20px;border-bottom:1px solid var(--border);background:var(--panel);display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:10px}
header h1{margin:0;font-size:18px;font-weight:700}header p{margin:4px 0 0;color:var(--dim);font-size:12px}
.badge{display:inline-block;padding:2px 8px;border-radius:999px;font-size:11px;font-weight:700;border:1px solid var(--border)}
.b-sat{background:rgba(52,211,153,.15);color:var(--ok);border-color:var(--ok)}.b-unsat{background:rgba(248,113,113,.15);color:var(--bad);border-color:var(--bad)}.b-unk{background:rgba(251,191,36,.15);color:var(--warn);border-color:var(--warn)}
.wrap{max-width:1280px;margin:0 auto;padding:16px;display:grid;grid-template-columns:280px 1fr 380px;gap:16px}
@media(max-width:1100px){.wrap{grid-template-columns:1fr 1fr}.wrap .right{grid-column:1 / -1}}
@media(max-width:700px){.wrap{grid-template-columns:1fr}}
.card{background:var(--panel);border:1px solid var(--border);border-radius:10px;overflow:hidden}
.card h2{margin:0;padding:10px 14px;font-size:13px;font-weight:650;border-bottom:1px solid var(--border);color:var(--dim);text-transform:uppercase;letter-spacing:.04em}
.list{padding:8px;display:flex;flex-direction:column;gap:6px;max-height:600px;overflow-y:auto}
.feat{padding:10px 12px;border-radius:8px;background:var(--panel2);border:1px solid transparent;cursor:pointer}
.feat:hover{border-color:var(--accent)}.feat.active{border-color:var(--accent);background:#1e2a44}
.feat .t{font-weight:600;font-size:13px}.feat .d{font-size:11px;color:var(--dim);margin-top:2px}
.feat .tag{font-family:var(--mono);font-size:10px;color:var(--accent)}
textarea{width:100%;height:360px;background:var(--panel2);color:var(--fg);border:none;padding:12px;font-family:var(--mono);font-size:12.5px;resize:vertical;outline:none}
.btns{display:flex;gap:8px;padding:10px 12px;border-top:1px solid var(--border);flex-wrap:wrap}
button{padding:8px 12px;border:1px solid var(--border);border-radius:7px;background:var(--panel2);color:var(--fg);font-size:13px;font-weight:600;cursor:pointer}
button.primary{background:var(--accent);border-color:var(--accent);color:#06121f}button:hover{filter:brightness(1.15)}button:disabled{opacity:.5}
.out{padding:12px;font-size:12.5px;min-height:200px}.kv{display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:6px;margin:8px 0}
.kv div{background:var(--panel2);border:1px solid var(--border);border-radius:6px;padding:5px 8px}.kv .k{color:var(--dim);font-size:10px}.kv .v{font-family:var(--mono);font-size:13px}
pre{background:var(--panel2);border:1px solid var(--border);border-radius:6px;padding:10px;overflow-x:auto;font-family:var(--mono);font-size:11.5px;white-space:pre-wrap}
h3{font-size:12px;color:var(--dim);margin:12px 0 6px}
a{color:var(--accent);text-decoration:none}
.tabs{display:flex;gap:6px;padding:8px 12px;border-bottom:1px solid var(--border)}
.tab{padding:4px 10px;border-radius:6px;font-size:12px;cursor:pointer;border:1px solid transparent}
.tab.active{background:var(--panel2);border-color:var(--border);color:var(--fg)}
.mono{font-family:var(--mono)}
</style>
</head>
<body>
<header>
<div><h1>polyrust-full v0.2 · 全特性代數驗證</h1><p>CDCL × Buchberger × QAP — 支援 struct/enum/impl/trait、Vec/String/HashMap、loop/match、mod、async、I/O、unsafe、lifetime</p></div>
<div><span class="badge">core 零依賴</span> <span class="badge">frontends/full axum</span> <a href="/health" target="_blank" class="badge">health</a></div>
</header>
<div class="wrap">
<div class="card left">
<h2>9 特性家族（點選載入示例）</h2>
<div class="list" id="featList"></div>
<h2 style="border-top:1px solid var(--border)">文檔</h2>
<div style="padding:10px 12px;font-size:12px;color:var(--dim)">
<div>· <a href="https://github.com/djrww/polyrust" target="_blank">原倉庫 v0.1.5</a></div>
<div>· 擴展方案見 <span class="mono">docs/EXTENSION_PLAN.md</span></div>
<div>· API: <span class="mono">POST /api/v2/check {source}</span></div>
<div>· 兼容 v0.1: <span class="mono">/api/check</span></div>
</div>
</div>
<div class="card center">
<h2>輸入 · .poly v0.2</h2>
<div class="tabs"><div class="tab active" data-tab="src">源碼</div><div class="tab" data-tab="lowered">降維後 (core)</div><div class="tab" data-tab="encoding">編碼說明</div></div>
<textarea id="src" spellcheck="false"></textarea>
<pre id="loweredView" style="display:none;min-height:360px;margin:0;border:none;border-radius:0"></pre>
<pre id="encodingView" style="display:none;min-height:360px;margin:0;border:none;border-radius:0"></pre>
<div class="btns">
<button class="primary" onclick="runCheck()">驗證 v0.2</button>
<button onclick="runLower()">僅降維</button>
<button onclick="runCheckV1()">兼容 v0.1 驗證</button>
<button style="background:var(--warn);color:#000;border-color:var(--warn)" onclick="runFixOracle()">🔧 Oracle 差異一鍵修復</button>
</div>
</div>
<div class="card right">
<h2>結果</h2>
<div class="out" id="out"><span style="color:var(--dim)">尚未執行。選擇左側特性或直接編輯源碼後點「驗證 v0.2」。</span></div>
</div>
</div>
<script>
const FEATURES = [
{ id:"struct", name:"struct / enum / impl / trait", tag:"積型/和型/方法表", desc:"struct Point, enum Option, impl, trait Display", code:[
"# @intent: struct + enum + impl + trait 示例",
"struct Point { x: i32, y: i32 }",
"enum Option<T> { Some(T), None }",
"trait Display { fn fmt(&self) -> String; }",
"impl Point {",
"  fn new(x: i32, y: i32) -> Point { Point { x: x, y: y } }",
"  fn norm(&self) -> i32 { self.x * self.x + self.y * self.y }",
"}",
"impl Display for Point {",
"  fn fmt(&self) -> String { String::from(\"Point\") }",
"}",
"fn main() {",
"  let p = Point { x: 3, y: 4 };",
"  let n = p.norm();",
"}"
].join("\n") },
{ id:"vec", name:"Vec / String / HashMap", tag:"泛型容器", desc:"Vec<T>, String, HashMap<K,V> 泛型統一", code:[
"# @intent: Vec + String + HashMap 泛型容器",
"# @import: vec",
"fn main() {",
"  let v: Vec<i32> = Vec::new();",
"  v.push(1);",
"  v.push(2);",
"  let s: String = String::from(\"hello\");",
"  let m: HashMap<String,i32> = HashMap::new();",
"  m.insert(s, 42);",
"  let len = v.len();",
"}"
].join("\n") },
{ id:"loop", name:"loop / while / for", tag:"有界+不變量", desc:"loop, while, for, @invariant, @fuel", code:[
"# @intent: 迴圈家族 — 有界模型檢測 + 不變量",
"# @fuel 5",
"# @invariant: x >= 0",
"fn main() {",
"  let x = 0;",
"  let mut y = 0;",
"  while x < 10 {",
"    y = y + x;",
"    x = x + 1;",
"  }",
"  loop {",
"    if y > 100 { break; }",
"    y = y + 1;",
"  }",
"  for i in 0..5 {",
"    y = y + i;",
"  }",
"}"
].join("\n") },
{ id:"match", name:"match", tag:"決策樹/窮舉", desc:"match enum, pattern, _ 通配", code:[
"# @intent: match 決策樹編譯",
"enum Option<T> { Some(T), None }",
"enum Result<T,E> { Ok(T), Err(E) }",
"fn main() {",
"  let opt = Option::Some(5);",
"  let val = match opt {",
"    Option::Some(x) => x,",
"    Option::None => 0,",
"  };",
"  let res: Result<i32,String> = Result::Ok(42);",
"  let out = match res {",
"    Result::Ok(v) => v,",
"    Result::Err(e) => 0,",
"  };",
"}"
].join("\n") },
{ id:"mod", name:"模組樹 mod", tag:"路徑解析/可見性", desc:"mod, pub, use, crate::", code:[
"# @intent: 模組樹與可見性",
"mod geometry {",
"  pub struct Point { pub x: i32, pub y: i32 }",
"  pub mod utils {",
"    pub fn distance(p: &super::Point) -> i32 { p.x * p.x + p.y * p.y }",
"  }",
"}",
"mod app {",
"  use crate::geometry::Point;",
"  use crate::geometry::utils::distance;",
"  pub fn run() -> i32 {",
"    let p = Point { x: 3, y: 4 };",
"    distance(&p)",
"  }",
"}",
"fn main() {",
"  let d = app::run();",
"}"
].join("\n") },
{ id:"async", name:"async / await", tag:"Future/StateMachine", desc:"async fn, await, Future", code:[
"# @intent: async/await → Future 狀態機",
"async fn fetch() -> i32 { 42 }",
"async fn process(x: i32) -> i32 { x + 1 }",
"fn main() {",
"  let fut = fetch();",
"  # 降維後: loop { match poll(fut) { Ready(v) => break v, Pending => yield } }",
"  let val = fut.await;",
"  let val2 = process(val).await;",
"}"
].join("\n") },
{ id:"io", name:"I/O", tag:"效應/Result", desc:"File, println!, Result, @pure", code:[
"# @intent: I/O 效應系統",
"# @pure false",
"fn main() {",
"  let s = String::from(\"hello\");",
"  println!(\"{}\", s);",
"  let f = File::open(\"data.txt\");",
"  let content = match f {",
"    Result::Ok(file) => file.read_to_string(),",
"    Result::Err(e) => String::from(\"\"),",
"  };",
"}"
].join("\n") },
{ id:"unsafe", name:"unsafe / raw ptr", tag:"上下文位元", desc:"unsafe block, *mut T, *const T", code:[
"# @intent: unsafe 上下文與裸指針",
"fn main() {",
"  let x = 5;",
"  let p: *mut i32 = &mut x as *mut i32; # safe: &mut -> *mut",
"  unsafe {",
"    *p = 10; # 需 unsafe",
"    let y = *p;",
"  }",
"  # *p = 20; # 若取消註釋 → UNSAT (需 unsafe)",
"}"
].join("\n") },
{ id:"lifetime", name:"lifetime 'a", tag:"outlives/NLL", desc:"'a, 'b: 'a, NLL region", code:[
"# @intent: lifetime 參數與 NLL",
"fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {",
"  if x.len() > y.len() { x } else { y }",
"}",
"fn first<'a, 'b: 'a>(x: &'a i32, y: &'b i32) -> &'a i32 { x }",
"struct Ref<'a, T> { r: &'a T }",
"impl<'a, T> Ref<'a, T> {",
"  fn get(&self) -> &'a T { self.r }",
"}",
"fn main() {",
"  let a = 5;",
"  let b = 10;",
"  let r = longest(&a, &b);",
"}"
].join("\n") },
{ id:"pat_or", name:"Pat Or (a|b)", tag:"FullPat::Or", desc:"a | b | c 模式，parse_pat.rs", code:[
"# @intent: pat_or SAT — Or 模式",
"enum Option<T> { Some(T), None }",
"fn main() {",
"  let opt = Option::Some(5);",
"  let y = match opt {",
"    Some(1) | Some(2) | Some(3) => 1,",
"    Some(x) | None => 0,",
"    _ => 0,",
"  };",
"}"
].join("\n") },
{ id:"pat_range", name:"Pat Range (0..10)", tag:"FullPat::Range", desc:"0..10, 0..=10, ..10, 0.. 模式", code:[
"# @intent: pat_range SAT — Range 模式",
"fn main() {",
"  let x = 5;",
"  let y = match x {",
"    0..10 => 1,",
"    0..=10 => 2,",
"    ..10 => 3,",
"    0.. => 4,",
"    _ => 0,",
"  };",
"}"
].join("\n") },
{ id:"closure", name:"Closure |x| x+1", tag:"FullExpr::Closure", desc:"||, |x|, |x,y|, move ||, async move", code:[
"# @intent: closure SAT",
"fn main() {",
"  let f = |x| x + 1;",
"  let g = || 42;",
"  let h = |x, y| x + y;",
"  let mv = move |x| x + 1;",
"  let v = vec![1,2,3];",
"  let sum = v.iter().map(|x| x*2).collect();",
"  let a = f(5);",
"}"
].join("\n") },
{ id:"return_break", name:"Return/Break/Continue", tag:"Return/Break", desc:"return, break label+expr, continue", code:[
"# @intent: return_break SAT",
"fn foo(x: i32) -> i32 {",
"  if x > 0 { return x; }",
"  return 0;",
"}",
"fn main() {",
"  let mut i = 0;",
"  loop {",
"    if i >= 5 { break; }",
"    if i == 2 { continue; }",
"    i = i + 1;",
"  }",
"  'outer: loop {",
"    loop {",
"      break 'outer 42;",
"    }",
"  }",
"}"
].join("\n") },
{ id:"try_cast", name:"Try ? / Cast as / Range", tag:"Try/Cast/Range", desc:"x?, x as i32, 0..10, 0..=10", code:[
"# @intent: try_cast SAT",
"fn may_fail() -> Result<i32, String> { Ok(42) }",
"fn main() {",
"  let r = may_fail();",
"  let v = r?; # Try",
"  let x = 5 as i64; # Cast",
"  let y = v as *mut i32; # Cast *mut",
"  let range = 0..10; # Range",
"  let range2 = 0..=10; # Inclusive",
"  let range3 = ..10;",
"}"
].join("\n") },
];

const $ = id=>document.getElementById(id);
function renderList(){
  const c = $("featList");
  c.innerHTML = FEATURES.map(f=>`<div class="feat" data-id="${f.id}" onclick="loadFeat('${f.id}')"><div class="t">${f.name}</div><div class="d">${f.desc}</div><div class="tag">${f.tag}</div></div>`).join("");
}
function loadFeat(id){
  const f = FEATURES.find(x=>x.id===id);
  if(!f) return;
  $("src").value = f.code;
  document.querySelectorAll(".feat").forEach(el=>el.classList.toggle("active", el.dataset.id===id));
  switchTab("src");
}
function switchTab(t){
  document.querySelectorAll(".tab").forEach(el=>el.classList.toggle("active", el.dataset.tab===t));
  $("src").style.display = t==="src"?"block":"none";
  $("loweredView").style.display = t==="lowered"?"block":"none";
  $("encodingView").style.display = t==="encoding"?"block":"none";
}
document.querySelectorAll(".tab").forEach(el=>el.addEventListener("click",()=>switchTab(el.dataset.tab)));

function esc(s){return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");}

async function runCheck(){
  const out = $("out");
  out.innerHTML = '<span style="color:var(--dim)">驗證中… (v0.2 + syn hybrid + QAP)</span>';
  try{
    const resp = await fetch("/api/v2/check",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source:$("src").value})});
    const j = await resp.json();
    let html = `<span class="badge ${j.verdict.startsWith("SAT")?"b-sat":j.verdict.startsWith("UNSAT")?"b-unsat":"b-unk"}">${esc(j.verdict)}</span> <span class="mono">${esc(j.features_used.join(", "))}</span> <span class="badge">${j.syn_used?"syn 完整 ✓":"手寫"}</span>`;
    if(j.errors && j.errors.length) html+=`<pre style="color:var(--bad)">errors: ${esc(j.errors.join("\n"))}</pre>`;
    html+=`<div class="kv">${kv("類型宇宙 N",j.type_universe_size)}${kv("宇宙顯示",j.type_universe_display.split("\n")[0])}${j.stats?kv("變量 n_vars",j.stats.n_vars)+kv("多項式",j.stats.n_polys)+kv("子句",j.stats.n_clauses)+kv("CDCL輪",j.stats.cdcl_rounds):""}${j.constraints_v2?kv("每節點位寬",j.constraints_v2.universe_n)+kv("product",j.constraints_v2.n_products)+kv("sum",j.constraints_v2.n_sums)+kv("match",j.constraints_v2.n_matches):""}</div>`;
    if(j.oracle){
      html+=`<h3>Oracle 對比 (Path C 混合)</h3><div class="kv">${kv("手寫 items",j.oracle.handwritten_items)}${kv("syn items",j.oracle.syn_items)}${kv("手寫 N",j.oracle.handwritten_universe_n)}${kv("syn N",j.oracle.syn_universe_n)}${kv("缺口",j.oracle.missing_in_handwritten.length)}${kv("差異",j.oracle.diff_items.length)}</div>`;
      if(j.oracle.missing_in_handwritten.length) html+=`<pre style="color:var(--warn)">手寫缺口: ${esc(j.oracle.missing_in_handwritten.join("\n"))}</pre>`;
      if(j.oracle.diff_items.length) html+=`<pre style="color:var(--bad)">差異: ${esc(j.oracle.diff_items.join("\n"))}</pre>`;
      html+=`<details><summary>手寫 display</summary><pre>${esc(j.oracle.handwritten_display.slice(0,2000))}</pre></details>`;
      html+=`<details><summary>syn display</summary><pre>${esc(j.oracle.syn_display.slice(0,2000))}</pre></details>`;
    }
    if(j.phase3){
      html+=`<h3>Phase3 詳細</h3><div class="kv">${kv("fuel",j.phase3.fuel)}${kv("lifetimes",j.phase3.lifetimes.length)}${kv("unsafe_allowed",j.phase3.unsafe_allowed)}${kv("no_io",j.phase3.no_io)}${kv("lifetime cycle",j.phase3.lifetime_graph.has_cycle)}${kv("borrowck err",j.phase3.borrowck.errors.length)}${kv("effects err",j.phase3.effects.errors.length)}${kv("Vec",j.phase3.stdlib.vec_count)}${kv("HashMap",j.phase3.stdlib.hashmap_count)}${kv("async states",j.phase3.async_machines.length?j.phase3.async_machines[0].n_states:0)}</div>`;
      if(j.phase3.lifetime_graph.edges.length) html+=`<pre>outlives: ${esc(j.phase3.lifetime_graph.edges.join(", "))}</pre>`;
      if(j.phase3.borrowck.errors.length) html+=`<pre style="color:var(--bad)">borrowck: ${esc(j.phase3.borrowck.errors.join("; "))}</pre>`;
      if(j.phase3.effects.errors.length) html+=`<pre style="color:var(--bad)">effects: ${esc(j.phase3.effects.errors.join("; "))}</pre>`;
      if(j.phase3.async_machines.length) html+=`<h3>Async State Machine (QAP)</h3><pre>${esc(j.phase3.async_machines[0].poly_text.join("\n"))}</pre>`;
      if(j.phase3.stdlib.r1cs.length) html+=`<h3>Stdlib R1CS</h3><pre>${esc(j.phase3.stdlib.r1cs.join("\n"))}</pre>`;
    }
    if(j.encoding_notes && j.encoding_notes.length){
      html+="<h3>編碼說明（選中特性）</h3>";
      j.encoding_notes.forEach(n=>{ html+=`<div><b>${esc(n.feature)}</b><pre>${esc(n.encoding)}</pre></div>`; });
    }
    html+=`<h3>降維後 (syn hybrid -> ProgramV2 display)</h3><pre>${esc(j.lowering_report.slice(0,5000))}</pre>`;
    html+=`<h3>降維後 core .poly (v0.1 兼容)</h3><pre>${esc(j.lowered_poly.slice(0,4000))}</pre>`;
    if(j.stats){
      html+=`<div>QAP: ${j.stats.qap_verified?"通過 ✓":"失敗"} | 變量 N=${j.type_universe_size} 每節點位寬=${j.constraints_v2?j.constraints_v2.universe_n:"?"} => n_vars = Σ_nodes N + aux = ${j.stats.n_vars}</div>`;
      html+=`<pre>Universe: ${esc(j.type_universe_display.slice(0,2000))}</pre>`;
    }
    out.innerHTML = html;
    $("loweredView").textContent = j.lowering_report + "\n\n--- core poly ---\n" + j.lowered_poly;
    $("encodingView").textContent = (j.encoding_notes||[]).map(n=>`## ${n.feature}\n${n.encoding}`).join("\n\n") + "\n\n## Universe N\n" + j.type_universe_display;
  }catch(e){ out.innerHTML = `<span class="badge b-unsat">錯誤</span><pre>${esc(String(e))}</pre>`; }
}

async function runFixOracle(){
  const out = $(\"out\");
  out.innerHTML = '<span style=\"color:var(--warn)\">Oracle 修復中… 調用 /api/v2/fix_oracle，自動將 syn 解析結果寫回 core AST</span>';
  try{
    const resp = await fetch(\"/api/v2/fix_oracle\",{method:\"POST\",headers:{\"Content-Type\":\"application/json\"},body:JSON.stringify({source:$(\"src\").value})});
    const j = await resp.json();
    let html = `<span class=\"badge ${j.fixed?\"b-sat\":\"b-unsat\"}\">${j.fixed?\"已修復 ✓\":\"修復失敗\"}</span> <span class=\"badge\">syn=${j.syn_can_parse?\"✓\":\"✗\"} 手寫=${j.handwritten_can_parse?\"✓\":\"✗\"}</span>`;
    html+=`<div class=\"kv\">${kv(\"缺口數\",j.oracle.missing_in_handwritten.length)}${kv(\"差異數\",j.diff.length)}${kv(\"建議數\",j.suggestions.length)}${j.constraints_v2?kv(\"修復後 N\",j.constraints_v2.universe_n)+kv(\"修復後 vars\",j.constraints_v2.n_vars):\"\"}</div>`;
    if(j.oracle.missing_in_handwritten.length) html+=`<pre style=\"color:var(--warn)\">手寫缺口: ${esc(j.oracle.missing_in_handwritten.join(\"\\n\"))}</pre>`;
    if(j.diff.length) html+=`<pre style=\"color:var(--bad)\">差異: ${esc(j.diff.join(\"\\n\"))}</pre>`;
    html+=`<h3>修復建議 (自動生成)</h3><pre>${esc(j.suggestions.join(\"\\n\"))}</pre>`;
    html+=`<h3>Diff 展示</h3><div style=\"display:grid;grid-template-columns:1fr 1fr;gap:8px\"><div><b>手寫 display</b><pre>${esc(j.handwritten_display.slice(0,3000))}</pre></div><div><b>syn display (真值)</b><pre>${esc(j.syn_display.slice(0,3000))}</pre></div></div>`;
    html+=`<h3>修復後 ProgramV2 (core AST)</h3><pre>${esc(j.fixed_program_display.slice(0,4000))}</pre>`;
    html+=`<h3>修復後 v2 Poly (core 兼容)</h3><pre>${esc(j.fixed_poly.slice(0,4000))}</pre>`;
    html+=`<details><summary>完整 OracleReport JSON</summary><pre>${esc(JSON.stringify(j.oracle,null,2).slice(0,5000))}</pre></details>`;
    out.innerHTML = html;
    $(\"loweredView\").textContent = \"=== 修復後 ProgramV2 ===\\n\"+j.fixed_program_display+\"\\n\\n=== 修復後 Poly ===\\n\"+j.fixed_poly+\"\\n\\n=== 建議 ===\\n\"+j.suggestions.join(\"\\n\");
    switchTab(\"lowered\");
  }catch(e){ out.innerHTML = `<span class=\"badge b-unsat\">錯誤</span><pre>${esc(String(e))}</pre>`; }
}

async function runLower(){
  const out = $("out");
  out.innerHTML = '<span style="color:var(--dim)">降維中…</span>';
  try{
    const resp = await fetch("/api/v2/lower",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source:$("src").value})});
    const j = await resp.json();
    $("loweredView").textContent = j.lowered;
    switchTab("lowered");
    out.innerHTML = `<span class="badge b-sat">降維完成</span> <span class="mono">${esc(j.features.join(", "))}</span><pre>${esc(j.lowered.slice(0,5000))}</pre>`;
  }catch(e){ out.innerHTML = `<pre>${esc(String(e))}</pre>`; }
}

async function runCheckV1(){
  const out = $("out");
  out.innerHTML = '<span style="color:var(--dim)">v0.1 兼容驗證中…</span>';
  try{
    const resp = await fetch("/api/check",{method:"POST",headers:{"Content-Type":"text/plain"},body:$("src").value});
    const j = await resp.json();
    if(j.status==="error"){ out.innerHTML = `<span class="badge b-unsat">錯誤</span><pre>${esc(j.error)}</pre>`; return; }
    const cls = j.verdict==="SAT"?"b-sat":"b-unsat";
    let html = `<span class="badge ${cls}">${esc(j.verdict)}</span> <span class="mono">${esc(j.source)}</span>`;
    html+=`<div class="kv">${kv("型別檢查",j.typechecks?"接受 ✓":"拒絕 ✗")}${kv("一致",j.agrees?"一致 ✓":"不一致")}${kv("變量",j.stats.n_vars)}${kv("多項式",j.stats.n_polys)}</div>`;
    if(j.generated && j.generated.code) html+=`<h3>生成碼 ${j.generated.rustc_compiles?"rustc ✓":""}</h3><pre>${esc(j.generated.code)}</pre>`;
    out.innerHTML = html;
  }catch(e){ out.innerHTML = `<pre>${esc(String(e))}</pre>`; }
}

function kv(k,v){return `<div><div class="k">${esc(k)}</div><div class="v">${esc(v)}</div></div>`;}

renderList();
loadFeat("struct");
</script>
</body>
</html>
"##;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8091);
    #[cfg(feature = "cors")]
    let app = Router::new()
        .route("/", get(|| async { Html(PAGE) }))
        .route("/health", get(health))
        .route("/api/v2/check", post(api::check_v2))
        .route("/api/v2/lower", post(api::lower_v2))
        .route("/api/v2/fix_oracle", post(api::fix_oracle))
        .route("/api/v2/syn_explain", post(syn_explain_handler))
        .route("/api/v2/syn_visit", post(syn_visit_handler))
        .route("/api/v2/syn_100", post(syn_100_handler))
        .route("/api/check", post(check_v1))
        .route("/api/expand", post(expand_v1))
        .layer(CorsLayer::permissive());
    #[cfg(not(feature = "cors"))]
    let app = Router::new()
        .route("/", get(|| async { Html(PAGE) }))
        .route("/health", get(health))
        .route("/api/v2/check", post(api::check_v2))
        .route("/api/v2/lower", post(api::lower_v2))
        .route("/api/v2/fix_oracle", post(api::fix_oracle))
        .route("/api/v2/syn_explain", post(syn_explain_handler))
        .route("/api/v2/syn_visit", post(syn_visit_handler))
        .route("/api/v2/syn_100", post(syn_100_handler))
        .route("/api/check", post(check_v1))
        .route("/api/expand", post(expand_v1));

    println!("polyrust-full v0.2 原型 @ 0.0.0.0:{}", port);
    println!("  features: struct/enum/impl/trait, Vec/String/HashMap, loop/match, mod, async, I/O, unsafe, lifetime");
    println!("  UI: http://0.0.0.0:{}/", port);
    println!("  API: POST /api/v2/check {{source}}");
    println!("  Lean embedded: {}", formal::startup_report());

    let listener = match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("bind {} failed: {}", port, e);
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("server error: {}", e);
        std::process::exit(1);
    }
}
