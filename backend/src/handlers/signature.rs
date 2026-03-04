// 電子簽章 API Handlers - GLP 合規

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{SignatureService, AnnotationService, AnnotationType, SignatureType, AuthService},
    AppError, AppState, Result,
};

// ============================================
// IDOR 防護：記錄存取權限檢查
// ============================================

/// 檢查使用者是否有權存取安樂死單據（PI、VET、CHAIR 或管理員）
async fn check_euthanasia_access(
    db: &sqlx::PgPool,
    order_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    if current_user.has_permission("animal.euthanasia.arbitrate") || current_user.is_admin() {
        return Ok(());
    }
    let related: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT pi_user_id, vet_user_id FROM euthanasia_orders WHERE id = $1"
    )
    .bind(order_id)
    .fetch_optional(db)
    .await?;

    match related {
        Some((pi_id, vet_id)) if pi_id == current_user.id || vet_id == current_user.id => Ok(()),
        Some(_) => Err(AppError::Forbidden("無權存取此安樂死單據".into())),
        None => Err(AppError::NotFound("找不到安樂死單據".into())),
    }
}

/// 檢查使用者是否有權存取轉讓記錄（透過動物所屬計畫關聯）
async fn check_transfer_access(
    db: &sqlx::PgPool,
    transfer_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
        return Ok(());
    }
    let has_access: Option<(i64,)> = sqlx::query_as(
        r#"SELECT 1 FROM animal_transfers t
           JOIN animals a ON t.animal_id = a.id
           LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
           WHERE t.id = $1 AND up.user_id = $2"#
    )
    .bind(transfer_id)
    .bind(current_user.id)
    .fetch_optional(db)
    .await?;

    if has_access.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此轉讓記錄".into()))
    }
}

/// 檢查使用者是否有權存取計畫書（PI、共同編輯者、審查委員或管理員）
async fn check_protocol_access(
    db: &sqlx::PgPool,
    protocol_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
        return Ok(());
    }
    let has_access: Option<(i64,)> = sqlx::query_as(
        r#"SELECT 1 FROM user_protocols
           WHERE protocol_id = $1 AND user_id = $2"#
    )
    .bind(protocol_id)
    .bind(current_user.id)
    .fetch_optional(db)
    .await?;

    if has_access.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此計畫書".into()))
    }
}

/// 檢查使用者是否有權存取犧牲/觀察記錄（透過動物所屬計畫關聯，記錄 ID 為 i32）
async fn check_animal_record_access(
    db: &sqlx::PgPool,
    table: &str,
    record_id: i32,
    current_user: &CurrentUser,
) -> Result<()> {
    if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
        return Ok(());
    }
    let query = format!(
        r#"SELECT 1 FROM {} r
           JOIN animals a ON r.animal_id = a.id
           LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
           WHERE r.id = $1 AND up.user_id = $2"#,
        table
    );
    let has_access: Option<(i64,)> = sqlx::query_as(&query)
        .bind(record_id)
        .bind(current_user.id)
        .fetch_optional(db)
        .await?;

    if has_access.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此記錄".into()))
    }
}

/// 檢查使用者是否有權存取犧牲記錄（animal_sacrifices.id 為 UUID）
async fn check_animal_record_access_uuid(
    db: &sqlx::PgPool,
    table: &str,
    record_id: Uuid,
    current_user: &CurrentUser,
) -> Result<()> {
    if current_user.has_permission("aup.protocol.view_all") || current_user.is_admin() {
        return Ok(());
    }
    let query = format!(
        r#"SELECT 1 FROM {} r
           JOIN animals a ON r.animal_id = a.id
           LEFT JOIN user_protocols up ON up.protocol_id = a.protocol_id
           WHERE r.id = $1 AND up.user_id = $2"#,
        table
    );
    let has_access: Option<(i64,)> = sqlx::query_as(&query)
        .bind(record_id)
        .bind(current_user.id)
        .fetch_optional(db)
        .await?;

    if has_access.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此記錄".into()))
    }
}
use serde_json::Value as JsonValue;

