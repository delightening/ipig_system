// 產品 CRUD 操作

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory,
        ProductQuery, ProductWithUom, UpdateProductRequest,
    },
    repositories, AppError, Result,
};

use super::{
    build_product_with_uom, format_product_sku,
    get_next_sequence, insert_product, insert_uom_conversions, resolve_category_codes,
    resolve_sku, sync_uom_conversions, validate_product_status, ProductService,
};

impl ProductService {
    /// 建立產品（SKU 自動生成）。
    pub async fn create(pool: &PgPool, req: &CreateProductRequest) -> Result<ProductWithUom> {
        let (category_code, subcategory_code) = resolve_category_codes(req);
        let sku = resolve_sku(pool, req.sku.as_deref(), &category_code, &subcategory_code).await?;
        let category_name =
            repositories::sku::find_category_name_by_code(pool, &category_code).await?;
        let subcategory_name =
            repositories::product::find_subcategory_name(pool, &category_code, &subcategory_code)
                .await?;
        let product =
            insert_product(pool, &sku, req, &category_code, &subcategory_code).await?;
        let uom_conversions =
            insert_uom_conversions(pool, product.id, &req.uom_conversions).await?;

        Ok(ProductWithUom {
            product,
            uom_conversions,
            category_name,
            subcategory_name,
        })
    }

    /// 取得產品列表（支援 keyword、category_code、subcategory_code、status 篩選）。
    pub async fn list(pool: &PgPool, query: &ProductQuery) -> Result<Vec<Product>> {
        repositories::product::list_products(pool, query).await
    }

    /// 取得單一產品。
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProductWithUom> {
        let product = repositories::product::find_product_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;
        let uom_conversions = repositories::product::list_uom_conversions(pool, id).await?;
        build_product_with_uom(pool, product, uom_conversions).await
    }

    /// 更新產品。
    /// 特例：若目前為 GEN-OTH，且使用者變更品類／子類為非 GEN-OTH，則自動產生新 SKU。
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<ProductWithUom> {
        let current = repositories::product::find_product_category_codes(pool, id).await?;
        let new_sku = Self::resolve_update_sku(pool, &current, req).await?;

        repositories::product::update_product(pool, id, new_sku.as_deref(), req)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        if let Some(ref conversions) = req.uom_conversions {
            sync_uom_conversions(pool, id, conversions).await?;
        }

        Self::get_by_id(pool, id).await
    }

    /// 判斷更新時是否需要重新產生 SKU（GEN-OTH → 其他品類時）。
    async fn resolve_update_sku(
        pool: &PgPool,
        current: &Option<(String, String)>,
        req: &UpdateProductRequest,
    ) -> Result<Option<String>> {
        let (new_cat, new_sub) = (
            req.category_code
                .as_deref()
                .unwrap_or(current.as_ref().map(|c| c.0.as_str()).unwrap_or("GEN")),
            req.subcategory_code
                .as_deref()
                .unwrap_or(current.as_ref().map(|c| c.1.as_str()).unwrap_or("OTH")),
        );
        let is_gen_oth = current
            .as_ref()
            .map(|(c, s)| c.as_str() == "GEN" && s.as_str() == "OTH")
            .unwrap_or(false);

        if is_gen_oth && (new_cat != "GEN" || new_sub != "OTH") {
            let seq = get_next_sequence(pool, new_cat, new_sub).await?;
            Ok(Some(format_product_sku(new_cat, new_sub, seq)))
        } else {
            Ok(None)
        }
    }

    /// 僅更新產品狀態（啟用/停用/停產）。
    pub async fn update_status(pool: &PgPool, id: Uuid, status: &str) -> Result<ProductWithUom> {
        let status = validate_product_status(status).map_err(AppError::Validation)?;
        let is_active = status == "active";
        let rows = sqlx::query(
            "UPDATE products SET status = $1, is_active = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(&status)
        .bind(is_active)
        .bind(id)
        .execute(pool)
        .await?;
        if rows.rows_affected() == 0 {
            return Err(AppError::NotFound("Product not found".to_string()));
        }
        Self::get_by_id(pool, id).await
    }

    /// 刪除產品（軟刪除）。
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE products SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Product not found".to_string()));
        }
        Ok(())
    }

    /// 硬刪除產品（僅在無單據、庫存、藥物選單關聯時允許；僅供 admin 使用）。
    pub async fn hard_delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE id = $1)")
                .bind(id)
                .fetch_one(pool)
                .await?;
        if !exists {
            return Err(AppError::NotFound("Product not found".to_string()));
        }
        Self::check_hard_delete_refs(pool, id).await?;
        repositories::product::delete_uom_conversions(pool, id).await?;
        let result = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Product not found".to_string()));
        }
        Ok(())
    }

    /// 檢查硬刪除時是否有關聯資料。
    async fn check_hard_delete_refs(pool: &PgPool, id: Uuid) -> Result<()> {
        let tables = [
            ("document_lines", "product_id"),
            ("stock_ledger", "product_id"),
            ("inventory_snapshots", "product_id"),
            ("storage_location_inventory", "product_id"),
        ];
        for (table, col) in tables {
            let count: i64 = sqlx::query_scalar(&format!(
                "SELECT COUNT(*) FROM {} WHERE {} = $1",
                table, col
            ))
            .bind(id)
            .fetch_one(pool)
            .await?;
            if count > 0 {
                return Err(AppError::BusinessRule(
                    "此產品已有單據、庫存或藥物選單關聯，無法硬刪除".to_string(),
                ));
            }
        }
        let drug_refs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM treatment_drug_options WHERE erp_product_id = $1",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        if drug_refs > 0 {
            return Err(AppError::BusinessRule(
                "此產品已有單據、庫存或藥物選單關聯，無法硬刪除".to_string(),
            ));
        }
        Ok(())
    }

    /// 取得產品類別列表。
    pub async fn list_categories(pool: &PgPool) -> Result<Vec<ProductCategory>> {
        let categories =
            sqlx::query_as::<_, ProductCategory>("SELECT * FROM product_categories ORDER BY code")
                .fetch_all(pool)
                .await?;
        Ok(categories)
    }

    /// 建立產品類別。
    pub async fn create_category(
        pool: &PgPool,
        req: &CreateCategoryRequest,
    ) -> Result<ProductCategory> {
        let category = sqlx::query_as::<_, ProductCategory>(
            r#"
            INSERT INTO product_categories (id, code, name, parent_id, is_active, created_at)
            VALUES ($1, $2, $3, $4, true, NOW())
            RETURNING *
            "#,
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
