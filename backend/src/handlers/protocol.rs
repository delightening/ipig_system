use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AssignReviewerRequest, AssignCoEditorRequest, ChangeStatusRequest, CreateCommentRequest, CreateProtocolRequest,
        CoEditorAssignmentResponse,
        Protocol, ProtocolListItem, ProtocolQuery, ProtocolResponse, ProtocolActivityResponse,
        ProtocolVersion, ReplyCommentRequest, ReviewAssignment, ReviewAssignmentResponse, ReviewComment, ReviewCommentResponse,
        UpdateProtocolRequest, UserProtocol, SaveDraftRequest, SubmitReplyRequest,
    },
    require_permission,
    services::{ProtocolService, PdfService},
    AppError, AppState, Result,
};


/// 建立專案
pub async fn create_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProtocolRequest>,
) -> Result<Json<Protocol>> {
    // 檢查是否有建立權限 (PI 角色或 aup.protocol.create 權限)
    let can_create = current_user.has_permission("aup.protocol.create") 
        || current_user.roles.contains(&"PI".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    
    if !can_create {
        return Err(AppError::Forbidden("Permission denied: requires aup.protocol.create or PI role".to_string()));
    }
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
    let has_view_all = current_user.has_permission("aup.protocol.view_all")
        || current_user.roles.iter().any(|r| ["IACUC_STAFF", "VET", "REVIEWER", "IACUC_CHAIR"].contains(&r.as_str()));
    
    // 檢查是否為純審查委員角色（只有 REVIEWER 或 VET，沒有其他管理角色）
    let is_reviewer_only = current_user.roles.iter()
        .all(|r| ["REVIEWER", "VET"].contains(&r.as_str()))
        && (current_user.roles.contains(&"REVIEWER".to_string()) 
            || current_user.roles.contains(&"VET".to_string()));
    
    let mut protocols = if has_view_all {
        // 有查看所有計畫權限的角色（包含審查委員）可以看到所有計畫
        ProtocolService::list(&state.db, &query).await?
    } else {
        // 只能查看自己的專案
        ProtocolService::get_my_protocols(&state.db, current_user.id).await?
    };
    
    // 審查委員特殊過濾：只能看審查中和已核准的計畫，不能看草稿
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
pub async fn get_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProtocolResponse>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    
    // 檢查當前使用者是否有查看此專案的權限
    // IACUC_CHAIR、IACUC_STAFF、SYSTEM_ADMIN、VET、REVIEWER 可以查看所有計畫書
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    
    // 檢查是否為 PI、CLIENT 或 CO_EDITOR
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
    
    // 檢查是否為已指派的審查委員
    let is_assigned_reviewer: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM review_assignments 
            WHERE protocol_id = $1 
            AND reviewer_id = $2
        )
        "#
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));

    // 檢查是否為已指派的獸醫審查員
    let is_assigned_vet: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM vet_review_assignments 
            WHERE protocol_id = $1 
            AND vet_id = $2
        )
        "#
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));
    
    if !has_view_all && protocol.protocol.pi_user_id != current_user.id && !is_pi_or_coeditor.0 && !is_assigned_reviewer.0 && !is_assigned_vet.0 {
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
    
    let protocol = ProtocolService::update(&state.db, id, &req, current_user.id).await?;
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

pub async fn change_protocol_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeStatusRequest>,
) -> Result<Json<Protocol>> {
    // 增加除錯日誌，追蹤權限判定過程
    tracing::info!("[ChangeStatus] User: {}, Target Status: {:?}", 
        current_user.id, req.to_status);

    // 檢查目標狀態
    if matches!(req.to_status, crate::models::ProtocolStatus::Deleted) {
        // 如果是要刪除，則檢查用戶是否有直接刪除權限
        tracing::info!("[ChangeStatus] Entering Delete Permission Check for status DELETED");
        require_permission!(current_user, "aup.protocol.delete");
    } else {
        // 普通狀態變更檢查
        tracing::info!("[ChangeStatus] Entering Normal Status Change Check");
        require_permission!(current_user, "aup.protocol.change_status");
    }
    
    let protocol = ProtocolService::change_status(&state.db, id, &req, current_user.id).await?;
    Ok(Json(protocol))
}

pub async fn get_protocol_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolVersion>>> {
    // 檢查是否有查看權限（IACUC 管理人員或相關審查委員）
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());

    if !has_view_all {
        // 檢查是否為計畫相關人員（PI, Co-editor, 或被指派的審查者）
        let is_authorized: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM protocols p WHERE p.id = $1 AND p.pi_user_id = $2
                UNION
                SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
                UNION
                SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
                UNION
                SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
            )
            "#
        )
        .bind(id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((false,));

        if !is_authorized.0 {
            require_permission!(current_user, "aup.protocol.view_own");
        }
    }
    
    let versions = ProtocolService::get_versions(&state.db, id).await?;
    Ok(Json(versions))
}

