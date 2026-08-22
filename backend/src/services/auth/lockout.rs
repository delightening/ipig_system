//! 帳號鎖定的查詢與管理員手動解除。
//!
//! 鎖定沒有持久旗標：`validate_credentials` 每次登入現算「視窗內的 login_failure
//! 筆數 >= 門檻」。因此「解鎖」不是清一個 flag，而是在 `login_events` 寫一筆
//! **解鎖標記**，把計數起點推到那一刻——失敗紀錄原封不動留在稽核鏈上，只是不再計入。
//!
//! 標記機制沿用 #1086（密碼變更寫 `lockout_reset`），本檔補上管理員手動的
//! `lockout_admin_clear`。兩種標記在計數端等價，分開型別是為了日後查稽核時
//! 分得出「使用者自己改了密碼」還是「管理員放行」。

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    config::Config,
    constants::{LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR, LOGIN_EVENT_LOCKOUT_RESET},
    models::AccountLockoutStatus,
    AppError, Result,
};

use super::AuthService;

impl AuthService {
    /// 計算目前計入鎖定的失敗次數。
    ///
    /// 起算點取「N 分鐘前」與「最後一筆解鎖標記」的較晚者。登入路徑與 admin 後台
    /// 查詢共用同一份判定，避免兩邊算出不同結果、後台顯示「已解鎖」但實際還進不去。
    pub(crate) async fn count_active_failures<'e, E>(
        executor: E,
        email: &str,
        window_minutes: i64,
    ) -> Result<i64>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let (fail_count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM login_events
            WHERE email = $1
              AND event_type = 'login_failure'
              AND created_at > NOW() - make_interval(mins => $2::integer)
              AND created_at > COALESCE(
                    (SELECT MAX(created_at) FROM login_events
                      WHERE email = $1 AND event_type IN ($3, $4)),
                    '-infinity'::timestamptz)
            "#,
        )
        .bind(email)
        .bind(window_minutes)
        .bind(LOGIN_EVENT_LOCKOUT_RESET)
        .bind(LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR)
        .fetch_one(executor)
        .await?;

        Ok(fail_count)
    }

    /// 查帳號目前的鎖定狀態（唯讀，供 admin 後台顯示）。
    pub async fn account_lock_status(
        pool: &PgPool,
        config: &Config,
        email: &str,
    ) -> Result<AccountLockoutStatus> {
        let window = config.account_lockout_duration_minutes;
        let max_attempts = config.account_lockout_max_attempts;

        let (user_exists, cleared_at): (bool, Option<DateTime<Utc>>) = sqlx::query_as(
            r#"
            SELECT
                EXISTS (SELECT 1 FROM users WHERE email = $1),
                (SELECT MAX(created_at) FROM login_events
                  WHERE email = $1 AND event_type IN ($2, $3))
            "#,
        )
        .bind(email)
        .bind(LOGIN_EVENT_LOCKOUT_RESET)
        .bind(LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR)
        .fetch_one(pool)
        .await?;

        let fail_count = Self::count_active_failures(pool, email, window).await?;
        let locked = !config.disable_account_lockout && fail_count >= max_attempts;

        // 自動解鎖時刻＝「第 (fail_count - max_attempts + 1) 舊的那筆失敗」滑出視窗的時間，
        // 也就是計數第一次掉到門檻以下的瞬間。OFFSET 從 0 起算，故偏移量為兩者之差。
        let unlock_at = if locked {
            Self::next_unlock_at(pool, email, window, fail_count - max_attempts).await?
        } else {
            None
        };

        Ok(AccountLockoutStatus {
            email: email.to_string(),
            user_exists,
            locked,
            fail_count,
            max_attempts,
            unlock_at,
            cleared_at,
        })
    }

    async fn next_unlock_at(
        pool: &PgPool,
        email: &str,
        window_minutes: i64,
        offset: i64,
    ) -> Result<Option<DateTime<Utc>>> {
        let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
            r#"
            SELECT created_at + make_interval(mins => $2::integer)
            FROM login_events
            WHERE email = $1
              AND event_type = 'login_failure'
              AND created_at > NOW() - make_interval(mins => $2::integer)
              AND created_at > COALESCE(
                    (SELECT MAX(created_at) FROM login_events
                      WHERE email = $1 AND event_type IN ($4, $5)),
                    '-infinity'::timestamptz)
            ORDER BY created_at ASC
            OFFSET $3 LIMIT 1
            "#,
        )
        .bind(email)
        .bind(window_minutes)
        .bind(offset.max(0))
        .bind(LOGIN_EVENT_LOCKOUT_RESET)
        .bind(LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|(unlock_at,)| unlock_at))
    }

    /// 管理員手動解除帳號鎖定：寫一筆 `lockout_admin_clear` 標記，回傳被解鎖的 user id。
    ///
    /// email 沒有對應帳號時回 404——`INSERT ... SELECT` 找不到 user 就不會插入任何列，
    /// `RETURNING` 因此無列可回。不存在的帳號本來就登不進去，沒有東西可解
    ///（帳號枚舉嘗試同樣會產生 brute_force 警報，那種警報按解鎖是無效操作）。
    pub async fn clear_account_lockout(pool: &PgPool, email: &str) -> Result<Uuid> {
        let user_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO login_events (id, user_id, email, event_type, created_at)
            SELECT $1, u.id, u.email, $3, NOW() FROM users u WHERE u.email = $2
            RETURNING user_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(email)
        .bind(LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR)
        .fetch_optional(pool)
        .await?;

        user_id.ok_or_else(|| AppError::NotFound(format!("找不到 email 為 {email} 的帳號")))
    }
}
