// HR 餘額管理（特休假 + 補休）

use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        AdjustBalanceRequest, AnnualLeaveBalanceView, AnnualLeaveEntitlement, BalanceSummary,
        CompTimeBalanceView, CreateAnnualLeaveRequest, ExpiredLeaveReport,
    },
    repositories,
    Result,
};

use super::HrService;

/// 計算特休假到期日。
/// 到期年 = 授予年 + 2。
/// - 若有到職日：優先以同月同日為到期日（例外處理 2/29）
/// - 無到職日：12/31
pub(super) fn compute_leave_expiry(
    entitlement_year: i32,
    hire_date: Option<NaiveDate>,
) -> Option<NaiveDate> {
    let yr = entitlement_year + 2;
    if let Some(hd) = hire_date {
        NaiveDate::from_ymd_opt(yr, hd.month(), hd.day())
            .or_else(|| NaiveDate::from_ymd_opt(yr, hd.month(), 28))
            .or_else(|| NaiveDate::from_ymd_opt(yr, 12, 31))
    } else {
        NaiveDate::from_ymd_opt(yr, 12, 31)
            .or_else(|| NaiveDate::from_ymd_opt(yr, 12, 30))
    }
}

impl HrService {
    pub async fn get_annual_leave_balances(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<AnnualLeaveBalanceView>> {
        let rows: Vec<(i32, f64, f64, f64, NaiveDate, i32, bool)> = sqlx::query_as(
            r#"
            SELECT 
                entitlement_year,
                entitled_days::float8,
                used_days::float8,
                (entitled_days - used_days)::float8 as remaining_days,
                expires_at,
                (expires_at - CURRENT_DATE)::integer as days_until_expiry,
                is_expired OR (expires_at < CURRENT_DATE) as is_expired
            FROM annual_leave_entitlements
            WHERE user_id = $1 
              AND (entitled_days - used_days) > 0
            ORDER BY expires_at ASC, entitlement_year DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let balances = rows
            .into_iter()
            .map(|r| AnnualLeaveBalanceView {
                entitlement_year: r.0,
                entitled_days: r.1,
                used_days: r.2,
                remaining_days: r.3,
                expires_at: r.4,
                days_until_expiry: r.5,
                is_expired: r.6,
            })
            .collect();
        Ok(balances)
    }

    pub async fn get_comp_time_balances(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<CompTimeBalanceView>> {
        let rows: Vec<(Uuid, NaiveDate, f64, f64, f64, NaiveDate, i32)> = sqlx::query_as(
            r#"
            SELECT id, earned_date, original_hours::float8, used_hours::float8,
                   (original_hours - used_hours)::float8 as remaining_hours,
                   expires_at, (expires_at - CURRENT_DATE)::integer as days_until_expiry
            FROM comp_time_balances
            WHERE user_id = $1 AND is_expired = false
            ORDER BY expires_at ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        let balances = rows
            .into_iter()
            .map(|r| CompTimeBalanceView {
                id: r.0,
                earned_date: r.1,
                original_hours: r.2,
                used_hours: r.3,
                remaining_hours: r.4,
                expires_at: r.5,
                days_until_expiry: r.6,
            })
            .collect();
        Ok(balances)
    }

    pub async fn get_balance_summary(pool: &PgPool, user_id: Uuid) -> Result<BalanceSummary> {
        let user_name_str = repositories::user::find_user_display_name_by_id(pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User not found: {}", user_id)))?;
        let user_name: (String,) = (user_name_str,);

        let annual: (f64, f64) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(entitled_days), 0)::float8, COALESCE(SUM(used_days), 0)::float8
            FROM annual_leave_entitlements WHERE user_id = $1 AND is_expired = false"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let comp: (f64, f64) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(original_hours), 0)::float8, COALESCE(SUM(used_hours), 0)::float8
            FROM comp_time_balances WHERE user_id = $1 AND is_expired = false"#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let expiring_annual: (f64,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(entitled_days - used_days), 0)::float8
            FROM annual_leave_entitlements
            WHERE user_id = $1 AND is_expired = false AND expires_at <= CURRENT_DATE + INTERVAL '14 days'"#,
        ).bind(user_id).fetch_one(pool).await?;

        let expiring_comp: (f64,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(original_hours - used_hours), 0)::float8
            FROM comp_time_balances
            WHERE user_id = $1 AND is_expired = false AND expires_at <= CURRENT_DATE + INTERVAL '14 days'"#,
        ).bind(user_id).fetch_one(pool).await?;

        Ok(BalanceSummary {
            user_id,
            user_name: user_name.0,
            annual_leave_total: annual.0,
            annual_leave_used: annual.1,
            annual_leave_remaining: annual.0 - annual.1,
            comp_time_total: comp.0,
            comp_time_used: comp.1,
            comp_time_remaining: comp.0 - comp.1,
            expiring_soon_days: expiring_annual.0,
            expiring_soon_hours: expiring_comp.0,
        })
    }

    pub async fn create_annual_leave_entitlement(
        pool: &PgPool,
        creator_id: Uuid,
        payload: &CreateAnnualLeaveRequest,
    ) -> Result<AnnualLeaveEntitlement> {
        let id = Uuid::new_v4();
        let yr = payload.entitlement_year + 2;
        let expires_at = compute_leave_expiry(payload.entitlement_year, payload.hire_date)
            .ok_or_else(|| AppError::Internal(format!("invalid expiry date for year {yr}")))?;

        let record = sqlx::query_as::<_, AnnualLeaveEntitlement>(
            r#"INSERT INTO annual_leave_entitlements (
                id, user_id, entitlement_year, entitled_days, expires_at, calculation_basis, notes, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
        )
        .bind(id).bind(payload.user_id).bind(payload.entitlement_year)
        .bind(payload.entitled_days).bind(expires_at)
        .bind(&payload.calculation_basis).bind(&payload.notes).bind(creator_id)
        .fetch_one(pool).await?;
        Ok(record)
    }

    pub async fn adjust_annual_leave(
        pool: &PgPool,
        id: Uuid,
        _adjuster_id: Uuid,
        payload: &AdjustBalanceRequest,
    ) -> Result<AnnualLeaveEntitlement> {
        let record = sqlx::query_as::<_, AnnualLeaveEntitlement>(
            r#"UPDATE annual_leave_entitlements
            SET entitled_days = entitled_days + $2, notes = COALESCE(notes || E'\n', '') || $3, updated_at = NOW()
            WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(payload.adjustment_days).bind(&payload.reason)
        .fetch_one(pool).await?;
        Ok(record)
    }

    pub async fn get_expired_leave_compensation_report(
        pool: &PgPool,
    ) -> Result<Vec<ExpiredLeaveReport>> {
        #[allow(clippy::type_complexity)]
        let rows: Vec<(Uuid, String, String, i32, f64, f64, f64, NaiveDate)> = sqlx::query_as(
            r#"SELECT ale.user_id, u.display_name, u.email, ale.entitlement_year,
                ale.entitled_days::float8, ale.used_days::float8,
                (ale.entitled_days - ale.used_days)::float8, ale.expires_at
            FROM annual_leave_entitlements ale INNER JOIN users u ON ale.user_id = u.id
            WHERE (ale.is_expired = true OR ale.expires_at < CURRENT_DATE)
              AND (ale.entitled_days - ale.used_days) > 0 AND u.is_active = true
            ORDER BY ale.expires_at ASC, u.display_name"#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ExpiredLeaveReport {
                user_id: r.0,
                user_name: r.1,
                user_email: r.2,
                entitlement_year: r.3,
                entitled_days: r.4,
                used_days: r.5,
                remaining_days: r.6,
                expires_at: r.7,
            })
            .collect())
    }

    pub async fn copy_previous_year_entitlement(
        pool: &PgPool,
        user_id: Uuid,
        new_year: i32,
        creator_id: Uuid,
        hire_date: Option<NaiveDate>,
    ) -> Result<Option<AnnualLeaveEntitlement>> {
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM annual_leave_entitlements WHERE user_id = $1 AND entitlement_year = $2",
        )
        .bind(user_id)
        .bind(new_year)
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Ok(None);
        }

