// 加班管理 Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{CreateOvertimeRequest, OvertimeQuery, OvertimeWithUser, PaginatedResponse, RejectOvertimeRequest, UpdateOvertimeRequest},
    services::{HrService, NotificationService, OvertimeLimitCheck, WeekdayOvertimeTiers, WorkHoursValidation},
    AppState, Result,
};

/// 列出加班記錄
#[utoipa::path(get, path = "/api/v1/hr/overtime", responses((status = 200)), tag = "HR 加班", security(("bearer" = [])))]
pub async fn list_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<OvertimeQuery>,
) -> Result<Json<PaginatedResponse<OvertimeWithUser>>> {
    let mut query = params;
    if query.view_all.unwrap_or(false) {
        let is_admin = current_user.is_admin();
        let has_view_all = current_user.has_permission("hr.overtime.view_all");
        if !is_admin && !has_view_all {
            query.user_id = Some(current_user.id);
        }
    } else if query.pending_approval.unwrap_or(false) {
        let is_admin = current_user.is_admin();
        let is_admin_staff = current_user.roles.contains(&crate::constants::ROLE_ADMIN_STAFF.to_string());
        if is_admin {
            query.status = Some("pending_admin_staff,pending_admin".to_string());
        } else if is_admin_staff {
            query.status = Some("pending_admin_staff".to_string());
        } else {
            query.status = Some("__none__".to_string());
        }
        query.user_id = None;
    } else {
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
#[utoipa::path(post, path = "/api/v1/hr/overtime", request_body = CreateOvertimeRequest, responses((status = 201)), tag = "HR 加班", security(("bearer" = [])))]
pub async fn create_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateOvertimeRequest>,
) -> Result<(StatusCode, Json<OvertimeWithUser>)> {
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::create_overtime(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// 更新加班記錄
pub async fn update_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOvertimeRequest>,
) -> Result<Json<OvertimeWithUser>> {
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::update_overtime(&state.db, &actor, id, &payload).await?;
    Ok(Json(record))
}

/// 刪除加班記錄
pub async fn delete_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let actor = ActorContext::User(current_user.clone());
    HrService::delete_overtime(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 提交加班申請
pub async fn submit_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<OvertimeWithUser>> {
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::submit_overtime(&state.db, &actor, id).await?;
    let db = state.db.clone();
    let applicant_name = record.user_name.clone();
    let overtime_date = record.overtime_date.to_string();
    let hours: f64 = record.hours.try_into().unwrap_or(0.0);
    let overtime_id = record.id;
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc.notify_overtime_submitted(overtime_id, &applicant_name, &overtime_date, hours).await {
            tracing::warn!("發送加班申請通知失敗: {e}");
        }
    });
    Ok(Json(record))
}

/// 核准加班申請
pub async fn approve_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<OvertimeWithUser>> {
    let is_admin = current_user.is_admin();
    let is_admin_staff = current_user.roles.contains(&crate::constants::ROLE_ADMIN_STAFF.to_string());
    if !is_admin && !is_admin_staff {
        return Err(crate::error::AppError::Forbidden("僅行政或負責人可審核加班申請".to_string()));
    }
    let current: (String,) = sqlx::query_as("SELECT status FROM overtime_records WHERE id = $1")
        .bind(id).fetch_one(&state.db).await?;
    let approval_level = match current.0.as_str() {
        "pending_admin_staff" => {
            if is_admin_staff || is_admin { "admin_staff" }
            else { return Err(crate::error::AppError::Forbidden("此階段需要行政審核".to_string())); }
        },
        "pending_admin" => {
            if is_admin { "admin" }
            else { return Err(crate::error::AppError::Forbidden("此階段需要負責人審核".to_string())); }
        },
        _ => return Err(crate::error::AppError::Validation(format!("無法審核狀態為 {} 的加班申請", current.0))),
    };
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::approve_overtime(&state.db, &actor, id, approval_level).await?;
    Ok(Json(record))
}

/// 勞基法 §32：檢查加班上限
#[derive(serde::Deserialize)]
pub struct OvertimeLimitQuery {
    pub user_id: Uuid,
    pub overtime_date: chrono::NaiveDate,
    pub hours: f64,
}

pub async fn check_overtime_limit(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<OvertimeLimitQuery>,
) -> Result<Json<OvertimeLimitCheck>> {
    let result = HrService::check_monthly_overtime_limit(
        &state.db, params.user_id, params.overtime_date, params.hours,
    ).await?;
    Ok(Json(result))
}

/// 勞基法 §30：驗證日/週工時
#[derive(serde::Deserialize)]
pub struct WorkHoursQuery {
    pub user_id: Uuid,
    pub work_date: chrono::NaiveDate,
}

pub async fn validate_work_hours(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<WorkHoursQuery>,
) -> Result<Json<WorkHoursValidation>> {
    let result = HrService::validate_work_hours(
        &state.db, params.user_id, params.work_date,
    ).await?;
    Ok(Json(result))
}

/// 勞基法 §24：平日加班分段計算
#[derive(serde::Deserialize)]
pub struct WeekdayTiersQuery {
    pub hours: f64,
}

pub async fn calculate_weekday_tiers(
    Query(params): Query<WeekdayTiersQuery>,
) -> Result<Json<WeekdayOvertimeTiers>> {
    let result = HrService::calculate_weekday_overtime_tiers(params.hours);
    Ok(Json(result))
}

/// 駁回加班申請
pub async fn reject_overtime(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectOvertimeRequest>,
) -> Result<Json<OvertimeWithUser>> {
    let can_reject = current_user.is_admin()
        || current_user.roles.contains(&crate::constants::ROLE_ADMIN_STAFF.to_string());
    if !can_reject {
        return Err(crate::error::AppError::Forbidden("僅行政或負責人可駁回加班申請".to_string()));
    }
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::reject_overtime(&state.db, &actor, id, &payload.reason).await?;
    Ok(Json(record))
}
