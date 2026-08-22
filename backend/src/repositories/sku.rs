use sqlx::PgPool;

use crate::{AppError, Result};

/// 依 code 查詢 SKU 類別名稱（出現在 product.rs + sku.rs 共 3 處）
pub async fn find_category_name_by_code(pool: &PgPool, code: &str) -> Result<Option<String>> {
    let name = sqlx::query_scalar("SELECT name FROM sku_categories WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;
    Ok(name)
}
