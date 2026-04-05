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

/// 依台灣勞基法 §38 計算特休假天數。
///
/// | 年資 | 天數 |
/// |------|------|
/// | 滿 6 個月未滿 1 年 | 3 天 |
/// | 滿 1 年未滿 2 年 | 7 天 |
/// | 滿 2 年未滿 3 年 | 10 天 |
/// | 滿 3 年未滿 5 年 | 14 天 |
/// | 滿 5 年未滿 10 年 | 15 天 |
/// | 滿 10 年以上 | 每年加 1 天，上限 30 天 |
pub fn calculate_annual_leave_days(seniority_months: i32) -> f64 {
    if seniority_months < 6 {
        0.0
    } else if seniority_months < 12 {
        3.0
    } else if seniority_months < 24 {
        7.0
    } else if seniority_months < 36 {
        10.0
    } else if seniority_months < 60 {
        14.0
    } else if seniority_months < 120 {
        15.0
    } else {
        // 滿 10 年起，每多 1 年加 1 天，上限 30 天
        let extra_years = (seniority_months - 120) / 12;
        let days = 15 + extra_years + 1; // 第 10 年本身就是 16 天
        (days as f64).min(30.0)
    }
}

/// 依到職日計算年資月數（至指定日期）
pub fn seniority_months(hire_date: NaiveDate, as_of: NaiveDate) -> i32 {
    let years = as_of.year() - hire_date.year();
    let months = as_of.month() as i32 - hire_date.month() as i32;
    let day_adj = if as_of.day() < hire_date.day() { -1 } else { 0 };
    (years * 12 + months + day_adj).max(0)
}

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

    /// 依勞基法 §38 自動計算特定員工的特休假天數並建立額度。
    /// 根據 hire_date 計算年資，自動推算天數。
    pub async fn auto_calculate_annual_leave(
        pool: &PgPool,
        user_id: Uuid,
        entitlement_year: i32,
        creator_id: Uuid,
    ) -> Result<Option<AnnualLeaveEntitlement>> {
        // 查詢是否已存在
        let existing: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM annual_leave_entitlements WHERE user_id = $1 AND entitlement_year = $2",
        )
        .bind(user_id)
        .bind(entitlement_year)
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Ok(None); // 已存在，不重複建立
        }

        // 查詢到職日
        let hire_info: Option<(Option<NaiveDate>,)> = sqlx::query_as(
            "SELECT hire_date FROM users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let hire_date = hire_info
            .and_then(|h| h.0)
            .ok_or_else(|| AppError::Validation("該員工無到職日，無法自動計算特休假".into()))?;

        // 計算到授予年度年底的年資月數
        let as_of = NaiveDate::from_ymd_opt(entitlement_year, 12, 31)
            .ok_or_else(|| AppError::Internal("invalid date".into()))?;
        let months = seniority_months(hire_date, as_of);
        let days = calculate_annual_leave_days(months);

        if days <= 0.0 {
            return Ok(None); // 年資不足，無特休假
        }

        let payload = CreateAnnualLeaveRequest {
            user_id,
            entitlement_year,
            entitled_days: days,
            hire_date: Some(hire_date),
            calculation_basis: Some(format!(
                "auto_calc_art38:seniority_{}m={}d",
                months, days
            )),
            notes: Some(format!(
                "勞基法§38自動計算：年資{}個月→特休{}天",
                months, days
            )),
        };

        let record = Self::create_annual_leave_entitlement(pool, creator_id, &payload).await?;
        Ok(Some(record))
    }

    /// 批次為所有在職員工自動計算指定年度的特休假。
    /// 回傳成功建立的筆數。
    pub async fn batch_auto_calculate_annual_leave(
        pool: &PgPool,
        entitlement_year: i32,
        creator_id: Uuid,
    ) -> Result<i32> {
        let active_users: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE is_active = true AND hire_date IS NOT NULL",
        )
        .fetch_all(pool)
        .await?;

        let mut count = 0;
        for (uid,) in active_users {
            if let Ok(Some(_)) = Self::auto_calculate_annual_leave(
                pool, uid, entitlement_year, creator_id,
            ).await {
                count += 1;
            }
        }
        Ok(count)
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
    use super::{compute_leave_expiry, calculate_annual_leave_days, seniority_months};
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

    // --- 勞基法 §38 特休假自動計算 ---

    #[test]
    fn test_annual_leave_under_6_months() {
        assert_eq!(calculate_annual_leave_days(0), 0.0);
        assert_eq!(calculate_annual_leave_days(5), 0.0);
    }

    #[test]
    fn test_annual_leave_6_to_12_months() {
        assert_eq!(calculate_annual_leave_days(6), 3.0);
        assert_eq!(calculate_annual_leave_days(11), 3.0);
    }

    #[test]
    fn test_annual_leave_1_to_2_years() {
        assert_eq!(calculate_annual_leave_days(12), 7.0);
        assert_eq!(calculate_annual_leave_days(23), 7.0);
    }

    #[test]
    fn test_annual_leave_2_to_3_years() {
        assert_eq!(calculate_annual_leave_days(24), 10.0);
        assert_eq!(calculate_annual_leave_days(35), 10.0);
    }

    #[test]
    fn test_annual_leave_3_to_5_years() {
        assert_eq!(calculate_annual_leave_days(36), 14.0);
        assert_eq!(calculate_annual_leave_days(59), 14.0);
    }

    #[test]
    fn test_annual_leave_5_to_10_years() {
        assert_eq!(calculate_annual_leave_days(60), 15.0);
        assert_eq!(calculate_annual_leave_days(119), 15.0);
    }

    #[test]
    fn test_annual_leave_10_plus_years() {
        assert_eq!(calculate_annual_leave_days(120), 16.0); // 滿 10 年
        assert_eq!(calculate_annual_leave_days(132), 17.0); // 滿 11 年
        assert_eq!(calculate_annual_leave_days(144), 18.0); // 滿 12 年
    }

    #[test]
    fn test_annual_leave_cap_at_30_days() {
        assert_eq!(calculate_annual_leave_days(120 + 14 * 12), 30.0); // 24 年
        assert_eq!(calculate_annual_leave_days(120 + 20 * 12), 30.0); // 30 年：上限
    }

    // --- seniority_months ---

    #[test]
    fn test_seniority_months_basic() {
        let hire = NaiveDate::from_ymd_opt(2024, 1, 15)
            .expect("valid date: 2024-01-15");
        let as_of = NaiveDate::from_ymd_opt(2026, 1, 15)
            .expect("valid date: 2026-01-15");
        assert_eq!(seniority_months(hire, as_of), 24);
    }

    #[test]
    fn test_seniority_months_mid_month() {
        let hire = NaiveDate::from_ymd_opt(2024, 3, 20)
            .expect("valid date: 2024-03-20");
        let as_of = NaiveDate::from_ymd_opt(2025, 3, 19)
            .expect("valid date: 2025-03-19");
        assert_eq!(seniority_months(hire, as_of), 11); // 未滿 12 個月
    }

    #[test]
    fn test_seniority_months_same_day() {
        let hire = NaiveDate::from_ymd_opt(2024, 6, 1)
            .expect("valid date: 2024-06-01");
        let as_of = NaiveDate::from_ymd_opt(2024, 6, 1)
            .expect("valid date: 2024-06-01");
        assert_eq!(seniority_months(hire, as_of), 0);
    }
}
