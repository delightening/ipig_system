// 動物轉讓 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        AnimalTransfer, AssignTransferPlanRequest, CreateTransferRequest, DataBoundaryResponse,
        RejectTransferRequest, TransferVetEvaluation, VetEvaluateTransferRequest,
    },
    require_permission,
    services::{access, AnimalTransferService},
    AppState, Result,
};

/// 取得資料隔離的時間界線
pub async fn get_animal_data_boundary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<DataBoundaryResponse>> {
    let result = AnimalTransferService::get_data_boundary(
        &state.db,
        animal_id,
        current_user.id,
        &current_user.roles,
    )
    .await?;
    Ok(Json(result))
}

/// 取得動物的轉讓記錄列表
#[utoipa::path(get, path = "/api/v1/animals/{animal_id}/transfers", params(("animal_id" = Uuid, Path, description = "動物 ID")), responses((status = 200, body = Vec<AnimalTransfer>), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn list_animal_transfers(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<AnimalTransfer>>> {
    // SEC-IDOR: 驗證使用者是否有權存取該動物（透過計畫成員資格）
    let scope =
        access::Scoped::<access::AnimalRead>::authorize(&state.db, &current_user, animal_id)
            .await?;
    let records = AnimalTransferService::list_transfers(&state.db, scope).await?;
    Ok(Json(records))
}

/// 取得單一轉讓記錄
#[utoipa::path(get, path = "/api/v1/transfers/{transfer_id}", params(("transfer_id" = Uuid, Path, description = "轉讓 ID")), responses((status = 200, body = AnimalTransfer), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn get_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<AnimalTransfer>> {
    let record = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    // SEC-IDOR: 透過轉讓記錄所屬動物驗證計畫存取權限
    access::require_animal_read_access(&state.db, &current_user, record.animal_id).await?;
    Ok(Json(record))
}

/// 取得轉讓的獸醫評估
pub async fn get_transfer_vet_evaluation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<Option<TransferVetEvaluation>>> {
    // SEC-IDOR: 先查轉讓記錄以取得 animal_id，驗證存取權限
    let transfer = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    let scope = access::Scoped::<access::AnimalRead>::authorize(
        &state.db,
        &current_user,
        transfer.animal_id,
    )
    .await?;
    let record =
        AnimalTransferService::get_transfer_vet_evaluation(&state.db, scope, transfer_id).await?;
    Ok(Json(record))
}

/// 步驟 1：發起轉讓
#[utoipa::path(post, path = "/api/v1/animals/{animal_id}/transfers", params(("animal_id" = Uuid, Path, description = "動物 ID")), request_body = CreateTransferRequest, responses((status = 200, body = AnimalTransfer), (status = 400), (status = 401)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn initiate_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateTransferRequest>,
) -> Result<Json<AnimalTransfer>> {
    // P0-3 SoD：協調段用 `animal.transfer.manage`（執秘），與獸醫評估、PI 同意分權。
    require_permission!(current_user, "animal.transfer.manage");
    // SEC-IDOR: 驗證使用者有權存取該動物（與讀取端點一致；防跨計畫發起轉讓）
    let scope =
        access::Scoped::<access::AnimalWrite>::authorize(&state.db, &current_user, animal_id)
            .await?;

    // Audit 已收進 service 層（TRANSFER_INITIATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let record = AnimalTransferService::initiate_transfer(&state.db, &actor, scope, &req).await?;
    Ok(Json(record))
}

/// 步驟 2：獸醫評估
pub async fn vet_evaluate_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<VetEvaluateTransferRequest>,
) -> Result<Json<AnimalTransfer>> {
    require_permission!(current_user, "animal.vet.recommend");
    // SEC-IDOR: 經轉讓記錄反查 animal_id，驗證計畫存取權限（防跨計畫竄改轉讓）
    let existing = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    let scope = access::Scoped::<access::AnimalWrite>::authorize(
        &state.db,
        &current_user,
        existing.animal_id,
    )
    .await?;

    // Audit 已收進 service 層（TRANSFER_VET_EVALUATE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalTransferService::vet_evaluate_transfer(&state.db, &actor, scope, transfer_id, &req)
            .await?;
    Ok(Json(record))
}

