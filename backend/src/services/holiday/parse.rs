//! 台灣國定假日 JSON 解析（純函式，可單測）。
//!
//! 預期格式為政府開放資料「政府行政機關辦公日曆表」之機器可讀鏡像：
//! ```json
//! [ {"date":"20260101","week":"四","isHoliday":true,"description":"開國紀念日"},
//!   {"date":"20260102","week":"五","isHoliday":false,"description":""} ]
//! ```
//! 僅取 `isHoliday == true` 的日期作為「放假日」集合。`date` 為 `YYYYMMDD`。
//!
//! ⚠️ 實際端點欄位須於有對外網路的環境驗證；此解析對 `isHoliday` 容錯接受
//! 布林或字串（"true"/"是"/"2" 等視為放假），以降低端點格式差異造成的全量失效。

use std::collections::HashSet;

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawDay {
    #[serde(default)]
    date: String,
    #[serde(rename = "isHoliday", default)]
    is_holiday: HolidayFlag,
}

/// `isHoliday` 容錯：接受布林或字串。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum HolidayFlag {
    Bool(bool),
    Text(String),
}

impl Default for HolidayFlag {
    fn default() -> Self {
        HolidayFlag::Bool(false)
    }
}

impl HolidayFlag {
    fn is_off(&self) -> bool {
        match self {
            HolidayFlag::Bool(b) => *b,
            HolidayFlag::Text(s) => {
                matches!(
                    s.trim().to_ascii_lowercase().as_str(),
                    "true" | "1" | "2" | "是" | "yes"
                )
            }
        }
    }
}

/// 解析整年假日 JSON，回傳放假日期集合。
/// 無法解析的單筆日期會被略過（容錯），整體無效 JSON 才回 `Err`。
pub fn parse_holiday_year(json: &str) -> Result<HashSet<NaiveDate>, serde_json::Error> {
    let days: Vec<RawDay> = serde_json::from_str(json)?;
    let mut set = HashSet::new();
    for d in days {
        if !d.is_holiday.is_off() {
            continue;
        }
        if let Ok(date) = NaiveDate::parse_from_str(d.date.trim(), "%Y%m%d") {
            set.insert(date);
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn parses_bool_holiday_flags() {
        let json = r#"[
            {"date":"20260101","week":"四","isHoliday":true,"description":"開國紀念日"},
            {"date":"20260102","week":"五","isHoliday":false,"description":""}
        ]"#;
        let set = parse_holiday_year(json).expect("parse ok");
        assert!(set.contains(&ymd(2026, 1, 1)));
        assert!(!set.contains(&ymd(2026, 1, 2)));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn tolerates_string_flags() {
        let json = r#"[
            {"date":"20260228","isHoliday":"是"},
            {"date":"20260301","isHoliday":"否"},
            {"date":"20260302","isHoliday":"2"}
        ]"#;
        let set = parse_holiday_year(json).expect("parse ok");
        assert!(set.contains(&ymd(2026, 2, 28)));
        assert!(set.contains(&ymd(2026, 3, 2)));
        assert!(!set.contains(&ymd(2026, 3, 1)));
    }

    #[test]
    fn skips_unparseable_dates_but_keeps_valid() {
        let json = r#"[
            {"date":"bad","isHoliday":true},
            {"date":"20260404","isHoliday":true}
        ]"#;
        let set = parse_holiday_year(json).expect("parse ok");
        assert_eq!(set.len(), 1);
        assert!(set.contains(&ymd(2026, 4, 4)));
    }

    #[test]
    fn invalid_json_errors() {
        assert!(parse_holiday_year("not json").is_err());
    }

    #[test]
    fn missing_flag_defaults_to_workday() {
        let json = r#"[{"date":"20260105"}]"#;
        let set = parse_holiday_year(json).expect("parse ok");
        assert!(set.is_empty());
    }
}
