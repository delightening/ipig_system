//! R30-3a EmailAdapter — 把 outbox event 透過 SMTP 寄出。
//!
//! Payload schema：
//! ```json
//! {
//!   "to": "user@example.com",
//!   "to_name": "顯示名稱",         // optional
//!   "subject": "...",
//!   "plain_body": "...",
//!   "html_body": "..."
//! }
//! ```
//!
//! 後續 PR-B 會由 caller (e.g., euthanasia notification) 把 template render 後
//! 直接塞 plain/html body 進來；本 PR 只實作通用 send 路徑，不綁 template。
//!
//! Caching：SMTP config 解析有 30s TTL 以避免每筆 event 重打 DB（gemini PR #305
//! medium 採納），同時保留 admin 改設定後 ≤30s 生效的 fast-feedback 行為。

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::Mutex;

use super::{ChannelAdapter, OutboxEvent};
use crate::{
    config::Config, services::system_settings::SmtpConfig, services::EmailService, AppError, Result,
};

/// 從 outbox payload 反序列化的 email message。
#[derive(Debug, Deserialize)]
struct EmailPayload {
    to: String,
    #[serde(default)]
    to_name: Option<String>,
    subject: String,
    plain_body: String,
    html_body: String,
}

/// SMTP 設定快取 — 30s TTL；過期後重 resolve。
const SMTP_CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Default)]
struct SmtpCache {
    cached_at: Option<Instant>,
    config: Option<SmtpConfig>,
}

/// EmailAdapter — 持有 DB pool + Config 以解析每次寄送時的最新 SMTP 設定。
///
/// SMTP 設定 DB-first（runtime mutable）— cache TTL 30s 兼顧吞吐與動態變更。
pub struct EmailAdapter {
    pool: PgPool,
    config: Config,
    cache: Arc<Mutex<SmtpCache>>,
}

impl EmailAdapter {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self {
            pool,
            config,
            cache: Arc::new(Mutex::new(SmtpCache::default())),
        }
    }

    async fn resolve_smtp_cached(&self) -> SmtpConfig {
        let mut cache = self.cache.lock().await;
        let fresh = cache
            .cached_at
            .is_some_and(|at| at.elapsed() < SMTP_CACHE_TTL);
        if fresh {
            if let Some(cfg) = cache.config.clone() {
                return cfg;
            }
        }
        let cfg = EmailService::resolve_smtp(&self.pool, &self.config).await;
        cache.cached_at = Some(Instant::now());
        cache.config = Some(cfg.clone());
        cfg
    }
}

#[async_trait]
impl ChannelAdapter for EmailAdapter {
    fn channel(&self) -> &'static str {
        "email"
    }

    async fn send(&self, event: &OutboxEvent) -> Result<()> {
        let payload: EmailPayload = serde_json::from_value(event.payload.clone()).map_err(|e| {
            AppError::Internal(format!(
                "outbox EmailAdapter: invalid payload for event {}: {}",
                event.id, e
            ))
        })?;

        let smtp = self.resolve_smtp_cached().await;

        // 防止 EmailService::send_email_smtp 在「SMTP 未配置 / 收件人無效」時
        // 回 Ok(()) 被 worker 標記 DONE 永久吞掉。outbox 必須對這兩種情況走
        // mark_failed 路徑（gemini & coderabbit PR #305 採納）。
        if smtp.host.is_none() {
            return Err(AppError::Internal(format!(
                "outbox EmailAdapter: SMTP host not configured (event {})",
                event.id
            )));
        }
        if payload.to.is_empty() || !payload.to.contains('@') {
            return Err(AppError::Internal(format!(
                "outbox EmailAdapter: invalid recipient '{}' (event {})",
                payload.to, event.id
            )));
        }

        let to_name = payload.to_name.as_deref().unwrap_or(&payload.to);
        EmailService::send_email_smtp(
            &smtp,
            &payload.to,
            to_name,
            &payload.subject,
            &payload.plain_body,
            &payload.html_body,
        )
        .await
        .map_err(|e| {
            AppError::Internal(format!(
                "outbox EmailAdapter: SMTP send failed for event {}: {}",
                event.id, e
            ))
        })
    }
}
