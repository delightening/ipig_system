// Balance Expiration Job
// 處理特休和補休到期

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

pub struct BalanceExpirationJob;

impl BalanceExpirationJob {
    /// 執行到期處理
    pub async fn run(pool: &PgPool) -> Result<ExpirationSummary> {
        let today = Utc::now().date_naive();
        
        // 處理特休到期
        let annual_expired = Self::expire_annual_leave(pool, today).await?;
        
        // 處理補休到期
        let comp_time_expired = Self::expire_comp_time(pool, today).await?;
        
        // 發送到期通知
        Self::send_expiry_warnings(pool, today).await?;
        
        Ok(ExpirationSummary {
            annual_leave_expired: annual_expired,
            comp_time_expired,
            processed_at: Utc::now(),
        })
    }
    
    /// 處理特休到期
    async fn expire_annual_leave(pool: &PgPool, today: NaiveDate) -> Result<i64> {
        // 標記已到期的特休
        let result = sqlx::query(
            r#"
            UPDATE annual_leave_entitlements
            SET is_expired = true,
                expired_days = entitled_days - used_days,
                expiry_processed_at = NOW(),
                updated_at = NOW()
            WHERE expires_at < $1
              AND is_expired = false
              AND entitled_days > used_days
            "#,
        )
        .bind(today)
        .execute(pool)
        .await?;
        
        let expired_count = result.rows_affected() as i64;
        
        // 記錄到期明細
        if expired_count > 0 {
            tracing::info!("Expired {} annual leave entitlements", expired_count);
        }
        
        Ok(expired_count)
    }
    
    /// 處理補休到期
    async fn expire_comp_time(pool: &PgPool, today: NaiveDate) -> Result<i64> {
        // 標記已到期的補休
        let result = sqlx::query(
            r#"
            UPDATE comp_time_balances
            SET is_expired = true,
                expired_hours = original_hours - used_hours,
                expiry_processed_at = NOW(),
                updated_at = NOW()
            WHERE expires_at < $1
              AND is_expired = false
              AND original_hours > used_hours
            "#,
        )
        .bind(today)
        .execute(pool)
        .await?;
        
        let expired_count = result.rows_affected() as i64;
        
        if expired_count > 0 {
            tracing::info!("Expired {} comp time balances", expired_count);
        }
        
        Ok(expired_count)
    }
    
    /// 發送即將到期警告
    async fn send_expiry_warnings(pool: &PgPool, today: NaiveDate) -> Result<()> {
        let warning_days = 30; // 30 天前警告
        let warning_date = today + chrono::Duration::days(warning_days);
        
        // 查找即將到期的特休
        let expiring_annual: Vec<(Uuid, String, f64, NaiveDate)> = sqlx::query_as(
            r#"
            SELECT a.user_id, u.email, a.entitled_days - a.used_days as remaining, a.expires_at
            FROM annual_leave_entitlements a
            JOIN users u ON a.user_id = u.id
            WHERE a.expires_at BETWEEN $1 AND $2
              AND a.is_expired = false
              AND a.entitled_days > a.used_days
            "#,
        )
        .bind(today)
        .bind(warning_date)
        .fetch_all(pool)
        .await?;
        
        // 查找即將到期的補休
        let expiring_comp: Vec<(Uuid, String, f64, NaiveDate)> = sqlx::query_as(
            r#"
            SELECT c.user_id, u.email, c.original_hours - c.used_hours as remaining, c.expires_at
            FROM comp_time_balances c
            JOIN users u ON c.user_id = u.id
            WHERE c.expires_at BETWEEN $1 AND $2
              AND c.is_expired = false
              AND c.original_hours > c.used_hours
            "#,
        )
        .bind(today)
        .bind(warning_date)
        .fetch_all(pool)
        .await?;
        
        // 建立通知
        for (user_id, _email, remaining, expires_at) in expiring_annual {
            let days_left = (expires_at - today).num_days();
            sqlx::query(
                r#"
                INSERT INTO notifications (
                    id, user_id, type, title, message,
                    data, is_read, created_at
                ) VALUES ($1, $2, 'balance_expiry', $3, $4, $5, false, NOW())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind("特休即將到期提醒")
            .bind(format!(
                "您有 {:.1} 天特休將於 {} 到期（{}天後），請儘早安排使用。",
                remaining,
                expires_at.format("%Y/%m/%d"),
                days_left
            ))
            .bind(serde_json::json!({
                "type": "annual_leave",
                "remaining_days": remaining,
                "expires_at": expires_at.to_string(),
                "days_left": days_left
            }))
            .execute(pool)
            .await?;
        }
        
        for (user_id, _email, remaining, expires_at) in expiring_comp {
            let days_left = (expires_at - today).num_days();
            sqlx::query(
                r#"
                INSERT INTO notifications (
                    id, user_id, type, title, message,
                    data, is_read, created_at
                ) VALUES ($1, $2, 'balance_expiry', $3, $4, $5, false, NOW())
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind("補休即將到期提醒")
            .bind(format!(
                "您有 {:.1} 小時補休將於 {} 到期（{}天後），請儘早安排使用。",
                remaining,
                expires_at.format("%Y/%m/%d"),
                days_left
            ))
            .bind(serde_json::json!({
                "type": "comp_time",
                "remaining_hours": remaining,
                "expires_at": expires_at.to_string(),
                "days_left": days_left
            }))
            .execute(pool)
            .await?;
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExpirationSummary {
    pub annual_leave_expired: i64,
    pub comp_time_expired: i64,
    pub processed_at: chrono::DateTime<Utc>,
}
