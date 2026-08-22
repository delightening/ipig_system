// 擴展的審計 Handlers
// 包含：ActivityLogs, LoginEvents, Sessions, SecurityAlerts, Dashboard

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use chrono::{FixedOffset, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    constants::SEC_EVENT_ACCOUNT_LOCKOUT_CLEAR,
    middleware::{ActorContext, CurrentUser},
    models::{
        ActivityLogQuery, AlertLockStatus, AuditDashboardStats, AuditLogQuery, AuditLogWithActor,
        BulkResolveAlertsRequest, ClearAccountLockoutRequest, ForceLogoutRequest, LoginEventQuery,
        LoginEventWithUser, PaginatedResponse, ResolveAlertRequest, SecurityAlert,
        SecurityAlertQuery, SessionQuery, SessionWithUser, UserActivityLog,
    },
    require_permission,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AlertLockService, AuditService, AuthService,
    },
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
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<AuditLogQueryParams>,
) -> Result<Json<PaginatedResponse<AuditLogWithActor>>> {
    require_permission!(current_user, "audit.logs.view");
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
    Ok(Json(PaginatedResponse::new(
        result,
        len,
        params.page.unwrap_or(1),
        params.per_page.unwrap_or(50),
    )))
}

/// 取得實體的變更歷史
pub async fn get_entity_history(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<AuditLogWithActor>>> {
    require_permission!(current_user, "audit.logs.view");
    let result = AuditService::get_entity_history(&state.db, &entity_type, entity_id).await?;
    Ok(Json(result))
}

// ============================================
// 新增的 Activity Logs
// ============================================

/// 列出活動日誌
#[utoipa::path(get, path = "/api/v1/admin/audit/activities", responses((status = 200)), tag = "稽核管理", security(("bearer" = [])))]
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

/// R32-A8i：匯出活動日誌為 PDF（docxtpl + Word COM / Gotenberg LibreOffice）。
///
/// 取代 frontend client-side HTML + `window.print()` 的 legacy PDF 路徑，與其他
/// R32 報表（warehouse / blood_test / surgery / medical_record / review_*）一致
/// 走 docx 路徑，列印與下載品質一致。
pub async fn export_activity_logs_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Response> {
    require_permission!(current_user, "audit.logs.export");
    let logs = AuditService::export_activities(&state.db, &query).await?;
    let payload = build_audit_log_payload(&state, &current_user, &query, &logs).await?;
    let (pdf_bytes, renderer) = state
        .pdf_service
        .render_audit_log_from_export_data(
            &payload,
            crate::services::pdf_service_client::DocxRenderFormat::Pdf,
        )
        .await?;

    let p_from = query
        .from
        .map(|d| d.format("%Y%m%d").to_string())
        .unwrap_or_else(|| "all".into());
    let p_to = query
        .to
        .map(|d| d.format("%Y%m%d").to_string())
        .unwrap_or_else(|| "all".into());
    let filename = format!("操作日誌_{}_{}.pdf", p_from, p_to);

    crate::utils::http::file_response_with_renderer(
        pdf_bytes,
        "application/pdf",
        &filename,
        false,
        renderer,
    )
    .map_err(AppError::Internal)
}

/// R32-A8i: 把 logs[] + meta + summary 組成 audit_log payload（對應
/// pdf-service `schemas/audit_log.py::AuditLogPayload`）。
async fn build_audit_log_payload(
    state: &AppState,
    current_user: &CurrentUser,
    query: &ActivityLogQuery,
    logs: &[UserActivityLog],
) -> Result<serde_json::Value> {
    let exporter_name =
        crate::repositories::user::find_user_display_name_by_id(&state.db, current_user.id)
            .await?
            .unwrap_or_else(|| current_user.email.clone());

    let now = crate::time::now_taiwan()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let period_from = query.from.map(|d| d.to_string()).unwrap_or_default();
    let period_to = query.to.map(|d| d.to_string()).unwrap_or_default();

    let total_count = logs.len() as i64;
    // 單次 fold：admin / failure / unique-users 三個計數一次走完（per Gemini PR #346）
    let (admin_count, failure_count, user_ids) = logs.iter().fold(
        (0i64, 0i64, std::collections::HashSet::new()),
        |(mut a, mut f, mut u), l| {
            if l.event_category == "ADMIN" || l.event_category == "SYSTEM" {
                a += 1;
            }
            if l.event_type.contains("FAILED")
                || l.event_type.contains("DENIED")
                || l.event_severity == "WARNING"
                || l.event_severity == "ERROR"
            {
                f += 1;
            }
            if let Some(uid) = l.actor_user_id {
                u.insert(uid);
            }
            (a, f, u)
        },
    );
    let user_count = user_ids.len() as i64;

    let entries: Vec<serde_json::Value> = logs
        .iter()
        .map(|l| {
            let resource = match (&l.entity_type, &l.entity_display_name) {
                (Some(t), Some(n)) => format!("{}: {}", t, n),
                (Some(t), None) => t.clone(),
                _ => String::new(),
            };
            let change_summary = l
                .changed_fields
                .as_ref()
                .map(|fs| fs.join(", "))
                .unwrap_or_default();
            serde_json::json!({
                "timestamp": l.created_at.with_timezone(&crate::time::taiwan_offset()).format("%Y-%m-%d %H:%M:%S").to_string(),
                "user": l.actor_display_name.clone().unwrap_or_else(|| l.actor_email.clone().unwrap_or_default()),
                "action": l.event_type,
                "resource": resource,
                "ip": l.ip_address.clone().unwrap_or_default(),
                "change_summary": change_summary,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "meta": {
            "system_name": "豬博士動物實驗管理系統",
            "period_from": period_from,
            "period_to": period_to,
            "exported_by": exporter_name,
            "export_time": now,
        },
        "summary": {
            "total_count": total_count,
            "user_count": user_count,
            "admin_count": admin_count,
            "failure_count": failure_count,
        },
        "entries": entries,
        "signature": {
            "admin_name": "",
            "admin_signature": "",
        },
    }))
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
    let from = params
        .from
        .as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let to = params
        .to
        .as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let from_str = params.from.as_deref().unwrap_or("all");
    let to_str = params.to.as_deref().unwrap_or("all");

    let query = ActivityLogQuery {
        user_id: params.user_id,
        event_category: params.event_category.clone(),
        event_type: None,
        entity_type: params.entity_type.clone(),
        entity_id: None,
        is_suspicious: None,
        query: None,
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
            // R30-11: CSV 時間欄位改為 GMT+8 (Asia/Taipei) 顯示
            // FixedOffset::east_opt(8 * 3600) 在程式啟動即固定，unwrap 為靜態安全（編譯期常數）
            let tz_taipei =
                FixedOffset::east_opt(8 * 3600).expect("FixedOffset(+08:00) is always valid");
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.write_record([
                "時間 (Asia/Taipei)",
                "操作者",
                "操作者信箱",
                "類別",
                "事件類型",
                "實體類型",
                "實體名稱",
                "IP 位址",
                "可疑",
                "變更欄位",
                "變更前",
                "變更後",
            ])
            .map_err(|e| AppError::Internal(format!("CSV write error: {}", e)))?;
            for log in &logs {
                let created_at = log
                    .created_at
                    .with_timezone(&tz_taipei)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                let actor = log.actor_display_name.as_deref().unwrap_or("");
                let email = log.actor_email.as_deref().unwrap_or("");
                let entity_name = log.entity_display_name.as_deref().unwrap_or("");
                let suspicious = if log.is_suspicious { "Y" } else { "N" };

                // R30-12: 補 changed_fields / before_data / after_data 三欄
                // before_data / after_data 已在 service 層套用 redact，CSV 端直接序列化。
                // 採 single-line JSON（CSV cell 內換行不友善），csv crate 會自動 quote 含逗號 / 引號的字串。
                let changed_fields = log
                    .changed_fields
                    .as_ref()
                    .map(|v| v.join("; "))
                    .unwrap_or_default();
                let before_str = log
                    .before_data
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap_or_default())
                    .unwrap_or_default();
                let after_str = log
                    .after_data
                    .as_ref()
                    .map(|v| serde_json::to_string(v).unwrap_or_default())
                    .unwrap_or_default();

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
                    changed_fields.as_str(),
                    before_str.as_str(),
                    after_str.as_str(),
                ])
                .map_err(|e| AppError::Internal(format!("CSV write error: {}", e)))?;
            }
            let csv_bytes = wtr
                .into_inner()
                .map_err(|e| AppError::Internal(format!("CSV write error: {:?}", e)))?;
            let bom = "\u{FEFF}";
            let body = format!("{}{}", bom, String::from_utf8_lossy(&csv_bytes));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
                .header(
                    header::CONTENT_DISPOSITION,
                    crate::utils::http::content_disposition_header(&filename),
                )
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?)
        }
        "json" => {
            let json_bytes = serde_json::to_vec(&logs)
                .map_err(|e| AppError::Internal(format!("JSON serialize error: {}", e)))?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
                .header(
                    header::CONTENT_DISPOSITION,
                    crate::utils::http::content_disposition_header(&filename),
                )
                .body(Body::from(json_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?)
        }
        _ => {
            let json_bytes = serde_json::to_vec(&logs)
                .map_err(|e| AppError::Internal(format!("JSON serialize error: {}", e)))?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
                .header(
                    header::CONTENT_DISPOSITION,
                    crate::utils::http::content_disposition_header(&filename),
                )
                .body(Body::from(json_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?)
        }
    }
}

