//! 撤銷「已核准、尚無有效下游單據」的採購單（PO）。
//!
//! PO 核准只做兩件事：標 `approved`、`receipt_status = 'pending'`——**不寫庫存、不過帳**
//! （`models::DocType::affects_stock` 不含 PO、`AccountingService` 對 PO 是 no-op）。
//! 所以它沒有任何東西需要「紅字沖銷」，走需要二人核准的沖銷（`reversal.rs`）只是繞路；
//! 而 `cancel` 又一律拒絕 `approved` 單據。本檔補上這個缺口：倉管或管理員單人撤銷，
//! 必填原因，進 HMAC chain。
//!
//! 已有有效入庫單的 PO 仍只能走沖銷——入庫單有庫存與會計副作用，撤銷 PO 不會回沖它們。
//!
//! 前置條件的判斷集中在 [`evaluate_revoke`]，`POST /documents/{id}/revoke`、詳情回應的
//! `can_revoke` / `revoke_blocked_by` / `has_effective_downstream`，以及 `create_reversal`
//! 對採購單的守衛都出自它（或它底下的 [`load_downstream`]），兩邊不會漂移。

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, DocStatus, DocType, Document, DocumentLine, DocumentRevokeSnapshot,
        DocumentWithLines, RevokeBlockedBy,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, NotificationService,
    },
    AppError, Result,
};

use super::DocumentService;

/// 撤銷原因 trim 後的字數下限。
const MIN_REVOKE_REASON_CHARS: usize = 5;

/// 稽核事件類型：與一般作廢（`DOC_CANCEL`）區分，稽核頁才看得出這是撤銷「已核准」單據。
const EVENT_DOC_REVOKE_APPROVED: &str = "DOC_REVOKE_APPROVED";

/// 角色判準比照 `create_reversal`：倉庫管理員或管理員。
fn has_revoke_role(user: &CurrentUser) -> bool {
    user.has_role(crate::constants::ROLE_WAREHOUSE_MANAGER) || user.is_admin()
}

/// 只看單據本身就能判定的前置條件（不查 DB）。
fn revoke_state_blocker(po: &Document) -> Option<String> {
    if po.doc_type != DocType::PO {
        return Some("僅採購單可撤銷".to_string());
    }
    if po.status != DocStatus::Approved {
        return Some("僅已核准的採購單可撤銷".to_string());
    }
    if po.reverses_doc_id.is_some() {
        return Some("沖銷單不可撤銷".to_string());
    }
    None
}

