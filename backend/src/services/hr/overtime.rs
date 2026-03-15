// HR 加班管理

use chrono::{Timelike, TimeZone, Utc};
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

/// 依加班類型回傳薪資乘數。
/// A=平日(1.0), B=假日(1.33), C=國定假日(1.66), D=天災(2.0)
pub(super) fn overtime_multiplier(overtime_type: &str) -> f64 {
    match overtime_type {
        "A" => 1.0,
        "B" => 1.33,
        "C" => 1.66,
        "D" => 2.0,
        _ => 1.0,
    }
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
    use super::{calc_hours_from_minutes, comp_time_hours_for_type, overtime_multiplier};

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
}
