//! 資源存取權限檢查（防範 IDOR）
//! 此模組依賴 CurrentUser（middleware 層別）與 AppError，屬於業務邏輯，置於 services/ 層。

use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, middleware::CurrentUser, Result};

/// 檢查使用者是否有權存取資源
/// - 資源擁有者（resource_owner_id == current_user.id）可存取
/// - 具備 admin_permission 權限者可存取（如 hr.leave.view_all）
/// - 否則回傳 403 Forbidden
pub fn check_resource_access(
    current_user: &CurrentUser,
    resource_owner_id: Uuid,
    admin_permission: &str,
) -> Result<()> {
    if current_user.id == resource_owner_id || current_user.has_permission(admin_permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此資源".into()))
    }
}

// ============================================
// 計畫書存取權限檢查
// ============================================

/// 角色列表：具有 view_all 權限的角色
const VIEW_ALL_ROLES: &[&str] = &[crate::constants::ROLE_IACUC_CHAIR, crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_VET, crate::constants::ROLE_REVIEWER];

/// 使用者是否為計畫的 PI / 委託人 / Co-Editor
pub async fn is_pi_or_coeditor(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_protocols
            WHERE protocol_id = $1 AND user_id = $2
              AND role_in_protocol IN ('PI', 'CLIENT', 'CO_EDITOR')
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫的指派審查委員
pub async fn is_assigned_reviewer(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM review_assignments
            WHERE protocol_id = $1 AND reviewer_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫的指派獸醫
pub async fn is_assigned_vet(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM vet_review_assignments
            WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否與計畫有任何關聯（PI / Co-Editor / 審查委員 / 獸醫）
pub async fn has_any_protocol_role(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
            UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
            UNION SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
            UNION SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫的 PI（用於 amendment）
pub async fn is_protocol_pi(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_protocols
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        )"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否與計畫有關聯（any role in user_protocols）
pub async fn has_protocol_membership(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM user_protocols
            WHERE user_id = $1 AND protocol_id = $2
        )"#,
    )
    .bind(user_id)
    .bind(protocol_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為計畫審查委員或獸醫（用於 review comment 權限）
pub async fn is_reviewer_or_vet(pool: &PgPool, protocol_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
            UNION SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
        )"#,
    )
    .bind(protocol_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否為 amendment 的指派審查員
pub async fn is_amendment_reviewer(pool: &PgPool, amendment_id: Uuid, user_id: Uuid) -> Result<bool> {
    let (exists,): (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM amendment_review_assignments
            WHERE amendment_id = $1 AND reviewer_id = $2
        )"#,
    )
    .bind(amendment_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 使用者是否有 view_all 計畫權限（含角色檢查）
pub fn has_protocol_view_all(current_user: &CurrentUser) -> bool {
    current_user.has_permission("aup.protocol.view_all")
        || current_user.roles.iter().any(|r| VIEW_ALL_ROLES.contains(&r.as_str()))
}

/// 檢查計畫查看權限（view_all 或有任何計畫角色），失敗回傳 403
pub async fn require_protocol_view_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
    pi_user_id: Uuid,
) -> Result<()> {
    if has_protocol_view_all(current_user) || current_user.id == pi_user_id {
        return Ok(());
    }
    if is_pi_or_coeditor(pool, protocol_id, current_user.id).await?
        || is_assigned_reviewer(pool, protocol_id, current_user.id).await?
        || is_assigned_vet(pool, protocol_id, current_user.id).await?
    {
        return Ok(());
    }
    Err(AppError::Forbidden("You don't have permission to view this protocol".into()))
}

/// 檢查計畫查看權限（不查 pi_user_id，用於不知道 PI 的情境）
pub async fn require_protocol_related_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    protocol_id: Uuid,
) -> Result<()> {
    if has_protocol_view_all(current_user) {
        return Ok(());
    }
    if has_any_protocol_role(pool, protocol_id, current_user.id).await? {
        return Ok(());
    }
    Err(AppError::Forbidden("You don't have permission to access this protocol".into()))
}

// ============================================
// 動物層級存取控制（C2: 防範動物醫療記錄 IDOR）
// ============================================

/// 取得動物所屬的 protocol_id（若動物不存在回傳 NotFound）
pub async fn get_animal_protocol_id(pool: &PgPool, animal_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar("SELECT protocol_id FROM animals WHERE id = $1")
        .bind(animal_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Animal not found".into()))
}

/// 檢查使用者是否有權限存取特定動物（透過其所屬計畫成員資格）。
/// view_all 角色（IACUC Chair/Staff/Vet/Reviewer）直接放行；其他人需為計畫成員。
pub async fn require_animal_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    animal_id: Uuid,
) -> Result<()> {
    if has_protocol_view_all(current_user) {
        return Ok(());
    }
    let protocol_id = get_animal_protocol_id(pool, animal_id).await?;
    require_protocol_related_access(pool, current_user, protocol_id).await
}

/// 透過觀察記錄 ID 取得 animal_id
pub async fn get_observation_animal_id(pool: &PgPool, observation_id: Uuid) -> Result<Uuid> {
    sqlx::query_scalar("SELECT animal_id FROM animal_observations WHERE id = $1")
        .bind(observation_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Observation not found".into()))
}

/// 透過照護紀錄 ID 取得 animal_id（care_medication_records → observation/surgery → animal）
pub async fn get_care_record_animal_id(pool: &PgPool, care_record_id: Uuid) -> Result<Uuid> {
    let animal_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT COALESCE(
            (SELECT animal_id FROM animal_observations WHERE id = c.record_id LIMIT 1),
            (SELECT animal_id FROM animal_surgeries WHERE id = c.record_id LIMIT 1)
        )
        FROM care_medication_records c
        WHERE c.id = $1
        "#,
    )
    .bind(care_record_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    animal_id.ok_or_else(|| AppError::NotFound("Care record not found".into()))
}

/// 檢查使用者是否有權限存取特定計畫（透過 IACUC 編號），用於 PDF 匯出等。
/// admin permission `animal.export.medical` 加上計畫成員才可通過。
pub async fn require_iacuc_protocol_access(
    pool: &PgPool,
    current_user: &CurrentUser,
    iacuc_no: &str,
) -> Result<Uuid> {
    let protocol_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM protocols WHERE protocol_no = $1",
    )
    .bind(iacuc_no)
    .fetch_optional(pool)
    .await?;

    let protocol_id = protocol_id
        .ok_or_else(|| AppError::NotFound(format!("Protocol '{}' not found", iacuc_no)))?;

    if has_protocol_view_all(current_user) {
        return Ok(protocol_id);
    }
    require_protocol_related_access(pool, current_user, protocol_id).await?;
    Ok(protocol_id)
}
