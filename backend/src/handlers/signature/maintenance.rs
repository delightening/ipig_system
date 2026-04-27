// 維修/保養驗收簽章 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    services::{SignatureService, SignatureType},
    AppError, AppState, Result,
};

use super::{sign_response, to_signature_infos, SignRecordRequest, SignRecordResponse, SignatureStatusResponse};

/// 為維修/保養紀錄驗收簽章
pub async fn sign_maintenance_reviewer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(record_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    // RBAC：與 EquipmentService::review_maintenance_record 一致；防止任意已認證使用者
    // 用自己密碼為任意 maintenance record 建立 reviewer 簽章。
    if !current_user.has_permission("equipment.maintenance.review")
        && !current_user.has_permission("equipment.manage")
    {
        return Err(AppError::Forbidden("無權驗收維修保養紀錄".into()));
    }

    // 狀態守衛 + 防覆寫（21 CFR §11.10(e)(1)：簽章記錄不得被竄改）。
    // SELECT FOR UPDATE 鎖住該 row，避免兩個 concurrent reviewer 同時通過檢查後雙寫。
    let mut tx = state.db.begin().await?;
    let existing: Option<(String, Option<Uuid>)> = sqlx::query_as(
        "SELECT status::text, reviewer_signature_id \
         FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
    )
    .bind(record_id)
    .fetch_optional(&mut *tx)
    .await?;

    let (status, existing_sig) =
        existing.ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

    if status != "pending_review" {
        return Err(AppError::BadRequest("此紀錄非待驗收狀態，無法簽章".into()));
    }
    if existing_sig.is_some() {
        return Err(AppError::Conflict("此紀錄已簽章，不得覆寫".into()));
    }

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

    sqlx::query(
        "UPDATE equipment_maintenance_records \
         SET reviewer_signature_id = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(signature.id)
    .bind(record_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

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
