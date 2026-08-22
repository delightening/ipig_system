//! R30-27c：桌機 ↔ 手機簽名 bridge HTTP endpoints
//!
//! 三個端點：
//! - `POST /signing-bridge/start` (auth required) — 桌機開 session
//! - `GET  /signing-bridge/:id/status` (auth required, owner-only) — 桌機輪詢
//! - `POST /signing-bridge/:id/submit` (**公開**，token-bearer 驗證) — 手機提交
//! - `GET  /signing-bridge/:id/consume` (auth required, owner-only) — 桌機取走 payload
//!
//! Routes 掛載在 `routes/user.rs`（authenticated routes，submit 例外手動掛 public）。
//! 注意：submit 需從 protected 路徑樹移出，加在 public_routes，由本檔的 wrapper 處理。

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{middleware::CurrentUser, services::SignatureBridgeService, AppState, Result};

#[derive(Deserialize, ToSchema)]
pub struct StartBridgeRequest {
    /// purpose 字串：role.create / role.update / role.delete（audit 用）
    pub purpose: String,
}

#[derive(Serialize, ToSchema)]
pub struct StartBridgeResponse {
    pub session_id: Uuid,
    /// plaintext mobile_token — 只此一次回給桌機，桌機編入 QR
    pub mobile_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    pub status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SubmitBridgeRequest {
    /// 手機從 QR query 拿到的 mobile_token
    pub mobile_token: String,
    /// 簽章 payload（同 MutationSignaturePayload 格式：password / handwriting_svg / stroke_data）
    pub payload: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct ConsumeResponse {
    pub payload: serde_json::Value,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

/// POST /signing-bridge/start — 桌機開 session（已登入）
#[utoipa::path(
    post,
    path = "/api/v1/signing-bridge/start",
    request_body = StartBridgeRequest,
    responses((status = 200, body = StartBridgeResponse)),
    tag = "簽章 Bridge",
    security(("bearer" = []))
)]
pub async fn start_bridge(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<StartBridgeRequest>,
) -> Result<Json<StartBridgeResponse>> {
    let s = SignatureBridgeService::start(&state.db, current_user.id, &req.purpose).await?;
    Ok(Json(StartBridgeResponse {
        session_id: s.session_id,
        mobile_token: s.mobile_token,
        expires_at: s.expires_at,
    }))
}

/// GET /signing-bridge/:id/status — 桌機輪詢（已登入，owner-only）
#[utoipa::path(
    get,
    path = "/api/v1/signing-bridge/{id}/status",
    params(("id" = Uuid, Path, description = "session id")),
    responses((status = 200, body = StatusResponse)),
    tag = "簽章 Bridge",
    security(("bearer" = []))
)]
pub async fn get_bridge_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusResponse>> {
    let status = SignatureBridgeService::get_status(&state.db, id, current_user.id).await?;
    Ok(Json(StatusResponse { status }))
}

/// GET /signing-bridge/:id/consume — 桌機取走 payload（status COMPLETED → CONSUMED）
#[utoipa::path(
    get,
    path = "/api/v1/signing-bridge/{id}/consume",
    params(("id" = Uuid, Path, description = "session id")),
    responses((status = 200, body = ConsumeResponse)),
    tag = "簽章 Bridge",
    security(("bearer" = []))
)]
pub async fn consume_bridge(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConsumeResponse>> {
    let p = SignatureBridgeService::consume(&state.db, &state.config, id, current_user.id).await?;
    Ok(Json(ConsumeResponse {
        payload: p.payload,
        submitted_at: p.submitted_at,
    }))
}

/// POST /signing-bridge/:id/submit — 手機提交（**公開**，token 驗證）
#[utoipa::path(
    post,
    path = "/api/v1/public/signing-bridge/{id}/submit",
    params(("id" = Uuid, Path, description = "session id")),
    request_body = SubmitBridgeRequest,
    responses((status = 200, description = "submitted")),
    tag = "簽章 Bridge"
)]
pub async fn submit_bridge_public(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<SubmitBridgeRequest>,
) -> Result<Json<serde_json::Value>> {
    SignatureBridgeService::submit(&state.db, &state.config, id, &req.mobile_token, req.payload)
        .await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
