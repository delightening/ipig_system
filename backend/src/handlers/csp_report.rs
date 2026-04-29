use axum::{body::Bytes, extract::Query, http::StatusCode};
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

/// R31-1: 區分 enforce vs report-only 違規來源
/// nginx 以 `?mode=ro` 標記 Report-Only header 的 report-uri
#[derive(Deserialize, Default)]
pub struct CspReportQuery {
    #[serde(default)]
    mode: Option<String>,
}

/// R25-3: 接收瀏覽器 CSP violation report（Content-Type: application/csp-report）
/// 不需認證，不需 CSRF — 瀏覽器以匿名方式送出。
/// 回傳 204 No Content（瀏覽器期望此狀態碼）。
///
/// R31-1 補強：query `?mode=ro` 用於區分 Report-Only header 觸發的違規（基準掃描期間並存兩個 CSP）
pub async fn csp_report_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(q): Query<CspReportQuery>,
    body: Bytes,
) -> StatusCode {
    let Ok(wrapper) = serde_json::from_slice::<CspReportWrapper>(&body) else {
        return StatusCode::NO_CONTENT;
    };
    let v = &wrapper.csp_report;
    let report_only = q.mode.as_deref() == Some("ro");
    tracing::warn!(
        event = "csp_violation",
        report_only,
        document_uri = ?v.document_uri,
        violated_directive = ?v.violated_directive,
        blocked_uri = ?v.blocked_uri,
    );
    let context = serde_json::json!({
        "document_uri": v.document_uri,
        "violated_directive": v.violated_directive,
        "blocked_uri": v.blocked_uri,
        "report_only": report_only,
    });
    let alert_type = if report_only {
        "CSP_VIOLATION_REPORT_ONLY"
    } else {
        "CSP_VIOLATION"
    };
    let _ = sqlx::query(
        "INSERT INTO security_alerts (alert_type, severity, title, context_data) \
         VALUES ($1, 'info', 'CSP violation reported', $2)",
    )
    .bind(alert_type)
    .bind(context)
    .execute(&state.db)
    .await;
    StatusCode::NO_CONTENT
}
