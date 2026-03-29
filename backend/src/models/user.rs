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
    #[validate(length(min = 10, max = 128, message = "Password must be at least 10 characters"))]
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
    #[serde(default = "default_is_internal")]
    pub is_internal: bool,
    #[serde(default)]
    pub role_ids: Vec<Uuid>,
    /// 帳號到期日（NULL = 永不過期），用於實習生等臨時帳號
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_is_internal() -> bool {
    true
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
    #[validate(length(min = 10, max = 128, message = "New password must be at least 10 characters"))]
    #[validate(custom(function = "validate_password_strength"))]
    pub new_password: String,
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
    #[validate(length(min = 10, max = 128, message = "New password must be at least 10 characters"))]
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
    #[validate(length(min = 10, max = 128, message = "New password must be at least 10 characters"))]
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
