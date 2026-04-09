// 獸醫師建議 Handlers
// - 舊版結構化表單（保留，未來巡場報告用）
// - 新版多筆紀錄 CRUD

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    services::{
        AnimalVetAdviceService, AnimalVetAdvice, UpsertVetAdviceRequest,
        VetAdviceRecord, VetAdviceRecordService,
        CreateVetAdviceRecordRequest, UpdateVetAdviceRecordRequest,
    },
    AppState, Result,
};

// ── 舊版結構化表單 ──────────────────────────────

/// 取得動物的獸醫師建議（結構化）
pub async fn get_animal_vet_advice(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalVetAdvice>>> {
    let advice = AnimalVetAdviceService::get_by_animal(&state.db, animal_id).await?;
    Ok(Json(advice))
}

/// 新增或更新動物的獸醫師建議（結構化）
pub async fn upsert_animal_vet_advice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<UpsertVetAdviceRequest>,
) -> Result<Json<AnimalVetAdvice>> {
    let advice =
        AnimalVetAdviceService::upsert(&state.db, animal_id, &req, current_user.id).await?;
    Ok(Json(advice))
}

// ── 新版多筆紀錄 CRUD ──────────────────────────────

/// 列出動物的獸醫師建議紀錄
pub async fn list_vet_advice_records(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<VetAdviceRecord>>> {
    let records = VetAdviceRecordService::list(&state.db, animal_id).await?;
    Ok(Json(records))
}

/// 新增獸醫師建議紀錄
pub async fn create_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateVetAdviceRecordRequest>,
) -> Result<Json<VetAdviceRecord>> {
    let record =
        VetAdviceRecordService::create(&state.db, animal_id, &req, current_user.id).await?;
    Ok(Json(record))
}

/// 更新獸醫師建議紀錄
pub async fn update_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVetAdviceRecordRequest>,
) -> Result<Json<VetAdviceRecord>> {
    let record =
        VetAdviceRecordService::update(&state.db, id, &req, current_user.id).await?;
    Ok(Json(record))
}

/// 刪除獸醫師建議紀錄（soft delete）
pub async fn delete_vet_advice_record(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    VetAdviceRecordService::delete(&state.db, id).await?;
    Ok(Json(()))
}
