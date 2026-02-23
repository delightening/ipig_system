//! Prometheus 指標收集端點
//!
//! GET /metrics（公開端點，無需認證，供 Prometheus scrape）
//!
//! 提供以下指標：
//! - `http_requests_total` — HTTP 請求計數（按路徑、方法、狀態碼）
//! - `http_request_duration_seconds` — HTTP 請求延遲（直方圖）
//! - `db_pool_connections` — 資料庫連線池狀態

use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

/// 回傳 Prometheus 格式的指標文字
///
/// 使用 `metrics-exporter-prometheus` 的 `PrometheusHandle` 渲染指標
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    // 更新 DB Pool gauge 指標
    let pool = &state.db;
    metrics::gauge!("db_pool_connections_active").set(pool.size() as f64);
    metrics::gauge!("db_pool_connections_idle").set(pool.num_idle() as f64);

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
