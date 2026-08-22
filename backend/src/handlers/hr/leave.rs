// 請假管理 Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        ApproveLeaveRequest, CancelLeaveRequest, CreateLeaveRequest, LeaveCalendarResponse,
        LeaveQuery, LeaveRequest, LeaveRequestWithUser, PaginatedResponse, ProxyRejectRequest,
        RejectLeaveRequest, UpdateLeaveRequest,
    },
    services::{HrService, NotificationService},
    AppState, Result,
};

/// 請假行事曆的查詢區間（FullCalendar 換月時帶入）
#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct LeaveCalendarQuery {
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
}

/// 請假行事曆
///
/// 與 `/hr/calendar/events`（Google 同步事件）不同：本端點直接讀 `leave_requests`，
/// 因此含**審核中**的假單，且可見範圍依觀看者的部門與代理關係動態決定。
///
/// 權限刻意用獨立的 `hr.leave.view_calendar` 而非 `hr.leave.view_all`：
/// 行事曆只揭露「誰哪天不在、誰代理」，不含事由與附件；沿用 `view_all` 會
/// 等於用行事曆繞過請假紀錄列表的權限閘。
#[utoipa::path(
    get,
    path = "/api/v1/hr/leaves/calendar",
    params(LeaveCalendarQuery),
    responses((status = 200, body = LeaveCalendarResponse)),
    tag = "HR 請假",
    security(("bearer" = []))
)]
pub async fn get_leave_calendar(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<LeaveCalendarQuery>,
) -> Result<Json<LeaveCalendarResponse>> {
    // 不另外檢查 is_admin()：`CurrentUser::has_permission` 本身已對 admin 短路
    // （middleware/auth.rs），多寫一個 `|| is_admin()` 是死碼，只會讓人誤以為
    // 這裡的授權語意比實際更寬。
    if !current_user.has_permission("hr.leave.view_calendar") {
        return Err(crate::error::AppError::Forbidden(
            "無權查看請假行事曆".into(),
        ));
    }
    let data =
        HrService::get_leave_calendar(&state.db, &current_user, params.start_date, params.end_date)
            .await?;
    Ok(Json(data))
}

/// 列出請假記錄
#[utoipa::path(get, path = "/api/v1/hr/leaves", responses((status = 200)), tag = "HR 請假", security(("bearer" = [])))]
pub async fn list_leaves(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<LeaveQuery>,
) -> Result<Json<PaginatedResponse<LeaveRequestWithUser>>> {
    let mut query = params;
    if query.view_all.unwrap_or(false) {
        let is_admin = current_user.is_admin();
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
#[utoipa::path(post, path = "/api/v1/hr/leaves", request_body = CreateLeaveRequest, responses((status = 201)), tag = "HR 請假", security(("bearer" = [])))]
pub async fn create_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateLeaveRequest>,
) -> Result<(StatusCode, Json<LeaveRequest>)> {
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::create_leave(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(record)))
}

/// 更新請假申請
pub async fn update_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::update_leave(&state.db, &actor, id, &payload).await?;
    Ok(Json(record))
}

/// 刪除請假申請
pub async fn delete_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let actor = ActorContext::User(current_user.clone());
    HrService::delete_leave(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 提交請假申請（送審後進「待代理確認」關，通知代理人確認）
pub async fn submit_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<LeaveRequest>> {
    let actor = ActorContext::User(current_user.clone());
    // 代理人「待您確認」的置頂待辦已改在 HrService::submit_leave 的 tx 內建立
    // （與狀態轉換 all-or-nothing，見該函式內註解），這裡不再另外 fire-and-forget
    // 一則 info 通知——那會讓代理人同一件事收到兩則（一則待辦、一則告知）。
    let record = HrService::submit_leave(&state.db, &actor, id).await?;
    Ok(Json(record))
}

/// 代理人確認請假（PENDING_PROXY → 單位主管/負責人關），並通知後續審核者
pub async fn proxy_confirm_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<LeaveRequest>> {
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::proxy_confirm_leave(&state.db, &actor, id).await?;
    let db = state.db.clone();
    let leave_id = record.id;
    let applicant_id = record.user_id;
    let leave_type = record.leave_type.clone();
    let start_date = record.start_date.to_string();
    let end_date = record.end_date.to_string();
    // 代理確認後假單進入審核關，通知審核者（依 notification_routing：單位主管/負責人角色）。
    // 申請人姓名由 notify_leave_submitted 依 applicant_id 反查（含在職/未刪除過濾）。
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_leave_submitted(leave_id, applicant_id, &leave_type, &start_date, &end_date)
            .await
        {
            tracing::warn!("發送請假待審通知失敗: {e}");
        }
    });
    Ok(Json(record))
}

