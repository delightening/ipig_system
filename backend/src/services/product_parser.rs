//! 產品匯入檔案解析模組
//!
//! 將 CSV 與 Excel 解析邏輯從 `ProductService` 提取至此，
//! 降低 `product.rs` 的函數長度，符合 ≤50 行規範。

use calamine::{open_workbook_from_rs, Data, Reader, Xls, Xlsx};
use std::io::Cursor;

use crate::{
    models::{ProductImportPreviewRow, ProductImportRow},
    AppError, Result,
};

/// 解析匯入檔中的布林欄位（追蹤批號、追蹤效期等）。
pub(crate) fn parse_bool(s: &str) -> bool {
    let s = s.trim().to_lowercase();
    matches!(s.as_str(), "true" | "1" | "yes" | "是" | "y")
}

/// 從 Excel 儲存格取得字串值。
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

/// 依表頭名稱找欄位索引（忽略空白、大小寫）。
fn csv_header_index(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    let key = name.trim().to_lowercase();
    if key.is_empty() {
        return None;
    }
    headers.iter().position(|h| h.trim().to_lowercase() == key)
}

/// 將「分類」欄的顯示名稱對應為品類代碼（耗材→CON、藥品→DRG 等）；已是代碼則回傳原值。
fn map_category_display_to_code(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let normalized = t.to_uppercase();
    if normalized.len() >= 2
        && normalized.len() <= 4
        && normalized.chars().all(|c| c.is_ascii_alphabetic())
    {
        return Some(normalized);
    }
    let mapping: &[(&str, &str)] = &[
        ("耗材", "CON"),
        ("藥品", "DRG"),
        ("醫材", "MED"),
        ("化學品", "CHM"),
        ("設備", "EQP"),
    ];
    let lower = t.to_lowercase();
    for (name, code) in mapping {
        if lower.contains(&name.to_lowercase()) {
            return Some((*code).to_string());
        }
    }
    Some(t.to_string())
}

/// 是否為「庫存清表／重構」格式：表頭含「品名」或「名稱」，且含「規格」。
fn is_stocklist_format(headers: &csv::StringRecord) -> bool {
    let has_name =
        csv_header_index(headers, "品名").is_some() || csv_header_index(headers, "名稱").is_some();
    has_name && csv_header_index(headers, "規格").is_some()
}

/// 解析 CSV 欄位索引配置。
struct CsvColumnIndices {
    name: usize,
    spec: usize,
    cat: usize,
    subcat: usize,
    uom: usize,
    batch: usize,
    expiry: usize,
    stock: usize,
    remark: usize,
}

/// 解析庫存清表格式的 CSV 欄位索引。
fn resolve_stocklist_indices(headers: &csv::StringRecord) -> Result<CsvColumnIndices> {
    let name = csv_header_index(headers, "品名")
        .or_else(|| csv_header_index(headers, "名稱"))
        .ok_or_else(|| {
            AppError::Validation("庫存清表格式 CSV 需有「品名」或「名稱」欄位".to_string())
        })?;
    let spec = csv_header_index(headers, "規格").ok_or_else(|| {
        AppError::Validation("庫存清表格式 CSV 需有「規格」欄位".to_string())
    })?;
    let uom = csv_header_index(headers, "單位").unwrap_or_else(|| spec.saturating_add(1));
    let remark = csv_header_index(headers, "製造廠商")
        .or_else(|| csv_header_index(headers, "包裝規格"))
        .unwrap_or(0);
    let cat = csv_header_index(headers, "品類代碼")
        .or_else(|| csv_header_index(headers, "分類"))
        .unwrap_or(usize::MAX);
    let subcat = csv_header_index(headers, "子類代碼").unwrap_or(usize::MAX);
    Ok(CsvColumnIndices {
        name,
        spec,
        cat,
        subcat,
        uom,
        batch: usize::MAX,
        expiry: usize::MAX,
        stock: usize::MAX,
        remark,
    })
}

