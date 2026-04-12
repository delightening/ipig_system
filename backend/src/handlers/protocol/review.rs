// 審查管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        AssignReviewerRequest, CreateCommentRequest, ReplyCommentRequest,
        ReviewAssignment, ReviewAssignmentResponse, ReviewComment, ReviewCommentResponse,
        SaveDraftRequest, SubmitReplyRequest,
    },
    require_permission,
    services::{access, NotificationService, ProtocolService},
    AppError, AppState, Result,
};

#[derive(Debug, serde::Deserialize)]
pub struct ProtocolIdQuery {
    pub protocol_id: Option<Uuid>,
    pub protocol_version_id: Option<Uuid>,
}

/// 指派審查委員
#[utoipa::path(post, path = "/api/v1/reviews/assignments", request_body = AssignReviewerRequest, responses((status = 200, description = "指派成功", body = ReviewAssignment)), tag = "審查管理", security(("bearer" = [])))]
pub async fn assign_reviewer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<AssignReviewerRequest>,
) -> Result<Json<ReviewAssignment>> {
    require_permission!(current_user, "aup.review.assign");
    let assignment = ProtocolService::assign_reviewer(&state.db, &req, current_user.id).await?;

    // 非同步通知：審查指派
    let db = state.db.clone();
    let protocol_id = req.protocol_id;
    let reviewer_id = req.reviewer_id;
    tokio::spawn(async move {
        // 查詢計畫資訊與 PI 姓名
        let info: Option<(String, String, Uuid)> = sqlx::query_as(
            "SELECT protocol_no, title, pi_user_id FROM protocols WHERE id = $1",
        )
        .bind(protocol_id)
        .fetch_optional(&db)
        .await
        .ok()
        .flatten();

        if let Some((pno, ptitle, pi_user_id)) = info {
            let pi_name: String = sqlx::query_scalar(
                "SELECT COALESCE(display_name, email) FROM users WHERE id = $1",
            )
            .bind(pi_user_id)
            .fetch_one(&db)
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

            let svc = NotificationService::new(db);
            if let Err(e) = svc
                .notify_review_assignment(protocol_id, &pno, &ptitle, &pi_name, reviewer_id, None)
                .await
            {
                tracing::warn!("發送審查指派通知失敗: {e}");
            }
        }
    });

    Ok(Json(assignment))
}

/// 列出審查委員指派清單
#[utoipa::path(get, path = "/api/v1/reviews/assignments", responses((status = 200, description = "指派清單", body = Vec<ReviewAssignmentResponse>)), tag = "審查管理", security(("bearer" = [])))]
pub async fn list_review_assignments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolIdQuery>,
) -> Result<Json<Vec<ReviewAssignmentResponse>>> {
    require_permission!(current_user, "aup.protocol.view_own");
    let protocol_id = query.protocol_id
        .ok_or_else(|| AppError::Validation("protocol_id is required".to_string()))?;
    if !access::has_protocol_view_all(&current_user)
        && !current_user.is_admin()
        && !access::is_reviewer_or_vet(&state.db, protocol_id, current_user.id).await?
    {
        return Err(AppError::Forbidden("You don't have permission to view reviewer assignments for this protocol".to_string()));
    }
    let assignments: Vec<ReviewAssignmentResponse> = sqlx::query_as(
        r#"SELECT ra.id, ra.protocol_id, ra.reviewer_id,
            COALESCE(u_rev.display_name, u_rev.email) as reviewer_name, u_rev.email as reviewer_email,
            ra.assigned_by, COALESCE(u_as.display_name, u_as.email) as assigned_by_name,
            ra.assigned_at, ra.completed_at, ra.is_primary_reviewer, ra.review_stage
        FROM review_assignments ra
        JOIN users u_rev ON ra.reviewer_id = u_rev.id
        JOIN users u_as ON ra.assigned_by = u_as.id
        WHERE ra.protocol_id = $1"#
    ).bind(protocol_id).fetch_all(&state.db).await?;
    let vet_assignments: Vec<ReviewAssignmentResponse> = sqlx::query_as(
        r#"SELECT vra.id, vra.protocol_id, vra.vet_id as reviewer_id,
            COALESCE(u_vet.display_name, u_vet.email) as reviewer_name, u_vet.email as reviewer_email,
            COALESCE(vra.assigned_by, vra.id) as assigned_by,
            COALESCE(u_as.display_name, u_as.email, 'System') as assigned_by_name,
            vra.assigned_at, vra.completed_at, true as is_primary_reviewer,
            'VET_REVIEW'::text as review_stage
        FROM vet_review_assignments vra
        JOIN users u_vet ON vra.vet_id = u_vet.id
        LEFT JOIN users u_as ON vra.assigned_by = u_as.id
        WHERE vra.protocol_id = $1"#
    ).bind(protocol_id).fetch_all(&state.db).await?;
    let review_count = assignments.len();
    let vet_count = vet_assignments.len();
    let mut all_assignments = assignments;
    all_assignments.extend(vet_assignments);
    tracing::info!(
        "[list_review_assignments] protocol_id: {}, found {} review + {} vet = {} total",
        protocol_id, review_count, vet_count, all_assignments.len()
    );
    Ok(Json(all_assignments))
}

