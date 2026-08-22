//! 補登歷史變更申請（Amendment Import Backfill）P6 — 平行 protocol import P1。
//!
//! 紙本世界早已核准過的歷史變更（變更第 N 號），無 live 審查與電子簽章。
//! 流程：`create_historical`（建 is_historical 草稿，回填原始日期）→
//! `finalize_historical`（DRAFT → EFFECTIVE，回填生效日）。
//!
//! 守衛：actor 不可 Anonymous；計劃須為「匯入計劃」（imported_at 非 NULL）且已核准；
//! 編號沿用 `generate_amendment_no`（MAX+1），歷史佔前號、後續 live 自動接續。
//! 權限（計劃負責人 SD / admin）由 handler 層 `access::can_backfill_historical_amendment` 守。

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Amendment, AmendmentStatus, AmendmentType,
        CreateHistoricalAmendmentRequest, FinalizeHistoricalAmendmentRequest,
        HistoricalAmendmentReviewer, RecordHistoricalReviewsRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

use super::AmendmentService;

impl AmendmentService {
    /// 補登歷史變更：建立 `is_historical=true` 的 DRAFT，回填原始送件 / 分類日期。
    /// `amendment_type` 須為 MAJOR / MINOR（不可 PENDING）。
    pub async fn create_historical(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateHistoricalAmendmentRequest,
    ) -> Result<Amendment> {
        req.validate()?;
        let actor_id = Self::backfill_actor_id(actor)?;

        // 歷史變更必為已分類（MAJOR=委員審 / MINOR=執秘審），不可 PENDING
        if req.amendment_type == AmendmentType::Pending {
            return Err(AppError::BadRequest(
                "補登歷史變更須指定 MAJOR 或 MINOR 分類".into(),
            ));
        }

        // 計劃須存在、為「匯入計劃」（imported_at 非 NULL）、且已核准
        let protocol = sqlx::query!(
            r#"SELECT status::text as "status!", imported_at FROM protocols WHERE id = $1"#,
            req.protocol_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Protocol not found".into()))?;

        if protocol.imported_at.is_none() {
            return Err(AppError::BusinessRule("僅匯入計劃可補登歷史變更".into()));
        }
        if !["APPROVED", "APPROVED_WITH_CONDITIONS"].contains(&protocol.status.as_str()) {
            return Err(AppError::BusinessRule("僅已核准計劃可補登歷史變更".into()));
        }

        let (amendment_no, revision_number) =
            Self::generate_amendment_no(pool, req.protocol_id).await?;
        let id = Uuid::new_v4();

        let mut tx = pool.begin().await?;

        // created_by / classified_by = 補登者（actor_id）；submitted_by 留 NULL（原始送件者
        // 多為非系統使用者）。回填原始 submitted_at / classified_at。
        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            INSERT INTO amendments (
                id, protocol_id, amendment_no, revision_number,
                amendment_type, status, title, description,
                change_items, changes_content, created_by,
                is_historical, submitted_at,
                classified_by, classified_at, classification_remark
            )
            VALUES (
                $1, $2, $3, $4,
                ($5::TEXT)::amendment_type, 'DRAFT', $6, $7,
                $8, $9, $10,
                true, $11,
                $10, $12, $13
            )
            RETURNING
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
            "#,
            id,
            req.protocol_id,
            amendment_no,
            revision_number,
            req.amendment_type.as_str(),
            req.title,
            req.description,
            req.change_items.as_deref(),
            req.changes_content,
            actor_id,
            req.submitted_at,
            req.classified_at,
            req.classification_remark,
        )
        .fetch_one(&mut *tx)
        .await?;

        Self::record_status_change(
            &mut *tx,
            id,
            None,
            AmendmentStatus::Draft,
            actor_id,
            Some("補登歷史變更草稿建立".to_string()),
        )
        .await?;

        let display = format!("{} — {}", amendment.amendment_no, amendment.title);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "AMENDMENT_IMPORT_BACKFILLED",
                entity: Some(AuditEntity::new("amendment", id, &display)),
                data_diff: Some(DataDiff::create_only(&amendment)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(amendment)
    }

    /// 完成補登歷史變更：DRAFT → EFFECTIVE（專屬路徑，繞泛型 transition guard）。
    /// 回填 effective_from、建版本快照、入 audit。僅限 `is_historical` 且 DRAFT。
    pub async fn finalize_historical(
        pool: &PgPool,
        actor: &ActorContext,
        amendment_id: Uuid,
        req: &FinalizeHistoricalAmendmentRequest,
    ) -> Result<Amendment> {
        req.validate()?;
        let actor_id = Self::backfill_actor_id(actor)?;

        let mut tx = pool.begin().await?;

        let before = sqlx::query_as!(
            Amendment,
            r#"
            SELECT
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
            FROM amendments
            WHERE id = $1
            FOR UPDATE
            "#,
            amendment_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Amendment not found".into()))?;

        // 互斥守衛：live 變更不可走此補登流程
        if !before.is_historical {
            return Err(AppError::BusinessRule(
                "僅補登歷史變更可由此流程完成生效".into(),
            ));
        }
        if before.status != AmendmentStatus::Draft {
            return Err(AppError::BusinessRule(format!(
                "僅 DRAFT 狀態的補登變更可完成生效（目前:{}）",
                before.status.as_str()
            )));
        }

        let effective = req.effective_from.unwrap_or_else(Utc::now);

        // CAS UPDATE：WHERE 帶 status='DRAFT' + is_historical，0 row 表 tx 內被改動。
        let after = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments SET
                status = 'EFFECTIVE'::amendment_status,
                effective_from = $2,
                updated_at = NOW()
            WHERE id = $1
              AND status = 'DRAFT'::amendment_status
              AND is_historical = true
            RETURNING
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
            "#,
            amendment_id,
            effective,
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("補登變更狀態於 tx 內被併發改動，請重試".into()))?;

        Self::record_status_change(
            &mut *tx,
            amendment_id,
            Some(AmendmentStatus::Draft),
            AmendmentStatus::Effective,
            actor_id,
            req.remark.clone(),
        )
        .await?;

        // 版本快照（content_snapshot NOT NULL）。歷史變更首版 version_no=1。
        let version_no = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(version_no), 0) + 1 as "v!" FROM amendment_versions WHERE amendment_id = $1"#,
            amendment_id
        )
        .fetch_one(&mut *tx)
        .await?;
        let snapshot = serde_json::json!({
            "title": after.title,
            "description": after.description,
            "change_items": after.change_items,
            "changes_content": after.changes_content,
        });
        sqlx::query!(
            r#"
            INSERT INTO amendment_versions (id, amendment_id, version_no, content_snapshot, submitted_by)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            Uuid::new_v4(),
            amendment_id,
            version_no,
            snapshot,
            actor_id,
        )
        .execute(&mut *tx)
        .await?;

