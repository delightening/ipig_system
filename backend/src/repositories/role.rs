use sqlx::PgPool;
use uuid::Uuid;

use crate::{models::Role, AppError, Result};

/// 依 id 查詢啟用中的角色（出現在 role.rs get/update/delete 共 3 處）
pub async fn find_role_by_id_active(pool: &PgPool, id: Uuid) -> Result<Option<Role>> {
    let role = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE id = $1 AND is_active = true",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(role)
}
