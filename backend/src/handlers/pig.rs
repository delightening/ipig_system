use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        BatchAssignRequest, BatchStartExperimentRequest, CreateObservationRequest,
        CreatePigRequest, CreatePigSourceRequest, CreateSacrificeRequest, CreateSurgeryRequest,
        CreateVaccinationRequest, CreateVetRecommendationRequest, CreateWeightRequest, Pig,
        PigListItem, PigObservation, PigQuery, PigSacrifice, PigSource, PigSurgery,
        PigVaccination, PigWeight, PigsByPen, UpdatePigRequest, UpdatePigSourceRequest,
        VetRecommendation, UpdateObservationRequest, UpdateSurgeryRequest, UpdateWeightRequest,
        UpdateVaccinationRequest, CopyRecordRequest, VersionHistoryResponse,
        CreateVetRecommendationWithAttachmentsRequest, ExportRequest, PigImportBatch, ObservationListItem, SurgeryListItem, ImportResult,
    },
    require_permission,
    services::PigService,
    AppError, AppState, Result,
};
use axum::extract::Multipart;

// ============================================
// 豬源管理
// ============================================

/// 列出所有豬源
pub async fn list_pig_sources(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigSource>>> {
    let sources = PigService::list_sources(&state.db).await?;
    Ok(Json(sources))
}

/// 建立豬源
pub async fn create_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "dev.user.create"); // 需要使用者建立權限
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let source = PigService::create_source(&state.db, &req).await?;
    Ok(Json(source))
}

/// 更新豬源
pub async fn update_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "dev.user.edit");
    
    let source = PigService::update_source(&state.db, id, &req).await?;
    Ok(Json(source))
}

/// 刪除豬源
pub async fn delete_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dev.user.delete");
    
    PigService::delete_source(&state.db, id).await?;
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
    let has_view_all = current_user.has_permission("pig.pig.view_all");
    let has_view_project = current_user.has_permission("pig.pig.view_project");
    
    if !has_view_all && !has_view_project {
        // 如果沒有查看權限，返回空列表
        // 這裡不拋出錯誤，而是返回空列表，避免洩露權限資訊
        return Ok(Json(vec![]));
    }
    
    // 如果只有 view_project 權限而沒有 view_all，則只能查看有 iacuc_no 的豬
    // 即只能查看屬於專案的豬，不能查看沒有 iacuc_no 的豬
    let pigs = PigService::list(&state.db, &query).await?;
    
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
    require_permission!(current_user, "pig.pig.view_all");
    
    let pigs = PigService::list_by_pen(&state.db).await?;
    Ok(Json(pigs))
}

/// 取得單個豬的詳細資訊
pub async fn get_pig(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Pig>> {
    let pig = PigService::get_by_id(&state.db, id).await?;
    Ok(Json(pig))
}

/// 建立新豬
pub async fn create_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "pig.pig.create");
    
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
    
    let pig = PigService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(pig))
}

/// 更新豬資訊
pub async fn update_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "pig.pig.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let pig = PigService::update(&state.db, id, &req).await?;
    Ok(Json(pig))
}

/// 刪除豬
pub async fn delete_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.pig.edit");
    
    PigService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Pig deleted successfully" })))
}

/// 批次分配豬的耳標
pub async fn batch_assign_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchAssignRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "pig.pig.assign");
    
    let pigs = PigService::batch_assign(&state.db, &req).await?;
    Ok(Json(pigs))
}

/// 批次開始實驗
pub async fn batch_start_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchStartExperimentRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "pig.pig.edit");
    
    let pigs = PigService::batch_start_experiment(&state.db, &req).await?;
    Ok(Json(pigs))
}

/// 標記豬為獸醫已讀
pub async fn mark_pig_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.vet.read");
    
    PigService::mark_vet_read(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

// ============================================
// 觀察記錄管理
// ============================================

/// 列出豬的所有觀察記錄
pub async fn list_pig_observations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigObservation>>> {
    let observations = PigService::list_observations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 列出豬的觀察記錄（包含獸醫建議）
pub async fn list_pig_observations_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<ObservationListItem>>> {
    let observations = PigService::list_observations_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 取得單個觀察記錄
pub async fn get_pig_observation(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<PigObservation>> {
    let observation = PigService::get_observation_by_id(&state.db, id).await?;
    Ok(Json(observation))
}

/// 建立觀察記錄
pub async fn create_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "pig.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let observation = PigService::create_observation(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(observation))
}

/// 更新觀察記錄
pub async fn update_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "pig.record.edit");
    
    let observation = PigService::update_observation(&state.db, id, &req, current_user.id).await?;
    Ok(Json(observation))
}

/// 刪除觀察記錄（軟刪除）
pub async fn delete_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_observation(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Observation deleted successfully" })))
}

/// 複製觀察記錄
pub async fn copy_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "pig.record.copy");
    
    let observation = PigService::copy_observation(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(observation))
}

