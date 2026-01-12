use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        NotificationQuery, UpdateNotificationSettingsRequest, MarkNotificationsReadRequest,
        PaginationQuery, PaginatedResponse, NotificationItem, NotificationSettings,
        UnreadNotificationCount, LowStockAlert, ExpiryAlert,
        ScheduledReport, CreateScheduledReportRequest, UpdateScheduledReportRequest,
        ReportHistory,
    },
    services::NotificationService,
    AppState,
};

/// 列出所有通知
pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<PaginatedResponse<NotificationItem>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_notifications(current_user.id, &query, pagination.page, pagination.per_page)
        .await?;
    Ok(Json(result))
}

/// 取得未讀通知數量
pub async fn get_unread_count(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UnreadNotificationCount>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let count = service.get_unread_count(current_user.id).await?;
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
    Extension(_current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<LowStockAlert>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_low_stock_alerts(pagination.page, pagination.per_page)
        .await?;
    Ok(Json(result))
}

/// 列出所有過期警示
pub async fn list_expiry_alerts(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ExpiryAlert>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_expiry_alerts(pagination.page, pagination.per_page)
        .await?;
    Ok(Json(result))
}

// ============================================
// 排程報表
// ============================================

/// 列出所有排程報表
pub async fn list_scheduled_reports(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ScheduledReport>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let reports = service.list_scheduled_reports().await?;
    Ok(Json(reports))
}

/// 取得單個排程報表
pub async fn get_scheduled_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScheduledReport>, AppError> {
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
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateScheduledReportRequest>,
) -> Result<Json<ScheduledReport>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let report = service.update_scheduled_report(id, request).await?;
    Ok(Json(report))
}

/// 刪除排程報表
pub async fn delete_scheduled_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.delete_scheduled_report(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 列出報表歷史記錄
pub async fn list_report_history(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ReportHistory>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let result = service
        .list_report_history(pagination.page, pagination.per_page)
        .await?;
    Ok(Json(result))
}

/// 下載報表檔案
pub async fn download_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReportHistory>, AppError> {
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
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    match crate::services::scheduler::SchedulerService::trigger_low_stock_check(&state.db, &state.config).await {
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
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    match crate::services::scheduler::SchedulerService::trigger_expiry_check(&state.db, &state.config).await {
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
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
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
