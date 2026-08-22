/// Pagination
pub const DEFAULT_PAGE_SIZE: i64 = 50;
pub const MAX_PAGE_SIZE: i64 = 200;
pub const REPORT_MAX_ROWS: i64 = 1000;
pub const ANALYSIS_MAX_ROWS: i64 = 5000;

/// Authentication
/// Guest demo 公開試用帳號 — 豁免帳號鎖定 / 暴力破解警報，避免他人錯密試用導致全體訪客無法進入
pub const GUEST_DEMO_EMAIL: &str = "guest@guest.com";
/// 軟刪除（匿名化）帳號的 email 網域後綴。`UserService::delete` 將 email 改寫為
/// `deleted_<uuid>@deleted.local`；`UserService::list` 據此把已刪除帳號排除於清單外，
/// 即使開啟「顯示停用帳號」也不顯示（僅顯示停用但未刪除的帳號）。
pub const DELETED_USER_EMAIL_DOMAIN: &str = "@deleted.local";
/// ⚠️ 死碼（2026-07-04 稽核 B-4）：實際 access token 效期由 `JWT_EXPIRATION_MINUTES`
/// （config.rs）決定；本常數已無任何引用，保留僅為歷史，勿據此推斷 token 效期。
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

/// AUP 計畫書 PDF render 快取（`PdfServiceClient::render_aup_from_working_content`）。
/// 「計畫內容」分頁預覽每次都觸發 WeasyPrint 全量 render（單份 ~15s），且 render 序列化
/// （cap=1）；同一份未修改的計畫書反覆預覽時純屬重算浪費。以「送進 print-pdf 的 body
/// （含已內嵌照片）的 sha256」為 key 快取 PDF bytes：同內容即命中、內容一改 hash 即變
/// 自動失效，無 staleness 風險。
///
/// 容量以**位元組**計（moka weigher），避免少數大 PDF 撐爆 RAM（prod 跑在筆電）。
pub const AUP_PDF_CACHE_MAX_BYTES: u64 = 64 * 1024 * 1024; // 64 MB
/// 快取 TTL（秒）。key 為內容 hash 故 TTL 不影響正確性，僅用於回收長期不再預覽的舊版本記憶體。
pub const AUP_PDF_CACHE_TTL_SECS: u64 = 1800; // 30 分鐘

/// File Upload — 各類別最大檔案大小 (bytes)
pub const MAX_UPLOAD_SIZE_BYTES: usize = 30 * 1024 * 1024; // 30MB 全域
pub const FILE_MAX_PROTOCOL_ATTACHMENT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_ANIMAL_PHOTO: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_DATA_IMPORT: usize = 100 * 1024 * 1024; // 100 MB（全庫 IDXF 匯入）
pub const FILE_MAX_PATHOLOGY_REPORT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_LEAVE_ATTACHMENT: usize = 10 * 1024 * 1024; // 10 MB
pub const FILE_MAX_OBSERVATION_ATTACHMENT: usize = 20 * 1024 * 1024; // 20 MB
pub const FILE_MAX_SOP_DOCUMENT: usize = 30 * 1024 * 1024; // 30 MB
pub const FILE_MAX_MESSAGE_ATTACHMENT: usize = 10 * 1024 * 1024; // 10 MB（站內信圖片，前端壓縮後通常 < 1MB）

/// Auth 短期 token 過期秒數
pub const TWO_FA_TEMP_EXPIRES_SECS: i64 = 300; // 5 分鐘
pub const REAUTH_EXPIRES_SECS: i64 = 300; // 5 分鐘

/// R66-B3: step-up（confirm-password / 2fa-disable / 電子簽章）密碼重驗的暴力破解
/// 防護門檻。**刻意獨立於登入鎖定**：step-up 只計 `reauth_failure`、登入只計
/// `login_failure`，兩計數器互不影響——攻擊者狂打 step-up 只鎖 step-up、不鎖登入
/// （避免 DoS），反之亦然。採 const 而非 config，因 `verify_password_by_id` 的呼叫端
/// （簽章 service）不持有 `Config`，threading config 會擴大改動面。
pub const STEP_UP_LOCKOUT_MAX_ATTEMPTS: i64 = 5;
pub const STEP_UP_LOCKOUT_WINDOW_MINS: i64 = 15;

/// 預設時區（HR 打卡、報表等）
pub const DEFAULT_TIMEZONE: &str = "Asia/Taipei";
/// UTC+8 偏移秒數（台灣時區）
pub const TAIWAN_OFFSET_SECS: i32 = 8 * 3600;

