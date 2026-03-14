use sqlx::PgPool;

use crate::{AppError, Result};

/// 依 SKU 檢查產品是否已存在（出現在 product.rs + sku.rs 共 3 處）
pub async fn exists_product_by_sku(pool: &PgPool, sku: &str) -> Result<bool> {
    let exists = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)",
    )
    .bind(sku)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)?;
    Ok(exists)
}
