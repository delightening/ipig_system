use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        CreateCategoryRequest, CreateProductRequest, Product, ProductCategory,
        ProductImportCheckResult, ProductImportDuplicateItem, ProductImportErrorDetail,
        ProductImportPreviewResult, ProductImportResult, ProductQuery,
        ProductUomConversion, ProductWithUom, UpdateProductRequest,
    },
    repositories, AppError, Result,
};

use super::product_parser;

/// 允許的產品狀態值（與 DB chk_product_status 一致）
const ALLOWED_STATUSES: [&str; 3] = ["active", "inactive", "discontinued"];

/// 格式化產品 SKU（分類-子分類-三位流水號），供建立與單元測試使用。
pub fn format_product_sku(category_code: &str, subcategory_code: &str, sequence: i32) -> String {
    format!("{}-{}-{:03}", category_code, subcategory_code, sequence)
}

/// 驗證並正規化產品狀態，與 DB chk_product_status 一致。
pub fn validate_product_status(status: &str) -> std::result::Result<String, String> {
    let s = status.trim().to_lowercase();
    if ALLOWED_STATUSES.contains(&s.as_str()) {
        Ok(s)
    } else {
        Err(format!("status 必須為: {}", ALLOWED_STATUSES.join(", ")))
    }
}

/// 從 request 中取得分類代碼，若未提供則使用預設值。
fn resolve_category_codes(req: &CreateProductRequest) -> (String, String) {
    let cat = req
        .category_code
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "GEN".to_string());
    let sub = req
        .subcategory_code
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "OTH".to_string());
    (cat, sub)
}

/// 解析 SKU：若請求有提供且非空則使用（並檢查唯一），否則自動生成。
async fn resolve_sku(
    pool: &PgPool,
    req_sku: Option<&str>,
    category_code: &str,
    subcategory_code: &str,
) -> Result<String> {
    if let Some(s) = req_sku.map(str::trim) {
        if !s.is_empty() {
            if repositories::product::exists_product_by_sku(pool, s).await? {
                return Err(AppError::Conflict("SKU already exists".to_string()));
            }
            return Ok(s.to_string());
        }
    }
    let sequence = get_next_sequence(pool, category_code, subcategory_code).await?;
    Ok(format_product_sku(category_code, subcategory_code, sequence))
}

/// 取得下一個 SKU 流水號。
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

/// 插入產品記錄至 DB。
async fn insert_product(
    pool: &PgPool,
    sku: &str,
    req: &CreateProductRequest,
    category_code: &str,
    subcategory_code: &str,
) -> Result<Product> {
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
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(sku)
    .bind(&req.name)
    .bind(&req.spec)
    .bind(category_code)
    .bind(subcategory_code)
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
    Ok(product)
}

/// 批次建立單位換算記錄。
async fn insert_uom_conversions(
    pool: &PgPool,
    product_id: Uuid,
    conversions: &[crate::models::UomConversionInput],
) -> Result<Vec<ProductUomConversion>> {
    let mut result = Vec::new();
    for conv in conversions {
        let uom = repositories::product::insert_uom_conversion(
            pool,
            product_id,
            &conv.uom,
            conv.factor_to_base,
        )
        .await?;
        result.push(uom);
    }
    Ok(result)
}

