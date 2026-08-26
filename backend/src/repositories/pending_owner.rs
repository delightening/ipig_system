//! 「誰有資格處理這一關」的共用查詢。
//!
//! 放在本目錄的理由（見 `mod.rs` 的定位說明）：同一組查詢被待處理人解析器跨模組重複使用
//! （採購核准、加班審核、設備報廢 / 維修、動物欄位更正、AUP 行政預審…），
//! 呼叫端不該各自知道 `user_roles` / `role_permissions` 的接法。
//!
//! 三個函式都只回**在職且未軟刪除**的帳號：停用 / 離職者出現在「卡在誰」名單上會讓人白等，
//! 與 `services/notification/helpers.rs::get_users_by_role` 的過濾條件一致。

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// 使用者的識別與顯示名。
pub type UserRef = (Uuid, String);

/// 持有指定權限碼的在職使用者。
pub async fn list_users_with_permission(
    pool: &PgPool,
    permission_code: &str,
) -> Result<Vec<UserRef>, AppError> {
    let rows = sqlx::query_as::<_, UserRef>(
        r#"
        SELECT DISTINCT u.id, u.display_name
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE p.code = $1
          AND u.is_active = true
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(permission_code)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 同時持有指定權限碼**與**清單中任一角色碼的在職使用者。
///
/// 存在的理由是有些關卡真的是兩個條件疊加，只查其中一個會列出點下去拿 403 的人：
/// - 單據倉管核准：`handlers/document.rs:210` 先 `require_permission!("erp.document.approve")`，
///   :213 再另外擋掉非 `WAREHOUSE_MANAGER`。
/// - 沖銷核准：`handlers/document.rs:343` 要 `erp.document.reverse_approve`，
///   `services/document/reversal.rs:196` 再要 `is_admin()`（`SYSTEM_ADMIN` 或 legacy `admin`，
///   故本函式收的是角色**清單**而非單一角色）。
pub async fn list_users_with_permission_and_any_role(
    pool: &PgPool,
    permission_code: &str,
    role_codes: &[String],
) -> Result<Vec<UserRef>, AppError> {
    if role_codes.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, UserRef>(
        r#"
        SELECT DISTINCT u.id, u.display_name
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN role_permissions rp ON ur.role_id = rp.role_id
        JOIN permissions p ON rp.permission_id = p.id
        WHERE p.code = $1
          AND u.is_active = true
          AND u.deleted_at IS NULL
          AND EXISTS (
              SELECT 1 FROM user_roles ur2
              JOIN roles r2 ON ur2.role_id = r2.id
              WHERE ur2.user_id = u.id AND r2.code = ANY($2)
          )
        "#,
    )
    .bind(permission_code)
    .bind(role_codes)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 持有清單中**任一**角色碼的在職使用者（加班兩關用：admin 或 ADMIN_STAFF）。
pub async fn list_users_with_any_role(
    pool: &PgPool,
    role_codes: &[String],
) -> Result<Vec<UserRef>, AppError> {
    if role_codes.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, UserRef>(
        r#"
        SELECT DISTINCT u.id, u.display_name
        FROM users u
        JOIN user_roles ur ON u.id = ur.user_id
        JOIN roles r ON ur.role_id = r.id
        WHERE r.code = ANY($1)
          AND u.is_active = true
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(role_codes)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
