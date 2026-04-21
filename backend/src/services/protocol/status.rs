use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, ChangeStatusRequest, CreatePartnerRequest, PartnerType, Protocol,
        ProtocolActivityType, ProtocolStatus,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, PartnerService,
    },
    AppError, Result,
};
use validator::Validate;

impl ProtocolService {
    /// 驗證計畫內容
    fn validate_protocol_content(content: &Option<Value>) -> Result<()> {
        let content = content
            .as_ref()
            .ok_or_else(|| AppError::Validation("Protocol content is empty".to_string()))?;

        // 驗證基本資料
        let basic = content
            .get("basic")
            .ok_or_else(|| AppError::Validation("Missing 'basic' section".to_string()))?;

        // 驗證標題 (AUP 2.2)
        if basic
            .get("study_title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err(AppError::Validation("Study title is required".to_string()));
        }

        // 驗證 GLP (AUP 2.1)
        let is_glp = basic
            .get("is_glp")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_glp
            && basic
                .get("registration_authorities")
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true)
        {
            return Err(AppError::Validation(
                "Registration authorities required for GLP study".to_string(),
            ));
        }

        // 驗證計畫類型 (AUP 2.7)
        if basic
            .get("project_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err(AppError::Validation("Project type is required".to_string()));
        }

        // 驗證動物總數 (AUP 8.1)
        if let Some(_animals_section) = content.get("animals") {
            // 這裡可以做更多檢查
        }

