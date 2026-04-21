// HR 加班管理

use chrono::{Datelike, NaiveDate, Timelike, TimeZone, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CreateOvertimeRequest, OvertimeRecord, OvertimeWithUser,
        OvertimeQuery, PaginatedResponse, UpdateOvertimeRequest,
    },
    Result,
};

use super::HrService;

/// 依加班類型回傳補休乘數（非加班費，本系統不計算加班費）。
/// A=平日(1.0), B=休息日(1.33), C=國定假日(1.66), D=天災(2.0)
pub(super) fn overtime_multiplier(overtime_type: &str) -> f64 {
    match overtime_type {
        "A" => 1.0,
        "B" => 1.33,
        "C" => 1.66,
        "D" => 2.0,
        _ => 1.0,
    }
}

// ============================================================
// 勞基法合規常數
// ============================================================

/// 勞基法 §30：每日標準工時上限 8 小時
pub const DAILY_REGULAR_HOURS: f64 = 8.0;

/// 勞基法 §30：每週標準工時上限 40 小時
pub const WEEKLY_REGULAR_HOURS: f64 = 40.0;

/// 勞基法 §32：每月加班時數上限 46 小時
pub const MONTHLY_OVERTIME_LIMIT: f64 = 46.0;

/// 勞基法 §32：特殊情況每月加班上限 54 小時（需勞資協議）
pub const MONTHLY_OVERTIME_LIMIT_EXTENDED: f64 = 54.0;

/// 勞基法 §32：每三個月加班上限 138 小時（46×3）
// R26-7: 勞基法季度上限，目前只實作月度上限；保留常數避免未來忘記法規值。
#[allow(dead_code)]
pub const QUARTERLY_OVERTIME_LIMIT: f64 = 138.0;

/// 平日加班分段結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct WeekdayOvertimeTiers {
    /// 前 2 小時
    pub tier1_hours: f64,
    /// 超過 2 小時
    pub tier2_hours: f64,
    /// 總時數
    pub total_hours: f64,
}

/// 勞基法 §24：平日加班分段計算。
/// - 前 2 小時為第一段
/// - 超過 2 小時為第二段
pub fn split_weekday_overtime(hours: f64) -> WeekdayOvertimeTiers {
    let total = (hours * 2.0).floor() / 2.0; // 四捨五入至 0.5
    let tier1 = total.min(2.0);
    let tier2 = (total - 2.0).max(0.0);
    WeekdayOvertimeTiers {
        tier1_hours: tier1,
        tier2_hours: tier2,
        total_hours: total,
    }
}

/// 加班上限驗證結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct OvertimeLimitCheck {
    /// 本月已累計加班時數
    pub monthly_total: f64,
    /// 本次申請時數
    pub requested_hours: f64,
    /// 合併後總時數
    pub projected_total: f64,
    /// 是否超過標準上限 (46hr)
    pub exceeds_standard_limit: bool,
    /// 是否超過特殊上限 (54hr)
    pub exceeds_extended_limit: bool,
    /// 警告訊息（空表示通過）
    pub warnings: Vec<String>,
}

/// 工時驗證結果
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkHoursValidation {
    /// 當日實際工時
    pub daily_hours: f64,
    /// 本週累計工時
    pub weekly_hours: f64,
    /// 是否超過每日標準 (8hr)
    pub exceeds_daily_limit: bool,
    /// 是否超過每週標準 (40hr)
    pub exceeds_weekly_limit: bool,
    /// 建議加班時數（超出 8 小時部分）
    pub suggested_overtime_hours: f64,
    /// 警告訊息
    pub warnings: Vec<String>,
}

/// 依加班類型回傳補休時數。
/// C/D 類型固定給 8 小時，其餘為 0。
pub(super) fn comp_time_hours_for_type(overtime_type: &str) -> f64 {
    match overtime_type {
        "C" | "D" => 8.0,
        _ => 0.0,
    }
}

