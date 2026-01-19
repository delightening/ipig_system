// HR Handlers
// 包含：Attendance, Overtime, Leave, Balances

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Datelike;
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        AdjustBalanceRequest, AnnualLeaveBalanceView, AnnualLeaveEntitlement,
        ApproveLeaveRequest, AttendanceCorrectionRequest, AttendanceQuery, AttendanceWithUser,
        BalanceQuery, BalanceSummary, CancelLeaveRequest, ClockInRequest, ClockOutRequest,
        CompTimeBalanceView, CreateAnnualLeaveRequest, CreateLeaveRequest, CreateOvertimeRequest,
        DashboardCalendarData, ExpiredLeaveReport, LeaveQuery, LeaveRequest, LeaveRequestWithUser, OvertimeQuery, OvertimeWithUser,
        PaginatedResponse, RejectLeaveRequest, RejectOvertimeRequest, UpdateLeaveRequest,
        UpdateOvertimeRequest,
    },
    services::HrService,
    AppState, Result,
};

// ============================================
// Attendance Handlers
// ============================================

/// 列出出勤記錄
pub async fn list_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Json<PaginatedResponse<AttendanceWithUser>>> {
    // 如果沒有指定 user_id，預設查自己的記錄
    let mut query = params;
    if query.user_id.is_none() && !current_user.has_permission("hr.attendance.view.all") {
        query.user_id = Some(current_user.id);
    }
    
    let result = HrService::list_attendance(&state.db, &query).await?;
    Ok(Json(result))
}

/// 打卡上班
pub async fn clock_in(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ClockInRequest>,
) -> Result<Json<serde_json::Value>> {
    let record = HrService::clock_in(
        &state.db,
        current_user.id,
        payload.source.as_deref(),
        None, // IP address not available in CurrentUser
    )
    .await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "clock_in_time": record.clock_in_time,
        "message": "打卡成功"
    })))
}

/// 打卡下班
pub async fn clock_out(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ClockOutRequest>,
) -> Result<Json<serde_json::Value>> {
    let record = HrService::clock_out(
        &state.db,
        current_user.id,
        payload.source.as_deref(),
        None,
    )
    .await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "clock_out_time": record.clock_out_time,
        "regular_hours": record.regular_hours,
        "message": "打卡成功"
    })))
}

/// 更正出勤記錄
pub async fn correct_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AttendanceCorrectionRequest>,
) -> Result<Json<serde_json::Value>> {
    HrService::correct_attendance(&state.db, id, current_user.id, &payload).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已更正出勤記錄"
    })))
}

// ============================================
// Overtime Handlers
// ============================================

/// 列出加班記錄
pub async fn list_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<OvertimeQuery>,
) -> Result<Json<PaginatedResponse<OvertimeWithUser>>> {
    let mut query = params;
    
    // 如果是查「待我審核」的加班申請
    if query.pending_approval.unwrap_or(false) {
        // 只顯示狀態為 pending 的記錄（已送審待審核）
        query.status = Some("pending".to_string());
        query.user_id = None; // 查看所有人的待審核申請
    } else {
        // 「我的加班」：強制只查自己的記錄
        query.user_id = Some(current_user.id);
    }
    
    let result = HrService::list_overtime(&state.db, &query).await?;
    Ok(Json(result))
}

