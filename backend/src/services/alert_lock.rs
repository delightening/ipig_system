//! 從安全警報反查「這則警報的對象現在是不是被鎖住了」。
//!
//! 管理員在安全警報頁看到一則警報時，關心的下一步通常是「那要不要放行」。
//! 但鎖定資訊散在兩個地方：帳號鎖定是 `login_events` 的滑動視窗計數，
//! IP 封鎖在 `ip_blocklist` 表。本 service 把兩者收斂成單一查詢，
//! 讓前端不必知道各種 alert_type 的 context_data 長什麼樣。

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    models::AlertLockStatus,
    services::{AuthService, IpBlocklistService},
    AppError, Result,
};

pub struct AlertLockService;

impl AlertLockService {
    /// 查警報對象目前的帳號鎖定 / IP 封鎖狀態。
    pub async fn status(pool: &PgPool, config: &Config, alert_id: Uuid) -> Result<AlertLockStatus> {
        let alert: Option<(Option<Uuid>, Option<Value>)> =
            sqlx::query_as("SELECT user_id, context_data FROM security_alerts WHERE id = $1")
                .bind(alert_id)
                .fetch_optional(pool)
                .await?;

        let (user_id, context) = alert.ok_or_else(|| AppError::NotFound("警報不存在".into()))?;

        // email 來源二選一：brute_force 直接寫在 context_data，
        // unusual_login 等以 user_id 關聯的則回查 users。
        let email = match context_text(&context, "email") {
            Some(email) => Some(email),
            None => find_email_by_user(pool, user_id).await?,
        };

        // reused_ip 是 REFRESH_TOKEN_REUSE 警報記錄來源 IP 的欄位名（見該警報的處理 SOP）
        let ip = context_text(&context, "ip").or_else(|| context_text(&context, "reused_ip"));

        let account = match email {
            Some(email) => Some(AuthService::account_lock_status(pool, config, &email).await?),
            None => None,
        };
        let ip_block =
            IpBlocklistService::find_active_by_alert_or_ip(pool, alert_id, ip.as_deref()).await?;

        Ok(AlertLockStatus {
            account,
            ip: ip_block,
        })
    }
}

fn context_text(context: &Option<Value>, key: &str) -> Option<String> {
    context.as_ref()?.get(key)?.as_str().map(str::to_string)
}

async fn find_email_by_user(pool: &PgPool, user_id: Option<Uuid>) -> Result<Option<String>> {
    let Some(user_id) = user_id else {
        return Ok(None);
    };
    let row: Option<(String,)> = sqlx::query_as("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(email,)| email))
}
