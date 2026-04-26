use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    models::{
        Amendment, AmendmentReviewAssignment, AmendmentStatus,
        AmendmentType, ChangeAmendmentStatusRequest,
        ClassifyAmendmentRequest, RecordAmendmentDecisionRequest,
    },
    services::SignatureService,
    AppError, Result, SYSTEM_USER_ID,
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
    let signature_input = format!(
        "{}{d}{}{d}{}{d}{}",
        signer_id,
        content_hash,
        timestamp.timestamp(),
        SIGNATURE_METHOD_INTERNAL,
        d = FIELD_DELIMITER
    );
    let signature_data = SignatureService::compute_hash(&signature_input);

    // CodeRabbit review #205：改用 query_scalar! macro 啟用 SQLX_OFFLINE 編譯期型別檢查，
    // 與檔案內其他查詢一致，避免 schema 漂移時 CI 漏掉。
    let sig_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO electronic_signatures (
            entity_type, entity_id, signer_id, signature_type,
            content_hash, signature_data, signature_method
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        AMENDMENT_ENTITY_TYPE,
        entity_id,
        signer_id,
        decision_word,
        content_hash,
        signature_data,
        SIGNATURE_METHOD_INTERNAL,
    )
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
    amendment_id: Uuid,
    ctx: TerminalDecisionContext<'_>,
    current_status: AmendmentStatus,
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
    ///
    /// C2 (GLP)：Minor → ADMIN_APPROVED 終態於同 tx 內建立 admin 簽章 + 回填 FK。
    /// Major → CLASSIFIED 後 reviewer 指派也在同 tx 內，避免 reviewer 寫入失敗時
    /// amendment 已成 CLASSIFIED 但無 reviewer 的不一致狀態（Gemini review #205, High）。
    ///
    /// R27-7：原本 ~110 行（>50 規範）；現抽 minor / major 兩個 helper，
    /// `classify` 主函式僅做驗證 + tx 邊界 + 分流。
    pub async fn classify(
        pool: &PgPool,
        id: Uuid,
        req: &ClassifyAmendmentRequest,
        classified_by: Uuid,
    ) -> Result<Amendment> {
        let current = Self::get_by_id_raw(pool, id).await?;

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
            return Err(AppError::BadRequest(
                "Cannot classify as PENDING".into(),
            ));
        }

        let mut tx = pool.begin().await?;

        let amendment = match req.amendment_type {
            AmendmentType::Minor => {
                Self::classify_minor_with_signature_tx(&mut tx, id, req, classified_by, &current)
                    .await?
            }
            AmendmentType::Major => {
                Self::classify_major_with_reviewers_tx(&mut tx, id, req, classified_by, &current)
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
                approved_signature_id, rejected_signature_id
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

        Ok(amendment)
    }

    /// R27-7 helper：Major 分類路徑 — CLASSIFIED + reviewer 指派 + history。
    /// Gemini #205 High：reviewer 指派與 status UPDATE 同 tx，避免孤兒狀態。
    async fn classify_major_with_reviewers_tx(
        tx: &mut Transaction<'_, Postgres>,
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
                approved_signature_id, rejected_signature_id
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

        Self::assign_reviewers_from_protocol_tx(tx, id, current.protocol_id, classified_by)
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
        amendment_id: Uuid,
        reviewer_id: Uuid,
        req: &RecordAmendmentDecisionRequest,
    ) -> Result<AmendmentReviewAssignment> {
        // 驗證決定值
        if !VALID_DECISIONS.contains(&req.decision.as_str()) {
            return Err(AppError::BadRequest("Invalid decision value".into()));
        }

        let mut tx = pool.begin().await?;

        // 終態守衛：在 tx 內 SELECT FOR UPDATE 鎖定 amendment row，避免並發
        // record_decision 同時通過守衛後皆寫入（PR #213 R7）。
        let current_status: AmendmentStatus = sqlx::query_scalar!(
            r#"SELECT status as "status: AmendmentStatus"
               FROM amendments WHERE id = $1 FOR UPDATE"#,
            amendment_id
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Amendment not found".into()))?;

        if matches!(
            current_status,
            AmendmentStatus::Approved
                | AmendmentStatus::Rejected
                | AmendmentStatus::AdminApproved
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
        Self::check_all_decisions_tx(&mut tx, amendment_id, reviewer_id, current_status).await?;

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
        amendment_id: Uuid,
        tipping_reviewer_id: Uuid,
        current_status: AmendmentStatus,
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

            Self::record_status_change(
                &mut **tx,
                amendment_id,
                Some(current_status),
                AmendmentStatus::RevisionRequired,
                SYSTEM_USER_ID,
                Some("審查委員要求修訂".to_string()),
            )
            .await?;
        } else if stats.rejected > 0 {
            apply_terminal_decision_tx(
                tx,
                amendment_id,
                TerminalDecisionContext {
                    signer_id: tipping_reviewer_id,
                    is_approve: false,
                    decision_summary: &summary,
                },
                current_status,
            )
            .await?;
        } else if stats.approved == stats.total {
            apply_terminal_decision_tx(
                tx,
                amendment_id,
                TerminalDecisionContext {
                    signer_id: tipping_reviewer_id,
                    is_approve: true,
                    decision_summary: &summary,
                },
                current_status,
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
