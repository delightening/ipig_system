// 猝死登記 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{AnimalSuddenDeath, CreateSuddenDeathRequest},
    require_permission,
    services::{access, AnimalMedicalService, AnimalService, AuditService},
    AppState, Result,
};

/// 取得動物的猝死記錄
pub async fn get_animal_sudden_death(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalSuddenDeath>>> {
    // SEC-IDOR: v2 審計發現原始修復遺漏此端點
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let record = AnimalMedicalService::get_sudden_death(&state.db, animal_id).await?;
    Ok(Json(record))
}

/// 登記猝死
pub async fn create_animal_sudden_death(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSuddenDeathRequest>,
) -> Result<Json<AnimalSuddenDeath>> {
    require_permission!(current_user, "animal.record.create");

    let record =
        AnimalMedicalService::create_sudden_death(&state.db, animal_id, &req, current_user.id)
            .await?;

    // 取得動物資訊用於日誌顯示
    let display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - 猝死", iacuc, animal.ear_tag)
        }
        _ => format!("猝死紀錄 (animal: {})", animal_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "SUDDEN_DEATH",
        Some("animal_sudden_deaths"),
        Some(animal_id),
        Some(&display),
        None,
        Some(serde_json::json!({
            "probable_cause": req.probable_cause,
            "requires_pathology": req.requires_pathology,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (SUDDEN_DEATH): {}", e);
    }

    Ok(Json(record))
}
