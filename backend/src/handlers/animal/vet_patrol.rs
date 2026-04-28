// 獸醫巡場報告 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{
        VetPatrolReport, VetPatrolReportService, VetPatrolReportWithEntries,
        CreateVetPatrolReportRequest, UpdateVetPatrolReportRequest,
    },
    AppState, Result,
};

/// 列出所有巡場報告
pub async fn list_vet_patrol_reports(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<VetPatrolReport>>> {
    // SEC-IDOR: 獸醫巡場報告需具備動物查閱權限
    require_permission!(current_user, "animal.record.view");
    let reports = VetPatrolReportService::list(&state.db).await?;
    Ok(Json(reports))
}

/// 取得單一巡場報告（含條目）
pub async fn get_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<VetPatrolReportWithEntries>>> {
    // SEC-IDOR: 獸醫巡場報告需具備動物查閱權限
    require_permission!(current_user, "animal.record.view");
    let report = VetPatrolReportService::get(&state.db, id).await?;
    Ok(Json(report))
}

/// 建立巡場報告
pub async fn create_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateVetPatrolReportRequest>,
) -> Result<Json<VetPatrolReport>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    let report = VetPatrolReportService::create(&state.db, &actor, &req).await?;
    Ok(Json(report))
}

/// 更新巡場報告
pub async fn update_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVetPatrolReportRequest>,
) -> Result<Json<VetPatrolReport>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    let report = VetPatrolReportService::update(&state.db, &actor, id, &req).await?;
    Ok(Json(report))
}

/// 刪除巡場報告（soft delete）
pub async fn delete_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    VetPatrolReportService::delete(&state.db, &actor, id).await?;
    Ok(Json(()))
}