/// 取得加班記錄詳細
pub async fn get_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<OvertimeWithUser>> {
    let record = HrService::get_overtime(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

/// 建立加班記錄
pub async fn create_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateOvertimeRequest>,
) -> Result<(StatusCode, Json<OvertimeWithUser>)> {
    let record = HrService::create_overtime(&state.db, current_user.id, &payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// 更新加班記錄
pub async fn update_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOvertimeRequest>,
) -> Result<Json<OvertimeWithUser>> {
    let record = HrService::update_overtime(&state.db, id, &current_user, &payload).await?;
    Ok(Json(record))
}

/// 刪除加班記錄
pub async fn delete_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    HrService::delete_overtime(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 提交加班申請
pub async fn submit_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<OvertimeWithUser>> {
    let record = HrService::submit_overtime(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

/// 核准加班申請
pub async fn approve_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<OvertimeWithUser>> {
    // 僅 IACUC_STAFF (執行秘書) 可審核加班申請
    if !current_user.roles.contains(&"IACUC_STAFF".to_string()) {
        return Err(crate::error::AppError::Forbidden("僅執行秘書可審核加班申請".to_string()));
    }
    let record = HrService::approve_overtime(&state.db, id, current_user.id).await?;
    Ok(Json(record))
}

/// 駁回加班申請
pub async fn reject_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectOvertimeRequest>,
) -> Result<Json<OvertimeWithUser>> {
    // 僅 IACUC_STAFF (執行秘書) 可審核加班申請
    if !current_user.roles.contains(&"IACUC_STAFF".to_string()) {
        return Err(crate::error::AppError::Forbidden("僅執行秘書可審核加班申請".to_string()));
    }
    let record =
        HrService::reject_overtime(&state.db, id, current_user.id, &payload.reason).await?;
    Ok(Json(record))
}

// ============================================
// Leave Handlers
// ============================================

/// 列出請假記錄
pub async fn list_leaves(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<LeaveQuery>,
) -> Result<Json<PaginatedResponse<LeaveRequestWithUser>>> {
    let mut query = params;
    
    // 如果是查待審核的請假
    if query.pending_approval.unwrap_or(false) {
        // 對於有審核權限的人，查看所有待審核狀態的請假
        // 不再使用 current_approver_id 過濾，因為目前沒有設定審批鏈
        query.user_id = None;
        // 可以在這裡添加權限檢查，確保只有有審核權限的人可以看待審核
    } else {
        // "我的請假" - 永遠只看自己的請假，無論有無 view.all 權限
        // 除非 API 明確傳入 user_id 且用戶有 view.all 權限
        if query.user_id.is_none() {
            query.user_id = Some(current_user.id);
        } else if query.user_id.unwrap() != current_user.id 
            && !current_user.has_permission("hr.leave.view.all") {
            // 如果嘗試查看別人的請假但沒有權限，強制查自己
            query.user_id = Some(current_user.id);
        }
    }
    
    let result = HrService::list_leaves(&state.db, &query, &current_user).await?;
    Ok(Json(result))
}

/// 取得請假詳細
pub async fn get_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<LeaveRequest>> {
    let record = HrService::get_leave(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

/// 建立請假申請
pub async fn create_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateLeaveRequest>,
) -> Result<(StatusCode, Json<LeaveRequest>)> {
    let record = HrService::create_leave(&state.db, current_user.id, &payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// 更新請假申請
pub async fn update_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    let record = HrService::update_leave(&state.db, id, &current_user, &payload).await?;
    Ok(Json(record))
}

/// 刪除請假申請
pub async fn delete_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    HrService::delete_leave(&state.db, id, &current_user).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 提交請假申請
pub async fn submit_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<LeaveRequest>> {
    let record = HrService::submit_leave(&state.db, id, &current_user).await?;
    Ok(Json(record))
}

/// 核准請假申請
pub async fn approve_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApproveLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    // 僅 IACUC_STAFF (執行秘書) 可審核請假申請
    if !current_user.roles.contains(&"IACUC_STAFF".to_string()) {
        return Err(crate::error::AppError::Forbidden("僅執行秘書可審核請假申請".to_string()));
    }
    let record = HrService::approve_leave(
        &state.db,
        id,
        current_user.id,
        payload.comments.as_deref(),
    )
    .await?;
    Ok(Json(record))
}

/// 駁回請假申請
pub async fn reject_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    // 僅 IACUC_STAFF (執行秘書) 可審核請假申請
    if !current_user.roles.contains(&"IACUC_STAFF".to_string()) {
        return Err(crate::error::AppError::Forbidden("僅執行秘書可審核請假申請".to_string()));
    }
    let record =
        HrService::reject_leave(&state.db, id, current_user.id, &payload.reason).await?;
    Ok(Json(record))
}

/// 取消請假
pub async fn cancel_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    let record =
        HrService::cancel_leave(&state.db, id, &current_user, payload.reason.as_deref()).await?;
    Ok(Json(record))
}

// ============================================
// Balance Handlers
// ============================================

/// 取得特休餘額
pub async fn get_annual_leave_balances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<Vec<AnnualLeaveBalanceView>>> {
    let user_id = params.user_id.unwrap_or(current_user.id);
    
    // 權限檢查
    if user_id != current_user.id && !current_user.has_permission("hr.balance.view.all") {
        return Err(crate::error::AppError::Forbidden(
            "無權查看他人餘額".to_string(),
        ));
    }
    
    let balances = HrService::get_annual_leave_balances(&state.db, user_id).await?;
    Ok(Json(balances))
}

/// 取得補休餘額
pub async fn get_comp_time_balances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<Vec<CompTimeBalanceView>>> {
    let user_id = params.user_id.unwrap_or(current_user.id);
    
    if user_id != current_user.id && !current_user.has_permission("hr.balance.view.all") {
        return Err(crate::error::AppError::Forbidden(
            "無權查看他人餘額".to_string(),
        ));
    }
    
    let balances = HrService::get_comp_time_balances(&state.db, user_id).await?;
    Ok(Json(balances))
}

/// 取得餘額摘要
pub async fn get_balance_summary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<BalanceQuery>,
) -> Result<Json<BalanceSummary>> {
    let user_id = params.user_id.unwrap_or(current_user.id);
    
    if user_id != current_user.id && !current_user.has_permission("hr.balance.view.all") {
        return Err(crate::error::AppError::Forbidden(
            "無權查看他人餘額".to_string(),
        ));
    }
    
    let summary = HrService::get_balance_summary(&state.db, user_id).await?;
    Ok(Json(summary))
}

/// 建立特休額度（管理員）
pub async fn create_annual_leave_entitlement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateAnnualLeaveRequest>,
) -> Result<(StatusCode, Json<AnnualLeaveEntitlement>)> {
    let entitlement =
        HrService::create_annual_leave_entitlement(&state.db, current_user.id, &payload)
            .await?;
    Ok((StatusCode::CREATED, Json(entitlement)))
}

/// 調整餘額（管理員）
pub async fn adjust_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdjustBalanceRequest>,
) -> Result<Json<AnnualLeaveEntitlement>> {
    let entitlement =
        HrService::adjust_annual_leave(&state.db, id, current_user.id, &payload).await?;
    Ok(Json(entitlement))
}

