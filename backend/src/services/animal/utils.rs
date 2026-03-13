use crate::models::AnimalBreed;

/// 動物相關工具函數
pub struct AnimalUtils;

impl AnimalUtils {
    /// 格式化耳號：如果是數字且 < 100，則補零至三位數
    pub fn format_ear_tag(ear_tag: &str) -> String {
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
    pub fn format_pen_location(pen_location: &str) -> String {
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

    /// 品種 enum 轉資料庫字串
    pub fn breed_to_db_value(breed: &AnimalBreed) -> &str {
        match breed {
            AnimalBreed::Minipig => "miniature",
            AnimalBreed::White => "white",
            AnimalBreed::LYD => "LYD",
            AnimalBreed::Other => "other",
        }
    }
}
