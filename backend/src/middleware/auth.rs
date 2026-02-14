use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppError, AppState, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,        // user_id
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    /// 模擬登入時記錄原始管理員 ID（SEC-11）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impersonated_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CurrentUser {
    pub id: Uuid,
    #[allow(dead_code)]
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    /// 若為模擬登入，記錄原始管理員 ID（SEC-11）
    pub impersonated_by: Option<Uuid>,
}

impl CurrentUser {
    pub fn has_permission(&self, permission: &str) -> bool {
        // 檢查是否有直接權限
        if self.permissions.contains(&permission.to_string()) {
            return true;
        }
        // 檢查是否為管理員角色
        if self.roles.iter().any(|r| r == "SYSTEM_ADMIN" || r == "admin" || r.to_lowercase() == "admin") {
            return true;
        }
        false
    }

    #[allow(dead_code)]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    // 嘗試從多個來源取得 token（優先順序：Cookie > Bearer Header）
    let token = extract_token_from_cookie(&request)
        .or_else(|| extract_token_from_bearer(&request))
        .ok_or(AppError::Unauthorized)?;

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let current_user = CurrentUser {
        id: token_data.claims.sub,
        email: token_data.claims.email,
        roles: token_data.claims.roles,
        permissions: token_data.claims.permissions,
        impersonated_by: token_data.claims.impersonated_by,
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
            return Err(crate::AppError::Forbidden(format!("Permission denied: requires {}", $permission)));
        }
    };
}
