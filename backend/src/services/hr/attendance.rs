// HR 出勤管理

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc, Weekday};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AttendanceBackfillRequest, AttendanceCorrectionRequest,
        AttendanceQuery, AttendanceRecord, AttendanceWithUser, MonthlyAttendanceSummary,
        PaginatedResponse,
    },
    repositories::hr as hr_repo,
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

/// 補卡／更正理由的長度界線。下限取 4 是為了讓「忘記打卡」這種最常見的正當理由過得了，
/// 同時擋掉空字串與「.」；上限防 audit log 被灌爆。
const MIN_CORRECTION_REASON_CHARS: usize = 4;
const MAX_CORRECTION_REASON_CHARS: usize = 500;

/// 補登紀錄的來源標記，與 `clock_in`/`clock_out` 寫的 `"web"` 區隔，
/// 讓稽核能一眼分辨「本人現場打的」與「他人事後補的」。
const BACKFILL_SOURCE: &str = "backfill";

/// 校驗補卡／更正理由，回傳去頭尾空白後的字串。
fn validate_correction_reason(reason: &str) -> Result<&str> {
    let trimmed = reason.trim();
    let len = trimmed.chars().count();
    if len < MIN_CORRECTION_REASON_CHARS {
        return Err(AppError::Validation(format!(
            "補卡理由至少需 {MIN_CORRECTION_REASON_CHARS} 個字"
        )));
    }
    if len > MAX_CORRECTION_REASON_CHARS {
        return Err(AppError::Validation(format!(
            "補卡理由不得超過 {MAX_CORRECTION_REASON_CHARS} 個字"
        )));
    }
    Ok(trimmed)
}

/// 台灣時區的今日。補卡不得補到未來，判定基準與 `clock_in` 的 `work_date` 一致。
fn taiwan_today() -> Result<NaiveDate> {
    let offset = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| AppError::Internal("invalid timezone offset UTC+8".to_string()))?;
    Ok(Utc::now().with_timezone(&offset).date_naive())
}

/// 把年月換算成 `(月初, 月底)` 的 inclusive 日期區間。
///
/// 月底一律用「下個月 1 號往前一天」求得，不查表也不寫死 28/30/31——
/// 閏年二月與跨年（12 月 → 次年 1 月）都由 chrono 自己處理。
fn month_bounds(year: i32, month: u32) -> Result<(NaiveDate, NaiveDate)> {
    let invalid = || AppError::Validation("年月格式不正確".into());
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(invalid)?;
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(invalid)?;
    let last_day = next_month
        .pred_opt()
        .ok_or_else(|| AppError::Internal("failed to compute last day of month".to_string()))?;
    Ok((first_day, last_day))
}

/// 工時月報的全體合計。
///
/// 抽成型別而不是散在匯出函式裡的五個區域變數，是為了讓「合計列必須蓋滿每一個數值欄」
/// 這件事有地方可以測——CodeRabbit 在 PR #35 抓到的正是漏掉其中兩欄。
#[derive(Debug, Default, PartialEq)]
pub struct MonthlyReportTotals {
    pub work_days: i64,
    pub regular_hours: f64,
    pub overtime_hours: f64,
    pub incomplete_days: i64,
    pub corrected_days: i64,
}

impl MonthlyReportTotals {
    pub fn of(rows: &[MonthlyAttendanceSummary]) -> Self {
        rows.iter().fold(Self::default(), |mut acc, r| {
            acc.work_days += r.work_days;
            acc.regular_hours += r.total_regular_hours;
            acc.overtime_hours += r.total_overtime_hours;
            acc.incomplete_days += r.incomplete_days;
            acc.corrected_days += r.corrected_days;
            acc
        })
    }
}

/// 單筆出勤的最長跨距。超過即視為填錯日期，不是超時工作。
///
/// 取 24 小時而**不是**「必須同一個日曆日」：夜班 22:00 → 隔天 06:00 是合法的，
/// 同日限制會把它擋掉。24 小時足以容納任何真實班別（含加班），又能擋住填錯年月日。
const MAX_ATTENDANCE_SPAN_HOURS: i64 = 24;

