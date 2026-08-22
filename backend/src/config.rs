use anyhow::Context;
use jsonwebtoken::{DecodingKey, EncodingKey};

use crate::constants::{
    ACCOUNT_LOCKOUT_DURATION_MINUTES, ACCOUNT_LOCKOUT_MAX_ATTEMPTS, MAX_SESSIONS_PER_USER,
    SESSION_IDLE_TIMEOUT_MINUTES,
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

/// 解析 boolean 環境變數，**預設 `true`**（fail-safe-on）。
/// 接受 trimmed "false" / "0"（大小寫不限）才視為 false；其他任何值（含未設定）皆為 true。
/// 用途：合規 / 安全相關 feature 應預設啟用，明確設 `=false` 才停用。
fn parse_bool_env_default_true(key: &str) -> bool {
    std::env::var(key)
        .map(|v| {
            let v = v.trim();
            !(v.eq_ignore_ascii_case("false") || v == "0")
        })
        .unwrap_or(true)
}

/// Read a secret value: prefer `{key}_FILE` (Docker Secrets path), fallback to `{key}` env var.
/// 讀取 secret：優先 `{KEY}_FILE`（Docker Secrets 檔案路徑），fallback `{KEY}` env。
/// `pub` 供 bin（如 backfill 工具）共用同一載入語意，避免重複實作。
pub fn read_secret(key: &str) -> Option<String> {
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

/// Like `read_secret`, but **fail loud** instead of falling back to `{key}` when
/// `{key}_FILE` is set but unreadable. `read_secret` 的靜默 fallback（warn + 讀裸 env）
/// 對伺服器啟動設定合理，但對一次性 destructive 工具（backfill/import/patch/provision）
/// 危險：`_FILE` 打錯路徑時，若裸 `DATABASE_URL` 剛好也存在（如本機 `.env` 殘留），
/// 工具會悄悄接到錯的 DB 執行寫入而不中止（Qodo review on PR #83 指出）。
/// `pub` 供上述 bin 工具共用。
pub fn read_secret_strict(key: &str) -> anyhow::Result<String> {
    let file_key = format!("{key}_FILE");
    if let Ok(path) = std::env::var(&file_key) {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {file_key}={path}"))?
            .trim()
            .to_string();
        // Qodo review on PR #83：_FILE 存在但內容為空時，若不擋，空字串會一路
        // 傳進 sqlx::connect() 等呼叫端，變成間接、不易理解的錯誤，而非清楚的
        // 「必須設定」訊息；env 分支已有 filter(!is_empty)，這裡對齊同一語意。
        anyhow::ensure!(!content.is_empty(), "{file_key}={path} 內容為空");
        return Ok(content);
    }
    std::env::var(key)
        .ok()
        .filter(|s| !s.is_empty())
        .with_context(|| format!("{key}（或 {key}_FILE）must be set"))
}

/// 讀取 `AUDIT_HMAC_KEY`（或 `_FILE`），長度 < 44 字元（SEC-34：HMAC-SHA256 最小密鑰長度）視為未設定。
/// `pub` 供伺服器啟動設定（`Config::from_env`）使用；一次性寫入型 bin 工具請改用
/// `read_audit_hmac_key_strict`（見下）避免 `_FILE` 讀取失敗時靜默落回裸 env 的錯誤 key。
pub fn read_audit_hmac_key() -> Option<String> {
    read_secret("AUDIT_HMAC_KEY").filter(|s| s.len() >= 44)
}

/// 嚴格版 `AUDIT_HMAC_KEY` 讀取，供會寫入 audit 的一次性 bin 工具（backfill/import/patch/
/// provision）使用：`AUDIT_HMAC_KEY_FILE` 已設定但讀取失敗時直接回傳 Err，**不**落回裸
/// `AUDIT_HMAC_KEY` env（CodeRabbit review on PR #83 指出：靜默 fallback 可能讓寫入型工具
/// 用到殘留在裸 env 的不同 key，造成本次寫入的 audit HMAC 與既有 chain 來源不一致）。
/// 兩者皆未設定 → `Ok(None)`（呼叫端允許本次不進 HMAC chain，由各 bin 自行印出警告）。
/// 長度 < 44 字元（SEC-34）一律視為未設定，與 `read_audit_hmac_key` 語意一致。
pub fn read_audit_hmac_key_strict() -> anyhow::Result<Option<String>> {
    let file_key = "AUDIT_HMAC_KEY_FILE";
    let raw = if let Ok(path) = std::env::var(file_key) {
        Some(
            std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {file_key}={path}"))?
                .trim()
                .to_string(),
        )
    } else {
        std::env::var("AUDIT_HMAC_KEY")
            .ok()
            .filter(|s| !s.is_empty())
    };
    Ok(raw.filter(|s| s.len() >= 44))
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
    /// R25-5: PostgreSQL 個別查詢執行上限（毫秒）；0 = 停用
    pub database_statement_timeout_ms: u64,
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
    /// R41-1: 閒置 session 強制 revoke 的閾值（分鐘）。每次 refresh 會檢查
    /// `now - refresh_token.last_used_at`，超過此值則拒絕 refresh，使用者需重新登入。
    /// 對齊 NICS 附表十普級「存取控制 / 帳號管理」閒置鎖定要求。
    /// 預設 30 分鐘；若需臨時停用此檢查可設為極大值（如 1440 = 24h）。
    pub auth_idle_timeout_minutes: i64,
    // Email settings
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    /// R22-11: LINE Notify token for security alerts
    pub line_notify_token: Option<String>,
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
    /// R66-B2/C6: app 層 AEAD at-rest 加密金鑰（`ENCRYPTION_KEY`，32 bytes base64）。
    /// 與 JWT / HMAC 金鑰隔離（blast-radius）。未設定 → 加密功能（如 2FA 啟用）回 Internal 拒絕。
    pub encryption_key: Option<crate::utils::crypto::EncryptionKey>,
    /// R26-2: HMAC chain 每日驗證 cron 是否實際執行驗證。
    ///
    /// **預設 `true`（R30-28 後）**：每日 02:00 UTC 跑 `verify_chain_range`，
    /// 對應 21 CFR §11.10(e) audit log 完整性合規。env `AUDIT_CHAIN_VERIFY_ACTIVE=false`
    /// 才停用（如大量 legacy 待 backfill 的暫時 opt-out）。
    ///
    /// **歷史背景（R30-28 前預設 `false`）**：deprecated `log_activity(&pool, ...)`
    /// 與 `log_activity_tx` 的 HMAC 編碼不同；R26-6 HMAC 版本化 + verifier try-both
    /// fallback 完成後（PR #158 解掉 false positive 風險），預設可安全翻為 `true`。
    pub audit_chain_verify_active: bool,
    /// R30-27：role / permission 變更是否強制電子簽章（密碼 + 手寫雙因子）。
    ///
    /// 預設 `false`（R30-27a backend ship 階段）：admin 仍可走舊流程改 role/permission，
    /// 但 service 已準備好接受 `mutation_signature` payload。
    /// **R30-27b 前端 dialog ship 後**，production 環境設 `ROLE_SIGNATURE_REQUIRED=true`
    /// 才正式 enforce 21 CFR §11.10(d) 存取控制簽章不可否認性。
    /// 啟用後 admin 對 role/permission 的 create/update/delete 必須帶密碼 + handwriting_svg。
    pub role_signature_required: bool,
    /// R30-17：retention enforcer 是否真實執行 hard-delete / partition drop。
    ///
    /// 預設 `false`，每日 03:00 UTC cron 會跳過真正執行（log 即可）；
    /// 在 staging 連續觀察 dry-run 報表 ≥7 天確認沒誤刪後，再設為 `true`
    /// 啟用實刪。環境變數：`RETENTION_ENFORCER_ENABLED=true`。
    pub retention_enforcer_enabled: bool,
    /// H7：JWT 私鑰檔路徑（從 `JWT_EC_PRIVATE_KEY_FILE` env 讀取，None = 用 PEM env）。
    /// 啟動時 `check_jwt_key_file_permissions` 用此檢查 unix mode；走 Config 統一
    /// 而非散落 std::env::var（CLAUDE.md：禁止散落讀取 env）。
    pub jwt_ec_private_key_file: Option<String>,
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
    /// PDF Service (FastAPI) URL
    pub pdf_service_url: String,
    /// PDF Service 內部服務認證 token（X-Internal-Token header）
    pub pdf_service_token: String,
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
    /// R24-3: Alertmanager webhook 共享 token（未設定 = 不啟用驗證，允許所有）
    pub alertmanager_webhook_token: Option<String>,
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
            database_statement_timeout_ms: std::env::var("DATABASE_STATEMENT_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .context("DATABASE_STATEMENT_TIMEOUT_MS must be a number")?,
            // CRIT-02: 使用非對稱式 ES256（ECDSA P-256）取代對稱式 HS256
            // 金鑰材料從環境變數（或 _FILE Docker Secrets）讀取，啟動時預解析
            jwt_keys: {
                let private_pem = require_secret("JWT_EC_PRIVATE_KEY").context(
                    "JWT_EC_PRIVATE_KEY（或 JWT_EC_PRIVATE_KEY_FILE）必須設定\n\
                              產生方式：openssl ecparam -name prime256v1 -genkey -noout | \\\n\
                                         openssl pkcs8 -topk8 -nocrypt",
                )?;
                let public_pem = require_secret("JWT_EC_PUBLIC_KEY").context(
                    "JWT_EC_PUBLIC_KEY（或 JWT_EC_PUBLIC_KEY_FILE）必須設定\n\
                              產生方式：openssl ec -in private.pem -pubout",
                )?;
                // ⚠️ 只接受 PKCS8（`-----BEGIN PRIVATE KEY-----`），**不接受** SEC1
                // （`-----BEGIN EC PRIVATE KEY-----`）。原因是 jsonwebtoken 選用
                // `rust_crypto` feature（見 Cargo.toml），其底層 p256 只有
                // `from_pkcs8_pem()`；SEC1 會回 InvalidKeyFormat。
                // （改用預設的 aws_lc_rs/ring 後端才會兩種都吃——但那是另一個決定。）
                // 2026-08-20：此處原本寫「SEC1 或 PKCS8」，照著產 SEC1 會讓 api 無限重啟，
                // 而錯誤訊息還說 SEC1 可以。已訂正為實際行為。
                let encoding = EncodingKey::from_ec_pem(private_pem.as_bytes()).context(
                    "JWT_EC_PRIVATE_KEY 格式錯誤：必須是 PKCS8 PEM（開頭為 -----BEGIN PRIVATE KEY-----）。\n\
                     SEC1 格式（-----BEGIN EC PRIVATE KEY-----）不受支援，請用上方的 openssl pkcs8 -topk8 轉換。",
                )?;
                let decoding = DecodingKey::from_ec_pem(public_pem.as_bytes())
                    .context("JWT_EC_PUBLIC_KEY 格式錯誤：需為 EC PEM 公鑰（SPKI 格式）")?;
                JwtKeys { encoding, decoding }
            },
            // CRIT-02: CSRF 密鑰與 JWT 金鑰隔離，防止單一金鑰洩漏同時破壞兩種保護機制
            csrf_secret: {
                // R82-6 (gemini security-high): 空 / 過短（含 CSRF_SECRET_FILE 指向空檔時
                // read_secret 的 file 分支會回 Some("")）一律視同未設 → 落入派生 fallback，
                // 並由 config_check 記 warn（prod fail-fast）。強度門檻 ≥44 對齊 AUDIT_HMAC_KEY。
                if let Some(s) = read_secret("CSRF_SECRET").filter(|s| s.len() >= 44) {
                    s
                } else {
                    // dev/CI/test 向後相容：未設 CSRF_SECRET 時從 EC 私鑰 PEM 派生。
                    // R82-6：prod 必須設獨立 CSRF_SECRET（或 _FILE）——缺席時 config_check
                    // 記軟性警告，main.rs 於 is_production() 時 fail-fast；避免 JWT 私鑰外洩
                    // 連帶推導出 CSRF secret（破壞金鑰隔離）。
                    use sha2::{Digest, Sha256};
                    let private_pem = require_secret("JWT_EC_PRIVATE_KEY")?;
                    let mut hasher = Sha256::new();
                    hasher.update(private_pem.as_bytes());
                    hasher.update(b":csrf-derived-v2");
                    hex::encode(hasher.finalize())
                }
            },
            // SEC-32: 統一使用 JWT_EXPIRATION_MINUTES，預設 15 分鐘
            // 對齊 NIST AAL2 / NICS 普級建議；必須 < auth_idle_timeout_minutes
            // 否則 warn_if_idle_window_unusable 會 fire。
            jwt_expiration_seconds: {
                let mins: i64 = std::env::var("JWT_EXPIRATION_MINUTES")
                    .unwrap_or_else(|_| "15".to_string())
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
            // R41-1: 預設由 `constants::SESSION_IDLE_TIMEOUT_MINUTES` 提供（現為 600 分鐘 / 10h；
            // 2026-05-25 per 使用者需求自 480/8h 上調，詳見 constants.rs 該常數註解）。
            // 歷史值 30 為 R41-1 落地時的 NICS 普級保守值，與 PR #455 sliding session
            // overhaul 設計的 8h `session_timeout_minutes` 衝突，會導致使用者離開電腦
            // 30+ 分鐘後第一次 refresh 即觸發 refresh_token revoke → 被踢出
            // （見 PROGRESS.md 2026-05-21）。預設改 480 對齊 sliding 設計；嚴格 idle
            // 需求環境可在 .env 顯式調低（fail-fast 仍排除 0 / 負值）。
            auth_idle_timeout_minutes: {
                let v: i64 = std::env::var("AUTH_IDLE_TIMEOUT_MINUTES")
                    .unwrap_or_else(|_| SESSION_IDLE_TIMEOUT_MINUTES.to_string())
                    .parse()
                    .context("AUTH_IDLE_TIMEOUT_MINUTES must be a number")?;
                if v < 1 {
                    anyhow::bail!(
                        "AUTH_IDLE_TIMEOUT_MINUTES must be >= 1 (got {v}). \
                         0/負值會讓 idle 檢查永遠拒絕 refresh。"
                    );
                }
                v
            },
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
            line_notify_token: read_secret("LINE_NOTIFY_TOKEN"),
            app_url: std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost".to_string()),
            cookie_secure: parse_bool_env("COOKIE_SECURE"),
            cookie_domain: std::env::var("COOKIE_DOMAIN")
                .ok()
                .filter(|s| !s.is_empty()),
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
            // SEC-34: HMAC-SHA256 密鑰最小長度 44 chars（= base64 編碼的 32 bytes = 256 bits）
            // 對應 NIST SP 800-107 建議：HMAC 密鑰至少與 hash 輸出長度相同（SHA256=32 bytes）。
            // `.env.example` 有對應文檔與 openssl rand -base64 32 產生指引。
            audit_hmac_key: read_audit_hmac_key(),
            // R66-B2/C6: AEAD at-rest 加密金鑰（32 bytes base64）。
            // Fail-fast：設定了但解析失敗 → 啟動即報錯（不靜默失效、避免誤以為資料已加密）。
            encryption_key: read_secret("ENCRYPTION_KEY")
                .map(|s| crate::utils::crypto::EncryptionKey::from_base64(&s))
                .transpose()
                .map_err(|e| anyhow::anyhow!("ENCRYPTION_KEY 無效：{e:?}"))?,
            // R30-28：預設 true（fail-safe-on）。合規路徑 GLP §11.10(e) / 21 CFR
            // 要求 audit log 完整性可追溯；新環境部署應自動啟用每日驗證。
            // 顯式設 `AUDIT_CHAIN_VERIFY_ACTIVE=false` 才停用（如 backfill 跑完前的暫時 opt-out）。
            audit_chain_verify_active: parse_bool_env_default_true("AUDIT_CHAIN_VERIFY_ACTIVE"),
            // R30-27：role/permission 簽章強制 — backend ship 階段預設 false，
            // 前端 dialog 完成後 production 設 ROLE_SIGNATURE_REQUIRED=true 才正式 enforce。
            role_signature_required: parse_bool_env("ROLE_SIGNATURE_REQUIRED"),
            // R30-17：retention enforcer 預設關閉，需顯式 RETENTION_ENFORCER_ENABLED=true 才啟用實刪。
            retention_enforcer_enabled: parse_bool_env("RETENTION_ENFORCER_ENABLED"),
            jwt_ec_private_key_file: std::env::var("JWT_EC_PRIVATE_KEY_FILE")
                .ok()
                .filter(|s| !s.is_empty()),
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
            upload_dir: std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()),
            geoip_db_path: std::env::var("GEOIP_DB_PATH")
                .unwrap_or_else(|_| "/app/geoip/GeoLite2-City.mmdb".to_string()),
            skip_migration_check: parse_bool_env("SKIP_MIGRATION_CHECK"),
            admin_initial_password: read_secret("ADMIN_INITIAL_PASSWORD"),
            test_user_password: read_secret("TEST_USER_PASSWORD"),
            dev_user_password: read_secret("DEV_USER_PASSWORD"),
            is_ci: std::env::var("CI").is_ok(),
            pdf_service_url: std::env::var("PDF_SERVICE_URL")
                .unwrap_or_else(|_| "http://print-pdf:9200".to_string()),
            pdf_service_token: read_secret("PDF_SERVICE_TOKEN").unwrap_or_default(),
            // 走 read_secret 以支援 METRICS_TOKEN_FILE：Prometheus 那端已用
            // credentials_file 讀 secret 檔，這裡對齊同一份來源，token 不必進 .env
            // （.env 以 env_file 灌進所有容器，範圍過大）。
            //
            // ⚠️ filter 不可省：read_secret 對「檔案存在但內容為空」會回 `Some("")`，而
            // metrics_handler 在 `Some(expected)` 分支下把「沒帶 Authorization」當成 provided=""，
            // 與空 expected 比對會相等 → 認證形同全開；又因為不是 None，下方「未設定」警告也不會叫，
            // 變成看起來有保護、實際沒有，比明著不設更危險。空值一律視為未設定。
            metrics_token: read_secret("METRICS_TOKEN").filter(|s| !s.is_empty()),
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
            // R37-7: 改走 read_secret() 支援 ALERTMANAGER_WEBHOOK_TOKEN_FILE（避免 .env 明文）
            alertmanager_webhook_token: read_secret("ALERTMANAGER_WEBHOOK_TOKEN"),
        });

        // R16-11: Production 模式下禁止關閉 CSRF
        if let Ok(ref mut config) = config {
            if config.cookie_secure && config.disable_csrf_for_tests {
                tracing::error!(
                    "DISABLE_CSRF_FOR_TESTS=true 在 production 模式（COOKIE_SECURE=true）下被忽略。\
                     CSRF 保護不可在生產環境中關閉。"
                );
                config.disable_csrf_for_tests = false;
            } else if !config.cookie_secure && config.disable_csrf_for_tests && !config.is_ci {
                tracing::warn!(
                    "DISABLE_CSRF_FOR_TESTS=true 且 COOKIE_SECURE=false — CSRF 保護已關閉。\
                     若非測試環境請立即移除 DISABLE_CSRF_FOR_TESTS。"
                );
            }
        }

        // R17-4: production 模式（cookie_secure=true）下未設 METRICS_TOKEN 時發出警告。
        // 「secret 檔存在但內容為空」也會走到這裡（上面已 filter 成 None），故警告文字要同時
        // 點出檔案來源，否則運維只會去找 env var 而漏掉空檔這個實際成因。
        if let Ok(ref config) = config {
            if config.cookie_secure && config.metrics_token.is_none() {
                tracing::warn!(
                    "METRICS_TOKEN 未設定（或 METRICS_TOKEN_FILE 指向的檔案為空），\
                     /metrics 端點無認證保護。請設定 METRICS_TOKEN，\
                     或確認 METRICS_TOKEN_FILE 指向的 secret 檔含有實際 token。"
                );
            }
        }

        config
    }

    pub fn is_email_enabled(&self) -> bool {
        self.smtp_host.is_some()
    }

    /// R41-1: 啟動時 sanity check — 若 access token TTL ≥ idle window，背景 polling
    /// 不會在 idle 期間觸發 refresh（access token 還沒過期）。下次 refresh 才會檢測
    /// 到 idle（最遲於 access token 過期當下）— 仍能擋「browser 留 token 之後沒人用」
    /// 的情境，但實際 idle 偵測精度被 access token TTL 上限拉低。
    ///
    /// 建議：access token TTL < idle window，例如 idle=480min 時 access=60min。
    /// 本函式只發出 warn 不阻止啟動（已部署環境彈性優先）。
    pub fn warn_if_idle_window_unusable(&self) {
        let access_token_minutes = self.jwt_expiration_seconds / 60;
        if access_token_minutes >= self.auth_idle_timeout_minutes {
            tracing::warn!(
                jwt_expiration_minutes = access_token_minutes,
                auth_idle_timeout_minutes = self.auth_idle_timeout_minutes,
                "[R41-1] access token TTL ≥ idle window: 閒置偵測精度受限。\
                 建議 JWT_EXPIRATION_MINUTES < AUTH_IDLE_TIMEOUT_MINUTES（例如 idle=480 時 access=60）"
            );
        }
    }
}

