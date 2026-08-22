// 專案 CRUD Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AcknowledgeNoticeRequest, AssignableUser, ChangeStatusRequest, CreateProtocolRequest,
        FinalizeImportRequest, ImportApprovedProtocolRequest, ImportReviewsRequest,
        NoticeAcknowledgementStatus, Protocol, ProtocolActivityResponse, ProtocolListItem,
        ProtocolNoticeAcknowledgement, ProtocolQuery, ProtocolResponse, ProtocolVersion,
        SaveVetReviewFormRequest, UpdateProtocolRequest,
    },
    require_permission,
    services::{access, NotificationService, ProtocolService, UserService},
    AppError, AppState, Result,
};

#[derive(Debug, Deserialize)]
pub struct CopyProtocolRequest {
    /// 新計畫的 PI（不填則沿用來源計畫的 PI）
    pub pi_user_id: Option<Uuid>,
}

/// 建立專案
#[utoipa::path(post, path = "/api/v1/protocols", request_body = CreateProtocolRequest, responses((status = 201, description = "建立成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn create_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProtocolRequest>,
) -> Result<Json<Protocol>> {
    // PI 角色本身已持有 aup.protocol.create（見 startup/permissions.rs），
    // has_permission() 另對 admin 一律回 true，故單一權限檢查即涵蓋原本三條件。
    if !current_user.has_permission("aup.protocol.create") {
        return Err(AppError::Forbidden(
            "Permission denied: requires aup.protocol.create".to_string(),
        ));
    }
    req.validate()?;
    let actor = ActorContext::User(current_user.clone());
    let protocol = ProtocolService::create(&state.db, &actor, &req, current_user.id).await?;
    Ok(Json(protocol))
}

/// 匯入已核准計畫（場內既有、已通過審查的計劃直接建立成 APPROVED，跳過審查流程）
#[utoipa::path(post, path = "/api/v1/protocols/import-approved", request_body = ImportApprovedProtocolRequest, responses((status = 200, description = "匯入成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn import_approved_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ImportApprovedProtocolRequest>,
) -> Result<Json<Protocol>> {
    let can_import =
        current_user.has_permission("aup.protocol.import_approved") || current_user.is_admin();
    if !can_import {
        return Err(AppError::Forbidden(
            "Permission denied: requires aup.protocol.import_approved".to_string(),
        ));
    }
    req.validate()?;
    let actor = ActorContext::User(current_user.clone());
    let protocol =
        ProtocolService::import_approved(&state.db, &actor, &req, current_user.id).await?;
    Ok(Json(protocol))
}

/// 列出可指派為計畫 PI / Study Director 的在職使用者（精簡資料）。
/// 供匯入頁下拉選擇 PI / SD；授權門檻同匯入計畫（具 import_approved 權限或 admin），
/// 因此 EXPERIMENT_STAFF 等匯入者亦可取用，不需 admin.user.view。
#[utoipa::path(get, path = "/api/v1/protocols/assignable-users", responses((status = 200, description = "可指派使用者清單", body = [AssignableUser])), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn list_assignable_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AssignableUser>>> {
    let can_view =
        current_user.has_permission("aup.protocol.import_approved") || current_user.is_admin();
    if !can_view {
        return Err(AppError::Forbidden(
            "Permission denied: requires aup.protocol.import_approved".to_string(),
        ));
    }
    let users = UserService::list_assignable_users(&state.db).await?;
    Ok(Json(users))
}

/// import P1：完成補登（清 import_pending + 建 v1 版本快照 + 記原始版本號）
#[utoipa::path(post, path = "/api/v1/protocols/{id}/finalize-import", request_body = FinalizeImportRequest, responses((status = 200, description = "完成補登", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn finalize_import_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<FinalizeImportRequest>,
) -> Result<Json<Protocol>> {
    // import P1：完成補登限 建立者 / 負責人(SD) / 管理者（與補登編輯同權限）。
    if !access::can_manage_import_pending(&state.db, id, &current_user).await? {
        return Err(AppError::Forbidden(
            "無權完成此計劃的補登（限建立者 / 計劃負責人 / 管理者）".to_string(),
        ));
    }
    req.validate()?;
    let actor = ActorContext::User(current_user.clone());
    let protocol =
        ProtocolService::finalize_import(&state.db, &actor, id, req.original_version_label).await?;
    Ok(Json(protocol))
}

