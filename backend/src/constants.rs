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
/// R7-P4-2: 認證端點從 100 降至 30/min（防暴力破解，帳號鎖定仍為第一道防線）
/// 若 E2E 測試需要更高限制，可透過環境變數覆蓋
pub const AUTH_RATE_LIMIT_PER_MINUTE: u32 = 30;
pub const API_RATE_LIMIT_PER_MINUTE: u32 = 600;
pub const WRITE_RATE_LIMIT_PER_MINUTE: u32 = 120;
pub const UPLOAD_RATE_LIMIT_PER_MINUTE: u32 = 30;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;
pub const RATE_LIMIT_CLEANUP_INTERVAL_SECS: u64 = 300;
/// E-5: 忘記密碼端點嚴格速率限制（防 email flooding）
pub const FORGOT_PASSWORD_RATE_LIMIT: u32 = 5;
pub const FORGOT_PASSWORD_RATE_WINDOW_SECS: u64 = 600; // 10 分鐘

/// CRIT-03: 使用者權限快取 TTL（秒）。角色異動後最多延遲此時間生效。
pub const PERMISSION_CACHE_TTL_SECS: u64 = 300; // 5 分鐘

/// File Upload — 各類別最大檔案大小 (bytes)
pub const MAX_UPLOAD_SIZE_BYTES: usize = 30 * 1024 * 1024; // 30MB 全域
pub const FILE_MAX_PROTOCOL_ATTACHMENT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_ANIMAL_PHOTO: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_DATA_IMPORT: usize = 100 * 1024 * 1024; // 100 MB（全庫 IDXF 匯入）
pub const FILE_MAX_PATHOLOGY_REPORT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_VET_RECOMMENDATION: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_LEAVE_ATTACHMENT: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_OBSERVATION_ATTACHMENT: usize = 20 * 1024 * 1024; // 20 MB
pub const FILE_MAX_SOP_DOCUMENT: usize = 30 * 1024 * 1024; // 30 MB

/// Auth 短期 token 過期秒數
pub const TWO_FA_TEMP_EXPIRES_SECS: i64 = 300; // 5 分鐘
pub const REAUTH_EXPIRES_SECS: i64 = 300; // 5 分鐘

/// 預設時區（HR 打卡、報表等）
pub const DEFAULT_TIMEZONE: &str = "Asia/Taipei";
/// UTC+8 偏移秒數（台灣時區）
pub const TAIWAN_OFFSET_SECS: i32 = 8 * 3600;

/// Calendar conflict resolution strategies
pub const CONFLICT_KEEP_IPIG: &str = "keep_ipig";
pub const CONFLICT_ACCEPT_GOOGLE: &str = "accept_google";
pub const CONFLICT_DISMISS: &str = "dismiss";
pub const CONFLICT_STATUS_RESOLVED_KEEP: &str = "resolved_keep_ipig";
pub const CONFLICT_STATUS_RESOLVED_ACCEPT: &str = "resolved_accept_google";
pub const CONFLICT_STATUS_DISMISSED: &str = "dismissed";
pub const CONFLICT_STATUS_RESOLVED: &str = "resolved";

/// Scheduler cron expressions
pub const CRON_DAILY_3AM: &str = "0 0 3 * * *";
pub const CRON_DAILY_330AM: &str = "0 30 3 * * *";
pub const CRON_EVERY_5MIN: &str = "0 */5 * * * *";
pub const CRON_EVERY_30MIN: &str = "0 */30 * * * *";

/// Session
pub const SESSION_IDLE_TIMEOUT_MINUTES: i64 = 30;
pub const MAX_SESSIONS_PER_USER: i64 = 5;

/// Password policy
pub const PASSWORD_MIN_LENGTH: usize = 10;
pub const DEFAULT_INSECURE_PASSWORD: &str = "iPig$ecure1";

/// Audit
pub const AUDIT_LOG_MAX_EXPORT: i64 = 10000;
pub const ACTIVITY_LOG_MAX_PER_PAGE: i64 = 500;

