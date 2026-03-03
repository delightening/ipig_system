use std::sync::OnceLock;
use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use crate::models::{CustomerCategory, PartnerType, SupplierCategory};
use crate::{
    models::{
        CreatePartnerRequest, PaginationParams, Partner, PartnerImportErrorDetail,
        PartnerImportResult, PartnerImportRow, PartnerQuery, UpdatePartnerRequest,
    },
    AppError, Result,
};

pub struct PartnerService;

/// 取得 Email 驗證用正則（靜態初始化，避免重複編譯）
fn email_regex() -> Result<&'static regex::Regex> {
    static EMAIL_REGEX: OnceLock<std::result::Result<regex::Regex, regex::Error>> = OnceLock::new();
    let res = EMAIL_REGEX.get_or_init(|| {
        regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
    });
    res.as_ref()
        .map_err(|e| AppError::Internal(format!("Email regex init: {}", e)))
}

impl PartnerService {
    /// 根據供應商類別生成代碼
    /// 格式：類型代碼 + {:03} 流水號
    /// 例如：藥001, 藥002, 耗001, 耗002, 飼001, 飼002, 儀001, 儀002
    pub async fn generate_code(
        pool: &PgPool,
        partner_type: crate::models::PartnerType,
        category: Option<SupplierCategory>,
    ) -> Result<String> {
        let prefix = match partner_type {
            crate::models::PartnerType::Supplier => match category {
                Some(SupplierCategory::Drug) => "藥",
                Some(SupplierCategory::Consumable) => "耗",
                Some(SupplierCategory::Feed) => "飼",
                Some(SupplierCategory::Equipment) => "儀",
                None => {
                    return Err(AppError::Validation(
                        "Supplier category is required for generating supplier code".to_string(),
                    ))
                }
            },
            crate::models::PartnerType::Customer => "客",
        };

        // 查詢該類別或類型的所有代碼
        let query = if partner_type == crate::models::PartnerType::Supplier {
            "SELECT code FROM partners WHERE supplier_category = $1 AND code LIKE $2 ORDER BY code DESC"
        } else {
            "SELECT code FROM partners WHERE partner_type = $1 AND code LIKE $2 ORDER BY code DESC"
        };

        let codes: Vec<String> = if partner_type == crate::models::PartnerType::Supplier {
            sqlx::query_scalar(query)
                .bind(category)
                .bind(format!("{}%", prefix))
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_scalar(query)
                .bind(partner_type)
                .bind(format!("{}%", prefix))
                .fetch_all(pool)
                .await?
        };

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

    /// 建立夥伴（供應商/客戶）
    pub async fn create(pool: &PgPool, req: &CreatePartnerRequest) -> Result<Partner> {
        // 如果 code 為空，則自動根據類型生成
        let code = match req
            .code
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            Some(provided) => provided.to_string(),
            None => Self::generate_code(pool, req.partner_type, req.supplier_category).await?,
        };

        // 檢查 code 是否已存在
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM partners WHERE code = $1)")
                .bind(&code)
                .fetch_one(pool)
                .await?;

        if exists {
            return Err(AppError::Conflict(
                "Partner code already exists".to_string(),
            ));
        }

