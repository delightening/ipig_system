use axum::{body::Bytes, http::StatusCode};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
struct CspReportWrapper {
    #[serde(rename = "csp-report")]
    csp_report: CspViolation,
}

#[derive(Deserialize)]
struct CspViolation {
    #[serde(rename = "document-uri")]
    document_uri: Option<String>,
    #[serde(rename = "violated-directive")]
    violated_directive: Option<String>,
    #[serde(rename = "blocked-uri")]
    blocked_uri: Option<String>,
}

/// R25-3: 接收瀏覽器 CSP violation report（Content-Type: application/csp-report）
/// 不需認證，不需 CSRF — 瀏覽器以匿名方式送出。
/// 回傳 204 No Content（瀏覽器期望此狀態碼）。
pub async fn csp_report_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    body: Bytes,
) -> StatusCode {
    let Ok(wrapper) = serde_json::from_slice::<CspReportWrapper>(&body) else {
        return StatusCode::NO_CONTENT;
    };
    let v = &wrapper.csp_report;
    tracing::warn!(
        event = "csp_violation",
        document_uri = ?v.document_uri,
        violated_directive = ?v.violated_directive,
        blocked_uri = ?v.blocked_uri,
    );
    let context = serde_json::json!({
        "document_uri": v.document_uri,
        "violated_directive": v.violated_directive,
        "blocked_uri": v.blocked_uri,
    });
    let _ = sqlx::query(
        "INSERT INTO security_alerts (alert_type, severity, title, context_data) \
         VALUES ('CSP_VIOLATION', 'info', 'CSP violation reported', $1)",
    )
    .bind(context)
    .execute(&state.db)
    .await;
    StatusCode::NO_CONTENT
}