/// 新增審查意見
#[utoipa::path(post, path = "/api/v1/reviews/comments", request_body = CreateCommentRequest, responses((status = 201, description = "建立成功", body = ReviewComment)), tag = "審查管理", security(("bearer" = [])))]
pub async fn create_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<ReviewComment>> {
    req.validate()?;
    let has_global_perm = current_user.has_permission("aup.review.comment")
        || current_user.roles.iter().any(|r| [crate::constants::ROLE_IACUC_CHAIR, crate::constants::ROLE_IACUC_STAFF].contains(&r.as_str()));
    if !has_global_perm {
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT protocol_id FROM protocol_versions WHERE id = $1"
        ).bind(req.protocol_version_id).fetch_one(&state.db).await?;
        if !access::is_reviewer_or_vet(&state.db, protocol_id, current_user.id).await? {
            require_permission!(current_user, "aup.review.comment");
        }
    }
    let comment = ProtocolService::add_comment(&state.db, &req, current_user.id).await?;
    // 非同步通知
    let db = state.db.clone();
    let pvid = req.protocol_version_id;
    let commenter = current_user.email.clone();
    let content = req.content.clone();
    tokio::spawn(async move {
        let info: Option<(Uuid, String, String)> = sqlx::query_as(
            r#"SELECT p.id, p.protocol_no, p.title FROM protocols p JOIN protocol_versions pv ON p.id = pv.protocol_id WHERE pv.id = $1"#,
        ).bind(pvid).fetch_optional(&db).await.ok().flatten();
        if let Some((pid, pno, ptitle)) = info {
            let svc = NotificationService::new(db.clone());
            if let Err(e) = svc.notify_review_comment_created(pid, &pno, &commenter, &content).await {
                tracing::warn!("發送審查意見通知失敗: {e}");
            }

            // 檢查是否所有指定委員都已發表意見 → 觸發 all_reviews_completed
            let all_commented: Option<(bool,)> = sqlx::query_as(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM review_assignments ra
                    WHERE ra.protocol_id = $1
                      AND NOT EXISTS(
                          SELECT 1 FROM review_comments rc
                          WHERE rc.protocol_id = ra.protocol_id
                            AND rc.reviewer_id = ra.reviewer_id
                            AND rc.parent_comment_id IS NULL
                      )
                )
                "#
            ).bind(pid).fetch_optional(&db).await.ok().flatten();
            if let Some((true,)) = all_commented {
                if let Err(e) = svc.notify_all_reviews_completed(pid, &pno, &ptitle).await {
                    tracing::warn!("發送全員意見完成通知失敗: {e}");
                }
            }
        }
    });
    Ok(Json(comment))
}

/// 列出審查意見清單
#[utoipa::path(get, path = "/api/v1/reviews/comments", responses((status = 200, description = "意見清單", body = Vec<ReviewCommentResponse>)), tag = "審查管理", security(("bearer" = [])))]
pub async fn list_review_comments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolIdQuery>,
) -> Result<Json<Vec<ReviewCommentResponse>>> {
    let protocol_id = if let Some(pid) = query.protocol_id {
        pid
    } else if let Some(pvid) = query.protocol_version_id {
        sqlx::query_scalar("SELECT protocol_id FROM protocol_versions WHERE id = $1")
            .bind(pvid).fetch_one(&state.db).await?
    } else {
        return Err(AppError::Validation("protocol_id or protocol_version_id is required".to_string()));
    };
    access::require_protocol_related_access(&state.db, &current_user, protocol_id).await?;
    let comments = ProtocolService::get_comments(&state.db, protocol_id).await?;
    Ok(Json(comments))
}

/// 標記審查意見為已解決
#[utoipa::path(post, path = "/api/v1/reviews/comments/{id}/resolve", params(("id" = Uuid, Path, description = "意見 ID")), responses((status = 200, description = "標記成功", body = ReviewComment)), tag = "審查管理", security(("bearer" = [])))]
pub async fn resolve_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReviewComment>> {
    let comment = ProtocolService::resolve_comment(&state.db, id, current_user.id).await?;

    // 非同步檢查：是否所有意見都已解決 → 觸發 all_comments_resolved
    let db = state.db.clone();
    if let Some(protocol_id) = comment.protocol_id {
    tokio::spawn(async move {
        // 取得計畫資訊
        let info: Option<(String, String)> = sqlx::query_as(
            "SELECT protocol_no, title FROM protocols WHERE id = $1",
        ).bind(protocol_id).fetch_optional(&db).await.ok().flatten();

        if let Some((pno, ptitle)) = info {
            // 檢查是否所有意見都已解決（僅檢查頂層意見，非回覆）
            let all_resolved: Option<(bool,)> = sqlx::query_as(
                r#"
                SELECT NOT EXISTS(
                    SELECT 1 FROM review_comments
                    WHERE protocol_id = $1
                      AND parent_comment_id IS NULL
                      AND is_resolved = false
                )
                "#
            ).bind(protocol_id).fetch_optional(&db).await.ok().flatten();

            if let Some((true,)) = all_resolved {
                let svc = NotificationService::new(db);
                if let Err(e) = svc.notify_all_comments_resolved(protocol_id, &pno, &ptitle).await {
                    tracing::warn!("發送全部意見已解決通知失敗: {e}");
                }
            }
        }
    });
    }

    Ok(Json(comment))
}