        // 將空字串轉換為 None，並驗證 email 格式（如果提供）
        let email = req
            .email
            .as_ref()
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| {
                let re = email_regex()?;
                if !re.is_match(e) {
                    return Err(AppError::Validation("Invalid email format".to_string()));
                }
                Ok(e.to_string())
            })
            .transpose()?;

        let partner = sqlx::query_as::<_, Partner>(
            r#"
            INSERT INTO partners (
                id, partner_type, code, name, supplier_category, customer_category, tax_id, phone, email, address, 
                payment_terms, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(req.partner_type)
        .bind(&code)
        .bind(&req.name)
        .bind(req.supplier_category)
        .bind(req.customer_category)
        .bind(req.tax_id.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.phone.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(&email)
        .bind(req.address.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .bind(req.payment_terms.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()))
        .fetch_one(pool)
        .await?;

        Ok(partner)
    }

    /// 取得夥伴列表
    pub async fn list(pool: &PgPool, query: &PartnerQuery) -> Result<Vec<Partner>> {
        let pagination = PaginationParams { page: query.page, per_page: query.per_page };
        let suffix = pagination.sql_suffix();

        let partners = if let Some(ref kw) = query.keyword {
            let pattern = format!("%{}%", kw);
            if let Some(partner_type) = query.partner_type {
                if let Some(is_active) = query.is_active {
                    let sql = [
                        "SELECT * FROM partners WHERE (code ILIKE $1 OR name ILIKE $1) AND partner_type = $2 AND is_active = $3 ORDER BY code",
                        suffix.as_str(),
                    ]
                    .concat();
                    sqlx::query_as::<_, Partner>(&sql)
                        .bind(&pattern)
                        .bind(partner_type)
                        .bind(is_active)
                        .fetch_all(pool)
                        .await?
                } else {
                    let sql = [
                        "SELECT * FROM partners WHERE (code ILIKE $1 OR name ILIKE $1) AND partner_type = $2 ORDER BY code",
                        suffix.as_str(),
                    ]
                    .concat();
                    sqlx::query_as::<_, Partner>(&sql)
                        .bind(&pattern)
                        .bind(partner_type)
                        .fetch_all(pool)
                        .await?
                }
            } else if let Some(is_active) = query.is_active {
                let sql = [
                    "SELECT * FROM partners WHERE (code ILIKE $1 OR name ILIKE $1) AND is_active = $2 ORDER BY code",
                    suffix.as_str(),
                ]
                .concat();
                sqlx::query_as::<_, Partner>(&sql)
                    .bind(&pattern)
                    .bind(is_active)
                    .fetch_all(pool)
                    .await?
            } else {
                let sql = [
                    "SELECT * FROM partners WHERE (code ILIKE $1 OR name ILIKE $1) ORDER BY code",
                    suffix.as_str(),
                ]
                .concat();
                sqlx::query_as::<_, Partner>(&sql)
                    .bind(&pattern)
                    .fetch_all(pool)
                    .await?
            }
        } else if let Some(partner_type) = query.partner_type {
            if let Some(is_active) = query.is_active {
                let sql = [
                    "SELECT * FROM partners WHERE partner_type = $1 AND is_active = $2 ORDER BY code",
                    suffix.as_str(),
                ]
                .concat();
                sqlx::query_as::<_, Partner>(&sql)
                    .bind(partner_type)
                    .bind(is_active)
                    .fetch_all(pool)
                    .await?
            } else {
                let sql = [
                    "SELECT * FROM partners WHERE partner_type = $1 ORDER BY code",
                    suffix.as_str(),
                ]
                .concat();
                sqlx::query_as::<_, Partner>(&sql)
                    .bind(partner_type)
                    .fetch_all(pool)
                    .await?
            }
        } else if let Some(is_active) = query.is_active {
            let sql = [
                "SELECT * FROM partners WHERE is_active = $1 ORDER BY code",
                suffix.as_str(),
            ]
            .concat();
            sqlx::query_as::<_, Partner>(&sql)
                .bind(is_active)
                .fetch_all(pool)
                .await?
        } else {
            let sql =
                ["SELECT * FROM partners ORDER BY code", suffix.as_str()].concat();
            sqlx::query_as::<_, Partner>(&sql)
                .fetch_all(pool)
                .await?
        };

        Ok(partners)
    }

    /// 取得單一夥伴
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Partner> {
        let partner = sqlx::query_as::<_, Partner>("SELECT * FROM partners WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Partner not found".to_string()))?;

        Ok(partner)
    }

    /// 更新夥伴
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdatePartnerRequest) -> Result<Partner> {
        // 處理 email：將空字串轉換為 None，並驗證格式（如果提供）
        let email = req
            .email
            .as_ref()
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| {
                let re = email_regex()?;
                if !re.is_match(e) {
                    return Err(AppError::Validation("Invalid email format".to_string()));
                }
                Ok(e.to_string())
            })
            .transpose()?;

        let partner = sqlx::query_as::<_, Partner>(
            r#"
            UPDATE partners SET
                name = COALESCE($1, name),
                tax_id = COALESCE($2, tax_id),
                phone = COALESCE($3, phone),
                email = COALESCE($4, email),
                address = COALESCE($5, address),
                payment_terms = COALESCE($6, payment_terms),
                is_active = COALESCE($7, is_active),
                updated_at = NOW()
            WHERE id = $8
            RETURNING *
            "#,
        )
        .bind(req.name.as_ref())
        .bind(
            req.tax_id
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
        )
        .bind(
            req.phone
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
        )
        .bind(&email)
        .bind(
            req.address
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
        )
        .bind(
            req.payment_terms
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty()),
        )
        .bind(req.is_active)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Partner not found".to_string()))?;

        Ok(partner)
    }

    /// 刪除夥伴（軟刪除）
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result =
            sqlx::query("UPDATE partners SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Partner not found".to_string()));
        }

        Ok(())
    }

    // ============================================
    // 夥伴匯入
    // ============================================

    /// 匯入夥伴（CSV 或 Excel）
    pub async fn import_partners(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
    ) -> Result<PartnerImportResult> {
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        let rows = if is_excel {
            Self::parse_partner_excel(file_data)?
        } else {
            Self::parse_partner_csv(file_data)?
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
                errors.push(PartnerImportErrorDetail {
                    row: row_number,
                    code: None,
                    error: "名稱為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            let partner_type = match Self::parse_partner_type(&row.partner_type) {
                Some(pt) => pt,
                None => {
                    errors.push(PartnerImportErrorDetail {
                        row: row_number,
                        code: None,
                        error: format!("無效的類型: {}，必須是 supplier/customer 或 供應商/客戶", row.partner_type),
                    });
                    error_count += 1;
                    continue;
                }
            };

            let (supplier_category, customer_category) = if partner_type == PartnerType::Supplier {
                let sc = row.supplier_category.as_deref().and_then(Self::parse_supplier_category);
                if sc.is_none() {
                    let has_value = row.supplier_category.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
                    if has_value {
                        errors.push(PartnerImportErrorDetail {
                            row: row_number,
                            code: None,
                            error: format!("無效的供應商類別: {}，必須是 drug/consumable/feed/equipment", row.supplier_category.as_deref().unwrap_or("")),
                        });
                    } else {
                        errors.push(PartnerImportErrorDetail {
                            row: row_number,
                            code: None,
                            error: "供應商必須填寫供應商類別 (drug/consumable/feed/equipment)".to_string(),
                        });
                    }
                    error_count += 1;
                    continue;
                }
                (sc, None)
            } else {
                let cc = row.customer_category.as_deref().and_then(Self::parse_customer_category);
                (None, cc)
            };

            let create_req = CreatePartnerRequest {
                partner_type,
                code: row.code.clone().filter(|s| !s.trim().is_empty()),
                supplier_category,
                customer_category,
                name: row.name.trim().to_string(),
                tax_id: row.tax_id.clone().filter(|s| !s.trim().is_empty()),
                phone: row.phone.clone().filter(|s| !s.trim().is_empty()),
                email: row.email.clone().filter(|s| !s.trim().is_empty()),
                address: row.address.clone().filter(|s| !s.trim().is_empty()),
                payment_terms: row.payment_terms.clone().filter(|s| !s.trim().is_empty()),
            };

            match Self::create(pool, &create_req).await {
                Ok(_partner) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(PartnerImportErrorDetail {
                        row: row_number,
                        code: row.code.clone(),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        Ok(PartnerImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    fn parse_partner_type(s: &str) -> Option<PartnerType> {
        match s.trim().to_lowercase().as_str() {
            "supplier" | "供應商" | "s" => Some(PartnerType::Supplier),
            "customer" | "客戶" | "c" => Some(PartnerType::Customer),
            _ => None,
        }
    }

    fn parse_supplier_category(s: &str) -> Option<SupplierCategory> {
        match s.trim().to_lowercase().as_str() {
            "drug" | "藥物" => Some(SupplierCategory::Drug),
            "consumable" | "耗材" => Some(SupplierCategory::Consumable),
            "feed" | "飼料" => Some(SupplierCategory::Feed),
            "equipment" | "儀器" => Some(SupplierCategory::Equipment),
            _ => None,
        }
    }

    fn parse_customer_category(s: &str) -> Option<CustomerCategory> {
        match s.trim().to_lowercase().as_str() {
            "internal" | "內部" => Some(CustomerCategory::Internal),
            "external" | "外部" => Some(CustomerCategory::External),
            "research" | "研究" => Some(CustomerCategory::Research),
            "other" | "其他" => Some(CustomerCategory::Other),
            _ => None,
        }
    }

    fn parse_partner_csv(file_data: &[u8]) -> Result<Vec<PartnerImportRow>> {
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
            let partner_type = record.get(0).unwrap_or("").to_string();
            let name = record.get(1).unwrap_or("").to_string();
            if name.trim().is_empty() {
                continue;
            }
            let supplier_category = record.get(2).filter(|s| !s.trim().is_empty()).map(String::from);
            let customer_category = record.get(3).filter(|s| !s.trim().is_empty()).map(String::from);
            let code = record.get(4).filter(|s| !s.trim().is_empty()).map(String::from);
            let tax_id = record.get(5).filter(|s| !s.trim().is_empty()).map(String::from);
            let phone = record.get(6).filter(|s| !s.trim().is_empty()).map(String::from);
            let email = record.get(7).filter(|s| !s.trim().is_empty()).map(String::from);
            let address = record.get(8).filter(|s| !s.trim().is_empty()).map(String::from);
            let payment_terms = record.get(9).filter(|s| !s.trim().is_empty()).map(String::from);

            rows.push(PartnerImportRow {
                partner_type,
                name,
                supplier_category,
                customer_category,
                code,
                tax_id,
                phone,
                email,
                address,
                payment_terms,
            });
        }
        Ok(rows)
    }

    fn parse_partner_excel(file_data: &[u8]) -> Result<Vec<PartnerImportRow>> {
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
        iter.next();

        for row in iter {
            if row.len() < 2 {
                continue;
            }
            let partner_type = Self::get_cell_string(row.first());
            let name = Self::get_cell_string(row.get(1));
            if name.trim().is_empty() {
                continue;
            }
            let supplier_category = Self::opt_cell_string(row.get(2));
            let customer_category = Self::opt_cell_string(row.get(3));
            let code = Self::opt_cell_string(row.get(4));
            let tax_id = Self::opt_cell_string(row.get(5));
            let phone = Self::opt_cell_string(row.get(6));
            let email = Self::opt_cell_string(row.get(7));
            let address = Self::opt_cell_string(row.get(8));
            let payment_terms = Self::opt_cell_string(row.get(9));

            rows.push(PartnerImportRow {
                partner_type,
                name,
                supplier_category,
                customer_category,
                code,
                tax_id,
                phone,
                email,
                address,
                payment_terms,
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

    fn opt_cell_string(cell: Option<&Data>) -> Option<String> {
        cell.and_then(|c| {
            let s = match c {
                Data::String(s) => s.clone(),
                Data::Float(f) => f.to_string(),
                Data::Int(i) => i.to_string(),
                Data::Bool(b) => b.to_string(),
                Data::DateTime(dt) => format!("{:?}", dt),
                _ => String::new(),
            };
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        })
    }

    /// 產生夥伴匯入模板
    pub fn generate_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let worksheet = workbook.add_worksheet();
        worksheet.set_column_width(0, 12.0)?;
        worksheet.set_column_width(1, 25.0)?;
        worksheet.set_column_width(2, 12.0)?;
        worksheet.set_column_width(3, 12.0)?;
        worksheet.set_column_width(4, 12.0)?;
        worksheet.set_column_width(5, 12.0)?;
        worksheet.set_column_width(6, 15.0)?;
        worksheet.set_column_width(7, 25.0)?;
        worksheet.set_column_width(8, 35.0)?;
        worksheet.set_column_width(9, 20.0)?;

        worksheet.write_string_with_format(0, 0, "類型*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "名稱*", &header_format)?;
        worksheet.write_string_with_format(0, 2, "供應商類別", &header_format)?;
        worksheet.write_string_with_format(0, 3, "客戶分類", &header_format)?;
        worksheet.write_string_with_format(0, 4, "代碼", &header_format)?;
        worksheet.write_string_with_format(0, 5, "統編", &header_format)?;
        worksheet.write_string_with_format(0, 6, "電話", &header_format)?;
        worksheet.write_string_with_format(0, 7, "Email", &header_format)?;
        worksheet.write_string_with_format(0, 8, "地址", &header_format)?;
        worksheet.write_string_with_format(0, 9, "付款條件", &header_format)?;

        worksheet.write_string(1, 0, "supplier")?;
        worksheet.write_string(1, 1, "範例供應商")?;
        worksheet.write_string(1, 2, "drug")?;
        worksheet.write_string(1, 3, "")?;
        worksheet.write_string(1, 4, "")?;
        worksheet.write_string(1, 5, "")?;
        worksheet.write_string(1, 6, "")?;
        worksheet.write_string(1, 7, "")?;
        worksheet.write_string(1, 8, "")?;
        worksheet.write_string(1, 9, "")?;

        worksheet.write_string(2, 0, "customer")?;
        worksheet.write_string(2, 1, "範例客戶")?;
        worksheet.write_string(2, 2, "")?;
        worksheet.write_string(2, 3, "internal")?;
        worksheet.write_string(2, 4, "")?;
        worksheet.write_string(2, 5, "")?;
        worksheet.write_string(2, 6, "")?;
        worksheet.write_string(2, 7, "")?;
        worksheet.write_string(2, 8, "")?;
        worksheet.write_string(2, 9, "")?;

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }
}
