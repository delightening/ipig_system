//! R84-5 正式沖銷單（紅字沖銷）。
//!
//! 設計見 `docs/spec/modules/ERP流程.md` §6.3.1。核心原則：沖銷**不是**重跑一次業務邏輯，
//! 而是原封不動地鏡射原單**實際寫入**的東西——這是標準紅字沖銷做法，也是唯一符合
//! GLP append-only 精神的方式（原始紀錄完全不動，只新增一筆方向相反的）。
//!
//! 兩階段權限（SoD）：`WAREHOUSE_MANAGER` 發起沖銷（建草稿），必須經 `ADMIN` 最終核准
//! 才會真的執行鏡射寫入。比照大金額 ADJ 的 `wm_approved` → `admin_approve` 範式，
//! 沿用 `requires_manager_approval` / `manager_approval_status` 欄位，不新增 schema。

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, DocStatus, Document, DocumentAuditSnapshot, DocumentLine,
        DocumentWithLines,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AccountingService, AuditService, StockService,
    },
    AppError, Result,
};

use super::DocumentService;

impl DocumentService {
    /// 發起沖銷：建立一張指向原單的沖銷草稿（`WAREHOUSE_MANAGER` 或 admin）。
    ///
    /// 此階段**不動庫存、不過帳**——只建立待核准的沖銷單。實際鏡射在
    /// [`DocumentService::approve_reversal`]（ADMIN 最終核准）才執行。
    pub async fn create_reversal(
        pool: &PgPool,
        actor: &ActorContext,
        original_id: Uuid,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        if !user.has_role(crate::constants::ROLE_WAREHOUSE_MANAGER) && !user.is_admin() {
            return Err(AppError::Forbidden("僅倉庫管理員或管理員可發起沖銷".into()));
        }

        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖原單：與 DB 的 partial unique index 雙重保證同一張單不會被沖銷兩次。
        // 只靠應用層 check-then-create 在並發下會有兩邊都查到「尚未沖銷」的 race。
        let original =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(original_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if original.status != DocStatus::Approved {
            return Err(AppError::BusinessRule("僅已核准的單據可以沖銷".to_string()));
        }
        if original.reverses_doc_id.is_some() {
            return Err(AppError::BusinessRule(
                "沖銷單本身不可再被沖銷；如需回復請重新開立原類型單據".to_string(),
            ));
        }

        let existing: Option<String> =
            sqlx::query_scalar("SELECT doc_no FROM documents WHERE reverses_doc_id = $1 LIMIT 1")
                .bind(original_id)
                .fetch_optional(&mut *tx)
                .await?;
        if let Some(doc_no) = existing {
            return Err(AppError::BusinessRule(format!(
                "此單據已於沖銷單 {} 沖銷過，不可重複沖銷",
                doc_no
            )));
        }

        let original_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(original_id)
        .fetch_all(&mut *tx)
        .await?;

        // 沖銷單沿用原單型別（不新增 DB enum 值，與 R84-9 選項 B 精神一致）；
        // 靠 reverses_doc_id 辨識其為沖銷單。
        let doc_no = Self::generate_doc_no(&mut tx, original.doc_type).await?;
        let reversal_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, warehouse_from_id, warehouse_to_id,
                partner_id, doc_date, remark, created_by, reverses_doc_id,
                requires_manager_approval, manager_approval_status, protocol_id, iacuc_no,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                $8, CURRENT_DATE, $9, $10, $11,
                true, 'wm_approved', $12, $13,
                NOW(), NOW()
            )
            "#,
        )
        .bind(reversal_id)
        .bind(original.doc_type)
        .bind(&doc_no)
        .bind(DocStatus::Submitted)
        .bind(original.warehouse_id)
        .bind(original.warehouse_from_id)
        .bind(original.warehouse_to_id)
        .bind(original.partner_id)
        .bind(format!("沖銷 {}", original.doc_no))
        .bind(user.id)
        .bind(original_id)
        .bind(original.protocol_id)
        .bind(&original.iacuc_no)
        .execute(&mut *tx)
        .await?;

