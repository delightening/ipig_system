// Facility Handlers
// 包含：Species, Facility, Building, Zone, Pen, Department
//
// 權限設計：
//   GET（查詢）：任何已登入使用者皆可讀取設施結構（棟/區/欄位為靜態配置資料）
//   POST/PUT/DELETE（管理）：需要 facility.manage 權限

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        BatchCreatePensRequest, Building, BuildingWithFacility, CommitteeMember,
        CreateBuildingRequest, CreateDepartmentRequest, CreateFacilityRequest, CreatePenRequest,
        CreateSpeciesRequest, CreateZoneRequest, Department, DepartmentMember,
        DepartmentWithManager, Facility, Pen, PenDetails, PenQuery, Species, SpeciesListItem,
        SpeciesListQuery, UpdateBuildingRequest, UpdateDepartmentRequest, UpdateFacilityRequest,
        UpdatePenRequest, UpdateSpeciesRequest, UpdateZoneRequest, Zone, ZoneWithBuilding,
    },
    require_permission,
    services::{FacilityService, UserService},
    AppState, Result,
};

// ============================================
// Species Handlers
// ============================================

/// 列出物種（含各物種目前的動物引用數）
///
/// 預設只回啟用中的物種；帶 `include_inactive=true` 才連停用的一起回。
/// 物種代碼全域唯一且不重用，停用物種必須看得到才能重新啟用。
#[utoipa::path(get, path = "/api/v1/facilities/species", params(SpeciesListQuery), responses((status = 200, description = "物種清單", body = Vec<SpeciesListItem>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_species(
    State(state): State<AppState>,
    Query(query): Query<SpeciesListQuery>,
) -> Result<Json<Vec<SpeciesListItem>>> {
    let species = FacilityService::list_species(&state.db, query.include_inactive).await?;
    Ok(Json(species))
}

/// 取得物種詳細
#[utoipa::path(get, path = "/api/v1/facilities/species/{id}", params(("id" = Uuid, Path, description = "物種 ID")), responses((status = 200, description = "物種資訊", body = Species)), tag = "設施管理", security(("bearer" = [])))]
pub async fn get_species(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Species>> {
    let species = FacilityService::get_species(&state.db, id).await?;
    Ok(Json(species))
}

/// 建立物種
#[utoipa::path(post, path = "/api/v1/facilities/species", request_body = CreateSpeciesRequest, responses((status = 201, description = "建立成功", body = Species)), tag = "設施管理", security(("bearer" = [])))]
pub async fn create_species(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateSpeciesRequest>,
) -> Result<(StatusCode, Json<Species>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let species = FacilityService::create_species(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(species)))
}

/// 更新物種
#[utoipa::path(put, path = "/api/v1/facilities/species/{id}", params(("id" = Uuid, Path, description = "物種 ID")), request_body = UpdateSpeciesRequest, responses((status = 200, description = "更新成功", body = Species)), tag = "設施管理", security(("bearer" = [])))]
pub async fn update_species(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSpeciesRequest>,
) -> Result<Json<Species>> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let species = FacilityService::update_species(&state.db, &actor, id, &payload).await?;
    Ok(Json(species))
}

/// 刪除物種
#[utoipa::path(delete, path = "/api/v1/facilities/species/{id}", params(("id" = Uuid, Path, description = "物種 ID")), responses((status = 204, description = "刪除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn delete_species(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    FacilityService::delete_species(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Facility Handlers
// ============================================

/// 列出所有設施
#[utoipa::path(get, path = "/api/v1/facilities", responses((status = 200, description = "設施清單", body = Vec<Facility>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_facilities(State(state): State<AppState>) -> Result<Json<Vec<Facility>>> {
    let facilities = FacilityService::list_facilities(&state.db).await?;
    Ok(Json(facilities))
}

/// 取得設施詳細
#[utoipa::path(get, path = "/api/v1/facilities/{id}", params(("id" = Uuid, Path, description = "設施 ID")), responses((status = 200, description = "設施資訊", body = Facility)), tag = "設施管理", security(("bearer" = [])))]
pub async fn get_facility(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Facility>> {
    let facility = FacilityService::get_facility(&state.db, id).await?;
    Ok(Json(facility))
}

/// 建立設施
#[utoipa::path(post, path = "/api/v1/facilities", request_body = CreateFacilityRequest, responses((status = 201, description = "建立成功", body = Facility)), tag = "設施管理", security(("bearer" = [])))]
pub async fn create_facility(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateFacilityRequest>,
) -> Result<(StatusCode, Json<Facility>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let facility = FacilityService::create_facility(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(facility)))
}

