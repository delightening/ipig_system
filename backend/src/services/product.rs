use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use crate::{
    models::{
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory, ProductImportErrorDetail,
        ProductImportCheckResult, ProductImportDuplicateItem, ProductImportPreviewResult,
        ProductImportPreviewRow, ProductImportResult, ProductImportRow,
        ProductQuery, ProductUomConversion, ProductWithUom, UpdateProductRequest,
    },
    AppError, Result,
};

/// 允許的產品狀態值（與 DB chk_product_status 一致）
const ALLOWED_STATUSES: [&str; 3] = ["active", "inactive", "discontinued"];

/// 格式化產品 SKU（分類-子分類-三位流水號），供建立與單元測試使用。
pub fn format_product_sku(category_code: &str, subcategory_code: &str, sequence: i32) -> String {
    format!("{}-{}-{:03}", category_code, subcategory_code, sequence)
}

/// 驗證並正規化產品狀態，與 DB chk_product_status 一致。
/// 回傳 `Ok(正規化字串)` 或 `Err(錯誤訊息)`，便於呼叫端轉成 `AppError::Validation`。
pub fn validate_product_status(status: &str) -> std::result::Result<String, String> {
    let s = status.trim().to_lowercase();
    if ALLOWED_STATUSES.contains(&s.as_str()) {
        Ok(s)
    } else {
        Err(format!(
            "status 必須為: {}",
            ALLOWED_STATUSES.join(", ")
        ))
    }
}

pub struct ProductService;

