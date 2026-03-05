use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use crate::{
    models::{
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory, ProductImportErrorDetail,
        ProductImportCheckResult, ProductImportDuplicateItem, ProductImportResult, ProductImportRow,
        ProductQuery, ProductUomConversion, ProductWithUom, UpdateProductRequest,
    },
    AppError, Result,
};

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

        // 生成 SKU
        let sequence = Self::get_next_sequence(pool, &category_code, &subcategory_code).await?;
        let sku = format!("{}-{}-{:03}", category_code, subcategory_code, sequence);

        // 檢查 SKU 是否已存在（以防併發）
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products WHERE sku = $1)")
                .bind(&sku)
                .fetch_one(pool)
                .await?;

        if exists {
            return Err(AppError::Conflict("SKU already exists".to_string()));
        }

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

    /// 取得產品列表
    pub async fn list(pool: &PgPool, query: &ProductQuery) -> Result<Vec<Product>> {
        let mut sql = String::from("SELECT * FROM products WHERE 1=1");
        let _params: Vec<String> = Vec::new();
        let mut param_idx = 1;

        if query.keyword.is_some() {
            sql.push_str(&format!(
                " AND (sku ILIKE ${0} OR name ILIKE ${0})",
                param_idx
            ));
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
                    "SELECT * FROM products WHERE (sku ILIKE $1 OR name ILIKE $1) ORDER BY sku",
                )
                .bind(&pattern)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(category_id) = query.category_id {
            if let Some(is_active) = query.is_active {
                sqlx::query_as::<_, Product>(
                    "SELECT * FROM products WHERE category_id = $1 AND is_active = $2 ORDER BY sku",
                )
                .bind(category_id)
                .bind(is_active)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, Product>(
                    "SELECT * FROM products WHERE category_id = $1 ORDER BY sku",
                )
                .bind(category_id)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(is_active) = query.is_active {
            sqlx::query_as::<_, Product>("SELECT * FROM products WHERE is_active = $1 ORDER BY sku")
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

        let rows = if is_excel {
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
        })
    }

    /// 匯入產品（CSV 或 Excel）
    /// * `skip_duplicates`: 若為 true，名稱+規格與既有產品相同的列將略過不建立
    pub async fn import_products(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        skip_duplicates: bool,
    ) -> Result<ProductImportResult> {
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let rows = if is_excel {
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

    fn parse_product_csv(file_data: &[u8]) -> Result<Vec<ProductImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let mut rows = Vec::new();
        for (i, result) in reader.records().enumerate() {
            let record = result.map_err(|e| AppError::Validation(format!("CSV 解析錯誤第 {} 行: {}", i + 2, e)))?;
            if record.len() < 2 {
                continue;
            }
            let name = record.get(0).unwrap_or("").to_string();
            if name.trim().is_empty() {
                continue;
            }
            let spec = record.get(1).filter(|s| !s.trim().is_empty()).map(String::from);
            let category_code = record.get(2).filter(|s| !s.trim().is_empty()).map(String::from);
            let subcategory_code = record.get(3).filter(|s| !s.trim().is_empty()).map(String::from);
            let base_uom = record.get(4).unwrap_or("PCS").to_string();
            let track_batch = Self::parse_bool(record.get(5).unwrap_or(""));
            let track_expiry = Self::parse_bool(record.get(6).unwrap_or(""));
            let safety_stock = record
                .get(7)
                .and_then(|s| s.trim().parse::<f64>().ok())
                .and_then(rust_decimal::Decimal::from_f64_retain);
            let remark = record.get(8).filter(|s| !s.trim().is_empty()).map(String::from);

            rows.push(ProductImportRow {
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
        Ok(rows)
    }

    fn parse_product_excel(file_data: &[u8]) -> Result<Vec<ProductImportRow>> {
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
        iter.next(); // 跳過標題

        for row in iter {
            if row.len() < 2 {
                continue;
            }
            let name = Self::get_cell_string(row.first());
            if name.trim().is_empty() {
                continue;
            }
            let spec = {
                let s = Self::get_cell_string(row.get(1));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let category_code = {
                let s = Self::get_cell_string(row.get(2));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let subcategory_code = {
                let s = Self::get_cell_string(row.get(3));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let base_uom = Self::get_cell_string(row.get(4));
            let base_uom = if base_uom.is_empty() {
                "PCS".to_string()
            } else {
                base_uom
            };
            let track_batch = Self::parse_bool(&Self::get_cell_string(row.get(5)));
            let track_expiry = Self::parse_bool(&Self::get_cell_string(row.get(6)));
            let safety_stock = row.get(7).and_then(|c| match c {
                Data::Float(f) => rust_decimal::Decimal::from_f64_retain(*f),
                Data::Int(i) => rust_decimal::Decimal::from_f64_retain(*i as f64),
                _ => None,
            });
            let remark = {
                let s = Self::get_cell_string(row.get(8));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };

            rows.push(ProductImportRow {
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
        Ok(rows)
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

    fn parse_bool(s: &str) -> bool {
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
        worksheet.set_column_width(0, 25.0)?;
        worksheet.set_column_width(1, 20.0)?;
        worksheet.set_column_width(2, 12.0)?;
        worksheet.set_column_width(3, 12.0)?;
        worksheet.set_column_width(4, 10.0)?;
        worksheet.set_column_width(5, 12.0)?;
        worksheet.set_column_width(6, 12.0)?;
        worksheet.set_column_width(7, 12.0)?;
        worksheet.set_column_width(8, 30.0)?;

        worksheet.write_string_with_format(0, 0, "名稱*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "規格", &header_format)?;
        worksheet.write_string_with_format(0, 2, "品類代碼", &header_format)?;
        worksheet.write_string_with_format(0, 3, "子類代碼", &header_format)?;
        worksheet.write_string_with_format(0, 4, "單位*", &header_format)?;
        worksheet.write_string_with_format(0, 5, "追蹤批號", &header_format)?;
        worksheet.write_string_with_format(0, 6, "追蹤效期", &header_format)?;
        worksheet.write_string_with_format(0, 7, "安全庫存", &header_format)?;
        worksheet.write_string_with_format(0, 8, "備註", &header_format)?;

        worksheet.write_string(1, 0, "範例產品")?;
        worksheet.write_string(1, 1, "100ml/瓶")?;
        worksheet.write_string(1, 2, "DRG")?;
        worksheet.write_string(1, 3, "OTH")?;
        worksheet.write_string(1, 4, "PCS")?;
        worksheet.write_string(1, 5, "false")?;
        worksheet.write_string(1, 6, "false")?;
        worksheet.write_string(1, 7, "10")?;
        worksheet.write_string(1, 8, "")?;

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }
}
