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

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_ear_tag ──

    #[test]
    fn test_ear_tag_single_digit_padded() {
        assert_eq!(AnimalUtils::format_ear_tag("5"), "005");
    }

    #[test]
    fn test_ear_tag_two_digit_padded() {
        assert_eq!(AnimalUtils::format_ear_tag("99"), "099");
    }

    #[test]
    fn test_ear_tag_three_digit_no_pad() {
        assert_eq!(AnimalUtils::format_ear_tag("100"), "100");
    }

    #[test]
    fn test_ear_tag_zero() {
        assert_eq!(AnimalUtils::format_ear_tag("0"), "000");
    }

    #[test]
    fn test_ear_tag_non_numeric_passthrough() {
        assert_eq!(AnimalUtils::format_ear_tag("abc"), "abc");
    }

    #[test]
    fn test_ear_tag_empty_passthrough() {
        assert_eq!(AnimalUtils::format_ear_tag(""), "");
    }

    #[test]
    fn test_ear_tag_large_number() {
        assert_eq!(AnimalUtils::format_ear_tag("999"), "999");
    }

    // ── format_pen_location ──

    #[test]
    fn test_pen_location_lowercase_normalized() {
        assert_eq!(AnimalUtils::format_pen_location("d1"), "D01");
    }

    #[test]
    fn test_pen_location_uppercase_single_digit() {
        assert_eq!(AnimalUtils::format_pen_location("C3"), "C03");
    }

    #[test]
    fn test_pen_location_hyphen_removed() {
        assert_eq!(AnimalUtils::format_pen_location("A-1"), "A01");
    }

    #[test]
    fn test_pen_location_two_digit_number() {
        assert_eq!(AnimalUtils::format_pen_location("B10"), "B10");
    }

    #[test]
    fn test_pen_location_empty() {
        assert_eq!(AnimalUtils::format_pen_location(""), "");
    }

    #[test]
    fn test_pen_location_whitespace_only() {
        assert_eq!(AnimalUtils::format_pen_location("  "), "");
    }

    #[test]
    fn test_pen_location_invalid_zone_passthrough() {
        // H 不在 A-G 範圍，應該原樣返回
        assert_eq!(AnimalUtils::format_pen_location("H1"), "H1");
    }

    #[test]
    fn test_pen_location_zone_only_no_number() {
        assert_eq!(AnimalUtils::format_pen_location("A"), "A");
    }

    // ── breed_to_db_value ──

    #[test]
    fn test_breed_minipig() {
        assert_eq!(AnimalUtils::breed_to_db_value(&AnimalBreed::Minipig), "miniature");
    }

    #[test]
    fn test_breed_white() {
        assert_eq!(AnimalUtils::breed_to_db_value(&AnimalBreed::White), "white");
    }

    #[test]
    fn test_breed_lyd() {
        assert_eq!(AnimalUtils::breed_to_db_value(&AnimalBreed::LYD), "LYD");
    }

    #[test]
    fn test_breed_other() {
        assert_eq!(AnimalUtils::breed_to_db_value(&AnimalBreed::Other), "other");
    }

}
