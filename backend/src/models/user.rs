use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// 主題偏好
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

impl Default for ThemePreference {
    fn default() -> Self {
        ThemePreference::Light
    }
}

/// 語言偏好
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LanguagePreference {
    #[serde(rename = "zh-TW")]
    ZhTW,
    #[serde(rename = "en")]
    En,
}

impl Default for LanguagePreference {
    fn default() -> Self {
        LanguagePreference::ZhTW
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub is_internal: bool,
    pub is_active: bool,
    pub must_change_password: bool,
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub theme_preference: String,
    pub language_preference: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
    #[validate(length(min = 1, message = "Display name is required"))]
    pub display_name: String,
    pub phone: Option<String>,
    pub organization: Option<String>,
    #[serde(default = "default_is_internal")]
    pub is_internal: bool,
    #[serde(default)]
    pub role_ids: Vec<Uuid>,
}

fn default_is_internal() -> bool {
    true
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,
    #[validate(length(min = 1, message = "Display name is required"))]
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub is_internal: Option<bool>,
    pub is_active: Option<bool>,
    pub role_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
    pub must_change_password: bool,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub is_internal: bool,
    pub is_active: bool,
    pub must_change_password: bool,
    pub theme_preference: String,
    pub language_preference: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
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

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// 修改自己的密碼請求
#[derive(Debug, Deserialize, Validate)]
pub struct ChangeOwnPasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    #[validate(custom(function = "validate_password_strength"))]
    pub new_password: String,
}

/// Admin 重設他人密碼請求
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

/// 忘記密碼請求
#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// 重設密碼請求（透過 token）
#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordWithTokenRequest {
    pub token: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
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
        err.message = Some("Password must contain uppercase, lowercase, and numeric characters".into());
        Err(err)
    }
}
