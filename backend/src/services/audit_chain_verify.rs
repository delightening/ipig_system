//! R26-2：HMAC audit chain 每日驗證 cron job。
//!
//! 對應 `docs/reviews/2026-04-21-rust-backend-review.md` SUGG-03：
//! 「audit HMAC chain 的 tamper 檢測未自動化」。
//!
//! ## 執行流程
//!
//! 每日 02:00 UTC（由 `SchedulerService::register_audit_chain_verify_job` 排程）：
//! 1. 檢查 `config.audit_chain_verify_active`；若 false 則 log + 早退
//!    （預設 false，等 R26-6 HMAC 版本化 + legacy backfill 完成後才設 true，
//!    避免對 deprecated `log_activity` 寫入的 legacy row 產生大量 false positive）
//! 2. 計算昨日時間範圍（UTC `[00:00, 24:00)`）
//! 3. 呼叫 [`AuditService::verify_chain_range`] 比對每筆 row 的 HMAC
//! 4. 若有 broken_links：
//!    - 透過 [`AuditService::create_security_alert`] 寫入 `security_alerts`
//!      （severity = critical / type = `audit_chain_broken`）
//!    - 透過 [`SecurityNotifier::dispatch`] 發送通知
//!    - **告警 payload 限大小**（top 20 IDs + 總數；不洩漏 hash 細節到外部通道）
//! 5. 若完整：僅 log info（避免每天一筆 notification 噪音）
//!
//! ## 失敗模式
//!
//! - **HMAC key 未設定**：`verify_chain_range` 回 `Err`；本函式 propagate
//! - **DB INSERT security_alert 失敗**：propagate `Err`；不再用 `Uuid::nil()`
//!   假性 dispatch（CodeRabbit PR #158 🟠 Major）
//! - **真的斷鏈**：寫 alert + 通知；**不** panic
//!
//! ## 維護記事
//!
//! - 待 R26-6 完成後將 `audit_chain_verify_active` 預設改為 `true`，並移除本檔關於
//!   「legacy 路徑可能導致 false positive」的條件式
//! - 待 R26-4（舊 `log_activity` 移除）完成後可移除 active 旗標機制（無 legacy
//!   row 再產生）

use chrono::{Duration, NaiveTime, TimeZone, Utc};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use super::audit::{BrokenChainLink, ChainVerificationReport};
use crate::{
    config::Config,
    services::{AuditService, SecurityNotification, SecurityNotifier},
    Result,
};

/// 告警外推 payload 中最多列出的 broken row IDs 數量，避免 email/webhook
/// payload 暴漲（CodeRabbit PR #158 🟠 Major）。剩餘以 `omitted_broken_ids`
/// 數字呈現。
const MAX_BROKEN_IDS_IN_ALERT: usize = 20;

const ALERT_TYPE: &str = "audit_chain_broken";
const ALERT_SEVERITY: &str = "critical";

/// 驗證昨日 audit chain 完整性並發送告警（若有斷鏈）。
///
/// 主要 entry point；保持 < 50 行（CodeRabbit PR #158 🟠 Refactor）。
pub async fn verify_yesterday_chain(pool: &PgPool, config: &Config) -> Result<()> {
    if !config.audit_chain_verify_active {
        info!(
            "[audit_chain_verify] cron 已排程但 audit_chain_verify_active=false；\
             本輪不執行（待 R26-6 HMAC 版本化 + legacy backfill 完成後啟用）"
        );
        return Ok(());
    }

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

    let alert_payload = build_chain_break_alert(&report, from, to);
    let alert_id = AuditService::create_security_alert(
        pool,
        ALERT_TYPE,
        ALERT_SEVERITY,
        &alert_payload.title,
        &alert_payload.description,
        &alert_payload.context_data,
    )
    .await?;

    dispatch_chain_break_notification(pool, config, alert_id, &alert_payload).await;

    warn!(
        "[audit_chain_verify] 斷鏈偵測已通知 alert_id={} broken_count={}",
        alert_id,
        report.broken_links.len()
    );

    Ok(())
}

/// 告警內容（title / description / context_data 三件套）。
struct AlertPayload {
    title: String,
    description: String,
    context_data: serde_json::Value,
}

