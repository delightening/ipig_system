// 專案 CRUD Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AssignCoEditorRequest, ChangeStatusRequest, CoEditorAssignmentResponse,
        CreateProtocolRequest, Protocol, ProtocolActivityResponse, ProtocolListItem,
        ProtocolQuery, ProtocolResponse, ProtocolVersion, UpdateProtocolRequest, UserProtocol,
        SaveVetReviewFormRequest,
    },
    require_permission,
    services::{NotificationService, ProtocolService},
    AppError, AppState, Result,
};

/// 建立專案
#[utoipa::path(post, path = "/api/protocols", request_body = CreateProtocolRequest, responses((status = 201, description = "建立成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn create_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProtocolRequest>,
) -> Result<Json<Protocol>> {
    let can_create = current_user.has_permission("aup.protocol.create")
        || current_user.roles.contains(&"PI".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    if !can_create {
        return Err(AppError::Forbidden("Permission denied: requires aup.protocol.create or PI role".to_string()));
    }
    req.validate()?;
    let protocol = ProtocolService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(protocol))
}

/// 列出所有專案
#[utoipa::path(get, path = "/api/protocols", responses((status = 200, description = "專案清單", body = Vec<ProtocolListItem>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn list_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolQuery>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    let has_view_all = current_user.has_permission("aup.protocol.view_all")
        || current_user.roles.iter().any(|r| ["IACUC_STAFF", "VET", "REVIEWER", "IACUC_CHAIR"].contains(&r.as_str()));
    let is_reviewer_only = current_user.roles.iter()
        .all(|r| ["REVIEWER", "VET"].contains(&r.as_str()))
        && (current_user.roles.contains(&"REVIEWER".to_string())
            || current_user.roles.contains(&"VET".to_string()));
    let mut protocols = if has_view_all {
        ProtocolService::list(&state.db, &query).await?
    } else {
        ProtocolService::get_my_protocols(&state.db, current_user.id).await?
    };
    if is_reviewer_only {
        protocols.retain(|p| {
            matches!(p.status,
                crate::models::ProtocolStatus::Submitted |
                crate::models::ProtocolStatus::PreReview |
                crate::models::ProtocolStatus::VetReview |
                crate::models::ProtocolStatus::UnderReview |
                crate::models::ProtocolStatus::Approved |
                crate::models::ProtocolStatus::ApprovedWithConditions |
                crate::models::ProtocolStatus::Closed
            )
        });
    }
    Ok(Json(protocols))
}

/// 取得單個專案
#[utoipa::path(get, path = "/api/protocols/{id}", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "專案詳細", body = ProtocolResponse)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProtocolResponse>> {
    require_permission!(current_user, "aup.protocol.view_own");
    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    let is_pi_or_coeditor: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol IN ('PI', 'CLIENT', 'CO_EDITOR'))"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    let is_assigned_reviewer: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2)"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    let is_assigned_vet: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2)"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    if !has_view_all && protocol.protocol.pi_user_id != current_user.id && !is_pi_or_coeditor.0 && !is_assigned_reviewer.0 && !is_assigned_vet.0 {
        return Err(AppError::Forbidden("You don't have permission to view this protocol".to_string()));
    }
    Ok(Json(protocol))
}

/// 更新專案
#[utoipa::path(put, path = "/api/protocols/{id}", params(("id" = Uuid, Path, description = "專案 ID")), request_body = UpdateProtocolRequest, responses((status = 200, description = "更新成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn update_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProtocolRequest>,
) -> Result<Json<Protocol>> {
    let has_edit_permission = current_user.permissions.contains(&"aup.protocol.edit".to_string());
    let is_authorized: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol IN ('PI', 'CLIENT', 'CO_EDITOR'))"#
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    if !has_edit_permission && !is_authorized.0 {
        return Err(AppError::Forbidden("You don't have permission to edit this protocol".to_string()));
    }
    req.validate()?;
    let protocol = ProtocolService::update(&state.db, id, &req, current_user.id).await?;
    Ok(Json(protocol))
}

