use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, Amendment, AmendmentReviewAssignment, AmendmentStatus, AmendmentType,
        ChangeAmendmentStatusRequest, ClassifyAmendmentRequest, MarkAmendmentEffectiveRequest,
        RecordAmendmentDecisionRequest,
    },
    services::{
        access::{AmendmentWrite, Scoped},
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, SignatureService,
    },
    AppError, Result,
};

use super::AmendmentService;

// ============================================================
// 常數（CodeRabbit review #205：抽出魔術字串）
// ============================================================

/// electronic_signatures.entity_type for amendment 決定簽章
const AMENDMENT_ENTITY_TYPE: &str = "amendment";

/// signature_method = 'internal' 表示非密碼/手寫驗證的系統觸發簽章
/// （與 SignatureService::sign 的 'password' / 'handwriting' 區別）
const SIGNATURE_METHOD_INTERNAL: &str = "internal";

const DECISION_APPROVE: &str = "APPROVE";
const DECISION_REJECT: &str = "REJECT";
const DECISION_REVISION: &str = "REVISION";

/// content / signature_input 用 `|` 分隔欄位（避免 `:` 與 decision_summary
/// 中的 `:` 衝突 — Gemini review #205）
const FIELD_DELIMITER: char = '|';

/// 合法的審查決定值
const VALID_DECISIONS: [&str; 3] = [DECISION_APPROVE, DECISION_REJECT, DECISION_REVISION];

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
    let entity_id = amendment_id.to_string();
    let decision_word = if is_approve {
        DECISION_APPROVE
    } else {
        DECISION_REJECT
    };
    // content = canonical 描述（決定時刻的快照），可用 SignatureService::verify
    // 驗證未被竄改。`|` 分隔避免與 decision_summary 內可能含的 `:` 衝突。
    let content = format!(
        "amendment_decision{d}{}{d}{}{d}{}",
        entity_id,
        decision_word,
        decision_summary,
        d = FIELD_DELIMITER
    );
    let content_hash = SignatureService::compute_hash(&content);
    let timestamp = Utc::now();
    // High-1 (#205/#213)：改用 HMAC-SHA256 v2 簽章（與 password/handwriting 路徑一致），
    // 取代原 plain SHA-256（v1，任何拿到 content_hash+timestamp+method 者可重算偽造）。
    // hash_input 用 SIGNATURE_METHOD_INTERNAL 維持與舊 input 的語意連續；
    // 防偽來自 AUDIT_HMAC_KEY 而非 hash_input 機密性。
    let (signature_data, hmac_version) = SignatureService::build_signature_data_v2(
        signer_id,
        &content_hash,
        timestamp.timestamp(),
        SIGNATURE_METHOD_INTERNAL,
    )?;

    // R30-10：amendment 終態決定（APPROVE / REJECT）皆為「行使審查/核准權限」的
    // 簽章行為，meaning 對齊 §11.50(a)(3) "approval" → SignatureMeaning::Approve。
    // 改用 runtime sqlx::query_scalar（非 macro）以避免 signature_meaning ENUM
    // 加入後 .sqlx offline cache 需重生的循環依賴。
    let sig_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO electronic_signatures (
            entity_type, entity_id, signer_id, signature_type,
            content_hash, signature_data, signature_method, meaning, hmac_version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::signature_meaning, $9)
        RETURNING id
        "#,
    )
    .bind(AMENDMENT_ENTITY_TYPE)
    .bind(&entity_id)
    .bind(signer_id)
    .bind(decision_word)
    .bind(&content_hash)
    .bind(&signature_data)
    .bind(SIGNATURE_METHOD_INTERNAL)
    .bind("APPROVE")
    .bind(hmac_version)
    .fetch_one(&mut **tx)
    .await?;

    Ok(sig_id)
}

/// C2 helper：終態決定的簽章相關參數封裝（CodeRabbit review #205：原 6 參數
/// 違反 ≤5 上限，抽 struct 讓呼叫端意圖更明確）。
pub(super) struct TerminalDecisionContext<'a> {
    pub signer_id: Uuid,
    pub is_approve: bool,
    pub decision_summary: &'a str,
}

