// Google Calendar Sync Handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        CalendarSyncConflict, CalendarSyncHistory, CalendarSyncStatus, ConflictQuery,
        ConflictWithDetails, ConnectCalendarRequest, EventSyncWithLeave,
        GoogleCalendarConfig, PaginatedResponse, ResolveConflictRequest, SyncHistoryQuery,
        UpdateCalendarConfigRequest,
    },
    services::CalendarService,
    AppState, Result,
};

// ============================================
// Config Handlers
// ============================================

/// 取得 Calendar 同步狀態
pub async fn get_calendar_status(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<CalendarSyncStatus>> {
    let status = CalendarService::get_sync_status(&state.db).await?;
    Ok(Json(status))
}

/// 取得 Calendar 設定
pub async fn get_calendar_config(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<GoogleCalendarConfig>> {
    let config = CalendarService::get_config(&state.db).await?;
    Ok(Json(config))
}

/// 連接 Google Calendar
pub async fn connect_calendar(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<ConnectCalendarRequest>,
) -> Result<Json<GoogleCalendarConfig>> {
    let config = CalendarService::connect(&state.db, &payload).await?;
    Ok(Json(config))
}

/// 斷開 Google Calendar
pub async fn disconnect_calendar(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<StatusCode> {
    CalendarService::disconnect(&state.db).await?;
    Ok(StatusCode::OK)
}

/// 更新 Calendar 設定
pub async fn update_calendar_config(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<UpdateCalendarConfigRequest>,
) -> Result<Json<GoogleCalendarConfig>> {
    let config = CalendarService::update_config(&state.db, &payload).await?;
    Ok(Json(config))
}

// ============================================
// Sync Handlers
// ============================================

/// 手動觸發同步
pub async fn trigger_sync(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<CalendarSyncHistory>> {
    let history = CalendarService::trigger_sync(&state.db, Some(current_user.id)).await?;
    Ok(Json(history))
}

/// 取得同步歷史
pub async fn list_sync_history(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<SyncHistoryQuery>,
) -> Result<Json<PaginatedResponse<CalendarSyncHistory>>> {
    let result = CalendarService::list_sync_history(&state.db, &params).await?;
    Ok(Json(result))
}

/// 取得待同步事件
pub async fn list_pending_syncs(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<EventSyncWithLeave>>> {
    let events = CalendarService::list_pending_syncs(&state.db).await?;
    Ok(Json(events))
}

// ============================================
// Conflict Handlers
// ============================================

/// 列出同步衝突
pub async fn list_conflicts(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<ConflictQuery>,
) -> Result<Json<PaginatedResponse<ConflictWithDetails>>> {
    let result = CalendarService::list_conflicts(&state.db, &params).await?;
    Ok(Json(result))
}

/// 取得衝突詳細
pub async fn get_conflict(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<CalendarSyncConflict>> {
    let conflict = CalendarService::get_conflict(&state.db, id).await?;
    Ok(Json(conflict))
}

/// 解決衝突
pub async fn resolve_conflict(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveConflictRequest>,
) -> Result<Json<CalendarSyncConflict>> {
    let conflict = CalendarService::resolve_conflict(
        &state.db,
        id,
        current_user.id,
        &payload.resolution,
        payload.notes.as_deref(),
    )
    .await?;
    Ok(Json(conflict))
}
