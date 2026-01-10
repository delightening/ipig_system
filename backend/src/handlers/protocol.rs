use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AssignReviewerRequest, ChangeStatusRequest, CreateCommentRequest, CreateProtocolRequest,
        Protocol, ProtocolListItem, ProtocolQuery, ProtocolResponse, ProtocolStatusHistory,
        ProtocolVersion, ReviewAssignment, ReviewComment, ReviewCommentResponse,
        UpdateProtocolRequest,
    },
    require_permission,
    services::ProtocolService,
    AppError, AppState, Result,
};

/// 建立計畫
pub async fn create_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProtocolRequest>,
) -> Result<Json<Protocol>> {
    require_permission!(current_user, "aup.protocol.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let protocol = ProtocolService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(protocol))
}

/// 取得計畫列表
pub async fn list_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolQuery>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    // 檢查是否有查看所有計畫的權限
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string());
    
    let protocols = if has_view_all {
        ProtocolService::list(&state.db, &query).await?
    } else {
        // 只能看自己的計畫
        ProtocolService::get_my_protocols(&state.db, current_user.id).await?
    };
    
    Ok(Json(protocols))
}

/// 取得單一計畫
pub async fn get_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProtocolResponse>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    
    // 檢查是否有權限查看（自己的計畫或有 view_all 權限）
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string());
    if !has_view_all && protocol.protocol.pi_user_id != current_user.id {
        return Err(AppError::Forbidden("You don't have permission to view this protocol".to_string()));
    }
    
    Ok(Json(protocol))
}

/// 更新計畫
pub async fn update_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProtocolRequest>,
) -> Result<Json<Protocol>> {
    require_permission!(current_user, "aup.protocol.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let protocol = ProtocolService::update(&state.db, id, &req).await?;
    Ok(Json(protocol))
}

/// 提交計畫
pub async fn submit_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Protocol>> {
    require_permission!(current_user, "aup.protocol.submit");
    
    let protocol = ProtocolService::submit(&state.db, id, current_user.id).await?;
    Ok(Json(protocol))
}

/// 變更計畫狀態
pub async fn change_protocol_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeStatusRequest>,
) -> Result<Json<Protocol>> {
    require_permission!(current_user, "aup.protocol.change_status");
    
    let protocol = ProtocolService::change_status(&state.db, id, &req, current_user.id).await?;
    Ok(Json(protocol))
}

/// 取得計畫版本列表
pub async fn get_protocol_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolVersion>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let versions = ProtocolService::get_versions(&state.db, id).await?;
    Ok(Json(versions))
}

/// 取得計畫狀態歷程
pub async fn get_protocol_status_history(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolStatusHistory>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let history = ProtocolService::get_status_history(&state.db, id).await?;
    Ok(Json(history))
}

/// 指派審查人員
pub async fn assign_reviewer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignReviewerRequest>,
) -> Result<Json<ReviewAssignment>> {
    require_permission!(current_user, "aup.review.assign");
    
    let assignment = ProtocolService::assign_reviewer(&state.db, &req, current_user.id).await?;
    Ok(Json(assignment))
}

/// 取得審查指派列表
pub async fn list_review_assignments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolIdQuery>,
) -> Result<Json<Vec<ReviewAssignment>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol_id = query.protocol_id
        .ok_or_else(|| AppError::Validation("protocol_id is required".to_string()))?;
    
    let assignments: Vec<ReviewAssignment> = sqlx::query_as(
        "SELECT * FROM review_assignments WHERE protocol_id = $1"
    )
    .bind(protocol_id)
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(assignments))
}

#[derive(Debug, serde::Deserialize)]
pub struct ProtocolIdQuery {
    pub protocol_id: Option<Uuid>,
    pub protocol_version_id: Option<Uuid>,
}

/// 新增審查意見
pub async fn create_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<ReviewComment>> {
    require_permission!(current_user, "aup.review.comment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let comment = ProtocolService::add_comment(&state.db, &req, current_user.id).await?;
    Ok(Json(comment))
}

/// 取得審查意見列表
pub async fn list_review_comments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolIdQuery>,
) -> Result<Json<Vec<ReviewCommentResponse>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol_version_id = query.protocol_version_id
        .ok_or_else(|| AppError::Validation("protocol_version_id is required".to_string()))?;
    
    let comments = ProtocolService::get_comments(&state.db, protocol_version_id).await?;
    Ok(Json(comments))
}

/// 解決審查意見
pub async fn resolve_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReviewComment>> {
    // PI 可以解決意見
    let comment = ProtocolService::resolve_comment(&state.db, id, current_user.id).await?;
    Ok(Json(comment))
}

/// 取得我的計畫列表
pub async fn get_my_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    let protocols = ProtocolService::get_my_protocols(&state.db, current_user.id).await?;
    Ok(Json(protocols))
}
