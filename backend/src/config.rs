use anyhow::Context;
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::constants::{
    ACCOUNT_LOCKOUT_DURATION_MINUTES, ACCOUNT_LOCKOUT_MAX_ATTEMPTS, MAX_SESSIONS_PER_USER,
};

/// ES256（ECDSA P-256）金鑰對，預解析以避免每次請求重新 parse PEM。
/// EncodingKey / DecodingKey 不實作 Debug，故手動實作。
#[derive(Clone)]
pub struct JwtKeys {
    /// 私鑰，用於簽發 JWT（signing）
    pub encoding: EncodingKey,
    /// 公鑰，用於驗證 JWT（verification）
    pub decoding: DecodingKey,
}

impl std::fmt::Debug for JwtKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtKeys")
            .field("encoding", &"[EC P-256 private key]")
            .field("decoding", &"[EC P-256 public key]")
            .finish()
    }
}

/// 解析 boolean 環境變數，接受 "true" / "1"（大小寫不限），預設 false
fn parse_bool_env(key: &str) -> bool {
    std::env::var(key)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

/// Read a secret value: prefer `{key}_FILE` (Docker Secrets path), fallback to `{key}` env var.
fn read_secret(key: &str) -> Option<String> {
    let file_key = format!("{}_FILE", key);
    if let Ok(path) = std::env::var(&file_key) {
        match std::fs::read_to_string(&path) {
            Ok(content) => return Some(content.trim().to_string()),
            Err(e) => tracing::warn!("Failed to read secret file {}: {}", path, e),
        }
    }
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

/// Like `read_secret` but returns an error if neither source is available.
fn require_secret(key: &str) -> anyhow::Result<String> {
    read_secret(key).with_context(|| format!("{key} (or {key}_FILE) must be set"))
}

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    /// 連線池最小維持連線數（預熱，減少取得連線延遲）
    pub database_min_connections: u32,
    /// 從連線池取得連線的逾時秒數（逾時回傳 PoolTimedOut）
    pub database_acquire_timeout_seconds: u64,
    pub database_retry_attempts: u32,
    pub database_retry_delay_seconds: u64,
    /// ES256 金鑰對（私鑰簽發、公鑰驗證）
    /// 讀取 JWT_EC_PRIVATE_KEY / JWT_EC_PUBLIC_KEY 環境變數（或 _FILE 後綴版本）
    pub jwt_keys: JwtKeys,
    /// CSRF HMAC 密鑰（應與 JWT 金鑰隔離，防止單一金鑰洩漏同時破壞兩種保護機制）
    /// 讀取 CSRF_SECRET 環境變數；若未設定，從私鑰 PEM 派生（向後相容）
    pub csrf_secret: String,
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
    /// 整合測試用：停用 CSRF 檢查（僅在 TEST_DATABASE_URL/DATABASE_URL 且 DISABLE_CSRF_FOR_TESTS=true 時使用）
    pub disable_csrf_for_tests: bool,
    /// SEC-20: 帳號鎖定功能（DISABLE_ACCOUNT_LOCKOUT=true 可關閉）
    pub disable_account_lockout: bool,
    /// SEC-20: 帳號鎖定最大失敗次數，預設 5
    pub account_lockout_max_attempts: i64,
    /// SEC-20: 帳號鎖定持續時間（分鐘），預設 15
    pub account_lockout_duration_minutes: i64,
    /// 檔案上傳目錄，預設 ./uploads
    pub upload_dir: String,
    /// GeoIP 資料庫路徑
    pub geoip_db_path: String,
    /// 是否跳過 migration 檢查（僅開發環境，從 dump 還原後使用）
    pub skip_migration_check: bool,
    /// 管理員初始密碼（啟動時建立/驗證 admin 帳號用）
    pub admin_initial_password: Option<String>,
    /// 測試帳號密碼（SEED_DEV_USERS=true 時用於 startup 檢查）
    pub test_user_password: Option<String>,
    /// 開發帳號密碼（SEED_DEV_USERS=true 時的開發帳號密碼）
    pub dev_user_password: Option<String>,
    /// 是否在 CI 環境中執行
    pub is_ci: bool,
    /// Gotenberg PDF 服務 URL
    pub gotenberg_url: String,
    /// Image Processor 微服務 URL（方案 D）
    pub image_processor_url: String,
    /// R17-4: Prometheus /metrics 端點 Bearer Token（未設定則無認證）
    pub metrics_token: Option<String>,
    /// R20-5: Anthropic API Key（AI 預審用）
    pub anthropic_api_key: Option<String>,
    /// R20-5: AI 預審模型，預設 claude-haiku-4-5
    pub ai_review_model: String,
    /// R20-5: 是否啟用 AI 預審，預設 true
    pub ai_review_enabled: bool,
    /// R20-5: AI 預審 API 呼叫逾時秒數，預設 30
    pub ai_review_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("PORT must be a number")?,
            database_url: require_secret("DATABASE_URL")?,
            database_max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "40".to_string())
                .parse()
                .context("DATABASE_MAX_CONNECTIONS must be a number")?,
            database_min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("DATABASE_MIN_CONNECTIONS must be a number")?,
            database_acquire_timeout_seconds: std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("DATABASE_ACQUIRE_TIMEOUT_SECONDS must be a number")?,
            database_retry_attempts: std::env::var("DATABASE_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("DATABASE_RETRY_ATTEMPTS must be a number")?,
            database_retry_delay_seconds: std::env::var("DATABASE_RETRY_DELAY_SECONDS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("DATABASE_RETRY_DELAY_SECONDS must be a number")?,
            // CRIT-02: 使用非對稱式 ES256（ECDSA P-256）取代對稱式 HS256
            // 金鑰材料從環境變數（或 _FILE Docker Secrets）讀取，啟動時預解析
            jwt_keys: {
                let private_pem = require_secret("JWT_EC_PRIVATE_KEY")
                    .context("JWT_EC_PRIVATE_KEY（或 JWT_EC_PRIVATE_KEY_FILE）必須設定\n\
                              產生方式：openssl ecparam -name prime256v1 -genkey -noout | \\\n\
                                         openssl pkcs8 -topk8 -nocrypt")?;
                let public_pem = require_secret("JWT_EC_PUBLIC_KEY")
                    .context("JWT_EC_PUBLIC_KEY（或 JWT_EC_PUBLIC_KEY_FILE）必須設定\n\
                              產生方式：openssl ec -in private.pem -pubout")?;
                let encoding = EncodingKey::from_ec_pem(private_pem.as_bytes())
                    .context("JWT_EC_PRIVATE_KEY 格式錯誤：需為 EC PEM 私鑰（SEC1 或 PKCS8 格式）")?;
                let decoding = DecodingKey::from_ec_pem(public_pem.as_bytes())
                    .context("JWT_EC_PUBLIC_KEY 格式錯誤：需為 EC PEM 公鑰（SPKI 格式）")?;
                JwtKeys { encoding, decoding }
            },
            // CRIT-02: CSRF 密鑰與 JWT 金鑰隔離，防止單一金鑰洩漏同時破壞兩種保護機制
            csrf_secret: {
                if let Some(s) = read_secret("CSRF_SECRET") {
                    s
                } else {
                    // 向後相容：從 EC 私鑰 PEM 派生（透過 HMAC-SHA256 產生獨立派生金鑰）
                    // 建議在正式環境設定獨立的 CSRF_SECRET 環境變數
                    use sha2::{Digest, Sha256};
                    let private_pem = require_secret("JWT_EC_PRIVATE_KEY")?;
                    let mut hasher = Sha256::new();
                    hasher.update(private_pem.as_bytes());
                    hasher.update(b":csrf-derived-v2");
                    format!("{:x}", hasher.finalize())
                }
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
                .unwrap_or_else(|_| MAX_SESSIONS_PER_USER.to_string())
                .parse()
                .context("MAX_SESSIONS_PER_USER must be a number")?,
            // Email settings
            smtp_host: std::env::var("SMTP_HOST").ok(),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: std::env::var("SMTP_USERNAME").ok(),
            smtp_password: read_secret("SMTP_PASSWORD"),
            smtp_from_email: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@erp.local".to_string()),
            smtp_from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "ERP System".to_string()),
            app_url: std::env::var("APP_URL")
                .unwrap_or_else(|_| "http://localhost".to_string()),
            cookie_secure: parse_bool_env("COOKIE_SECURE"),
            cookie_domain: std::env::var("COOKIE_DOMAIN").ok().filter(|s| !s.is_empty()),
            seed_dev_users: parse_bool_env("SEED_DEV_USERS"),
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
            // R7-P4-4: 預設 false（安全優先），有反向代理時才設 TRUST_PROXY_HEADERS=true
            trust_proxy_headers: parse_bool_env("TRUST_PROXY_HEADERS"),
            // SEC-31: CORS 允許的 Origin 清單
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            audit_hmac_key: read_secret("AUDIT_HMAC_KEY").filter(|s| s.len() >= 16),
            disable_csrf_for_tests: parse_bool_env("DISABLE_CSRF_FOR_TESTS"),
            disable_account_lockout: parse_bool_env("DISABLE_ACCOUNT_LOCKOUT"),
            account_lockout_max_attempts: std::env::var("ACCOUNT_LOCKOUT_MAX_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(ACCOUNT_LOCKOUT_MAX_ATTEMPTS as i64),
            account_lockout_duration_minutes: std::env::var("ACCOUNT_LOCKOUT_DURATION_MINUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(ACCOUNT_LOCKOUT_DURATION_MINUTES),
            upload_dir: std::env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            geoip_db_path: std::env::var("GEOIP_DB_PATH")
                .unwrap_or_else(|_| "/app/geoip/GeoLite2-City.mmdb".to_string()),
            skip_migration_check: parse_bool_env("SKIP_MIGRATION_CHECK"),
            admin_initial_password: read_secret("ADMIN_INITIAL_PASSWORD"),
            test_user_password: std::env::var("TEST_USER_PASSWORD").ok(),
            dev_user_password: std::env::var("DEV_USER_PASSWORD").ok(),
            is_ci: std::env::var("CI").is_ok(),
            gotenberg_url: std::env::var("GOTENBERG_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            image_processor_url: std::env::var("IMAGE_PROCESSOR_URL")
                .unwrap_or_else(|_| "http://image-processor:3100".to_string()),
            metrics_token: std::env::var("METRICS_TOKEN").ok().filter(|s| !s.is_empty()),
            anthropic_api_key: read_secret("ANTHROPIC_API_KEY"),
            ai_review_model: std::env::var("AI_REVIEW_MODEL")
                .unwrap_or_else(|_| "claude-haiku-4-5".to_string()),
            ai_review_enabled: std::env::var("AI_REVIEW_ENABLED")
                .map(|v| v.to_lowercase() != "false" && v != "0")
                .unwrap_or(true),
            ai_review_timeout_secs: std::env::var("AI_REVIEW_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        });

        // R16-11: Production 模式下禁止關閉 CSRF
        if let Ok(ref mut config) = config {
            if config.cookie_secure && config.disable_csrf_for_tests {
                tracing::error!(
                    "DISABLE_CSRF_FOR_TESTS=true 在 production 模式（COOKIE_SECURE=true）下被忽略。\
                     CSRF 保護不可在生產環境中關閉。"
                );
                config.disable_csrf_for_tests = false;
            }
        }

        // R17-4: production 模式（cookie_secure=true）下未設 METRICS_TOKEN 時發出警告
        if let Ok(ref config) = config {
            if config.cookie_secure && config.metrics_token.is_none() {
                tracing::warn!(
                    "METRICS_TOKEN 未設定，/metrics 端點無認證保護。\
                     建議設定 METRICS_TOKEN 環境變數以啟用 Bearer 認證。"
                );
            }
        }

        config
    }

    pub fn is_email_enabled(&self) -> bool {
        self.smtp_host.is_some()
    }
}

#[cfg(test)]
impl JwtKeys {
    /// 產生測試用 ES256 金鑰對（每次呼叫產生新的隨機金鑰）
    pub fn for_testing() -> Self {
        use p256::{
            pkcs8::{EncodePrivateKey, LineEnding},
            SecretKey,
        };
        use p256::pkcs8::spki::EncodePublicKey;

        let secret_key = SecretKey::random(&mut rand::thread_rng());
        let private_pem = secret_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("Failed to encode test EC private key");
        let public_pem = secret_key
            .public_key()
            .to_public_key_pem(LineEnding::LF)
            .expect("Failed to encode test EC public key");
        JwtKeys {
            encoding: EncodingKey::from_ec_pem(private_pem.as_bytes())
                .expect("Valid test EC private key"),
            decoding: DecodingKey::from_ec_pem(public_pem.as_bytes())
                .expect("Valid test EC public key"),
        }
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
            database_min_connections: 2,
            database_acquire_timeout_seconds: 30,
            database_retry_attempts: 5,
            database_retry_delay_seconds: 5,
            jwt_keys: JwtKeys::for_testing(),
            csrf_secret: "b".repeat(64),
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
            disable_csrf_for_tests: false,
            disable_account_lockout: false,
            account_lockout_max_attempts: 5,
            account_lockout_duration_minutes: 15,
            upload_dir: "./uploads".to_string(),
            geoip_db_path: "/app/geoip/GeoLite2-City.mmdb".to_string(),
            skip_migration_check: false,
            admin_initial_password: None,
            test_user_password: None,
            dev_user_password: None,
            is_ci: false,
            gotenberg_url: "http://localhost:3000".to_string(),
            image_processor_url: "http://localhost:3100".to_string(),
            metrics_token: None,
            anthropic_api_key: None,
            ai_review_model: "claude-haiku-4-5".to_string(),
            ai_review_enabled: true,
            ai_review_timeout_secs: 30,
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
    fn test_jwt_keys_for_testing_is_valid() {
        let keys = JwtKeys::for_testing();
        // 驗證簽發/驗證互通：簽一個 token 然後驗證
        use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize)]
        struct TestClaims {
            sub: String,
            exp: i64,
        }
        let claims = TestClaims {
            sub: "test".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
        };
        let token = encode(&Header::new(Algorithm::ES256), &claims, &keys.encoding)
            .expect("sign should succeed");
        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = false;
        decode::<TestClaims>(&token, &keys.decoding, &validation)
            .expect("verify should succeed");
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
