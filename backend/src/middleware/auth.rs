use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
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
        // Guest 全通行（寫入由 guest_guard middleware 攔截）
        if self.is_guest() {
            return true;
        }
        if self.permissions.contains(&permission.to_string()) {
            return true;
        }
        if self.is_admin() {
            return true;
        }
        false
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    // 嘗試從多個來源取得 token（優先順序：Bearer Header > Cookie）
    // Bearer 優先可避免 Cookie 殘留覆蓋正確的 Authorization header，
    // 同時降低 Cookie 注入攻擊風險
    let token = extract_token_from_bearer(&request)
        .or_else(|| extract_token_from_cookie(&request))
        .ok_or(AppError::Unauthorized)?;

    // H3: 明確指定演算法並驗證 audience/issuer，防止跨環境 token 重放
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["ipig-system"]);
    validation.set_issuer(&["ipig-backend"]);
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized)?;

    // SEC-23: 檢查 JWT 是否已被撤銷（黑名單）
    if !token_data.claims.jti.is_empty() && state.jwt_blacklist.is_revoked(&token_data.claims.jti) {
        tracing::warn!(
            "[Auth] JWT jti={} 已被撤銷，拒絕存取",
            token_data.claims.jti
        );
        return Err(AppError::Unauthorized);
    }

    let claims = token_data.claims;

    // JWT 中省略 permissions 以符合 4096 bytes cookie 限制。
    // 非 admin 使用者從資料庫動態載入，admin 的 has_permission() 直接回傳 true 不需載入。
    let is_admin = claims.roles.iter().any(|r| {
        r == crate::constants::ROLE_SYSTEM_ADMIN || r == crate::constants::ROLE_ADMIN_LEGACY
    });
    let permissions = if !is_admin {
        sqlx::query_scalar::<_, String>(
            r#"SELECT DISTINCT p.code FROM permissions p
               INNER JOIN role_permissions rp ON p.id = rp.permission_id
               INNER JOIN user_roles ur ON rp.role_id = ur.role_id
               INNER JOIN roles r ON r.id = ur.role_id
               WHERE ur.user_id = $1 AND r.is_active = true"#,
        )
        .bind(claims.sub)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("[Auth] 無法載入使用者 {} 的權限: {}", claims.sub, e);
            AppError::Internal("無法載入使用者權限".to_string())
        })?
    } else {
        vec![]
    };

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