/// 更新設施
#[utoipa::path(put, path = "/api/v1/facilities/{id}", params(("id" = Uuid, Path, description = "設施 ID")), request_body = UpdateFacilityRequest, responses((status = 200, description = "更新成功", body = Facility)), tag = "設施管理", security(("bearer" = [])))]
pub async fn update_facility(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateFacilityRequest>,
) -> Result<Json<Facility>> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let facility = FacilityService::update_facility(&state.db, &actor, id, &payload).await?;
    Ok(Json(facility))
}

/// 刪除設施
#[utoipa::path(delete, path = "/api/v1/facilities/{id}", params(("id" = Uuid, Path, description = "設施 ID")), responses((status = 204, description = "刪除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn delete_facility(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    FacilityService::delete_facility(&state.db, &actor, id).await?;
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
#[utoipa::path(get, path = "/api/v1/facilities/buildings", responses((status = 200, description = "棟舍清單", body = Vec<BuildingWithFacility>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_buildings(
    State(state): State<AppState>,
    Query(params): Query<BuildingQuery>,
) -> Result<Json<Vec<BuildingWithFacility>>> {
    let buildings = FacilityService::list_buildings(&state.db, params.facility_id).await?;
    Ok(Json(buildings))
}

/// 取得棟舍詳細
#[utoipa::path(get, path = "/api/v1/facilities/buildings/{id}", params(("id" = Uuid, Path, description = "棟舍 ID")), responses((status = 200, description = "棟舍資訊", body = Building)), tag = "設施管理", security(("bearer" = [])))]
pub async fn get_building(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Building>> {
    let building = FacilityService::get_building(&state.db, id).await?;
    Ok(Json(building))
}

/// 建立棟舍
#[utoipa::path(post, path = "/api/v1/facilities/buildings", request_body = CreateBuildingRequest, responses((status = 201, description = "建立成功", body = Building)), tag = "設施管理", security(("bearer" = [])))]
pub async fn create_building(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateBuildingRequest>,
) -> Result<(StatusCode, Json<Building>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let building = FacilityService::create_building(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(building)))
}

/// 更新棟舍
#[utoipa::path(put, path = "/api/v1/facilities/buildings/{id}", params(("id" = Uuid, Path, description = "棟舍 ID")), request_body = UpdateBuildingRequest, responses((status = 200, description = "更新成功", body = Building)), tag = "設施管理", security(("bearer" = [])))]
pub async fn update_building(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBuildingRequest>,
) -> Result<Json<Building>> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let building = FacilityService::update_building(&state.db, &actor, id, &payload).await?;
    Ok(Json(building))
}

/// 刪除棟舍
#[utoipa::path(delete, path = "/api/v1/facilities/buildings/{id}", params(("id" = Uuid, Path, description = "棟舍 ID")), responses((status = 204, description = "刪除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn delete_building(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    FacilityService::delete_building(&state.db, &actor, id).await?;
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
#[utoipa::path(get, path = "/api/v1/facilities/zones", responses((status = 200, description = "區域清單", body = Vec<ZoneWithBuilding>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_zones(
    State(state): State<AppState>,
    Query(params): Query<ZoneQuery>,
) -> Result<Json<Vec<ZoneWithBuilding>>> {
    let zones = FacilityService::list_zones(&state.db, params.building_id).await?;
    Ok(Json(zones))
}

/// 取得區域詳細
#[utoipa::path(get, path = "/api/v1/facilities/zones/{id}", params(("id" = Uuid, Path, description = "區域 ID")), responses((status = 200, description = "區域資訊", body = Zone)), tag = "設施管理", security(("bearer" = [])))]
pub async fn get_zone(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<Zone>> {
    let zone = FacilityService::get_zone(&state.db, id).await?;
    Ok(Json(zone))
}

/// 建立區域
#[utoipa::path(post, path = "/api/v1/facilities/zones", request_body = CreateZoneRequest, responses((status = 201, description = "建立成功", body = Zone)), tag = "設施管理", security(("bearer" = [])))]
pub async fn create_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateZoneRequest>,
) -> Result<(StatusCode, Json<Zone>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let zone = FacilityService::create_zone(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(zone)))
}

