//! LLM 整合層：**可插拔 provider 適配** + **`.poly` 護欄**（guardrail）。
//!
//! ## 設計（零依賴，std-only）
//!
//! 1. **任何 API 皆可接入**：
//!    - `openai`：OpenAI 官方與**所有 OpenAI 相容端點**（Groq、Together、
//!      OpenRouter、DeepSeek、Moonshot、vLLM、LM Studio、Ollama `/v1` …）
//!    - `anthropic`：Anthropic Messages API 原生適配
//!    - `gemini`：Google Generative Language API 原生適配
//!    - `mock`：內建測試用腳本 provider（離線、確定性）
//!    新增 provider = 實作 [`LlmProvider`] trait 一個 `chat` 方法。
//!
//! 2. **護欄性質**：LLM 只是「建議引擎」，唯一職責是把自然語言翻成 `.poly`；
//!    每一輪輸出都必須通過三道閘門才被接受：
//!    - 閘門 0（結構）：恰好一個 ```` ```poly ```` 圍欄代碼塊、含 `# @intent`
//!    - 閘門 1（語法）：`dsl::resolve`（metadata + import + Mini-Rust 解析）
//!    - 閘門 2（語義）：完整代數管線（CDCL × Buchberger × QAP）判定
//!    失敗時把**精確錯誤**（解析器錯誤／`checker_msg`／展開日誌）回餵 LLM
//!    要求修復，直到 `max_attempts` 用盡。LLM 產出永遠不被信任——
//!    通過形式化管線的才算數。

use std::cell::Cell;
use std::io::{Read, Write};

use crate::dsl;
use crate::json::J;
use crate::pipeline;

// ─────────────────────────────────────────────────────────────────────────────
// §1 極簡 JSON 讀取器（只讀不寫；寫側沿用 `json.rs` 的 `J`）
// ─────────────────────────────────────────────────────────────────────────────

/// JSON 值（讀取側）。`Bool` 與 `as_bool` 為完整 JSON 支援的一部分，
/// 現行 provider 回包解析暫未消費布林欄位，故允許未讀。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Val {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Val>),
    Obj(Vec<(String, Val)>),
}

impl Val {
    pub fn get(&self, key: &str) -> Option<&Val> {
        if let Val::Obj(pairs) = self {
            pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Val::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        if let Val::Num(n) = self {
            Some(*n)
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub fn as_bool(&self) -> Option<bool> {
        if let Val::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }
}

struct P<'a> {
    b: &'a [u8],
    i: usize,
}

/// 寬鬆但正確的遞迴下降 JSON 解析（僅本模組讀 LLM 回包用）。
pub fn parse_json(s: &str) -> Result<Val, String> {
    let mut p = P { b: s.as_bytes(), i: 0 };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.i != p.b.len() {
        return Err(format!("JSON 結尾有多餘字元（位置 {}）", p.i));
    }
    Ok(v)
}

impl<'a> P<'a> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }
    fn bump(&mut self) -> Option<u8> {
        let c = self.peek();
        if c.is_some() {
            self.i += 1;
        }
        c
    }
    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.i += 1;
            } else {
                break;
            }
        }
    }
    fn expect(&mut self, c: u8) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("JSON 期望 {:?}，位置 {}", c as char, self.i))
        }
    }
    fn value(&mut self) -> Result<Val, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.obj(),
            Some(b'[') => self.arr(),
            Some(b'"') => Ok(Val::Str(self.string()?)),
            Some(b't') => self.lit("true", Val::Bool(true)),
            Some(b'f') => self.lit("false", Val::Bool(false)),
            Some(b'n') => self.lit("null", Val::Null),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.num(),
            other => Err(format!("JSON 無法辨識 {:?}，位置 {}", other, self.i)),
        }
    }
    fn lit(&mut self, word: &str, v: Val) -> Result<Val, String> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(format!("JSON 字面量錯誤，位置 {}", self.i))
        }
    }
    fn obj(&mut self) -> Result<Val, String> {
        self.expect(b'{')?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Val::Obj(pairs));
        }
        loop {
            self.skip_ws();
            let k = self.string()?;
            self.skip_ws();
            self.expect(b':')?;
            let v = self.value()?;
            pairs.push((k, v));
            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(format!("JSON 物件分隔錯誤，位置 {}", self.i)),
            }
        }
        Ok(Val::Obj(pairs))
    }
    fn arr(&mut self) -> Result<Val, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Val::Arr(items));
        }
        loop {
            let v = self.value()?;
            items.push(v);
            self.skip_ws();
            match self.bump() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(format!("JSON 陣列分隔錯誤，位置 {}", self.i)),
            }
        }
        Ok(Val::Arr(items))
    }
    fn num(&mut self) -> Result<Val, String> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-') {
                self.i += 1;
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.b[start..self.i]).map_err(|e| e.to_string())?;
        s.parse::<f64>()
            .map(Val::Num)
            .map_err(|_| format!("JSON 數字錯誤：{}", s))
    }
    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let c = self
                .bump()
                .ok_or_else(|| "JSON 字串未閉合".to_string())?;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = self
                        .bump()
                        .ok_or_else(|| "JSON 跳脫未閉合".to_string())?;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'u' => {
                            let cp = self.hex4()?;
                            // 代理對（surrogate pair）
                            if (0xD800..0xDC00).contains(&cp) {
                                if self.peek() == Some(b'\\')
                                    && self.b.get(self.i + 1) == Some(&b'u')
                                {
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    let combined =
                                        0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    out.push(
                                        char::from_u32(combined)
                                            .ok_or("JSON 無效代理對")?,
                                    );
                                } else {
                                    return Err("JSON 孤立高代理".to_string());
                                }
                            } else {
                                out.push(char::from_u32(cp).ok_or("JSON 無效 \\u")?);
                            }
                        }
                        _ => return Err(format!("JSON 無效跳脫 \\{}", e as char)),
                    }
                }
                _ => {
                    // UTF-8 多字節：回退到字節邊界，取完整 char
                    self.i -= 1;
                    let rest = std::str::from_utf8(&self.b[self.i..])
                        .map_err(|_| "JSON 非 UTF-8".to_string())?;
                    let ch = rest.chars().next().ok_or("JSON 空字元")?;
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
        Ok(out)
    }
    fn hex4(&mut self) -> Result<u32, String> {
        let mut v: u32 = 0;
        for _ in 0..4 {
            let c = self.bump().ok_or("JSON \\u 不足 4 碼")? as char;
            v = v * 16
                + c.to_digit(16)
                    .ok_or_else(|| format!("JSON \\u 非法位元 {}", c))?;
        }
        Ok(v)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §2 HTTP 客戶端（std-only：http:// 裸 TCP；https:// 走系統 curl）
// ─────────────────────────────────────────────────────────────────────────────

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

/// POST JSON。`headers` 為額外標頭（不含 Host/Content-Length/Content-Type）。
pub fn http_post(url: &str, headers: &[(String, String)], body: &str) -> Result<HttpResponse, String> {
    if url.starts_with("http://") {
        http_post_raw(url, headers, body)
    } else if url.starts_with("https://") {
        http_post_curl(url, headers, body)
    } else {
        Err(format!("URL 必須以 http:// 或 https:// 開頭：{}", url))
    }
}

fn split_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .ok_or_else(|| format!("無效 URL：{}", url))?;
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match hostport.rfind(':') {
        Some(i) => (
            hostport[..i].to_string(),
            hostport[i + 1..]
                .parse::<u16>()
                .map_err(|_| format!("無效埠號：{}", hostport))?,
        ),
        None => (
            hostport.to_string(),
            if url.starts_with("https") { 443 } else { 80 },
        ),
    };
    Ok((host, port, path.to_string()))
}

