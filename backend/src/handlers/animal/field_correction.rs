//! 動物欄位修正申請 Handlers
//! 耳號、出生日期、性別、品種等欄位需經 admin 批准後才能修改

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        CreateAnimalFieldCorrectionRequest,
        ReviewAnimalFieldCorrectionRequest,
        AnimalFieldCorrectionRequestListItem,
    },
    require_permission,
    services::AnimalFieldCorrectionService,
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

    let id = AnimalFieldCorrectionService::create_request(
        &state.db,
        animal_id,
        &req,
        current_user.id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "id": id })))
}

/// 列出待審核的修正申請（僅 admin）
pub async fn list_pending_animal_field_corrections(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<AnimalFieldCorrectionRequestListItem>>> {
    if !current_user.is_admin() {
        return Err(crate::AppError::Forbidden(
            "僅系統管理員可審核動物欄位修正申請".to_string(),
        ));
    }

    let list = AnimalFieldCorrectionService::list_pending(&state.db).await?;
    Ok(Json(list))
}

/// 審核修正申請（僅 admin）
pub async fn review_animal_field_correction(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(request_id): Path<Uuid>,
    Json(req): Json<ReviewAnimalFieldCorrectionRequest>,
) -> Result<Json<serde_json::Value>> {
    if !current_user.is_admin() {
        return Err(crate::AppError::Forbidden(
            "僅系統管理員可審核動物欄位修正申請".to_string(),
        ));
    }
    AnimalFieldCorrectionService::review(
        &state.db,
        request_id,
        &req,
        current_user.id,
    )
    .await?;

    Ok(Json(serde_json::json!({ "message": "審核完成" })))
}

/// 取得某動物的修正申請列表
pub async fn list_animal_field_corrections(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<AnimalFieldCorrectionRequestListItem>>> {
    require_permission!(current_user, "animal.animal.edit");

    let list = AnimalFieldCorrectionService::list_by_animal(&state.db, animal_id).await?;
    Ok(Json(list))
}