/// 回覆審查意見
#[utoipa::path(post, path = "/api/v1/reviews/comments/reply", request_body = ReplyCommentRequest, responses((status = 200, description = "回覆成功", body = ReviewComment)), tag = "審查管理", security(("bearer" = [])))]
pub async fn reply_review_comment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ReplyCommentRequest>,
) -> Result<Json<ReviewComment>> {
    req.validate()?;
    if !current_user.has_permission("aup.review.reply") {
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"
        ).bind(req.parent_comment_id).fetch_one(&state.db).await?;
        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (
                SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
                UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
            )"#
        ).bind(protocol_id).bind(current_user.id).fetch_one(&state.db).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Permission denied: requires aup.review.reply or being the protocol owner/co-editor".to_string()));
        }
    }
    let comment = ProtocolService::reply_comment(&state.db, &req, current_user.id).await?;
    Ok(Json(comment))
}

/// 儲存草稿回覆
#[utoipa::path(post, path = "/api/v1/reviews/comments/draft", request_body = SaveDraftRequest, responses((status = 200, description = "儲存成功", body = ReviewComment)), tag = "審查管理", security(("bearer" = [])))]
pub async fn save_reply_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SaveDraftRequest>,
) -> Result<Json<ReviewComment>> {
    req.validate()?;
    if !current_user.has_permission("aup.review.reply") {
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"
        ).bind(req.comment_id).fetch_one(&state.db).await?;
        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (
                SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
                UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
            )"#
        ).bind(protocol_id).bind(current_user.id).fetch_one(&state.db).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Permission denied: requires aup.review.reply or being the protocol owner/co-editor".to_string()));
        }
    }
    let comment = ProtocolService::save_reply_draft(&state.db, req.comment_id, &req.draft_content, current_user.id).await?;
    Ok(Json(comment))
}

/// 取得草稿回覆
#[utoipa::path(get, path = "/api/v1/reviews/comments/{id}/draft", params(("id" = Uuid, Path, description = "意見 ID")), responses((status = 200, description = "草稿內容")), tag = "審查管理", security(("bearer" = [])))]
pub async fn get_reply_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<Option<String>>> {
    if !current_user.has_permission("aup.review.reply") {
        let (protocol_id,): (Uuid,) = sqlx::query_as(
            "SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"
        ).bind(comment_id).fetch_one(&state.db).await?;
        let is_owner = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS (
                SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
                UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2
            )"#
        ).bind(protocol_id).bind(current_user.id).fetch_one(&state.db).await?;
        if !is_owner {
            return Err(AppError::Forbidden("Permission denied: requires aup.review.reply or being the protocol owner/co-editor".to_owned()));
        }
    }
    let draft = ProtocolService::get_reply_draft(&state.db, comment_id).await?;
    Ok(Json(draft))
}

/// 正式送出草稿回覆
#[utoipa::path(post, path = "/api/v1/reviews/comments/submit-draft", request_body = SubmitReplyRequest, responses((status = 200, description = "送出成功", body = ReviewComment)), tag = "審查管理", security(("bearer" = [])))]
pub async fn submit_reply_from_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SubmitReplyRequest>,
) -> Result<Json<ReviewComment>> {
    let comment_info: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT pv.protocol_id FROM review_comments rc JOIN protocol_versions pv ON rc.protocol_version_id = pv.id WHERE rc.id = $1"#
    ).bind(req.comment_id).fetch_optional(&state.db).await?;
    let (protocol_id,) = comment_info
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
    let is_authorized: (bool,) = sqlx::query_as(
        r#"SELECT EXISTS(
            SELECT 1 FROM protocols WHERE id = $1 AND pi_user_id = $2
            UNION SELECT 1 FROM user_protocols WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol IN ('PI', 'CO_EDITOR')
        )"#
    ).bind(protocol_id).bind(current_user.id).fetch_one(&state.db).await?;
    if !is_authorized.0 {
        return Err(AppError::Forbidden("Only PI or Co-editor can submit reply from draft".to_string()));
    }
    let comment = ProtocolService::submit_reply_from_draft(&state.db, req.comment_id, current_user.id).await?;
    Ok(Json(comment))
}
