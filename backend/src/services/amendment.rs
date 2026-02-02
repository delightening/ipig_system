use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    models::{
        Amendment, AmendmentListItem, AmendmentQuery, AmendmentReviewAssignment, 
        AmendmentReviewAssignmentResponse, AmendmentStatus, AmendmentStatusHistory, 
        AmendmentType, AmendmentVersion, ChangeAmendmentStatusRequest, 
        ClassifyAmendmentRequest, CreateAmendmentRequest, RecordAmendmentDecisionRequest, 
        UpdateAmendmentRequest,
    },
    AppError, Result,
};

pub struct AmendmentService;

impl AmendmentService {
    /// 產生變更編號
    /// 格式：{IACUC_NO}-R{序號:02}
    /// 例如：PIG-114001-R01, PIG-114001-R02
    pub async fn generate_amendment_no(pool: &PgPool, protocol_id: Uuid) -> Result<(String, i32)> {
        // 取得原計畫的 IACUC NO
        let protocol = sqlx::query!(
            r#"SELECT iacuc_no FROM protocols WHERE id = $1"#,
            protocol_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".into()))?;

        let iacuc_no = protocol.iacuc_no
            .ok_or_else(|| AppError::BadRequest("Protocol has no IACUC number".into()))?;

        // 取得目前最大的 revision_number
        let max_revision = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(revision_number), 0) as "max!" FROM amendments WHERE protocol_id = $1"#,
            protocol_id
        )
        .fetch_one(pool)
        .await?;

        let new_revision = max_revision + 1;
        let amendment_no = format!("{}-R{:02}", iacuc_no, new_revision);

        Ok((amendment_no, new_revision))
    }

    /// 建立變更申請
    pub async fn create(
        pool: &PgPool,
        req: &CreateAmendmentRequest,
        created_by: Uuid,
    ) -> Result<Amendment> {
        req.validate()?;

        // 檢查計畫是否存在且為已核准狀態
        let protocol = sqlx::query!(
            r#"SELECT status::text as "status!" FROM protocols WHERE id = $1"#,
            req.protocol_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".into()))?;

        // 只有已核准的計畫才能提交變更申請
        if !["APPROVED", "APPROVED_WITH_CONDITIONS"].contains(&protocol.status.as_str()) {
            return Err(AppError::BadRequest(
                "Only approved protocols can have amendments".into(),
            ));
        }

        let (amendment_no, revision_number) = 
            Self::generate_amendment_no(pool, req.protocol_id).await?;

        let id = Uuid::new_v4();

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            INSERT INTO amendments (
                id, protocol_id, amendment_no, revision_number,
                amendment_type, status, title, description, 
                change_items, changes_content, created_by
            )
            VALUES ($1, $2, $3, $4, 'PENDING', 'DRAFT', $5, $6, $7, $8, $9)
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
            req.protocol_id,
            amendment_no,
            revision_number,
            req.title,
            req.description,
            req.change_items.as_deref(),
            req.changes_content,
            created_by
        )
        .fetch_one(pool)
        .await?;

        // 記錄狀態歷程
        Self::record_status_change(
            pool,
            id,
            None,
            AmendmentStatus::Draft,
            created_by,
            Some("變更申請草稿建立".to_string()),
        )
        .await?;

        Ok(amendment)
    }

    /// 更新變更申請
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateAmendmentRequest,
    ) -> Result<Amendment> {
        req.validate()?;

        // 檢查是否為草稿狀態
        let current = Self::get_by_id_raw(pool, id).await?;
        if current.status != AmendmentStatus::Draft 
            && current.status != AmendmentStatus::RevisionRequired {
            return Err(AppError::BadRequest(
                "Only draft or revision required amendments can be updated".into(),
            ));
        }

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET 
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                change_items = COALESCE($4, change_items),
                changes_content = COALESCE($5, changes_content),
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
            req.title,
            req.description,
            req.change_items.as_deref(),
            req.changes_content,
        )
        .fetch_one(pool)
        .await?;

        Ok(amendment)
    }

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
            let _ = sqlx::query!(
                r#"UPDATE amendments SET status = 'REVISION_REQUIRED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(pool)
            .await?;

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
            let _ = sqlx::query!(
                r#"UPDATE amendments SET status = 'REJECTED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(pool)
            .await?;

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
            let _ = sqlx::query!(
                r#"UPDATE amendments SET status = 'APPROVED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(pool)
            .await?;

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

    /// 記錄狀態變更
    async fn record_status_change(
        pool: &PgPool,
        amendment_id: Uuid,
        from_status: Option<AmendmentStatus>,
        to_status: AmendmentStatus,
        changed_by: Uuid,
        remark: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO amendment_status_history 
                (id, amendment_id, from_status, to_status, changed_by, remark)
            VALUES ($1, $2, ($3::TEXT)::amendment_status, ($4::TEXT)::amendment_status, $5, $6)
            "#,
            Uuid::new_v4(),
            amendment_id,
            from_status.map(|s| s.as_str()),
            to_status.as_str(),
            changed_by,
            remark,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 建立版本快照
    async fn create_version_snapshot(
        pool: &PgPool,
        amendment_id: Uuid,
        submitted_by: Uuid,
    ) -> Result<AmendmentVersion> {
        // 取得目前最大版本號
        let max_version = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(version_no), 0) as "max!" FROM amendment_versions WHERE amendment_id = $1"#,
            amendment_id
        )
        .fetch_one(pool)
        .await?;

        let new_version = max_version + 1;

        // 取得目前變更申請內容作為快照
        let current = Self::get_by_id_raw(pool, amendment_id).await?;

        let snapshot = serde_json::json!({
            "title": current.title,
            "description": current.description,
            "change_items": current.change_items,
            "changes_content": current.changes_content,
        });

        let version = sqlx::query_as!(
            AmendmentVersion,
            r#"
            INSERT INTO amendment_versions (id, amendment_id, version_no, content_snapshot, submitted_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, amendment_id, version_no, content_snapshot, submitted_at, submitted_by
            "#,
            Uuid::new_v4(),
            amendment_id,
            new_version,
            snapshot,
            submitted_by,
        )
        .fetch_one(pool)
        .await?;

        Ok(version)
    }

    /// 取得單一變更申請（原始）
    async fn get_by_id_raw(pool: &PgPool, id: Uuid) -> Result<Amendment> {
        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            SELECT 
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at
            FROM amendments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Amendment not found".into()))?;

        Ok(amendment)
    }

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
