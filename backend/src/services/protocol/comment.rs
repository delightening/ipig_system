use sqlx::PgPool;
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    models::{
        CreateCommentRequest, ProtocolActivityType, ReplyCommentRequest,
        ReviewComment, ReviewCommentResponse,
    },
    AppError, Result,
};

impl ProtocolService {
    /// 新增審查意見
    pub async fn add_comment(
        pool: &PgPool,
        req: &CreateCommentRequest,
        reviewer_id: Uuid,
    ) -> Result<ReviewComment> {
        // 獲取關聯的 protocol_id
        let protocol_id: Uuid = sqlx::query_scalar(
            "SELECT protocol_id FROM protocol_versions WHERE id = $1"
        )
        .bind(req.protocol_version_id)
        .fetch_one(pool)
        .await?;

        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            INSERT INTO review_comments (
                id, protocol_version_id, protocol_id, reviewer_id, 
                content, review_stage, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 'UNDER_REVIEW', NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(req.protocol_version_id)
        .bind(protocol_id)
        .bind(reviewer_id)
        .bind(&req.content)
        .fetch_one(pool)
        .await?;

        Self::record_activity(
            pool,
            protocol_id,
            ProtocolActivityType::CommentAdded,
            reviewer_id,
            None,
            None,
            Some(("comment", comment.id, &comment.content)),
            None,
            None,
        ).await?;

        Ok(comment)
    }

    /// 取得審查意見（含回覆）
    pub async fn get_comments(pool: &PgPool, protocol_id: Uuid) -> Result<Vec<ReviewCommentResponse>> {
        let comments = sqlx::query_as::<_, ReviewCommentResponse>(
            r#"
            SELECT 
                c.id, c.protocol_version_id, c.protocol_id, c.reviewer_id,
                u.display_name as reviewer_name, u.email as reviewer_email,
                c.content, c.is_resolved, c.resolved_by, c.resolved_at, 
                c.parent_comment_id, c.replied_by,
                ru.display_name as replied_by_name, ru.email as replied_by_email,
                c.draft_content, c.drafted_by, 
                du.display_name as drafted_by_name,
                c.draft_updated_at,
                c.created_at
            FROM review_comments c
            LEFT JOIN users u ON c.reviewer_id = u.id
            LEFT JOIN users ru ON c.replied_by = ru.id
            LEFT JOIN users du ON c.drafted_by = du.id
            WHERE c.protocol_id = $1 
               OR c.protocol_version_id IN (SELECT id FROM protocol_versions WHERE protocol_id = $1)
            ORDER BY 
                COALESCE(c.parent_comment_id, c.id) ASC,
                c.created_at ASC
            "#
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;

        Ok(comments)
    }

    /// 回覆審查意見
    pub async fn reply_comment(
        pool: &PgPool,
        req: &ReplyCommentRequest,
        replied_by: Uuid,
    ) -> Result<ReviewComment> {
        // 驗證父評論存在並獲取 protocol_version_id
        let parent_comment: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT protocol_version_id 
            FROM review_comments 
            WHERE id = $1 AND parent_comment_id IS NULL
            "#
        )
        .bind(req.parent_comment_id)
        .fetch_optional(pool)
        .await?;

        let (protocol_version_id,) = parent_comment
            .ok_or_else(|| AppError::NotFound("Parent comment not found".to_string()))?;

        // 嘗試獲取 protocol_id
        let target_protocol_id: Uuid = sqlx::query_scalar(
            "SELECT protocol_id FROM review_comments WHERE id = $1"
        )
        .bind(req.parent_comment_id)
        .fetch_one(pool)
        .await?;

        // 插入回覆
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            INSERT INTO review_comments (
                id, protocol_version_id, protocol_id, reviewer_id, content, 
                parent_comment_id, replied_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_version_id)
        .bind(target_protocol_id)
        .bind(replied_by)
        .bind(&req.content)
        .bind(req.parent_comment_id)
        .bind(replied_by)
        .fetch_one(pool)
        .await?;

        // 記錄活動日誌
        Self::record_activity(
            pool,
            target_protocol_id,
            ProtocolActivityType::CommentReplied,
            replied_by,
            None,
            None,
            Some(("comment", req.parent_comment_id, "Comment Reply")),
            Some({
                let reply_text: String = if req.content.chars().count() > 50 {
                    format!("{}...", req.content.chars().take(47).collect::<String>())
                } else {
                    req.content.clone()
                };
                format!("Reply: {}", reply_text)
            }),
            None,
        ).await?;

        Ok(comment)
    }

    /// 解決審查意見
    pub async fn resolve_comment(pool: &PgPool, comment_id: Uuid, resolved_by: Uuid) -> Result<ReviewComment> {
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            UPDATE review_comments SET
                is_resolved = true,
                resolved_by = $2,
                resolved_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(comment_id)
        .bind(resolved_by)
        .fetch_one(pool)
        .await?;

        Ok(comment)
    }

    /// 儲存草稿回覆
    /// Coeditor 或 PI 可以先儲存草稿，稍後由 PI 正式送出
    pub async fn save_reply_draft(
        pool: &PgPool,
        comment_id: Uuid,
        draft_content: &str,
        drafted_by: Uuid,
    ) -> Result<ReviewComment> {
        // 驗證目標評論存在
        let comment_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM review_comments WHERE id = $1)"
        )
        .bind(comment_id)
        .fetch_one(pool)
        .await?;

