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

    async fn send_webhook(channel_config: &serde_json::Value, notification: &SecurityNotification) {
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

        // R66-C1b: DNS rebinding 防護——自行解析並驗證每個 IP 非私有，再把連線 pin 到該組
        // 已驗證 IP（reqwest connect 不再重新解析），閉合 check→connect 的 TOCTOU 視窗。
        let Some((pin_host, pinned_addrs)) = resolve_validated_addrs(url).await else {
            let url_safe = truncate_url(url);
            tracing::error!(
                "[R22-12] Webhook URL rejected (DNS resolves to private/unresolvable): {url_safe}"
            );
            return;
        };

        // SSRF-pin 需 per-target 解析覆寫，無法沿用共用 HTTP_CLIENT（Gemini #7 連線池）；
        // 安全告警 webhook 低頻，per-send client 成本可忽略。TLS 仍對 hostname（非 pinned IP）
        // 驗 SNI / 憑證。
        let client = match reqwest::Client::builder()
            .resolve_to_addrs(&pin_host, &pinned_addrs)
            .timeout(std::time::Duration::from_secs(10))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("[R22-12] webhook client 建立失敗: {e}");
                return;
            }
        };
        let result = client.post(url).json(&payload).send().await;

        let url_safe = truncate_url(url);

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("[R22-12] Webhook sent to {url_safe}");
            }
            Ok(resp) => {
                tracing::error!(
                    "[R22-12] Webhook to {url_safe} failed: HTTP {}",
                    resp.status()
                );
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
    // Reject bare private/loopback IP addresses。
    // R66-C1：先移除末尾 '.'（FQDN 絕對網域寫法，解析結果相同）——否則 "localhost." /
    // "127.0.0.1." 會讓下方 IP parse 與 hostname 比對雙雙失效而繞過 SSRF 防護（Gemini #773 HIGH）。
    let host = host.strip_suffix('.').unwrap_or(host);
    // 注意：url::Url::host_str() 對 IPv6 literal 回傳帶方括號的 "[::1]"，須先去括號才能 parse。
    let host_for_ip = host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(ip) = host_for_ip.parse::<std::net::IpAddr>() {
        if !is_safe_public_ip(&ip) {
            return false;
        }
    }
    // Reject known private hostnames
    let h = host.to_lowercase();
    if h == "localhost"
        || h.ends_with(".local")
        || h.ends_with(".internal")
        || h.ends_with(".intranet")
    {
        return false;
    }
    true
}

/// 判定單一 IP 是否為「可安全外連的公開位址」（非 loopback / 私有 / link-local / broadcast /
/// IPv6 ULA / IPv4-mapped 私有等）。
///
/// R66-C1 的字面 IP 規則抽出共用：除 `is_safe_webhook_url` 對 URL 字面 IP 使用外，
/// R66-C1b 對 **DNS 解析結果**逐一複用，閉合 DNS rebinding（TOCTOU）。
fn is_safe_public_ip(ip: &std::net::IpAddr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() {
        return false;
    }
    match ip {
        std::net::IpAddr::V4(v4) => {
            !(v4.is_private() || v4.is_link_local() || v4.is_broadcast() || v4.is_multicast())
            // 224.0.0.0/4（防本地網路探測，Gemini #777）
        }
        // R66-C1：IPv6 私有段（ULA / link-local / IPv4-mapped）。手動位元遮罩避免依賴
        // unstable 的 `Ipv6Addr::is_unique_local`。`::1` 已由上方 `is_loopback` 先擋。
        std::net::IpAddr::V6(v6) => {
            if v6.is_multicast() {
                return false; // ff00::/8（Gemini #777）
            }
            let seg0 = v6.segments()[0];
            // ULA fc00::/7（fcXX / fdXX）
            if (seg0 & 0xfe00) == 0xfc00 {
                return false;
            }
            // link-local fe80::/10
            if (seg0 & 0xffc0) == 0xfe80 {
                return false;
            }
            // IPv4-mapped（::ffff:a.b.c.d）/ deprecated IPv4-compatible（::a.b.c.d）以 V4 規則複檢
            if let Some(v4) = v6.to_ipv4() {
                if v4.is_private()
                    || v4.is_loopback()
                    || v4.is_link_local()
                    || v4.is_broadcast()
                    || v4.is_unspecified()
                {
                    return false;
                }
            }
            true
        }
    }
}

