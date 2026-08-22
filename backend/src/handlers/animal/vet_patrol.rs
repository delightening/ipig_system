// 獸醫巡場報告 Handlers

use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{CreateNotificationRequest, NotificationType},
    require_permission,
    services::{
        access, CreateVetPatrolReportRequest, FileCategory, FileService,
        UpdateVetPatrolReportRequest, VetPatrolEntryPhoto, VetPatrolListFilter, VetPatrolPhoto,
        VetPatrolReport, VetPatrolReportService, VetPatrolReportWithEntries,
    },
    AppState, Result,
};

#[derive(Debug, Deserialize)]
pub struct ListReportsQuery {
    /// R40-15：直接 deserialize 為 enum；URL 值 = snake_case（`completed` default /
    /// `my_drafts` / `my_acknowledgements` / `my_followups` / `all`）。
    ///
    /// 兩種「沒值」情境（CodeRabbit PR #407 review 釐清）：
    ///   - 缺漏（無 `?status=...`）→ `#[serde(default)]` 給 `None` → `unwrap_or_default()`
    ///     落回 `Completed`
    ///   - **不認得的值**（如 `?status=foo`）→ serde 拋 deserialization error → axum
    ///     Query extractor 回 **400 Bad Request**（非 fallback；嚴格驗證刻意為之）
    #[serde(default)]
    pub status: Option<VetPatrolListFilter>,
}

/// 列出巡場報告（R39+ 兩階段流程 filter）
pub async fn list_vet_patrol_reports(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(q): Query<ListReportsQuery>,
) -> Result<Json<Vec<VetPatrolReport>>> {
    // R75-5: 巡場為全場內部福利文件；限內部 staff（view_all 或 SD），擋外部 CLIENT/PI 跨客戶讀取
    access::require_vet_patrol_view(&current_user)?;
    let filter = q.status.unwrap_or_default();
    // status=all（全系統所有報告，含他人草稿/在途）限 admin；非 admin 降級為 relevant
    // （只看與我相關），避免非 admin 經直接 API 讀取他人草稿。
    let filter = match filter {
        VetPatrolListFilter::All if !current_user.is_admin() => VetPatrolListFilter::Relevant,
        other => other,
    };
    let reports = VetPatrolReportService::list(&state.db, filter, current_user.id).await?;
    Ok(Json(reports))
}

/// R39+ phase 1：獸醫送出給追蹤者（draft → awaiting_follow_up）
#[derive(Debug, Deserialize)]
pub struct SubmitForFollowupRequest {
    pub follow_up_user_id: Uuid,
}

pub async fn submit_for_followup(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<SubmitForFollowupRequest>,
) -> Result<Json<VetPatrolReport>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    let report =
        VetPatrolReportService::submit_for_followup(&state.db, &actor, id, req.follow_up_user_id)
            .await?;

    // 置頂待辦（bell）已由 service 在**同一個 tx 內**建立
    // （見 VetPatrolReportService::create_followup_pin_tx）——
    // 與報告狀態轉換 all-or-nothing，不再是這裡的 best-effort。
    // 理由：原本 commit 後才建立，INSERT 失敗只 warn → 獸醫看到送出成功、
    // 追蹤者卻永遠收不到待辦；且與 retract/delete 的 tx 內解除形成競態。
    let notification_content = format!(
        "{} 已將「{}」巡場報告指派給您追蹤；請填寫各條目的「追蹤改善」欄位後送出完成。",
        current_user.email, report.patrol_date,
    );

    // 站內信（MessagingService 1-1 direct thread）—— 這條維持 best-effort：
    // 它是輔助通知管道，失敗不該讓已成立的送出動作回滾。
    let thread_req = crate::services::CreateThreadRequest {
        thread_type: "direct".to_string(),
        recipient_ids: vec![req.follow_up_user_id],
        subject: None,
        body: notification_content.clone(),
        attachment_ids: vec![],
    };
    if let Err(e) =
        crate::services::MessagingService::create_thread(&state.db, &actor, &thread_req).await
    {
        tracing::warn!(
            "vet_patrol submit_for_followup messaging thread 失敗 user={}: {e}",
            req.follow_up_user_id
        );
    }

    Ok(Json(report))
}

