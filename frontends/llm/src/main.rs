// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! polyrust-nl —— ureq 原生 TLS 傳輸的 LLM 前端
//!
//! 核心（`polyrust-core`）的 LLM 傳輸走「裸 TCP（http）＋系統 curl（https）」，
//! 零依賴但依賴主機有 curl。本前端用 `ureq`（純 Rust，rustls）實作同一條
//! `LlmProvider` 契約，**護欄邏輯仍全部來自核心**——只是換了傳輸層：
//!
//! ```text
//! polyrust-nl "<需求>" --base-url https://openrouter.ai/api/v1 \
//!     --model <slug> [--provider openrouter] [--api-key sk-...] \
//!     [--attempts 3] [--json] [--funnel-log path] [--no-gen]
//! ```
//!
//! 非 OpenAI 相容供應商（anthropic/gemini/mock）自動回落到核心的
//! `build_provider`（curl 傳輸），功能不缺。

use polyrust_core::json::J;
use polyrust_core::llm::{self, LlmProvider, Msg};
use std::time::Duration;

/// ureq 版 OpenAI Chat Completions（覆蓋一切 OpenAI 相容端點）。
struct UreqOpenAi {
    base_url: String,
    api_key: String,
    model: String,
    temperature: f64,
    max_tokens: u64,
}

impl LlmProvider for UreqOpenAi {
    fn name(&self) -> String {
        format!("ureq-openai-compatible:{}@{}", self.model, self.base_url)
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
        ])
        .to_string();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        // 免費端點常見瞬態失敗：429 限流、5xx、回包 content 為空——重試 3 次。
        let mut last_err = String::new();
        for attempt in 0..3usize {
            if attempt > 0 {
                std::thread::sleep(Duration::from_secs(3 * attempt as u64));
            }
            let (status, body) = match ureq::post(&url)
                .set("Content-Type", "application/json")
                .set("Authorization", &format!("Bearer {}", self.api_key))
                .timeout(Duration::from_secs(180))
                .send_string(&req)
            {
                Ok(r) => {
                    let s = r.status();
                    (s, r.into_string().unwrap_or_default())
                }
                Err(ureq::Error::Status(code, r)) => (code, r.into_string().unwrap_or_default()),
                Err(e) => {
                    last_err = format!("連線錯誤：{}", e);
                    continue;
                }
            };
            let v = match llm::parse_json(&body) {
                Ok(v) => v,
                Err(e) => {
                    last_err = format!("回包非 JSON：{}（body：{:.200}）", e, body);
                    if status >= 500 || status == 429 {
                        continue;
                    }
                    return Err(last_err);
                }
            };
            if status == 429 || status >= 500 {
                let msg = v
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(&body);
                last_err = format!("API 回傳 {}（限流/伺服器錯誤）：{:.200}", status, msg);
                continue;
            }
            if !(200..300).contains(&status) {
                let msg = v
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(&body);
                return Err(format!("API 回傳 {}：{:.300}", status, msg));
            }
            let choice = v.get("choices").and_then(|c| match c {
                llm::Val::Arr(a) => a.first(),
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
                    last_err = "回包 content 為空（免費模型常見瞬態現象）".to_string();
                    continue;
                }
            }
        }
        Err(format!("{}（已重試 3 次）", last_err))
    }
}

fn env_any(names: &[&str]) -> Option<String> {
    names
        .iter()
        .filter_map(|n| std::env::var(n).ok())
        .find(|v| !v.is_empty())
}

