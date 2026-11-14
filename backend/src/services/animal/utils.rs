use chrono::NaiveDate;

use crate::{models::AnimalBreed, AppError, Result};

/// 動物相關工具函數
pub struct AnimalUtils;

impl AnimalUtils {
    /// 出生日期不可晚於「今日」（台灣時間）；當天允許。
    ///
    /// 只用於新寫入（建立、Excel 匯入、欄位更正申請），不回頭檢查既有資料。
    /// `today` 由呼叫端傳入（正式路徑用 `crate::time::today_taiwan_naive()`），
    /// 讓邊界可以在單元測試中釘死。
    pub fn ensure_birth_date_not_future(birth_date: NaiveDate, today: NaiveDate) -> Result<()> {
        if birth_date > today {
            return Err(AppError::Validation(format!(
                "出生日期不可晚於今日（{}），請確認輸入的日期",
                today
            )));
        }
        Ok(())
    }

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

    /// 欄位代碼的正規形式：補零（A–G + 數字）後，再大寫、去 `-`、去前後空白。
    ///
    /// 用來比對「請求的代碼」與「資料庫裡欄位的代碼」。資料庫端的對應規則是
    /// `repositories::pen::canonical_code_expr`，兩邊必須一致，否則存成 `A1` 的欄位
    /// 會對不到自己、或誤選到另一區存成 `A01` 的欄位。兩邊的測試各放一份**同一張**期望表
    /// （本檔單元測試與 `tests/api_animal_pen_single_source.rs`），改規則時兩邊要一起改。
    pub fn canonical_pen_code(code: &str) -> String {
        Self::format_pen_location(code)
            .trim()
            .to_uppercase()
            .replace('-', "")
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

    /// 從歷史 `IACUC_CHANGE` 稽核事件的顯示文字還原 (before, after) 資料。
    ///
    /// 這類事件在修正前寫入時 `before_data/after_data` 是空的，只有顯示文字
    /// `[舊 → 新] 耳號`（由 `AnimalService::update` 同一處產生，舊值沒有時寫 `(無)`）。
    /// 稽核紀錄不可修改（`trg_user_activity_logs_immutable`），所以讀取時由文字補回，
    /// 讓時間軸能顯示「舊 → 新」。格式對不上就回 `None`，呼叫端維持原狀。
    ///
    /// 回傳值形如 `({"iacuc_no": null}, {"iacuc_no": "PIG-114005"})`，與新寫入事件的
    /// 欄位名一致；歷史事件沒有實際日期，所以不含 `experiment_date`。
    pub fn iacuc_change_data_from_display(
        display: &str,
    ) -> Option<(serde_json::Value, serde_json::Value)> {
        // 耳號固定在最後（三位數、不含 "] "），從右邊切才不受 IACUC 編號內容影響
        let (inner, _ear_tag) = display.rsplit_once("] ")?;
        let inner = inner.strip_prefix('[')?;
        let (old, new) = inner.split_once(" → ")?;
        let (old, new) = (old.trim(), new.trim());
        if new.is_empty() {
            return None;
        }
        let old = if old.is_empty() || old == "(無)" {
            serde_json::Value::Null
        } else {
            serde_json::Value::String(old.to_string())
        };
        Some((
            serde_json::json!({ "iacuc_no": old }),
            serde_json::json!({ "iacuc_no": new }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── canonical_pen_code ──

    /// 期望表（輸入, 正規形式）。與 `tests/api_animal_pen_single_source.rs` 裡驗證資料庫端
    /// `canonical_code_expr` 的那張表是同一份，兩邊必須一起改。
    const CANONICAL_PEN_CODE_CASES: &[(&str, &str)] = &[
        ("A1", "A01"),
        ("a1", "A01"),
        ("A-1", "A01"),
        ("a-01", "A01"),
        ("A01", "A01"),
        ("A001", "A01"),
        ("A0", "A00"),
        // 三位以上的數字不可被截斷（SQL 端的 lpad 曾把 A100 截成 A10）
        ("A100", "A100"),
        ("a-100", "A100"),
        ("A010", "A10"),
        ("A1000", "A1000"),
        ("B10", "B10"),
        ("G7", "G07"),
        (" c3 ", "C03"),
        ("Q-1", "Q1"),
        ("Q01", "Q01"),
        ("A", "A"),
        ("A1B", "A1B"),
        ("H1", "H1"),
    ];

    #[test]
    fn canonical_pen_code_matches_expectation_table() {
        for (input, expected) in CANONICAL_PEN_CODE_CASES {
            assert_eq!(
                AnimalUtils::canonical_pen_code(input),
                *expected,
                "輸入 {input:?}"
            );
        }
    }

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

    // ── ensure_birth_date_not_future ──

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("測試用日期必須合法")
    }

    #[test]
    fn test_birth_date_future_rejected() {
        let today = ymd(2026, 11, 2);
        let tomorrow = ymd(2026, 11, 3);
        assert!(AnimalUtils::ensure_birth_date_not_future(tomorrow, today).is_err());
    }

    #[test]
    fn test_birth_date_today_and_past_allowed() {
        let today = ymd(2026, 11, 2);
        assert!(AnimalUtils::ensure_birth_date_not_future(today, today).is_ok());
        let past = ymd(2025, 1, 15);
        assert!(AnimalUtils::ensure_birth_date_not_future(past, today).is_ok());
    }

    // ── iacuc_change_data_from_display ──

    #[test]
    fn test_iacuc_display_first_assignment() {
        let (before, after) =
            AnimalUtils::iacuc_change_data_from_display("[(無) → PIG-114005] 243")
                .expect("標準格式應可解析");
        assert_eq!(before, serde_json::json!({ "iacuc_no": null }));
        assert_eq!(after, serde_json::json!({ "iacuc_no": "PIG-114005" }));
    }

    #[test]
    fn test_iacuc_display_change_between_plans() {
        let (before, after) =
            AnimalUtils::iacuc_change_data_from_display("[PIG-114004 → PIG-114005] 243")
                .expect("標準格式應可解析");
        assert_eq!(before, serde_json::json!({ "iacuc_no": "PIG-114004" }));
        assert_eq!(after, serde_json::json!({ "iacuc_no": "PIG-114005" }));
    }

    #[test]
    fn test_iacuc_display_unparseable_returns_none() {
        assert!(AnimalUtils::iacuc_change_data_from_display("").is_none());
        assert!(AnimalUtils::iacuc_change_data_from_display("243").is_none());
        assert!(AnimalUtils::iacuc_change_data_from_display("[PIG-1] 243").is_none());
        // 新值為空不是合法的分配結果
        assert!(AnimalUtils::iacuc_change_data_from_display("[PIG-1 → ] 243").is_none());
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
