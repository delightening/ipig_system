use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;
use sqlx::PgPool;

use crate::{
    middleware::CurrentUser,
    models::{
        BatchAssignRequest, CreateObservationRequest,
        CreatePigRequest, CreatePigSourceRequest, CreateSacrificeRequest, CreateSurgeryRequest,
        CreateVaccinationRequest, CreateVetRecommendationRequest, CreateWeightRequest, Pig,
        PigListItem, PigObservation, PigQuery, PigSacrifice, PigSource, PigSurgery,
        PigVaccination, PigWeight, PigWeightResponse, PigsByPen, UpdatePigRequest, UpdatePigSourceRequest,
        VetRecommendation, VetRecordType, UpdateObservationRequest, UpdateSurgeryRequest, UpdateWeightRequest,
        UpdateVaccinationRequest, CopyRecordRequest, VersionHistoryResponse,
        CreateVetRecommendationWithAttachmentsRequest, ExportRequest, PigImportBatch, ObservationListItem, SurgeryListItem, ImportResult,
        DeleteRequest,  // GLP: 刪除請求含原因
        // 血液檢查相關
        BloodTestTemplate, BloodTestListItem, PigBloodTestWithItems,
        CreateBloodTestRequest, UpdateBloodTestRequest,
        CreateBloodTestTemplateRequest, UpdateBloodTestTemplateRequest,
        // 血液檢查組合
        BloodTestPanelWithItems,
        CreateBloodTestPanelRequest, UpdateBloodTestPanelRequest, UpdateBloodTestPanelItemsRequest,
    },
    require_permission,
    services::{AnimalService, AuditService, PdfService},
    AppError, AppState, Result,
};
use axum::extract::Multipart;

// ============================================
// Helper functions for notification
// ============================================

/// 從觀察紀錄 ID 取得豬隻資訊（用於發送通知）
async fn get_pig_info_from_observation(
    pool: &PgPool,
    observation_id: Uuid,
) -> std::result::Result<Option<(Uuid, String, Option<Uuid>)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
        r#"
        SELECT p.id, p.ear_tag, pr.id as protocol_id
        FROM pig_observations po
        JOIN pigs p ON po.pig_id = p.id
        LEFT JOIN protocols pr ON p.iacuc_no = pr.iacuc_no
        WHERE po.id = $1
        "#
    )
    .bind(observation_id)
    .fetch_optional(pool)
    .await
}

/// 從手術紀錄 ID 取得豬隻資訊（用於發送通知）
async fn get_pig_info_from_surgery(
    pool: &PgPool,
    surgery_id: Uuid,
) -> std::result::Result<Option<(Uuid, String, Option<Uuid>)>, sqlx::Error> {
    sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
        r#"
        SELECT p.id, p.ear_tag, pr.id as protocol_id
        FROM pig_surgeries ps
        JOIN pigs p ON ps.pig_id = p.id
        LEFT JOIN protocols pr ON p.iacuc_no = pr.iacuc_no
        WHERE ps.id = $1
        "#
    )
    .bind(surgery_id)
    .fetch_optional(pool)
    .await
}

// ============================================
// 豬源管理
// ============================================


/// 列出所有豬源
pub async fn list_pig_sources(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigSource>>> {
    let sources = AnimalService::list_sources(&state.db).await?;
    Ok(Json(sources))
}

/// 建立豬源
pub async fn create_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "animal.animal.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let source = AnimalService::create_source(&state.db, &req).await?;
    Ok(Json(source))
}

/// 更新豬源
pub async fn update_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "animal.animal.edit");
    
    let source = AnimalService::update_source(&state.db, id, &req).await?;
    Ok(Json(source))
}

/// 刪除豬源
pub async fn delete_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.animal.delete");
    
    AnimalService::delete_source(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Pig source deleted successfully" })))
}

// ============================================
// 豬管理
// ============================================

