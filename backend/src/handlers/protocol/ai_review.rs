//! R20-2/R20-6/R20-7: AI 預審與驗證 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::{
    constants::{ROLE_IACUC_CHAIR, ROLE_IACUC_STAFF, ROLE_SYSTEM_ADMIN},
    middleware::CurrentUser,
    models::ai_review::{AiReviewResponse, BatchReturnRequest, BatchReturnResponse, ValidationResult},
    services::{access, validate_protocol_content, AiReviewService},
    AppError, AppState, Result,
};

/// R20-2: POST /api/v1/protocols/{id}/validate
/// Level 1 規則引擎驗證（無需 AI）
pub async fn validate_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ValidationResult>> {
    // 權限：protocol owner 或 IACUC_STAFF
    check_protocol_access(&state, id, &current_user).await?;

    let protocol = sqlx::query_as::<_, crate::models::Protocol>(
        "SELECT * FROM protocols WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

    let content = protocol
        .working_content
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Protocol has no content".to_string()))?;

    let result = validate_protocol_content(content);
    Ok(Json(result))
}

/// R20-6: POST /api/v1/protocols/{id}/ai-review
/// 客戶端 AI 預審（Level 1 + Level 2）
pub async fn ai_review_protocol(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AiReviewResponse>> {
    // 權限：protocol owner
    check_protocol_owner(&state, id, &current_user).await?;

    // Rate limit 檢查
    let remaining = AiReviewService::remaining_daily_count(&state.db, current_user.id).await?;
    if remaining <= 0 {
        return Err(AppError::TooManyRequests(
            "今日 AI 預審次數已用完（每日上限 10 次）".to_string(),
        ));
    }

    let result = AiReviewService::review_protocol(
        &state.db,
        &state.config,
        id,
        "client_pre_submit",
        Some(current_user.id),
    )
    .await?;

    Ok(Json(result))
}

/// R20-6: GET /api/v1/protocols/{id}/ai-review/latest
pub async fn get_latest_ai_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<AiReviewResponse>>> {
    check_protocol_access(&state, id, &current_user).await?;

    let result = AiReviewService::get_latest(&state.db, id, "client_pre_submit").await?;
    Ok(Json(result))
}

/// R20-6: GET /api/v1/protocols/{id}/ai-review/remaining
pub async fn get_ai_review_remaining(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<serde_json::Value>> {
    let remaining = AiReviewService::remaining_daily_count(&state.db, current_user.id).await?;
    Ok(Json(serde_json::json!({ "remaining": remaining })))
}

/// R20-7: POST /api/v1/protocols/{id}/staff-review-assist
/// 執行秘書 AI 標註
pub async fn staff_review_assist(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<AiReviewResponse>> {
    // 權限：IACUC_STAFF 或 IACUC_CHAIR
    if !current_user
        .roles
        .iter()
        .any(|r| [crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_IACUC_CHAIR, crate::constants::ROLE_SYSTEM_ADMIN].contains(&r.as_str()))
    {
        return Err(AppError::Forbidden(
            "僅限 IACUC 執行秘書或主委使用".to_string(),
        ));
    }

    let result = AiReviewService::review_protocol(
        &state.db,
        &state.config,
        id,
        "staff_pre_review",
        Some(current_user.id),
    )
    .await?;

    Ok(Json(result))
}

/// R20-7: GET /api/v1/protocols/{id}/staff-review-assist/latest
pub async fn get_latest_staff_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<AiReviewResponse>>> {
    if !current_user
        .roles
        .iter()
        .any(|r| [crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_IACUC_CHAIR, crate::constants::ROLE_SYSTEM_ADMIN].contains(&r.as_str()))
    {
        return Err(AppError::Forbidden(
            "僅限 IACUC 執行秘書或主委使用".to_string(),
        ));
    }

    let result = AiReviewService::get_latest(&state.db, id, "staff_pre_review").await?;
    Ok(Json(result))
}

/// POST /api/v1/protocols/{id}/staff-review-assist/batch-return
/// 原子操作：AI 標註轉補件（建立 review_comments + 狀態變更為 PRE_REVIEW_REVISION_REQUIRED）
pub async fn staff_batch_return(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    Json(req): Json<BatchReturnRequest>,
) -> Result<Json<BatchReturnResponse>> {
    let is_staff = current_user.roles.iter().any(|r| {
        [ROLE_IACUC_STAFF, ROLE_IACUC_CHAIR, ROLE_SYSTEM_ADMIN].contains(&r.as_str())
    });
    if !is_staff {
        return Err(AppError::Forbidden("僅限 IACUC 執行秘書或主委使用".to_string()));
    }

    let result = AiReviewService::batch_return(&state.db, protocol_id, &req, current_user.id).await?;
    Ok(Json(result))
}

// ── 輔助函式 ──

async fn check_protocol_access(
    state: &AppState,
    protocol_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    let is_staff = current_user
        .roles
        .iter()
        .any(|r| [crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_IACUC_CHAIR, crate::constants::ROLE_SYSTEM_ADMIN].contains(&r.as_str()));
    if is_staff {
        return Ok(());
    }

    // 檢查是否為 protocol owner
    let is_owner = access::is_pi_or_coeditor(&state.db, protocol_id, current_user.id).await?;
    if !is_owner {
        return Err(AppError::Forbidden(
            "You don't have access to this protocol".to_string(),
        ));
    }
    Ok(())
}

async fn check_protocol_owner(
    state: &AppState,
    protocol_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    let is_owner = access::is_pi_or_coeditor(&state.db, protocol_id, current_user.id).await?;
    let is_staff = current_user
        .roles
        .iter()
        .any(|r| [crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_SYSTEM_ADMIN].contains(&r.as_str()));
    if !is_owner && !is_staff {
        return Err(AppError::Forbidden(
            "只有計畫主持人可以使用 AI 預審".to_string(),
        ));
    }
    Ok(())
}