/// 取得使用者活動時間線
pub async fn get_user_activity_timeline(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedResponse<UserActivityLog>>> {
    require_permission!(current_user, "audit.logs.view");
    let mut q = query;
    q.user_id = Some(user_id);
    let result = AuditService::list_activities(&state.db, &q).await?;
    Ok(Json(result))
}

// ============================================
// Login Events
// ============================================

/// 列出登入事件
#[utoipa::path(get, path = "/api/v1/admin/audit/logins", responses((status = 200)), tag = "稽核管理", security(("bearer" = [])))]
pub async fn list_login_events(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<LoginEventQuery>,
) -> Result<Json<PaginatedResponse<LoginEventWithUser>>> {
    require_permission!(current_user, "audit.logs.view");
    let result = AuditService::list_login_events(&state.db, &query).await?;
    Ok(Json(result))
}

// ============================================
// Sessions
// ============================================

/// 列出活躍 Sessions
pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<SessionQuery>,
) -> Result<Json<PaginatedResponse<SessionWithUser>>> {
    require_permission!(current_user, "audit.logs.view");
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
    // R75-7: 強制登出為破壞性安全運維操作，須 audit.alerts.manage（與 resolve_security_alert 一致）；
    // 原僅 audit.logs.view（讀權限，QAU 亦持有）→ 品質稽核員可踢任意 session。
    require_permission!(current_user, "audit.alerts.manage");
    AuditService::force_logout_session(
        &state.db,
        session_id,
        current_user.id,
        payload.reason.as_deref(),
    )
    .await?;

    // SEC: 敏感操作二級審計 — 強制登出
    let db = state.db.clone();
    let spawn_actor = ActorContext::User(current_user.clone());
    let display = payload
        .reason
        .clone()
        .unwrap_or_else(|| format!("session {}", session_id));
    tokio::spawn(async move {
        if let Err(e) = AuditService::log_activity_oneshot(
            &db,
            &spawn_actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "FORCE_LOGOUT",
                entity: Some(AuditEntity::new("session", session_id, &display)),
                data_diff: None,
                request_context: None,
            },
        )
        .await
        {
            tracing::error!("審計日誌寫入失敗: {e}");
        }
    });

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已強制登出該 Session"
    })))
}

