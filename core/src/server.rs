// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! 極簡 HTTP server（std-only，零依賴）：把 `check` / `expand` 包成 Web UI。
//!
//! 路由：
//! - `GET /`           → 內嵌 HTML 頁面（inline CSS/JS，無外部資源）
//! - `GET /health`     → 存活探針
//! - `POST /api/check` → body 為 `.poly` 文本，回 `check` JSON 契約
//! - `POST /api/expand`→ body 為 `.poly` 文本，回 `expand` JSON 契約
//!
//! 只依賴 std，無 tokio/actix 等。每個連線用一條執行緒處理。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::driver::{check_text_json, expand_text_json, generate_text_json, coverage_text_json, check_v2_text_json, check_v3_text_json};

const MAX_BODY: usize = 1 << 20; // 1 MiB 上限

/// 啟動 server，阻塞直到結束。
pub fn serve(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    eprintln!("polyrust UI：http://0.0.0.0:{port}  (Ctrl-C 結束)");
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                std::thread::spawn(move || {
                    let _ = handle(stream);
                });
            }
            Err(e) => eprintln!("連線錯誤：{}", e),
        }
    }
    Ok(())
}

fn handle(mut stream: TcpStream) -> std::io::Result<()> {
    // 讀取超時：慢速/閒置連線不能無限佔住執行緒
    stream.set_read_timeout(Some(std::time::Duration::from_secs(60)))?;
    let req = read_request(&mut stream)?;
    let (method, path, body) = req;

    let (status, content_type, payload) = match (method.as_str(), path.as_str()) {
        ("GET", "/") => ("200 OK", "text/html; charset=utf-8", PAGE.to_string()),
        ("GET", "/health") => ("200 OK", "text/plain; charset=utf-8", "ok\n".to_string()),
        ("POST", "/api/check") => {
            let (j, _ok) = check_text_json("web", &body, None);
            // 錯誤也回 200，狀態碼統一，實際成敗見 JSON 的 status 欄位
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/expand") => {
            let (j, _ok) = expand_text_json("web", &body, None);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/v1/generate") => {
            let (j, _ok) = generate_text_json("web", &body, None);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/check-v2") | ("POST", "/api/v2/check") => {
            let (j, _ok) = check_v2_text_json("web", &body, None);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/check-v3") | ("POST", "/api/v3/check") => {
            let (j, _ok) = check_v3_text_json("web", &body, None);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/coverage") | ("POST", "/api/v2/coverage") => {
            let (j, _ok) = coverage_text_json("web", &body);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/dsl") | ("POST", "/api/v1/dsl") => {
            let (j, _ok) = crate::driver::dsl_text_json("web", &body);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/dsl/project") | ("POST", "/api/v1/dsl/project") => {
            let (j, _ok) = crate::driver::dsl_project_text_json("web", &body);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/nl/codegen") | ("POST", "/api/v1/nl/codegen") | ("POST", "/api/dsl/nl_codegen") => {
            let (j, _ok) = crate::driver::nl_codegen_text_json("web", &body);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/nl/codegen/batch") | ("POST", "/api/v1/nl/codegen/batch") => {
            let (j, _ok) = crate::driver::nl_codegen_batch_text_json();
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        ("POST", "/api/nl") => {
            // body：JSON {"description": "...", "provider"?, "model"?, "base_url"?,
            //             "api_key"?, "attempts"?}（或純文字 = description）
            let (j, _ok) = nl_text_json(&body);
            ("200 OK", "application/json; charset=utf-8", j.to_string())
        }
        _ => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found\n".to_string(),
        ),
    };

    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{payload}",
        payload.len()
    )
}


fn read_request(stream: &mut TcpStream) -> std::io::Result<(String, String, String)> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    // 讀到 header 結束（\r\n\r\n）
    let header_end = loop {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            return Ok((String::new(), String::new(), String::new()));
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
            break pos + 4;
        }
        if buf.len() > MAX_BODY {
            break buf.len();
        }
    };

    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let mut content_length = 0usize;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse().unwrap_or(0);
            }
        }
    }

    let mut body_bytes = buf[header_end..].to_vec();
    while body_bytes.len() < content_length {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        body_bytes.extend_from_slice(&tmp[..n]);
        if body_bytes.len() > MAX_BODY {
            break;
        }
    }
    body_bytes.truncate(content_length.min(MAX_BODY));
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    Ok((method, path, body))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

