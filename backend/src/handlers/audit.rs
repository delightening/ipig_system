// 擴展的審計 Handlers
// 包含：ActivityLogs, LoginEvents, Sessions, SecurityAlerts, Dashboard

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use chrono::Utc;
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
    require_permission,
    services::AuditService,
    AppError, AppState, Result,
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
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedResponse<UserActivityLog>>> {
    require_permission!(current_user, "audit.logs.view");
    let result = AuditService::list_activities(&state.db, &query).await?;
    Ok(Json(result))
}

/// 匯出活動日誌（不分頁，供 CSV/PDF 匯出使用）
pub async fn export_activity_logs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<Vec<UserActivityLog>>> {
    require_permission!(current_user, "audit.logs.export");
    let result = AuditService::export_activities(&state.db, &query).await?;
    Ok(Json(result))
}

/// 稽核日誌匯出 API（P1-M0）：合規稽核與外部系統整合
/// GET /admin/audit-logs/export?format=csv|json&from=&to=&user_id=&event_category=&entity_type=
#[derive(Debug, Deserialize)]
pub struct AuditLogExportParams {
    pub format: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "user_id")]
    pub user_id: Option<Uuid>,
    #[serde(rename = "event_category")]
    pub event_category: Option<String>,
    #[serde(rename = "entity_type")]
    pub entity_type: Option<String>,
}

pub async fn export_audit_logs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AuditLogExportParams>,
) -> Result<Response> {
    require_permission!(current_user, "audit.logs.export");

    let format = params.format.as_deref().unwrap_or("json");
    let from = params.from.as_ref().and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let to = params.to.as_ref().and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let from_str = params.from.as_deref().unwrap_or("all");
    let to_str = params.to.as_deref().unwrap_or("all");

    let query = ActivityLogQuery {
        user_id: params.user_id,
        event_category: params.event_category.clone(),
        event_type: None,
        entity_type: params.entity_type.clone(),
        entity_id: None,
        is_suspicious: None,
        from,
        to,
        page: None,
        per_page: None,
    };

    let logs = AuditService::export_activities(&state.db, &query).await?;

    let ext = if format == "csv" { "csv" } else { "json" };
    let filename = format!("audit_logs_{}_{}.{}", from_str, to_str, ext);

    match format {
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.write_record([
                "時間",
                "操作者",
                "操作者信箱",
                "類別",
                "事件類型",
                "實體類型",
                "實體名稱",
                "IP 位址",
                "可疑",
            ])
            .map_err(|e| AppError::Internal(format!("CSV write error: {}", e)))?;
            for log in &logs {
                let created_at = log.created_at.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S").to_string();
                let actor = log.actor_display_name.as_deref().unwrap_or("");
                let email = log.actor_email.as_deref().unwrap_or("");
                let entity_name = log.entity_display_name.as_deref().unwrap_or("");
                let suspicious = if log.is_suspicious { "Y" } else { "N" };
                wtr.write_record([
                    created_at.as_str(),
                    actor,
                    email,
                    log.event_category.as_str(),
                    log.event_type.as_str(),
                    log.entity_type.as_deref().unwrap_or(""),
                    entity_name,
                    log.ip_address.as_deref().unwrap_or(""),
                    suspicious,
                ])
                .map_err(|e| AppError::Internal(format!("CSV write error: {}", e)))?;
            }
            let csv_bytes = wtr.into_inner().map_err(|e| AppError::Internal(format!("CSV write error: {:?}", e)))?;
            let bom = "\u{FEFF}";
            let body = format!("{}{}", bom, String::from_utf8_lossy(&csv_bytes));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
        }
        "json" | _ => {
            let json_bytes = serde_json::to_vec(&logs).map_err(|e| AppError::Internal(format!("JSON serialize error: {}", e)))?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(Body::from(json_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
        }
    }
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

    // SEC: 敏感操作二級審計 — 強制登出
    let db = state.db.clone();
    let actor = current_user.id;
    let reason = payload.reason.clone();
    tokio::spawn(async move {
        let _ = AuditService::log_activity(
            &db, actor, "SECURITY", "FORCE_LOGOUT",
            Some("session"), Some(session_id), None,
            None, Some(serde_json::json!({ "session_id": session_id, "reason": reason })),
            None, None,
        ).await;
    });

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