/// C2 helper：將終態（APPROVED 或 REJECTED）的決定流程收斂為單一函式，
/// 由 [`check_all_decisions_tx`] 兩個分支共用，避免邏輯重複。
///
/// 流程：建立 electronic_signatures → UPDATE amendments 的 status + 對應簽章 FK
/// → record_status_change → 回傳新簽章 UUID。
async fn apply_terminal_decision_tx(
    tx: &mut Transaction<'_, Postgres>,
    actor: &ActorContext,
    amendment_id: Uuid,
    ctx: TerminalDecisionContext<'_>,
    current_status: AmendmentStatus,
    amendment_no: &str,
) -> Result<Uuid> {
    let sig_id = insert_decision_signature_tx(
        tx,
        amendment_id,
        ctx.signer_id,
        ctx.is_approve,
        ctx.decision_summary,
    )
    .await?;

    // 用 conditional column 寫入避免兩條幾乎一樣的 SQL（COALESCE 對 UUID 型別 OK）
    let (new_status, history_remark) = if ctx.is_approve {
        sqlx::query!(
            r#"UPDATE amendments
               SET status = 'APPROVED', approved_signature_id = $2, updated_at = NOW()
               WHERE id = $1"#,
            amendment_id,
            sig_id
        )
        .execute(&mut **tx)
        .await?;
        (
            AmendmentStatus::Approved,
            format!("全體審查委員核准（簽章 {sig_id}）"),
        )
    } else {
        sqlx::query!(
            r#"UPDATE amendments
               SET status = 'REJECTED', rejected_signature_id = $2, updated_at = NOW()
               WHERE id = $1"#,
            amendment_id,
            sig_id
        )
        .execute(&mut **tx)
        .await?;
        (
            AmendmentStatus::Rejected,
            format!("審查委員否決（簽章 {sig_id}）"),
        )
    };

    AmendmentService::record_status_change(
        &mut **tx,
        amendment_id,
        Some(current_status),
        new_status,
        ctx.signer_id,
        Some(history_remark),
    )
    .await?;

    // R71-4：終態核准/否決補寫 user_activity_logs / HMAC chain（原僅寫簽章表 + history，
    // 在 user audit 軸上隱形；對比 mark_effective 已有 audit）。display 用 amendment_no
    // 與 classify / REVISION 一致（gemini #729）。
    AuditService::log_activity_tx(
        tx,
        actor,
        ActivityLogEntry {
            event_category: "AUP",
            event_type: if ctx.is_approve {
                "AMENDMENT_APPROVE"
            } else {
                "AMENDMENT_REJECT"
            },
            entity: Some(AuditEntity::new(
                AMENDMENT_ENTITY_TYPE,
                amendment_id,
                amendment_no,
            )),
            data_diff: None,
            request_context: None,
        },
    )
    .await?;

    Ok(sig_id)
}

impl AmendmentService {
    /// 提交變更申請
    pub async fn submit(
        pool: &PgPool,
        scope: Scoped<AmendmentWrite>,
        id: Uuid,
        submitted_by: Uuid,
    ) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;
        ensure_amendment_scope(&current, &scope)?;
        ensure_live_amendment(&current)?;

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
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
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
        let remark = if is_resubmit {
            "變更申請已重送"
        } else {
            "變更申請已提交"
        };
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
    ///
    /// C2 (GLP)：Minor → ADMIN_APPROVED 終態於同 tx 內建立 admin 簽章 + 回填 FK。
    /// Major → CLASSIFIED 後 reviewer 指派也在同 tx 內，避免 reviewer 寫入失敗時
    /// amendment 已成 CLASSIFIED 但無 reviewer 的不一致狀態（Gemini review #205, High）。
    ///
    /// R27-7：原本 ~110 行（>50 規範）；現抽 minor / major 兩個 helper，
    /// `classify` 主函式僅做驗證 + tx 邊界 + 分流。
    pub async fn classify(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &ClassifyAmendmentRequest,
    ) -> Result<Amendment> {
        // R71-4：改吃 ActorContext，分類者供終態 audit 歸因。
        let classified_by = actor.require_user()?.id;
        let current = Self::get_by_id_raw(pool, id).await?;
        ensure_live_amendment(&current)?;

        // 只有已提交或已重送的申請可以分類
        if current.status != AmendmentStatus::Submitted
            && current.status != AmendmentStatus::Resubmitted
        {
            return Err(AppError::BadRequest(
                "Only submitted amendments can be classified".into(),
            ));
        }

        // 不能分類為待分類
        if req.amendment_type == AmendmentType::Pending {
            return Err(AppError::BadRequest("Cannot classify as PENDING".into()));
        }

        let mut tx = pool.begin().await?;

        let amendment = match req.amendment_type {
            AmendmentType::Minor => {
                Self::classify_minor_with_signature_tx(
                    &mut tx,
                    actor,
                    id,
                    req,
                    classified_by,
                    &current,
                )
                .await?
            }
            AmendmentType::Major => {
                Self::classify_major_with_reviewers_tx(
                    &mut tx,
                    actor,
                    id,
                    req,
                    classified_by,
                    &current,
                )
                .await?
            }
            AmendmentType::Pending => unreachable!(),
        };

        tx.commit().await?;
        Ok(amendment)
    }