/// 引用此 PO 的下游單據（`source_doc_id` 指向 PO）：非作廢、非沖銷單、未被已核准沖銷單沖銷者，
/// 依建立時間排序。
///
/// 「還算不算數」的判準與入庫進度、未入庫提醒同源，沿用 [`crate::exclude_reversed_grn!`]——
/// 不自己抄一份述詞，否則兩邊對「這張入庫單還算不算數」會給出不同答案。
/// 撤銷的擋下理由、詳情回應的 `has_effective_downstream`、以及 `create_reversal` 對採購單的
/// 守衛，全部從這個函式的結果推導。
pub(crate) async fn load_downstream(
    conn: &mut PgConnection,
    po_id: Uuid,
) -> Result<Vec<RevokeBlockedBy>> {
    let rows = sqlx::query_as(concat!(
        r#"
        SELECT g.doc_no, g.doc_type, g.status
        FROM documents g
        WHERE g.source_doc_id = $1
          AND g.status <> 'cancelled'
          AND g.reverses_doc_id IS NULL
        "#,
        crate::exclude_reversed_grn!(),
        r#"
        ORDER BY g.created_at
        "#
    ))
    .bind(po_id)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// 一次評估的完整結果：端點用 `blocker`，詳情回應用其餘欄位，彼此不會漂移。
pub(crate) struct RevokeEvaluation {
    /// 第一個擋下的理由（繁中，可直接給使用者）；`None` 表示資料面可撤銷。
    pub blocker: Option<String>,
    /// 擋住撤銷、尚未處理完（草稿／待核准）的下游單據；有多張時取最早建立的。
    pub unfinished: Option<RevokeBlockedBy>,
    /// 是否有有效下游單據（已核准且未被沖銷）。
    pub has_effective_downstream: bool,
}

impl RevokeEvaluation {
    fn blocked(reason: String) -> Self {
        Self {
            blocker: Some(reason),
            unfinished: None,
            has_effective_downstream: false,
        }
    }
}

/// 撤銷的資料面前置條件。
///
/// 草稿／待核准的下游單據一律擋下（不連帶作廢）：它們核准後就會變成指向已作廢 PO 的有效單據。
/// 它們排在已核准者之前——使用者要先處理它們，才輪得到「已有有效入庫單」那條路。
///
/// 呼叫者必須已在同一個 tx 內對 PO 做過 `FOR UPDATE`，否則檢查與後續更新之間有空窗。
pub(crate) async fn evaluate_revoke(
    conn: &mut PgConnection,
    po: &Document,
) -> Result<RevokeEvaluation> {
    if let Some(reason) = revoke_state_blocker(po) {
        return Ok(RevokeEvaluation::blocked(reason));
    }

    // 這張 PO 本身已被沖銷（或沖銷單待核准）：沖銷單以 `reverses_doc_id` 指向原單，
    // 不帶 `source_doc_id`，下面的下游單據查詢抓不到它。放行的話原單會變 cancelled、
    // 卻留著一張指向它的有效沖銷單；待核准的沖銷單核准時也會作用在已作廢的原單上。
    // 已作廢的沖銷單不佔名額（migration 017），不擋。
    let reversal_doc_no: Option<String> = sqlx::query_scalar(
        "SELECT doc_no FROM documents \
         WHERE reverses_doc_id = $1 AND status IN ('submitted', 'approved') \
         ORDER BY created_at LIMIT 1",
    )
    .bind(po.id)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some(doc_no) = reversal_doc_no {
        return Ok(RevokeEvaluation::blocked(format!(
            "此採購單已有沖銷單 {}，不可撤銷",
            doc_no
        )));
    }

    let downstream = load_downstream(conn, po.id).await?;
    let unfinished = downstream
        .iter()
        .find(|d| d.status != DocStatus::Approved)
        .cloned();
    // 撤銷的擋下理由維持寬鬆：任何已核准且未被沖銷的下游單據都擋（撤銷不會回沖它們）。
    let effective = downstream.iter().find(|d| d.status == DocStatus::Approved);
    // 但「可以沖銷這張 PO」只認有效的入庫單：非入庫單的已核准單據不能讓 PO 沖銷放行，
    // 否則沒有入庫單的 PO 也能走二人核准的沖銷（按鈕分流與 `create_reversal` 守衛共用此判準）。
    let has_effective_grn = downstream.iter().any(is_effective_grn);

    let blocker = match (&unfinished, effective) {
        (Some(d), _) => Some(format!("請先處理單據 {}", d.doc_no)),
        (None, Some(d)) if d.doc_type == DocType::GRN => Some(format!(
            "此採購單已有入庫單 {}，請先沖銷該入庫單（沖銷核准後即可撤銷採購單）",
            d.doc_no
        )),
        (None, Some(d)) => Some(format!(
            "此採購單已有已核准的單據 {}，請先沖銷該單據（沖銷核准後即可撤銷採購單）",
            d.doc_no
        )),
        (None, None) => None,
    };

    Ok(RevokeEvaluation {
        blocker,
        unfinished,
        has_effective_downstream: has_effective_grn,
    })
}

/// 有效入庫單：已核准的入庫單（`load_downstream` 已排除作廢與已被沖銷者）。
fn is_effective_grn(d: &RevokeBlockedBy) -> bool {
    d.doc_type == DocType::GRN && d.status == DocStatus::Approved
}

/// 詳情回應裡和撤銷／沖銷按鈕分流有關的三個欄位，由 [`DocumentService::revoke_view`] 一次算出。
#[derive(Debug, Default)]
pub struct RevokeView {
    pub can_revoke: bool,
    pub revoke_blocked_by: Option<RevokeBlockedBy>,
    pub has_effective_downstream: bool,
}

impl DocumentService {
    /// 詳情回應的 `can_revoke` / `revoke_blocked_by` / `has_effective_downstream` 來源。
    ///
    /// 與 [`DocumentService::revoke_approved_po`] 共用 [`evaluate_revoke`]。單據本身就不符
    /// （非 PO、非 Approved、沖銷單）時不查 DB，三個欄位維持預設值。
    pub async fn revoke_view(
        pool: &PgPool,
        user: &CurrentUser,
        po: &Document,
    ) -> Result<RevokeView> {
        if revoke_state_blocker(po).is_some() {
            return Ok(RevokeView::default());
        }
        let mut conn = pool.acquire().await?;
        let evaluation = evaluate_revoke(&mut conn, po).await?;
        let privileged = has_revoke_role(user);
        Ok(RevokeView {
            can_revoke: privileged && evaluation.blocker.is_none(),
            // 擋下者的單號與狀態是別人建立的單據資訊；詳情的存取檢查只看「建單者」，
            // 所以沒有撤銷角色的建單者不該讀到。前端本來就只在有角色時才用這個欄位。
            revoke_blocked_by: if privileged {
                evaluation.unfinished
            } else {
                None
            },
            has_effective_downstream: evaluation.has_effective_downstream,
        })
    }

    /// 這位使用者現在能不能撤銷這張單；`revoke_view().can_revoke` 的簡寫。
    pub async fn can_revoke(pool: &PgPool, user: &CurrentUser, po: &Document) -> Result<bool> {
        Ok(Self::revoke_view(pool, user, po).await?.can_revoke)
    }

    /// 這張採購單有沒有有效入庫單（已核准且未被沖銷的 GRN）。`create_reversal` 用它擋掉
    /// 「沒有東西可沖銷」的採購單，與撤銷是互補的兩條路。
    ///
    /// 呼叫者必須已在同一個 tx 內對 PO 做過 `FOR UPDATE`。
    pub(crate) async fn po_has_effective_downstream(
        conn: &mut PgConnection,
        po_id: Uuid,
    ) -> Result<bool> {
        Ok(load_downstream(conn, po_id)
            .await?
            .iter()
            .any(is_effective_grn))
    }

    /// 撤銷已核准、尚無有效下游單據的採購單（status → `cancelled`）。
    ///
    /// 不寫庫存、不過帳（見檔頭）。整個動作在單一 tx：鎖 PO → 檢查 → 更新 → 稽核。
    /// 鎖 PO 與 `create_additional_grn` 互斥，避免撤銷的同時有人開出指向已作廢 PO 的入庫單。
    pub async fn revoke_approved_po(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        if !has_revoke_role(user) {
            return Err(AppError::Forbidden(
                "僅倉庫管理員或管理員可撤銷已核准的採購單".into(),
            ));
        }

        let reason = reason.trim();
        if reason.chars().count() < MIN_REVOKE_REASON_CHARS {
            return Err(AppError::Validation(format!(
                "請填寫撤銷原因（至少 {} 字）",
                MIN_REVOKE_REASON_CHARS
            )));
        }

        let mut tx = pool.begin().await?;

        let po = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if let Some(blocker) = evaluate_revoke(&mut tx, &po).await?.blocker {
            return Err(AppError::BusinessRule(blocker));
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        // `receipt_status` 不動：它記的是撤銷當下的入庫進度，作廢單據保留這個歷史值與
        // `cancel` 的行為一致；列表與詳情以 `status` 優先顯示「已撤銷」。
        sqlx::query("UPDATE documents SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(DocStatus::Cancelled)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        let display = format!("{}: {} (撤銷)", po.doc_type.prefix(), after_doc.doc_no);
        let before_snap = DocumentRevokeSnapshot {
            document: &po,
            lines: &lines,
            revoke_reason: None,
        };
        let after_snap = DocumentRevokeSnapshot {
            document: &after_doc,
            lines: &lines,
            revoke_reason: Some(reason),
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: EVENT_DOC_REVOKE_APPROVED,
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        // 已核准 PO 可能有置頂的「未入庫提醒」；撤銷為終態，同 tx 解除，
        // 不等排程對帳的安全網（見 services/notification/reconcile.rs）。
        NotificationService::resolve_pinned_notifications_tx(&mut tx, "document", id).await?;

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }
}
