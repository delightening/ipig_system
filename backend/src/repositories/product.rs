use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{Product, ProductUomConversion},
    AppError, Result,
};

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

/// 查詢子分類名稱。
pub async fn find_subcategory_name(
    pool: &PgPool,
    category_code: &str,
    subcategory_code: &str,
) -> Result<Option<String>> {
    let name: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sku_subcategories WHERE category_code = $1 AND code = $2",
    )
    .bind(category_code)
    .bind(subcategory_code)
    .fetch_optional(pool)
    .await?;
    Ok(name)
}

/// 依名稱+規格檢查產品是否已存在（匯入重複檢測用）。
pub async fn exists_product_by_name_spec(
    pool: &PgPool,
    name: &str,
    spec: Option<&str>,
) -> Result<bool> {
    let exists: bool = if spec.is_none() {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM products
                WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
                  AND (spec IS NULL OR TRIM(COALESCE(spec, '')) = '')
            )
            "#,
        )
        .bind(name)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM products
                WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
                  AND LOWER(TRIM(COALESCE(spec, ''))) = LOWER(TRIM($2))
            )
            "#,
        )
        .bind(name)
        .bind(spec.unwrap_or(""))
        .fetch_one(pool)
        .await?
    };
    Ok(exists)
}

/// 依名稱+規格查詢既有產品的 id 與 sku（匯入重複預檢用）。
#[derive(sqlx::FromRow)]
pub struct ExistingProductRef {
    pub id: Uuid,
    pub sku: String,
}

pub async fn find_product_by_name_spec(
    pool: &PgPool,
    name: &str,
    spec: Option<&str>,
) -> Result<Option<ExistingProductRef>> {
    let row = if spec.is_none() {
        sqlx::query_as::<_, ExistingProductRef>(
            r#"
            SELECT id, sku FROM products
            WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
              AND (spec IS NULL OR TRIM(COALESCE(spec, '')) = '')
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as::<_, ExistingProductRef>(
            r#"
            SELECT id, sku FROM products
            WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
              AND LOWER(TRIM(COALESCE(spec, ''))) = LOWER(TRIM($2))
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(spec.unwrap_or(""))
        .fetch_optional(pool)
        .await?
    };
    Ok(row)
}

/// 查詢產品目前的分類代碼。
pub async fn find_product_category_codes(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<(String, String)>> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT COALESCE(category_code, 'GEN'), COALESCE(subcategory_code, 'OTH') FROM products WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 依 ID 查詢單一產品。
pub async fn find_product_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Product>> {
    let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(product)
}

/// 查詢產品的單位換算列表。
pub async fn list_uom_conversions(
    pool: &PgPool,
    product_id: Uuid,
) -> Result<Vec<ProductUomConversion>> {
    let conversions = sqlx::query_as::<_, ProductUomConversion>(
        "SELECT * FROM product_uom_conversions WHERE product_id = $1 ORDER BY uom",
    )
    .bind(product_id)
    .fetch_all(pool)
    .await?;
    Ok(conversions)
}

/// 刪除產品的所有單位換算。
pub async fn delete_uom_conversions(pool: &PgPool, product_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM product_uom_conversions WHERE product_id = $1")
        .bind(product_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 插入單筆單位換算。
pub async fn insert_uom_conversion(
    pool: &PgPool,
    product_id: Uuid,
    uom: &str,
    factor_to_base: rust_decimal::Decimal,
) -> Result<ProductUomConversion> {
    let conv = sqlx::query_as::<_, ProductUomConversion>(
        r#"
        INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(product_id)
    .bind(uom)
    .bind(factor_to_base)
    .fetch_one(pool)
    .await?;
    Ok(conv)
}
