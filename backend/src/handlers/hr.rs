// HR Handlers
// 包含：Attendance, Overtime, Leave, Balances

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        AdjustBalanceRequest, AnnualLeaveBalanceView, AnnualLeaveEntitlement,
        ApproveLeaveRequest, AttendanceCorrectionRequest, AttendanceQuery, AttendanceWithUser,
        BalanceQuery, BalanceSummary, CancelLeaveRequest, ClockInRequest, ClockOutRequest,
        CompTimeBalanceView, CreateAnnualLeaveRequest, CreateLeaveRequest, CreateOvertimeRequest,
        LeaveQuery, LeaveRequest, LeaveRequestWithUser, OvertimeQuery, OvertimeWithUser,
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
