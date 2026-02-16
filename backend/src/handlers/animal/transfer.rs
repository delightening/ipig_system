// 動物轉讓 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        AnimalTransfer, TransferVetEvaluation, DataBoundaryResponse,
        CreateTransferRequest, VetEvaluateTransferRequest,
        AssignTransferPlanRequest, RejectTransferRequest,
    },
    require_permission,
    services::{AnimalService, AuditService},
    AppState, Result,
};

/// 取得資料隔離的時間界線
pub async fn get_animal_data_boundary(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<DataBoundaryResponse>> {
    let result = AnimalService::get_data_boundary(
        &state.db,
        animal_id,
        current_user.id,
        &current_user.roles,
    ).await?;
    Ok(Json(result))
}

/// 取得動物的轉讓記錄列表
pub async fn list_animal_transfers(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
) -> Result<Json<Vec<AnimalTransfer>>> {
    let records = AnimalService::list_transfers(&state.db, animal_id).await?;
    Ok(Json(records))
}

/// 取得單一轉讓記錄
pub async fn get_transfer(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<AnimalTransfer>> {
    let record = AnimalService::get_transfer(&state.db, transfer_id).await?;
    Ok(Json(record))
}

/// 取得轉讓的獸醫評估
pub async fn get_transfer_vet_evaluation(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<Option<TransferVetEvaluation>>> {
    let record = AnimalService::get_transfer_vet_evaluation(&state.db, transfer_id).await?;
    Ok(Json(record))
}

/// 步驟 1：發起轉讓
pub async fn initiate_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    Json(req): Json<CreateTransferRequest>,
) -> Result<Json<AnimalTransfer>> {
    require_permission!(current_user, "animal.record.create");

    let record = AnimalService::initiate_transfer(&state.db, animal_id, &req, current_user.id).await?;

    // 稽核日誌
    let display = match AnimalService::get_by_id(&state.db, animal_id).await {
        Ok(animal) => {
            let iacuc = animal.iacuc_no.as_deref().unwrap_or("未指派");
            format!("[{}] {} - 發起轉讓", iacuc, animal.ear_tag)
        }
        _ => format!("轉讓發起 (animal: {})", animal_id),
    };

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TRANSFER_INITIATE",
        Some("animal_transfers"), Some(record.id),
        Some(&display),
        None,
        Some(serde_json::json!({
            "reason": req.reason,
            "from_iacuc_no": record.from_iacuc_no,
        })),
        None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TRANSFER_INITIATE): {}", e);
    }

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

    let record = AnimalService::vet_evaluate_transfer(&state.db, transfer_id, &req, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TRANSFER_VET_EVALUATE",
        Some("animal_transfers"), Some(transfer_id),
        Some(&format!("轉讓獸醫評估：{}", if req.is_fit_for_transfer { "適合轉讓" } else { "不適合轉讓" })),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TRANSFER_VET_EVALUATE): {}", e);
    }

    Ok(Json(record))
}

/// 步驟 3：指定新計劃
pub async fn assign_transfer_plan(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<AssignTransferPlanRequest>,
) -> Result<Json<AnimalTransfer>> {
    require_permission!(current_user, "animal.record.create");

    let record = AnimalService::assign_transfer_plan(&state.db, transfer_id, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TRANSFER_ASSIGN_PLAN",
        Some("animal_transfers"), Some(transfer_id),
        Some(&format!("轉讓指定新計劃：{}", req.to_iacuc_no)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TRANSFER_ASSIGN_PLAN): {}", e);
    }

    Ok(Json(record))
}

/// 步驟 4：PI 同意
pub async fn approve_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<AnimalTransfer>> {
    require_permission!(current_user, "animal.record.create");

    let record = AnimalService::approve_transfer(&state.db, transfer_id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TRANSFER_APPROVE",
        Some("animal_transfers"), Some(transfer_id),
        Some("PI 同意轉讓"),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TRANSFER_APPROVE): {}", e);
    }

    Ok(Json(record))
}

/// 步驟 5：完成轉讓
pub async fn complete_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<AnimalTransfer>> {
    require_permission!(current_user, "animal.record.create");

    let record = AnimalService::complete_transfer(&state.db, transfer_id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TRANSFER_COMPLETE",
        Some("animal_transfers"), Some(transfer_id),
        Some(&format!("轉讓完成：{} → {}", record.from_iacuc_no, record.to_iacuc_no.as_deref().unwrap_or("未知"))),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TRANSFER_COMPLETE): {}", e);
    }

    Ok(Json(record))
}

/// 拒絕轉讓
pub async fn reject_transfer(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<RejectTransferRequest>,
) -> Result<Json<AnimalTransfer>> {
    require_permission!(current_user, "animal.record.create");

    let record = AnimalService::reject_transfer(&state.db, transfer_id, &req, current_user.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "ANIMAL", "TRANSFER_REJECT",
        Some("animal_transfers"), Some(transfer_id),
        Some(&format!("拒絕轉讓：{}", req.reason)),
        None, None, None, None,
    ).await {
        tracing::error!("寫入 user_activity_logs 失敗 (TRANSFER_REJECT): {}", e);
    }

    Ok(Json(record))
}