/// 校驗「更正後」的最終上下班時間順序與跨距。
///
/// ⚠️ 必須驗**合併後**的值，不能只驗 request 帶來的兩個欄位（CodeRabbit PR #35 指出）：
/// 更正請求可以只帶一邊。只送 `clock_in_time=18:00`、而既有紀錄的
/// `clock_out_time=17:00` 時，request 那兩欄的檢查根本不成立（另一邊是 None），
/// 於是負區間被寫進 DB，`compute_regular_hours` 又把它算成 0.0——
/// 資料庫留下一筆下班早於上班、工時 0 的紀錄，而且沒有任何錯誤訊息。
///
/// ⚠️ 上界同樣必要（CodeRabbit PR #35 第二輪指出）：只檢查「晚於」的話，
/// `work_date` 是 8/25 而 `clock_out` 填成 8/27 會過關，而 `compute_regular_hours`
/// 只扣 `work_date` 當天那一小時午休 → 單日存進 55 小時工時。
/// 前端走 `<input type="time">` + 固定日期到不了這個狀態，但 API 直接打得到，
/// 而補登本來就是「他人代填任意時間」的路徑。
fn validate_final_time_order(
    final_in: Option<DateTime<Utc>>,
    final_out: Option<DateTime<Utc>>,
) -> Result<()> {
    let (Some(ci), Some(co)) = (final_in, final_out) else {
        return Ok(());
    };
    if co <= ci {
        return Err(AppError::Validation("下班時間必須晚於上班時間".into()));
    }
    if (co - ci).num_hours() > MAX_ATTENDANCE_SPAN_HOURS {
        return Err(AppError::Validation(format!(
            "單筆出勤的上下班間隔不得超過 {MAX_ATTENDANCE_SPAN_HOURS} 小時，請確認日期是否填錯"
        )));
    }
    Ok(())
}

