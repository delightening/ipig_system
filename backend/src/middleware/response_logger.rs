//! R22-3: 安全回應記錄 middleware
//!
//! 攔截 403 Forbidden 回應，將 permission denied 事件寫入 user_activity_logs。
//! R22-6: 同一 IP 短時間內累積多次 403 → 產生 IDOR probe alert。

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::constants::SEC_EVENT_PERMISSION_DENIED;
use crate::middleware::CurrentUser;
use crate::services::{AlertThresholdService, AuditService, SecurityNotifier, SecurityNotification};
use crate::AppState;

pub async fn security_response_logger(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_id = request
        .extensions()
        .get::<CurrentUser>()
        .map(|u| u.id);

    let response = next.run(request).await;

    if response.status() == StatusCode::FORBIDDEN {
        let db = state.db.clone();
        let config = state.config.clone();
        tokio::spawn(async move {
            let _ = AuditService::log_security_event(
                &db,
                SEC_EVENT_PERMISSION_DENIED,
                None,
                None,
                Some(&path),
                Some(&method),
                serde_json::json!({
                    "user_id": user_id,
                    "path": path,
                    "method": method,
                }),
            )
            .await;

            // R22-6: IDOR probe detection (by user_id)
            if let Some(uid) = user_id {
                if let Err(e) = check_idor_probe(&db, &config, &uid.to_string()).await {
                    tracing::error!("[R22-6] IDOR probe check failed: {e}");
                }
            }
        });
    }

    response
}

/// R22-6: 檢查同一使用者短時間內是否累積過多 403，產生 IDOR probe alert
async fn check_idor_probe(
    pool: &PgPool,
    config: &Config,
    user_id_str: &str,
) -> std::result::Result<(), sqlx::Error> {
    let threshold = AlertThresholdService::idor_403_threshold(pool).await;
    let window_mins = AlertThresholdService::idor_403_window_mins(pool).await;
    let dedup_mins = AlertThresholdService::alert_escalation_dedup_mins(pool).await;

    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_activity_logs
        WHERE event_type = 'PERMISSION_DENIED'
          AND after_data->>'user_id' = $1
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
        "使用者 {user_id_str} 在過去 {window_mins} 分鐘內累積 {count} 次權限拒絕"
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

    let notification = SecurityNotification {
        alert_id,
        alert_type: "idor_probe".to_string(),
        severity: "critical".to_string(),
        title: "偵測到可能的 IDOR 探測攻擊".to_string(),
        description: Some(description),
        context_data: Some(serde_json::json!({ "user_id": user_id_str, "count": count })),
        created_at: chrono::Utc::now(),
    };
    SecurityNotifier::dispatch(pool, config, &notification).await;

    Ok(())
}
