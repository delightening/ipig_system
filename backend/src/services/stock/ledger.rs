use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    models::{DocType, Document, DocumentLine, StockDirection, StockLedgerDetail, StockLedgerQuery},
    AppError, Result,
};

use super::StockService;

/// 庫存流水記錄所需參數
struct LedgerEntryParams<'a> {
    warehouse_id: Uuid,
    product_id: Uuid,
    document: &'a Document,
    line: &'a DocumentLine,
    direction: StockDirection,
    qty: Decimal,
    unit_price: Option<Decimal>,
}

impl StockService {
    /// 處理單據核准後的庫存變動
    pub async fn process_document(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
    ) -> Result<()> {
        for line in lines {
            Self::process_single_line(tx, document, line).await?;
        }

        let affected_items = Self::collect_affected_items(document, lines);
        for (warehouse_id, product_id) in affected_items {
            Self::update_inventory_snapshot(tx, warehouse_id, product_id).await?;
        }
        Ok(())
    }

    /// 處理單一明細行的庫存變動
    async fn process_single_line(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        match document.doc_type {
            DocType::GRN => Self::process_grn(tx, document, line).await?,
            DocType::PR => Self::process_return_out(tx, document, line, "PR").await?,
            DocType::DO => Self::process_return_out(tx, document, line, "DO").await?,
            DocType::TR => Self::process_transfer(tx, document, line).await?,
            DocType::ADJ => Self::process_adjustment(tx, document, line).await?,
            DocType::SR | DocType::RTN => Self::process_return_in(tx, document, line).await?,
            _ => {} // PO, SO, STK 等不直接影響庫存
        }
        Ok(())
    }

