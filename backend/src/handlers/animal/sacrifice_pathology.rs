// 犧牲與病理 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{AnimalSacrifice, AnimalSampling, CreateSacrificeRequest, CreateSamplingRequest},
    require_permission,
    services::{AnimalMedicalService, AnimalService, AuditService, FileService},
    AppState, Result,
};

/// 取得動物的犧牲記錄
pub async fn get_animal_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalSacrifice>>> {
    let record = AnimalMedicalService::get_sacrifice(&state.db, animal_id).await?;
    Ok(Json(record))
}

/// 建立犧牲紀錄（申請）
pub async fn create_animal_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<AnimalSacrifice>> {
    require_permission!(current_user, "animal.record.create");

    let record =
        AnimalMedicalService::create_sacrifice(&state.db, animal_id, &req, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "SACRIFICE_REG",
        Some("animal_sacrifices"),
        Some(animal_id),
        Some(&format!("登記犧牲: {}", animal_id)),
        None,
        Some(serde_json::json!({ "sacrifice_type": req.sacrifice_type })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (SACRIFICE_REG): {}", e);
    }

    Ok(Json(record))
}

/// 確認犧牲（由 PI 或 Admin 確認）
pub async fn confirm_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnimalSacrifice>> {
    require_permission!(current_user, "animal.record.create");

    let record = AnimalMedicalService::confirm_sacrifice(&state.db, id, current_user.id).await?;

    // 自動清理欄位位置（移出欄位）
    if let Err(e) = sqlx::query("UPDATE animals SET pen_location = NULL WHERE id = $1")
        .bind(record.animal_id)
        .execute(&state.db)
        .await
    {
        tracing::warn!("犧牲自動移出欄位失敗: {}", e);
    }

    Ok(Json(record))
}

/// 取得動物的採樣記錄
pub async fn get_animal_sampling(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<AnimalSampling>>> {
    let records = AnimalMedicalService::list_samplings(&state.db, animal_id).await?;
    Ok(Json(records))
}

/// 建立採樣紀錄
pub async fn create_animal_sampling(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSamplingRequest>,
) -> Result<Json<AnimalSampling>> {
    require_permission!(current_user, "animal.record.create");

    let record =
        AnimalMedicalService::create_sampling(&state.db, animal_id, &req, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "SAMPLING_REG",
        Some("animal_samplings"),
        Some(animal_id),
        Some(&format!("登記採樣: {}", animal_id)),
        None,
        Some(serde_json::json!({ "sampling_type": req.sampling_type })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (SAMPLING_REG): {}", e);
    }

    Ok(Json(record))
}

/// 取得動物的病理紀錄（包含附件列表）
pub async fn get_animal_pathology(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.pathology.view");

    let sacrifice = AnimalMedicalService::get_sacrifice(&state.db, animal_id).await?;
    let samplings = AnimalMedicalService::list_samplings(&state.db, animal_id).await?;
    let attachments = FileService::list_by_entity(&state.db, "pathology", &animal_id).await?;

    Ok(Json(serde_json::json!({
        "sacrifice": sacrifice,
        "samplings": samplings,
        "attachments": attachments,
    })))
}

/// 刪除採樣紀錄
pub async fn delete_animal_sampling(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.create");

    AnimalMedicalService::delete_sampling(&state.db, id).await?;

    Ok(Json(
        serde_json::json!({ "message": "Sampling record deleted" }),
    ))
}
