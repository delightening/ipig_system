use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, AppState, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    /// JWT 唯一識別碼，用於黑名單撤銷（SEC-23）
    #[serde(default)]
    pub jti: String,
    /// 模擬登入時記錄原始管理員 ID（SEC-11）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impersonated_by: Option<Uuid>,
    /// H3: JWT issuer（ipig-backend）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub iss: String,
    /// H3: JWT audience（ipig-system）
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub aud: String,
}

/// SEC-33：敏感操作二級認證用 JWT claims（短期 reauth token）
#[derive(Debug, Serialize, Deserialize)]
pub struct ReauthClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    /// 固定為 "reauth" 以區別一般 access token
    pub purpose: String,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    /// 當前 JWT 的 jti，供登出時加入黑名單（SEC-23）
    pub jti: String,
    /// JWT 過期時間戳，供黑名單清理用（SEC-23）
    pub exp: i64,
    /// 若為模擬登入，記錄原始管理員 ID（SEC-11）
    pub impersonated_by: Option<Uuid>,
}

impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        self.roles
            .iter()
            .any(|r| r == crate::constants::ROLE_SYSTEM_ADMIN || r == crate::constants::ROLE_ADMIN_LEGACY)
    }

    pub fn is_guest(&self) -> bool {
        self.roles
            .iter()
            .any(|r| r == crate::constants::ROLE_GUEST)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        // Guest 無任何權限（前端以靜態 demo data 展示，後端 guest_guard 攔截讀取）
        if self.is_guest() {
            return false;
        }
        if self.permissions.iter().any(|p| p == permission) {
            return true;
        }
        if self.is_admin() {
            return true;
        }
        false
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    // 嘗試從多個來源取得 token（優先順序：Bearer Header > Cookie）。
    // Bearer 優先可避免 Cookie 殘留覆蓋正確的 Authorization header，
    // 同時降低 Cookie 注入攻擊風險。
    let token = extract_token_from_bearer(&request)
        .or_else(|| extract_token_from_cookie(&request))
        .ok_or(AppError::Unauthorized)?;

    // R27-3：拆 validate_jwt 與 load_permissions 兩段，控制 auth_middleware 行數。
    let claims = validate_jwt(&state, &token)?;
    let permissions = load_permissions(&state, &claims).await?;

    let current_user = CurrentUser {
        id: claims.sub,
        email: claims.email,
        roles: claims.roles,
        permissions,
        jti: claims.jti,
        exp: claims.exp,
        impersonated_by: claims.impersonated_by,
    };

    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

/// 解碼並驗證 JWT — H3 演算法 / audience / issuer + SEC-23 黑名單。
fn validate_jwt(state: &AppState, token: &str) -> Result<Claims> {
    // H3: 明確指定演算法並驗證 audience/issuer，防止跨環境 token 重放。
    // SEC-UPG: 使用 ES256（ECDSA P-256）取代 HS256，防止對稱金鑰暴力破解。
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&["ipig-system"]);
    validation.set_issuer(&["ipig-backend"]);
    let token_data = decode::<Claims>(token, &state.config.jwt_keys.decoding, &validation)
        .map_err(|_| AppError::Unauthorized)?;

    // SEC-23: JWT 黑名單撤銷檢查。
    if !token_data.claims.jti.is_empty()
        && state.jwt_blacklist.is_revoked(&token_data.claims.jti)
    {
        tracing::warn!(
            "[Auth] JWT jti={} 已被撤銷，拒絕存取",
            token_data.claims.jti
        );
        return Err(AppError::Unauthorized);
    }
    Ok(token_data.claims)
}

