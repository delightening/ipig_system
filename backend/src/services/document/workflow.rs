use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{
        DocStatus, DocType, Document, DocumentLine, DocumentWithLines,
    },
    services::StockService,
    AppError, Result,
};

use super::DocumentService;

impl DocumentService {
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
                "SELECT setting_value::DECIMAL FROM system_settings WHERE setting_key = 'scrap_approval_threshold'"
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

        tx.commit().await?;

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
