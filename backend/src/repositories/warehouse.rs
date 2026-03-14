use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppError, Result};

/// 依 id 查詢倉庫名稱（在 document/crud.rs 中出現 3 次）
pub async fn find_warehouse_name_by_id(pool: &PgPool, id: Uuid) -> Result<Option<String>> {
    let name = sqlx::query_scalar("SELECT name FROM warehouses WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(name)
}
