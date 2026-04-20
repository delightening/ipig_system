//! R24-3: Alertmanager → R22 SecurityNotifier 轉發
//!
//! 接收 Alertmanager webhook payload（infra metric alert），轉譯為 SecurityNotification
//! 後呼叫 `SecurityNotifier::dispatch()`，複用既有 Email/LINE/Webhook 通知管道。
//!
//! 路徑：POST /api/webhooks/alertmanager（於 /api/v1 外層，不經 auth/csrf）
//! 防護：ALERTMANAGER_WEBHOOK_TOKEN 環境變數（X-Webhook-Token header 匹配才接受）

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::services::{SecurityNotification, SecurityNotifier};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AlertmanagerPayload {
    #[serde(default)]
    pub status: String, // "firing" or "resolved"
    #[serde(default)]
    pub alerts: Vec<AlertmanagerAlert>,
    #[serde(default)]
    pub group_key: String,
    #[serde(default)]
    pub common_labels: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct AlertmanagerAlert {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub labels: serde_json::Value,
    #[serde(default)]
    pub annotations: serde_json::Value,
    #[serde(default)]
    pub starts_at: String,
}

pub async fn alertmanager_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AlertmanagerPayload>,
) -> StatusCode {
    // 共享 token 防護：若環境變數未設定，視為不啟用（allow all）
    // 支援兩種 header：Authorization: Bearer <token>（Alertmanager 預設）或 X-Webhook-Token（自定義）
    if let Some(expected) = state.config.alertmanager_webhook_token.as_deref() {
        let bearer = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .unwrap_or("");
        let custom = headers
            .get("x-webhook-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if bearer != expected && custom != expected {
            tracing::warn!("[R24-3] Alertmanager webhook: invalid token");
            return StatusCode::UNAUTHORIZED;
        }
    }

    // 忽略 resolved 通知（避免告警 + 解除都通知一次）
    if payload.status == "resolved" {
        return StatusCode::NO_CONTENT;
    }

    // 遍歷每一筆 alert → 呼叫 SecurityNotifier
    let db = state.db.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        for alert in payload.alerts {
            let labels = &alert.labels;
            let severity = labels
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("warning")
                .to_string();
            let alertname = labels
                .get("alertname")
                .and_then(|v| v.as_str())
                .unwrap_or("UnknownAlert")
                .to_string();
            let summary = alert
                .annotations
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let description = alert
                .annotations
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let notification = SecurityNotification {
                alert_id: Uuid::new_v4(),
                alert_type: format!("infra_{alertname}"),
                severity,
                title: if summary.is_empty() {
                    format!("Infra 告警：{alertname}")
                } else {
                    summary
                },
                description,
                context_data: Some(serde_json::json!({
                    "source": "alertmanager",
                    "labels": labels,
                    "annotations": alert.annotations,
                })),
                created_at: chrono::Utc::now(),
            };
            SecurityNotifier::dispatch(&db, &config, &notification).await;
        }
    });

    StatusCode::OK
}
