use anyhow::Context;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_retry_attempts: u32,
    pub database_retry_delay_seconds: u64,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub jwt_expiration_seconds: i64,
    pub jwt_refresh_expiration_days: i64,
    /// 每個使用者同時可擁有的最大活躍 Session 數量（SEC-28）
    pub max_sessions_per_user: i64,
    // Email settings
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub app_url: String,
    // Cookie settings
    pub cookie_secure: bool,
    pub cookie_domain: Option<String>,
    // Development settings
    pub seed_dev_users: bool,
    /// 打卡允許的 IP 範圍（CIDR 格式，如 "192.168.1.0/24,10.0.0.1"）
    /// 空陣列表示不限制
    pub allowed_clock_ip_ranges: Vec<String>,
    /// 辦公室 GPS 座標（None 表示不啟用 GPS 驗證）
    pub clock_office_latitude: Option<f64>,
    pub clock_office_longitude: Option<f64>,
    /// GPS 打卡允許半徑（公尺），預設 200
    pub clock_gps_radius_meters: f64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("PORT must be a number")?,
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            database_max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("DATABASE_MAX_CONNECTIONS must be a number")?,
            database_retry_attempts: std::env::var("DATABASE_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("DATABASE_RETRY_ATTEMPTS must be a number")?,
            database_retry_delay_seconds: std::env::var("DATABASE_RETRY_DELAY_SECONDS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("DATABASE_RETRY_DELAY_SECONDS must be a number")?,
            jwt_secret: {
                let secret = std::env::var("JWT_SECRET")
                    .context("JWT_SECRET must be set")?;
                // SEC-21: JWT Secret 最小長度驗證，防止使用弱金鑰
                if secret.len() < 32 {
                    anyhow::bail!(
                        "JWT_SECRET 長度不足：目前 {} 字元，至少需要 32 字元。\n\
                         建議使用 `openssl rand -base64 48` 產生安全金鑰。",
                        secret.len()
                    );
                }
                secret
            },
            jwt_expiration_hours: std::env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .context("JWT_EXPIRATION_HOURS must be a number")?,
            // SEC-25: 優先使用分鐘級設定，預設 15 分鐘
            jwt_expiration_seconds: {
                if let Ok(mins) = std::env::var("JWT_EXPIRATION_MINUTES") {
                    let m: i64 = mins.parse().context("JWT_EXPIRATION_MINUTES must be a number")?;
                    m * 60
                } else if let Ok(hrs) = std::env::var("JWT_EXPIRATION_HOURS") {
                    let h: i64 = hrs.parse().unwrap_or(0);
                    if h > 0 { h * 3600 } else { 900 } // 預設 15 分鐘
                } else {
                    900 // 15 分鐘
                }
            },
            jwt_refresh_expiration_days: std::env::var("JWT_REFRESH_EXPIRATION_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .context("JWT_REFRESH_EXPIRATION_DAYS must be a number")?,
            max_sessions_per_user: std::env::var("MAX_SESSIONS_PER_USER")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("MAX_SESSIONS_PER_USER must be a number")?,
            // Email settings
            smtp_host: std::env::var("SMTP_HOST").ok(),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: std::env::var("SMTP_USERNAME").ok(),
            smtp_password: std::env::var("SMTP_PASSWORD").ok(),
            smtp_from_email: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@erp.local".to_string()),
            smtp_from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "ERP System".to_string()),
            app_url: std::env::var("APP_URL")
                .unwrap_or_else(|_| "http://localhost".to_string()),
            cookie_secure: std::env::var("COOKIE_SECURE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            cookie_domain: std::env::var("COOKIE_DOMAIN").ok().filter(|s| !s.is_empty()),
            seed_dev_users: std::env::var("SEED_DEV_USERS")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            allowed_clock_ip_ranges: std::env::var("ALLOWED_CLOCK_IP_RANGES")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            clock_office_latitude: std::env::var("CLOCK_OFFICE_LATITUDE")
                .ok()
                .and_then(|s| s.parse().ok()),
            clock_office_longitude: std::env::var("CLOCK_OFFICE_LONGITUDE")
                .ok()
                .and_then(|s| s.parse().ok()),
            clock_gps_radius_meters: std::env::var("CLOCK_GPS_RADIUS_METERS")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200.0),
        })
    }

    pub fn is_email_enabled(&self) -> bool {
        self.smtp_host.is_some()
    }
}
