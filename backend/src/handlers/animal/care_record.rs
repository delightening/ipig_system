// 照護紀錄（疼痛評估）Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::DeleteRequest,
    require_permission,
    services::{
        access, CareRecord, CareRecordService, CareVetRecordType, CreateCareRecordRequest,
        UpdateCareRecordRequest,
    },
    AppState, Result,
};

/// GET /animals/:id/care-records — 列出動物的照護紀錄
pub async fn list_care_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<CareRecord>>> {
    // C2: 驗證使用者對此動物所屬計畫的存取權限，防止 IDOR
    let scope =
        access::Scoped::<access::AnimalRead>::authorize(&state.db, &current_user, animal_id)
            .await?;
    let records = CareRecordService::list_by_animal(&state.db, scope).await?;
    Ok(Json(records))
}

/// POST /animals/:id/care-records — 建立照護紀錄
pub async fn create_care_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateCareRecordRequest>,
) -> Result<Json<CareRecord>> {
    require_permission!(current_user, "animal.record.create");
    // C2: 驗證使用者對此動物所屬計畫的存取權限，防止 IDOR
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;
    // SEC-IDOR: 驗證 body 的 record_id（觀察/手術紀錄）確實屬於 path 的 animal_id，
    // 否則使用者可用自己有權的 animal_id 為他人動物的紀錄掛上照護紀錄（繞過存取邊界）
    let target_animal_id = match req.record_type {
        CareVetRecordType::Observation => {
            access::get_observation_animal_id(&state.db, req.record_id).await?
        }
        CareVetRecordType::Surgery => {
            access::get_surgery_animal_id(&state.db, req.record_id).await?
        }
    };
    if target_animal_id != animal_id {
        return Err(crate::AppError::Forbidden(
            "照護紀錄所引用的觀察/手術紀錄不屬於指定動物".into(),
        ));
    }
    let actor = ActorContext::User(current_user.clone());
    let record = CareRecordService::create(&state.db, &actor, scope, &req).await?;
    Ok(Json(record))
}

/// GET /observations/:id/care-records — 列出觀察紀錄的照護紀錄
pub async fn list_observation_care_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(observation_id): Path<Uuid>,
) -> Result<Json<Vec<CareRecord>>> {
    // SEC-IDOR: v2 審計發現 — 透過觀察紀錄所屬動物驗證計畫存取權限
    let animal_id = access::get_observation_animal_id(&state.db, observation_id).await?;
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;
    let records = CareRecordService::list_by_record(
        &state.db,
        CareVetRecordType::Observation,
        observation_id,
    )
    .await?;
    Ok(Json(records))
}

/// GET /surgeries/:id/care-records — 列出手術紀錄的照護紀錄
pub async fn list_surgery_care_records(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(surgery_id): Path<Uuid>,
) -> Result<Json<Vec<CareRecord>>> {
    // SEC-IDOR: v2 審計發現 — 透過手術紀錄所屬動物驗證計畫存取權限
    let surgery_animal_id: Uuid =
        sqlx::query_scalar("SELECT animal_id FROM animal_surgeries WHERE id = $1")
            .bind(surgery_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| crate::AppError::NotFound("Surgery not found".into()))?;
    access::require_animal_read_access(&state.db, &current_user, surgery_animal_id).await?;
    let records =
        CareRecordService::list_by_record(&state.db, CareVetRecordType::Surgery, surgery_id)
            .await?;
    Ok(Json(records))
}

/// PUT /care-records/:id — 更新照護紀錄
pub async fn update_care_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCareRecordRequest>,
) -> Result<Json<CareRecord>> {
    require_permission!(current_user, "animal.record.edit");
    // C2: 透過照護紀錄找到 animal，再驗證計畫存取權限，防止 IDOR
    let animal_id = access::get_care_record_animal_id(&state.db, id).await?;
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;
    let actor = ActorContext::User(current_user.clone());
    let record = CareRecordService::update(&state.db, &actor, scope, id, &req).await?;
    Ok(Json(record))
}

/// DELETE /care-records/:id — 刪除照護紀錄（軟刪除 + 刪除原因）- GLP 合規
pub async fn delete_care_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.record.delete");
    req.validate()?;

    // SEC-IDOR (Gemini PR #182)：透過照護紀錄找到 animal，再驗證計畫存取權限
    let animal_id = access::get_care_record_animal_id(&state.db, id).await?;
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    // Audit 已收進 service 層（CARE_RECORD_DELETE with change_reasons，tx 內）
    let actor = ActorContext::User(current_user.clone());
    CareRecordService::soft_delete_with_reason(&state.db, &actor, scope, id, &req.reason).await?;

    Ok(Json(
        serde_json::json!({ "message": "Care record deleted" }),
    ))
}