// ============================================
// Request/Response DTOs
// ============================================

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct SignRecordRequest {
    /// 密碼（密碼驗證模式用）
    pub password: Option<String>,
    pub signature_type: Option<String>,
    /// 手寫簽名 SVG（手寫簽名模式用）
    pub handwriting_svg: Option<String>,
    /// 手寫簽名筆跡點資料
    pub stroke_data: Option<JsonValue>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignRecordResponse {
    pub signature_id: Uuid,
    pub signed_at: String,
    pub is_locked: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateAnnotationRequest {
    #[validate(length(min = 1, message = "內容為必填"))]
    pub content: String,
    pub annotation_type: String,
    pub password: Option<String>, // CORRECTION 類型需要密碼
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AnnotationResponse {
    pub id: Uuid,
    pub annotation_type: String,
    pub content: String,
    pub created_by_name: Option<String>,
    pub created_at: String,
    pub has_signature: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignatureStatusResponse {
    pub is_signed: bool,
    pub is_locked: bool,
    pub signatures: Vec<SignatureInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SignatureInfo {
    pub id: Uuid,
    pub signature_type: String,
    pub signer_name: Option<String>,
    pub signed_at: String,
    pub signature_method: Option<String>,
    pub handwriting_svg: Option<String>,
}

// ============================================
// Sacrifice Record Signature
// ============================================

/// 為犧牲記錄簽章
#[utoipa::path(
    post,
    path = "/api/signatures/sacrifice/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到犧牲記錄")
    ),
    params(("id" = Uuid, Path, description = "犧牲記錄 ID (UUID)")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_sacrifice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sacrifice_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.sacrifice");
    check_animal_record_access_uuid(&state.db, "animal_sacrifices", sacrifice_id, &current_user).await?;

    // 驗證：密碼或手寫簽名擇一
    let has_password = req.password.as_ref().is_some_and(|p| !p.is_empty());
    let has_handwriting = req.handwriting_svg.as_ref().is_some_and(|s| !s.is_empty());
    if !has_password && !has_handwriting {
        return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
    }

    // 取得犧牲記錄內容用於生成雜湊
    let sacrifice_content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'sacrifice_id:', id::text, 
            ',animal_id:', animal_id::text, 
            ',date:', COALESCE(sacrifice_date::text, ''),
            ',confirmed:', confirmed_sacrifice::text
        ) FROM animal_sacrifices WHERE id = $1
        "#
    )
    .bind(sacrifice_id)
    .fetch_optional(&state.db)
    .await?;

    let content = sacrifice_content
        .ok_or_else(|| AppError::NotFound("找不到犧牲記錄".into()))?;

    let sig_type = match req.signature_type.as_deref() {
        Some("WITNESS") => SignatureType::Witness,
        Some("APPROVE") => SignatureType::Approve,
        _ => SignatureType::Confirm,
    };

    // 依簽章方式建立簽章
    let signature = if has_handwriting {
        let svg = req.handwriting_svg.as_deref()
            .ok_or_else(|| AppError::Internal("missing handwriting SVG".into()))?;
        SignatureService::sign_with_handwriting(
            &state.db,
            "sacrifice",
            &sacrifice_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            None,
            None,
            svg,
            req.stroke_data.as_ref(),
        ).await?
    } else {
        let password = req.password.as_deref()
            .ok_or_else(|| AppError::Internal("missing password".into()))?;
        let user = AuthService::verify_password_by_id(&state.db, current_user.id, password)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        SignatureService::sign(
            &state.db,
            "sacrifice",
            &sacrifice_id.to_string(),
            current_user.id,
            &user.password_hash,
            sig_type,
            &content,
            None,
            None,
        ).await?
    };

    // 鎖定記錄（animal_sacrifices.id 為 UUID）
    SignatureService::lock_record_uuid(&state.db, "sacrifice", sacrifice_id, current_user.id).await?;

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: true,
    }))
}

