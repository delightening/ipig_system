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
        CreateVetRecommendationWithAttachmentsRequest, ExportRequest, PigImportBatch,
        PigExportRecord, ObservationListItem, SurgeryListItem, ImportResult,
    },
    require_permission,
    services::PigService,
    AppError, AppState, Result,
};
use axum::extract::Multipart;

// ============================================
// 鞊祇靘?
// ============================================

/// ??鞊祇靘??”
pub async fn list_pig_sources(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigSource>>> {
    let sources = PigService::list_sources(&state.db).await?;
    Ok(Json(sources))
}

/// 撱箇?鞊祇靘?
pub async fn create_pig_source(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigSourceRequest>,
) -> Result<Json<PigSource>> {
    require_permission!(current_user, "dev.user.create"); // ?芣?蝞∠??∪隞亙遣蝡?    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let source = PigService::create_source(&state.db, &req).await?;
    Ok(Json(source))
}

/// ?湔鞊祇靘?
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

/// ?芷鞊祇靘?
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
// 鞊祇蝞∠?
// ============================================

/// ??鞊祇?”
pub async fn list_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PigQuery>,
) -> Result<Json<Vec<PigListItem>>> {
    // 瑼Ｘ甈?
    let has_view_all = current_user.has_permission("pig.pig.view_all");
    let has_view_project = current_user.has_permission("pig.pig.view_project");
    
    if !has_view_all && !has_view_project {
        // 憒??冽瘝?隞颱??亦?甈?嚗??征?”???舫隤?        // ?見?垢?臭誑甇?虜憿舐內嚗?舀?????        return Ok(Json(vec![]));
    }
    
    // 憒???view_project 甈?雿???view_all嚗?閬?瞈曉憿舐內閮?抒?鞊祇
    // ?桀??????惇?鳴?敺??臭誑?寞? iacuc_no ?蕪
    let pigs = PigService::list(&state.db, &query).await?;
    
    // 憒??芣? view_project 甈?嚗?瞈曉憿舐內??iacuc_no ?惇??    // 嚗?閮剜? iacuc_no ?惇?餃惇?潭????恬?
    let filtered_pigs = if has_view_all {
        pigs
    } else {
        // ?芷＊蝷箏歇??蝯西??怎?鞊祇嚗? iacuc_no嚗?        pigs.into_iter()
            .filter(|pig| pig.iacuc_no.is_some())
            .collect()
    };
    
    Ok(Json(filtered_pigs))
}

/// 靘?雿?蝯?敺惇??pub async fn list_pigs_by_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigsByPen>>> {
    require_permission!(current_user, "pig.pig.view_all");
    
    let pigs = PigService::list_by_pen(&state.db).await?;
    Ok(Json(pigs))
}

/// ???桐?鞊祇
pub async fn get_pig(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Pig>> {
    let pig = PigService::get_by_id(&state.db, id).await?;
    Ok(Json(pig))
}

/// 撱箇?鞊祇
pub async fn create_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePigRequest>,
) -> Result<Json<Pig>> {
    require_permission!(current_user, "pig.pig.create");
    
    // 閮??交?啁?隢?嚗?潸矽閰佗?
    tracing::debug!("Create pig request: ear_tag={}, breed={:?}, gender={:?}, entry_date={:?}, birth_date={:?}, entry_weight={:?}", 
        req.ear_tag, req.breed, req.gender, req.entry_date, req.birth_date, req.entry_weight);
    
    // 撽?隢?
    if let Err(validation_errors) = req.validate() {
        let error_messages: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    let field_name = match *field {
                        "ear_tag" => "?唾?",
                        "breed" => "?車",
                        "gender" => "?批",
                        "entry_date" => "?脣?交?",
                        "birth_date" => "?箇??交?",
                        "entry_weight" => "?脣擃?",
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

/// ?湔鞊祇
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

/// ?芷鞊祇
pub async fn delete_pig(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.pig.edit");
    
    PigService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Pig deleted successfully" })))
}

/// ?寞活??鞊祇?唾???pub async fn batch_assign_pigs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchAssignRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "pig.pig.assign");
    
    let pigs = PigService::batch_assign(&state.db, &req).await?;
    Ok(Json(pigs))
}

/// ?寞活?脣撖阡?
pub async fn batch_start_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<BatchStartExperimentRequest>,
) -> Result<Json<Vec<Pig>>> {
    require_permission!(current_user, "pig.pig.edit");
    
    let pigs = PigService::batch_start_experiment(&state.db, &req).await?;
    Ok(Json(pigs))
}

/// 璅??賊撣怠歇霈
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
// 閫撖岫撽???// ============================================

/// ??閫撖岫撽???銵?pub async fn list_pig_observations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigObservation>>> {
    let observations = PigService::list_observations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// ??閫撖岫撽???銵剁??怎?怠葦撱箄降?賊?嚗?pub async fn list_pig_observations_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<ObservationListItem>>> {
    let observations = PigService::list_observations_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(observations))
}

