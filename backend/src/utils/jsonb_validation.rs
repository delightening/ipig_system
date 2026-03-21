//! JSONB 結構驗證工具
//!
//! 針對使用者可輸入的 JSONB 欄位提供結構驗證，
//! 透過反序列化至具型 struct 確保資料格式正確。

use serde::Deserialize;

use crate::{AppError, Result};

// ============================================
// 觀察紀錄：treatments
// ============================================

/// 單筆給藥/治療項目
#[derive(Debug, Deserialize)]
pub struct TreatmentItem {
    pub drug: String,
    pub dosage: String,
    pub end_date: Option<String>,
}

/// 驗證 treatments JSONB（應為 TreatmentItem 陣列）
pub fn validate_treatments(value: &serde_json::Value) -> Result<()> {
    let items: Vec<TreatmentItem> = serde_json::from_value(value.clone()).map_err(|e| {
        AppError::Validation(format!(
            "Invalid treatments format: expected array of {{drug, dosage, end_date?}}. {}",
            e
        ))
    })?;

    for (i, item) in items.iter().enumerate() {
        if item.drug.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "treatments[{}].drug cannot be empty",
                i
            )));
        }
        if item.dosage.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "treatments[{}].dosage cannot be empty",
                i
            )));
        }
    }

    Ok(())
}

// ============================================
// 觀察紀錄：equipment_used
// ============================================

/// 驗證 equipment_used JSONB（應為字串陣列）
pub fn validate_equipment_used(value: &serde_json::Value) -> Result<()> {
    let items: Vec<String> = serde_json::from_value(value.clone()).map_err(|_| {
        AppError::Validation(
            "Invalid equipment_used format: expected array of strings".to_string(),
        )
    })?;

    for (i, item) in items.iter().enumerate() {
        if item.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "equipment_used[{}] cannot be empty",
                i
            )));
        }
    }

    Ok(())
}

// ============================================
// 手術紀錄：vital_signs
// ============================================

/// 單筆生命徵象紀錄
#[derive(Debug, Deserialize)]
pub struct VitalSignEntry {
    pub time: String,
    pub heart_rate: Option<f64>,
    pub respiration_rate: Option<f64>,
    pub temperature: Option<f64>,
    pub spo2: Option<f64>,
}

/// 驗證 vital_signs JSONB（應為 VitalSignEntry 陣列）
pub fn validate_vital_signs(value: &serde_json::Value) -> Result<()> {
    let _items: Vec<VitalSignEntry> = serde_json::from_value(value.clone()).map_err(|e| {
        AppError::Validation(format!(
            "Invalid vital_signs format: expected array of {{time, heart_rate?, respiration_rate?, temperature?, spo2?}}. {}",
            e
        ))
    })?;

    Ok(())
}

// ============================================
// 手術紀錄：藥物 JSONB 欄位通用驗證
// (induction_anesthesia, pre_surgery_medication,
//  anesthesia_maintenance, post_surgery_medication)
// ============================================

/// 驗證手術藥物 JSONB（接受 object 或 array of objects）
///
/// 這些欄位結構較彈性（前端使用 Record<string, unknown>），
/// 僅確保為合法 JSON object 或 array，不接受純 scalar 值。
pub fn validate_medication_jsonb(field_name: &str, value: &serde_json::Value) -> Result<()> {
    match value {
        serde_json::Value::Object(_) | serde_json::Value::Array(_) => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Invalid {} format: expected JSON object or array",
            field_name
        ))),
    }
}

// ============================================
// 獸醫建議附件
// ============================================

/// 附件項目
#[derive(Debug, Deserialize)]
pub struct AttachmentItem {
    pub file_name: String,
    pub file_path: String,
    pub file_type: Option<String>,
}

/// 驗證 attachments JSONB（應為 AttachmentItem 陣列）
pub fn validate_attachments(value: &serde_json::Value) -> Result<()> {
    let items: Vec<AttachmentItem> = serde_json::from_value(value.clone()).map_err(|e| {
        AppError::Validation(format!(
            "Invalid attachments format: expected array of {{file_name, file_path, file_type?}}. {}",
            e
        ))
    })?;

    for (i, item) in items.iter().enumerate() {
        if item.file_name.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "attachments[{}].file_name cannot be empty",
                i
            )));
        }
        if item.file_path.trim().is_empty() {
            return Err(AppError::Validation(format!(
                "attachments[{}].file_path cannot be empty",
                i
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_treatments() {
        let data = json!([
            {"drug": "Aspirin", "dosage": "100mg"},
            {"drug": "Ibuprofen", "dosage": "200mg", "end_date": "2026-04-01"}
        ]);
        assert!(validate_treatments(&data).is_ok());
    }

    #[test]
    fn test_treatments_empty_drug() {
        let data = json!([{"drug": "", "dosage": "100mg"}]);
        let err = validate_treatments(&data).expect_err("should reject empty drug");
        assert!(err.to_string().contains("drug cannot be empty"));
    }

    #[test]
    fn test_treatments_invalid_format() {
        let data = json!("not an array");
        let err = validate_treatments(&data).expect_err("should reject non-array");
        assert!(err.to_string().contains("Invalid treatments format"));
    }

    #[test]
    fn test_valid_equipment_used() {
        let data = json!(["Syringe", "Scalpel"]);
        assert!(validate_equipment_used(&data).is_ok());
    }

    #[test]
    fn test_equipment_used_invalid_type() {
        let data = json!([1, 2, 3]);
        let err = validate_equipment_used(&data).expect_err("should reject non-strings");
        assert!(err.to_string().contains("Invalid equipment_used format"));
    }

    #[test]
    fn test_valid_vital_signs() {
        let data = json!([
            {"time": "09:00", "heart_rate": 72.0, "respiration_rate": 18.0, "temperature": 37.5, "spo2": 98.0}
        ]);
        assert!(validate_vital_signs(&data).is_ok());
    }

    #[test]
    fn test_vital_signs_invalid() {
        let data = json!({"not": "an array"});
        let err = validate_vital_signs(&data).expect_err("should reject non-array");
        assert!(err.to_string().contains("Invalid vital_signs format"));
    }

    #[test]
    fn test_medication_jsonb_object() {
        let data = json!({"drug": "Ketamine", "dose": "10mg/kg"});
        assert!(validate_medication_jsonb("induction_anesthesia", &data).is_ok());
    }

    #[test]
    fn test_medication_jsonb_array() {
        let data = json!([{"drug": "Ketamine"}]);
        assert!(validate_medication_jsonb("induction_anesthesia", &data).is_ok());
    }

    #[test]
    fn test_medication_jsonb_scalar_rejected() {
        let data = json!("just a string");
        let err = validate_medication_jsonb("induction_anesthesia", &data)
            .expect_err("should reject scalar");
        assert!(err.to_string().contains("expected JSON object or array"));
    }

    #[test]
    fn test_valid_attachments() {
        let data = json!([
            {"file_name": "photo.jpg", "file_path": "/uploads/photo.jpg", "file_type": "image/jpeg"}
        ]);
        assert!(validate_attachments(&data).is_ok());
    }

    #[test]
    fn test_attachments_empty_name() {
        let data = json!([{"file_name": "", "file_path": "/uploads/x.jpg"}]);
        let err = validate_attachments(&data).expect_err("should reject empty file_name");
        assert!(err.to_string().contains("file_name cannot be empty"));
    }
}
