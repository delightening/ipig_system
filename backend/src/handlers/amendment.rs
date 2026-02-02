use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        Amendment, AmendmentListItem, AmendmentQuery, AmendmentReviewAssignmentResponse,
        AmendmentStatusHistory, AmendmentVersion, ChangeAmendmentStatusRequest,
        ClassifyAmendmentRequest, CreateAmendmentRequest, RecordAmendmentDecisionRequest,
        UpdateAmendmentRequest,
    },
    require_permission,
    services::AmendmentService,
    AppError, AppState, Result,
};

/// 建立變更申請
/// POST /amendments
pub async fn create_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateAmendmentRequest>,
) -> Result<Json<Amendment>> {
    // 權限檢查：PI 可以建立變更申請
    // 檢查使用者是否為該計畫的 PI
    let is_pi = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        ) as "exists!"
        "#,
        current_user.user_id,
        req.protocol_id
    )
    .fetch_one(&state.pool)
    .await?;

    if !is_pi && !current_user.roles.contains(&"admin".to_string()) 
        && !current_user.roles.contains(&"SYSTEM_ADMIN".to_string()) {
        return Err(AppError::Forbidden("Only PI can create amendments".into()));
    }

    req.validate()?;
    let amendment = AmendmentService::create(&state.pool, &req, current_user.user_id).await?;
    Ok(Json(amendment))
}

/// 列出變更申請
/// GET /amendments
pub async fn list_amendments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AmendmentQuery>,
) -> Result<Json<Vec<AmendmentListItem>>> {
    // IACUC_STAFF, CHAIR, SYSTEM_ADMIN 可以查看所有變更申請
    // 其他角色只能查看自己相關的
    let is_staff = current_user.roles.iter().any(|r| 
        ["admin", "SYSTEM_ADMIN", "IACUC_STAFF", "CHAIR"].contains(&r.as_str())
    );

    let amendments = if is_staff {
        AmendmentService::list(&state.pool, &query).await?
    } else {
        // 查詢使用者相關的計畫
        let my_protocols: Vec<Uuid> = sqlx::query_scalar!(
            r#"SELECT protocol_id FROM user_protocols WHERE user_id = $1"#,
            current_user.user_id
        )
        .fetch_all(&state.pool)
        .await?;

        // 只返回使用者相關計畫的變更申請
        let all = AmendmentService::list(&state.pool, &query).await?;
        all.into_iter()
            .filter(|a| my_protocols.contains(&a.protocol_id))
            .collect()
    };

    Ok(Json(amendments))
}

/// 取得單一變更申請
/// GET /amendments/:id
pub async fn get_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Amendment>> {
    let amendment = AmendmentService::get_by_id(&state.pool, id).await?;

    // 權限檢查
    let is_staff = current_user.roles.iter().any(|r| 
        ["admin", "SYSTEM_ADMIN", "IACUC_STAFF", "CHAIR", "REVIEWER", "VET"].contains(&r.as_str())
    );

    if !is_staff {
        // 檢查是否為相關人員
        let is_related = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_protocols 
                WHERE user_id = $1 AND protocol_id = $2
            ) as "exists!"
            "#,
            current_user.user_id,
            amendment.protocol_id
        )
        .fetch_one(&state.pool)
        .await?;

        if !is_related {
            return Err(AppError::Forbidden("Not authorized to view this amendment".into()));
        }
    }

    Ok(Json(amendment))
}

/// 更新變更申請
/// PATCH /amendments/:id
pub async fn update_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAmendmentRequest>,
) -> Result<Json<Amendment>> {
    let current = AmendmentService::get_by_id(&state.pool, id).await?;

    // 檢查是否為該計畫的 PI
    let is_pi = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        ) as "exists!"
        "#,
        current_user.user_id,
        current.protocol_id
    )
    .fetch_one(&state.pool)
    .await?;

    if !is_pi && !current_user.roles.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden("Only PI can update amendments".into()));
    }

    req.validate()?;
    let amendment = AmendmentService::update(&state.pool, id, &req).await?;
    Ok(Json(amendment))
}

