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

/// Rate Limiting (requests per window) — P2-R4-14 集中管理
pub const AUTH_RATE_LIMIT_PER_MINUTE: u32 = 100; // 100/min 以支援 E2E 測試
pub const API_RATE_LIMIT_PER_MINUTE: u32 = 600;
pub const WRITE_RATE_LIMIT_PER_MINUTE: u32 = 120;
pub const UPLOAD_RATE_LIMIT_PER_MINUTE: u32 = 30;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;
pub const RATE_LIMIT_CLEANUP_INTERVAL_SECS: u64 = 300;

/// File Upload — 各類別最大檔案大小 (bytes)
pub const MAX_UPLOAD_SIZE_BYTES: usize = 30 * 1024 * 1024; // 30MB 全域
pub const FILE_MAX_PROTOCOL_ATTACHMENT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_ANIMAL_PHOTO: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_DATA_IMPORT: usize = 100 * 1024 * 1024; // 100 MB（全庫 IDXF 匯入）
pub const FILE_MAX_PATHOLOGY_REPORT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_VET_RECOMMENDATION: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_LEAVE_ATTACHMENT: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_OBSERVATION_ATTACHMENT: usize = 20 * 1024 * 1024; // 20 MB

/// Auth 短期 token 過期秒數
pub const TWO_FA_TEMP_EXPIRES_SECS: i64 = 300; // 5 分鐘
pub const REAUTH_EXPIRES_SECS: i64 = 300; // 5 分鐘

/// 預設時區（HR 打卡、報表等）
pub const DEFAULT_TIMEZONE: &str = "Asia/Taipei";
/// UTC+8 偏移秒數（台灣時區）
pub const TAIWAN_OFFSET_SECS: i32 = 8 * 3600;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_constants() {
        assert!(DEFAULT_PAGE_SIZE <= MAX_PAGE_SIZE, "預設頁大小應不超過最大值");
        assert!(MAX_PAGE_SIZE >= 1);
    }

    #[test]
    fn test_password_policy() {
        assert!(PASSWORD_MIN_LENGTH >= 8, "密碼最小長度應 ≥ 8");
    }

    #[test]
    fn test_rate_limit_constants() {
        assert!(WRITE_RATE_LIMIT_PER_MINUTE <= API_RATE_LIMIT_PER_MINUTE);
        assert!(UPLOAD_RATE_LIMIT_PER_MINUTE <= WRITE_RATE_LIMIT_PER_MINUTE);
    }

    #[test]
    fn test_file_size_constants() {
        assert!(FILE_MAX_ANIMAL_PHOTO <= FILE_MAX_PROTOCOL_ATTACHMENT);
        assert!(FILE_MAX_PROTOCOL_ATTACHMENT <= MAX_UPLOAD_SIZE_BYTES);
    }

    #[test]
    fn test_audit_constants() {
        assert!(AUDIT_LOG_MAX_EXPORT > 0, "匯出上限應為正數");
        assert!(ACTIVITY_LOG_MAX_PER_PAGE > 0, "每頁筆數上限應為正數");
    }

    #[test]
    fn test_audit_export_reasonable_limit() {
        // 匯出上限應在合理範圍內（避免過大記憶體使用）
        assert!(AUDIT_LOG_MAX_EXPORT <= 100_000);
    }
}
