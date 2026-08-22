//! R40-A 站內信 handlers

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    require_permission,
    services::{
        CreateThreadRequest, FileService, MessageAttachment, MessageWithSender, MessagingService,
        RecipientSummary, SendMessageRequest, ThreadSummary, ThreadWithMessages,
    },
    AppState, Result,
};

pub async fn list_threads(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ThreadSummary>>> {
    require_permission!(current_user, "messaging.send");
    let threads = MessagingService::list_my_threads(&state.db, current_user.id).await?;
    Ok(Json(threads))
}

/// 列出依 access matrix 可寄送的收件人（供新對話 picker，取代借用 HR 的 /hr/staff）
pub async fn list_recipients(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<RecipientSummary>>> {
    require_permission!(current_user, "messaging.send");
    let recipients = MessagingService::list_recipients(&state.db, current_user.id).await?;
    Ok(Json(recipients))
}

pub async fn create_thread(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateThreadRequest>,
) -> Result<Json<ThreadWithMessages>> {
    require_permission!(current_user, "messaging.send");
    let actor = ActorContext::User(current_user.clone());
    let thread = MessagingService::create_thread(&state.db, &actor, &req).await?;
    Ok(Json(thread))
}

pub async fn get_thread(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ThreadWithMessages>> {
    require_permission!(current_user, "messaging.send");
    let actor = ActorContext::User(current_user.clone());
    let thread = MessagingService::get_thread(&state.db, &actor, id).await?;
    Ok(Json(thread))
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(thread_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<MessageWithSender>> {
    require_permission!(current_user, "messaging.send");
    let actor = ActorContext::User(current_user.clone());
    let msg = MessagingService::send_message(&state.db, &actor, thread_id, &req).await?;
    Ok(Json(msg))
}

pub async fn mark_thread_read(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "messaging.send");
    MessagingService::mark_thread_read(&state.db, current_user.id, thread_id).await?;
    Ok(Json(()))
}

pub async fn delete_message(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<()>> {
    require_permission!(current_user, "messaging.send");
    let actor = ActorContext::User(current_user.clone());
    MessagingService::soft_delete_message(&state.db, &actor, message_id).await?;
    Ok(Json(()))
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}

pub async fn unread_count(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<UnreadCountResponse>> {
    require_permission!(current_user, "messaging.send");
    let count = MessagingService::unread_count(&state.db, current_user.id).await?;
    Ok(Json(UnreadCountResponse { count }))
}

pub async fn upload_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> Result<Json<MessageAttachment>> {
    require_permission!(current_user, "messaging.send");

    let mut file_bytes: Option<(String, String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("Failed to read multipart field: {e}")))?
    {
        if field.name() == Some("file") {
            let file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "image.jpg".to_string());
            let content_type = field
                .content_type()
                .map(String::from)
                .unwrap_or_else(|| "application/octet-stream".to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("Failed to read file: {e}")))?;
            file_bytes = Some((file_name, content_type, data.to_vec()));
        }
    }

    let (file_name, content_type, data) =
        file_bytes.ok_or_else(|| AppError::Validation("缺少 file 欄位".into()))?;

    let attachment = MessagingService::upload_attachment(
        &state.db,
        current_user.id,
        &file_name,
        &content_type,
        &data,
    )
    .await?;
    Ok(Json(attachment))
}

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    #[serde(default)]
    pub inline: Option<String>,
}

pub async fn download_attachment(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(attachment_id): Path<Uuid>,
    axum::extract::Query(q): axum::extract::Query<DownloadQuery>,
) -> Result<Response> {
    require_permission!(current_user, "messaging.send");

    // Access：(a) message 已綁定 → thread participant 或 admin (b) orphan → uploader 自己或 admin
    let row: Option<(String, String, String, Option<Uuid>, Uuid)> = sqlx::query_as(
        r#"SELECT a.file_path, a.file_name, a.mime_type, a.message_id, a.uploaded_by
           FROM message_attachments a
           WHERE a.id = $1"#,
    )
    .bind(attachment_id)
    .fetch_optional(&state.db)
    .await?;
    let (file_path, file_name, mime_type, message_id, uploaded_by) =
        row.ok_or_else(|| AppError::NotFound("找不到附件".into()))?;

    let is_admin = current_user.has_permission("messaging.admin_view");
    if let Some(mid) = message_id {
        if !is_admin {
            let in_thread: bool = sqlx::query_scalar(
                r#"SELECT EXISTS(
                      SELECT 1 FROM messages m
                      JOIN message_thread_participants p
                        ON p.thread_id = m.thread_id AND p.user_id = $1 AND p.left_at IS NULL
                      WHERE m.id = $2 AND m.deleted_at IS NULL
                   )"#,
            )
            .bind(current_user.id)
            .bind(mid)
            .fetch_one(&state.db)
            .await?;
            if !in_thread {
                return Err(AppError::Forbidden("無權存取此附件".into()));
            }
        }
    } else if uploaded_by != current_user.id && !is_admin {
        // orphan 只允許 uploader 自己讀（剛上傳時的預覽）
        return Err(AppError::Forbidden("無權存取此附件".into()));
    }

    let (data, _) = FileService::read(&file_path).await?;
    let inline = matches!(q.inline.as_deref(), Some("1"));
    let disposition = if inline {
        crate::utils::http::content_disposition_inline_header(&file_name)
    } else {
        crate::utils::http::content_disposition_header(&file_name)
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Body::from(data))
        .map_err(|e| AppError::Internal(format!("Failed to build response: {e}")))
}
