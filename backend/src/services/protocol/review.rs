use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::history::event_type_for;
use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{AssignReviewerRequest, ProtocolActivityType, ReviewAssignment},
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

/// R32-A8e: review_reply / review_result 匯出用 row。
#[derive(sqlx::FromRow)]
struct ReviewCommentRow {
    id: Uuid,
    #[sqlx(default)]
    reviewer_id: Option<Uuid>,
    #[sqlx(default)]
    reviewer_name: Option<String>,
    content: String,
    /// 計畫書項次（如 4.1.2，補登填寫）
    #[sqlx(default)]
    section_no: Option<String>,
    parent_comment_id: Option<Uuid>,
    review_stage: Option<String>,
}

impl ReviewCommentRow {
    /// 分組鍵：系統內審查者用 id、院外審查者用姓名，確保不同審查者不被合併。
    fn reviewer_key(&self) -> String {
        match self.reviewer_id {
            Some(id) => format!("id:{id}"),
            None => format!("name:{}", self.reviewer_name.as_deref().unwrap_or("")),
        }
    }
}

/// 列印審查結果時，意見前綴計畫書項次（如「【4.1.2】意見內容」）；無項次則原樣。
fn prefix_section_no(row: &(String, Option<String>)) -> String {
    let (content, section_no) = row;
    match section_no
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) => format!("【{s}】{content}"),
        None => content.clone(),
    }
}

impl ProtocolService {
    /// 指派審查人員
    pub async fn assign_reviewer(
        pool: &PgPool,
        req: &AssignReviewerRequest,
        assigned_by: Uuid,
    ) -> Result<ReviewAssignment> {
        // H1: 防止自我審核（提交者 ≠ 審查者），確保 IACUC 獨立審查原則
        let submitter_id: Option<Uuid> =
            sqlx::query_scalar("SELECT pi_user_id FROM protocols WHERE id = $1")
                .bind(req.protocol_id)
                .fetch_optional(pool)
                .await?;

        if let Some(pi_id) = submitter_id {
            if pi_id == req.reviewer_id {
                return Err(AppError::Validation(
                    "審查委員不得為計畫提交者（PI），違反 IACUC 獨立審查原則".to_string(),
                ));
            }
        }

        let assignment = sqlx::query_as::<_, ReviewAssignment>(
            r#"
            INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by, assigned_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (protocol_id, reviewer_id) DO UPDATE SET assigned_at = NOW()
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(req.protocol_id)
        .bind(req.reviewer_id)
        .bind(assigned_by)
        .fetch_one(pool)
        .await?;

        Self::record_activity(
            pool,
            req.protocol_id,
            ProtocolActivityType::ReviewerAssigned,
            assigned_by,
            None,
            None,
            Some(("user", req.reviewer_id, "Reviewer")),
            Some(format!("Assigned reviewer {}", req.reviewer_id)),
            None,
        )
        .await?;

        Ok(assignment)
    }

    /// Transaction 版本：在既有 transaction 內指派正式審查委員（R26-8 Phase 2 需用）
    pub(super) async fn assign_primary_reviewer_tx(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
        reviewer_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<ReviewAssignment> {
        let assignment = sqlx::query_as::<_, ReviewAssignment>(
            r#"
            INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by, assigned_at, is_primary_reviewer, review_stage)
            VALUES ($1, $2, $3, $4, NOW(), true, 'UNDER_REVIEW')
            ON CONFLICT (protocol_id, reviewer_id) DO UPDATE SET
                assigned_at = NOW(),
                is_primary_reviewer = true,
                review_stage = 'UNDER_REVIEW'
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_id)
        .bind(reviewer_id)
        .bind(assigned_by)
        .fetch_one(&mut **tx)
        .await?;

        Ok(assignment)
    }

    /// Transaction 版本：在既有 transaction 內指派獸醫審查員（R26-8 Phase 2 需用）。
    /// 同 tx 內記錄 `VetAssigned` activity，確保 audit trail 與指派同原子性。
    pub(super) async fn assign_vet_reviewer_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        protocol_id: Uuid,
        vet_id: Option<Uuid>,
    ) -> Result<()> {
        let assigned_by = actor
            .actor_user_id()
            .unwrap_or(crate::middleware::SYSTEM_USER_ID);
        // 如果未指定獸醫師，從系統設定取得預設獸醫師
        let vet_user_id = if let Some(id) = vet_id {
            id
        } else {
            // 從系統設定取得預設獸醫師
            let setting: Option<serde_json::Value> = sqlx::query_scalar(
                "SELECT value FROM system_settings WHERE key = 'default_vet_reviewer'",
            )
            .fetch_optional(&mut **tx)
            .await?;

            let default_vet_id = setting
                .and_then(|v| v.get("user_id")?.as_str().map(|s| s.to_string()))
                .and_then(|s| Uuid::parse_str(&s).ok());

            match default_vet_id {
                Some(id) => id,
                None => {
                    // 如果沒有設定預設獸醫師，查找第一個具有 VET 角色的使用者
                    let first_vet: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT u.id FROM users u
                        INNER JOIN user_roles ur ON u.id = ur.user_id
                        INNER JOIN roles r ON ur.role_id = r.id
                        WHERE r.code = 'VET' AND u.is_active = true
                        LIMIT 1
                        "#,
                    )
                    .fetch_optional(&mut **tx)
                    .await?;

                    first_vet.ok_or_else(|| {
                        AppError::BusinessRule("系統中沒有可用的獸醫師".to_string())
                    })?
                }
            }
        };

