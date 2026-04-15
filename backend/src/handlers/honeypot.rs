//! R22-16: 蜜罐端點
//!
//! 註冊常見攻擊目標路徑，任何存取立即建立 critical security_alert + 主動通知。
//! 回傳 404 不洩露蜜罐身份。
//! Gemini #2: 加入 per-IP 頻率限制，避免自動掃描器灌爆系統。

use axum::{
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    response::IntoResponse,
};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::constants::SEC_EVENT_HONEYPOT_HIT;
use crate::middleware::real_ip::extract_real_ip_with_trust;
use crate::services::{AuditService, SecurityNotification, SecurityNotifier};
use crate::AppState;

/// Gemini #2: 蜜罐 per-IP 頻率限制（同一 IP 每 5 分鐘只處理一次）
static HONEYPOT_THROTTLE: std::sync::LazyLock<DashMap<String, Instant>> =
    std::sync::LazyLock::new(DashMap::new);

const HONEYPOT_THROTTLE_SECS: u64 = 300; // 5 分鐘

fn should_process_honeypot(ip: &str) -> bool {
    let now = Instant::now();
    let window = Duration::from_secs(HONEYPOT_THROTTLE_SECS);
    if let Some(last) = HONEYPOT_THROTTLE.get(ip) {
        if now.duration_since(*last) < window {
            return false;
        }
    }
    HONEYPOT_THROTTLE.insert(ip.to_string(), now);
    // 防止 map 無限成長
    if HONEYPOT_THROTTLE.len() > 5_000 {
        let keys: Vec<String> = HONEYPOT_THROTTLE
            .iter()
            .filter(|e| now.duration_since(*e.value()) > window)
            .take(500)
            .map(|e| e.key().clone())
            .collect();
        for k in keys {
            HONEYPOT_THROTTLE.remove(&k);
        }
    }
    true
}

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

    // Gemini #2: 同一 IP 5 分鐘內重複存取不再 spawn DB/通知
    if !should_process_honeypot(&ip) {
        return StatusCode::NOT_FOUND;
    }

    let db = state.db.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        let _ = AuditService::log_security_event(
            &db,
            SEC_EVENT_HONEYPOT_HIT,
            None,
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