/// import P2：補登審查文件（執秘 / 委員 / 獸醫意見）至真實審查表。
/// 限補登中（import_pending）計劃；權限同補登編輯（建立者 / 負責人 / 管理者）。
#[utoipa::path(post, path = "/api/v1/protocols/{id}/import-reviews", request_body = ImportReviewsRequest, responses((status = 204, description = "已記錄補登審查文件")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn record_import_reviews(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ImportReviewsRequest>,
) -> Result<StatusCode> {
    if !access::can_manage_import_pending(&state.db, id, &current_user).await? {
        return Err(AppError::Forbidden(
            "無權補登此計劃的審查文件（限建立者 / 計劃負責人 / 管理者）".to_string(),
        ));
    }
    let actor = ActorContext::User(current_user.clone());
    ProtocolService::record_import_reviews(&state.db, &actor, id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// R64-5c：刪除誤匯的匯入計劃（admin only，硬刪 + 限無下游資料）以重新匯入。
#[utoipa::path(delete, path = "/api/v1/protocols/{id}/imported", responses((status = 204, description = "已刪除匯入計劃")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn delete_imported_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden(
            "僅系統管理員可刪除匯入計劃".to_string(),
        ));
    }
    let actor = ActorContext::User(current_user.clone());
    ProtocolService::delete_imported_protocol(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Admin 軟刪除計畫（限「已否決」狀態）：設為 DELETED 從列表隱藏，保留資料供稽核。
#[utoipa::path(post, path = "/api/v1/protocols/{id}/soft-delete", responses((status = 204, description = "已軟刪除計畫")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn soft_delete_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅系統管理員可軟刪除計畫".to_string()));
    }
    let actor = ActorContext::User(current_user.clone());
    ProtocolService::soft_delete_protocol(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 申請人（PI/SD）簽署當前生效的動物試驗申請須知（手寫電子簽章）。
#[utoipa::path(post, path = "/api/v1/protocols/{id}/acknowledge-notice", responses((status = 200, description = "已簽署", body = ProtocolNoticeAcknowledgement)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn acknowledge_notice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<AcknowledgeNoticeRequest>,
) -> Result<Json<ProtocolNoticeAcknowledgement>> {
    let actor = ActorContext::User(current_user.clone());
    // R75-P4：簽署授權前移至 Scoped<NoticeSign>（authorize 內跑 can_sign_notice）→
    // service 吃證明、漏授權即編譯不過。
    let scope =
        access::Scoped::<access::NoticeSign>::authorize(&state.db, &current_user, id).await?;
    let ack = ProtocolService::acknowledge_notice(
        &state.db,
        &actor,
        scope,
        &req.handwriting_svg,
        req.stroke_data.as_ref(),
    )
    .await?;
    Ok(Json(ack))
}

/// 取得計畫的須知簽署狀態（當前生效須知 + 是否已簽）。供填表/送審顯示。
#[utoipa::path(get, path = "/api/v1/protocols/{id}/notice-acknowledgement", responses((status = 200, description = "簽署狀態", body = NoticeAcknowledgementStatus)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_notice_acknowledgement_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<NoticeAcknowledgementStatus>> {
    let scope =
        access::Scoped::<access::ProtocolId>::authorize(&state.db, &current_user, id).await?;
    Ok(Json(
        ProtocolService::get_notice_status(&state.db, scope).await?,
    ))
}

