//! R40-A 站內信系統 service 層
//!
//! 設計：docs/plans/messaging.md
//! Schema：migration 060_messaging.sql
//!
//! 模組：
//! - access：role category + access matrix（誰能寫給誰）
//! - thread：對話 thread CRUD
//! - message：訊息 CRUD
//! - attachment：圖片附件上傳/列出/刪除（呼叫 FileService）

mod access;
mod attachment;
mod message;
mod thread;

pub use access::{MessagingCategory, RecipientSummary};
pub use attachment::MessageAttachment;
pub use message::{Message, MessageWithSender};
pub use thread::{
    CreateThreadRequest, MessageThread, SendMessageRequest, ThreadParticipantSummary,
    ThreadSummary, ThreadWithMessages,
};

use sqlx::PgPool;

pub struct MessagingService;

impl MessagingService {
    // 公開 API 透過 sub-module 的 impl 實作；本入口保留作 namespace。
    // 實際呼叫例：`MessagingService::create_thread(...)` → `thread::create(...)`
    pub async fn create_thread(
        pool: &PgPool,
        actor: &crate::middleware::ActorContext,
        req: &CreateThreadRequest,
    ) -> crate::Result<ThreadWithMessages> {
        thread::create(pool, actor, req).await
    }

    pub async fn list_my_threads(
        pool: &PgPool,
        user_id: uuid::Uuid,
    ) -> crate::Result<Vec<ThreadSummary>> {
        thread::list_for_user(pool, user_id).await
    }

    pub async fn get_thread(
        pool: &PgPool,
        actor: &crate::middleware::ActorContext,
        thread_id: uuid::Uuid,
    ) -> crate::Result<ThreadWithMessages> {
        thread::get(pool, actor, thread_id).await
    }

    pub async fn send_message(
        pool: &PgPool,
        actor: &crate::middleware::ActorContext,
        thread_id: uuid::Uuid,
        req: &SendMessageRequest,
    ) -> crate::Result<MessageWithSender> {
        message::send(pool, actor, thread_id, req).await
    }

    pub async fn mark_thread_read(
        pool: &PgPool,
        user_id: uuid::Uuid,
        thread_id: uuid::Uuid,
    ) -> crate::Result<()> {
        thread::mark_read(pool, user_id, thread_id).await
    }

    pub async fn soft_delete_message(
        pool: &PgPool,
        actor: &crate::middleware::ActorContext,
        message_id: uuid::Uuid,
    ) -> crate::Result<()> {
        message::soft_delete(pool, actor, message_id).await
    }

    pub async fn upload_attachment(
        pool: &PgPool,
        uploader_id: uuid::Uuid,
        file_name: &str,
        content_type: &str,
        data: &[u8],
    ) -> crate::Result<MessageAttachment> {
        attachment::upload_orphan(pool, uploader_id, file_name, content_type, data).await
    }

    pub async fn unread_count(pool: &PgPool, user_id: uuid::Uuid) -> crate::Result<i64> {
        thread::unread_thread_count(pool, user_id).await
    }

    /// 依 access matrix 列出 sender 可寄送的收件人候選（供新對話 picker）
    pub async fn list_recipients(
        pool: &PgPool,
        sender_id: uuid::Uuid,
    ) -> crate::Result<Vec<RecipientSummary>> {
        access::list_allowed_recipients(pool, sender_id).await
    }

    /// scheduler 用：30 天前軟刪訊息 hard delete + unlink attachments
    pub async fn cleanup_soft_deleted_messages(pool: &PgPool) -> crate::Result<(u64, usize)> {
        message::cleanup_soft_deleted(pool).await
    }
}
