//! PI 代理授權 handlers（migration 010）。
//!
//! - `authorize_pi_delegate` / `revoke_pi_delegate`：核准/撤銷代理人，授權規則見
//!   `services/protocol/pi_delegate.rs` 檔案頂端說明（僅現任 SD 可核准他人；SD 指定
//!   自己須改由執行秘書/admin 核准；撤銷則現任 SD 或執行秘書/admin 皆可）。
//! - 查詢目前生效中的代理人走 `GET /protocols/{id}` 回應內的
//!   `pi_delegate`/`is_pi_delegate`（已有 `Scoped<ProtocolView>` 存取控管），
//!   不另開一支無額外授權檢查的唯讀端點。

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{AuthorizePiDelegateRequest, ProtocolPiDelegate, RevokePiDelegateRequest},
    services::ProtocolService,
    AppState, Result,
};

/// 核准 PI 代理人。
#[utoipa::path(
    post,
    path = "/api/v1/protocols/{id}/pi-delegate",
    request_body = AuthorizePiDelegateRequest,
    responses(
        (status = 200, description = "核准成功", body = ProtocolPiDelegate),
        (status = 403, description = "無權核准（非現任 SD；或 SD 核准自己需改由執秘/admin）"),
        (status = 404, description = "找不到計劃書"),
        (status = 422, description = "PI 已有真帳號、尚未指派 SD、代理人資格不符，或已有生效中代理人")
    ),
    params(("id" = Uuid, Path, description = "計劃書 ID")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn authorize_pi_delegate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<AuthorizePiDelegateRequest>,
) -> Result<Json<ProtocolPiDelegate>> {
    req.validate()?;
    let actor = ActorContext::User(current_user);
    let delegate = ProtocolService::authorize_pi_delegate(
        &state.db,
        &actor,
        id,
        req.delegate_user_id,
        req.reason.as_deref(),
    )
    .await?;
    Ok(Json(delegate))
}

/// 撤銷此計畫目前生效中的代理人。
#[utoipa::path(
    delete,
    path = "/api/v1/protocols/{id}/pi-delegate",
    request_body = RevokePiDelegateRequest,
    responses(
        (status = 200, description = "撤銷成功"),
        (status = 403, description = "無權撤銷"),
        (status = 404, description = "找不到計劃書，或目前沒有生效中的代理人")
    ),
    params(("id" = Uuid, Path, description = "計劃書 ID")),
    tag = "計畫書管理",
    security(("bearer" = []))
)]
pub async fn revoke_pi_delegate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<RevokePiDelegateRequest>,
) -> Result<Json<serde_json::Value>> {
    req.validate()?;
    let actor = ActorContext::User(current_user);
    ProtocolService::revoke_pi_delegate(&state.db, &actor, id, req.reason.as_deref()).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
