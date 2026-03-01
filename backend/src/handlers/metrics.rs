//! Prometheus 與 Web Vitals 指標收集端點
//!
//! - GET /metrics（公開端點，供 Prometheus scrape）
//! - POST /api/metrics/vitals（前端 Web Vitals 上報，需認證）
//!
//! Prometheus 提供：
//! - `http_requests_total` — HTTP 請求計數
//! - `http_request_duration_seconds` — 請求延遲
//! - `db_pool_connections` — 連線池狀態

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::AppState;

/// Web Vitals 上報 payload（與 web-vitals 的 Metric 對應）
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct WebVitalsMetric {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub rating: Option<String>,
    pub delta: f64,
    #[serde(rename = "navigationType")]
    pub navigation_type: Option<String>,
}

/// POST /api/metrics/vitals — 接收前端 Web Vitals 指標
///
/// 開發環境可記錄至日誌；正式環境可轉送 APM 或儲存。
pub async fn vitals_handler(Json(_metric): Json<WebVitalsMetric>) -> impl IntoResponse {
    tracing::debug!(
        name = %_metric.name,
        value = %_metric.value,
        "Web Vitals metric received"
    );
    StatusCode::NO_CONTENT
}

/// 回傳 Prometheus 格式的指標文字
///
/// 使用 `metrics-exporter-prometheus` 的 `PrometheusHandle` 渲染指標
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let pool = &state.db;
    let pool_size = pool.size() as f64;
    let pool_idle = pool.num_idle() as f64;
    metrics::gauge!("db_pool_connections_total").set(pool_size);
    metrics::gauge!("db_pool_connections_idle").set(pool_idle);
    metrics::gauge!("db_pool_connections_active").set(pool_size - pool_idle);

    // 從 PrometheusHandle 渲染指標
    let handle = state.metrics_handle.as_ref();

    match handle {
        Some(h) => (
            StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            h.render(),
        )
            .into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, "Metrics not available").into_response(),
    }
}
