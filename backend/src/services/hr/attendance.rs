// HR 出勤管理

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc, Weekday};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AttendanceCorrectionRequest, AttendanceQuery, AttendanceRecord,
        AttendanceWithUser, PaginatedResponse,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    Result,
};

use super::HrService;

/// 將 UTC 打卡時間轉為台灣時間 (UTC+8) 並格式化為 HH:MM:SS；None 顯示為 "-"
/// 出勤時間於 DB 以 UTC 儲存，匯出時須轉回台灣時區，否則匯出值會少 8 小時。
fn format_clock_time(t: Option<DateTime<Utc>>) -> String {
    t.map(|t| {
        t.with_timezone(&crate::time::taiwan_offset())
            .format("%H:%M:%S")
            .to_string()
    })
    .unwrap_or_else(|| "-".to_string())
}

/// 計算扣除午休後的實際工時（小時，f64）。
///
/// 規則（定案 2026-06-11）：
/// - 平日（週一~週五，依 `work_date` 判定）若工作時段與 12:00–13:00（台灣時間）
///   有重疊，扣除「實際重疊」時間（e.g. 08:30→17:30 扣整 1hr=8.0；08:30→12:30 只扣 30 分）。
/// - 週末（六、日）值班照全時計，不扣午休。
/// - 已知限制：落在平日的國定假日仍被當平日扣午休（本系統無國定假日行事曆，列為 follow-up）。
fn compute_regular_hours(
    clock_in: DateTime<Utc>,
    clock_out: DateTime<Utc>,
    work_date: NaiveDate,
) -> f64 {
    let raw = (clock_out - clock_in).num_seconds() as f64 / 3600.0;
    if raw <= 0.0 {
        return 0.0;
    }

    // 週末值班不扣午休
    if matches!(work_date.weekday(), Weekday::Sat | Weekday::Sun) {
        return raw;
    }

    // 以 work_date 當天 12:00–13:00（台灣時間）為午休窗，換算回 UTC 後與工作時段求重疊
    let tz = crate::time::taiwan_offset();
    let to_utc = |h: u32, m: u32| -> Option<DateTime<Utc>> {
        let naive = work_date.and_hms_opt(h, m, 0)?;
        tz.from_local_datetime(&naive)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
    };
    let (Some(lunch_start), Some(lunch_end)) = (to_utc(12, 0), to_utc(13, 0)) else {
        return raw; // 理論上不會發生（固定 offset 必為 single）
    };

    // 重疊 = max(0, min(out, 13:00) − max(in, 12:00))
    let overlap_start = clock_in.max(lunch_start);
    let overlap_end = clock_out.min(lunch_end);
    let overlap = (overlap_end - overlap_start).num_seconds().max(0) as f64 / 3600.0;

    (raw - overlap).max(0.0)
}

/// 將工時 f64 轉為 DB 用的 `Decimal`（NaN/inf 等異常值回傳 None）。
fn regular_hours_decimal(hours: f64) -> Option<Decimal> {
    Decimal::from_f64_retain(hours)
}

impl HrService {
    // ============================================
    // Attendance
    // ============================================