/// 列出所有豬
pub async fn list_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PigQuery>,
) -> Result<Json<Vec<PigListItem>>> {
    // 檢查權限
    let has_view_all = current_user.has_permission("animal.animal.view_all");
    let has_view_project = current_user.has_permission("animal.animal.view_project");
    
    if !has_view_all && !has_view_project {
        // 如果沒有查看權限，返回空列表
        // 這裡不拋出錯誤，而是返回空列表，避免洩露權限資訊
        return Ok(Json(vec![]));
    }
    
    // 如果只有 view_project 權限而沒有 view_all，則只能查看有 iacuc_no 的豬
    // 即只能查看屬於專案的豬，不能查看沒有 iacuc_no 的豬
    let pigs = AnimalService::list(&state.db, &query).await?;
    
    // 如果只有 view_project 權限，則過濾出有 iacuc_no 的豬
    // 即只返回屬於專案的豬，過濾掉沒有 iacuc_no 的豬
    let filtered_pigs = if has_view_all {
        pigs
    } else {
        // 過濾出有 iacuc_no 的豬，只顯示屬於專案的豬
        pigs.into_iter()
            .filter(|pig| pig.iacuc_no.is_some())
            .collect()
    };
    
    Ok(Json(filtered_pigs))
}

/// 按欄位列出所有豬
pub async fn list_pigs_by_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigsByPen>>> {
    require_permission!(current_user, "animal.animal.view_all");
    
    let pigs = AnimalService::list_by_pen(&state.db).await?;
    Ok(Json(pigs))
}

/// 取得單個豬的詳細資訊
pub async fn get_pig(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Pig>> {
    let pig = AnimalService::get_by_id(&state.db, id).await?;
    Ok(Json(pig))
}

/// 建立新豬
pub async fn create_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "animal.animal.create");
    
    // 記錄建立豬的請求資訊，用於除錯
    tracing::debug!("Create pig request: ear_tag={}, breed={:?}, gender={:?}, entry_date={:?}, birth_date={:?}, entry_weight={:?}", 
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
    
    let pig = AnimalService::create(&state.db, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PIG_CREATE",
        Some("pig"), Some(pig.id), Some(&pig.ear_tag),
        None,
        Some(serde_json::json!({
            "ear_tag": pig.ear_tag,
            "breed": format!("{:?}", req.breed),
            "gender": format!("{:?}", req.gender),
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PIG_CREATE): {}", e);
    }

    Ok(Json(pig))
}

/// 更新豬資訊
pub async fn update_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "animal.animal.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let pig = AnimalService::update(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PIG_UPDATE",
        Some("pig"), Some(id), Some(&pig.ear_tag),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PIG_UPDATE): {}", e);
    }

    Ok(Json(pig))
}

/// 刪除豬（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_pig(
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
        &state.db, current_user.id, "ANIMAL", "PIG_DELETE",
        Some("pig"), Some(id), Some(&format!("豬隻 {} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PIG_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Pig deleted successfully" })))
}

