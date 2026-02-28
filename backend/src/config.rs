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
    /// SEC-30: 是否信任反向代理 header（如 X-Forwarded-For, X-Real-Ip）
    /// 設為 true 表示後端在反向代理/Cloudflare Tunnel 後方，可信任 proxy header
    /// 設為 false 表示直接面向外網，僅使用 socket IP
    pub trust_proxy_headers: bool,
    /// SEC-31: CORS 允許的 Origin 清單
    pub cors_allowed_origins: Vec<String>,
    /// SEC-34: 稽核日誌 HMAC-SHA256 密鑰
    pub audit_hmac_key: Option<String>,
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
            // SEC-32: 統一使用 JWT_EXPIRATION_MINUTES，預設 360 分鐘（6 小時）
            jwt_expiration_seconds: {
                let mins: i64 = std::env::var("JWT_EXPIRATION_MINUTES")
                    .unwrap_or_else(|_| "360".to_string())
                    .parse()
                    .context("JWT_EXPIRATION_MINUTES must be a number")?;
                mins * 60
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
            // SEC-30: IP Header 信任策略
            trust_proxy_headers: std::env::var("TRUST_PROXY_HEADERS")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true),
            // SEC-31: CORS 允許的 Origin 清單
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            // SEC-34: 稽核日誌 HMAC 密鑰
            audit_hmac_key: std::env::var("AUDIT_HMAC_KEY").ok().filter(|s| s.len() >= 16),
        })
    }

    pub fn is_email_enabled(&self) -> bool {
        self.smtp_host.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 產生最小可用 Config（不依賴環境變數）
    fn minimal_config() -> Config {
        Config {
            host: "0.0.0.0".to_string(),
            port: 3000,
            database_url: "postgres://test:test@localhost/test".to_string(),
            database_max_connections: 10,
            database_retry_attempts: 5,
            database_retry_delay_seconds: 5,
            jwt_secret: "a".repeat(32),
            jwt_expiration_seconds: 21600,
            jwt_refresh_expiration_days: 7,
            max_sessions_per_user: 5,
            smtp_host: None,
            smtp_port: 587,
            smtp_username: None,
            smtp_password: None,
            smtp_from_email: "noreply@test.local".to_string(),
            smtp_from_name: "Test".to_string(),
            app_url: "http://localhost".to_string(),
            cookie_secure: false,
            cookie_domain: None,
            seed_dev_users: false,
            allowed_clock_ip_ranges: vec![],
            clock_office_latitude: None,
            clock_office_longitude: None,
            clock_gps_radius_meters: 200.0,
            trust_proxy_headers: true,
            cors_allowed_origins: vec!["http://localhost:8080".to_string()],
            audit_hmac_key: None,
        }
    }

    #[test]
    fn test_email_disabled_when_no_host() {
        let config = minimal_config();
        assert!(!config.is_email_enabled());
    }

    #[test]
    fn test_email_enabled_when_host_set() {
        let mut config = minimal_config();
        config.smtp_host = Some("smtp.example.com".to_string());
        assert!(config.is_email_enabled());
    }

    #[test]
    fn test_default_gps_radius() {
        let config = minimal_config();
        assert_eq!(config.clock_gps_radius_meters, 200.0);
    }

    #[test]
    fn test_jwt_secret_min_length() {
        let config = minimal_config();
        assert!(config.jwt_secret.len() >= 32);
    }

    #[test]
    fn test_audit_hmac_key_none_by_default() {
        let config = minimal_config();
        assert!(config.audit_hmac_key.is_none());
    }

    #[test]
    fn test_cors_origins_default() {
        let config = minimal_config();
        assert_eq!(config.cors_allowed_origins, vec!["http://localhost:8080"]);
    }

    #[test]
    fn test_cookie_secure_default_false() {
        let config = minimal_config();
        assert!(!config.cookie_secure);
    }
}
