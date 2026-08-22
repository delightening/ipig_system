//! import P2：補登審查文件——將歷史審查意見 / 獸醫評比寫入「真實」審查表
//! （review_comments / vet_review_assignments），支援系統內審查者（FK）與院外
//! 審查者（填姓名）。倫理委員會主席核准同意函以附件上傳，不在此模組。
//!
//! 僅允許於 import_pending=true 期間呼叫，且為「全量取代」語意（重送會清掉先前
//! 補登的審查資料再重建），避免重複累積。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{
        ImportCommitteeReviewer, ImportReviewComment, ImportReviewsRequest, ImportVetReview,
        Protocol,
    },
    services::{audit::ActivityLogEntry, AuditService},
    AppError, Result,
};

impl ProtocolService {
    /// 記錄補登審查文件（全量取代）。僅限 import_pending 計劃。
    pub async fn record_import_reviews(
        pool: &PgPool,
        actor: &ActorContext,
        protocol_id: Uuid,
        req: &ImportReviewsRequest,
    ) -> Result<()> {
        // Service 層拒絕 Anonymous 觸發 mutation（CLAUDE.md §ActorContext::Anonymous 規範 2）
        if matches!(actor, ActorContext::Anonymous) {
            return Err(AppError::Forbidden(
                "補登審查文件須由已登入使用者或系統觸發".into(),
            ));
        }
        Self::validate_import_reviews(req)?;

        let mut tx = pool.begin().await?;
        let protocol: Protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(protocol_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;
        if !protocol.import_pending {
            return Err(AppError::BusinessRule(
                "此計劃非補登中狀態，無法補登審查文件".to_string(),
            ));
        }

        // 全量取代：清掉先前補登的審查資料（import_pending 計劃僅有補登來源資料）
        sqlx::query("DELETE FROM review_comments WHERE protocol_id = $1")
            .bind(protocol_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM vet_review_assignments WHERE protocol_id = $1")
            .bind(protocol_id)
            .execute(&mut *tx)
            .await?;

        for c in &req.secretary_comments {
            Self::insert_import_comment(&mut tx, protocol_id, c, "PRE_REVIEW").await?;
        }
        for reviewer in &req.committee_reviewers {
            Self::insert_committee_reviewer(&mut tx, protocol_id, reviewer).await?;
        }
        if let Some(vet) = &req.vet_review {
            Self::insert_import_vet_review(&mut tx, protocol_id, vet).await?;
        }

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_IMPORT_REVIEWS_RECORDED",
                entity: Some(crate::services::audit::AuditEntity::new(
                    "protocol",
                    protocol_id,
                    &protocol.title,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// 驗證：每條意見 reviewer_id / reviewer_name 至少其一、content 非空。
    fn validate_import_reviews(req: &ImportReviewsRequest) -> Result<()> {
        let all_comments = req.secretary_comments.iter().chain(
            req.committee_reviewers
                .iter()
                .flat_map(|r| r.first_round.iter().chain(r.second_round.iter())),
        );
        for c in all_comments {
            if c.content.trim().is_empty() {
                return Err(AppError::Validation("審查意見內容不可為空".into()));
            }
        }
        // 執秘意見的審查者身分在每條意見上（委員 / 獸醫的身分在外層 reviewer/vet）；
        // 缺 id|name 會違反 review_comments CHECK，須在此擋為 400 而非 DB 500。
        for c in &req.secretary_comments {
            if c.reviewer_id.is_none() && reviewer_name_empty(&c.reviewer_name) {
                return Err(AppError::Validation(
                    "執秘意見須指定系統帳號或填寫姓名".into(),
                ));
            }
        }
        for r in &req.committee_reviewers {
            if r.reviewer_id.is_none() && reviewer_name_empty(&r.reviewer_name) {
                return Err(AppError::Validation("委員須指定系統帳號或填寫姓名".into()));
            }
        }
        if let Some(vet) = &req.vet_review {
            if vet.vet_id.is_none() && reviewer_name_empty(&vet.vet_name) {
                return Err(AppError::Validation(
                    "獸醫師須指定系統帳號或填寫姓名".into(),
                ));
            }
        }
        Ok(())
    }

    /// 插入一條審查意見（含可選的申請人回覆子意見）。回傳意見 id。
    async fn insert_import_comment(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
        c: &ImportReviewComment,
        review_stage: &str,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO review_comments
                 (id, protocol_id, reviewer_id, reviewer_name, content, section_no, review_stage, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())"#,
        )
        .bind(id)
        .bind(protocol_id)
        .bind(c.reviewer_id)
        .bind(name_or_fallback(&c.reviewer_id, &c.reviewer_name))
        .bind(c.content.trim())
        .bind(section_no_or_none(&c.section_no))
        .bind(review_stage)
        .execute(&mut **tx)
        .await?;

        if let Some(reply) = c.reply.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(
                r#"INSERT INTO review_comments
                     (id, protocol_id, reviewer_id, reviewer_name, content, parent_comment_id, review_stage, created_at, updated_at)
                   VALUES ($1, $2, NULL, $3, $4, $5, $6, NOW(), NOW())"#,
            )
            .bind(Uuid::new_v4())
            .bind(protocol_id)
            .bind("申請人")
            .bind(reply)
            .bind(id)
            .bind(review_stage)
            .execute(&mut **tx)
            .await?;
        }
        Ok(id)
    }

    /// 插入一位委員的一審 + 二審意見。
    async fn insert_committee_reviewer(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
        reviewer: &ImportCommitteeReviewer,
    ) -> Result<()> {
        for c in &reviewer.first_round {
            let merged = merge_reviewer(c, reviewer);
            Self::insert_import_comment(tx, protocol_id, &merged, "UNDER_REVIEW").await?;
        }
        for c in &reviewer.second_round {
            let merged = merge_reviewer(c, reviewer);
            Self::insert_import_comment(tx, protocol_id, &merged, "FINAL_REVIEW").await?;
        }
        Ok(())
    }

    /// 插入獸醫師評比與意見（vet_review_assignments，review_form JSONB）。
    async fn insert_import_vet_review(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
        vet: &ImportVetReview,
    ) -> Result<()> {
        let review_form = serde_json::json!({
            "items": vet.items,
            "vet_signature": null,
            "signed_at": vet.signed_at,
        });
        sqlx::query(
            r#"INSERT INTO vet_review_assignments
                 (id, protocol_id, vet_id, vet_name, assigned_at, completed_at, decision, decision_remark, review_form)
               VALUES ($1, $2, $3, $4, NOW(), $5, $6, $7, $8)"#,
        )
        .bind(Uuid::new_v4())
        .bind(protocol_id)
        .bind(vet.vet_id)
        .bind(name_or_fallback(&vet.vet_id, &vet.vet_name))
        .bind(vet.signed_at)
        .bind(vet.decision.as_deref())
        .bind(vet.decision_remark.as_deref())
        .bind(review_form)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

/// reviewer_name trim 後是否為空。
fn reviewer_name_empty(name: &Option<String>) -> bool {
    name.as_deref().map(str::trim).unwrap_or("").is_empty()
}

/// 項次 trim 後存入；空字串視為 None。
fn section_no_or_none(section_no: &Option<String>) -> Option<String> {
    section_no
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// 系統內審查者（有 id）不存姓名（顯示用 users.display_name）；院外存 trim 後姓名。
fn name_or_fallback(id: &Option<Uuid>, name: &Option<String>) -> Option<String> {
    if id.is_some() {
        return None;
    }
    name.as_deref().map(str::trim).map(String::from)
}

/// 委員的 reviewer_id / reviewer_name 套用到其每條意見。
fn merge_reviewer(
    c: &ImportReviewComment,
    reviewer: &ImportCommitteeReviewer,
) -> ImportReviewComment {
    ImportReviewComment {
        reviewer_id: reviewer.reviewer_id,
        reviewer_name: reviewer.reviewer_name.clone(),
        content: c.content.clone(),
        reply: c.reply.clone(),
        section_no: c.section_no.clone(),
    }
}
