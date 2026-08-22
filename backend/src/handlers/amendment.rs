use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        Amendment, AmendmentListItem, AmendmentQuery, AmendmentReviewAssignmentResponse,
        AmendmentStatusHistory, AmendmentVersion, ChangeAmendmentStatusRequest,
        ClassifyAmendmentRequest, CreateAmendmentRequest, CreateHistoricalAmendmentRequest,
        FinalizeHistoricalAmendmentRequest, MarkAmendmentEffectiveRequest, PendingCountResponse,
        RecordAmendmentDecisionRequest, RecordHistoricalReviewsRequest, UpdateAmendmentRequest,
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
    // R75-P4：PI 寫入授權前移至 Scoped<AmendmentWrite>（authorize = is_admin || 計畫 PI）→
    // service 吃證明、漏授權即編譯不過。
    let scope = access::Scoped::<access::AmendmentWrite>::authorize(
        &state.db,
        &current_user,
        req.protocol_id,
    )
    .await?;

    req.validate()?;
    let amendment = AmendmentService::create(&state.db, scope, &req, current_user.id).await?;
    Ok(Json(amendment))
}

/// 補登歷史變更申請（P6）：建立 is_historical 草稿。
/// POST /amendments/historical
/// 權限：計劃負責人 SD / 管理者，且計劃須為匯入計劃（imported_at 非 NULL）。
pub async fn create_historical_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateHistoricalAmendmentRequest>,
) -> Result<Json<Amendment>> {
    if !access::can_backfill_historical_amendment(&state.db, req.protocol_id, &current_user).await?
    {
        return Err(AppError::Forbidden(
            "無權補登此計劃的歷史變更（限計劃負責人 / 管理者，且須為匯入計劃）".into(),
        ));
    }
    req.validate()?;
    let actor = ActorContext::User(current_user.clone());
    let amendment = AmendmentService::create_historical(&state.db, &actor, &req).await?;
    Ok(Json(amendment))
}

/// 完成補登歷史變更（DRAFT → EFFECTIVE）。
/// POST /amendments/:id/finalize-historical
/// 權限：計劃負責人 SD / 管理者（由 amendment 解析所屬計劃後檢查）。
pub async fn finalize_historical_amendment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<FinalizeHistoricalAmendmentRequest>,
) -> Result<Json<Amendment>> {
    let protocol_id = AmendmentService::get_by_id_raw(&state.db, id)
        .await?
        .protocol_id;
    if !access::can_backfill_historical_amendment(&state.db, protocol_id, &current_user).await? {
        return Err(AppError::Forbidden(
            "無權完成此歷史變更的補登（限計劃負責人 / 管理者）".into(),
        ));
    }
    req.validate()?;
    let actor = ActorContext::User(current_user.clone());
    let amendment = AmendmentService::finalize_historical(&state.db, &actor, id, &req).await?;
    Ok(Json(amendment))
}

