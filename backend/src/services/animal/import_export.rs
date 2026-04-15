use calamine::{open_workbook_from_rs, Data, Range, Reader, Xls, Xlsx};
use sqlx::PgPool;
use std::io::Cursor;
use uuid::Uuid;

use super::{utils::AnimalUtils, AnimalMedicalService, AnimalService, AnimalWeightService};
use crate::{
    models::{
        AnimalBreed, AnimalGender, AnimalImportRow, AnimalSource, CreateAnimalRequest,
        CreateWeightRequest, ImportErrorDetail, ImportResult, ImportType, UpdateAnimalRequest,
        WeightImportRow,
    },
    AppError, Result,
};

/// 驗證後的基本資料行
struct ValidatedBasicRow {
    formatted_ear_tag: String,
    breed: AnimalBreed,
    breed_other: Option<String>,
    gender: AnimalGender,
    entry_date: chrono::NaiveDate,
    birth_date: Option<chrono::NaiveDate>,
    entry_weight: rust_decimal::Decimal,
    pen_location: Option<String>,
    pre_experiment_code: Option<String>,
    remark: Option<String>,
    source_code: Option<String>,
    iacuc_no: Option<String>,
}

/// 驗證後的體重資料行
struct ValidatedWeightRow {
    formatted_ear_tag: String,
    measure_date: chrono::NaiveDate,
    weight: rust_decimal::Decimal,
}

pub struct AnimalImportExportService;

// ============================================
// 共用解析輔助函式
// ============================================

/// 解析品種字串，回傳 (AnimalBreed, Option<breed_other>)
fn parse_import_breed(raw: &str) -> (AnimalBreed, Option<String>) {
    match raw.to_lowercase().as_str() {
        "minipig" | "miniature" | "mini" | "m" | "迷你豬" | "迷你" => (AnimalBreed::Minipig, None),
        "white" | "w" | "白豬" | "大白豬" => (AnimalBreed::White, None),
        _ => {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() {
                (AnimalBreed::Other, None)
            } else {
                (AnimalBreed::Other, Some(trimmed))
            }
        }
    }
}

/// 解析性別字串
fn parse_import_gender(raw: &str) -> Option<AnimalGender> {
    match raw.to_lowercase().as_str() {
        "male" | "m" | "公" | "公豬" => Some(AnimalGender::Male),
        "female" | "f" | "母" | "母豬" => Some(AnimalGender::Female),
        _ => None,
    }
}

/// 解析日期字串（支援 `/` 與 `-` 分隔）
fn parse_date_field(raw: &str) -> Option<chrono::NaiveDate> {
    let normalized = raw.replace("/", "-");
    chrono::NaiveDate::parse_from_str(&normalized, "%Y-%m-%d").ok()
}

/// 解析體重字串，移除單位後轉為 f64
fn parse_weight_value(raw: &str) -> Option<f64> {
    let cleaned = raw.replace("kg", "").replace("公斤", "");
    let val: f64 = cleaned.trim().parse().ok()?;
    if val > 0.0 { Some(val) } else { None }
}

/// 格式化耳號（數字補零至三位）
fn format_ear_tag(raw: &str) -> String {
    if let Ok(num) = raw.parse::<u32>() {
        format!("{:03}", num)
    } else {
        raw.to_string()
    }
}

/// 開啟 Excel 檔案並回傳第一個工作表的範圍（支援 .xlsx 與 .xls）
fn open_excel_range(file_data: &[u8]) -> Result<Range<Data>> {
    let cursor = Cursor::new(file_data);
    if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
        let sheet_name = wb
            .sheet_names()
            .first()
            .cloned()
            .ok_or_else(|| AppError::Validation("Excel 檔案中沒有工作表".to_string()))?;
        return wb
            .worksheet_range(&sheet_name)
            .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)));
    }

    let cursor = Cursor::new(file_data);
    let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor).map_err(|_e| {
        AppError::Validation(
            "無法讀取 Excel 檔案，請確認檔案格式為 .xlsx 或 .xls".to_string(),
        )
    })?;
    let sheet_name = wb
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| AppError::Validation("Excel 檔案中沒有工作表".to_string()))?;
    wb.worksheet_range(&sheet_name)
        .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))
}

