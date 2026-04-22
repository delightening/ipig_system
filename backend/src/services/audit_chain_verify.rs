//! R26-2：HMAC audit chain 每日驗證 cron job。
//!
//! 對應 `docs/reviews/2026-04-21-rust-backend-review.md` SUGG-03：
//! 「audit HMAC chain 的 tamper 檢測未自動化」。
//!
//! ## 執行流程
//!
//! 每日 02:00（由 `SchedulerService::register_audit_chain_verify_job` 排程）：
//! 1. 計算昨日時間範圍（UTC `[00:00, 24:00)`）
//! 2. 呼叫 [`AuditService::verify_chain_range`] 比對每筆 row 的 HMAC
//! 3. 若有 broken_links：
//!    - INSERT `security_alerts`（severity = critical，type = `audit_chain_broken`）
//!    - 呼叫 `SecurityNotifier::dispatch`（Slack / Email / ...）
//! 4. 若完整：僅 log info（避免每天一筆 notification 噪音）
//!
//! ## 失敗模式
//!
//! - **HMAC key 未設定**：verify_chain_range 回 Err，cron 記 error 並 skip
//! - **DB 連線失敗**：retry 由 tokio-cron-scheduler 的下一輪觸發處理
//! - **真的斷鏈**：寫 security_alert + 通知；**不** panic、**不** 阻擋其他 cron

use chrono::{Duration, NaiveTime, TimeZone, Utc};
use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    config::Config,
    services::{AuditService, SecurityNotification, SecurityNotifier},
    Result,
};

/// 驗證昨日 audit chain 完整性並發送告警（若有斷鏈）。
pub async fn verify_yesterday_chain(pool: &PgPool, config: &Config) -> Result<()> {
    let (from, to) = yesterday_range_utc();

    info!(
        "[audit_chain_verify] 啟動昨日驗證 range={} ~ {}",
        from, to
    );

    let report = AuditService::verify_chain_range(pool, from, to).await?;

    info!(
        "[audit_chain_verify] 驗證完成 total={} verified={} skipped_no_hash={} broken={}",
        report.total_rows,
        report.verified_rows,
        report.skipped_no_hash,
        report.broken_links.len()
    );

    if report.is_intact() {
        return Ok(());
    }

    // 斷鏈偵測 → 寫 security_alert + 觸發通知
    let broken_ids: Vec<String> = report
        .broken_links
        .iter()
        .map(|b| b.id.to_string())
        .collect();

    let severity = "critical";
    let alert_type = "audit_chain_broken";
    let title = format!(
        "[SEC] Audit HMAC chain 斷鏈偵測：{} / {} rows",
        report.broken_links.len(),
        report.verified_rows
    );
    let description = format!(
        "昨日 ({} ~ {}) 驗證 {} 筆 audit row，其中 {} 筆 HMAC 不匹配。可能原因：\
         (1) DB 被繞過手動修改 row；(2) HMAC key 輪替未完整遷移；\
         (3) bug 導致 HMAC 計算不一致。斷鏈 row IDs：{}",
        from,
        to,
        report.verified_rows,
        report.broken_links.len(),
        broken_ids.join(", ")
    );
    let context_data = serde_json::json!({
        "range_from": from.to_rfc3339(),
        "range_to": to.to_rfc3339(),
        "total_rows": report.total_rows,
        "verified_rows": report.verified_rows,
        "skipped_no_hash": report.skipped_no_hash,
        "broken_count": report.broken_links.len(),
        "broken_ids": broken_ids,
        "first_broken": report.broken_links.first().map(|b| {
            serde_json::json!({
                "id": b.id,
                "created_at": b.created_at.to_rfc3339(),
                "expected_hash": b.expected_hash,
                "stored_hash": b.stored_hash,
            })
        }),
    });

    // INSERT security_alerts（提供後續事件追蹤、解決流程）
    let alert_id: Uuid = match sqlx::query_scalar(
        r#"
        INSERT INTO security_alerts (alert_type, severity, title, description, context_data, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        RETURNING id
        "#,
    )
    .bind(alert_type)
    .bind(severity)
    .bind(&title)
    .bind(&description)
    .bind(&context_data)
    .fetch_one(pool)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            error!(
                "[audit_chain_verify] 寫入 security_alerts 失敗（仍發送通知）: {}",
                e
            );
            Uuid::nil()
        }
    };

    let notification = SecurityNotification {
        alert_id,
        alert_type: alert_type.to_string(),
        severity: severity.to_string(),
        title,
        description: Some(description),
        context_data: Some(context_data),
        created_at: Utc::now(),
    };

    SecurityNotifier::dispatch(pool, config, &notification).await;

    warn!(
        "[audit_chain_verify] 斷鏈偵測已通知 alert_id={} broken_count={}",
        alert_id,
        report.broken_links.len()
    );

    Ok(())
}

/// 計算昨日 UTC 時間範圍 `[yesterday 00:00, today 00:00)`。
fn yesterday_range_utc() -> (
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
) {
    let today_utc = Utc::now().date_naive();
    let yesterday = today_utc - Duration::days(1);
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("00:00:00 valid");
    let from = Utc
        .from_utc_datetime(&yesterday.and_time(midnight));
    let to = Utc
        .from_utc_datetime(&today_utc.and_time(midnight));
    (from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yesterday_range_is_24h_block() {
        let (from, to) = yesterday_range_utc();
        let diff = to - from;
        assert_eq!(diff.num_hours(), 24);
        // to 必為今日 UTC 00:00
        assert_eq!(to.format("%H:%M:%S").to_string(), "00:00:00");
    }

    #[test]
    fn yesterday_range_ends_before_now() {
        let (_from, to) = yesterday_range_utc();
        assert!(to <= Utc::now());
    }
}
