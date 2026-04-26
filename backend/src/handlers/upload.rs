use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
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

/// H3 rollback：DB 寫入失敗時清掉已落地的檔案，避免孤兒。
///
/// 失敗只記 warn 不轉錯誤（檔案不存在是預期 idempotent 行為；其他錯誤等
/// cron 清掃兜底，不應遮蔽原本的 DB 錯誤）。CodeRabbit review #207 採納
/// 的 DRY helper，原本三處（handle_upload / sacrifice / sop）重複此 pattern。
async fn cleanup_orphan_upload(file_path: &str, context: &str) {
    if let Err(unlink_err) = FileService::delete(file_path).await {
        tracing::warn!(
            "[{context}] H3 rollback unlink 失敗 path={file_path} err={unlink_err}; \
             檔案孤兒，需後續 cron 清掃"
        );
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

// ─────────────────────────────────────────────────────────────────
// 權限檢查：根據 entity_type 對照上傳端的 require_permission!
// ─────────────────────────────────────────────────────────────────

/// 檢查使用者是否有權存取指定 entity_type 的附件（防範 IDOR）
///
/// 權限對照與上傳端一致：
/// - protocol        → aup.protocol.edit
/// - animal/pathology → animal.animal.edit
/// - vet_recommendation → animal.vet.upload_attachment
/// - observation     → animal.record.create
/// - leave_request   → 本人（uploaded_by）或 hr.leave.view_all
/// - 其他 / 未知     → 僅 Admin
fn check_attachment_permission(
    current_user: &CurrentUser,
    entity_type: &str,
    uploaded_by: Option<Uuid>,
) -> Result<()> {
    // Admin 一律放行
    if current_user.is_admin() {
        return Ok(());
    }

    match entity_type {
        "protocol" => {
            require_permission!(current_user, "aup.protocol.edit");
        }
        "animal" | "pathology" => {
            require_permission!(current_user, "animal.animal.edit");
        }
        "vet_recommendation" => {
            require_permission!(current_user, "animal.vet.upload_attachment");
        }
        "observation" => {
            require_permission!(current_user, "animal.record.create");
        }
        "leave_request" => {
            // 請假附件：上傳者本人可存取，否則需 hr.leave.view_all
            if uploaded_by.is_some_and(|id| id == current_user.id) {
                return Ok(());
            }
            require_permission!(current_user, "hr.leave.view_all");
        }
        _ => {
            return Err(AppError::Forbidden("無權存取此附件".into()));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────
// 通用上傳處理（消除 7 個 handler 的重複代碼）
// ─────────────────────────────────────────────────────────────────

/// 通用附件上傳處理：讀取 multipart 欄位、上傳檔案、寫入 attachments 表
///
/// 串流安全說明：Axum Multipart 以串流方式接收資料，`field.bytes()` 會將單一欄位
/// 完整讀入記憶體。全域 `DefaultBodyLimit`（30 MB）限制了整體請求大小，
/// 此處再以 `category.max_file_size()` 做欄位級檢查，確保不會超出預期。
async fn handle_upload(
    db: &PgPool,
    current_user_id: Uuid,
    category: FileCategory,
    entity_type: &str,
    entity_id: &str,
    multipart: &mut Multipart,
) -> Result<Vec<UploadResponse>> {
    let max_size = category.max_file_size();
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

        // MIME 類型預檢：在讀取檔案資料前就拒絕不允許的類型
        if !category.allowed_mime_types().contains(&content_type.as_str()) {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed for this category",
                content_type
            )));
        }

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {}", e))
        })?;

        // 欄位級大小檢查（全域 DefaultBodyLimit 已限制整體請求，此處做更精細的類別限制）
        if data.len() > max_size {
            return Err(AppError::Validation(format!(
                "File '{}' exceeds maximum allowed size of {} MB",
                file_name,
                max_size / 1024 / 1024
            )));
        }

        let upload_result = FileService::upload(
            category,
            &file_name,
            &content_type,
            &data,
            Some(entity_id),
        )
        .await?;

        // H3 (GLP)：file 與 metadata 的 per-file atomicity。filesystem 不能參與
        // PG tx，故 fail-fast unlink：upload 成功但 save_attachment 失敗時，
        // 立即清掉該 orphan 檔。
        //
        // 已成功 commit 的前一筆檔案保持不動（per-file 原子性，非整批原子性 —
        // 整批跨 fs+DB 原子是不可能的）。返回 Err 時，呼叫端應理解「之前批次
        // 內的成功項已落地」，與 axum handler 的部分成功語意一致。
        if let Err(e) =
            save_attachment(db, entity_type, entity_id, &upload_result, current_user_id).await
        {
            cleanup_orphan_upload(&upload_result.file_path, "handle_upload").await;
            return Err(e);
        }
        results.push(UploadResponse::from(upload_result));
    }

    if results.is_empty() {
        return Err(AppError::Validation("No files uploaded".to_string()));
    }

    Ok(results)
}