/// 建立匯入錯誤
fn import_err(row: i32, ear_tag: Option<String>, error: String) -> ImportErrorDetail {
    ImportErrorDetail {
        row,
        ear_tag,
        error,
    }
}

/// 更新匯入批次狀態並組裝 ImportResult
async fn finalize_import_batch(
    pool: &PgPool,
    batch_id: Uuid,
    total_rows: i32,
    success_count: i32,
    error_count: i32,
    errors: Vec<ImportErrorDetail>,
) -> Result<ImportResult> {
    let error_details = if errors.is_empty() {
        None
    } else {
        Some(serde_json::to_value(&errors).unwrap_or(serde_json::Value::Null))
    };

    AnimalMedicalService::update_import_batch_status(
        pool,
        batch_id,
        crate::models::ImportStatus::Completed,
        success_count,
        error_count,
        error_details.as_ref().map(|v| v.to_string()).as_deref(),
    )
    .await?;

    Ok(ImportResult {
        batch_id,
        total_rows,
        success_count,
        error_count,
        errors,
    })
}

/// 從 Excel 儲存格取得字串值
fn get_cell_string(cell: Option<&Data>) -> String {
    let Some(c) = cell else {
        return String::new();
    };
    match c {
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => f.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => {
            let f = dt.as_f64();
            let days = f as i64;
            if let Some(base_date) = chrono::NaiveDate::from_ymd_opt(1899, 12, 30) {
                if let Some(date) = base_date.checked_add_signed(chrono::Duration::days(days)) {
                    return date.format("%Y-%m-%d").to_string();
                }
            }
            f.to_string()
        }
        _ => String::new(),
    }
}

/// 將 Excel 儲存格轉為 Option<String>（空字串回傳 None）
fn cell_to_option(cell: Option<&Data>) -> Option<String> {
    let s = get_cell_string(cell);
    if s.is_empty() { None } else { Some(s) }
}

// ============================================
// 基本資料行驗證
// ============================================

/// 驗證單行基本資料，回傳解析後的結構或錯誤
fn validate_basic_row(
    row: &AnimalImportRow,
    row_number: i32,
    seen_ear_tags: &mut std::collections::HashSet<String>,
) -> std::result::Result<ValidatedBasicRow, ImportErrorDetail> {
    if row.ear_tag.is_empty() {
        return Err(import_err(row_number, None, "耳號為必填欄位".into()));
    }
    if !seen_ear_tags.insert(row.ear_tag.clone()) {
        return Err(import_err(row_number, Some(row.ear_tag.clone()), "耳號在檔案中重複".into()));
    }

    let (breed, raw_breed_other) = parse_import_breed(&row.breed);

    let gender = parse_import_gender(&row.gender).ok_or_else(|| {
        import_err(
            row_number,
            Some(row.ear_tag.clone()),
            format!("無效的性別值: {}，必須是 male/female 或 m/f (或公/母)", row.gender),
        )
    })?;

    let entry_date = parse_date_field(&row.entry_date).ok_or_else(|| {
        import_err(
            row_number,
            Some(row.ear_tag.clone()),
            format!("無效的進場日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式", row.entry_date),
        )
    })?;

    let birth_date = parse_optional_date(row, row_number)?;

    let entry_weight = parse_entry_weight(row, row_number)?;

    let pen_location = resolve_pen_location(row);
    if pen_location.is_none() || pen_location.as_ref().is_some_and(|s| s.is_empty()) {
        return Err(import_err(
            row_number,
            Some(row.ear_tag.clone()),
            "欄位為必填，請填寫 Field Region 或 Field Number".into(),
        ));
    }

    let breed_other = resolve_breed_other(&breed, raw_breed_other, &row.breed_other);

    Ok(ValidatedBasicRow {
        formatted_ear_tag: format_ear_tag(&row.ear_tag),
        breed,
        breed_other,
        gender,
        entry_date,
        birth_date,
        entry_weight,
        pen_location,
        pre_experiment_code: row.pre_experiment_code.clone(),
        remark: row.remark.clone(),
        source_code: row.source_code.clone(),
        iacuc_no: row.iacuc_no.clone(),
    })
}

