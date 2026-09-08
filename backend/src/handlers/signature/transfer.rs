// 轉讓記錄簽章 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    services::{SignatureService, SignatureType},
    AppState, Result,
};

use super::{
    sign_response, to_signature_infos, SignRecordRequest, SignRecordResponse,
    SignatureStatusResponse,
};

/// 為轉讓記錄簽章（PI 同意 / 完成轉讓）
#[utoipa::path(
    post,
    path = "/api/v1/signatures/transfer/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到轉讓記錄")
    ),
    params(("id" = Uuid, Path, description = "轉讓記錄 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_transfer_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    // CSO #2：轉讓簽章須三方權責（VET 或 轉出/轉入計劃 PI），非僅計畫成員。
    //
    // P0-3（2026-09-05）：本處原本還疊了一道 `require_permission!("animal.record.create")`，
    // 與 `handlers/animal/transfer.rs::approve_transfer` 是同一個既有缺陷——VET 角色
    // **沒有**該權限碼（`startup/permissions.rs` 的 VET 清單），於是純獸醫（未兼
    // EXPERIMENT_STAFF）在走到下面這道檢查之前就被 403 擋掉，而該檢查第一件事正是
    // `has_role(VET) → Ok`。簽署權責由 check_transfer_signing_authority 單獨認定。
    SignatureService::check_transfer_signing_authority(&state.db, transfer_id, &current_user)
        .await?;

    let content = SignatureService::fetch_transfer_content(&state.db, transfer_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(),
        SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db,
        "transfer",
        &transfer_id.to_string(),
        current_user.id,
        sig_type,
        &content,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;

    Ok(Json(sign_response(&signature, false)))
}

/// 取得轉讓記錄簽章狀態
#[utoipa::path(
    get,
    path = "/api/v1/signatures/transfer/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "轉讓記錄 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_transfer_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    SignatureService::check_transfer_access(&state.db, transfer_id, &current_user).await?;

    let entity_id = transfer_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "transfer", &entity_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "transfer", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: to_signature_infos(infos),
    }))
}