/// 批次分配豬的耳標
pub async fn batch_assign_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchAssignRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "animal.info.assign");
    
    let pigs = AnimalService::batch_assign(&state.db, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PIG_BATCH_ASSIGN",
        Some("pig"), None,
        Some(&format!("批次分配 {} 隻至 {}", pigs.len(), &req.iacuc_no)),
        None,
        Some(serde_json::json!({
            "count": pigs.len(),
            "iacuc_no": &req.iacuc_no,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PIG_BATCH_ASSIGN): {}", e);
    }

    Ok(Json(pigs))
}



/// 標記豬為獸醫已讀
pub async fn mark_pig_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    AnimalService::mark_vet_read(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

// ============================================
// 觀察記錄管理
// ============================================

/// 列出豬的所有觀察記錄
pub async fn list_pig_observations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<PigObservation>>> {
    let observations = AnimalService::list_observations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 列出豬的觀察記錄（包含獸醫建議）
pub async fn list_pig_observations_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<ObservationListItem>>> {
    let observations = AnimalService::list_observations_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 取得單個觀察記錄
pub async fn get_pig_observation(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PigObservation>> {
    let observation = AnimalService::get_observation_by_id(&state.db, id).await?;
    Ok(Json(observation))
}

/// 建立觀察記錄
pub async fn create_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    // 檢查是否為緊急給藥，需驗證是否有緊急給藥權限
    if req.is_emergency {
        require_permission!(current_user, "animal.record.emergency");
    }
    
    let observation = AnimalService::create_observation(&state.db, pig_id, &req, current_user.id).await?;
    
    // 如果是緊急給藥，發送通知給 VET 和 PI
    if req.is_emergency {
        // 取得豬隻資訊
        if let Ok(pig) = AnimalService::get_by_id(&state.db, pig_id).await {
            let notification_service = crate::services::NotificationService::new(state.db.clone());
            let emergency_reason = req.emergency_reason.as_deref().unwrap_or("未提供原因");
            
            // 異步發送通知，不阻塞主流程
            let _ = notification_service.notify_emergency_medication(
                pig_id,
                observation.id,
                &pig.ear_tag,
                pig.iacuc_no.as_deref(),
                &current_user.email,
                emergency_reason,
            ).await;
            
            tracing::warn!(
                "[Emergency Medication] User {} recorded emergency medication for pig {} (observation {})",
                current_user.email,
                pig.ear_tag,
                observation.id
            );
        }
    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "OBSERVATION_CREATE",
        Some("pig_observation"), Some(pig_id),
        Some(&format!("觀察紀錄 #{} (pig: {})", observation.id, pig_id)),
        None,
        Some(serde_json::json!({
            "observation_id": observation.id,
            "is_emergency": req.is_emergency,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_CREATE): {}", e);
    }

    Ok(Json(observation))
}

/// 更新觀察記錄
pub async fn update_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.edit");
    
    let observation = AnimalService::update_observation(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "OBSERVATION_UPDATE",
        Some("pig_observation"), None,
        Some(&format!("觀察紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_UPDATE): {}", e);
    }

    Ok(Json(observation))
}

/// 刪除觀察記錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AnimalService::soft_delete_observation_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "OBSERVATION_DELETE",
        Some("pig_observation"), None,
        Some(&format!("觀察紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (OBSERVATION_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Observation deleted successfully" })))
}

/// 複製觀察記錄
pub async fn copy_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.copy");
    
    let observation = AnimalService::copy_observation(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(observation))
}

/// 標記觀察記錄為獸醫已讀
pub async fn mark_observation_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    AnimalService::mark_observation_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得觀察記錄的版本歷史
pub async fn get_observation_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = AnimalService::get_record_versions(&state.db, "observation", id).await?;
    Ok(Json(versions))
}

// ============================================
// 手術記錄管理
// ============================================

/// 列出豬的所有手術記錄
pub async fn list_pig_surgeries(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<PigSurgery>>> {
    let surgeries = AnimalService::list_surgeries(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// 列出豬的手術記錄（包含獸醫建議）
pub async fn list_pig_surgeries_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<SurgeryListItem>>> {
    let surgeries = AnimalService::list_surgeries_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// 取得單個手術記錄
pub async fn get_pig_surgery(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PigSurgery>> {
    let surgery = AnimalService::get_surgery_by_id(&state.db, id).await?;
    Ok(Json(surgery))
}

/// 建立手術記錄
pub async fn create_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let surgery = AnimalService::create_surgery(&state.db, pig_id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SURGERY_CREATE",
        Some("pig_surgery"), Some(pig_id),
        Some(&format!("手術紀錄 #{} (pig: {})", surgery.id, pig_id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_CREATE): {}", e);
    }

    Ok(Json(surgery))
}

/// 更新手術記錄
pub async fn update_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "animal.record.edit");
    
    let surgery = AnimalService::update_surgery(&state.db, id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SURGERY_UPDATE",
        Some("pig_surgery"), None,
        Some(&format!("手術紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_UPDATE): {}", e);
    }

    Ok(Json(surgery))
}

/// 刪除手術記錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    AnimalService::soft_delete_surgery_with_reason(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SURGERY_DELETE",
        Some("pig_surgery"), None,
        Some(&format!("手術紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SURGERY_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Surgery deleted successfully" })))
}

/// 複製手術記錄
pub async fn copy_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "animal.record.copy");
    
    let surgery = AnimalService::copy_surgery(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 標記手術記錄為獸醫已讀
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    AnimalService::mark_surgery_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得手術記錄的版本歷史
pub async fn get_surgery_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = AnimalService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}

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

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "WEIGHT_CREATE",
        Some("pig_weight"), Some(pig_id),
        Some(&format!("體重紀錄 (pig: {})", pig_id)),
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

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VACCINATION_CREATE",
        Some("pig_vaccination"), Some(pig_id),
        Some(&format!("疑苗紀錄 (pig: {})", pig_id)),
        None, None, None, None,
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

// ============================================
// 犧牲/安樂死記錄管理
// ============================================

/// 取得豬的犧牲記錄
pub async fn get_pig_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Option<PigSacrifice>>> {
    let sacrifice = AnimalService::get_sacrifice(&state.db, pig_id).await?;
    Ok(Json(sacrifice))
}

/// 建立或更新犧牲記錄
pub async fn upsert_pig_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<PigSacrifice>> {
    require_permission!(current_user, "animal.record.create");
    
    let sacrifice = AnimalService::upsert_sacrifice(&state.db, pig_id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "SACRIFICE_UPSERT",
        Some("pig_sacrifice"), Some(pig_id),
        Some(&format!("犧牲/安樂死紀錄 (pig: {})", pig_id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (SACRIFICE_UPSERT): {}", e);
    }

    Ok(Json(sacrifice))
}

// ============================================
// 獸醫建議管理
// ============================================

/// 為觀察記錄新增獸醫建議
pub async fn add_observation_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation(&state.db, VetRecordType::Observation, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_observation(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "觀察紀錄";
        let _ = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await;
    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("觀察紀錄 #{} 獸醫建議", id)),
        None,
        Some(serde_json::json!({ "record_type": "observation", "record_id": id, "is_urgent": req.is_urgent })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}

/// 為手術記錄新增獸醫建議
pub async fn add_surgery_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation(&state.db, VetRecordType::Surgery, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_surgery(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "手術紀錄";
        let _ = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await;
    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("手術紀錄 #{} 獸醫建議", id)),
        None,
        Some(serde_json::json!({ "record_type": "surgery", "record_id": id, "is_urgent": req.is_urgent })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}
pub async fn add_observation_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    require_permission!(current_user, "animal.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation_with_attachments(&state.db, VetRecordType::Observation, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_observation(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "觀察紀錄";
        let _ = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await;
    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("觀察紀錄 #{} 獸醫建議 (含附件)", id)),
        None,
        Some(serde_json::json!({ "record_type": "observation", "record_id": id, "is_urgent": req.is_urgent, "has_attachments": true })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}
pub async fn add_surgery_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    require_permission!(current_user, "animal.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = AnimalService::add_vet_recommendation_with_attachments(&state.db, VetRecordType::Surgery, id, &req, current_user.id).await?;
    
    // 發送通知給 PI/Coeditor
    if let Ok(Some((pig_id, ear_tag, protocol_id))) = get_pig_info_from_surgery(&state.db, id).await {
        let notification_service = crate::services::NotificationService::new(state.db.clone());
        let record_type_str = "手術紀錄";
        let _ = notification_service.notify_vet_recommendation(
            pig_id,
            &ear_tag,
            protocol_id,
            record_type_str,
            &req.content,
            req.is_urgent,
            Some(&state.config),
        ).await;
    }

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "VET_RECOMMENDATION_ADD",
        Some("vet_recommendation"), None,
        Some(&format!("手術紀錄 #{} 獸醫建議 (含附件)", id)),
        None,
        Some(serde_json::json!({ "record_type": "surgery", "record_id": id, "is_urgent": req.is_urgent, "has_attachments": true })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (VET_RECOMMENDATION_ADD): {}", e);
    }

    Ok(Json(recommendation))
}
pub async fn get_observation_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = AnimalService::get_vet_recommendations(&state.db, VetRecordType::Observation, id).await?;
    Ok(Json(recommendations))
}

/// 取得手術記錄的所有獸醫建議
pub async fn get_surgery_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = AnimalService::get_vet_recommendations(&state.db, VetRecordType::Surgery, id).await?;
    Ok(Json(recommendations))
}

// ============================================
// 匯入匯出
// ============================================

/// 匯出豬的醫療資料
pub async fn export_pig_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<ExportRequest>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");
    
    // 取得醫療資料
    let data = AnimalService::get_pig_medical_data(&state.db, pig_id).await?;
    
    // 建立匯出記錄
    let _record = AnimalService::create_export_record(
        &state.db,
        Some(pig_id),
        None,
        req.export_type,
        req.format,
        None,
        current_user.id,
    ).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "MEDICAL_EXPORT",
        Some("pig"), Some(pig_id),
        Some(&format!("匯出醫療資料 (pig: {})", pig_id)),
        None,
        Some(serde_json::json!({ "format": format!("{:?}", req.format), "export_type": format!("{:?}", req.export_type) })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (MEDICAL_EXPORT): {}", e);
    }

    match req.format {
        crate::models::ExportFormat::Pdf => {
            let pdf_bytes = PdfService::generate_medical_pdf(&data)?;
            let filename = format!("medical_record_{}.pdf", pig_id);
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(Body::from(pdf_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
        }
        _ => {
            // 目前僅 PDF 支援後端生成，其他格式暫時維持原樣（返回 JSON 由前端處理或待後續實現）
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&serde_json::json!({
                    "data": data,
                    "format": req.format,
                    "export_type": req.export_type,
                })).unwrap()))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
        }
    }
}

/// 匯出專案的醫療資料
pub async fn export_project_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(iacuc_no): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Response> {
    require_permission!(current_user, "animal.export.medical");
    
    // 取得專案下所有豬的醫療資料
    let data = AnimalService::get_project_medical_data(&state.db, &iacuc_no).await?;
    
    // 建立匯出記錄
    let _record = AnimalService::create_export_record(
        &state.db,
        None,
        Some(&iacuc_no),
        req.export_type,
        req.format,
        None,
        current_user.id,
    ).await?;
    
    match req.format {
        crate::models::ExportFormat::Pdf => {
            let pdf_bytes = PdfService::generate_project_medical_pdf(&iacuc_no, &data)?;
            let filename = format!("project_medical_{}.pdf", iacuc_no);
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(Body::from(pdf_bytes))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&serde_json::json!({
                    "data": data,
                    "format": req.format,
                    "export_type": req.export_type,
                })).unwrap()))
                .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
        }
    }
}

