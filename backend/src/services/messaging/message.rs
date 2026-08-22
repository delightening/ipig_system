use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, FileService,
    },
    AppError, Result,
};

use super::{attachment::MessageAttachment, thread::SendMessageRequest};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl crate::models::audit_diff::AuditRedact for Message {}

#[derive(Debug, Serialize)]
pub struct MessageWithSender {
    #[serde(flatten)]
    pub message: Message,
    pub sender_display_name: String,
    pub attachments: Vec<MessageAttachment>,
}

#[derive(FromRow)]
struct MessageRow {
    id: Uuid,
    thread_id: Uuid,
    sender_id: Uuid,
    body: String,
    created_at: DateTime<Utc>,
    edited_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    sender_display_name: String,
}

impl MessageRow {
    fn into_with_sender(
        self,
        attachments: Vec<super::attachment::MessageAttachment>,
    ) -> MessageWithSender {
        MessageWithSender {
            message: Message {
                id: self.id,
                thread_id: self.thread_id,
                sender_id: self.sender_id,
                body: self.body,
                created_at: self.created_at,
                edited_at: self.edited_at,
                deleted_at: self.deleted_at,
            },
            sender_display_name: self.sender_display_name,
            attachments,
        }
    }
}

pub(super) async fn list_for_thread(
    pool: &PgPool,
    thread_id: Uuid,
) -> Result<Vec<MessageWithSender>> {
    let rows = sqlx::query_as::<_, MessageRow>(
        r#"SELECT m.id, m.thread_id, m.sender_id, m.body, m.created_at, m.edited_at, m.deleted_at,
                  COALESCE(u.display_name, u.email) AS sender_display_name
           FROM messages m
           JOIN users u ON u.id = m.sender_id
           WHERE m.thread_id = $1 AND m.deleted_at IS NULL
           ORDER BY m.created_at"#,
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await?;

    // 單次 SELECT 拉所有 message 的 attachments，避免 N+1
    let message_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let mut atts_by_msg = super::attachment::list_for_message_ids(pool, &message_ids).await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let attachments = atts_by_msg.remove(&row.id).unwrap_or_default();
        out.push(row.into_with_sender(attachments));
    }
    Ok(out)
}

pub(super) async fn send(
    pool: &PgPool,
    actor: &ActorContext,
    thread_id: Uuid,
    req: &SendMessageRequest,
) -> Result<MessageWithSender> {
    let sender_id = actor.require_user()?.id;

    if req.body.trim().is_empty() && req.attachment_ids.is_empty() {
        return Err(AppError::Validation("訊息內容與附件不可同時為空".into()));
    }

    // 必須是該 thread 的 active participant
    let is_participant: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
              SELECT 1 FROM message_thread_participants
              WHERE thread_id = $1 AND user_id = $2 AND left_at IS NULL
           )"#,
    )
    .bind(thread_id)
    .bind(sender_id)
    .fetch_one(pool)
    .await?;
    if !is_participant {
        return Err(AppError::Forbidden("您不在此對話中，無法發訊息".into()));
    }

    let thread_alive: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM message_threads WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(thread_id)
    .fetch_one(pool)
    .await?;
    if !thread_alive {
        return Err(AppError::NotFound("對話不存在或已刪除".into()));
    }

    let mut tx = pool.begin().await?;

    let msg_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO messages (thread_id, sender_id, body)
           VALUES ($1, $2, $3) RETURNING id"#,
    )
    .bind(thread_id)
    .bind(sender_id)
    .bind(req.body.trim())
    .fetch_one(&mut *tx)
    .await?;

    if !req.attachment_ids.is_empty() {
        let updated = sqlx::query(
            r#"UPDATE message_attachments SET message_id = $1
               WHERE id = ANY($2) AND message_id IS NULL AND uploaded_by = $3"#,
        )
        .bind(msg_id)
        .bind(&req.attachment_ids)
        .bind(sender_id)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != req.attachment_ids.len() as u64 {
            return Err(AppError::Validation(
                "部分附件 ID 無效、已被使用、或非您所上傳".into(),
            ));
        }
    }

    // Sender auto-mark read
    sqlx::query(
        r#"UPDATE message_thread_participants SET last_read_at = NOW()
           WHERE thread_id = $1 AND user_id = $2"#,
    )
    .bind(thread_id)
    .bind(sender_id)
    .execute(&mut *tx)
    .await?;

    AuditService::log_activity_tx(
        &mut tx,
        actor,
        ActivityLogEntry {
            event_category: "MESSAGING",
            event_type: "MESSAGE_SENT",
            entity: Some(AuditEntity::new("messages", msg_id, "訊息")),
            data_diff: None,
            request_context: None,
        },
    )
    .await?;

    tx.commit().await?;

    let row = sqlx::query_as::<_, MessageRow>(
        r#"SELECT m.id, m.thread_id, m.sender_id, m.body, m.created_at, m.edited_at, m.deleted_at,
                  COALESCE(u.display_name, u.email) AS sender_display_name
           FROM messages m JOIN users u ON u.id = m.sender_id
           WHERE m.id = $1"#,
    )
    .bind(msg_id)
    .fetch_one(pool)
    .await?;

    let attachments = super::attachment::list_for_message(pool, msg_id).await?;
    Ok(row.into_with_sender(attachments))
}

