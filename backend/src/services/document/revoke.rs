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
//! 前置條件的判斷集中在 [`revoke_blocker`]，`POST /documents/{id}/revoke` 與詳情回應的
//! `can_revoke` 共用同一個函式，兩邊不會漂移。

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, DocStatus, DocType, Document, DocumentLine, DocumentRevokeSnapshot,
        DocumentWithLines,
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

/// 撤銷的資料面前置條件：回傳第一個擋下的理由（繁中，可直接給使用者），`None` 表示可撤銷。
///
/// 「有效下游單據」的判準與入庫進度、未入庫提醒同源，沿用
/// [`crate::exclude_reversed_grn!`]——不自己抄一份述詞，否則兩邊對「這張入庫單還算不算數」
/// 會給出不同答案。再加上：已作廢者不算；草稿／待核准的單據照樣擋，因為它們核准後就會
/// 變成指向已作廢 PO 的有效單據。
///
/// 呼叫者必須已在同一個 tx 內對 PO 做過 `FOR UPDATE`，否則檢查與後續更新之間有空窗。
pub(crate) async fn revoke_blocker(
    conn: &mut PgConnection,
    po: &Document,
) -> Result<Option<String>> {
    if let Some(reason) = revoke_state_blocker(po) {
        return Ok(Some(reason));
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
        return Ok(Some(format!("此採購單已有沖銷單 {}，不可撤銷", doc_no)));
    }

    // 未處理完的單據（草稿／待核准）排在已核准者之前：使用者要先處理它們，
    // 才輪得到「已有有效入庫單」那條路。
    let blocking: Option<(String, DocType, DocStatus)> = sqlx::query_as(concat!(
        r#"
        SELECT g.doc_no, g.doc_type, g.status
        FROM documents g
        WHERE g.source_doc_id = $1
          AND g.status <> 'cancelled'
          AND g.reverses_doc_id IS NULL
        "#,
        crate::exclude_reversed_grn!(),
        r#"
        ORDER BY (g.status = 'approved') ASC, g.created_at
        LIMIT 1
        "#
    ))
    .bind(po.id)
    .fetch_optional(&mut *conn)
    .await?;

    Ok(blocking.map(|(doc_no, doc_type, status)| match status {
        DocStatus::Approved if doc_type == DocType::GRN => {
            format!("此採購單已有入庫單 {}，請改走沖銷流程", doc_no)
        }
        DocStatus::Approved => {
            format!("此採購單已有已核准的單據 {}，請改走沖銷流程", doc_no)
        }
        _ => format!("請先處理單據 {}", doc_no),
    }))
}

impl DocumentService {
    /// 這位使用者現在能不能撤銷這張單；詳情回應的 `can_revoke` 來源。
    ///
    /// 與 [`DocumentService::revoke_approved_po`] 共用 [`revoke_blocker`]。角色不符或單據
    /// 本身就不符時不查 DB。
    pub async fn can_revoke(pool: &PgPool, user: &CurrentUser, po: &Document) -> Result<bool> {
        if !has_revoke_role(user) || revoke_state_blocker(po).is_some() {
            return Ok(false);
        }
        let mut conn = pool.acquire().await?;
        Ok(revoke_blocker(&mut conn, po).await?.is_none())
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

        if let Some(blocker) = revoke_blocker(&mut tx, &po).await? {
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