pub async fn get_protocol_activities(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolActivityResponse>>> {
    // 比照版本列表放寬權限
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());

    if !has_view_all {
        let is_authorized: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM protocols p WHERE p.id = $1 AND p.pi_user_id = $2
                UNION
                SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
                UNION
                SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
                UNION
                SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
            )
            "#
        )
        .bind(id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((false,));

        if !is_authorized.0 {
            require_permission!(current_user, "aup.protocol.view_own");
        }
    }
    
    let activities = ProtocolService::get_activities(&state.db, id).await?;
    Ok(Json(activities))
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

/// 列出 co-editor 列表
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
pub async fn remove_co_editor(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((protocol_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<()>> {
    require_permission!(current_user, "aup.review.assign");
    
    ProtocolService::remove_co_editor(&state.db, protocol_id, user_id).await?;
    Ok(Json(()))
}

/// 列出審查委員指派清單
pub async fn list_review_assignments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolIdQuery>,
) -> Result<Json<Vec<ReviewAssignmentResponse>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol_id = query.protocol_id
        .ok_or_else(|| AppError::Validation("protocol_id is required".to_string()))?;

    // 檢查當前使用者是否有查看所有專案的權限 (管理員、預審員、主席等)
    let has_view_all = current_user.has_permission("aup.protocol.view_all")
        || current_user.roles.iter().any(|r| ["IACUC_CHAIR", "IACUC_STAFF", "SYSTEM_ADMIN", "admin"].contains(&r.as_str()));

    if !has_view_all {
        // 檢查是否為已指派的審查委員
        let is_assigned_reviewer: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM review_assignments 
                WHERE protocol_id = $1 
                AND reviewer_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((false,));

        // 檢查是否為已指派的獸醫審查員
        let is_assigned_vet: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM vet_review_assignments 
                WHERE protocol_id = $1 
                AND vet_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((false,));

        if !is_assigned_reviewer.0 && !is_assigned_vet.0 {
            return Err(AppError::Forbidden("You don't have permission to view reviewer assignments for this protocol".to_string()));
        }
    }
    
    // 獲取一般審查指派
    let assignments: Vec<ReviewAssignmentResponse> = sqlx::query_as(
        r#"
        SELECT 
            ra.id,
            ra.protocol_id,
            ra.reviewer_id,
            COALESCE(u_rev.display_name, u_rev.email) as reviewer_name,
            u_rev.email as reviewer_email,
            ra.assigned_by,
            COALESCE(u_as.display_name, u_as.email) as assigned_by_name,
            ra.assigned_at,
            ra.completed_at,
            ra.is_primary_reviewer,
            ra.review_stage
        FROM review_assignments ra
        JOIN users u_rev ON ra.reviewer_id = u_rev.id
        JOIN users u_as ON ra.assigned_by = u_as.id
        WHERE ra.protocol_id = $1
        "#
    )
    .bind(protocol_id)
    .fetch_all(&state.db)
    .await?;

    // 獲取獸醫審查指派並轉換為 ReviewAssignmentResponse
    let vet_assignments: Vec<ReviewAssignmentResponse> = sqlx::query_as(
        r#"
        SELECT 
            vra.id,
            vra.protocol_id,
            vra.vet_id as reviewer_id,
            COALESCE(u_vet.display_name, u_vet.email) as reviewer_name,
            u_vet.email as reviewer_email,
            COALESCE(vra.assigned_by, vra.id) as assigned_by,
            COALESCE(u_as.display_name, u_as.email, 'System') as assigned_by_name,
            vra.assigned_at,
            vra.completed_at,
            true as is_primary_reviewer,
            'VET_REVIEW'::text as review_stage
        FROM vet_review_assignments vra
        JOIN users u_vet ON vra.vet_id = u_vet.id
        LEFT JOIN users u_as ON vra.assigned_by = u_as.id
        WHERE vra.protocol_id = $1
        "#
    )
    .bind(protocol_id)
    .fetch_all(&state.db)
    .await?;
    
    // 調試日誌：記錄查詢結果
    let review_count = assignments.len();
    let vet_count = vet_assignments.len();
    
    let mut all_assignments = assignments;
    all_assignments.extend(vet_assignments);
    
    tracing::info!(
        "[list_review_assignments] protocol_id: {}, found {} review assignments and {} vet assignments, total: {}",
        protocol_id,
        review_count,
        vet_count,
        all_assignments.len()
    );
    
    Ok(Json(all_assignments))
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 檢查是否有查看權限（IACUC 管理人員或相關審查委員）
    let has_global_perm = current_user.permissions.contains(&"aup.review.comment".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());

    if !has_global_perm {
        // 查出計畫 ID
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT protocol_id FROM protocol_versions WHERE id = $1"
        )
        .bind(req.protocol_version_id)
        .fetch_one(&state.db)
        .await?;

        // 檢查是否為被指派的審查委員或獸醫
        let is_authorized: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
                UNION
                SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((false,));

        if !is_authorized.0 {
            require_permission!(current_user, "aup.review.comment");
        }
    }
    
    let comment = ProtocolService::add_comment(&state.db, &req, current_user.id).await?;
    Ok(Json(comment))
}

