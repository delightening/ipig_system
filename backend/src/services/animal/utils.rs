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

        // 以「字元」切分，不可用位元組切片：`normalized.len()` 回的是**位元組**長度，
        // 舊寫法 `&normalized[0..1]` 對首字為多位元組的字串（中文欄位 code、emoji）
        // 會在非字元邊界切下去而 panic（"羊" 的 len() 是 3，因此連長度守門都擋不住）。
        // 該 panic 在 create / update / Excel 匯入三個寫入路徑與四個讀取路徑皆可觸發。
        let mut chars = normalized.chars();
        if let Some(zone) = chars.next() {
            let num_part = chars.as_str();
            // 驗證 zone 是否在 A-G 範圍
            if !num_part.is_empty() && matches!(zone, 'A'..='G') {
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

    /// 迴歸測試：多位元組字元不可 panic。
    ///
    /// 舊寫法以位元組守門與切片（`normalized.len() >= 2` + `&normalized[0..1]`），
    /// 單一中文字的 `len()` 是 3、守門放行後即在非字元邊界切下去 →
    /// `byte index 1 is not a char boundary`。`pen_location` 是 API 可自由輸入的字串
    /// （create / update / Excel 匯入），且四個讀取路徑也會對 DB 既有值重新格式化，
    /// 因此這個 panic 過去可由任何已登入使用者觸發。
    #[test]
    fn test_pen_location_multibyte_does_not_panic() {
        // prod 曾實際存在 code 為「羊」的欄位
        assert_eq!(AnimalUtils::format_pen_location("羊"), "羊");
        assert_eq!(AnimalUtils::format_pen_location("羊1"), "羊1");
        assert_eq!(AnimalUtils::format_pen_location("羊舍01"), "羊舍01");
        // 非中文的多位元組字元同樣不可炸
        assert_eq!(AnimalUtils::format_pen_location("🐷"), "🐷");
        assert_eq!(AnimalUtils::format_pen_location("Ä1"), "Ä1");
        // 前後空白仍照既有規則 trim
        assert_eq!(AnimalUtils::format_pen_location("  羊  "), "羊");
    }

    /// 非 A–G 開頭者一律原樣返回——羊舍 S / 檢疫舍 Q 走的就是這條路徑。
    #[test]
    fn test_pen_location_non_ag_zone_passthrough() {
        assert_eq!(AnimalUtils::format_pen_location("S01"), "S01");
        assert_eq!(AnimalUtils::format_pen_location("Q01"), "Q01");
        // 不會被補零，因為只有 A–G 會正規化
        assert_eq!(AnimalUtils::format_pen_location("S1"), "S1");
    }

    // ── breed_to_db_value ──

    #[test]
    fn test_breed_minipig() {
        assert_eq!(
            AnimalUtils::breed_to_db_value(&AnimalBreed::Minipig),
            "miniature"
        );
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
