use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 主題偏好
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    Light,
    Dark,
    System,
}

/// 語言偏好
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema, Default)]
pub enum LanguagePreference {
    #[serde(rename = "zh-TW")]
    #[default]
    ZhTW,
    #[serde(rename = "en")]
    En,
}

/// 使用者訓練/資格資料
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UserTraining {
    pub code: String,                   // A~F
    pub certificate_no: Option<String>, // 證書編號
    pub received_date: Option<String>,  // 取得時間
}

// User 含密碼雜湊與 TOTP 敏感欄位 — 必須覆寫 redacted_fields。
// 雖然 password_hash / totp_secret_encrypted / totp_backup_codes 已有
// `#[serde(skip_serializing)]` 讓它們在一般 JSON 序列化時被排除，但 audit
// 的 DataDiff 套一層 redact 機制作為 defence-in-depth：若某欄位改名或誤加
// `#[serde(skip_serializing_if)]` 條件跳脫，仍能由 redact 兜底。
impl crate::models::audit_diff::AuditRedact for User {
    fn redacted_fields() -> &'static [&'static str] {
        &[
            "password_hash",
            "totp_secret_encrypted",
            "totp_backup_codes",
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub organization: Option<String>,
    pub is_internal: bool,
    pub is_active: bool,
    pub must_change_password: bool,
    /// 所屬部門（migration 118；NULL = 未指派）。請假兩關審核以此反查 L1 部門主管。
    pub department_id: Option<Uuid>,
    // 登入失敗鎖定
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    // 使用者偏好
    pub theme_preference: String,
    pub language_preference: String,
    // 時間戳
    pub last_login_at: Option<DateTime<Utc>>,
    // AUP 第 8 節人員資料
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    pub aup_roles: Vec<String>,
    pub years_experience: i32,
    pub trainings: sqlx::types::Json<Vec<UserTraining>>,
    // 帳號到期日（NULL = 永不過期）
    pub expires_at: Option<DateTime<Utc>>,
    // TOTP 2FA
    pub totp_enabled: bool,
    #[serde(skip_serializing)]
    pub totp_secret_encrypted: Option<String>,
    #[serde(skip_serializing)]
    pub totp_backup_codes: Option<Vec<String>>,
    /// CSO #3：最後成功驗證的 TOTP time-step（防同窗重放）。
    #[serde(skip_serializing)]
    pub last_totp_step: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub organization: Option<String>,
    pub is_internal: bool,
    pub is_active: bool,
    pub must_change_password: bool,
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub theme_preference: String,
    pub language_preference: String,
    pub last_login_at: Option<DateTime<Utc>>,
    // AUP 第 8 節人員資料
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    pub aup_roles: Vec<String>,
    pub years_experience: i32,
    pub trainings: sqlx::types::Json<Vec<UserTraining>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 254, message = "Email must be at most 254 characters"))]
    pub email: String,
    #[validate(length(
        min = 10,
        max = 128,
        message = "Password must be at least 10 characters"
    ))]
    pub password: String,
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: String,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub organization: Option<String>,
    // AUP 第 8 節人員資料
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    #[serde(default)]
    pub aup_roles: Vec<String>,
    #[serde(default)]
    pub years_experience: i32,
    #[serde(default)]
    pub trainings: Vec<UserTraining>,
    /// 是否為本場受僱人員。
    ///
    /// **刻意不給 serde default**（原本預設 `true`）。兩條建立帳號的路徑先前
    /// 各自靜默猜測——這裡猜 true、邀請流程硬編 false——造成 9 位在職同仁的
    /// 分類與角色矛盾。邀請那條已改為必填，這條若留著預設值，等於
    /// `POST /users` 仍可靜默建出內部人員，兩條路徑的嚴謹度又分岔。
    pub is_internal: bool,
    #[serde(default)]
    pub role_ids: Vec<Uuid>,
    /// 帳號到期日（NULL = 永不過期），用於實習生等臨時帳號
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 254, message = "Email must be at most 254 characters"))]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Display name must be 1-100 characters"))]
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub organization: Option<String>,
    // AUP 第 8 節人員資料
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    pub aup_roles: Option<Vec<String>>,
    pub years_experience: Option<i32>,
    pub trainings: Option<Vec<UserTraining>>,
    pub is_internal: Option<bool>,
    pub is_active: Option<bool>,
    pub role_ids: Option<Vec<Uuid>>,
    /// 帳號到期日（NULL = 永不過期）
    pub expires_at: Option<DateTime<Utc>>,
    pub version: Option<i32>,
    /// 移除角色時，若對方身上仍有「未結清事項」（待他確認的職務代理假單、待他簽核的
    /// 請假單）預設會被擋下。僅系統管理員可帶此旗標強制執行，屆時系統會把那些待辦
    /// 釋出，避免單據跟著權限一起凍結。詳見 `UserService::find_unsettled_items`。
    pub force_role_change: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 254, message = "Email must be at most 254 characters"))]
    pub email: String,
    #[validate(length(min = 1, max = 128, message = "Password must be 1-128 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
    pub must_change_password: bool,
}