// ============================================
// Security Alerts
// ============================================

#[derive(Debug, Deserialize)]
pub struct RecentAlertsParams {
    /// ISO 8601 timestamp，回傳此時間之後的警報
    pub after: Option<chrono::DateTime<Utc>>,
}

/// 查詢指定時間之後的新安全警報（polling 端點）
pub async fn get_recent_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<RecentAlertsParams>,
) -> Result<Json<Vec<SecurityAlert>>> {
    require_permission!(current_user, "audit.logs.view");
    let after = params
        .after
        .unwrap_or_else(|| Utc::now() - chrono::Duration::seconds(60));
    let alerts = AuditService::find_recent_alerts(&state.db, after).await?;
    Ok(Json(alerts))
}

/// 列出安全警報
pub async fn list_security_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<SecurityAlertQuery>,
) -> Result<Json<PaginatedResponse<SecurityAlert>>> {
    require_permission!(current_user, "audit.logs.view");
    let result = AuditService::list_security_alerts(&state.db, &query).await?;
    Ok(Json(result))
}

/// 解決安全警報
pub async fn resolve_security_alert(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveAlertRequest>,
) -> Result<Json<SecurityAlert>> {
    require_permission!(current_user, "audit.alerts.manage");
    let alert = AuditService::resolve_alert(
        &state.db,
        id,
        current_user.id,
        payload.resolution_notes.as_deref(),
    )
    .await?;

    Ok(Json(alert))
}

