// HR 出勤管理

use chrono::Utc;
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
                            let mask = if prefix_len == 0 { 0u32 } else { !0u32 << (32 - prefix_len) };
                            let network_bits = u32::from(network_ip) & mask;
                            let client_bits = u32::from(client_v4) & mask;
                            if network_bits == client_bits {
                                return true;
                            }
                        }
                    }
                }
            } else {
                // 單一 IP 格式：如 "125.231.147.132"
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
            let clock_in = r.clock_in_time.map(|t| t.format("%H:%M:%S").to_string());
            worksheet.write_string(rw, 2, clock_in.as_deref().unwrap_or("-"))?;
            let clock_out = r.clock_out_time.map(|t| t.format("%H:%M:%S").to_string());
            worksheet.write_string(rw, 3, clock_out.as_deref().unwrap_or("-"))?;
            let hours = r.regular_hours.map(|h| format!("{:.1}", h)).unwrap_or_else(|| "-".to_string());
            worksheet.write_string(rw, 4, &hours)?;
            let ot = r.overtime_hours.map(|h| format!("{:.1}", h)).unwrap_or_else(|| "-".to_string());
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

        let after = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            UPDATE attendance_records
            SET clock_out_time = NOW(),
                clock_out_source = $3,
                clock_out_ip = $4::inet,
                clock_out_latitude = $5,
                clock_out_longitude = $6,
                regular_hours = EXTRACT(EPOCH FROM (NOW() - clock_in_time)) / 3600,
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

    /// 將出勤狀態字串轉為中文顯示名稱
    // R26-7: 出勤狀態中文化輔助，目前未串接；留 util 備用
    #[allow(dead_code)]
    pub(super) fn attendance_status_display(status: &str) -> &str {
        match status {
            "normal" => "正常",
            "late" => "遲到",
            "early_leave" => "早退",
            "absent" => "缺勤",
            _ => status,
        }
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

        let after = sqlx::query_as::<_, AttendanceRecord>(
            r#"
            UPDATE attendance_records
            SET original_clock_in = clock_in_time,
                original_clock_out = clock_out_time,
                clock_in_time = COALESCE($2, clock_in_time),
                clock_out_time = COALESCE($3, clock_out_time),
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
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "correct {} reason={}",
            after.work_date, payload.reason
        );
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
    use super::HrService;

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
        let ranges = vec![
            "192.168.1.0/24".to_string(),
            "10.0.0.1".to_string(),
        ];
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

    // --- attendance_status_display ---

    #[test]
    fn test_attendance_status_display_known() {
        assert_eq!(HrService::attendance_status_display("normal"), "正常");
        assert_eq!(HrService::attendance_status_display("late"), "遲到");
        assert_eq!(HrService::attendance_status_display("early_leave"), "早退");
        assert_eq!(HrService::attendance_status_display("absent"), "缺勤");
    }

    #[test]
    fn test_attendance_status_display_unknown_passthrough() {
        assert_eq!(HrService::attendance_status_display("other"), "other");
    }
}
