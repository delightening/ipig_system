// 體重 + 疫苗記錄管理 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateVaccinationRequest, CreateWeightRequest, DeleteRequest, PigVaccination, PigWeight,
        PigWeightResponse, UpdateVaccinationRequest, UpdateWeightRequest,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppError, AppState, Result,
};

// ============================================
// 體重記錄管理
// ============================================

/// 列出豬的所有體重記錄
pub async fn list_pig_weights(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<PigWeightResponse>>> {
    let weights = AnimalService::list_weights(&state.db, pig_id).await?;
    Ok(Json(weights))
}

/// 建立體重記錄
pub async fn create_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "animal.record.create");
    
    let weight = AnimalService::create_weight(&state.db, pig_id, &req, current_user.id).await?;

    // 取得豬隻資訊用於日誌顯示
    let weight_display = match AnimalService::get_by_id(&state.db, pig_id).await {
        Ok(pig) => {
            let iacuc = pig.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - {} kg", iacuc, pig.ear_tag, req.weight)
        }
        _ => format!("體重紀錄 (pig: {})", pig_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "WEIGHT_CREATE",
        Some("pig_weight"), Some(pig_id),
        Some(&weight_display),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (WEIGHT_CREATE): {}", e);
    }

    Ok(Json(weight))
}

/// 更新體重記錄
pub async fn update_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "animal.record.edit");
    
    let weight = AnimalService::update_weight(&state.db, id, &req).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "WEIGHT_UPDATE",
        Some("pig_weight"), None,
        Some(&format!("體重紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (WEIGHT_UPDATE): {}", e);
    }

    Ok(Json(weight))
}

/// 刪除體重記錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AnimalService::soft_delete_weight_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "WEIGHT_DELETE",
        Some("pig_weight"), None,
        Some(&format!("體重紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (WEIGHT_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Weight record deleted successfully" })))
}

// ============================================
// 疫苗接種記錄管理
// ============================================

/// 列出豬的所有疫苗接種記錄
pub async fn list_pig_vaccinations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<PigVaccination>>> {
    let vaccinations = AnimalService::list_vaccinations(&state.db, pig_id).await?;
    Ok(Json(vaccinations))
}

/// 建立疫苗接種記錄
pub async fn create_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "animal.record.create");
    
    let vaccination = AnimalService::create_vaccination(&state.db, pig_id, &req, current_user.id).await?;

    // 取得豬隻資訊用於日誌顯示
    let vac_display = match AnimalService::get_by_id(&state.db, pig_id).await {
        Ok(pig) => {
            let iacuc = pig.iacuc_no.as_deref().unwrap_or("未指派");
            let vaccine_name = req.vaccine.as_deref().unwrap_or("未指定疫苗");
            format!("[{}] {} - {}", iacuc, pig.ear_tag, vaccine_name)
        }
        _ => format!("疫苗紀錄 (pig: {})", pig_id),
    };

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VACCINATION_CREATE",
        Some("pig_vaccination"), Some(pig_id),
        Some(&vac_display),
        None,
        Some(serde_json::json!({
            "vaccine": req.vaccine,
            "deworming_dose": req.deworming_dose,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VACCINATION_CREATE): {}", e);
    }

    Ok(Json(vaccination))
}

/// 更新疫苗接種記錄
pub async fn update_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "animal.record.edit");
    
    let vaccination = AnimalService::update_vaccination(&state.db, id, &req).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VACCINATION_UPDATE",
        Some("pig_vaccination"), None,
        Some(&format!("疑苗紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VACCINATION_UPDATE): {}", e);
    }

    Ok(Json(vaccination))
}

/// 刪除疫苗接種記錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AnimalService::soft_delete_vaccination_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VACCINATION_DELETE",
        Some("pig_vaccination"), None,
        Some(&format!("疑苗紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VACCINATION_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Vaccination record deleted successfully" })))
}