        // 明細原樣複製（數量為正）——方向相反體現在 stock_ledger 鏡射，不在明細數量上，
        // 否則會撞到 validate_line_qty_price 的「移動單據 qty 必須 > 0」規則。
        for line in &original_lines {
            sqlx::query(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark, storage_location_id,
                    storage_location_from_id, storage_location_to_id, warehouse_id
                )
                VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#,
            )
            .bind(reversal_id)
            .bind(line.line_no)
            .bind(line.product_id)
            .bind(line.qty)
            .bind(&line.uom)
            .bind(line.unit_price)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&line.remark)
            .bind(line.storage_location_id)
            .bind(line.storage_location_from_id)
            .bind(line.storage_location_to_id)
            .bind(line.warehouse_id)
            .execute(&mut *tx)
            .await?;
        }

        let reversal = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(reversal_id)
            .fetch_one(&mut *tx)
            .await?;
        let reversal_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(reversal_id)
        .fetch_all(&mut *tx)
        .await?;

        let display = format!("沖銷單 {}（原單 {}）", reversal.doc_no, original.doc_no);
        let after_snap = DocumentAuditSnapshot {
            document: &reversal,
            lines: &reversal_lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_REVERSAL_CREATE",
                entity: Some(AuditEntity::new("document", reversal_id, &display)),
                data_diff: Some(DataDiff::compute(
                    None::<&DocumentAuditSnapshot>,
                    Some(&after_snap),
                )),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Self::get_by_id(pool, reversal_id).await
    }

    /// ADMIN 最終核准沖銷單：執行庫存與會計的反向鏡射。
    ///
    /// 與一般 `admin_approve` 的差別在於**不呼叫** `StockService::process_document` /
    /// `AccountingService::post_document`（那會用當下狀態重算），而是鏡射原單實際寫入的紀錄。
    pub async fn approve_reversal(
        pool: &PgPool,
        actor: &ActorContext,
        reversal_id: Uuid,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        if !user.is_admin() {
            return Err(AppError::Forbidden(
                "沖銷單須由管理員最終核准（發起人不得自行核准）".into(),
            ));
        }
        let admin_id = user.id;

        let mut tx = pool.begin().await?;

        let reversal =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(reversal_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        let original_id = reversal.reverses_doc_id.ok_or_else(|| {
            AppError::BusinessRule("此單據不是沖銷單，無法以沖銷流程核准".to_string())
        })?;
        if reversal.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule(
                "沖銷單必須為已提交狀態才能核准".to_string(),
            ));
        }
        if reversal.created_by == admin_id {
            return Err(AppError::Forbidden(
                "職務分離：沖銷單的發起人不得自行核准".into(),
            ));
        }

        let original =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(original_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("原單據不存在".to_string()))?;

        let before_doc = reversal.clone();

        // 庫存鏡射：ledger 反向 + SLI 反向 + snapshot 重算（三者缺一即產生 drift，
        // 見 StockService::reverse_document_stock 的說明）。
        StockService::reverse_document_stock(&mut tx, &original, &reversal).await?;

        // 會計鏡射：借貸互換。刻意**不**用 best-effort SAVEPOINT——
        // 庫存退回但傳票沒鏡射成功會讓帳永久歪掉，屬合規路徑，失敗即整筆 rollback。
        AccountingService::reverse_document_posting(&mut tx, &original, &reversal, admin_id)
            .await?;

        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                manager_approval_status = 'approved',
                manager_approved_by = $2,
                manager_approved_at = NOW(),
                approved_by = $2,
                approved_at = NOW(),
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(DocStatus::Approved)
        .bind(admin_id)
        .bind(reversal_id)
        .execute(&mut *tx)
        .await?;

        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(reversal_id)
            .fetch_one(&mut *tx)
            .await?;
        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(reversal_id)
        .fetch_all(&mut *tx)
        .await?;

        let display = format!(
            "沖銷單 {}（原單 {}，ADMIN 核准）",
            after_doc.doc_no, original.doc_no
        );
        let before_snap = DocumentAuditSnapshot {
            document: &before_doc,
            lines: &lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_REVERSAL_APPROVE",
                entity: Some(AuditEntity::new("document", reversal_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Self::get_by_id(pool, reversal_id).await
    }
}