        Ok(())
    }

    /// 提交計畫 — Service-driven：所有 DB 操作（版本快照、UPDATE、狀態歷程、
    /// audit log、HMAC chain）在單一 transaction 內原子完成；失敗時整體 rollback。
    ///
    /// 這是 PR #3 的 **pattern demonstration**：後續 R26 模組（animals / hr / equipment）
    /// 依此模式改造。
    ///
    /// **變更自舊版**：
    /// - 簽名：`(pool, id, submitted_by)` → `(pool, actor, id)`
    ///   actor_user_id 從 `actor.actor_user_id()` 取得；透過 `actor.require_user()`
    ///   確保只有真實登入使用者可送出計畫
    /// - 所有 DB 操作綁同一 tx（`pool.begin()` → 各步 `&mut *tx` → `tx.commit()`）
    /// - IACUC numbering 走 tx 版本（`generate_apig_no(&mut tx)`），advisory lock
    ///   與本次 UPDATE 同 tx 提交，完整修復 CRIT-01 race condition
    /// - Audit 走 `log_activity_tx` + 含 before/after DataDiff，GLP 稽核軌跡完整
    /// - Handler 不再需要額外 `tokio::spawn(audit)` fire-and-forget
    pub async fn submit(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<Protocol> {
        // 送出計畫必須由真實登入使用者觸發（不可 System / Anonymous）
        let _user = actor.require_user()?;

        let mut tx = pool.begin().await?;

        // 讀取 before 狀態（含權限/狀態轉移檢查）
        let before = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        if before.status != ProtocolStatus::Draft
            && before.status != ProtocolStatus::RevisionRequired
            && before.status != ProtocolStatus::PreReviewRevisionRequired
            && before.status != ProtocolStatus::VetRevisionRequired
        {
            return Err(AppError::BusinessRule(format!(
                "Cannot submit protocol in {} status",
                before.status.as_str()
            )));
        }

        Self::validate_protocol_content(&before.working_content)?;

        let new_status = if before.status == ProtocolStatus::Draft {
            ProtocolStatus::Submitted
        } else {
            ProtocolStatus::Resubmitted
        };

        // 建立版本快照
        let version_no = Self::get_next_version_no_tx(&mut tx, id).await?;
        sqlx::query(
            r#"
            INSERT INTO protocol_versions (id, protocol_id, version_no, content_snapshot, submitted_at, submitted_by)
            VALUES ($1, $2, $3, $4, NOW(), $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(version_no)
        .bind(&before.working_content)
        .bind(actor.actor_user_id())
        .execute(&mut *tx)
        .await?;

        // 生成 APIG 編號（若需要）— 在同一 tx 內，advisory lock 保證唯一
        let new_iacuc_no = if new_status == ProtocolStatus::Submitted {
            let needs_apig = before
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);

            if needs_apig {
                Some(Self::generate_apig_no(&mut tx).await?)
            } else {
                before.iacuc_no.clone()
            }
        } else {
            before.iacuc_no.clone()
        };

        // UPDATE protocols
        let after = sqlx::query_as::<_, Protocol>(
            "UPDATE protocols SET status = $2, iacuc_no = $3, updated_at = NOW() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(new_status)
        .bind(&new_iacuc_no)
        .fetch_one(&mut *tx)
        .await?;

        // 狀態變更歷程 + 全域 audit log（tx 版本，HMAC 涵蓋）
        Self::record_status_change_tx(
            &mut tx,
            actor,
            id,
            Some(before.status),
            new_status,
            None,
        )
        .await?;

        // Service-driven audit：before/after snapshot 進 HMAC chain
        let diff = DataDiff::compute(Some(&before), Some(&after));
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "PROTOCOL_SUBMIT",
                entity: Some(AuditEntity::new("protocol", id, &before.title)),
                data_diff: Some(diff),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    /// 變更狀態
    pub async fn change_status(
        pool: &PgPool,
        id: Uuid,
        req: &ChangeStatusRequest,
        changed_by: Uuid,
    ) -> Result<Protocol> {
        let protocol = sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // 驗證 DELETED 狀態：僅允許草稿或需修訂狀態
        if req.to_status == ProtocolStatus::Deleted
            && protocol.status != ProtocolStatus::Draft
            && protocol.status != ProtocolStatus::RevisionRequired
        {
            return Err(AppError::BusinessRule(
                "Only draft or revision-required protocols can be deleted".to_string(),
            ));
        }

        // 驗證 UNDER_REVIEW 狀態必須提供 2-3 位審查委員
        if req.to_status == ProtocolStatus::UnderReview {
            // 檢查上一個狀態（從預審、獸醫審查或提交/重送進入）
            if protocol.status != ProtocolStatus::VetReview
                && protocol.status != ProtocolStatus::Resubmitted
                && protocol.status != ProtocolStatus::PreReview
                && protocol.status != ProtocolStatus::Submitted
            {
                return Err(AppError::BusinessRule(
                    "必須從提交、預審、獸醫審查或重送狀態進入正式審查".to_string(),
                ));
            }

            let reviewer_ids = req
                .reviewer_ids
                .as_ref()
                .ok_or_else(|| AppError::Validation("必須選擇審查委員".to_string()))?;

            if reviewer_ids.len() < 2 || reviewer_ids.len() > 3 {
                return Err(AppError::Validation("必須選擇 2-3 位審查委員".to_string()));
            }
        }

        // 驗證 PRE_REVIEW 狀態必須從 SUBMITTED、RESUBMITTED 或 PRE_REVIEW_REVISION_REQUIRED 進入
        if req.to_status == ProtocolStatus::PreReview {
            if protocol.status != ProtocolStatus::Submitted
                && protocol.status != ProtocolStatus::Resubmitted
                && protocol.status != ProtocolStatus::PreReviewRevisionRequired
            {
                return Err(AppError::BusinessRule(
                    "必須從已送審或行政補件狀態進入行政預審".to_string(),
                ));
            }

            // 只有從 SUBMITTED 進入時才需要檢查 co-editor
            if protocol.status == ProtocolStatus::Submitted {
                let co_editor_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM user_protocols WHERE protocol_id = $1 AND role_in_protocol = 'CO_EDITOR'"
                )
                .bind(id)
                .fetch_one(pool)
                .await?;

                if co_editor_count == 0 {
                    return Err(AppError::BusinessRule(
                        "進入行政預審前必須指派至少一位試驗工作人員 (Co-editor)".to_string(),
                    ));
                }
            }
        }

        // 驗證 PRE_REVIEW_REVISION_REQUIRED 狀態必須從 PRE_REVIEW 進入
        if req.to_status == ProtocolStatus::PreReviewRevisionRequired
            && protocol.status != ProtocolStatus::PreReview
        {
            return Err(AppError::BusinessRule(
                "只能從行政預審狀態要求補件".to_string(),
            ));
        }

        // 驗證 VET_REVIEW 狀態必須從 PRE_REVIEW、SUBMITTED、RESUBMITTED 或 VET_REVISION_REQUIRED 進入
        if req.to_status == ProtocolStatus::VetReview
            && protocol.status != ProtocolStatus::PreReview
            && protocol.status != ProtocolStatus::Submitted
            && protocol.status != ProtocolStatus::Resubmitted
            && protocol.status != ProtocolStatus::VetRevisionRequired
        {
            return Err(AppError::BusinessRule(
                "必須從行政預審、已送審、重送或獸醫修訂狀態進入獸醫審查".to_string(),
            ));
        }

        // 驗證 VET_REVISION_REQUIRED 狀態必須從 VET_REVIEW 進入
        if req.to_status == ProtocolStatus::VetRevisionRequired
            && protocol.status != ProtocolStatus::VetReview
        {
            return Err(AppError::BusinessRule(
                "只能從獸醫審查狀態要求修訂".to_string(),
            ));
        }

        // 驗證 APPROVED / APPROVED_WITH_CONDITIONS 狀態：所有被指派的審查委員必須發表過意見
        if req.to_status == ProtocolStatus::Approved
            || req.to_status == ProtocolStatus::ApprovedWithConditions
        {
            // 檢查是否從 UNDER_REVIEW 狀態進入
            if protocol.status != ProtocolStatus::UnderReview {
                return Err(AppError::BusinessRule(
                    "必須從正式審查狀態進入核准".to_string(),
                ));
            }

            // 查詢所有被指派的正式審查委員
            let assigned_reviewers: Vec<Uuid> = sqlx::query_scalar(
                r#"
                SELECT reviewer_id FROM review_assignments 
                WHERE protocol_id = $1 AND is_primary_reviewer = true
                "#,
            )
            .bind(id)
            .fetch_all(pool)
            .await?;

            if assigned_reviewers.is_empty() {
                return Err(AppError::BusinessRule(
                    "尚未指派審查委員，無法核准".to_string(),
                ));
            }

            // 查詢已發表意見的審查委員（包含透過 protocol_id 或 protocol_version_id 發表的意見）
            let reviewers_with_comments: Vec<Uuid> = sqlx::query_scalar(
                r#"
                SELECT DISTINCT reviewer_id FROM review_comments 
                WHERE (protocol_id = $1 OR protocol_version_id IN (
                    SELECT id FROM protocol_versions WHERE protocol_id = $1
                ))
                AND parent_comment_id IS NULL
                "#,
            )
            .bind(id)
            .fetch_all(pool)
            .await?;

            // 找出尚未發表意見的審查委員
            let missing_reviewers: Vec<&Uuid> = assigned_reviewers
                .iter()
                .filter(|r| !reviewers_with_comments.contains(r))
                .collect();

            if !missing_reviewers.is_empty() {
                // 查詢尚未發表意見的審查委員姓名
                let missing_names: Vec<String> = sqlx::query_scalar(
                    r#"
                    SELECT COALESCE(display_name, email) FROM users 
                    WHERE id = ANY($1::uuid[])
                    "#,
                )
                .bind(missing_reviewers.to_vec())
                .fetch_all(pool)
                .await?;

                return Err(AppError::BusinessRule(format!(
                    "以下審查委員尚未發表意見，無法核准：{}",
                    missing_names.join("、")
                )));
            }
        }

        // IACUC 編號生成規則：
        // 1. 在計劃被提交審查與核准前（Submitted 狀態），生成 APIG-{ROC}{03}
        // 2. 在計劃被核准時（Approved 狀態），生成 PIG-{ROC}{03}
        let new_iacuc_no = if req.to_status == ProtocolStatus::Submitted {
            // 如果還沒有 APIG 編號，則生成
            let needs_apig = protocol
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);

            if needs_apig {
                Some(Self::generate_apig_no_pool(pool).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else if req.to_status == ProtocolStatus::PreReview {
            // 如果狀態變為 PreReview 但還沒有 APIG 編號，則生成（備用邏輯）
            let needs_apig = protocol
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);

            if needs_apig {
                Some(Self::generate_apig_no_pool(pool).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else if req.to_status == ProtocolStatus::Approved
            || req.to_status == ProtocolStatus::ApprovedWithConditions
        {
            // 核准時生成 IACUC 編號（PIG-{ROC}{03}）
            Some(Self::generate_iacuc_no_pool(pool).await?)
        } else {
            protocol.iacuc_no.clone()
        };

        let updated = sqlx::query_as::<_, Protocol>(
            r#"
            UPDATE protocols SET 
                status = $2, 
                iacuc_no = $3,
                updated_at = NOW() 
            WHERE id = $1 
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(req.to_status)
        .bind(&new_iacuc_no)
        .fetch_one(pool)
        .await?;

        // 記錄狀態變更（若進入 UNDER_REVIEW，remark 中包含審查委員姓名）
        let status_remark = if req.to_status == ProtocolStatus::UnderReview {
            if let Some(reviewer_ids) = &req.reviewer_ids {
                let names: Vec<String> = sqlx::query_scalar(
                    "SELECT COALESCE(display_name, email) FROM users WHERE id = ANY($1::uuid[])",
                )
                .bind(reviewer_ids)
                .fetch_all(pool)
                .await?;
                let reviewer_list = names.join("、");
                Some(format!("指派審查委員：{}", reviewer_list))
            } else {
                req.remark.clone()
            }
        } else {
            req.remark.clone()
        };
        Self::record_status_change(
            pool,
            id,
            Some(protocol.status),
            req.to_status,
            changed_by,
            status_remark,
        )
        .await?;

        // 當狀態變為 UNDER_REVIEW 時，自動指派選定的審查委員（標記為正式審查委員）
        if req.to_status == ProtocolStatus::UnderReview {
            if let Some(reviewer_ids) = &req.reviewer_ids {
                for reviewer_id in reviewer_ids {
                    Self::assign_primary_reviewer(pool, id, *reviewer_id, changed_by).await?;
                }

                // 記錄審查委員指派詳細資訊到活動歷程 extra_data
                let reviewer_info: Vec<(Uuid, String)> = sqlx::query_as(
                    "SELECT id, COALESCE(display_name, email) FROM users WHERE id = ANY($1::uuid[])"
                )
                .bind(reviewer_ids)
                .fetch_all(pool)
                .await?;

                let extra = serde_json::json!({
                    "reviewers": reviewer_info.iter().map(|(rid, name)| {
                        serde_json::json!({"id": rid, "name": name})
                    }).collect::<Vec<_>>()
                });

                let reviewer_names: Vec<&str> =
                    reviewer_info.iter().map(|(_, n)| n.as_str()).collect();
                Self::record_activity(
                    pool,
                    id,
                    ProtocolActivityType::ReviewerAssigned,
                    changed_by,
                    None,
                    Some(format!("指派 {} 位審查委員", reviewer_ids.len())),
                    None,
                    Some(format!("審查委員：{}", reviewer_names.join("、"))),
                    Some(extra),
                )
                .await?;
            }
        }

        // 當狀態變為 VET_REVIEW 時，自動指派獸醫師
        if req.to_status == ProtocolStatus::VetReview {
            Self::assign_vet_reviewer(pool, id, req.vet_id, changed_by).await?;
        }

        // 當計劃通過時，自動依照 IACUC No. 自動填入客戶
        if req.to_status == ProtocolStatus::Approved
            || req.to_status == ProtocolStatus::ApprovedWithConditions
        {
            if let Some(iacuc_no) = new_iacuc_no.as_ref() {
                // 檢查是否已存在該客戶（客戶代碼 = IACUC No.）
                let existing_customer: Option<uuid::Uuid> = sqlx::query_scalar(
                    "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1",
                )
                .bind(iacuc_no)
                .fetch_optional(pool)
                .await?;

                // 如果不存在，則創建新客戶
                if existing_customer.is_none() {
                    let create_req = CreatePartnerRequest {
                        partner_type: PartnerType::Customer,
                        code: Some(iacuc_no.clone()),
                        supplier_category: None,
                        customer_category: None,
                        name: iacuc_no.clone(),
                        tax_id: None,
                        phone: None,
                        phone_ext: None,
                        email: None,
                        address: None,
                        payment_terms: None,
                    };

                    // 驗證請求
                    if let Err(validation_errors) = create_req.validate() {
                        tracing::warn!(
                            "Failed to validate customer creation request for IACUC {}: {:?}",
                            iacuc_no,
                            validation_errors
                        );
                    } else {
                        // 創建客戶，忽略錯誤（例如代碼衝突），因為可能已經存在
                        if let Err(e) = PartnerService::create(pool, &create_req).await {
                            tracing::warn!(
                                "Failed to create customer for IACUC {}: {}",
                                iacuc_no,
                                e
                            );
                        } else {
                            tracing::info!(
                                "Automatically created customer for IACUC: {}",
                                iacuc_no
                            );
                        }
                    }
                }
            }
        }

        // 當計劃結案時，自動停用對應的客戶
        if req.to_status == ProtocolStatus::Closed {
            if let Some(iacuc_no) = protocol.iacuc_no.as_ref() {
                // 查找對應的客戶（客戶代碼 = IACUC No.）
                let customer_id: Option<uuid::Uuid> = sqlx::query_scalar(
                    "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1",
                )
                .bind(iacuc_no)
                .fetch_optional(pool)
                .await?;

                // 如果找到客戶，則停用該客戶
                if let Some(customer_id) = customer_id {
                    let result = sqlx::query(
                        "UPDATE partners SET is_active = false, updated_at = NOW() WHERE id = $1",
                    )
                    .bind(customer_id)
                    .execute(pool)
                    .await?;

                    if result.rows_affected() > 0 {
                        tracing::info!(
                            "Automatically deactivated customer for closed IACUC: {}",
                            iacuc_no
                        );
                    } else {
                        tracing::warn!(
                            "Failed to deactivate customer for IACUC {}: customer not found",
                            iacuc_no
                        );
                    }
                } else {
                    tracing::warn!("No customer found for closed IACUC: {}", iacuc_no);
                }
            }
        }

        // 記錄活動日誌（狀態變更）
        if let Err(e) = AuditService::log_activity(
            pool,
            changed_by,
            "AUP",
            "PROTOCOL_STATUS_CHANGE",
            Some("protocol"),
            Some(id),
            Some(&protocol.title),
            Some(serde_json::json!({ "status": protocol.status })),
            Some(serde_json::json!({ "status": req.to_status, "remark": req.remark })),
            None,
            None,
        )
        .await
        {
            tracing::error!(
                "寫入 user_activity_logs 失敗 (PROTOCOL_STATUS_CHANGE): {}",
                e
            );
        }

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::ProtocolService;
    use serde_json::json;

    // --- validate_protocol_content ---

    #[test]
    fn test_validate_content_missing_content() {
        let result = ProtocolService::validate_protocol_content(&None);
        assert!(result.is_err());
        assert!(result
            .expect_err("should return error")
            .to_string()
            .contains("content is empty"));
    }

    #[test]
    fn test_validate_content_missing_basic_section() {
        let content = json!({ "animals": {} });
        let result = ProtocolService::validate_protocol_content(&Some(content));
        assert!(result.is_err());
        assert!(result
            .expect_err("should return error")
            .to_string()
            .contains("Missing 'basic' section"));
    }

    #[test]
    fn test_validate_content_missing_study_title() {
        let content = json!({
            "basic": {
                "study_title": "   ",
                "project_type": "research"
            }
        });
        let result = ProtocolService::validate_protocol_content(&Some(content));
        assert!(result.is_err());
        assert!(result
            .expect_err("should return error")
            .to_string()
            .contains("Study title is required"));
    }

    #[test]
    fn test_validate_content_glp_without_authorities() {
        let content = json!({
            "basic": {
                "study_title": "Test Study",
                "is_glp": true,
                "registration_authorities": [],
                "project_type": "research"
            }
        });
        let result = ProtocolService::validate_protocol_content(&Some(content));
        assert!(result.is_err());
        assert!(result
            .expect_err("should return error")
            .to_string()
            .contains("Registration authorities required"));
    }

    #[test]
    fn test_validate_content_glp_with_authorities_ok() {
        let content = json!({
            "basic": {
                "study_title": "GLP Study",
                "is_glp": true,
                "registration_authorities": ["FDA"],
                "project_type": "research"
            }
        });
        assert!(ProtocolService::validate_protocol_content(&Some(content)).is_ok());
    }

    #[test]
    fn test_validate_content_missing_project_type() {
        let content = json!({
            "basic": {
                "study_title": "Test Study",
                "is_glp": false,
                "project_type": ""
            }
        });
        let result = ProtocolService::validate_protocol_content(&Some(content));
        assert!(result.is_err());
        assert!(result
            .expect_err("should return error")
            .to_string()
            .contains("Project type is required"));
    }

    #[test]
    fn test_validate_content_valid() {
        let content = json!({
            "basic": {
                "study_title": "Valid Study",
                "is_glp": false,
                "project_type": "experiment"
            }
        });
        assert!(ProtocolService::validate_protocol_content(&Some(content)).is_ok());
    }
}
