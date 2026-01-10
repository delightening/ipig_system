use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory, ProductQuery,
        ProductUomConversion, ProductWithUom, UpdateProductRequest,
    },
    AppError, Result,
};

pub struct ProductService;

impl ProductService {
    /// 建立產品（SKU 自動生成）
    pub async fn create(pool: &PgPool, req: &CreateProductRequest) -> Result<ProductWithUom> {
        // 使用預設分類碼（如未提供）
        let category_code = req.category_code.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| "GEN".to_string());
        let subcategory_code = req.subcategory_code.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| "OTH".to_string());
        
        // 生成 SKU
        let sequence = Self::get_next_sequence(pool, &category_code, &subcategory_code).await?;
        let sku = format!("{}-{}-{:03}", category_code, subcategory_code, sequence);

        // 檢查 SKU 是否已存在（以防併發）
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)"
        )
        .bind(&sku)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(AppError::Conflict("SKU already exists".to_string()));
        }

        // 查詢類別名稱
        let category_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sku_categories WHERE code = $1"
        )
        .bind(&category_code)
        .fetch_optional(pool)
        .await?;

        let subcategory_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sku_subcategories WHERE category_code = $1 AND code = $2"
        )
        .bind(&category_code)
        .bind(&subcategory_code)
        .fetch_optional(pool)
        .await?;

        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (
                id, sku, name, spec, category_code, subcategory_code, base_uom,
                pack_unit, pack_qty, track_batch, track_expiry, default_expiry_days,
                safety_stock, safety_stock_uom, reorder_point, reorder_point_uom,
                barcode, image_url, license_no, storage_condition, tags, remark,
                is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&sku)
        .bind(&req.name)
        .bind(&req.spec)
        .bind(&category_code)
        .bind(&subcategory_code)
        .bind(&req.base_uom)
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
        .bind(&req.remark)
        .fetch_one(pool)
        .await?;

        // 建立單位換算
        let mut uom_conversions = Vec::new();
        for conv in &req.uom_conversions {
            let uom = sqlx::query_as::<_, ProductUomConversion>(
                r#"
                INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#
            )
            .bind(Uuid::new_v4())
            .bind(product.id)
            .bind(&conv.uom)
            .bind(conv.factor_to_base)
            .fetch_one(pool)
            .await?;
            uom_conversions.push(uom);
        }

        Ok(ProductWithUom { product, uom_conversions, category_name, subcategory_name })
    }

    /// 取得下一個 SKU 流水號
    async fn get_next_sequence(pool: &PgPool, category_code: &str, subcategory_code: &str) -> Result<i32> {
        let pattern = format!("{}-{}-___", category_code, subcategory_code);
        let max_seq: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT MAX(CAST(SUBSTRING(sku FROM '\d{3}$') AS INTEGER))
            FROM products
            WHERE sku LIKE $1
            "#
        )
        .bind(&pattern.replace("___", "___"))
        .fetch_optional(pool)
        .await?
        .flatten();

        Ok(max_seq.unwrap_or(0) + 1)
    }

    /// 取得產品列表
    pub async fn list(pool: &PgPool, query: &ProductQuery) -> Result<Vec<Product>> {
        let mut sql = String::from("SELECT * FROM products WHERE 1=1");
        let mut params: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if query.keyword.is_some() {
            sql.push_str(&format!(" AND (sku ILIKE ${0} OR name ILIKE ${0})", param_idx));
            param_idx += 1;
        }
        if query.category_id.is_some() {
            sql.push_str(&format!(" AND category_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.is_active.is_some() {
            sql.push_str(&format!(" AND is_active = ${}", param_idx));
        }
        sql.push_str(" ORDER BY sku");

        // 由於 SQLx 的動態查詢限制，這裡使用分支處理
        let products = if let Some(ref kw) = query.keyword {
            let pattern = format!("%{}%", kw);
            if let Some(category_id) = query.category_id {
                if let Some(is_active) = query.is_active {
                    sqlx::query_as::<_, Product>(
                        "SELECT * FROM products WHERE (sku ILIKE $1 OR name ILIKE $1) AND category_id = $2 AND is_active = $3 ORDER BY sku"
                    )
                    .bind(&pattern)
                    .bind(category_id)
                    .bind(is_active)
                    .fetch_all(pool)
                    .await?
                } else {
                    sqlx::query_as::<_, Product>(
                        "SELECT * FROM products WHERE (sku ILIKE $1 OR name ILIKE $1) AND category_id = $2 ORDER BY sku"
                    )
                    .bind(&pattern)
                    .bind(category_id)
                    .fetch_all(pool)
                    .await?
                }
            } else if let Some(is_active) = query.is_active {
                sqlx::query_as::<_, Product>(
                    "SELECT * FROM products WHERE (sku ILIKE $1 OR name ILIKE $1) AND is_active = $2 ORDER BY sku"
                )
                .bind(&pattern)
                .bind(is_active)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, Product>(
                    "SELECT * FROM products WHERE (sku ILIKE $1 OR name ILIKE $1) ORDER BY sku"
                )
                .bind(&pattern)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(category_id) = query.category_id {
            if let Some(is_active) = query.is_active {
                sqlx::query_as::<_, Product>(
                    "SELECT * FROM products WHERE category_id = $1 AND is_active = $2 ORDER BY sku"
                )
                .bind(category_id)
                .bind(is_active)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, Product>(
                    "SELECT * FROM products WHERE category_id = $1 ORDER BY sku"
                )
                .bind(category_id)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(is_active) = query.is_active {
            sqlx::query_as::<_, Product>(
                "SELECT * FROM products WHERE is_active = $1 ORDER BY sku"
            )
            .bind(is_active)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY sku")
                .fetch_all(pool)
                .await?
        };

        Ok(products)
    }

    /// 取得單一產品
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProductWithUom> {
        let product = sqlx::query_as::<_, Product>(
            "SELECT * FROM products WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        let uom_conversions = sqlx::query_as::<_, ProductUomConversion>(
            "SELECT * FROM product_uom_conversions WHERE product_id = $1 ORDER BY uom"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        // 查詢類別名稱
        let category_name: Option<String> = if let Some(ref cat_code) = product.category_code {
            sqlx::query_scalar("SELECT name FROM sku_categories WHERE code = $1")
                .bind(cat_code)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        let subcategory_name: Option<String> = if let (Some(ref cat_code), Some(ref sub_code)) = (&product.category_code, &product.subcategory_code) {
            sqlx::query_scalar("SELECT name FROM sku_subcategories WHERE category_code = $1 AND code = $2")
                .bind(cat_code)
                .bind(sub_code)
                .fetch_optional(pool)
                .await?
        } else {
            None
        };

        Ok(ProductWithUom { product, uom_conversions, category_name, subcategory_name })
    }

    /// 更新產品
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateProductRequest) -> Result<ProductWithUom> {
        let _product = sqlx::query_as::<_, Product>(
            r#"
            UPDATE products SET
                name = COALESCE($1, name),
                spec = COALESCE($2, spec),
                category_code = COALESCE($3, category_code),
                subcategory_code = COALESCE($4, subcategory_code),
                pack_unit = COALESCE($5, pack_unit),
                pack_qty = COALESCE($6, pack_qty),
                track_batch = COALESCE($7, track_batch),
                track_expiry = COALESCE($8, track_expiry),
                default_expiry_days = COALESCE($9, default_expiry_days),
                safety_stock = COALESCE($10, safety_stock),
                safety_stock_uom = COALESCE($11, safety_stock_uom),
                reorder_point = COALESCE($12, reorder_point),
                reorder_point_uom = COALESCE($13, reorder_point_uom),
                barcode = COALESCE($14, barcode),
                image_url = COALESCE($15, image_url),
                license_no = COALESCE($16, license_no),
                storage_condition = COALESCE($17, storage_condition),
                tags = COALESCE($18, tags),
                status = COALESCE($19, status),
                remark = COALESCE($20, remark),
                is_active = COALESCE($21, is_active),
                updated_at = NOW()
            WHERE id = $22
            RETURNING *
            "#
        )
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
        .await?
        .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        // 如果要更新單位換算
        if let Some(ref conversions) = req.uom_conversions {
            // 刪除現有換算
            sqlx::query("DELETE FROM product_uom_conversions WHERE product_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            // 建立新換算
            for conv in conversions {
                sqlx::query(
                    "INSERT INTO product_uom_conversions (id, product_id, uom, factor_to_base) VALUES ($1, $2, $3, $4)"
                )
                .bind(Uuid::new_v4())
                .bind(id)
                .bind(&conv.uom)
                .bind(conv.factor_to_base)
                .execute(pool)
                .await?;
            }
        }

        Self::get_by_id(pool, id).await
    }

    /// 刪除產品（軟刪除）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE products SET is_active = false, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Product not found".to_string()));
        }

        Ok(())
    }

    /// 取得產品類別列表
    pub async fn list_categories(pool: &PgPool) -> Result<Vec<ProductCategory>> {
        let categories = sqlx::query_as::<_, ProductCategory>(
            "SELECT * FROM product_categories ORDER BY code"
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    /// 建立產品類別
    pub async fn create_category(pool: &PgPool, req: &CreateCategoryRequest) -> Result<ProductCategory> {
        let category = sqlx::query_as::<_, ProductCategory>(
            r#"
            INSERT INTO product_categories (id, code, name, parent_id, is_active, created_at)
            VALUES ($1, $2, $3, $4, true, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.code)
        .bind(&req.name)
        .bind(req.parent_id)
        .fetch_one(pool)
        .await?;

        Ok(category)
    }
}
