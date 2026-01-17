// 擴展的審計 Handlers
// 包含：ActivityLogs, LoginEvents, Sessions, SecurityAlerts, Dashboard

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        ActivityLogQuery, AuditDashboardStats, AuditLogQuery, AuditLogWithActor,
        ForceLogoutRequest, LoginEventQuery, LoginEventWithUser, PaginatedResponse,
        ResolveAlertRequest, SecurityAlert, SecurityAlertQuery, SessionQuery, SessionWithUser,
        UserActivityLog,
    },
    services::AuditService,
    AppState, Result,
};

// ============================================
// 原有的 Audit Logs (保持相容)
// ============================================

#[derive(Debug, Deserialize)]
pub struct AuditLogQueryParams {
    pub entity_type: Option<String>,
    pub action: Option<String>,
    pub user_id: Option<Uuid>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    pub page: Option<i64>,
    #[serde(rename = "perPage")]
    pub per_page: Option<i64>,
}

/// 列出審計日誌（原有功能）
pub async fn list_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQueryParams>,
) -> Result<Json<PaginatedResponse<AuditLogWithActor>>> {
    let query = AuditLogQuery {
        entity_type: params.entity_type,
        action: params.action,
        actor_user_id: params.user_id,
        entity_id: None,
        start_date: params.start_date.and_then(|s| s.parse().ok()),
        end_date: params.end_date.and_then(|s| s.parse().ok()),
    };
    let result = AuditService::list(&state.db, &query).await?;
    let len = result.len() as i64;
    Ok(Json(PaginatedResponse::new(result, len, params.page.unwrap_or(1), params.per_page.unwrap_or(50))))
}

/// 取得實體的變更歷史
pub async fn get_entity_history(
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<AuditLogWithActor>>> {
    let result = AuditService::get_entity_history(&state.db, &entity_type, entity_id).await?;
    Ok(Json(result))
}

// ============================================
// 新增的 Activity Logs
// ============================================

/// 列出活動日誌
pub async fn list_activity_logs(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedResponse<UserActivityLog>>> {
    let result = AuditService::list_activities(&state.db, &query).await?;
    Ok(Json(result))
}

/// 取得使用者活動時間線
pub async fn get_user_activity_timeline(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedResponse<UserActivityLog>>> {
    let mut q = query;
    q.user_id = Some(user_id);
    let result = AuditService::list_activities(&state.db, &q).await?;
    Ok(Json(result))
}

// ============================================
// Login Events
// ============================================

/// 列出登入事件
pub async fn list_login_events(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<LoginEventQuery>,
) -> Result<Json<PaginatedResponse<LoginEventWithUser>>> {
    let result = AuditService::list_login_events(&state.db, &query).await?;
    Ok(Json(result))
}

// ============================================
// Sessions
// ============================================

/// 列出活躍 Sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<SessionQuery>,
) -> Result<Json<PaginatedResponse<SessionWithUser>>> {
    let result = AuditService::list_sessions(&state.db, &query).await?;
    Ok(Json(result))
}

/// 強制登出 Session
pub async fn force_logout_session(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<ForceLogoutRequest>,
) -> Result<Json<serde_json::Value>> {
    AuditService::force_logout_session(
        &state.db,
        session_id,
        current_user.id,
        payload.reason.as_deref(),
    )
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已強制登出該 Session"
    })))
}

// ============================================
// Security Alerts
// ============================================

/// 列出安全警報
pub async fn list_security_alerts(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<SecurityAlertQuery>,
) -> Result<Json<PaginatedResponse<SecurityAlert>>> {
    let result = AuditService::list_security_alerts(&state.db, &query).await?;
    Ok(Json(result))
}

/// 取得安全警報詳細
#[allow(dead_code)]
pub async fn get_security_alert(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<SecurityAlert>> {
    let alert = AuditService::get_security_alert(&state.db, id).await?;
    Ok(Json(alert))
}

/// 解決安全警報
pub async fn resolve_security_alert(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveAlertRequest>,
) -> Result<Json<SecurityAlert>> {
    let alert = AuditService::resolve_alert(
        &state.db,
        id,
        current_user.id,
        payload.resolution_notes.as_deref(),
    )
    .await?;

    Ok(Json(alert))
}

// ============================================
// Dashboard
// ============================================

/// 取得審計儀表板統計
pub async fn get_audit_dashboard(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<AuditDashboardStats>> {
    let stats = AuditService::get_dashboard_stats(&state.db).await?;
    Ok(Json(stats))
}