fn http_post_raw(url: &str, headers: &[(String, String)], body: &str) -> Result<HttpResponse, String> {
    let (host, port, path) = split_url(url)?;
    let addr = format!("{}:{}", host, port);
    let mut stream = std::net::TcpStream::connect(&addr)
        .map_err(|e| format!("連線 {} 失敗：{}", addr, e))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(180)))
        .ok();
    let mut req = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        path,
        host,
        body.len()
    );
    for (k, v) in headers {
        req.push_str(&format!("{}: {}\r\n", k, v));
    }
    req.push_str("\r\n");
    req.push_str(body);
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("送出請求失敗：{}", e))?;

    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(e) => return Err(format!("讀取回應失敗：{}", e)),
        }
    }
    parse_http_response(&buf)
}

fn parse_http_response(buf: &[u8]) -> Result<HttpResponse, String> {
    let header_end = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("回應缺少標頭")?;
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("無法解析狀態行：{}", status_line))?;

    let mut content_length: Option<usize> = None;
    let mut chunked = false;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            let v = v.trim();
            if k == "content-length" {
                content_length = v.parse().ok();
            }
            if k == "transfer-encoding" && v.to_ascii_lowercase().contains("chunked") {
                chunked = true;
            }
        }
    }
    let body_bytes = &buf[header_end + 4..];
    let body = if chunked {
        decode_chunked(body_bytes)?
    } else if let Some(n) = content_length {
        String::from_utf8_lossy(&body_bytes[..n.min(body_bytes.len())]).to_string()
    } else {
        String::from_utf8_lossy(body_bytes).to_string()
    };
    Ok(HttpResponse { status, body })
}

fn decode_chunked(mut b: &[u8]) -> Result<String, String> {
    let mut out = Vec::with_capacity(8);
    loop {
        let nl = b
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("chunked：找不到大小行")?;
        let size_str = String::from_utf8_lossy(&b[..nl]);
        let size = usize::from_str_radix(size_str.trim(), 16)
            .map_err(|_| format!("chunked：無效塊大小 {:?}", size_str))?;
        b = &b[nl + 2..];
        if size == 0 {
            break;
        }
        if b.len() < size + 2 {
            return Err("chunked：塊被截斷".to_string());
        }
        out.extend_from_slice(&b[..size]);
        b = &b[size + 2..];
    }
    Ok(String::from_utf8_lossy(&out).to_string())
}

