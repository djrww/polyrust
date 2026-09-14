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

use crate::driver::{check_text_json, expand_text_json, generate_text_json};

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

/// 讀取一個 HTTP 請求（方法、路徑、body）。
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
<title>polyrust — 代數形式化驗證</title>
<style>
  :root {
    --bg: #0e1117; --panel: #161b26; --panel2: #1c2333; --border: #2a3348;
    --fg: #d7deea; --dim: #8591a8; --accent: #4ea1ff; --ok: #34d399;
    --bad: #f87171; --warn: #fbbf24; --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
  * { box-sizing: border-box; }
  body { margin: 0; font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
         background: var(--bg); color: var(--fg); line-height: 1.5; }
  header { padding: 18px 24px; border-bottom: 1px solid var(--border); background: var(--panel); }
  header h1 { margin: 0; font-size: 18px; font-weight: 650; }
  header p { margin: 4px 0 0; color: var(--dim); font-size: 13px; }
  .wrap { max-width: 1080px; margin: 0 auto; padding: 20px 24px; display: grid;
          grid-template-columns: 1fr 1fr; gap: 20px; }
  @media (max-width: 820px) { .wrap { grid-template-columns: 1fr; } }
  .card { background: var(--panel); border: 1px solid var(--border); border-radius: 10px; }
  .card h2 { margin: 0; padding: 12px 16px; font-size: 14px; font-weight: 650;
             border-bottom: 1px solid var(--border); color: var(--dim); text-transform: uppercase;
             letter-spacing: .04em; }
  textarea { width: 100%; height: 420px; background: var(--panel2); color: var(--fg);
             border: none; padding: 14px; font-family: var(--mono); font-size: 13px;
             resize: vertical; outline: none; }
  .btns { display: flex; gap: 10px; padding: 14px 16px; border-top: 1px solid var(--border); }
  button { flex: 1; padding: 10px 14px; border: 1px solid var(--border); border-radius: 8px;
           background: var(--panel2); color: var(--fg); font-size: 14px; font-weight: 600;
           cursor: pointer; }
  button.primary { background: var(--accent); border-color: var(--accent); color: #06121f; }
  button:hover { filter: brightness(1.1); }
  button:disabled { opacity: .5; cursor: not-allowed; }
  .out { padding: 16px; min-height: 200px; font-size: 13px; }
  .badge { display: inline-block; padding: 2px 10px; border-radius: 999px; font-weight: 700;
           font-size: 12px; letter-spacing: .03em; }
  .b-sat { background: rgba(52,211,153,.15); color: var(--ok); border: 1px solid var(--ok); }
  .b-unsat { background: rgba(248,113,113,.15); color: var(--bad); border: 1px solid var(--bad); }
  .b-err { background: rgba(251,191,36,.15); color: var(--warn); border: 1px solid var(--warn); }
  .kv { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px,1fr)); gap: 8px;
        margin: 12px 0; }
  .kv div { background: var(--panel2); border: 1px solid var(--border); border-radius: 6px;
            padding: 6px 10px; }
  .kv .k { color: var(--dim); font-size: 11px; }
  .kv .v { font-family: var(--mono); font-size: 14px; }
  pre { background: var(--panel2); border: 1px solid var(--border); border-radius: 8px;
        padding: 12px; overflow-x: auto; font-family: var(--mono); font-size: 12.5px; }
  .msg { color: var(--dim); }
  .err { color: var(--bad); }
  .ok { color: var(--ok); }
  .tick { color: var(--ok); } .cross { color: var(--bad); }
  h3 { font-size: 13px; color: var(--dim); margin: 14px 0 6px; }
  .tag { font-family: var(--mono); font-size: 12px; color: var(--dim); }
</style>
</head>
<body>
<header>
  <h1>polyrust · 代數形式化驗證</h1>
  <p>CDCL × Buchberger × QAP — 貼上 <span class="tag">.poly</span> 描述，驗證型別/借用並還原可編譯的 Rust 代碼</p>
</header>
<div class="wrap">
  <div class="card">
    <h2>輸入（.poly 描述）</h2>
    <textarea id="src" spellcheck="false"></textarea>
    <div class="btns">
      <button id="btnCheck" class="primary" onclick="run('check')">驗證 + 生成</button>
      <button id="btnExpand" onclick="run('expand')">純展開</button>
      <button id="btnGen" onclick="run('generate')">生成碼</button>
    </div>
  </div>
  <div class="card">
    <h2>結果</h2>
    <div class="out" id="out"><span class="msg">尚未執行。按「驗證 + 生成」開始。</span></div>
  </div>
</div>
<script>
const DEFAULT_SRC = [
"# @intent: 模板：計算 (x + K) 的平方與立方",
"# @import: basic",
"# @import: math",
"# @set init = 5",
"# @set k = 1",
"",
"fn main() {",
"    let x = {{init}};",
"    let y = sqr!(x + {{k}});",
"    let z = cube(x);",
"}",
""
].join("\n");

const $ = (id) => document.getElementById(id);
$("src").value = DEFAULT_SRC;

function esc(s) {
  return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;");
}

async function run(mode) {
  const out = $("out");
  const b1 = $("btnCheck"), b2 = $("btnExpand"), b3 = $("btnGen");
  b1.disabled = b2.disabled = b3.disabled = true;
  out.innerHTML = '<span class="msg">執行中…</span>';
  try {
    const url = mode === "generate" ? "/api/v1/generate" : "/api/" + mode;
    const resp = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "text/plain; charset=utf-8" },
      body: $("src").value,
    });
    const j = await resp.json();
    if (j.status === "error") {
      out.innerHTML = '<span class="badge b-err">錯誤</span><pre class="err">' + esc(j.error) + "</pre>";
      return;
    }
    render(mode, j);
  } catch (e) {
    out.innerHTML = '<span class="badge b-err">錯誤</span><pre class="err">' + esc(String(e)) + "</pre>";
  } finally {
    b1.disabled = b2.disabled = b3.disabled = false;
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
  // check
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
    (llm::guardrail_to_json(&nl, &result), ok)
}