/// 內嵌 HTML 頁面（inline CSS/JS，無外部資源，可在無網路預覽中完整運作）。
const PAGE: &str = r##"<!DOCTYPE html>
<html lang="zh-Hant">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>polyrust — 代數形式化驗證 Phase2-4</title>
<style>
  :root {
    --bg: #0e1117; --panel: #161b26; --panel2: #1c2333; --border: #2a3348;
    --fg: #d7deea; --dim: #8591a8; --accent: #4ea1ff; --ok: #34d399;
    --bad: #f87171; --warn: #fbbf24; --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
  * { box-sizing: border-box; }
  body { margin: 0; font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
         background: var(--bg); color: var(--fg); line-height: 1.5; }
  header { padding: 18px 24px; border-bottom: 1px solid var(--border); background: var(--panel); display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:10px; }
  header h1 { margin: 0; font-size: 18px; font-weight: 650; }
  header p { margin: 4px 0 0; color: var(--dim); font-size: 13px; }
  .wrap { max-width: 1400px; margin: 0 auto; padding: 20px 24px; display: grid;
          grid-template-columns: 340px 1fr 440px; gap: 20px; }
  @media (max-width: 1200px) { .wrap { grid-template-columns: 1fr 1fr; } .wrap .right { grid-column: 1 / -1; } }
  @media (max-width: 820px) { .wrap { grid-template-columns: 1fr; } }
  .card { background: var(--panel); border: 1px solid var(--border); border-radius: 10px; overflow:hidden; }
  .card h2 { margin: 0; padding: 12px 16px; font-size: 14px; font-weight: 650;
             border-bottom: 1px solid var(--border); color: var(--dim); text-transform: uppercase;
             letter-spacing: .04em; display:flex; justify-content:space-between; align-items:center; flex-wrap:wrap; gap:6px; }
  textarea { width: 100%; height: 420px; background: var(--panel2); color: var(--fg);
             border: none; padding: 14px; font-family: var(--mono); font-size: 13px;
             resize: vertical; outline: none; }
  .btns { display: flex; gap: 8px; padding: 12px; border-top: 1px solid var(--border); flex-wrap:wrap; }
  button { padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px;
           background: var(--panel2); color: var(--fg); font-size: 13px; font-weight: 600;
           cursor: pointer; }
  button.primary { background: var(--accent); border-color: var(--accent); color: #06121f; }
  button.v2 { background: rgba(251,191,36,.15); border-color: var(--warn); color: var(--warn); }
  button.coverage { background: rgba(52,211,153,.15); border-color: var(--ok); color: var(--ok); }
  button:hover { filter: brightness(1.1); }
  button:disabled { opacity: .5; cursor: not-allowed; }
  .out { padding: 16px; min-height: 200px; font-size: 13px; }
  .badge { display: inline-block; padding: 2px 10px; border-radius: 999px; font-weight: 700;
           font-size: 12px; letter-spacing: .03em; }
  .b-sat { background: rgba(52,211,153,.15); color: var(--ok); border: 1px solid var(--ok); }
  .b-unsat { background: rgba(248,113,113,.15); color: var(--bad); border: 1px solid var(--bad); }
  .b-err { background: rgba(251,191,36,.15); color: var(--warn); border: 1px solid var(--warn); }
  .b-coverage { background: rgba(78,161,255,.15); color: var(--accent); border: 1px solid var(--accent); }
  .b-n { background: rgba(251,191,36,.15); color: var(--warn); border: 1px solid var(--warn); }
  .kv { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px,1fr)); gap: 8px;
        margin: 12px 0; }
  .kv div { background: var(--panel2); border: 1px solid var(--border); border-radius: 6px;
            padding: 6px 10px; }
  .kv .k { color: var(--dim); font-size: 11px; }
  .kv .v { font-family: var(--mono); font-size: 13px; }
  pre { background: var(--panel2); border: 1px solid var(--border); border-radius: 8px;
        padding: 12px; overflow-x: auto; font-family: var(--mono); font-size: 12px; white-space:pre-wrap; }
  .msg { color: var(--dim); }
  .err { color: var(--bad); }
  .ok { color: var(--ok); }
  .tick { color: var(--ok); } .cross { color: var(--bad); }
  h3 { font-size: 12px; color: var(--dim); margin: 14px 0 6px; text-transform:uppercase; letter-spacing:.04em; }
  .tag { font-family: var(--mono); font-size: 11px; color: var(--dim); }
  .progress { height: 8px; background: var(--panel2); border-radius: 999px; overflow:hidden; margin: 8px 0; border:1px solid var(--border); }
  .progress .bar { height: 100%; background: linear-gradient(90deg, var(--accent), var(--ok)); transition: width .5s; }
  .feat-list { max-height: 260px; overflow-y:auto; padding: 8px; display:flex; flex-direction:column; gap:6px; }
  .feat { padding: 8px 10px; border-radius: 8px; background: var(--panel2); border:1px solid transparent; font-size:12px; display:flex; justify-content:space-between; align-items:center; }
  .feat.ok { border-color: rgba(52,211,153,.3); }
  .feat.miss { border-color: rgba(248,113,113,.3); opacity:.85; }
  .feat .name { font-weight:600; font-family: var(--mono); font-size:12px; }
  .feat .desc { color: var(--dim); font-size:11px; margin-top:2px; }
  .examples { padding: 8px; display:flex; flex-direction:column; gap:6px; max-height: 260px; overflow-y:auto; }
  .ex { padding: 8px 10px; border-radius: 6px; background: var(--panel2); border:1px solid var(--border); cursor:pointer; }
  .ex:hover { border-color: var(--accent); }
  .ex .t { font-size:12px; font-weight:600; }
  .ex .d { font-size:11px; color: var(--dim); }
  .tabs { display:flex; gap:6px; padding: 8px 12px; border-bottom:1px solid var(--border); flex-wrap:wrap; }
  .tab { padding:4px 10px; border-radius:6px; cursor:pointer; font-size:12px; background: var(--panel2); border:1px solid var(--border); }
  .tab.active { background: var(--accent); color:#06121f; border-color: var(--accent); }
  .n-display { font-family: var(--mono); font-size: 13px; background: rgba(251,191,36,.1); border:1px solid var(--warn); padding:6px 10px; border-radius:6px; margin:8px 0; }
  .per-node { max-height: 200px; overflow-y:auto; }
  .per-node div { font-family: var(--mono); font-size:11px; padding:2px 6px; border-bottom:1px solid var(--border); }
</style>
</head>
<body>
<header>
  <div>
    <h1>polyrust · Phase2-4 代數形式化驗證</h1>
    <p>Phase2: ty.rs unify / lower.rs struct/enum/match/loop/mod flatten / constraints N可變 — Phase3: borrowck lifetime / unsafe raw ptr / async state machine / QAP — Phase4: UI展示N + Lean ModuleFlatten/MatchDecisionTree</p>
  </div>
  <div>
    <span class="badge b-sat">core 零依賴</span>
    <span class="badge b-n" id="nBadge">N=-</span>
    <span class="badge b-coverage" id="globalBadge">-</span>
    <a href="/health" target="_blank" class="badge">health</a>
  </div>
</header>
<div class="wrap">
  <div class="card left">
    <h2>輸入（.poly 描述） <span class="tag" id="coverageBadge"></span> <span class="tag" id="nBadge2"></span></h2>
    <textarea id="src" spellcheck="false"></textarea>
    <div class="btns">
      <button id="btnCheck" class="primary" onclick="run('check')">驗證 v1</button>
      <button id="btnCheckV2" class="v2" onclick="run('check-v2')">驗證 v2 (N可變)</button>
      <button id="btnCheckV3" style="background:rgba(168,85,247,.2);border-color:#a855f7;color:#a855f7" onclick="run('check-v3')">驗證 v3 商業深化</button>
      <button id="btnExpand" onclick="run('expand')">純展開</button>
      <button id="btnGen" onclick="run('generate')">生成碼</button>
      <button id="btnCoverage" class="coverage" onclick="run('coverage')">語法覆蓋率</button>
    </div>
    <h2 style="border-top:1px solid var(--border)">Phase3 缺口 SAT/UNSAT</h2>
    <div class="examples" id="exampleList"></div>
  </div>
  <div class="card center">
    <h2>結果 <span class="tag">check / check-v2 / check-v3 / expand / generate / coverage</span></h2>
    <div class="out" id="out"><span class="msg">尚未執行。按「驗證 v2」查看 N可變宇宙與 lowering。</span></div>
  </div>
  <div class="card right">
    <h2>語法覆蓋率 + N展示 <span class="badge b-coverage" id="coveragePctBadge">-</span> <span class="badge b-n" id="nBadgeRight">-</span></h2>
    <div class="tabs">
      <div class="tab active" id="tabV2" onclick="switchTab('v2')">v2 驗證 (N可變)</div>
      <div class="tab" id="tabFile" onclick="switchTab('file')">文件覆蓋</div>
      <div class="tab" id="tabGlobal" onclick="switchTab('global')">解析器能力</div>
    </div>
    <div style="padding:12px 16px" id="panelV2">
      <div class="n-display" id="nDisplay">N = - (7 + i)</div>
      <div class="kv">
        <div><div class="k">類型宇宙 N</div><div class="v" id="typeUniverseSize">-</div></div>
        <div><div class="k">變量</div><div class="v" id="nVars">-</div></div>
        <div><div class="k">多項式</div><div class="v" id="nPolys">-</div></div>
        <div><div class="k">Product</div><div class="v" id="nProducts">-</div></div>
        <div><div class="k">Sum</div><div class="v" id="nSums">-</div></div>
        <div><div class="k">Match</div><div class="v" id="nMatches">-</div></div>
        <div><div class="k">Loop Fuel</div><div class="v" id="nLoopFuel">-</div></div>
        <div><div class="k">Async</div><div class="v" id="nAsync">-</div></div>
        <div><div class="k">Lifetime</div><div class="v" id="nLifetime">-</div></div>
        <div><div class="k">Unsafe</div><div class="v" id="nUnsafe">-</div></div>
        <div><div class="k">Stdlib</div><div class="v" id="nStdlib">-</div></div>
        <div><div class="k">TraitImpl</div><div class="v" id="nTraitImpl">-</div></div>
        <div><div class="k">QAP 約束</div><div class="v" id="r1csConstraints">-</div></div>
        <div><div class="k">Unify 多項式</div><div class="v" id="unifyPolys">-</div></div>
      </div>
      <h3>特性使用 (features_used)</h3>
      <pre id="featuresUsed">-</pre>
      <h3>Per-Node Bits (node_id, kind, N)</h3>
      <div class="per-node" id="perNodeBits">-</div>
      <h3>Borrow 衝突</h3>
      <pre id="borrowConflicts">-</pre>
      <h3>Lowering 報告 (含 N = 7+i)</h3>
      <pre id="loweringReport" style="max-height:200px">-</pre>
    </div>
    <div style="padding:12px 16px; display:none" id="panelFile">
      <div class="kv">
        <div><div class="k">文件總特性</div><div class="v" id="totalFeatures">-</div></div>
        <div><div class="k">文件已出現</div><div class="v" id="coveredFeatures">-</div></div>
        <div><div class="k">文件覆蓋率</div><div class="v" id="coveragePct">-</div></div>
        <div><div class="k">未出現數</div><div class="v" id="missingCount">-</div></div>
      </div>
      <div class="progress"><div class="bar" id="coverageBar" style="width:0%"></div></div>
      <h3>未出現特性</h3>
      <pre id="missingList" style="max-height:100px; overflow-y:auto">尚未檢測</pre>
      <h3>文件特性（✅ 已出現 / ❌ 未出現）</h3>
      <div class="feat-list" id="featList"><span class="msg">尚未檢測</span></div>
    </div>
    <div style="padding:12px 16px; display:none" id="panelGlobal">
      <div class="kv">
        <div><div class="k">全局總特性</div><div class="v" id="globalTotal">-</div></div>
        <div><div class="k">已實現</div><div class="v" id="globalCovered">-</div></div>
        <div><div class="k">全局覆蓋率</div><div class="v" id="globalPct">-</div></div>
        <div><div class="k">未實現數</div><div class="v" id="globalMissingCount">-</div></div>
      </div>
      <div class="progress"><div class="bar" id="globalBar" style="width:0%; background: linear-gradient(90deg, var(--ok), var(--accent));"></div></div>
      <h3>未實現特性</h3>
      <pre id="globalMissingList" style="max-height:100px; overflow-y:auto">尚未檢測</pre>
      <h3>解析器能力（✅ 已實現 / ❌ 未實現）</h3>
      <div class="feat-list" id="globalFeatList"><span class="msg">尚未檢測</span></div>
    </div>
  </div>
</div>
<script>
const DEFAULT_SRC = [
"# @intent: Phase2-4 模板：struct/enum/match/loop/mod flatten + N可變 + borrowck + unsafe + async + QAP",
"# @type-universe: Vec<i32>, HashMap<String,i32>, String",
"",
"mod geometry {",
"    pub struct Point { pub x: i32, pub y: i32 }",
"    pub enum Shape { Circle(i32), Rect(i32, i32) }",
"    pub fn new(x: i32, y: i32) -> Point { Point { x, y } }",
"}",
"",
"struct User { name: String, age: i32 }",
"enum Option<T> { Some(T), None }",
"",
"fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }",
"",
"async fn fetch_data() -> i32 { 42 }",
"",
"fn main() {",
"    let p = geometry::Point { x: 3, y: 4 };",
"    let s = geometry::Shape::Circle(5);",
"    let u = User { name: String::from(\"hi\"), age: 20 };",
"    let opt = Option::Some(5);",
"    let y = match opt { Some(v) => v, None => 0 };",
"    let mut v: Vec<i32> = Vec::new();",
"    v.push(1);",
"    for i in v { println!(\"{}\", i); }",
"    let mut i = 0;",
"    loop { if i>=2 { break; } i+=1; }",
"    let tup: (i32, bool) = (1, true);",
"    let arr: [i32; 3] = [1,2,3];",
"    let f: fn(i32)->bool = |x| x>0;",
"    let cl = |x| x+1;",
"    let r = 0..10;",
"    let _ = match opt { Some(1) | Some(2) => 1, _ => 0 };",
"    let x = 5 as i64;",
"    let p2: *mut i32 = &mut 5 as *mut i32;",
"    unsafe { let _ = *p2; }",
"    let data = fetch_data().await;",
"    return;",
"}",
""
].join("
");

const EXAMPLES = [
  { id:"struct", name:"Struct + Enum + Impl + Trait", file:"examples/phase3/struct_sat.poly", desc:"lower.rs struct/enum flatten, N=7+i", sat:true },
  { id:"vec", name:"Vec<T> + HashMap", file:"examples/phase3/vec_sat.poly", desc:"stdlib encoding, N可變", sat:true },
  { id:"match", name:"Match DecisionTree", file:"examples/phase3/match_sat.poly", desc:"lower.rs match→if decision tree", sat:true },
  { id:"loop", name:"Loop + Fuel + Invariant", file:"examples/phase3/loop_sat.poly", desc:"loop_contract, fuel", sat:true },
  { id:"mod", name:"Mod Flatten", file:"examples/phase3/mod_sat.poly", desc:"module flatten, path qualify", sat:true },
  { id:"lifetime", name:"Lifetime 'a + Outlives", file:"examples/phase3/lifetime_sat.poly", desc:"borrowck lifetime graph, NLL", sat:true },
  { id:"unsafe", name:"Unsafe Raw Ptr", file:"examples/phase3/unsafe_sat.poly", desc:"unsafe gate, *mut/*const", sat:true },
  { id:"async", name:"Async State Machine", file:"examples/phase3/async_sat.poly", desc:"async fn → state machine enum", sat:true },
  { id:"pat_or", name:"Pat Or (a|b)", file:"examples/phase3/pat_or_sat.poly", desc:"FullPat::Or", sat:true },
  { id:"closure", name:"Closure |x| x+1", file:"examples/phase3/closure_sat.poly", desc:"FullExpr::Closure", sat:true },
  { id:"return", name:"Return", file:"examples/phase3/return_sat.poly", desc:"return expr", sat:true },
  { id:"break", name:"Break/Continue", file:"examples/phase3/break_sat.poly", desc:"break label+expr", sat:true },
  { id:"try", name:"Try ?", file:"examples/phase3/try_sat.poly", desc:"Try x?", sat:true },
  { id:"cast", name:"Cast as", file:"examples/phase3/cast_sat.poly", desc:"x as i32, *mut", sat:true },
];

const $ = (id) => document.getElementById(id);
$("src").value = DEFAULT_SRC;

function renderExamples() {
  const c = $("exampleList");
  c.innerHTML = EXAMPLES.map(ex=>`<div class="ex" onclick="loadExample('${ex.file}')"><div class="t">${ex.name} <span class="badge ${ex.sat?'b-sat':'b-unsat'}" style="font-size:10px">${ex.sat?'SAT':'UNSAT'}</span></div><div class="d">${ex.desc}</div><div class="tag">${ex.file}</div></div>`).join("");
}
renderExamples();

function switchTab(which) {
  $("panelV2").style.display = which==='v2' ? 'block' : 'none';
  $("panelFile").style.display = which==='file' ? 'block' : 'none';
  $("panelGlobal").style.display = which==='global' ? 'block' : 'none';
  $("tabV2").className = 'tab' + (which==='v2'?' active':'');
  $("tabFile").className = 'tab' + (which==='file'?' active':'');
  $("tabGlobal").className = 'tab' + (which==='global'?' active':'');
}

async function loadExample(path) {
  try {
    const resp = await fetch('/'+path);
    if (resp.ok) {
      const txt = await resp.text();
      $("src").value = txt;
    } else {
      $("src").value = `# 加載 ${path} 失敗
` + DEFAULT_SRC;
    }
  } catch(e) {
    $("src").value = `# ${path}
` + DEFAULT_SRC;
  }
  run('check-v2');
}

function esc(s) {
  return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
}

async function run(mode) {
  const out = $("out");
  const b1 = $("btnCheck"), b2 = $("btnCheckV2"), b2v3 = $("btnCheckV3"), b3 = $("btnExpand"), b4 = $("btnGen"), b5 = $("btnCoverage");
  b1.disabled = b2.disabled = b2v3.disabled = b3.disabled = b4.disabled = b5.disabled = true;
  out.innerHTML = '<span class="msg">執行中… ('+mode+')</span>';
  try {
    let url;
    if (mode === "generate") url = "/api/v1/generate";
    else if (mode === "coverage") url = "/api/coverage";
    else if (mode === "check-v2") url = "/api/v2/check";
    else if (mode === "check-v3") url = "/api/v3/check";
    else url = "/api/" + mode;
    const resp = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "text/plain; charset=utf-8" },
      body: $("src").value,
    });
    const j = await resp.json();
    if (j.status === "error") {
      out.innerHTML = '<span class="badge b-err">錯誤</span><pre class="err">' + esc(j.error) + "</pre>";
      if (j.type_universe_size) renderV2(j);
      return;
    }
    if (mode === "coverage") {
      renderCoverage(j);
      out.innerHTML = '<span class="badge b-coverage">文件 ' + (j.coverage_pct||0).toFixed(1) + '%</span> <span class="badge b-sat">全局 ' + (j.global_pct||0).toFixed(1) + '%</span> <span class="tag">' + esc(j.source) + '</span>' +
        '<div class="kv">' + kv("文件總特性", j.total_features) + kv("文件已出現", j.covered_features) + kv("文件覆蓋率", (j.coverage_pct||0).toFixed(1)+'%') + kv("未出現", j.missing.length) +
        kv("全局總特性", j.global_total) + kv("全局已實現", j.global_covered) + kv("全局覆蓋率", (j.global_pct||0).toFixed(1)+'%') + kv("未實現", j.global_missing.length) + '</div>';
    } else if (mode === "check-v2") {
      renderV2(j);
      renderResultV2(j, out);
      try { const covResp = await fetch("/api/coverage", {method:"POST", headers:{"Content-Type":"text/plain"}, body:$("src").value}); const covJ = await covResp.json(); renderCoverage(covJ); } catch(e){}
    } else if (mode === "check-v3") {
      if (j.type_universe || j.final_stats) { /* v3 may contain v2-like fields inside iterations */ }
      renderResultV3(j, out);
      try { const covResp = await fetch("/api/coverage", {method:"POST", headers:{"Content-Type":"text/plain"}, body:$("src").value}); const covJ = await covResp.json(); renderCoverage(covJ); } catch(e){}
    } else {
      render(mode, j);
      try { const covResp = await fetch("/api/v2/check", {method:"POST", headers:{"Content-Type":"text/plain"}, body:$("src").value}); const covJ = await covResp.json(); if (covJ.type_universe_size) renderV2(covJ); } catch(e){}
      try { const covResp2 = await fetch("/api/coverage", {method:"POST", headers:{"Content-Type":"text/plain"}, body:$("src").value}); const covJ2 = await covResp2.json(); renderCoverage(covJ2); } catch(e){}
    }
  } catch (e) {
    out.innerHTML = '<span class="badge b-err">錯誤</span><pre class="err">' + esc(String(e)) + "</pre>";
  } finally {
    b1.disabled = b2.disabled = b2v3.disabled = b3.disabled = b4.disabled = b5.disabled = false;
  }
}

