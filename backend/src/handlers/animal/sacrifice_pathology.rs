// 犧牲/安樂死 + 病理報告 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{AnimalPathologyReport, AnimalSacrifice, CreateSacrificeRequest},
    require_permission,
    services::{access, AnimalMedicalService},
    AppState, Result,
};

// ============================================
// 犧牲/安樂死記錄管理
// ============================================

/// 取得動物的犧牲記錄
#[utoipa::path(get, path = "/api/v1/animals/{animal_id}/sacrifice", params(("animal_id" = Uuid, Path, description = "動物 ID")), responses((status = 200, body = Option<AnimalSacrifice>), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn get_animal_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalSacrifice>>> {
    // SEC-IDOR: v2 審計發現原始修復遺漏此端點
    access::require_animal_access(&state.db, &current_user, animal_id).await?;
    let sacrifice = AnimalMedicalService::get_sacrifice(&state.db, animal_id).await?;
    Ok(Json(sacrifice))
}

/// 建立或更新犧牲記錄
#[utoipa::path(post, path = "/api/v1/animals/{animal_id}/sacrifice", params(("animal_id" = Uuid, Path, description = "動物 ID")), request_body = CreateSacrificeRequest, responses((status = 200, body = AnimalSacrifice), (status = 400), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn upsert_animal_sacrifice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateSacrificeRequest>,
) -> Result<Json<AnimalSacrifice>> {
    require_permission!(current_user, "animal.record.create");

    // Audit + animal 狀態轉換（若 confirmed_sacrifice）已收進 service 層（SACRIFICE_UPSERT，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let sacrifice =
        AnimalMedicalService::upsert_sacrifice(&state.db, &actor, animal_id, &req).await?;
    Ok(Json(sacrifice))
}

// ============================================
// 病理報告管理
// ============================================

/// 取得動物的病理報告
#[utoipa::path(get, path = "/api/v1/animals/{animal_id}/pathology", params(("animal_id" = Uuid, Path, description = "動物 ID")), responses((status = 200, body = Option<AnimalPathologyReport>), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn get_animal_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Option<AnimalPathologyReport>>> {
    require_permission!(current_user, "animal.pathology.view");
    // SEC-IDOR: v2 審計追加 — 權限檢查外需同時驗證計畫歸屬
    access::require_animal_access(&state.db, &current_user, animal_id).await?;

    let report = AnimalMedicalService::get_pathology_report(&state.db, animal_id).await?;
    Ok(Json(report))
}

/// 建立或更新病理報告
#[utoipa::path(post, path = "/api/v1/animals/{animal_id}/pathology", params(("animal_id" = Uuid, Path, description = "動物 ID")), responses((status = 200, body = AnimalPathologyReport), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn upsert_animal_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<AnimalPathologyReport>> {
    require_permission!(current_user, "animal.pathology.upload");

    // Audit 已收進 service 層（PATHOLOGY_UPSERT，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let report =
        AnimalMedicalService::upsert_pathology_report(&state.db, &actor, animal_id).await?;
    Ok(Json(report))
}
