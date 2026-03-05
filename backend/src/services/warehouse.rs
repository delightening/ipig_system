use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use crate::{
    models::{
        CreateWarehouseRequest, PaginationParams, ShelfNode, UpdateWarehouseRequest, Warehouse,
        WarehouseImportErrorDetail, WarehouseImportResult, WarehouseImportRow, WarehouseQuery,
        WarehouseTreeNode,
    },
    AppError, Result,
};

pub struct WarehouseService;

impl WarehouseService {
    /// 自動生成倉庫代碼（流水號格式：WH001, WH002, ...）
    async fn generate_code(pool: &PgPool) -> Result<String> {
        let prefix = "WH";

        // 取得所有以 WH 開頭的代碼
        let codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM warehouses WHERE code LIKE $1 ORDER BY code DESC")
                .bind(format!("{}%", prefix))
                .fetch_all(pool)
                .await?;

        // 解析序號並找出最大值
        let max_seq = codes
            .iter()
            .filter_map(|code| {
                if code.starts_with(prefix) && code.len() > prefix.len() {
                    let num_str = &code[prefix.len()..];
                    num_str.parse::<i32>().ok()
                } else {
                    None
                }
            })
            .max();

        let seq = max_seq.map(|s| s + 1).unwrap_or(1);
        Ok(format!("{}{:03}", prefix, seq))
    }

    /// 解析倉庫代碼序號（例如 WH001 → 1），供單元測試驗證
    #[cfg(test)]
    pub fn parse_code_sequence(code: &str, prefix: &str) -> Option<i32> {
        if !code.starts_with(prefix) || code.len() <= prefix.len() {
            return None;
        }
        let num_str = &code[prefix.len()..];
        num_str.parse::<i32>().ok()
    }