#[cfg(test)]
impl JwtKeys {
    /// 測試用 EC P-256 私鑰（PKCS8 PEM，來自 jsonwebtoken crate 官方測試金鑰）
    pub const TEST_PRIVATE_KEY_PEM: &'static str = "-----BEGIN PRIVATE KEY-----\n\
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgWTFfCGljY6aw3Hrt\n\
kHmPRiazukxPLb6ilpRAewjW8nihRANCAATDskChT+Altkm9X7MI69T3IUmrQU0L\n\
950IxEzvw/x5BMEINRMrXLBJhqzO9Bm+d6JbqA21YQmd1Kt4RzLJR1W+\n\
-----END PRIVATE KEY-----\n";

    /// 測試用 EC P-256 公鑰（SPKI PEM，來自 jsonwebtoken crate 官方測試金鑰）
    pub const TEST_PUBLIC_KEY_PEM: &'static str = "-----BEGIN PUBLIC KEY-----\n\
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEw7JAoU/gJbZJvV+zCOvU9yFJq0FN\n\
C/edCMRM78P8eQTBCDUTK1ywSYaszvQZvneiW6gNtWEJndSreEcyyUdVvg==\n\
-----END PUBLIC KEY-----\n";

    /// 建立測試用 JwtKeys（固定金鑰，不隨機，確保跨測試一致性）
    pub fn for_testing() -> Self {
        JwtKeys {
            encoding: EncodingKey::from_ec_pem(Self::TEST_PRIVATE_KEY_PEM.as_bytes())
                .expect("Valid test EC private key"),
            decoding: DecodingKey::from_ec_pem(Self::TEST_PUBLIC_KEY_PEM.as_bytes())
                .expect("Valid test EC public key"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `read_secret` 對「檔案存在但內容為空」會回 `Some("")`，呼叫端若不 filter，
    /// metrics 認證會因為 expected 與 provided 同為空字串而全開（見 metrics_token 欄位註解）。
    /// 這裡鎖住「空值一律視為未設定」的語義。
    ///
    /// 用 env var 操作 process 全域狀態，故加 `#[serial]` 避免與其他測試互相污染。
    #[test]
    #[serial_test::serial]
    fn empty_metrics_secret_file_is_treated_as_unset() {
        let dir = std::env::temp_dir().join("ipig_metrics_token_test");
        std::fs::create_dir_all(&dir).expect("建立測試目錄");
        let empty = dir.join("empty_token.txt");
        let filled = dir.join("filled_token.txt");
        // 只有空白字元也算空（read_secret 會 trim）
        std::fs::write(&empty, "   \n").expect("寫入空 token 檔");
        std::fs::write(&filled, "real-token-value\n").expect("寫入有值 token 檔");

        let read_metrics_token = || read_secret("METRICS_TOKEN").filter(|s: &String| !s.is_empty());

        // set_var / remove_var 的不安全性來自「與其他執行緒併發讀寫環境變數」。本測試標了
        // #[serial_test::serial]，同一時間不會有其他測試執行緒動環境變數；讀取端也只有本
        // 函式內同步呼叫的 read_secret。
        // SAFETY: 由 #[serial] 保證無併發存取，讀取端為同步呼叫，故無資料競爭。
        unsafe {
            std::env::remove_var("METRICS_TOKEN");
            std::env::set_var("METRICS_TOKEN_FILE", &empty);
        }
        assert_eq!(
            read_metrics_token(),
            None,
            "空的 secret 檔必須視為未設定，否則 /metrics 認證會對空 token 放行"
        );

        // SAFETY: 同上——#[serial] 保證無併發存取，讀取端為同步呼叫。
        unsafe {
            std::env::set_var("METRICS_TOKEN_FILE", &filled);
        }
        assert_eq!(
            read_metrics_token(),
            Some("real-token-value".to_string()),
            "有值的 secret 檔應正常讀出並 trim"
        );

        // 空 env var（無檔案來源）同樣視為未設定
        // SAFETY: 同上——#[serial] 保證無併發存取，讀取端為同步呼叫。
        unsafe {
            std::env::remove_var("METRICS_TOKEN_FILE");
            std::env::set_var("METRICS_TOKEN", "");
        }
        assert_eq!(read_metrics_token(), None, "空的 env var 必須視為未設定");

        // SAFETY: 同上——收尾清除，#[serial] 保證無併發存取。
        unsafe {
            std::env::remove_var("METRICS_TOKEN");
        }
        let _ = std::fs::remove_file(&empty);
        let _ = std::fs::remove_file(&filled);
    }

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
            database_statement_timeout_ms: 30000,
            jwt_keys: JwtKeys::for_testing(),
            csrf_secret: "b".repeat(64),
            jwt_expiration_seconds: 900,
            jwt_refresh_expiration_days: 7,
            max_sessions_per_user: 5,
            auth_idle_timeout_minutes: 30,
            smtp_host: None,
            smtp_port: 587,
            smtp_username: None,
            smtp_password: None,
            smtp_from_email: "noreply@test.local".to_string(),
            smtp_from_name: "Test".to_string(),
            line_notify_token: None,
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
            encryption_key: None,
            audit_chain_verify_active: false,
            role_signature_required: false,
            retention_enforcer_enabled: false,
            jwt_ec_private_key_file: None,
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
            pdf_service_url: "http://localhost:9210".to_string(),
            pdf_service_token: "test-token".to_string(),
            metrics_token: None,
            anthropic_api_key: None,
            ai_review_model: "claude-haiku-4-5".to_string(),
            ai_review_enabled: true,
            ai_review_timeout_secs: 30,
            alertmanager_webhook_token: None,
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
        decode::<TestClaims>(&token, &keys.decoding, &validation).expect("verify should succeed");
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