fn http_post_curl(url: &str, headers: &[(String, String)], body: &str) -> Result<HttpResponse, String> {
    let mut cmd = std::process::Command::new("curl");
    cmd.arg("-sS")
        .arg("--max-time")
        .arg("180")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-w")
        .arg("\n__POLYRUST_HTTP_STATUS__:%{http_code}")
        .arg(url);
    for (k, v) in headers {
        cmd.arg("-H").arg(format!("{}: {}", k, v));
    }
    cmd.arg("--data-binary")
        .arg("@-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("無法啟動 curl（https 需要）：{}", e))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(body.as_bytes())
            .map_err(|e| format!("寫入 curl stdin 失敗：{}", e))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("curl 執行失敗：{}", e))?;
    if !out.status.success() {
        return Err(format!(
            "curl 失敗：{}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let raw = String::from_utf8_lossy(&out.stdout).to_string();
    match raw.rfind("__POLYRUST_HTTP_STATUS__:") {
        Some(i) => {
            let status: u16 = raw[i + 25..].trim().parse().unwrap_or(0);
            let body = raw[..i].trim_end_matches('\n').to_string();
            Ok(HttpResponse { status, body })
        }
        None => Ok(HttpResponse {
            status: 0,
            body: raw,
        }),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §3 Provider 抽象與實作
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Msg {
    pub role: String,    // "user" / "assistant"
    pub content: String,
}

/// 任何 LLM API 的統一介面。新增供應商只需實作 `chat`。
pub trait LlmProvider {
    fn name(&self) -> String;
    /// 多輪對話（`system` 為系統提示，`msgs` 依序為對話史）。
    fn chat(&self, system: &str, msgs: &[Msg]) -> Result<String, String>;
}

/// OpenAI Chat Completions，以及**一切 OpenAI 相容端點**
/// （Groq／Together／OpenRouter／DeepSeek／Moonshot／vLLM／LM Studio／Ollama…）。
pub struct OpenAiCompat {
    pub base_url: String, // 結尾不含 /
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u64,
}

impl LlmProvider for OpenAiCompat {
    fn name(&self) -> String {
        format!("openai-compatible:{}@{}", self.model, self.base_url)
    }
    fn chat(&self, system: &str, msgs: &[Msg]) -> Result<String, String> {
        let mut messages = vec![J::obj(vec![
            ("role", J::s("system")),
            ("content", J::s(system)),
        ])];
        for m in msgs {
            messages.push(J::obj(vec![
                ("role", J::s(&m.role)),
                ("content", J::s(&m.content)),
            ]));
        }
        let req = J::obj(vec![
            ("model", J::s(&self.model)),
            ("messages", J::Arr(messages)),
            ("temperature", J::Float(self.temperature)),
            ("max_tokens", J::Int(self.max_tokens as i64)),
        ]);
        let s = req.to_string();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let headers = vec![("Authorization".to_string(), format!("Bearer {}", self.api_key))];

        // 免費端點常見瞬態失敗：429 限流、5xx、回包 content 為空。
        // 在 provider 層自動重試（最多 3 次），避免把瞬態問題當成硬失敗。
        let mut last_err = String::new();
        for attempt in 0..3usize {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_secs(3 * attempt as u64));
            }
            let resp = match http_post(&url, &headers, &s) {
                Ok(r) => r,
                Err(e) => {
                    last_err = format!("連線錯誤：{}", e);
                    continue;
                }
            };
            let v = match parse_json(&resp.body) {
                Ok(v) => v,
                Err(e) => {
                    last_err = format!("回包非 JSON：{}（body：{:.200}）", e, resp.body);
                    if resp.status >= 500 || resp.status == 429 {
                        continue;
                    }
                    return Err(last_err);
                }
            };
            if resp.status == 429 || resp.status >= 500 {
                let msg = v
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(&resp.body);
                last_err = format!("API 回傳 {}（限流/伺服器錯誤）：{:.200}", resp.status, msg);
                continue;
            }
            if resp.status < 200 || resp.status >= 300 {
                let msg = v
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(&resp.body);
                return Err(format!("API 回傳 {}：{:.300}", resp.status, msg));
            }
            let choice = v.get("choices").and_then(|c| match c {
                Val::Arr(a) => a.first(),
                _ => None,
            });
            let message = choice.and_then(|c| c.get("message"));
            // content 優先；免費模型有時把輸出放在 reasoning
            let content = message
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    message
                        .and_then(|m| m.get("reasoning"))
                        .and_then(|c| c.as_str())
                        .filter(|s| !s.trim().is_empty())
                });
            match content {
                Some(txt) => return Ok(txt.to_string()),
                None => {
                    last_err =
                        "回包 content 為空（免費模型常見瞬態現象）".to_string();
                    continue;
                }
            }
        }
        Err(format!("{}（已重試 3 次）", last_err))
    }
}

/// Anthropic Messages API 原生適配。
pub struct Anthropic {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u64,
}

impl LlmProvider for Anthropic {
    fn name(&self) -> String {
        format!("anthropic:{}", self.model)
    }
    fn chat(&self, system: &str, msgs: &[Msg]) -> Result<String, String> {
        let messages: Vec<J> = msgs
            .iter()
            .map(|m| {
                J::obj(vec![
                    ("role", J::s(&m.role)),
                    ("content", J::s(&m.content)),
                ])
            })
            .collect();
        let req = J::obj(vec![
            ("model", J::s(&self.model)),
            ("max_tokens", J::Int(self.max_tokens as i64)),
            ("temperature", J::Float(self.temperature)),
            ("system", J::s(system)),
            ("messages", J::Arr(messages)),
        ]);
        let s = req.to_string();
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let headers = vec![
            ("x-api-key".to_string(), self.api_key.clone()),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ];
        let resp = http_post(&url, &headers, &s)?;
        let v = parse_json(&resp.body).map_err(|e| format!("回包非 JSON：{}（body：{:.200}）", e, resp.body))?;
        if resp.status < 200 || resp.status >= 300 {
            let msg = v
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or(&resp.body);
            return Err(format!("API 回傳 {}：{:.300}", resp.status, msg));
        }
        v.get("content")
            .and_then(|c| match c {
                Val::Arr(a) => a.first(),
                _ => None,
            })
            .and_then(|blk| blk.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("回包缺 content[0].text：{:.200}", resp.body))
    }
}

/// Google Generative Language（Gemini）原生適配。
pub struct Gemini {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u64,
}

impl LlmProvider for Gemini {
    fn name(&self) -> String {
        format!("gemini:{}", self.model)
    }
    fn chat(&self, system: &str, msgs: &[Msg]) -> Result<String, String> {
        let contents: Vec<J> = msgs
            .iter()
            .map(|m| {
                let role = if m.role == "assistant" { "model" } else { "user" };
                J::obj(vec![
                    ("role", J::s(role)),
                    (
                        "parts",
                        J::Arr(vec![J::obj(vec![("text", J::s(&m.content))])]),
                    ),
                ])
            })
            .collect();
        let req = J::obj(vec![
            (
                "system_instruction",
                J::obj(vec![(
                    "parts",
                    J::Arr(vec![J::obj(vec![("text", J::s(system))])]),
                )]),
            ),
            ("contents", J::Arr(contents)),
            (
                "generationConfig",
                J::obj(vec![
                    ("temperature", J::Float(self.temperature)),
                    ("maxOutputTokens", J::Int(self.max_tokens as i64)),
                ]),
            ),
        ]);
        let s = req.to_string();
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            self.model,
            self.api_key
        );
        let resp = http_post(&url, &[], &s)?;
        let v = parse_json(&resp.body).map_err(|e| format!("回包非 JSON：{}（body：{:.200}）", e, resp.body))?;
        if resp.status < 200 || resp.status >= 300 {
            let msg = v
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or(&resp.body);
            return Err(format!("API 回傳 {}：{:.300}", resp.status, msg));
        }
        v.get("candidates")
            .and_then(|c| match c {
                Val::Arr(a) => a.first(),
                _ => None,
            })
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| match p {
                Val::Arr(a) => a.first(),
                _ => None,
            })
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("回包缺 candidates[0].content.parts[0].text：{:.200}", resp.body))
    }
}

