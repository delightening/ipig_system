//! R22-16: 蜜罐端點
//!
//! 註冊常見攻擊目標路徑，任何存取立即建立 critical security_alert + 主動通知。
//! 回傳 404 不洩露蜜罐身份。

use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::constants::SEC_EVENT_HONEYPOT_HIT;
use crate::middleware::real_ip::extract_real_ip_with_trust;
use crate::services::{AuditService, SecurityNotification, SecurityNotifier};
use crate::AppState;

pub async fn honeypot_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<axum::body::Body>,
) -> impl IntoResponse {
    let ip = extract_real_ip_with_trust(
        request.headers(),
        &addr,
        state.config.trust_proxy_headers,
    );
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let db = state.db.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        // 記錄安全事件
        let _ = AuditService::log_security_event(
            &db,
            SEC_EVENT_HONEYPOT_HIT,
            Some(&ip),
            Some(&user_agent),
            Some(&path),
            Some(&method),
            serde_json::json!({
                "ip": ip,
                "path": path,
                "method": method,
                "user_agent": user_agent,
            }),
        )
        .await;

        // 建立 critical alert
        let alert_id = Uuid::new_v4();
        let description = format!("IP {ip} 存取蜜罐端點 {path}（UA: {user_agent}）");
        let _ = sqlx::query(
            r#"
            INSERT INTO security_alerts (
                id, alert_type, severity, title, description,
                context_data, created_at, updated_at, status
            ) VALUES (
                $1, 'honeypot_hit', 'critical',
                '蜜罐端點被觸發 — 可能的攻擊者探測',
                $2, $3, NOW(), NOW(), 'open'
            )
            "#,
        )
        .bind(alert_id)
        .bind(&description)
        .bind(serde_json::json!({
            "ip": ip,
            "path": path,
            "user_agent": user_agent,
        }))
        .execute(&db)
        .await;

        // 主動推送通知
        let notification = SecurityNotification {
            alert_id,
            alert_type: "honeypot_hit".to_string(),
            severity: "critical".to_string(),
            title: "蜜罐端點被觸發 — 可能的攻擊者探測".to_string(),
            description: Some(description),
            context_data: Some(serde_json::json!({ "ip": ip, "path": path })),
            created_at: chrono::Utc::now(),
        };
        SecurityNotifier::dispatch(&db, &config, &notification).await;
    });

    StatusCode::NOT_FOUND
}