/// 批次解決安全警報
pub async fn bulk_resolve_security_alerts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<BulkResolveAlertsRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "audit.alerts.manage");
    if payload.ids.is_empty() {
        return Err(AppError::BadRequest("ids 不可為空".to_string()));
    }
    let count = AuditService::bulk_resolve_alerts(
        &state.db,
        &payload.ids,
        current_user.id,
        payload.resolution_notes.as_deref(),
    )
    .await?;

    Ok(Json(serde_json::json!({ "resolved_count": count })))
}

/// 查警報對象目前的鎖定狀態（帳號鎖定 / IP 封鎖）
pub async fn get_alert_lock_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AlertLockStatus>> {
    require_permission!(current_user, "audit.logs.view");
    let status = AlertLockService::status(&state.db, &state.config, id).await?;
    Ok(Json(status))
}

/// 解除帳號鎖定
///
/// 與 `resolve_security_alert` 刻意分開：關掉警報是「我看過了」的稽核動作，
/// 放行帳號是安全控制變更。真的遇到爆破攻擊時，關警報不該順手把門打開。
pub async fn clear_account_lockout(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<ClearAccountLockoutRequest>,
) -> Result<Json<serde_json::Value>> {
    // 與 unblock_ip 同級：破壞性安全運維操作，不可只憑讀權限
    require_permission!(current_user, "audit.alerts.manage");
    if payload.reason.trim().is_empty() {
        return Err(AppError::BadRequest("解鎖原因不可為空".to_string()));
    }

    let user_id = AuthService::clear_account_lockout(&state.db, &payload.email).await?;

    // 對齊 IP 解封：安全控制變更須進 HMAC 稽核鏈
    let actor = ActorContext::User(current_user.clone());
    let entity_display = format!("{} — {}", payload.email, payload.reason);
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "SECURITY",
            event_type: SEC_EVENT_ACCOUNT_LOCKOUT_CLEAR,
            entity: Some(AuditEntity::new("users", user_id, &entity_display)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("帳號解鎖稽核日誌寫入失敗: {e}");
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ============================================
// Dashboard
// ============================================

/// 取得審計儀表板統計
#[utoipa::path(get, path = "/api/v1/admin/audit/dashboard", responses((status = 200)), tag = "稽核管理", security(("bearer" = [])))]
pub async fn get_audit_dashboard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<AuditDashboardStats>> {
    require_permission!(current_user, "audit.logs.view");
    let stats = AuditService::get_dashboard_stats(&state.db).await?;
    Ok(Json(stats))
}

// ============================================
// R22-17: Security Events
// ============================================

/// R22-17: 列出安全事件（event_category = 'SECURITY' 的 user_activity_logs）
pub async fn list_security_events(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<Json<PaginatedResponse<UserActivityLog>>> {
    require_permission!(current_user, "audit.logs.view");

    // 強制過濾 event_category = SECURITY
    let mut security_query = query;
    security_query.event_category = Some("SECURITY".to_string());

    let result = AuditService::list_activities(&state.db, &security_query).await?;
    Ok(Json(result))
}