/// 提交專案
#[utoipa::path(post, path = "/api/protocols/{id}/submit", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "提交成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn submit_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Protocol>> {
    let has_submit_permission = current_user.permissions.contains(&"aup.protocol.submit".to_string());
    let is_authorized: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol IN ('PI', 'CO_EDITOR'))"#,
    ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
    if !has_submit_permission && !is_authorized.0 {
        return Err(AppError::Forbidden("You don't have permission to submit this protocol".to_string()));
    }
    let protocol = ProtocolService::submit(&state.db, id, current_user.id).await?;
    Ok(Json(protocol))
}

/// 變更專案狀態
#[utoipa::path(post, path = "/api/protocols/{id}/status", params(("id" = Uuid, Path, description = "專案 ID")), request_body = ChangeStatusRequest, responses((status = 200, description = "狀態變更成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn change_protocol_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeStatusRequest>,
) -> Result<Json<Protocol>> {
    tracing::info!("[ChangeStatus] User: {}, Target Status: {:?}", current_user.id, req.to_status);
    if matches!(req.to_status, crate::models::ProtocolStatus::Deleted) {
        tracing::info!("[ChangeStatus] Entering Delete Permission Check for status DELETED");
        require_permission!(current_user, "aup.protocol.delete");
    } else {
        tracing::info!("[ChangeStatus] Entering Normal Status Change Check");
        require_permission!(current_user, "aup.protocol.change_status");
    }
    let protocol = ProtocolService::change_status(&state.db, id, &req, current_user.id).await?;
    let db = state.db.clone();
    let protocol_id = protocol.id;
    let protocol_no = protocol.protocol_no.clone();
    let protocol_title = protocol.title.clone();
    let new_status = protocol.status.as_str().to_lowercase();
    let operator_id = current_user.id;
    let reason = req.remark.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc.notify_protocol_review_progress(
            protocol_id, &protocol_no, &protocol_title, &new_status, operator_id, reason.as_deref(), Some(&config),
        ).await {
            tracing::warn!("發送計畫審查進度通知失敗: {e}");
        }
    });
    Ok(Json(protocol))
}

/// 取得專案版本
#[utoipa::path(get, path = "/api/protocols/{id}/versions", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "版本清單", body = Vec<ProtocolVersion>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolVersion>>> {
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    if !has_view_all {
        let is_authorized: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM protocols p WHERE p.id = $1 AND p.pi_user_id = $2
                UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
                UNION SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
                UNION SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
            )"#
        ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
        if !is_authorized.0 { require_permission!(current_user, "aup.protocol.view_own"); }
    }
    let versions = ProtocolService::get_versions(&state.db, id).await?;
    Ok(Json(versions))
}

/// 取得專案活動歷程
#[utoipa::path(get, path = "/api/protocols/{id}/activities", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "活動歷程", body = Vec<ProtocolActivityResponse>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol_activities(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolActivityResponse>>> {
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    if !has_view_all {
        let is_authorized: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM protocols p WHERE p.id = $1 AND p.pi_user_id = $2
                UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
                UNION SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
                UNION SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
            )"#
        ).bind(id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
        if !is_authorized.0 { require_permission!(current_user, "aup.protocol.view_own"); }
    }
    let activities = ProtocolService::get_activities(&state.db, id).await?;
    Ok(Json(activities))
}

/// 指派 co-editor
#[utoipa::path(post, path = "/api/protocols/{id}/co-editors", request_body = AssignCoEditorRequest, responses((status = 200, description = "指派成功", body = UserProtocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn assign_co_editor(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignCoEditorRequest>,
) -> Result<Json<UserProtocol>> {
    require_permission!(current_user, "aup.review.assign");
    let assignment = ProtocolService::assign_co_editor(&state.db, &req, current_user.id).await?;
    Ok(Json(assignment))
}

/// 列出 co-editor 列表
#[utoipa::path(get, path = "/api/protocols/{id}/co-editors", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "Co-Editor 清單", body = Vec<CoEditorAssignmentResponse>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn list_co_editors(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CoEditorAssignmentResponse>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    let co_editors = ProtocolService::list_co_editors(&state.db, id).await?;
    Ok(Json(co_editors))
}

