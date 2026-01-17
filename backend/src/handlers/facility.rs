// Facility Handlers
// 包含：Species, Facility, Building, Zone, Pen, Department

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        Building, BuildingWithFacility, CreateBuildingRequest, CreateDepartmentRequest,
        CreateFacilityRequest, CreatePenRequest, CreateSpeciesRequest, CreateZoneRequest,
        Department, DepartmentWithManager, Facility, Pen, PenDetails, PenQuery, Species,
        UpdateBuildingRequest, UpdateDepartmentRequest, UpdateFacilityRequest, UpdatePenRequest,
        UpdateSpeciesRequest, UpdateZoneRequest, Zone, ZoneWithBuilding,
    },
    services::FacilityService,
    AppState, Result,
};

// ============================================
// Species Handlers
// ============================================

/// 列出所有物種
pub async fn list_species(
    State(state): State<AppState>,
) -> Result<Json<Vec<Species>>> {
    let species = FacilityService::list_species(&state.db).await?;
    Ok(Json(species))
}

/// 取得物種詳細
pub async fn get_species(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Species>> {
    let species = FacilityService::get_species(&state.db, id).await?;
    Ok(Json(species))
}

/// 建立物種
pub async fn create_species(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateSpeciesRequest>,
) -> Result<(StatusCode, Json<Species>)> {
    let species = FacilityService::create_species(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(species)))
}

/// 更新物種
pub async fn update_species(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSpeciesRequest>,
) -> Result<Json<Species>> {
    let species = FacilityService::update_species(&state.db, id, &payload).await?;
    Ok(Json(species))
}

/// 刪除物種
pub async fn delete_species(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    FacilityService::delete_species(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Facility Handlers
// ============================================

/// 列出所有設施
pub async fn list_facilities(
    State(state): State<AppState>,
) -> Result<Json<Vec<Facility>>> {
    let facilities = FacilityService::list_facilities(&state.db).await?;
    Ok(Json(facilities))
}

/// 取得設施詳細
pub async fn get_facility(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Facility>> {
    let facility = FacilityService::get_facility(&state.db, id).await?;
    Ok(Json(facility))
}

/// 建立設施
pub async fn create_facility(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateFacilityRequest>,
) -> Result<(StatusCode, Json<Facility>)> {
    let facility = FacilityService::create_facility(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(facility)))
}

/// 更新設施
pub async fn update_facility(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFacilityRequest>,
) -> Result<Json<Facility>> {
    let facility = FacilityService::update_facility(&state.db, id, &payload).await?;
    Ok(Json(facility))
}

/// 刪除設施
pub async fn delete_facility(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    FacilityService::delete_facility(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Building Handlers
// ============================================

#[derive(Debug, serde::Deserialize)]
pub struct BuildingQuery {
    pub facility_id: Option<Uuid>,
}

/// 列出所有棟舍
pub async fn list_buildings(
    State(state): State<AppState>,
    Query(params): Query<BuildingQuery>,
) -> Result<Json<Vec<BuildingWithFacility>>> {
    let buildings = FacilityService::list_buildings(&state.db, params.facility_id).await?;
    Ok(Json(buildings))
}

/// 取得棟舍詳細
pub async fn get_building(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Building>> {
    let building = FacilityService::get_building(&state.db, id).await?;
    Ok(Json(building))
}

/// 建立棟舍
pub async fn create_building(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateBuildingRequest>,
) -> Result<(StatusCode, Json<Building>)> {
    let building = FacilityService::create_building(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(building)))
}

/// 更新棟舍
pub async fn update_building(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBuildingRequest>,
) -> Result<Json<Building>> {
    let building = FacilityService::update_building(&state.db, id, &payload).await?;
    Ok(Json(building))
}

/// 刪除棟舍
pub async fn delete_building(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    FacilityService::delete_building(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Zone Handlers
// ============================================

#[derive(Debug, serde::Deserialize)]
pub struct ZoneQuery {
    pub building_id: Option<Uuid>,
}

/// 列出所有區域
pub async fn list_zones(
    State(state): State<AppState>,
    Query(params): Query<ZoneQuery>,
) -> Result<Json<Vec<ZoneWithBuilding>>> {
    let zones = FacilityService::list_zones(&state.db, params.building_id).await?;
    Ok(Json(zones))
}

/// 取得區域詳細
pub async fn get_zone(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Zone>> {
    let zone = FacilityService::get_zone(&state.db, id).await?;
    Ok(Json(zone))
}

/// 建立區域
pub async fn create_zone(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateZoneRequest>,
) -> Result<(StatusCode, Json<Zone>)> {
    let zone = FacilityService::create_zone(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(zone)))
}

/// 更新區域
pub async fn update_zone(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateZoneRequest>,
) -> Result<Json<Zone>> {
    let zone = FacilityService::update_zone(&state.db, id, &payload).await?;
    Ok(Json(zone))
}

/// 刪除區域
pub async fn delete_zone(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    FacilityService::delete_zone(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Pen Handlers
// ============================================

/// 列出所有欄位
pub async fn list_pens(
    State(state): State<AppState>,
    Query(params): Query<PenQuery>,
) -> Result<Json<Vec<PenDetails>>> {
    let pens = FacilityService::list_pens(&state.db, &params).await?;
    Ok(Json(pens))
}

/// 取得欄位詳細
pub async fn get_pen(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Pen>> {
    let pen = FacilityService::get_pen(&state.db, id).await?;
    Ok(Json(pen))
}

/// 建立欄位
pub async fn create_pen(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<CreatePenRequest>,
) -> Result<(StatusCode, Json<Pen>)> {
    let pen = FacilityService::create_pen(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(pen)))
}

/// 更新欄位
pub async fn update_pen(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePenRequest>,
) -> Result<Json<Pen>> {
    let pen = FacilityService::update_pen(&state.db, id, &payload).await?;
    Ok(Json(pen))
}

/// 刪除欄位
pub async fn delete_pen(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    FacilityService::delete_pen(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Department Handlers
// ============================================

/// 列出所有部門
pub async fn list_departments(
    State(state): State<AppState>,
) -> Result<Json<Vec<DepartmentWithManager>>> {
    let departments = FacilityService::list_departments(&state.db).await?;
    Ok(Json(departments))
}

/// 取得部門詳細
pub async fn get_department(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Department>> {
    let department = FacilityService::get_department(&state.db, id).await?;
    Ok(Json(department))
}

/// 建立部門
pub async fn create_department(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateDepartmentRequest>,
) -> Result<(StatusCode, Json<Department>)> {
    let department = FacilityService::create_department(&state.db, &payload).await?;
    Ok((StatusCode::CREATED, Json(department)))
}

/// 更新部門
pub async fn update_department(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDepartmentRequest>,
) -> Result<Json<Department>> {
    let department = FacilityService::update_department(&state.db, id, &payload).await?;
    Ok(Json(department))
}

/// 刪除部門
pub async fn delete_department(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    FacilityService::delete_department(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