// ─────────────────────────────────────────────────────────────────
// 上傳 Handler（已去重，各 handler 僅保留權限檢查與參數差異）
// ─────────────────────────────────────────────────────────────────

/// 上傳 AUP 專案附件
pub async fn upload_protocol_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "aup.protocol.edit");
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::ProtocolAttachment,
        "protocol",
        &protocol_id.to_string(),
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

/// 上傳動物照片
pub async fn upload_animal_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "animal.animal.edit");
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::AnimalPhoto,
        "animal",
        &animal_id.to_string(),
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

/// 上傳病理報告
pub async fn upload_pathology_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "animal.animal.edit");
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::PathologyReport,
        "pathology",
        &animal_id.to_string(),
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

/// 上傳獸醫建議附件
pub async fn upload_vet_recommendation_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((record_type, record_id)): Path<(String, i32)>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "animal.vet.upload_attachment");
    let entity_id = format!("{}_{}", record_type, record_id);
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::VetRecommendation,
        "vet_recommendation",
        &entity_id,
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

/// 上傳觀察紀錄附件（照片與文件）
pub async fn upload_observation_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(observation_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "animal.record.create");
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::ObservationAttachment,
        "observation",
        &observation_id.to_string(),
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

/// 上傳請假附件（診斷證明等）
pub async fn upload_leave_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    let results = handle_upload(
        &state.db,
        current_user.id,
        FileCategory::LeaveAttachment,
        "leave_request",
        &current_user.id.to_string(),
        &mut multipart,
    )
    .await?;
    Ok(Json(results))
}

/// 上傳犧牲記錄照片（有額外的犧牲記錄驗證與不同的存表邏輯，不使用通用函式）
pub async fn upload_sacrifice_photo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(animal_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    require_permission!(current_user, "animal.record.create");

    // 檢查犧牲記錄是否存在（animal_sacrifices.id 為 UUID）
    let sacrifice_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM animal_sacrifices WHERE animal_id = $1",
    )
    .bind(animal_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        AppError::Validation(
            "Sacrifice record not found. Please create the sacrifice record first.".to_string(),
        )
    })?;

    let category = FileCategory::AnimalPhoto;
    let max_size = category.max_file_size();
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

        // MIME 類型預檢
        if !category.allowed_mime_types().contains(&content_type.as_str()) {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed for this category",
                content_type
            )));
        }

        let data = field.bytes().await.map_err(|e| {
            AppError::Validation(format!("Failed to read file data: {}", e))
        })?;

        // 欄位級大小檢查
        if data.len() > max_size {
            return Err(AppError::Validation(format!(
                "File '{}' exceeds maximum allowed size of {} MB",
                file_name,
                max_size / 1024 / 1024
            )));
        }

        let upload_result = FileService::upload(
            category,
            &file_name,
            &content_type,
            &data,
            Some(&animal_id.to_string()),
        )
        .await?;

        // H3 (GLP)：file 與 metadata 的 per-file atomicity，同 handle_upload。
        if let Err(e) = save_animal_record_attachment(
            &state.db,
            "sacrifice",
            sacrifice_id,
            "photo",
            &upload_result,
        )
        .await
        {
            cleanup_orphan_upload(&upload_result.file_path, "sacrifice upload").await;
            return Err(e);
        }

        results.push(UploadResponse::from(upload_result));
    }

    if results.is_empty() {
        return Err(AppError::Validation("No files uploaded".to_string()));
    }

    Ok(Json(results))
}

