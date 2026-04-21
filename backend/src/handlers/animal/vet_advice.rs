// 獸醫師建議 Handlers
// - 舊版結構化表單（保留，未來巡場報告用）
// - 新版多筆紀錄 CRUD

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{
        access, AnimalVetAdviceService, AnimalVetAdvice, UpsertVetAdviceRequest,
        VetAdviceRecord, VetAdviceRecordService,
        CreateVetAdviceRecordRequest, UpdateVetAdviceRecordRequest,
    },
    AppState, Result,
};

// ── 舊版結構化表單 ──────────────────────────────

/// 取得動物的獸醫師建議（結構化）
pub async fn get_animal_vet_advice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalVetAdvice>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
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
    let actor = ActorContext::User(current_user.clone());
    let advice =
        AnimalVetAdviceService::create_or_update(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(advice))
}

// ── 新版多筆紀錄 CRUD ──────────────────────────────

/// 列出動物的獸醫師建議紀錄
pub async fn list_vet_advice_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<VetAdviceRecord>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
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
    let actor = ActorContext::User(current_user.clone());
    let record = VetAdviceRecordService::create(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(record))
}

/// 更新獸醫師建議紀錄
pub async fn update_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVetAdviceRecordRequest>,
) -> Result<Json<VetAdviceRecord>> {
    let actor = ActorContext::User(current_user.clone());
    let record = VetAdviceRecordService::update(&state.db, &actor, id, &req).await?;
    Ok(Json(record))
}

/// 刪除獸醫師建議紀錄（soft delete）
pub async fn delete_vet_advice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>> {
    // SEC-IDOR: v2 CI 掃描發現 — 寫入操作必須驗證權限與動物歸屬
    require_permission!(current_user, "animal.vet.recommend");
    let actor = ActorContext::User(current_user.clone());
    VetAdviceRecordService::delete(&state.db, &actor, id).await?;
    Ok(Json(()))
}