    /// 建立倉庫
    pub async fn create(pool: &PgPool, req: &CreateWarehouseRequest) -> Result<Warehouse> {
        // 如果 code 為空或未提供，則自動生成
        let code = match req
            .code
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            Some(provided_code) => {
                // 檢查 code 是否已存在
                let exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM warehouses WHERE code = $1)")
                        .bind(provided_code)
                        .fetch_one(pool)
                        .await?;

                if exists {
                    return Err(AppError::Conflict(
                        "Warehouse code already exists".to_string(),
                    ));
                }

                provided_code.to_string()
            }
            None => Self::generate_code(pool).await?,
        };

        let warehouse = sqlx::query_as::<_, Warehouse>(
            r#"
            INSERT INTO warehouses (id, code, name, address, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&code)
        .bind(&req.name)
        .bind(&req.address)
        .fetch_one(pool)
        .await?;

        Ok(warehouse)
    }

    /// 取得倉庫列表
    pub async fn list(pool: &PgPool, query: &WarehouseQuery) -> Result<Vec<Warehouse>> {
        let pagination = PaginationParams { page: query.page, per_page: query.per_page };
        let suffix = pagination.sql_suffix();

        let mut qb = sqlx::QueryBuilder::new("SELECT * FROM warehouses WHERE 1=1");

        if let Some(ref kw) = query.keyword {
            let pattern = format!("%{}%", kw);
            qb.push(" AND (code ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR name ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(is_active) = query.is_active {
            qb.push(" AND is_active = ");
            qb.push_bind(is_active);
        }

        qb.push(" ORDER BY code");
        qb.push(&suffix);

        let warehouses = qb
            .build_query_as::<Warehouse>()
            .fetch_all(pool)
            .await?;

        Ok(warehouses)
    }

    /// 取得倉庫樹（含貨架，location_type = shelf）
    pub async fn list_with_shelves(pool: &PgPool) -> Result<Vec<WarehouseTreeNode>> {
        let warehouses: Vec<Warehouse> = sqlx::query_as(
            "SELECT * FROM warehouses WHERE is_active = true ORDER BY code",
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(warehouses.len());
        for wh in warehouses {
            let shelves: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
                r#"
                SELECT id, code, name FROM storage_locations
                WHERE warehouse_id = $1 AND is_active = true AND location_type = 'shelf'
                ORDER BY row_index, col_index, code
                "#,
            )
            .bind(wh.id)
            .fetch_all(pool)
            .await?;

            let shelf_nodes: Vec<ShelfNode> = shelves
                .into_iter()
                .map(|(id, code, name)| ShelfNode { id, code, name })
                .collect();

            result.push(WarehouseTreeNode {
                id: wh.id,
                code: wh.code,
                name: wh.name,
                shelves: shelf_nodes,
            });
        }
        Ok(result)
    }

    /// 取得單一倉庫
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Warehouse> {
        let warehouse = sqlx::query_as::<_, Warehouse>("SELECT * FROM warehouses WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        Ok(warehouse)
    }

    /// 更新倉庫
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateWarehouseRequest,
    ) -> Result<Warehouse> {
        let warehouse = sqlx::query_as::<_, Warehouse>(
            r#"
            UPDATE warehouses SET
                name = COALESCE($1, name),
                address = COALESCE($2, address),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.address)
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Warehouse not found".to_string()))?;

        Ok(warehouse)
    }

    /// 刪除倉庫（軟刪除）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "UPDATE warehouses SET is_active = false, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Warehouse not found".to_string()));
        }

        Ok(())
    }

    // ============================================
    // 倉庫匯入
    // ============================================

    /// 匯入倉庫（CSV 或 Excel）
    pub async fn import_warehouses(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
    ) -> Result<WarehouseImportResult> {
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let rows = if is_excel {
            Self::parse_warehouse_excel(file_data)?
        } else {
            Self::parse_warehouse_csv(file_data)?
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
                errors.push(WarehouseImportErrorDetail {
                    row: row_number,
                    code: None,
                    error: "名稱為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            let create_req = CreateWarehouseRequest {
                code: row.code.clone().filter(|s| !s.trim().is_empty()),
                name: row.name.trim().to_string(),
                address: row.address.clone().filter(|s| !s.trim().is_empty()),
            };

            match Self::create(pool, &create_req).await {
                Ok(_warehouse) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(WarehouseImportErrorDetail {
                        row: row_number,
                        code: row.code.clone(),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        Ok(WarehouseImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    fn parse_warehouse_csv(file_data: &[u8]) -> Result<Vec<WarehouseImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());

        let mut rows = Vec::new();
        for (i, result) in reader.records().enumerate() {
            let record = result.map_err(|e| AppError::Validation(format!("CSV 解析錯誤第 {} 行: {}", i + 2, e)))?;
            if record.is_empty() {
                continue;
            }
            let name = record.get(0).unwrap_or("").to_string();
            if name.trim().is_empty() {
                continue;
            }
            let code = record.get(1).filter(|s| !s.trim().is_empty()).map(String::from);
            let address = record.get(2).filter(|s| !s.trim().is_empty()).map(String::from);

            rows.push(WarehouseImportRow {
                name,
                code,
                address,
            });
        }
        Ok(rows)
    }

    fn parse_warehouse_excel(file_data: &[u8]) -> Result<Vec<WarehouseImportRow>> {
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
            if row.is_empty() {
                continue;
            }
            let name = Self::get_cell_string(row.first());
            if name.trim().is_empty() {
                continue;
            }
            let code = {
                let s = Self::get_cell_string(row.get(1));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let address = {
                let s = Self::get_cell_string(row.get(2));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };

            rows.push(WarehouseImportRow {
                name,
                code,
                address,
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

    /// 產生倉庫匯入模板
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
        worksheet.set_column_width(1, 15.0)?;
        worksheet.set_column_width(2, 40.0)?;

        worksheet.write_string_with_format(0, 0, "名稱*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "代碼", &header_format)?;
        worksheet.write_string_with_format(0, 2, "地址", &header_format)?;

        worksheet.write_string(1, 0, "範例倉庫")?;
        worksheet.write_string(1, 1, "WH001")?;
        worksheet.write_string(1, 2, "")?;

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }
}

#[cfg(test)]
mod tests {
    use super::WarehouseService;

    #[test]
    fn test_parse_code_sequence_valid() {
        assert_eq!(WarehouseService::parse_code_sequence("WH001", "WH"), Some(1));
        assert_eq!(WarehouseService::parse_code_sequence("WH002", "WH"), Some(2));
        assert_eq!(WarehouseService::parse_code_sequence("WH999", "WH"), Some(999));
    }

    #[test]
    fn test_parse_code_sequence_prefix_mismatch() {
        assert!(WarehouseService::parse_code_sequence("WH001", "WS").is_none());
        assert!(WarehouseService::parse_code_sequence("WS001", "WH").is_none());
    }

    #[test]
    fn test_parse_code_sequence_empty_suffix() {
        assert!(WarehouseService::parse_code_sequence("WH", "WH").is_none());
    }

    #[test]
    fn test_parse_code_sequence_invalid_digits() {
        assert!(WarehouseService::parse_code_sequence("WHabc", "WH").is_none());
    }

    #[test]
    fn test_parse_code_sequence_leading_zeros() {
        assert_eq!(WarehouseService::parse_code_sequence("WH001", "WH"), Some(1));
        assert_eq!(WarehouseService::parse_code_sequence("WH012", "WH"), Some(12));
    }

    #[test]
    fn test_parse_code_sequence_case_sensitive() {
        // 前綴區分大小寫
        assert!(WarehouseService::parse_code_sequence("wh001", "WH").is_none());
    }
}
