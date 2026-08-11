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
    services::{access, FileCategory, FileService, UploadResult},
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

/// 將附件的 `entity_id`（DB 存為 UUID 字串）解析為 Uuid，失敗時 fail-closed 回 Forbidden
/// （不洩漏「存在與否」，與物件層級守衛一致）。
fn parse_entity_uuid(entity_id: &str) -> Result<Uuid> {
    Uuid::parse_str(entity_id).map_err(|_| AppError::Forbidden("無權存取此附件".into()))
}

/// 檢查使用者是否有權存取指定附件（防範 IDOR）——本函式只用於**讀取**
/// （`list_attachments` / `download_attachment`），刪除走 `delete_attachment`
/// 自己的「上傳者本人或 admin」判準，不經過這裡。
///
/// 兩層檢查（CSO-r3 #1）：
/// 1. 全域角色權限（與上傳端 `require_permission!` 一致，protocol 除外，見下）。
/// 2. **物件層級**存取：將 `entity_id` 解析回所屬動物 / 計畫，套用與其他動物 handler
///    相同的 `access::require_animal_access` / `require_protocol_related_access`，
///    避免持有全域權限但非該計畫成員者（如 EXPERIMENT_STAFF）跨計畫讀取附件。
///
/// 權限與物件對照：
/// - protocol           → `require_protocol_related_access`（view_all 或計畫關聯即可讀，
///   2026-08-14 移除 `aup.protocol.edit` 前置檢查——R76-2 拆除 CO_EDITOR 後，這個權限碼
///   對「能不能讀附件」已無意義；讀取只需要看得到計畫本身，不需要編輯權，見
///   `upload_protocol_attachment` 的寫入端另有更嚴格的 `require_protocol_edit`）
/// - animal / pathology  → animal.animal.edit + 該動物存取權（entity_id = animal_id）
/// - observation        → animal.record.create + 觀察記錄所屬動物存取權
/// - vet_recommendation  → animal.vet.upload_attachment（限 VET，本身即 view_all 角色）
/// - leave_request      → 本人（uploaded_by）或 hr.leave.view_all
/// - 其他 / 未知        → 僅 Admin
async fn check_attachment_permission(
    db: &PgPool,
    current_user: &CurrentUser,
    entity_type: &str,
    entity_id: &str,
    uploaded_by: Option<Uuid>,
) -> Result<()> {
    // Admin 一律放行
    if current_user.is_admin() {
        return Ok(());
    }

    match entity_type {
        "protocol" => {
            let protocol_id = parse_entity_uuid(entity_id)?;
            access::require_protocol_related_access(db, current_user, protocol_id).await?;
        }
        "animal" | "pathology" => {
            require_permission!(current_user, "animal.animal.edit");
            let animal_id = parse_entity_uuid(entity_id)?;
            access::require_animal_access(db, current_user, animal_id).await?;
        }
        "observation" => {
            require_permission!(current_user, "animal.record.create");
            let observation_id = parse_entity_uuid(entity_id)?;
            // R94-4: 反查與授權收進單一入口。維持 Write 強度（原為 require_animal_access），
            // 上傳附件屬寫入動作，不可放寬成 Read。
            let _scope = access::Scoped::<access::AnimalWrite>::from_observation(
                db,
                current_user,
                observation_id,
            )
            .await?;
        }
        "vet_recommendation" => {
            // entity_id 為 `{record_type}_{record_id}` 複合鍵（非 UUID）；此權限限 VET，
            // 而 VET 本身為 view_all 角色，物件層級已由角色涵蓋。
            require_permission!(current_user, "animal.vet.upload_attachment");
        }
        "leave_request" => {
            // 請假附件：上傳者本人可存取，否則需 hr.leave.view_all。
            if uploaded_by.is_some_and(|id| id == current_user.id) {
                return Ok(());
            }
            // CodeRabbit #530：list_attachments 傳 uploaded_by=None，改用 entity_id
            //（上傳時即 = 本人 user_id，見 upload_leave_attachment）判斷所有權，
            // 否則本人無法列出自己的請假附件。
            if parse_entity_uuid(entity_id).is_ok_and(|owner_id| owner_id == current_user.id) {
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
pub(crate) async fn handle_upload(
    db: &PgPool,
    current_user_id: Uuid,
    category: FileCategory,
    entity_type: &str,
    entity_id: &str,
    multipart: &mut Multipart,
) -> Result<Vec<UploadResponse>> {
    const MAX_FILES_PER_REQUEST: usize = 20;
    let max_size = category.max_file_size();
    let mut results = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {}", e)))?
    {
        if results.len() >= MAX_FILES_PER_REQUEST {
            return Err(AppError::Validation(format!(
                "單次上傳最多 {} 個檔案",
                MAX_FILES_PER_REQUEST
            )));
        }
        let file_name = field
            .file_name()
            .map(String::from)
            .unwrap_or_else(|| "unnamed".to_string());

        let content_type = field
            .content_type()
            .map(String::from)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // MIME 類型預檢：在讀取檔案資料前就拒絕不允許的類型
        if !category
            .allowed_mime_types()
            .contains(&content_type.as_str())
        {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed for this category",
                content_type
            )));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Validation(format!("Failed to read file data: {}", e)))?;

        // 欄位級大小檢查（全域 DefaultBodyLimit 已限制整體請求，此處做更精細的類別限制）
        if data.len() > max_size {
            return Err(AppError::Validation(format!(
                "File '{}' exceeds maximum allowed size of {} MB",
                file_name,
                max_size / 1024 / 1024
            )));
        }

        let upload_result =
            FileService::upload(category, &file_name, &content_type, &data, Some(entity_id))
                .await?;

        // H3 (GLP)：file 與 metadata 的 per-file atomicity。filesystem 不能參與
        // PG tx，故 fail-fast unlink：upload 成功但 save_attachment 失敗時，
        // 立即清掉該 orphan 檔。
        //
        // 已成功 commit 的前一筆檔案保持不動（per-file 原子性，非整批原子性 —
        // 整批跨 fs+DB 原子是不可能的）。返回 Err 時，呼叫端應理解「之前批次
        // 內的成功項已落地」，與 axum handler 的部分成功語意一致。
        if let Err(e) = save_attachment(
            db,
            category,
            entity_type,
            entity_id,
            &upload_result,
            current_user_id,
        )
        .await
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
///
/// 2026-08-14：改用物件層級的 `require_protocol_edit`（admin/PI/SD，與
/// `can_edit_protocol` 同一份判準），取代原本只查角色權限碼 `aup.protocol.edit`、
/// 完全沒有物件層級檢查的版本——R76-2 拆除 CO_EDITOR 後，那個權限碼只殘留在
/// EXPERIMENT_STAFF/INTERN 的角色清單裡且已收回，若不換掉會讓這兩個角色連自己
/// 是 SD 的計畫都傳不了附件；改用 `require_protocol_edit` 同時補上原本缺失的
/// 「你是不是這個計畫的相關人」檢查（先前任何持有 aup.protocol.edit 角色權限者
/// 可對任意 protocol_id 上傳，不限自己相關的計畫）。
pub async fn upload_protocol_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(protocol_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Vec<UploadResponse>>> {
    access::require_protocol_edit(&state.db, &current_user, protocol_id).await?;
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
    let sacrifice_id: Uuid =
        sqlx::query_scalar("SELECT id FROM animal_sacrifices WHERE animal_id = $1")
            .bind(animal_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                AppError::Validation(
                    "Sacrifice record not found. Please create the sacrifice record first."
                        .to_string(),
                )
            })?;

    let category = FileCategory::AnimalPhoto;
    let max_size = category.max_file_size();
    let mut results = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {}", e)))?
    {
        let file_name = field
            .file_name()
            .map(String::from)
            .unwrap_or_else(|| "unnamed".to_string());

        let content_type = field
            .content_type()
            .map(String::from)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // MIME 類型預檢
        if !category
            .allowed_mime_types()
            .contains(&content_type.as_str())
        {
            return Err(AppError::Validation(format!(
                "File type '{}' is not allowed for this category",
                content_type
            )));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Validation(format!("Failed to read file data: {}", e)))?;

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
    let entity_type = query
        .entity_type
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::BadRequest("entity_type 為必填參數".to_string()))?;
    let entity_id = query.entity_id.unwrap_or_default();

    // IDOR 防護：全域權限 + 物件層級存取（依 entity_id 解析所屬動物 / 計畫）
    check_attachment_permission(&state.db, &current_user, &entity_type, &entity_id, None).await?;

    // entity_id 在 DB 為 UUID，用 entity_id::text 回傳以符合 struct 的 String 型別
    let attachments: Vec<Attachment> = sqlx::query_as(
        r#"
        SELECT id, entity_type, entity_id::text AS entity_id, file_name, file_path,
               file_size::bigint AS file_size, mime_type, uploaded_by, created_at
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
                  file_size::bigint AS file_size, mime_type, uploaded_by, created_at
           FROM attachments WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Attachment not found".to_string()))?;

    // IDOR 防護：全域權限 + 物件層級存取（依 entity_id 解析所屬動物 / 計畫）
    check_attachment_permission(
        &state.db,
        &current_user,
        &attachment.entity_type,
        &attachment.entity_id,
        Some(attachment.uploaded_by),
    )
    .await?;

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
                  file_size::bigint AS file_size, mime_type, uploaded_by, created_at
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
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM qa_sop_documents WHERE id = $1)")
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

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read file data: {e}")))?;

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

    let row: Option<(Option<String>, String)> =
        sqlx::query_as("SELECT file_path, title FROM qa_sop_documents WHERE id = $1")
            .bind(sop_id)
            .fetch_optional(&state.db)
            .await?;

    let (file_path, title) =
        row.ok_or_else(|| AppError::NotFound("SOP document not found".to_string()))?;

    let file_path =
        file_path.ok_or_else(|| AppError::NotFound("No file uploaded for this SOP".to_string()))?;

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
    category: FileCategory,
    entity_type: &str,
    entity_id: &str,
    upload_result: &UploadResult,
    uploaded_by: Uuid,
) -> Result<Uuid> {
    let id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO attachments (id, category, entity_type, entity_id, file_name, file_path, file_size, mime_type, uploaded_by)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(category.as_str())
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
