// 請假管理 Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        ApproveLeaveRequest, CancelLeaveRequest, CreateLeaveRequest, LeaveQuery, LeaveRequest,
        LeaveRequestWithUser, PaginatedResponse, RejectLeaveRequest, UpdateLeaveRequest,
    },
    services::{HrService, NotificationService},
    AppState, Result,
};

/// 列出請假記錄
pub async fn list_leaves(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<LeaveQuery>,
) -> Result<Json<PaginatedResponse<LeaveRequestWithUser>>> {
    let mut query = params;
    if query.view_all.unwrap_or(false) {
        let is_admin = current_user.roles.contains(&"admin".to_string());
        let has_view_all = current_user.has_permission("hr.leave.view_all");
        if is_admin || has_view_all {
            query.user_id = None;
        } else {
            query.user_id = Some(current_user.id);
        }
    } else if query.pending_approval.unwrap_or(false) {
        query.user_id = None;
    } else if query.user_id.is_none()
        || (query.user_id.is_some_and(|uid| uid != current_user.id)
            && !current_user.has_permission("hr.leave.view_all"))
    {
        query.user_id = Some(current_user.id);
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
    let db = state.db.clone();
    let leave_id = record.id;
    let leave_type = record.leave_type.clone();
    let start_date = record.start_date.to_string();
    let end_date = record.end_date.to_string();
    let applicant_name = current_user.email.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_leave_submitted(
                leave_id,
                &applicant_name,
                &leave_type,
                &start_date,
                &end_date,
            )
            .await
        {
            tracing::warn!("發送請假申請通知失敗: {e}");
        }
    });
    Ok(Json(record))
}

/// 核准請假申請
pub async fn approve_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApproveLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    let current: (String, Uuid) =
        sqlx::query_as("SELECT status::text, user_id FROM leave_requests WHERE id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await?;
    let (current_status, applicant_id) = current;
    let is_admin = current_user.roles.contains(&"admin".to_string());
    let is_admin_staff = current_user.roles.contains(&"ADMIN_STAFF".to_string());
    match current_status.as_str() {
        "PENDING_L1" => {
            if !is_admin_staff && !is_admin {
                let is_dept_manager = matches!(sqlx::query_as::<_, (bool,)>(
                    r#"SELECT EXISTS(SELECT 1 FROM users u JOIN departments d ON u.department_id = d.id WHERE u.id = $1 AND d.manager_id = $2)"#
                ).bind(applicant_id).bind(current_user.id).fetch_optional(&state.db).await, Ok(Some((true,))));
                if !is_dept_manager {
                    return Err(crate::error::AppError::Forbidden(
                        "此階段需要部門主管、行政或負責人審核".to_string(),
                    ));
                }
            }
        }
        "PENDING_HR" => {
            if !is_admin_staff && !is_admin {
                return Err(crate::error::AppError::Forbidden(
                    "此階段需要行政或負責人審核".to_string(),
                ));
            }
        }
        "PENDING_GM" => {
            if !is_admin {
                return Err(crate::error::AppError::Forbidden(
                    "此階段需要負責人審核".to_string(),
                ));
            }
        }
        _ => {
            return Err(crate::error::AppError::Validation(format!(
                "無法審核狀態為 {} 的請假申請",
                current_status
            )));
        }
    }
    let record =
        HrService::approve_leave(&state.db, id, current_user.id, payload.comments.as_deref())
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
    let is_admin = current_user.roles.contains(&"admin".to_string());
    let is_admin_staff = current_user.roles.contains(&"ADMIN_STAFF".to_string());
    if !is_admin_staff && !is_admin {
        let current: Option<(String, Uuid)> =
            sqlx::query_as("SELECT status::text, user_id FROM leave_requests WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.db)
                .await?;
        if let Some((status, applicant_id)) = current {
            if status == "PENDING_L1" {
                let is_dept_manager = matches!(sqlx::query_as::<_, (bool,)>(
                    r#"SELECT EXISTS(SELECT 1 FROM users u JOIN departments d ON u.department_id = d.id WHERE u.id = $1 AND d.manager_id = $2)"#
                ).bind(applicant_id).bind(current_user.id).fetch_optional(&state.db).await, Ok(Some((true,))));
                if !is_dept_manager {
                    return Err(crate::error::AppError::Forbidden(
                        "僅部門主管、行政或負責人可駁回請假申請".to_string(),
                    ));
                }
            } else {
                return Err(crate::error::AppError::Forbidden(
                    "僅行政或負責人可駁回此階段的請假申請".to_string(),
                ));
            }
        } else {
            return Err(crate::error::AppError::NotFound(
                "請假申請不存在".to_string(),
            ));
        }
    }
    let record = HrService::reject_leave(&state.db, id, current_user.id, &payload.reason).await?;
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
