use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CreateScheduledReportRequest, ExpiryAlert, LowStockAlert, MarkNotificationsReadRequest,
        NotificationItem, NotificationQuery, NotificationSettings, PaginatedResponse,
        PaginationQuery, ReportHistory, ScheduledReport, UnreadNotificationCount,
        UpdateNotificationSettingsRequest, UpdateScheduledReportRequest,
    },
    repositories::notification as notification_repo,
    require_permission,
    services::NotificationService,
    AppState,
};

/// IDOR 防護：檢查排程報表擁有權（建立者或管理員可存取）
async fn check_scheduled_report_access(
    db: &sqlx::PgPool,
    report_id: Uuid,
    current_user: &CurrentUser,
) -> Result<(), AppError> {
    if current_user.is_admin() {
        return Ok(());
    }
    let owner = notification_repo::find_scheduled_report_owner(db, report_id).await?;

    match owner {
        Some(created_by) if created_by == current_user.id => Ok(()),
        Some(_) => Err(AppError::Forbidden("無權存取此排程報表".into())),
        None => Err(AppError::NotFound("找不到排程報表".into())),
    }
}

/// 列出所有通知
#[utoipa::path(get, path = "/api/v1/notifications", responses((status = 200)), tag = "通知", security(("bearer" = [])))]
pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<PaginatedResponse<NotificationItem>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_notifications(
            current_user.id,
            &query,
            pagination.page,
            pagination.per_page,
        )
        .await?;
    Ok(Json(result))
}

/// 取得未讀通知數量
#[utoipa::path(get, path = "/api/v1/notifications/unread-count", responses((status = 200)), tag = "通知", security(("bearer" = [])))]
pub async fn get_unread_count(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UnreadNotificationCount>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let count = service.get_unread_count(current_user.id).await?;
    Ok(Json(UnreadNotificationCount { count }))
}

/// 取得待處理（待辦）數量 —— 驚嘆號入口的紅點。
///
/// 與 [`get_unread_count`] 分開：鈴鐺數「未讀通知」、驚嘆號數「未完成待辦」。
/// 兩者互斥（`get_unread_count` 已排除待辦），同一件事不會在畫面上被數兩次。
#[utoipa::path(get, path = "/api/v1/notifications/action-required-count", responses((status = 200)), tag = "通知", security(("bearer" = [])))]
pub async fn get_action_required_count(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UnreadNotificationCount>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let count = service.get_action_required_count(current_user.id).await?;
    Ok(Json(UnreadNotificationCount { count }))
}

/// 標記通知為已讀
pub async fn mark_as_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<MarkNotificationsReadRequest>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service
        .mark_as_read(current_user.id, &request.notification_ids)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 標記所有通知為已讀
pub async fn mark_all_as_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.mark_all_as_read(current_user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 刪除通知
pub async fn delete_notification(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.delete_notification(current_user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 取得通知設定
pub async fn get_notification_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<NotificationSettings>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let settings = service.get_settings(current_user.id).await?;
    Ok(Json(settings))
}

/// 更新通知設定
pub async fn update_notification_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<UpdateNotificationSettingsRequest>,
) -> Result<Json<NotificationSettings>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let settings = service.update_settings(current_user.id, request).await?;
    Ok(Json(settings))
}

// ============================================
// 庫存警示
// ============================================

/// 列出所有低庫存警示
pub async fn list_low_stock_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<LowStockAlert>>, AppError> {
    // F1（pentest 2026-07-05）：庫存告警屬內部營運資料，須具庫存檢視權；
    // 修補前無 gate → 外部 CLIENT 等任何已認證者可讀全院缺貨。
    require_permission!(current_user, "erp.stock.view");
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_low_stock_alerts(pagination.page, pagination.per_page)
        .await?;
    Ok(Json(result))
}

/// R35-17: list_expiry_alerts 的 query — 在 PaginationQuery 之上加可選 within_days。
#[derive(Debug, serde::Deserialize)]
pub struct ExpiryAlertsQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    /// R35-17: 只回 `days_until_expiry <= within_days` 的項目（含已過期負值）。
    /// Dashboard widget 傳 7；省略則回全部。
    pub within_days: Option<i32>,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    20
}