/// ???桐?閫撖???pub async fn get_pig_observation(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<PigObservation>> {
    let observation = PigService::get_observation_by_id(&state.db, id).await?;
    Ok(Json(observation))
}

/// 撱箇?閫撖岫撽???pub async fn create_pig_observation(
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

/// ?湔閫撖岫撽???pub async fn update_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateObservationRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "pig.record.edit");
    
    let observation = PigService::update_observation(&state.db, id, &req, current_user.id).await?;
    Ok(Json(observation))
}

/// 頠?方?撖岫撽???pub async fn delete_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_observation(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Observation deleted successfully" })))
}

/// 銴ˊ閫撖岫撽???pub async fn copy_pig_observation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigObservation>> {
    require_permission!(current_user, "pig.record.copy");
    
    let observation = PigService::copy_observation(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(observation))
}

/// 璅?閫撖???怠葦撌脰?
pub async fn mark_observation_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.vet.read");
    
    PigService::mark_observation_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// ??閫撖????祆風??pub async fn get_observation_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = PigService::get_record_versions(&state.db, "observation", id).await?;
    Ok(Json(versions))
}

// ============================================
// ??蝝??// ============================================

/// ????蝝??銵?pub async fn list_pig_surgeries(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigSurgery>>> {
    let surgeries = PigService::list_surgeries(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// ????蝝??銵剁??怎?怠葦撱箄降?賊?嚗?pub async fn list_pig_surgeries_with_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<SurgeryListItem>>> {
    let surgeries = PigService::list_surgeries_with_recommendations(&state.db, pig_id).await?;
    Ok(Json(surgeries))
}

/// ???桐???蝝??pub async fn get_pig_surgery(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<PigSurgery>> {
    let surgery = PigService::get_surgery_by_id(&state.db, id).await?;
    Ok(Json(surgery))
}

/// 撱箇???蝝??pub async fn create_pig_surgery(
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

/// ?湔??蝝??pub async fn update_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateSurgeryRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "pig.record.edit");
    
    let surgery = PigService::update_surgery(&state.db, id, &req, current_user.id).await?;
    Ok(Json(surgery))
}

/// 頠?斗?銵???pub async fn delete_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_surgery(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Surgery deleted successfully" })))
}

/// 銴ˊ??蝝??pub async fn copy_pig_surgery(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CopyRecordRequest>,
) -> Result<Json<PigSurgery>> {
    require_permission!(current_user, "pig.record.copy");
    
    let surgery = PigService::copy_surgery(&state.db, pig_id, req.source_id, current_user.id).await?;
    Ok(Json(surgery))
}

/// 璅???蝝??怠葦撌脰?
pub async fn mark_surgery_vet_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.vet.read");
    
    PigService::mark_surgery_vet_read(&state.db, id, current_user.id).await?;
    Ok(Json(serde_json::json!({ "message": "Marked as read" })))
}

/// ????蝝???祆風??pub async fn get_surgery_versions(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<VersionHistoryResponse>> {
    let versions = PigService::get_record_versions(&state.db, "surgery", id).await?;
    Ok(Json(versions))
}

// ============================================
// 擃?蝝??// ============================================

/// ??擃?蝝??銵?pub async fn list_pig_weights(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigWeight>>> {
    let weights = PigService::list_weights(&state.db, pig_id).await?;
    Ok(Json(weights))
}

/// 撱箇?擃?蝝??pub async fn create_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "pig.record.create");
    
    let weight = PigService::create_weight(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(weight))
}

/// ?湔擃?蝝??pub async fn update_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateWeightRequest>,
) -> Result<Json<PigWeight>> {
    require_permission!(current_user, "pig.record.edit");
    
    let weight = PigService::update_weight(&state.db, id, &req).await?;
    Ok(Json(weight))
}

/// 頠?日?????pub async fn delete_pig_weight(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_weight(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Weight record deleted successfully" })))
}

// ============================================
// ?怨?/撽蝝??// ============================================

/// ???怨?/撽蝝??銵?pub async fn list_pig_vaccinations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Vec<PigVaccination>>> {
    let vaccinations = PigService::list_vaccinations(&state.db, pig_id).await?;
    Ok(Json(vaccinations))
}

/// 撱箇??怨?/撽蝝??pub async fn create_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<CreateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "pig.record.create");
    
    let vaccination = PigService::create_vaccination(&state.db, pig_id, &req, current_user.id).await?;
    Ok(Json(vaccination))
}

/// ?湔?怨?/撽蝝??pub async fn update_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateVaccinationRequest>,
) -> Result<Json<PigVaccination>> {
    require_permission!(current_user, "pig.record.edit");
    
    let vaccination = PigService::update_vaccination(&state.db, id, &req).await?;
    Ok(Json(vaccination))
}

/// 頠?斤??撽蝝??pub async fn delete_pig_vaccination(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.record.delete");
    
    PigService::soft_delete_vaccination(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Vaccination record deleted successfully" })))
}

// ============================================
// ?抒/?⊥見蝝??// ============================================

