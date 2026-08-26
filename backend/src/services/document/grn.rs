use rust_decimal::Decimal;

use crate::time;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{
    models::{
        DocStatus, DocType, Document, DocumentLine, DocumentWithLines, PoReceiptItem,
        PoReceiptStatus,
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
    /// 本行 `uom` 對 `base_uom` 的換算率，恆 > 0（SQL 端已把 NULL / 非正數收斂成 1）。
    factor_to_base: Decimal,
}

use super::DocumentService;

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
    /// 從採購單建立入庫單（部分入庫 / 手動建立用）
    pub async fn create_additional_grn(
        pool: &PgPool,
        po_id: Uuid,
        created_by: Uuid,
    ) -> Result<DocumentWithLines> {
        // 檢查採購單狀態
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'",
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        if po.status != DocStatus::Approved {
            return Err(AppError::BusinessRule(
                "Purchase order must be approved".to_string(),
            ));
        }

        // 取得採購單明細
        let po_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量（換算成 base_uom 再加總）。
        // PO 與 GRN 對同一品項可以填不同單位（採購論箱、入庫論盒），直接 SUM(qty) 會把
        // 箱與盒相加。查無換算列即代表該行就是 base_uom（factor 1）——這個推論由
        // `DocumentService::assert_lines_uom_defined` 在建/改單時保證。
        let received_qty: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id, COALESCE(SUM(dl.qty * CASE WHEN c.factor_to_base > 0 THEN c.factor_to_base ELSE 1 END), 0) as received
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = dl.product_id AND c.uom = dl.uom
            WHERE d.source_doc_id = $1
              AND d.doc_type = 'GRN'
              AND d.status = 'approved'
            GROUP BY dl.product_id
            "#,
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> =
            received_qty.into_iter().collect();

        // 逐 PO 明細行的換算率（同一品項可能在兩行用不同單位，故以 line id 為 key）
        let po_factors: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.id, CASE WHEN c.factor_to_base > 0 THEN c.factor_to_base ELSE 1 END
            FROM document_lines dl
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = dl.product_id AND c.uom = dl.uom
            WHERE dl.document_id = $1
            "#,
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;
        let po_factor_map: std::collections::HashMap<Uuid, Decimal> =
            po_factors.into_iter().collect();

        // 計算剩餘數量：兩邊都換到 base_uom 相減，再除回本行單位開立 GRN。
        let remaining_lines: Vec<_> = po_lines
            .iter()
            .filter_map(|line| {
                let factor = po_factor_map.get(&line.id).copied().unwrap_or(Decimal::ONE);
                let received_base = received_map
                    .get(&line.product_id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let remaining_base = line.qty * factor - received_base;
                if remaining_base > Decimal::ZERO {
                    Some((line.clone(), remaining_base / factor))
                } else {
                    None
                }
            })
            .collect();

        if remaining_lines.is_empty() {
            return Err(AppError::BusinessRule(
                "All items have been received".to_string(),
            ));
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
            "#,
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
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(grn_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(*remaining) // 預設為剩餘數量
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

    /// Medium-2 (#290相關)：GRN 核准守衛 — 確保核准後「同品項累計已核准入庫量」
    /// 不超過採購量（防超量入庫）。
    ///
    /// 在核准 tx 內、GRN 已標 `approved` 之後呼叫 → `received` 已含本張 GRN。違反即
    /// 回 Conflict 使整個核准 tx 回滾。
    ///
    /// 併發（bot review #629）：先對 PO 行 `FOR UPDATE`，序列化同一 PO 的並發 GRN 核准——
    /// Read Committed 下兩個並發核准互看不到對方未 commit 的 `approved`，否則皆通過守衛 → 超收。
    /// 比對：以 UNION ALL 彙整「PO 採購量」與「已核准 GRN 入庫量」再 GROUP BY 比較，連
    /// 「PO 未採購品項卻被入庫」（ordered=0、received>0）也能攔截（原 LEFT JOIN 以 PO 明細
    /// 為主體會漏掉）。
    /// 註：不在 `update_po_receipt_status`（被 recompute-all 迴圈共用）內 raise，避免對既有
    /// legacy 超收資料炸錯。
    pub(crate) async fn ensure_no_over_receipt(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po_id: Uuid,
    ) -> Result<()> {
        // 序列化同一 PO 的並發 GRN 核准（鎖 PO 行直到本 tx 結束）
        sqlx::query("SELECT 1 FROM documents WHERE id = $1 FOR UPDATE")
            .bind(po_id)
            .execute(&mut **tx)
            .await?;

        let over: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT product_id
            FROM (
                SELECT pl.product_id,
                       SUM(pl.qty * CASE WHEN pc.factor_to_base > 0 THEN pc.factor_to_base ELSE 1 END) AS ordered,
                       0::numeric AS received
                FROM document_lines pl
                LEFT JOIN product_uom_conversions pc
                       ON pc.product_id = pl.product_id AND pc.uom = pl.uom
                WHERE pl.document_id = $1
                GROUP BY pl.product_id
                UNION ALL
                SELECT gl.product_id,
                       0::numeric AS ordered,
                       SUM(gl.qty * CASE WHEN gc.factor_to_base > 0 THEN gc.factor_to_base ELSE 1 END) AS received
                FROM documents g
                JOIN document_lines gl ON g.id = gl.document_id
                LEFT JOIN product_uom_conversions gc
                       ON gc.product_id = gl.product_id AND gc.uom = gl.uom
                WHERE g.source_doc_id = $1 AND g.doc_type = 'GRN' AND g.status = 'approved'
                GROUP BY gl.product_id
            ) t
            GROUP BY product_id
            HAVING SUM(received) > SUM(ordered)
            LIMIT 1
            "#,
        )
        .bind(po_id)
        .fetch_optional(&mut **tx)
        .await?;

        if over.is_some() {
            return Err(AppError::Conflict(
                "入庫數量超過採購量，無法核准（請檢查是否重複入庫）".to_string(),
            ));
        }
        Ok(())
    }

    /// GRN 核准後，重新計算並回寫 PO 的 receipt_status
    pub(crate) async fn update_po_receipt_status(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        po_id: Uuid,
    ) -> Result<()> {
        let row: (Decimal, Decimal) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(pl.qty * CASE WHEN pc.factor_to_base > 0 THEN pc.factor_to_base ELSE 1 END), 0) AS total_ordered,
                COALESCE((
                    SELECT SUM(gl.qty * CASE WHEN gc.factor_to_base > 0 THEN gc.factor_to_base ELSE 1 END)
                    FROM documents g
                    JOIN document_lines gl ON g.id = gl.document_id
                    LEFT JOIN product_uom_conversions gc
                           ON gc.product_id = gl.product_id AND gc.uom = gl.uom
                    WHERE g.source_doc_id = $1
                      AND g.doc_type = 'GRN'
                      AND g.status = 'approved'
                ), 0) AS total_received
            FROM document_lines pl
            LEFT JOIN product_uom_conversions pc
                   ON pc.product_id = pl.product_id AND pc.uom = pl.uom
            WHERE pl.document_id = $1
            "#,
        )
        .bind(po_id)
        .fetch_one(&mut **tx)
        .await?;

        let status = receipt_status_label(row.1, row.0);

        sqlx::query("UPDATE documents SET receipt_status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(po_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    /// 取得採購單入庫狀態
    pub async fn get_po_receipt_status(pool: &PgPool, po_id: Uuid) -> Result<PoReceiptStatus> {
        let po = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE id = $1 AND doc_type = 'PO'",
        )
        .bind(po_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".to_string()))?;

        // 取得採購單明細
        let po_lines: Vec<PoLineRow> = sqlx::query_as(
            r#"
            SELECT dl.product_id, p.sku, p.name, p.base_uom, dl.uom, dl.unit_price, dl.qty,
                   CASE WHEN c.factor_to_base > 0 THEN c.factor_to_base ELSE 1 END AS factor_to_base
            FROM document_lines dl
            JOIN products p ON dl.product_id = p.id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = dl.product_id AND c.uom = dl.uom
            WHERE dl.document_id = $1
            ORDER BY dl.line_no
            "#,
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        // 取得已入庫數量（換算成 base_uom 再加總，理由同 create_additional_grn）
        let received: Vec<(Uuid, Decimal)> = sqlx::query_as(
            r#"
            SELECT dl.product_id,
                   COALESCE(SUM(dl.qty * CASE WHEN c.factor_to_base > 0
                                              THEN c.factor_to_base ELSE 1 END), 0)
            FROM documents d
            JOIN document_lines dl ON d.id = dl.document_id
            LEFT JOIN product_uom_conversions c
                   ON c.product_id = dl.product_id AND c.uom = dl.uom
            WHERE d.source_doc_id = $1
              AND d.doc_type = 'GRN'
              AND d.status = 'approved'
            GROUP BY dl.product_id
            "#,
        )
        .bind(po_id)
        .fetch_all(pool)
        .await?;

        let received_map: std::collections::HashMap<Uuid, Decimal> = received.into_iter().collect();

        // 每行的數量以**該行的單位**呈現（採購論箱就顯示箱），但跨行加總與 status 判定
        // 一律走 base_uom——否則 3 箱 + 2 盒會被加成 5。
        let mut ordered_base = Decimal::ZERO;
        let mut received_base_total = Decimal::ZERO;
        let items: Vec<PoReceiptItem> = po_lines
            .into_iter()
            .map(|row| {
                let received_line_base = received_map
                    .get(&row.product_id)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                ordered_base += row.qty * row.factor_to_base;
                received_base_total += received_line_base;
                let received_qty = received_line_base / row.factor_to_base;
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

        let status = receipt_status_label(received_base_total, ordered_base).to_string();

        Ok(PoReceiptStatus {
            po_id,
            po_no: po.doc_no,
            status,
            items,
        })
    }

    /// 重新計算所有已核准 PO 的入庫狀態
    pub async fn recalculate_all_po_receipt_status(pool: &PgPool) -> Result<i64> {
        // 取得所有已核准的 PO
        let pos: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM documents WHERE doc_type = 'PO' AND status = 'approved'",
        )
        .fetch_all(pool)
        .await?;

        let mut tx = pool.begin().await?;
        let mut count = 0i64;
        for (po_id,) in &pos {
            Self::update_po_receipt_status(&mut tx, *po_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed recalculating PO {po_id}: {e}");
                    e
                })?;
            count += 1;
        }
        tx.commit().await?;

        tracing::info!("Recalculated receipt_status for {} POs", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::receipt_status_label;
    use rust_decimal::Decimal;

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