/// 解析選填出生日期
fn parse_optional_date(
    row: &AnimalImportRow,
    row_number: i32,
) -> std::result::Result<Option<chrono::NaiveDate>, ImportErrorDetail> {
    let Some(ref bd) = row.birth_date else {
        return Ok(None);
    };
    if bd.is_empty() {
        return Ok(None);
    }
    parse_date_field(bd).map(Some).ok_or_else(|| {
        import_err(
            row_number,
            Some(row.ear_tag.clone()),
            format!("無效的出生日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式", bd),
        )
    })
}

/// 解析必填進場體重
fn parse_entry_weight(
    row: &AnimalImportRow,
    row_number: i32,
) -> std::result::Result<rust_decimal::Decimal, ImportErrorDetail> {
    row.entry_weight
        .as_ref()
        .and_then(|w| parse_weight_value(w))
        .map(|w| rust_decimal::Decimal::from_f64_retain(w).unwrap_or_default())
        .ok_or_else(|| {
            import_err(
                row_number,
                Some(row.ear_tag.clone()),
                "進場體重為必填欄位且必須是大於 0 的數字".into(),
            )
        })
}

/// 組合欄位位置
fn resolve_pen_location(row: &AnimalImportRow) -> Option<String> {
    row.pen_location
        .clone()
        .or_else(|| match (&row.field_region, &row.field_number) {
            (Some(r), Some(n)) => Some(format!("{}{}", r, n)),
            (Some(r), None) => Some(r.clone()),
            (None, Some(n)) => Some(n.clone()),
            (None, None) => None,
        })
        .map(|s| AnimalUtils::format_pen_location(&s))
}

/// 決定 breed_other 值
fn resolve_breed_other(
    breed: &AnimalBreed,
    raw_breed_other: Option<String>,
    explicit_breed_other: &Option<String>,
) -> Option<String> {
    if !matches!(breed, AnimalBreed::Other) {
        return None;
    }
    raw_breed_other.or_else(|| {
        explicit_breed_other.as_ref().and_then(|s| {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        })
    })
}

// ============================================
// 基本資料行 DB 處理
// ============================================

/// 處理已驗證的基本資料行（DB 查詢 + 建立動物）
async fn process_basic_row(
    pool: &PgPool,
    validated: ValidatedBasicRow,
    row_number: i32,
    original_ear_tag: &str,
    created_by: Uuid,
) -> std::result::Result<(), ImportErrorDetail> {
    // 查找來源 ID
    let source_id = find_source_id(pool, &validated.source_code).await.map_err(|e| {
        import_err(row_number, Some(original_ear_tag.to_string()), format!("查詢來源失敗: {}", e))
    })?;

    // 檢查耳號是否已存在
    let existing = find_animal_id_by_ear_tag(pool, &validated.formatted_ear_tag)
        .await
        .map_err(|e| {
            import_err(row_number, Some(validated.formatted_ear_tag.clone()), format!("查詢失敗: {}", e))
        })?;
    if existing.is_some() {
        return Err(import_err(
            row_number,
            Some(validated.formatted_ear_tag),
            "耳號已存在於系統中".into(),
        ));
    }

    let create_req = build_create_request(&validated, source_id);

    let animal = AnimalService::create(pool, &create_req, created_by)
        .await
        .map_err(|e| {
            import_err(row_number, Some(original_ear_tag.to_string()), format!("建立失敗: {}", e))
        })?;

    // 如有計畫編號則更新
    update_iacuc_if_present(pool, animal.id, &validated.iacuc_no, created_by).await;

    Ok(())
}

/// 查找動物來源 ID
async fn find_source_id(pool: &PgPool, source_code: &Option<String>) -> Result<Option<Uuid>> {
    let Some(ref code) = source_code else {
        return Ok(None);
    };
    if code.is_empty() {
        return Ok(None);
    }
    let source = sqlx::query_as::<_, AnimalSource>(
        "SELECT * FROM animal_sources WHERE code = $1 AND is_active = true",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;
    Ok(source.map(|s| s.id))
}

/// 根據耳號查找動物 ID（僅查找未刪除的動物）
async fn find_animal_id_by_ear_tag(pool: &PgPool, ear_tag: &str) -> Result<Option<Uuid>> {
    let id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM animals WHERE ear_tag = $1 AND deleted_at IS NULL",
    )
    .bind(ear_tag)
    .fetch_optional(pool)
    .await?;
    Ok(id)
}