/// 列出審查意見清單
pub async fn list_review_comments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolIdQuery>,
) -> Result<Json<Vec<ReviewCommentResponse>>> {
    let protocol_id = if let Some(pid) = query.protocol_id {
        pid
    } else if let Some(pvid) = query.protocol_version_id {
        sqlx::query_scalar("SELECT protocol_id FROM protocol_versions WHERE id = $1")
            .bind(pvid)
            .fetch_one(&state.db)
            .await?
    } else {
        return Err(AppError::Validation("protocol_id or protocol_version_id is required".to_string()));
    };

    // 比照版本列表放寬讀取權限
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"REVIEWER".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());

    if !has_view_all {
        let is_authorized: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM protocols p WHERE p.id = $1 AND p.pi_user_id = $2
                UNION
                SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
                UNION
                SELECT 1 FROM review_assignments WHERE protocol_id = $1 AND reviewer_id = $2
                UNION
                SELECT 1 FROM vet_review_assignments WHERE protocol_id = $1 AND vet_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((false,));

        if !is_authorized.0 {
            require_permission!(current_user, "aup.protocol.view_own");
        }
    }
    
    let comments = ProtocolService::get_comments(&state.db, protocol_id).await?;
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 檢查權限：要麼有全域回復權限，要麼是該計畫的 PI/Co-editor
    if !current_user.has_permission("aup.review.reply") {
        // 查出計畫 ID
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"
        )
        .bind(req.parent_comment_id)
        .fetch_one(&state.db)
        .await?;

        // 檢查是否為該計畫的 PI 或 Co-editor
        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
                UNION
                SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await?;

        if !is_owner {
            return Err(AppError::Forbidden("Permission denied: requires aup.review.reply or being the protocol owner/co-editor".to_string()));
        }
    }
    
    let comment = ProtocolService::reply_comment(&state.db, &req, current_user.id).await?;
    Ok(Json(comment))
}

/// 儲存草稿回覆
/// Coeditor 或 PI 可以先儲存草稿，稍後由 PI 正式送出
pub async fn save_reply_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SaveDraftRequest>,
) -> Result<Json<ReviewComment>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 同步 reply_review_comment 的權限邏輯
    if !current_user.has_permission("aup.review.reply") {
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"
        )
        .bind(req.comment_id)
        .fetch_one(&state.db)
        .await?;

        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM protocols WHERE id = $1 AND pi_id = $2
                UNION
                SELECT 1 FROM protocol_co_editors WHERE protocol_id = $1 AND user_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await?;

        if !is_owner {
            return Err(AppError::Forbidden("Permission denied: requires aup.review.reply or being the protocol owner/co-editor".to_string()));
        }
    }
    
    let comment = ProtocolService::save_reply_draft(
        &state.db, 
        req.comment_id, 
        &req.draft_content, 
        current_user.id
    ).await?;
    Ok(Json(comment))
}

/// 取得草稿回覆
pub async fn get_reply_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<Option<String>>> {
    // 同步權限邏輯
    if !current_user.has_permission("aup.review.reply") {
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"
        )
        .bind(comment_id)
        .fetch_one(&state.db)
        .await?;

        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM protocols WHERE id = $1 AND pi_id = $2
                UNION
                SELECT 1 FROM protocol_co_editors WHERE protocol_id = $1 AND user_id = $2
            )
            "#
        )
        .bind(protocol_id)
        .bind(current_user.id)
        .fetch_one(&state.db)
        .await?;

        if !is_owner {
            return Err(AppError::Forbidden("Permission denied: requires aup.review.reply or being the protocol owner/co-editor".to_owned()));
        }
    }
    
    let draft = ProtocolService::get_reply_draft(&state.db, comment_id).await?;
    Ok(Json(draft))
}