/// 補卡不得作用於自己（2026-08-26 使用者裁定）。
///
/// 行政的卡由負責人補、負責人的卡由行政或管理員補——任何人都不能改自己的工時。
/// **admin 一併適用**：`has_permission` 對 admin 短路回 true（`middleware/auth.rs`），
/// 不在此擋的話，系統管理員會是全場唯一能改自己工時的人，那正是最該防的那個洞。
/// 目前 admin / ADMIN_STAFF / DIRECTOR 三個角色都持有補卡權，互補得動，不會鎖死。
fn reject_self_correction(target_user_id: Uuid, operator_id: Uuid) -> Result<()> {
    if target_user_id == operator_id {
        return Err(AppError::Forbidden(
            "不得補登或更正自己的出勤紀錄，請由其他具補卡權限者代為處理".into(),
        ));
    }
    Ok(())
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
        let reason = validate_correction_reason(&payload.reason)?;

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

        reject_self_correction(before.user_id, corrector_id)?;

        // 依更正後的最終上/下班時間重算工時（扣午休）。缺任一時間則保留原值。
        let final_in = payload.clock_in_time.or(before.clock_in_time);
        let final_out = payload.clock_out_time.or(before.clock_out_time);
        // 驗的是**合併後**的值，不是 request 帶來的那兩欄——理由見 validate_final_time_order
        validate_final_time_order(final_in, final_out)?;
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
        .bind(reason)
        .bind(regular_hours)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("correct {} reason={}", after.work_date, reason);
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

    /// 補登出勤（補卡）——為**缺漏日**建立紀錄。
    ///
    /// `correct_attendance` 只 UPDATE 既有 row，整天沒打卡的日子在 DB 裡沒有 row、
    /// 會直接 404；那才是補卡最常見的情境，所以另開這條建立路徑。
    ///
    /// 寫入時把 `is_corrected` 標為 true 並填 `corrected_by` / `correction_reason`，
    /// 讓補登的紀錄在列表與月報上與本人現場打的卡可區分（來源另標 `backfill`）。
    pub async fn backfill_attendance(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &AttendanceBackfillRequest,
    ) -> Result<AttendanceRecord> {
        let user = actor.require_user()?;
        let operator_id = user.id;

        reject_self_correction(payload.user_id, operator_id)?;
        let reason = validate_correction_reason(&payload.reason)?;

        // 兩個時間都沒有 → 補出一筆沒有工時的空紀錄，對月報毫無意義，直接擋掉
        if payload.clock_in_time.is_none() && payload.clock_out_time.is_none() {
            return Err(AppError::Validation(
                "補卡至少需填寫上班或下班其中一個時間".into(),
            ));
        }
        // 補登沒有「合併既有值」這回事（該日本來就沒有 row），兩個時間都來自 request，
        // 但仍走同一個校驗函式，避免兩條路徑日後各自漂移
        validate_final_time_order(payload.clock_in_time, payload.clock_out_time)?;
        if payload.work_date > taiwan_today()? {
            return Err(AppError::Validation("不得補登未來日期的出勤".into()));
        }

        // 目標人員必須存在。刻意**不要求 is_active**：離職當月的工時常常要等
        // 帳號停用之後才結算，要求在職會讓最後一份月報永遠補不齊。
        let target_exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id = $1")
            .bind(payload.user_id)
            .fetch_optional(pool)
            .await?;
        if target_exists.is_none() {
            return Err(AppError::NotFound("指定人員不存在".into()));
        }

        let regular_hours = match (payload.clock_in_time, payload.clock_out_time) {
            (Some(ci), Some(co)) => {
                regular_hours_decimal(compute_regular_hours(ci, co, payload.work_date))
            }
            _ => None,
        };

        let mut tx = pool.begin().await?;

        // UNIQUE (user_id, work_date)：該日已有紀錄就不是「補漏」而是「更正」，
        // 走 PUT /{id}。這裡回 Conflict 而不是靜默覆蓋，避免既有打卡被無聲蓋掉。
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM attendance_records WHERE user_id = $1 AND work_date = $2 FOR UPDATE",
        )
        .bind(payload.user_id)
        .bind(payload.work_date)
        .fetch_optional(&mut *tx)
        .await?;
        if existing.is_some() {
            return Err(AppError::Conflict(
                "該日已有出勤紀錄，請改用更正功能修改時間".into(),
            ));
        }

        let after = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            INSERT INTO attendance_records (
                id, user_id, work_date, clock_in_time, clock_out_time,
                regular_hours, status, clock_in_source, clock_out_source,
                is_corrected, corrected_by, corrected_at, correction_reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'normal', $7, $7, true, $8, NOW(), $9)
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
        .bind(payload.user_id)
        .bind(payload.work_date)
        .bind(payload.clock_in_time)
        .bind(payload.clock_out_time)
        .bind(regular_hours)
        .bind(BACKFILL_SOURCE)
        .bind(operator_id)
        .bind(reason)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("backfill {} reason={}", after.work_date, reason);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "HR",
                event_type: "ATTENDANCE_BACKFILL",
                entity: Some(AuditEntity::new("attendance_record", after.id, &display)),
                // before = None：這筆紀錄在此之前不存在，diff 記的是「無中生有的整列」
                data_diff: Some(DataDiff::compute(None::<&AttendanceRecord>, Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    /// 工時月報：把某年月的出勤按人彙總。年月在此換算成月初 / 月底日期後交給 repository。
    pub async fn monthly_attendance_report(
        pool: &PgPool,
        year: i32,
        month: u32,
        user_id: Option<Uuid>,
    ) -> Result<Vec<MonthlyAttendanceSummary>> {
        let (first_day, last_day) = month_bounds(year, month)?;
        hr_repo::summarize_monthly_attendance(pool, first_day, last_day, user_id).await
    }

    /// 工時月報匯出 Excel。欄位與畫面上的月報表一致，最後一列為全體合計。
    pub async fn export_monthly_report_to_excel(
        pool: &PgPool,
        year: i32,
        month: u32,
        user_id: Option<Uuid>,
    ) -> Result<Vec<u8>> {
        use rust_xlsxwriter::{Format, FormatAlign, Workbook};

        let rows = Self::monthly_attendance_report(pool, year, month, user_id).await?;

        let mut workbook = Workbook::new();
        let header_format = Format::new()
            .set_bold()
            .set_background_color("#4472C4")
            .set_font_color("#FFFFFF")
            .set_align(FormatAlign::Center);
        let total_format = Format::new().set_bold();

        let worksheet = workbook.add_worksheet();
        worksheet.set_name(format!("{year}-{month:02} 工時月報"))?;
        worksheet.set_column_width(0, 25.0)?;
        worksheet.set_column_width(1, 28.0)?;
        for col in 2..=6 {
            worksheet.set_column_width(col, 14.0)?;
        }

        for (col, title) in [
            "人員名稱",
            "Email",
            "出勤天數",
            "總工時",
            "總加班時數",
            "打卡不完整天數",
            "補登／更正天數",
        ]
        .iter()
        .enumerate()
        {
            worksheet.write_string_with_format(0, col as u16, *title, &header_format)?;
        }

        let totals = MonthlyReportTotals::of(&rows);

        for (idx, r) in rows.iter().enumerate() {
            let row = idx as u32 + 1;
            worksheet.write_string(row, 0, &r.user_name)?;
            worksheet.write_string(row, 1, &r.user_email)?;
            worksheet.write_number(row, 2, r.work_days as f64)?;
            worksheet.write_number(row, 3, r.total_regular_hours)?;
            worksheet.write_number(row, 4, r.total_overtime_hours)?;
            worksheet.write_number(row, 5, r.incomplete_days as f64)?;
            worksheet.write_number(row, 6, r.corrected_days as f64)?;
        }

        // 合計列要蓋滿每一個數值欄。漏掉 incomplete / corrected 兩欄會讓 Excel 的合計
        // 與畫面上的合計不一致——看報表的人會以為那兩欄沒有值（CodeRabbit PR #35 指出）。
        let total_row = rows.len() as u32 + 1;
        worksheet.write_string_with_format(total_row, 0, "合計", &total_format)?;
        worksheet.write_number(total_row, 2, totals.work_days as f64)?;
        worksheet.write_number(total_row, 3, totals.regular_hours)?;
        worksheet.write_number(total_row, 4, totals.overtime_hours)?;
        worksheet.write_number(total_row, 5, totals.incomplete_days as f64)?;
        worksheet.write_number(total_row, 6, totals.corrected_days as f64)?;

        worksheet.set_freeze_panes(1, 0)?;
        Ok(workbook.save_to_buffer()?)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_regular_hours, format_clock_time, month_bounds, reject_self_correction,
        validate_correction_reason, validate_final_time_order, HrService, MonthlyAttendanceSummary,
        MonthlyReportTotals, MAX_CORRECTION_REASON_CHARS,
    };
    use uuid::Uuid;

    // --- 更正的時間順序：必須驗「合併後」的值 ---

    fn utc(h: u32, m: u32) -> chrono::DateTime<chrono::Utc> {
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms(2026, 8, 25, h, m, 0)
            .single()
            .expect("valid timestamp")
    }

    /// 核心迴歸（CodeRabbit PR #35 抓到的洞）：更正請求只帶一邊時，
    /// 必須拿它與**既有紀錄的另一邊**合併後再驗順序。
    /// 只驗 request 那兩欄的話，這個情境會靜默寫入負區間、工時被算成 0.0。
    #[test]
    fn partial_correction_with_inverted_merged_range_is_rejected() {
        // request 只帶 clock_in=18:00，既有紀錄的 clock_out=17:00
        let err = validate_final_time_order(Some(utc(18, 0)), Some(utc(17, 0)));
        assert!(
            err.is_err(),
            "合併後下班早於上班必須擋下，否則 DB 會留下負區間 + 工時 0 的紀錄"
        );
    }

    #[test]
    fn equal_in_and_out_is_rejected() {
        assert!(validate_final_time_order(Some(utc(9, 0)), Some(utc(9, 0))).is_err());
    }

    /// 上界迴歸（CodeRabbit PR #35 第二輪）：`work_date` 是 8/25、`clock_out` 卻填 8/27，
    /// 只檢查「晚於」會放行，而 `compute_regular_hours` 只扣當天午休 → 單日 55 小時工時。
    #[test]
    fn multi_day_span_is_rejected() {
        use chrono::TimeZone;
        let ci = utc(0, 0); // 2026-08-25 00:00Z
        let co = chrono::Utc
            .with_ymd_and_hms(2026, 8, 27, 8, 0, 0)
            .single()
            .expect("valid timestamp");
        assert!(
            validate_final_time_order(Some(ci), Some(co)).is_err(),
            "跨兩天以上必須擋下，否則單日會存進數十小時工時"
        );
    }

    /// 但夜班（跨日、未超過 24 小時）必須放行——用「同一個日曆日」當判準會把它擋掉。
    #[test]
    fn overnight_shift_within_24h_is_allowed() {
        use chrono::TimeZone;
        let ci = utc(14, 0); // 2026-08-25 22:00 台灣時間
        let co = chrono::Utc
            .with_ymd_and_hms(2026, 8, 26, 0, 0, 0)
            .single()
            .expect("valid timestamp"); // 隔天 08:00 台灣時間
        validate_final_time_order(Some(ci), Some(co)).expect("夜班跨日應放行");
    }

    #[test]
    fn normal_range_and_one_sided_values_are_allowed() {
        validate_final_time_order(Some(utc(1, 0)), Some(utc(9, 0))).expect("正常區間應放行");
        // 只有一邊 → 無從比較，放行（工時保留原值 / 標為不完整）
        validate_final_time_order(Some(utc(1, 0)), None).expect("只有上班卡應放行");
        validate_final_time_order(None, Some(utc(9, 0))).expect("只有下班卡應放行");
        validate_final_time_order(None, None).expect("兩邊皆無應放行");
    }

    // --- 工時月報合計 ---

    fn summary(
        days: i64,
        reg: f64,
        ot: f64,
        incomplete: i64,
        corrected: i64,
    ) -> MonthlyAttendanceSummary {
        MonthlyAttendanceSummary {
            user_id: Uuid::new_v4(),
            user_name: "測試員一".into(),
            user_email: "staff@example.com".into(),
            work_days: days,
            total_regular_hours: reg,
            total_overtime_hours: ot,
            incomplete_days: incomplete,
            corrected_days: corrected,
        }
    }

    /// 合計要蓋滿**每一個**數值欄。Excel 合計列漏欄會與畫面上的合計不一致，
    /// 看報表的人會以為那兩欄沒有值（CodeRabbit PR #35 指出）。
    #[test]
    fn totals_cover_every_numeric_column() {
        let totals = MonthlyReportTotals::of(&[
            summary(21, 168.5, 6.0, 1, 2),
            summary(22, 176.0, 0.0, 0, 3),
        ]);
        assert_eq!(totals.work_days, 43);
        assert_eq!(totals.regular_hours, 344.5);
        assert_eq!(totals.overtime_hours, 6.0);
        assert_eq!(
            totals.incomplete_days, 1,
            "漏掉 incomplete_days 就是那個 bug"
        );
        assert_eq!(totals.corrected_days, 5, "漏掉 corrected_days 就是那個 bug");
    }

    #[test]
    fn totals_of_empty_report_are_all_zero() {
        assert_eq!(MonthlyReportTotals::of(&[]), MonthlyReportTotals::default());
    }

    // --- 補卡：不得作用於自己 ---

    /// 核心迴歸：補卡／更正一律不得改到自己的紀錄（2026-08-26 使用者裁定）。
    /// 這條若退化，持有補卡權的人就能自己給自己補工時，稽核上是最嚴重的那個洞。
    #[test]
    fn self_correction_is_rejected() {
        let me = Uuid::new_v4();
        assert!(
            reject_self_correction(me, me).is_err(),
            "補自己的卡必須被擋下"
        );
    }

    #[test]
    fn correcting_someone_else_is_allowed() {
        let me = Uuid::new_v4();
        let colleague = Uuid::new_v4();
        reject_self_correction(colleague, me).expect("補別人的卡應該放行");
    }

    // --- 補卡理由長度 ---

    /// 下限刻意取 4，讓最常見的正當理由「忘記打卡」（4 字）過得了。
    /// 這條是把該邊界釘住：改成 5 就會把最常見的案例擋在門外。
    #[test]
    fn four_character_reason_is_accepted() {
        assert_eq!(
            validate_correction_reason("忘記打卡").expect("4 字應通過"),
            "忘記打卡"
        );
    }

    #[test]
    fn reason_is_trimmed_before_length_check() {
        assert_eq!(
            validate_correction_reason("  忘記打卡  ").expect("去空白後 4 字應通過"),
            "忘記打卡",
            "理由要去頭尾空白後才判長度，也才是寫進 DB 的值"
        );
        assert!(
            validate_correction_reason("      ").is_err(),
            "只有空白的理由等同沒填"
        );
    }

    #[test]
    fn too_short_or_too_long_reason_is_rejected() {
        assert!(validate_correction_reason("").is_err());
        assert!(validate_correction_reason("忘").is_err());
        let too_long = "字".repeat(MAX_CORRECTION_REASON_CHARS + 1);
        assert!(
            validate_correction_reason(&too_long).is_err(),
            "超過上限的理由會灌爆 audit log"
        );
    }

    // --- 工時月報的月份邊界 ---

    /// 月底用「下個月 1 號往前一天」求得，閏年二月與跨年都不能算錯，
    /// 否則月報會漏掉當月最後一天的工時。
    #[test]
    fn month_bounds_handles_leap_year_and_year_rollover() {
        let (first, last) = month_bounds(2028, 2).expect("2028 是閏年");
        assert_eq!(first.to_string(), "2028-02-01");
        assert_eq!(last.to_string(), "2028-02-29", "閏年二月是 29 天");

        let (first, last) = month_bounds(2026, 12).expect("十二月");
        assert_eq!(first.to_string(), "2026-12-01");
        assert_eq!(last.to_string(), "2026-12-31", "十二月的下個月是次年一月");

        let (_, last) = month_bounds(2027, 2).expect("平年二月");
        assert_eq!(last.to_string(), "2027-02-28");
    }

    #[test]
    fn month_bounds_rejects_invalid_month() {
        assert!(month_bounds(2026, 0).is_err());
        assert!(month_bounds(2026, 13).is_err());
    }

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