/// 列出所有匯入批次
pub async fn list_import_batches(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigImportBatch>>> {
    require_permission!(current_user, "animal.animal.import");
    
    let batches = AnimalService::list_import_batches(&state.db, 50).await?;
    Ok(Json(batches))
}

/// 下載豬基礎資料匯入範本
pub async fn download_basic_import_template(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "animal.animal.import");
    
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("xlsx");
    
    let (data, filename, content_type) = if format == "csv" {
        let csv_data = AnimalService::generate_basic_import_template_csv()?;
        (
            csv_data,
            "pig_basic_import_template.csv",
            "text/csv; charset=utf-8",
        )
    } else {
        let excel_data = AnimalService::generate_basic_import_template()?;
        (
            excel_data,
            "pig_basic_import_template.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
    };
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
}

/// 下載豬體重匯入範本
pub async fn download_weight_import_template(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "animal.animal.import");
    
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("xlsx");
    
    let (data, filename, content_type) = if format == "csv" {
        let csv_data = AnimalService::generate_weight_import_template_csv()?;
        (
            csv_data,
            "pig_weight_import_template.csv",
            "text/csv; charset=utf-8",
        )
    } else {
        let excel_data = AnimalService::generate_weight_import_template()?;
        (
            excel_data,
            "pig_weight_import_template.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
    };
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
}

/// 匯入豬基礎資料
pub async fn import_basic_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "animal.animal.import");

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");

    // 解析 multipart 資料
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("解析檔案欄位失敗: {}", e))
    })? {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());

            let data = field.bytes().await.map_err(|e| {
                AppError::Validation(format!("讀取檔案資料失敗: {}", e))
            })?;

            file_data = Some(data.to_vec());
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("未找到檔案".to_string())
    })?;

    // 檢查檔案大小，限制為 10MB 以內
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    // 執行匯入
    let result = AnimalService::import_basic_data(
        &state.db,
        &file_data,
        &file_name,
        current_user.id,
    )
    .await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PIG_IMPORT",
        Some("pig"), None,
        Some(&format!("匯入豬基礎資料: {} (成功: {}, 失敗: {})", file_name, result.success_count, result.error_count)),
        None,
        Some(serde_json::json!({
            "file_name": file_name,
            "success_count": result.success_count,
            "error_count": result.error_count,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PIG_IMPORT): {}", e);
    }

    Ok(Json(result))
}

