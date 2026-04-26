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
    // 嘗試從多個來源取得 token（優先順序：Bearer Header > Cookie）
    // Bearer 優先可避免 Cookie 殘留覆蓋正確的 Authorization header，
    // 同時降低 Cookie 注入攻擊風險
    let token = extract_token_from_bearer(&request)
        .or_else(|| extract_token_from_cookie(&request))
        .ok_or(AppError::Unauthorized)?;

    // H3: 明確指定演算法並驗證 audience/issuer，防止跨環境 token 重放
    // SEC-UPG: 使用 ES256（ECDSA P-256）取代 HS256，防止對稱金鑰暴力破解
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&["ipig-system"]);
    validation.set_issuer(&["ipig-backend"]);
    let token_data = decode::<Claims>(
        &token,
        &state.config.jwt_keys.decoding,
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

    // BIZ-16 + H2：每請求驗證帳號狀態 + 載入權限。
    //
    // H2 (CodeRabbit 併發審查)：原本 DashMap-based 快取在 cache miss 時，多執行緒
    // 會同時跑 4-table JOIN（stampede）。改 moka::future::Cache + try_get_with
    // 後，cache miss 由「single-flight loader」處理 — 並發請求只有一個 task
    // 執行 DB 查詢，其餘等待同一結果。
    //
    // loader 同時做 user 狀態檢查（is_active + expires_at）與 perms 載入；
    // cache hit 即代表 TTL 內狀態驗證 + 權限載入皆已完成（5 分鐘最壞延遲，
    // 與原行為等價）。Loader 失敗（status 拒絕 / DB 錯誤）不會被快取，
    // 下次請求會重試 — 與原本「快取拒絕值」的退化行為相反，更符合
    // 「拒絕越快越好」直覺。
    let is_admin = claims.roles.iter().any(|r| {
        r == crate::constants::ROLE_SYSTEM_ADMIN || r == crate::constants::ROLE_ADMIN_LEGACY
    });
    let permissions = if is_admin {
        // admin 由 has_permission() 直接放行，不需載入 perms 也不需 cache。
        // 但仍需驗證 admin 帳號狀態（停用/過期 admin 應一樣被擋）。
        check_user_active_status(&state.db, claims.sub).await?;
        Vec::new()
    } else {
        let user_id = claims.sub;
        let db = state.db.clone();
        state
            .permission_cache
            .try_get_with::<_, AppError>(user_id, async move {
                check_user_active_status(&db, user_id).await?;
                let perms = sqlx::query_scalar::<_, String>(
                    r#"SELECT DISTINCT p.code FROM permissions p
                       INNER JOIN role_permissions rp ON p.id = rp.permission_id
                       INNER JOIN user_roles ur ON rp.role_id = ur.role_id
                       INNER JOIN roles r ON r.id = ur.role_id
                       WHERE ur.user_id = $1 AND r.is_active = true"#,
                )
                .bind(user_id)
                .fetch_all(&db)
                .await
                .map_err(|e| {
                    tracing::error!("[Auth] 無法載入使用者 {} 的權限: {}", user_id, e);
                    AppError::Internal("無法載入使用者權限".to_string())
                })?;
                Ok(perms)
            })
            .await
            .map_err(|arc_err| {
                // CodeRabbit review #210：保留 loader 原始 AppError variant，
                // 避免 Forbidden/NotFound 等被吞成 500。先 try_unwrap（單一 Arc
                // 持有時可拿走），失敗則 match 可 clone 的 variant；Database/Anyhow
                // 不可 clone → 化為 Internal（保留訊息便於追蹤）。
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
            })?
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

/// BIZ-16 helper：檢查 user 是否仍 active 且未過期，否則回 Unauthorized。
/// 由 H2 cache loader 與 admin 路徑共用。
async fn check_user_active_status(pool: &sqlx::PgPool, user_id: Uuid) -> Result<()> {
    let row: Option<(bool, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT is_active, expires_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
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
