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
