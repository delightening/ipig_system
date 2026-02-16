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
use serde_json::Value as JsonValue;

// ============================================
// Request/Response DTOs
// ============================================

#[derive(Debug, Deserialize, Validate)]
pub struct SignRecordRequest {
    /// 密碼（密碼驗證模式用）
    pub password: Option<String>,
    pub signature_type: Option<String>,
    /// 手寫簽名 SVG（手寫簽名模式用）
    pub handwriting_svg: Option<String>,
    /// 手寫簽名筆跡點資料
    pub stroke_data: Option<JsonValue>,
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
    pub signature_method: Option<String>,
    pub handwriting_svg: Option<String>,
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
        let svg = req.handwriting_svg.as_deref().unwrap();
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
        let password = req.password.as_deref().unwrap();
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
pub async fn sign_observation_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(observation_id): Path<i32>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
    require_permission!(current_user, "animal.record.view");

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
        let svg = req.handwriting_svg.as_deref().unwrap();
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
        let password = req.password.as_deref().unwrap();
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
pub async fn sign_euthanasia_order(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
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
        let svg = req.handwriting_svg.as_deref().unwrap();
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
        let password = req.password.as_deref().unwrap();
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
pub async fn get_euthanasia_signature_status(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
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
pub async fn sign_transfer_record(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
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
        let svg = req.handwriting_svg.as_deref().unwrap();
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
        let password = req.password.as_deref().unwrap();
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
pub async fn get_transfer_signature_status(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
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
pub async fn sign_protocol_review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    Json(req): Json<SignRecordRequest>,
) -> Result<Json<SignRecordResponse>> {
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
        let svg = req.handwriting_svg.as_deref().unwrap();
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
        let password = req.password.as_deref().unwrap();
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
pub async fn get_protocol_signature_status(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
) -> Result<Json<SignatureStatusResponse>> {
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
