//! R22-9/10/11/12: Security alert notification dispatcher
//!
//! Reads enabled channels from `security_notification_channels` table,
//! dispatches to Email / LINE Notify / Webhook. All errors are logged, never propagated.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::services::system_settings::SmtpConfig;
use crate::services::EmailService;

/// Payload passed to all notification channels
#[derive(Debug, Clone)]
pub struct SecurityNotification {
    pub alert_id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub context_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Channel row from `security_notification_channels`
#[derive(Debug, sqlx::FromRow)]
struct ChannelRow {
    channel: String,
    config_json: serde_json::Value,
    min_severity: String,
}

pub struct SecurityNotifier;

impl SecurityNotifier {
    /// Dispatch notification to all enabled channels. Fire-and-forget safe.
    pub async fn dispatch(pool: &PgPool, config: &Config, notification: &SecurityNotification) {
        let channels: Vec<ChannelRow> = match sqlx::query_as(
            "SELECT channel, config_json, min_severity FROM security_notification_channels WHERE is_enabled = true",
        )
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!("[R22-9] Failed to load notification channels: {e}");
                return;
            }
        };

        for ch in &channels {
            if !severity_meets_minimum(&notification.severity, &ch.min_severity) {
                continue;
            }

            match ch.channel.as_str() {
                "email" => Self::send_email(pool, config, &ch.config_json, notification).await,
                "line" => Self::send_line(&ch.config_json, config, notification).await,
                "webhook" => Self::send_webhook(&ch.config_json, notification).await,
                other => tracing::warn!("[R22-9] Unknown channel: {other}"),
            }
        }
    }

    // ── Email ──────────────────────────────────────────────────

    async fn send_email(
        pool: &PgPool,
        config: &Config,
        channel_config: &serde_json::Value,
        notification: &SecurityNotification,
    ) {
        let smtp = EmailService::resolve_smtp(pool, config).await;
        if !smtp.is_email_enabled() {
            tracing::debug!("[R22-10] Email disabled, skipping security alert email");
            return;
        }

        let recipients: Vec<String> = channel_config
            .get("recipients")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        for recipient in &recipients {
            if let Err(e) = send_security_email(&smtp, recipient, notification).await {
                tracing::error!("[R22-10] Failed to send security email to {recipient}: {e}");
            }
        }
    }

    // ── LINE Notify ────────────────────────────────────────────

    async fn send_line(
        channel_config: &serde_json::Value,
        config: &Config,
        notification: &SecurityNotification,
    ) {
        let token = channel_config
            .get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| config.line_notify_token.clone());

        let Some(token) = token else {
            tracing::warn!("[R22-11] LINE Notify token not configured");
            return;
        };

        let message = format!(
            "\n[iPig Security Alert]\n嚴重度: {}\n類型: {}\n標題: {}\n{}",
            notification.severity,
            notification.alert_type,
            notification.title,
            notification
                .description
                .as_deref()
                .unwrap_or("(no description)")
        );

        let client = reqwest::Client::new();
        let result = client
            .post("https://notify-api.line.me/api/notify")
            .bearer_auth(&token)
            .form(&[("message", &message)])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("[R22-11] LINE Notify sent successfully");
            }
            Ok(resp) => {
                tracing::error!("[R22-11] LINE Notify failed: HTTP {}", resp.status());
            }
            Err(e) => {
                tracing::error!("[R22-11] LINE Notify request failed: {e}");
            }
        }
    }

    // ── Webhook ────────────────────────────────────────────────

    async fn send_webhook(
        channel_config: &serde_json::Value,
        notification: &SecurityNotification,
    ) {
        let Some(url) = channel_config.get("url").and_then(|v| v.as_str()) else {
            tracing::warn!("[R22-12] Webhook URL not configured");
            return;
        };

        let payload = serde_json::json!({
            "alert_id": notification.alert_id,
            "alert_type": notification.alert_type,
            "severity": notification.severity,
            "title": notification.title,
            "description": notification.description,
            "context_data": notification.context_data,
            "timestamp": notification.created_at.to_rfc3339(),
        });

        let client = reqwest::Client::new();
        let result = client
            .post(url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("[R22-12] Webhook sent to {url}");
            }
            Ok(resp) => {
                tracing::error!("[R22-12] Webhook to {url} failed: HTTP {}", resp.status());
            }
            Err(e) => {
                tracing::error!("[R22-12] Webhook to {url} request failed: {e}");
            }
        }
    }
}

