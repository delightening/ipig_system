// Login Tracker Service
// 追蹤登入事件並檢測異常

use chrono::{Timelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

pub struct LoginTracker;

impl LoginTracker {
    /// 記錄成功登入
    pub async fn log_success(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        let device_info = parse_user_agent(user_agent);
        let is_unusual_time = check_unusual_time();
        let is_new_device = check_new_device(pool, user_id, user_agent).await;
        let is_unusual_location = check_unusual_location(pool, user_id, ip).await;
        
        sqlx::query(
            r#"
            INSERT INTO login_events (
                id, user_id, email, event_type,
                ip_address, user_agent,
                device_type, browser, os,
                is_unusual_time, is_unusual_location, is_new_device,
                created_at
            ) VALUES (
                $1, $2, $3, 'login_success',
                $4, $5, $6, $7, $8, $9, $10, $11, NOW()
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .bind(user_agent)
        .bind(&device_info.device_type)
        .bind(&device_info.browser)
        .bind(&device_info.os)
        .bind(is_unusual_time)
        .bind(is_unusual_location)
        .bind(is_new_device)
        .execute(pool)
        .await?;
        
        // 如果有異常，建立警報
        if is_unusual_time || is_unusual_location || is_new_device {
            Self::create_login_alert(
                pool,
                user_id,
                is_unusual_time,
                is_unusual_location,
                is_new_device,
            )
            .await?;
        }
        
        Ok(())
    }
    
    /// 記錄失敗登入
    pub async fn log_failure(
        pool: &PgPool,
        email: &str,
        ip: Option<&str>,
        user_agent: Option<&str>,
        reason: &str,
    ) -> Result<()> {
        let device_info = parse_user_agent(user_agent);
        
        // 查找 user_id（如果 email 存在）
        let user_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;
        
        sqlx::query(
            r#"
            INSERT INTO login_events (
                id, user_id, email, event_type,
                ip_address, user_agent,
                device_type, browser, os,
                failure_reason,
                created_at
            ) VALUES (
                $1, $2, $3, 'login_failure',
                $4, $5, $6, $7, $8, $9, NOW()
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .bind(user_agent)
        .bind(&device_info.device_type)
        .bind(&device_info.browser)
        .bind(&device_info.os)
        .bind(reason)
        .execute(pool)
        .await?;
        
        // 檢查暴力破解
        Self::check_brute_force(pool, email, ip).await?;
        
        Ok(())
    }
    
    /// 記錄登出
    pub async fn log_logout(
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        ip: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO login_events (id, user_id, email, event_type, ip_address, created_at)
            VALUES ($1, $2, $3, 'logout', $4, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// 檢查暴力破解攻擊
    async fn check_brute_force(pool: &PgPool, email: &str, ip: Option<&str>) -> Result<()> {
        // 檢查過去 15 分鐘的失敗次數
        let (fail_count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM login_events
            WHERE email = $1
              AND event_type = 'login_failure'
              AND created_at > NOW() - INTERVAL '15 minutes'
            "#,
        )
        .bind(email)
        .fetch_one(pool)
        .await?;
        
        if fail_count >= 5 {
            // 建立暴力破解警報
            sqlx::query(
                r#"
                INSERT INTO security_alerts (
                    id, alert_type, severity, title, description,
                    metadata, detected_at, status
                ) VALUES (
                    $1, 'brute_force', 'high',
                    '偵測到可能的暴力破解攻擊',
                    $2, $3, NOW(), 'open'
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(format!(
                "Email {} 在過去 15 分鐘內有 {} 次失敗登入嘗試",
                email, fail_count
            ))
            .bind(serde_json::json!({
                "email": email,
                "ip": ip,
                "fail_count": fail_count
            }))
            .execute(pool)
            .await?;
        }
        
        Ok(())
    }
    
    /// 建立登入異常警報
    async fn create_login_alert(
        pool: &PgPool,
        user_id: Uuid,
        unusual_time: bool,
        unusual_location: bool,
        new_device: bool,
    ) -> Result<()> {
        let mut reasons = Vec::new();
        if unusual_time {
            reasons.push("非工作時間登入");
        }
        if unusual_location {
            reasons.push("來自新的 IP 位置");
        }
        if new_device {
            reasons.push("使用新裝置");
        }
        
        sqlx::query(
            r#"
            INSERT INTO security_alerts (
                id, alert_type, severity, title, description,
                user_id, detected_at, status
            ) VALUES (
                $1, 'unusual_login', 'medium',
                '偵測到異常登入',
                $2, $3, NOW(), 'open'
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(reasons.join("、"))
        .bind(user_id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
}

// ============================================
// Helper Functions
// ============================================

struct DeviceInfo {
    device_type: Option<String>,
    browser: Option<String>,
    os: Option<String>,
}

fn parse_user_agent(ua: Option<&str>) -> DeviceInfo {
    let ua = match ua {
        Some(s) => s,
        None => {
            return DeviceInfo {
                device_type: None,
                browser: None,
                os: None,
            }
        }
    };
    
    // 簡單解析 (可以用更完整的 library 如 woothee)
    let device_type = if ua.contains("Mobile") || ua.contains("Android") {
        Some("mobile".to_string())
    } else if ua.contains("Tablet") || ua.contains("iPad") {
        Some("tablet".to_string())
    } else {
        Some("desktop".to_string())
    };
    
    let browser = if ua.contains("Chrome") && !ua.contains("Edge") {
        Some("Chrome".to_string())
    } else if ua.contains("Firefox") {
        Some("Firefox".to_string())
    } else if ua.contains("Safari") && !ua.contains("Chrome") {
        Some("Safari".to_string())
    } else if ua.contains("Edge") {
        Some("Edge".to_string())
    } else {
        None
    };
    
    let os = if ua.contains("Windows") {
        Some("Windows".to_string())
    } else if ua.contains("Mac OS") {
        Some("macOS".to_string())
    } else if ua.contains("Linux") {
        Some("Linux".to_string())
    } else if ua.contains("Android") {
        Some("Android".to_string())
    } else if ua.contains("iOS") || ua.contains("iPhone") {
        Some("iOS".to_string())
    } else {
        None
    };
    
    DeviceInfo {
        device_type,
        browser,
        os,
    }
}

fn check_unusual_time() -> bool {
    let hour = Utc::now().hour();
    // 非工作時間：晚上 10 點到早上 6 點
    hour >= 22 || hour < 6
}

async fn check_new_device(pool: &PgPool, user_id: Uuid, user_agent: Option<&str>) -> bool {
    let ua = match user_agent {
        Some(s) => s,
        None => return false,
    };
    
    // 檢查過去 30 天是否用過這個 user agent
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM login_events
        WHERE user_id = $1
          AND user_agent = $2
          AND event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(user_id)
    .bind(ua)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    
    count == 0
}

async fn check_unusual_location(pool: &PgPool, user_id: Uuid, ip: Option<&str>) -> bool {
    let ip = match ip {
        Some(s) => s,
        None => return false,
    };
    
    // 簡化版：檢查是否為新 IP（30 天內未見過）
    // 完整版應該用 GeoIP 比對地理位置
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM login_events
        WHERE user_id = $1
          AND ip_address = $2
          AND event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(user_id)
    .bind(ip)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    
    count == 0
}
