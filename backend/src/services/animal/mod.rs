// 動物管理服務模組
//
// 將原 animal.rs 拆分為以下子模組：
// - core: 動物 CRUD、列表、批次分配
// - blood_test: 血液檢測模板與面板管理
// - medical: 病歷紀錄、疫苗、獸醫建議
// - surgery: 手術紀錄
// - observation: 觀察紀錄
// - weight: 體重紀錄
// - source: 動物來源管理
// - import_export: 動物資料匯入匯出

mod blood_test;
mod core;
mod import_export;
mod medical;
mod observation;
mod source;
mod surgery;
mod transfer;
mod weight;
pub mod care_record;

pub struct AnimalService;

impl AnimalService {
    /// 格式化耳號：如果是數字且 < 100，則補零至三位數
    fn format_ear_tag(ear_tag: &str) -> String {
        if let Ok(num) = ear_tag.parse::<u32>() {
            if num < 100 {
                format!("{:03}", num)
            } else {
                ear_tag.to_string()
            }
        } else {
            ear_tag.to_string()
        }
    }

    /// 格式化欄位編號：正規化為 [A-G][01-09] 格式
    /// 例如 "d1" -> "D01", "C3" -> "C03", "A-1" -> "A01"
    fn format_pen_location(pen_location: &str) -> String {
        let trimmed = pen_location.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // 正規化格式：[A-G][數字]，統一大寫
        let normalized = trimmed.to_uppercase().replace('-', "");

        if normalized.len() >= 2 {
            let zone = &normalized[0..1];
            let num_part = &normalized[1..];

            // 驗證 zone 是否在 A-G 範圍
            if matches!(zone, "A" | "B" | "C" | "D" | "E" | "F" | "G") {
                if let Ok(num) = num_part.parse::<u32>() {
                    return format!("{}{:02}", zone, num);
                }
            }
        }

        trimmed.to_string()
    }
}
