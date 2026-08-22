use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook};

use crate::{models::AvailablePigRow, AppError, Result};

/// R47 — 可用豬隻 Excel 匯出（9 欄）
///
/// 欄位：品種 / 耳號 / 性別 / 出生日 / 月齡 / 最近體重(kg) / 量測日 / 位置 / 備註
/// header bold + 凍結首列。回 xlsx 檔案 bytes。
/// 詳見 `docs/plans/r47-available-pigs-query.md` §2.6。
pub fn build_available_pigs_xlsx(rows: &[AvailablePigRow]) -> Result<Vec<u8>> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet
        .set_name("可用豬隻")
        .map_err(|e| AppError::Internal(format!("xlsx set_name: {e}")))?;

    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);
    let body_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let headers = [
        "品種",
        "耳號",
        "性別",
        "出生日",
        "月齡",
        "最近體重(kg)",
        "量測日",
        "位置",
        "備註",
    ];
    for (col, label) in headers.iter().enumerate() {
        sheet
            .write_string_with_format(0, col as u16, *label, &header_format)
            .map_err(|e| AppError::Internal(format!("xlsx header: {e}")))?;
    }

    // 凍結首列
    sheet
        .set_freeze_panes(1, 0)
        .map_err(|e| AppError::Internal(format!("xlsx freeze: {e}")))?;

    // 欄寬
    for (col, width) in [10.0, 8.0, 6.0, 12.0, 8.0, 14.0, 12.0, 10.0, 24.0]
        .iter()
        .enumerate()
    {
        sheet
            .set_column_width(col as u16, *width)
            .map_err(|e| AppError::Internal(format!("xlsx width: {e}")))?;
    }

    for (i, row) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        let pen = row.pen_location.as_deref().unwrap_or_default();
        let weight_str = format!("{:.1}", row.latest_weight_kg);

        sheet
            .write_string_with_format(r, 0, row.breed.display_name(), &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[breed]: {e}")))?;
        sheet
            .write_string_with_format(r, 1, &row.ear_tag, &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[ear_tag]: {e}")))?;
        sheet
            .write_string_with_format(r, 2, row.gender.display_name(), &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[gender]: {e}")))?;
        sheet
            .write_string_with_format(
                r,
                3,
                row.birth_date.format("%Y-%m-%d").to_string(),
                &body_format,
            )
            .map_err(|e| AppError::Internal(format!("xlsx body[birth_date]: {e}")))?;
        sheet
            .write_number_with_format(r, 4, row.age_months as f64, &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[age_months]: {e}")))?;
        sheet
            .write_string_with_format(r, 5, &weight_str, &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[weight]: {e}")))?;
        sheet
            .write_string_with_format(
                r,
                6,
                row.weight_measured_at.format("%Y-%m-%d").to_string(),
                &body_format,
            )
            .map_err(|e| AppError::Internal(format!("xlsx body[measured_at]: {e}")))?;
        sheet
            .write_string_with_format(r, 7, pen, &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[pen]: {e}")))?;
        let remark = row.remark.as_deref().unwrap_or_default();
        sheet
            .write_string_with_format(r, 8, remark, &body_format)
            .map_err(|e| AppError::Internal(format!("xlsx body[remark]: {e}")))?;
    }

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| AppError::Internal(format!("xlsx save: {e}")))?;
    Ok(bytes)
}
