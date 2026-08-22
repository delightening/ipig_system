use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    services::{FileCategory, FileService},
    AppError, Result,
};

/// 站內信附件
///
/// 上傳流程：先 POST `/messages/attachments` 拿 attachment_id（message_id 暫為 NULL，orphan）
/// → 在 create_thread / send_message 時把 attachment_ids 帶上 → service 端 UPDATE 補 message_id。
/// orphan 超過 24 小時由 scheduler 清掉（暫先不寫，初版可手動掃）。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub message_id: Option<Uuid>,
    /// 上傳者；用於：(a) 補綁時驗 sender_id == uploaded_by 防跨使用者挪用 (b) 孤兒附件 GC
    #[serde(skip_serializing)]
    pub uploaded_by: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

pub(super) async fn upload_orphan(
    pool: &PgPool,
    uploader_id: Uuid,
    file_name: &str,
    content_type: &str,
    data: &[u8],
) -> Result<MessageAttachment> {
    let category = FileCategory::MessageAttachment;
    // MIME 大小寫 normalize；client 送 "Image/JPEG" 也接受
    let content_type_normalized = content_type.to_ascii_lowercase();
    if !category
        .allowed_mime_types()
        .contains(&content_type_normalized.as_str())
    {
        return Err(AppError::Validation(format!(
            "不允許的檔案類型：{content_type}"
        )));
    }
    // 大小 check 由 caller 在讀完 multipart bytes 之前先做（handler 端有 MAX_UPLOAD_SIZE_BYTES 限制）；
    // 此處再 assert 一次作 defense-in-depth
    if data.len() > category.max_file_size() {
        return Err(AppError::Validation(format!(
            "檔案超過上限 {} MB",
            category.max_file_size() / 1024 / 1024
        )));
    }

    let upload = FileService::upload(
        category,
        file_name,
        &content_type_normalized,
        data,
        Some(&uploader_id.to_string()),
    )
    .await?;

    let result = sqlx::query_as::<_, MessageAttachment>(
        r#"INSERT INTO message_attachments
               (message_id, uploaded_by, file_name, file_path, file_size, mime_type, sort_order)
           VALUES (NULL, $1, $2, $3, $4, $5, 0)
           RETURNING id, message_id, uploaded_by, file_name, file_path, file_size, mime_type, sort_order, created_at"#,
    )
    .bind(uploader_id)
    .bind(&upload.file_name)
    .bind(&upload.file_path)
    .bind(upload.file_size)
    .bind(&upload.mime_type)
    .fetch_one(pool)
    .await;

    match result {
        Ok(att) => Ok(att),
        Err(e) => {
            if let Err(unlink_err) = FileService::delete(&upload.file_path).await {
                tracing::warn!(
                    "messaging attachment rollback unlink failed path={} err={unlink_err}",
                    upload.file_path
                );
            }
            Err(e.into())
        }
    }
}

pub(super) async fn list_for_message(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<Vec<MessageAttachment>> {
    let rows = sqlx::query_as::<_, MessageAttachment>(
        r#"SELECT id, message_id, uploaded_by, file_name, file_path, file_size, mime_type, sort_order, created_at
           FROM message_attachments
           WHERE message_id = $1
           ORDER BY sort_order, created_at"#,
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 一次撈整個 thread 的 attachments（給 list_for_thread 用，避免 N+1）
pub(super) async fn list_for_message_ids(
    pool: &PgPool,
    message_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<MessageAttachment>>> {
    if message_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = sqlx::query_as::<_, MessageAttachment>(
        r#"SELECT id, message_id, uploaded_by, file_name, file_path, file_size, mime_type, sort_order, created_at
           FROM message_attachments
           WHERE message_id = ANY($1)
           ORDER BY message_id, sort_order, created_at"#,
    )
    .bind(message_ids)
    .fetch_all(pool)
    .await?;

    let mut map: std::collections::HashMap<Uuid, Vec<MessageAttachment>> =
        std::collections::HashMap::new();
    for att in rows {
        if let Some(mid) = att.message_id {
            map.entry(mid).or_default().push(att);
        }
    }
    Ok(map)
}