        let previous: Option<(f64, Option<String>)> = sqlx::query_as(
            r#"SELECT entitled_days::float8, notes FROM annual_leave_entitlements WHERE user_id = $1 AND entitlement_year = $2"#,
        ).bind(user_id).bind(new_year - 1).fetch_optional(pool).await?;

        if let Some((entitled_days, notes)) = previous {
            let payload = CreateAnnualLeaveRequest {
                user_id,
                entitlement_year: new_year,
                entitled_days,
                hire_date,
                calculation_basis: Some("copied_from_previous".to_string()),
                notes: Some(format!(
                    "沿用{}年度設定。{}",
                    new_year - 1,
                    notes.unwrap_or_default()
                )),
            };
            let record = Self::create_annual_leave_entitlement(pool, creator_id, &payload).await?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compute_leave_expiry;
    use chrono::NaiveDate;

    // --- compute_leave_expiry ---

    #[test]
    fn test_expiry_without_hire_date() {
        // 2024 年授予 → 到期年 2026/12/31
        let result = compute_leave_expiry(2024, None);
        assert_eq!(result, NaiveDate::from_ymd_opt(2026, 12, 31));
    }

    #[test]
    fn test_expiry_with_hire_date_normal() {
        // 2023 年授予，到職日 2020-05-15 → 到期 2025-05-15
        let hire_date = NaiveDate::from_ymd_opt(2020, 5, 15);
        let result = compute_leave_expiry(2023, hire_date);
        assert_eq!(result, NaiveDate::from_ymd_opt(2025, 5, 15));
    }

    #[test]
    fn test_expiry_with_hire_date_feb29_leap() {
        // 到職日 2020-02-29（閏年），到期年 2023（非閏年）→ 退回到 02-28
        let hire_date = NaiveDate::from_ymd_opt(2020, 2, 29);
        let result = compute_leave_expiry(2021, hire_date); // 到期年 2023（非閏年）
        assert_eq!(result, NaiveDate::from_ymd_opt(2023, 2, 28));
    }

    #[test]
    fn test_expiry_year_calculation() {
        // 確認 entitlement_year + 2 邏輯
        let result = compute_leave_expiry(2025, None);
        assert_eq!(result, NaiveDate::from_ymd_opt(2027, 12, 31));
    }
}