/// 解析一般格式（無 SKU 欄位）的 CSV 欄位索引。
fn resolve_general_indices(headers: &csv::StringRecord) -> CsvColumnIndices {
    CsvColumnIndices {
        name: csv_header_index(headers, "名稱")
            .or_else(|| csv_header_index(headers, "品名"))
            .unwrap_or(0),
        spec: csv_header_index(headers, "規格").unwrap_or(1),
        cat: csv_header_index(headers, "品類代碼")
            .or_else(|| csv_header_index(headers, "分類"))
            .unwrap_or(2),
        subcat: csv_header_index(headers, "子類代碼").unwrap_or(3),
        uom: csv_header_index(headers, "單位").unwrap_or(4),
        batch: csv_header_index(headers, "追蹤批號").unwrap_or(5),
        expiry: csv_header_index(headers, "追蹤效期").unwrap_or(6),
        stock: csv_header_index(headers, "安全庫存").unwrap_or(7),
        remark: csv_header_index(headers, "備註").unwrap_or(8),
    }
}

/// 從 CSV record 解析單列資料為 `ProductImportRow`。
fn parse_csv_row(
    record: &csv::StringRecord,
    idx: &CsvColumnIndices,
    has_sku: bool,
    is_stocklist: bool,
) -> Option<ProductImportRow> {
    let name = record.get(idx.name).unwrap_or("").to_string();
    if name.trim().is_empty() {
        return None;
    }
    let sku = if has_sku && !is_stocklist {
        record
            .get(0)
            .and_then(|s| {
                let t = s.trim();
                if t.is_empty() { None } else { Some(t.to_string()) }
            })
    } else {
        None
    };
    let spec = record
        .get(idx.spec)
        .filter(|s| !s.trim().is_empty())
        .map(String::from);
    let category_code = if idx.cat < record.len() {
        record.get(idx.cat).and_then(map_category_display_to_code)
    } else {
        None
    };
    let subcategory_code = if idx.subcat < record.len() && idx.subcat != idx.cat {
        record
            .get(idx.subcat)
            .filter(|s| !s.trim().is_empty())
            .map(String::from)
    } else {
        None
    };
    let base_uom = if idx.uom < record.len() {
        let u = record.get(idx.uom).map(|s| s.trim().to_string()).unwrap_or_default();
        if u.is_empty() { "PCS".to_string() } else { u }
    } else {
        "PCS".to_string()
    };
    let track_batch = if idx.batch < record.len() {
        parse_bool(record.get(idx.batch).unwrap_or(""))
    } else {
        true
    };
    let track_expiry = if idx.expiry < record.len() {
        parse_bool(record.get(idx.expiry).unwrap_or(""))
    } else {
        true
    };
    let safety_stock = if idx.stock < record.len() {
        record
            .get(idx.stock)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .and_then(rust_decimal::Decimal::from_f64_retain)
    } else {
        None
    };
    let remark = if idx.remark < record.len() {
        record.get(idx.remark).filter(|s| !s.trim().is_empty()).map(String::from)
    } else {
        None
    };

    Some(ProductImportRow {
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
    })
}

/// 解析產品匯入 CSV，回傳 (列資料, 是否含 SKU 欄位)。
pub fn parse_product_csv(file_data: &[u8]) -> Result<(Vec<ProductImportRow>, bool)> {
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

    let is_stocklist = is_stocklist_format(headers);
    let indices = if is_stocklist {
        resolve_stocklist_indices(headers)?
    } else if has_sku_column {
        CsvColumnIndices {
            name: 1, spec: 2, cat: 3, subcat: 4, uom: 5,
            batch: 6, expiry: 7, stock: 8, remark: 9,
        }
    } else {
        resolve_general_indices(headers)
    };

    let min_cols_stocklist = indices.name.max(indices.spec) + 1;
    let mut rows = Vec::new();
    for (i, result) in reader.records().enumerate() {
        let record = result.map_err(|e| {
            AppError::Validation(format!("CSV 解析錯誤第 {} 行: {}", i + 2, e))
        })?;
        if is_stocklist && record.len() < min_cols_stocklist {
            continue;
        }
        if !is_stocklist
            && (if has_sku_column { record.len() < 3 } else { record.len() < 2 })
        {
            continue;
        }
        if let Some(row) = parse_csv_row(&record, &indices, has_sku_column, is_stocklist) {
            rows.push(row);
        }
    }
    Ok((rows, has_sku_column && !is_stocklist))
}

/// 從 Excel 表頭解析欄位偏移。
fn resolve_excel_offsets(has_sku: bool) -> (usize, usize, usize, usize, usize, usize, usize, usize, usize) {
    if has_sku {
        (1, 2, 3, 4, 5, 6, 7, 8, 9)
    } else {
        (0, 1, 2, 3, 4, 5, 6, 7, 8)
    }
}