impl ProductService {
    /// 建立產品（SKU 自動生成）
    pub async fn create(pool: &PgPool, req: &CreateProductRequest) -> Result<ProductWithUom> {
        // 使用預設分類碼（如未提供）
        let category_code = req
            .category_code
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "GEN".to_string());
        let subcategory_code = req
            .subcategory_code
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "OTH".to_string());

        // SKU：若請求有提供且非空則使用（並檢查唯一），否則自動生成
        let sku = if let Some(s) = req.sku.as_deref().map(str::trim) {
            if s.is_empty() {
                let sequence =
                    Self::get_next_sequence(pool, &category_code, &subcategory_code).await?;
                format_product_sku(&category_code, &subcategory_code, sequence)
            } else {
                let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)")
                    .bind(s)
                    .fetch_one(pool)
                    .await?;
                if exists {
                    return Err(AppError::Conflict("SKU already exists".to_string()));
                }
                s.to_string()
            }
        } else {
            let sequence =
                Self::get_next_sequence(pool, &category_code, &subcategory_code).await?;
            format_product_sku(&category_code, &subcategory_code, sequence)
        };

        // 查詢類別名稱
        let category_name: Option<String> =
            sqlx::query_scalar("SELECT name FROM sku_categories WHERE code = $1")
                .bind(&category_code)
                .fetch_optional(pool)
                .await?;

        let subcategory_name: Option<String> = sqlx::query_scalar(
            "SELECT name FROM sku_subcategories WHERE category_code = $1 AND code = $2",
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
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(product.id)
            .bind(&conv.uom)
            .bind(conv.factor_to_base)
            .fetch_one(pool)
            .await?;
            uom_conversions.push(uom);
        }

        Ok(ProductWithUom {
            product,
            uom_conversions,
            category_name,
            subcategory_name,
        })
    }

    /// 取得下一個 SKU 流水號
    async fn get_next_sequence(
        pool: &PgPool,
        category_code: &str,
        subcategory_code: &str,
    ) -> Result<i32> {
        let pattern = format!("{}-{}-___", category_code, subcategory_code);
        let max_seq: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT MAX(CAST(SUBSTRING(sku FROM '\d{3}$') AS INTEGER))
            FROM products
            WHERE sku LIKE $1
            "#,
        )
        .bind(&pattern)
        .fetch_optional(pool)
        .await?
        .flatten();

        Ok(max_seq.unwrap_or(0) + 1)
    }

    /// 取得產品列表（支援 keyword、category_code、subcategory_code、status 篩選）
    pub async fn list(pool: &PgPool, query: &ProductQuery) -> Result<Vec<Product>> {
        let mut conditions = Vec::new();
        let mut param_idx: i32 = 1;

        if query.keyword.is_some() {
            conditions.push(format!("(sku ILIKE ${} OR name ILIKE ${})", param_idx, param_idx));
            param_idx += 1;
        }
        if query.category_id.is_some() {
            conditions.push(format!("category_id = ${}", param_idx));
            param_idx += 1;
        }
        if query.category_code.is_some() {
            conditions.push(format!("category_code = ${}", param_idx));
            param_idx += 1;
        }
        if query.subcategory_code.is_some() {
            conditions.push(format!("subcategory_code = ${}", param_idx));
            param_idx += 1;
        }
        if query.status.is_some() {
            conditions.push(format!("status = ${}", param_idx));
            param_idx += 1;
        }
        if query.is_active.is_some() {
            conditions.push(format!("is_active = ${}", param_idx));
        }

        let where_clause = if conditions.is_empty() {
            String::from("1=1")
        } else {
            conditions.join(" AND ")
        };
        let sql = format!("SELECT * FROM products WHERE {} ORDER BY sku", where_clause);

        let mut q = sqlx::query_as::<_, Product>(&sql);
        if let Some(ref kw) = query.keyword {
            q = q.bind(format!("%{}%", kw));
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

        let products = q.fetch_all(pool).await?;
        Ok(products)
    }

    /// 取得單一產品
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<ProductWithUom> {
        let product = sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Product not found".to_string()))?;

        let uom_conversions = sqlx::query_as::<_, ProductUomConversion>(
            "SELECT * FROM product_uom_conversions WHERE product_id = $1 ORDER BY uom",
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

        let subcategory_name: Option<String> = if let (Some(ref cat_code), Some(ref sub_code)) =
            (&product.category_code, &product.subcategory_code)
        {
            sqlx::query_scalar(
                "SELECT name FROM sku_subcategories WHERE category_code = $1 AND code = $2",
            )
            .bind(cat_code)
            .bind(sub_code)
            .fetch_optional(pool)
            .await?
        } else {
            None
        };

        Ok(ProductWithUom {
            product,
            uom_conversions,
            category_name,
            subcategory_name,
        })
    }

    /// 更新產品
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<ProductWithUom> {
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
            "#,
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

    /// 僅更新產品狀態（啟用/停用/停產）
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &str,
    ) -> Result<ProductWithUom> {
        let status = validate_product_status(status)
            .map_err(|msg| AppError::Validation(msg))?;
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

    /// 刪除產品（軟刪除）
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

    /// 取得產品類別列表
    pub async fn list_categories(pool: &PgPool) -> Result<Vec<ProductCategory>> {
        let categories =
            sqlx::query_as::<_, ProductCategory>("SELECT * FROM product_categories ORDER BY code")
                .fetch_all(pool)
                .await?;

        Ok(categories)
    }

    /// 建立產品類別
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

    // ============================================
    // 產品匯入
    // ============================================

    /// 匯入預覽：解析檔案並回傳列資料，供前端「依序設定 SKU」使用（不寫入 DB）
    pub fn preview_import(file_data: &[u8], file_name: &str) -> Result<ProductImportPreviewResult> {
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let (rows, has_sku_column) = if is_excel {
            Self::parse_product_excel(file_data)?
        } else {
            Self::parse_product_csv(file_data)?
        };

        let preview_rows: Vec<ProductImportPreviewRow> = rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| {
                let row_number = (index + 2) as i32; // 1-based 且跳過標題列
                let safety_stock = row
                    .safety_stock
                    .and_then(|d| d.to_string().parse::<f64>().ok());
                ProductImportPreviewRow {
                    row: row_number,
                    name: row.name,
                    spec: row.spec,
                    category_code: row.category_code,
                    subcategory_code: row.subcategory_code,
                    base_uom: row.base_uom,
                    track_batch: row.track_batch,
                    track_expiry: row.track_expiry,
                    safety_stock,
                    remark: row.remark,
                }
            })
            .collect();

        Ok(ProductImportPreviewResult {
            rows: preview_rows,
            has_sku_column,
        })
    }

    /// 匯入預檢：檢查名稱+規格是否與既有產品重複（規則一）
    pub async fn check_import_duplicates(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
    ) -> Result<ProductImportCheckResult> {
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let (rows, has_sku_column) = if is_excel {
            Self::parse_product_excel(file_data)?
        } else {
            Self::parse_product_csv(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        let mut duplicates = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            if row.name.trim().is_empty() {
                continue;
            }

            let name_trim = row.name.trim();
            let spec_trim = row.spec.as_deref().map(|s| s.trim()).unwrap_or("");
            let spec_normalized = if spec_trim.is_empty() { None } else { Some(spec_trim.to_string()) };

            #[derive(sqlx::FromRow)]
            struct ExistingRow {
                id: Uuid,
                sku: String,
            }

            let existing: Option<ExistingRow> = if spec_normalized.is_none() {
                sqlx::query_as(
                    r#"
                    SELECT id, sku FROM products
                    WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
                      AND (spec IS NULL OR TRIM(COALESCE(spec, '')) = '')
                    LIMIT 1
                    "#,
                )
                .bind(name_trim)
                .fetch_optional(pool)
                .await?
            } else {
                sqlx::query_as(
                    r#"
                    SELECT id, sku FROM products
                    WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
                      AND LOWER(TRIM(COALESCE(spec, ''))) = LOWER(TRIM($2))
                    LIMIT 1
                    "#,
                )
                .bind(name_trim)
                .bind(spec_normalized.as_deref().unwrap_or(""))
                .fetch_optional(pool)
                .await?
            };

            if let Some(existing) = existing {
                duplicates.push(ProductImportDuplicateItem {
                    row: row_number,
                    name: row.name.trim().to_string(),
                    spec: row.spec.clone().filter(|s| !s.trim().is_empty()),
                    existing_sku: existing.sku,
                    existing_id: existing.id,
                });
            }
        }

        Ok(ProductImportCheckResult {
            total_rows: rows.len() as i32,
            duplicate_count: duplicates.len() as i32,
            duplicates,
            has_sku_column,
        })
    }

    /// 匯入產品（CSV 或 Excel）
    /// * `skip_duplicates`: 若為 true，名稱+規格與既有產品相同的列將略過不建立
    /// * `regenerate_sku_for_duplicates`: 若為 true，名稱+規格與既有產品相同的列仍會建立，但 SKU 改由系統自動產生（忽略檔案內的 SKU）
    pub async fn import_products(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        skip_duplicates: bool,
        regenerate_sku_for_duplicates: bool,
    ) -> Result<ProductImportResult> {
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let (rows, _has_sku_column) = if is_excel {
            Self::parse_product_excel(file_data)?
        } else {
            Self::parse_product_csv(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            if row.name.trim().is_empty() {
                errors.push(ProductImportErrorDetail {
                    row: row_number,
                    sku: None,
                    error: "名稱為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            if row.base_uom.trim().is_empty() {
                errors.push(ProductImportErrorDetail {
                    row: row_number,
                    sku: None,
                    error: "單位為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 規則一：若 skip_duplicates=true 且名稱+規格已存在，略過此列
            if skip_duplicates {
                let name_trim = row.name.trim();
                let spec_trim = row.spec.as_deref().map(|s| s.trim()).unwrap_or("");
                let spec_normalized = if spec_trim.is_empty() { None } else { Some(spec_trim.to_string()) };

                let exists: bool = if spec_normalized.is_none() {
                    sqlx::query_scalar(
                        r#"
                        SELECT EXISTS(
                            SELECT 1 FROM products
                            WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
                              AND (spec IS NULL OR TRIM(COALESCE(spec, '')) = '')
                        )
                        "#,
                    )
                    .bind(name_trim)
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
                    .bind(name_trim)
                    .bind(spec_normalized.as_deref().unwrap_or(""))
                    .fetch_one(pool)
                    .await?
                };

                if exists {
                    continue; // 略過重複列
                }
            }

            // 是否對本列使用自動產生 SKU（重複且選擇「更改流水號」時）
            let use_auto_sku_for_this_row = if regenerate_sku_for_duplicates {
                let name_trim = row.name.trim();
                let spec_trim = row.spec.as_deref().map(|s| s.trim()).unwrap_or("");
                let spec_normalized = if spec_trim.is_empty() { None } else { Some(spec_trim.to_string()) };
                let exists: bool = if spec_normalized.is_none() {
                    sqlx::query_scalar(
                        r#"
                        SELECT EXISTS(
                            SELECT 1 FROM products
                            WHERE LOWER(TRIM(name)) = LOWER(TRIM($1))
                              AND (spec IS NULL OR TRIM(COALESCE(spec, '')) = '')
                        )
                        "#,
                    )
                    .bind(name_trim)
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
                    .bind(name_trim)
                    .bind(spec_normalized.as_deref().unwrap_or(""))
                    .fetch_one(pool)
                    .await?
                };
                exists
            } else {
                false
            };

            let category_code = row
                .category_code
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("GEN")
                .to_string();
            let subcategory_code = row
                .subcategory_code
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("OTH")
                .to_string();

            let create_req = CreateProductRequest {
                sku: if use_auto_sku_for_this_row {
                    None // 重複列且選擇「更改流水號」：由系統自動產生 SKU
                } else {
                    row.sku.clone().filter(|s| !s.trim().is_empty())
                },
                name: row.name.trim().to_string(),
                spec: row.spec.clone().filter(|s| !s.trim().is_empty()),
                category_code: Some(category_code),
                subcategory_code: Some(subcategory_code),
                base_uom: row.base_uom.trim().to_string(),
                pack_unit: None,
                pack_qty: None,
                track_batch: row.track_batch,
                track_expiry: row.track_expiry,
                default_expiry_days: None,
                safety_stock: row.safety_stock,
                safety_stock_uom: None,
                reorder_point: None,
                reorder_point_uom: None,
                barcode: None,
                image_url: None,
                license_no: None,
                storage_condition: None,
                tags: None,
                remark: row.remark.clone(),
                uom_conversions: vec![],
            };

            match Self::create(pool, &create_req).await {
                Ok(_product) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(ProductImportErrorDetail {
                        row: row_number,
                        sku: None,
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        Ok(ProductImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    /// 解析產品匯入 CSV，回傳 (列資料, 是否含 SKU 欄位)
    fn parse_product_csv(file_data: &[u8]) -> Result<(Vec<ProductImportRow>, bool)> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers = reader
            .headers()
            .map_err(|e| AppError::Validation(format!("CSV 標題列讀取錯誤: {}", e)))?;
        let has_sku_column = headers.len() >= 10
            && headers
                .get(0)
                .map(|h| h.trim().to_uppercase().contains("SKU"))
                .unwrap_or(false);

        let mut rows = Vec::new();
        for (i, result) in reader.records().enumerate() {
            let record = result.map_err(|e| AppError::Validation(format!("CSV 解析錯誤第 {} 行: {}", i + 2, e)))?;
            let (name_idx, spec_idx, cat_idx, subcat_idx, uom_idx, batch_idx, expiry_idx, stock_idx, remark_idx) = if has_sku_column {
                if record.len() < 3 {
                    continue;
                }
                (1, 2, 3, 4, 5, 6, 7, 8, 9)
            } else {
                if record.len() < 2 {
                    continue;
                }
                (0, 1, 2, 3, 4, 5, 6, 7, 8)
            };

            let name = record.get(name_idx).unwrap_or("").to_string();
            if name.trim().is_empty() {
                continue;
            }
            let sku = if has_sku_column {
                record.get(0).and_then(|s| {
                    let t = s.trim();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t.to_string())
                    }
                })
            } else {
                None
            };
            let spec = record.get(spec_idx).filter(|s| !s.trim().is_empty()).map(String::from);
            let category_code = record.get(cat_idx).filter(|s| !s.trim().is_empty()).map(String::from);
            let subcategory_code = record.get(subcat_idx).filter(|s| !s.trim().is_empty()).map(String::from);
            let base_uom = record.get(uom_idx).unwrap_or("PCS").to_string();
            let track_batch = Self::parse_bool(record.get(batch_idx).unwrap_or(""));
            let track_expiry = Self::parse_bool(record.get(expiry_idx).unwrap_or(""));
            let safety_stock = record
                .get(stock_idx)
                .and_then(|s| s.trim().parse::<f64>().ok())
                .and_then(rust_decimal::Decimal::from_f64_retain);
            let remark = record.get(remark_idx).filter(|s| !s.trim().is_empty()).map(String::from);

            rows.push(ProductImportRow {
                sku,
                name,
                spec,
                category_code,
                subcategory_code,
                base_uom,
                track_batch,
                track_expiry,
                safety_stock,
                remark,
            });
        }
        Ok((rows, has_sku_column))
    }

    /// 解析產品匯入 Excel，回傳 (列資料, 是否含 SKU 欄位)
    fn parse_product_excel(file_data: &[u8]) -> Result<(Vec<ProductImportRow>, bool)> {
        let range = {
            let cursor = Cursor::new(file_data);
            if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
                let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
                    AppError::Validation("Excel 檔案中沒有工作表".to_string())
                })?;
                wb.worksheet_range(&sheet_name)
                    .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
            } else {
                let cursor = Cursor::new(file_data);
                let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor).map_err(|_| {
                    AppError::Validation("無法讀取 Excel 檔案，請使用 .xlsx 或 .xls 格式".to_string())
                })?;
                let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
                    AppError::Validation("Excel 檔案中沒有工作表".to_string())
                })?;
                wb.worksheet_range(&sheet_name)
                    .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
            }
        };

        let mut rows = Vec::new();
        let mut iter = range.rows();
        let header_row = iter.next().ok_or_else(|| AppError::Validation("Excel 無標題列".to_string()))?;
        let has_sku_column = header_row.len() >= 10
            && Self::get_cell_string(header_row.get(0)).trim().to_uppercase().contains("SKU");

        let (name_col, spec_col, cat_col, subcat_col, uom_col, batch_col, expiry_col, stock_col, remark_col) = if has_sku_column {
            (1, 2, 3, 4, 5, 6, 7, 8, 9)
        } else {
            (0, 1, 2, 3, 4, 5, 6, 7, 8)
        };

        for row in iter {
            if row.len() < if has_sku_column { 3 } else { 2 } {
                continue;
            }
            let name = Self::get_cell_string(row.get(name_col));
            if name.trim().is_empty() {
                continue;
            }
            let sku = if has_sku_column {
                let s = Self::get_cell_string(row.get(0));
                if s.trim().is_empty() {
                    None
                } else {
                    Some(s.trim().to_string())
                }
            } else {
                None
            };
            let spec = {
                let s = Self::get_cell_string(row.get(spec_col));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let category_code = {
                let s = Self::get_cell_string(row.get(cat_col));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let subcategory_code = {
                let s = Self::get_cell_string(row.get(subcat_col));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let base_uom = Self::get_cell_string(row.get(uom_col));
            let base_uom = if base_uom.is_empty() {
                "PCS".to_string()
            } else {
                base_uom
            };
            let track_batch = Self::parse_bool(&Self::get_cell_string(row.get(batch_col)));
            let track_expiry = Self::parse_bool(&Self::get_cell_string(row.get(expiry_col)));
            let safety_stock = row.get(stock_col).and_then(|c| match c {
                Data::Float(f) => rust_decimal::Decimal::from_f64_retain(*f),
                Data::Int(i) => rust_decimal::Decimal::from_f64_retain(*i as f64),
                _ => None,
            });
            let remark = {
                let s = Self::get_cell_string(row.get(remark_col));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };

            rows.push(ProductImportRow {
                sku,
                name,
                spec,
                category_code,
                subcategory_code,
                base_uom,
                track_batch,
                track_expiry,
                safety_stock,
                remark,
            });
        }
        Ok((rows, has_sku_column))
    }

    fn get_cell_string(cell: Option<&Data>) -> String {
        cell.map(|c| match c {
            Data::String(s) => s.clone(),
            Data::Float(f) => f.to_string(),
            Data::Int(i) => i.to_string(),
            Data::Bool(b) => b.to_string(),
            Data::DateTime(dt) => format!("{:?}", dt),
            _ => String::new(),
        })
        .unwrap_or_default()
    }

    /// 解析匯入檔中的布林欄位（追蹤批號、追蹤效期等），供 CSV/Excel 匯入與單元測試。
    pub(crate) fn parse_bool(s: &str) -> bool {
        let s = s.trim().to_lowercase();
        matches!(s.as_str(), "true" | "1" | "yes" | "是" | "y")
    }

    /// 產生產品匯入模板
    pub fn generate_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let worksheet = workbook.add_worksheet();
        worksheet.set_column_width(0, 16.0)?;  // SKU編碼
        worksheet.set_column_width(1, 25.0)?;
        worksheet.set_column_width(2, 20.0)?;
        worksheet.set_column_width(3, 12.0)?;
        worksheet.set_column_width(4, 12.0)?;
        worksheet.set_column_width(5, 10.0)?;
        worksheet.set_column_width(6, 12.0)?;
        worksheet.set_column_width(7, 12.0)?;
        worksheet.set_column_width(8, 12.0)?;
        worksheet.set_column_width(9, 30.0)?;

        worksheet.write_string_with_format(0, 0, "SKU編碼", &header_format)?;
        worksheet.write_string_with_format(0, 1, "名稱*", &header_format)?;
        worksheet.write_string_with_format(0, 2, "規格", &header_format)?;
        worksheet.write_string_with_format(0, 3, "品類代碼", &header_format)?;
        worksheet.write_string_with_format(0, 4, "子類代碼", &header_format)?;
        worksheet.write_string_with_format(0, 5, "單位*", &header_format)?;
        worksheet.write_string_with_format(0, 6, "追蹤批號", &header_format)?;
        worksheet.write_string_with_format(0, 7, "追蹤效期", &header_format)?;
        worksheet.write_string_with_format(0, 8, "安全庫存", &header_format)?;
        worksheet.write_string_with_format(0, 9, "備註", &header_format)?;

        worksheet.write_string(1, 0, "DRG-OTH-001")?;
        worksheet.write_string(1, 1, "範例產品")?;
        worksheet.write_string(1, 2, "100ml/瓶")?;
        worksheet.write_string(1, 3, "DRG")?;
        worksheet.write_string(1, 4, "OTH")?;
        worksheet.write_string(1, 5, "PCS")?;
        worksheet.write_string(1, 6, "false")?;
        worksheet.write_string(1, 7, "false")?;
        worksheet.write_string(1, 8, "10")?;
        worksheet.write_string(1, 9, "")?;

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }
}

#[cfg(test)]
mod tests {
    use super::{format_product_sku, validate_product_status, ProductService};

    // --- format_product_sku ---
    #[test]
    fn test_format_product_sku_normal() {
        assert_eq!(
            format_product_sku("DRG", "OTH", 1),
            "DRG-OTH-001"
        );
        assert_eq!(
            format_product_sku("GEN", "OTH", 42),
            "GEN-OTH-042"
        );
    }

    #[test]
    fn test_format_product_sku_zero_pad() {
        assert_eq!(format_product_sku("CAT", "SUB", 0), "CAT-SUB-000");
        assert_eq!(format_product_sku("A", "B", 999), "A-B-999");
    }

    #[test]
    fn test_format_product_sku_large_sequence() {
        assert_eq!(format_product_sku("X", "Y", 1000), "X-Y-1000");
    }

    // --- validate_product_status ---
    #[test]
    fn test_validate_product_status_allowed() {
        assert_eq!(validate_product_status("active").unwrap(), "active");
        assert_eq!(validate_product_status("inactive").unwrap(), "inactive");
        assert_eq!(validate_product_status("discontinued").unwrap(), "discontinued");
        assert_eq!(validate_product_status("  ACTIVE  ").unwrap(), "active");
    }

    #[test]
    fn test_validate_product_status_invalid() {
        assert!(validate_product_status("pending").is_err());
        assert!(validate_product_status("").is_err());
        assert!(validate_product_status("ActiveX").is_err());
    }

    #[test]
    fn test_validate_product_status_error_message() {
        let msg = validate_product_status("x").unwrap_err();
        assert!(msg.contains("active"));
        assert!(msg.contains("inactive"));
        assert!(msg.contains("discontinued"));
    }

    // --- parse_bool (ProductService) ---
    #[test]
    fn test_parse_bool_true_variants() {
        assert!(ProductService::parse_bool("true"));
        assert!(ProductService::parse_bool("1"));
        assert!(ProductService::parse_bool("yes"));
        assert!(ProductService::parse_bool("是"));
        assert!(ProductService::parse_bool("y"));
        assert!(ProductService::parse_bool("  YES  "));
    }

    #[test]
    fn test_parse_bool_false() {
        assert!(!ProductService::parse_bool("false"));
        assert!(!ProductService::parse_bool("0"));
        assert!(!ProductService::parse_bool("no"));
        assert!(!ProductService::parse_bool(""));
        assert!(!ProductService::parse_bool("n"));
        assert!(!ProductService::parse_bool("other"));
    }
}