// ─────────────────────────────────────────────────────────────────
// 讀取 / 下載 / 刪除 Handler（已加入 IDOR 權限檢查）
// ─────────────────────────────────────────────────────────────────

/// 列出附件清單
pub async fn list_attachments(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<UploadQuery>,
) -> Result<Json<Vec<Attachment>>> {
    let entity_type = query.entity_type
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("entity_type 為必填參數".to_string()))?;
    let entity_id = query.entity_id.unwrap_or_default();

    // IDOR 防護：根據 entity_type 檢查使用者權限
    check_attachment_permission(&current_user, &entity_type, None)?;

    // entity_id 在 DB 為 UUID，用 entity_id::text 回傳以符合 struct 的 String 型別
    let attachments: Vec<Attachment> = sqlx::query_as(
        r#"
        SELECT id, entity_type, entity_id::text AS entity_id, file_name, file_path,
               file_size, mime_type, uploaded_by, created_at
        FROM attachments
        WHERE entity_type = $1 AND entity_id::text = $2
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
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    // 從資料庫查詢附件資訊（entity_id::text 以符合 struct 的 String 型別）
    let attachment: Attachment = sqlx::query_as(
        r#"SELECT id, entity_type, entity_id::text AS entity_id, file_name, file_path,
                  file_size, mime_type, uploaded_by, created_at
           FROM attachments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // IDOR 防護：根據 entity_type 與上傳者檢查使用者權限
    check_attachment_permission(
        &current_user,
        &attachment.entity_type,
        Some(attachment.uploaded_by),
    )?;

    // 讀取檔案資料
    let (data, _) = FileService::read(&attachment.file_path).await?;

    // 建立回應
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &attachment.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&attachment.file_name),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?;

    Ok(response)
}