/// 代理人退回請假（PENDING_PROXY → DRAFT），並通知申請人重新指定代理人
pub async fn proxy_reject_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ProxyRejectRequest>,
) -> Result<Json<LeaveRequest>> {
    let actor = ActorContext::User(current_user.clone());
    let record =
        HrService::proxy_reject_leave(&state.db, &actor, id, payload.reason.as_deref()).await?;
    let db = state.db.clone();
    let leave_id = record.id;
    let applicant_id = record.user_id;
    let leave_type = record.leave_type.clone();
    let start_date = record.start_date.to_string();
    let end_date = record.end_date.to_string();
    let reason = payload.reason.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_leave_proxy_rejected(
                leave_id,
                applicant_id,
                &leave_type,
                &start_date,
                &end_date,
                reason.as_deref(),
            )
            .await
        {
            tracing::warn!("發送代理人退回通知失敗: {e}");
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
    // 授權（兩關 + 職責分離 + admin 卡關代批）全數集中於 HrService::approve_leave。
    let actor = ActorContext::User(current_user.clone());
    let record =
        HrService::approve_leave(&state.db, &actor, id, payload.comments.as_deref()).await?;
    let db = state.db.clone();
    let leave_id = record.id;
    let leave_type = record.leave_type.clone();
    let start_date = record.start_date.to_string();
    let end_date = record.end_date.to_string();
    if record.status == crate::models::LeaveStatus::Approved.as_str() {
        // R72-3：最終核准後通知申請人 + 職務代理人「已生效」（中間階段不發）。
        let recipient = record.user_id;
        let proxy_id = record.proxy_user_id;
        tokio::spawn(async move {
            let svc = NotificationService::new(db);
            if let Err(e) = svc
                .notify_leave_approved(leave_id, recipient, &leave_type, &start_date, &end_date)
                .await
            {
                tracing::warn!("發送請假核准通知失敗: {e}");
            }
            if let Some(proxy) = proxy_id {
                if let Err(e) = svc
                    .notify_leave_proxy_effective(
                        leave_id,
                        proxy,
                        &leave_type,
                        &start_date,
                        &end_date,
                    )
                    .await
                {
                    tracing::warn!("發送代理人生效通知失敗: {e}");
                }
            }
        });
    }
    // 中途核准（單位主管 → 負責人）：下一關待辦的建立已下沉至
    // `HrService::approve_leave` 的 tx 內，與當關待辦的解除原子化，這裡不再另外
    // best-effort 建立（CodeRabbit PR #67 review：commit 後 tokio::spawn 建立與
    // tx 內解除不對稱，失敗會讓假單靜默卡關；且在此重複建立會產生重複置頂待辦）。
    Ok(Json(record))
}

/// 駁回請假申請
pub async fn reject_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    // 授權（與核准同一資格：該關合法審核人、職責分離、admin 卡關代批）集中於 HrService::reject_leave。
    let actor = ActorContext::User(current_user.clone());
    let record = HrService::reject_leave(&state.db, &actor, id, &payload.reason).await?;
    // 駁回後通知申請人（含駁回原因）；resolver event_subject 已過濾停用/已刪除帳號（GDPR）。
    let db = state.db.clone();
    let leave_id = record.id;
    let applicant_id = record.user_id;
    let leave_type = record.leave_type.clone();
    let start_date = record.start_date.to_string();
    let end_date = record.end_date.to_string();
    let reason = payload.reason.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_leave_rejected(
                leave_id,
                applicant_id,
                &leave_type,
                &start_date,
                &end_date,
                Some(&reason),
            )
            .await
        {
            tracing::warn!("發送請假駁回通知失敗: {e}");
        }
    });
    Ok(Json(record))
}

/// 取消請假
pub async fn cancel_leave(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelLeaveRequest>,
) -> Result<Json<LeaveRequest>> {
    // IDOR 授權守衛、was_approved 推導、申請人名稱反查皆已下沉至 HrService::cancel_leave
    // （符 CLAUDE.md §4：handler 禁止 SQL / DB 讀取）。
    let actor = ActorContext::User(current_user.clone());
    let outcome = HrService::cancel_leave(&state.db, &actor, id, payload.reason.as_deref()).await?;
    let record = outcome.record;

    // 若原本是已核准的假單，通知所有核准經手人
    if outcome.was_approved && record.status == "CANCELLED" {
        let db = state.db.clone();
        let leave_id = record.id;
        let leave_type = record.leave_type.clone();
        let start_date = record.start_date.to_string();
        let end_date = record.end_date.to_string();
        let applicant_name = outcome.applicant_name;
        let reason = record.cancellation_reason.clone();
        tokio::spawn(async move {
            let svc = NotificationService::new(db);
            if let Err(e) = svc
                .notify_leave_cancelled(
                    leave_id,
                    &applicant_name,
                    &leave_type,
                    &start_date,
                    &end_date,
                    reason.as_deref(),
                )
                .await
            {
                tracing::warn!("發送請假取消通知失敗: {e}");
            }
        });
    }

    Ok(Json(record))
}