/// 更新區域
#[utoipa::path(put, path = "/api/v1/facilities/zones/{id}", params(("id" = Uuid, Path, description = "區域 ID")), request_body = UpdateZoneRequest, responses((status = 200, description = "更新成功", body = Zone)), tag = "設施管理", security(("bearer" = [])))]
pub async fn update_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateZoneRequest>,
) -> Result<Json<Zone>> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let zone = FacilityService::update_zone(&state.db, &actor, id, &payload).await?;
    Ok(Json(zone))
}

/// 刪除區域
#[utoipa::path(delete, path = "/api/v1/facilities/zones/{id}", params(("id" = Uuid, Path, description = "區域 ID")), responses((status = 204, description = "刪除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn delete_zone(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    FacilityService::delete_zone(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Pen Handlers
// ============================================

/// 列出所有欄位
#[utoipa::path(get, path = "/api/v1/facilities/pens", responses((status = 200, description = "欄位清單", body = Vec<PenDetails>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_pens(
    State(state): State<AppState>,
    Query(params): Query<PenQuery>,
) -> Result<Json<Vec<PenDetails>>> {
    let pens = FacilityService::list_pens(&state.db, &params).await?;
    Ok(Json(pens))
}

/// 取得欄位詳細
#[utoipa::path(get, path = "/api/v1/facilities/pens/{id}", params(("id" = Uuid, Path, description = "欄位 ID")), responses((status = 200, description = "欄位資訊", body = Pen)), tag = "設施管理", security(("bearer" = [])))]
pub async fn get_pen(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<Pen>> {
    let pen = FacilityService::get_pen(&state.db, id).await?;
    Ok(Json(pen))
}

/// 建立欄位
#[utoipa::path(post, path = "/api/v1/facilities/pens", request_body = CreatePenRequest, responses((status = 201, description = "建立成功", body = Pen)), tag = "設施管理", security(("bearer" = [])))]
pub async fn create_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreatePenRequest>,
) -> Result<(StatusCode, Json<Pen>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let pen = FacilityService::create_pen(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(pen)))
}

/// 批次建立欄位
pub async fn batch_create_pens(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<BatchCreatePensRequest>,
) -> Result<(StatusCode, Json<Vec<Pen>>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let pens = FacilityService::batch_create_pens(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(pens)))
}

/// 更新欄位
#[utoipa::path(put, path = "/api/v1/facilities/pens/{id}", params(("id" = Uuid, Path, description = "欄位 ID")), request_body = UpdatePenRequest, responses((status = 200, description = "更新成功", body = Pen)), tag = "設施管理", security(("bearer" = [])))]
pub async fn update_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePenRequest>,
) -> Result<Json<Pen>> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let pen = FacilityService::update_pen(&state.db, &actor, id, &payload).await?;
    Ok(Json(pen))
}

/// 刪除欄位
#[utoipa::path(delete, path = "/api/v1/facilities/pens/{id}", params(("id" = Uuid, Path, description = "欄位 ID")), responses((status = 204, description = "刪除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn delete_pen(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    FacilityService::delete_pen(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Department Handlers
// ============================================

/// 列出所有部門
#[utoipa::path(get, path = "/api/v1/facilities/departments", responses((status = 200, description = "部門清單", body = Vec<DepartmentWithManager>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_departments(
    State(state): State<AppState>,
) -> Result<Json<Vec<DepartmentWithManager>>> {
    let departments = FacilityService::list_departments(&state.db).await?;
    Ok(Json(departments))
}

/// 取得部門詳細
#[utoipa::path(get, path = "/api/v1/facilities/departments/{id}", params(("id" = Uuid, Path, description = "部門 ID")), responses((status = 200, description = "部門資訊", body = Department)), tag = "設施管理", security(("bearer" = [])))]
pub async fn get_department(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Department>> {
    let department = FacilityService::get_department(&state.db, id).await?;
    Ok(Json(department))
}

/// 建立部門
#[utoipa::path(post, path = "/api/v1/facilities/departments", request_body = CreateDepartmentRequest, responses((status = 201, description = "建立成功", body = Department)), tag = "設施管理", security(("bearer" = [])))]
pub async fn create_department(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(payload): Json<CreateDepartmentRequest>,
) -> Result<(StatusCode, Json<Department>)> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let department = FacilityService::create_department(&state.db, &actor, &payload).await?;
    Ok((StatusCode::CREATED, Json(department)))
}

