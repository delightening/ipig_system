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

/// ??雿輻??”
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

/// ???芾???賊?
pub async fn get_unread_count(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UnreadNotificationCount>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let count = service.get_unread_count(current_user.id).await?;
    Ok(Json(UnreadNotificationCount { count }))
}

/// 璅???箏歇霈
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

/// 璅????箏歇霈
pub async fn mark_all_as_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.mark_all_as_read(current_user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// ?芷?
pub async fn delete_notification(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.delete_notification(current_user.id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// ???閮剖?
pub async fn get_notification_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<NotificationSettings>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let settings = service.get_settings(current_user.id).await?;
    Ok(Json(settings))
}

/// ?湔?閮剖?
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
// ?郎?賊?
// ============================================

/// ??雿澈摮?霅血?銵?
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

/// ?????郎?”
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
// 摰??梯”?賊?
// ============================================

/// ??摰??梯”?”
pub async fn list_scheduled_reports(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ScheduledReport>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let reports = service.list_scheduled_reports().await?;
    Ok(Json(reports))
}

/// ???桐?摰??梯”
pub async fn get_scheduled_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScheduledReport>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let report = service.get_scheduled_report(id).await?;
    Ok(Json(report))
}

/// 撱箇?摰??梯”
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

/// ?湔摰??梯”
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

/// ?芷摰??梯”
pub async fn delete_scheduled_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.delete_scheduled_report(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// ???梯”甇瑕閮?
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

/// 銝??梯”瑼?
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
// ??閫貊?瑼Ｘ
// ============================================

/// ??閫貊雿澈摮炎??
pub async fn trigger_low_stock_check(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    match crate::services::scheduler::SchedulerService::trigger_low_stock_check(&state.db, &state.config).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "雿澈摮炎?亙歇摰?"
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("瑼Ｘ憭望?: {}", e)
        }))),
    }
}

/// ??閫貊??瑼Ｘ
pub async fn trigger_expiry_check(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    match crate::services::scheduler::SchedulerService::trigger_expiry_check(&state.db, &state.config).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "message": "??瑼Ｘ撌脣???
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("瑼Ｘ憭望?: {}", e)
        }))),
    }
}

/// ??皜????
pub async fn trigger_notification_cleanup(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>, AppError> {
    let service = NotificationService::new(state.db.clone());
    
    match service.cleanup_old_notifications().await {
        Ok(deleted) => Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("撌脫???{} 蝑??", deleted)
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("皜?憭望?: {}", e)
        }))),
    }
}

