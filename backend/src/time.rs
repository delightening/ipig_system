//! 系統統一使用台灣時間 (Asia/Taipei, UTC+8)。
//! 業務「今日」、報表日期、PDF/郵件顯示日期等皆以此為準。

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

use crate::constants::TAIWAN_OFFSET_SECS;

/// 台灣時區偏移 (UTC+8)
#[inline]
pub fn taiwan_offset() -> FixedOffset {
    FixedOffset::east_opt(TAIWAN_OFFSET_SECS).expect("TAIWAN_OFFSET_SECS is valid")
}

/// 取得當前時間（台灣時間）
#[inline]
pub fn now_taiwan() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&taiwan_offset())
}

/// 取得台灣「今日」日期（無時間），用於 partition_date、報表 as_of、請假/出勤「今天」等
#[inline]
pub fn today_taiwan_naive() -> NaiveDate {
    now_taiwan().date_naive()
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use super::*;

    #[test]
    fn test_taiwan_offset_is_utc_plus_8() {
        let tz = taiwan_offset();
        assert_eq!(tz.utc_minus_local(), -8 * 3600);
    }

    #[test]
    fn test_today_taiwan_naive_is_naive_date() {
        let d = today_taiwan_naive();
        assert!(d.year() >= 2020 && d.year() <= 2030);
    }
}
