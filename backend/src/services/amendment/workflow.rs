use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        Amendment, AmendmentReviewAssignment, AmendmentStatus,
        AmendmentType, ChangeAmendmentStatusRequest,
        ClassifyAmendmentRequest, RecordAmendmentDecisionRequest,
    },
    AppError, Result,
};

use super::AmendmentService;

impl AmendmentService {
    /// 提交變更申請
    pub async fn submit(pool: &PgPool, id: Uuid, submitted_by: Uuid) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;

        // 只有草稿或需修訂狀態可以提交
        let (new_status, is_resubmit) = match current.status {
            AmendmentStatus::Draft => (AmendmentStatus::Submitted, false),
            AmendmentStatus::RevisionRequired => (AmendmentStatus::Resubmitted, true),
            _ => {
                return Err(AppError::BadRequest(
                    "Only draft or revision required amendments can be submitted".into(),
                ));
            }
        };

        // 更新狀態
        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET 
                status = ($2::TEXT)::amendment_status,
                submitted_by = $3,
                submitted_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at
            "#,
            id,
            new_status.as_str(),
            submitted_by,
        )
        .fetch_one(pool)
        .await?;

        // 建立版本快照
        Self::create_version_snapshot(pool, id, submitted_by).await?;

        // 記錄狀態歷程
        let remark = if is_resubmit { "變更申請已重送" } else { "變更申請已提交" };
        Self::record_status_change(
            pool,
            id,
            Some(current.status),
            new_status,
            submitted_by,
            Some(remark.to_string()),
        )
        .await?;

        Ok(amendment)
    }

    /// 分類變更申請（由 IACUC_STAFF 執行）
    pub async fn classify(
        pool: &PgPool,
        id: Uuid,
        req: &ClassifyAmendmentRequest,
        classified_by: Uuid,
    ) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;

        // 只有已提交或已重送的申請可以分類
        if current.status != AmendmentStatus::Submitted 
            && current.status != AmendmentStatus::Resubmitted {
            return Err(AppError::BadRequest(
                "Only submitted amendments can be classified".into(),
            ));
        }

        // 不能分類為待分類
        if req.amendment_type == AmendmentType::Pending {
            return Err(AppError::BadRequest(
                "Cannot classify as PENDING".into(),
            ));
        }

        // 決定下一個狀態
        let new_status = match req.amendment_type {
            AmendmentType::Major => AmendmentStatus::Classified, // 需要進入審查
            AmendmentType::Minor => AmendmentStatus::AdminApproved, // 小變更直接核准
            AmendmentType::Pending => unreachable!(),
        };

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET 
                amendment_type = ($2::TEXT)::amendment_type,
                status = ($3::TEXT)::amendment_status,
                classified_by = $4,
                classified_at = NOW(),
                classification_remark = $5,
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at
            "#,
            id,
            req.amendment_type.as_str(),
            new_status.as_str(),
            classified_by,
            req.remark,
        )
        .fetch_one(pool)
        .await?;

        // 記錄狀態歷程
        Self::record_status_change(
            pool,
            id,
            Some(current.status),
            new_status,
            classified_by,
            req.remark.clone(),
        )
        .await?;

        // 如果是重大變更，複製原計畫的審查委員
        if req.amendment_type == AmendmentType::Major {
            Self::assign_reviewers_from_protocol(pool, id, current.protocol_id, classified_by).await?;
        }

        Ok(amendment)
    }

    /// 從原計畫複製審查委員
    pub async fn assign_reviewers_from_protocol(
        pool: &PgPool,
        amendment_id: Uuid,
        protocol_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignment>> {
        // 取得原計畫的審查委員
        let reviewers = sqlx::query!(
            r#"SELECT reviewer_id FROM review_assignments WHERE protocol_id = $1"#,
            protocol_id
        )
        .fetch_all(pool)
        .await?;

        let mut assignments = Vec::new();

        for reviewer in reviewers {
            let assignment = sqlx::query_as!(
                AmendmentReviewAssignment,
                r#"
                INSERT INTO amendment_review_assignments (amendment_id, reviewer_id, assigned_by)
                VALUES ($1, $2, $3)
                ON CONFLICT (amendment_id, reviewer_id) DO NOTHING
                RETURNING 
                    id, amendment_id, reviewer_id, assigned_by, assigned_at,
                    decision, decided_at, comment
                "#,
                amendment_id,
                reviewer.reviewer_id,
                assigned_by
            )
            .fetch_optional(pool)
            .await?;

            if let Some(a) = assignment {
                assignments.push(a);
            }
        }

        Ok(assignments)
    }

    /// 開始審查（變更狀態為 UNDER_REVIEW）
    pub async fn start_review(pool: &PgPool, id: Uuid, changed_by: Uuid) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;

        if current.status != AmendmentStatus::Classified {
            return Err(AppError::BadRequest(
                "Only classified amendments can start review".into(),
            ));
        }

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET status = 'UNDER_REVIEW', updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Self::record_status_change(
            pool,
            id,
            Some(current.status),
            AmendmentStatus::UnderReview,
            changed_by,
            Some("開始審查".to_string()),
        )
        .await?;

        Ok(amendment)
    }

    /// 記錄審查決定
    pub async fn record_decision(
        pool: &PgPool,
        amendment_id: Uuid,
        reviewer_id: Uuid,
        req: &RecordAmendmentDecisionRequest,
    ) -> Result<AmendmentReviewAssignment> {
        // 驗證決定值
        if !["APPROVE", "REJECT", "REVISION"].contains(&req.decision.as_str()) {
            return Err(AppError::BadRequest("Invalid decision value".into()));
        }

        let assignment = sqlx::query_as!(
            AmendmentReviewAssignment,
            r#"
            UPDATE amendment_review_assignments
            SET 
                decision = $3,
                decided_at = NOW(),
                comment = $4
            WHERE amendment_id = $1 AND reviewer_id = $2
            RETURNING 
                id, amendment_id, reviewer_id, assigned_by, assigned_at,
                decision, decided_at, comment
            "#,
            amendment_id,
            reviewer_id,
            req.decision,
            req.comment,
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Review assignment not found".into()))?;

        // 檢查所有審查委員是否都已完成決定
        Self::check_all_decisions(pool, amendment_id).await?;

        Ok(assignment)
    }

    /// 檢查所有審查委員是否都已完成決定，並自動更新狀態
    async fn check_all_decisions(pool: &PgPool, amendment_id: Uuid) -> Result<()> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as "total!",
                COUNT(decision) as "decided!",
                COUNT(*) FILTER (WHERE decision = 'APPROVE') as "approved!",
                COUNT(*) FILTER (WHERE decision = 'REJECT') as "rejected!",
                COUNT(*) FILTER (WHERE decision = 'REVISION') as "revision!"
            FROM amendment_review_assignments
            WHERE amendment_id = $1
            "#,
            amendment_id
        )
        .fetch_one(pool)
        .await?;

        // 所有人都還沒決定就不處理
        if stats.decided != stats.total {
            return Ok(());
        }

        let current = Self::get_by_id_raw(pool, amendment_id).await?;

        // 有任何人要求修訂，狀態變為需修訂
        if stats.revision > 0 {
            if let Err(e) = sqlx::query!(
                r#"UPDATE amendments SET status = 'REVISION_REQUIRED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(pool)
            .await {
                tracing::warn!("更新修正案狀態失敗: {e}");
            }

            Self::record_status_change(
                pool,
                amendment_id,
                Some(current.status),
                AmendmentStatus::RevisionRequired,
                Uuid::nil(), // system
                Some("審查委員要求修訂".to_string()),
            )
            .await?;
        }
        // 有任何人否決
        else if stats.rejected > 0 {
            if let Err(e) = sqlx::query!(
                r#"UPDATE amendments SET status = 'REJECTED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(pool)
            .await {
                tracing::warn!("更新修正案狀態失敗: {e}");
            }


            Self::record_status_change(
                pool,
                amendment_id,
                Some(current.status),
                AmendmentStatus::Rejected,
                Uuid::nil(),
                Some("審查委員否決".to_string()),
            )
            .await?;
        }
        // 全部核准
        else if stats.approved == stats.total {
            if let Err(e) = sqlx::query!(
                r#"UPDATE amendments SET status = 'APPROVED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(pool)
            .await {
                tracing::warn!("更新修正案狀態失敗: {e}");
            }


            Self::record_status_change(
                pool,
                amendment_id,
                Some(current.status),
                AmendmentStatus::Approved,
                Uuid::nil(),
                Some("全體審查委員核准".to_string()),
            )
            .await?;
        }

        Ok(())
    }

    /// 變更狀態
    pub async fn change_status(
        pool: &PgPool,
        id: Uuid,
        req: &ChangeAmendmentStatusRequest,
        changed_by: Uuid,
    ) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET status = ($2::TEXT)::amendment_status, updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at
            "#,
            id,
            req.to_status.as_str(),
        )
        .fetch_one(pool)
        .await?;

        Self::record_status_change(
            pool,
            id,
            Some(current.status),
            req.to_status,
            changed_by,
            req.remark.clone(),
        )
        .await?;

        Ok(amendment)
    }
}