/// 將開始與結束分鐘數換算為以 0.5 小時為單位的工作時數。
pub(super) fn calc_hours_from_minutes(start_minutes: i64, end_minutes: i64) -> f64 {
    let raw = (end_minutes - start_minutes) as f64 / 60.0;
    (raw * 2.0).floor() / 2.0
}

impl HrService {
    // ============================================
    // Overtime
    // ============================================

    pub async fn list_overtime(
        pool: &PgPool,
        query: &OvertimeQuery,
    ) -> Result<PaginatedResponse<OvertimeWithUser>> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(500);
        let offset = (page - 1) * per_page;

        // Handle comma-separated status values (e.g., "pending_admin_staff,pending_admin")
        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM overtime_records
            WHERE ($1::uuid IS NULL OR user_id = $1)
              AND ($2::text IS NULL OR status = ANY(string_to_array($2, ',')))
              AND ($3::date IS NULL OR overtime_date >= $3)
              AND ($4::date IS NULL OR overtime_date <= $4)
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE ($1::uuid IS NULL OR o.user_id = $1)
              AND ($2::text IS NULL OR o.status = ANY(string_to_array($2, ',')))
              AND ($3::date IS NULL OR o.overtime_date >= $3)
              AND ($4::date IS NULL OR o.overtime_date <= $4)
            ORDER BY o.overtime_date DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(query.user_id)
        .bind(&query.status)
        .bind(query.from)
        .bind(query.to)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn get_overtime(
        pool: &PgPool,
        id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<OvertimeWithUser> {
        let record = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        let has_view_all = current_user.has_permission("hr.overtime.view_all");
        let is_owner = record.user_id == current_user.id;
        if !has_view_all && !is_owner {
            return Err(AppError::Forbidden("無權存取此加班紀錄".into()));
        }

        Ok(record)
    }

    pub async fn create_overtime(
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateOvertimeRequest,
    ) -> Result<OvertimeWithUser> {
        // 將 NaiveTime 結合 overtime_date 轉換為 DateTime<Utc>
        let start_datetime = Utc.from_utc_datetime(
            &payload.overtime_date.and_time(payload.start_time)
        );
        let end_datetime = Utc.from_utc_datetime(
            &payload.overtime_date.and_time(payload.end_time)
        );
        
        // 計算時數 (從 NaiveTime)，以 0.5 小時為單位四捨五入
        let start_minutes = payload.start_time.hour() as i64 * 60 + payload.start_time.minute() as i64;
        let end_minutes = payload.end_time.hour() as i64 * 60 + payload.end_time.minute() as i64;
        let hours = calc_hours_from_minutes(start_minutes, end_minutes);

        let multiplier = overtime_multiplier(&payload.overtime_type);
        let comp_time_hours = comp_time_hours_for_type(&payload.overtime_type);
        let expires_at = payload.overtime_date + chrono::Duration::days(365);

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO overtime_records (
                id, user_id, overtime_date, start_time, end_time, hours,
                overtime_type, multiplier, comp_time_hours, comp_time_expires_at,
                status, reason
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'draft', $11)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(payload.overtime_date)
        .bind(start_datetime)
        .bind(end_datetime)
        .bind(hours)
        .bind(&payload.overtime_type)
        .bind(multiplier)
        .bind(comp_time_hours)
        .bind(expires_at)
        .bind(&payload.reason)
        .execute(pool)
        .await?;

        let record = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn update_overtime(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
        payload: &UpdateOvertimeRequest,
    ) -> Result<OvertimeWithUser> {
        sqlx::query(
            r#"
            UPDATE overtime_records
            SET start_time = COALESCE($2, start_time),
                end_time = COALESCE($3, end_time),
                overtime_type = COALESCE($4, overtime_type),
                reason = COALESCE($5, reason),
                updated_at = NOW()
            WHERE id = $1 AND status = 'draft'
            "#,
        )
        .bind(id)
        .bind(payload.start_time)
        .bind(payload.end_time)
        .bind(&payload.overtime_type)
        .bind(&payload.reason)
        .execute(pool)
        .await?;

        Self::get_overtime(pool, id, _current_user).await
    }

    pub async fn delete_overtime(pool: &PgPool, id: Uuid, _current_user: &CurrentUser) -> Result<()> {
        sqlx::query("DELETE FROM overtime_records WHERE id = $1 AND status = 'draft'")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn submit_overtime(
        pool: &PgPool,
        id: Uuid,
        _current_user: &CurrentUser,
    ) -> Result<OvertimeWithUser> {
        sqlx::query(
            r#"
            UPDATE overtime_records
            SET status = 'pending_admin_staff', submitted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND status = 'draft'
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_overtime(pool, id, _current_user).await
    }

    pub async fn approve_overtime(
        pool: &PgPool,
        id: Uuid,
        approver_id: Uuid,
        approval_level: &str, // "admin_staff" or "admin"
    ) -> Result<OvertimeWithUser> {
        // Get current record to check status
        let current: OvertimeRecord = sqlx::query_as(
            "SELECT * FROM overtime_records WHERE id = $1"
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        // Determine next status based on current status and approval level
        let (expected_status, next_status, is_final) = match approval_level {
            "admin_staff" => ("pending_admin_staff", "pending_admin", false),
            "admin" => ("pending_admin", "approved", true),
            _ => return Err(AppError::Validation("無效的審核層級".to_string())),
        };

        // Verify current status matches expected
        if current.status != expected_status {
            return Err(AppError::Validation(format!(
                "目前狀態為 {}，無法進行 {} 層級審核",
                current.status, approval_level
            )));
        }

        // Update status
        let record: OvertimeRecord = sqlx::query_as(
            r#"
            UPDATE overtime_records
            SET status = $2, approved_by = $3, approved_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(next_status)
        .bind(approver_id)
        .fetch_one(pool)
        .await?;

        // Record approval in overtime_approvals table
        sqlx::query(
            r#"
            INSERT INTO overtime_approvals (id, overtime_record_id, approver_id, approval_level, action)
            VALUES ($1, $2, $3, $4, 'APPROVE')
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(approver_id)
        .bind(approval_level)
        .execute(pool)
        .await?;

        // Only credit comp_time after final approval (admin level)
        if is_final {
            sqlx::query(
                r#"
                INSERT INTO comp_time_balances (
                    id, user_id, overtime_record_id, original_hours, earned_date, expires_at
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(record.user_id)
            .bind(record.id)
            .bind(record.comp_time_hours)
            .bind(record.overtime_date)
            .bind(record.comp_time_expires_at)
            .execute(pool)
            .await?;
        }

        let result = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    // ============================================================
    // 勞基法合規驗證
    // ============================================================

    /// 勞基法 §32：檢查本月加班上限。
    /// 查詢該員工當月已核准/待審加班時數，加上本次申請是否超限。
    pub async fn check_monthly_overtime_limit(
        pool: &PgPool,
        user_id: Uuid,
        overtime_date: NaiveDate,
        requested_hours: f64,
    ) -> Result<OvertimeLimitCheck> {
        let year = overtime_date.year();
        let month = overtime_date.month();

        let row: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(hours), 0)::float8
            FROM overtime_records
            WHERE user_id = $1
              AND EXTRACT(YEAR FROM overtime_date) = $2
              AND EXTRACT(MONTH FROM overtime_date) = $3
              AND status NOT IN ('rejected', 'cancelled')
            "#,
        )
        .bind(user_id)
        .bind(year as f64)
        .bind(month as f64)
        .fetch_one(pool)
        .await?;

        let monthly_total = row.0;
        let projected = monthly_total + requested_hours;
        let mut warnings = Vec::new();

        if projected > MONTHLY_OVERTIME_LIMIT {
            warnings.push(format!(
                "本月加班將達 {:.1} 小時，超過勞基法§32標準上限 {} 小時",
                projected, MONTHLY_OVERTIME_LIMIT
            ));
        }
        if projected > MONTHLY_OVERTIME_LIMIT_EXTENDED {
            warnings.push(format!(
                "本月加班將達 {:.1} 小時，超過勞基法§32特殊上限 {} 小時（需勞資協議）",
                projected, MONTHLY_OVERTIME_LIMIT_EXTENDED
            ));
        }

        Ok(OvertimeLimitCheck {
            monthly_total,
            requested_hours,
            projected_total: projected,
            exceeds_standard_limit: projected > MONTHLY_OVERTIME_LIMIT,
            exceeds_extended_limit: projected > MONTHLY_OVERTIME_LIMIT_EXTENDED,
            warnings,
        })
    }

    /// 勞基法 §30：驗證日/週工時。
    /// 計算指定日期的實際工時，以及該週的累計工時。
    pub async fn validate_work_hours(
        pool: &PgPool,
        user_id: Uuid,
        work_date: NaiveDate,
    ) -> Result<WorkHoursValidation> {
        use chrono::Datelike;

        // 查詢當日工時
        let daily: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(regular_hours, 0)::float8
            FROM attendance_records
            WHERE user_id = $1 AND work_date = $2
            "#,
        )
        .bind(user_id)
        .bind(work_date)
        .fetch_optional(pool)
        .await?
        .unwrap_or((0.0,));

        // 計算該週的週一到週日
        let weekday = work_date.weekday().num_days_from_monday(); // 0=Mon
        let week_start = work_date - chrono::Duration::days(weekday as i64);
        let week_end = week_start + chrono::Duration::days(6);

        // 查詢本週累計工時
        let weekly: (f64,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(regular_hours), 0)::float8
            FROM attendance_records
            WHERE user_id = $1 AND work_date BETWEEN $2 AND $3
            "#,
        )
        .bind(user_id)
        .bind(week_start)
        .bind(week_end)
        .fetch_one(pool)
        .await?;

        let daily_hours = daily.0;
        let weekly_hours = weekly.0;
        let mut warnings = Vec::new();

        let exceeds_daily = daily_hours > DAILY_REGULAR_HOURS;
        let exceeds_weekly = weekly_hours > WEEKLY_REGULAR_HOURS;
        let suggested_ot = (daily_hours - DAILY_REGULAR_HOURS).max(0.0);

        if exceeds_daily {
            warnings.push(format!(
                "當日工時 {:.1}hr 超過勞基法§30標準 {}hr，建議登錄 {:.1}hr 加班",
                daily_hours, DAILY_REGULAR_HOURS, suggested_ot
            ));
        }
        if exceeds_weekly {
            warnings.push(format!(
                "本週累計 {:.1}hr 超過勞基法§30標準 {}hr",
                weekly_hours, WEEKLY_REGULAR_HOURS
            ));
        }

        Ok(WorkHoursValidation {
            daily_hours,
            weekly_hours,
            exceeds_daily_limit: exceeds_daily,
            exceeds_weekly_limit: exceeds_weekly,
            suggested_overtime_hours: suggested_ot,
            warnings,
        })
    }

    /// 勞基法 §24：計算平日加班分段時數。
    pub fn calculate_weekday_overtime_tiers(hours: f64) -> WeekdayOvertimeTiers {
        split_weekday_overtime(hours)
    }

    pub async fn reject_overtime(
        pool: &PgPool,
        id: Uuid,
        rejecter_id: Uuid,
        reason: &str,
    ) -> Result<OvertimeWithUser> {
        sqlx::query(
            r#"
            UPDATE overtime_records
            SET status = 'rejected', rejected_by = $2, rejected_at = NOW(), rejection_reason = $3, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(rejecter_id)
        .bind(reason)
        .execute(pool)
        .await?;

        let result = sqlx::query_as::<_, OvertimeWithUser>(
            r#"
            SELECT 
                o.id, o.user_id, u.email as user_email, u.display_name as user_name,
                o.overtime_date, o.start_time, o.end_time, o.hours,
                o.overtime_type, o.multiplier, o.comp_time_hours, o.comp_time_expires_at,
                o.status, o.reason
            FROM overtime_records o
            INNER JOIN users u ON o.user_id = u.id
            WHERE o.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{calc_hours_from_minutes, comp_time_hours_for_type, overtime_multiplier, split_weekday_overtime};

    // --- overtime_multiplier ---

    #[test]
    fn test_overtime_multiplier_known_types() {
        assert_eq!(overtime_multiplier("A"), 1.0);
        assert_eq!(overtime_multiplier("B"), 1.33);
        assert_eq!(overtime_multiplier("C"), 1.66);
        assert_eq!(overtime_multiplier("D"), 2.0);
    }

    #[test]
    fn test_overtime_multiplier_unknown_defaults_to_one() {
        assert_eq!(overtime_multiplier("X"), 1.0);
        assert_eq!(overtime_multiplier(""), 1.0);
    }

    // --- comp_time_hours_for_type ---

    #[test]
    fn test_comp_time_hours_c_d_get_eight() {
        assert_eq!(comp_time_hours_for_type("C"), 8.0);
        assert_eq!(comp_time_hours_for_type("D"), 8.0);
    }

    #[test]
    fn test_comp_time_hours_a_b_get_zero() {
        assert_eq!(comp_time_hours_for_type("A"), 0.0);
        assert_eq!(comp_time_hours_for_type("B"), 0.0);
        assert_eq!(comp_time_hours_for_type("X"), 0.0);
    }

    // --- calc_hours_from_minutes ---

    #[test]
    fn test_calc_hours_exact() {
        // 18:00 - 09:00 = 9h
        assert_eq!(calc_hours_from_minutes(540, 1080), 9.0);
    }

    #[test]
    fn test_calc_hours_half_hour_rounding() {
        // 62 分鐘 ≈ 1.0 小時（捨入至 0.5）
        assert_eq!(calc_hours_from_minutes(0, 62), 1.0);
        // 45 分鐘 → 0.5 小時
        assert_eq!(calc_hours_from_minutes(0, 45), 0.5);
        // 30 分鐘 → 0.5 小時
        assert_eq!(calc_hours_from_minutes(0, 30), 0.5);
    }

    #[test]
    fn test_calc_hours_one_and_half() {
        // 90 分鐘 = 1.5 小時
        assert_eq!(calc_hours_from_minutes(0, 90), 1.5);
    }

    // --- split_weekday_overtime (勞基法 §24) ---

    #[test]
    fn test_weekday_tiers_under_two_hours() {
        let t = split_weekday_overtime(1.5);
        assert_eq!(t.tier1_hours, 1.5);
        assert_eq!(t.tier2_hours, 0.0);
        assert_eq!(t.total_hours, 1.5);
    }

    #[test]
    fn test_weekday_tiers_exactly_two_hours() {
        let t = split_weekday_overtime(2.0);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 0.0);
    }

    #[test]
    fn test_weekday_tiers_over_two_hours() {
        let t = split_weekday_overtime(3.5);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 1.5);
        assert_eq!(t.total_hours, 3.5);
    }

    #[test]
    fn test_weekday_tiers_four_hours() {
        let t = split_weekday_overtime(4.0);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 2.0);
    }

    #[test]
    fn test_weekday_tiers_rounds_to_half() {
        // 2h20m = 2.33... → 捨入至 2.0
        let t = split_weekday_overtime(2.33);
        assert_eq!(t.total_hours, 2.0);
        assert_eq!(t.tier1_hours, 2.0);
        assert_eq!(t.tier2_hours, 0.0);
    }

    #[test]
    fn test_weekday_tiers_zero() {
        let t = split_weekday_overtime(0.0);
        assert_eq!(t.tier1_hours, 0.0);
        assert_eq!(t.tier2_hours, 0.0);
    }
}
