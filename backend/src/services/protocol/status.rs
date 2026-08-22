use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::history::{activity_type_for_status, event_type_for};
use super::ProtocolService;
use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalStatus, ChangeStatusRequest, CreatePartnerRequest, PartnerType,
        Protocol, ProtocolActivityType, ProtocolStatus,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, PartnerService,
    },
    AppError, Result,
};
use validator::Validate;

impl ProtocolService {
    /// Transaction 版本：在既有 transaction 內變更計畫狀態（R26-8 Phase 2 核心）
    /// 所有 DB 操作（驗證、狀態更新、編號生成、指派、客戶建立、稽核日誌）於單一 tx 內原子完成。
    async fn change_status_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        id: Uuid,
        req: &ChangeStatusRequest,
    ) -> Result<Protocol> {
        // 讀取前置狀態（FOR UPDATE 確保同 tx 內鎖定）
        let protocol =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Protocol not found".to_string()))?;

        // CSO-r2 #2: 終態 / 已核准叢集 egress 鎖 — 防止已核准計畫經泛型 change_status
        // 被退回 review pipeline、否決或刪除（既有 per-target guard 僅管控「進入」審查狀態）。
        if !protocol.status.can_change_status_to(req.to_status) {
            return Err(AppError::BusinessRule(format!(
                "不允許將計畫狀態從「{}」變更為「{}」",
                protocol.status.display_name(),
                req.to_status.display_name()
            )));
        }

        // R89-8（2026-08-13）：計畫擁有人（PI/SD）結案自己計畫的窄縫通道。Handler 只做
        // 粗篩（目標狀態＝CLOSED 且無完整 aup.protocol.change_status 權限才放行到這裡）；
        // 這裡做完整檢查：① 操作者必須持有 aup.protocol.close_own——Qodo/PR-Agent
        // 於本 PR review 期間指出，`ProtocolService::change_status` 並非只有這個 handler
        // 呼叫（`services/mcp/tools.rs`、`services/protocol/ai_review.rs` 皆直接呼叫），
        // 若只靠 handler 那層 `require_permission!`，這些路徑會繞過 close_own 檢查，
        // 只要符合擁有人+狀態就放行，即使 actor 從未被授予這個權限；② 現在狀態必須是
        // 已核准（含附條件）——state machine 允許的其他來源狀態（如 SUSPENDED → CLOSED）
        // 仍須完整權限，不開放給窄縫；③ 操作者必須是這個 protocol 的擁有人。持有完整
        // change_status 權限者（IACUC_CHAIR/IACUC_STAFF/admin）不受此段限制。
        //
        // 「擁有人」判定與 `/my-projects` 的 `can_edit` 投影同一份定義（`my_protocols.rs`
        // 的 `my_protocols_sql`）：pi_user_id 或 study_director_user_id 本人，或
        // user_protocols 表 role_in_protocol='PI' 的委派 PI。前端結案鈕正是用 can_edit
        // 決定要不要顯示——兩邊定義不一致的話，委派 PI 會看到按鈕但點下去 403（R89 這批
        // 稽核本身就是在抓這種「按鈕能點但實際被鎖」的缺口，不能在新功能重蹈覆轍）。
        if req.to_status == ProtocolStatus::Closed {
            if let ActorContext::User(u) = actor {
                if !u.has_permission("aup.protocol.change_status") {
                    if !u.has_permission("aup.protocol.close_own") {
                        return Err(AppError::Forbidden("缺少結案自己計畫所需權限".to_string()));
                    }
                    if !matches!(
                        protocol.status,
                        ProtocolStatus::Approved | ProtocolStatus::ApprovedWithConditions
                    ) {
                        return Err(AppError::Forbidden(
                            "僅已核准（含附條件）的計畫可由計畫擁有人結案".to_string(),
                        ));
                    }
                    let is_direct_owner = protocol.pi_user_id == u.id
                        || protocol.study_director_user_id == Some(u.id);
                    let is_delegate_pi = if is_direct_owner {
                        false
                    } else {
                        sqlx::query_scalar::<_, bool>(
                            "SELECT EXISTS(SELECT 1 FROM user_protocols \
                             WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol = 'PI')",
                        )
                        .bind(id)
                        .bind(u.id)
                        .fetch_one(&mut **tx)
                        .await?
                    };
                    if !is_direct_owner && !is_delegate_pi {
                        return Err(AppError::Forbidden(
                            "僅計畫擁有人（PI/SD）本人可結案自己的計畫".to_string(),
                        ));
                    }
                }
            }
        }

        // Admin 駁回通道（2026-06-12）：預審階段（SUBMITTED / PRE_REVIEW）駁回為系統管理員
        // 專用，且必填理由。委員會審後（UNDER_REVIEW → REJECTED）維持既有權限不受此限。
        if req.to_status == ProtocolStatus::Rejected
            && matches!(
                protocol.status,
                ProtocolStatus::Submitted | ProtocolStatus::PreReview
            )
        {
            let is_admin_actor = match actor {
                ActorContext::System { .. } => true,
                ActorContext::User(u) => u.is_admin(),
                ActorContext::Anonymous => false,
            };
            if !is_admin_actor {
                return Err(AppError::Forbidden(
                    "預審階段駁回計畫僅限系統管理員".to_string(),
                ));
            }
            if req
                .remark
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
            {
                return Err(AppError::Validation("駁回計畫必須填寫理由".to_string()));
            }
        }

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

            // 只有從 SUBMITTED 進入時才需檢查已指派 SD（計劃負責人，內部試驗工作人員）。
            // 取代原「須有 ≥1 CO_EDITOR」規則（CO_EDITOR 角色已拆除，SD 為其內部負責人後繼）。
            if protocol.status == ProtocolStatus::Submitted
                && protocol.study_director_user_id.is_none()
            {
                return Err(AppError::BusinessRule(
                    "進入行政預審前必須指派計劃負責人（Study Director）".to_string(),
                ));
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
            // 檢查是否從 UNDER_REVIEW（正式審查通過）、SUSPENDED（暫停復原）或
            // APPROVED_WITH_CONDITIONS（附條件轉正式核准）狀態進入。三者皆沿用
            // 原核准時留下的審查委員意見與電子簽章（暫停不會作廢簽章，作廢是
            // admin 另呼叫 /signatures/:id/invalidate 的獨立動作），故下方
            // 「審查委員均已表意」與「須有有效簽章」守衛仍照舊沿用同一套查詢。
            if protocol.status != ProtocolStatus::UnderReview
                && protocol.status != ProtocolStatus::Suspended
                && protocol.status != ProtocolStatus::ApprovedWithConditions
            {
                return Err(AppError::BusinessRule(
                    "必須從正式審查、已暫停或附條件核准狀態進入核准".to_string(),
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
            .fetch_all(&mut **tx)
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
            .fetch_all(&mut **tx)
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
                .fetch_all(&mut **tx)
                .await?;

                return Err(AppError::BusinessRule(format!(
                    "以下審查委員尚未發表意見，無法核准：{}",
                    missing_names.join("、")
                )));
            }

            // R71-7：「已核准必有簽章」不變式 —— APPROVED 前須已有有效 protocol 電子簽章
            // （IACUC 主席/秘書經 /signatures/protocol/:id 簽核），否則拒絕。先簽再核准（做法 A）。
            // 用 FOR UPDATE 鎖住該有效簽章列：與 SignatureService::invalidate（同樣 SELECT ...
            // FOR UPDATE 鎖簽章列）互斥，避免「檢查通過 → 並行作廢 → 核准提交」的 TOCTOU 競態，
            // 確保核准提交當下簽章仍有效。
            let valid_signature_id: Option<String> = sqlx::query_scalar(
                "SELECT id::text FROM electronic_signatures \
                 WHERE entity_type = 'protocol' AND entity_id = $1 AND is_valid = true \
                 LIMIT 1 FOR UPDATE",
            )
            .bind(id.to_string())
            .fetch_optional(&mut **tx)
            .await?;
            if valid_signature_id.is_none() {
                return Err(AppError::BusinessRule(
                    "核准前須先完成計畫電子簽章（IACUC 主席/秘書）".to_string(),
                ));
            }
        }

        // 結案守門：計畫結案前，該計畫下所有動物必須皆已離場（安樂死 / 猝死 / 已轉讓）。
        // 涵蓋兩類串接：
        //   (1) 已分配 — animals.iacuc_no = protocols.iacuc_no 文字比對（無 FK；已分配才寫 iacuc_no）
        //   (2) 已預約 earmark — animals.reserved_protocol_id = protocol.id
        //       （預約只設 reserved_protocol_id、不寫 iacuc_no，故必須另條件涵蓋，
        //        否則「有豬預約給此計畫」時仍可結案 → 預約懸空成孤兒）
        // 若仍有存活動物（未分配 / 實驗中 / 實驗完成 / 已預約），拒絕結案。存活判定對齊
        // AnimalStatus::is_active_in_facility：status NOT IN euthanized/sudden_death/transferred。
        // 狀態值以 AnimalStatus enum 綁定為參數（單一事實來源），不硬編碼字面字串。
        // 以 CTE FOR UPDATE 鎖住候選動物列，關閉「查完無存活 → 並發 assign 進來 → 結案」的
        // TOCTOU 競態窗口（protocol 列已 FOR UPDATE，此處補鎖 animal 列）。
        if req.to_status == ProtocolStatus::Closed {
            let alive_count: i64 = sqlx::query_scalar(
                "WITH locked AS ( \
                     SELECT id FROM animals \
                     WHERE deleted_at IS NULL \
                       AND status NOT IN ($2, $3, $4) \
                       AND (iacuc_no = $1 OR reserved_protocol_id = $5) \
                     FOR UPDATE \
                 ) SELECT COUNT(*) FROM locked",
            )
            .bind(protocol.iacuc_no.as_deref())
            .bind(AnimalStatus::Euthanized)
            .bind(AnimalStatus::SuddenDeath)
            .bind(AnimalStatus::Transferred)
            .bind(id)
            .fetch_one(&mut **tx)
            .await?;

            if alive_count > 0 {
                return Err(AppError::BusinessRule(format!(
                    "計畫下仍有 {alive_count} 隻存活動物（含已預約），須全部完成犧牲（安樂死）、\
                     猝死或轉讓、或先解除預約後才能結案"
                )));
            }
        }

        // IACUC 編號生成規則：在 tx 內使用 _tx 版本（確保 advisory lock 同 tx 提交）
        let changed_by = actor
            .actor_user_id()
            .unwrap_or(crate::middleware::SYSTEM_USER_ID);

        let new_iacuc_no = if req.to_status == ProtocolStatus::Submitted {
            let needs_apig = protocol
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);

            if needs_apig {
                Some(Self::generate_apig_no(&mut *tx).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else if req.to_status == ProtocolStatus::PreReview {
            let needs_apig = protocol
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("APIG-"))
                .unwrap_or(true);

            if needs_apig {
                Some(Self::generate_apig_no(&mut *tx).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else if req.to_status == ProtocolStatus::Approved
            || req.to_status == ProtocolStatus::ApprovedWithConditions
        {
            // 暫停復原（SUSPENDED → APPROVED/APPROVED_WITH_CONDITIONS）沿用既有的
            // PIG- 編號，不重新生成——該編號已綁定動物 iacuc_no 與客戶代碼，
            // 重新生成會讓既有綁定變成孤兒。
            let needs_pig_no = protocol
                .iacuc_no
                .as_ref()
                .map(|no| !no.starts_with("PIG-"))
                .unwrap_or(true);

            if needs_pig_no {
                Some(Self::generate_iacuc_no(&mut *tx).await?)
            } else {
                protocol.iacuc_no.clone()
            }
        } else {
            protocol.iacuc_no.clone()
        };

        // 更新計畫狀態
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
        .fetch_one(&mut **tx)
        .await?;

        // UnderReview 時預先撈一次 reviewer (id, name)，同時供 status_remark 與後續活動記錄使用，避免重複查詢。
        let reviewer_info: Option<Vec<(Uuid, String)>> = if req.to_status
            == ProtocolStatus::UnderReview
        {
            if let Some(reviewer_ids) = &req.reviewer_ids {
                let info: Vec<(Uuid, String)> = sqlx::query_as(
                        "SELECT id, COALESCE(display_name, email) FROM users WHERE id = ANY($1::uuid[])",
                    )
                    .bind(reviewer_ids)
                    .fetch_all(&mut **tx)
                    .await?;
                Some(info)
            } else {
                None
            }
        } else {
            None
        };

        // 記錄狀態變更（tx 版本）
        let status_remark = if let Some(info) = reviewer_info.as_ref() {
            let reviewer_list = info
                .iter()
                .map(|(_, n)| n.as_str())
                .collect::<Vec<_>>()
                .join("、");
            Some(format!("指派審查委員：{}", reviewer_list))
        } else {
            req.remark.clone()
        };

        Self::record_status_change_tx(
            &mut *tx,
            actor,
            id,
            Some(protocol.status),
            req.to_status,
            status_remark,
        )
        .await?;

        // PR #269 Option C：record_activity_tx 不再寫 user_activity_logs，
        // 由呼叫端負責補 audit。此處附 before/after diff 供 HMAC chain。
        let status_event_type = event_type_for(activity_type_for_status(req.to_status));
        AuditService::log_activity_tx(
            &mut *tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: status_event_type,
                entity: Some(AuditEntity::new("protocol", id, &updated.title)),
                data_diff: Some(DataDiff::compute(Some(&protocol), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        // 當狀態變為 UNDER_REVIEW 時，自動指派選定的審查委員（tx 版本）
        if let Some(info) = reviewer_info {
            if let Some(reviewer_ids) = &req.reviewer_ids {
                for reviewer_id in reviewer_ids {
                    Self::assign_primary_reviewer_tx(&mut *tx, id, *reviewer_id, changed_by)
                        .await?;
                }

                let extra = serde_json::json!({
                    "reviewers": info.iter().map(|(rid, name)| {
                        serde_json::json!({"id": rid, "name": name})
                    }).collect::<Vec<_>>()
                });

                let reviewer_names: Vec<&str> = info.iter().map(|(_, n)| n.as_str()).collect();
                Self::record_activity_tx(
                    &mut *tx,
                    actor,
                    id,
                    ProtocolActivityType::ReviewerAssigned,
                    None,
                    Some(format!("指派 {} 位審查委員", reviewer_ids.len())),
                    None,
                    Some(format!("審查委員：{}", reviewer_names.join("、"))),
                    Some(extra),
                )
                .await?;

                // PR #269 Option C：補 audit log（無 diff 的 timeline 事件）
                AuditService::log_activity_tx(
                    &mut *tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "AUP",
                        event_type: event_type_for(ProtocolActivityType::ReviewerAssigned),
                        entity: Some(AuditEntity::new("protocol", id, &updated.title)),
                        data_diff: None,
                        request_context: None,
                    },
                )
                .await?;
            }
        }

        // 當狀態變為 VET_REVIEW 時，自動指派獸醫師（tx 版本，含 audit 記錄）
        if req.to_status == ProtocolStatus::VetReview {
            Self::assign_vet_reviewer_tx(&mut *tx, actor, id, req.vet_id).await?;
        }

        // 當計劃通過時，自動依照 IACUC No. 創建客戶（tx 版本，原子操作）
        if req.to_status == ProtocolStatus::Approved
            || req.to_status == ProtocolStatus::ApprovedWithConditions
        {
            if let Some(iacuc_no) = new_iacuc_no.as_ref() {
                // 檢查是否已存在該客戶（客戶代碼 = IACUC No.）
                let existing_customer: Option<uuid::Uuid> = sqlx::query_scalar(
                    "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1",
                )
                .bind(iacuc_no)
                .fetch_optional(&mut **tx)
                .await?;

                // 如果不存在，則創建新客戶（同 tx 內）
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

                    if let Err(validation_errors) = create_req.validate() {
                        tracing::warn!(
                            "Failed to validate customer creation request for IACUC {}: {:?}",
                            iacuc_no,
                            validation_errors
                        );
                    } else {
                        let actor_for_customer = ActorContext::System {
                            reason: "auto_create_customer_from_iacuc",
                        };
                        if let Err(e) =
                            PartnerService::create_tx(&mut *tx, &actor_for_customer, &create_req)
                                .await
                        {
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

        // 當計劃結案時，自動停用對應的客戶（tx 版本）
        if req.to_status == ProtocolStatus::Closed {
            if let Some(iacuc_no) = protocol.iacuc_no.as_ref() {
                let customer_id: Option<uuid::Uuid> = sqlx::query_scalar(
                    "SELECT id FROM partners WHERE partner_type = 'customer' AND code = $1",
                )
                .bind(iacuc_no)
                .fetch_optional(&mut **tx)
                .await?;

                if let Some(customer_id) = customer_id {
                    let result = sqlx::query(
                        "UPDATE partners SET is_active = false, updated_at = NOW() WHERE id = $1",
                    )
                    .bind(customer_id)
                    .execute(&mut **tx)
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

        Ok(updated)
    }

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

        // 驗證計畫類型 (AUP 2.7)：複選陣列（新）或單一字串（舊資料相容）皆接受。
        let project_type_filled = match basic.get("project_type") {
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::String(s)) => !s.trim().is_empty(),
            _ => false,
        };
        if !project_type_filled {
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
    pub async fn submit(pool: &PgPool, actor: &ActorContext, id: Uuid) -> Result<Protocol> {
        // 送出計畫必須由真實登入使用者觸發（不可 System / Anonymous）
        let _user = actor.require_user()?;

        let mut tx = pool.begin().await?;

        // 讀取 before 狀態（含權限/狀態轉移檢查）
        let before =
            sqlx::query_as::<_, Protocol>("SELECT * FROM protocols WHERE id = $1 FOR UPDATE")
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

        // 須知簽署門檻（PR-B）：須知為「初次送審」前的一次性簽署，故僅於 DRAFT→SUBMITTED 檢查。
        // 補件重送（*_REVISION_REQUIRED）不再要求——`acknowledge_notice` 僅允許 DRAFT 簽署、
        // 簽署卡片亦只在 DRAFT 顯示，若在補件狀態仍檢查會造成「要簽卻無法簽」的死鎖。
        // 匯入舊計劃走 import_approved（直接進 APPROVED、不經此 submit），故不受此限。
        if before.status == ProtocolStatus::Draft {
            if let Some(active) =
                crate::repositories::application_notice::ApplicationNoticeRepository::find_active(
                    pool,
                )
                .await?
            {
                let signed = crate::repositories::application_notice::NoticeAcknowledgementRepository::find_by_protocol(pool, id)
                    .await?
                    .map(|ack| ack.notice_id == active.id)
                    .unwrap_or(false);
                if !signed {
                    return Err(AppError::BadRequest(
                        "尚未簽署最新版動物試驗申請須知".to_string(),
                    ));
                }
            }
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

        // protocol_activities 時間軸（PR #269 Option C 後此處不再寫 user_activity_logs）
        Self::record_status_change_tx(&mut tx, actor, id, Some(before.status), new_status, None)
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
    /// 變更計畫狀態（pool-based wrapper，開啟 tx 後委派至 change_status_tx）
    pub async fn change_status(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &ChangeStatusRequest,
    ) -> Result<Protocol> {
        let mut tx = pool.begin().await?;
        let result = Self::change_status_tx(&mut tx, actor, id, req).await?;
        tx.commit().await?;
        Ok(result)
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

    #[test]
    fn test_validate_content_accepts_project_type_array() {
        // 複選改版：project_type 為非空陣列應通過提交驗證
        let content = json!({
            "basic": {
                "study_title": "Valid Study",
                "is_glp": false,
                "project_type": ["1_basic_research", "5_biologics_manufacturing"]
            }
        });
        assert!(ProtocolService::validate_protocol_content(&Some(content)).is_ok());
    }

    #[test]
    fn test_validate_content_rejects_empty_project_type_array() {
        let content = json!({
            "basic": {
                "study_title": "Valid Study",
                "is_glp": false,
                "project_type": []
            }
        });
        let result = ProtocolService::validate_protocol_content(&Some(content));
        assert!(result.is_err());
        assert!(result
            .expect_err("should return error")
            .to_string()
            .contains("Project type is required"));
    }
}
