//! R22-9/10/11/12: Security alert notification dispatcher
//!
//! Reads enabled channels from `security_notification_channels` table,
//! dispatches to Email / LINE Notify / Webhook. All errors are logged, never propagated.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::config::Config;
use crate::services::system_settings::SmtpConfig;
use crate::services::EmailService;

/// Gemini #7: 共用 reqwest::Client（複用 TCP 連線池）
static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

/// Gemini #6: notification channels 60 秒 cache，避免每次 dispatch 都查 DB
type ChannelCache = Option<(Vec<CachedChannel>, Instant)>;
static CHANNEL_CACHE: std::sync::LazyLock<Mutex<ChannelCache>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

const CHANNEL_CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
struct CachedChannel {
    channel: String,
    config_json: serde_json::Value,
    min_severity: String,
}

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
    /// Gemini #6: channels 設定有 60 秒 cache
    pub async fn dispatch(pool: &PgPool, config: &Config, notification: &SecurityNotification) {
        let channels = Self::load_channels(pool).await;

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

    /// Gemini #6: 從 cache 或 DB 載入 channels
    async fn load_channels(pool: &PgPool) -> Vec<CachedChannel> {
        if let Ok(guard) = CHANNEL_CACHE.lock() {
            if let Some((cached, ts)) = guard.as_ref() {
                if ts.elapsed() < CHANNEL_CACHE_TTL {
                    return cached.clone();
                }
            }
        }

        let rows: Vec<ChannelRow> = match sqlx::query_as(
            "SELECT channel, config_json, min_severity FROM security_notification_channels WHERE is_enabled = true",
        )
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                tracing::error!("[R22-9] Failed to load notification channels: {e}");
                return vec![];
            }
        };

        let cached: Vec<CachedChannel> = rows
            .into_iter()
            .map(|r| CachedChannel {
                channel: r.channel,
                config_json: r.config_json,
                min_severity: r.min_severity,
            })
            .collect();

        if let Ok(mut guard) = CHANNEL_CACHE.lock() {
            *guard = Some((cached.clone(), Instant::now()));
        }

        cached
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

        let client = &*HTTP_CLIENT;
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

        // SSRF guard: only allow https:// to public hosts
        if !is_safe_webhook_url(url) {
            let url_safe = truncate_url(url);
            tracing::error!("[R22-12] Webhook URL rejected (SSRF guard): {url_safe}");
            return;
        }

        let payload = serde_json::json!({
            "alert_id": notification.alert_id,
            "alert_type": notification.alert_type,
            "severity": notification.severity,
            "title": notification.title,
            "description": notification.description,
            "context_data": notification.context_data,
            "timestamp": notification.created_at.to_rfc3339(),
        });

        let client = &*HTTP_CLIENT;
        let result = client
            .post(url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        let url_safe = truncate_url(url);

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("[R22-12] Webhook sent to {url_safe}");
            }
            Ok(resp) => {
                tracing::error!("[R22-12] Webhook to {url_safe} failed: HTTP {}", resp.status());
            }
            Err(e) => {
                tracing::error!("[R22-12] Webhook to {url_safe} request failed: {e}");
            }
        }
    }
}

/// Truncate URL to 40 chars for log messages — prevents leaking tokens in Slack/Teams webhook URLs
fn truncate_url(url: &str) -> String {
    if url.len() > 40 {
        format!("{}...", &url[..40])
    } else {
        url.to_string()
    }
}

/// SSRF guard: reject non-https or private/loopback destinations
fn is_safe_webhook_url(url_str: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url_str) else {
        return false;
    };
    if parsed.scheme() != "https" {
        return false;
    }
    let Some(host) = parsed.host_str() else {
        return false;
    };
    // Reject bare private/loopback IP addresses
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if ip.is_loopback() || ip.is_unspecified() {
            return false;
        }
        if let std::net::IpAddr::V4(v4) = ip {
            if v4.is_private() || v4.is_link_local() || v4.is_broadcast() {
                return false;
            }
        }
    }
    // Reject known private hostnames
    let h = host.to_lowercase();
    if h == "localhost" || h.ends_with(".local") || h.ends_with(".internal") || h.ends_with(".intranet") {
        return false;
    }
    true
}

/// Escape HTML special characters to prevent injection in email bodies
fn html_escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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

    // HTML-escape user-controlled fields to prevent HTML injection in admin emails
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
        html_escape_text(&notification.alert_type),
        html_escape_text(&notification.title),
        html_escape_text(
            notification
                .description
                .as_deref()
                .unwrap_or("(no description)")
        ),
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
