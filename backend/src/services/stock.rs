use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    models::{
        DocType, Document, DocumentLine, InventoryOnHand, InventoryQuery, LowStockAlert,
        StockDirection, StockLedgerDetail, StockLedgerQuery,
    },
    AppError, Result,
};

pub struct StockService;

impl StockService {
    /// 處理單據核准後的庫存變動
    pub async fn process_document(
        tx: &mut Transaction<'_, Postgres>,
        document: &Document,
        lines: &[DocumentLine],
    ) -> Result<()> {
        for line in lines {
            match document.doc_type {
                DocType::GRN => {
                    // 採購入庫：增加庫存
                    let warehouse_id = document.warehouse_id
                        .ok_or_else(|| AppError::BusinessRule("Warehouse is required for GRN".to_string()))?;
                    
                    Self::create_ledger_entry(
                        tx,
                        warehouse_id,
                        line.product_id,
                        document,
                        line,
                        StockDirection::In,
                        line.qty,
                        line.unit_price,
                    ).await?;
                }
                DocType::PR => {
                    // 採購退貨：減少庫存
                    let warehouse_id = document.warehouse_id
                        .ok_or_else(|| AppError::BusinessRule("Warehouse is required for PR".to_string()))?;
                    
                    // 檢查庫存
                    Self::check_stock_available(tx, warehouse_id, line.product_id, line.qty).await?;
                    
                    Self::create_ledger_entry(
                        tx,
                        warehouse_id,
                        line.product_id,
                        document,
                        line,
                        StockDirection::Out,
                        line.qty,
                        line.unit_price,
                    ).await?;
                }
                DocType::DO => {
                    // 銷售出庫：減少庫存
                    let warehouse_id = document.warehouse_id
                        .ok_or_else(|| AppError::BusinessRule("Warehouse is required for DO".to_string()))?;
                    
                    // 檢查庫存
                    Self::check_stock_available(tx, warehouse_id, line.product_id, line.qty).await?;
                    
                    Self::create_ledger_entry(
                        tx,
                        warehouse_id,
                        line.product_id,
                        document,
                        line,
                        StockDirection::Out,
                        line.qty,
                        line.unit_price,
                    ).await?;
                }
                DocType::TR => {
                    // 調撥：從來源倉減少，目標倉增加
                    let from_warehouse = document.warehouse_from_id
                        .ok_or_else(|| AppError::BusinessRule("Source warehouse is required for transfer".to_string()))?;
                    let to_warehouse = document.warehouse_to_id
                        .ok_or_else(|| AppError::BusinessRule("Target warehouse is required for transfer".to_string()))?;
                    
                    // 檢查來源倉庫存
                    Self::check_stock_available(tx, from_warehouse, line.product_id, line.qty).await?;
                    
                    // 從來源倉扣減
                    Self::create_ledger_entry(
                        tx,
                        from_warehouse,
                        line.product_id,
                        document,
                        line,
                        StockDirection::TransferOut,
                        line.qty,
                        None,
                    ).await?;
                    
                    // 增加到目標倉
                    Self::create_ledger_entry(
                        tx,
                        to_warehouse,
                        line.product_id,
                        document,
                        line,
                        StockDirection::TransferIn,
                        line.qty,
                        None,
                    ).await?;
                }
                DocType::ADJ => {
                    // 調整：正數增加，負數減少
                    let warehouse_id = document.warehouse_id
                        .ok_or_else(|| AppError::BusinessRule("Warehouse is required for adjustment".to_string()))?;
                    
                    if line.qty > Decimal::ZERO {
                        Self::create_ledger_entry(
                            tx,
                            warehouse_id,
                            line.product_id,
                            document,
                            line,
                            StockDirection::AdjustIn,
                            line.qty,
                            line.unit_price,
                        ).await?;
                    } else {
                        // 檢查庫存
                        Self::check_stock_available(tx, warehouse_id, line.product_id, -line.qty).await?;
                        
                        Self::create_ledger_entry(
                            tx,
                            warehouse_id,
                            line.product_id,
                            document,
                            line,
                            StockDirection::AdjustOut,
                            -line.qty,
                            line.unit_price,
                        ).await?;
                    }
                }
                _ => {
                    // PO, SO, STK 等不直接影響庫存的單據
                }
            }
        }

        Ok(())
    }

    /// 建立庫存流水記錄
    async fn create_ledger_entry(
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        product_id: Uuid,
        document: &Document,
        line: &DocumentLine,
        direction: StockDirection,
        qty: Decimal,
        unit_price: Option<Decimal>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO stock_ledger (
                id, warehouse_id, product_id, trx_date, doc_type, doc_id, doc_no,
                line_id, direction, qty_base, unit_cost, batch_no, expiry_date, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(warehouse_id)
        .bind(product_id)
        .bind(Utc::now())
        .bind(&document.doc_type)
        .bind(document.id)
        .bind(&document.doc_no)
        .bind(line.id)
        .bind(direction)
        .bind(qty)
        .bind(unit_price)
        .bind(&line.batch_no)
        .bind(line.expiry_date)
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
        let on_hand: Decimal = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(
                CASE 
                    WHEN direction IN ('in', 'transfer_in', 'adjust_in') THEN qty_base
                    WHEN direction IN ('out', 'transfer_out', 'adjust_out') THEN -qty_base
                END
            ), 0) as qty
            FROM stock_ledger
            WHERE warehouse_id = $1 AND product_id = $2
            "#
        )
        .bind(warehouse_id)
        .bind(product_id)
        .fetch_one(&mut **tx)
        .await?;