/// Check if alert severity meets the channel minimum
fn severity_meets_minimum(alert_severity: &str, min_severity: &str) -> bool {
    let level = |s: &str| match s {
        "critical" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    };
    level(alert_severity) >= level(min_severity)
}

/// R22-10: Send security alert email
async fn send_security_email(
    smtp: &SmtpConfig,
    to_email: &str,
    notification: &SecurityNotification,
) -> anyhow::Result<()> {
    let severity_badge = match notification.severity.as_str() {
        "critical" => "🔴 CRITICAL",
        "warning" => "🟡 WARNING",
        _ => "ℹ️ INFO",
    };

    let subject = format!(
        "[iPig Security] {} — {}",
        severity_badge, notification.title
    );

    let plain_body = format!(
        "Security Alert\n\nSeverity: {}\nType: {}\nTitle: {}\n\n{}\n\nAlert ID: {}\nTime: {}",
        notification.severity,
        notification.alert_type,
        notification.title,
        notification
            .description
            .as_deref()
            .unwrap_or("(no description)"),
        notification.alert_id,
        notification.created_at,
    );

    let html_body = format!(
        r#"<div style="font-family:sans-serif;max-width:600px;margin:0 auto">
<h2 style="color:#dc2626">{severity_badge}</h2>
<table style="width:100%;border-collapse:collapse">
<tr><td style="padding:8px;font-weight:bold">Type</td><td style="padding:8px">{}</td></tr>
<tr><td style="padding:8px;font-weight:bold">Title</td><td style="padding:8px">{}</td></tr>
<tr><td style="padding:8px;font-weight:bold">Description</td><td style="padding:8px">{}</td></tr>
<tr><td style="padding:8px;font-weight:bold">Alert ID</td><td style="padding:8px;font-size:12px">{}</td></tr>
<tr><td style="padding:8px;font-weight:bold">Time</td><td style="padding:8px">{}</td></tr>
</table>
<p style="color:#666;font-size:12px;margin-top:20px">This is an automated security alert from iPig System.</p>
</div>"#,
        notification.alert_type,
        notification.title,
        notification
            .description
            .as_deref()
            .unwrap_or("(no description)"),
        notification.alert_id,
        notification.created_at,
    );

    EmailService::send_email_smtp(smtp, to_email, "Admin", &subject, &plain_body, &html_body)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_meets_minimum_critical_passes_all() {
        assert!(severity_meets_minimum("critical", "info"));
        assert!(severity_meets_minimum("critical", "warning"));
        assert!(severity_meets_minimum("critical", "critical"));
    }

    #[test]
    fn test_severity_meets_minimum_warning_blocks_critical() {
        assert!(severity_meets_minimum("warning", "info"));
        assert!(severity_meets_minimum("warning", "warning"));
        assert!(!severity_meets_minimum("warning", "critical"));
    }

    #[test]
    fn test_severity_meets_minimum_info_blocks_higher() {
        assert!(severity_meets_minimum("info", "info"));
        assert!(!severity_meets_minimum("info", "warning"));
        assert!(!severity_meets_minimum("info", "critical"));
    }

    #[test]
    fn test_severity_unknown_defaults_to_zero() {
        assert!(!severity_meets_minimum("unknown", "info"));
        assert!(severity_meets_minimum("critical", "unknown"));
    }
}
