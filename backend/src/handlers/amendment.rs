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
        UpdateAmendmentRequest, PendingCountResponse,
    },
    require_permission,
    services::{access, AmendmentService, NotificationService},
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
        current_user.id,
        req.protocol_id
    )
    .fetch_one(&state.db)
    .await?;

    if !is_pi && !current_user.is_admin() {
        return Err(AppError::Forbidden("Only PI can create amendments".into()));
    }

    req.validate()?;
    let amendment = AmendmentService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(amendment))
}

/// 列出變更申請
/// GET /amendments
pub async fn list_amendments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AmendmentQuery>,
) -> Result<Json<Vec<AmendmentListItem>>> {
    let is_staff = current_user.has_permission("aup.protocol.view_all");

    let amendments = if is_staff {
        AmendmentService::list(&state.db, &query).await?
    } else {
        // 查詢使用者相關的計畫
        let my_protocols: Vec<Uuid> = sqlx::query_scalar!(
            r#"SELECT protocol_id FROM user_protocols WHERE user_id = $1"#,
            current_user.id
        )
        .fetch_all(&state.db)
        .await?;

        // 只返回使用者相關計畫的變更申請
        let all = AmendmentService::list(&state.db, &query).await?;
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
    let amendment = AmendmentService::get_by_id(&state.db, id).await?;

    let is_staff = current_user.has_permission("aup.protocol.view_all");

    if !is_staff {
        // 檢查是否為相關人員
        let is_related = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_protocols 
                WHERE user_id = $1 AND protocol_id = $2
            ) as "exists!"
            "#,
            current_user.id,
            amendment.protocol_id
        )
        .fetch_one(&state.db)
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
    let current = AmendmentService::get_by_id(&state.db, id).await?;

    // 檢查是否為該計畫的 PI
    let is_pi = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        ) as "exists!"
        "#,
        current_user.id,
        current.protocol_id
    )
    .fetch_one(&state.db)
    .await?;

    if !is_pi && !current_user.is_admin() {
        return Err(AppError::Forbidden("Only PI can update amendments".into()));
    }

    req.validate()?;
    let amendment = AmendmentService::update(&state.db, id, &req).await?;
    Ok(Json(amendment))
}

/// 提交變更申請
/// POST /amendments/:id/submit
pub async fn submit_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Amendment>> {
    let current = AmendmentService::get_by_id(&state.db, id).await?;

    // 只有 PI 可以提交
    let is_pi = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_protocols 
            WHERE user_id = $1 AND protocol_id = $2 AND role_in_protocol = 'PI'
        ) as "exists!"
        "#,
        current_user.id,
        current.protocol_id
    )
    .fetch_one(&state.db)
    .await?;

    if !is_pi && !current_user.is_admin() {
        return Err(AppError::Forbidden("Only PI can submit amendments".into()));
    }

    let amendment = AmendmentService::submit(&state.db, id, current_user.id).await?;

    // 非同步通知 IACUC_STAFF
    let db = state.db.clone();
    let amendment_id = amendment.id;
    let protocol_id = amendment.protocol_id;
    let amendment_title = amendment.title.clone();
    let operator_id = current_user.id;
    let config = state.config.clone();
    tokio::spawn(async move {
        // 查 protocol_no
        let protocol_no: Option<String> = sqlx::query_scalar(
            "SELECT protocol_no FROM protocols WHERE id = $1"
        )
        .bind(protocol_id)
        .fetch_optional(&db)
        .await
        .ok()
        .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc.notify_amendment_progress(
            amendment_id, protocol_id, &protocol_no, &amendment_title,
            "submitted", operator_id, None, Some(&config),
        ).await {
            tracing::warn!("發送修正案進度通知失敗: {e}");
        }

    });

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
    require_permission!(current_user, "aup.amendment.classify");

    let amendment = AmendmentService::classify(&state.db, id, &req, current_user.id).await?;
    Ok(Json(amendment))
}