/// 載入使用者權限 + 驗證帳號狀態。
///
/// 行為（Gemini PR #218 High 採納）：
/// - **所有使用者**（包括 admin）皆走 `try_get_with` single-flight + 真實
///   載入 perms（4-table JOIN），確保 cache value 對所有 user_id 語意一致。
/// - 4-table JOIN 對 admin 是「無用 perms」但僅 cache miss 才跑（5min 一次），
///   微小 cost；換得「cache 內容真實」的防禦性深度（未來改 has_permission
///   行為時 cache 內容仍可信）。
/// - 角色變更時 cache 失效由 handlers/{user,role}.rs 的 invalidate 處理。
///
/// H2：moka try_get_with single-flight，cache miss 時並發請求共享同一個 loader
/// future，避免 4-table JOIN stampede。loader 失敗（status 拒絕 / DB 錯誤）
/// 不會被快取，下次請求重試。
async fn load_permissions(state: &AppState, claims: &Claims) -> Result<Vec<String>> {
    let user_id = claims.sub;
    let db = state.db.clone();

    state
        .permission_cache
        .try_get_with::<_, AppError>(user_id, async move {
            check_user_active_status(&db, user_id).await?;
            crate::repositories::user::list_permission_codes_by_user(&db, user_id).await
        })
        .await
        .map_err(map_cache_loader_error)
}

/// CodeRabbit review #210：保留 loader 原始 AppError variant，
/// 避免 Forbidden/NotFound 等被吞成 500。
fn map_cache_loader_error(arc_err: std::sync::Arc<AppError>) -> AppError {
    match std::sync::Arc::try_unwrap(arc_err) {
        Ok(e) => e,
        Err(arc) => match &*arc {
            AppError::Unauthorized => AppError::Unauthorized,
            AppError::InvalidCredentials(m) => AppError::InvalidCredentials(m.clone()),
            AppError::Forbidden(m) => AppError::Forbidden(m.clone()),
            AppError::NotFound(m) => AppError::NotFound(m.clone()),
            AppError::Validation(m) => AppError::Validation(m.clone()),
            AppError::BadRequest(m) => AppError::BadRequest(m.clone()),
            AppError::Conflict(m) => AppError::Conflict(m.clone()),
            AppError::BusinessRule(m) => AppError::BusinessRule(m.clone()),
            AppError::TooManyRequests(m) => AppError::TooManyRequests(m.clone()),
            AppError::Internal(m) => AppError::Internal(m.clone()),
            e => AppError::Internal(format!("permission cache loader: {e}")),
        },
    }
}

/// BIZ-16 helper：檢查 user 是否仍 active 且未過期，否則回 Unauthorized。
///
/// R27-4：SQL 已下放至 `repositories::user::find_user_active_status_by_id`；
/// 本函式僅做業務判斷（拒停用 / 拒過期 / 拒不存在）。
async fn check_user_active_status(pool: &sqlx::PgPool, user_id: Uuid) -> Result<()> {
    let row = crate::repositories::user::find_user_active_status_by_id(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("[Auth] 無法查詢使用者 {} 狀態: {}", user_id, e);
            AppError::Internal("無法驗證使用者狀態".to_string())
        })?;

    match row {
        Some((is_active, expires_at)) => {
            if !is_active {
                tracing::warn!("[Auth] 使用者 {} 帳號已停用，拒絕存取", user_id);
                return Err(AppError::Unauthorized);
            }
            if let Some(exp) = expires_at {
                if exp < chrono::Utc::now() {
                    tracing::warn!("[Auth] 使用者 {} 帳號已過期，拒絕存取", user_id);
                    return Err(AppError::Unauthorized);
                }
            }
            Ok(())
        }
        None => {
            tracing::warn!("[Auth] 使用者 {} 不存在，拒絕存取", user_id);
            Err(AppError::Unauthorized)
        }
    }
}

/// 從 Cookie header 提取 access_token
fn extract_token_from_cookie(request: &Request) -> Option<String> {
    let cookie_header = request.headers().get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("access_token="))
        .map(|s| s.trim_start_matches("access_token=").to_string())
}

/// 從 Authorization: Bearer header 提取 token（向後相容）
fn extract_token_from_bearer(request: &Request) -> Option<String> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    auth_header.strip_prefix("Bearer ").map(|s| s.to_string())
}

/// 權限檢查巨集
#[macro_export]
macro_rules! require_permission {
    ($user:expr, $permission:expr) => {
        if !$user.has_permission($permission) {
            return Err($crate::AppError::Forbidden(format!(
                "Permission denied: requires {}",
                $permission
            )));
        }
    };
}