    /// R27-7 helper：Minor 分類路徑 — 終態 ADMIN_APPROVED + classifier 簽章 + history。
    async fn classify_minor_with_signature_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        id: Uuid,
        req: &ClassifyAmendmentRequest,
        classified_by: Uuid,
        current: &Amendment,
    ) -> Result<Amendment> {
        let summary = format!("admin_approved|type=MINOR|classifier={classified_by}");
        let sig_id = insert_decision_signature_tx(tx, id, classified_by, true, &summary).await?;

        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET
                amendment_type = 'MINOR'::amendment_type,
                status = 'ADMIN_APPROVED'::amendment_status,
                classified_by = $2,
                classified_at = NOW(),
                classification_remark = $3,
                approved_signature_id = $4,
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
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
            "#,
            id,
            classified_by,
            req.remark,
            sig_id,
        )
        .fetch_one(&mut **tx)
        .await?;

        let history_remark = Some(format!(
            "{}（行政核准簽章 {sig_id}）",
            req.remark.as_deref().unwrap_or("")
        ));
        Self::record_status_change(
            &mut **tx,
            id,
            Some(current.status),
            AmendmentStatus::AdminApproved,
            classified_by,
            history_remark,
        )
        .await?;

        // R71-4：Minor 分類即終態（ADMIN_APPROVED + 行政核准簽章），補 audit chain。
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "AMENDMENT_CLASSIFY_MINOR",
                entity: Some(AuditEntity::new(
                    AMENDMENT_ENTITY_TYPE,
                    id,
                    &amendment.amendment_no,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        Ok(amendment)
    }

    /// R27-7 helper：Major 分類路徑 — CLASSIFIED + reviewer 指派 + history。
    /// Gemini #205 High：reviewer 指派與 status UPDATE 同 tx，避免孤兒狀態。
    async fn classify_major_with_reviewers_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        id: Uuid,
        req: &ClassifyAmendmentRequest,
        classified_by: Uuid,
        current: &Amendment,
    ) -> Result<Amendment> {
        let amendment = sqlx::query_as!(
            Amendment,
            r#"
            UPDATE amendments
            SET
                amendment_type = 'MAJOR'::amendment_type,
                status = 'CLASSIFIED'::amendment_status,
                classified_by = $2,
                classified_at = NOW(),
                classification_remark = $3,
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
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
            "#,
            id,
            classified_by,
            req.remark,
        )
        .fetch_one(&mut **tx)
        .await?;

        Self::record_status_change(
            &mut **tx,
            id,
            Some(current.status),
            AmendmentStatus::Classified,
            classified_by,
            req.remark.clone(),
        )
        .await?;

        Self::assign_reviewers_from_protocol_tx(tx, id, current.protocol_id, classified_by).await?;

        // R71-4：Major 分類（CLASSIFIED + 指派審查委員）補 audit chain。
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: "AMENDMENT_CLASSIFY_MAJOR",
                entity: Some(AuditEntity::new(
                    AMENDMENT_ENTITY_TYPE,
                    id,
                    &amendment.amendment_no,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        Ok(amendment)
    }

    /// 從原計畫複製審查委員（pool 版，向後相容）
    pub async fn assign_reviewers_from_protocol(
        pool: &PgPool,
        amendment_id: Uuid,
        protocol_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignment>> {
        Self::assign_reviewers_inner(pool, amendment_id, protocol_id, assigned_by).await
    }

    /// 從原計畫複製審查委員（tx 版，由 classify 使用以保證原子性）
    async fn assign_reviewers_from_protocol_tx(
        tx: &mut Transaction<'_, Postgres>,
        amendment_id: Uuid,
        protocol_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignment>> {
        Self::assign_reviewers_inner(&mut **tx, amendment_id, protocol_id, assigned_by).await
    }

    /// 內部：批量 INSERT 審查委員（避免 N+1），接受任何 Executor。
    async fn assign_reviewers_inner<'e, E>(
        executor: E,
        amendment_id: Uuid,
        protocol_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<Vec<AmendmentReviewAssignment>>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
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
        .fetch_all(executor)
        .await?;

        Ok(assignments)
    }

    /// 開始審查（變更狀態為 UNDER_REVIEW）
    pub async fn start_review(pool: &PgPool, id: Uuid, changed_by: Uuid) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;
        ensure_live_amendment(&current)?;

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
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
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
    /// C2 (GLP)：record_decision、終態守衛、assignment UPDATE、聚合決定
    /// (check_all_decisions_tx) + 終態簽章寫入皆於同一 tx 完成，確保 audit
    /// trail 與決定簽章不被部分寫入。
    ///
    /// **終態守衛（CodeRabbit review #205 R7 / PR #213）**：
    /// 已 APPROVED / REJECTED / ADMIN_APPROVED 的 amendment 不可再被 reviewer
    /// 改決定，避免：
    /// 1. status 翻轉（已核准 → 強制改決定 → 跑 check_all_decisions → 變 REJECTED）
    /// 2. approved_signature_id / rejected_signature_id 被覆寫
    /// 3. 違反 21 CFR §11.10(e) 「audit trail 不得遮蔽先前記錄」
    ///
    /// 守衛用 SELECT FOR UPDATE 在 tx 內鎖定 amendment row，避免 TOCTOU
    /// 與並發 record_decision 同時通過守衛後皆寫入。
    pub async fn record_decision(
        pool: &PgPool,
        actor: &ActorContext,
        amendment_id: Uuid,
        req: &RecordAmendmentDecisionRequest,
    ) -> Result<AmendmentReviewAssignment> {
        // R71-4：改吃 ActorContext（拒 Anonymous），reviewer 即操作者，供終態 audit 歸因。
        let reviewer_id = actor.require_user()?.id;

        // 驗證決定值
        if !VALID_DECISIONS.contains(&req.decision.as_str()) {
            return Err(AppError::BadRequest("Invalid decision value".into()));
        }

        let mut tx = pool.begin().await?;

        // 終態守衛：在 tx 內 SELECT FOR UPDATE 鎖定 amendment row，避免並發
        // record_decision 同時通過守衛後皆寫入（PR #213 R7）。
        // R71-4：一併取 amendment_no 供終態 audit 的 entity_display_name（gemini #729，
        // 與 classify 路徑一致、利於人工檢視）。runtime query_as 與本檔 change_status 的
        // FOR UPDATE 取法一致（免 sqlx offline cache 重建）。
        let (current_status, amendment_no): (AmendmentStatus, String) =
            sqlx::query_as("SELECT status, amendment_no FROM amendments WHERE id = $1 FOR UPDATE")
                .bind(amendment_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Amendment not found".into()))?;

        if matches!(
            current_status,
            AmendmentStatus::Approved
                | AmendmentStatus::Rejected
                | AmendmentStatus::AdminApproved
                | AmendmentStatus::Effective
        ) {
            return Err(AppError::Conflict(
                "Amendment 已進入終態，不可再記錄審查決定".into(),
            ));
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
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Review assignment not found".into()))?;

        // 檢查所有審查委員是否都已完成決定，傳入 reviewer_id 作為「最後 tipping」
        // 簽章主體（PR #205 C2）。守衛已防止已終態 amendment 走到此處，所以新
        // 終態只會是首次寫入。
        // R27-9：current_status 已在守衛 SELECT FOR UPDATE 取得，傳入避免重複查詢。
        Self::check_all_decisions_tx(
            &mut tx,
            actor,
            amendment_id,
            reviewer_id,
            current_status,
            &amendment_no,
        )
        .await?;

        tx.commit().await?;
        Ok(assignment)
    }

    /// 檢查所有審查委員是否都已完成決定，並自動更新狀態（tx 版）
    ///
    /// C2 (GLP)：終態（APPROVED/REJECTED）由 [`apply_terminal_decision_tx`] 統一
    /// 處理 — 同 tx 內建立 electronic_signatures 並回填 amendments.{approved,
    /// rejected}_signature_id（21 CFR §11.50/§11.70 非否認性）。
    /// REVISION_REQUIRED 不簽章（非終態）。
    ///
    /// CodeRabbit review #205：原本 111 行（超過 ≤50 規範）+ APPROVED/REJECTED
    /// 邏輯重複；現抽 helper 後本函式僅做統計與分流。
    ///
    /// R27-9：`current_status` 由 caller 傳入（已在 record_decision 守衛
    /// SELECT FOR UPDATE 取得），避免同 tx 內重複查 amendments.status。
    async fn check_all_decisions_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        amendment_id: Uuid,
        tipping_reviewer_id: Uuid,
        current_status: AmendmentStatus,
        amendment_no: &str,
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

        let summary = format!(
            "total={}|approved={}|rejected={}|revision={}",
            stats.total, stats.approved, stats.rejected, stats.revision
        );

        if stats.revision > 0 {
            // 非終態，不簽章
            sqlx::query!(
                r#"UPDATE amendments SET status = 'REVISION_REQUIRED', updated_at = NOW() WHERE id = $1"#,
                amendment_id
            )
            .execute(&mut **tx)
            .await?;

            // R71-4：REVISION 歷程歸因改用觸發決定的審查委員（原誤用 SYSTEM_USER_ID，
            // 歸因失真——非系統觸發，而是該名 reviewer 投下 REVISION 才翻終態）。
            Self::record_status_change(
                &mut **tx,
                amendment_id,
                Some(current_status),
                AmendmentStatus::RevisionRequired,
                tipping_reviewer_id,
                Some("審查委員要求修訂".to_string()),
            )
            .await?;

            // R71-4：REVISION 決議補 audit chain（與 APPROVE/REJECT 一致）。
            AuditService::log_activity_tx(
                tx,
                actor,
                ActivityLogEntry {
                    event_category: "AUP",
                    event_type: "AMENDMENT_REVISION_REQUIRED",
                    entity: Some(AuditEntity::new(
                        AMENDMENT_ENTITY_TYPE,
                        amendment_id,
                        amendment_no,
                    )),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await?;
        } else if stats.rejected > 0 {
            apply_terminal_decision_tx(
                tx,
                actor,
                amendment_id,
                TerminalDecisionContext {
                    signer_id: tipping_reviewer_id,
                    is_approve: false,
                    decision_summary: &summary,
                },
                current_status,
                amendment_no,
            )
            .await?;
        } else if stats.approved == stats.total {
            apply_terminal_decision_tx(
                tx,
                actor,
                amendment_id,
                TerminalDecisionContext {
                    signer_id: tipping_reviewer_id,
                    is_approve: true,
                    decision_summary: &summary,
                },
                current_status,
                amendment_no,
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
        // R30-25a：EFFECTIVE 終態必須由專屬 service 寫入（同步 effective_from 與簽章），
        // 避免泛型 change_status 端點建出 status='EFFECTIVE' AND effective_from=NULL
        // 的不變式破洞。R30-25b/c 完成前一律拒絕。
        if req.to_status == AmendmentStatus::Effective {
            return Err(AppError::BadRequest(
                "EFFECTIVE 狀態僅能由生效流程寫入（R30-25b/c 待實作），不接受手動變更".into(),
            ));
        }

        // CSO #3：原本零轉移驗證 + 無鎖 → REJECTED/APPROVED 等終態可被退回 DRAFT/UNDER_REVIEW
        // 繞過 GLP change-control 審查流程。改為 tx 內 FOR UPDATE 取現態 + 驗證合法轉移。
        let mut tx = pool.begin().await?;

        let (current_status, is_historical) = sqlx::query_as::<_, (AmendmentStatus, bool)>(
            "SELECT status, is_historical FROM amendments WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("變更申請不存在".into()))?;

        // 互斥守衛：補登歷史變更不可走 live 狀態變更（DRAFT→SUBMITTED 等）
        if is_historical {
            return Err(AppError::BusinessRule(
                "補登歷史變更不可走 live 審查流程".into(),
            ));
        }

        if !current_status.can_transition_to(req.to_status) {
            return Err(AppError::BusinessRule(format!(
                "不允許的狀態轉移：{} → {}",
                current_status.display_name(),
                req.to_status.display_name()
            )));
        }

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
                approved_signature_id, rejected_signature_id,
                effective_from, version, is_historical
            "#,
            id,
            req.to_status.as_str(),
        )
        .fetch_one(&mut *tx)
        .await?;

        // R71-5：history 寫入移入同一 tx —— 原本在 tx.commit() 之後才用 pool 寫，
        // 留下「狀態已變、歷程遺失」窗口（commit 後 record_status_change 失敗 → 無 history）。
        Self::record_status_change(
            &mut *tx,
            id,
            Some(current_status),
            req.to_status,
            changed_by,
            req.remark.clone(),
        )
        .await?;

        tx.commit().await?;

        Ok(amendment)
    }

    /// R30-25b：將已核准的 amendment 標記為 EFFECTIVE（GLP §58 正式生效）。
    ///
    /// 守衛：
    /// - 來源狀態必須為 `Approved` 或 `AdminApproved`（其餘拒絕，避免跳過審查）
    /// - `effective_from` 須為 NULL（不可重複生效）
    /// - tx + SELECT FOR UPDATE 防 race（與並行 change_status 守衛同一致策略）
    ///
    /// 副作用（同 tx 原子）：
    /// - amendments.status / effective_from 寫入
    /// - amendment_status_history 寫入 from→to + remark
    /// - user_activity_logs 寫入 AMENDMENT_EFFECTIVE event（HMAC chain）
    pub async fn mark_effective(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &MarkAmendmentEffectiveRequest,
    ) -> Result<Amendment> {
        let user = actor.require_user()?;
        let changed_by = user.id;
        let mut tx = pool.begin().await?;

        let before = select_amendment_for_update_tx(&mut tx, id).await?;
        ensure_live_amendment(&before)?;
        validate_mark_effective_preconditions(&before)?;

        let new_effective: DateTime<Utc> = req.effective_from.unwrap_or_else(Utc::now);
        let after = update_amendment_to_effective_tx(&mut tx, id, new_effective).await?;

        Self::record_status_change(
            &mut *tx,
            id,
            Some(before.status),
            AmendmentStatus::Effective,
            changed_by,
            req.remark
                .clone()
                .or_else(|| Some(format!("標記為 EFFECTIVE（生效時點 {new_effective}）"))),
        )
        .await?;

        log_amendment_effective_tx(&mut tx, actor, id, &before, &after).await?;

        tx.commit().await?;
        Ok(after)
    }
}

// ============================================================
// R30-25b helpers — 把 mark_effective 主流程拆出（CLAUDE.md §2 ≤50 行 + DRY）
// ============================================================

/// 互斥守衛：補登歷史變更（is_historical）不可走 live 審查流程（submit / classify /
/// start_review / change_status / mark_effective）。歷史變更走 import_backfill 專屬路徑。
fn ensure_live_amendment(amendment: &Amendment) -> Result<()> {
    if amendment.is_historical {
        return Err(AppError::Forbidden(
            "補登歷史變更不可走 live 審查流程".into(),
        ));
    }
    Ok(())
}

/// 守衛：`Scoped<AmendmentWrite>` 證明須對應本變更申請所屬計畫。
///
/// update / submit 等 id-keyed 操作的證明是「對某計畫的 PI 寫入授權」，此處綁定
/// amendment id ↔ 已授權 protocol，使型別層保證（持有證明 ⟺ 已授權）對 id-keyed
/// 操作仍成立——防呼叫端以計畫 A 的證明寫入屬於計畫 B 的變更申請。
pub(super) fn ensure_amendment_scope(
    amendment: &Amendment,
    scope: &Scoped<AmendmentWrite>,
) -> Result<()> {
    if amendment.protocol_id != scope.id() {
        return Err(AppError::Forbidden("變更申請不屬於已授權的計畫".into()));
    }
    Ok(())
}

/// SELECT FOR UPDATE 鎖 row：避免並行 change_status / mark_effective 互踩。
/// 用 runtime query_as（非 macro）：本 PR 新增的 query 尚未進 .sqlx 離線 cache，
/// 且本機開發環境未直連 DB；待後續 PR + DB 連線時統一遷移為 macro。
async fn select_amendment_for_update_tx(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Amendment> {
    sqlx::query_as::<_, Amendment>(
        r#"
        SELECT
            id, protocol_id, amendment_no, revision_number,
            amendment_type, status,
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
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Amendment not found".into()))
}

/// 守衛：source status 必須為 APPROVED / ADMIN_APPROVED + effective_from 須為 NULL。
fn validate_mark_effective_preconditions(before: &Amendment) -> Result<()> {
    if !matches!(
        before.status,
        AmendmentStatus::Approved | AmendmentStatus::AdminApproved
    ) {
        return Err(AppError::BadRequest(format!(
            "僅 APPROVED / ADMIN_APPROVED 狀態可標記為 EFFECTIVE（目前:{}）",
            before.status.as_str()
        )));
    }
    if before.effective_from.is_some() {
        return Err(AppError::Conflict("amendment 已生效，不可重複標記".into()));
    }
    Ok(())
}

/// CAS UPDATE：WHERE 帶 status ∈ {APPROVED, ADMIN_APPROVED} + effective_from IS NULL
/// defense-in-depth — FOR UPDATE 已鎖 row 防同 tx 併發；CAS 在 SQL 層額外擋
/// future race / planner 重排 / 跨 tx 競爭，0 row affected 時回 Conflict。
async fn update_amendment_to_effective_tx(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    new_effective: DateTime<Utc>,
) -> Result<Amendment> {
    sqlx::query_as::<_, Amendment>(
        r#"
        UPDATE amendments
        SET
            status = 'EFFECTIVE'::amendment_status,
            effective_from = $2,
            updated_at = NOW()
        WHERE id = $1
          AND status IN ('APPROVED'::amendment_status, 'ADMIN_APPROVED'::amendment_status)
          AND effective_from IS NULL
        RETURNING
            id, protocol_id, amendment_no, revision_number,
            amendment_type, status,
            title, description, change_items,
            changes_content, submitted_by, submitted_at,
            classified_by, classified_at, classification_remark,
            created_by, created_at, updated_at,
            approved_signature_id, rejected_signature_id,
            effective_from, version, is_historical
        "#,
    )
    .bind(id)
    .bind(new_effective)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::Conflict("Amendment 狀態於 tx 內被併發改動，請重試".into()))
}

/// 寫 AMENDMENT_EFFECTIVE event 進 HMAC chain（同 tx 原子）。
async fn log_amendment_effective_tx(
    tx: &mut Transaction<'_, Postgres>,
    actor: &ActorContext,
    id: Uuid,
    before: &Amendment,
    after: &Amendment,
) -> Result<()> {
    let display = format!("{} — {}", after.amendment_no, after.title);
    AuditService::log_activity_tx(
        tx,
        actor,
        ActivityLogEntry {
            event_category: "AUP",
            event_type: "AMENDMENT_EFFECTIVE",
            entity: Some(AuditEntity::new("amendment", id, &display)),
            data_diff: Some(DataDiff::compute(Some(before), Some(after))),
            request_context: None,
        },
    )
    .await?;
    Ok(())
}