/// 取得犧牲記錄簽章狀態
#[utoipa::path(
    get,
    path = "/api/signatures/sacrifice/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "犧牲記錄 ID (UUID)")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_sacrifice_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sacrifice_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    require_permission!(current_user, "animal.record.view");
    check_animal_record_access_uuid(&state.db, "animal_sacrifices", sacrifice_id, &current_user).await?;
    let is_signed = SignatureService::is_signed(&state.db, "sacrifice", &sacrifice_id.to_string()).await?;
    let is_locked = SignatureService::is_locked_uuid(&state.db, "sacrifice", sacrifice_id).await?;
    
    let signatures = SignatureService::get_signatures(&state.db, "sacrifice", &sacrifice_id.to_string()).await?;
    
    let mut signature_infos = Vec::new();
    for sig in signatures {
        let signer_name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1"
        )
        .bind(sig.signer_id)
        .fetch_optional(&state.db)
        .await?;

        signature_infos.push(SignatureInfo {
            id: sig.id,
            signature_type: sig.signature_type,
            signer_name,
            signed_at: sig.signed_at.to_rfc3339(),
            signature_method: sig.signature_method,
            handwriting_svg: sig.handwriting_svg,
        });
    }

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked,
        signatures: signature_infos,
    }))
}

// ============================================
// Observation Record Signature
// ============================================

/// 為觀察記錄簽章
#[utoipa::path(
    post,
    path = "/api/signatures/observation/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到觀察記錄")
    ),
    params(("id" = i32, Path, description = "觀察記錄 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_observation_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(observation_id): Path<i32>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.view");
    check_animal_record_access(&state.db, "animal_observations", observation_id, &current_user).await?;

    // 驗證：密碼或手寫簽名擇一
    let has_password = req.password.as_ref().is_some_and(|p| !p.is_empty());
    let has_handwriting = req.handwriting_svg.as_ref().is_some_and(|s| !s.is_empty());
    if !has_password && !has_handwriting {
        return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
    }

    let content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'observation_id:', id::text,
            ',animal_id:', animal_id::text,
            ',date:', event_date::text,
            ',content:', content
        ) FROM animal_observations WHERE id = $1
        "#
    )
    .bind(observation_id)
    .fetch_optional(&state.db)
    .await?;

    let content = content.ok_or_else(|| AppError::NotFound("找不到觀察記錄".into()))?;

    // 依簽章方式建立簽章
    let signature = if has_handwriting {
        let svg = req.handwriting_svg.as_deref()
            .ok_or_else(|| AppError::Internal("missing handwriting SVG".into()))?;
        SignatureService::sign_with_handwriting(
            &state.db,
            "observation",
            &observation_id.to_string(),
            current_user.id,
            SignatureType::Confirm,
            &content,
            None,
            None,
            svg,
            req.stroke_data.as_ref(),
        ).await?
    } else {
        let password = req.password.as_deref()
            .ok_or_else(|| AppError::Internal("missing password".into()))?;
        let user = AuthService::verify_password_by_id(&state.db, current_user.id, password)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        SignatureService::sign(
            &state.db,
            "observation",
            &observation_id.to_string(),
            current_user.id,
            &user.password_hash,
            SignatureType::Confirm,
            &content,
            None,
            None,
        ).await?
    };

    SignatureService::lock_record(&state.db, "observation", observation_id, current_user.id).await?;

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: true,
    }))
}

// ============================================
// Euthanasia Order Signature
// ============================================

/// 為安樂死單據簽章（PI 同意 / 獸醫執行）
#[utoipa::path(
    post,
    path = "/api/signatures/euthanasia/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到安樂死單據")
    ),
    params(("id" = Uuid, Path, description = "安樂死單據 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_euthanasia_order(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    check_euthanasia_access(&state.db, order_id, &current_user).await?;

    // 驗證：密碼或手寫簽名擇一
    let has_password = req.password.as_ref().is_some_and(|p| !p.is_empty());
    let has_handwriting = req.handwriting_svg.as_ref().is_some_and(|s| !s.is_empty());
    if !has_password && !has_handwriting {
        return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
    }

    // 取得安樂死單據內容用於生成雜湊
    let euthanasia_content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'euthanasia_id:', id::text,
            ',animal_id:', animal_id::text,
            ',reason:', reason,
            ',status:', status
        ) FROM euthanasia_orders WHERE id = $1
        "#
    )
    .bind(order_id)
    .fetch_optional(&state.db)
    .await?;

    let content = euthanasia_content
        .ok_or_else(|| AppError::NotFound("找不到安樂死單據".into()))?;

    let sig_type = match req.signature_type.as_deref() {
        Some("APPROVE") => SignatureType::Approve,
        Some("WITNESS") => SignatureType::Witness,
        _ => SignatureType::Confirm,
    };

    let signature = if has_handwriting {
        let svg = req.handwriting_svg.as_deref()
            .ok_or_else(|| AppError::Internal("missing handwriting SVG".into()))?;
        SignatureService::sign_with_handwriting(
            &state.db,
            "euthanasia",
            &order_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            None,
            None,
            svg,
            req.stroke_data.as_ref(),
        ).await?
    } else {
        let password = req.password.as_deref()
            .ok_or_else(|| AppError::Internal("missing password".into()))?;
        let user = AuthService::verify_password_by_id(&state.db, current_user.id, password)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        SignatureService::sign(
            &state.db,
            "euthanasia",
            &order_id.to_string(),
            current_user.id,
            &user.password_hash,
            sig_type,
            &content,
            None,
            None,
        ).await?
    };

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: false, // 安樂死單據由狀態機管理，不使用 lock 機制
    }))
}