/// 更新部門
#[utoipa::path(put, path = "/api/v1/facilities/departments/{id}", params(("id" = Uuid, Path, description = "部門 ID")), request_body = UpdateDepartmentRequest, responses((status = 200, description = "更新成功", body = Department)), tag = "設施管理", security(("bearer" = [])))]
pub async fn update_department(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateDepartmentRequest>,
) -> Result<Json<Department>> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let department = FacilityService::update_department(&state.db, &actor, id, &payload).await?;
    Ok(Json(department))
}

/// 刪除部門
#[utoipa::path(delete, path = "/api/v1/facilities/departments/{id}", params(("id" = Uuid, Path, description = "部門 ID")), responses((status = 204, description = "刪除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn delete_department(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, "facility.manage");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    FacilityService::delete_department(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// Department Members Handlers（部門成員指派）
//
// 權限：查詢成員需 admin.user.view；指派 / 移除需 admin.user.edit
//   （department_id 屬 users 欄位，故 mutation 走 UserService + admin.user.edit gate）。
// ============================================

/// 指派成員請求（body 帶 user_id）
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct AssignMemberRequest {
    pub user_id: Uuid,
    /// 明知對方是外部人員仍要編入本部門。
    ///
    /// 外部人員可以合法屬於某個部門（IACUC 是內部部門卻聘用外部委員），
    /// 但那必須是**刻意的動作**而非順手點到。前端用「一併顯示外部人員」
    /// 開關表達這個意圖，本欄位是同一件事的伺服器端對應——少了它，過期的
    /// 前端、競態、或任何繞過該開關的呼叫端都能靜默把外部人員塞進任一部門。
    #[serde(default)]
    pub allow_external: bool,
}

/// 列出單一部門的成員
#[utoipa::path(get, path = "/api/v1/facilities/departments/{id}/members", params(("id" = Uuid, Path, description = "部門 ID")), responses((status = 200, description = "成員清單", body = Vec<DepartmentMember>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_department_members(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DepartmentMember>>> {
    require_permission!(current_user, "admin.user.view");
    let members = UserService::list_department_members(&state.db, Some(id)).await?;
    Ok(Json(members))
}

/// 列出所有已指派部門的成員（供組織圖一次載入）
#[utoipa::path(get, path = "/api/v1/facilities/department-members", responses((status = 200, description = "全部門成員清單", body = Vec<DepartmentMember>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_all_department_members(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<DepartmentMember>>> {
    require_permission!(current_user, "admin.user.view");
    let members = UserService::list_department_members(&state.db, None).await?;
    Ok(Json(members))
}

/// IACUC 委員名冊（依角色列出，與部門編制無關）
///
/// 委員身分來自角色（IACUC_CHAIR / REVIEWER / IACUC_STAFF），人事歸屬來自
/// `users.department_id`——兩者正交。委員會是跨部門的橫切面（獸醫可以兼任
/// 審查委員），而 `department_id` 是單值的，裝不下這種雙重身分。
/// 故名冊獨立於部門樹查詢，同一個人可以同時出現在兩處。
#[utoipa::path(get, path = "/api/v1/facilities/committee-members", responses((status = 200, description = "IACUC 委員名冊", body = Vec<CommitteeMember>)), tag = "設施管理", security(("bearer" = [])))]
pub async fn list_committee_members(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<CommitteeMember>>> {
    // 與部門成員清單同一道閘：兩者都是「誰在組織裡的哪個位置」的唯讀資訊。
    require_permission!(current_user, "admin.user.view");
    let members = UserService::list_committee_members(&state.db).await?;
    Ok(Json(members))
}

/// 指派使用者到部門
#[utoipa::path(post, path = "/api/v1/facilities/departments/{id}/members", params(("id" = Uuid, Path, description = "部門 ID")), request_body = AssignMemberRequest, responses((status = 204, description = "指派成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn assign_department_member(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AssignMemberRequest>,
) -> Result<StatusCode> {
    require_permission!(current_user, "admin.user.edit");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    UserService::assign_user_department(
        &state.db,
        &actor,
        payload.user_id,
        id,
        payload.allow_external,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 將使用者移出部門（僅當該員確屬此部門時才清除）
#[utoipa::path(delete, path = "/api/v1/facilities/departments/{id}/members/{user_id}", params(("id" = Uuid, Path, description = "部門 ID"), ("user_id" = Uuid, Path, description = "使用者 ID")), responses((status = 204, description = "移除成功")), tag = "設施管理", security(("bearer" = [])))]
pub async fn remove_department_member(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    require_permission!(current_user, "admin.user.edit");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    UserService::remove_user_department(&state.db, &actor, user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