/// 補登歷史變更審查文件（P6-3）：全量取代委員審查指派。
/// POST /amendments/:id/historical-reviews
/// 權限：計劃負責人 SD / 管理者。
pub async fn record_historical_amendment_reviews(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<RecordHistoricalReviewsRequest>,
) -> Result<StatusCode> {
    let protocol_id = AmendmentService::get_by_id_raw(&state.db, id)
        .await?
        .protocol_id;
    if !access::can_backfill_historical_amendment(&state.db, protocol_id, &current_user).await? {
        return Err(AppError::Forbidden(
            "無權補登此歷史變更的審查文件（限計劃負責人 / 管理者）".into(),
        ));
    }
    let actor = ActorContext::User(current_user.clone());
    AmendmentService::record_historical_reviews(&state.db, &actor, id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
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
        // SQL 層直接過濾使用者可見的計畫（避免取全部再客端 filter）
        AmendmentService::list_for_user(&state.db, &query, current_user.id).await?
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

    // R34-3：IDOR 檢查改用 access::require_protocol_related_access（更廣 — 包含 PI / co-PI /
    // reviewer / vet_reviewer）；其他 amendment endpoint 已用 check_amendment_access 走相同 helper
    access::require_protocol_related_access(&state.db, &current_user, amendment.protocol_id)
        .await?;

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

    // R75-P4：PI 寫入授權前移至 Scoped<AmendmentWrite>（service 再以證明綁定本 amendment 所屬計畫）。
    let scope = access::Scoped::<access::AmendmentWrite>::authorize(
        &state.db,
        &current_user,
        current.protocol_id,
    )
    .await?;

    req.validate()?;
    let amendment = AmendmentService::update(&state.db, scope, id, &req).await?;
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

    // R75-P4：PI 寫入授權前移至 Scoped<AmendmentWrite>（service 再以證明綁定本 amendment 所屬計畫）。
    let scope = access::Scoped::<access::AmendmentWrite>::authorize(
        &state.db,
        &current_user,
        current.protocol_id,
    )
    .await?;

    let amendment = AmendmentService::submit(&state.db, scope, id, current_user.id).await?;

    // 非同步通知 IACUC_STAFF
    let db = state.db.clone();
    let amendment_id = amendment.id;
    let protocol_id = amendment.protocol_id;
    let amendment_title = amendment.title.clone();
    let operator_id = current_user.id;
    let config = state.config.clone();
    tokio::spawn(async move {
        // 查 protocol_no
        let protocol_no: Option<String> =
            sqlx::query_scalar("SELECT protocol_no FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_optional(&db)
                .await
                .ok()
                .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_amendment_progress(
                amendment_id,
                protocol_id,
                &protocol_no,
                &amendment_title,
                "submitted",
                operator_id,
                None,
                Some(&config),
            )
            .await
        {
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

    let actor = ActorContext::User(current_user.clone());
    let amendment = AmendmentService::classify(&state.db, &actor, id, &req).await?;
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
        return Err(AppError::Forbidden("無權啟動審查".into()));
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
        let protocol_no: Option<String> =
            sqlx::query_scalar("SELECT protocol_no FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_optional(&db)
                .await
                .ok()
                .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_amendment_progress(
                amendment_id,
                protocol_id,
                &protocol_no,
                &amendment_title,
                "under_review",
                operator_id,
                None,
                Some(&config),
            )
            .await
        {
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
    // H6 (GLP §11.70 / ISO A.5.18) 防禦深度：reviewer 必須同時具備
    // 1. amendment_review_assignments 的指派（DB layer）
    // 2. aup.amendment.approve 明確權限（RBAC layer）
    // admin 一律放行（require_permission! 已涵蓋 admin 短路）
    require_permission!(current_user, "aup.amendment.approve");

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
        return Err(AppError::Forbidden("無權記錄審查決定".into()));
    }

    let actor = ActorContext::User(current_user.clone());
    let assignment = AmendmentService::record_decision(&state.db, &actor, id, &req).await?;

    // 返回完整資訊
    let assignments = AmendmentService::get_review_assignments(&state.db, id).await?;
    let result = assignments
        .into_iter()
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
        return Err(AppError::Forbidden("無權變更狀態".into()));
    }

    let amendment = AmendmentService::change_status(&state.db, id, &req, current_user.id).await?;

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
        let protocol_no: Option<String> =
            sqlx::query_scalar("SELECT protocol_no FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_optional(&db)
                .await
                .ok()
                .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_amendment_progress(
                amendment_id,
                protocol_id,
                &protocol_no,
                &amendment_title,
                &new_status,
                operator_id,
                remark.as_deref(),
                Some(&config),
            )
            .await
        {
            tracing::warn!("發送修正案進度通知失敗: {e}");
        }
    });

    Ok(Json(amendment))
}

/// R30-25b：標記修正案為 EFFECTIVE（GLP §58 正式生效）
/// POST /amendments/:id/effective
pub async fn mark_amendment_effective(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<MarkAmendmentEffectiveRequest>,
) -> Result<Json<Amendment>> {
    // R30-25b follow-up：DTO validation（remark length 等）
    req.validate()?;

    // 與 change_amendment_status 同層級（IACUC_STAFF/CHAIR），不另開 permission
    if !current_user.has_permission("aup.protocol.change_status") {
        return Err(AppError::Forbidden("無權標記修正案為生效".into()));
    }

    // R30-25b follow-up：IDOR 防護 — amendment 屬 protocol，要求 protocol-related access
    // （與其他 amendment handler 對齊；amendment 不掛 animal，不適用 animal IDOR pattern）
    let amendment_pre = AmendmentService::get_by_id(&state.db, id).await?;
    check_amendment_access(&state.db, id, amendment_pre.protocol_id, &current_user).await?;

    let actor = ActorContext::User(current_user.clone());
    let amendment = AmendmentService::mark_effective(&state.db, &actor, id, &req).await?;

    // 非同步通知（與 change_amendment_status 行為對齊）
    let db = state.db.clone();
    let amendment_id = amendment.id;
    let protocol_id = amendment.protocol_id;
    let amendment_title = amendment.title.clone();
    let new_status = amendment.status.as_str().to_lowercase();
    let operator_id = current_user.id;
    let remark = req.remark.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        let protocol_no: Option<String> =
            sqlx::query_scalar("SELECT protocol_no FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_optional(&db)
                .await
                .ok()
                .flatten();
        let protocol_no = protocol_no.unwrap_or_default();

        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_amendment_progress(
                amendment_id,
                protocol_id,
                &protocol_no,
                &amendment_title,
                &new_status,
                operator_id,
                remark.as_deref(),
                Some(&config),
            )
            .await
        {
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
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<PendingCountResponse>> {
    // R75-9：原忽略 current_user 對所有人回全域數（洩漏全院審查工作量）。改比照
    // list_amendments：staff（view_all）看全域 triage、其餘只計自己可見計畫的 pending。
    let count = if current_user.has_permission("aup.protocol.view_all") {
        AmendmentService::get_pending_count(&state.db).await?
    } else {
        AmendmentService::get_pending_count_for_user(&state.db, current_user.id).await?
    };
    Ok(Json(PendingCountResponse { count }))
}