/// 列出所有專案
#[utoipa::path(get, path = "/api/v1/protocols", responses((status = 200, description = "專案清單", body = Vec<ProtocolListItem>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn list_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProtocolQuery>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    let has_view_all = current_user.has_permission("aup.protocol.view_all")
        || current_user.roles.iter().any(|r| {
            [
                crate::constants::ROLE_IACUC_STAFF,
                crate::constants::ROLE_VET,
                crate::constants::ROLE_REVIEWER,
                crate::constants::ROLE_IACUC_CHAIR,
            ]
            .contains(&r.as_str())
        });
    // 刻意保留角色硬判（不轉 permission）：這是「角色集合形狀」檢查——使用者的
    // 全部角色是否恰好落在 {REVIEWER, VET} 之內——而非「持有某項能力」。permission
    // 只能表達加法式授予，無法表達「僅限這些、不含其他」的排他性，故無等價權限碼可轉。
    let is_reviewer_only = current_user.roles.iter().all(|r| {
        [crate::constants::ROLE_REVIEWER, crate::constants::ROLE_VET].contains(&r.as_str())
    }) && (current_user.has_role(crate::constants::ROLE_REVIEWER)
        || current_user.has_role(crate::constants::ROLE_VET));
    // 草稿監督角色：執秘 / 主席 / admin 對草稿全可見（唯讀）；其餘 view_all 角色（如
    // EXPERIMENT_STAFF）僅見自己為 PI/SD/成員的草稿，避免草稿經本端點外洩。
    let viewer_sees_all_drafts = current_user.is_admin()
        || current_user.roles.iter().any(|r| {
            [
                crate::constants::ROLE_IACUC_STAFF,
                crate::constants::ROLE_IACUC_CHAIR,
            ]
            .contains(&r.as_str())
        });
    // SO 銷貨計畫下拉的 SD 收斂（2026-07-22 裁定）：對齊 authorize_sales_document——
    // admin 可開任何計畫故不過濾；其餘（含 view_all 角色）僅列「自己是該計畫 SD」
    // 的計畫，避免下拉列出選了也會被後端擋下的計畫。
    //
    // 判準與 `authorize_sales_document` 必須一致：兩邊原本共用
    // `aup.protocol.sd_authority` 權限碼，該碼隨全域 STUDY_DIRECTOR 角色移除後，
    // 一起改為顯式的 `is_admin()`——若一邊改一邊沒改，下拉會列出送出後才被擋的計畫。
    let sd_only = query.sd_only && !current_user.is_admin();
    let mut protocols = if sd_only {
        ProtocolService::get_my_protocols(&state.db, current_user.id, &query, true).await?
    } else if current_user.is_admin() || has_view_all {
        ProtocolService::list(
            &state.db,
            &query,
            current_user.id,
            current_user.is_admin(),
            viewer_sees_all_drafts,
        )
        .await?
    } else {
        ProtocolService::get_my_protocols(&state.db, current_user.id, &query, false).await?
    };
    if is_reviewer_only {
        protocols.retain(|p| {
            matches!(
                p.status,
                crate::models::ProtocolStatus::Submitted
                    | crate::models::ProtocolStatus::PreReview
                    | crate::models::ProtocolStatus::VetReview
                    | crate::models::ProtocolStatus::UnderReview
                    | crate::models::ProtocolStatus::Approved
                    | crate::models::ProtocolStatus::ApprovedWithConditions
                    | crate::models::ProtocolStatus::Closed
            )
        });
    }
    Ok(Json(protocols))
}

/// 取得單個專案
#[utoipa::path(get, path = "/api/v1/protocols/{id}", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "專案詳細", body = ProtocolResponse)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProtocolResponse>> {
    require_permission!(current_user, "aup.protocol.view_own");
    // R75-P4：授權前移至 Scoped<ProtocolView>（authorize 內取 pi_user_id 兼存在性檢查 + 跑
    // require_protocol_view_access），fail-closed 早於資料組裝；get_for_view 須持證明才能取單筆。
    let scope =
        access::Scoped::<access::ProtocolView>::authorize(&state.db, &current_user, id).await?;
    let mut response = ProtocolService::get_for_view(&state.db, scope).await?;
    // 權威 can_edit：與後端 can_edit_protocol 一致（admin / PI 含成員 PI / SD / 補登管理者），
    // 供前端編輯·送出按鈕 gating，避免前端自行重算而漏掉 backend 放行的情境。
    response.can_edit = access::can_edit_protocol(&state.db, &current_user, id).await?;
    // 權威 can_write_amendment：與 Scoped<AmendmentWrite> 同一判斷（admin / PI，不含 SD），
    // 供前端修正案建立·送審按鈕 gating，避免前端硬判角色漏掉 backend 放行的情境
    // （或反之，讓 SD / EXPERIMENT_STAFF 看到必定 403 的按鈕）。
    response.can_write_amendment =
        access::can_write_amendment(&state.db, &current_user, id).await?;
    Ok(Json(response))
}

