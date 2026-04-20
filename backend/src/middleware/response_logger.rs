//! R22-3: 安全回應記錄 middleware
//!
//! 攔截 403 Forbidden 回應，將 permission denied 事件寫入 user_activity_logs。
//! R22-6: 同一 IP 短時間內累積多次 403 → 產生 IDOR probe alert。

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::config::Config;
use crate::constants::{SEC_EVENT_AUTO_SUSPENDED, SEC_EVENT_PERMISSION_DENIED};
use crate::middleware::real_ip::extract_real_ip_with_trust;
use crate::middleware::CurrentUser;
use crate::services::{AlertThresholdService, AuditService, IpBlocklistService, SecurityNotifier, SecurityNotification};
use crate::AppState;

pub async fn security_response_logger(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_id = request
        .extensions()
        .get::<CurrentUser>()
        .map(|u| u.id);
    // R24-1: 取來源 IP 以供 403 稽核與 IDOR probe 自動封 IP 使用
    let ip = extract_real_ip_with_trust(
        request.headers(),
        &addr,
        state.config.trust_proxy_headers,
    );

    let response = next.run(request).await;

    if response.status() == StatusCode::FORBIDDEN {
        let db = state.db.clone();
        let config = state.config.clone();
        let ip_owned = ip.clone();
        tokio::spawn(async move {
            // Gemini #5: 使用 actor_user_id 欄位（有索引）而非 JSONB after_data
            let _ = AuditService::log_security_event(
                &db,
                SEC_EVENT_PERMISSION_DENIED,
                user_id,
                Some(&ip_owned),
                None,
                Some(&path),
                Some(&method),
                serde_json::json!({
                    "path": path,
                    "method": method,
                    "ip": ip_owned,
                }),
            )
            .await;

            // R22-6: IDOR probe detection (by user_id)
            if let Some(uid) = user_id {
                if let Err(e) =
                    check_idor_probe(&db, &config, &uid.to_string(), &ip_owned).await
                {
                    tracing::error!("[R22-6] IDOR probe check failed: {e}");
                }
            }
        });
    }

    response
}

/// R22-6: 檢查同一使用者短時間內是否累積過多 403，產生 IDOR probe alert
/// R24-1: 新增 ip 參數以支援同步封 IP
async fn check_idor_probe(
    pool: &PgPool,
    config: &Config,
    user_id_str: &str,
    ip: &str,
) -> std::result::Result<(), sqlx::Error> {
    let threshold = AlertThresholdService::idor_403_threshold(pool).await;
    let window_mins = AlertThresholdService::idor_403_window_mins(pool).await;
    let dedup_mins = AlertThresholdService::alert_escalation_dedup_mins(pool).await;

    // Gemini #4+5: 使用 actor_user_id（有索引）+ partition_date（partition pruning）
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_activity_logs
        WHERE event_type = 'PERMISSION_DENIED'
          AND actor_user_id = $1::uuid
          AND partition_date >= (NOW() - make_interval(mins => $2::integer))::date
          AND created_at > NOW() - make_interval(mins => $2::integer)
        "#,
    )
    .bind(user_id_str)
    .bind(window_mins as i32)
    .fetch_one(pool)
    .await?;

    if count < threshold {
        return Ok(());
    }

    let user_label: String = sqlx::query_as(
        "SELECT email FROM users WHERE id = $1::uuid",
    )
    .bind(user_id_str)
    .fetch_optional(pool)
    .await?
    .map(|(s,): (String,)| s)
    .unwrap_or_else(|| user_id_str.to_string());

    // Dedup
    let (existing,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM security_alerts
        WHERE alert_type = 'idor_probe'
          AND context_data->>'user_id' = $1
          AND created_at > NOW() - make_interval(mins => $2::integer)
          AND status = 'open'
        "#,
    )
    .bind(user_id_str)
    .bind(dedup_mins as i32)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Ok(());
    }

    let alert_id = Uuid::new_v4();
    let description = format!(
        "使用者 {user_label} 在過去 {window_mins} 分鐘內累積 {count} 次權限拒絕"
    );
    sqlx::query(
        r#"
        INSERT INTO security_alerts (
            id, alert_type, severity, title, description,
            context_data, created_at, updated_at, status
        ) VALUES (
            $1, 'idor_probe', 'critical',
            '偵測到可能的 IDOR 探測攻擊',
            $2, $3, NOW(), NOW(), 'open'
        )
        "#,
    )
    .bind(alert_id)
    .bind(&description)
    .bind(serde_json::json!({
        "user_id": user_id_str,
        "count": count,
        "window_mins": window_mins,
    }))
    .execute(pool)
    .await?;

    tracing::warn!("[R22-6] IDOR probe alert created for user {user_id_str}");

    // R22-6: 自動封鎖探測使用者（可透過 security_alert_config.idor_auto_block_enabled=0 關閉）
    if AlertThresholdService::idor_auto_block_enabled(pool).await {
        auto_block_user(pool, user_id_str, alert_id).await;
        // R24-1: 同步封鎖來源 IP 24h（防止攻擊者切換 user 繼續探測）
        IpBlocklistService::auto_block(
            pool,
            ip,
            "R22-6_idor",
            Some(alert_id),
            &format!(
                "IDOR 探測：使用者 {user_label} 在 {window_mins} 分內累積 {count} 次 403"
            ),
            Some(24),
        )
        .await;
    }

    let notification = SecurityNotification {
        alert_id,
        alert_type: "idor_probe".to_string(),
        severity: "critical".to_string(),
        title: "偵測到可能的 IDOR 探測攻擊".to_string(),
        description: Some(description),
        context_data: Some(serde_json::json!({ "user_id": user_id_str, "email": user_label, "count": count })),
        created_at: chrono::Utc::now(),
    };
    SecurityNotifier::dispatch(pool, config, &notification).await;

    Ok(())
}

/// R22-6: 封鎖 IDOR 探測使用者，設 is_active=false 並寫入稽核紀錄
async fn auto_block_user(pool: &PgPool, user_id_str: &str, alert_id: Uuid) {
    let result = sqlx::query(
        "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1::uuid AND is_active = true",
    )
    .bind(user_id_str)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 1 => {
            tracing::warn!("[R22-6] User {user_id_str} auto-suspended (alert {alert_id})");
            let user_id_uuid = uuid::Uuid::parse_str(user_id_str).ok();
            let _ = AuditService::log_security_event(
                pool,
                SEC_EVENT_AUTO_SUSPENDED,
                user_id_uuid,
                None,
                None,
                None,
                None,
                serde_json::json!({
                    "user_id": user_id_str,
                    "reason": "idor_probe",
                    "alert_id": alert_id,
                }),
            )
            .await;
        }
        Ok(_) => {
            tracing::debug!("[R22-6] User {user_id_str} already inactive, skip auto-suspend");
        }
        Err(e) => {
            tracing::error!("[R22-6] Failed to auto-suspend user {user_id_str}: {e}");
        }
    }
}