/// 取得安樂死單據簽章狀態
#[utoipa::path(
    get,
    path = "/api/signatures/euthanasia/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "安樂死單據 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_euthanasia_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    check_euthanasia_access(&state.db, order_id, &current_user).await?;
    let is_signed = SignatureService::is_signed(&state.db, "euthanasia", &order_id.to_string()).await?;

    let signatures = SignatureService::get_signatures(&state.db, "euthanasia", &order_id.to_string()).await?;

    let mut signature_infos = Vec::new();
    for sig in signatures {
        let signer_name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1"
        )
        .bind(sig.signer_id)
        .fetch_optional(&state.db)
        .await?;

        signature_infos.push(SignatureInfo {
            id: sig.id,
            signature_type: sig.signature_type,
            signer_name,
            signed_at: sig.signed_at.to_rfc3339(),
            signature_method: sig.signature_method,
            handwriting_svg: sig.handwriting_svg,
        });
    }

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: signature_infos,
    }))
}

// ============================================
// Transfer Record Signature
// ============================================

/// 為轉讓記錄簽章（PI 同意 / 完成轉讓）
#[utoipa::path(
    post,
    path = "/api/signatures/transfer/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到轉讓記錄")
    ),
    params(("id" = Uuid, Path, description = "轉讓記錄 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_transfer_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.create");
    check_transfer_access(&state.db, transfer_id, &current_user).await?;

    // 驗證：密碼或手寫簽名擇一
    let has_password = req.password.as_ref().is_some_and(|p| !p.is_empty());
    let has_handwriting = req.handwriting_svg.as_ref().is_some_and(|s| !s.is_empty());
    if !has_password && !has_handwriting {
        return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
    }

    // 取得轉讓記錄內容用於生成雜湊
    let transfer_content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'transfer_id:', id::text,
            ',animal_id:', animal_id::text,
            ',from_iacuc:', from_iacuc_no,
            ',status:', status
        ) FROM animal_transfers WHERE id = $1
        "#
    )
    .bind(transfer_id)
    .fetch_optional(&state.db)
    .await?;

    let content = transfer_content
        .ok_or_else(|| AppError::NotFound("找不到轉讓記錄".into()))?;

    let sig_type = match req.signature_type.as_deref() {
        Some("APPROVE") => SignatureType::Approve,
        Some("WITNESS") => SignatureType::Witness,
        _ => SignatureType::Confirm,
    };

    let signature = if has_handwriting {
        let svg = req.handwriting_svg.as_deref()
            .ok_or_else(|| AppError::Internal("missing handwriting SVG".into()))?;
        SignatureService::sign_with_handwriting(
            &state.db,
            "transfer",
            &transfer_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            None,
            None,
            svg,
            req.stroke_data.as_ref(),
        ).await?
    } else {
        let password = req.password.as_deref()
            .ok_or_else(|| AppError::Internal("missing password".into()))?;
        let user = AuthService::verify_password_by_id(&state.db, current_user.id, password)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        SignatureService::sign(
            &state.db,
            "transfer",
            &transfer_id.to_string(),
            current_user.id,
            &user.password_hash,
            sig_type,
            &content,
            None,
            None,
        ).await?
    };

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: false, // 轉讓由狀態機管理
    }))
}

