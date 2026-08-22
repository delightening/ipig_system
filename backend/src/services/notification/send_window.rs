//! 員工通知 email 寄送時間窗（台灣時間，週一至週五 09:00–17:00、排除國定假日）。
//!
//! 純函式：時間判斷與「下一個合法窗口」計算與 I/O 解耦——假日資訊由呼叫端
//! 預先查成集合後以 `is_holiday` 閉包注入，便於單元測試（不需網路 / DB）。
//!
//! 註：使用者明確要求「排除週六日」，故補班週六亦視為不寄（窗口僅取週一至週五）。

use chrono::{DateTime, Datelike, Days, FixedOffset, NaiveDate, TimeZone, Timelike, Weekday};

use crate::constants::{STAFF_EMAIL_WINDOW_END_HOUR, STAFF_EMAIL_WINDOW_START_HOUR};

/// 找下一個合法窗口時最多往後掃描的天數（防無界迴圈）。
/// 連假最長不過約 9 天，14 天有足夠餘裕。
const MAX_SCAN_DAYS: u64 = 14;

/// 該日是否為「工作日」＝週一至週五且非國定假日。
fn is_workday(date: NaiveDate, is_holiday: &impl Fn(NaiveDate) -> bool) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) && !is_holiday(date)
}

/// 現在是否落在員工通知寄送窗內（工作日 ∧ 09:00 ≤ 當地小時 < 17:00）。
pub(crate) fn is_within_staff_window(
    now: DateTime<FixedOffset>,
    is_holiday: &impl Fn(NaiveDate) -> bool,
) -> bool {
    is_workday(now.date_naive(), is_holiday)
        && (STAFF_EMAIL_WINDOW_START_HOUR..STAFF_EMAIL_WINDOW_END_HOUR).contains(&now.hour())
}

/// 把某日 09:00（窗口起點）組成帶時區的時間點。
fn window_start_of(tz: FixedOffset, date: NaiveDate) -> Option<DateTime<FixedOffset>> {
    let naive = date.and_hms_opt(STAFF_EMAIL_WINDOW_START_HOUR, 0, 0)?;
    tz.from_local_datetime(&naive).single()
}

/// 計算下一個合法寄送窗口的起始時間點：
/// - 已在窗內 → 回傳 `now`（立即可寄）。
/// - 今天是工作日且尚未到 09:00 → 今天 09:00。
/// - 否則 → 之後第一個工作日的 09:00（有界掃描，跨週末 / 連假）。
pub(crate) fn next_window_open(
    now: DateTime<FixedOffset>,
    is_holiday: &impl Fn(NaiveDate) -> bool,
) -> DateTime<FixedOffset> {
    if is_within_staff_window(now, is_holiday) {
        return now;
    }

    let tz = now.timezone();
    let today = now.date_naive();

    // 今天是工作日且還沒到 09:00 → 今天開窗。
    if is_workday(today, is_holiday) && now.hour() < STAFF_EMAIL_WINDOW_START_HOUR {
        if let Some(dt) = window_start_of(tz, today) {
            return dt;
        }
    }

    // 否則找之後第一個工作日的 09:00。
    for n in 1..=MAX_SCAN_DAYS {
        if let Some(date) = today.checked_add_days(Days::new(n)) {
            if is_workday(date, is_holiday) {
                if let Some(dt) = window_start_of(tz, date) {
                    return dt;
                }
            }
        }
    }

    // 理論上不可達（14 天內必有工作日）；保底回 now 避免 panic。
    now
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::TAIWAN_OFFSET_SECS;

    fn tz() -> FixedOffset {
        FixedOffset::east_opt(TAIWAN_OFFSET_SECS).expect("valid offset")
    }

    /// 建構台灣時間 DateTime（年月日時分）。
    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<FixedOffset> {
        tz().with_ymd_and_hms(y, m, d, h, min, 0)
            .single()
            .expect("valid datetime")
    }

    fn no_holiday(_: NaiveDate) -> bool {
        false
    }

    // 2026-06-25 是週四、2026-06-26 週五、06-27 週六、06-28 週日、06-29 週一。

    #[test]
    fn within_window_weekday_business_hours() {
        // 週四 10:00 → 窗內
        assert!(is_within_staff_window(dt(2026, 6, 25, 10, 0), &no_holiday));
    }

    #[test]
    fn boundary_open_and_close() {
        // 09:00 整點視為窗內；16:59 窗內；17:00 視為窗外（exclusive）。
        assert!(is_within_staff_window(dt(2026, 6, 26, 9, 0), &no_holiday));
        assert!(is_within_staff_window(dt(2026, 6, 26, 16, 59), &no_holiday));
        assert!(!is_within_staff_window(dt(2026, 6, 26, 17, 0), &no_holiday));
        assert!(!is_within_staff_window(dt(2026, 6, 26, 8, 59), &no_holiday));
    }

    #[test]
    fn weekend_is_outside() {
        assert!(!is_within_staff_window(dt(2026, 6, 27, 10, 0), &no_holiday)); // 週六
        assert!(!is_within_staff_window(dt(2026, 6, 28, 10, 0), &no_holiday)); // 週日
    }

    #[test]
    fn holiday_weekday_is_outside() {
        let is_h = |d: NaiveDate| d == NaiveDate::from_ymd_opt(2026, 6, 25).expect("valid");
        assert!(!is_within_staff_window(dt(2026, 6, 25, 10, 0), &is_h));
    }

    #[test]
    fn before_open_same_workday_defers_to_today_9am() {
        let got = next_window_open(dt(2026, 6, 26, 7, 30), &no_holiday); // 週五早上
        assert_eq!(got, dt(2026, 6, 26, 9, 0));
    }

    #[test]
    fn after_close_friday_defers_to_monday_9am() {
        let got = next_window_open(dt(2026, 6, 26, 18, 0), &no_holiday); // 週五傍晚
        assert_eq!(got, dt(2026, 6, 29, 9, 0)); // → 下週一 09:00
    }

    #[test]
    fn saturday_defers_to_monday_9am() {
        let got = next_window_open(dt(2026, 6, 27, 10, 0), &no_holiday); // 週六
        assert_eq!(got, dt(2026, 6, 29, 9, 0));
    }

    #[test]
    fn within_window_returns_now() {
        let now = dt(2026, 6, 25, 10, 0);
        assert_eq!(next_window_open(now, &no_holiday), now);
    }

    #[test]
    fn long_holiday_run_skips_to_first_workday() {
        // 模擬週一(6/29)~週三(7/1)連假 → 週五傍晚發 → 應跳到 7/2(週四) 09:00
        let holidays = |d: NaiveDate| {
            matches!(
                (d.year(), d.month(), d.day()),
                (2026, 6, 29) | (2026, 6, 30) | (2026, 7, 1)
            )
        };
        let got = next_window_open(dt(2026, 6, 26, 18, 0), &holidays);
        assert_eq!(got, dt(2026, 7, 2, 9, 0));
    }

    #[test]
    fn bounded_scan_never_panics_all_holidays() {
        // 全部都是假日 → 有界掃描後保底回 now，不 panic。
        let all = |_: NaiveDate| true;
        let now = dt(2026, 6, 27, 10, 0);
        let _ = next_window_open(now, &all);
    }
}
