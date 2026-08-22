//! R24-1: IP 黑名單管理 API（admin 後台）
//!
//! GET   /admin/audit/ip-blocklist       — 列表（?only_active=bool&limit=&offset=）
//! POST  /admin/audit/ip-blocklist       — 手動新增
//! PATCH /admin/audit/ip-blocklist/:id/unblock — 解除封鎖

use std::net::IpAddr;

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, IpBlocklistEntry, IpBlocklistService,
    },
    AppError, AppState, Result,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub only_active: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddRequest {
    pub ip_address: String,
    pub reason: String,
    pub ttl_hours: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UnblockRequest {
    pub reason: String,
}

pub async fn list_ip_blocklist(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<IpBlocklistEntry>>> {
    require_permission!(current_user, "audit.logs.view");
    let only_active = q.only_active.unwrap_or(true);
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let rows = IpBlocklistService::list(&state.db, only_active, limit, offset).await?;
    Ok(Json(rows))
}

pub async fn add_ip_blocklist(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<AddRequest>,
) -> Result<Json<serde_json::Value>> {
    // R75-7（Gemini #754）：手動封鎖 IP 為變更操作，僅 audit.logs.view（讀權限，QAU 亦持有）
    // 時可封鎖任意 IP（含 admin/正常用戶）發動 DoS；與 unblock 一致收緊為 audit.alerts.manage。
    require_permission!(current_user, "audit.alerts.manage");
    let ip: IpAddr = payload
        .ip_address
        .parse()
        .map_err(|_| AppError::Validation(format!("無效的 IP 格式：{}", payload.ip_address)))?;
    let id = IpBlocklistService::manual_add(
        &state.db,
        ip,
        &payload.reason,
        payload.ttl_hours,
        current_user.id,
    )
    .await?;

    // CSO #4：IP 封鎖屬安全控制變更，須進 HMAC 稽核鏈（原本僅寫 ip_blocklist 表 blocked_by）
    let actor = ActorContext::User(current_user.clone());
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "SECURITY",
            event_type: "IP_BLOCKLIST_ADD",
            entity: Some(AuditEntity::new("ip_blocklist", id, &payload.ip_address)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("IP 封鎖稽核日誌寫入失敗: {e}");
    }

    Ok(Json(serde_json::json!({ "id": id })))
}

pub async fn unblock_ip(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UnblockRequest>,
) -> Result<Json<serde_json::Value>> {
    // R75-7: 解封 IP 為破壞性安全運維操作，須 audit.alerts.manage；原僅 audit.logs.view
    // （讀權限，QAU 亦持有）→ 品質稽核員可解封任意被封鎖 IP、弱化網路控制。
    require_permission!(current_user, "audit.alerts.manage");
    IpBlocklistService::unblock(&state.db, id, current_user.id, &payload.reason).await?;

    // CSO #4：解除封鎖同樣進 HMAC 稽核鏈
    let actor = ActorContext::User(current_user.clone());
    if let Err(e) = AuditService::log_activity_oneshot(
        &state.db,
        &actor,
        ActivityLogEntry {
            event_category: "SECURITY",
            event_type: "IP_BLOCKLIST_UNBLOCK",
            entity: Some(AuditEntity::new("ip_blocklist", id, &payload.reason)),
            data_diff: None,
            request_context: None,
        },
    )
    .await
    {
        tracing::error!("IP 解封稽核日誌寫入失敗: {e}");
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}
