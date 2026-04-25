use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
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

/// C2 (GLP §11.50/§11.70)：插入決定簽章記錄（內部，無密碼/手寫驗證模式），
/// 回傳新建簽章的 UUID 以供回填到 amendments.{approved,rejected}_signature_id。
///
/// 適用情境：
/// - check_all_decisions 自動聚合審查委員決定 → 終態（APPROVED/REJECTED）時，
///   由「最後一位 tipping reviewer」當簽章主體
/// - classify(Minor) → ADMIN_APPROVED 時，由 admin 當簽章主體
///
/// 不適用：需要使用者主動提供密碼/手寫的簽章（用 SignatureService::sign_record）
async fn insert_decision_signature_tx(
    tx: &mut Transaction<'_, Postgres>,
    amendment_id: Uuid,
    signer_id: Uuid,
    is_approve: bool,
    decision_summary: &str,
) -> Result<Uuid> {
    // content = canonical 描述（決定時刻的快照），可用 SignatureService::verify 驗證未被竄改
    let entity_type = "amendment";
    let entity_id = amendment_id.to_string();
    let decision_word = if is_approve { "APPROVE" } else { "REJECT" };
    let content = format!(
        "amendment_decision:{}:{}:{}",
        entity_id, decision_word, decision_summary
    );
    let content_hash = format!("{:x}", Sha256::digest(content.as_bytes()));
    let timestamp = Utc::now();
    let signature_input = format!(
        "{}:{}:{}:internal",
        signer_id, content_hash, timestamp.timestamp()
    );
    let signature_data = format!("{:x}", Sha256::digest(signature_input.as_bytes()));

    let sig_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO electronic_signatures (
            entity_type, entity_id, signer_id, signature_type,
            content_hash, signature_data, signature_method
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'internal')
        RETURNING id
        "#,
    )
    .bind(entity_type)
    .bind(&entity_id)
    .bind(signer_id)
    .bind(decision_word)
    .bind(&content_hash)
    .bind(&signature_data)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sig_id)
}

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
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
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

        let mut tx = pool.begin().await?;

        // C2 (GLP)：Minor → ADMIN_APPROVED 是終態，由 classifier 當簽章主體
        let approved_sig_id = if new_status == AmendmentStatus::AdminApproved {
            let summary = format!("admin_approved:type=MINOR,classifier={}", classified_by);
            Some(insert_decision_signature_tx(
                &mut tx, id, classified_by, /*is_approve=*/ true, &summary,
            ).await?)
        } else {
            None
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
                approved_signature_id = COALESCE($6, approved_signature_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING
                id, protocol_id, amendment_no, revision_number,
                amendment_type as "amendment_type: AmendmentType",
                status as "status: AmendmentStatus",
                title, description, change_items,
                changes_content, submitted_by, submitted_at,
                classified_by, classified_at, classification_remark,
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
            "#,
            id,
            req.amendment_type.as_str(),
            new_status.as_str(),
            classified_by,
            req.remark,
            approved_sig_id,
        )
        .fetch_one(&mut *tx)
        .await?;

        // 記錄狀態歷程
        let history_remark = if let Some(sig_id) = approved_sig_id {
            Some(format!("{}（行政核准簽章 {}）", req.remark.as_deref().unwrap_or(""), sig_id))
        } else {
            req.remark.clone()
        };
        Self::record_status_change_tx(
            &mut tx,
            id,
            Some(current.status),
            new_status,
            classified_by,
            history_remark,
        )
        .await?;

        tx.commit().await?;

        // 如果是重大變更，複製原計畫的審查委員（tx 外，非簽章關鍵路徑）
        if req.amendment_type == AmendmentType::Major {
            Self::assign_reviewers_from_protocol(pool, id, current.protocol_id, classified_by).await?;
        }

        Ok(amendment)
    }

    /// 從原計畫複製審查委員（批量 INSERT，避免 N+1）
    pub async fn assign_reviewers_from_protocol(
        pool: &PgPool,
        amendment_id: Uuid,
        protocol_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignment>> {
        let assignments = sqlx::query_as::<_, AmendmentReviewAssignment>(
            r#"
            INSERT INTO amendment_review_assignments (amendment_id, reviewer_id, assigned_by)
            SELECT $1, ra.reviewer_id, $3
            FROM review_assignments ra
            WHERE ra.protocol_id = $2
            ON CONFLICT (amendment_id, reviewer_id) DO NOTHING
            RETURNING
                id, amendment_id, reviewer_id, assigned_by, assigned_at,
                decision, decided_at, comment
            "#,
        )
        .bind(amendment_id)
        .bind(protocol_id)
        .bind(assigned_by)
        .fetch_all(pool)
        .await?;

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
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
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
    ///
    /// C2 (GLP)：record_decision 與聚合決定 (check_all_decisions_tx) + 終態簽章
    /// 寫入皆於同一 tx 完成，確保 audit trail 與決定簽章不被部分寫入。
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

        let mut tx = pool.begin().await?;

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
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Review assignment not found".into()))?;

        // 檢查所有審查委員是否都已完成決定，傳入 reviewer_id 作為「最後 tipping」簽章主體
        Self::check_all_decisions_tx(&mut tx, amendment_id, reviewer_id).await?;

        tx.commit().await?;
        Ok(assignment)
    }

    /// 檢查所有審查委員是否都已完成決定，並自動更新狀態（tx 版）
    ///
    /// C2 (GLP)：終態（APPROVED/REJECTED）時，於同 tx 內建立 electronic_signatures
    /// 記錄並回填到 amendments.{approved,rejected}_signature_id（21 CFR §11.50/§11.70 非否認性）。
    /// REVISION_REQUIRED 不簽章（非終態）。
    async fn check_all_decisions_tx(
        tx: &mut Transaction<'_, Postgres>,
        amendment_id: Uuid,
        tipping_reviewer_id: Uuid,
    ) -> Result<()> {
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
        .fetch_one(&mut **tx)
        .await?;

        // 所有人都還沒決定就不處理
        if stats.decided != stats.total {
            return Ok(());
        }

        // 取 current status（同 tx 讀，避免 race）
        let current_status = sqlx::query_scalar!(
            r#"SELECT status as "status: AmendmentStatus" FROM amendments WHERE id = $1"#,
            amendment_id
        )
        .fetch_one(&mut **tx)
        .await?;

        let summary = format!(
            "total={},approved={},rejected={},revision={}",
            stats.total, stats.approved, stats.rejected, stats.revision
        );

        // 有任何人要求修訂 → 需修訂（非終態，不簽章）
        if stats.revision > 0 {
            sqlx::query!(
                r#"UPDATE amendments SET status = 'REVISION_REQUIRED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(&mut **tx)
            .await?;

            Self::record_status_change_tx(
                tx,
                amendment_id,
                Some(current_status),
                AmendmentStatus::RevisionRequired,
                Uuid::nil(), // system（非終態，無 reviewer 主體）
                Some("審查委員要求修訂".to_string()),
            )
            .await?;
        }
        // 有任何人否決 → 終態，簽章
        else if stats.rejected > 0 {
            let sig_id = insert_decision_signature_tx(
                tx, amendment_id, tipping_reviewer_id, /*is_approve=*/ false, &summary,
            )
            .await?;

            sqlx::query!(
                r#"UPDATE amendments SET status = 'REJECTED', rejected_signature_id = $2, updated_at = NOW() WHERE id = $1"#,
                amendment_id,
                sig_id
            )
            .execute(&mut **tx)
            .await?;

            Self::record_status_change_tx(
                tx,
                amendment_id,
                Some(current_status),
                AmendmentStatus::Rejected,
                tipping_reviewer_id,
                Some(format!("審查委員否決（簽章 {}）", sig_id)),
            )
            .await?;
        }
        // 全部核准 → 終態，簽章
        else if stats.approved == stats.total {
            let sig_id = insert_decision_signature_tx(
                tx, amendment_id, tipping_reviewer_id, /*is_approve=*/ true, &summary,
            )
            .await?;

            sqlx::query!(
                r#"UPDATE amendments SET status = 'APPROVED', approved_signature_id = $2, updated_at = NOW() WHERE id = $1"#,
                amendment_id,
                sig_id
            )
            .execute(&mut **tx)
            .await?;

            Self::record_status_change_tx(
                tx,
                amendment_id,
                Some(current_status),
                AmendmentStatus::Approved,
                tipping_reviewer_id,
                Some(format!("全體審查委員核准（簽章 {}）", sig_id)),
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
                created_by, created_at, updated_at,
                approved_signature_id, rejected_signature_id
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