    /// 檢查 IP 是否在允許的 CIDR 範圍內
    /// 支援格式：單一 IP（如 "10.0.4.1"）或 CIDR（如 "10.0.4.0/24"）
    pub fn is_ip_in_ranges(ip: &str, ranges: &[String]) -> bool {
        use std::net::{IpAddr, Ipv4Addr};

        let client_ip: IpAddr = match ip.parse() {
            Ok(addr) => addr,
            Err(_) => return false,
        };

        for range in ranges {
            if let Some((network_str, prefix_str)) = range.split_once('/') {
                // CIDR 格式：如 "10.0.4.0/24"
                if let (Ok(network_ip), Ok(prefix_len)) = (
                    network_str.trim().parse::<Ipv4Addr>(),
                    prefix_str.trim().parse::<u32>(),
                ) {
                    if prefix_len <= 32 {
                        if let IpAddr::V4(client_v4) = client_ip {
                            let mask = if prefix_len == 0 {
                                0u32
                            } else {
                                !0u32 << (32 - prefix_len)
                            };
                            let network_bits = u32::from(network_ip) & mask;
                            let client_bits = u32::from(client_v4) & mask;
                            if network_bits == client_bits {
                                return true;
                            }
                        }
                    }
                }
            } else {
                // 單一 IP 格式：如 "203.0.113.42"
                if let Ok(allowed_ip) = range.trim().parse::<IpAddr>() {
                    if client_ip == allowed_ip {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub async fn list_attendance(
        pool: &PgPool,
        query: &AttendanceQuery,
    ) -> Result<PaginatedResponse<AttendanceWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM attendance_records
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::date IS NULL OR work_date >= $2)
              AND ($3::date IS NULL OR work_date <= $3)
              AND ($4::text IS NULL OR status = $4)
            "#,
        )
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, AttendanceWithUser>(
            r#"
            SELECT 
                a.id, a.user_id, u.email as user_email, u.display_name as user_name,
                a.work_date, a.clock_in_time, a.clock_out_time,
                a.regular_hours, a.overtime_hours, a.status, a.remark, a.is_corrected
            FROM attendance_records a
            INNER JOIN users u ON a.user_id = u.id
            WHERE ($1::uuid IS NULL OR a.user_id = $1)
              AND ($2::date IS NULL OR a.work_date >= $2)
              AND ($3::date IS NULL OR a.work_date <= $3)
              AND ($4::text IS NULL OR a.status = $4)
            ORDER BY a.work_date DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(query.from)
        .bind(query.to)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    /// 匯出出勤記錄為 Excel
    pub async fn export_attendance_to_excel(
        pool: &PgPool,
        query: &AttendanceQuery,
    ) -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let mut export_query = query.clone();
        export_query.per_page = Some(10000);
        export_query.page = Some(1);
        let result = Self::list_attendance(pool, &export_query).await?;

        let mut workbook = Workbook::new();
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);

        let worksheet = workbook.add_worksheet();
        worksheet.set_column_width(0, 18.0)?;
        worksheet.set_column_width(1, 25.0)?;
        worksheet.set_column_width(2, 12.0)?;
        worksheet.set_column_width(3, 12.0)?;
        worksheet.set_column_width(4, 12.0)?;
        worksheet.set_column_width(5, 12.0)?;
        worksheet.set_column_width(6, 12.0)?;
        worksheet.set_column_width(7, 30.0)?;

        worksheet.write_string_with_format(0, 0, "日期", &header_format)?;
        worksheet.write_string_with_format(0, 1, "人員名稱", &header_format)?;
        worksheet.write_string_with_format(0, 2, "上班", &header_format)?;
        worksheet.write_string_with_format(0, 3, "下班", &header_format)?;
        worksheet.write_string_with_format(0, 4, "工作時數", &header_format)?;
        worksheet.write_string_with_format(0, 5, "加班時數", &header_format)?;
        worksheet.write_string_with_format(0, 6, "狀態", &header_format)?;
        worksheet.write_string_with_format(0, 7, "備註", &header_format)?;

        let status_display = |s: &str| -> String {
            match s {
                "normal" => "正常".to_string(),
                "late" => "遲到".to_string(),
                "early_leave" => "早退".to_string(),
                "absent" => "缺勤".to_string(),
                _ => s.to_string(),
            }
        };

        for (row, r) in result.data.iter().enumerate() {
            let rw = (row + 1) as u32;
            worksheet.write_string(rw, 0, r.work_date.to_string())?;
            worksheet.write_string(rw, 1, &r.user_name)?;
            worksheet.write_string(rw, 2, format_clock_time(r.clock_in_time))?;
            worksheet.write_string(rw, 3, format_clock_time(r.clock_out_time))?;
            let hours = r
                .regular_hours
                .map(|h| format!("{:.1}", h))
                .unwrap_or_else(|| "-".to_string());
            worksheet.write_string(rw, 4, &hours)?;
            let ot = r
                .overtime_hours
                .map(|h| format!("{:.1}", h))
                .unwrap_or_else(|| "-".to_string());
            worksheet.write_string(rw, 5, &ot)?;
            worksheet.write_string(rw, 6, status_display(&r.status))?;
            let remark = if r.is_corrected {
                r.remark
                    .as_ref()
                    .map(|s| format!("已更正；{}", s))
                    .unwrap_or_else(|| "已更正".to_string())
            } else {
                r.remark.clone().unwrap_or_default()
            };
            worksheet.write_string(rw, 7, &remark)?;
        }

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }

    pub async fn clock_in(
        pool: &PgPool,
        actor: &ActorContext,
        source: Option<&str>,
        ip: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Result<AttendanceRecord> {
        let user = actor.require_user()?;
        let user_id = user.id;

        // 使用台灣時區 (UTC+8) 的日期，而不是 UTC 日期
        // 這樣當使用者在凌晨打卡時，work_date 會是正確的本地日期
        let taipei_offset = chrono::FixedOffset::east_opt(8 * 3600)
            .ok_or_else(|| AppError::Internal("invalid timezone offset UTC+8".to_string()))?;
        let today = Utc::now().with_timezone(&taipei_offset).date_naive();

        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE：行鎖 + before 快照（若當日已有 attendance row）
        let before: Option<AttendanceRecord> = sqlx::query_as(
            r#"SELECT id, user_id, work_date, clock_in_time, clock_out_time,
                    regular_hours, overtime_hours, status, clock_in_source,
                    clock_in_ip::TEXT, clock_out_source, clock_out_ip::TEXT,
                    clock_in_latitude, clock_in_longitude,
                    clock_out_latitude, clock_out_longitude,
                    remark, is_corrected, corrected_by, corrected_at,
                    correction_reason, created_at, updated_at
               FROM attendance_records WHERE user_id = $1 AND work_date = $2 FOR UPDATE"#,
        )
        .bind(user_id)
        .bind(today)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(ref record) = before {
            if record.clock_in_time.is_some() {
                return Err(AppError::Validation("今天已經打卡上班".to_string()));
            }
        }

        let after = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            INSERT INTO attendance_records (id, user_id, work_date, clock_in_time, clock_in_source, clock_in_ip, clock_in_latitude, clock_in_longitude, status)
            VALUES ($1, $2, $3, NOW(), $4, $5::inet, $6, $7, 'normal')
            ON CONFLICT (user_id, work_date) DO UPDATE SET
                clock_in_time = NOW(),
                clock_in_source = $4,
                clock_in_ip = $5::inet,
                clock_in_latitude = $6,
                clock_in_longitude = $7,
                updated_at = NOW()
            RETURNING id, user_id, work_date, clock_in_time, clock_out_time,
                    regular_hours, overtime_hours, status, clock_in_source,
                    clock_in_ip::TEXT, clock_out_source, clock_out_ip::TEXT,
                    clock_in_latitude, clock_in_longitude,
                    clock_out_latitude, clock_out_longitude,
                    remark, is_corrected, corrected_by, corrected_at,
                    correction_reason, created_at, updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(today)
        .bind(source.unwrap_or("web"))
        .bind(ip)
        .bind(latitude)
        .bind(longitude)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} {}", after.work_date, user.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "ATTENDANCE_CLOCK_IN",
                entity: Some(AuditEntity::new("attendance_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(before.as_ref(), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    pub async fn clock_out(
        pool: &PgPool,
        actor: &ActorContext,
        source: Option<&str>,
        ip: Option<&str>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) -> Result<AttendanceRecord> {
        let user = actor.require_user()?;
        let user_id = user.id;

        // 使用台灣時區 (UTC+8) 的日期，與 clock_in 保持一致
        let taipei_offset = chrono::FixedOffset::east_opt(8 * 3600)
            .ok_or_else(|| AppError::Internal("invalid timezone offset UTC+8".to_string()))?;
        let today = Utc::now().with_timezone(&taipei_offset).date_naive();

        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AttendanceRecord>(
            r#"SELECT id, user_id, work_date, clock_in_time, clock_out_time,
                    regular_hours, overtime_hours, status, clock_in_source,
                    clock_in_ip::TEXT, clock_out_source, clock_out_ip::TEXT,
                    clock_in_latitude, clock_in_longitude,
                    clock_out_latitude, clock_out_longitude,
                    remark, is_corrected, corrected_by, corrected_at,
                    correction_reason, created_at, updated_at
               FROM attendance_records WHERE user_id = $1 AND work_date = $2 FOR UPDATE"#,
        )
        .bind(user_id)
        .bind(today)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Validation("請先打卡上班".to_string()))?;

        // 在 Rust 端固定下班時間，並以扣除午休後的工時寫入 regular_hours
        // （平日扣 12:00–13:00 實際重疊，週末值班不扣；見 compute_regular_hours）
        let clock_out_time = Utc::now();
        let regular_hours = before
            .clock_in_time
            .map(|ci| compute_regular_hours(ci, clock_out_time, today))
            .and_then(regular_hours_decimal);

        let after = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            UPDATE attendance_records
            SET clock_out_time = $3,
                clock_out_source = $4,
                clock_out_ip = $5::inet,
                clock_out_latitude = $6,
                clock_out_longitude = $7,
                regular_hours = $8,
                updated_at = NOW()
            WHERE user_id = $1 AND work_date = $2
            RETURNING id, user_id, work_date, clock_in_time, clock_out_time,
                    regular_hours, overtime_hours, status, clock_in_source,
                    clock_in_ip::TEXT, clock_out_source, clock_out_ip::TEXT,
                    clock_in_latitude, clock_in_longitude,
                    clock_out_latitude, clock_out_longitude,
                    remark, is_corrected, corrected_by, corrected_at,
                    correction_reason, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(today)
        .bind(clock_out_time)
        .bind(source.unwrap_or("web"))
        .bind(ip)
        .bind(latitude)
        .bind(longitude)
        .bind(regular_hours)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("{} {}", after.work_date, user.email);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "ATTENDANCE_CLOCK_OUT",
                entity: Some(AuditEntity::new("attendance_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    pub async fn correct_attendance(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &AttendanceCorrectionRequest,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let corrector_id = user.id;
        let mut tx = pool.begin().await?;

        let before = sqlx::query_as::<_, AttendanceRecord>(
            r#"SELECT id, user_id, work_date, clock_in_time, clock_out_time,
                    regular_hours, overtime_hours, status, clock_in_source,
                    clock_in_ip::TEXT, clock_out_source, clock_out_ip::TEXT,
                    clock_in_latitude, clock_in_longitude,
                    clock_out_latitude, clock_out_longitude,
                    remark, is_corrected, corrected_by, corrected_at,
                    correction_reason, created_at, updated_at
               FROM attendance_records WHERE id = $1 FOR UPDATE"#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("出勤紀錄不存在".into()))?;

        // 依更正後的最終上/下班時間重算工時（扣午休）。缺任一時間則保留原值。
        let final_in = payload.clock_in_time.or(before.clock_in_time);
        let final_out = payload.clock_out_time.or(before.clock_out_time);
        let regular_hours = match (final_in, final_out) {
            (Some(ci), Some(co)) => {
                regular_hours_decimal(compute_regular_hours(ci, co, before.work_date))
            }
            _ => before.regular_hours,
        };

        let after = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            UPDATE attendance_records
            SET original_clock_in = clock_in_time,
                original_clock_out = clock_out_time,
                clock_in_time = COALESCE($2, clock_in_time),
                clock_out_time = COALESCE($3, clock_out_time),
                regular_hours = $6,
                is_corrected = true,
                corrected_by = $4,
                corrected_at = NOW(),
                correction_reason = $5,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, work_date, clock_in_time, clock_out_time,
                    regular_hours, overtime_hours, status, clock_in_source,
                    clock_in_ip::TEXT, clock_out_source, clock_out_ip::TEXT,
                    clock_in_latitude, clock_in_longitude,
                    clock_out_latitude, clock_out_longitude,
                    remark, is_corrected, corrected_by, corrected_at,
                    correction_reason, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(payload.clock_in_time)
        .bind(payload.clock_out_time)
        .bind(corrector_id)
        .bind(&payload.reason)
        .bind(regular_hours)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("correct {} reason={}", after.work_date, payload.reason);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "ATTENDANCE_CORRECT",
                entity: Some(AuditEntity::new("attendance_record", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_regular_hours, format_clock_time, HrService};

    // --- compute_regular_hours（扣除午休工時）---

    /// 以台灣時間 (UTC+8) 的 HH:MM 建構 UTC 時間點
    fn tw(date: chrono::NaiveDate, h: u32, m: u32) -> chrono::DateTime<chrono::Utc> {
        use chrono::TimeZone;
        let naive = date.and_hms_opt(h, m, 0).expect("valid HH:MM");
        crate::time::taiwan_offset()
            .from_local_datetime(&naive)
            .single()
            .expect("fixed offset is always single")
            .with_timezone(&chrono::Utc)
    }

    fn weekday() -> chrono::NaiveDate {
        // 2026-06-11 為週四
        chrono::NaiveDate::from_ymd_opt(2026, 6, 11).expect("valid date")
    }

    fn saturday() -> chrono::NaiveDate {
        // 2026-06-13 為週六
        chrono::NaiveDate::from_ymd_opt(2026, 6, 13).expect("valid date")
    }

    #[test]
    fn test_weekday_full_day_deducts_one_hour() {
        // 08:30–17:30（跨 12:00–13:00）→ 9hr − 1hr 午休 = 8.0
        let d = weekday();
        assert_eq!(compute_regular_hours(tw(d, 8, 30), tw(d, 17, 30), d), 8.0);
    }

    #[test]
    fn test_weekday_morning_only_no_deduction() {
        // 08:30–12:00（未進午休）→ 3.5，不扣
        let d = weekday();
        assert_eq!(compute_regular_hours(tw(d, 8, 30), tw(d, 12, 0), d), 3.5);
    }

    #[test]
    fn test_weekday_afternoon_only_no_deduction() {
        // 13:00–17:30（午休後上班）→ 4.5，不扣
        let d = weekday();
        assert_eq!(compute_regular_hours(tw(d, 13, 0), tw(d, 17, 30), d), 4.5);
    }

    #[test]
    fn test_weekday_partial_overlap_deducts_actual() {
        // 08:30–12:30 → raw 4.0，與午休重疊 30 分 → 3.5
        let d = weekday();
        assert_eq!(compute_regular_hours(tw(d, 8, 30), tw(d, 12, 30), d), 3.5);
    }

    #[test]
    fn test_weekday_exactly_lunch_window_is_zero() {
        // 12:00–13:00 整段在午休 → 0.0
        let d = weekday();
        assert_eq!(compute_regular_hours(tw(d, 12, 0), tw(d, 13, 0), d), 0.0);
    }

    #[test]
    fn test_weekend_duty_no_deduction() {
        // 週六值班 08:30–17:30 → 9.0 全時計，不扣午休
        let d = saturday();
        assert_eq!(compute_regular_hours(tw(d, 8, 30), tw(d, 17, 30), d), 9.0);
    }

    #[test]
    fn test_non_positive_span_is_zero() {
        let d = weekday();
        assert_eq!(compute_regular_hours(tw(d, 17, 30), tw(d, 8, 30), d), 0.0);
    }

    // --- format_clock_time（匯出時區轉換）---

    #[test]
    fn test_format_clock_time_converts_utc_to_taiwan() {
        use chrono::{TimeZone, Utc};
        // 2026-06-09 01:30:00 UTC → 台灣時間 (UTC+8) 應為 09:30:00
        let utc = Utc
            .with_ymd_and_hms(2026, 6, 9, 1, 30, 0)
            .single()
            .expect("valid UTC datetime");
        assert_eq!(format_clock_time(Some(utc)), "09:30:00");
    }

    #[test]
    fn test_format_clock_time_crosses_day_boundary() {
        use chrono::{TimeZone, Utc};
        // 2026-06-09 18:00:00 UTC → 台灣時間隔日 02:00:00（僅取時間部分）
        let utc = Utc
            .with_ymd_and_hms(2026, 6, 9, 18, 0, 0)
            .single()
            .expect("valid UTC datetime");
        assert_eq!(format_clock_time(Some(utc)), "02:00:00");
    }

    #[test]
    fn test_format_clock_time_none_is_dash() {
        assert_eq!(format_clock_time(None), "-");
    }

    // --- is_ip_in_ranges ---

    #[test]
    fn test_ip_exact_match() {
        let ranges = vec!["192.168.1.100".to_string()];
        assert!(HrService::is_ip_in_ranges("192.168.1.100", &ranges));
        assert!(!HrService::is_ip_in_ranges("192.168.1.101", &ranges));
    }

    #[test]
    fn test_ip_cidr_match() {
        let ranges = vec!["10.0.4.0/24".to_string()];
        assert!(HrService::is_ip_in_ranges("10.0.4.1", &ranges));
        assert!(HrService::is_ip_in_ranges("10.0.4.254", &ranges));
        assert!(!HrService::is_ip_in_ranges("10.0.5.1", &ranges));
    }

    #[test]
    fn test_ip_cidr_slash_32() {
        let ranges = vec!["172.16.0.1/32".to_string()];
        assert!(HrService::is_ip_in_ranges("172.16.0.1", &ranges));
        assert!(!HrService::is_ip_in_ranges("172.16.0.2", &ranges));
    }

    #[test]
    fn test_ip_multiple_ranges() {
        let ranges = vec!["192.168.1.0/24".to_string(), "10.0.0.1".to_string()];
        assert!(HrService::is_ip_in_ranges("192.168.1.50", &ranges));
        assert!(HrService::is_ip_in_ranges("10.0.0.1", &ranges));
        assert!(!HrService::is_ip_in_ranges("8.8.8.8", &ranges));
    }

    #[test]
    fn test_ip_empty_ranges() {
        assert!(!HrService::is_ip_in_ranges("192.168.1.1", &[]));
    }

    #[test]
    fn test_ip_invalid_ip() {
        let ranges = vec!["192.168.1.0/24".to_string()];
        assert!(!HrService::is_ip_in_ranges("not-an-ip", &ranges));
    }
}
