// SPDX-License-Identifier: (AGPL-3.0-only OR LicenseRef-PolyRust-Commercial)
//! polyrust-http —— axum/tokio HTTP API 前端
//!
//! 與核心自帶的零依賴 `server.rs`（std-only TCP）平行存在：
//! 需要正式路由／併發／生態整合時用本前端；要零依賴部署時用核心 `polyrust serve`。
//!
//! 端點（契約與 `docs/LLM.md` 一致）：
//! * `GET  /health`     —— 載入狀態（含 Lean 嵌入自檢）
//! * `POST /api/nl`     —— 自然語言 → .poly 護欄（JSON 契約）
//! * `GET  /api/funnel?path=<ndjson>` —— 漏斗聚合
//!
//! 用法：`polyrust-http [port]`（預設 8090）
//!
//! 注意：`run_guardrail` 為阻塞呼叫（LLM 同步 IO），本前端直接在 handler 內執行——
//! 它是運維／整合用途的薄前端，不追求高併發。

use axum::{extract::Query, http::StatusCode, routing::get, routing::post, Json, Router};
use polyrust_core::{formal, llm};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct NlReq {
    description: String,
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    api_key: Option<String>,
    attempts: Option<usize>,
    do_gen: Option<bool>,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "crate": "polyrust-http",
        "core": "polyrust-core",
        "lean_embedded": formal::embedded(),
        "formal_report": formal::startup_report(),
    }))
}

async fn nl(Json(req): Json<NlReq>) -> (StatusCode, Json<serde_json::Value>) {
    if req.description.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "verdict": "ERROR",
                "final_reason": "description 不可為空"
            })),
        );
    }
    let mut cfg = llm::LlmConfig::default();
    if let Some(p) = req.provider {
        cfg.provider = p;
    }
    cfg.model = req.model;
    cfg.base_url = req.base_url;
    cfg.api_key = req.api_key;
    if let Some(a) = req.attempts {
        cfg.attempts = a.clamp(1, 16);
    }
    let provider = match llm::build_provider(&cfg) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "verdict": "ERROR",
                    "final_reason": e
                })),
            )
        }
    };
    let do_gen = req.do_gen.unwrap_or(true);
    let r = llm::run_guardrail(provider.as_ref(), &req.description, cfg.attempts, do_gen);
    let line = llm::guardrail_to_json(&req.description, &r).to_string();
    // 漏斗落盤（與核心 CLI/serve 同一條路：--funnel-log 或 POLYRUST_FUNNEL_LOG）
    let _ = llm::funnel_log_append(None, &line);
    let v: serde_json::Value =
        serde_json::from_str(&line).unwrap_or_else(|_| serde_json::json!({"status":"error"}));
    (StatusCode::OK, Json(v))
}

async fn funnel(Query(q): Query<HashMap<String, String>>) -> (StatusCode, Json<serde_json::Value>) {
    let path = q
        .get("path")
        .cloned()
        .unwrap_or_else(|| "output/guardrail-funnel.ndjson".to_string());
    match std::fs::read_to_string(&path) {
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": "error",
                "final_reason": format!("讀不到漏斗日誌 {}：{}", path, e)
            })),
        ),
        Ok(text) => match llm::funnel_from_json_text(&text) {
            Err(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"status":"error","final_reason": e})),
            ),
            Ok(stats) => {
                let s = llm::funnel_to_json(&stats).to_string();
                let v: serde_json::Value = serde_json::from_str(&s)
                    .unwrap_or_else(|_| serde_json::json!({"status":"error"}));
                (StatusCode::OK, Json(v))
            }
        },
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/nl", post(nl))
        .route("/api/funnel", get(funnel));
    println!("polyrust-http（axum）@ 0.0.0.0:{}", port);
    println!("  〔{}〕", formal::startup_report());
    let listener = match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("綁定埠號 {} 失敗：{}", port, e);
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("server 錯誤：{}", e);
        std::process::exit(1);
    }
}

// ── P1-D7：契約/回路測試（handler 直調，tower/網絡零需求） ────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn nl_req(description: &str, provider: Option<&str>) -> NlReq {
        NlReq {
            description: description.into(),
            provider: provider.map(|p| p.into()),
            model: None,
            base_url: None,
            api_key: None,
            attempts: None,
            do_gen: None,
        }
    }

    #[tokio::test]
    async fn health_reports_ok_and_crate() {
        let Json(v) = health().await;
        assert_eq!(v["ok"], serde_json::json!(true));
        assert_eq!(v["crate"], serde_json::json!("polyrust-http"));
        assert!(v.get("formal_report").is_some(), "health 契約應載 formal_report");
    }

    #[tokio::test]
    async fn nl_empty_description_is_400_error() {
        let (sc, Json(v)) = nl(Json(nl_req("   ", None))).await;
        assert_eq!(sc, StatusCode::BAD_REQUEST);
        assert_eq!(v["status"], serde_json::json!("error"));
        assert_eq!(v["verdict"], serde_json::json!("ERROR"));
    }

    #[tokio::test]
    async fn nl_unknown_provider_is_400_error() {
        let (sc, Json(v)) = nl(Json(nl_req("x", Some("__no_such_provider__")))).await;
        assert_eq!(sc, StatusCode::BAD_REQUEST);
        assert_eq!(v["status"], serde_json::json!("error"));
    }

    #[tokio::test]
    async fn nl_mock_provider_round_trip_is_200() {
        // mock provider 離線確定性（核心內建），可做完整回路不設網絡。
        let (sc, Json(v)) = nl(Json(nl_req("寫一個 i32 加法函數", Some("mock")))).await;
        assert_eq!(sc, StatusCode::OK, "mock 回路應 200，got {v}");
        assert!(v.get("status").is_some(), "契約應有 status 欄位: {v}");
    }

    #[tokio::test]
    async fn funnel_missing_file_is_404() {
        let mut q = HashMap::new();
        q.insert("path".into(), "/tmp/polyrust-test-no-such-funnel.ndjson".into());
        let (sc, Json(v)) = funnel(Query(q)).await;
        assert_eq!(sc, StatusCode::NOT_FOUND);
        assert_eq!(v["status"], serde_json::json!("error"));
    }

    #[tokio::test]
    async fn funnel_garbage_is_422_and_valid_line_is_200() {
        let bad = "/tmp/polyrust-test-funnel-bad.ndjson";
        std::fs::write(bad, "呢行唔係 JSON\n").unwrap();
        let mut q = HashMap::new();
        q.insert("path".into(), bad.to_string());
        let (sc, _) = funnel(Query(q)).await;
        assert_eq!(sc, StatusCode::UNPROCESSABLE_ENTITY, "垃圾行應 422");

        let good = "/tmp/polyrust-test-funnel-ok.ndjson";
        std::fs::write(good, "{\"status\":\"ok\",\"provider\":\"mock\"}\n").unwrap();
        let mut q2 = HashMap::new();
        q2.insert("path".into(), good.to_string());
        let (sc2, Json(v)) = funnel(Query(q2)).await;
        assert_eq!(sc2, StatusCode::OK, "合法 ndjson 應 200: {v}");
        assert_eq!(v["runs"], serde_json::json!(1), "單行日誌 runs 應為 1: {v}");
    }
}
