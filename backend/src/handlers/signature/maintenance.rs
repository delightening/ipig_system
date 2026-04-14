// 維修/保養驗收簽章 Handlers

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

use super::{sign_response, to_signature_infos, SignRecordRequest, SignRecordResponse, SignatureStatusResponse};

/// 為維修/保養紀錄驗收簽章
pub async fn sign_maintenance_reviewer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(record_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    let content = format!("maintenance_reviewer:{}", record_id);
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(),
        SignatureType::Approve,
    );

    let signature = SignatureService::sign_record(
        &state.db,
        "maintenance_reviewer",
        &record_id.to_string(),
        current_user.id,
        sig_type,
        &content,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;

    // 更新維修紀錄的驗收簽章 ID
    sqlx::query(
        "UPDATE equipment_maintenance_records SET reviewer_signature_id = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(signature.id)
    .bind(record_id)
    .execute(&state.db)
    .await?;

    Ok(Json(sign_response(&signature, true)))
}

/// 取得維修/保養紀錄簽章狀態
pub async fn get_maintenance_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(record_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    if !current_user.has_permission("equipment.view") && !current_user.is_admin() {
        return Err(crate::AppError::Forbidden("Permission denied: requires equipment.view".into()));
    }
    let entity_id = record_id.to_string();
    let sigs =
        SignatureService::get_signature_infos(&state.db, "maintenance_reviewer", &entity_id)
            .await?;

    let is_signed = !sigs.is_empty();

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: is_signed,
        signatures: to_signature_infos(sigs),
    }))
}
