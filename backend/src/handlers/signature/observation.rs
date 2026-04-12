// 觀察記錄簽章 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{SignatureService, SignatureType},
    AppState, Result,
};

use super::{sign_response, SignRecordRequest, SignRecordResponse};

/// 為觀察記錄簽章
#[utoipa::path(
    post,
    path = "/api/v1/signatures/observation/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到觀察記錄")
    ),
    params(("id" = i32, Path, description = "觀察記錄 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_observation_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(observation_id): Path<i32>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.view");
    SignatureService::check_animal_record_access(
        &state.db, "animal_observations", observation_id, &current_user,
    ).await?;

    let content = SignatureService::fetch_observation_content(&state.db, observation_id).await?;

    let signature = SignatureService::sign_record(
        &state.db, "observation", &observation_id.to_string(),
        current_user.id, SignatureType::Confirm, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    SignatureService::lock_record(&state.db, "observation", observation_id, current_user.id).await?;
    Ok(Json(sign_response(&signature, true)))
}
