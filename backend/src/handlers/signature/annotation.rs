// 記錄附註 Handlers

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{
        AnnotationService, AnnotationType, SignatureService, SignatureType,
    },
    AppError, AppState, Result,
};

use super::{AnnotationResponse, CreateAnnotationRequest};

/// 新增附註到已鎖定的記錄
#[utoipa::path(
    post,
    path = "/api/v1/annotations/{record_type}/{record_id}",
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
    path = "/api/v1/annotations/{record_type}/{record_id}",
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