/// 更新專案
#[utoipa::path(put, path = "/api/v1/protocols/{id}", params(("id" = Uuid, Path, description = "專案 ID")), request_body = UpdateProtocolRequest, responses((status = 200, description = "更新成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn update_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProtocolRequest>,
) -> Result<Json<Protocol>> {
    // 欄位感知授權：內容變更（標題 / 表單 / 日期）須 can_edit（PI / SD / admin）；
    // 純 SD 指派（僅 study_director_user_id / version）允許執秘 / admin 協調指派。
    // update 吃 Scoped<ProtocolEdit> → 漏授權即編譯不過；SD 值的合法性由 service
    // validate_and_authorize_sd 進一步把關。
    // 窮舉解構：未來 UpdateProtocolRequest 新增欄位時，編譯器強制在此分類為
    // 內容相關 / 非內容相關，避免漏分類造成授權旁路（Gemini review）。
    let UpdateProtocolRequest {
        title,
        working_content,
        start_date,
        end_date,
        study_director_user_id: _,
        version: _,
        source_form_version,
    } = &req;
    // source_form_version（重選版本 / 升級最新版）屬結構性內容變更 → 歸內容類、須 can_edit。
    let touches_content = title.is_some()
        || working_content.is_some()
        || start_date.is_some()
        || end_date.is_some()
        || source_form_version.is_some();
    let scope = access::Scoped::<access::ProtocolEdit>::authorize_update(
        &state.db,
        &current_user,
        id,
        touches_content,
    )
    .await?;
    req.validate()?;
    let actor = ActorContext::User(current_user.clone());
    let protocol = ProtocolService::update(&state.db, &actor, scope, &req).await?;
    Ok(Json(protocol))
}

/// 提交專案
#[utoipa::path(post, path = "/api/v1/protocols/{id}/submit", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "提交成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn submit_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Protocol>> {
    // 送出權收緊為計畫關係人：admin / PI / SD（對齊 can_edit_protocol 與原始 spec §4.1
    // 「提交計畫僅 PI ✓」）。執秘 / CLIENT / CO_EDITOR 不再可代為送出。
    let can_submit = current_user.is_admin()
        || access::is_protocol_pi(&state.db, id, current_user.id).await?
        || access::is_study_director(&state.db, id, current_user.id).await?;
    if !can_submit {
        return Err(AppError::Forbidden(
            "You don't have permission to submit this protocol".to_string(),
        ));
    }
    // Service-driven: pass ActorContext; service 內含 transaction + audit + HMAC chain
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let protocol = ProtocolService::submit(&state.db, &actor, id).await?;

    // 非同步通知：計畫已提交
    let db = state.db.clone();
    let p_id = protocol.id;
    let p_no = protocol.protocol_no.clone();
    let p_title = protocol.title.clone();
    tokio::spawn(async move {
        // 查詢 PI 姓名（客人）：以研究資料 basic.pi 為準，fallback FK 使用者
        let pi_name: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT {} FROM protocols p JOIN users u ON p.pi_user_id = u.id WHERE p.id = $1",
            crate::utils::pi_sql::pi_display_name("u.display_name, u.email"),
        )))
        .bind(p_id)
        .fetch_one(&db)
        .await
        .unwrap_or_else(|_| "Unknown".to_string());

        let svc = NotificationService::new(db);
        if let Err(e) = svc
            .notify_protocol_submitted(p_id, &p_no, &p_title, &pi_name)
            .await
        {
            tracing::warn!("發送計畫提交通知失敗: {e}");
        }
    });

    Ok(Json(protocol))
}

