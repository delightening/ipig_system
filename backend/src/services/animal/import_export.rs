use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        AnimalBreed, AnimalGender, AnimalImportRow, AnimalSource, CreateAnimalRequest,
        CreateWeightRequest, ImportErrorDetail, ImportResult, ImportType, UpdateAnimalRequest,
        WeightImportRow,
    },
    AppError, Result,
};

impl AnimalService {
    // ============================================
    // 模板生成功能
    // ============================================

    /// 生成動物基本資料匯入模板
    pub fn generate_basic_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();

        // 創建格式
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let example_format = Format::new().set_font_color("#808080").set_italic();

        // 創建工作表
        let worksheet = workbook.add_worksheet();

        // 設置欄位寬度
        worksheet.set_column_width(0, 15.0)?; // 耳號
        worksheet.set_column_width(1, 12.0)?; // 品種
        worksheet.set_column_width(2, 10.0)?; // 性別
        worksheet.set_column_width(3, 15.0)?; // 出生日期
        worksheet.set_column_width(4, 15.0)?; // 進場日期
        worksheet.set_column_width(5, 12.0)?; // 進場體重
        worksheet.set_column_width(6, 12.0)?; // 欄位編號
        worksheet.set_column_width(7, 15.0)?; // 實驗前代號
        worksheet.set_column_width(8, 12.0)?; // 備註

        // 寫入標題行
        worksheet.write_string_with_format(0, 0, "耳號*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "品種*", &header_format)?;
        worksheet.write_string_with_format(0, 2, "性別*", &header_format)?;
        worksheet.write_string_with_format(0, 3, "出生日期", &header_format)?;
        worksheet.write_string_with_format(0, 4, "進場日期*", &header_format)?;
        worksheet.write_string_with_format(0, 5, "進場體重*", &header_format)?;
        worksheet.write_string_with_format(0, 6, "欄位編號", &header_format)?;
        worksheet.write_string_with_format(0, 7, "實驗前代號", &header_format)?;
        worksheet.write_string_with_format(0, 8, "備註", &header_format)?;

        // 寫入說明行
        worksheet.write_string_with_format(1, 0, "001", &example_format)?;
        worksheet.write_string(1, 1, "miniature")?;
        worksheet.write_string(1, 2, "male")?;
        worksheet.write_string_with_format(1, 3, "2024-01-15", &example_format)?;
        worksheet.write_string_with_format(1, 4, "2024-02-01", &example_format)?;
        worksheet.write_string(1, 5, "25.5")?;
        worksheet.write_string(1, 6, "A-01")?;
        worksheet.write_string(1, 7, "PRE-001")?;
        worksheet.write_string(1, 8, "")?;

        // 凍結第一行
        worksheet.set_freeze_panes(1, 0)?;

        // 將工作簿轉換為字節數組
        let buffer = workbook.save_to_buffer()?;

        Ok(buffer)
    }

    /// 生成動物體重匯入模板
    pub fn generate_weight_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();

        // 創建格式
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let example_format = Format::new().set_font_color("#808080").set_italic();

        // 創建工作表
        let worksheet = workbook.add_worksheet();

        // 設置欄位寬度
        worksheet.set_column_width(0, 15.0)?; // 耳號
        worksheet.set_column_width(1, 15.0)?; // 測量日期
        worksheet.set_column_width(2, 12.0)?; // 體重

        // 寫入標題行
        worksheet.write_string_with_format(0, 0, "耳號*", &header_format)?;
        worksheet.write_string_with_format(0, 1, "測量日期*", &header_format)?;
        worksheet.write_string_with_format(0, 2, "體重(kg)*", &header_format)?;

        // 寫入說明行
        worksheet.write_string_with_format(1, 0, "001", &example_format)?;
        worksheet.write_string_with_format(1, 1, "2024-02-01", &example_format)?;
        worksheet.write_string(1, 2, "25.5")?;

        // 凍結第一行
        worksheet.set_freeze_panes(1, 0)?;

        // 將工作簿轉換為字節數組
        let buffer = workbook.save_to_buffer()?;