/// 正式送出草稿回覆（只有 PI 可以執行）
pub async fn submit_reply_from_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SubmitReplyRequest>,
) -> Result<Json<ReviewComment>> {
    // 取得評論對應的計畫
    let comment_info: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT pv.protocol_id 
        FROM review_comments rc 
        JOIN protocol_versions pv ON rc.protocol_version_id = pv.id 
        WHERE rc.id = $1
        "#
    )
    .bind(req.comment_id)
    .fetch_optional(&state.db)
    .await?;

    let (protocol_id,) = comment_info
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

    // 檢查當前用戶是否為該計畫的 PI
    let is_pi: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE protocol_id = $1 
            AND user_id = $2 
            AND role_in_protocol = 'PI'
        )
        "#
    )
    .bind(protocol_id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));

    if !is_pi.0 {
        return Err(AppError::Forbidden("Only PI can submit reply from draft".to_string()));
    }

    let comment = ProtocolService::submit_reply_from_draft(&state.db, req.comment_id, current_user.id).await?;
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

/// 取得專案的動物統計（儀表板用）
pub async fn get_protocol_animal_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    // 取得該專案的 IACUC No
    let protocol: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT iacuc_no FROM protocols WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    
    let iacuc_no = match protocol {
        Some((Some(no),)) => no,
        _ => return Ok(Json(serde_json::json!({
            "approved_count": 0,
            "in_use_count": 0,
            "completed_count": 0,
            "remaining_count": 0
        }))),
    };
    
    // 統計動物數量
    let stats: (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE status IN ('assigned', 'in_experiment')) as in_use_count,
            COUNT(*) FILTER (WHERE status = 'completed') as completed_count,
            COUNT(*) as total_count
        FROM pigs
        WHERE iacuc_no = $1
        "#
    )
    .bind(&iacuc_no)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0, 0, 0));
    
    // 從 protocol 的 working_content 取得核准數量（如果有的話）
    let approved_count: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT 
            (working_content->>'animal_count')::bigint as approved_count
        FROM protocols
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((None,));
    
    let approved = approved_count.0.unwrap_or(stats.2);
    let in_use = stats.0;
    let completed = stats.1;
    let remaining = approved - stats.2;
    
    Ok(Json(serde_json::json!({
        "approved_count": approved,
        "in_use_count": in_use,
        "completed_count": completed,
        "remaining_count": remaining.max(0)
    })))
}

/// 匯出計畫書 PDF
pub async fn export_protocol_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    require_permission!(current_user, "aup.protocol.view_own");
    
    let protocol = ProtocolService::get_by_id(&state.db, id).await?;
    
    // 檢查當前使用者是否有查看此專案的權限
    // IACUC_CHAIR、IACUC_STAFF、SYSTEM_ADMIN、VET 可以查看所有計畫書
    let has_view_all = current_user.permissions.contains(&"aup.protocol.view_all".to_string())
        || current_user.roles.contains(&"IACUC_CHAIR".to_string())
        || current_user.roles.contains(&"IACUC_STAFF".to_string())
        || current_user.roles.contains(&"VET".to_string())
        || current_user.roles.contains(&"SYSTEM_ADMIN".to_string());
    
    // 檢查是否為 PI、CLIENT 或 CO_EDITOR
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
    
    // 檢查是否為已指派的審查委員
    let is_assigned_reviewer: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM review_assignments 
            WHERE protocol_id = $1 
            AND reviewer_id = $2
        )
        "#
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));

    // 檢查是否為已指派的獸醫審查員
    let is_assigned_vet: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM vet_review_assignments 
            WHERE protocol_id = $1 
            AND vet_id = $2
        )
        "#
    )
    .bind(id)
    .bind(current_user.id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((false,));
    
    if !has_view_all && protocol.protocol.pi_user_id != current_user.id && !is_pi_or_coeditor.0 && !is_assigned_reviewer.0 && !is_assigned_vet.0 {
        return Err(AppError::Forbidden("You don't have permission to export this protocol".to_string()));
    }
    
    // 生成 PDF
    let pdf_bytes = PdfService::generate_protocol_pdf(&protocol)?;
    
    // 設定檔案名稱
    let filename = format!("{}_AUP計畫書.pdf", protocol.protocol.title);
    let encoded_filename = urlencoding::encode(&filename);
    
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename*=UTF-8''{}", encoded_filename)),
        ],
        pdf_bytes,
    ))
}

