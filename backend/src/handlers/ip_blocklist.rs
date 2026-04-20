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
    middleware::CurrentUser, require_permission, services::{IpBlocklistEntry, IpBlocklistService},
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
    require_permission!(current_user, "audit.logs.view");
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
    Ok(Json(serde_json::json!({ "id": id })))
}

pub async fn unblock_ip(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UnblockRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "audit.logs.view");
    IpBlocklistService::unblock(&state.db, id, current_user.id, &payload.reason).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
