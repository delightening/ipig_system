// 設備報廢簽章 Handlers

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

/// 為報廢申請簽章（申請人簽章）
pub async fn sign_disposal_applicant(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(disposal_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    let content = format!("disposal_applicant:{}", disposal_id);
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(),
        SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db,
        "disposal_applicant",
        &disposal_id.to_string(),
        current_user.id,
        sig_type,
        &content,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;

    // 更新報廢紀錄的申請人簽章 ID
    sqlx::query(
        "UPDATE equipment_disposals SET applicant_signature_id = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(signature.id)
    .bind(disposal_id)
    .execute(&state.db)
    .await?;

    Ok(Json(sign_response(&signature, false)))
}

/// 為報廢核准簽章（核准人簽章）
pub async fn sign_disposal_approver(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(disposal_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    let content = format!("disposal_approver:{}", disposal_id);
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(),
        SignatureType::Approve,
    );

    let signature = SignatureService::sign_record(
        &state.db,
        "disposal_approver",
        &disposal_id.to_string(),
        current_user.id,
        sig_type,
        &content,
        req.password.as_deref(),
        req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    )
    .await?;

    // 更新報廢紀錄的核准人簽章 ID
    sqlx::query(
        "UPDATE equipment_disposals SET approver_signature_id = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(signature.id)
    .bind(disposal_id)
    .execute(&state.db)
    .await?;

    Ok(Json(sign_response(&signature, true)))
}

/// 取得報廢紀錄簽章狀態
pub async fn get_disposal_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(disposal_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    if !current_user.has_permission("equipment.view") && !current_user.is_admin() {
        return Err(crate::AppError::Forbidden("Permission denied: requires equipment.view".into()));
    }
    let entity_id = disposal_id.to_string();
    let applicant_sigs =
        SignatureService::get_signature_infos(&state.db, "disposal_applicant", &entity_id).await?;
    let approver_sigs =
        SignatureService::get_signature_infos(&state.db, "disposal_approver", &entity_id).await?;

    let mut all_sigs = applicant_sigs;
    all_sigs.extend(approver_sigs);

    let is_signed = !all_sigs.is_empty();

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: is_signed,
        signatures: to_signature_infos(all_sigs),
    }))
}
