use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::CurrentUser,
    models::{
        DocStatus, DocType, Document, DocumentLine, DocumentWithLines,
    },
    services::{AccountingService, StockService},
    AppError, Result,
};

use super::DocumentService;

impl DocumentService {
    /// 檢查單據存取權限（IDOR 防護）
    /// 建立者、倉庫管理員或管理員可存取
    pub fn check_access(current_user: &CurrentUser, created_by: Uuid) -> Result<()> {
        let is_creator = current_user.id == created_by;
        let is_warehouse_manager = current_user.has_role(crate::constants::ROLE_WAREHOUSE_MANAGER);
        let is_admin = current_user.is_admin();

        if is_creator || is_warehouse_manager || is_admin {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此文件".into()))
        }
    }

    /// 送審
    /// 對於調整單(ADJ)，若報廢金額超過門檻，需要主管簽核
    pub async fn submit(pool: &PgPool, id: Uuid) -> Result<DocumentWithLines> {
        // 取得單據資訊
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Draft {
            return Err(AppError::BusinessRule("Document must be in draft status to submit".to_string()));
        }

        let mut tx = pool.begin().await?;

        // 對於調整單(報廢)，計算總金額並檢查是否需要主管簽核
        let (requires_manager_approval, scrap_total_amount) = if document.doc_type == DocType::ADJ {
            // 計算調整單總金額
            let total_amount: Option<Decimal> = sqlx::query_scalar(
                r#"
                SELECT SUM(ABS(dl.qty) * COALESCE(dl.unit_price, 0)) as total
                FROM document_lines dl
                WHERE dl.document_id = $1
                "#
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            let scrap_amount = total_amount.unwrap_or(Decimal::ZERO);

            // 從系統設定取得報廢簽核門檻
            let threshold: Option<Decimal> = sqlx::query_scalar(
                "SELECT (value #>> '{}')::DECIMAL FROM system_settings WHERE key = 'scrap_approval_threshold'"
            )
            .fetch_optional(&mut *tx)
            .await?;

            let threshold_amount = threshold.unwrap_or(Decimal::new(5000, 0)); // 預設 5000

            if scrap_amount >= threshold_amount {
                (Some(true), Some(scrap_amount))
            } else {
                (Some(false), Some(scrap_amount))
            }
        } else {
            (None, None)
        };

        // 更新單據狀態
        if requires_manager_approval == Some(true) {
            // 需要主管簽核的報廢單
            sqlx::query(
                r#"
                UPDATE documents SET 
                    status = $1, 
                    requires_manager_approval = true,
                    scrap_total_amount = $2,
                    manager_approval_status = 'pending',
                    updated_at = NOW()
                WHERE id = $3
                "#
            )
            .bind(DocStatus::Submitted)
            .bind(scrap_total_amount)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            tracing::info!(
                "[Scrap Approval] Document {} requires manager approval. Total amount: {:?}",
                id,
                scrap_total_amount
            );
        } else {
            // 一般單據或金額未超過門檻
            sqlx::query(
                r#"
                UPDATE documents SET 
                    status = $1, 
                    requires_manager_approval = COALESCE($2, false),
                    scrap_total_amount = $3,
                    updated_at = NOW()
                WHERE id = $4
                "#
            )
            .bind(DocStatus::Submitted)
            .bind(requires_manager_approval)
            .bind(scrap_total_amount)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 核准（寫入庫存流水）
    /// 採購單核准後會自動產生入庫單（草稿）
    /// 大金額 ADJ 調整單：WAREHOUSE_MANAGER 核准後進入 wm_approved 狀態，等待 ADMIN 最終核准
    pub async fn approve(pool: &PgPool, id: Uuid, approved_by: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule("Document must be in submitted status to approve".to_string()));
        }

        // 大金額 ADJ：WAREHOUSE_MANAGER 核准後不直接生效，改為等待 ADMIN 核准
        let needs_admin = document.requires_manager_approval == Some(true)
            && document.manager_approval_status.as_deref() == Some("pending");

        if needs_admin {
            sqlx::query(
                r#"
                UPDATE documents SET
                    manager_approval_status = 'wm_approved',
                    approved_by = $1,
                    approved_at = NOW(),
                    updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(approved_by)
            .bind(id)
            .execute(pool)
            .await?;

            tracing::info!(
                "[ADJ Approval] Document {} approved by warehouse manager, awaiting admin approval",
                id
            );

            return Self::get_by_id(pool, id).await;
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let mut tx = pool.begin().await?;

        // 檢查庫存並寫入流水
        if document.doc_type.affects_stock() {
            StockService::process_document(&mut tx, &document, &lines).await?;
        }

        // 會計過帳（GRN/DO 產生傳票分錄）
        if let Err(e) = AccountingService::post_document(&mut tx, &document, &lines, approved_by).await {
            tracing::warn!("會計過帳跳過或失敗 (document {}): {}", document.id, e);
            // 不阻擋核准，會計為附加功能
        }

        // 更新單據狀態
        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                approved_by = $2,
                approved_at = NOW(),
                updated_at = NOW()
            WHERE id = $3
            "#
        )
        .bind(DocStatus::Approved)
        .bind(approved_by)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 如果是採購單，自動產生入庫單（草稿）
        if document.doc_type == DocType::PO {
            Self::create_grn_from_po(&mut tx, &document, &lines, approved_by).await?;

            // 更新採購單的入庫狀態
            sqlx::query(
                "UPDATE documents SET receipt_status = 'pending' WHERE id = $1"
            )
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }

        // 如果是入庫單核准，回寫更新來源採購單的 receipt_status
        if document.doc_type == DocType::GRN {
            if let Some(po_id) = document.source_doc_id {
                Self::update_po_receipt_status(&mut tx, po_id).await?;
            }
        }

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// ADMIN 最終核准（大金額 ADJ 調整單）
    /// 前置條件：requires_manager_approval = true 且 manager_approval_status = 'wm_approved'
    pub async fn admin_approve(pool: &PgPool, id: Uuid, admin_id: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule("單據必須為已提交狀態".to_string()));
        }

        if document.requires_manager_approval != Some(true)
            || document.manager_approval_status.as_deref() != Some("wm_approved")
        {
            return Err(AppError::BusinessRule(
                "此單據尚未經倉庫管理員核准，無法進行管理員最終核准".to_string(),
            ));
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no"
        )
        .bind(id)
        .fetch_all(pool)
        .await?;

        let mut tx = pool.begin().await?;

        // 寫入庫存流水
        if document.doc_type.affects_stock() {
            StockService::process_document(&mut tx, &document, &lines).await?;
        }

        // 會計過帳
        if let Err(e) = AccountingService::post_document(&mut tx, &document, &lines, admin_id).await {
            tracing::warn!("會計過帳跳過或失敗 (document {}): {}", document.id, e);
        }

        // 更新單據為最終核准
        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                manager_approval_status = 'approved',
                manager_approved_by = $2,
                manager_approved_at = NOW(),
                updated_at = NOW()
            WHERE id = $3
            "#
        )
        .bind(DocStatus::Approved)
        .bind(admin_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::info!("[ADJ Admin Approval] Document {} approved by admin {}", id, admin_id);

        Self::get_by_id(pool, id).await
    }

    /// ADMIN 駁回（大金額 ADJ 調整單）
    /// 單據退回草稿狀態，建立者可修改後重新提交
    pub async fn admin_reject(pool: &PgPool, id: Uuid, admin_id: Uuid, reason: &str) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule("單據必須為已提交狀態".to_string()));
        }

        if document.requires_manager_approval != Some(true)
            || document.manager_approval_status.as_deref() != Some("wm_approved")
        {
            return Err(AppError::BusinessRule(
                "此單據尚未經倉庫管理員核准，無法進行管理員駁回".to_string(),
            ));
        }

        // 退回草稿狀態，清除倉庫核准資訊
        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                manager_approval_status = 'rejected',
                manager_approved_by = $2,
                manager_approved_at = NOW(),
                manager_reject_reason = $3,
                approved_by = NULL,
                approved_at = NULL,
                updated_at = NOW()
            WHERE id = $4
            "#
        )
        .bind(DocStatus::Draft)
        .bind(admin_id)
        .bind(reason)
        .bind(id)
        .execute(pool)
        .await?;

        tracing::info!(
            "[ADJ Admin Reject] Document {} rejected by admin {}. Reason: {}",
            id, admin_id, reason
        );

        Self::get_by_id(pool, id).await
    }

    /// 作廢
    pub async fn cancel(pool: &PgPool, id: Uuid) -> Result<DocumentWithLines> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status == DocStatus::Approved {
            return Err(AppError::BusinessRule("Cannot cancel approved documents. Use reversal instead.".to_string()));
        }

        sqlx::query(
            "UPDATE documents SET status = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(DocStatus::Cancelled)
        .bind(id)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, id).await
    }
}
