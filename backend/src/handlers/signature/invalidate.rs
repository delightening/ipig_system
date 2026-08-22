// R30-9b: 撤銷電子簽章 handler
//
// 對應設計：docs/design/r30-9-signature-audit-chain.md §4.4
// 與 SIGNATURE_CREATE 路徑不同：撤銷不寫新 signature row（避免 recursive），
// 改為 UPDATE 既有 row 的 invalidated_* 欄位 + 寫 SIGNATURE_INVALIDATED audit
// chain（v3 含 sig fingerprint，由 services 層 log_signature_event_tx 處理）。

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::SignatureService,
    utils::validation::validate_reason_text,
    AppState, Result,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct InvalidateSignatureRequest {
    /// 撤銷理由（會寫入 audit chain v3 binding，trim 後 ≥1 / ≤1000 字元）。
    /// 用 custom validator 走 trim 規則，與 service 層一致，避免帶前後空白的合法輸入被誤拒。
    #[validate(custom(function = "validate_reason_text"))]
    pub reason: String,
    /// 操作者密碼（撤銷不可逆，要求密碼驗證以確保不可否認性）
    #[validate(length(min = 1, message = "password 必填"))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InvalidateSignatureResponse {
    pub status: &'static str,
    pub signature_id: Uuid,
}

/// 撤銷電子簽章（admin only）
///
/// 寫 audit chain v3 SIGNATURE_INVALIDATED 事件 + UPDATE invalidated_* 欄位，
/// 同 tx atomic。**不**自動 revert 已使用該簽章的記錄（amendment / protocol /
/// sacrifice 等），人工流程處理。
#[utoipa::path(
    post,
    path = "/api/v1/signatures/{id}/invalidate",
    request_body = InvalidateSignatureRequest,
    responses(
        (status = 200, description = "撤銷成功", body = InvalidateSignatureResponse),
        (status = 400, description = "reason 為空或過長"),
        (status = 401, description = "密碼錯誤"),
        (status = 403, description = "缺 signature.invalidate 權限"),
        (status = 404, description = "找不到簽章"),
        (status = 409, description = "簽章已被撤銷")
    ),
    params(("id" = Uuid, Path, description = "簽章 UUID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn invalidate_signature(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(signature_id): Path<Uuid>,
    Json(req): Json<InvalidateSignatureRequest>,
) -> Result<Json<InvalidateSignatureResponse>> {
    require_permission!(current_user, "signature.invalidate");
    req.validate()?;

    let actor = ActorContext::User(current_user);
    SignatureService::invalidate(&state.db, &actor, signature_id, &req.reason, &req.password)
        .await?;

    // 撤銷成功不洩漏 signer / record 細節（response 僅回 sig_id 確認）。
    // 詳細 audit 已寫進 user_activity_logs（admin 走 audit log 頁查）。
    Ok(Json(InvalidateSignatureResponse {
        status: "invalidated",
        signature_id,
    }))
}