/// OpenAI 相容 → 本前端 ureq 傳輸；其餘回落核心 `build_provider`。
fn build_frontend_provider(cfg: &llm::LlmConfig) -> Result<Box<dyn LlmProvider>, String> {
    let is_openai_compat = matches!(
        cfg.provider.as_str(),
        "openai" | "ollama" | "vllm" | "lmstudio" | "openrouter" | "groq" | "deepseek"
    ) || !matches!(cfg.provider.as_str(), "anthropic" | "gemini" | "mock");
    if cfg.provider == "mock" || !is_openai_compat {
        return llm::build_provider(cfg);
    }
    let default_base = match cfg.provider.as_str() {
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
        .ok_or_else(|| {
            format!(
                "自訂 provider '{}' 需要端點：--base-url 或 POLYRUST_LLM_BASE_URL",
                cfg.provider
            )
        })?;
    let api_key = cfg
        .api_key
        .clone()
        .or_else(|| env_any(&["POLYRUST_LLM_API_KEY", "OPENAI_API_KEY"]))
        .unwrap_or_else(|| "not-needed".to_string());
    let model = cfg
        .model
        .clone()
        .or_else(|| env_any(&["POLYRUST_LLM_MODEL", "OPENAI_MODEL"]))
        .unwrap_or_else(|| "local-model".to_string());
    Ok(Box::new(UreqOpenAi {
        base_url,
        api_key,
        model,
        temperature: cfg.temperature,
        max_tokens: cfg.max_tokens,
    }))
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json = args.iter().any(|a| a == "--json" || a == "-j");
    let no_gen = args.iter().any(|a| a == "--no-gen");

    const VALUE_FLAGS: &[&str] = &[
        "--provider",
        "--model",
        "--base-url",
        "--api-key",
        "--attempts",
        "--funnel-log",
    ];
    let positional: Vec<String> = {
        let mut v = Vec::new();
        let mut skip = false;
        for a in args.iter().skip(1) {
            if skip {
                skip = false;
                continue;
            }
            if a.starts_with("--") {
                if VALUE_FLAGS.contains(&a.as_str()) {
                    skip = true;
                }
                continue;
            }
            if a == "-j" {
                continue;
            }
            v.push(a.clone());
        }
        v
    };
    let raw = positional
        .first()
        .cloned()
        .unwrap_or_else(|| "-".to_string());
    let nl = if raw == "-" {
        let mut buf = String::new();
        if std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).is_err() {
            eprintln!("讀取 stdin 失敗");
            std::process::exit(1);
        }
        buf
    } else {
        raw
    };

    let mut cfg = llm::LlmConfig::default();
    cfg.provider = "openrouter".to_string(); // 前端預設走 OpenAI 相容自訂端點
    if let Some(p) = flag(&args, "--provider") {
        cfg.provider = p.to_string();
    }
    cfg.model = flag(&args, "--model").map(|s| s.to_string());
    cfg.base_url = flag(&args, "--base-url").map(|s| s.to_string());
    cfg.api_key = flag(&args, "--api-key").map(|s| s.to_string());
    if let Some(a) = flag(&args, "--attempts").and_then(|s| s.parse::<usize>().ok()) {
        cfg.attempts = a.clamp(1, 16);
    }

    let provider = match build_frontend_provider(&cfg) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("provider 設定錯誤：{}", e);
            std::process::exit(1);
        }
    };
    if !json {
        println!("□ provider：{}", provider.name());
        println!("□ 護欄：三道閘門 + Tier-0 快篩（邏輯來自 polyrust-core）");
    }

    let result = llm::run_guardrail(provider.as_ref(), &nl, cfg.attempts, !no_gen);
    let line = llm::guardrail_to_json(&nl, &result).to_string();
    if let Some(path) = flag(&args, "--funnel-log") {
        if let Err(e) = llm::funnel_log_append(Some(path), &line) {
            eprintln!("漏斗日誌寫入失敗：{}", e);
        }
    } else {
        let _ = llm::funnel_log_append(None, &line);
    }

    if json {
        println!("{}", line);
    } else if result.ok {
        println!("□ 判定：SAT — 護欄通過（ureq 傳輸）");
    } else {
        println!("□ 判定：{} — {}", result.verdict, result.final_reason);
    }
    std::process::exit(if result.ok { 0 } else { 1 });
}