/// 提交變更申請
/// POST /amendments/:id/submit
pub async fn submit_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Amendment>> {
    let current = AmendmentService::get_by_id(&state.pool, id).await?;

    // 只有 PI 可以提交
    let is_pi = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        ) as "exists!"
        "#,
        current_user.user_id,
        current.protocol_id
    )
    .fetch_one(&state.pool)
    .await?;

    if !is_pi && !current_user.roles.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden("Only PI can submit amendments".into()));
    }

    let amendment = AmendmentService::submit(&state.pool, id, current_user.user_id).await?;
    Ok(Json(amendment))
}

/// 分類變更申請（IACUC_STAFF）
/// POST /amendments/:id/classify
pub async fn classify_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ClassifyAmendmentRequest>,
) -> Result<Json<Amendment>> {
    // 只有 IACUC_STAFF 可以分類
    require_permission!(state, current_user, "amendment.classify");

    let amendment = AmendmentService::classify(&state.pool, id, &req, current_user.user_id).await?;
    Ok(Json(amendment))
}

/// 開始審查（IACUC_STAFF/CHAIR）
/// POST /amendments/:id/start-review
pub async fn start_amendment_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Amendment>> {
    // 只有 IACUC_STAFF/CHAIR 可以開始審查
    let is_authorized = current_user.roles.iter().any(|r| 
        ["admin", "SYSTEM_ADMIN", "IACUC_STAFF", "CHAIR"].contains(&r.as_str())
    );

    if !is_authorized {
        return Err(AppError::Forbidden("Not authorized to start review".into()));
    }

    let amendment = AmendmentService::start_review(&state.pool, id, current_user.user_id).await?;
    Ok(Json(amendment))
}

/// 記錄審查決定（審查委員）
/// POST /amendments/:id/decision
pub async fn record_amendment_decision(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<RecordAmendmentDecisionRequest>,
) -> Result<Json<AmendmentReviewAssignmentResponse>> {
    // 檢查使用者是否為指派的審查委員
    let is_reviewer = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM amendment_review_assignments 
            WHERE amendment_id = $1 AND reviewer_id = $2
        ) as "exists!"
        "#,
        id,
        current_user.user_id
    )
    .fetch_one(&state.pool)
    .await?;

    if !is_reviewer && !current_user.roles.contains(&"admin".to_string()) {
        return Err(AppError::Forbidden("Not authorized to record decision".into()));
    }

    let assignment = AmendmentService::record_decision(
        &state.pool, 
        id, 
        current_user.user_id, 
        &req
    ).await?;

    // 返回完整資訊
    let assignments = AmendmentService::get_review_assignments(&state.pool, id).await?;
    let result = assignments.into_iter()
        .find(|a| a.id == assignment.id)
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

    Ok(Json(result))
}

/// 變更狀態
/// POST /amendments/:id/status
pub async fn change_amendment_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeAmendmentStatusRequest>,
) -> Result<Json<Amendment>> {
    // 只有 IACUC_STAFF/CHAIR 可以直接變更狀態
    let is_authorized = current_user.roles.iter().any(|r| 
        ["admin", "SYSTEM_ADMIN", "IACUC_STAFF", "CHAIR"].contains(&r.as_str())
    );

    if !is_authorized {
        return Err(AppError::Forbidden("Not authorized to change status".into()));
    }

    let amendment = AmendmentService::change_status(
        &state.pool, 
        id, 
        &req, 
        current_user.user_id
    ).await?;

    Ok(Json(amendment))
}

/// 取得版本列表
/// GET /amendments/:id/versions
pub async fn get_amendment_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentVersion>>> {
    let versions = AmendmentService::get_versions(&state.pool, id).await?;
    Ok(Json(versions))
}

/// 取得狀態歷程
/// GET /amendments/:id/history
pub async fn get_amendment_history(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentStatusHistory>>> {
    let history = AmendmentService::get_status_history(&state.pool, id).await?;
    Ok(Json(history))
}

/// 取得審查委員指派列表
/// GET /amendments/:id/assignments
pub async fn get_amendment_assignments(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentReviewAssignmentResponse>>> {
    let assignments = AmendmentService::get_review_assignments(&state.pool, id).await?;
    Ok(Json(assignments))
}

/// 取得計畫的變更申請列表
/// GET /protocols/:id/amendments
pub async fn list_protocol_amendments(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentListItem>>> {
    let amendments = AmendmentService::list_by_protocol(&state.pool, protocol_id).await?;
    Ok(Json(amendments))
}