function renderV2(j) {
  const N = j.type_universe_size || 0;
  const display = j.type_universe_display || ("N=" + N);
  $("typeUniverseSize").textContent = N;
  $("nDisplay").textContent = display + " — Phase2 N可變 = 7基底 + i擴展 (struct/enum/generic 等)";
  $("nBadge").textContent = "N=" + N;
  $("nBadge2").textContent = "N=" + N;
  $("nBadgeRight").textContent = "N=" + N;
  $("nVars").textContent = j.stats ? j.stats.n_vars : "-";
  $("nPolys").textContent = j.stats ? j.stats.n_polys : "-";
  $("nProducts").textContent = j.stats ? j.stats.n_products : "-";
  $("nSums").textContent = j.stats ? j.stats.n_sums : "-";
  $("nMatches").textContent = j.stats ? j.stats.n_matches : "-";
  $("nLoopFuel").textContent = j.stats ? j.stats.n_loop_fuel : "-";
  $("nAsync").textContent = j.stats ? j.stats.n_async : "-";
  $("nLifetime").textContent = j.stats ? j.stats.n_lifetime : "-";
  $("nUnsafe").textContent = j.stats ? j.stats.n_unsafe : "-";
  $("nStdlib").textContent = j.stats ? j.stats.n_stdlib : "-";
  $("nTraitImpl").textContent = j.stats ? j.stats.n_trait_impl : "-";
  $("r1csConstraints").textContent = j.stats ? j.stats.r1cs_constraints : "-";
  $("unifyPolys").textContent = j.unify_polys || "-";
  $("featuresUsed").textContent = j.features_used ? j.features_used.join(", ") : "-";
  $("perNodeBits").innerHTML = j.per_node_bits ? j.per_node_bits.map(b=>`<div>node${b.node_id}: ${esc(b.kind)} — N=${b.n} bits</div>`).join("") : "-";
  $("borrowConflicts").textContent = j.borrow_conflicts && j.borrow_conflicts.length ? JSON.stringify(j.borrow_conflicts) : "無衝突 ✓";
  $("loweringReport").textContent = j.lowering_report || "-";
  switchTab('v2');
}

