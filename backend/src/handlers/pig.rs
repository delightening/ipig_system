use axum::{
    extract::{Path, Query, State},
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
        CreateVetRecommendationWithAttachmentsRequest, ExportRequest, PigImportBatch,
        PigExportRecord, ObservationListItem, SurgeryListItem,
    },
    require_permission,
    services::PigService,
    AppError, AppState, Result,
};

// ============================================
// 豬隻來源
// ============================================

/// 取得豬隻來源列表
pub async fn list_pig_sources(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigSource>>> {
    let sources = PigService::list_sources(&state.db).await?;
    Ok(Json(sources))
}

/// 建立豬隻來源
pub async fn create_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "admin.user.create"); // 只有管理員可以建立
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let source = PigService::create_source(&state.db, &req).await?;
    Ok(Json(source))
}

/// 更新豬隻來源
pub async fn update_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "admin.user.edit");
    
    let source = PigService::update_source(&state.db, id, &req).await?;
    Ok(Json(source))
}

/// 刪除豬隻來源
pub async fn delete_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "admin.user.delete");
    
    PigService::delete_source(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Pig source deleted successfully" })))
}

// ============================================
// 豬隻管理
// ============================================

/// 取得豬隻列表
pub async fn list_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PigQuery>,
) -> Result<Json<Vec<PigListItem>>> {
    // 檢查權限
    let has_view_all = current_user.permissions.contains(&"animal.pig.view_all".to_string());
    let has_view_project = current_user.permissions.contains(&"animal.pig.view_project".to_string());
    
    if !has_view_all && !has_view_project {
        return Err(AppError::Forbidden("You don't have permission to view pigs".to_string()));
    }
    
    let pigs = PigService::list(&state.db, &query).await?;
    Ok(Json(pigs))
}

/// 依欄位分組取得豬隻
pub async fn list_pigs_by_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigsByPen>>> {
    require_permission!(current_user, "animal.pig.view_all");
    
    let pigs = PigService::list_by_pen(&state.db).await?;
    Ok(Json(pigs))
}

/// 取得單一豬隻
pub async fn get_pig(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Pig>> {
    let pig = PigService::get_by_id(&state.db, id).await?;
    Ok(Json(pig))
}

/// 建立豬隻
pub async fn create_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "animal.pig.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let pig = PigService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(pig))
}

/// 更新豬隻
pub async fn update_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "animal.pig.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let pig = PigService::update(&state.db, id, &req).await?;
    Ok(Json(pig))
}

/// 刪除豬隻
pub async fn delete_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.pig.edit");
    
    PigService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Pig deleted successfully" })))
}

/// 批次分配豬隻至計劃
pub async fn batch_assign_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchAssignRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "animal.pig.assign");
    
    let pigs = PigService::batch_assign(&state.db, &req).await?;
    Ok(Json(pigs))
}

/// 批次進入實驗
pub async fn batch_start_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchStartExperimentRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "animal.pig.edit");
    
    let pigs = PigService::batch_start_experiment(&state.db, &req).await?;
    Ok(Json(pigs))
}

/// 標記獸醫師已讀
pub async fn mark_pig_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    PigService::mark_vet_read(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

// ============================================
// 觀察試驗紀錄
// ============================================

/// 取得觀察試驗紀錄列表
pub async fn list_pig_observations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigObservation>>> {
    let observations = PigService::list_observations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 取得觀察試驗紀錄列表（含獸醫師建議數量）
pub async fn list_pig_observations_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<ObservationListItem>>> {
    let observations = PigService::list_observations_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// 取得單一觀察紀錄
pub async fn get_pig_observation(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<PigObservation>> {
    let observation = PigService::get_observation_by_id(&state.db, id).await?;
    Ok(Json(observation))
}

/// 建立觀察試驗紀錄
pub async fn create_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let observation = PigService::create_observation(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(observation))
}

/// 更新觀察試驗紀錄
pub async fn update_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.edit");
    
    let observation = PigService::update_observation(&state.db, id, &req, current_user.id).await?;
    Ok(Json(observation))
}

/// 軟刪除觀察試驗紀錄
pub async fn delete_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    
    PigService::soft_delete_observation(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Observation deleted successfully" })))
}

/// 複製觀察試驗紀錄
pub async fn copy_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "animal.record.copy");
    
    let observation = PigService::copy_observation(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(observation))
}

/// 標記觀察紀錄獸醫師已讀
pub async fn mark_observation_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    PigService::mark_observation_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得觀察紀錄版本歷史
pub async fn get_observation_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = PigService::get_record_versions(&state.db, "observation", id).await?;
    Ok(Json(versions))
}

// ============================================
// 手術紀錄
// ============================================