/// R22: Security event types (attack detection & alerting)
pub const SEC_EVENT_RATE_LIMIT_AUTH: &str = "RATE_LIMIT_AUTH";
pub const SEC_EVENT_RATE_LIMIT_API: &str = "RATE_LIMIT_API";
pub const SEC_EVENT_RATE_LIMIT_WRITE: &str = "RATE_LIMIT_WRITE";
pub const SEC_EVENT_RATE_LIMIT_UPLOAD: &str = "RATE_LIMIT_UPLOAD";
pub const SEC_EVENT_RATE_LIMIT_FORGOT_PW: &str = "RATE_LIMIT_FORGOT_PASSWORD";
pub const SEC_EVENT_RATE_LIMIT_AI_KEY: &str = "RATE_LIMIT_AI_KEY";
pub const SEC_EVENT_AI_KEY_DEACTIVATED: &str = "AI_KEY_DEACTIVATED";
pub const SEC_EVENT_AI_KEY_EXPIRED: &str = "AI_KEY_EXPIRED";
pub const SEC_EVENT_PERMISSION_DENIED: &str = "PERMISSION_DENIED";
pub const SEC_EVENT_ACCOUNT_LOCKOUT: &str = "ACCOUNT_LOCKOUT";
pub const SEC_EVENT_HONEYPOT_HIT: &str = "HONEYPOT_HIT";
pub const SEC_EVENT_AUTO_SUSPENDED: &str = "USER_AUTO_SUSPENDED";

/// Role codes
pub const ROLE_SYSTEM_ADMIN: &str = "SYSTEM_ADMIN";
pub const ROLE_ADMIN_LEGACY: &str = "admin";
pub const ROLE_PI: &str = "PI";
pub const ROLE_IACUC_STAFF: &str = "IACUC_STAFF";
pub const ROLE_VET: &str = "VET";
pub const ROLE_REVIEWER: &str = "REVIEWER";
pub const ROLE_IACUC_CHAIR: &str = "IACUC_CHAIR";
pub const ROLE_EXPERIMENT_STAFF: &str = "EXPERIMENT_STAFF";
pub const ROLE_WAREHOUSE_MANAGER: &str = "WAREHOUSE_MANAGER";
pub const ROLE_ADMIN_STAFF: &str = "ADMIN_STAFF";
pub const ROLE_GUEST: &str = "GUEST";
pub const ROLE_QAU: &str = "QAU";
pub const ROLE_STUDY_DIRECTOR: &str = "STUDY_DIRECTOR";
pub const ROLE_TEST_FACILITY_MANAGEMENT: &str = "TEST_FACILITY_MANAGEMENT";

/// Leave type codes
pub const LEAVE_ANNUAL: &str = "ANNUAL";
pub const LEAVE_PERSONAL: &str = "PERSONAL";
pub const LEAVE_SICK: &str = "SICK";
pub const LEAVE_COMPENSATORY: &str = "COMPENSATORY";
pub const LEAVE_MARRIAGE: &str = "MARRIAGE";
pub const LEAVE_BEREAVEMENT: &str = "BEREAVEMENT";
pub const LEAVE_MATERNITY: &str = "MATERNITY";
pub const LEAVE_PATERNITY: &str = "PATERNITY";
pub const LEAVE_MENSTRUAL: &str = "MENSTRUAL";
pub const LEAVE_OFFICIAL: &str = "OFFICIAL";

/// 假別代碼轉換為中文顯示名稱（共用於 dashboard 與 calendar）
pub fn get_leave_type_display(leave_type: &str) -> &'static str {
    match leave_type {
        LEAVE_ANNUAL => "特休假",
        LEAVE_PERSONAL => "事假",
        LEAVE_SICK => "病假",
        LEAVE_COMPENSATORY => "補休假",
        LEAVE_MARRIAGE => "婚假",
        LEAVE_BEREAVEMENT => "喪假",
        LEAVE_MATERNITY => "產假",
        LEAVE_PATERNITY => "陪產假",
        LEAVE_MENSTRUAL => "生理假",
        LEAVE_OFFICIAL => "公假",
        _ => "請假",
    }
}

// ============================================================
// R28-M3：Advisory Lock Key 中央註冊
// ============================================================
//
// PostgreSQL `pg_advisory_lock(key bigint)` / `pg_advisory_xact_lock(key bigint)`
// 的 key 命名空間是全域的。本系統使用兩種策略：
//
// 1. **靜態 i64 常數**（整個系統唯一、跨 instance）：
//    cron job multi-instance lock 等。
//    為避免與 `hashtext()` 結果（i32 範圍）衝突，**靜態常數的 magnitude 必須超出
//    i32 範圍**（即 `< i32::MIN as i64` 或 `> i32::MAX as i64`）。具體 bit pattern
//    不重要（正/負皆可），由下方 `test_static_lock_keys_outside_i32_range` 強制驗證。
//
// 2. **`hashtext($string)` 動態派生**（i32 範圍）：
//    依「鍵字串」分組的鎖，例如 per-email login lock、HMAC chain serialization。
//    所有 hashtext key 字串集中在此檔，避免不同模組無意間用相同 key 互搶。
//
// **新增 advisory lock 必更新本表**，確保命名空間不衝突。