/// 從 Excel 列解析單筆 `ProductImportRow`。
fn parse_excel_row(row: &[Data], has_sku: bool) -> Option<ProductImportRow> {
    let (name_col, spec_col, cat_col, subcat_col, uom_col, batch_col, expiry_col, stock_col, remark_col) =
        resolve_excel_offsets(has_sku);

    let name = get_cell_string(row.get(name_col));
    if name.trim().is_empty() {
        return None;
    }
    let sku = if has_sku {
        let s = get_cell_string(row.first());
        if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }
    } else {
        None
    };
    let opt_str = |col: usize| -> Option<String> {
        let s = get_cell_string(row.get(col));
        if s.is_empty() { None } else { Some(s) }
    };
    let base_uom = get_cell_string(row.get(uom_col));
    let base_uom = if base_uom.is_empty() { "PCS".to_string() } else { base_uom };
    let safety_stock = row.get(stock_col).and_then(|c| match c {
        Data::Float(f) => rust_decimal::Decimal::from_f64_retain(*f),
        Data::Int(i) => rust_decimal::Decimal::from_f64_retain(*i as f64),
        _ => None,
    });

    Some(ProductImportRow {
        sku,
        name,
        spec: opt_str(spec_col),
        category_code: opt_str(cat_col),
        subcategory_code: opt_str(subcat_col),
        base_uom,
        track_batch: parse_bool(&get_cell_string(row.get(batch_col))),
        track_expiry: parse_bool(&get_cell_string(row.get(expiry_col))),
        safety_stock,
        remark: opt_str(remark_col),
    })
}

/// 開啟 Excel 工作簿並取得第一張工作表的資料範圍。
fn open_excel_range(file_data: &[u8]) -> Result<calamine::Range<Data>> {
    let cursor = Cursor::new(file_data);
    if let Ok(mut wb) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
        let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
            AppError::Validation("Excel 檔案中沒有工作表".to_string())
        })?;
        wb.worksheet_range(&sheet_name)
            .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))
    } else {
        let cursor = Cursor::new(file_data);
        let mut wb = open_workbook_from_rs::<Xls<_>, _>(cursor).map_err(|_| {
            AppError::Validation("無法讀取 Excel 檔案，請使用 .xlsx 或 .xls 格式".to_string())
        })?;
        let sheet_name = wb.sheet_names().first().cloned().ok_or_else(|| {
            AppError::Validation("Excel 檔案中沒有工作表".to_string())
        })?;
        wb.worksheet_range(&sheet_name)
            .map_err(|e| AppError::Validation(format!("無法讀取工作表: {}", e)))
    }
}

/// 解析產品匯入 Excel，回傳 (列資料, 是否含 SKU 欄位)。
pub fn parse_product_excel(file_data: &[u8]) -> Result<(Vec<ProductImportRow>, bool)> {
    let range = open_excel_range(file_data)?;
    let mut iter = range.rows();
    let header_row = iter
        .next()
        .ok_or_else(|| AppError::Validation("Excel 無標題列".to_string()))?;

    let has_sku_column = header_row.len() >= 10
        && get_cell_string(header_row.first())
            .trim()
            .to_uppercase()
            .contains("SKU");

    let min_cols = if has_sku_column { 3 } else { 2 };
    let mut rows = Vec::new();
    for row in iter {
        if row.len() < min_cols {
            continue;
        }
        if let Some(parsed) = parse_excel_row(row, has_sku_column) {
            rows.push(parsed);
        }
    }
    Ok((rows, has_sku_column))
}

/// 解析匯入檔案（自動偵測格式），回傳 (列資料, 是否含 SKU 欄位)。
pub fn parse_import_file(file_data: &[u8], file_name: &str) -> Result<(Vec<ProductImportRow>, bool)> {
    let is_excel = file_name.ends_with(".xlsx") || file_name.ends_with(".xls");
    let is_csv = file_name.ends_with(".csv");

    if !is_excel && !is_csv {
        return Err(AppError::Validation(
            "不支援的檔案格式，請使用 Excel (.xlsx, .xls) 或 CSV 格式".to_string(),
        ));
    }

    if is_excel {
        parse_product_excel(file_data)
    } else {
        parse_product_csv(file_data)
    }
}

