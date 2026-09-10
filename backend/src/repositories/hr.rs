// HR repository — 出勤統計 SQL 集中（從 handlers/hr/dashboard.rs 下沉，R34-1）
use chrono::NaiveDate;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    constants::DEFAULT_TIMEZONE,
    models::{LeaveStatus, MonthlyAttendanceSummary},
    AppError, Result,
};

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

/// 工時月報：把某月份的出勤紀錄按人彙總成一列。
///
/// 日期區間 inclusive，由呼叫端算好月初 / 月底（`first_day` / `last_day`）傳入，
/// SQL 內不做月份運算——避免時區與閏月在 SQL 與 Rust 兩邊各算一次而分歧。
///
/// `user_id` 為 `Some` 時只回那個人（一般員工看自己）；`None` 回全體有紀錄的人。
///
/// 🔴 **使用者集合 = 有出勤紀錄的人 ∪ 有已核准加班的人**（CodeRabbit 於 PR #35 指出）。
///
/// 原本以 `attendance_records` 為主表 INNER JOIN `users`，理由寫的是「零紀錄者沒有工時可報」
/// ——那句話本身沒錯，錯在**把「沒有出勤列」等同於「沒有工時」**：
/// `overtime_records.attendance_id` 可為 NULL（`002_schema.sql:4100`），而 `create_overtime`
/// 的 INSERT 根本沒有這個欄位、整個 `overtime.rs` 也從不寫 `attendance_records`（實查 0 處）。
/// 所以「假日到場加班、沒打卡」的人**有工時卻沒有出勤列**，在舊查詢下整列消失——
/// 不是加班時數少算，是那個人根本不出現在月報上。
///
/// 改成先用 CTE 取兩邊的使用者聯集，再 LEFT JOIN 出勤明細。聚合式全部沿用：
/// 沒有出勤列的人 `a.*` 皆為 NULL，`FILTER` 條件不成立故計為 0、`SUM` 由 `COALESCE` 補 0，
/// 語意與原本一致，不需要為新情況另寫分支。
pub async fn summarize_monthly_attendance(
    pool: &PgPool,
    first_day: NaiveDate,
    last_day: NaiveDate,
    user_id: Option<Uuid>,
) -> Result<Vec<MonthlyAttendanceSummary>> {
    let sql = r#"
        WITH report_users AS (
            -- 有出勤紀錄的人
            SELECT DISTINCT a.user_id
              FROM attendance_records a
             WHERE a.work_date >= $1
               AND a.work_date <= $2
               AND ($3::uuid IS NULL OR a.user_id = $3)
            UNION
            -- 有已核准加班、但可能完全沒有出勤列的人（overtime_records.attendance_id 可為 NULL）
            SELECT DISTINCT o.user_id
              FROM overtime_records o
             WHERE o.status = 'approved'
               AND o.overtime_date >= $1
               AND o.overtime_date <= $2
               AND ($3::uuid IS NULL OR o.user_id = $3)
        )
        SELECT
            u.id                                              AS user_id,
            u.display_name                                    AS user_name,
            u.email                                           AS user_email,
            COUNT(*) FILTER (WHERE a.clock_in_time IS NOT NULL)::bigint AS work_days,
            -- ::float8 而非留在 numeric：numeric 會被 rust_decimal 序列化成 JSON 字串，
            -- 前端排序就變成字串比較（"9.5" > "168.5"）。見 MonthlyAttendanceSummary 的註解。
            COALESCE(SUM(a.regular_hours), 0)::float8         AS total_regular_hours,
            -- ⚠️ 加班時數取自 **overtime_records**，不是 attendance_records.overtime_hours。
            -- 後者全 backend 只有 SELECT、沒有任何一處寫入，schema 預設 0
            --（`002_schema.sql` attendance_records.overtime_hours），拿它加總會得到恆為 0 的
            -- 假欄位。真正的加班在另一張表，這正是「加班歸加班、正常歸正常」的兩張卡。
            -- 只計 status='approved'（小寫，見 services/hr/overtime.rs:155）：draft 尚未送審、
            -- voided 已作廢，都不該進月報。
            COALESCE((
                SELECT SUM(o.hours) FROM overtime_records o
                WHERE o.user_id = u.id
                  AND o.status = 'approved'
                  AND o.overtime_date >= $1
                  AND o.overtime_date <= $2
            ), 0)::float8                                     AS total_overtime_hours,
            COUNT(*) FILTER (
                WHERE (a.clock_in_time IS NULL) <> (a.clock_out_time IS NULL)
            )::bigint                                         AS incomplete_days,
            COUNT(*) FILTER (WHERE a.is_corrected)::bigint    AS corrected_days
        FROM report_users ru
        INNER JOIN users u ON u.id = ru.user_id
        -- ⚠️ 日期範圍放在 ON 而不是 WHERE：放 WHERE 會把「沒有出勤列」那些人的 NULL 列濾掉，
        -- LEFT JOIN 就退化回 INNER JOIN，這個修正等於沒做。
        LEFT JOIN attendance_records a
               ON a.user_id = u.id
              AND a.work_date >= $1
              AND a.work_date <= $2
        GROUP BY u.id, u.display_name, u.email
        ORDER BY u.display_name
    "#;

    sqlx::query_as::<_, MonthlyAttendanceSummary>(sql)
        .bind(first_day)
        .bind(last_day)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}
