// 安樂死單據簽章 Handlers

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

/// 為安樂死單據簽章（PI 同意 / 獸醫執行）
#[utoipa::path(
    post,
    path = "/api/v1/signatures/euthanasia/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到安樂死單據")
    ),
    params(("id" = Uuid, Path, description = "安樂死單據 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_euthanasia_order(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    SignatureService::check_euthanasia_access(&state.db, order_id, &current_user).await?;

    let content = SignatureService::fetch_euthanasia_content(&state.db, order_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(), SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db, "euthanasia", &order_id.to_string(),
        current_user.id, sig_type, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    Ok(Json(sign_response(&signature, false)))
}

/// 取得安樂死單據簽章狀態
#[utoipa::path(
    get,
    path = "/api/v1/signatures/euthanasia/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "安樂死單據 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_euthanasia_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    SignatureService::check_euthanasia_access(&state.db, order_id, &current_user).await?;

    let entity_id = order_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "euthanasia", &entity_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "euthanasia", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: to_signature_infos(infos),
    }))
}
