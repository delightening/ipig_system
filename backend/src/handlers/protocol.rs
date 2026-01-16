use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AssignReviewerRequest, AssignCoEditorRequest, ChangeStatusRequest, CreateCommentRequest, CreateProtocolRequest,
        Protocol, ProtocolListItem, ProtocolQuery, ProtocolResponse, ProtocolStatusHistory,
        ProtocolVersion, ReplyCommentRequest, ReviewAssignment, ReviewComment, ReviewCommentResponse,
        UpdateProtocolRequest, UserProtocol,
    },
    require_permission,
    services::ProtocolService,
    AppError, AppState, Result,
};

/// 建立專案
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

/// 列出所有專案
pub async fn list_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolQuery>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    // 檢查當前使用者是否有查看所有專案的權限
    // IACUC_STAFF 可以查看所有專案，其他角色只能查看自己的專案，詳見 role.md 和 AUP 規格說明
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string())
        || current_user.roles.contains(&"CHAIR".to_string());
    
    let protocols = if has_view_all {
        ProtocolService::list(&state.db, &query).await?
    } else {
        // 只能查看自己的專案
        ProtocolService::get_my_protocols(&state.db, current_user.id).await?
    };
    
    Ok(Json(protocols))
}

/// 取得單個專案
pub async fn get_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProtocolResponse>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    
    // 檢查當前使用者是否有查看此專案的權限
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string());
    let is_pi_or_coeditor: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE protocol_id = $1 
            AND user_id = $2 
            AND role_in_protocol IN ('PI', 'CLIENT', 'CO_EDITOR')
        )
        "#
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));
    
    if !has_view_all && protocol.protocol.pi_user_id != current_user.id && !is_pi_or_coeditor.0 {
        return Err(AppError::Forbidden("You don't have permission to view this protocol".to_string()));
    }
    
    Ok(Json(protocol))
}

/// 更新專案
/// 允許 PI、CLIENT 或 co-editor 編輯協議
pub async fn update_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProtocolRequest>,
) -> Result<Json<Protocol>> {
    // 檢查是否有編輯權限
    let has_edit_permission = current_user.permissions.contains(&"aup.protocol.edit".to_string());
    
    // 檢查是否為協議的 PI、CLIENT 或 co-editor
    let is_authorized: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE protocol_id = $1 
            AND user_id = $2 
            AND role_in_protocol IN ('PI', 'CLIENT', 'CO_EDITOR')
        )
        "#
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));
    
    if !has_edit_permission && !is_authorized.0 {
        return Err(AppError::Forbidden("You don't have permission to edit this protocol".to_string()));
    }
    
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let protocol = ProtocolService::update(&state.db, id, &req).await?;
    Ok(Json(protocol))
}

/// 提交專案
/// 允許 PI 或 co-editor 提交協議
pub async fn submit_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Protocol>> {
    // 檢查是否有提交權限
    let has_submit_permission = current_user.permissions.contains(&"aup.protocol.submit".to_string());
    
    // 檢查是否為協議的 PI 或 co-editor
    let is_authorized: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE protocol_id = $1 
            AND user_id = $2 
            AND role_in_protocol IN ('PI', 'CO_EDITOR')
        )
        "#,
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));
    
    if !has_submit_permission && !is_authorized.0 {
        return Err(AppError::Forbidden("You don't have permission to submit this protocol".to_string()));
    }
    
    let protocol = ProtocolService::submit(&state.db, id, current_user.id).await?;
    Ok(Json(protocol))
}

/// 變更專案狀態
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

/// 列出專案所有版本
pub async fn get_protocol_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolVersion>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let versions = ProtocolService::get_versions(&state.db, id).await?;
    Ok(Json(versions))
}

/// 列出專案狀態變更歷史
pub async fn get_protocol_status_history(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolStatusHistory>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let history = ProtocolService::get_status_history(&state.db, id).await?;
    Ok(Json(history))
}

/// 指派審查委員
pub async fn assign_reviewer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignReviewerRequest>,
) -> Result<Json<ReviewAssignment>> {
    require_permission!(current_user, "aup.review.assign");
    
    let assignment = ProtocolService::assign_reviewer(&state.db, &req, current_user.id).await?;
    Ok(Json(assignment))
}

/// 指派 co-editor（試驗工作人員）
/// IACUC_STAFF 可以指派 EXPERIMENT_STAFF 為協議的 co-editor
pub async fn assign_co_editor(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignCoEditorRequest>,
) -> Result<Json<UserProtocol>> {
    require_permission!(current_user, "aup.review.assign");
    
    let assignment = ProtocolService::assign_co_editor(&state.db, &req, current_user.id).await?;
    Ok(Json(assignment))
}

/// 列出審查委員指派清單
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

/// 列出審查意見清單
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

/// 標記審查意見為已解決
pub async fn resolve_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReviewComment>> {
    // PI 或 co-editor 可以標記意見為已解決
    let comment = ProtocolService::resolve_comment(&state.db, id, current_user.id).await?;
    Ok(Json(comment))
}

/// 回覆審查意見
/// 允許 PI 或 co-editor 回覆審查委員的意見
pub async fn reply_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ReplyCommentRequest>,
) -> Result<Json<ReviewComment>> {
    require_permission!(current_user, "aup.review.reply");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let comment = ProtocolService::reply_comment(&state.db, &req, current_user.id).await?;
    Ok(Json(comment))
}

/// 列出我的專案清單
pub async fn get_my_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    let protocols = ProtocolService::get_my_protocols(&state.db, current_user.id).await?;
    Ok(Json(protocols))
}