/// 移除 co-editor
#[utoipa::path(delete, path = "/api/protocols/{id}/co-editors/{user_id}", params(("id" = Uuid, Path, description = "專案 ID"), ("user_id" = Uuid, Path, description = "使用者 ID")), responses((status = 200, description = "移除成功")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn remove_co_editor(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((protocol_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<()>> {
    require_permission!(current_user, "aup.review.assign");
    ProtocolService::remove_co_editor(&state.db, protocol_id, user_id, current_user.id).await?;
    Ok(Json(()))
}

/// 列出我的專案清單
#[utoipa::path(get, path = "/api/my-projects", responses((status = 200, description = "我的專案清單", body = Vec<ProtocolListItem>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_my_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    let protocols = ProtocolService::get_my_protocols(&state.db, current_user.id).await?;
    Ok(Json(protocols))
}

/// 取得專案的動物統計（儀表板用）
#[utoipa::path(get, path = "/api/protocols/{id}/animal-stats", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "動物統計")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol_animal_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "aup.protocol.view_own");
    let protocol: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT iacuc_no FROM protocols WHERE id = $1"
    ).bind(id).fetch_optional(&state.db).await?;
    let iacuc_no = match protocol {
        Some((Some(no),)) => no,
        _ => return Ok(Json(serde_json::json!({
            "approved_count": 0, "in_use_count": 0, "completed_count": 0, "remaining_count": 0
        }))),
    };
    let stats: (i64, i64, i64) = sqlx::query_as(
        r#"SELECT
            COUNT(*) FILTER (WHERE status = 'in_experiment') as in_use_count,
            COUNT(*) FILTER (WHERE status IN ('completed', 'euthanized', 'sudden_death')) as completed_count,
            COUNT(*) as total_count
        FROM animals WHERE iacuc_no = $1 AND deleted_at IS NULL"#
    ).bind(&iacuc_no).fetch_one(&state.db).await.unwrap_or((0, 0, 0));
    let approved_count: (Option<i64>,) = sqlx::query_as(
        r#"SELECT (working_content->>'animal_count')::bigint as approved_count FROM protocols WHERE id = $1"#
    ).bind(id).fetch_one(&state.db).await.unwrap_or((None,));
    let approved = approved_count.0.unwrap_or(stats.2);
    let remaining = approved - stats.2;
    Ok(Json(serde_json::json!({
        "approved_count": approved, "in_use_count": stats.0, "completed_count": stats.1, "remaining_count": remaining.max(0)
    })))
}

/// 儲存獸醫審查表
#[utoipa::path(post, path = "/api/reviews/vet-form", request_body = SaveVetReviewFormRequest, responses((status = 200, description = "儲存成功")), tag = "審查管理", security(("bearer" = [])))]
pub async fn save_vet_review_form(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SaveVetReviewFormRequest>,
) -> Result<Json<()>> {
    let is_vet = current_user.roles.contains(&"VET".to_string()) || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    if !is_vet {
        let is_assigned: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2)"
        ).bind(req.protocol_id).bind(current_user.id).fetch_one(&state.db).await.unwrap_or((false,));
        if !is_assigned.0 {
            return Err(AppError::Forbidden("Permission denied: You are not assigned as a vet for this protocol".to_string()));
        }
    }
    ProtocolService::save_vet_review_form(&state.db, req.protocol_id, current_user.id, &req.review_form).await?;
    if let Err(e) = ProtocolService::record_activity(
        &state.db, req.protocol_id,
        crate::models::ProtocolActivityType::StatusChanged,
        current_user.id, None, None,
        Some(("VET_REVIEW_FORM", req.protocol_id, "獸醫審查表")),
        Some("填寫獸醫核選表".to_string()), Some(req.review_form.clone()),
    ).await {
        tracing::warn!("記錄活動失敗: {e}");
    }
    Ok(Json(()))
}
