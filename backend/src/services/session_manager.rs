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
            ) VALUES ($1, $2, NOW(), NOW(), $3, $4, true)
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
    
    /// 更新 Session 活動時間
    pub async fn update_activity(pool: &PgPool, session_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_activity_at = NOW(),
                page_view_count = page_view_count + 1
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(session_id)
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
    
    /// 檢查 Session 是否有效
    pub async fn is_session_valid(pool: &PgPool, session_id: Uuid) -> Result<bool> {
        let (is_active,): (bool,) = sqlx::query_as(
            "SELECT is_active FROM user_sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or((false,));
        
        Ok(is_active)
    }
}