/// R39++ phase 2：追蹤者按「確認收到」（awaiting_acknowledgement → awaiting_follow_up）
pub async fn acknowledge_receipt(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VetPatrolReport>> {
    // 追蹤者（陪同人員）階段動作：追蹤者通常為非獸醫 staff（如 EXPERIMENT_STAFF），
    // 不具 animal.vet.recommend。handler 僅守 view gate（限內部可檢視 staff）；
    // 「必須是指派的 follow_up_user_id」由 service 層擁有權檢查（見 acknowledge_receipt）強制。
    access::require_vet_patrol_view(&current_user)?;
    let actor = ActorContext::User(current_user.clone());
    let (report, newly_acked) =
        VetPatrolReportService::acknowledge_receipt(&state.db, &actor, id).await?;

    // 僅在「真的由本次請求完成確認」時通知獸醫；冪等 no-op（前端自動觸發的 race /
    // 雙觸發）不重複發通知與站內信。
    if newly_acked {
        notify_vet_followup_stage(
            &state,
            &report,
            format!("[巡場報告] 追蹤者已收到 {}", report.patrol_date),
            format!(
                "{} 已確認收到「{}」巡場報告，正在準備回覆。",
                current_user.email, report.patrol_date,
            ),
            "acknowledge_receipt",
        )
        .await;
    }

    Ok(Json(report))
}

/// 通知獸醫追蹤者完成了某階段動作（fire-and-forget）。
///
/// acknowledge_receipt / complete_followup 共用：兩者只 title + content 不同，
/// 其他都重複（target=created_by、type=VetRecommendation、related_entity_id=report.id）。
async fn notify_vet_followup_stage(
    state: &AppState,
    report: &VetPatrolReport,
    title: String,
    content: String,
    stage_label: &str,
) {
    let Some(vet_id) = report.created_by else {
        return;
    };
    let notif_svc = crate::services::NotificationService::new(state.db.clone());
    if let Err(e) = notif_svc
        .create_notification(CreateNotificationRequest {
            user_id: vet_id,
            notification_type: NotificationType::VetRecommendation,
            title,
            content: Some(content),
            related_entity_type: Some("vet_patrol_reports".to_string()),
            related_entity_id: Some(report.id),
        })
        .await
    {
        tracing::warn!("vet_patrol {stage_label} notification 失敗 vet={vet_id}: {e}");
    }
}

/// R39++ phase 3：追蹤者按「確認完成」（awaiting_follow_up → completed, locked）
pub async fn complete_followup(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VetPatrolReport>> {
    // 追蹤者階段動作：同 acknowledge_receipt，僅守 view gate；
    // 「必須是指派的 follow_up_user_id」由 service 層 complete_followup 強制。
    access::require_vet_patrol_view(&current_user)?;
    let actor = ActorContext::User(current_user.clone());
    // 置頂待辦的解除在 service 的同一個 tx 內完成（見
    // VetPatrolReportService::complete_followup），不在此處另外呼叫——
    // 先 commit 狀態再解除會開競態窗口，詳見
    // NotificationService::resolve_pinned_notifications_tx。
    let report = VetPatrolReportService::complete_followup(&state.db, &actor, id).await?;

    notify_vet_followup_stage(
        &state,
        &report,
        format!("[巡場報告] 追蹤者已完成回覆 {}", report.patrol_date),
        format!(
            "{} 已完成「{}」巡場報告的追蹤改善回覆，報告已鎖定。",
            current_user.email, report.patrol_date,
        ),
        "complete_followup",
    )
    .await;

    Ok(Json(report))
}

/// R39+++ 棄置草稿（HARD delete + unlink 照片實體檔）
///
/// 用於：使用者開了新增 dialog → 預建空 draft → 沒按「存草稿」就關閉。
/// status 必須是 draft；非自己的草稿或非 admin 也擋下。
pub async fn discard_vet_patrol_draft(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    let unlink_failures = VetPatrolReportService::discard_draft(&state.db, &actor, id).await?;
    if unlink_failures > 0 {
        tracing::warn!(
            "vet_patrol discard_draft id={id} unlink_failures={unlink_failures}（DB row 已清，但有檔案 unlink 失敗）"
        );
    }
    Ok(StatusCode::NO_CONTENT)
}

/// 取得單一巡場報告（含條目）
pub async fn get_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<VetPatrolReportWithEntries>>> {
    // R75-5: 巡場為全場內部福利文件；限內部 staff（view_all 或 SD），擋外部 CLIENT/PI 跨客戶讀取
    access::require_vet_patrol_view(&current_user)?;
    let report = VetPatrolReportService::get(&state.db, id).await?;
    Ok(Json(report))
}

/// 建立巡場報告
///
/// R39: 回傳 `VetPatrolReportWithEntries`（不只 report meta），讓前端 auto-save 後拿到
/// server-assigned entry IDs，省一次額外 GET round-trip。
pub async fn create_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateVetPatrolReportRequest>,
) -> Result<Json<VetPatrolReportWithEntries>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    let report = VetPatrolReportService::create(&state.db, &actor, &req).await?;
    let with_entries = VetPatrolReportService::get(&state.db, report.id)
        .await?
        .ok_or_else(|| AppError::Internal("剛建立的報告卻找不到，極不應該".into()))?;
    Ok(Json(with_entries))
}

