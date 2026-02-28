/// Pagination
pub const DEFAULT_PAGE_SIZE: i64 = 50;
pub const MAX_PAGE_SIZE: i64 = 200;
pub const REPORT_MAX_ROWS: i64 = 1000;
pub const ANALYSIS_MAX_ROWS: i64 = 5000;

/// Authentication
pub const ACCESS_TOKEN_EXPIRY_HOURS: i64 = 24;
pub const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 30;
pub const ACCOUNT_LOCKOUT_MAX_ATTEMPTS: i32 = 5;
pub const ACCOUNT_LOCKOUT_DURATION_MINUTES: i64 = 30;

/// Rate Limiting (requests per window)
pub const AUTH_RATE_LIMIT_PER_MINUTE: u32 = 10;
pub const API_RATE_LIMIT_PER_MINUTE: u32 = 300;
pub const WRITE_RATE_LIMIT_PER_MINUTE: u32 = 120;
pub const UPLOAD_RATE_LIMIT_PER_MINUTE: u32 = 30;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;
pub const RATE_LIMIT_CLEANUP_INTERVAL_SECS: u64 = 300;

/// File Upload
pub const MAX_UPLOAD_SIZE_BYTES: usize = 30 * 1024 * 1024; // 30MB

/// Scheduler cron expressions
pub const CRON_DAILY_3AM: &str = "0 0 3 * * *";
pub const CRON_DAILY_330AM: &str = "0 30 3 * * *";
pub const CRON_EVERY_5MIN: &str = "0 */5 * * * *";
pub const CRON_EVERY_30MIN: &str = "0 */30 * * * *";

/// Session
pub const SESSION_IDLE_TIMEOUT_MINUTES: i64 = 30;
pub const MAX_SESSIONS_PER_USER: i64 = 5;

/// Password policy
pub const PASSWORD_MIN_LENGTH: usize = 8;

/// Audit
pub const AUDIT_LOG_MAX_EXPORT: i64 = 10000;
pub const ACTIVITY_LOG_MAX_PER_PAGE: i64 = 500;