/// 列出所有過期警示
///
/// R35-17: 加 `?within_days=N` query — Dashboard widget 用 `within_days=7` 取「7 天內到期」。
pub async fn list_expiry_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(q): Query<ExpiryAlertsQuery>,
) -> Result<Json<PaginatedResponse<ExpiryAlert>>, AppError> {
    // F1（pentest 2026-07-05）：效期告警屬內部庫存資料，須具庫存檢視權。
    require_permission!(current_user, "erp.stock.view");
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_expiry_alerts(q.page, q.per_page, q.within_days)
        .await?;
    Ok(Json(result))
}

// ============================================
// 排程報表
// ============================================

/// 列出排程報表（管理員看全部，一般使用者只看自己的）。
/// 使用者範圍下推到 service/SQL，handler 不做資料過濾。
pub async fn list_scheduled_reports(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ScheduledReport>>, AppError> {
    let owner_filter = if current_user.is_admin() {
        None
    } else {
        Some(current_user.id)
    };
    let service = NotificationService::new(state.db.clone());
    let reports = service.list_scheduled_reports(owner_filter).await?;
    Ok(Json(reports))
}

/// 取得單個排程報表
pub async fn get_scheduled_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScheduledReport>, AppError> {
    check_scheduled_report_access(&state.db, id, &current_user).await?;
    let service = NotificationService::new(state.db.clone());
    let report = service.get_scheduled_report(id).await?;
    Ok(Json(report))
}

/// 建立排程報表
pub async fn create_scheduled_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateScheduledReportRequest>,
) -> Result<(StatusCode, Json<ScheduledReport>), AppError> {
    let service = NotificationService::new(state.db.clone());
    let report = service
        .create_scheduled_report(request, current_user.id)
        .await?;
    Ok((StatusCode::CREATED, Json(report)))
}

/// 更新排程報表
pub async fn update_scheduled_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateScheduledReportRequest>,
) -> Result<Json<ScheduledReport>, AppError> {
    check_scheduled_report_access(&state.db, id, &current_user).await?;
    let service = NotificationService::new(state.db.clone());
    let report = service.update_scheduled_report(id, request).await?;
    Ok(Json(report))
}

/// 刪除排程報表
pub async fn delete_scheduled_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    check_scheduled_report_access(&state.db, id, &current_user).await?;
    let service = NotificationService::new(state.db.clone());
    service.delete_scheduled_report(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 列出報表歷史記錄（僅限管理員）
pub async fn list_report_history(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ReportHistory>>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅限管理員查看報表歷史".into()));
    }
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_report_history(pagination.page, pagination.per_page)
        .await?;
    Ok(Json(result))
}

/// 下載報表檔案
pub async fn download_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReportHistory>, AppError> {
    // SEC-IDOR: 報表下載需要 IDOR 保護（建立者或 admin 可存取）
    check_scheduled_report_access(&state.db, id, &current_user).await?;
    let service = NotificationService::new(state.db.clone());
    let report = service.get_report_history(id).await?;
    Ok(Json(report))
}

// ============================================
// 手動觸發通知檢查
// ============================================

/// 手動觸發低庫存檢查
pub async fn trigger_low_stock_check(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可手動觸發系統檢查".into()));
    }
    match crate::services::scheduler::SchedulerService::trigger_low_stock_check(
        &state.db,
        &state.config,
    )
    .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "低庫存檢查已成功執行"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("檢查失敗: {}", e)
        }))),
    }
}

/// 手動觸發過期檢查
pub async fn trigger_expiry_check(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可手動觸發系統檢查".into()));
    }
    match crate::services::scheduler::SchedulerService::trigger_expiry_check(
        &state.db,
        &state.config,
    )
    .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "過期檢查已成功執行"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("檢查失敗: {}", e)
        }))),
    }
}

/// 手動清理舊通知
pub async fn trigger_notification_cleanup(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可手動觸發系統檢查".into()));
    }
    let service = NotificationService::new(state.db.clone());

    match service.cleanup_old_notifications().await {
        Ok(deleted) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("已刪除 {} 筆舊通知", deleted)
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("清理失敗: {}", e)
        }))),
    }
}

/// 手動觸發採購單未入庫檢查
pub async fn trigger_po_pending_receipt_check(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !current_user.is_admin() {
        return Err(AppError::Forbidden("僅管理員可手動觸發系統檢查".into()));
    }
    match crate::services::scheduler::SchedulerService::trigger_po_pending_receipt_check(&state.db)
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "採購單未入庫檢查已成功執行"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("檢查失敗: {}", e)
        }))),
    }
}
