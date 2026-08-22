use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        Animal, CreatePlannedExperimentRequest, PlannedExperiment, PlannedExperimentResponse,
        ReservableAnimalRow, ReservableQuery, ReservationPlanningGroup, ReserveAnimalsRequest,
        UnreserveAnimalsRequest, UpdatePlannedExperimentRequest, UpdateRemarkRequest,
    },
    require_permission,
    services::{AnimalService, PlannedExperimentService},
    AppState, Result,
};

/// 動物預約與試驗規劃的**檢視**權限（唯讀）。
///
/// 授予執行秘書 `IACUC_STAFF`、試驗工作人員 `EXPERIMENT_STAFF`（全體）、
/// 負責人 `DIRECTOR`；admin 自動 bypass。
///
/// 原本另授予全域 `STUDY_DIRECTOR` 角色，用意是涵蓋「已被指派為 SD 但帳號未帶
/// `EXPERIMENT_STAFF`」的歷史帳號。該角色已移除——而 `validate_and_authorize_sd`
/// 本來就強制 SD 必須具 `EXPERIMENT_STAFF`，新指派的 SD 一律被上面第二項涵蓋。
/// 獸醫 `VET` 與計畫主持人 `PI` 不給（使用者 2026-08-07 裁定）。
///
/// 拆分理由：本檔原本讀寫共用 [`PERM_MANAGE`]，導致 SD 與試驗工作人員**連頁面都進不來**，
/// 而不只是看不到操作按鈕。見 `docs/audit/button-permission-gate-2026-08-07.md` §6。
const PERM_VIEW: &str = "animal.planning.view";

/// 動物預約與試驗規劃的**操作**權限：新增預定試驗 / 批次預約 / 正式分配 /
/// 解除預約 / 備註編輯。僅執行秘書（`IACUC_STAFF`）+ admin（自動 bypass）。
///
/// ⚠️ 刻意**不**沿用 `animal.info.assign`：那個權限同時被
/// [`crate::handlers::batch_assign_animals`]（動物列表的批次分配到計畫）使用，
/// 且 prod 上由 EXPERIMENT_STAFF / VET 等角色持有。沿用它就做不到「僅執秘可操作本頁」，
/// 而從那些角色拔除又會連帶砍掉批次分配功能。故本頁自帶專屬權限。
const PERM_MANAGE: &str = "animal.planning.manage";

pub async fn list_planned_experiments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<PlannedExperimentResponse>>> {
    require_permission!(current_user, PERM_VIEW);
    Ok(Json(PlannedExperimentService::list(&state.db).await?))
}

pub async fn get_planned_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlannedExperiment>> {
    require_permission!(current_user, PERM_VIEW);
    Ok(Json(PlannedExperimentService::get(&state.db, id).await?))
}

pub async fn create_planned_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePlannedExperimentRequest>,
) -> Result<(StatusCode, Json<PlannedExperiment>)> {
    require_permission!(current_user, PERM_MANAGE);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    let v = PlannedExperimentService::create(&state.db, &actor, &req).await?;
    Ok((StatusCode::CREATED, Json(v)))
}

pub async fn update_planned_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePlannedExperimentRequest>,
) -> Result<Json<PlannedExperiment>> {
    require_permission!(current_user, PERM_MANAGE);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    let v = PlannedExperimentService::update(&state.db, &actor, id, &req).await?;
    Ok(Json(v))
}

pub async fn delete_planned_experiment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    require_permission!(current_user, PERM_MANAGE);
    let actor = ActorContext::User(current_user);
    PlannedExperimentService::delete(&state.db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 批次預約動物到試驗（Phase 2）。回實際被預約的動物。
pub async fn reserve_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<ReserveAnimalsRequest>,
) -> Result<Json<Vec<Animal>>> {
    require_permission!(current_user, PERM_MANAGE);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    Ok(Json(
        PlannedExperimentService::reserve(&state.db, &actor, &req).await?,
    ))
}

/// 批次解除預約（Phase 2）。
pub async fn unreserve_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UnreserveAnimalsRequest>,
) -> Result<Json<Vec<Animal>>> {
    require_permission!(current_user, PERM_MANAGE);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    Ok(Json(
        PlannedExperimentService::unreserve(&state.db, &actor, &req).await?,
    ))
}

/// 搜尋可預約動物（未分配未預約，體重/月齡/性別篩選，Phase 2）。
pub async fn list_reservable_animals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReservableQuery>,
) -> Result<Json<Vec<ReservableAnimalRow>>> {
    require_permission!(current_user, PERM_VIEW);
    Ok(Json(
        PlannedExperimentService::search_reservable(&state.db, &query).await?,
    ))
}

/// 更新動物備註（Phase 5）：規劃頁整格 inline 編輯。權限同規劃頁其餘操作
/// （`animal.planning.manage`：執秘 / admin 在此管理全場活豬備註）。
/// audit 走 `ANIMAL_REMARK_UPDATE`。
pub async fn update_animal_remark(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRemarkRequest>,
) -> Result<Json<Animal>> {
    require_permission!(current_user, PERM_MANAGE);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    Ok(Json(
        AnimalService::update_remark(&state.db, &actor, id, req.remark).await?,
    ))
}

/// 規劃分組查詢（Phase 3）：全場動物按試驗分組 + 需求/已預約/已分配/缺口。
pub async fn get_reservation_planning(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ReservationPlanningGroup>>> {
    require_permission!(current_user, PERM_VIEW);
    Ok(Json(
        PlannedExperimentService::get_reservation_planning(&state.db).await?,
    ))
}