/// 從 verification report 組出告警內容。
///
/// **payload 大小限制**：broken_ids 最多 [`MAX_BROKEN_IDS_IN_ALERT`] 筆 + omit
/// count；first_broken 只露 id + created_at（**不**外洩 expected_hash /
/// stored_hash 給 email/webhook 等外部通道）。完整 hash 細節留在 DB
/// `user_activity_logs` 自己查。
fn build_chain_break_alert(
    report: &ChainVerificationReport,
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
) -> AlertPayload {
    let broken_ids: Vec<String> = report
        .broken_links
        .iter()
        .take(MAX_BROKEN_IDS_IN_ALERT)
        .map(|b| b.id.to_string())
        .collect();
    let omitted_broken_ids = report.broken_links.len().saturating_sub(broken_ids.len());

    let title = format!(
        "[SEC] Audit HMAC chain 斷鏈偵測：{} / {} rows",
        report.broken_links.len(),
        report.verified_rows
    );

    let description = format!(
        "昨日 ({} ~ {}) 驗證 {} 筆 audit row，其中 {} 筆 HMAC 不匹配。可能原因：\
         (1) DB 被繞過手動修改 row；(2) HMAC key 輪替未完整遷移；\
         (3) bug 導致 HMAC 計算不一致。前 {} 筆 row IDs：{}{}",
        from,
        to,
        report.verified_rows,
        report.broken_links.len(),
        broken_ids.len(),
        broken_ids.join(", "),
        if omitted_broken_ids > 0 {
            format!("（另 {omitted_broken_ids} 筆未列；查 user_activity_logs 取完整清單）")
        } else {
            String::new()
        },
    );

    let context_data = serde_json::json!({
        "range_from": from.to_rfc3339(),
        "range_to": to.to_rfc3339(),
        "total_rows": report.total_rows,
        "verified_rows": report.verified_rows,
        "skipped_no_hash": report.skipped_no_hash,
        "broken_count": report.broken_links.len(),
        "omitted_broken_ids": omitted_broken_ids,
        "broken_ids": broken_ids,
        "first_broken": report.broken_links.first().map(broken_link_summary),
    });

    AlertPayload {
        title,
        description,
        context_data,
    }
}

/// 從 BrokenChainLink 萃取**外推安全**的摘要（不含 hash 值）。
fn broken_link_summary(link: &BrokenChainLink) -> serde_json::Value {
    serde_json::json!({
        "id": link.id,
        "created_at": link.created_at.to_rfc3339(),
    })
}

/// 發送告警通知（包成 helper 讓 verify_yesterday_chain 主流程更短）。
async fn dispatch_chain_break_notification(
    pool: &PgPool,
    config: &Config,
    alert_id: Uuid,
    payload: &AlertPayload,
) {
    let notification = SecurityNotification {
        alert_id,
        alert_type: ALERT_TYPE.to_string(),
        severity: ALERT_SEVERITY.to_string(),
        title: payload.title.clone(),
        description: Some(payload.description.clone()),
        context_data: Some(payload.context_data.clone()),
        created_at: Utc::now(),
    };
    SecurityNotifier::dispatch(pool, config, &notification).await;
}

/// 計算昨日 UTC 時間範圍 `[yesterday 00:00, today 00:00)`。
fn yesterday_range_utc() -> (
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
) {
    let today_utc = Utc::now().date_naive();
    let yesterday = today_utc - Duration::days(1);
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("00:00:00 valid");
    let from = Utc.from_utc_datetime(&yesterday.and_time(midnight));
    let to = Utc.from_utc_datetime(&today_utc.and_time(midnight));
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

    #[test]
    fn alert_payload_truncates_broken_ids() {
        // 構造 25 筆 broken — 應只列 20
        let report = ChainVerificationReport {
            range_from: Utc::now(),
            range_to: Utc::now(),
            total_rows: 30,
            verified_rows: 30,
            skipped_no_hash: 0,
            broken_links: (0..25)
                .map(|_| BrokenChainLink {
                    id: Uuid::new_v4(),
                    created_at: Utc::now(),
                    expected_hash: "deadbeef".into(),
                    stored_hash: "cafebabe".into(),
                    stored_previous_hash: None,
                })
                .collect(),
        };
        let payload = build_chain_break_alert(&report, Utc::now(), Utc::now());
        let ctx = &payload.context_data;
        assert_eq!(ctx["broken_count"], 25);
        assert_eq!(ctx["omitted_broken_ids"], 5);
        assert_eq!(
            ctx["broken_ids"]
                .as_array()
                .expect("broken_ids should be a JSON array")
                .len(),
            20
        );
        // 確認外推 payload 不含 hash 細節
        let first = ctx["first_broken"]
            .as_object()
            .expect("first_broken should be a JSON object");
        assert!(!first.contains_key("expected_hash"));
        assert!(!first.contains_key("stored_hash"));
    }
}
