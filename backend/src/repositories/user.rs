use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppError, Result};

/// 查詢使用者顯示名稱（出現在 document/blood_test/hr/signature 等多處）
pub async fn find_user_display_name_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<String>> {
    let name = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(name)
}

/// 查詢使用者帳號狀態 (is_active, expires_at)。
///
/// 使用情境：auth_middleware 每請求驗證帳號狀態。
/// 回傳 `None` 表示使用者不存在；caller 應視為 Unauthorized。
///
/// R27-4：原本 SQL 寫在 middleware 內違反分層職責（CLAUDE.md「Middleware 禁業務邏輯」），
/// 移到 repository 層讓 services/access.rs 等其他模組也能復用。
pub async fn find_user_active_status_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<(bool, Option<DateTime<Utc>>)>> {
    let row: Option<(bool, Option<DateTime<Utc>>)> =
        sqlx::query_as("SELECT is_active, expires_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;
    Ok(row)
}

/// 查詢使用者所有有效權限 code（4-table JOIN: permissions × role_permissions ×
/// user_roles × roles，篩 `r.is_active = true`）。
///
/// 使用情境：auth_middleware 的 permission_cache loader（H2 single-flight）。
/// R27-4：從 middleware 抽到 repository 層；同 SELECT 也能被 services/access.rs
/// 等其他模組復用。
pub async fn list_permission_codes_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<String>> {
    let perms = sqlx::query_scalar::<_, String>(
        r#"SELECT DISTINCT p.code FROM permissions p
           INNER JOIN role_permissions rp ON p.id = rp.permission_id
           INNER JOIN user_roles ur ON rp.role_id = ur.role_id
           INNER JOIN roles r ON r.id = ur.role_id
           WHERE ur.user_id = $1 AND r.is_active = true"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("[repositories::user] 載入使用者 {} 權限失敗: {}", user_id, e);
        AppError::Internal("無法載入使用者權限".to_string())
    })?;
    Ok(perms)
}