/// 更新巡場報告
///
/// R39: 同 create — 回傳 WithEntries 省 GET round-trip；同時對 service 回傳的
/// `to_unlink` 在 commit 後逐一刪檔（CASCADE 清掉的 entry photo 實體檔案）。
pub async fn update_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVetPatrolReportRequest>,
) -> Result<Json<VetPatrolReportWithEntries>> {
    // update 為雙用途端點：草稿期由發起獸醫改全部欄位；待追蹤期由追蹤者（可能非獸醫）
    // 補填「追蹤改善」。故 handler 只守 view gate，實際權限由 service 層依 status 分派：
    // 草稿→限 created_by；待追蹤→限 follow_up_user_id（見 VetPatrolReportService::update）。
    access::require_vet_patrol_view(&current_user)?;
    let actor = ActorContext::User(current_user.clone());
    let (_report, to_unlink) = VetPatrolReportService::update(&state.db, &actor, id, &req).await?;
    for path in &to_unlink {
        if let Err(e) = FileService::delete(path).await {
            tracing::warn!("vet_patrol entry photo unlink failed path={path} err={e}");
        }
    }
    let with_entries = VetPatrolReportService::get(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到剛更新的報告".into()))?;
    Ok(Json(with_entries))
}

/// 刪除巡場報告（soft delete）
pub async fn delete_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    // 置頂待辦的解除在 service 的同一個 tx 內完成（見 VetPatrolReportService::delete）。
    VetPatrolReportService::delete(&state.db, &actor, id).await?;
    Ok(Json(()))
}

/// 撤回巡場報告到草稿（送出後、未完成前；填報獸醫或 admin）。
/// service 內驗證 status（awaiting_acknowledgement / awaiting_follow_up）+ 權限
/// （created_by 或 admin），並 reset 工作流欄位 + 寫 RETRACTED audit。
pub async fn retract_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    // 置頂待辦的解除在 service 的同一個 tx 內完成
    // （見 VetPatrolReportService::retract_to_draft）。
    VetPatrolReportService::retract_to_draft(&state.db, &actor, id).await?;
    Ok(Json(()))
}

// ─────────────────────────────────────────────────────────────────
// 照片附件
// ─────────────────────────────────────────────────────────────────