/// 匯入豬體重資料
pub async fn import_weight_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "animal.animal.import");

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");

    // 解析 multipart 資料
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("解析檔案欄位失敗: {}", e))
    })? {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());

            let data = field.bytes().await.map_err(|e| {
                AppError::Validation(format!("讀取檔案資料失敗: {}", e))
            })?;

            file_data = Some(data.to_vec());
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("未找到檔案".to_string())
    })?;

    // 檢查檔案大小，限制為 10MB 以內
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("檔案大小不能超過 10MB".to_string()));
    }

    // 執行匯入
    let result = AnimalService::import_weight_data(
        &state.db,
        &file_data,
        &file_name,
        current_user.id,
    )
    .await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "WEIGHT_IMPORT",
        Some("pig_weight"), None,
        Some(&format!("匯入體重資料: {} (成功: {}, 失敗: {})", file_name, result.success_count, result.error_count)),
        None,
        Some(serde_json::json!({
            "file_name": file_name,
            "success_count": result.success_count,
            "error_count": result.error_count,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (WEIGHT_IMPORT): {}", e);
    }

    Ok(Json(result))
}

// ============================================
// 病理報告管理
// ============================================

/// 取得豬的病理報告
pub async fn get_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Option<crate::models::PigPathologyReport>>> {
    require_permission!(current_user, "animal.pathology.view");
    
    let report = AnimalService::get_pathology_report(&state.db, pig_id).await?;
    Ok(Json(report))
}

/// 建立或更新病理報告
pub async fn upsert_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<crate::models::PigPathologyReport>> {
    require_permission!(current_user, "animal.pathology.upload");
    
    let report = AnimalService::upsert_pathology_report(&state.db, pig_id, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "PATHOLOGY_UPSERT",
        Some("pig_pathology"), Some(pig_id),
        Some(&format!("病理報告 (pig: {})", pig_id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (PATHOLOGY_UPSERT): {}", e);
    }

    Ok(Json(report))
}

// ============================================
// 儀表板 API
// ============================================

/// 取得最近的獸醫師評論（儀表板用）
pub async fn get_vet_comments(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>> {
    let limit: i64 = params
        .get("per_page")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    
    // 查詢最近的獸醫建議
    let comments = sqlx::query_as::<_, (i32, Uuid, String, Option<String>, String, chrono::NaiveDateTime, String)>(
        r#"
        SELECT 
            vr.id,
            p.id as pig_id,
            p.ear_tag,
            p.pen_location,
            vr.content,
            vr.created_at,
            u.display_name as created_by_name
        FROM vet_recommendations vr
        INNER JOIN pig_observations po ON vr.record_type = 'observation'::vet_record_type AND vr.record_id = po.id
        INNER JOIN pigs p ON po.pig_id = p.id
        INNER JOIN users u ON vr.created_by = u.id
        ORDER BY vr.created_at DESC
        LIMIT $1
        "#
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    
    let data: Vec<serde_json::Value> = comments
        .into_iter()
        .map(|(id, pig_id, ear_tag, pen_location, content, created_at, created_by_name)| {
            serde_json::json!({
                "id": id.to_string(),
                "pig_id": pig_id.to_string(),
                "pig_ear_tag": ear_tag,
                "pen_location": pen_location,
                "content": content,
                "created_at": created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
                "author_name": created_by_name
            })
        })
        .collect();
    
    Ok(Json(serde_json::json!({ "data": data })))
}

// ============================================
// 血液檢查管理
// ============================================

/// 列出豬的所有血液檢查紀錄
pub async fn list_pig_blood_tests(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
) -> Result<Json<Vec<BloodTestListItem>>> {
    let tests = AnimalService::list_blood_tests(&state.db, pig_id).await?;
    Ok(Json(tests))
}

/// 取得單筆血液檢查（含明細）
pub async fn get_pig_blood_test(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PigBloodTestWithItems>> {
    let test = AnimalService::get_blood_test_by_id(&state.db, id).await?;
    Ok(Json(test))
}

/// 建立血液檢查
pub async fn create_pig_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<Uuid>,
    Json(req): Json<CreateBloodTestRequest>,
) -> Result<Json<PigBloodTestWithItems>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let test = AnimalService::create_blood_test(&state.db, pig_id, &req, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "BLOOD_TEST_CREATE",
        Some("pig_blood_test"), Some(pig_id),
        Some(&format!("血液檢查紀錄 (pig: {})", pig_id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (BLOOD_TEST_CREATE): {}", e);
    }

    Ok(Json(test))
}

/// 更新血液檢查
pub async fn update_pig_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestRequest>,
) -> Result<Json<PigBloodTestWithItems>> {
    require_permission!(current_user, "animal.record.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let test = AnimalService::update_blood_test(&state.db, id, &req).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "BLOOD_TEST_UPDATE",
        Some("pig_blood_test"), None,
        Some(&format!("血液檢查紀錄 #{}", id)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (BLOOD_TEST_UPDATE): {}", e);
    }

    Ok(Json(test))
}

/// 刪除血液檢查（軟刪除 + 刪除原因）
pub async fn delete_pig_blood_test(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    AnimalService::soft_delete_blood_test(&state.db, id, &req.reason, current_user.id).await?;

    // 記錄活動紀錄
    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "BLOOD_TEST_DELETE",
        Some("pig_blood_test"), None,
        Some(&format!("血液檢查紀錄 #{} (原因: {})", id, req.reason)),
        None,
        Some(serde_json::json!({ "reason": req.reason })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (BLOOD_TEST_DELETE): {}", e);
    }

    Ok(Json(serde_json::json!({ "message": "Blood test record deleted successfully" })))
}

// ============================================
// 血液檢查項目模板管理
// ============================================

/// 列出啟用中的血液檢查項目模板
pub async fn list_blood_test_templates(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestTemplate>>> {
    let templates = AnimalService::list_blood_test_templates(&state.db).await?;
    Ok(Json(templates))
}

/// 列出所有模板（含停用）- 管理用
pub async fn list_all_blood_test_templates(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestTemplate>>> {
    let templates = AnimalService::list_all_blood_test_templates(&state.db).await?;
    Ok(Json(templates))
}

/// 建立血液檢查項目模板
pub async fn create_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateBloodTestTemplateRequest>,
) -> Result<Json<BloodTestTemplate>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let template = AnimalService::create_blood_test_template(&state.db, &req).await?;

    Ok(Json(template))
}

/// 更新血液檢查項目模板
pub async fn update_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestTemplateRequest>,
) -> Result<Json<BloodTestTemplate>> {
    require_permission!(current_user, "animal.record.edit");

    let template = AnimalService::update_blood_test_template(&state.db, id, &req).await?;

    Ok(Json(template))
}

/// 刪除血液檢查項目模板（停用）
pub async fn delete_blood_test_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");

    AnimalService::delete_blood_test_template(&state.db, id).await?;

    Ok(Json(serde_json::json!({ "message": "Template deactivated successfully" })))
}

// ============================================
// 血液檢查組合 (Panel) 管理
// ============================================

/// 列出啟用中的血液檢查組合（含項目）
pub async fn list_blood_test_panels(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPanelWithItems>>> {
    let panels = AnimalService::list_blood_test_panels(&state.db).await?;
    Ok(Json(panels))
}

/// 列出所有組合（含停用）- 管理用
pub async fn list_all_blood_test_panels(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<BloodTestPanelWithItems>>> {
    let panels = AnimalService::list_all_blood_test_panels(&state.db).await?;
    Ok(Json(panels))
}

/// 建立血液檢查組合
pub async fn create_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateBloodTestPanelRequest>,
) -> Result<Json<BloodTestPanelWithItems>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let panel = AnimalService::create_blood_test_panel(&state.db, &req).await?;

    Ok(Json(panel))
}

/// 更新血液檢查組合
pub async fn update_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestPanelRequest>,
) -> Result<Json<BloodTestPanelWithItems>> {
    require_permission!(current_user, "animal.record.edit");

    let panel = AnimalService::update_blood_test_panel(&state.db, id, &req).await?;

    Ok(Json(panel))
}

/// 更新組合內的項目
pub async fn update_blood_test_panel_items(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBloodTestPanelItemsRequest>,
) -> Result<Json<BloodTestPanelWithItems>> {
    require_permission!(current_user, "animal.record.edit");

    let panel = AnimalService::update_blood_test_panel_items(&state.db, id, &req).await?;

    Ok(Json(panel))
}

/// 刪除血液檢查組合（停用）
pub async fn delete_blood_test_panel(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");

    AnimalService::delete_blood_test_panel(&state.db, id).await?;

    Ok(Json(serde_json::json!({ "message": "Panel deactivated successfully" })))
}
