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
    services::{
        AnnotationService, AnnotationType, ElectronicSignature, SignatureInfoDto,
        SignatureService, SignatureType,
    },
    AppError, AppState, Result,
};

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
// 內部輔助函式
// ============================================

/// 將 SignatureInfoDto 轉換為 handler 層的 SignatureInfo
fn to_signature_infos(dtos: Vec<SignatureInfoDto>) -> Vec<SignatureInfo> {
    dtos.into_iter()
        .map(|dto| SignatureInfo {
            id: dto.id,
            signature_type: dto.signature_type,
            signer_name: dto.signer_name,
            signed_at: dto.signed_at.to_rfc3339(),
            signature_method: dto.signature_method,
            handwriting_svg: dto.handwriting_svg,
        })
        .collect()
}

/// 建構簽章回應
fn sign_response(sig: &ElectronicSignature, is_locked: bool) -> SignRecordResponse {
    SignRecordResponse {
        signature_id: sig.id,
        signed_at: sig.signed_at.to_rfc3339(),
        is_locked,
    }
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
    SignatureService::check_animal_record_access_uuid(
        &state.db, "animal_sacrifices", sacrifice_id, &current_user,
    ).await?;

    let content = SignatureService::fetch_sacrifice_content(&state.db, sacrifice_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(), SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db, "sacrifice", &sacrifice_id.to_string(),
        current_user.id, sig_type, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    SignatureService::lock_record_uuid(&state.db, "sacrifice", sacrifice_id, current_user.id).await?;
    Ok(Json(sign_response(&signature, true)))
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
    SignatureService::check_animal_record_access_uuid(
        &state.db, "animal_sacrifices", sacrifice_id, &current_user,
    ).await?;

    let entity_id = sacrifice_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "sacrifice", &entity_id).await?;
    let is_locked = SignatureService::is_locked_uuid(&state.db, "sacrifice", sacrifice_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "sacrifice", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked,
        signatures: to_signature_infos(infos),
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
    SignatureService::check_animal_record_access(
        &state.db, "animal_observations", observation_id, &current_user,
    ).await?;

    let content = SignatureService::fetch_observation_content(&state.db, observation_id).await?;

    let signature = SignatureService::sign_record(
        &state.db, "observation", &observation_id.to_string(),
        current_user.id, SignatureType::Confirm, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    SignatureService::lock_record(&state.db, "observation", observation_id, current_user.id).await?;
    Ok(Json(sign_response(&signature, true)))
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
    SignatureService::check_euthanasia_access(&state.db, order_id, &current_user).await?;

    let content = SignatureService::fetch_euthanasia_content(&state.db, order_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(), SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db, "euthanasia", &order_id.to_string(),
        current_user.id, sig_type, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    Ok(Json(sign_response(&signature, false)))
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
    SignatureService::check_euthanasia_access(&state.db, order_id, &current_user).await?;

    let entity_id = order_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "euthanasia", &entity_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "euthanasia", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: to_signature_infos(infos),
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
    SignatureService::check_transfer_access(&state.db, transfer_id, &current_user).await?;

    let content = SignatureService::fetch_transfer_content(&state.db, transfer_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(), SignatureType::Confirm,
    );

    let signature = SignatureService::sign_record(
        &state.db, "transfer", &transfer_id.to_string(),
        current_user.id, sig_type, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    Ok(Json(sign_response(&signature, false)))
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
    SignatureService::check_transfer_access(&state.db, transfer_id, &current_user).await?;

    let entity_id = transfer_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "transfer", &entity_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "transfer", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: to_signature_infos(infos),
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
    SignatureService::check_protocol_access(&state.db, protocol_id, &current_user).await?;

    let content = SignatureService::fetch_protocol_content(&state.db, protocol_id).await?;
    let sig_type = SignatureService::parse_signature_type(
        req.signature_type.as_deref(), SignatureType::Approve,
    );

    let signature = SignatureService::sign_record(
        &state.db, "protocol", &protocol_id.to_string(),
        current_user.id, sig_type, &content,
        req.password.as_deref(), req.handwriting_svg.as_deref(),
        req.stroke_data.as_ref(),
    ).await?;

    Ok(Json(sign_response(&signature, false)))
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
    SignatureService::check_protocol_access(&state.db, protocol_id, &current_user).await?;

    let entity_id = protocol_id.to_string();
    let is_signed = SignatureService::is_signed(&state.db, "protocol", &entity_id).await?;
    let infos = SignatureService::get_signature_infos(&state.db, "protocol", &entity_id).await?;

    Ok(Json(SignatureStatusResponse {
        is_signed,
        is_locked: false,
        signatures: to_signature_infos(infos),
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

    let is_locked = SignatureService::is_locked(&state.db, &record_type, record_id).await?;
    if !is_locked {
        return Err(AppError::Validation("只能對已鎖定的記錄新增附註".into()));
    }

    let annotation_type = match req.annotation_type.as_str() {
        "CORRECTION" => AnnotationType::Correction,
        "ADDENDUM" => AnnotationType::Addendum,
        _ => AnnotationType::Note,
    };

    let signature_id = if annotation_type == AnnotationType::Correction {
        let sig = SignatureService::sign_record(
            &state.db,
            &format!("{}_annotation", record_type),
            &record_id.to_string(),
            current_user.id,
            SignatureType::Confirm,
            &req.content,
            req.password.as_deref(),
            None,
            None,
        ).await?;
        Some(sig.id)
    } else {
        None
    };

    let annotation = AnnotationService::create(
        &state.db, &record_type, record_id, annotation_type,
        &req.content, current_user.id, signature_id,
    ).await?;

    Ok(Json(AnnotationResponse {
        id: annotation.id,
        annotation_type: annotation.annotation_type,
        content: annotation.content,
        created_by_name: None,
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
    match record_type.as_str() {
        "sacrifice" => SignatureService::check_animal_record_access(
            &state.db, "animal_sacrifices", record_id, &current_user,
        ).await?,
        "observation" => SignatureService::check_animal_record_access(
            &state.db, "animal_observations", record_id, &current_user,
        ).await?,
        _ => { require_permission!(current_user, "animal.record.view"); }
    }

    let annotations = AnnotationService::get_by_record(&state.db, &record_type, record_id).await?;
    let responses = AnnotationService::enrich_with_names(&state.db, annotations).await?;

    Ok(Json(responses.into_iter().map(|(ann, name)| AnnotationResponse {
        id: ann.id,
        annotation_type: ann.annotation_type,
        content: ann.content,
        created_by_name: name,
        created_at: ann.created_at.to_rfc3339(),
        has_signature: ann.signature_id.is_some(),
    }).collect()))
}
