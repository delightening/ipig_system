// 出勤管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{AttendanceCorrectionRequest, AttendanceQuery, AttendanceWithUser, ClockInRequest, ClockOutRequest, PaginatedResponse},
    services::HrService,
    AppState, Result,
};

/// 列出出勤記錄
pub async fn list_attendance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Json<PaginatedResponse<AttendanceWithUser>>> {
    let mut query = params;
    if query.user_id.is_none() && !current_user.has_permission("hr.attendance.view_all") {
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
    let record = HrService::clock_in(&state.db, current_user.id, payload.source.as_deref(), None).await?;
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
    let record = HrService::clock_out(&state.db, current_user.id, payload.source.as_deref(), None).await?;
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