/// 將 `ProductImportRow` 轉換為預覽行。
pub fn to_preview_row(index: usize, row: ProductImportRow) -> ProductImportPreviewRow {
    let row_number = (index + 2) as i32;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_bool ---

    #[test]
    fn test_parse_bool_truthy_values() {
        assert!(parse_bool("true"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("是"));
        assert!(parse_bool("y"));
        assert!(parse_bool("  True  "));
        assert!(parse_bool("YES"));
        assert!(parse_bool("Y"));
    }

    #[test]
    fn test_parse_bool_falsy_values() {
        assert!(!parse_bool("false"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("否"));
        assert!(!parse_bool(""));
        assert!(!parse_bool("   "));
        assert!(!parse_bool("random"));
    }

    // --- map_category_display_to_code ---

    #[test]
    fn test_map_category_code_passthrough() {
        assert_eq!(map_category_display_to_code("CON"), Some("CON".to_string()));
        assert_eq!(map_category_display_to_code("DRG"), Some("DRG".to_string()));
        assert_eq!(map_category_display_to_code("med"), Some("MED".to_string()));
    }

    #[test]
    fn test_map_category_display_name() {
        assert_eq!(map_category_display_to_code("耗材"), Some("CON".to_string()));
        assert_eq!(map_category_display_to_code("藥品"), Some("DRG".to_string()));
        assert_eq!(map_category_display_to_code("醫材"), Some("MED".to_string()));
        assert_eq!(map_category_display_to_code("化學品"), Some("CHM".to_string()));
        assert_eq!(map_category_display_to_code("設備"), Some("EQP".to_string()));
    }

    #[test]
    fn test_map_category_empty_returns_none() {
        assert_eq!(map_category_display_to_code(""), None);
        assert_eq!(map_category_display_to_code("   "), None);
    }

    #[test]
    fn test_map_category_unknown_returns_as_is() {
        assert_eq!(map_category_display_to_code("未知分類"), Some("未知分類".to_string()));
    }

    // --- csv_header_index ---

    #[test]
    fn test_csv_header_index_found() {
        let headers = csv::StringRecord::from(vec!["名稱", "規格", "單位"]);
        assert_eq!(csv_header_index(&headers, "名稱"), Some(0));
        assert_eq!(csv_header_index(&headers, "規格"), Some(1));
        assert_eq!(csv_header_index(&headers, "單位"), Some(2));
    }

    #[test]
    fn test_csv_header_index_case_insensitive() {
        let headers = csv::StringRecord::from(vec!["Name", "SPEC"]);
        assert_eq!(csv_header_index(&headers, "name"), Some(0));
        assert_eq!(csv_header_index(&headers, "spec"), Some(1));
    }

    #[test]
    fn test_csv_header_index_not_found() {
        let headers = csv::StringRecord::from(vec!["名稱", "規格"]);
        assert_eq!(csv_header_index(&headers, "單位"), None);
    }

    #[test]
    fn test_csv_header_index_empty_name() {
        let headers = csv::StringRecord::from(vec!["名稱"]);
        assert_eq!(csv_header_index(&headers, ""), None);
        assert_eq!(csv_header_index(&headers, "  "), None);
    }

    // --- is_stocklist_format ---

    #[test]
    fn test_is_stocklist_format_true() {
        let headers = csv::StringRecord::from(vec!["品名", "規格", "單位"]);
        assert!(is_stocklist_format(&headers));

        let headers = csv::StringRecord::from(vec!["名稱", "規格", "備註"]);
        assert!(is_stocklist_format(&headers));
    }

    #[test]
    fn test_is_stocklist_format_false() {
        let headers = csv::StringRecord::from(vec!["品名", "單位"]);
        assert!(!is_stocklist_format(&headers));

        let headers = csv::StringRecord::from(vec!["SKU", "規格"]);
        assert!(!is_stocklist_format(&headers));
    }

    // --- parse_product_csv ---

    #[test]
    fn test_parse_product_csv_stocklist_format() {
        // 含「名稱」+「規格」表頭 → 被偵測為庫存清表格式
        // 庫存清表模式下 track_batch / track_expiry 預設為 true
        let csv = "名稱,規格,品類代碼,子類代碼,單位\n\
                   酒精,75%,CON,,瓶\n\
                   棉花,,MED,,包";
        let (rows, has_sku) = parse_product_csv(csv.as_bytes()).expect("parse should succeed");
        assert!(!has_sku);
        assert_eq!(rows.len(), 2);

        assert_eq!(rows[0].name, "酒精");
        assert_eq!(rows[0].spec.as_deref(), Some("75%"));
        assert_eq!(rows[0].category_code.as_deref(), Some("CON"));
        assert_eq!(rows[0].base_uom, "瓶");
        assert!(rows[0].track_batch);  // stocklist default
        assert!(rows[0].track_expiry); // stocklist default

        assert_eq!(rows[1].name, "棉花");
        assert_eq!(rows[1].base_uom, "包");
    }

    #[test]
    fn test_parse_product_csv_general_format() {
        // 使用與 resolve_general_indices 一致的中文表頭，驗證表頭名稱匹配邏輯。
        // 「產品規格」≠「規格」→ 不觸發庫存清表偵測，但名稱/品類代碼/單位/追蹤批號/追蹤效期/備註
        // 等欄位均透過 csv_header_index 匹配。
        let csv = "名稱,產品規格,品類代碼,子類代碼,單位,追蹤批號,追蹤效期,安全庫存,備註\n\
                   酒精,75%,CON,,瓶,true,false,10,消毒用\n\
                   棉花,,MED,,包,false,false,,";
        let (rows, has_sku) = parse_product_csv(csv.as_bytes()).expect("parse should succeed");
        assert!(!has_sku);
        assert_eq!(rows.len(), 2);

        assert_eq!(rows[0].name, "酒精");
        assert_eq!(rows[0].spec.as_deref(), Some("75%"));
        assert_eq!(rows[0].category_code.as_deref(), Some("CON"));
        assert_eq!(rows[0].base_uom, "瓶");
        assert!(rows[0].track_batch);
        assert!(!rows[0].track_expiry);
        assert_eq!(rows[0].safety_stock, rust_decimal::Decimal::from_f64_retain(10.0));
        assert_eq!(rows[0].remark.as_deref(), Some("消毒用"));

        assert_eq!(rows[1].name, "棉花");
        assert_eq!(rows[1].category_code.as_deref(), Some("MED"));
        assert_eq!(rows[1].base_uom, "包");
        assert!(!rows[1].track_batch);
        assert!(!rows[1].track_expiry);
    }

    #[test]
    fn test_parse_product_csv_skips_empty_names() {
        let csv = "名稱,規格\n\
                   有名字,規格A\n\
                   ,規格B\n\
                   另一個,規格C";
        let (rows, _) = parse_product_csv(csv.as_bytes()).expect("parse should succeed");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "有名字");
        assert_eq!(rows[1].name, "另一個");
    }

    // --- to_preview_row ---

    #[test]
    fn test_to_preview_row_index_mapping() {
        let row = ProductImportRow {
            name: "測試產品".to_string(),
            base_uom: "PCS".to_string(),
            track_batch: true,
            track_expiry: false,
            ..Default::default()
        };
        let preview = to_preview_row(0, row);
        assert_eq!(preview.row, 2); // index 0 → row 2 (header is row 1)
        assert_eq!(preview.name, "測試產品");

        let row2 = ProductImportRow {
            name: "產品二".to_string(),
            base_uom: "BOX".to_string(),
            ..Default::default()
        };
        let preview2 = to_preview_row(5, row2);
        assert_eq!(preview2.row, 7);
    }

    #[test]
    fn test_to_preview_row_preserves_fields() {
        let row = ProductImportRow {
            sku: Some("SKU-001".to_string()),
            name: "Test".to_string(),
            spec: Some("100ml".to_string()),
            category_code: Some("DRG".to_string()),
            subcategory_code: Some("ANT".to_string()),
            base_uom: "BTL".to_string(),
            track_batch: true,
            track_expiry: true,
            safety_stock: rust_decimal::Decimal::from_f64_retain(50.0),
            remark: Some("備註".to_string()),
        };
        let preview = to_preview_row(0, row);
        assert_eq!(preview.name, "Test");
        assert_eq!(preview.spec.as_deref(), Some("100ml"));
        assert_eq!(preview.category_code.as_deref(), Some("DRG"));
        assert_eq!(preview.base_uom, "BTL");
        assert!(preview.track_batch);
        assert!(preview.track_expiry);
        assert_eq!(preview.safety_stock, Some(50.0));
        assert_eq!(preview.remark.as_deref(), Some("備註"));
    }
}
