use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        ChairDecisionRequest, CreateEuthanasiaAppealRequest, CreateEuthanasiaOrderRequest,
    },
    services::EuthanasiaService,
    AppState,
};

/// 輔助結構：安樂死 Email 通知所需資訊
#[derive(FromRow)]
struct EuthanasiaEmailInfo {
    ear_tag: String,
    iacuc_no: Option<String>,
    email: String,
    display_name: String,
    vet_name: Option<String>,
}

/// 建立安樂死單據 (獸醫)
/// POST /api/euthanasia/orders
pub async fn create_order(
    State(state): State<AppState>,
    Extension(auth): Extension<CurrentUser>,
    Json(req): Json<CreateEuthanasiaOrderRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 驗證請求
    req.validate()?;

    // 驗證權限：只有 VET 可以建立
    if !auth.has_permission("animal.euthanasia.create") && !auth.has_role("VET") {
        return Err(AppError::Forbidden("無權限開立安樂死單".to_string()));
    }

    let order = EuthanasiaService::create_order(&state.db, &req, auth.id).await?;

    // 發送 Email 通知給 PI
    // 取得必要資訊
    let animal_email_info = sqlx::query_as::<_, EuthanasiaEmailInfo>(
        r#"
        SELECT p.ear_tag, p.iacuc_no, u.email, u.display_name, vu.display_name as vet_name
        FROM animals p
        JOIN users u ON u.id = $1
        JOIN users vu ON vu.id = $2
        WHERE p.id = $3
        "#,
    )
    .bind(order.pi_user_id)
    .bind(order.vet_user_id)
    .bind(order.animal_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(info) = animal_email_info {
        let deadline = order.deadline_at.format("%Y-%m-%d %H:%M").to_string();
        if let Err(e) = crate::services::EmailService::send_euthanasia_order_email(
            &state.config,
            &info.email,
            &info.display_name,
            &info.ear_tag,
            info.iacuc_no.as_deref(),
            info.vet_name.as_deref().unwrap_or(""),
            &order.reason,
            &deadline,
        ).await {
            tracing::warn!("發送安樂死通知郵件失敗: {e}");
        }

    }

    Ok((StatusCode::CREATED, Json(order)))
}

/// 取得安樂死單據詳情
/// GET /api/euthanasia/orders/{id}
pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(_auth): Extension<CurrentUser>,
) -> Result<impl IntoResponse, AppError> {
    let order = EuthanasiaService::get_order_by_id(&state.db, id).await?;

    Ok(Json(order))
}

/// 取得 PI 的待處理安樂死單據
/// GET /api/euthanasia/orders/pending
pub async fn get_pending_orders(
    State(state): State<AppState>,
    Extension(auth): Extension<CurrentUser>,
) -> Result<impl IntoResponse, AppError> {
    let orders = EuthanasiaService::get_pending_orders_for_pi(&state.db, auth.id).await?;

    Ok(Json(orders))
}

/// PI 同意執行安樂死
/// POST /api/euthanasia/orders/{id}/approve
pub async fn approve_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<CurrentUser>,
) -> Result<impl IntoResponse, AppError> {
    let order = EuthanasiaService::pi_approve(&state.db, id, auth.id).await?;

    Ok(Json(order))
}

/// PI 申請暫緩
/// POST /api/euthanasia/orders/{id}/appeal
pub async fn appeal_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<CurrentUser>,
    Json(req): Json<CreateEuthanasiaAppealRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let appeal = EuthanasiaService::pi_appeal(&state.db, id, auth.id, &req).await?;

    Ok((StatusCode::CREATED, Json(appeal)))
}

/// CHAIR 裁決暫緩申請
/// POST /api/euthanasia/appeals/{id}/decide
pub async fn decide_appeal(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<CurrentUser>,
    Json(req): Json<ChairDecisionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 驗證權限：只有 CHAIR 可以裁決
    if !auth.has_role("IACUC_CHAIR") {
        return Err(AppError::Forbidden("無權限進行仲裁".to_string()));
    }

    let appeal = EuthanasiaService::chair_decide(&state.db, id, auth.id, &req).await?;

    Ok(Json(appeal))
}

/// 執行安樂死
/// POST /api/euthanasia/orders/{id}/execute
pub async fn execute_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(auth): Extension<CurrentUser>,
) -> Result<impl IntoResponse, AppError> {
    // 驗證權限：只有 VET 可以執行
    if !auth.has_permission("animal.euthanasia.execute") && !auth.has_role("VET") {
        return Err(AppError::Forbidden("無權限執行安樂死".to_string()));
    }

    let order = EuthanasiaService::execute(&state.db, id, auth.id).await?;

    Ok(Json(order))
}