/// R66-C1b：DNS rebinding 防護。自行解析 `url` 的 host，逐一驗證解析出的 IP 皆為公開位址，
/// 回傳 `(host, 已驗證 addrs)` 供 reqwest `resolve_to_addrs` pin。任一 IP 落在私有/loopback/
/// link-local、或解析失敗 / 空集合 → `None`（拒絕）。
///
/// 由呼叫端以回傳的 addrs `resolve_to_addrs(host, &addrs)`，使 reqwest connect 時**不再重新
/// 解析**——攻擊者無法在「驗證後 → 連線前」把公開 hostname rebind 到內網（TOCTOU 閉合）。
async fn resolve_validated_addrs(url_str: &str) -> Option<(String, Vec<std::net::SocketAddr>)> {
    let parsed = url::Url::parse(url_str).ok()?;
    let host = parsed.host_str()?.to_string();
    let port = parsed.port_or_known_default()?;
    // is_safe_webhook_url 已先擋私有字面 IP / 私有 hostname；此處對 DNS 結果複檢。
    // resolve_to_addrs 的 key 須與 reqwest 由 URL 取得的 host 一致 → pin key 用原始 host；
    // lookup_host 不接受末尾 '.' 無妨，但**不接受 IPv6 literal 的方括號**（"[2606::1]" 會解析
    // 失敗 → 公開 IPv6-literal webhook 被誤拒，Gemini #777），故 lookup 用去點 + 去括號的 host。
    let lookup_host = host.strip_suffix('.').unwrap_or(&host);
    let lookup_host = lookup_host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(lookup_host);
    let addrs: Vec<std::net::SocketAddr> = tokio::net::lookup_host((lookup_host, port))
        .await
        .ok()?
        .collect();
    if addrs.is_empty() || addrs.iter().any(|a| !is_safe_public_ip(&a.ip())) {
        return None;
    }
    Some((host, addrs))
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

    EmailService::send_email_smtp(smtp, to_email, "Admin", &subject, &plain_body, &html_body).await
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

    // R66-C1：SSRF guard
    #[test]
    fn ssrf_allows_public_https() {
        assert!(is_safe_webhook_url("https://hooks.slack.com/services/xxx"));
        assert!(is_safe_webhook_url("https://example.com/webhook"));
    }

    #[test]
    fn ssrf_rejects_non_https_and_private_hostnames() {
        assert!(!is_safe_webhook_url("http://example.com/webhook")); // 非 https
        assert!(!is_safe_webhook_url("https://localhost/x"));
        assert!(!is_safe_webhook_url("https://foo.local/x"));
        assert!(!is_safe_webhook_url("https://svc.internal/x"));
    }

    #[test]
    fn ssrf_rejects_ipv4_private_loopback() {
        assert!(!is_safe_webhook_url("https://127.0.0.1/x"));
        assert!(!is_safe_webhook_url("https://10.0.0.5/x"));
        assert!(!is_safe_webhook_url("https://192.168.1.1/x"));
        assert!(!is_safe_webhook_url("https://169.254.1.1/x")); // link-local
    }

    #[test]
    fn ssrf_rejects_ipv6_private() {
        assert!(!is_safe_webhook_url("https://[::1]/x")); // loopback
        assert!(!is_safe_webhook_url("https://[fc00::1]/x")); // ULA
        assert!(!is_safe_webhook_url("https://[fd12:3456::1]/x")); // ULA
        assert!(!is_safe_webhook_url("https://[fe80::1]/x")); // link-local
        assert!(!is_safe_webhook_url("https://[::ffff:10.0.0.1]/x")); // IPv4-mapped private
        assert!(!is_safe_webhook_url("https://[::10.0.0.1]/x")); // IPv4-compatible（deprecated）
    }

    // R66-C1 #773：末尾點 FQDN 繞過（DNS 解析相同）
    #[test]
    fn ssrf_rejects_trailing_dot_bypass() {
        assert!(!is_safe_webhook_url("https://localhost./x"));
        assert!(!is_safe_webhook_url("https://127.0.0.1./x"));
        assert!(!is_safe_webhook_url("https://10.0.0.1./x"));
        assert!(!is_safe_webhook_url("https://svc.internal./x"));
    }

    // ── R66-C1b: is_safe_public_ip（DNS 解析結果複用的 IP 安全判定）─────────────
    #[test]
    fn safe_public_ip_accepts_public() {
        use std::net::IpAddr;
        let parse = |s: &str| s.parse::<IpAddr>().expect("parse IP");
        assert!(is_safe_public_ip(&parse("8.8.8.8")));
        assert!(is_safe_public_ip(&parse("1.1.1.1")));
        assert!(is_safe_public_ip(&parse("2606:4700:4700::1111")));
    }

    #[test]
    fn safe_public_ip_rejects_private_and_special() {
        use std::net::IpAddr;
        for s in [
            "127.0.0.1",        // loopback
            "0.0.0.0",          // unspecified
            "10.0.0.5",         // private
            "192.168.1.1",      // private
            "172.16.0.1",       // private
            "169.254.1.1",      // link-local
            "255.255.255.255",  // broadcast
            "::1",              // v6 loopback
            "fc00::1",          // ULA
            "fd12:3456::1",     // ULA
            "fe80::1",          // v6 link-local
            "::ffff:10.0.0.1",  // IPv4-mapped private
            "::ffff:127.0.0.1", // IPv4-mapped loopback
            "224.0.0.1",        // IPv4 multicast 224.0.0.0/4
            "239.255.255.250",  // IPv4 multicast（SSDP）
            "ff02::1",          // IPv6 multicast ff00::/8
        ] {
            assert!(
                !is_safe_public_ip(&s.parse::<IpAddr>().expect("parse IP")),
                "{s} 應判定為非公開（不安全）"
            );
        }
    }

    // ── R66-C1b: resolve_validated_addrs（DNS rebinding pin）— 離線確定性案例 ────
    // localhost 由本機解析（無外部網路）；字面 IP 不走 DNS。
    #[tokio::test]
    async fn resolve_rejects_host_resolving_to_loopback() {
        // localhost → 127.0.0.1 / ::1（皆 loopback）→ 拒絕
        assert!(resolve_validated_addrs("https://localhost/x")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn resolve_rejects_literal_private_ip() {
        assert!(resolve_validated_addrs("https://10.0.0.5/x")
            .await
            .is_none());
        assert!(resolve_validated_addrs("https://[fc00::1]/x")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn resolve_accepts_and_pins_public_ip() {
        // 字面公開 IP：lookup_host 直接回該 IP（不走 DNS），驗證通過並 pin。
        let pinned = resolve_validated_addrs("https://8.8.8.8/x").await;
        let (host, addrs) = pinned.expect("公開 IP 應通過並回 pin addrs");
        assert_eq!(host, "8.8.8.8");
        assert!(addrs.iter().all(|a| a.ip().to_string() == "8.8.8.8"));
        assert!(addrs.iter().all(|a| a.port() == 443), "https 預設 port 443");
    }

    // Gemini #777：公開 IPv6 literal（host_str 帶方括號）須去括號才能 lookup_host，
    // 否則會被誤拒。
    #[tokio::test]
    async fn resolve_accepts_public_ipv6_literal() {
        let pinned = resolve_validated_addrs("https://[2606:4700:4700::1111]/x").await;
        let (host, addrs) = pinned.expect("公開 IPv6 literal 不應被誤拒");
        assert_eq!(
            host, "[2606:4700:4700::1111]",
            "pin key 保留原始（含括號）host"
        );
        assert!(!addrs.is_empty());
    }
}