        Ok(buffer)
    }

    /// 生成動物基本資料匯入 CSV 模板
    pub fn generate_basic_import_template_csv() -> Result<Vec<u8>> {
        // 優先讀取專案根目錄下的 file imput.csv
        // 在 Docker 或 生產環境中可能需要調整路徑，此處先嘗試相對路徑
        let paths = [
            "file imput.csv",
            "../file imput.csv",
            "./backend/file imput.csv",
        ];
        for path in paths {
            if let Ok(data) = std::fs::read(path) {
                return Ok(data);
            }
        }

        // 如果找不到檔案，則生成預設模板
        let mut wtr = csv::Writer::from_writer(vec![]);

        // 寫入標題行
        wtr.write_record([
            "耳號*",
            "品種*",
            "性別*",
            "出生日期",
            "進場日期*",
            "進場體重*",
            "欄位編號",
            "實驗前代號",
            "備註",
        ])
        .map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;

        // 寫入範例行
        wtr.write_record([
            "001",
            "miniature",
            "male",
            "2024-01-15",
            "2024-02-01",
            "25.5",
            "A-01",
            "PRE-001",
            "",
        ])
        .map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;

        wtr.flush()
            .map_err(|e| AppError::Internal(format!("CSV flush 失敗: {}", e)))?;
        wtr.into_inner()
            .map_err(|e| AppError::Internal(format!("CSV 生成失敗: {}", e)))
    }

    /// 生成動物體重匯入 CSV 模板
    pub fn generate_weight_import_template_csv() -> Result<Vec<u8>> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // 寫入標題行
        wtr.write_record(["耳號*", "測量日期*", "體重(kg)*"])
            .map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;

        // 寫入範例行
        wtr.write_record(["001", "2024-02-01", "25.5"])
            .map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;

        wtr.flush()
            .map_err(|e| AppError::Internal(format!("CSV flush 失敗: {}", e)))?;
        wtr.into_inner()
            .map_err(|e| AppError::Internal(format!("CSV 生成失敗: {}", e)))
    }

    // ============================================
    // 檔案匯入處理
    // ============================================

    /// 匯入動物基本資料
    pub async fn import_basic_data(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        created_by: Uuid,
    ) -> Result<ImportResult> {
        // 根據檔案副檔名判斷格式
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        // 解析檔案
        let rows = if is_excel {
            Self::parse_excel_file(file_data)?
        } else {
            Self::parse_csv_file(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        // 建立匯入批次記錄
        let batch = Self::create_import_batch(
            pool,
            ImportType::AnimalBasic,
            file_name,
            rows.len() as i32,
            created_by,
        )
        .await?;

        // 處理每一行資料
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();
        let mut seen_ear_tags = std::collections::HashSet::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32; // +2 因為第一行是標題，索引從 0 開始

            // 驗證必填欄位
            if row.ear_tag.is_empty() {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: None,
                    error: "耳號為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 檢查檔案內重複
            if !seen_ear_tags.insert(row.ear_tag.clone()) {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(row.ear_tag.clone()),
                    error: "耳號在檔案中重複".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 驗證品種（保存原始值用於 breed_other）
            let (breed, raw_breed_other) = match row.breed.to_lowercase().as_str() {
                "minipig" | "miniature" | "mini" | "m" | "迷你豬" | "迷你" => {
                    (AnimalBreed::Minipig, None)
                }
                "white" | "w" | "白豬" | "大白豬" => (AnimalBreed::White, None),
                _ => {
                    // 非標準品種，將原始值保存到 breed_other
                    let trimmed = row.breed.trim().to_string();
                    if trimmed.is_empty() {
                        (AnimalBreed::Other, None)
                    } else {
                        (AnimalBreed::Other, Some(trimmed))
                    }
                }
            };

            // 驗證性別
            let gender = match row.gender.to_lowercase().as_str() {
                "male" | "m" | "公" | "公豬" => AnimalGender::Male,
                "female" | "f" | "母" | "母豬" => AnimalGender::Female,
                _ => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!(
                            "無效的性別值: {}，必須是 male/female 或 m/f (或公/母)",
                            row.gender
                        ),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 驗證進場日期
            let entry_date_str = row.entry_date.replace("/", "-");
            let entry_date = match chrono::NaiveDate::parse_from_str(&entry_date_str, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!(
                            "無效的進場日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式",
                            row.entry_date
                        ),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 解析出生日期（選填）
            let birth_date = if let Some(ref bd) = row.birth_date {
                if bd.is_empty() {
                    None
                } else {
                    let bd_str = bd.replace("/", "-");
                    match chrono::NaiveDate::parse_from_str(&bd_str, "%Y-%m-%d") {
                        Ok(date) => Some(date),
                        Err(_) => {
                            errors.push(ImportErrorDetail {
                                row: row_number,
                                ear_tag: Some(row.ear_tag.clone()),
                                error: format!(
                                    "無效的出生日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式",
                                    bd
                                ),
                            });
                            error_count += 1;
                            continue;
                        }
                    }
                }
            } else {
                None
            };

            // 解析進場體重（必填）
            let entry_weight = match row.entry_weight.as_ref().and_then(|w| {
                w.replace("kg", "")
                    .replace("公斤", "")
                    .trim()
                    .parse::<f64>()
                    .ok()
            }) {
                Some(w) if w > 0.0 => rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default(),
                _ => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: "進場體重為必填欄位且必須是大於 0 的數字".to_string(),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 查找來源 ID（如果有提供 source_code）
            let source_id = if let Some(ref code) = row.source_code {
                if !code.is_empty() {
                    let source = sqlx::query_as::<_, AnimalSource>(
                        "SELECT * FROM animal_sources WHERE code = $1 AND is_active = true",
                    )
                    .bind(code)
                    .fetch_optional(pool)
                    .await?;
                    source.map(|s| s.id)
                } else {
                    None
                }
            } else {
                None
            };

            // 格式化耳號
            let formatted_ear_tag = if let Ok(num) = row.ear_tag.parse::<u32>() {
                format!("{:03}", num)
            } else {
                row.ear_tag.clone()
            };

            // 檢查耳號是否已存在（僅檢查未刪除的動物）
            let existing = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM animals WHERE ear_tag = $1 AND deleted_at IS NULL",
            )
            .bind(&formatted_ear_tag)
            .fetch_optional(pool)
            .await?;

            if existing.is_some() {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(formatted_ear_tag),
                    error: "耳號已存在於系統中".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 處理欄位位置並格式化
            let pen_location = row
                .pen_location
                .clone()
                .or_else(|| match (&row.field_region, &row.field_number) {
                    (Some(r), Some(n)) => Some(format!("{}{}", r, n)),
                    (Some(r), None) => Some(r.clone()),
                    (None, Some(n)) => Some(n.clone()),
                    (None, None) => None,
                })
                .map(|s| Self::format_pen_location(&s));

            // 驗證欄位位置（必填）
            if pen_location.is_none() || pen_location.as_ref().map(|s| s.is_empty()).unwrap_or(true)
            {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(row.ear_tag.clone()),
                    error: "欄位為必填，請填寫 Field Region 或 Field Number".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 處理品種其他（當 breed 為 Other 時使用）
            // 優先使用從 CSV 欄位解析出的非標準品種值，若沒有則使用明確的 breed_other 欄位
            let breed_other = if matches!(breed, AnimalBreed::Other) {
                raw_breed_other.or_else(|| {
                    row.breed_other.clone().and_then(|s| {
                        let trimmed = s.trim().to_string();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed)
                        }
                    })
                })
            } else {
                None
            };

            // 建立動物資料
            let create_req = CreateAnimalRequest {
                ear_tag: formatted_ear_tag.clone(),
                breed,
                breed_other,
                source_id,
                gender,
                birth_date,
                entry_date,
                entry_weight,
                pen_location,
                pre_experiment_code: row.pre_experiment_code.clone(),
                remark: row.remark.clone(),
                force_create: true, // 匯入流程已有自己的耳號檢查邏輯
            };

            match Self::create(pool, &create_req, created_by).await {
                Ok(animal) => {
                    // 如果有計畫編號，更新它
                    if let Some(ref iacuc) = row.iacuc_no {
                        if !iacuc.is_empty() {
                            if let Err(e) = Self::update(
                                pool,
                                animal.id,
                                &UpdateAnimalRequest {
                                    iacuc_no: Some(iacuc.clone()),
                                    ..Default::default()
                                },
                                created_by,
                            )
                            .await
                            {
                                tracing::warn!("更新動物資料失敗: {e}");
                            }
                        }
                    }
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        // 更新批次結果
        let error_details = if errors.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&errors).unwrap_or(serde_json::Value::Null))
        };

        let _batch = Self::update_import_batch_result(
            pool,
            batch.id,
            success_count,
            error_count,
            error_details,
        )
        .await?;

        Ok(ImportResult {
            batch_id: batch.id,
            total_rows: rows.len() as i32,
            success_count,
            error_count,
            errors,
        })
    }

    /// 匯入體重資料
    pub async fn import_weight_data(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        created_by: Uuid,
    ) -> Result<ImportResult> {
        // 根據檔案副檔名判斷格式
        let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
        let is_csv = file_name.ends_with(".csv");

        if !is_excel && !is_csv {
            return Err(AppError::Validation(
                "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
            ));
        }

        // 解析檔案
        let rows = if is_excel {
            Self::parse_weight_excel_file(file_data)?
        } else {
            Self::parse_weight_csv_file(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        // 建立匯入批次記錄
        let batch = Self::create_import_batch(
            pool,
            ImportType::AnimalWeight,
            file_name,
            rows.len() as i32,
            created_by,
        )
        .await?;

        // 處理每一行資料
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            // 驗證必填欄位
            if row.ear_tag.is_empty() {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: None,
                    error: "耳號為必填欄位".to_string(),
                });
                error_count += 1;
                continue;
            }

            // 驗證測量日期
            let measure_date_str = row.measure_date.replace("/", "-");
            let measure_date =
                match chrono::NaiveDate::parse_from_str(&measure_date_str, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => {
                        errors.push(ImportErrorDetail {
                            row: row_number,
                            ear_tag: Some(row.ear_tag.clone()),
                            error: format!(
                                "無效的測量日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式",
                                row.measure_date
                            ),
                        });
                        error_count += 1;
                        continue;
                    }
                };

            // 驗證體重
            let weight_val = row
                .weight
                .replace("kg", "")
                .replace("公斤", "")
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0);
            if weight_val <= 0.0 {
                errors.push(ImportErrorDetail {
                    row: row_number,
                    ear_tag: Some(row.ear_tag.clone()),
                    error: format!("無效的體重值: {}，必須是大於 0 的數字", row.weight),
                });
                error_count += 1;
                continue;
            }

            // 格式化耳號
            let formatted_ear_tag = if let Ok(num) = row.ear_tag.parse::<u32>() {
                format!("{:03}", num)
            } else {
                row.ear_tag.clone()
            };

            // 查找動物（僅查找未刪除的動物）
            let animal_id_result = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM animals WHERE ear_tag = $1 AND deleted_at IS NULL",
            )
            .bind(&formatted_ear_tag)
            .fetch_optional(pool)
            .await?;

            let animal_id = match animal_id_result {
                Some(id) => id,
                None => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: "找不到對應的動物".to_string(),
                    });
                    error_count += 1;
                    continue;
                }
            };

            // 建立體重紀錄
            let weight_decimal =
                rust_decimal::Decimal::from_f64_retain(weight_val).unwrap_or_default();

            let create_req = CreateWeightRequest {
                measure_date,
                weight: weight_decimal,
            };

            match Self::create_weight(pool, animal_id, &create_req, created_by).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    errors.push(ImportErrorDetail {
                        row: row_number,
                        ear_tag: Some(row.ear_tag.clone()),
                        error: format!("建立失敗: {}", e),
                    });
                    error_count += 1;
                }
            }
        }

        // 更新批次結果
        let error_details = if errors.is_empty() {
            None
        } else {
            Some(serde_json::to_value(&errors).unwrap_or(serde_json::Value::Null))
        };

        let _batch = Self::update_import_batch_result(
            pool,
            batch.id,
            success_count,
            error_count,
            error_details,
        )
        .await?;

        Ok(ImportResult {
            batch_id: batch.id,
            total_rows: rows.len() as i32,
            success_count,
            error_count,
            errors,
        })
    }

    /// 解析 Excel 檔案（基本資料）
    fn parse_excel_file(file_data: &[u8]) -> Result<Vec<AnimalImportRow>> {
        let range =
            {
                let cursor = Cursor::new(file_data);
                if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
                    let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
                        AppError::Validation("Excel 檔案中沒有工作表".to_string())
                    })?;
                    wb.worksheet_range(&sheet_name)
                        .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
                } else {
                    let cursor = Cursor::new(file_data);
                    let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor).map_err(|_e| {
                        AppError::Validation(
                            "無法讀取 Excel 檔案，請確認檔案格式為 .xlsx 或 .xls".to_string(),
                        )
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

        // 跳過標題行
        iter.next();

        for row in iter {
            if row.len() < 4 {
                continue; // 跳過資料不足的行
            }

            let ear_tag = Self::get_cell_string(row.first());
            let breed = Self::get_cell_string(row.get(1));
            let gender = Self::get_cell_string(row.get(2));
            let birth_date = {
                let s = Self::get_cell_string(row.get(3));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let entry_date = Self::get_cell_string(row.get(4));
            let entry_weight = row.get(5).and_then(|c| match c {
                Data::Float(f) => Some(*f),
                Data::Int(i) => Some(*i as f64),
                _ => None,
            });
            let pen_location = {
                let s = Self::get_cell_string(row.get(6));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let pre_experiment_code = {
                let s = Self::get_cell_string(row.get(7));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let remark = {
                let s = Self::get_cell_string(row.get(8));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };
            let breed_other = {
                let s = Self::get_cell_string(row.get(9));
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            };

            // 如果必填欄位為空，跳過這行
            if ear_tag.is_empty() || breed.is_empty() || gender.is_empty() || entry_date.is_empty()
            {
                continue;
            }

            rows.push(AnimalImportRow {
                ear_tag,
                breed,
                breed_other,
                gender,
                source_code: None, // Excel 範本中沒有這個欄位
                birth_date,
                entry_date,
                entry_weight: entry_weight.map(|w| w.to_string()),
                pen_location,
                pre_experiment_code,
                remark,
                iacuc_no: None,
                field_region: None,
                field_number: None,
            });
        }

        Ok(rows)
    }

    /// 解析 CSV 檔案（基本資料）
    fn parse_csv_file(file_data: &[u8]) -> Result<Vec<AnimalImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());
        let mut rows = Vec::new();

        let mut row_idx = 1;
        for result in reader.deserialize::<AnimalImportRow>() {
            row_idx += 1;
            match result {
                Ok(row) => {
                    if !row.ear_tag.is_empty() {
                        rows.push(row);
                    }
                }
                Err(e) => {
                    return Err(AppError::Validation(format!(
                        "基本資料 CSV 第 {} 行解析錯誤: {}",
                        row_idx, e
                    )));
                }
            }
        }

        Ok(rows)
    }

    /// 解析 Excel 檔案（體重資料）
    fn parse_weight_excel_file(file_data: &[u8]) -> Result<Vec<WeightImportRow>> {
        let range =
            {
                let cursor = Cursor::new(file_data);
                if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
                    let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
                        AppError::Validation("Excel 檔案中沒有工作表".to_string())
                    })?;
                    wb.worksheet_range(&sheet_name)
                        .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))?
                } else {
                    let cursor = Cursor::new(file_data);
                    let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor).map_err(|_e| {
                        AppError::Validation(
                            "無法讀取 Excel 檔案，請確認檔案格式為 .xlsx 或 .xls".to_string(),
                        )
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

        // 跳過標題行
        iter.next();

        for row in iter {
            if row.len() < 3 {
                continue;
            }

            let ear_tag = Self::get_cell_string(row.first()).trim().to_string();
            let measure_date = Self::get_cell_string(row.get(1)).trim().to_string();
            let weight = row
                .get(2)
                .and_then(|c| match c {
                    Data::Float(f) => Some(*f),
                    Data::Int(i) => Some(*i as f64),
                    _ => None,
                })
                .unwrap_or(0.0);

            if ear_tag.is_empty() || measure_date.is_empty() || weight <= 0.0 {
                continue;
            }

            rows.push(WeightImportRow {
                ear_tag,
                measure_date,
                weight: weight.to_string(),
            });
        }

        Ok(rows)
    }

    /// 解析 CSV 檔案（體重資料）
    fn parse_weight_csv_file(file_data: &[u8]) -> Result<Vec<WeightImportRow>> {
        let content = String::from_utf8_lossy(file_data);
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(content.as_bytes());
        let mut rows = Vec::new();

        let mut row_idx = 1;
        for result in reader.deserialize::<WeightImportRow>() {
            row_idx += 1;
            match result {
                Ok(row) => {
                    if !row.ear_tag.is_empty() {
                        rows.push(row);
                    }
                }
                Err(e) => {
                    return Err(AppError::Validation(format!(
                        "體重資料 CSV 第 {} 行解析錯誤: {}",
                        row_idx, e
                    )));
                }
            }
        }

        Ok(rows)
    }

    /// 從 Excel 儲存格取得字串值
    fn get_cell_string(cell: Option<&Data>) -> String {
        if let Some(c) = cell {
            match c {
                Data::String(s) => s.trim().to_string(),
                Data::Int(i) => i.to_string(),
                Data::Float(f) => f.to_string(),
                Data::Bool(b) => b.to_string(),
                Data::DateTime(dt) => {
                    let f = dt.as_f64();
                    let days = f as i64;
                    // Excel 基準日期 1899-12-30
                    if let Some(base_date) = chrono::NaiveDate::from_ymd_opt(1899, 12, 30) {
                        if let Some(date) =
                            base_date.checked_add_signed(chrono::Duration::days(days))
                        {
                            return date.format("%Y-%m-%d").to_string();
                        }
                    }
                    f.to_string()
                }
                _ => String::new(),
            }
        } else {
            String::new()
        }
    }
}
