use std::collections::HashMap;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::middleware::ActorContext;
use crate::models::audit_diff::{AuditRedact, DataDiff};
use crate::services::audit::{ActivityLogEntry, AuditEntity};
use crate::services::AuditService;
use crate::Result;

/// R28-3：敏感設定 key 的 audit data_diff 應 redact，避免明文寫入 audit_log。
/// 對應 R26 `AuditRedact` pattern。
const REDACTED_SETTING_KEYS: &[&str] = &[
    "smtp_password",
    "anthropic_api_key",
    "metrics_token",
    "alertmanager_webhook_token",
    "line_notify_token",
    "pdf_service_token",
];

fn is_redacted_key(key: &str) -> bool {
    REDACTED_SETTING_KEYS.contains(&key)
}

fn redact_value(key: &str, value: &Value) -> Value {
    if is_redacted_key(key) && !value.is_null() {
        Value::String("***REDACTED***".to_string())
    } else {
        value.clone()
    }
}

/// R28-3：System setting 變更的 audit snapshot。實作 `AuditRedact` 但回傳空陣列
/// 因為敏感值已在外層由 `redact_value` 預先處理（key-based redact）。
#[derive(serde::Serialize)]
struct SettingSnapshot<'a> {
    key: &'a str,
    value: &'a Value,
}

impl AuditRedact for SettingSnapshot<'_> {
    fn redacted_fields() -> &'static [&'static str] {
        &[]
    }
}

pub struct SystemSettingsService {
    pool: PgPool,
}

/// Resolved SMTP configuration from DB (with .env fallback)
#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: Option<String>,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_email: String,
    pub from_name: String,
}

impl SmtpConfig {
    pub fn is_email_enabled(&self) -> bool {
        self.host.is_some()
    }
}

impl SystemSettingsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_all_settings(&self) -> anyhow::Result<HashMap<String, Value>> {
        let rows: Vec<(String, Value)> = sqlx::query_as(
            "SELECT key, value FROM system_settings ORDER BY key"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map = HashMap::new();
        for (key, value) in rows {
            let unwrapped = Self::unwrap_jsonb(value);
            map.insert(key, unwrapped);
        }
        Ok(map)
    }

    /// 更新系統設定 — Service-driven audit (R26 SDD + R28-3 audit-coverage)
    ///
    /// R28-3：原 upsert 無 audit。改為先讀 before snapshot、tx 內 upsert、
    /// 每 key 寫一筆 SETTING_UPDATE audit log（含 before/after diff）。
    /// 敏感 key（smtp_password 等）的 value 在 audit 中 redact 為 "***REDACTED***"。
    pub async fn update_settings(
        &self,
        actor: &ActorContext,
        settings: &HashMap<String, Value>,
    ) -> Result<()> {
        let updated_by = actor.require_user()?.id;

        let mut tx = self.pool.begin().await?;

        // 一次讀齊所有將被更新 key 的 before snapshot
        let keys: Vec<String> = settings.keys().cloned().collect();
        let before_rows: Vec<(String, Value)> = sqlx::query_as(
            "SELECT key, value FROM system_settings WHERE key = ANY($1)",
        )
        .bind(&keys)
        .fetch_all(&mut *tx)
        .await?;
        let before_map: HashMap<String, Value> = before_rows.into_iter().collect();

        for (key, value) in settings {
            let json_value = Self::wrap_jsonb(value.clone());
            sqlx::query(
                r#"
                INSERT INTO system_settings (key, value, updated_at, updated_by)
                VALUES ($1, $2, NOW(), $3)
                ON CONFLICT (key) DO UPDATE
                SET value = $2, updated_at = NOW(), updated_by = $3
                "#
            )
            .bind(key)
            .bind(&json_value)
            .bind(updated_by)
            .execute(&mut *tx)
            .await?;

            // 寫 audit；敏感 key 的 value redact
            let before_value = before_map.get(key).cloned().unwrap_or(Value::Null);
            let event_type = if before_map.contains_key(key) {
                "SETTING_UPDATE"
            } else {
                "SETTING_CREATE"
            };
            let display = format!("system_settings/{key}");
            // user_activity_logs.entity_id 是 UUID 不是 string，將 setting key
            // 用 deterministic v5 namespace UUID 編出來，可日後 group by 該 key
            // 的所有變更歷史。
            // user_activity_logs.entity_id 是 UUID。setting key 為字串，無自然 UUID；
            // 用 SHA-256 前 16 bytes deterministic 編出來，可日後 group by 該 key
            // 的所有變更歷史（同 key 永遠對應同 UUID）。
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"system_setting/");
            hasher.update(key.as_bytes());
            let digest = hasher.finalize();
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(&digest[..16]);
            let entity_id = Uuid::from_bytes(bytes);
            let before_redacted = redact_value(key, &before_value);
            let after_redacted = redact_value(key, value);
            let before_snap = SettingSnapshot {
                key,
                value: &before_redacted,
            };
            let after_snap = SettingSnapshot {
                key,
                value: &after_redacted,
            };
            let data_diff = if before_map.contains_key(key) {
                DataDiff::compute(Some(&before_snap), Some(&after_snap))
            } else {
                DataDiff::create_only(&after_snap)
            };
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "SYSTEM",
                    event_type,
                    entity: Some(AuditEntity::new("system_setting", entity_id, &display)),
                    data_diff: Some(data_diff),
                    request_context: None,
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("audit log SETTING_{event_type} failed: {e}"))?;
        }

        tx.commit().await?;
        tracing::info!(
            "System settings updated by {}: {:?}",
            updated_by,
            settings.keys().collect::<Vec<_>>()
        );
        Ok(())
    }

    /// Resolve SMTP config: DB values take priority, .env Config as fallback
    pub async fn resolve_smtp_config(&self, config: &Config) -> SmtpConfig {
        let settings = self.get_all_settings().await.unwrap_or_default();

        let host = Self::get_string(&settings, "smtp_host")
            .or_else(|| config.smtp_host.clone());
        let port = Self::get_string(&settings, "smtp_port")
            .and_then(|s| s.parse().ok())
            .unwrap_or(config.smtp_port);
        let username = Self::get_string(&settings, "smtp_username")
            .or_else(|| config.smtp_username.clone());
        let password = Self::get_string(&settings, "smtp_password")
            .or_else(|| config.smtp_password.clone());
        let from_email = Self::get_string(&settings, "smtp_from_email")
            .unwrap_or_else(|| config.smtp_from_email.clone());
        let from_name = Self::get_string(&settings, "smtp_from_name")
            .unwrap_or_else(|| config.smtp_from_name.clone());

        SmtpConfig { host, port, username, password, from_email, from_name }
    }

    fn get_string(settings: &HashMap<String, Value>, key: &str) -> Option<String> {
        settings.get(key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    /// JSONB in Postgres wraps scalar values, so `"hello"` is stored as `"hello"`.
    /// We unwrap the outer layer if the value is a simple type stored as JSONB.
    fn unwrap_jsonb(value: Value) -> Value {
        value
    }

    fn wrap_jsonb(value: Value) -> Value {
        value
    }
}
