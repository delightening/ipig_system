use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        Amendment, AmendmentListItem, AmendmentQuery, AmendmentReviewAssignmentResponse,
        AmendmentStatus, AmendmentStatusHistory, AmendmentType, AmendmentVersion,
    },
    Result,
};

use super::AmendmentService;

impl AmendmentService {
    /// 取得單一變更申請（含關聯資訊）
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Amendment> {
        Self::get_by_id_raw(pool, id).await
    }

    /// 列出變更申請
    pub async fn list(pool: &PgPool, query: &AmendmentQuery) -> Result<Vec<AmendmentListItem>> {
        let amendments = sqlx::query_as!(
            AmendmentListItem,
            r#"
            SELECT 
                a.id, a.protocol_id, a.amendment_no, a.revision_number,
                a.amendment_type as "amendment_type: AmendmentType",
                a.status as "status: AmendmentStatus",
                a.title, a.description, a.change_items,
                a.submitted_at, a.classified_at,
                a.created_at, a.updated_at,
                p.iacuc_no as protocol_iacuc_no,
                p.title as protocol_title,
                u.display_name as submitted_by_name,
                c.display_name as classified_by_name
            FROM amendments a
            JOIN protocols p ON a.protocol_id = p.id
            LEFT JOIN users u ON a.submitted_by = u.id
            LEFT JOIN users c ON a.classified_by = c.id
            WHERE 
                ($1::uuid IS NULL OR a.protocol_id = $1)
                AND ($2::text IS NULL OR a.status::text = $2)
                AND ($3::text IS NULL OR a.amendment_type::text = $3)
            ORDER BY a.created_at DESC
            "#,
            query.protocol_id,
            query.status.map(|s| s.as_str().to_string()),
            query.amendment_type.map(|t| t.as_str().to_string()),
        )
        .fetch_all(pool)
        .await?;

        Ok(amendments)
    }

    /// 列出使用者可見的變更申請（SQL 層過濾，避免取全部再客端 filter）
    pub async fn list_for_user(
        pool: &PgPool,
        query: &AmendmentQuery,
        user_id: Uuid,
    ) -> Result<Vec<AmendmentListItem>> {
        let amendments = sqlx::query_as::<_, AmendmentListItem>(
            r#"
            SELECT
                a.id, a.protocol_id, a.amendment_no, a.revision_number,
                a.amendment_type as "amendment_type: AmendmentType",
                a.status as "status: AmendmentStatus",
                a.title, a.description, a.change_items,
                a.submitted_at, a.classified_at,
                a.created_at, a.updated_at,
                p.iacuc_no as protocol_iacuc_no,
                p.title as protocol_title,
                u.display_name as submitted_by_name,
                c.display_name as classified_by_name
            FROM amendments a
            JOIN protocols p ON a.protocol_id = p.id
            LEFT JOIN users u ON a.submitted_by = u.id
            LEFT JOIN users c ON a.classified_by = c.id
            WHERE
                a.protocol_id IN (SELECT protocol_id FROM user_protocols WHERE user_id = $1)
                AND ($2::uuid IS NULL OR a.protocol_id = $2)
                AND ($3::text IS NULL OR a.status::text = $3)
                AND ($4::text IS NULL OR a.amendment_type::text = $4)
            ORDER BY a.created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(query.protocol_id)
        .bind(query.status.map(|s| s.as_str().to_string()))
        .bind(query.amendment_type.map(|t| t.as_str().to_string()))
        .fetch_all(pool)
        .await?;

        Ok(amendments)
    }

    /// 列出計畫的所有變更申請
    pub async fn list_by_protocol(pool: &PgPool, protocol_id: Uuid) -> Result<Vec<AmendmentListItem>> {
        Self::list(pool, &AmendmentQuery {
            protocol_id: Some(protocol_id),
            status: None,
            amendment_type: None,
        }).await
    }

    /// 取得版本列表
    pub async fn get_versions(pool: &PgPool, amendment_id: Uuid) -> Result<Vec<AmendmentVersion>> {
        let versions = sqlx::query_as!(
            AmendmentVersion,
            r#"
            SELECT id, amendment_id, version_no, content_snapshot, submitted_at, submitted_by
            FROM amendment_versions
            WHERE amendment_id = $1
            ORDER BY version_no DESC
            "#,
            amendment_id
        )
        .fetch_all(pool)
        .await?;

        Ok(versions)
    }

    /// 取得狀態歷程
    pub async fn get_status_history(pool: &PgPool, amendment_id: Uuid) -> Result<Vec<AmendmentStatusHistory>> {
        let history = sqlx::query_as!(
            AmendmentStatusHistory,
            r#"
            SELECT 
                id, amendment_id,
                from_status as "from_status: AmendmentStatus",
                to_status as "to_status: AmendmentStatus",
                changed_by, remark, created_at
            FROM amendment_status_history
            WHERE amendment_id = $1
            ORDER BY created_at DESC
            "#,
            amendment_id
        )
        .fetch_all(pool)
        .await?;

        Ok(history)
    }

    /// 取得審查委員指派列表
    pub async fn get_review_assignments(
        pool: &PgPool,
        amendment_id: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignmentResponse>> {
        let assignments = sqlx::query_as!(
            AmendmentReviewAssignmentResponse,
            r#"
            SELECT 
                ara.id, ara.amendment_id, ara.reviewer_id, ara.assigned_by, ara.assigned_at,
                ara.decision, ara.decided_at, ara.comment,
                u.display_name as reviewer_name,
                u.email as reviewer_email
            FROM amendment_review_assignments ara
            JOIN users u ON ara.reviewer_id = u.id
            WHERE ara.amendment_id = $1
            ORDER BY ara.assigned_at
            "#,
            amendment_id
        )
        .fetch_all(pool)
        .await?;

        Ok(assignments)
    }

    /// 取得待處理變更申請數量 (包含待分類 SUBMITTED/RESUBMITTED 和待審查 CLASSIFIED/UNDER_REVIEW)
    pub async fn get_pending_count(pool: &PgPool) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM amendments 
            WHERE status IN ('SUBMITTED', 'RESUBMITTED', 'CLASSIFIED', 'UNDER_REVIEW')
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}