/// 標記觀察記錄為獸醫已讀
pub async fn mark_observation_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.vet.read");
    
    PigService::mark_observation_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得觀察記錄的版本歷史
pub async fn get_observation_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = PigService::get_record_versions(&state.db, "observation", id).await?;
    Ok(Json(versions))
}

// ============================================
// 手術記錄管理
// ============================================

/// 列出豬的所有手術記錄
pub async fn list_pig_surgeries(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigSurgery>>> {
    let surgeries = PigService::list_surgeries(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// 列出豬的手術記錄（包含獸醫建議）
pub async fn list_pig_surgeries_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<SurgeryListItem>>> {
    let surgeries = PigService::list_surgeries_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// 取得單個手術記錄
pub async fn get_pig_surgery(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<PigSurgery>> {
    let surgery = PigService::get_surgery_by_id(&state.db, id).await?;
    Ok(Json(surgery))
}

/// 建立手術記錄
pub async fn create_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "pig.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let surgery = PigService::create_surgery(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(surgery))
}

/// 更新手術記錄
pub async fn update_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "pig.record.edit");
    
    let surgery = PigService::update_surgery(&state.db, id, &req, current_user.id).await?;
    Ok(Json(surgery))
}

/// 刪除手術記錄（軟刪除）
pub async fn delete_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_surgery(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Surgery deleted successfully" })))
}

/// 複製手術記錄
pub async fn copy_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "pig.record.copy");
    
    let surgery = PigService::copy_surgery(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 標記手術記錄為獸醫已讀
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.vet.read");
    
    PigService::mark_surgery_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得手術記錄的版本歷史
pub async fn get_surgery_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = PigService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}

// ============================================
// 體重記錄管理
// ============================================

/// 列出豬的所有體重記錄
pub async fn list_pig_weights(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigWeight>>> {
    let weights = PigService::list_weights(&state.db, pig_id).await?;
    Ok(Json(weights))
}

/// 建立體重記錄
pub async fn create_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "pig.record.create");
    
    let weight = PigService::create_weight(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(weight))
}

/// 更新體重記錄
pub async fn update_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "pig.record.edit");
    
    let weight = PigService::update_weight(&state.db, id, &req).await?;
    Ok(Json(weight))
}

/// 刪除體重記錄（軟刪除）
pub async fn delete_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_weight(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Weight record deleted successfully" })))
}

// ============================================
// 疫苗接種記錄管理
// ============================================

/// 列出豬的所有疫苗接種記錄
pub async fn list_pig_vaccinations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigVaccination>>> {
    let vaccinations = PigService::list_vaccinations(&state.db, pig_id).await?;
    Ok(Json(vaccinations))
}

/// 建立疫苗接種記錄
pub async fn create_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "pig.record.create");
    
    let vaccination = PigService::create_vaccination(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(vaccination))
}

/// 更新疫苗接種記錄
pub async fn update_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "pig.record.edit");
    
    let vaccination = PigService::update_vaccination(&state.db, id, &req).await?;
    Ok(Json(vaccination))
}

/// 刪除疫苗接種記錄（軟刪除）
pub async fn delete_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_vaccination(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Vaccination record deleted successfully" })))
}

// ============================================
// 犧牲/安樂死記錄管理
// ============================================

/// 取得豬的犧牲記錄
pub async fn get_pig_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Option<PigSacrifice>>> {
    let sacrifice = PigService::get_sacrifice(&state.db, pig_id).await?;
    Ok(Json(sacrifice))
}

/// 建立或更新犧牲記錄
pub async fn upsert_pig_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<PigSacrifice>> {
    require_permission!(current_user, "pig.record.create");
    
    let sacrifice = PigService::upsert_sacrifice(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(sacrifice))
}

// ============================================
// 獸醫建議管理
// ============================================