    /// GRN 採購入庫
    async fn process_grn(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = document.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("Warehouse is required for GRN".to_string())
        })?;

        Self::create_ledger_entry(tx, LedgerEntryParams {
            warehouse_id, product_id: line.product_id, document, line,
            direction: StockDirection::In, qty: line.qty, unit_price: line.unit_price,
        }).await?;

        if let Some(storage_location_id) = line.storage_location_id {
            Self::upsert_storage_location_inventory(
                tx, storage_location_id, line.product_id,
                line.qty, line.batch_no.clone(), line.expiry_date,
            ).await?;
        }
        Ok(())
    }

    /// PR/DO 採購退貨或銷貨出庫（扣減庫存）
    async fn process_return_out(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
        doc_label: &str,
    ) -> Result<()> {
        let warehouse_id = document.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule(format!("Warehouse is required for {}", doc_label))
        })?;
        Self::check_stock_available(tx, warehouse_id, line.product_id, line.qty).await?;
        Self::create_ledger_entry(tx, LedgerEntryParams {
            warehouse_id, product_id: line.product_id, document, line,
            direction: StockDirection::Out, qty: line.qty, unit_price: line.unit_price,
        }).await
    }

    /// TR 調撥
    async fn process_transfer(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let from_warehouse = document.warehouse_from_id.ok_or_else(|| {
            AppError::BusinessRule("Source warehouse is required for transfer".to_string())
        })?;
        let to_warehouse = document.warehouse_to_id.ok_or_else(|| {
            AppError::BusinessRule("Target warehouse is required for transfer".to_string())
        })?;

        Self::check_stock_available(tx, from_warehouse, line.product_id, line.qty).await?;

        Self::create_ledger_entry(tx, LedgerEntryParams {
            warehouse_id: from_warehouse, product_id: line.product_id, document, line,
            direction: StockDirection::TransferOut, qty: line.qty, unit_price: None,
        }).await?;
        Self::create_ledger_entry(tx, LedgerEntryParams {
            warehouse_id: to_warehouse, product_id: line.product_id, document, line,
            direction: StockDirection::TransferIn, qty: line.qty, unit_price: None,
        }).await
    }

    /// ADJ 調整
    async fn process_adjustment(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = document.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("Warehouse is required for adjustment".to_string())
        })?;

        if line.qty > Decimal::ZERO {
            Self::create_ledger_entry(tx, LedgerEntryParams {
                warehouse_id, product_id: line.product_id, document, line,
                direction: StockDirection::AdjustIn, qty: line.qty, unit_price: line.unit_price,
            }).await?;
        } else {
            Self::check_stock_available(tx, warehouse_id, line.product_id, -line.qty).await?;
            Self::create_ledger_entry(tx, LedgerEntryParams {
                warehouse_id, product_id: line.product_id, document, line,
                direction: StockDirection::AdjustOut, qty: -line.qty, unit_price: line.unit_price,
            }).await?;
        }

        if let Some(storage_location_id) = line.storage_location_id {
            Self::upsert_storage_location_inventory(
                tx, storage_location_id, line.product_id,
                line.qty, line.batch_no.clone(), line.expiry_date,
            ).await?;
        }
        Ok(())
    }

    /// SR/RTN 銷貨退貨（庫存增加）
    async fn process_return_in(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        line: &DocumentLine,
    ) -> Result<()> {
        let warehouse_id = document.warehouse_id.ok_or_else(|| {
            AppError::BusinessRule("Warehouse is required for sales return".to_string())
        })?;
        Self::create_ledger_entry(tx, LedgerEntryParams {
            warehouse_id, product_id: line.product_id, document, line,
            direction: StockDirection::In, qty: line.qty, unit_price: line.unit_price,
        }).await
    }

    /// 收集所有涉及的 (warehouse_id, product_id) 組合
    fn collect_affected_items(
        document: &Document,
        lines: &[DocumentLine],
    ) -> std::collections::HashSet<(Uuid, Uuid)> {
        let mut items = std::collections::HashSet::new();
        for line in lines {
            match document.doc_type {
                DocType::GRN | DocType::PR | DocType::DO | DocType::ADJ | DocType::SR | DocType::RTN => {
                    if let Some(warehouse_id) = document.warehouse_id {
                        items.insert((warehouse_id, line.product_id));
                    }
                }
                DocType::TR => {
                    if let (Some(from_wh), Some(to_wh)) = (document.warehouse_from_id, document.warehouse_to_id) {
                        items.insert((from_wh, line.product_id));
                        items.insert((to_wh, line.product_id));
                    }
                }
                _ => {}
            }
        }
        items
    }

    /// 更新庫存快照 (核准單據後呼叫)
    async fn update_inventory_snapshot(
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        product_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base, avg_cost, updated_at)
            SELECT
                $1, $2,
                COALESCE(SUM(
                    CASE
                        WHEN direction IN ('in', 'transfer_in', 'adjust_in') THEN qty_base
                        WHEN direction IN ('out', 'transfer_out', 'adjust_out') THEN -qty_base
                        ELSE 0
                    END
                ), 0),
                AVG(unit_cost),
                NOW()
            FROM stock_ledger
            WHERE warehouse_id = $1 AND product_id = $2
            ON CONFLICT (warehouse_id, product_id) DO UPDATE
            SET
                on_hand_qty_base = EXCLUDED.on_hand_qty_base,
                avg_cost = EXCLUDED.avg_cost,
                updated_at = NOW()
            "#,
        )
        .bind(warehouse_id)
        .bind(product_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 建立庫存流水記錄
    async fn create_ledger_entry(
        tx: &mut Transaction<'_, Postgres>,
        params: LedgerEntryParams<'_>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stock_ledger (
                id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no,
                line_id, direction, qty_base, unit_cost, batch_no, expiry_date, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(params.warehouse_id)
        .bind(params.product_id)
        .bind(Utc::now())
        .bind(params.document.doc_type)
        .bind(params.document.id)
        .bind(&params.document.doc_no)
        .bind(params.line.id)
        .bind(params.direction)
        .bind(params.qty)
        .bind(params.unit_price)
        .bind(&params.line.batch_no)
        .bind(params.line.expiry_date)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 更新/新增儲位庫存 (GRN 入庫時使用)
    async fn upsert_storage_location_inventory(
        tx: &mut Transaction<'_, Postgres>,
        storage_location_id: Uuid,
        product_id: Uuid,
        qty: Decimal,
        batch_no: Option<String>,
        expiry_date: Option<chrono::NaiveDate>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO storage_location_inventory (
                id, storage_location_id, product_id, on_hand_qty, batch_no, expiry_date, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1900-01-01'::date))
            DO UPDATE SET
                on_hand_qty = storage_location_inventory.on_hand_qty + EXCLUDED.on_hand_qty,
                updated_at = NOW()
            "#
        )
        .bind(Uuid::new_v4())
        .bind(storage_location_id)
        .bind(product_id)
        .bind(qty)
        .bind(&batch_no)
        .bind(expiry_date)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE storage_locations SET
                current_count = (
                    SELECT COUNT(DISTINCT product_id)
                    FROM storage_location_inventory
                    WHERE storage_location_id = $1
                ),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(storage_location_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 檢查庫存是否足夠
    async fn check_stock_available(
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        product_id: Uuid,
        required_qty: Decimal,
    ) -> Result<()> {
        let on_hand: Option<Decimal> = sqlx::query_scalar(
            r#"
            SELECT on_hand_qty_base
            FROM inventory_snapshots
            WHERE warehouse_id = $1 AND product_id = $2
            "#,
        )
        .bind(warehouse_id)
        .bind(product_id)
        .fetch_optional(&mut **tx)
        .await?;

        let on_hand = on_hand.unwrap_or(Decimal::ZERO);
        if on_hand < required_qty {
            let product_name: String =
                sqlx::query_scalar("SELECT name FROM products WHERE id = $1")
                    .bind(product_id)
                    .fetch_one(&mut **tx)
                    .await?;

            return Err(AppError::BusinessRule(format!(
                "Insufficient stock for product '{}'. Available: {}, Required: {}",
                product_name, on_hand, required_qty
            )));
        }
        Ok(())
    }

    /// 查詢庫存流水（使用 QueryBuilder 避免 format! 動態 SQL）
    pub async fn get_ledger(
        pool: &PgPool,
        query: &StockLedgerQuery,
    ) -> Result<Vec<StockLedgerDetail>> {
        use sqlx::QueryBuilder;

        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            r#"
            SELECT
                sl.id, sl.warehouse_id, w.name as warehouse_name,
                sl.product_id, p.sku as product_sku, p.name as product_name,
                sl.trx_date, sl.doc_type, sl.doc_id, sl.doc_no,
                sl.direction, sl.qty_base, sl.unit_cost,
                sl.batch_no, sl.expiry_date,
                NULL::numeric as running_balance,
                d.iacuc_no
            FROM stock_ledger sl
            INNER JOIN warehouses w ON sl.warehouse_id = w.id
            INNER JOIN products p ON sl.product_id = p.id
            LEFT JOIN documents d ON sl.doc_id = d.id
            WHERE 1=1
            "#,
        );

        if let Some(warehouse_id) = query.warehouse_id {
            qb.push(" AND sl.warehouse_id = ");
            qb.push_bind(warehouse_id);
        }
        if let Some(product_id) = query.product_id {
            qb.push(" AND sl.product_id = ");
            qb.push_bind(product_id);
        }
        if let Some(batch_no) = &query.batch_no {
            qb.push(" AND sl.batch_no = ");
            qb.push_bind(batch_no);
        }
        if let Some(date_from) = query.date_from {
            qb.push(" AND sl.trx_date >= ");
            qb.push_bind(date_from);
        }
        if let Some(date_to) = query.date_to {
            qb.push(" AND sl.trx_date <= ");
            qb.push_bind(date_to);
        }
        if let Some(doc_type) = query.doc_type {
            qb.push(" AND sl.doc_type = ");
            qb.push_bind(doc_type);
        }

        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);
        qb.push(" ORDER BY sl.trx_date DESC, sl.created_at DESC LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let ledger = qb.build_query_as::<StockLedgerDetail>().fetch_all(pool).await?;
        Ok(ledger)
    }
}
