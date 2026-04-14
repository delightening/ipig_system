// Session Manager Service
// 管理使用者 Sessions

use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

pub struct SessionManager;

impl SessionManager {
    /// 建立新 Session
    pub async fn create_session(
        pool: &PgPool,
        user_id: Uuid,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO user_sessions (
                id, user_id, started_at, last_activity_at,
                ip_address, user_agent, is_active
            ) VALUES ($1, $2, NOW(), NOW(), $3::INET, $4, true)
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .bind(ip)
        .bind(user_agent)
        .execute(pool)
        .await?;

        Ok(session_id)
    }

    /// SEC-AUDIT-009: 絕對 session 最大存活時間（分鐘）
    /// 無論活動與否，超過此時間的 session 一律失效
    const ABSOLUTE_SESSION_TIMEOUT_MINUTES: i32 = 480; // 8 小時

    /// 更新 Session 活動時間
    ///
    /// SEC-AUDIT-009: 加入絕對 session timeout 檢查，
    /// 即使持續有活動，超過 8 小時的 session 也會被強制結束。
    pub async fn update_activity(pool: &PgPool, session_id: Uuid, ip: Option<&str>) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_activity_at = NOW(),
                page_view_count = page_view_count + 1,
                ip_address = COALESCE($2::INET, ip_address)
            WHERE id = $1 AND is_active = true
              AND (NOW() - started_at) < INTERVAL '1 minute' * $3
            "#,
        )
        .bind(session_id)
        .bind(ip)
        .bind(Self::ABSOLUTE_SESSION_TIMEOUT_MINUTES)
        .execute(pool)
        .await?;

        // 若沒有更新任何列，表示 session 已超過絕對 timeout
        if result.rows_affected() == 0 {
            // 自動關閉超時 session
            sqlx::query(
                r#"
                UPDATE user_sessions
                SET is_active = false, ended_at = NOW(), ended_reason = 'absolute_timeout'
                WHERE id = $1 AND is_active = true
                  AND (NOW() - started_at) >= INTERVAL '1 minute' * $2
                "#,
            )
            .bind(session_id)
            .bind(Self::ABSOLUTE_SESSION_TIMEOUT_MINUTES)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    /// 透過 user_id 更新最近的 active session 活動時間與 IP
    pub async fn update_activity_by_user(
        pool: &PgPool,
        user_id: Uuid,
        ip: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_activity_at = NOW(),
                page_view_count = page_view_count + 1,
                ip_address = COALESCE($2::INET, ip_address)
            WHERE id = (
                SELECT id FROM user_sessions
                WHERE user_id = $1 AND is_active = true
                ORDER BY started_at DESC
                LIMIT 1
            )
            "#,
        )
        .bind(user_id)
        .bind(ip)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 記錄操作
    pub async fn record_action(pool: &PgPool, session_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_activity_at = NOW(),
                action_count = action_count + 1
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 結束 Session（正常登出）
    pub async fn end_session(pool: &PgPool, session_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'logout'
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 結束使用者的所有 Sessions（強制登出）
    pub async fn end_all_sessions(pool: &PgPool, user_id: Uuid, reason: &str) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = $2
            WHERE user_id = $1 AND is_active = true
            "#,
        )
        .bind(user_id)
        .bind(reason)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// 強制登出單一 Session
    pub async fn force_logout(
        pool: &PgPool,
        session_id: Uuid,
        admin_id: Uuid,
        reason: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'forced_logout'
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        // 記錄到審計日誌
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, actor_user_id, action, entity_type, entity_id,
                after_data, created_at
            ) VALUES ($1, $2, 'force_logout', 'session', $3, $4, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(session_id)
        .bind(serde_json::json!({ "reason": reason.unwrap_or("admin_action") }))
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 清理過期 Sessions
    pub async fn cleanup_expired(pool: &PgPool, inactive_minutes: i32) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'timeout'
            WHERE is_active = true
              AND last_activity_at < NOW() - INTERVAL '1 minute' * $1
            "#,
        )
        .bind(inactive_minutes)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// 取得使用者活躍 Session 數量
    pub async fn get_active_session_count(pool: &PgPool, user_id: Uuid) -> Result<i64> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND is_active = true",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// 結束超過上限的最舊 Sessions（SEC-28: Session 併發限制）
    ///
    /// SEC-AUDIT-004: 使用單一 SQL 語句原子性執行 count + cleanup，
    /// 消除先前 check-then-act 的 TOCTOU 競爭條件。
    pub async fn end_excess_sessions(
        pool: &PgPool,
        user_id: Uuid,
        max_sessions: i64,
    ) -> Result<()> {
        // 原子性操作：在單一 SQL 中計算超額 sessions 並關閉最舊的
        // 使用 subquery 避免先 SELECT COUNT 再 UPDATE 的 TOCTOU 問題
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_active = false,
                ended_at = NOW(),
                ended_reason = 'session_limit'
            WHERE id IN (
                SELECT id FROM (
                    SELECT id, ROW_NUMBER() OVER (
                        PARTITION BY user_id ORDER BY started_at DESC
                    ) AS rn
                    FROM user_sessions
                    WHERE user_id = $1 AND is_active = true
                ) ranked
                WHERE rn > $2
            )
            "#,
        )
        .bind(user_id)
        .bind(max_sessions)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// 檢查 Session 是否有效
    pub async fn is_session_valid(pool: &PgPool, session_id: Uuid) -> Result<bool> {
        let (is_active,): (bool,) =
            sqlx::query_as("SELECT is_active FROM user_sessions WHERE id = $1")
                .bind(session_id)
                .fetch_optional(pool)
                .await?
                .unwrap_or((false,));

        Ok(is_active)
    }
}