/// 為觀察記錄新增獸醫建議
pub async fn add_observation_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "pig.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation(&state.db, "observation", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 為手術記錄新增獸醫建議
pub async fn add_surgery_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "pig.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation(&state.db, "surgery", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 為觀察記錄新增獸醫建議（帶附件）
pub async fn add_observation_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "pig.vet.recommend");
    require_permission!(current_user, "pig.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation_with_attachments(&state.db, "observation", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 為手術記錄新增獸醫建議（帶附件）
pub async fn add_surgery_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "pig.vet.recommend");
    require_permission!(current_user, "pig.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation_with_attachments(&state.db, "surgery", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 取得觀察記錄的所有獸醫建議
pub async fn get_observation_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = PigService::get_vet_recommendations(&state.db, "observation", id).await?;
    Ok(Json(recommendations))
}

/// 取得手術記錄的所有獸醫建議
pub async fn get_surgery_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = PigService::get_vet_recommendations(&state.db, "surgery", id).await?;
    Ok(Json(recommendations))
}

// ============================================
// 匯入匯出
// ============================================

/// 匯出豬的醫療資料
pub async fn export_pig_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.export.medical");
    
    // 取得醫療資料
    let data = PigService::get_pig_medical_data(&state.db, pig_id).await?;
    
    // 建立匯出記錄
    let _record = PigService::create_export_record(
        &state.db,
        Some(pig_id),
        None,
        req.export_type,
        req.format,
        None,
        current_user.id,
    ).await?;
    
    // 返回資料，前端負責轉換為 PDF/Excel 格式並下載
    Ok(Json(serde_json::json!({
        "data": data,
        "format": req.format,
        "export_type": req.export_type,
    })))
}

/// 匯出專案的醫療資料
pub async fn export_project_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(iacuc_no): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.export.medical");
    
    // 取得專案下所有豬的醫療資料
    let data = PigService::get_project_medical_data(&state.db, &iacuc_no).await?;
    
    // 建立匯出記錄
    let _record = PigService::create_export_record(
        &state.db,
        None,
        Some(&iacuc_no),
        req.export_type,
        req.format,
        None,
        current_user.id,
    ).await?;
    
    Ok(Json(serde_json::json!({
        "data": data,
        "format": req.format,
        "export_type": req.export_type,
    })))
}

/// 列出所有匯入批次
pub async fn list_import_batches(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigImportBatch>>> {
    require_permission!(current_user, "pig.pig.import");
    
    let batches = PigService::list_import_batches(&state.db, 50).await?;
    Ok(Json(batches))
}

/// 下載豬基礎資料匯入範本
pub async fn download_basic_import_template(
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response> {
    require_permission!(current_user, "pig.pig.import");
    
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("xlsx");
    
    let (data, filename, content_type) = if format == "csv" {
        let csv_data = PigService::generate_basic_import_template_csv()?;
        (
            csv_data,
            "pig_basic_import_template.csv",
            "text/csv; charset=utf-8",
        )
    } else {
        let excel_data = PigService::generate_basic_import_template()?;
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
    require_permission!(current_user, "pig.pig.import");
    
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("xlsx");
    
    let (data, filename, content_type) = if format == "csv" {
        let csv_data = PigService::generate_weight_import_template_csv()?;
        (
            csv_data,
            "pig_weight_import_template.csv",
            "text/csv; charset=utf-8",
        )
    } else {
        let excel_data = PigService::generate_weight_import_template()?;
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
    require_permission!(current_user, "pig.pig.import");

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
    let result = PigService::import_basic_data(
        &state.db,
        &file_data,
        &file_name,
        current_user.id,
    )
    .await?;

    Ok(Json(result))
}

/// 匯入豬體重資料
pub async fn import_weight_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "pig.pig.import");

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
    let result = PigService::import_weight_data(
        &state.db,
        &file_data,
        &file_name,
        current_user.id,
    )
    .await?;

    Ok(Json(result))
}

// ============================================
// 病理報告管理
// ============================================

/// 取得豬的病理報告
pub async fn get_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Option<crate::models::PigPathologyReport>>> {
    require_permission!(current_user, "pig.pathology.view");
    
    let report = PigService::get_pathology_report(&state.db, pig_id).await?;
    Ok(Json(report))
}

/// 建立或更新病理報告
pub async fn upsert_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<crate::models::PigPathologyReport>> {
    require_permission!(current_user, "pig.pathology.upload");
    
    let report = PigService::upsert_pathology_report(&state.db, pig_id, current_user.id).await?;
    Ok(Json(report))
}