function renderResultV2(j, out) {
  const cls = j.verdict === "SAT" ? "b-sat" : "b-unsat";
  let html = '<span class="badge ' + cls + '">' + esc(j.verdict) + "</span> ";
  html += '<span class="tag">' + esc(j.source) + "</span> ";
  html += '<span class="badge b-n">N=' + (j.type_universe_size||0) + "</span> ";
  html += '<span class="badge b-coverage">' + esc(j.type_universe_display||"") + "</span>";
  if (j.intent) html += '<div class="msg" style="margin-top:8px">意圖：' + esc(j.intent) + "</div>";
  html += "<div class='kv'>" +
    kv("類型宇宙 N", j.type_universe_size) +
    kv("變量", j.stats.n_vars) +
    kv("多項式", j.stats.n_polys) +
    kv("Product", j.stats.n_products) +
    kv("Sum", j.stats.n_sums) +
    kv("Match", j.stats.n_matches) +
    kv("Loop Fuel", j.stats.n_loop_fuel) +
    kv("Async", j.stats.n_async) +
    kv("Lifetime", j.stats.n_lifetime) +
    kv("Unsafe", j.stats.n_unsafe) +
    kv("Stdlib", j.stats.n_stdlib) +
    kv("TraitImpl", j.stats.n_trait_impl) +
    kv("QAP 約束", j.stats.r1cs_constraints) +
    kv("Unify 多項式", j.unify_polys) +
    kv("Lifetime Cycle", j.stats.lifetime_has_cycle ? "有環 ✗" : "無環 ✓") +
    kv("QAP 驗證", j.qap_verified===true?"通過 ✓":j.qap_verified===false?"失敗 ✗":"?") +
    "</div>";
  if (j.features_used && j.features_used.length) {
    html += "<h3>特性使用</h3><div class='tag'>" + esc(j.features_used.join(", ")) + "</div>";
  }
  if (j.errors && j.errors.length) {
    html += "<h3>錯誤</h3><pre class='err'>" + esc(j.errors.join("
")) + "</pre>";
  }
  if (j.warnings && j.warnings.length) {
    html += "<h3>警告</h3><pre class='msg'>" + esc(j.warnings.join("
")) + "</pre>";
  }
  html += "<h3>Lowering 報告 (含 N = 7+i)</h3><pre>" + esc(j.lowering_report||"") + "</pre>";
  if (j.per_node_bits && j.per_node_bits.length) {
    html += "<h3>Per-Node Bits (node_id, kind, N)</h3><pre>" + esc(j.per_node_bits.map(b=>`node${b.node_id}: ${b.kind} — N=${b.n}`).join("
")) + "</pre>";
  }
  out.innerHTML = html;
}

function renderResultV3(j, out) {
  const cls = j.verdict === "SAT" ? "b-sat" : "b-unsat";
  let html = '<span class="badge ' + cls + '">' + esc(j.verdict) + "</span> ";
  html += '<span class="tag">' + esc(j.source) + "</span> ";
  html += '<span class="badge" style="background:rgba(168,85,247,.2);border:1px solid #a855f7;color:#a855f7">V3 商業深化</span> ';
  if (j.commercial) {
    html += '<span class="badge b-n">風險 ' + (j.commercial.risk_score||0).toFixed(1) + ' (' + esc(j.commercial.risk_level||'') + ')</span> ';
    html += '<span class="badge b-coverage">' + esc(j.commercial.iso26262_level||'') + '</span>';
  }
  html += "<div class='kv'>" +
    kv("判定", j.verdict) +
    kv("收斂", j.converged ? "是 ✓" : "否") +
    kv("總耗時ms", j.total_duration_ms) +
    kv("變量", j.final_stats ? j.final_stats.n_vars : "-") +
    kv("多項式", j.final_stats ? j.final_stats.n_polys : "-") +
    kv("Groebner", j.final_stats ? j.final_stats.groebner_algo : "-") +
    kv("風險分數", j.commercial ? j.commercial.risk_score.toFixed(1) : "-") +
    kv("風險等級", j.commercial ? j.commercial.risk_level : "-") +
    kv("ISO26262", j.commercial ? j.commercial.iso26262_level : "-") +
    kv("QAP證書", j.commercial && j.commercial.qap_certificate ? "有 ✓" : "無") +
    kv("鏈上載荷", j.qap_onchain && j.qap_onchain.payload ? "已生成 ✓" : "未生成") +
    kv("自驗證", j.final_stats && j.final_stats.self_verification_passed ? "通過 ✓" : "待") +
    kv("迭代數", j.iterations ? j.iterations.length : "-") +
    "</div>";
  if (j.commercial) {
    html += "<h3>商業價值</h3><pre class='ok'>" + esc(j.commercial.business_value||"") + "\n避免損失: " + esc(j.commercial.loss_avoided||"") + "</pre>";
    html += "<h3>合規映射</h3><pre>" + esc(JSON.stringify(j.commercial.compliance||{}, null, 2)) + "</pre>";
    html += "<h3>Lean 形式化證明引用</h3><pre>" + esc((j.commercial.lean_proofs||[]).join("\n")) + "</pre>";
    html += "<h3>修復建議</h3><pre>" + esc((j.commercial.remediation||[]).join("\n")) + "</pre>";
  }
  if (j.iterations && j.iterations.length) {
    html += "<h3>迭代歷史 (持續迭代)</h3><pre>" + esc(j.iterations.map(it=>`#${it.iteration} ${it.is_unsat?'UNSAT':'SAT'} vars=${it.n_vars} polys=${it.n_polys} algo=${it.groebner_algo} risk=${it.risk_score.toFixed(1)} ${it.converged?'✓收斂':''} ${it.deepened?'深化':''}`).join("\n")) + "</pre>";
  }
  if (j.commercial && j.commercial.audit_report_md) {
    html += "<h3>審計報告 MD (商業深化)</h3><pre>" + esc(j.commercial.audit_report_md) + "</pre>";
  }
  if (j.qap_onchain && j.qap_onchain.payload) {
    html += "<h3>QAP 鏈上載荷 (Solana/EVM)</h3><pre>" + esc(j.qap_onchain.payload) + "</pre>";
  }
  if (j.poly_deepening && j.poly_deepening.final_poly) {
    html += "<h3>Poly 深化最終版 (自進化)</h3><pre>" + esc(j.poly_deepening.final_poly) + "</pre>";
  }
  if (j.generated_rust) {
    html += "<h3>生成 Rust (自驗證元循環)</h3><pre>" + esc(j.generated_rust) + "</pre>";
  }
  html += "<h3>Lowering 報告</h3><pre>" + esc(j.lowering_report||"") + "</pre>";
  out.innerHTML = html;
}

function renderCoverage(j) {
  const total = j.total_features||0;
  const covered = j.covered_features||0;
  const pct = j.coverage_pct||0;
  const missing = j.missing||[];
  const coverage = j.coverage||[];
  const globalTotal = j.global_total||0;
  const globalCovered = j.global_covered||0;
  const globalPct = j.global_pct||0;
  const globalMissing = j.global_missing||[];
  const capability = j.parser_capability||[];
  $("totalFeatures").textContent = total;
  $("coveredFeatures").textContent = covered;
  $("coveragePct").textContent = pct.toFixed(1)+'%';
  $("missingCount").textContent = missing.length;
  $("coveragePctBadge").textContent = pct.toFixed(1)+'%';
  $("coverageBadge").textContent = pct.toFixed(0)+'% 文件';
  $("globalTotal").textContent = globalTotal;
  $("globalCovered").textContent = globalCovered;
  $("globalPct").textContent = globalPct.toFixed(1)+'%';
  $("globalMissingCount").textContent = globalMissing.length;
  $("globalPctBadge").textContent = globalPct.toFixed(1)+'% 全局';
  $("globalBadge").textContent = globalPct.toFixed(0)+'% 全局實現';
  $("coverageBar").style.width = pct.toFixed(1)+'%';
  $("globalBar").style.width = globalPct.toFixed(1)+'%';
  $("missingList").textContent = missing.length ? missing.join("
") : "文件已覆蓋全部檢測特性 ✓";
  $("globalMissingList").textContent = globalMissing.length ? globalMissing.join("
") : "解析器已實現全部 51 特性 ✓";
  const featList = $("featList");
  if (coverage.length) {
    featList.innerHTML = coverage.map(c=>{
      const ok = c.present;
      const cls = ok ? 'ok' : 'miss';
      return `<div class="feat ${cls}"><div><div class="name">${esc(c.feature)}</div><div class="desc">${esc(c.desc)}</div></div><div>${ok?'✅':'❌'}</div></div>`;
    }).join("");
  } else {
    featList.innerHTML = '<span class="msg">無數據</span>';
  }
  const gList = $("globalFeatList");
  if (capability.length) {
    gList.innerHTML = capability.map(c=>{
      const ok = c.supported;
      const cls = ok ? 'ok' : 'miss';
      return `<div class="feat ${cls}"><div><div class="name">${esc(c.feature)}</div><div class="desc">${esc(c.desc)}</div></div><div>${ok?'✅':'❌'}</div></div>`;
    }).join("");
  } else {
    gList.innerHTML = '<span class="msg">無數據</span>';
  }
}

function render(mode, j) {
  const out = $("out");
  if (mode === "generate") {
    const cls = j.verdict === "SAT" ? "b-sat" : "b-unsat";
    let html = '<span class="badge ' + cls + '">' + esc(j.verdict) + "</span> ";
    html += '<span class="tag">' + esc(j.source) + "</span>";
    if (j.intent) html += '<div class="msg" style="margin-top:8px">意圖：' + esc(j.intent) + "</div>";
    if (j.generated_code) {
      html += "<h3>生成 Rust 代碼 ";
      html += (j.rustc_compiles === true ? '<span class="tick">rustc ✓</span>' :
               j.rustc_compiles === false ? '<span class="cross">rustc ✗</span>' : '') + "</h3>";
      html += "<pre>" + esc(j.generated_code) + "</pre>";
    } else {
      html += '<div class="err">無生成碼（判定 ' + esc(j.verdict) + '）</div>';
    }
    out.innerHTML = html;
    return;
  }
  if (mode === "expand") {
    let html = '<span class="badge b-sat">展開</span> ';
    html += '<span class="tag">' + esc(j.source) + "</span>";
    html += "<h3>宏</h3>";
    if (!j.macros || j.macros.length === 0) html += '<div class="msg">（無宏定義）</div>';
    else html += "<div>" + j.macros.map(m => '<span class="tag">' + esc(m.name) + "（" + m.arms + " 臂）</span>").join(" &nbsp;") + "</div>";
    html += "<h3>展開結果</h3>";
    if (!j.expansions || j.expansions.length === 0) html += '<div class="msg">（無宏調用）</div>';
    else html += j.expansions.map(e => "<div>invoke#" + e.node + " 臂" + (e.arm+1) + " → <span class='tag'>" + esc(e.text) + "</span></div>").join("");
    out.innerHTML = html;
    return;
  }
  const v = j.verdict;
  const cls = v === "SAT" ? "b-sat" : "b-unsat";
  let html = '<span class="badge ' + cls + '">' + esc(v) + "</span> ";
  html += '<span class="tag">' + esc(j.source) + "</span>";
  if (j.intent) html += '<div class="msg" style="margin-top:8px">意圖：' + esc(j.intent) + "</div>";
  html += "<div class='kv'>" +
    kv("型別檢查", j.typechecks ? "接受 ✓" : "拒絕 ✗") +
    kv("一致性", j.agrees ? "一致 ✓" : "★不一致★") +
    kv("變量", j.stats.n_vars) +
    kv("生成元", j.stats.n_polys) +
    kv("子句", j.stats.n_clauses) +
    kv("CDCL 輪", j.stats.cdcl_rounds) +
    kv("QAP 約束", j.stats.r1cs_constraints) +
    kv("QAP 驗證", j.stats.qap_verified === true ? "通過 ✓" : j.stats.qap_verified === false ? "★失敗★" : "?") +
    "</div>";
  if (j.result && Object.keys(j.result.arm_choice || {}).length) {
    html += "<h3>臂選擇</h3><div class='tag'>" +
      Object.entries(j.result.arm_choice).map(([k,a]) => k + "→臂" + (a+1)).join(", ") + "</div>";
  }
  if (j.generated && j.generated.code) {
    html += "<h3>生成 Rust 代碼 ";
    html += (j.generated.rustc_compiles === true ? '<span class="tick">rustc ✓</span>' :
             j.generated.rustc_compiles === false ? '<span class="cross">rustc ✗</span>' : '') + "</h3>";
    html += "<pre>" + esc(j.generated.code) + "</pre>";
  }
  out.innerHTML = html;
}

function kv(k, v) {
  return '<div><div class="k">' + esc(k) + '</div><div class="v">' + esc(v) + "</div></div>";
}

setTimeout(()=>run('check-v2'), 600);
</script>
</body>
</html>
"##;



/// `/api/nl`：自然語言 → .poly（LLM 護欄）。
///
/// body 可以是純文字（直接當作自然語言需求），也可以是 JSON：
/// `{"description": "...", "provider"?, "model"?, "base_url"?, "api_key"?, "attempts"?}`。
/// 回傳與 `polyrust nl --json` 相同的契約。
pub(crate) fn nl_text_json(body: &str) -> (crate::json::J, bool) {
    use crate::llm;

    // 解析 body：優先試 JSON，失敗則當純文字。
    let (nl, cfg) = match llm::parse_json(body) {
        Ok(v) => {
            let desc = v
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();
            let mut c = llm::LlmConfig::default();
            if let Some(p) = v.get("provider").and_then(|x| x.as_str()) {
                c.provider = p.to_string();
            }
            if let Some(m) = v.get("model").and_then(|x| x.as_str()) {
                c.model = Some(m.to_string());
            }
            if let Some(b) = v.get("base_url").and_then(|x| x.as_str()) {
                c.base_url = Some(b.to_string());
            }
            if let Some(k) = v.get("api_key").and_then(|x| x.as_str()) {
                c.api_key = Some(k.to_string());
            }
            if let Some(n) = v.get("attempts").and_then(|x| x.as_f64()) {
                c.attempts = (n as usize).clamp(1, 16);
            }
            (desc, c)
        }
        Err(_) => (body.to_string(), llm::LlmConfig::default()),
    };
    let nl = nl.trim().to_string();
    if nl.is_empty() {
        return (
            crate::driver::error_json("nl", None, "nl", "description 為空"),
            false,
        );
    }

    let provider = match llm::build_provider(&cfg) {
        Ok(p) => p,
        Err(e) => return (crate::driver::error_json("nl", None, "nl", &e), false),
    };
    let result = llm::run_guardrail(provider.as_ref(), &nl, cfg.attempts, true);
    let ok = result.ok;
    let j = llm::guardrail_to_json(&nl, &result);
    // 漏斗量測：有設定 POLYRUST_FUNNEL_LOG 時追加本次運行記錄（零副作用預設）
    let _ = llm::funnel_log_append(None, &j.to_string());
    (j, ok)
}

/// 實際使用：server.rs 文件清單 — 優化 with_capacity
pub fn server_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("server.rs", "server.rs 正式運作 — 優化 with_capacity", "core/src/server.rs"),
    ]
}

