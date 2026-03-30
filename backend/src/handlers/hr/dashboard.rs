// HR 儀表板 + 員工列表 Handlers

use axum::{
    extract::{Query, State},
    Extension, Json,
};
use chrono::Datelike;

use crate::{
    middleware::CurrentUser,
    models::DashboardCalendarData,
    services::HrService,
    AppState, Result,
};

/// 工作人員出勤統計（儀表板用）
pub async fn get_attendance_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.has_permission("hr.attendance.view_all") && !current_user.is_admin() {
        return Err(crate::error::AppError::Forbidden("無權查看出勤統計".to_string()));
    }
    let start_date = params.get("start_date").cloned();
    let end_date = params.get("end_date").cloned();
    let (start_date, end_date) = match (start_date, end_date) {
        (Some(s), Some(e)) => (s, e),
        _ => {
            let now = chrono::Utc::now();
            let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .unwrap_or_else(|| now.date_naive());
            let end = now.date_naive();
            (start.format("%Y-%m-%d").to_string(), end.format("%Y-%m-%d").to_string())
        }
    };
    let stats = sqlx::query_as::<_, (uuid::Uuid, String, i64, i64, i64, rust_decimal::Decimal)>(
        r#"
        SELECT 
            u.id as user_id, u.display_name,
            COUNT(DISTINCT CASE WHEN a.clock_in_time IS NOT NULL THEN DATE(a.clock_in_time) END) as attendance_days,
            COUNT(DISTINCT CASE WHEN a.clock_in_time IS NOT NULL AND EXTRACT(HOUR FROM a.clock_in_time AT TIME ZONE 'Asia/Taipei') >= 9 
                AND EXTRACT(MINUTE FROM a.clock_in_time AT TIME ZONE 'Asia/Taipei') > 0 THEN a.id END) as late_count,
            COALESCE((SELECT SUM(l.total_days) FROM leave_requests l
                WHERE l.user_id = u.id AND l.status = 'APPROVED'
                AND l.start_date >= $1::date AND l.end_date <= $2::date), 0)::bigint as leave_days,
            COALESCE((SELECT SUM(o.hours) FROM overtime_records o
                WHERE o.user_id = u.id AND o.status = 'approved'
                AND o.overtime_date >= $1::date AND o.overtime_date <= $2::date), 0) as overtime_hours
        FROM users u
        LEFT JOIN attendance_records a ON u.id = a.user_id 
            AND DATE(a.clock_in_time) >= $1::date AND DATE(a.clock_in_time) <= $2::date
        WHERE u.is_active = true AND u.email != 'admin@ipig.local'
        AND NOT EXISTS (
            SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = u.id AND (r.code = 'SYSTEM_ADMIN' OR r.code = 'admin')
        )
        GROUP BY u.id, u.display_name ORDER BY u.display_name
        "#
    ).bind(&start_date).bind(&end_date).fetch_all(&state.db).await?;
    let data: Vec<serde_json::Value> = stats.into_iter()
        .map(|(user_id, display_name, attendance_days, late_count, leave_days, overtime_hours)| {
            serde_json::json!({
                "user_id": user_id.to_string(), "display_name": display_name,
                "attendance_days": attendance_days, "late_count": late_count,
                "leave_days": leave_days,
                "overtime_hours": overtime_hours.to_string().parse::<f64>().unwrap_or(0.0)
            })
        }).collect();
    Ok(Json(serde_json::json!({ "data": data })))
}

/// 取得儀表板日曆資料
pub async fn get_dashboard_calendar(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<DashboardCalendarData>> {
    let data = HrService::get_dashboard_calendar(&state.db).await?;
    Ok(Json(data))
}

/// 工作人員簡易資訊
#[derive(Debug, serde::Serialize)]
pub struct StaffInfo {
    pub id: uuid::Uuid,
    pub display_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub entry_date: Option<chrono::NaiveDate>,
    pub position: Option<String>,
    pub aup_roles: Vec<String>,
    pub years_experience: i32,
    pub trainings: serde_json::Value,
}

/// 工作人員列表（供請假代理人選擇）
pub async fn list_staff_for_proxy(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<StaffInfo>>> {
    let staff = sqlx::query_as::<_, (
        uuid::Uuid, String, String, Option<String>, Option<String>,
        Option<chrono::NaiveDate>, Option<String>, Vec<String>, i32, serde_json::Value
    )>(
        r#"SELECT DISTINCT u.id, u.display_name, u.email, u.phone, u.organization,
               u.entry_date, u.position, u.aup_roles, u.years_experience, u.trainings
        FROM users u
        INNER JOIN user_roles ur ON u.id = ur.user_id
        INNER JOIN roles r ON ur.role_id = r.id
        WHERE u.is_active = true AND r.code = 'EXPERIMENT_STAFF'
        ORDER BY u.display_name"#
    ).fetch_all(&state.db).await?;
    let result: Vec<StaffInfo> = staff.into_iter()
        .map(|(id, display_name, email, phone, organization, entry_date, position, aup_roles, years_experience, trainings)|
            StaffInfo { id, display_name, email, phone, organization, entry_date, position, aup_roles, years_experience, trainings })
        .collect();
    Ok(Json(result))
}

/// 內部員工列表（排除 admin；供特休管理、人員訓練等使用）
pub async fn list_internal_users_for_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<StaffInfo>>> {
    if !current_user.is_admin()
        && !current_user.roles.contains(&crate::constants::ROLE_ADMIN_STAFF.to_string())
        && !current_user.has_permission("hr.balance.manage")
        && !current_user.has_permission("training.view")
        && !current_user.has_permission("training.manage")
    {
        return Err(crate::error::AppError::Forbidden("無權查看員工列表".to_string()));
    }
    let staff = sqlx::query_as::<_, (
        uuid::Uuid, String, String, Option<String>, Option<String>,
        Option<chrono::NaiveDate>, Option<String>, Vec<String>, i32, serde_json::Value
    )>(
        r#"SELECT u.id, u.display_name, u.email, u.phone, u.organization,
               u.entry_date, u.position, u.aup_roles, u.years_experience, u.trainings
        FROM users u
        WHERE u.is_active = true AND u.is_internal = true AND u.email != 'admin@ipig.local'
        AND NOT EXISTS (
            SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = u.id AND (r.code = 'SYSTEM_ADMIN' OR r.code = 'admin')
        )
        ORDER BY u.display_name"#
    ).fetch_all(&state.db).await?;
    let result: Vec<StaffInfo> = staff.into_iter()
        .map(|(id, display_name, email, phone, organization, entry_date, position, aup_roles, years_experience, trainings)|
            StaffInfo { id, display_name, email, phone, organization, entry_date, position, aup_roles, years_experience, trainings })
        .collect();
    Ok(Json(result))
}