/// 變更專案狀態
#[utoipa::path(post, path = "/api/v1/protocols/{id}/status", params(("id" = Uuid, Path, description = "專案 ID")), request_body = ChangeStatusRequest, responses((status = 200, description = "狀態變更成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn change_protocol_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<ChangeStatusRequest>,
) -> Result<Json<Protocol>> {
    tracing::info!(
        "[ChangeStatus] User: {}, Target Status: {:?}",
        current_user.id,
        req.to_status
    );
    if matches!(req.to_status, crate::models::ProtocolStatus::Deleted) {
        tracing::info!("[ChangeStatus] Entering Delete Permission Check for status DELETED");
        require_permission!(current_user, "aup.protocol.delete");
    } else if req.to_status == crate::models::ProtocolStatus::Closed
        && !current_user.has_permission("aup.protocol.change_status")
    {
        // R89-8：計畫擁有人（PI/SD）結案自己計畫的窄縫通道，不要求完整
        // aup.protocol.change_status（那個權限開放審查流程各階段的狀態轉換能力，
        // 僅限 IACUC_CHAIR/IACUC_STAFF）。粗篩在這裡：目標是 CLOSED 且沒有完整權限，
        // 才走窄權限；「是不是本人」與「現在狀態是否可結案」交給
        // ProtocolService::change_status_tx（已鎖 row，避免這裡與 tx 內查兩次）。
        tracing::info!("[ChangeStatus] Entering PI/SD Close-Own Permission Check");
        require_permission!(current_user, "aup.protocol.close_own");
    } else {
        tracing::info!("[ChangeStatus] Entering Normal Status Change Check");
        require_permission!(current_user, "aup.protocol.change_status");
    }
    // 防範 IDOR：確認使用者與此計畫書有關聯（view_all 角色直接通過）
    access::require_protocol_related_access(&state.db, &current_user, id).await?;
    let actor = ActorContext::User(current_user.clone());
    let protocol = ProtocolService::change_status(&state.db, &actor, id, &req).await?;
    let db = state.db.clone();
    let protocol_id = protocol.id;
    let protocol_no = protocol.protocol_no.clone();
    let protocol_title = protocol.title.clone();
    let new_status = protocol.status.as_str().to_lowercase();
    let operator_id = current_user.id;
    let pi_user_id = protocol.pi_user_id;
    let reason = req.remark.clone();
    let config = state.config.clone();
    tokio::spawn(async move {
        let svc = NotificationService::new(db);
        // 角色驅動：路由表通知（行政/獸醫/審查委員等）
        if let Err(e) = svc
            .notify_protocol_review_progress(
                protocol_id,
                &protocol_no,
                &protocol_title,
                &new_status,
                operator_id,
                reason.as_deref(),
                Some(&config),
            )
            .await
        {
            tracing::warn!("發送計畫審查進度通知失敗: {e}");
        }
        // 直接通知 PI 狀態變更。避免重複：(1) 操作者本人是 PI 時不通知；
        // (2) notify_protocol_review_progress 已對終局/退回狀態通知 PI/Coeditor，
        //     這些狀態跳過，否則 PI 會收到兩則相同的「計畫狀態更新」通知。
        if operator_id != pi_user_id
            && !NotificationService::review_progress_notifies_pi(&new_status)
        {
            if let Err(e) = svc
                .notify_protocol_status_change(
                    protocol_id,
                    &protocol_no,
                    &protocol_title,
                    &new_status,
                    pi_user_id,
                    reason.as_deref(),
                )
                .await
            {
                tracing::warn!("發送計畫狀態變更通知給 PI 失敗: {e}");
            }
        }
    });

    // 進入 UnderReview 時，通知被指派的審查委員
    if req.to_status == crate::models::ProtocolStatus::UnderReview {
        if let Some(reviewer_ids) = req.reviewer_ids.clone() {
            let db = state.db.clone();
            let config = state.config.clone();
            let pid = protocol.id;
            let pno = protocol.protocol_no.clone();
            let ptitle = protocol.title.clone();
            tokio::spawn(async move {
                // PI 姓名（客人）：以研究資料 basic.pi 為準，fallback FK 使用者
                let pi_name: String = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT {} FROM protocols p JOIN users u ON p.pi_user_id = u.id WHERE p.id = $1",
                    crate::utils::pi_sql::pi_display_name("u.display_name, u.email"),
                )))
                .bind(pid)
                .fetch_one(&db)
                .await
                .unwrap_or_else(|_| "Unknown".to_string());

                // 並行發送所有審查委員通知（取代逐個順序發送）
                let handles: Vec<_> = reviewer_ids
                    .into_iter()
                    .map(|rid| {
                        let db = db.clone();
                        let config = config.clone();
                        let pno = pno.clone();
                        let ptitle = ptitle.clone();
                        let pi_name = pi_name.clone();
                        tokio::spawn(async move {
                            let svc = NotificationService::new(db);
                            if let Err(e) = svc
                                .notify_review_assignment(
                                    pid,
                                    &pno,
                                    &ptitle,
                                    &pi_name,
                                    rid,
                                    None,
                                    Some(config.as_ref()),
                                )
                                .await
                            {
                                tracing::warn!("發送審查委員指派通知失敗 (reviewer={}): {e}", rid);
                            }
                        })
                    })
                    .collect();
                for h in handles {
                    let _ = h.await;
                }
            });
        }
    }

    // R20-8: 進入 PreReview 時自動觸發執行秘書 AI 標註
    if req.to_status == crate::models::ProtocolStatus::PreReview {
        let db = state.db.clone();
        let config = state.config.clone();
        let pid = protocol.id;
        tokio::spawn(async move {
            if let Err(e) = crate::services::AiReviewService::review_protocol(
                &db,
                &config,
                pid,
                "staff_pre_review",
                None,
            )
            .await
            {
                tracing::warn!("[R20-8] 自動觸發執行秘書 AI 標註失敗: {e}");
            }
        });
    }

    Ok(Json(protocol))
}