/// 員工通知 email 寄送時間窗（台灣時間，週一至週五、排除國定假日）。
/// 窗外產生的通知 email 延後到下一個合法窗口寄出（站內通知不延後）。
/// 寄送條件：`WINDOW_START_HOUR <= 當地小時 < WINDOW_END_HOUR`。
pub const STAFF_EMAIL_WINDOW_START_HOUR: u32 = 9; // 09:00
pub const STAFF_EMAIL_WINDOW_END_HOUR: u32 = 17; // 17:00（exclusive）

/// 台灣國定假日資料來源（政府開放資料）。`{year}` 會被替換為西元年。
/// 預期回傳 JSON 陣列：`[{"date":"YYYYMMDD","isHoliday":true,...}, ...]`
/// （行政院人事行政總處「政府行政機關辦公日曆表」之機器可讀鏡像格式）。
/// 可由 system_settings key `holiday_calendar_url` 覆寫，指向任何回傳相同 JSON 形狀的端點。
///
/// ⚠️ 實際端點格式須於有對外網路的環境驗證；建置沙箱因 egress 政策（403）無法 live 驗證。
pub const HOLIDAY_CALENDAR_URL_TEMPLATE_DEFAULT: &str =
    "https://cdn.jsdelivr.net/gh/ruyut/TaiwanCalendar/data/{year}.json";
/// system_settings 中假日 URL 模板的 key。
pub const SETTING_HOLIDAY_CALENDAR_URL: &str = "holiday_calendar_url";

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
///
/// `AUTH_IDLE_TIMEOUT_MINUTES` env 未設時 `config.rs::Config::from_env` 的 fallback。
/// 2026-05-22: 30 → 480（8h）對齊 PR #455 sliding session overhaul。
/// 2026-05-25: 480 → 600（10h）per 使用者需求；搭配前端 per-tab idle tracking，
/// 閒置分頁前端自行清畫面，不影響活躍分頁。
pub const SESSION_IDLE_TIMEOUT_MINUTES: i64 = 600;
/// 2026-05-18: 5 → 10. 使用者常開多分頁 / 多裝置（手機+電腦），5 個太低會
/// 頻繁觸發 session_limit 砍舊 session（搭配 end_excess_sessions 改 ORDER BY
/// last_activity_at 後，舊的 = 最不活躍的，被砍合理）。
pub const MAX_SESSIONS_PER_USER: i64 = 10;

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
/// admin 手動解除帳號鎖定（安全控制變更，進 HMAC 稽核鏈）
pub const SEC_EVENT_ACCOUNT_LOCKOUT_CLEAR: &str = "ACCOUNT_LOCKOUT_CLEAR";
pub const SEC_EVENT_HONEYPOT_HIT: &str = "HONEYPOT_HIT";
pub const SEC_EVENT_AUTO_SUSPENDED: &str = "USER_AUTO_SUSPENDED";
/// R35-15: 偵測到已撤銷的 refresh token 被重用 — 視為 token 洩漏
pub const SEC_EVENT_REFRESH_TOKEN_REUSE: &str = "REFRESH_TOKEN_REUSE";

// login_events 的「解鎖標記」事件型別。
// 鎖定是「視窗內失敗筆數 >= 門檻」的現算結果，沒有旗標可清；解鎖的做法是寫一筆標記，
// 計數端只算最後一筆標記之後的失敗。失敗紀錄本身不刪（暴力破解的稽核證據）。
/// 密碼變更觸發的自動解鎖（#1086）
pub const LOGIN_EVENT_LOCKOUT_RESET: &str = "lockout_reset";
/// 管理員在安全警報頁手動解鎖
pub const LOGIN_EVENT_LOCKOUT_ADMIN_CLEAR: &str = "lockout_admin_clear";