        if !comment_exists {
            return Err(AppError::NotFound("Comment not found".to_string()));
        }

        // 更新草稿內容
        let comment = sqlx::query_as::<_, ReviewComment>(
            r#"
            UPDATE review_comments SET
                draft_content = $2,
                drafted_by = $3,
                draft_updated_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(comment_id)
        .bind(draft_content)
        .bind(drafted_by)
        .fetch_one(pool)
        .await?;

        Ok(comment)
    }

    /// 取得草稿回覆
    pub async fn get_reply_draft(pool: &PgPool, comment_id: Uuid) -> Result<Option<String>> {
        let draft: Option<String> = sqlx::query_scalar(
            "SELECT draft_content FROM review_comments WHERE id = $1"
        )
        .bind(comment_id)
        .fetch_one(pool)
        .await?;

        Ok(draft)
    }

    /// 將草稿正式送出為回覆（只有 PI 可以執行）
    pub async fn submit_reply_from_draft(
        pool: &PgPool,
        comment_id: Uuid,
        submitted_by: Uuid,
    ) -> Result<ReviewComment> {
        // 取得目標評論及其草稿內容
        let target = sqlx::query_as::<_, (Uuid, Option<String>)>(
            r#"
            SELECT protocol_version_id, draft_content 
            FROM review_comments 
            WHERE id = $1 AND parent_comment_id IS NULL
            "#
        )
        .bind(comment_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found or is already a reply".to_string()))?;

        let (protocol_version_id, draft_content) = target;

        let content = draft_content
            .ok_or_else(|| AppError::BadRequest("No draft content to submit".to_string()))?;

        // 取得 protocol_id
        let target_protocol_id: Uuid = sqlx::query_scalar(
            "SELECT protocol_id FROM review_comments WHERE id = $1"
        )
        .bind(comment_id)
        .fetch_one(pool)
        .await?;

        // 建立正式回覆
        let reply = sqlx::query_as::<_, ReviewComment>(
            r#"
            INSERT INTO review_comments (
                id, protocol_version_id, protocol_id, reviewer_id, content, 
                parent_comment_id, replied_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_version_id)
        .bind(target_protocol_id)
        .bind(submitted_by)
        .bind(&content)
        .bind(comment_id)
        .bind(submitted_by)
        .fetch_one(pool)
        .await?;

        // 清除原評論的草稿內容
        sqlx::query(
            r#"
            UPDATE review_comments SET
                draft_content = NULL,
                drafted_by = NULL,
                draft_updated_at = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(comment_id)
        .execute(pool)
        .await?;

        Ok(reply)
    }
}
