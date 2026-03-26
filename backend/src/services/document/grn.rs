use rust_decimal::Decimal;

use crate::time;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    models::{
        DocStatus, DocType, Document, DocumentLine, DocumentWithLines,
        PoReceiptStatus, PoReceiptItem,
    },
    AppError, Result,
};

/// 採購單明細查詢結果（避免 clippy::type_complexity）
#[derive(FromRow)]
struct PoLineRow {
    product_id: Uuid,
    sku: String,
    name: String,
    base_uom: String,
    uom: String,
    unit_price: Option<Decimal>,
    qty: Decimal,
}

use super::DocumentService;

/// 從上一張單據的單號字串解析下一個流水號。
/// 例如：`next_seq_from_last_no("GRN-260314-005")` → `6`
/// 若解析失敗則回傳 1（從頭開始）。
pub(super) fn next_seq_from_last_no(last_no: Option<&str>) -> i32 {
    match last_no {
        Some(no) => {
            let parts: Vec<&str> = no.split('-').collect();
            if parts.len() >= 3 {
                parts[2].parse::<i32>().unwrap_or(0) + 1
            } else {
                1
            }
        }
        None => 1,
    }
}

/// 依據已入庫量與採購量決定入庫狀態字串。
pub(super) fn receipt_status_label(
    total_received: rust_decimal::Decimal,
    total_ordered: rust_decimal::Decimal,
) -> &'static str {
    if total_received == rust_decimal::Decimal::ZERO {
        "pending"
    } else if total_received < total_ordered {
        "partial"
    } else {
        "complete"
    }
}

impl DocumentService {
    /// 從採購單建立入庫單（草稿）
    pub(crate) async fn create_grn_from_po(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po: &Document,
        po_lines: &[DocumentLine],
        created_by: Uuid,
    ) -> Result<Uuid> {
        // 產生入庫單編號 (GRN-YYMMDD-{03})，以台灣日期為準
        let today = time::now_taiwan();
        let year = today.format("%y").to_string(); // 2-digit year
        let month_day = today.format("%m%d").to_string();
        let date_str = format!("{}{}", year, month_day);
        let prefix = format!("GRN-{}", date_str);
        
        let last_no: Option<String> = sqlx::query_scalar(
            "SELECT doc_no FROM documents WHERE doc_no LIKE $1 ORDER BY doc_no DESC LIMIT 1"
        )
        .bind(format!("{}%", prefix))
        .fetch_optional(&mut **tx)
        .await?;

        let seq = next_seq_from_last_no(last_no.as_deref());
        let doc_no = format!("{}-{:03}", prefix, seq);

        // 建立入庫單頭
        let grn_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, partner_id, doc_date,
                source_doc_id, remark, created_by, created_at, updated_at
            )
            VALUES ($1, 'GRN', $2, 'draft', $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#
        )
        .bind(grn_id)
        .bind(&doc_no)
        .bind(po.warehouse_id)
        .bind(po.partner_id)
        .bind(time::today_taiwan_naive())
        .bind(po.id)  // source_doc_id 關聯到採購單
        .bind(format!("自動產生自採購單 {}", po.doc_no))
        .bind(created_by)
        .execute(&mut **tx)
        .await?;

