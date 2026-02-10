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
        let ua = parse_user_agent(user_agent); // Keep this line, as `ua` is used later
        let (is_unusual_time, is_unusual_location, is_new_device, is_mass_login) = tokio::join!(
            async { check_unusual_time() },
            async { check_unusual_location(pool, user_id, ip).await },
            async { check_new_device(pool, user_id, user_agent).await },
            async { check_mass_login(pool, user_id).await }
        );
        
        // 插入登入成功日誌
        sqlx::query(
            r#"
            INSERT INTO login_events (
                id, user_id, email, event_type, 
                ip_address, user_agent, device_type, browser, os,
                is_unusual_time, is_unusual_location, is_new_device, is_mass_login,
                created_at
            ) VALUES (
                $1, $2, $3, 'login_success',
                $4, $5, $6, $7, $8,
                $9, $10, $11, $12, NOW()
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(email)
        .bind(ip)
        .bind(user_agent)
        .bind(&ua.device_type) // Use ua.device_type
        .bind(&ua.browser)     // Use ua.browser
        .bind(&ua.os)          // Use ua.os
        .bind(is_unusual_time)
        .bind(is_unusual_location)
        .bind(is_new_device)
        .bind(is_mass_login)
        .execute(pool)
        .await?;
        
        // 如果有異常，建立個人警報
        if is_unusual_time || is_unusual_location || is_new_device || is_mass_login {
            Self::create_login_alert(
                pool,
                user_id,
                email,
                is_unusual_time,
                is_unusual_location,
                is_new_device,
                is_mass_login,
            )
            .await?;
        }

        // 檢查全域多帳號大量登入 (疑似腳本)
        check_global_mass_login(pool).await?;
        
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
                $4::INET, $5, $6, $7, $8, $9, NOW()
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
            VALUES ($1, $2, $3, 'logout', $4::INET, NOW())
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
                    context_data, created_at, updated_at, status
                ) VALUES (
                    $1, 'brute_force', 'critical',
                    '偵測到可能的暴力破解攻擊',
                    $2, $3, NOW(), NOW(), 'open'
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
        email: &str,
        unusual_time: bool,
        unusual_location: bool,
        new_device: bool,
        mass_login: bool,
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
        if mass_login {
            reasons.push("同時大量登入");
        }
        
        let title = format!("偵測到異常登入 ({})", email);
        let description = format!("帳號 {} 的登入觸發異常偵測：{}", email, reasons.join("、"));
        
        sqlx::query(
            r#"
            INSERT INTO security_alerts (
                id, alert_type, severity, title, description,
                user_id, created_at, updated_at, status
            ) VALUES (
                $1, 'unusual_login', 'warning',
                $2, $3, $4, NOW(), NOW(), 'open'
            )
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&title)
        .bind(&description)
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
    // 台灣時區 UTC+8
    let taiwan_hour = (Utc::now().hour() + 8) % 24;
    // 非工作時間：晚上 10 點到早上 6 點（台灣時間）
    taiwan_hour >= 22 || taiwan_hour < 6
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

async fn check_mass_login(pool: &PgPool, user_id: Uuid) -> bool {
    // 檢查過去 15 分鐘內成功登入次數
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM login_events
        WHERE user_id = $1
          AND event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '15 minutes'
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    // 如果連同本次（尚未寫入前的查詢）已有 4 次以上，則本次標記為大量登入
    count >= 4
}

async fn check_global_mass_login(pool: &PgPool) -> Result<()> {
    // 檢查過去 5 分鐘內，成功登入的不同帳號數量
    let (count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT user_id) FROM login_events
        WHERE event_type = 'login_success'
          AND created_at > NOW() - INTERVAL '5 minutes'
        "#,
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    // 如果 5 分鐘內超過 10 個不同帳號登入，觸發全域警報
    if count >= 10 {
        create_global_mass_login_alert(pool, count).await?;
    }

    Ok(())
}

async fn create_global_mass_login_alert(pool: &PgPool, account_count: i64) -> Result<()> {
    // 檢查是否最近 10 分鐘內已經發過相同的全域警報 (避免洗版)
    let (recent_alert_count,): (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM security_alerts
        WHERE alert_type = 'global_mass_login'
          AND created_at > NOW() - INTERVAL '10 minutes'
          AND status = 'open'
        "#,
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    if recent_alert_count > 0 {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO security_alerts (
            id, alert_type, severity, title, description,
            context_data, created_at, updated_at, status
        ) VALUES (
            $1, 'global_mass_login', 'critical',
            '偵測到疑似腳本之多帳號大量登入',
            $2, $3, NOW(), NOW(), 'open'
        )
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(format!(
        "系統偵測到全域在過去 5 分鐘內有 {} 個不同帳號成功登入，疑似為自動化腳本行為。",
        account_count
    ))
    .bind(serde_json::json!({
        "account_count": account_count,
        "time_window_minutes": 5
    }))
    .execute(pool)
    .await?;

    Ok(())
}