/// 取得手術紀錄列表
pub async fn list_pig_surgeries(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigSurgery>>> {
    let surgeries = PigService::list_surgeries(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// 取得手術紀錄列表（含獸醫師建議數量）
pub async fn list_pig_surgeries_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<SurgeryListItem>>> {
    let surgeries = PigService::list_surgeries_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// 取得單一手術紀錄
pub async fn get_pig_surgery(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<PigSurgery>> {
    let surgery = PigService::get_surgery_by_id(&state.db, id).await?;
    Ok(Json(surgery))
}

/// 建立手術紀錄
pub async fn create_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "animal.record.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let surgery = PigService::create_surgery(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(surgery))
}

/// 更新手術紀錄
pub async fn update_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "animal.record.edit");
    
    let surgery = PigService::update_surgery(&state.db, id, &req, current_user.id).await?;
    Ok(Json(surgery))
}

/// 軟刪除手術紀錄
pub async fn delete_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    
    PigService::soft_delete_surgery(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Surgery deleted successfully" })))
}

/// 複製手術紀錄
pub async fn copy_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "animal.record.copy");
    
    let surgery = PigService::copy_surgery(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 標記手術紀錄獸醫師已讀
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.vet.read");
    
    PigService::mark_surgery_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// 取得手術紀錄版本歷史
pub async fn get_surgery_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = PigService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}

// ============================================
// 體重紀錄
// ============================================

/// 取得體重紀錄列表
pub async fn list_pig_weights(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigWeight>>> {
    let weights = PigService::list_weights(&state.db, pig_id).await?;
    Ok(Json(weights))
}

/// 建立體重紀錄
pub async fn create_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "animal.record.create");
    
    let weight = PigService::create_weight(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(weight))
}

/// 更新體重紀錄
pub async fn update_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "animal.record.edit");
    
    let weight = PigService::update_weight(&state.db, id, &req).await?;
    Ok(Json(weight))
}

/// 軟刪除體重紀錄
pub async fn delete_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    
    PigService::soft_delete_weight(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Weight record deleted successfully" })))
}

// ============================================
// 疫苗/驅蟲紀錄
// ============================================

/// 取得疫苗/驅蟲紀錄列表
pub async fn list_pig_vaccinations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigVaccination>>> {
    let vaccinations = PigService::list_vaccinations(&state.db, pig_id).await?;
    Ok(Json(vaccinations))
}

/// 建立疫苗/驅蟲紀錄
pub async fn create_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "animal.record.create");
    
    let vaccination = PigService::create_vaccination(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(vaccination))
}

/// 更新疫苗/驅蟲紀錄
pub async fn update_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "animal.record.edit");
    
    let vaccination = PigService::update_vaccination(&state.db, id, &req).await?;
    Ok(Json(vaccination))
}

/// 軟刪除疫苗/驅蟲紀錄
pub async fn delete_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    
    PigService::soft_delete_vaccination(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Vaccination record deleted successfully" })))
}

// ============================================
// 犧牲/採樣紀錄
// ============================================

/// 取得犧牲/採樣紀錄
pub async fn get_pig_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Option<PigSacrifice>>> {
    let sacrifice = PigService::get_sacrifice(&state.db, pig_id).await?;
    Ok(Json(sacrifice))
}

/// 建立/更新犧牲/採樣紀錄
pub async fn upsert_pig_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<PigSacrifice>> {
    require_permission!(current_user, "animal.record.create");
    
    let sacrifice = PigService::upsert_sacrifice(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(sacrifice))
}

// ============================================
// 獸醫師建議
// ============================================

/// 新增觀察紀錄的獸醫師建議
pub async fn add_observation_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation(&state.db, "observation", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 新增手術紀錄的獸醫師建議
pub async fn add_surgery_vet_recommendation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation(&state.db, "surgery", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 新增觀察紀錄的獸醫師建議（含附件）
pub async fn add_observation_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    require_permission!(current_user, "animal.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation_with_attachments(&state.db, "observation", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 新增手術紀錄的獸醫師建議（含附件）
pub async fn add_surgery_vet_recommendation_with_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<CreateVetRecommendationWithAttachmentsRequest>,
) -> Result<Json<VetRecommendation>> {
    require_permission!(current_user, "animal.vet.recommend");
    require_permission!(current_user, "animal.vet.upload_attachment");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let recommendation = PigService::add_vet_recommendation_with_attachments(&state.db, "surgery", id, &req, current_user.id).await?;
    Ok(Json(recommendation))
}

/// 取得觀察紀錄的獸醫師建議
pub async fn get_observation_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = PigService::get_vet_recommendations(&state.db, "observation", id).await?;
    Ok(Json(recommendations))
}

/// 取得手術紀錄的獸醫師建議
pub async fn get_surgery_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = PigService::get_vet_recommendations(&state.db, "surgery", id).await?;
    Ok(Json(recommendations))
}

// ============================================
// 匯出功能
// ============================================

/// 匯出豬隻病歷資料
pub async fn export_pig_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.export.medical");
    
    // 取得病歷資料
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
    
    // 返回資料（實際 PDF/Excel 生成需要前端處理或後端另外實作）
    Ok(Json(serde_json::json!({
        "data": data,
        "format": req.format,
        "export_type": req.export_type,
    })))
}

/// 匯出計畫病歷資料
pub async fn export_project_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(iacuc_no): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.export.medical");
    
    // 取得計畫所有豬隻病歷資料
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

/// 取得匯入批次歷史
pub async fn list_import_batches(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigImportBatch>>> {
    require_permission!(current_user, "animal.pig.import");
    
    let batches = PigService::list_import_batches(&state.db, 50).await?;
    Ok(Json(batches))
}

// ============================================
// 病理報告
// ============================================

/// 取得病理報告
pub async fn get_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Option<crate::models::PigPathologyReport>>> {
    require_permission!(current_user, "animal.pathology.view");
    
    let report = PigService::get_pathology_report(&state.db, pig_id).await?;
    Ok(Json(report))
}

/// 建立/更新病理報告
pub async fn upsert_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<crate::models::PigPathologyReport>> {
    require_permission!(current_user, "animal.pathology.upload");
    
    let report = PigService::upsert_pathology_report(&state.db, pig_id, current_user.id).await?;
    Ok(Json(report))
}