        // 建立入庫單明細（從採購單帶入）
        for (idx, line) in po_lines.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(grn_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(line.qty)  // 預設數量等於採購數量
            .bind(&line.uom)
            .bind(line.unit_price)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&line.remark)
            .execute(&mut **tx)
            .await?;
        }

        Ok(grn_id)
    }

    /// 從採購單建立額外入庫單（部分入庫用）
    pub async fn create_additional_grn(
        pool: &PgPool,
        po_id: Uuid,
        created_by: Uuid,
    ) -> Result<DocumentWithLines> {
        // 檢查採購單狀態
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'"
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        if po.status != DocStatus::Approved {
            return Err(AppError::BusinessRule("Purchase order must be approved".to_string()));
        }

        // 取得採購單明細
        let po_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no"
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量
        let received_qty: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id, COALESCE(SUM(dl.qty), 0) as received
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.source_doc_id = $1 
              AND d.doc_type = 'GRN' 
              AND d.status = 'approved'
            GROUP BY dl.product_id
            "#
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> = 
            received_qty.into_iter().collect();

        // 計算剩餘數量
        let remaining_lines: Vec<_> = po_lines
            .iter()
            .filter_map(|line| {
                let received = received_map.get(&line.product_id).copied().unwrap_or(Decimal::ZERO);
                let remaining = line.qty - received;
                if remaining > Decimal::ZERO {
                    Some((line.clone(), remaining))
                } else {
                    None
                }
            })
            .collect();

        if remaining_lines.is_empty() {
            return Err(AppError::BusinessRule("All items have been received".to_string()));
        }

        let mut tx = pool.begin().await?;

        // 產生入庫單編號 (統一格式：YYMMDD-{02})
        let doc_no = Self::generate_doc_no(&mut tx, DocType::GRN).await?;

        // 建立入庫單
        let grn_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, partner_id, doc_date,
                source_doc_id, remark, created_by, created_at, updated_at
            )
            VALUES ($1, 'GRN', $2, 'draft', $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#
        )
        .bind(grn_id)
        .bind(&doc_no)
        .bind(po.warehouse_id)
        .bind(po.partner_id)
        .bind(time::today_taiwan_naive())
        .bind(po.id)
        .bind(format!("追加入庫 - 採購單 {}", po.doc_no))
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        // 建立入庫單明細（只含剩餘數量）
        for (idx, (line, remaining)) in remaining_lines.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom, unit_price,
                    batch_no, expiry_date, remark
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(grn_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(*remaining)  // 預設為剩餘數量
            .bind(&line.uom)
            .bind(line.unit_price)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&line.remark)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::get_by_id(pool, grn_id).await
    }

    /// GRN 核准後，重新計算並回寫 PO 的 receipt_status
    pub(crate) async fn update_po_receipt_status(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po_id: Uuid,
    ) -> Result<()> {
        let row: (Decimal, Decimal) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(pl.qty), 0) AS total_ordered,
                COALESCE((
                    SELECT SUM(gl.qty)
                    FROM documents g
                    JOIN document_lines gl ON g.id = gl.document_id
                    WHERE g.source_doc_id = $1
                      AND g.doc_type = 'GRN'
                      AND g.status = 'approved'
                ), 0) AS total_received
            FROM document_lines pl
            WHERE pl.document_id = $1
            "#
        )
        .bind(po_id)
        .fetch_one(&mut **tx)
        .await?;

        let status = receipt_status_label(row.1, row.0);

        sqlx::query(
            "UPDATE documents SET receipt_status = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(status)
        .bind(po_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 取得採購單入庫狀態
    pub async fn get_po_receipt_status(pool: &PgPool, po_id: Uuid) -> Result<PoReceiptStatus> {
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'"
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        // 取得採購單明細
        let po_lines: Vec<PoLineRow> = sqlx::query_as(
            r#"
            SELECT dl.product_id, p.sku, p.name, p.base_uom, dl.uom, dl.unit_price, dl.qty
            FROM document_lines dl
            JOIN products p ON dl.product_id = p.id
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量
        let received: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id, COALESCE(SUM(dl.qty), 0)
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            WHERE d.source_doc_id = $1 
              AND d.doc_type = 'GRN' 
              AND d.status = 'approved'
            GROUP BY dl.product_id
            "#
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> = 
            received.into_iter().collect();

        let items: Vec<PoReceiptItem> = po_lines
            .into_iter()
            .map(|row| {
                let received_qty = received_map.get(&row.product_id).copied().unwrap_or(Decimal::ZERO);
                PoReceiptItem {
                    product_id: row.product_id,
                    product_sku: row.sku,
                    product_name: row.name,
                    base_uom: row.base_uom,
                    uom: row.uom,
                    unit_price: row.unit_price,
                    ordered_qty: row.qty,
                    received_qty,
                    remaining_qty: row.qty - received_qty,
                }
            })
            .collect();

        let total_ordered: Decimal = items.iter().map(|i| i.ordered_qty).sum();
        let total_received: Decimal = items.iter().map(|i| i.received_qty).sum();

        let status = receipt_status_label(total_received, total_ordered).to_string();

        Ok(PoReceiptStatus {
            po_id,
            po_no: po.doc_no,
            status,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{next_seq_from_last_no, receipt_status_label};
    use rust_decimal::Decimal;

    // --- next_seq_from_last_no ---

    #[test]
    fn test_next_seq_no_previous() {
        assert_eq!(next_seq_from_last_no(None), 1);
    }

    #[test]
    fn test_next_seq_from_valid_doc_no() {
        assert_eq!(next_seq_from_last_no(Some("GRN-260314-005")), 6);
        assert_eq!(next_seq_from_last_no(Some("GRN-260314-001")), 2);
        assert_eq!(next_seq_from_last_no(Some("GRN-260314-099")), 100);
    }

    #[test]
    fn test_next_seq_non_numeric_seq_defaults_to_one() {
        assert_eq!(next_seq_from_last_no(Some("GRN-260314-abc")), 1);
    }

    #[test]
    fn test_next_seq_too_few_segments() {
        assert_eq!(next_seq_from_last_no(Some("GRN-260314")), 1);
        assert_eq!(next_seq_from_last_no(Some("GRN")), 1);
    }

    // --- receipt_status_label ---

    #[test]
    fn test_receipt_status_pending_when_zero_received() {
        assert_eq!(
            receipt_status_label(Decimal::ZERO, Decimal::new(100, 0)),
            "pending"
        );
    }

    #[test]
    fn test_receipt_status_partial_when_some_received() {
        assert_eq!(
            receipt_status_label(Decimal::new(50, 0), Decimal::new(100, 0)),
            "partial"
        );
    }

    #[test]
    fn test_receipt_status_complete_when_fully_received() {
        assert_eq!(
            receipt_status_label(Decimal::new(100, 0), Decimal::new(100, 0)),
            "complete"
        );
    }
}