/// 組建 CreateAnimalRequest
fn build_create_request(
    validated: &ValidatedBasicRow,
    source_id: Option<Uuid>,
) -> CreateAnimalRequest {
    CreateAnimalRequest {
        ear_tag: validated.formatted_ear_tag.clone(),
        breed: validated.breed,
        breed_other: validated.breed_other.clone(),
        source_id,
        gender: validated.gender,
        birth_date: validated.birth_date,
        entry_date: validated.entry_date,
        entry_weight: validated.entry_weight,
        pen_location: validated.pen_location.clone(),
        pre_experiment_code: validated.pre_experiment_code.clone(),
        remark: validated.remark.clone(),
        pen_id: None,
        species_id: None,
        force_create: true,
    }
}

/// 如有 IACUC 編號則更新動物資料
async fn update_iacuc_if_present(
    pool: &PgPool,
    animal_id: Uuid,
    iacuc_no: &Option<String>,
    created_by: Uuid,
) {
    let Some(ref iacuc) = iacuc_no else { return };
    if iacuc.is_empty() {
        return;
    }
    if let Err(e) = AnimalService::update(
        pool,
        animal_id,
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

// ============================================
// 體重資料行驗證
// ============================================

/// 驗證單行體重資料
fn validate_weight_row(
    row: &WeightImportRow,
    row_number: i32,
) -> std::result::Result<ValidatedWeightRow, ImportErrorDetail> {
    if row.ear_tag.is_empty() {
        return Err(import_err(row_number, None, "耳號為必填欄位".into()));
    }

    let measure_date = parse_date_field(&row.measure_date).ok_or_else(|| {
        import_err(
            row_number,
            Some(row.ear_tag.clone()),
            format!("無效的測量日期格式: {}，必須是 YYYY-MM-DD 或 YYYY/MM/DD 格式", row.measure_date),
        )
    })?;

    let weight_val = parse_weight_value(&row.weight).ok_or_else(|| {
        import_err(
            row_number,
            Some(row.ear_tag.clone()),
            format!("無效的體重值: {}，必須是大於 0 的數字", row.weight),
        )
    })?;

    Ok(ValidatedWeightRow {
        formatted_ear_tag: format_ear_tag(&row.ear_tag),
        measure_date,
        weight: rust_decimal::Decimal::from_f64_retain(weight_val).unwrap_or_default(),
    })
}

/// 處理已驗證的體重資料行（DB 查詢 + 建立體重紀錄）
async fn process_weight_row(
    pool: &PgPool,
    validated: ValidatedWeightRow,
    row_number: i32,
    original_ear_tag: &str,
    created_by: Uuid,
) -> std::result::Result<(), ImportErrorDetail> {
    let animal_id = find_animal_id_by_ear_tag(pool, &validated.formatted_ear_tag)
        .await
        .map_err(|e| {
            import_err(row_number, Some(original_ear_tag.to_string()), format!("查詢失敗: {}", e))
        })?
        .ok_or_else(|| {
            import_err(row_number, Some(original_ear_tag.to_string()), "找不到對應的動物".into())
        })?;

    let create_req = CreateWeightRequest {
        measure_date: validated.measure_date,
        weight: validated.weight,
    };

    AnimalWeightService::create(pool, animal_id, &create_req, created_by)
        .await
        .map_err(|e| {
            import_err(row_number, Some(original_ear_tag.to_string()), format!("建立失敗: {}", e))
        })?;

    Ok(())
}

// ============================================
// 檔案格式判斷
// ============================================

/// 判斷檔案格式，回傳 (is_excel, is_csv)
fn detect_file_format(file_name: &str) -> Result<(bool, bool)> {
    let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
    let is_csv = file_name.ends_with(".csv");
    if !is_excel && !is_csv {
        return Err(AppError::Validation(
            "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
        ));
    }
    Ok((is_excel, is_csv))
}

impl AnimalImportExportService {
    // ============================================
    // 模板生成功能
    // ============================================

    /// 生成動物基本資料匯入模板
    pub fn generate_basic_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();

        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let example_format = Format::new().set_font_color("#808080").set_italic();

        let worksheet = workbook.add_worksheet();
        Self::write_basic_template_headers(worksheet, &header_format)?;
        Self::write_basic_template_example(worksheet, &example_format)?;
        worksheet.set_freeze_panes(1, 0)?;

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }

    /// 寫入基本資料模板標題行
    fn write_basic_template_headers(
        worksheet: &mut rust_xlsxwriter::Worksheet,
        fmt: &rust_xlsxwriter::Format,
    ) -> Result<()> {
        let headers = [
            (0, 15.0, "耳號*"),
            (1, 12.0, "品種*"),
            (2, 10.0, "性別*"),
            (3, 15.0, "出生日期"),
            (4, 15.0, "進場日期*"),
            (5, 12.0, "進場體重*"),
            (6, 12.0, "欄位編號"),
            (7, 15.0, "實驗前代號"),
            (8, 12.0, "備註"),
        ];
        for (col, width, title) in headers {
            worksheet.set_column_width(col, width)?;
            worksheet.write_string_with_format(0, col, title, fmt)?;
        }
        Ok(())
    }

    /// 寫入基本資料模板範例行
    fn write_basic_template_example(
        worksheet: &mut rust_xlsxwriter::Worksheet,
        fmt: &rust_xlsxwriter::Format,
    ) -> Result<()> {
        worksheet.write_string_with_format(1, 0, "001", fmt)?;
        worksheet.write_string(1, 1, "miniature")?;
        worksheet.write_string(1, 2, "male")?;
        worksheet.write_string_with_format(1, 3, "2024-01-15", fmt)?;
        worksheet.write_string_with_format(1, 4, "2024-02-01", fmt)?;
        worksheet.write_string(1, 5, "25.5")?;
        worksheet.write_string(1, 6, "A-01")?;
        worksheet.write_string(1, 7, "PRE-001")?;
        worksheet.write_string(1, 8, "")?;
        Ok(())
    }

    /// 生成動物體重匯入模板
    pub fn generate_weight_import_template() -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut workbook = Workbook::new();

        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let example_format = Format::new().set_font_color("#808080").set_italic();

        let worksheet = workbook.add_worksheet();

        let headers = [(0, 15.0, "耳號*"), (1, 15.0, "測量日期*"), (2, 12.0, "體重(kg)*")];
        for (col, width, title) in headers {
            worksheet.set_column_width(col, width)?;
            worksheet.write_string_with_format(0, col, title, &header_format)?;
        }

        worksheet.write_string_with_format(1, 0, "001", &example_format)?;
        worksheet.write_string_with_format(1, 1, "2024-02-01", &example_format)?;
        worksheet.write_string(1, 2, "25.5")?;

        worksheet.set_freeze_panes(1, 0)?;

        let buffer = workbook.save_to_buffer()?;
        Ok(buffer)
    }

    /// 生成動物基本資料匯入 CSV 模板
    pub fn generate_basic_import_template_csv() -> Result<Vec<u8>> {
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

        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record([
            "耳號*", "品種*", "性別*", "出生日期", "進場日期*", "進場體重*", "欄位編號",
            "實驗前代號", "備註",
        ])
        .map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;

        wtr.write_record([
            "001", "miniature", "male", "2024-01-15", "2024-02-01", "25.5", "A-01", "PRE-001", "",
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

        wtr.write_record(["耳號*", "測量日期*", "體重(kg)*"])
            .map_err(|e| AppError::Internal(format!("CSV 寫入失敗: {}", e)))?;

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
        let (is_excel, _) = detect_file_format(file_name)?;

        let rows = if is_excel {
            Self::parse_excel_file(file_data)?
        } else {
            Self::parse_csv_file(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        let batch = AnimalMedicalService::create_import_batch(
            pool,
            ImportType::AnimalBasic,
            file_name,
            rows.len() as i32,
            created_by,
        )
        .await?;

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();
        let mut seen_ear_tags = std::collections::HashSet::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            let validated = match validate_basic_row(row, row_number, &mut seen_ear_tags) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(e);
                    error_count += 1;
                    continue;
                }
            };

            match process_basic_row(pool, validated, row_number, &row.ear_tag, created_by).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    errors.push(e);
                    error_count += 1;
                }
            }
        }

        finalize_import_batch(pool, batch.id, rows.len() as i32, success_count, error_count, errors)
            .await
    }

    /// 匯入體重資料
    pub async fn import_weight_data(
        pool: &PgPool,
        file_data: &[u8],
        file_name: &str,
        created_by: Uuid,
    ) -> Result<ImportResult> {
        let (is_excel, _) = detect_file_format(file_name)?;

        let rows = if is_excel {
            Self::parse_weight_excel_file(file_data)?
        } else {
            Self::parse_weight_csv_file(file_data)?
        };

        if rows.is_empty() {
            return Err(AppError::Validation("檔案中沒有資料".to_string()));
        }

        let batch = AnimalMedicalService::create_import_batch(
            pool,
            ImportType::AnimalWeight,
            file_name,
            rows.len() as i32,
            created_by,
        )
        .await?;

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, row) in rows.iter().enumerate() {
            let row_number = (index + 2) as i32;

            let validated = match validate_weight_row(row, row_number) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(e);
                    error_count += 1;
                    continue;
                }
            };

            match process_weight_row(pool, validated, row_number, &row.ear_tag, created_by).await {
                Ok(()) => success_count += 1,
                Err(e) => {
                    errors.push(e);
                    error_count += 1;
                }
            }
        }

        finalize_import_batch(pool, batch.id, rows.len() as i32, success_count, error_count, errors)
            .await
    }

    // ============================================
    // 檔案解析
    // ============================================

    /// 解析 Excel 檔案（基本資料）
    fn parse_excel_file(file_data: &[u8]) -> Result<Vec<AnimalImportRow>> {
        let range = open_excel_range(file_data)?;
        let mut rows = Vec::new();
        let mut iter = range.rows();
        iter.next(); // 跳過標題行

        for row in iter {
            if let Some(parsed) = Self::parse_basic_excel_row(row) {
                rows.push(parsed);
            }
        }
        Ok(rows)
    }

    /// 解析單行 Excel 基本資料
    fn parse_basic_excel_row(row: &[Data]) -> Option<AnimalImportRow> {
        if row.len() < 4 {
            return None;
        }

        let ear_tag = get_cell_string(row.first());
        let breed = get_cell_string(row.get(1));
        let gender = get_cell_string(row.get(2));
        let entry_date = get_cell_string(row.get(4));

        if ear_tag.is_empty() || breed.is_empty() || gender.is_empty() || entry_date.is_empty() {
            return None;
        }

        let entry_weight = row.get(5).and_then(|c| match c {
            Data::Float(f) => Some(*f),
            Data::Int(i) => Some(*i as f64),
            _ => None,
        });

        Some(AnimalImportRow {
            ear_tag,
            breed,
            breed_other: cell_to_option(row.get(9)),
            gender,
            source_code: None,
            birth_date: cell_to_option(row.get(3)),
            entry_date,
            entry_weight: entry_weight.map(|w| w.to_string()),
            pen_location: cell_to_option(row.get(6)),
            pre_experiment_code: cell_to_option(row.get(7)),
            remark: cell_to_option(row.get(8)),
            iacuc_no: None,
            field_region: None,
            field_number: None,
        })
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
                Ok(row) if !row.ear_tag.is_empty() => rows.push(row),
                Ok(_) => {}
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
        let range = open_excel_range(file_data)?;
        let mut rows = Vec::new();
        let mut iter = range.rows();
        iter.next(); // 跳過標題行

        for row in iter {
            if let Some(parsed) = Self::parse_weight_excel_row(row) {
                rows.push(parsed);
            }
        }
        Ok(rows)
    }

    /// 解析單行 Excel 體重資料
    fn parse_weight_excel_row(row: &[Data]) -> Option<WeightImportRow> {
        if row.len() < 3 {
            return None;
        }

        let ear_tag = get_cell_string(row.first()).trim().to_string();
        let measure_date = get_cell_string(row.get(1)).trim().to_string();
        let weight = row.get(2).and_then(|c| match c {
            Data::Float(f) => Some(*f),
            Data::Int(i) => Some(*i as f64),
            _ => None,
        })?;

        if ear_tag.is_empty() || measure_date.is_empty() || weight <= 0.0 {
            return None;
        }

        Some(WeightImportRow {
            ear_tag,
            measure_date,
            weight: weight.to_string(),
        })
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
                Ok(row) if !row.ear_tag.is_empty() => rows.push(row),
                Ok(_) => {}
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_import_breed ──

    #[test]
    fn test_breed_minipig_variants() {
        assert_eq!(parse_import_breed("minipig").0, AnimalBreed::Minipig);
        assert_eq!(parse_import_breed("Mini").0, AnimalBreed::Minipig);
        assert_eq!(parse_import_breed("迷你豬").0, AnimalBreed::Minipig);
        assert_eq!(parse_import_breed("M").0, AnimalBreed::Minipig);
    }

    #[test]
    fn test_breed_white_variants() {
        assert_eq!(parse_import_breed("white").0, AnimalBreed::White);
        assert_eq!(parse_import_breed("W").0, AnimalBreed::White);
        assert_eq!(parse_import_breed("白豬").0, AnimalBreed::White);
        assert_eq!(parse_import_breed("大白豬").0, AnimalBreed::White);
    }

    #[test]
    fn test_breed_unknown_returns_other_with_name() {
        let (breed, other) = parse_import_breed("Duroc");
        assert_eq!(breed, AnimalBreed::Other);
        assert_eq!(other, Some("Duroc".to_string()));
    }

    #[test]
    fn test_breed_empty_returns_other_none() {
        let (breed, other) = parse_import_breed("");
        assert_eq!(breed, AnimalBreed::Other);
        assert_eq!(other, None);
    }

    // ── parse_import_gender ──

    #[test]
    fn test_gender_male_variants() {
        assert_eq!(parse_import_gender("male"), Some(AnimalGender::Male));
        assert_eq!(parse_import_gender("M"), Some(AnimalGender::Male));
        assert_eq!(parse_import_gender("公"), Some(AnimalGender::Male));
        assert_eq!(parse_import_gender("公豬"), Some(AnimalGender::Male));
    }

    #[test]
    fn test_gender_female_variants() {
        assert_eq!(parse_import_gender("female"), Some(AnimalGender::Female));
        assert_eq!(parse_import_gender("F"), Some(AnimalGender::Female));
        assert_eq!(parse_import_gender("母"), Some(AnimalGender::Female));
        assert_eq!(parse_import_gender("母豬"), Some(AnimalGender::Female));
    }

    #[test]
    fn test_gender_unknown_returns_none() {
        assert_eq!(parse_import_gender("other"), None);
        assert_eq!(parse_import_gender(""), None);
    }

    // ── parse_date_field ──

    #[test]
    fn test_date_dash_format() {
        let d = parse_date_field("2025-03-15");
        assert_eq!(d, Some(chrono::NaiveDate::from_ymd_opt(2025, 3, 15).expect("valid date")));
    }

    #[test]
    fn test_date_slash_format() {
        let d = parse_date_field("2025/03/15");
        assert_eq!(d, Some(chrono::NaiveDate::from_ymd_opt(2025, 3, 15).expect("valid date")));
    }

    #[test]
    fn test_date_invalid_returns_none() {
        assert_eq!(parse_date_field("not-a-date"), None);
        assert_eq!(parse_date_field(""), None);
        assert_eq!(parse_date_field("2025-13-01"), None);
    }

    // ── parse_weight_value ──

    #[test]
    fn test_weight_plain_number() {
        assert_eq!(parse_weight_value("25.5"), Some(25.5));
    }

    #[test]
    fn test_weight_with_kg_suffix() {
        assert_eq!(parse_weight_value("30kg"), Some(30.0));
    }

    #[test]
    fn test_weight_with_chinese_suffix() {
        assert_eq!(parse_weight_value("45.2公斤"), Some(45.2));
    }

    #[test]
    fn test_weight_zero_returns_none() {
        assert_eq!(parse_weight_value("0"), None);
    }

    #[test]
    fn test_weight_negative_returns_none() {
        assert_eq!(parse_weight_value("-5"), None);
    }

    #[test]
    fn test_weight_invalid_returns_none() {
        assert_eq!(parse_weight_value("abc"), None);
        assert_eq!(parse_weight_value(""), None);
    }

    // ── format_ear_tag (local version) ──

    #[test]
    fn test_local_ear_tag_pads_all_numbers() {
        // 與 AnimalUtils::format_ear_tag 不同：這裡所有數字都補零
        assert_eq!(format_ear_tag("5"), "005");
        assert_eq!(format_ear_tag("100"), "100");
        assert_eq!(format_ear_tag("999"), "999");
    }

    #[test]
    fn test_local_ear_tag_non_numeric() {
        assert_eq!(format_ear_tag("abc"), "abc");
    }
}