        let display = format!("{} — {}", after.amendment_no, after.title);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "AMENDMENT_IMPORT_FINALIZED",
                entity: Some(AuditEntity::new("amendment", amendment_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(after)
    }

    /// 補登歷史變更的委員審查文件（P6-3）：全量取代 amendment_review_assignments。
    /// 僅限 is_historical + DRAFT（finalize 後不可再補）。支援院外委員（填姓名）。
    pub async fn record_historical_reviews(
        pool: &PgPool,
        actor: &ActorContext,
        amendment_id: Uuid,
        req: &RecordHistoricalReviewsRequest,
    ) -> Result<()> {
        let actor_id = Self::backfill_actor_id(actor)?;
        validate_historical_reviews(req)?;

        let mut tx = pool.begin().await?;
        let row = sqlx::query!(
            r#"SELECT is_historical, status::text as "status!" FROM amendments WHERE id = $1 FOR UPDATE"#,
            amendment_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Amendment not found".into()))?;

        if !row.is_historical {
            return Err(AppError::BusinessRule(
                "僅補登歷史變更可補登審查文件".into(),
            ));
        }
        if row.status != "DRAFT" {
            return Err(AppError::BusinessRule(
                "僅 DRAFT 階段（未完成補登）可補登審查文件".into(),
            ));
        }

        // 全量取代（補登來源唯一，重送清掉先前再重建）
        sqlx::query!(
            "DELETE FROM amendment_review_assignments WHERE amendment_id = $1",
            amendment_id
        )
        .execute(&mut *tx)
        .await?;

        for r in &req.reviewers {
            let name = historical_reviewer_name(r);
            sqlx::query!(
                r#"
                INSERT INTO amendment_review_assignments
                    (id, amendment_id, reviewer_id, reviewer_name, assigned_by, assigned_at, decision, decided_at, comment)
                VALUES ($1, $2, $3, $4, $5, COALESCE($6, NOW()), $7, $6, $8)
                "#,
                Uuid::new_v4(),
                amendment_id,
                r.reviewer_id,
                name,
                actor_id,
                r.decided_at,
                r.decision.as_deref(),
                r.comment.as_deref(),
            )
            .execute(&mut *tx)
            .await?;
        }

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "AMENDMENT_IMPORT_REVIEWS_RECORDED",
                entity: Some(AuditEntity::new(
                    "amendment",
                    amendment_id,
                    "歷史變更審查補登",
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 補登動作的 actor id（拒絕 Anonymous）。created_by / submitted_by 等 NOT NULL 欄位用。
    fn backfill_actor_id(actor: &ActorContext) -> Result<Uuid> {
        actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("補登歷史變更須由已登入使用者或系統觸發".into()))
    }
}

/// 驗證每位委員：系統帳號 / 姓名至少其一；決定值合法。
fn validate_historical_reviews(req: &RecordHistoricalReviewsRequest) -> Result<()> {
    for r in &req.reviewers {
        if r.reviewer_id.is_none() && reviewer_name_empty(&r.reviewer_name) {
            return Err(AppError::Validation("委員須指定系統帳號或填寫姓名".into()));
        }
        if let Some(d) = r.decision.as_deref() {
            if !["APPROVE", "REJECT", "REVISION"].contains(&d) {
                return Err(AppError::Validation(
                    "審查決定須為 APPROVE / REJECT / REVISION".into(),
                ));
            }
        }
    }
    Ok(())
}

fn reviewer_name_empty(name: &Option<String>) -> bool {
    name.as_deref().map(str::trim).unwrap_or("").is_empty()
}

/// 系統內委員（有 id）不存姓名（顯示走 users.display_name）；院外存 trim 後姓名。
fn historical_reviewer_name(r: &HistoricalAmendmentReviewer) -> Option<String> {
    if r.reviewer_id.is_some() {
        return None;
    }
    r.reviewer_name.as_deref().map(str::trim).map(String::from)
}