/// 取得過期特休假報表（供會計補償用）
/// 列出所有過期但仍有餘額的特休假，供會計部門處理補償
pub async fn get_expired_leave_compensation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ExpiredLeaveReport>>> {
    // 權限檢查：僅管理員或有 hr.balance.manage 權限的人可查看
    if !current_user.has_permission("hr.balance.manage") 
        && !current_user.roles.contains(&"admin".to_string())
        && !current_user.roles.contains(&"IACUC_STAFF".to_string()) {
        return Err(crate::error::AppError::Forbidden(
            "無權查看過期特休報表".to_string(),
        ));
    }
    
    let reports = HrService::get_expired_leave_compensation_report(&state.db).await?;
    Ok(Json(reports))
}

// ============================================
// 儀表板統計 API
// ============================================

/// 工作人員出勤統計（儀表板用）
pub async fn get_attendance_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    // 權限檢查：僅管理員可查看
    if !current_user.has_permission("hr.attendance.view.all") && !current_user.roles.contains(&"admin".to_string()) {
        return Err(crate::error::AppError::Forbidden(
            "無權查看出勤統計".to_string(),
        ));
    }
    
    let start_date = params.get("start_date").cloned();
    let end_date = params.get("end_date").cloned();
    
    // 如果沒有指定日期，預設為本月
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
    
    // 查詢出勤統計
    // 注意: attendance_records 沒有 is_late 欄位，用 clock_in_time 時間判斷 (09:00 後視為遲到)
    let stats = sqlx::query_as::<_, (uuid::Uuid, String, i64, i64, i64, rust_decimal::Decimal)>(
        r#"
        SELECT 
            u.id as user_id,
            u.display_name,
            COUNT(DISTINCT CASE WHEN a.clock_in_time IS NOT NULL THEN DATE(a.clock_in_time) END) as attendance_days,
            COUNT(DISTINCT CASE WHEN a.clock_in_time IS NOT NULL AND EXTRACT(HOUR FROM a.clock_in_time AT TIME ZONE 'Asia/Taipei') >= 9 
                AND EXTRACT(MINUTE FROM a.clock_in_time AT TIME ZONE 'Asia/Taipei') > 0 THEN a.id END) as late_count,
            COALESCE((
                SELECT SUM(l.total_days)
                FROM leave_requests l
                WHERE l.user_id = u.id 
                AND l.status = 'APPROVED'
                AND l.start_date >= $1::date 
                AND l.end_date <= $2::date
            ), 0)::bigint as leave_days,
            COALESCE((
                SELECT SUM(o.hours)
                FROM overtime_records o
                WHERE o.user_id = u.id 
                AND o.status = 'approved'
                AND o.overtime_date >= $1::date 
                AND o.overtime_date <= $2::date
            ), 0) as overtime_hours
        FROM users u
        LEFT JOIN attendance_records a ON u.id = a.user_id 
            AND DATE(a.clock_in_time) >= $1::date 
            AND DATE(a.clock_in_time) <= $2::date
        WHERE u.is_active = true
        AND u.email != 'admin@ipig.local'
        AND NOT EXISTS (
            SELECT 1 FROM user_roles ur
            JOIN roles r ON ur.role_id = r.id
            WHERE ur.user_id = u.id AND (r.code = 'SYSTEM_ADMIN' OR r.code = 'admin')
        )
        GROUP BY u.id, u.display_name
        ORDER BY u.display_name
        "#
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_all(&state.db)
    .await?;
    
    let data: Vec<serde_json::Value> = stats
        .into_iter()
        .map(|(user_id, display_name, attendance_days, late_count, leave_days, overtime_hours)| {
            serde_json::json!({
                "user_id": user_id.to_string(),
                "display_name": display_name,
                "attendance_days": attendance_days,
                "late_count": late_count,
                "leave_days": leave_days,
                "overtime_hours": overtime_hours.to_string().parse::<f64>().unwrap_or(0.0)
            })
        })
        .collect();
    
    Ok(Json(serde_json::json!({ "data": data })))
}

// ============================================
// Dashboard Calendar
// ============================================

/// 取得儀表板日曆資料
pub async fn get_dashboard_calendar(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<DashboardCalendarData>> {
    let data = HrService::get_dashboard_calendar(&state.db).await?;
    Ok(Json(data))
}
