// 獸醫巡場報告 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    services::{
        VetPatrolReport, VetPatrolReportService, VetPatrolReportWithEntries,
        CreateVetPatrolReportRequest, UpdateVetPatrolReportRequest,
    },
    AppState, Result,
};

/// 列出所有巡場報告
pub async fn list_vet_patrol_reports(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<VetPatrolReport>>> {
    let reports = VetPatrolReportService::list(&state.db).await?;
    Ok(Json(reports))
}

/// 取得單一巡場報告（含條目）
pub async fn get_vet_patrol_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<VetPatrolReportWithEntries>>> {
    let report = VetPatrolReportService::get(&state.db, id).await?;
    Ok(Json(report))
}

/// 建立巡場報告
pub async fn create_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateVetPatrolReportRequest>,
) -> Result<Json<VetPatrolReport>> {
    let report = VetPatrolReportService::create(&state.db, &req, current_user.id).await?;
    Ok(Json(report))
}

/// 更新巡場報告
pub async fn update_vet_patrol_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVetPatrolReportRequest>,
) -> Result<Json<VetPatrolReport>> {
    let report = VetPatrolReportService::update(&state.db, id, &req, current_user.id).await?;
    Ok(Json(report))
}

/// 刪除巡場報告（soft delete）
pub async fn delete_vet_patrol_report(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    VetPatrolReportService::delete(&state.db, id).await?;
    Ok(Json(()))
}
