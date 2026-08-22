//! 動物欄位修正申請 Handlers
//! 耳號、出生日期、性別、品種等欄位需經 admin 批准後才能修改

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AnimalFieldCorrectionRequestListItem, CreateAnimalFieldCorrectionRequest,
        ReviewAnimalFieldCorrectionRequest,
    },
    require_permission,
    services::{access, AnimalFieldCorrectionService},
    AppState, Result,
};

/// 建立動物欄位修正申請（staff 可呼叫）
pub async fn create_animal_field_correction_request(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateAnimalFieldCorrectionRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.animal.edit");
    // R75-2: 補動物存取守衛（原僅權限檢查；新增 404 存在檢查 + 防無關 edit 權者跨動物提案）
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    let id =
        AnimalFieldCorrectionService::create_request(&state.db, animal_id, &req, current_user.id)
            .await?;

    Ok(Json(serde_json::json!({ "id": id })))
}

/// 列出待審核的修正申請（需 animal.field_correction.review 權限）
pub async fn list_pending_animal_field_corrections(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalFieldCorrectionRequestListItem>>> {
    require_permission!(current_user, "animal.field_correction.review");

    let list = AnimalFieldCorrectionService::list_pending(&state.db).await?;
    Ok(Json(list))
}

/// 審核修正申請（需 animal.field_correction.review 權限）
pub async fn review_animal_field_correction(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(request_id): Path<Uuid>,
    Json(req): Json<ReviewAnimalFieldCorrectionRequest>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "animal.field_correction.review");

    let actor = ActorContext::User(current_user.clone());
    AnimalFieldCorrectionService::review(&state.db, &actor, request_id, &req).await?;

    Ok(Json(serde_json::json!({ "message": "審核完成" })))
}

/// 取得某動物的修正申請列表
pub async fn list_animal_field_corrections(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<AnimalFieldCorrectionRequestListItem>>> {
    require_permission!(current_user, "animal.animal.edit");
    // R75-2: 補動物存取守衛（原僅權限檢查，可列舉任意動物的修正申請）
    access::require_animal_read_access(&state.db, &current_user, animal_id).await?;

    let list = AnimalFieldCorrectionService::list_by_animal(&state.db, animal_id).await?;
    Ok(Json(list))
}