/// 取得專案版本
#[utoipa::path(get, path = "/api/v1/protocols/{id}/versions", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "版本清單", body = Vec<ProtocolVersion>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol_versions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolVersion>>> {
    let scope =
        access::Scoped::<access::ProtocolId>::authorize(&state.db, &current_user, id).await?;
    let versions = ProtocolService::get_versions(&state.db, scope).await?;
    Ok(Json(versions))
}

/// 取得專案活動歷程
#[utoipa::path(get, path = "/api/v1/protocols/{id}/activities", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "活動歷程", body = Vec<ProtocolActivityResponse>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol_activities(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<ProtocolActivityResponse>>> {
    let scope =
        access::Scoped::<access::ProtocolId>::authorize(&state.db, &current_user, id).await?;
    let activities = ProtocolService::get_activities(&state.db, scope).await?;
    Ok(Json(activities))
}

/// 列出我的專案清單
#[utoipa::path(get, path = "/api/v1/my-projects", responses((status = 200, description = "我的專案清單", body = Vec<ProtocolListItem>)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_my_protocols(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ProtocolListItem>>> {
    // /my-projects 不帶查詢參數（前端 widget 直呼），以預設 query（無過濾）取成員全清單。
    let protocols = ProtocolService::get_my_protocols(
        &state.db,
        current_user.id,
        &ProtocolQuery::default(),
        false,
    )
    .await?;
    Ok(Json(protocols))
}

