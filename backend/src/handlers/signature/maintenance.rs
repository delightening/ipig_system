// 維修/保養驗收簽章 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    services::{EquipmentService, SignatureService, SignatureType},
    AppError, AppState, Result,
};

use super::{sign_response, to_signature_infos, SignRecordRequest, SignRecordResponse, SignatureStatusResponse};

/// 為維修/保養紀錄驗收簽章
///
/// Service-driven：handler 只解 request、組 ActorContext、轉呼叫
/// `EquipmentService::sign_maintenance_review_tx`（內含 RBAC + lock + status guard +
/// sign_record_tx + UPDATE + audit log，全 tx 原子）。
pub async fn sign_maintenance_reviewer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(record_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(),
        SignatureType::Approve,
    );
    let actor = ActorContext::User(current_user);

    let signature = EquipmentService::sign_maintenance_review_tx(
        &state.db,
        &actor,
        record_id,
        sig_type,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
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
        return Err(AppError::Forbidden("Permission denied: requires equipment.view".into()));
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
