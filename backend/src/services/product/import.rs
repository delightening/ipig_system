// 產品匯入相關功能

use sqlx::PgPool;

use crate::{
    middleware::ActorContext,
    models::{
        CreateProductRequest, ProductImportCheckResult, ProductImportDuplicateItem,
        ProductImportErrorDetail, ProductImportPreviewResult, ProductImportResult,
    },
    repositories, AppError, Result,
};

use super::ProductService;
use crate::services::product_parser;

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

impl ProductService {
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

    /// 批量檢查匯入資料的重複情況（單一查詢取代 N 次查詢）。
    async fn find_all_duplicates(
        pool: &PgPool,
        rows: &[crate::models::ProductImportRow],
    ) -> Result<Vec<ProductImportDuplicateItem>> {
        // 收集所有非空名稱與正規化後的 spec
        let mut names: Vec<String> = Vec::with_capacity(rows.len());
        let mut specs: Vec<String> = Vec::with_capacity(rows.len());
        let mut row_numbers: Vec<i32> = Vec::with_capacity(rows.len());
        let mut original_specs: Vec<Option<String>> = Vec::with_capacity(rows.len());

        for (index, row) in rows.iter().enumerate() {
            if row.name.trim().is_empty() {
                continue;
            }
            let (name_trim, spec_normalized) = normalize_name_spec(&row.name, row.spec.as_deref());
            names.push(name_trim);
            specs.push(spec_normalized.unwrap_or_default());
            row_numbers.push((index + 2) as i32);
            original_specs.push(row.spec.clone().filter(|s| !s.trim().is_empty()));
        }

        if names.is_empty() {
            return Ok(Vec::new());
        }

        // 單一查詢：用 UNNEST 建立臨時列，再 JOIN products 找重複
        let matched: Vec<(i32, String, String, String, uuid::Uuid)> = sqlx::query_as(
            r#"
            SELECT d.row_number, d.name, p.sku, d.spec, p.id
            FROM UNNEST($1::int[], $2::text[], $3::text[]) AS d(row_number, name, spec)
            INNER JOIN products p
              ON LOWER(TRIM(p.name)) = LOWER(TRIM(d.name))
              AND (
                  (d.spec = '' AND (p.spec IS NULL OR TRIM(COALESCE(p.spec, '')) = ''))
                  OR LOWER(TRIM(COALESCE(p.spec, ''))) = LOWER(TRIM(d.spec))
              )
            "#,
        )
        .bind(&row_numbers)
        .bind(&names)
        .bind(&specs)
        .fetch_all(pool)
        .await?;

        let duplicates = matched
            .into_iter()
            .map(|(row_number, name, sku, _spec, id)| {
                let idx = row_numbers.iter().position(|r| *r == row_number).unwrap_or(0);
                ProductImportDuplicateItem {
                    row: row_number,
                    name,
                    spec: original_specs.get(idx).cloned().flatten(),
                    existing_sku: sku,
                    existing_id: id,
                }
            })
            .collect();

        Ok(duplicates)
    }

    /// 匯入產品（CSV 或 Excel）— Service-driven audit（N+1 粒度：每筆 create
    /// 產生 PRODUCT_CREATE，批次結束額外寫 1 筆 PRODUCT_IMPORT summary）。
    pub async fn import_products(
        pool: &PgPool,
        actor: &ActorContext,
        file_data: &[u8],
        file_name: &str,
        skip_duplicates: bool,
        regenerate_sku_for_duplicates: bool,
    ) -> Result<ProductImportResult> {
        let _user = actor.require_user()?;

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
                pool,
                actor,
                row,
                row_number,
                skip_duplicates,
                regenerate_sku_for_duplicates,
            )
            .await
            {
                Ok(true) => success_count += 1,
                Ok(false) => {} // skipped
                Err(e) => {
                    errors.push(e);
                    error_count += 1;
                }
            }
        }

        // 批次 summary audit（per-row 細節由個別 PRODUCT_CREATE 事件查）
        let summary_id = uuid::Uuid::new_v4();
        let display = format!(
            "import {} (success={}, errors={})",
            file_name, success_count, error_count
        );
        let mut tx = pool.begin().await?;
        crate::services::AuditService::log_activity_tx(
            &mut tx,
            actor,
            crate::services::audit::ActivityLogEntry {
                event_category: "ERP",
                event_type: "PRODUCT_IMPORT",
                entity: Some(crate::services::audit::AuditEntity::new(
                    "product_import_job",
                    summary_id,
                    &display,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;

        Ok(ProductImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    /// 處理單列匯入：回傳 Ok(true) 成功、Ok(false) 略過、Err 失敗。
    async fn import_single_row(
        pool: &PgPool,
        actor: &ActorContext,
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
        Self::create(pool, actor, &create_req).await.map(|_| true).map_err(|e| {
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