        if on_hand < required_qty {
            let product_name: String = sqlx::query_scalar(
                "SELECT name FROM products WHERE id = $1"
            )
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

    /// 查詢庫存現況
    pub async fn get_on_hand(pool: &PgPool, query: &InventoryQuery) -> Result<Vec<InventoryOnHand>> {
        let mut sql = String::from(
            r#"
            SELECT 
                w.id as warehouse_id,
                w.code as warehouse_code,
                w.name as warehouse_name,
                p.id as product_id,
                p.sku as product_sku,
                p.name as product_name,
                p.base_uom,
                COALESCE(SUM(
                    CASE 
                        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                        ELSE 0
                    END
                ), 0) as qty_on_hand,
                AVG(sl.unit_cost) as avg_cost,
                p.safety_stock,
                p.reorder_point
            FROM warehouses w
            CROSS JOIN products p
            LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
            WHERE w.is_active = true AND p.is_active = true
            "#
        );

        if query.warehouse_id.is_some() {
            sql.push_str(" AND w.id = $1");
        }

        sql.push_str(
            r#"
            GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.safety_stock, p.reorder_point
            HAVING COALESCE(SUM(
                CASE 
                    WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                    WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                    ELSE 0
                END
            ), 0) != 0
            ORDER BY w.code, p.sku
            "#
        );

        let inventory = if let Some(warehouse_id) = query.warehouse_id {
            sqlx::query_as::<_, InventoryOnHand>(&sql)
                .bind(warehouse_id)
                .fetch_all(pool)
                .await?
        } else {
            let simple_sql = r#"
                SELECT 
                    w.id as warehouse_id,
                    w.code as warehouse_code,
                    w.name as warehouse_name,
                    p.id as product_id,
                    p.sku as product_sku,
                    p.name as product_name,
                    p.base_uom,
                    COALESCE(SUM(
                        CASE 
                            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                            WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                            ELSE 0
                        END
                    ), 0) as qty_on_hand,
                    AVG(sl.unit_cost) as avg_cost,
                    p.safety_stock,
                    p.reorder_point
                FROM warehouses w
                CROSS JOIN products p
                LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
                WHERE w.is_active = true AND p.is_active = true
                GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.safety_stock, p.reorder_point
                HAVING COALESCE(SUM(
                    CASE 
                        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                        ELSE 0
                    END
                ), 0) != 0
                ORDER BY w.code, p.sku
            "#;
            sqlx::query_as::<_, InventoryOnHand>(simple_sql)
                .fetch_all(pool)
                .await?
        };

        Ok(inventory)
    }

    /// 查詢庫存流水
    pub async fn get_ledger(pool: &PgPool, _query: &StockLedgerQuery) -> Result<Vec<StockLedgerDetail>> {
        let ledger = sqlx::query_as::<_, StockLedgerDetail>(
            r#"
            SELECT 
                sl.id,
                sl.warehouse_id,
                w.name as warehouse_name,
                sl.product_id,
                p.sku as product_sku,
                p.name as product_name,
                sl.trx_date,
                sl.doc_type,
                sl.doc_id,
                sl.doc_no,
                sl.direction,
                sl.qty_base,
                sl.unit_cost,
                sl.batch_no,
                sl.expiry_date,
                NULL::numeric as running_balance
            FROM stock_ledger sl
            INNER JOIN warehouses w ON sl.warehouse_id = w.id
            INNER JOIN products p ON sl.product_id = p.id
            ORDER BY sl.trx_date DESC, sl.created_at DESC
            LIMIT 1000
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(ledger)
    }

    /// 查詢低庫存警示
    pub async fn get_low_stock_alerts(pool: &PgPool) -> Result<Vec<LowStockAlert>> {
        let alerts = sqlx::query_as::<_, InventoryOnHand>(
            r#"
            SELECT 
                w.id as warehouse_id,
                w.code as warehouse_code,
                w.name as warehouse_name,
                p.id as product_id,
                p.sku as product_sku,
                p.name as product_name,
                p.base_uom,
                COALESCE(SUM(
                    CASE 
                        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                        ELSE 0
                    END
                ), 0) as qty_on_hand,
                AVG(sl.unit_cost) as avg_cost,
                p.safety_stock,
                p.reorder_point
            FROM warehouses w
            CROSS JOIN products p
            LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
            WHERE w.is_active = true AND p.is_active = true
              AND p.safety_stock IS NOT NULL
            GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.safety_stock, p.reorder_point
            HAVING COALESCE(SUM(
                CASE 
                    WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                    WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                    ELSE 0
                END
            ), 0) < p.safety_stock
            ORDER BY w.code, p.sku
            "#
        )
        .fetch_all(pool)
        .await?;

        let result: Vec<LowStockAlert> = alerts
            .into_iter()
            .filter_map(|inv| {
                let safety_stock = inv.safety_stock;
                let reorder_point = inv.reorder_point;
                let stock_status = if inv.qty_on_hand <= rust_decimal::Decimal::ZERO {
                    "out_of_stock".to_string()
                } else if let Some(ss) = safety_stock {
                    if inv.qty_on_hand < ss {
                        "low".to_string()
                    } else {
                        "ok".to_string()
                    }
                } else {
                    "ok".to_string()
                };
                Some(LowStockAlert {
                    warehouse_id: inv.warehouse_id,
                    warehouse_name: inv.warehouse_name,
                    product_id: inv.product_id,
                    product_sku: inv.product_sku,
                    product_name: inv.product_name,
                    base_uom: inv.base_uom,
                    qty_on_hand: inv.qty_on_hand,
                    safety_stock,
                    reorder_point,
                    stock_status,
                })
            })
            .collect();

        Ok(result)
    }
}