/// R35-15: refresh token 撤銷原因
pub const REFRESH_TOKEN_REVOKED_NORMAL_ROTATION: &str = "normal_rotation";
pub const REFRESH_TOKEN_REVOKED_REUSE_DETECTED: &str = "reuse_detected";
pub const REFRESH_TOKEN_REVOKED_PASSWORD_CHANGED: &str = "password_changed";
pub const REFRESH_TOKEN_REVOKED_ADMIN_LOGOUT: &str = "admin_logout";
/// R41-1: refresh 時 last_used_at 超過 idle window 時的撤銷 reason
pub const REFRESH_TOKEN_REVOKED_IDLE_TIMEOUT: &str = "idle_timeout";
/// R46-1: refresh token reuse race window — `rotated_at` 距今 ≤ 此秒數視為
/// 併發 race（多分頁同時 refresh / 行動裝置斷網重試），不觸發 family revoke。
///
/// 2026-05-20 拉長 5 → 300：實測在 prod-on-laptop 多 tab 工作流下，access
/// token 過期時並行 polling（unread-count / pending-count / alerts/recent）
/// 跨 tab 各自送 in-flight refresh，第二個 request 帶舊 cookie 抵達 backend
/// 時若已過 5 秒 race window → family revoke → 使用者連續被登出。前端已補
/// navigator.locks 跨 tab 互斥（authBroadcast），這裡 race window 拉長作為
/// 保底；trade-off：真實 token leak 攻擊的早期偵測延遲從 5 秒擴大到 5 分鐘，
/// 對非高價值目標（1 人 vet system）可接受。
pub const REFRESH_TOKEN_REUSE_RACE_WINDOW_SECS: i64 = 300;
/// R46-3: stale-tab heuristic 門檻（秒）。token 被 revoke 後超過此時間才重用
/// 且 last_ip / last_user_agent baseline 為 NULL（pre-R46-2 legacy data 或從未
/// 經過 rotation），視為「使用者用了 day-old 的 browser tab」而非實時攻擊，
/// severity 降為 `warning`（仍走 family revoke + alert，只是不升 critical）。
///
/// 取 1 小時：真實攻擊者通常分鐘級內就會用偷到的 token；橫跨小時級的 reuse
/// 多半是 stale tab / 重新整理。
pub const REFRESH_TOKEN_REUSE_STALE_THRESHOLD_SECS: i64 = 3600;
/// R41-1: AppError::Validation 回傳的 session idle timeout 錯誤碼（前端依此 i18n）
pub const SESSION_ERROR_IDLE_TIMEOUT: &str = "session_idle_timeout";

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
/// 負責人：請假審核鏈終審關（單位主管之上），全公司層級角色。
pub const ROLE_DIRECTOR: &str = "DIRECTOR";
pub const ROLE_GUEST: &str = "GUEST";
pub const ROLE_QAU: &str = "QAU";
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

/// R63-C3: scheduler leader election multi-instance lock。
/// 多 instance 部署時僅單一 instance 跑排程。
/// `pg_try_advisory_lock` (session-scoped) by `scheduler.rs::SchedulerService::start`.
/// 值在 i32 範圍外（避免與 hashtext() 衝突，由下方測試強制）。
pub const SCHEDULER_LEADER_LOCK_ID: i64 = 7_370_000_000_000;

/// audit log HMAC chain 序列化鎖。
/// 並發 audit 寫入序列化，避免 chain 跳 row（指向 rollback 的死連結）。
/// `pg_advisory_xact_lock(hashtext($1))` by `audit.rs::log_activity_tx`.
pub const AUDIT_LOG_CHAIN_LOCK_KEY: &str = "audit_log_chain";

/// Protocol 編號生成鎖。
/// 序列化 APIG / PIG 編號生成，避免並發 max_seq+1 重複（CRIT-01）。
/// `pg_advisory_xact_lock(hashtext($1))` by `protocol/numbering.rs::acquire_numbering_lock`.
pub const PROTOCOL_NUMBERING_LOCK_KEY: &str = "protocol_iacuc_number_gen";

/// 單據編號（`documents.doc_no`）生成鎖。
/// 序列化 `{PREFIX}-YYMMDD-{NN}` 的 max+1 取號，避免並發撞上 `documents_doc_no_key`
/// 唯一索引——與 CRIT-01 同型的缺陷，只是當初漏在 documents 上（2026-08-03 修）。
/// `pg_advisory_xact_lock(hashtext($1))` by `document/crud.rs::acquire_doc_no_lock`.
///
/// 用單一全域鎖而非 per-doc_type：與上方 protocol 取號慣例一致，且本系統單日單一
/// doc_type 開單量僅十餘張，爭用可忽略。
pub const DOCUMENT_NUMBERING_LOCK_KEY: &str = "document_doc_no_gen";

// ------------------------------------------------------------------
// 主檔代碼自動產生鎖（2026-08-03）
//
// 三者與上方 document / protocol 取號屬**同一缺陷家族**：read-max-then-insert，
// 無鎖時並發會算出同一個代碼、撞上各自的唯一約束（實測 prod：
// `warehouses_code_key`、`partners_code_key`、`storage_locations_warehouse_id_code_key`）。
//
// 這三張表的代碼空間彼此獨立，故**各給一把鎖**（不像 protocol 的 APIG/PIG 共用
// 流水號空間需共鎖）；建倉庫不該阻塞建供應商。
// ------------------------------------------------------------------

/// 倉庫代碼（`warehouses.code`，`WH{:03}`）生成鎖。
/// `pg_advisory_xact_lock(hashtext($1))` by `warehouse.rs::acquire_code_lock`.
pub const WAREHOUSE_CODE_LOCK_KEY: &str = "warehouse_code_gen";

