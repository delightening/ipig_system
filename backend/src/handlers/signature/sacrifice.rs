// 犧牲記錄簽章 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{SignatureService, SignatureType},
    AppState, Result,
};

use super::{
    sign_response, to_signature_infos, SignRecordRequest, SignRecordResponse,
    SignatureStatusResponse,
};

/// 為犧牲記錄簽章
#[utoipa::path(
    post,
    path = "/api/v1/signatures/sacrifice/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到犧牲記錄")
    ),
    params(("id" = Uuid, Path, description = "犧牲記錄 ID (UUID)")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_sacrifice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sacrifice_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.sacrifice");
    SignatureService::check_animal_record_access_uuid(
        &state.db, "animal_sacrifices", sacrifice_id, &current_user,
    ).await?;

    let content = SignatureService::fetch_sacrifice_content(&state.db, sacrifice_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(), SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db, "sacrifice", &sacrifice_id.to_string(),
        current_user.id, sig_type, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    SignatureService::lock_record_uuid(&state.db, "sacrifice", sacrifice_id, current_user.id).await?;
    Ok(Json(sign_response(&signature, true)))
}

/// 取得犧牲記錄簽章狀態
#[utoipa::path(
    get,
    path = "/api/v1/signatures/sacrifice/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "犧牲記錄 ID (UUID)")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_sacrifice_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sacrifice_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    require_permission!(current_user, "animal.record.view");
    SignatureService::check_animal_record_access_uuid(
        &state.db, "animal_sacrifices", sacrifice_id, &current_user,
    ).await?;

    let entity_id = sacrifice_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "sacrifice", &entity_id).await?;
    let is_locked = SignatureService::is_locked_uuid(&state.db, "sacrifice", sacrifice_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "sacrifice", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked,
        signatures: to_signature_infos(infos),
    }))
}
