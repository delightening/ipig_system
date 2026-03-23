use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{Product, ProductQuery, ProductUomConversion, UpdateProductRequest},
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

/// 更新產品欄位（SKU 為 None 時保留原值）。
pub async fn update_product(
    pool: &PgPool,
    id: Uuid,
    sku: Option<&str>,
    req: &UpdateProductRequest,
) -> Result<Option<Product>> {
    let product = sqlx::query_as::<_, Product>(UPDATE_PRODUCT_SQL)
        .bind(sku)
        .bind(&req.name)
        .bind(&req.spec)
        .bind(&req.category_code)
        .bind(&req.subcategory_code)
        .bind(&req.pack_unit)
        .bind(req.pack_qty)
        .bind(req.track_batch)
        .bind(req.track_expiry)
        .bind(req.default_expiry_days)
        .bind(req.safety_stock)
        .bind(&req.safety_stock_uom)
        .bind(req.reorder_point)
        .bind(&req.reorder_point_uom)
        .bind(&req.barcode)
        .bind(&req.image_url)
        .bind(&req.license_no)
        .bind(&req.storage_condition)
        .bind(&req.tags)
        .bind(&req.status)
        .bind(&req.remark)
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(product)
}

const UPDATE_PRODUCT_SQL: &str = r#"
UPDATE products SET
    sku = COALESCE($1, sku),
    name = COALESCE($2, name),
    spec = COALESCE($3, spec),
    category_code = COALESCE($4, category_code),
    subcategory_code = COALESCE($5, subcategory_code),
    pack_unit = COALESCE($6, pack_unit),
    pack_qty = COALESCE($7, pack_qty),
    track_batch = COALESCE($8, track_batch),
    track_expiry = COALESCE($9, track_expiry),
    default_expiry_days = COALESCE($10, default_expiry_days),
    safety_stock = COALESCE($11, safety_stock),
    safety_stock_uom = COALESCE($12, safety_stock_uom),
    reorder_point = COALESCE($13, reorder_point),
    reorder_point_uom = COALESCE($14, reorder_point_uom),
    barcode = COALESCE($15, barcode),
    image_url = COALESCE($16, image_url),
    license_no = COALESCE($17, license_no),
    storage_condition = COALESCE($18, storage_condition),
    tags = COALESCE($19, tags),
    status = COALESCE($20, status),
    remark = COALESCE($21, remark),
    is_active = COALESCE($22, is_active),
    updated_at = NOW()
WHERE id = $23
RETURNING *
"#;

/// 依查詢條件列出產品（動態 WHERE + 參數綁定）
pub async fn list_products(pool: &PgPool, query: &ProductQuery) -> Result<Vec<Product>> {
    let sql = build_list_sql(query);
    let q = bind_list_params(sqlx::query_as::<_, Product>(&sql), query);
    let products = q.fetch_all(pool).await?;
    Ok(products)
}

/// 根據查詢條件建構產品列表 SQL（參數化 $1, $2...，無注入風險）
fn build_list_sql(query: &ProductQuery) -> String {
    let mut conditions = Vec::new();
    let mut idx: i32 = 1;
    if query.keyword.is_some() {
        conditions.push(std::format!("(sku ILIKE ${idx} OR name ILIKE ${idx})"));
        idx += 1;
    }
    if query.category_id.is_some() {
        conditions.push(std::format!("category_id = ${idx}"));
        idx += 1;
    }
    if query.category_code.is_some() {
        conditions.push(std::format!("category_code = ${idx}"));
        idx += 1;
    }
    if query.subcategory_code.is_some() {
        conditions.push(std::format!("subcategory_code = ${idx}"));
        idx += 1;
    }
    if query.status.is_some() {
        conditions.push(std::format!("status = ${idx}"));
        idx += 1;
    }
    if query.is_active.is_some() {
        conditions.push(std::format!("is_active = ${idx}"));
    }
    let where_clause = if conditions.is_empty() {
        "1=1".to_string()
    } else {
        conditions.join(" AND ")
    };
    // Dynamic SQL with bind params ($1, $2...) — no injection risk
    ["SELECT * FROM products WHERE ", &where_clause, " ORDER BY sku"].concat()
}

/// 依查詢條件綁定參數至 SQL query
fn bind_list_params<'q>(
    mut q: sqlx::query::QueryAs<'q, sqlx::Postgres, Product, sqlx::postgres::PgArguments>,
    query: &'q ProductQuery,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, Product, sqlx::postgres::PgArguments> {
    if let Some(ref kw) = query.keyword {
        q = q.bind(std::format!("%{kw}%"));
    }
    if let Some(category_id) = query.category_id {
        q = q.bind(category_id);
    }
    if let Some(ref c) = query.category_code {
        q = q.bind(c.as_str());
    }
    if let Some(ref s) = query.subcategory_code {
        q = q.bind(s.as_str());
    }
    if let Some(ref s) = query.status {
        q = q.bind(s.as_str());
    }
    if let Some(is_active) = query.is_active {
        q = q.bind(is_active);
    }
    q
}