/// 開始審查（IACUC_STAFF/CHAIR）
/// POST /amendments/:id/start-review
pub async fn start_amendment_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Amendment>> {
    if !current_user.has_permission("aup.protocol.review") {
        return Err(AppError::Forbidden("Not authorized to start review".into()));
    }

    let amendment = AmendmentService::start_review(&state.db, id, current_user.id).await?;

    // 非同步通知審查委員
    let db = state.db.clone();
    let amendment_id = amendment.id;
    let protocol_id = amendment.protocol_id;
    let amendment_title = amendment.title.clone();
    let operator_id = current_user.id;
    let config = state.config.clone();
    tokio::spawn(async move {
        let protocol_no: Option<String> = sqlx::query_scalar(
            "SELECT protocol_no FROM protocols WHERE id = $1"
        )
        .bind(protocol_id)
        .fetch_optional(&db)
        .await
        .ok()
        .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc.notify_amendment_progress(
            amendment_id, protocol_id, &protocol_no, &amendment_title,
            "under_review", operator_id, None, Some(&config),
        ).await {
            tracing::warn!("發送修正案進度通知失敗: {e}");
        }

    });

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
        current_user.id
    )
    .fetch_one(&state.db)
    .await?;

    if !is_reviewer && !current_user.is_admin() {
        return Err(AppError::Forbidden("Not authorized to record decision".into()));
    }

    let assignment = AmendmentService::record_decision(
        &state.db, 
        id, 
        current_user.id, 
        &req
    ).await?;

    // 返回完整資訊
    let assignments = AmendmentService::get_review_assignments(&state.db, id).await?;
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
    if !current_user.has_permission("aup.protocol.change_status") {
        return Err(AppError::Forbidden("Not authorized to change status".into()));
    }

    let amendment = AmendmentService::change_status(
        &state.db, 
        id, 
        &req, 
        current_user.id
    ).await?;

    // 非同步通知
    let db = state.db.clone();
    let amendment_id = amendment.id;
    let protocol_id = amendment.protocol_id;
    let amendment_title = amendment.title.clone();
    let new_status = amendment.status.as_str().to_lowercase();
    let operator_id = current_user.id;
    let remark = req.remark.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        let protocol_no: Option<String> = sqlx::query_scalar(
            "SELECT protocol_no FROM protocols WHERE id = $1"
        )
        .bind(protocol_id)
        .fetch_optional(&db)
        .await
        .ok()
        .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc.notify_amendment_progress(
            amendment_id, protocol_id, &protocol_no, &amendment_title,
            &new_status, operator_id, remark.as_deref(), Some(&config),
        ).await {
            tracing::warn!("發送修正案進度通知失敗: {e}");
        }

    });

    Ok(Json(amendment))
}

/// 檢查使用者是否有權存取該修正案所屬的計畫（IDOR 防護）
async fn check_amendment_access(
    db: &sqlx::PgPool,
    _amendment_id: Uuid,
    protocol_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    access::require_protocol_related_access(db, current_user, protocol_id).await
}

/// 取得版本列表
/// GET /amendments/:id/versions
pub async fn get_amendment_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentVersion>>> {
    let amendment = AmendmentService::get_by_id(&state.db, id).await?;
    check_amendment_access(&state.db, id, amendment.protocol_id, &current_user).await?;
    let versions = AmendmentService::get_versions(&state.db, id).await?;
    Ok(Json(versions))
}

/// 取得狀態歷程
/// GET /amendments/:id/history
pub async fn get_amendment_history(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentStatusHistory>>> {
    let amendment = AmendmentService::get_by_id(&state.db, id).await?;
    check_amendment_access(&state.db, id, amendment.protocol_id, &current_user).await?;
    let history = AmendmentService::get_status_history(&state.db, id).await?;
    Ok(Json(history))
}

/// 取得審查委員指派列表
/// GET /amendments/:id/assignments
pub async fn get_amendment_assignments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentReviewAssignmentResponse>>> {
    let amendment = AmendmentService::get_by_id(&state.db, id).await?;
    check_amendment_access(&state.db, id, amendment.protocol_id, &current_user).await?;
    let assignments = AmendmentService::get_review_assignments(&state.db, id).await?;
    Ok(Json(assignments))
}

/// 取得計畫的變更申請列表
/// GET /protocols/:id/amendments
pub async fn list_protocol_amendments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<Vec<AmendmentListItem>>> {
    access::require_protocol_related_access(&state.db, &current_user, protocol_id).await?;
    let amendments = AmendmentService::list_by_protocol(&state.db, protocol_id).await?;
    Ok(Json(amendments))
}

/// 取得待處理變更申請數量
/// GET /amendments/pending-count
pub async fn get_pending_count(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<PendingCountResponse>> {
    let count = AmendmentService::get_pending_count(&state.db).await?;
    Ok(Json(PendingCountResponse { count }))
}