/// 離線測試用：依序吐出預設腳本（最後一則循環使用）。
pub struct MockProvider {
    pub label: String,
    pub script: Vec<String>,
    idx: Cell<usize>,
}

impl MockProvider {
    pub fn new(label: &str, script: Vec<String>) -> Self {
        MockProvider {
            label: label.to_string(),
            script,
            idx: Cell::new(0),
        }
    }
}

impl LlmProvider for MockProvider {
    fn name(&self) -> String {
        format!("mock:{}", self.label)
    }
    fn chat(&self, _system: &str, _msgs: &[Msg]) -> Result<String, String> {
        let i = self.idx.get();
        let out = if self.script.is_empty() {
            String::new()
        } else {
            self.script[i.min(self.script.len() - 1)].clone()
        };
        self.idx.set(i + 1);
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §4 設定（旗標 + 環境變數）
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct LlmConfig {
    pub provider: String,   // openai / anthropic / gemini / ollama / mock / 自訂名
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub attempts: usize,
    pub temperature: f64,
    pub max_tokens: u64,
    /// mock provider 的腳本（測試用）
    pub mock_script: Vec<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        LlmConfig {
            provider: "openai".to_string(),
            model: None,
            base_url: None,
            api_key: None,
            attempts: 3,
            temperature: 0.2,
            max_tokens: 2048,
            mock_script: Vec::new(),
        }
    }
}

fn env_any(names: &[&str]) -> Option<String> {
    for n in names {
        if let Ok(v) = std::env::var(n) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

/// 依設定構造 provider。錯誤訊息說明缺了什麼（金鑰／端點）。
pub fn build_provider(cfg: &LlmConfig) -> Result<Box<dyn LlmProvider>, String> {
    let p = cfg.provider.to_ascii_lowercase();
    match p.as_str() {
        "mock" => Ok(Box::new(MockProvider::new("cli", cfg.mock_script.clone()))),
        "anthropic" | "claude" => {
            let api_key = cfg
                .api_key
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_API_KEY", "ANTHROPIC_API_KEY"]))
                .ok_or("anthropic 需要金鑰：--api-key 或環境變數 ANTHROPIC_API_KEY（或 POLYRUST_LLM_API_KEY）")?;
            let base_url = cfg
                .base_url
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_BASE_URL", "ANTHROPIC_BASE_URL"]))
                .unwrap_or_else(|| "https://api.anthropic.com".to_string());
            let model = cfg
                .model
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_MODEL", "ANTHROPIC_MODEL"]))
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
            Ok(Box::new(Anthropic {
                base_url,
                api_key,
                model,
                temperature: cfg.temperature,
                max_tokens: cfg.max_tokens,
            }))
        }
        "gemini" | "google" => {
            let api_key = cfg
                .api_key
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY"]))
                .ok_or("gemini 需要金鑰：--api-key 或環境變數 GEMINI_API_KEY／GOOGLE_API_KEY")?;
            let base_url = cfg
                .base_url
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_BASE_URL", "GEMINI_BASE_URL"]))
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string());
            let model = cfg
                .model
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_MODEL", "GEMINI_MODEL"]))
                .unwrap_or_else(|| "gemini-2.0-flash".to_string());
            Ok(Box::new(Gemini {
                base_url,
                api_key,
                model,
                temperature: cfg.temperature,
                max_tokens: cfg.max_tokens,
            }))
        }
        // openai / ollama / vllm / lmstudio / 任何自訂名 → OpenAI 相容
        other => {
            let default_base = match other {
                "openai" => Some("https://api.openai.com/v1"),
                "ollama" => Some("http://127.0.0.1:11434/v1"),
                "vllm" => Some("http://127.0.0.1:8000/v1"),
                "lmstudio" => Some("http://127.0.0.1:1234/v1"),
                _ => None,
            };
            let base_url = cfg
                .base_url
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_BASE_URL", "OPENAI_BASE_URL"]))
                .or_else(|| default_base.map(|s| s.to_string()))
                .ok_or(format!(
                    "自訂 provider '{}' 需要端點：--base-url 或 POLYRUST_LLM_BASE_URL（OpenAI 相容格式，例如 http://localhost:8000/v1）",
                    other
                ))?;
            let api_key = cfg
                .api_key
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_API_KEY", "OPENAI_API_KEY"]))
                .unwrap_or_else(|| "not-needed".to_string()); // 本地端點通常免金鑰
            let default_model = match other {
                "openai" => "gpt-4o-mini",
                "ollama" => "llama3.1",
                _ => "local-model",
            };
            let model = cfg
                .model
                .clone()
                .or_else(|| env_any(&["POLYRUST_LLM_MODEL", "OPENAI_MODEL"]))
                .unwrap_or_else(|| default_model.to_string());
            Ok(Box::new(OpenAiCompat {
                base_url,
                api_key,
                model,
                temperature: cfg.temperature,
                max_tokens: cfg.max_tokens,
            }))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §5 護欄（guardrail）：三道閘門 + 修復回餵迴圈
// ─────────────────────────────────────────────────────────────────────────────

/// 護欄系統提示：把 LLM 限定在「自然語言 → .poly」這一件事上。
pub fn guardrail_system_prompt() -> String {
    let ex1 = r#"```poly
# @intent: 計算整數平方（函式版本）
fn sqr(x: i32) -> i32 { x * x }
fn main() {
    let a = sqr(5);
    let b = sqr(a);
}
```"#;
    let ex2 = r#"```poly
# @intent: 用宏比較兩數並取較大者（多臂、型別導向）
macro_rules! pick_max { ($a:expr, $b:expr) => { if $a >= $b { $a } else { $b } } }
fn main() {
    let m = pick_max!(3, 7);
}
```"#;
    let ex3 = r#"```poly
# @intent: 「鍵值資料庫」的最小計算核心模型（簡化：用條件表達式模擬固定槽位的存與取，無真實 I/O）
fn lookup(key: i32, v1: i32, v2: i32) -> i32 {
    if key == 1 { v1 } else { if key == 2 { v2 } else { 0 } }
}
fn main() {
    let got = lookup(2, 100, 200);
}
```"#;
    format!(
        r#"你是 polyrust 形式化驗證管線的翻譯層。你的唯一職責：把使用者的自然語言需求翻譯成 polyrust 的 `.poly` 描述語言。你寫的程式會被一道形式化管線（代數約束 + Gröbner 基）逐位元驗證；通過才會被接受。

【輸出格式——硬性護欄】
1. 只輸出恰好一個圍欄代碼塊，圍欄標記為 ```poly
2. 代碼塊內只能有 .poly 源碼；不要輸出任何解釋、標題或其他 markdown
3. 第一行必須是 `# @intent: <一句話概括使用者需求>`
4. 收到「修復反饋」時，針對反饋修正並重新輸出完整代碼塊

【.poly 語法（Mini-Rust 可判定子集）】
- 註釋/元資料行以 `#` 開頭；`# @intent: ...` 必需
- 函式：`fn name(p1: T1, p2: T2) -> R {{ 體 }}`；必須有 `fn main() {{ ... }}`
- 型別只有七種：i32、bool、()、&i32、&mut i32、&bool、&mut bool
- 語句鏈：`let x = e;`（可 `let mut`、可標註 `let x: T = e;`）；每句以分號 `;` 結束；最後一個表達式是程式結果
- 條件：`if cond {{ e1 }} else {{ e2 }}`——兩分支都要有大括號，且必須有 else
- 運算子：算術 `+ - *`；比較 `< <= >= == !=`（結果 bool）；布林 `&&` 與一元 `!`；整數取負 `-e`
- 函式呼叫 `f(a, b)`；宏呼叫 `m!(a, b)`
- 宏定義：`macro_rules! m {{ ($e:expr) => {{ 模板 }}; ($e:expr, $f:expr) => {{ 模板 }} }}`
- 引用：`&x` / `&mut x` 只能作用於變數；解引用 `*e`；賦值 `x = v` 或 `*r = v`
- 字面量：整數（i32）、`true`/`false`（bool）、`()`
- 沒有：循環、字串、陣列、struct/enum、match、閉包、浮點數

【典型拒絕原因（管線會回 UNSAT）】
- 型別錯配：例如 `1 + true`、對 bool 做算術
- 重複可變借用：同一變數的兩個 `&mut` 同時活躍
- 宏的所有臂都定不出一致型別

【超大範圍需求的處理——落地化規則（重要）】
使用者的需求可能遠超 .poly 子集（例如「開發一個 GUI 平台」「寫一個數據庫」）。
此時**嚴禁**幻覺子集之外的語法：沒有循環、沒有陣列、沒有字串、沒有
struct/enum、沒有 match、沒有閉包、沒有模組、沒有 I/O、沒有浮點數。
正確做法：**抽取需求的計算核心**，寫一個子集內的最小代表程式：
- 反應式渲染/狀態管理 → 狀態更新後重新計算派生值（信號 = 函式）
- 數據庫/鍵值存儲 → 用條件表達式模擬固定槽位的存與取（查詢 = 比較）
- 審核/發佈流程 → 布林門控條件鏈（`&&` 組合各條件）
- 錄製/狀態機 → 開始/停止/計數的條件遞推
`# @intent` 必須誠實寫明這是「XX 的最小計算核心模型（簡化：…）」，
絕對不要假裝實現了完整系統。

【範例】
使用者：「計算 x 的平方」
{ex1}

使用者：「寫一個宏取兩個數的較大值」
{ex2}

使用者：「開發一個鍵值資料庫」（超大範圍需求 → 抽取計算核心）
{ex3}
"#
    )
}

/// 計算輸出中 ` ```poly ` 圍欄的數量（護欄：必須恰好一個）。
pub fn poly_fence_count(text: &str) -> usize {
    text.lines()
        .filter(|l| {
            let t = l.trim_start().to_ascii_lowercase();
            t.starts_with("```poly")
        })
        .count()
}

/// 從 LLM 輸出抽取 ```` ```poly ```` 圍欄塊；退路：任意圍欄但內容以 `# @intent` 開頭。
pub fn extract_poly_block(text: &str) -> Option<String> {
    // 優先是 ```poly 圍欄
    let mut rest = text;
    while let Some(start) = rest.find("```") {
        let after = &rest[start + 3..];
        let line_end = after.find('\n').unwrap_or(after.len());
        let tag = after[..line_end].trim();
        let body_start = start + 3 + line_end + 1;
        if body_start > rest.len() {
            break;
        }
        let body_rest = &rest[body_start.min(rest.len())..];
        let close = body_rest.find("```");
        if tag.eq_ignore_ascii_case("poly") {
            let body = match close {
                Some(c) => &body_rest[..c],
                None => body_rest,
            };
            return Some(body.trim().to_string());
        }
        // 記錄非 poly 圍欄作退路候選（稍後檢查）
        rest = &rest[start + 3..];
    }
    // 退路：第一個圍欄，且內容看起來是 .poly（以 # @intent 開頭）
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        let line_end = after.find('\n').unwrap_or(after.len());
        let body_start = start + 3 + line_end + 1;
        if body_start <= text.len() {
            let body_rest = &text[body_start..];
            if let Some(close) = body_rest.find("```") {
                let body = body_rest[..close].trim();
                if body.starts_with("# @intent") {
                    return Some(body.to_string());
                }
            }
        }
    }
    None
}

#[derive(Clone, Debug)]
pub struct Attempt {
    pub n: usize,
    /// 該輪結果（漏斗各層，由淺至深）：
    /// - `provider-error`：LLM 端失敗（未進入任何閘門）
    /// - `structure-error`：閘門 0 拒絕（圍欄塊/`@intent` 結構）
    /// - `syntax-error`：閘門 1 拒絕（`.poly` 元資料或 Mini-Rust 解析）
    /// - `checker-reject`：Tier-0 快篩拒絕（獨立檢查器 ⇒ 完整管線必 UNSAT）
    /// - `unresolved` / `unsat`：閘門 2（完整代數管線）錯誤 / 判定 UNSAT
    /// - `sat`：通過全部閘門
    pub outcome: String,
    /// 回餵給 LLM 的修復訊息（若有）
    pub feedback: Option<String>,
}

#[derive(Clone, Debug)]
pub struct GuardrailResult {
    pub ok: bool,
    pub verdict: String, // SAT / UNSAT / ERROR
    pub poly: Option<String>,
    pub checker_msg: String,
    pub attempts: Vec<Attempt>,
    pub generated_code: Option<String>,
    pub generated_file: Option<String>,
    pub rustc_compiles: Option<bool>,
    pub provider: String,
    pub final_reason: String,
}

/// 護欄主迴圈。`do_gen`：SAT 時是否順勢生成 Rust 代碼。
///
/// 三道閘門＋一層快篩（借鏡 `rlzl` 分層過濾哲學）：
/// 結構（圍欄塊 + @intent）→ 語法（dsl::resolve + 解析）→
/// **Tier-0 checker 快篩**（廉價直接推導；拒絕 ⇒ 完整管線必 UNSAT）→
/// 語義（完整代數管線）。
/// 每道失敗都把**精確錯誤**回餵給 LLM；任何閘門都過不了的輸出永遠不被接受。
pub fn run_guardrail(
    provider: &dyn LlmProvider,
    nl: &str,
    max_attempts: usize,
    do_gen: bool,
) -> GuardrailResult {
    let system = guardrail_system_prompt();
    let mut history: Vec<Msg> = vec![Msg {
        role: "user".to_string(),
        content: nl.to_string(),
    }];
    let mut attempts: Vec<Attempt> = Vec::new();
    let max = max_attempts.max(1);

    for n in 1..=max {
        let resp = match provider.chat(&system, &history) {
            Ok(r) => r,
            Err(e) => {
                attempts.push(Attempt {
                    n,
                    outcome: "provider-error".to_string(),
                    feedback: None,
                });
                return GuardrailResult {
                    ok: false,
                    verdict: "ERROR".to_string(),
                    poly: None,
                    checker_msg: String::new(),
                    attempts,
                    generated_code: None,
                    generated_file: None,
                    rustc_compiles: None,
                    provider: provider.name(),
                    final_reason: format!("provider 錯誤：{}", e),
                };
            }
        };

        // ── 閘門 0：結構 ──
        if poly_fence_count(&resp) > 1 {
            let fb = "輸出出現了多個 ```poly 代碼塊。必須只輸出恰好一個代碼塊（多個候選會讓驗證結果不明確）。請選定一個版本重新輸出。".to_string();
            attempts.push(Attempt {
                n,
                outcome: "structure-error".to_string(),
                feedback: Some(fb.clone()),
            });
            history.push(Msg { role: "assistant".to_string(), content: resp });
            history.push(Msg { role: "user".to_string(), content: fb });
            continue;
        }
        let src = match extract_poly_block(&resp) {
            Some(s) if !s.is_empty() && s.len() <= 65536 => s,
            Some(_) => {
                let fb = "代碼塊過長（上限 64 KB）或為空。請輸出恰好一個 ```poly 圍欄塊。".to_string();
                attempts.push(Attempt {
                    n,
                    outcome: "structure-error".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
                continue;
            }
            None => {
                let fb = "找不到 ```poly 圍欄代碼塊。你必須輸出恰好一個以 ```poly 開頭、``` 結束的代碼塊，塊內只能是 .poly 源碼，第一行為 # @intent: ...。".to_string();
                attempts.push(Attempt {
                    n,
                    outcome: "structure-error".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
                continue;
            }
        };
        if !src.contains("@intent") {
            let fb = "缺少 `# @intent:` 行。第一行必須是 `# @intent: <一句話概括使用者需求>`。".to_string();
            attempts.push(Attempt {
                n,
                outcome: "structure-error".to_string(),
                feedback: Some(fb.clone()),
            });
            history.push(Msg { role: "assistant".to_string(), content: resp });
            history.push(Msg { role: "user".to_string(), content: fb });
            continue;
        }

        // ── 閘門 1：語法（.poly 元資料 + import；Mini-Rust 程式體解析）──
        let poly = match dsl::resolve(&src, None) {
            Ok(p) => p,
            Err(e) => {
                let fb = format!(
                    "語法錯誤（.poly 元資料/模板層），解析器回報：{}\n請修正後重新輸出完整代碼塊。",
                    e
                );
                attempts.push(Attempt {
                    n,
                    outcome: "syntax-error".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
                continue;
            }
        };
        let prog = match crate::minirust::parse::Parser::parse_program(&poly.source) {
            Ok(p) => p,
            Err(e) => {
                // 偵測子集外語法：回餵時直接指向「落地化規則」，加速收斂
                let banned = [
                    "for ", "while ", "loop", "struct", "enum", "match ", "Vec", "String",
                    "println", "use ", "impl ", "trait ", "f32", "f64",
                ];
                let hit: Vec<&str> = banned.iter().copied().filter(|b| poly.source.contains(b)).collect();
                let hint = if hit.is_empty() {
                    String::new()
                } else {
                    format!(
                        "\n注意：偵測到子集之外的語法（{}）。.poly 沒有循環/結構體/字串/陣列/浮點/模块——若原需求很大，請按【落地化規則】抽取計算核心重寫，並在 @intent 誠實標明簡化。",
                        hit.join("、")
                    )
                };
                let fb = format!(
                    "語法錯誤（Mini-Rust 程式體），解析器回報：{}\n請修正後重新輸出完整代碼塊。{}",
                    e, hint
                );
                attempts.push(Attempt {
                    n,
                    outcome: "syntax-error".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
                continue;
            }
        };

        // ── Tier-0 快篩（借鏡 `rlzl` 的「廉價謂詞先行」哲學）──
        // 獨立檢查器直接推導；拒絕 ⟺ 完整代數管線必判 UNSAT
        // （窮舉一致性 oracle `polyrust exhaust` 與 T6 義務見證此等價）。
        // 故直接回餵檢查器的精確型別錯誤，跳過昂貴的 CDCL×Buchberger×QAP。
        {
            let mut exp0 = crate::minirust::macros::Expander::new(prog.macros.clone(), prog.next_id);
            if let Err(ce) = crate::minirust::checker::check_program(&prog, &mut exp0) {
                let fb = format!(
                    "Tier-0 檢查器拒絕（語義不可定型）：{}\n等價的完整代數管線對此程序必判 UNSAT。這是語義錯誤（例如型別錯配、重複可變借用）。請重新理解需求並重寫，而不是只做表面修改。",
                    ce
                );
                attempts.push(Attempt {
                    n,
                    outcome: "checker-reject".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
                continue;
            }
        }

        // ── 閘門 2：語義（完整代數管線：CDCL × Buchberger × QAP）──
        match pipeline::run_pipeline("nl", &poly.source, do_gen) {
            Err(e) => {
                let fb = format!("管線錯誤：{}\n請修正後重新輸出完整代碼塊。", e);
                attempts.push(Attempt {
                    n,
                    outcome: "unresolved".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
            }
            Ok(pr) if pr.is_unsat => {
                let log = if pr.expansion_log.is_empty() {
                    String::new()
                } else {
                    format!("；展開日誌：{}", pr.expansion_log.join("；"))
                };
                let fb = format!(
                    "形式化管線判定 UNSAT（拒絕）：{}{}\n這代表程式在語義上不可定型（例如型別錯配、重複可變借用）。請重新理解需求並重寫，而不是只做表面修改。",
                    pr.checker_msg, log
                );
                attempts.push(Attempt {
                    n,
                    outcome: "unsat".to_string(),
                    feedback: Some(fb.clone()),
                });
                history.push(Msg { role: "assistant".to_string(), content: resp });
                history.push(Msg { role: "user".to_string(), content: fb });
            }
            Ok(pr) => {
                attempts.push(Attempt {
                    n,
                    outcome: "sat".to_string(),
                    feedback: None,
                });
                return GuardrailResult {
                    ok: true,
                    verdict: "SAT".to_string(),
                    poly: Some(src),
                    checker_msg: pr.checker_msg,
                    attempts,
                    generated_code: pr.generated_code,
                    generated_file: pr.generated_file,
                    rustc_compiles: pr.rustc_compiles,
                    provider: provider.name(),
                    final_reason: format!("第 {} 輪通過三道閘門", n),
                };
            }
        }
    }

    let last = attempts.last();
    let outcome = last.map(|a| a.outcome.clone()).unwrap_or_default();
    GuardrailResult {
        ok: false,
        verdict: if outcome == "unsat" || outcome == "checker-reject" {
            "UNSAT"
        } else {
            "ERROR"
        }
        .to_string(),
        poly: None,
        checker_msg: String::new(),
        attempts,
        generated_code: None,
        generated_file: None,
        rustc_compiles: None,
        provider: provider.name(),
        final_reason: format!(
            "{} 輪護欄嘗試全部被拒絕（最後一輪：{}）",
            max, outcome
        ),
    }
}

/// 護欄結果 → JSON（`nl` 子命令與 `/api/nl` 的契約）。
pub fn guardrail_to_json(nl: &str, r: &GuardrailResult) -> J {
    let attempts: Vec<J> = r
        .attempts
        .iter()
        .map(|a| {
            J::obj(vec![
                ("n", J::Int(a.n as i64)),
                ("outcome", J::s(&a.outcome)),
                ("feedback", J::opt_str(a.feedback.as_deref())),
            ])
        })
        .collect();
    J::obj(vec![
        ("api_version", J::s("0.1")),
        ("mode", J::s("nl")),
        ("description", J::s(nl)),
        ("provider", J::s(&r.provider)),
        ("status", J::s(if r.ok { "ok" } else { "error" })),
        ("verdict", J::s(&r.verdict)),
        ("poly", J::opt_str(r.poly.as_deref())),
        ("checker_msg", J::s(&r.checker_msg)),
        ("attempts", J::Int(r.attempts.len() as i64)),
        ("attempt_log", J::Arr(attempts)),
        (
            "generated",
            J::obj(vec![
                ("code", J::opt_str(r.generated_code.as_deref())),
                ("file", J::opt_str(r.generated_file.as_deref())),
                ("rustc_compiles", J::opt_bool(r.rustc_compiles)),
            ]),
        ),
        ("final_reason", J::s(&r.final_reason)),
    ])
}

// ─────────────────────────────────────────────────────────────────────────────
// §5b 漏斗量測（借鏡 `rlzl` 「測量一切、誠實記錄」文化）
// ─────────────────────────────────────────────────────────────────────────────

/// 護欄漏斗聚合統計：多次 `nl` 護欄運行的 `attempt_log` 汇总。
///
/// 漏斗分層（由淺至深）：
/// 到達（扣 provider 錯誤）→ 結構閘門 → 語法閘門 → Tier-0 checker → 語義閘門（SAT）。
#[derive(Clone, Debug, Default)]
pub struct FunnelStats {
    pub runs: u64,
    pub ok_runs: u64,
    pub attempts_total: u64,
    pub provider_error: u64,
    pub structure_rejects: u64,
    pub syntax_rejects: u64,
    pub checker_rejects: u64,
    pub unsat_rejects: u64,
    pub unresolved: u64,
    pub sat: u64,
    /// 通過時所在輪次總和（÷ `sat` = 平均多少輪收斂）
    pub sat_attempt_sum: u64,
    /// (provider 名, 運行數, 成功數)
    pub providers: Vec<(String, u64, u64)>,
}

impl FunnelStats {
    fn bump_provider(&mut self, name: &str, ok: bool) {
        if let Some(e) = self.providers.iter_mut().find(|(p, _, _)| p == name) {
            e.1 += 1;
            if ok {
                e.2 += 1;
            }
        } else {
            self.providers
                .push((name.to_string(), 1, if ok { 1 } else { 0 }));
        }
    }

    fn bump_outcome(&mut self, outcome: &str, n: u64) {
        self.attempts_total += 1;
        match outcome {
            "provider-error" => self.provider_error += 1,
            "structure-error" => self.structure_rejects += 1,
            "syntax-error" => self.syntax_rejects += 1,
            "checker-reject" => self.checker_rejects += 1,
            "unsat" => self.unsat_rejects += 1,
            "unresolved" => self.unresolved += 1,
            "sat" => {
                self.sat += 1;
                self.sat_attempt_sum += n;
            }
            _ => {}
        }
    }

    /// 漏斗各層的「到達/通過」數（由淺至深）。
    pub fn funnel_rows(&self) -> Vec<(&'static str, u64)> {
        let reached = self.attempts_total.saturating_sub(self.provider_error);
        let pass_structure = reached.saturating_sub(self.structure_rejects);
        let pass_syntax = pass_structure.saturating_sub(self.syntax_rejects);
        let pass_tier0 = pass_syntax.saturating_sub(self.checker_rejects);
        vec![
            ("到達護欄（輪次，不含 provider 錯誤）", reached),
            ("→ 通過閘門 0 結構（圍欄 + @intent）", pass_structure),
            ("→ 通過閘門 1 語法（.poly + Mini-Rust 解析）", pass_syntax),
            ("→ 通過 Tier-0 checker 快篩（廉價謂詞）", pass_tier0),
            ("→ 通過閘門 2 語義（完整管線，SAT）", self.sat),
        ]
    }
}

/// 由護欄結果序列聚合漏斗統計。
#[allow(dead_code)] // 供單元測試與程式庫呼叫者使用（CLI 路徑走 NDJSON 聚合）
pub fn funnel_from_results(rs: &[GuardrailResult]) -> FunnelStats {
    let mut s = FunnelStats::default();
    for r in rs {
        s.runs += 1;
        if r.ok {
            s.ok_runs += 1;
        }
        s.bump_provider(&r.provider, r.ok);
        for a in &r.attempts {
            s.bump_outcome(&a.outcome, a.n as u64);
        }
    }
    s
}

/// 由 NDJSON 漏斗日誌（每行一個 `guardrail_to_json` 輸出）聚合統計。
pub fn funnel_from_json_text(text: &str) -> Result<FunnelStats, String> {
    let mut s = FunnelStats::default();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v = parse_json(line)
            .map_err(|e| format!("第 {} 行不是有效 JSON：{}", i + 1, e))?;
        let ok = v.get("status").and_then(|x| x.as_str()) == Some("ok");
        let provider = v
            .get("provider")
            .and_then(|x| x.as_str())
            .unwrap_or("?");
        s.runs += 1;
        if ok {
            s.ok_runs += 1;
        }
        s.bump_provider(provider, ok);
        if let Some(Val::Arr(log)) = v.get("attempt_log") {
            for a in log {
                let outcome = a
                    .get("outcome")
                    .and_then(|x| x.as_str())
                    .unwrap_or("?");
                let n = a.get("n").and_then(|x| x.as_f64()).unwrap_or(0.0) as u64;
                s.bump_outcome(outcome, n);
            }
        }
    }
    Ok(s)
}

/// 漏斗統計 → JSON（`funnel` 子命令契約）。
pub fn funnel_to_json(s: &FunnelStats) -> J {
    let providers: Vec<J> = s
        .providers
        .iter()
        .map(|(p, runs, ok)| {
            J::obj(vec![
                ("provider", J::s(p)),
                ("runs", J::Int(*runs as i64)),
                ("ok", J::Int(*ok as i64)),
            ])
        })
        .collect();
    let rows: Vec<J> = s
        .funnel_rows()
        .into_iter()
        .map(|(label, v)| J::obj(vec![("gate", J::s(label)), ("count", J::Int(v as i64))]))
        .collect();
    J::obj(vec![
        ("api_version", J::s("0.1")),
        ("mode", J::s("funnel")),
        ("status", J::s("ok")),
        ("runs", J::Int(s.runs as i64)),
        ("ok_runs", J::Int(s.ok_runs as i64)),
        ("attempts_total", J::Int(s.attempts_total as i64)),
        (
            "outcomes",
            J::obj(vec![
                ("provider-error", J::Int(s.provider_error as i64)),
                ("structure-error", J::Int(s.structure_rejects as i64)),
                ("syntax-error", J::Int(s.syntax_rejects as i64)),
                ("checker-reject", J::Int(s.checker_rejects as i64)),
                ("unsat", J::Int(s.unsat_rejects as i64)),
                ("unresolved", J::Int(s.unresolved as i64)),
                ("sat", J::Int(s.sat as i64)),
            ]),
        ),
        ("funnel", J::Arr(rows)),
        ("providers", J::Arr(providers)),
        (
            "avg_attempts_to_sat",
            if s.sat > 0 {
                J::Float(s.sat_attempt_sum as f64 / s.sat as f64)
            } else {
                J::Null
            },
        ),
    ])
}

/// 把本次護欄運行的 JSON 行追加到漏斗日誌（NDJSON）。
///
/// 路徑優先序：顯式參數 > 環境變數 `POLYRUST_FUNNEL_LOG`；都沒有則不寫
/// （預設零副作用）。
pub fn funnel_log_append(explicit: Option<&str>, json_line: &str) -> Result<(), String> {
    let path = match explicit
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("POLYRUST_FUNNEL_LOG").ok())
        .filter(|s| !s.is_empty())
    {
        Some(p) => p,
        None => return Ok(()),
    };
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("無法開啟漏斗日誌 {}：{}", path, e))?;
    writeln!(f, "{}", json_line).map_err(|e| format!("寫入漏斗日誌失敗：{}", e))?;
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// §6 測試（離線：MockProvider + 護欄邏輯 + JSON 讀取器）
// ─────────────────────────────────────────────────────────────────────────────

/// 實際使用：llm.rs 文件清單 — 優化 with_capacity
pub fn llm_file_list() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("llm.rs", "llm.rs 正式運作 — 優化 with_capacity", "core/src/llm.rs"),
    ]
}

