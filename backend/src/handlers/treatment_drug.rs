// 治療方式藥物選項 Handler

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateTreatmentDrugRequest, ImportFromErpRequest, TreatmentDrugOption, TreatmentDrugQuery,
        UpdateTreatmentDrugRequest,
    },
    require_permission,
    services::TreatmentDrugService,
    AppState, Result,
};

/// 列出啟用藥物選項（供動物醫療紀錄的 DrugCombobox 使用）
///
/// F1（pentest 2026-07-05）：此清單為內部醫療用藥參考，修補前無 gate → 外部
/// CLIENT 等任何已認證者可讀藥物處方（劑量單位/值）。改要求 `animal.animal.view_all`
/// ——DrugCombobox 的消費者（EXPERIMENT_STAFF / VET）皆持該權，外部 CLIENT
/// （僅 `animal.animal.view_project`）被擋。
#[utoipa::path(get, path = "/api/v1/treatment-drugs", responses((status = 200)), tag = "治療藥物", security(("bearer" = [])))]
pub async fn list_treatment_drugs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<TreatmentDrugOption>>> {
    require_permission!(current_user, "animal.animal.view_all");
    let service = TreatmentDrugService::new(state.db.clone());
    let options = service.list_active().await?;
    Ok(Json(options))
}

/// 列出所有藥物選項（管理員，含篩選）
#[utoipa::path(get, path = "/api/v1/admin/treatment-drugs", params(TreatmentDrugQuery), responses((status = 200)), tag = "治療藥物", security(("bearer" = [])))]
pub async fn admin_list_treatment_drugs(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<TreatmentDrugQuery>,
) -> Result<Json<Vec<TreatmentDrugOption>>> {
    require_permission!(current_user, "admin.treatment_drug.view");
    let service = TreatmentDrugService::new(state.db.clone());
    let options = service.list(query).await?;
    Ok(Json(options))
}

/// 建立藥物選項（管理員）
#[utoipa::path(post, path = "/api/v1/admin/treatment-drugs", request_body = CreateTreatmentDrugRequest, responses((status = 201)), tag = "治療藥物", security(("bearer" = [])))]
pub async fn create_treatment_drug(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateTreatmentDrugRequest>,
) -> Result<(StatusCode, Json<TreatmentDrugOption>)> {
    require_permission!(current_user, "admin.treatment_drug.create");
    request.validate()?;

    let service = TreatmentDrugService::new(state.db.clone());
    let option = service.create(request, Some(current_user.id)).await?;
    Ok((StatusCode::CREATED, Json(option)))
}

/// 更新藥物選項（管理員）
#[utoipa::path(put, path = "/api/v1/admin/treatment-drugs/{id}", params(("id" = Uuid, Path, description = "藥物選項 ID")), request_body = UpdateTreatmentDrugRequest, responses((status = 200)), tag = "治療藥物", security(("bearer" = [])))]
pub async fn update_treatment_drug(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTreatmentDrugRequest>,
) -> Result<Json<TreatmentDrugOption>> {
    require_permission!(current_user, "admin.treatment_drug.edit");
    let service = TreatmentDrugService::new(state.db.clone());
    let option = service.update(id, request).await?;
    Ok(Json(option))
}

/// 刪除藥物選項（管理員，軟刪除）
#[utoipa::path(delete, path = "/api/v1/admin/treatment-drugs/{id}", params(("id" = Uuid, Path, description = "藥物選項 ID")), responses((status = 204)), tag = "治療藥物", security(("bearer" = [])))]
pub async fn delete_treatment_drug(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "admin.treatment_drug.delete");
    let service = TreatmentDrugService::new(state.db.clone());
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 從 ERP 匯入藥物選項（管理員）
#[utoipa::path(post, path = "/api/v1/admin/treatment-drugs/import-from-erp", request_body = ImportFromErpRequest, responses((status = 201)), tag = "治療藥物", security(("bearer" = [])))]
pub async fn import_treatment_drugs_from_erp(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<ImportFromErpRequest>,
) -> Result<(StatusCode, Json<Vec<TreatmentDrugOption>>)> {
    require_permission!(current_user, "admin.treatment_drug.create");
    let service = TreatmentDrugService::new(state.db.clone());
    let imported = service
        .import_from_erp(request, Some(current_user.id))
        .await?;
    Ok((StatusCode::CREATED, Json(imported)))
}