/// 刪除附件
pub async fn delete_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    // 查詢附件資訊（entity_id::text 以符合 struct 的 String 型別）
    let attachment: Attachment = sqlx::query_as(
        r#"SELECT id, entity_type, entity_id::text AS entity_id, file_name, file_path,
                  file_size, mime_type, uploaded_by, created_at
           FROM attachments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // 檢查權限，只有上傳者或管理員可以刪除
    let is_admin = current_user.is_admin();
    if attachment.uploaded_by != current_user.id && !is_admin {
        return Err(AppError::Forbidden(
            "You can only delete your own attachments".to_string(),
        ));
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

// ─────────────────────────────────────────────────────────────────
// DB 輔助函式
// ─────────────────────────────────────────────────────────────────

/// 儲存動物記錄附件到 animal_record_attachments 表（record_id 為 UUID）
async fn save_animal_record_attachment(
    db: &PgPool,
    record_type: &str,
    record_id: Uuid,
    file_type: &str,
    upload_result: &UploadResult,
) -> Result<uuid::Uuid> {
    let id: (uuid::Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO animal_record_attachments (id, record_type, record_id, file_type, file_name, file_path, file_size, mime_type, created_at)
        VALUES (gen_random_uuid(), $1::animal_record_type, $2, $3::animal_file_type, $4, $5, $6, $7, NOW())
        RETURNING id
        "#,
    )
    .bind(record_type)
    .bind(record_id)
    .bind(file_type)
    .bind(&upload_result.file_name)
    .bind(&upload_result.file_path)
    .bind(upload_result.file_size)
    .bind(&upload_result.mime_type)
    .fetch_one(db)
    .await?;

    Ok(id.0)
}

// ─────────────────────────────────────────────────────────────────
// SOP 文件上傳 / 下載
// ─────────────────────────────────────────────────────────────────

/// 上傳 SOP 文件（PDF / Word），寫入檔案系統並更新 qa_sop_documents.file_path
pub async fn upload_sop_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sop_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    require_permission!(current_user, "qau.sop.manage");

    // 確認 SOP 記錄存在
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM qa_sop_documents WHERE id = $1)",
    )
    .bind(sop_id)
    .fetch_one(&state.db)
    .await?;

    if !exists {
        return Err(AppError::NotFound("SOP document not found".to_string()));
    }

    let category = FileCategory::SopDocument;
    let max_size = category.max_file_size();

    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {e}")))?
        .ok_or_else(|| AppError::Validation("No file uploaded".to_string()))?;

    let file_name = field
        .file_name()
        .map(String::from)
        .unwrap_or_else(|| "unnamed".to_string());

    let content_type = field
        .content_type()
        .map(String::from)
        .unwrap_or_else(|| "application/octet-stream".to_string());

    if !category
        .allowed_mime_types()
        .contains(&content_type.as_str())
    {
        return Err(AppError::Validation(format!(
            "File type '{}' is not allowed. Only PDF and Word documents are accepted.",
            content_type
        )));
    }

    let data = field.bytes().await.map_err(|e| {
        AppError::Validation(format!("Failed to read file data: {e}"))
    })?;

    if data.len() > max_size {
        return Err(AppError::Validation(format!(
            "File '{}' exceeds maximum allowed size of {} MB",
            file_name,
            max_size / 1024 / 1024
        )));
    }

    let upload_result = FileService::upload(
        category,
        &file_name,
        &content_type,
        &data,
        Some(&sop_id.to_string()),
    )
    .await?;

    // H3 (GLP)：UPDATE qa_sop_documents 失敗 OR rows_affected=0（sop_id 在檢查
    // 後被刪除）→ unlink 上傳檔避免孤兒。Gemini review #207：rows_affected=0
    // 是「Ok 但無更新」的隱性洞，必須與 Err 同等處理。
    let update_result =
        sqlx::query("UPDATE qa_sop_documents SET file_path = $1, updated_at = NOW() WHERE id = $2")
            .bind(&upload_result.file_path)
            .bind(sop_id)
            .execute(&state.db)
            .await;

    let (rollback_reason, return_err) = match update_result {
        Err(e) => ("update_error", AppError::Database(e)),
        Ok(res) if res.rows_affected() == 0 => (
            "sop_not_found",
            AppError::NotFound("SOP document not found".to_string()),
        ),
        Ok(_) => return Ok(Json(UploadResponse::from(upload_result))),
    };

    cleanup_orphan_upload(
        &upload_result.file_path,
        &format!("sop upload reason={rollback_reason}"),
    )
    .await;
    Err(return_err)
}

/// 下載 SOP 文件
pub async fn download_sop_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(sop_id): Path<Uuid>,
) -> Result<Response> {
    require_permission!(current_user, "qau.sop.view");

    let row: Option<(Option<String>, String)> = sqlx::query_as(
        "SELECT file_path, title FROM qa_sop_documents WHERE id = $1",
    )
    .bind(sop_id)
    .fetch_optional(&state.db)
    .await?;

    let (file_path, title) = row
        .ok_or_else(|| AppError::NotFound("SOP document not found".to_string()))?;

    let file_path = file_path
        .ok_or_else(|| AppError::NotFound("No file uploaded for this SOP".to_string()))?;

    let (data, mime_type) = FileService::read(&file_path).await?;

    // 從 file_path 取得副檔名
    let ext = std::path::Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("pdf");
    let download_name = format!("{}.{}", title, ext);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            crate::utils::http::content_disposition_header(&download_name),
        )
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))?;

    Ok(response)
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