pub(super) async fn soft_delete(
    pool: &PgPool,
    actor: &ActorContext,
    message_id: Uuid,
) -> Result<()> {
    let user_id = actor.require_user()?.id;
    let owner_id: Option<Uuid> =
        sqlx::query_scalar("SELECT sender_id FROM messages WHERE id = $1 AND deleted_at IS NULL")
            .bind(message_id)
            .fetch_optional(pool)
            .await?;
    let owner = owner_id.ok_or_else(|| AppError::NotFound("訊息不存在".into()))?;
    let is_admin = actor
        .as_user()
        .map(|u| u.has_permission("messaging.admin_view"))
        .unwrap_or(false);
    if owner != user_id && !is_admin {
        return Err(AppError::Forbidden("只能刪除自己發的訊息".into()));
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE messages SET deleted_at = NOW() WHERE id = $1")
        .bind(message_id)
        .execute(&mut *tx)
        .await?;
    AuditService::log_activity_tx(
        &mut tx,
        actor,
        ActivityLogEntry {
            event_category: "MESSAGING",
            event_type: "MESSAGE_DELETED",
            entity: Some(AuditEntity::new("messages", message_id, "訊息")),
            data_diff: None,
            request_context: None,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// scheduler GC：30 天前軟刪訊息 hard delete + unlink 附件實體檔案
pub(super) async fn cleanup_soft_deleted(pool: &PgPool) -> Result<(u64, usize)> {
    const BATCH_SIZE: i64 = 100;
    let mut total_deleted: u64 = 0;
    let mut unlink_failures: usize = 0;

    loop {
        let mut tx = pool.begin().await?;
        let stale_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"SELECT id FROM messages
               WHERE deleted_at IS NOT NULL
                 AND deleted_at < NOW() - INTERVAL '30 days'
               ORDER BY deleted_at
               LIMIT $1
               FOR UPDATE SKIP LOCKED"#,
        )
        .bind(BATCH_SIZE)
        .fetch_all(&mut *tx)
        .await?;

        if stale_ids.is_empty() {
            tx.commit().await?;
            break;
        }

        let attachment_paths: Vec<String> = sqlx::query_scalar(
            "SELECT file_path FROM message_attachments WHERE message_id = ANY($1)",
        )
        .bind(&stale_ids)
        .fetch_all(&mut *tx)
        .await?;

        let result = sqlx::query(
            r#"DELETE FROM messages
               WHERE id = ANY($1)
                 AND deleted_at IS NOT NULL
                 AND deleted_at < NOW() - INTERVAL '30 days'"#,
        )
        .bind(&stale_ids)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        let affected = result.rows_affected();
        total_deleted += affected;

        for path in &attachment_paths {
            if let Err(e) = FileService::delete(path).await {
                tracing::warn!("messaging GC unlink failed path={path} err={e}");
                unlink_failures += 1;
            }
        }

        // 終止條件：(a) 拿不到滿一批 → 已清完 (b) DELETE rows_affected = 0 →
        // 同 tx 內理論上不會發生，但為極端 race / SKIP LOCKED 重撈同列場景做防護，避免無限迴圈
        if (stale_ids.len() as i64) < BATCH_SIZE || affected == 0 {
            break;
        }
    }

    Ok((total_deleted, unlink_failures))
}