/// 列出某份報告的照片
pub async fn list_vet_patrol_photos(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<Vec<VetPatrolPhoto>>> {
    access::require_vet_patrol_view(&current_user)?;
    let photos = VetPatrolReportService::list_photos(&state.db, report_id).await?;
    Ok(Json(photos))
}

/// R40-17：照片 multipart 解析共用 helper（report-level + entry-level 共用）。
/// 回傳 (file_name, content_type, data, caption)；caption 缺漏時為空字串。
async fn parse_photo_multipart(
    mut multipart: Multipart,
    category: FileCategory,
) -> Result<(String, String, Vec<u8>, String)> {
    let max_size = category.max_file_size();
    let mut file_bytes: Option<(String, String, Vec<u8>)> = None;
    let mut caption = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {e}")))?
    {
        match field.name() {
            Some("file") => {
                let file_name = field
                    .file_name()
                    .map(String::from)
                    .unwrap_or_else(|| "photo.jpg".to_string());
                let content_type = field
                    .content_type()
                    .map(String::from)
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                if !category
                    .allowed_mime_types()
                    .contains(&content_type.as_str())
                {
                    return Err(AppError::Validation(format!(
                        "不允許的檔案類型：{content_type}"
                    )));
                }
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("Failed to read file data: {e}")))?;
                if data.len() > max_size {
                    return Err(AppError::Validation(format!(
                        "檔案 '{file_name}' 超過上限 {} MB",
                        max_size / 1024 / 1024
                    )));
                }
                file_bytes = Some((file_name, content_type, data.to_vec()));
            }
            Some("caption") => {
                caption = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("Failed to read caption: {e}")))?;
            }
            _ => {}
        }
    }

    let (file_name, content_type, data) =
        file_bytes.ok_or_else(|| AppError::Validation("缺少 file 欄位".to_string()))?;
    Ok((file_name, content_type, data, caption))
}

/// R40-18：照片下載 Response builder 共用 helper
fn build_photo_download_response(
    data: Vec<u8>,
    file_name: &str,
    mime_type: &str,
) -> Result<Response> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(file_name),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}

/// 上傳照片（multipart：file 欄位 + 選填 caption 欄位）
pub async fn upload_vet_patrol_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(report_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<Json<VetPatrolPhoto>> {
    require_permission!(current_user, "animal.vet.recommend");

    // R75-12: 在讀 multipart / 寫磁碟前驗報告可寫（存在 + 未完成鎖定 + 限建立者/admin）
    VetPatrolReportService::ensure_report_photo_writable(&state.db, report_id, &current_user)
        .await?;

    let (file_name, content_type, data, caption) =
        parse_photo_multipart(multipart, FileCategory::AnimalPhoto).await?;

    let photo = VetPatrolReportService::upload_and_insert_photo(
        &state.db,
        report_id,
        current_user.id,
        &file_name,
        &content_type,
        &data,
        caption.trim(),
    )
    .await?;
    Ok(Json(photo))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePhotoCaptionRequest {
    pub caption: String,
}

/// 更新照片解說
pub async fn update_vet_patrol_photo_caption(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(photo_id): Path<Uuid>,
    Json(req): Json<UpdatePhotoCaptionRequest>,
) -> Result<Json<VetPatrolPhoto>> {
    require_permission!(current_user, "animal.vet.recommend");
    // R75-12: 限報告建立者 + 未鎖定
    VetPatrolReportService::ensure_report_photo_writable_by_photo(
        &state.db,
        photo_id,
        &current_user,
    )
    .await?;
    let photo =
        VetPatrolReportService::update_photo_caption(&state.db, photo_id, req.caption.trim())
            .await?;
    Ok(Json(photo))
}

/// 下載照片二進位
pub async fn download_vet_patrol_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(photo_id): Path<Uuid>,
) -> Result<Response> {
    access::require_vet_patrol_view(&current_user)?;
    let info = VetPatrolReportService::find_photo_for_download(&state.db, photo_id).await?;
    let (data, _) = FileService::read(&info.file_path).await?;
    build_photo_download_response(data, &info.file_name, &info.mime_type)
}

/// 刪除照片
pub async fn delete_vet_patrol_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(photo_id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "animal.vet.recommend");
    // R75-12: 限報告建立者 + 未鎖定
    VetPatrolReportService::ensure_report_photo_writable_by_photo(
        &state.db,
        photo_id,
        &current_user,
    )
    .await?;
    let file_path = VetPatrolReportService::delete_photo(&state.db, photo_id).await?;
    if let Err(unlink_err) = FileService::delete(&file_path).await {
        tracing::warn!("vet_patrol photo unlink 失敗 path={file_path} err={unlink_err}");
    }
    Ok(Json(()))
}

