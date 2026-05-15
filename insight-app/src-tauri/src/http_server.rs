use axum::{
    extract::State as AxumState,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use log::{info, warn};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

use crate::classifier::ClassifierConfig;
use crate::db;
use crate::models::WebVisitRequest;

#[derive(Clone)]
struct HttpState {
    db_conn: Arc<Mutex<Connection>>,
    classifier: Arc<Mutex<ClassifierConfig>>,
}

async fn handle_health() -> &'static str {
    "ok"
}

async fn handle_web_visit(
    AxumState(state): AxumState<HttpState>,
    Json(payload): Json<WebVisitRequest>,
) -> StatusCode {
    if payload.url.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    // 跳过浏览器内部页面
    if payload.url.starts_with("chrome://")
        || payload.url.starts_with("edge://")
        || payload.url.starts_with("about:")
        || payload.url.starts_with("chrome-extension://")
    {
        return StatusCode::OK;
    }

    // 敏感 URL 过滤
    let classifier = match state.classifier.lock() {
        Ok(c) => c,
        Err(_) => return StatusCode::OK,
    };

    if classifier.is_sensitive_url(&payload.url) {
        info!("Skipped sensitive URL");
        return StatusCode::OK;
    }

    let domain = match extract_domain(&payload.url) {
        Some(d) => d,
        None => return StatusCode::BAD_REQUEST,
    };

    let title = payload.title.unwrap_or_default();
    // 插件发送毫秒时间戳，转为秒
    let ts = payload
        .timestamp
        .map(|t| if t > 1_000_000_000_000 { t / 1000 } else { t })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    // 插件发送毫秒 duration，转为秒
    let duration_secs = payload
        .duration
        .map(|d| if d > 1000 { d / 1000 } else { d })
        .unwrap_or(0);

    let db = state.db_conn.lock().unwrap();
    match db::upsert_web_visit(&db, &domain, &payload.url, &title, ts, duration_secs) {
        Ok(_) => {
            info!("Web visit: {domain} - {title}");
            StatusCode::OK
        }
        Err(e) => {
            warn!("Failed to store web visit: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn extract_domain(url: &str) -> Option<String> {
    let stripped = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let domain = stripped.split('/').next()?;
    let domain = domain.split(':').next()?; // 去除端口号
    let domain = domain.strip_prefix("www.").unwrap_or(domain);
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}

pub fn start_http_server(
    db_conn: Arc<Mutex<Connection>>,
    classifier: Arc<Mutex<ClassifierConfig>>,
    port: u16,
) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async move {
        let state = HttpState {
            db_conn,
            classifier,
        };

        let cors = CorsLayer::new()
            .allow_origin([
                "http://127.0.0.1:5678".parse().unwrap(),
                "http://localhost:5678".parse().unwrap(),
            ])
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE]);

        let app = Router::new()
            .route("/api/web-visit", post(handle_web_visit))
            .route("/api/health", get(handle_health))
            .layer(cors)
            .with_state(state);

        let addr = format!("127.0.0.1:{port}");
        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                info!("HTTP server listening on {addr}");
                if let Err(e) = axum::serve(listener, app).await {
                    warn!("HTTP server error: {e}");
                }
            }
            Err(e) => {
                warn!("Failed to bind HTTP server on {addr}: {e}");
            }
        }
    });
}
