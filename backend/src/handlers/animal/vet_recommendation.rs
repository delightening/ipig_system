// 獸醫建議 (AnimalVetRecommendation) Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{CreateVetRecommendationRequest, VetRecommendation},
    require_permission,
    services::{AnimalMedicalService, AnimalService, AuditService},
    AppState, Result,
};

/// 取得動物的獸醫建議列表
pub async fn list_animal_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let records = AnimalMedicalService::list_vet_recommendations(&state.db, animal_id).await?;
    Ok(Json(records))
}

/// 建立獸醫建議
pub async fn create_animal_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");

    let record = AnimalMedicalService::create_vet_recommendation(
        &state.db,
        animal_id,
        &req,
        current_user.id,
    )
    .await?;

    // 處理日誌顯示
    let display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - 獸醫建議", iacuc, animal.ear_tag)
        }
        _ => format!("獸醫建議紀錄 (animal: {})", animal_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db,
        current_user.id,
        "ANIMAL",
        "VET_RECOMMENDATION",
        Some("animal_vet_recommendations"),
        Some(animal_id),
        Some(&display),
        None,
        Some(serde_json::json!({
            "diagnosis": req.diagnosis,
            "treatment_plan": req.treatment_plan,
        })),
        None,
        None,
    )
    .await
    {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION): {}", e);
    }

    Ok(Json(record))
}

/// 更新獸醫建議
pub async fn update_animal_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");

    let record =
        AnimalMedicalService::update_vet_recommendation(&state.db, id, &req, current_user.id)
            .await?;

    Ok(Json(record))
}

/// 刪除獸醫建議
pub async fn delete_animal_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.recommend");

    AnimalMedicalService::delete_vet_recommendation(&state.db, id).await?;

    Ok(Json(serde_json::json!({
        "message": "Vet recommendation record deleted"
    })))
}