/// ???抒/?⊥見蝝??pub async fn get_pig_sacrifice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Option<PigSacrifice>>> {
    let sacrifice = PigService::get_sacrifice(&state.db, pig_id).await?;
    Ok(Json(sacrifice))
}

/// 撱箇?/?湔?抒/?⊥見蝝??pub async fn upsert_pig_sacrifice(
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
// ?賊撣怠遣霅?// ============================================

/// ?啣?閫撖????賊撣怠遣霅?pub async fn add_observation_vet_recommendation(
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

/// ?啣???蝝???賊撣怠遣霅?pub async fn add_surgery_vet_recommendation(
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

/// ?啣?閫撖????賊撣怠遣霅堆??恍?隞塚?
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

/// ?啣???蝝???賊撣怠遣霅堆??恍?隞塚?
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

/// ??閫撖????賊撣怠遣霅?pub async fn get_observation_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = PigService::get_vet_recommendations(&state.db, "observation", id).await?;
    Ok(Json(recommendations))
}

/// ????蝝???賊撣怠遣霅?pub async fn get_surgery_vet_recommendations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<VetRecommendation>>> {
    let recommendations = PigService::get_vet_recommendations(&state.db, "surgery", id).await?;
    Ok(Json(recommendations))
}

// ============================================
// ?臬?
// ============================================

/// ?臬鞊祇?風鞈?
pub async fn export_pig_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.export.medical");
    
    // ???風鞈?
    let data = PigService::get_pig_medical_data(&state.db, pig_id).await?;
    
    // 撱箇??臬閮?
    let _record = PigService::create_export_record(
        &state.db,
        Some(pig_id),
        None,
        req.export_type,
        req.format,
        None,
        current_user.id,
    ).await?;
    
    // 餈?鞈?嚗祕??PDF/Excel ???閬?蝡航???敺垢?血?撖虫?嚗?    Ok(Json(serde_json::json!({
        "data": data,
        "format": req.format,
        "export_type": req.export_type,
    })))
}

/// ?臬閮?風鞈?
pub async fn export_project_medical_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(iacuc_no): Path<String>,
    Json(req): Json<ExportRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "pig.export.medical");
    
    // ??閮??惇?餌?甇瑁???    let data = PigService::get_project_medical_data(&state.db, &iacuc_no).await?;
    
    // 撱箇??臬閮?
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

/// ???臬?寞活甇瑕
pub async fn list_import_batches(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PigImportBatch>>> {
    require_permission!(current_user, "pig.pig.import");
    
    let batches = PigService::list_import_batches(&state.db, 50).await?;
    Ok(Json(batches))
}

/// 銝?鞊祇?箸鞈??臬璅⊥
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

/// 銝?鞊祇擃??臬璅⊥
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

/// ?臬鞊祇?箸鞈?
pub async fn import_basic_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "pig.pig.import");

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");

    // 閫?? multipart 鞈?
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("?⊥?霈???單?獢? {}", e))
    })? {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());

            let data = field.bytes().await.map_err(|e| {
                AppError::Validation(format!("?⊥?霈??獢??? {}", e))
            })?;

            file_data = Some(data.to_vec());
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("?芣?靘?獢?.to_string())
    })?;

    // 瑼Ｘ瑼?憭批?嚗?0MB ?嚗?    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("瑼?憭批?頞? 10MB ?".to_string()));
    }

    // ?瑁??臬
    let result = PigService::import_basic_data(
        &state.db,
        &file_data,
        &file_name,
        current_user.id,
    )
    .await?;

    Ok(Json(result))
}

/// ?臬鞊祇擃?鞈?
pub async fn import_weight_data(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>> {
    require_permission!(current_user, "pig.pig.import");

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name = String::from("unknown");

    // 閫?? multipart 鞈?
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("?⊥?霈???單?獢? {}", e))
    })? {
        if field.name() == Some("file") {
            file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());

            let data = field.bytes().await.map_err(|e| {
                AppError::Validation(format!("?⊥?霈??獢??? {}", e))
            })?;

            file_data = Some(data.to_vec());
        }
    }

    let file_data = file_data.ok_or_else(|| {
        AppError::Validation("?芣?靘?獢?.to_string())
    })?;

    // 瑼Ｘ瑼?憭批?嚗?0MB ?嚗?    if file_data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation("瑼?憭批?頞? 10MB ?".to_string()));
    }

    // ?瑁??臬
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
// ???勗?
// ============================================

/// ?????勗?
pub async fn get_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<Option<crate::models::PigPathologyReport>>> {
    require_permission!(current_user, "pig.pathology.view");
    
    let report = PigService::get_pathology_report(&state.db, pig_id).await?;
    Ok(Json(report))
}

/// 撱箇?/?湔???勗?
pub async fn upsert_pig_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
) -> Result<Json<crate::models::PigPathologyReport>> {
    require_permission!(current_user, "pig.pathology.upload");
    
    let report = PigService::upsert_pathology_report(&state.db, pig_id, current_user.id).await?;
    Ok(Json(report))
}


