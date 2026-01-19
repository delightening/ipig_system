// 電子簽章 API Handlers - GLP 合規

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{SignatureService, AnnotationService, AnnotationType, SignatureType, AuthService},
    AppError, AppState, Result,
};

// ============================================
// Request/Response DTOs
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct SignRecordRequest {
    #[validate(length(min = 1, message = "密碼為必填"))]
    pub password: String,
    pub signature_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignRecordResponse {
    pub signature_id: Uuid,
    pub signed_at: String,
    pub is_locked: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAnnotationRequest {
    #[validate(length(min = 1, message = "內容為必填"))]
    pub content: String,
    pub annotation_type: String,
    pub password: Option<String>, // CORRECTION 類型需要密碼
}

#[derive(Debug, Serialize)]
pub struct AnnotationResponse {
    pub id: Uuid,
    pub annotation_type: String,
    pub content: String,
    pub created_by_name: Option<String>,
    pub created_at: String,
    pub has_signature: bool,
}

#[derive(Debug, Serialize)]
pub struct SignatureStatusResponse {
    pub is_signed: bool,
    pub is_locked: bool,
    pub signatures: Vec<SignatureInfo>,
}

#[derive(Debug, Serialize)]
pub struct SignatureInfo {
    pub id: Uuid,
    pub signature_type: String,
    pub signer_name: Option<String>,
    pub signed_at: String,
}

// ============================================
// Sacrifice Record Signature
// ============================================

/// 為犧牲記錄簽章
pub async fn sign_sacrifice_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sacrifice_id): Path<i32>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.sacrifice");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 驗證密碼
    let user = AuthService::verify_password_by_id(&state.db, current_user.id, &req.password)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    // 取得犧牲記錄內容用於生成雜湊
    let sacrifice_content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'sacrifice_id:', id::text, 
            ',pig_id:', pig_id::text, 
            ',date:', COALESCE(sacrifice_date::text, ''),
            ',confirmed:', confirmed_sacrifice::text
        ) FROM pig_sacrifices WHERE id = $1
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

    // 建立簽章
    let signature = SignatureService::sign(
        &state.db,
        "sacrifice",
        &sacrifice_id.to_string(),
        current_user.id,
        &user.password_hash,
        sig_type,
        &content,
        None,
        None,
    ).await?;

    // 鎖定記錄
    SignatureService::lock_record(&state.db, "sacrifice", sacrifice_id, current_user.id).await?;

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: true,
    }))
}

/// 取得犧牲記錄簽章狀態
pub async fn get_sacrifice_signature_status(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(sacrifice_id): Path<i32>,
) -> Result<Json<SignatureStatusResponse>> {
    let is_signed = SignatureService::is_signed(&state.db, "sacrifice", &sacrifice_id.to_string()).await?;
    let is_locked = SignatureService::is_locked(&state.db, "sacrifice", sacrifice_id).await?;
    
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
pub async fn sign_observation_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(observation_id): Path<i32>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.view");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user = AuthService::verify_password_by_id(&state.db, current_user.id, &req.password)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    let content: Option<String> = sqlx::query_scalar(
        r#"
        SELECT CONCAT(
            'observation_id:', id::text,
            ',pig_id:', pig_id::text,
            ',date:', event_date::text,
            ',content:', content
        ) FROM pig_observations WHERE id = $1
        "#
    )
    .bind(observation_id)
    .fetch_optional(&state.db)
    .await?;

    let content = content.ok_or_else(|| AppError::NotFound("找不到觀察記錄".into()))?;

    let signature = SignatureService::sign(
        &state.db,
        "observation",
        &observation_id.to_string(),
        current_user.id,
        &user.password_hash,
        SignatureType::Confirm,
        &content,
        None,
        None,
    ).await?;

    SignatureService::lock_record(&state.db, "observation", observation_id, current_user.id).await?;

    Ok(Json(SignRecordResponse {
        signature_id: signature.id,
        signed_at: signature.signed_at.to_rfc3339(),
        is_locked: true,
    }))
}

// ============================================
// Annotations
// ============================================

/// 新增附註到已鎖定的記錄
pub async fn add_record_annotation(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((record_type, record_id)): Path<(String, i32)>,
    Json(req): Json<CreateAnnotationRequest>,
) -> Result<Json<AnnotationResponse>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

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
pub async fn get_record_annotations(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path((record_type, record_id)): Path<(String, i32)>,
) -> Result<Json<Vec<AnnotationResponse>>> {
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