        // 驗證指定的使用者是獸醫師
        let is_vet: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1 AND r.code = 'VET'
            )
            "#,
        )
        .bind(vet_user_id)
        .fetch_one(&mut **tx)
        .await?;

        if !is_vet.0 {
            return Err(AppError::Validation(
                "指定的使用者不具有獸醫師角色".to_string(),
            ));
        }

        // 建立獸醫審查指派記錄
        sqlx::query(
            r#"
            INSERT INTO vet_review_assignments (id, protocol_id, vet_id, assigned_by, assigned_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (protocol_id) DO UPDATE SET
                vet_id = $3,
                assigned_by = $4,
                assigned_at = NOW(),
                completed_at = NULL,
                decision = NULL,
                decision_remark = NULL
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(protocol_id)
        .bind(vet_user_id)
        .bind(assigned_by)
        .execute(&mut **tx)
        .await?;

        Self::record_activity_tx(
            tx,
            actor,
            protocol_id,
            ProtocolActivityType::VetAssigned,
            None,
            None,
            Some(("user", vet_user_id, "Vet")),
            Some(format!("Assigned vet reviewer {}", vet_user_id)),
            None,
        )
        .await?;

        // PR #269 Option C：補 audit log（無 diff 的 timeline 事件）
        let protocol_title: String =
            sqlx::query_scalar("SELECT title FROM protocols WHERE id = $1")
                .bind(protocol_id)
                .fetch_optional(&mut **tx)
                .await?
                .unwrap_or_else(|| "Unknown Protocol".to_string());
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: event_type_for(ProtocolActivityType::VetAssigned),
                entity: Some(AuditEntity::new("protocol", protocol_id, &protocol_title)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        Ok(())
    }

    /// R32-A8e：聚合審查意見回覆表 PDF 匯出資料。
    ///
    /// 文件設計：每件 protocol 1 份 review_reply.docx（schema 詳見
    /// pdf-service/app/schemas/review_reply.py）。
    pub async fn get_review_reply_export_data(
        pool: &PgPool,
        scope: access::Scoped<access::ProtocolId>,
    ) -> Result<serde_json::Value> {
        let protocol_id = scope.id();
        let protocol = Self::get_by_id(pool, protocol_id).await?;
        let comments = Self::fetch_review_comments(pool, protocol_id).await?;
        let (application_no, pi_name) = Self::extract_review_metadata(&protocol);
        let (vet_items, vet_signed_date) = Self::build_vet_review_items(&protocol);
        let secretary_items = Self::build_secretary_items(&comments);
        let committees = Self::build_committee_items(&comments);

        Ok(serde_json::json!({
            "application_no": application_no,
            "study_title": protocol.protocol.title,
            "pi_name": pi_name,
            "secretary_items": secretary_items,
            "vet_items": vet_items,
            "vet_signed_date": vet_signed_date,
            "committee_1": committees[0],
            "committee_2": committees[1],
            "committee_3": committees[2],
            "committee_4": committees[3],
        }))
    }

    async fn fetch_review_comments(
        pool: &PgPool,
        protocol_id: Uuid,
    ) -> Result<Vec<ReviewCommentRow>> {
        let comments = sqlx::query_as::<_, ReviewCommentRow>(
            r#"SELECT id, reviewer_id, reviewer_name, content, section_no, parent_comment_id, review_stage
               FROM review_comments
               WHERE protocol_id = $1
               ORDER BY COALESCE(parent_comment_id, id), created_at"#,
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;
        Ok(comments)
    }

    fn extract_review_metadata(protocol: &crate::models::ProtocolResponse) -> (String, String) {
        let basic = protocol
            .protocol
            .working_content
            .as_ref()
            .and_then(|c| c.get("basic"));
        let application_no = protocol
            .protocol
            .iacuc_no
            .clone()
            .or_else(|| {
                basic.and_then(|b| {
                    b.get("apply_study_number")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                })
            })
            .unwrap_or_else(|| "尚未指派".into());
        let pi_name = basic
            .and_then(|b| b.get("pi"))
            .and_then(|pi| pi.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("未指定")
            .to_string();
        (application_no, pi_name)
    }

    fn build_vet_review_items(
        protocol: &crate::models::ProtocolResponse,
    ) -> (Vec<serde_json::Value>, String) {
        let vet_form: Option<crate::models::VetReviewForm> = protocol
            .vet_review
            .as_ref()
            .and_then(|vr| vr.review_form.as_ref())
            .and_then(|fv| serde_json::from_value(fv.clone()).ok());
        let items = vet_form
            .as_ref()
            .map(|f| {
                f.items
                    .iter()
                    .map(|item| {
                        serde_json::json!({
                            "status": item.compliance.clone(),
                            "opinion": item.comment.clone().unwrap_or_default(),
                            "reply": item.pi_reply.clone().unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let signed_date = vet_form
            .as_ref()
            .and_then(|f| f.signed_at.map(|d| d.format("%Y-%m-%d").to_string()))
            .unwrap_or_default();
        (items, signed_date)
    }

    fn collect_replies(comments: &[ReviewCommentRow], parent_id: Uuid) -> String {
        comments
            .iter()
            .filter(|r| r.parent_comment_id == Some(parent_id))
            .map(|r| r.content.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn build_secretary_items(comments: &[ReviewCommentRow]) -> Vec<serde_json::Value> {
        comments
            .iter()
            .filter(|c| {
                c.parent_comment_id.is_none() && c.review_stage.as_deref() == Some("PRE_REVIEW")
            })
            .enumerate()
            .map(|(idx, c)| {
                serde_json::json!({
                    "item_no": (idx + 1).to_string(),
                    "section_no": c.section_no.clone().unwrap_or_default(),
                    "opinion": c.content,
                    "reply": Self::collect_replies(comments, c.id),
                })
            })
            .collect()
    }

    /// R32-A8e bug fix（coderabbit PR #332）：committee 同時收一審 (UNDER_REVIEW)
    /// 與二審 (FINAL_REVIEW)；以 reviewer × item_no 配對成同一筆，opinion_2nd
    ///   與 reply_2nd 從二審補入，避免二審意見遺失。
    fn build_committee_items(comments: &[ReviewCommentRow]) -> [Vec<serde_json::Value>; 4] {
        // reviewer 出現順序（取一審 + 二審 union 的前 4 位）；
        // 以 reviewer_key 分組——系統內審查者用 id、院外審查者用姓名。
        let mut reviewer_order: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for c in comments {
            let stage = c.review_stage.as_deref();
            if matches!(
                stage,
                Some("UNDER_REVIEW") | Some("FINAL_REVIEW") | Some("FINAL")
            ) && c.parent_comment_id.is_none()
                && seen.insert(c.reviewer_key())
            {
                reviewer_order.push(c.reviewer_key());
            }
        }

        let collect_round = |rid: &str, stages: &[&str]| -> Vec<&ReviewCommentRow> {
            comments
                .iter()
                .filter(|c| {
                    c.reviewer_key() == rid
                        && c.parent_comment_id.is_none()
                        && c.review_stage
                            .as_deref()
                            .is_some_and(|s| stages.contains(&s))
                })
                .collect()
        };

        let mut result: [Vec<serde_json::Value>; 4] = Default::default();
        for (slot, rid) in reviewer_order.iter().take(4).enumerate() {
            let first_round = collect_round(rid, &["UNDER_REVIEW"]);
            let second_round = collect_round(rid, &["FINAL_REVIEW", "FINAL"]);
            // 以 index 對齊（第 1 條一審 ↔ 第 1 條二審；不存在則空）
            let max_len = first_round.len().max(second_round.len());
            result[slot] = (0..max_len)
                .map(|idx| {
                    let first = first_round.get(idx);
                    let second = second_round.get(idx);
                    serde_json::json!({
                        "item_no": (idx + 1).to_string(),
                        "section_no": first
                            .and_then(|c| c.section_no.clone())
                            .or_else(|| second.and_then(|c| c.section_no.clone()))
                            .unwrap_or_default(),
                        "opinion_1st": first.map(|c| c.content.clone()).unwrap_or_default(),
                        "reply_1st": first
                            .map(|c| Self::collect_replies(comments, c.id))
                            .unwrap_or_default(),
                        "opinion_2nd": second.map(|c| c.content.clone()).unwrap_or_default(),
                        "reply_2nd": second
                            .map(|c| Self::collect_replies(comments, c.id))
                            .unwrap_or_default(),
                    })
                })
                .collect();
        }
        result
    }

    /// R32-A8c：聚合審核結果 PDF 匯出資料。
    ///
    /// 文件設計（per vet/QA 確認）：每件 protocol 1 份 review_result.docx，
    /// 內含召集人彙整後的初審 + 複審結論（不是每位委員一份）。
    ///
    /// 實作策略：自動填 review_comments 為 `revision_opinions`（依 stage 分組後
    /// 以 newline join），decision flags / signatures / dates 留白讓召集人在
    ///   列印後手寫簽名。
    pub async fn get_review_result_export_data(
        pool: &PgPool,
        scope: access::Scoped<access::ProtocolId>,
    ) -> Result<serde_json::Value> {
        let protocol_id = scope.id();
        let protocol = Self::get_by_id(pool, protocol_id).await?;

        // 依 review_stage 分初/複審。當前 stage 值含 INITIAL_REVIEW / FINAL_REVIEW /
        // UNDER_REVIEW；若資料 stage 未細分（只有 UNDER_REVIEW），全部歸初審。
        // R32-A8c bug fix（coderabbit PR #332）：排除 parent_comment_id IS NOT NULL，
        // 否則申請人回覆會被當審查意見列印
        let initial_rows: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT content, section_no FROM review_comments \
             WHERE protocol_id = $1 \
               AND parent_comment_id IS NULL \
               AND COALESCE(review_stage, 'UNDER_REVIEW') IN ('INITIAL_REVIEW', 'INITIAL', 'UNDER_REVIEW') \
             ORDER BY created_at",
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;
        let initial_comments: Vec<String> = initial_rows.iter().map(prefix_section_no).collect();

        let final_rows: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT content, section_no FROM review_comments \
             WHERE protocol_id = $1 \
               AND parent_comment_id IS NULL \
               AND review_stage IN ('FINAL_REVIEW', 'FINAL') \
             ORDER BY created_at",
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;
        let final_comments: Vec<String> = final_rows.iter().map(prefix_section_no).collect();

        Ok(serde_json::json!({
            "protocol": {
                "id": protocol.protocol.id,
                "iacuc_no": protocol.protocol.iacuc_no,
                "protocol_no": protocol.protocol.protocol_no,
                "title": protocol.protocol.title,
                "status": protocol.protocol.status,
            },
            "initial_comments": initial_comments,
            "final_comments": final_comments,
        }))
    }
}