/// 取得轉讓記錄簽章狀態
#[utoipa::path(
    get,
    path = "/api/signatures/transfer/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "轉讓記錄 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_transfer_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    check_transfer_access(&state.db, transfer_id, &current_user).await?;
    let is_signed = SignatureService::is_signed(&state.db, "transfer", &transfer_id.to_string()).await?;

    let signatures = SignatureService::get_signatures(&state.db, "transfer", &transfer_id.to_string()).await?;

    let mut signature_infos = Vec::new();
    for sig in signatures {
        let signer_name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1"
        )
        .bind(sig.signer_id)
        .fetch_optional(&state.db)
        .await?;

        signature_infos.push(SignatureInfo {
            id: sig.id,
            signature_type: sig.signature_type,
            signer_name,
            signed_at: sig.signed_at.to_rfc3339(),
            signature_method: sig.signature_method,
            handwriting_svg: sig.handwriting_svg,
        });
    }

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: signature_infos,
    }))
}

// ============================================
// Protocol Review Signature
// ============================================

/// 為計劃審查簽章（IACUC 委員核准）
#[utoipa::path(
    post,
    path = "/api/signatures/protocol/{id}",
    request_body = SignRecordRequest,
    responses(
        (status = 200, description = "簽章成功", body = SignRecordResponse),
        (status = 400, description = "請提供密碼或手寫簽名"),
        (status = 401, description = "未授權或密碼錯誤"),
        (status = 404, description = "找不到計劃書")
    ),
    params(("id" = Uuid, Path, description = "計劃書 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn sign_protocol_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    check_protocol_access(&state.db, protocol_id, &current_user).await?;

    // 驗證：密碼或手寫簽名擇一
    let has_password = req.password.as_ref().is_some_and(|p| !p.is_empty());
    let has_handwriting = req.handwriting_svg.as_ref().is_some_and(|s| !s.is_empty());
    if !has_password && !has_handwriting {
        return Err(AppError::Validation("請提供密碼或手寫簽名".into()));
    }

    // 取得計劃內容用於生成雜湊
    let protocol_content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'protocol_id:', id::text,
            ',title:', title,
            ',status:', status
        ) FROM protocols WHERE id = $1
        "#
    )
    .bind(protocol_id)
    .fetch_optional(&state.db)
    .await?;

    let content = protocol_content
        .ok_or_else(|| AppError::NotFound("找不到計劃書".into()))?;

    let sig_type = match req.signature_type.as_deref() {
        Some("CONFIRM") => SignatureType::Confirm,
        Some("WITNESS") => SignatureType::Witness,
        _ => SignatureType::Approve,  // 審查預設為核准
    };

    let signature = if has_handwriting {
        let svg = req.handwriting_svg.as_deref()
            .ok_or_else(|| AppError::Internal("missing handwriting SVG".into()))?;
        SignatureService::sign_with_handwriting(
            &state.db,
            "protocol",
            &protocol_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            None,
            None,
            svg,
            req.stroke_data.as_ref(),
        ).await?
    } else {
        let password = req.password.as_deref()
            .ok_or_else(|| AppError::Internal("missing password".into()))?;
        let user = AuthService::verify_password_by_id(&state.db, current_user.id, password)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        SignatureService::sign(
            &state.db,
            "protocol",
            &protocol_id.to_string(),
            current_user.id,
            &user.password_hash,
            sig_type,
            &content,
            None,
            None,
        ).await?
    };

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: false,
    }))
}

/// 取得計劃審查簽章狀態
#[utoipa::path(
    get,
    path = "/api/signatures/protocol/{id}",
    responses(
        (status = 200, description = "簽章狀態", body = SignatureStatusResponse),
        (status = 404, description = "找不到記錄")
    ),
    params(("id" = Uuid, Path, description = "計劃書 ID")),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_protocol_signature_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
    check_protocol_access(&state.db, protocol_id, &current_user).await?;
    let is_signed = SignatureService::is_signed(&state.db, "protocol", &protocol_id.to_string()).await?;

    let signatures = SignatureService::get_signatures(&state.db, "protocol", &protocol_id.to_string()).await?;

    let mut signature_infos = Vec::new();
    for sig in signatures {
        let signer_name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1"
        )
        .bind(sig.signer_id)
        .fetch_optional(&state.db)
        .await?;

        signature_infos.push(SignatureInfo {
            id: sig.id,
            signature_type: sig.signature_type,
            signer_name,
            signed_at: sig.signed_at.to_rfc3339(),
            signature_method: sig.signature_method,
            handwriting_svg: sig.handwriting_svg,
        });
    }

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: signature_infos,
    }))
}

