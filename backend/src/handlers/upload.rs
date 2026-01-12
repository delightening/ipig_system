use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    require_permission,
    services::{FileCategory, FileService, UploadResult},
    AppState, Result,
};

/// 上傳回應
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
}

impl From<UploadResult> for UploadResponse {
    fn from(result: UploadResult) -> Self {
        Self {
            id: result.file_id,
            file_name: result.file_name,
            file_path: result.file_path,
            file_size: result.file_size,
            mime_type: result.mime_type,
        }
    }
}

/// 附件資料結構
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub uploaded_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 上傳查詢參數
#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

/// 上傳 AUP 專案附件
pub async fn upload_protocol_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "aup.protocol.edit");

    let mut results = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("Failed to read multipart field: {}", e))
    })? {
        let file_name = field
            .file_name()
            .map(String::from)
            .unwrap_or_else(|| "unnamed".to_string());
        
        let content_type = field
            .content_type()
            .map(String::from)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {}", e))
        })?;

        // 上傳檔案
        let upload_result = FileService::upload(
            FileCategory::ProtocolAttachment,
            &file_name,
            &content_type,
            &data,
            Some(&protocol_id.to_string()),
        ).await?;

        // 儲存附件記錄到資料庫
        save_attachment(
            &state.db,
            "protocol",
            &protocol_id.to_string(),
            &upload_result,
            current_user.id,
        ).await?;

        results.push(UploadResponse::from(upload_result));
    }

    if results.is_empty() {
        return Err(AppError::Validation("No files uploaded".to_string()));
    }

    Ok(Json(results))
}

/// 上傳豬隻照片
pub async fn upload_pig_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "pig.pig.edit");

    let mut results = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("Failed to read multipart field: {}", e))
    })? {
        let file_name = field
            .file_name()
            .map(String::from)
            .unwrap_or_else(|| "unnamed".to_string());
        
        let content_type = field
            .content_type()
            .map(String::from)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {}", e))
        })?;

        let upload_result = FileService::upload(
            FileCategory::PigPhoto,
            &file_name,
            &content_type,
            &data,
            Some(&pig_id.to_string()),
        ).await?;

        save_attachment(
            &state.db,
            "pig",
            &pig_id.to_string(),
            &upload_result,
            current_user.id,
        ).await?;

        results.push(UploadResponse::from(upload_result));
    }

    if results.is_empty() {
        return Err(AppError::Validation("No files uploaded".to_string()));
    }

    Ok(Json(results))
}

/// 上傳病理報告
pub async fn upload_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(pig_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "pig.pig.edit");

    let mut results = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("Failed to read multipart field: {}", e))
    })? {
        let file_name = field
            .file_name()
            .map(String::from)
            .unwrap_or_else(|| "unnamed".to_string());
        
        let content_type = field
            .content_type()
            .map(String::from)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {}", e))
        })?;

        let upload_result = FileService::upload(
            FileCategory::PathologyReport,
            &file_name,
            &content_type,
            &data,
            Some(&pig_id.to_string()),
        ).await?;

        save_attachment(
            &state.db,
            "pathology",
            &pig_id.to_string(),
            &upload_result,
            current_user.id,
        ).await?;

        results.push(UploadResponse::from(upload_result));
    }

    if results.is_empty() {
        return Err(AppError::Validation("No files uploaded".to_string()));
    }

    Ok(Json(results))
}

/// 上傳獸醫建議附件
pub async fn upload_vet_recommendation_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((record_type, record_id)): Path<(String, i32)>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "pig.vet.upload_attachment");

    let mut results = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::Validation(format!("Failed to read multipart field: {}", e))
    })? {
        let file_name = field
            .file_name()
            .map(String::from)
            .unwrap_or_else(|| "unnamed".to_string());
        
        let content_type = field
            .content_type()
            .map(String::from)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {}", e))
        })?;

        let entity_id = format!("{}_{}", record_type, record_id);
        let upload_result = FileService::upload(
            FileCategory::VetRecommendation,
            &file_name,
            &content_type,
            &data,
            Some(&entity_id),
        ).await?;

        save_attachment(
            &state.db,
            "vet_recommendation",
            &entity_id,
            &upload_result,
            current_user.id,
        ).await?;

        results.push(UploadResponse::from(upload_result));
    }

    if results.is_empty() {
        return Err(AppError::Validation("No files uploaded".to_string()));
    }

    Ok(Json(results))
}

/// 列出附件清單
pub async fn list_attachments(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Query(query): Query<UploadQuery>,
) -> Result<Json<Vec<Attachment>>> {
    let entity_type = query.entity_type.unwrap_or_default();
    let entity_id = query.entity_id.unwrap_or_default();

    let attachments: Vec<Attachment> = sqlx::query_as(
        r#"
        SELECT id, entity_type, entity_id, file_name, file_path, 
               file_size, mime_type, uploaded_by, created_at
        FROM attachments
        WHERE entity_type = $1 AND entity_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(&entity_type)
    .bind(&entity_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(attachments))
}

/// 下載附件
pub async fn download_attachment(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    // 從資料庫查詢附件資訊
    let attachment: Attachment = sqlx::query_as(
        r#"SELECT * FROM attachments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // 讀取檔案資料
    let (data, _) = FileService::read(&attachment.file_path).await?;

    // 建立回應
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &attachment.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", attachment.file_name),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// 刪除附件
pub async fn delete_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    // 查詢附件資訊
    let attachment: Attachment = sqlx::query_as(
        r#"SELECT * FROM attachments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // 檢查權限，只有上傳者或管理員可以刪除
    let is_admin = current_user.roles.contains(&"SYSTEM_ADMIN".to_string())
        || current_user.roles.contains(&"admin".to_string());
    if attachment.uploaded_by != current_user.id && !is_admin {
        return Err(AppError::Forbidden("You can only delete your own attachments".to_string()));
    }

    // 刪除檔案
    FileService::delete(&attachment.file_path).await?;

    // 從資料庫刪除記錄
    sqlx::query(r#"DELETE FROM attachments WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// 儲存附件記錄到資料庫
async fn save_attachment(
    db: &PgPool,
    entity_type: &str,
    entity_id: &str,
    upload_result: &UploadResult,
    uploaded_by: Uuid,
) -> Result<Uuid> {
    let id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO attachments (id, entity_type, entity_id, file_name, file_path, file_size, mime_type, uploaded_by)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(&upload_result.file_name)
    .bind(&upload_result.file_path)
    .bind(upload_result.file_size)
    .bind(&upload_result.mime_type)
    .bind(uploaded_by)
    .fetch_one(db)
    .await?;

    Ok(id.0)
}
