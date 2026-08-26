//! 「誰有資格處理這一關」的共用查詢。
//!
//! 放在本目錄的理由（見 `mod.rs` 的定位說明）：同一組查詢被待處理人解析器跨模組重複使用
//! （採購核准、加班審核、設備報廢 / 維修、動物欄位更正、AUP 行政預審…），
//! 呼叫端不該各自知道 `user_roles` / `role_permissions` 的接法。
//!
//! 三個函式都只回**在職且未軟刪除**的帳號：停用 / 離職者出現在「卡在誰」名單上會讓人白等，
//! 與 `services/notification/helpers.rs::get_users_by_role` 的過濾條件一致。
//!
//! # 「持有權限」的判定必須與 `CurrentUser::has_permission` 一致
//!
//! ⚠️ 那個方法在 `middleware/auth.rs:88-90` **對管理員一律回 true**，不看
//! `role_permissions` 有沒有那一列。只比對授權表會漏掉靠這條短路取得權限的管理員，
//! 候選名單就會少列真的能核准的人。
//!
//! 下面兩支查詢因此都帶一段「或具管理員角色」的 OR 分支。**兩處必須一起改**——
//! 刻意不用 `format!` 共用字串：CI 有守衛擋 `format!` 內含 SQL 關鍵字
//! （`.github/workflows/ci.yml:296`，防注入），寫成常數拼接會直接紅燈。

use sqlx::PgPool;
use uuid::Uuid;

use crate::constants::{ROLE_ADMIN_LEGACY, ROLE_SYSTEM_ADMIN};
use crate::error::AppError;

/// 使用者的識別與顯示名。
pub type UserRef = (Uuid, String);

/// 在 `has_permission()` 眼中等同持有一切權限的角色。
fn admin_role_codes() -> Vec<String> {
    vec![ROLE_SYSTEM_ADMIN.to_string(), ROLE_ADMIN_LEGACY.to_string()]
}

/// 在職且**有效持有**指定權限碼的使用者（含管理員短路）。
pub async fn list_users_with_permission(
    pool: &PgPool,
    permission_code: &str,
) -> Result<Vec<UserRef>, AppError> {
    let rows = sqlx::query_as::<_, UserRef>(
        r#"
        SELECT DISTINCT u.id, u.display_name
        FROM users u
        WHERE u.is_active = true
          AND u.deleted_at IS NULL
          AND (
              EXISTS (
                  SELECT 1 FROM user_roles urp
                  JOIN role_permissions rp ON urp.role_id = rp.role_id
                  JOIN permissions p ON rp.permission_id = p.id
                  WHERE urp.user_id = u.id AND p.code = $1
              )
              OR EXISTS (
                  SELECT 1 FROM user_roles ura
                  JOIN roles ra ON ura.role_id = ra.id
                  WHERE ura.user_id = u.id AND ra.code = ANY($2)
              )
          )
        "#,
    )
    .bind(permission_code)
    .bind(admin_role_codes())
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 在職、有效持有指定權限碼、**且**具備清單中任一角色的使用者。
///
/// 存在的理由是有些關卡真的是兩個條件疊加，只查其中一個會列出點下去拿 403 的人：
/// - 單據倉管核准：`handlers/document.rs:210` 先 `require_permission!("erp.document.approve")`，
///   :213 再另外擋掉非 `WAREHOUSE_MANAGER`。
/// - 沖銷核准：`handlers/document.rs:343` 要 `erp.document.reverse_approve`，
///   `services/document/reversal.rs:196` 再要 `is_admin()`（`SYSTEM_ADMIN` 或 legacy `admin`，
///   故本函式收的是角色**清單**而非單一角色）。
///
/// ⚠️ **角色那一半沒有管理員短路**：`handlers/document.rs:213` 比對的是
/// `current_user.roles.contains(...)`，管理員不具該角色一樣會被擋，故不可比照權限那半放寬。
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
        WHERE u.is_active = true
          AND u.deleted_at IS NULL
          AND (
              EXISTS (
                  SELECT 1 FROM user_roles urp
                  JOIN role_permissions rp ON urp.role_id = rp.role_id
                  JOIN permissions p ON rp.permission_id = p.id
                  WHERE urp.user_id = u.id AND p.code = $1
              )
              OR EXISTS (
                  SELECT 1 FROM user_roles ura
                  JOIN roles ra ON ura.role_id = ra.id
                  WHERE ura.user_id = u.id AND ra.code = ANY($2)
              )
          )
          AND EXISTS (
              SELECT 1 FROM user_roles ur2
              JOIN roles r2 ON ur2.role_id = r2.id
              WHERE ur2.user_id = u.id AND r2.code = ANY($3)
          )
        "#,
    )
    .bind(permission_code)
    .bind(admin_role_codes())
    .bind(role_codes)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 持有清單中**任一**角色碼的在職使用者。
///
/// 純角色型關卡用（例：加班兩關看的是 `is_admin()` 與 `ADMIN_STAFF` 角色，
/// 沒有對應的權限碼——見 `services/hr/overtime.rs:248-249`）。
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