// ============================================
// Annotations
// ============================================

/// 新增附註到已鎖定的記錄
#[utoipa::path(
    post,
    path = "/api/annotations/{record_type}/{record_id}",
    request_body = CreateAnnotationRequest,
    responses(
        (status = 200, description = "附註建立成功", body = AnnotationResponse),
        (status = 400, description = "只能對已鎖定的記錄新增附註或驗證失敗"),
        (status = 401, description = "未授權或密碼錯誤")
    ),
    params(
        ("record_type" = String, Path, description = "紀錄類型 (sacrifice, observation 等)"),
        ("record_id" = i32, Path, description = "紀錄 ID")
    ),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn add_record_annotation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((record_type, record_id)): Path<(String, i32)>,
    Json(req): Json<CreateAnnotationRequest>,
) -> Result<Json<AnnotationResponse>> {
    req.validate()?;

    // 檢查記錄是否已鎖定
    let is_locked = SignatureService::is_locked(&state.db, &record_type, record_id).await?;
    if !is_locked {
        return Err(AppError::Validation("只能對已鎖定的記錄新增附註".into()));
    }

    let annotation_type = match req.annotation_type.as_str() {
        "CORRECTION" => AnnotationType::Correction,
        "ADDENDUM" => AnnotationType::Addendum,
        _ => AnnotationType::Note,
    };

    let mut signature_id = None;

    // 如果是 CORRECTION 類型，需要簽章
    if annotation_type == AnnotationType::Correction {
        let password = req.password
            .ok_or_else(|| AppError::Validation("更正附註需要密碼確認".into()))?;

        let user = AuthService::verify_password_by_id(&state.db, current_user.id, &password)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let signature = SignatureService::sign(
            &state.db,
            &format!("{}_annotation", record_type),
            &record_id.to_string(),
            current_user.id,
            &user.password_hash,
            SignatureType::Confirm,
            &req.content,
            None,
            None,
        ).await?;

        signature_id = Some(signature.id);
    }

    let annotation = AnnotationService::create(
        &state.db,
        &record_type,
        record_id,
        annotation_type,
        &req.content,
        current_user.id,
        signature_id,
    ).await?;

    Ok(Json(AnnotationResponse {
        id: annotation.id,
        annotation_type: annotation.annotation_type,
        content: annotation.content,
        created_by_name: None, // Will be fetched from DB if needed
        created_at: annotation.created_at.to_rfc3339(),
        has_signature: annotation.signature_id.is_some(),
    }))
}

/// 取得記錄的所有附註
#[utoipa::path(
    get,
    path = "/api/annotations/{record_type}/{record_id}",
    responses(
        (status = 200, description = "附註清單", body = [AnnotationResponse])
    ),
    params(
        ("record_type" = String, Path, description = "紀錄類型"),
        ("record_id" = i32, Path, description = "紀錄 ID")
    ),
    tag = "電子簽章",
    security(("bearer" = []))
)]
pub async fn get_record_annotations(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((record_type, record_id)): Path<(String, i32)>,
) -> Result<Json<Vec<AnnotationResponse>>> {
    // IDOR 防護：依記錄類型檢查存取權限
    match record_type.as_str() {
        "sacrifice" => check_animal_record_access(&state.db, "animal_sacrifices", record_id, &current_user).await?,
        "observation" => check_animal_record_access(&state.db, "animal_observations", record_id, &current_user).await?,
        _ => { require_permission!(current_user, "animal.record.view"); }
    }
    let annotations = AnnotationService::get_by_record(&state.db, &record_type, record_id).await?;

    let mut responses = Vec::new();
    for ann in annotations {
        let created_by_name: Option<String> = sqlx::query_scalar(
            "SELECT display_name FROM users WHERE id = $1"
        )
        .bind(ann.created_by)
        .fetch_optional(&state.db)
        .await?;

        responses.push(AnnotationResponse {
            id: ann.id,
            annotation_type: ann.annotation_type,
            content: ann.content,
            created_by_name,
            created_at: ann.created_at.to_rfc3339(),
            has_signature: ann.signature_id.is_some(),
        });
    }

    Ok(Json(responses))
}
