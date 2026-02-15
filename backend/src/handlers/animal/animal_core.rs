// 動物管理 Handlers

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        BatchAssignRequest, CreateAnimalRequest, DeleteRequest, Animal, AnimalListItem, AnimalQuery,
        AnimalsByPen, UpdateAnimalRequest,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppError, AppState, Result,
};

/// 列出所有動物
pub async fn list_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AnimalQuery>,
) -> Result<Json<Vec<AnimalListItem>>> {
    // 檢查權限
    let has_view_all = current_user.has_permission("animal.animal.view_all");
    let has_view_project = current_user.has_permission("animal.animal.view_project");
    
    if !has_view_all && !has_view_project {
        // 如果沒有查看權限，返回空列表
        // 這裡不拋出錯誤，而是返回空列表，避免洩露權限資訊
        return Ok(Json(vec![]));
    }
    
    // 如果只有 view_project 權限而沒有 view_all，則只能查看有 iacuc_no 的動物
    // 即只能查看屬於專案的動物，不能查看沒有 iacuc_no 的動物
    let animals = AnimalService::list(&state.db, &query).await?;
    
    // 如果只有 view_project 權限，則過濾出有 iacuc_no 的動物
    // 即只返回屬於專案的動物，過濾掉沒有 iacuc_no 的動物
    let filtered_animals = if has_view_all {
        animals
    } else {
        // 過濾出有 iacuc_no 的動物，只顯示屬於專案的動物
        animals.into_iter()
            .filter(|a| a.iacuc_no.is_some())
            .collect()
    };
    
    Ok(Json(filtered_animals))
}

/// 按欄位列出所有動物
pub async fn list_animals_by_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalsByPen>>> {
    require_permission!(current_user, "animal.animal.view_all");
    
    let animals = AnimalService::list_by_pen(&state.db).await?;
    Ok(Json(animals))
}

/// 取得單個動物的詳細資訊
pub async fn get_animal(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Animal>> {
    let animal = AnimalService::get_by_id(&state.db, id).await?;
    Ok(Json(animal))
}

/// 建立新動物
pub async fn create_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateAnimalRequest>,
) -> Result<Json<Animal>> {
    require_permission!(current_user, "animal.animal.create");
    
    // 記錄建立動物的請求資訊，用於除錯
    tracing::debug!("Create animal request: ear_tag={}, breed={:?}, gender={:?}, entry_date={:?}, birth_date={:?}, entry_weight={:?}", 
        req.ear_tag, req.breed, req.gender, req.entry_date, req.birth_date, req.entry_weight);
    
    // 驗證請求資料
    if let Err(validation_errors) = req.validate() {
        let error_messages: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    let field_name = match *field {
                        "ear_tag" => "耳標",
                        "breed" => "品種",
                        "gender" => "性別",
                        "entry_date" => "入場日期",
                        "birth_date" => "出生日期",
                        "entry_weight" => "入場體重",
                        _ => field,
                    };
                    format!("{}: {}", field_name, e.message.as_ref().unwrap_or(&e.code))
                })
            })
            .collect();
        let error_msg = error_messages.join("; ");
        tracing::warn!("Validation failed: {}", error_msg);
        return Err(AppError::Validation(error_msg));
    }
    
    let animal = AnimalService::create(&state.db, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "CREATE",
        Some("animal"), Some(animal.id), Some(&animal.ear_tag),
        None,
        Some(serde_json::json!({
            "ear_tag": animal.ear_tag,
            "breed": format!("{:?}", req.breed),
            "gender": format!("{:?}", req.gender),
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_CREATE): {}", e);
    }

    Ok(Json(animal))
}

/// 更新動物資訊
pub async fn update_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAnimalRequest>,
) -> Result<Json<Animal>> {
    require_permission!(current_user, "animal.animal.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let animal = AnimalService::update(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "UPDATE",
        Some("animal"), Some(id), Some(&animal.ear_tag),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_UPDATE): {}", e);
    }

    Ok(Json(animal))
}

/// 刪除動物（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_animal(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.animal.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AnimalService::delete_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "ANIMAL_DELETE",
        Some("animal"), Some(id), Some(&format!("動物 {} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Animal deleted successfully" })))
}

/// 批次分配動物的耳標
pub async fn batch_assign_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchAssignRequest>,
) -> Result<Json<Vec<Animal>>> {
    require_permission!(current_user, "animal.info.assign");
    
    let animals = AnimalService::batch_assign(&state.db, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "ANIMAL_BATCH_ASSIGN",
        Some("animal"), None,
        Some(&format!("批次分配 {} 隻至 {}", animals.len(), &req.iacuc_no)),
        None,
        Some(serde_json::json!({
            "count": animals.len(),
            "iacuc_no": &req.iacuc_no,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (ANIMAL_BATCH_ASSIGN): {}", e);
    }

    Ok(Json(animals))
}

/// 標記動物為獸醫已讀
pub async fn mark_animal_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    AnimalService::mark_vet_read(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}
