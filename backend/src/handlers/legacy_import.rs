use axum::{extract::Path, extract::State, Extension, Json};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateImportBatchRequest, ImportBatchReport, IngestPigRecordsRequest, LegacyImportBatch,
        IngestPigRecordsResponse,
    },
    require_permission,
    services::legacy_import::LegacyImportService,
    AppState, Result,
};

/// POST /api/legacy-import/batches
pub async fn create_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateImportBatchRequest>,
) -> Result<Json<LegacyImportBatch>> {
    require_permission!(current_user, "animal.legacy_import.write");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let batch = LegacyImportService::create_batch(&state.db, &actor, &req).await?;
    Ok(Json(batch))
}

/// POST /api/legacy-import/batches/{batch_id}/records
pub async fn ingest_pig_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<IngestPigRecordsRequest>,
) -> Result<Json<IngestPigRecordsResponse>> {
    require_permission!(current_user, "animal.legacy_import.write");
    let actor = crate::middleware::ActorContext::User(current_user.clone());
    let resp = LegacyImportService::ingest_pig_records(&state.db, &actor, batch_id, &req).await?;
    Ok(Json(resp))
}

/// GET /api/legacy-import/batches/{batch_id}/report
pub async fn get_batch_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<ImportBatchReport>> {
    require_permission!(current_user, "animal.legacy_import.write");
    let report = LegacyImportService::get_batch_report(&state.db, batch_id).await?;
    Ok(Json(report))
}

