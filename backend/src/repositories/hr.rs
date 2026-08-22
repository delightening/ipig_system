// HR repository — 出勤統計 SQL 集中（從 handlers/hr/dashboard.rs 下沉，R34-1）
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{constants::DEFAULT_TIMEZONE, models::LeaveStatus, AppError, Result};

/// 判斷使用者在指定日期是否「正在請假」（已核准且涵蓋該日的假單）。
///
/// 用於員工通知寄送：請假中的收件人不寄 email（站內通知仍照常）。
/// 綁 `LeaveStatus::Approved` 參數（不寫魔術字串）；命中既有
/// `idx_leave_user` / `idx_leave_date_range`。日期區間 inclusive（start_date / end_date 當日皆算）。
pub async fn exists_approved_leave_covering_date(
    pool: &PgPool,
    user_id: Uuid,
    date: NaiveDate,
) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM leave_requests
            WHERE user_id = $1
              AND status = $2::leave_status
              AND start_date <= $3
              AND end_date >= $3
        )
        "#,
    )
    .bind(user_id)
    .bind(LeaveStatus::Approved.as_str())
    .bind(date)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

/// 出勤統計單筆。
///
/// `overtime_hours` 為 `Option<f64>` — None 表示資料缺失（R34-5）：
/// 由前端決定顯示「無資料」或 0，而非後端 silent fallback `unwrap_or(0.0)`。
#[derive(Debug, Clone)]
pub struct AttendanceStat {
    pub user_id: Uuid,
    pub display_name: String,
    pub attendance_days: i64,
    pub late_count: i64,
    pub leave_days: i64,
    pub overtime_hours: Option<f64>,
}

type AttendanceStatRow = (Uuid, String, i64, i64, i64, Option<rust_decimal::Decimal>);

fn row_to_attendance_stat(row: AttendanceStatRow) -> AttendanceStat {
    let (user_id, display_name, attendance_days, late_count, leave_days, overtime_hours) = row;
    // R34-5：保留 NULL → None 的語義；rust_decimal → f64 失敗回 None（非 0.0）
    AttendanceStat {
        user_id,
        display_name,
        attendance_days,
        late_count,
        leave_days,
        overtime_hours: overtime_hours.and_then(|d| d.to_f64()),
    }
}

/// 查詢工作人員出勤統計（儀表板用）。
///
/// 排除 admin 帳號 + SYSTEM_ADMIN/admin role 的使用者。日期區間 inclusive。
///
/// coderabbit-fix: 命名遵循 `list_{entities}_by_{field}`、日期改強型別 `NaiveDate`、
/// 時區用 bind 參數 `$3` 注入避免字串拼接 SQL（即使 `DEFAULT_TIMEZONE` 是 const）。
pub async fn list_attendance_stats_by_date_range(
    pool: &PgPool,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<AttendanceStat>> {
    let sql = r#"
        SELECT
            u.id as user_id, u.display_name,
            COUNT(DISTINCT CASE WHEN a.clock_in_time IS NOT NULL THEN DATE(a.clock_in_time) END) as attendance_days,
            COUNT(DISTINCT CASE WHEN a.clock_in_time IS NOT NULL AND EXTRACT(HOUR FROM a.clock_in_time AT TIME ZONE $3) >= 9
                AND EXTRACT(MINUTE FROM a.clock_in_time AT TIME ZONE $3) > 0 THEN a.id END) as late_count,
            COALESCE((SELECT SUM(l.total_days) FROM leave_requests l
                WHERE l.user_id = u.id AND l.status = 'APPROVED'
                AND l.start_date >= $1 AND l.end_date <= $2), 0)::bigint as leave_days,
            (SELECT SUM(o.hours) FROM overtime_records o
                WHERE o.user_id = u.id AND o.status = 'approved'
                AND o.overtime_date >= $1 AND o.overtime_date <= $2) as overtime_hours
        FROM users u
        LEFT JOIN attendance_records a ON u.id = a.user_id
            AND DATE(a.clock_in_time) >= $1 AND DATE(a.clock_in_time) <= $2
        WHERE u.is_active = true AND u.email != 'admin@ipigsystem.asia'
        AND NOT EXISTS (
            SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = u.id AND (r.code = 'SYSTEM_ADMIN' OR r.code = 'admin')
        )
        GROUP BY u.id, u.display_name ORDER BY u.display_name
    "#;
    let rows: Vec<AttendanceStatRow> = sqlx::query_as(sql)
        .bind(start_date)
        .bind(end_date)
        .bind(DEFAULT_TIMEZONE)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

    Ok(rows.into_iter().map(row_to_attendance_stat).collect())
}
