use std::collections::HashMap;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;

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

    pub async fn update_settings(
        &self,
        settings: &HashMap<String, Value>,
        updated_by: Uuid,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

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
        }

        tx.commit().await?;
        tracing::info!("System settings updated: {:?}", settings.keys().collect::<Vec<_>>());
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