// ─────────────────────────────────────────────────────────────────
// R39: entry-level 照片附件
// ─────────────────────────────────────────────────────────────────

/// 列出某個 entry 的照片
pub async fn list_vet_patrol_entry_photos(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<Vec<VetPatrolEntryPhoto>>> {
    access::require_vet_patrol_view(&current_user)?;
    let photos = VetPatrolReportService::list_entry_photos(&state.db, entry_id).await?;
    Ok(Json(photos))
}

/// 上傳 entry 照片（multipart：file + caption）
pub async fn upload_vet_patrol_entry_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(entry_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<Json<VetPatrolEntryPhoto>> {
    require_permission!(current_user, "animal.vet.recommend");

    // R75-12: 驗 entry 父報告可寫（含 completed-lock + 限建立者/admin）
    VetPatrolReportService::ensure_entry_photo_writable(&state.db, entry_id, &current_user).await?;

    let (file_name, content_type, data, caption) =
        parse_photo_multipart(multipart, FileCategory::AnimalPhoto).await?;

    let photo = VetPatrolReportService::upload_and_insert_entry_photo(
        &state.db,
        entry_id,
        current_user.id,
        &file_name,
        &content_type,
        &data,
        caption.trim(),
    )
    .await?;
    Ok(Json(photo))
}

/// 更新 entry 照片解說
pub async fn update_vet_patrol_entry_photo_caption(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(photo_id): Path<Uuid>,
    Json(req): Json<UpdatePhotoCaptionRequest>,
) -> Result<Json<VetPatrolEntryPhoto>> {
    require_permission!(current_user, "animal.vet.recommend");
    // R75-12: 限報告建立者 + 未鎖定
    VetPatrolReportService::ensure_entry_photo_writable_by_photo(
        &state.db,
        photo_id,
        &current_user,
    )
    .await?;
    let photo =
        VetPatrolReportService::update_entry_photo_caption(&state.db, photo_id, req.caption.trim())
            .await?;
    Ok(Json(photo))
}

/// 下載 entry 照片
pub async fn download_vet_patrol_entry_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(photo_id): Path<Uuid>,
) -> Result<Response> {
    access::require_vet_patrol_view(&current_user)?;
    let info = VetPatrolReportService::find_entry_photo_for_download(&state.db, photo_id).await?;
    let (data, _) = FileService::read(&info.file_path).await?;
    build_photo_download_response(data, &info.file_name, &info.mime_type)
}

/// 刪除 entry 照片
pub async fn delete_vet_patrol_entry_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(photo_id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "animal.vet.recommend");
    // R75-12: 限報告建立者 + 未鎖定
    VetPatrolReportService::ensure_entry_photo_writable_by_photo(
        &state.db,
        photo_id,
        &current_user,
    )
    .await?;
    let actor = ActorContext::User(current_user.clone());
    let file_path = VetPatrolReportService::delete_entry_photo(&state.db, &actor, photo_id).await?;
    if let Err(unlink_err) = FileService::delete(&file_path).await {
        tracing::warn!("vet_patrol entry photo unlink 失敗 path={file_path} err={unlink_err}");
    }
    Ok(Json(()))
}