/// 取得專案的動物統計（儀表板用）
#[utoipa::path(get, path = "/api/v1/protocols/{id}/animal-stats", params(("id" = Uuid, Path, description = "專案 ID")), responses((status = 200, description = "動物統計")), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn get_protocol_animal_stats(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "aup.protocol.view_own");
    // R75-4：防跨客戶讀取。view_own 權限幾乎所有角色（含外部 CLIENT）皆持有，故須加
    // 「能檢視此計畫」物件層檢查，否則 CLIENT 可查任一計畫的動物計數（跨客戶 metadata）。
    access::require_protocol_related_access(&state.db, &current_user, id).await?;
    let protocol: Option<(Option<String>,)> =
        sqlx::query_as("SELECT iacuc_no FROM protocols WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let iacuc_no = match protocol {
        Some((Some(no),)) => no,
        _ => {
            return Ok(Json(serde_json::json!({
                "approved_count": 0, "in_use_count": 0, "completed_count": 0, "remaining_count": 0
            })))
        }
    };
    let stats: (i64, i64, i64) = sqlx::query_as(
        r#"SELECT
            COUNT(*) FILTER (WHERE status = 'in_experiment') as in_use_count,
            COUNT(*) FILTER (WHERE status IN ('completed', 'euthanized', 'sudden_death')) as completed_count,
            COUNT(*) as total_count
        FROM animals WHERE iacuc_no = $1 AND deleted_at IS NULL"#
    ).bind(&iacuc_no).fetch_one(&state.db).await?;
    let approved_count: (Option<i64>,) = sqlx::query_as(
        r#"SELECT (working_content->>'animal_count')::bigint as approved_count FROM protocols WHERE id = $1"#
    ).bind(id).fetch_one(&state.db).await?;
    let approved = approved_count.0.unwrap_or(stats.2);
    let remaining = approved - stats.2;
    Ok(Json(serde_json::json!({
        "approved_count": approved, "in_use_count": stats.0, "completed_count": stats.1, "remaining_count": remaining.max(0)
    })))
}

/// 儲存獸醫審查表
#[utoipa::path(post, path = "/api/v1/reviews/vet-form", request_body = SaveVetReviewFormRequest, responses((status = 200, description = "儲存成功")), tag = "審查管理", security(("bearer" = [])))]
pub async fn save_vet_review_form(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<SaveVetReviewFormRequest>,
) -> Result<Json<()>> {
    let is_vet = current_user.has_permission("aup.protocol.vet_review");
    if !is_vet && !access::is_assigned_vet(&state.db, req.protocol_id, current_user.id).await? {
        return Err(AppError::Forbidden(
            "Permission denied: You are not assigned as a vet for this protocol".to_string(),
        ));
    }
    ProtocolService::save_vet_review_form(
        &state.db,
        req.protocol_id,
        current_user.id,
        &req.review_form,
    )
    .await?;
    if let Err(e) = ProtocolService::record_activity(
        &state.db,
        req.protocol_id,
        crate::models::ProtocolActivityType::StatusChanged,
        current_user.id,
        None,
        None,
        Some(("VET_REVIEW_FORM", req.protocol_id, "獸醫審查表")),
        Some("填寫獸醫核選表".to_string()),
        Some(req.review_form.clone()),
    )
    .await
    {
        tracing::warn!("記錄活動失敗: {e}");
    }
    Ok(Json(()))
}

/// 複製既有計畫建立新草稿
#[utoipa::path(post, path = "/api/v1/protocols/{id}/copy", params(("id" = Uuid, Path, description = "來源計畫 ID")), responses((status = 201, description = "複製成功", body = Protocol)), tag = "計畫書管理", security(("bearer" = [])))]
pub async fn copy_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Protocol>)> {
    // PI 角色本身已持有 aup.protocol.create（見 startup/permissions.rs），
    // has_permission() 另對 admin 一律回 true，故單一權限檢查即涵蓋原本三條件。
    if !current_user.has_permission("aup.protocol.create") {
        return Err(AppError::Forbidden(
            "Permission denied: requires aup.protocol.create".to_string(),
        ));
    }
    // R75-1：防跨客戶複製。複製會回傳來源計畫完整 working_content，故須先驗「能檢視來源計畫」
    // （view_all 角色或為該計畫成員）。否則僅憑 create 權的 PI 可複製任一客戶計畫並讀其內容。
    let scope =
        access::Scoped::<access::ProtocolId>::authorize(&state.db, &current_user, id).await?;
    let actor = ActorContext::User(current_user.clone());
    let protocol = ProtocolService::copy(&state.db, &actor, scope, current_user.id).await?;
    Ok((StatusCode::CREATED, Json(protocol)))
}