/// 夥伴代碼（`partners.code`，如 `藥001` / `客001`）生成鎖。
/// 供應商各類別（藥/耗/飼/儀）與客戶共用本鎖：類別間流水號雖獨立，但新建頻率極低，
/// 不值得為此再拆四把 key。
/// `pg_advisory_xact_lock(hashtext($1))` by `partner.rs::acquire_code_lock`.
pub const PARTNER_CODE_LOCK_KEY: &str = "partner_code_gen";

/// 儲位代碼（`storage_locations.code`，`{A-Z}{:02}`）生成鎖。
/// 唯一約束是 `(warehouse_id, code)`，但取號量極小，用單一鎖而非 per-warehouse，
/// 避免 lock key 命名空間膨脹。
/// `pg_advisory_xact_lock(hashtext($1))` by `storage_location.rs::acquire_code_lock`.
pub const STORAGE_LOCATION_CODE_LOCK_KEY: &str = "storage_location_code_gen";

// 補充說明：per-email login attempt lock 不在此列舉，因 key 是 email 本身
// （`pg_advisory_xact_lock(hashtext($email))`）— 由 `auth/login.rs::validate_credentials`
// 直接綁 user 提供的 email 字串。命名空間獨立（一般 user email 不會與上述
// constant 字串衝突）。
//
// 補充說明（Low-5 #239）：stock ledger 過帳鎖用 **2-arg** overload
// `pg_advisory_xact_lock(warehouse_id::int4, product_id::int4)`（`stock/ledger.rs`），
// 序列化「同倉同品」的庫存過帳。PostgreSQL 的 2-arg `(int4, int4)` 與上述 1-arg
// `(int8)` overload 屬**不同命名空間**，物理上不可能與本表的 1-arg / hashtext 鎖衝突，
// 故不需納入上方常數，但在此登記以維持「所有 advisory lock 集中可查」。

#[cfg(test)]
mod tests {
    use super::*;

    /// R28-M3：驗證靜態 i64 lock key 落在 i32 範圍外，避免與 hashtext() 結果衝突。
    /// hashtext() 回 i32（範圍 -2^31 到 2^31-1）。靜態常數應在此範圍外。
    #[test]
    fn test_static_lock_keys_outside_i32_range() {
        // i32::MIN = -2147483648, i32::MAX = 2147483647
        for (name, key) in [
            ("AUDIT_CHAIN_VERIFY_LOCK_KEY", AUDIT_CHAIN_VERIFY_LOCK_KEY),
            ("SCHEDULER_LEADER_LOCK_ID", SCHEDULER_LEADER_LOCK_ID),
        ] {
            assert!(
                key < i32::MIN as i64 || key > i32::MAX as i64,
                "{name} ({key:#x}) 必須在 i32 範圍外，避免與 hashtext() 衝突"
            );
        }
    }

    #[test]
    fn test_pagination_constants() {
        const {
            assert!(
                DEFAULT_PAGE_SIZE <= MAX_PAGE_SIZE,
                "預設頁大小應不超過最大值"
            );
        }
        const {
            assert!(MAX_PAGE_SIZE >= 1);
        }
    }

    #[test]
    fn test_password_policy() {
        const {
            assert!(PASSWORD_MIN_LENGTH >= 8, "密碼最小長度應 ≥ 8");
        }
    }

    #[test]
    fn test_rate_limit_constants() {
        const {
            assert!(WRITE_RATE_LIMIT_PER_MINUTE <= API_RATE_LIMIT_PER_MINUTE);
        }
        const {
            assert!(UPLOAD_RATE_LIMIT_PER_MINUTE <= WRITE_RATE_LIMIT_PER_MINUTE);
        }
    }

    #[test]
    fn test_file_size_constants() {
        const {
            assert!(FILE_MAX_ANIMAL_PHOTO <= FILE_MAX_PROTOCOL_ATTACHMENT);
        }
        const {
            assert!(FILE_MAX_PROTOCOL_ATTACHMENT <= MAX_UPLOAD_SIZE_BYTES);
        }
    }

    #[test]
    fn test_audit_constants() {
        const {
            assert!(AUDIT_LOG_MAX_EXPORT > 0, "匯出上限應為正數");
        }
        const {
            assert!(ACTIVITY_LOG_MAX_PER_PAGE > 0, "每頁筆數上限應為正數");
        }
    }

    #[test]
    fn test_audit_export_reasonable_limit() {
        // 匯出上限應在合理範圍內（避免過大記憶體使用）
        const {
            assert!(AUDIT_LOG_MAX_EXPORT <= 100_000);
        }
    }

    #[test]
    fn test_leave_type_display() {
        assert_eq!(get_leave_type_display(LEAVE_ANNUAL), "特休假");
        assert_eq!(get_leave_type_display(LEAVE_PERSONAL), "事假");
        assert_eq!(get_leave_type_display(LEAVE_SICK), "病假");
        assert_eq!(get_leave_type_display("UNKNOWN"), "請假");
    }
}