/// 步驟 3：指定新計劃
pub async fn assign_transfer_plan(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<AssignTransferPlanRequest>,
) -> Result<Json<AnimalTransfer>> {
    // P0-3 SoD：指定轉入計畫屬協調段，與發起 / 完成 / 拒絕同一把關。
    require_permission!(current_user, "animal.transfer.manage");
    // SEC-IDOR: 經轉讓記錄反查 animal_id，驗證計畫存取權限
    let existing = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    let scope = access::Scoped::<access::AnimalWrite>::authorize(
        &state.db,
        &current_user,
        existing.animal_id,
    )
    .await?;

    // Audit 已收進 service 層（TRANSFER_ASSIGN_PLAN，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalTransferService::assign_transfer_plan(&state.db, &actor, scope, transfer_id, &req)
            .await?;
    Ok(Json(record))
}

/// 步驟 4：PI 同意
#[utoipa::path(post, path = "/api/v1/transfers/{transfer_id}/approve", params(("transfer_id" = Uuid, Path, description = "轉讓 ID")), responses((status = 200, body = AnimalTransfer), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn approve_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<AnimalTransfer>> {
    // SEC-IDOR: 經轉讓記錄反查 animal_id，驗證計畫存取權限
    let existing = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    let scope = access::Scoped::<access::AnimalWrite>::authorize(
        &state.db,
        &current_user,
        existing.animal_id,
    )
    .await?;
    // #179 SoD：核准（PI 同意）須具轉讓簽署權責——VET 或轉出/入計劃 PI。
    // 發起人自核另由 service 層擋。
    //
    // P0-3（2026-09-05）：本處原本還疊了一道 `require_permission!("animal.record.create")`，
    // 那是既有缺陷——VET 角色**沒有**該權限碼（`startup/permissions.rs` 的 VET 清單），
    // 於是純獸醫（未兼 EXPERIMENT_STAFF）在走到本檢查之前就被 403 擋掉，
    // 這個函式從來沒有被 VET-only 使用者真正通過過，與下方檢查自己的錯誤訊息
    //「僅獸醫師或轉出 / 轉入計劃主持人可簽署此轉讓」直接矛盾。
    // 簽署權責由 check_transfer_signing_authority 單獨認定，不再另加權限碼閘。
    crate::services::SignatureService::check_transfer_signing_authority(
        &state.db,
        transfer_id,
        &current_user,
    )
    .await?;

    // Audit 已收進 service 層（TRANSFER_APPROVE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalTransferService::approve_transfer(&state.db, &actor, scope, transfer_id).await?;
    Ok(Json(record))
}

/// 步驟 5：完成轉讓
#[utoipa::path(post, path = "/api/v1/transfers/{transfer_id}/complete", params(("transfer_id" = Uuid, Path, description = "轉讓 ID")), responses((status = 200, body = AnimalTransfer), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn complete_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<AnimalTransfer>> {
    // P0-3 SoD：完成轉讓（含 external 離場終態）屬協調段，由執秘按下。
    require_permission!(current_user, "animal.transfer.manage");
    // SEC-IDOR: 經轉讓記錄反查 animal_id，驗證計畫存取權限
    let existing = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    let scope = access::Scoped::<access::AnimalWrite>::authorize(
        &state.db,
        &current_user,
        existing.animal_id,
    )
    .await?;

    // Audit 已收進 service 層（TRANSFER_COMPLETE，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalTransferService::complete_transfer(&state.db, &actor, scope, transfer_id).await?;
    Ok(Json(record))
}

/// 拒絕轉讓
#[utoipa::path(post, path = "/api/v1/transfers/{transfer_id}/reject", params(("transfer_id" = Uuid, Path, description = "轉讓 ID")), request_body = RejectTransferRequest, responses((status = 200, body = AnimalTransfer), (status = 400), (status = 401), (status = 404)), tag = "動物子模組", security(("bearer" = [])))]
pub async fn reject_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<RejectTransferRequest>,
) -> Result<Json<AnimalTransfer>> {
    // P0-3 SoD：拒絕轉讓屬協調段，與發起 / 指定計畫 / 完成同一把關。
    require_permission!(current_user, "animal.transfer.manage");
    // SEC-IDOR: 經轉讓記錄反查 animal_id，驗證計畫存取權限
    let existing = AnimalTransferService::get_transfer(&state.db, transfer_id).await?;
    let scope = access::Scoped::<access::AnimalWrite>::authorize(
        &state.db,
        &current_user,
        existing.animal_id,
    )
    .await?;

    // Audit 已收進 service 層（TRANSFER_REJECT，tx 內）
    let actor = ActorContext::User(current_user.clone());
    let record =
        AnimalTransferService::reject_transfer(&state.db, &actor, scope, transfer_id, &req).await?;
    Ok(Json(record))
}