/// 計劃指派用的精簡使用者資料（PI / Study Director 下拉）。
/// 不含 email 以外的聯絡資訊、權限清單等敏感欄位，供具計畫匯入權限者（非僅 admin）取用。
#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow)]
pub struct AssignableUser {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub roles: Vec<String>,
}

/// 部門成員精簡資料（部門成員清單 / 組織圖用）。
/// 不含權限等敏感欄位；`department_id` 供組織圖依部門分組。
#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow)]
pub struct DepartmentMember {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub department_id: Option<Uuid>,
    /// 是否為本場受僱人員。
    ///
    /// 部門可以合法地同時有內部與外部成員（IACUC 外聘委員），成員名冊與組織圖
    /// 若不標示，編進去之後就再也分不出誰是外聘的。
    pub is_internal: bool,
}

/// IACUC 委員名冊的一列。
///
/// **與部門編制無關**——委員身分來自角色（IACUC_CHAIR / REVIEWER / IACUC_STAFF），
/// 而人事歸屬來自 `users.department_id`，兩者正交：
///
/// * 委員會是跨部門的橫切面（獸醫可以兼任審查委員），
/// * 但 `department_id` 是單值的，裝不下「同時屬於獸醫部與 IACUC」。
///
/// 硬把委員塞進 IACUC 部門會逼出「一個人要屬於兩個部門」的假問題。正解是
/// 部門樹照常呈現人事歸屬，委員名冊另外依角色查出來——同一個人可以同時
/// 出現在兩處，各自表達不同的事實。
///
/// `department_name` 帶出他實際歸屬哪個部門，讓名冊看得出「這位委員平時在哪」。
#[derive(Debug, Serialize, ToSchema, FromRow)]
pub struct CommitteeMember {
    pub id: Uuid,
    pub display_name: String,
    /// 人事歸屬部門名稱；未編入任何部門時為 `None`
    pub department_name: Option<String>,
    /// 是否為本場受僱人員（外聘委員為 false）
    pub is_internal: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub phone_ext: Option<String>,
    pub organization: Option<String>,
    pub is_internal: bool,
    pub is_active: bool,
    pub must_change_password: bool,
    pub theme_preference: String,
    pub language_preference: String,
    pub last_login_at: Option<DateTime<Utc>>,
    // AUP 第 8 節人員資料
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    pub aup_roles: Vec<String>,
    pub years_experience: i32,
    pub trainings: Vec<UserTraining>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub totp_enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

impl UserResponse {
    pub fn from_user(user: &User, roles: Vec<String>, permissions: Vec<String>) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            phone: user.phone.clone(),
            phone_ext: user.phone_ext.clone(),
            organization: user.organization.clone(),
            is_internal: user.is_internal,
            is_active: user.is_active,
            must_change_password: user.must_change_password,
            theme_preference: user.theme_preference.clone(),
            language_preference: user.language_preference.clone(),
            last_login_at: user.last_login_at,
            entry_date: user.entry_date,
            position: user.position.clone(),
            aup_roles: user.aup_roles.clone(),
            years_experience: user.years_experience,
            trainings: user.trainings.0.clone(),
            roles,
            permissions,
            totp_enabled: user.totp_enabled,
            expires_at: user.expires_at,
        }
    }
}

/// 使用者偏好設定請求
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePreferencesRequest {
    pub theme_preference: Option<String>,
    pub language_preference: Option<String>,
}

/// 帳號鎖定狀態
#[derive(Debug, Serialize)]
pub struct AccountLockStatus {
    pub is_locked: bool,
    pub locked_until: Option<DateTime<Utc>>,
    pub remaining_attempts: i32,
}

/// 登入失敗常數
pub const MAX_LOGIN_ATTEMPTS: i32 = 5;
pub const LOCK_DURATION_MINUTES: i64 = 15;

// ============================================
// 2FA TOTP Models
// ============================================

/// 登入時若 2FA 啟用，回傳此結構要求前端輸入 TOTP
#[derive(Debug, Serialize, ToSchema)]
pub struct TwoFactorRequiredResponse {
    pub requires_2fa: bool,
    pub temp_token: String,
}

/// 2FA 登入驗證請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TwoFactorLoginRequest {
    pub temp_token: String,
    #[validate(length(min = 6, max = 8, message = "驗證碼為 6~8 碼"))]
    pub code: String,
}

/// 2FA 設定回應（含 otpauth URI 和備用碼）
#[derive(Debug, Serialize, ToSchema)]
pub struct TwoFactorSetupResponse {
    pub otpauth_uri: String,
    pub backup_codes: Vec<String>,
}

/// 2FA 確認啟用請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TwoFactorConfirmRequest {
    #[validate(length(equal = 6, message = "驗證碼必須為 6 碼"))]
    pub code: String,
}