/// 組合產品與分類名稱為 `ProductWithUom`。
async fn build_product_with_uom(
    pool: &PgPool,
    product: Product,
    uom_conversions: Vec<ProductUomConversion>,
) -> Result<ProductWithUom> {
    let category_name = match product.category_code.as_deref() {
        Some(cat_code) => repositories::sku::find_category_name_by_code(pool, cat_code).await?,
        None => None,
    };
    let subcategory_name =
        if let (Some(ref cat_code), Some(ref sub_code)) =
            (&product.category_code, &product.subcategory_code)
        {
            repositories::product::find_subcategory_name(pool, cat_code, sub_code).await?
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


/// 同步單位換算：刪除既有後重新建立。
async fn sync_uom_conversions(
    pool: &PgPool,
    product_id: Uuid,
    conversions: &[crate::models::UomConversionInput],
) -> Result<()> {
    repositories::product::delete_uom_conversions(pool, product_id).await?;
    for conv in conversions {
        repositories::product::insert_uom_conversion(pool, product_id, &conv.uom, conv.factor_to_base)
            .await?;
    }
    Ok(())
}

/// 正規化匯入列的名稱+規格，回傳 (trimmed_name, normalized_spec)。
fn normalize_name_spec(name: &str, spec: Option<&str>) -> (String, Option<String>) {
    let name_trim = name.trim().to_string();
    let spec_trim = spec.map(|s| s.trim()).unwrap_or("");
    let spec_normalized = if spec_trim.is_empty() {
        None
    } else {
        Some(spec_trim.to_string())
    };
    (name_trim, spec_normalized)
}

/// 驗證匯入列的必填欄位，回傳錯誤訊息（若有）。
fn validate_import_row(row: &crate::models::ProductImportRow) -> Option<String> {
    if row.name.trim().is_empty() {
        return Some("名稱為必填欄位".to_string());
    }
    if row.base_uom.trim().is_empty() {
        return Some("單位為必填欄位".to_string());
    }
    None
}

/// 將匯入列轉換為 `CreateProductRequest`。
fn build_import_create_request(
    row: &crate::models::ProductImportRow,
    use_auto_sku: bool,
) -> CreateProductRequest {
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

    CreateProductRequest {
        sku: if use_auto_sku {
            None
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
    }
}

/// 根據查詢條件建構產品列表 SQL。
fn build_list_sql(query: &ProductQuery) -> String {
    let mut conditions = Vec::new();
    let mut idx: i32 = 1;
    if query.keyword.is_some() {
        conditions.push(format!("(sku ILIKE ${idx} OR name ILIKE ${idx})"));
        idx += 1;
    }
    if query.category_id.is_some() {
        conditions.push(format!("category_id = ${idx}"));
        idx += 1;
    }
    if query.category_code.is_some() {
        conditions.push(format!("category_code = ${idx}"));
        idx += 1;
    }
    if query.subcategory_code.is_some() {
        conditions.push(format!("subcategory_code = ${idx}"));
        idx += 1;
    }
    if query.status.is_some() {
        conditions.push(format!("status = ${idx}"));
        idx += 1;
    }
    if query.is_active.is_some() {
        conditions.push(format!("is_active = ${idx}"));
    }
    let where_clause = if conditions.is_empty() {
        "1=1".to_string()
    } else {
        conditions.join(" AND ")
    };
    format!("SELECT * FROM products WHERE {where_clause} ORDER BY sku")
}

/// 依查詢條件綁定參數至 SQL query。
fn bind_list_params<'q>(
    mut q: sqlx::query::QueryAs<'q, sqlx::Postgres, Product, sqlx::postgres::PgArguments>,
    query: &'q ProductQuery,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, Product, sqlx::postgres::PgArguments> {
    if let Some(ref kw) = query.keyword {
        q = q.bind(format!("%{kw}%"));
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

pub struct ProductService;

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
        let sql = build_list_sql(query);
        let q = bind_list_params(sqlx::query_as::<_, Product>(&sql), query);
        let products = q.fetch_all(pool).await?;
        Ok(products)
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

    // ============================================
    // 產品匯入
    // ============================================

    /// 匯入預覽：解析檔案並回傳列資料（不寫入 DB）。
    pub fn preview_import(
        file_data: &[u8],
        file_name: &str,
    ) -> Result<ProductImportPreviewResult> {
        let (rows, has_sku_column) = product_parser::parse_import_file(file_data, file_name)?;
        let preview_rows: Vec<_> = rows
            .into_iter()
            .enumerate()
            .map(|(i, row)| product_parser::to_preview_row(i, row))
            .collect();
        Ok(ProductImportPreviewResult {
            rows: preview_rows,
            has_sku_column,
        })
    }

    /// 匯入預檢：檢查名稱+規格是否與既有產品重複。
    pub async fn check_import_duplicates(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
    ) -> Result<ProductImportCheckResult> {
        let (rows, has_sku_column) = product_parser::parse_import_file(file_data, file_name)?;
        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }
        let duplicates = Self::find_all_duplicates(pool, &rows).await?;
        Ok(ProductImportCheckResult {
            total_rows: rows.len() as i32,
            duplicate_count: duplicates.len() as i32,
            duplicates,
            has_sku_column,
        })
    }

    /// 逐列檢查匯入資料的重複情況。
    async fn find_all_duplicates(
        pool: &PgPool,
        rows: &[crate::models::ProductImportRow],
    ) -> Result<Vec<ProductImportDuplicateItem>> {
        let mut duplicates = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;
            if row.name.trim().is_empty() {
                continue;
            }
            let (name_trim, spec_normalized) = normalize_name_spec(&row.name, row.spec.as_deref());
            let existing = repositories::product::find_product_by_name_spec(
                pool,
                &name_trim,
                spec_normalized.as_deref(),
            )
            .await?;
            if let Some(existing) = existing {
                duplicates.push(ProductImportDuplicateItem {
                    row: row_number,
                    name: name_trim,
                    spec: row.spec.clone().filter(|s| !s.trim().is_empty()),
                    existing_sku: existing.sku,
                    existing_id: existing.id,
                });
            }
        }
        Ok(duplicates)
    }

    /// 匯入產品（CSV 或 Excel）。
    pub async fn import_products(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        skip_duplicates: bool,
        regenerate_sku_for_duplicates: bool,
    ) -> Result<ProductImportResult> {
        let (rows, _) = product_parser::parse_import_file(file_data, file_name)?;
        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;
            match Self::import_single_row(
                pool, row, row_number, skip_duplicates, regenerate_sku_for_duplicates,
            )
            .await
            {
                Ok(true) => success_count += 1,
                Ok(false) => {}  // skipped
                Err(e) => {
                    errors.push(e);
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

    /// 處理單列匯入：回傳 Ok(true) 成功、Ok(false) 略過、Err 失敗。
    async fn import_single_row(
        pool: &PgPool,
        row: &crate::models::ProductImportRow,
        row_number: i32,
        skip_duplicates: bool,
        regenerate_sku_for_duplicates: bool,
    ) -> std::result::Result<bool, ProductImportErrorDetail> {
        if let Some(err_msg) = validate_import_row(row) {
            return Err(ProductImportErrorDetail {
                row: row_number, sku: None, error: err_msg,
            });
        }
        let use_auto_sku = Self::should_use_auto_sku(
            pool, row, skip_duplicates, regenerate_sku_for_duplicates,
        )
        .await
        .map_err(|e| ProductImportErrorDetail {
            row: row_number, sku: None, error: format!("檢查重複失敗: {e}"),
        })?;
        let Some(auto) = use_auto_sku else {
            return Ok(false); // skip_duplicates 略過
        };
        let create_req = build_import_create_request(row, auto);
        Self::create(pool, &create_req).await.map(|_| true).map_err(|e| {
            ProductImportErrorDetail {
                row: row_number, sku: None, error: format!("建立失敗: {e}"),
            }
        })
    }

    /// 判斷匯入列是否應使用自動 SKU。
    /// 回傳 `None` 表示應略過此列（skip_duplicates），
    /// `Some(true)` 表示使用自動 SKU，`Some(false)` 表示使用原始 SKU。
    async fn should_use_auto_sku(
        pool: &PgPool,
        row: &crate::models::ProductImportRow,
        skip_duplicates: bool,
        regenerate_sku_for_duplicates: bool,
    ) -> Result<Option<bool>> {
        let (name_trim, spec_normalized) = normalize_name_spec(&row.name, row.spec.as_deref());
        let exists = repositories::product::exists_product_by_name_spec(
            pool,
            &name_trim,
            spec_normalized.as_deref(),
        )
        .await?;

        if skip_duplicates && exists {
            return Ok(None); // 略過重複列
        }
        if regenerate_sku_for_duplicates && exists {
            return Ok(Some(true)); // 重複但使用自動 SKU
        }
        Ok(Some(false))
    }

    /// 產生產品匯入模板。
    pub fn generate_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);
        let worksheet = workbook.add_worksheet();
        Self::write_template_headers(worksheet, &header_format)?;
        Self::write_template_example(worksheet)?;
        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }

    /// 寫入模板表頭。
    fn write_template_headers(
        worksheet: &mut rust_xlsxwriter::Worksheet,
        fmt: &rust_xlsxwriter::Format,
    ) -> Result<()> {
        let headers = [
            (0, 16.0, "SKU編碼"),
            (1, 25.0, "名稱*"),
            (2, 20.0, "規格"),
            (3, 12.0, "品類代碼"),
            (4, 12.0, "子類代碼"),
            (5, 10.0, "單位*"),
            (6, 12.0, "追蹤批號"),
            (7, 12.0, "追蹤效期"),
            (8, 12.0, "安全庫存"),
            (9, 30.0, "備註"),
        ];
        for (col, width, label) in headers {
            worksheet.set_column_width(col, width)?;
            worksheet.write_string_with_format(0, col, label, fmt)?;
        }
        Ok(())
    }

    /// 寫入模板範例列。
    fn write_template_example(worksheet: &mut rust_xlsxwriter::Worksheet) -> Result<()> {
        let values = [
            "DRG-OTH-001", "範例產品", "100ml/瓶", "DRG", "OTH",
            "PCS", "false", "false", "10", "",
        ];
        for (col, val) in values.iter().enumerate() {
            worksheet.write_string(1, col as u16, *val)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{format_product_sku, validate_product_status};
    use crate::services::product_parser::parse_bool;

    #[test]
    fn test_format_product_sku_normal() {
        assert_eq!(format_product_sku("DRG", "OTH", 1), "DRG-OTH-001");
        assert_eq!(format_product_sku("GEN", "OTH", 42), "GEN-OTH-042");
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

    #[test]
    fn test_validate_product_status_allowed() {
        assert_eq!(validate_product_status("active").expect("valid"), "active");
        assert_eq!(
            validate_product_status("inactive").expect("valid"),
            "inactive"
        );
        assert_eq!(
            validate_product_status("discontinued").expect("valid"),
            "discontinued"
        );
        assert_eq!(
            validate_product_status("  ACTIVE  ").expect("valid"),
            "active"
        );
    }

    #[test]
    fn test_validate_product_status_invalid() {
        assert!(validate_product_status("pending").is_err());
        assert!(validate_product_status("").is_err());
        assert!(validate_product_status("ActiveX").is_err());
    }

    #[test]
    fn test_validate_product_status_error_message() {
        let msg = validate_product_status("x").expect_err("invalid status");
        assert!(msg.contains("active"));
        assert!(msg.contains("inactive"));
        assert!(msg.contains("discontinued"));
    }

    #[test]
    fn test_parse_bool_true_variants() {
        assert!(parse_bool("true"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("是"));
        assert!(parse_bool("y"));
        assert!(parse_bool("  YES  "));
    }

    #[test]
    fn test_parse_bool_false() {
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool(""));
        assert!(!parse_bool("n"));
        assert!(!parse_bool("other"));
    }
}