/// H1（cron）: audit_chain_verify multi-instance lock。
/// 跨 pod 唯一，僅單一 instance 真的跑 audit chain verify。
/// `pg_try_advisory_lock` (session-scoped) by `audit_chain_verify.rs::AuditChainVerifyLock`.
pub const AUDIT_CHAIN_VERIFY_LOCK_KEY: i64 = 0x1A2B_3C4D_5E6F_7081_u64 as i64;

/// audit log HMAC chain 序列化鎖。
/// 並發 audit 寫入序列化，避免 chain 跳 row（指向 rollback 的死連結）。
/// `pg_advisory_xact_lock(hashtext($1))` by `audit.rs::log_activity_tx`.
pub const AUDIT_LOG_CHAIN_LOCK_KEY: &str = "audit_log_chain";

/// Protocol 編號生成鎖。
/// 序列化 APIG / PIG 編號生成，避免並發 max_seq+1 重複（CRIT-01）。
/// `pg_advisory_xact_lock(hashtext($1))` by `protocol/numbering.rs::acquire_numbering_lock`.
pub const PROTOCOL_NUMBERING_LOCK_KEY: &str = "protocol_iacuc_number_gen";

// 補充說明：per-email login attempt lock 不在此列舉，因 key 是 email 本身
// （`pg_advisory_xact_lock(hashtext($email))`）— 由 `auth/login.rs::validate_credentials`
// 直接綁 user 提供的 email 字串。命名空間獨立（一般 user email 不會與上述
// constant 字串衝突）。

#[cfg(test)]
mod tests {
    use super::*;

    /// R28-M3：驗證靜態 i64 lock key 落在 i32 範圍外，避免與 hashtext() 結果衝突。
    /// hashtext() 回 i32（範圍 -2^31 到 2^31-1）。靜態常數應在此範圍外。
    #[test]
    fn test_static_lock_keys_outside_i32_range() {
        // i32::MIN = -2147483648, i32::MAX = 2147483647
        let key = AUDIT_CHAIN_VERIFY_LOCK_KEY;
        assert!(
            key < i32::MIN as i64 || key > i32::MAX as i64,
            "AUDIT_CHAIN_VERIFY_LOCK_KEY ({key:#x}) 必須在 i32 範圍外，避免與 hashtext() 衝突"
        );
    }

    #[test]
    fn test_pagination_constants() {
        const { assert!(DEFAULT_PAGE_SIZE <= MAX_PAGE_SIZE, "預設頁大小應不超過最大值"); }
        const { assert!(MAX_PAGE_SIZE >= 1); }
    }

    #[test]
    fn test_password_policy() {
        const { assert!(PASSWORD_MIN_LENGTH >= 8, "密碼最小長度應 ≥ 8"); }
    }

    #[test]
    fn test_rate_limit_constants() {
        const { assert!(WRITE_RATE_LIMIT_PER_MINUTE <= API_RATE_LIMIT_PER_MINUTE); }
        const { assert!(UPLOAD_RATE_LIMIT_PER_MINUTE <= WRITE_RATE_LIMIT_PER_MINUTE); }
    }

    #[test]
    fn test_file_size_constants() {
        const { assert!(FILE_MAX_ANIMAL_PHOTO <= FILE_MAX_PROTOCOL_ATTACHMENT); }
        const { assert!(FILE_MAX_PROTOCOL_ATTACHMENT <= MAX_UPLOAD_SIZE_BYTES); }
    }

    #[test]
    fn test_audit_constants() {
        const { assert!(AUDIT_LOG_MAX_EXPORT > 0, "匯出上限應為正數"); }
        const { assert!(ACTIVITY_LOG_MAX_PER_PAGE > 0, "每頁筆數上限應為正數"); }
    }

    #[test]
    fn test_audit_export_reasonable_limit() {
        // 匯出上限應在合理範圍內（避免過大記憶體使用）
        const { assert!(AUDIT_LOG_MAX_EXPORT <= 100_000); }
    }

    #[test]
    fn test_leave_type_display() {
        assert_eq!(get_leave_type_display(LEAVE_ANNUAL), "特休假");
        assert_eq!(get_leave_type_display(LEAVE_PERSONAL), "事假");
        assert_eq!(get_leave_type_display(LEAVE_SICK), "病假");
        assert_eq!(get_leave_type_display("UNKNOWN"), "請假");
    }
}