/// 2FA 停用請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct TwoFactorDisableRequest {
    #[validate(length(min = 1, message = "請輸入密碼"))]
    pub password: String,
    #[validate(length(min = 6, max = 8, message = "驗證碼為 6~8 碼"))]
    pub code: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 修改自己的密碼請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangeOwnPasswordRequest {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Current password must be 1-128 characters"
    ))]
    pub current_password: String,
    #[validate(length(
        min = 10,
        max = 128,
        message = "New password must be at least 10 characters"
    ))]
    #[validate(custom(function = "validate_password_strength"))]
    pub new_password: String,
    /// C3 (GLP §11.300 / NIST SP 800-63)：新密碼二次確認，避免使用者誤輸入。
    /// 用 validator must_match 確保與 `new_password` 一致；handler 走 `req.validate()?`。
    #[validate(length(min = 1, message = "請再次輸入新密碼"))]
    #[validate(must_match(other = "new_password", message = "新密碼與確認密碼不一致"))]
    pub new_password_confirmation: String,
}

/// SEC-33：敏感操作二級認證 — 確認密碼請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ConfirmPasswordRequest {
    #[validate(length(min = 1, message = "請輸入密碼"))]
    pub password: String,
}

/// Admin 重設他人密碼請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordRequest {
    #[validate(length(
        min = 10,
        max = 128,
        message = "New password must be at least 10 characters"
    ))]
    pub new_password: String,
}

/// 忘記密碼請求
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// 重設密碼請求（透過 token）
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordWithTokenRequest {
    pub token: String,
    #[validate(length(
        min = 10,
        max = 128,
        message = "New password must be at least 10 characters"
    ))]
    #[validate(custom(function = "validate_password_strength"))]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    /// R35-15: 同一登入鏈共享，reuse detection 觸發整族撤銷。
    pub family_id: Uuid,
    /// R35-15: 撤銷原因（normal_rotation / reuse_detected / password_changed / admin_logout）。
    pub revoked_reason: Option<String>,
    /// R41-1: 最後使用時間（每次 refresh 更新）。閒置超過
    /// `Config::auth_idle_timeout_minutes` 即拒絕 refresh，強制使用者重新登入。
    pub last_used_at: DateTime<Utc>,
    /// R46-1: normal_rotation 撤銷時的時間戳。reuse detection 時用於 race window
    /// 判定（≤5s 視為併發 race，不觸發 family revoke）。非 normal_rotation
    /// 撤銷或尚未撤銷時為 NULL。
    pub rotated_at: Option<DateTime<Utc>>,
    /// R46-2: rotation 當下 client IP（TEXT）。reuse 真案時與當前 request IP
    /// 比對，相同則 severity 降為 warning（疑似 browser bug）；不同維持 critical。
    pub last_ip: Option<String>,
    /// R46-2: rotation 當下 client User-Agent。reuse 真案時與當前 request UA
    /// 比對，相同則 severity 降為 warning。
    pub last_user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 驗證密碼強度（至少包含大小寫字母和數字）
fn validate_password_strength(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if has_uppercase && has_lowercase && has_digit {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("password_strength");
        err.message =
            Some("Password must contain uppercase, lowercase, and numeric characters".into());
        Err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // validate_password_strength validator 測試
    // ==========================================

    #[test]
    fn test_password_strength_valid() {
        assert!(validate_password_strength("Abcdef1!").is_ok());
    }

    #[test]
    fn test_password_strength_no_uppercase() {
        assert!(validate_password_strength("abcdef1!").is_err());
    }

    #[test]
    fn test_password_strength_no_lowercase() {
        assert!(validate_password_strength("ABCDEF1!").is_err());
    }

    #[test]
    fn test_password_strength_no_digit() {
        assert!(validate_password_strength("Abcdefgh").is_err());
    }

    // ==========================================
    // ThemePreference 序列化測試
    // ==========================================

    #[test]
    fn test_theme_preference_serde() {
        let light = ThemePreference::Light;
        let json = serde_json::to_string(&light).expect("序列化 Light 失敗");
        assert_eq!(json, "\"light\"");

        let dark: ThemePreference = serde_json::from_str("\"dark\"").expect("反序列化 dark 失敗");
        assert_eq!(dark, ThemePreference::Dark);

        let system: ThemePreference =
            serde_json::from_str("\"system\"").expect("反序列化 system 失敗");
        assert_eq!(system, ThemePreference::System);
    }

    #[test]
    fn test_theme_preference_default() {
        assert_eq!(ThemePreference::default(), ThemePreference::Light);
    }

    // ==========================================
    // LanguagePreference 序列化測試
    // ==========================================

    #[test]
    fn test_language_preference_serde() {
        let zh = LanguagePreference::ZhTW;
        let json = serde_json::to_string(&zh).expect("序列化 ZhTW 失敗");
        assert_eq!(json, "\"zh-TW\"");

        let en: LanguagePreference = serde_json::from_str("\"en\"").expect("反序列化 en 失敗");
        assert_eq!(en, LanguagePreference::En);
    }

    #[test]
    fn test_language_preference_default() {
        assert_eq!(LanguagePreference::default(), LanguagePreference::ZhTW);
    }

    // ==========================================
    // 常數測試
    // ==========================================

    #[test]
    fn test_login_constants() {
        assert_eq!(MAX_LOGIN_ATTEMPTS, 5);
        assert_eq!(LOCK_DURATION_MINUTES, 15);
    }
}
