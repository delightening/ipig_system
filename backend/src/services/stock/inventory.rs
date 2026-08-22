use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AssignUnassignedRequest, DocType, Document, DocumentAuditSnapshot,
        DocumentLine, InventoryOnHand, InventoryQuery, LowStockTotal, LowStockWarehouseQty,
        UnassignedInventory, UnassignedSourceDoc, UnassignedSourceQuery,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, DocumentService,
    },
    time, AppError, Result,
};

use super::StockService;

/// `v_grn_line_unshelved` 查詢列，供分配時 FIFO 攤扣（最舊來源先扣）。
#[derive(Debug, sqlx::FromRow)]
struct GrnUnshelvedRow {
    document_line_id: Uuid,
    batch_no: Option<String>,
    expiry_date: Option<chrono::NaiveDate>,
    remaining_unshelved: Decimal,
}

/// `get_low_stock_totals` 的扁平查詢列（一品項 × 一倉庫），於 Rust 端彙總為 LowStockTotal
#[derive(Debug, sqlx::FromRow)]
struct LowStockTotalRow {
    product_id: Uuid,
    product_sku: String,
    product_name: String,
    base_uom: String,
    safety_stock: Option<Decimal>,
    reorder_point: Option<Decimal>,
    total_qty: Decimal,
    warehouse_id: Uuid,
    warehouse_code: String,
    warehouse_name: String,
    wh_qty: Decimal,
}

/// storage_location_inventory 查詢共用的動態 filter 建構器（keyword / product_id / batch_no）
struct SliFilterBuilder {
    keyword: String,
    product: String,
    batch: String,
}

impl SliFilterBuilder {
    fn new(start_idx: u8, query: &InventoryQuery) -> Self {
        let mut idx = start_idx;
        let keyword = if query.keyword.as_ref().is_some_and(|k| !k.is_empty()) {
            let f = format!(
                " AND (p.name ILIKE '%' || ${idx} || '%' OR p.sku ILIKE '%' || ${idx} || '%')"
            );
            idx += 1;
            f
        } else {
            String::new()
        };
        let product = if query.product_id.is_some() {
            let f = format!(" AND p.id = ${idx}");
            idx += 1;
            f
        } else {
            String::new()
        };
        let batch = if query.batch_no.as_ref().is_some_and(|b| !b.is_empty()) {
            let f = format!(" AND sli.batch_no ILIKE '%' || ${idx} || '%'");
            idx += 1;
            f
        } else {
            String::new()
        };
        let _ = idx;
        Self {
            keyword,
            product,
            batch,
        }
    }

    /// 按建構順序 bind 參數（keyword → product_id → batch_no）
    fn bind_all<'q>(
        &self,
        mut q: sqlx::query::QueryAs<
            'q,
            sqlx::Postgres,
            InventoryOnHand,
            sqlx::postgres::PgArguments,
        >,
        query: &'q InventoryQuery,
    ) -> sqlx::query::QueryAs<'q, sqlx::Postgres, InventoryOnHand, sqlx::postgres::PgArguments>
    {
        if let Some(keyword) = &query.keyword {
            if !keyword.is_empty() {
                q = q.bind(keyword);
            }
        }
        if let Some(product_id) = query.product_id {
            q = q.bind(product_id);
        }
        if let Some(batch_no) = &query.batch_no {
            if !batch_no.is_empty() {
                q = q.bind(batch_no);
            }
        }
        q
    }
}

impl StockService {
    /// 查詢庫存現況
    /// - 指定 storage_location_id：查 storage_location_inventory（貨架級）
    /// - 指定 warehouse_id 或全部：查 stock_ledger（倉庫級）
    pub async fn get_on_hand(
        pool: &PgPool,
        query: &InventoryQuery,
    ) -> Result<Vec<InventoryOnHand>> {
        if let Some(days) = query.expiry_within_days {
            return Self::get_on_hand_expiry(pool, query, days).await;
        }

        if let Some(loc_id) = query.storage_location_id {
            return Self::get_on_hand_by_location(pool, query, loc_id).await;
        }

        if let Some(warehouse_id) = query.warehouse_id {
            return Self::get_on_hand_by_warehouse(pool, query, warehouse_id).await;
        }

        if let Some(product_id) = query.product_id {
            return Self::get_on_hand_product_across_warehouses(pool, product_id).await;
        }

        Self::get_on_hand_all_warehouses(pool, query).await
    }

    /// 貨架級查詢
    async fn get_on_hand_by_location(
        pool: &PgPool,
        query: &InventoryQuery,
        loc_id: Uuid,
    ) -> Result<Vec<InventoryOnHand>> {
        let filters = SliFilterBuilder::new(2, query);
        let sql = format!(
            r#"
            SELECT
                sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.id as storage_location_id, sl.code as storage_location_code, sl.name as storage_location_name,
                p.id as product_id, p.sku as product_sku, p.name as product_name,
                p.base_uom, p.category_code,
                sli.on_hand_qty as qty_on_hand, NULL::numeric as avg_cost,
                sli.batch_no, sli.expiry_date, p.safety_stock, p.reorder_point,
                sli.updated_at as last_updated_at
            FROM storage_location_inventory sli
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            JOIN warehouses w ON sl.warehouse_id = w.id
            JOIN products p ON sli.product_id = p.id
            WHERE sl.id = $1 AND sl.is_active = true AND w.is_active = true AND p.is_active = true
              AND sli.on_hand_qty > 0
              {kw} {pf} {bf}
            ORDER BY p.sku, sli.expiry_date, sli.batch_no
            "#,
            kw = filters.keyword,
            pf = filters.product,
            bf = filters.batch,
        );
        let q = filters.bind_all(
            sqlx::query_as::<_, InventoryOnHand>(sqlx::AssertSqlSafe(sql)).bind(loc_id),
            query,
        );
        Ok(q.fetch_all(pool).await?)
    }

    /// 倉庫級查詢（指定 warehouse_id）
    async fn get_on_hand_by_warehouse(
        pool: &PgPool,
        query: &InventoryQuery,
        warehouse_id: Uuid,
    ) -> Result<Vec<InventoryOnHand>> {
        let filters = SliFilterBuilder::new(2, query);
        let sql = format!(
            r#"
            SELECT
                sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.id as storage_location_id, sl.code as storage_location_code, sl.name as storage_location_name,
                p.id as product_id, p.sku as product_sku, p.name as product_name,
                p.base_uom, p.category_code,
                sli.on_hand_qty as qty_on_hand, NULL::numeric as avg_cost,
                sli.batch_no, sli.expiry_date, p.safety_stock, p.reorder_point,
                sli.updated_at as last_updated_at
            FROM storage_location_inventory sli
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            JOIN warehouses w ON sl.warehouse_id = w.id
            JOIN products p ON sli.product_id = p.id
            WHERE w.id = $1 AND sl.is_active = true AND w.is_active = true AND p.is_active = true
              AND sli.on_hand_qty > 0
              {kw} {pf} {bf}
            ORDER BY p.sku, sli.expiry_date, sli.batch_no
            "#,
            kw = filters.keyword,
            pf = filters.product,
            bf = filters.batch,
        );
        let q = filters.bind_all(
            sqlx::query_as::<_, InventoryOnHand>(sqlx::AssertSqlSafe(sql)).bind(warehouse_id),
            query,
        );
        Ok(q.fetch_all(pool).await?)
    }

    /// 全倉庫概覽（無指定 warehouse_id）：每倉一列，僅顯示現有量 ≠ 0 的品項。
    ///
    /// 低庫存改用 [`Self::get_low_stock_totals`]（全公司總量），不再經此路徑。
    async fn get_on_hand_all_warehouses(
        pool: &PgPool,
        query: &InventoryQuery,
    ) -> Result<Vec<InventoryOnHand>> {
        let keyword_filter = if query.keyword.as_ref().is_some_and(|k| !k.is_empty()) {
            " AND (p.name ILIKE '%' || $1 || '%' OR p.sku ILIKE '%' || $1 || '%')"
        } else {
            ""
        };
        let sql = format!(
            r#"
            SELECT
                w.id as warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                NULL::uuid as storage_location_id, NULL::varchar as storage_location_code,
                NULL::varchar as storage_location_name,
                p.id as product_id, p.sku as product_sku, p.name as product_name,
                p.base_uom, p.category_code,
                COALESCE(SUM(
                    CASE
                        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                        ELSE 0
                    END
                ), 0) as qty_on_hand,
                AVG(sl.unit_cost) FILTER (WHERE sl.unit_cost IS NOT NULL) as avg_cost,
                NULL::varchar as batch_no, NULL::date as expiry_date,
                p.safety_stock, p.reorder_point,
                MAX(sl.created_at) as last_updated_at
            FROM warehouses w
            CROSS JOIN products p
            LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
            WHERE w.is_active = true AND p.is_active = true
              {keyword_filter}
            GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.category_code, p.safety_stock, p.reorder_point
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
        let mut q = sqlx::query_as::<_, InventoryOnHand>(sqlx::AssertSqlSafe(sql));
        if let Some(keyword) = &query.keyword {
            if !keyword.is_empty() {
                q = q.bind(keyword);
            }
        }
        Ok(q.fetch_all(pool).await?)
    }

    /// 產品維度跨倉查詢（指定 product_id、未指定倉庫/儲位）。
    ///
    /// 回傳「貨架級列（跨所有倉）＋各倉未分配餘量列（帳面總量 − 貨架加總，
    /// `storage_location_id` 為 NULL）」：列彼此不重疊、每倉加總 = 帳面總量。
    /// SO 儲位挑選器（qtyByLoc / qtyByWh）與產品詳情庫存快照皆依此契約分組。
    ///
    /// 修復 2026-07-22：舊版此情境落入全倉概覽路徑，該路徑忽略 `product_id`，
    /// 回傳「全品項」彙總 → 挑選器/快照把整倉總量誤標為單品存量、貨架分佈全空。
    /// keyword / batch_no 不與 product_id 並用（product_id 已鎖定單一品項），此路徑不套用。
    async fn get_on_hand_product_across_warehouses(
        pool: &PgPool,
        product_id: Uuid,
    ) -> Result<Vec<InventoryOnHand>> {
        let shelf_rows = sqlx::query_as::<_, InventoryOnHand>(
            r#"
            SELECT
                sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.id as storage_location_id, sl.code as storage_location_code, sl.name as storage_location_name,
                p.id as product_id, p.sku as product_sku, p.name as product_name,
                p.base_uom, p.category_code,
                sli.on_hand_qty as qty_on_hand, NULL::numeric as avg_cost,
                sli.batch_no, sli.expiry_date, p.safety_stock, p.reorder_point,
                sli.updated_at as last_updated_at
            FROM storage_location_inventory sli
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            JOIN warehouses w ON sl.warehouse_id = w.id
            JOIN products p ON sli.product_id = p.id
            WHERE p.id = $1 AND sl.is_active = true AND w.is_active = true AND p.is_active = true
              AND sli.on_hand_qty > 0
            ORDER BY w.code, sl.code, sli.expiry_date, sli.batch_no
            "#,
        )
        .bind(product_id)
        .fetch_all(pool)
        .await?;

        let agg_rows = sqlx::query_as::<_, InventoryOnHand>(
            r#"
            SELECT
                w.id as warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                NULL::uuid as storage_location_id, NULL::varchar as storage_location_code,
                NULL::varchar as storage_location_name,
                p.id as product_id, p.sku as product_sku, p.name as product_name,
                p.base_uom, p.category_code,
                COALESCE(SUM(
                    CASE
                        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                        ELSE 0
                    END
                ), 0) as qty_on_hand,
                AVG(sl.unit_cost) FILTER (WHERE sl.unit_cost IS NOT NULL) as avg_cost,
                NULL::varchar as batch_no, NULL::date as expiry_date,
                p.safety_stock, p.reorder_point,
                MAX(sl.created_at) as last_updated_at
            FROM warehouses w
            CROSS JOIN products p
            LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
            WHERE w.is_active = true AND p.is_active = true AND p.id = $1
            GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.category_code, p.safety_stock, p.reorder_point
            HAVING COALESCE(SUM(
                CASE
                    WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                    WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                    ELSE 0
                END
            ), 0) != 0
            ORDER BY w.code
            "#,
        )
        .bind(product_id)
        .fetch_all(pool)
        .await?;

        // 各倉貨架加總 → 未分配餘量 = 帳面總量 − 貨架加總（≠0 才列，含負值以暴露資料不一致）
        let mut shelf_sum: std::collections::HashMap<Uuid, Decimal> =
            std::collections::HashMap::new();
        for row in &shelf_rows {
            *shelf_sum.entry(row.warehouse_id).or_default() += row.qty_on_hand;
        }
        let mut out = shelf_rows;
        for mut agg in agg_rows {
            let assigned = shelf_sum
                .get(&agg.warehouse_id)
                .copied()
                .unwrap_or_default();
            let remainder = agg.qty_on_hand - assigned;
            if remainder != Decimal::ZERO {
                agg.qty_on_hand = remainder;
                out.push(agg);
            }
        }
        Ok(out)
    }

    /// 效期預警查詢：回傳 N 天內到期的批號級庫存
    async fn get_on_hand_expiry(
        pool: &PgPool,
        query: &InventoryQuery,
        days: i32,
    ) -> Result<Vec<InventoryOnHand>> {
        let has_keyword = query.keyword.as_ref().is_some_and(|k| !k.is_empty());
        let has_warehouse = query.warehouse_id.is_some();
        let mut idx = 2u8;

        let warehouse_filter = if has_warehouse {
            let f = format!(" AND w.id = ${idx}");
            idx += 1;
            f
        } else {
            String::new()
        };
        let keyword_filter = if has_keyword {
            let f = format!(
                " AND (p.name ILIKE '%' || ${idx} || '%' OR p.sku ILIKE '%' || ${idx} || '%')"
            );
            idx += 1;
            let _ = idx;
            f
        } else {
            String::new()
        };

        let sql = format!(
            r#"
            SELECT
                sl.warehouse_id, w.code as warehouse_code, w.name as warehouse_name,
                sl.id as storage_location_id, sl.code as storage_location_code, sl.name as storage_location_name,
                p.id as product_id, p.sku as product_sku, p.name as product_name,
                p.base_uom, p.category_code,
                sli.on_hand_qty as qty_on_hand, NULL::numeric as avg_cost,
                sli.batch_no, sli.expiry_date, p.safety_stock, p.reorder_point,
                sli.updated_at as last_updated_at
            FROM storage_location_inventory sli
            JOIN storage_locations sl ON sli.storage_location_id = sl.id
            JOIN warehouses w ON sl.warehouse_id = w.id
            JOIN products p ON sli.product_id = p.id
            WHERE sl.is_active = true AND w.is_active = true AND p.is_active = true
              AND sli.on_hand_qty > 0
              AND sli.expiry_date IS NOT NULL
              AND sli.expiry_date <= CURRENT_DATE + $1
              {warehouse_filter}
              {keyword_filter}
            ORDER BY sli.expiry_date ASC, p.sku
            "#
        );
        let mut q = sqlx::query_as::<_, InventoryOnHand>(sqlx::AssertSqlSafe(sql)).bind(days);
        if let Some(warehouse_id) = query.warehouse_id {
            q = q.bind(warehouse_id);
        }
        if let Some(keyword) = &query.keyword {
            if !keyword.is_empty() {
                q = q.bind(keyword);
            }
        }
        Ok(q.fetch_all(pool).await?)
    }

    /// 查詢低庫存彙總（全公司總量 < 公司預設安全庫存；一品項一筆 + 各倉分布）
    ///
    /// 與舊版逐倉庫比較不同：先加總**所有倉庫**該品項的庫存量（`stock_ledger`），再與
    /// `products.safety_stock`（公司預設安全庫存）比較，避免同品項在多倉各自低於門檻而
    /// 重複虛報。`warehouse_breakdown` 僅含該品項有庫存（qty ≠ 0）的倉庫，供 drill-down。
    pub async fn get_low_stock_totals(pool: &PgPool) -> Result<Vec<LowStockTotal>> {
        let rows: Vec<LowStockTotalRow> = sqlx::query_as(
            r#"
            WITH wh_product AS (
                SELECT
                    w.id AS warehouse_id, w.code AS warehouse_code, w.name AS warehouse_name,
                    p.id AS product_id, p.sku AS product_sku, p.name AS product_name,
                    p.base_uom, p.safety_stock, p.reorder_point,
                    COALESCE(SUM(
                        CASE
                            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                            WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                            ELSE 0
                        END
                    ), 0) AS wh_qty
                FROM warehouses w
                CROSS JOIN products p
                LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
                WHERE w.is_active = true AND p.is_active = true AND p.safety_stock IS NOT NULL
                GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.safety_stock, p.reorder_point
            ),
            totals AS (
                SELECT product_id, SUM(wh_qty) AS total_qty
                FROM wh_product
                GROUP BY product_id
            )
            SELECT
                wp.product_id, wp.product_sku, wp.product_name, wp.base_uom,
                wp.safety_stock, wp.reorder_point, t.total_qty,
                wp.warehouse_id, wp.warehouse_code, wp.warehouse_name, wp.wh_qty
            FROM wh_product wp
            JOIN totals t ON wp.product_id = t.product_id
            WHERE t.total_qty < wp.safety_stock
            ORDER BY wp.product_sku, wp.warehouse_code
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(Self::group_low_stock_rows(rows))
    }

    /// 將扁平的「品項 × 倉庫」列彙總為一品項一筆 [`LowStockTotal`]。
    ///
    /// rows 須已依 `product_sku` 排序（同品項相鄰）。breakdown 僅收 qty ≠ 0 的倉庫；
    /// stock_status 依全公司總量判定（≤0 缺貨、否則低於安全）。
    fn group_low_stock_rows(rows: Vec<LowStockTotalRow>) -> Vec<LowStockTotal> {
        let mut result: Vec<LowStockTotal> = Vec::new();
        for row in rows {
            let entry = (row.wh_qty != Decimal::ZERO).then_some(LowStockWarehouseQty {
                warehouse_id: row.warehouse_id,
                warehouse_code: row.warehouse_code,
                warehouse_name: row.warehouse_name,
                qty_on_hand: row.wh_qty,
            });
            match result.last_mut() {
                Some(last) if last.product_id == row.product_id => {
                    last.warehouse_breakdown.extend(entry);
                }
                _ => {
                    let stock_status = if row.total_qty <= Decimal::ZERO {
                        "out_of_stock".to_string()
                    } else {
                        "low".to_string()
                    };
                    result.push(LowStockTotal {
                        product_id: row.product_id,
                        product_sku: row.product_sku,
                        product_name: row.product_name,
                        base_uom: row.base_uom,
                        total_on_hand: row.total_qty,
                        safety_stock: row.safety_stock,
                        reorder_point: row.reorder_point,
                        stock_status,
                        warehouse_breakdown: entry.into_iter().collect(),
                    });
                }
            }
        }
        result
    }

    /// 查詢未分配庫存（倉庫層級有庫存，但尚未分配到任何儲位）
    pub async fn get_unassigned_inventory(
        pool: &PgPool,
        query: &InventoryQuery,
    ) -> Result<Vec<UnassignedInventory>> {
        use sqlx::QueryBuilder;

        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            r#"
            WITH wh_stock AS (
                SELECT
                    w.id AS warehouse_id, w.name AS warehouse_name,
                    p.id AS product_id, p.sku AS product_sku,
                    p.name AS product_name, p.base_uom,
                    COALESCE(SUM(
                        CASE
                            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                            WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                            ELSE 0
                        END
                    ), 0) AS qty_on_warehouse
                FROM warehouses w
                JOIN stock_ledger sl ON w.id = sl.warehouse_id
                JOIN products p ON p.id = sl.product_id
                WHERE w.is_active = true AND p.is_active = true
            "#,
        );

        if let Some(warehouse_id) = query.warehouse_id {
            qb.push(" AND w.id = ");
            qb.push_bind(warehouse_id);
        }
        if let Some(keyword) = &query.keyword {
            let pattern = format!("%{}%", keyword);
            qb.push(" AND (p.sku ILIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR p.name ILIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }
        if let Some(product_id) = query.product_id {
            qb.push(" AND p.id = ");
            qb.push_bind(product_id);
        }

        qb.push(
            r#"
                GROUP BY w.id, w.name, p.id, p.sku, p.name, p.base_uom
            ),
            shelf_stock AS (
                SELECT
                    sl.warehouse_id, sli.product_id,
                    COALESCE(SUM(sli.on_hand_qty), 0) AS qty_on_shelves
                FROM storage_location_inventory sli
                JOIN storage_locations sl ON sli.storage_location_id = sl.id
                JOIN warehouses w ON sl.warehouse_id = w.id
                WHERE w.is_active = true
                GROUP BY sl.warehouse_id, sli.product_id
            )
            SELECT
                ws.warehouse_id, ws.warehouse_name, ws.product_id, ws.product_sku,
                ws.product_name, ws.base_uom, ws.qty_on_warehouse,
                COALESCE(ss.qty_on_shelves, 0) AS qty_on_shelves,
                ws.qty_on_warehouse - COALESCE(ss.qty_on_shelves, 0) AS qty_unassigned
            FROM wh_stock ws
            LEFT JOIN shelf_stock ss
                ON ws.warehouse_id = ss.warehouse_id AND ws.product_id = ss.product_id
            WHERE ws.qty_on_warehouse > 0
              AND ws.qty_on_warehouse > COALESCE(ss.qty_on_shelves, 0)
            ORDER BY ws.warehouse_name, ws.product_sku
            "#,
        );

        let rows = qb
            .build_query_as::<UnassignedInventory>()
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    /// 將未分配庫存分配到指定儲位
    ///
    /// - 驗證：qty > 0、儲位屬於該倉庫、分配量 ≤ 目前未分配量
    /// - 動作：UPSERT storage_location_inventory（同 product+batch+expiry 累加），更新 current_count
    /// - 不寫 stock_ledger（倉庫總量不變，僅儲位層級重新分布）
    /// - 精確歸屬：按 FIFO 把本次分配量攤回造成未分配的 GRN 明細，逐筆寫
    ///   `line_shelf_allocations` 審計（攤不回來源的剩餘量記 document_line_id = NULL）
    pub async fn assign_unassigned(
        pool: &PgPool,
        req: &AssignUnassignedRequest,
        actor: &ActorContext,
    ) -> Result<()> {
        if req.qty <= Decimal::ZERO {
            return Err(AppError::Validation("qty 必須大於 0".to_string()));
        }

        // 上架記一張自核准 TR 移轉單 + HMAC 稽核，皆需操作者身份（匿名不可）。
        let created_by = actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("分配未分配庫存需由已登入使用者觸發".to_string()))?;

        // 驗證儲位屬於該倉庫
        let shelf_wh_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT warehouse_id FROM storage_locations WHERE id = $1 AND is_active = true",
        )
        .bind(req.storage_location_id)
        .fetch_optional(pool)
        .await?;

        match shelf_wh_id {
            None => return Err(AppError::NotFound("儲位不存在或已停用".to_string())),
            Some(wh) if wh != req.warehouse_id => {
                return Err(AppError::BusinessRule("儲位不屬於指定倉庫".to_string()))
            }
            _ => {}
        }

        let mut tx = pool.begin().await?;

        // 序列化同一品項的分配：鎖產品列，避免並發（如 UI 雙擊）同時讀到相同
        // current_unassigned / remaining_unshelved 而繞過數量驗證，造成 storage 超額
        // 分配（phantom stock）與 GRN 明細被超扣。
        sqlx::query("SELECT 1 FROM products WHERE id = $1 FOR UPDATE")
            .bind(req.product_id)
            .execute(&mut *tx)
            .await?;

        // TR 明細 uom 帶品項基本單位（DocumentLine.uom 非空）。
        let base_uom: String = sqlx::query_scalar("SELECT base_uom FROM products WHERE id = $1")
            .bind(req.product_id)
            .fetch_one(&mut *tx)
            .await?;

        // 是否指定批號/效期（分配對話框選了特定批）。指定時只從相符批號的來源攤扣、
        // 並以該批號驗證可用量；未指定則跨全部批號依 FIFO 自動分批上架。
        let batch_specified = req.batch_no.is_some() || req.expiry_date.is_some();

        // FIFO 來源（造成未分配的 GRN 未上架明細；指定批號時僅取相符者）。
        let sources: Vec<GrnUnshelvedRow> = sqlx::query_as(
            r#"
            SELECT document_line_id, batch_no, expiry_date, remaining_unshelved
            FROM v_grn_line_unshelved
            WHERE warehouse_id = $1 AND product_id = $2
              AND ($3::boolean = false OR (
                    COALESCE(batch_no, '') = COALESCE($4, '')
                AND COALESCE(expiry_date, '1900-01-01'::date) = COALESCE($5, '1900-01-01'::date)
              ))
            ORDER BY doc_created_at ASC, line_no ASC
            "#,
        )
        .bind(req.warehouse_id)
        .bind(req.product_id)
        .bind(batch_specified)
        .bind(req.batch_no.as_deref())
        .bind(req.expiry_date)
        .fetch_all(&mut *tx)
        .await?;

        // 可用量上限驗證：
        // - 指定批號：以相符來源的剩餘未上架量為上限（不 fallback 到無來源）。
        // - 未指定：以品項層級未分配量（含 legacy 非 GRN）為上限。
        if batch_specified {
            let batch_available: Decimal = sources.iter().map(|s| s.remaining_unshelved).sum();
            if req.qty > batch_available {
                return Err(AppError::BusinessRule(format!(
                    "分配量 {} 超過該批號可用未分配量 {}",
                    req.qty, batch_available
                )));
            }
        } else {
            let current_unassigned: Decimal = sqlx::query_scalar(
                r#"
                SELECT
                    COALESCE((
                        SELECT SUM(
                            CASE
                                WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
                                WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
                                ELSE 0
                            END
                        )
                        FROM stock_ledger sl
                        WHERE sl.warehouse_id = $1 AND sl.product_id = $2
                    ), 0)
                    -
                    COALESCE((
                        SELECT SUM(sli.on_hand_qty)
                        FROM storage_location_inventory sli
                        JOIN storage_locations sloc ON sli.storage_location_id = sloc.id
                        WHERE sloc.warehouse_id = $1 AND sli.product_id = $2
                    ), 0)
                "#,
            )
            .bind(req.warehouse_id)
            .bind(req.product_id)
            .fetch_one(&mut *tx)
            .await?;
            if req.qty > current_unassigned {
                return Err(AppError::BusinessRule(format!(
                    "分配量 {} 超過可用未分配量 {}",
                    req.qty, current_unassigned
                )));
            }
        }

        // 精確歸屬：按 FIFO 逐來源攤扣，逐筆寫 line_shelf_allocations 審計（v_grn_line_unshelved
        // 「剩餘未上架量」唯一依據，必留），並收集 (批號, 效期, 數量) 供下方 TR 明細忠實上架。
        // 不再於此直接 upsert storage_location_inventory —— 改由自核准 TR 的 process_document
        // 記帳（process_transfer 於目標儲位 upsert sli + 更新 current_count），避免雙重計數。
        #[allow(clippy::type_complexity)]
        let mut tr_specs: Vec<(Option<String>, Option<chrono::NaiveDate>, Decimal)> = Vec::new();
        let mut remaining = req.qty;
        for src in sources {
            if remaining <= Decimal::ZERO {
                break;
            }
            let take = remaining.min(src.remaining_unshelved);
            if take <= Decimal::ZERO {
                continue;
            }
            sqlx::query(
                r#"
                INSERT INTO line_shelf_allocations (
                    document_line_id, storage_location_id, product_id,
                    batch_no, expiry_date, qty, created_by
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(src.document_line_id)
            .bind(req.storage_location_id)
            .bind(req.product_id)
            .bind(src.batch_no.as_deref())
            .bind(src.expiry_date)
            .bind(take)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;
            tr_specs.push((src.batch_no.clone(), src.expiry_date, take));
            remaining -= take;
        }

        // 攤不回 GRN 來源的剩餘量（legacy 匯入 / 070 baseline，僅未指定批號時可能發生）→
        // 以 req 批號上架並記無來源審計，避免帳實斷點。指定批號時 remaining 必為 0。
        if remaining > Decimal::ZERO {
            sqlx::query(
                r#"
                INSERT INTO line_shelf_allocations (
                    document_line_id, storage_location_id, product_id,
                    batch_no, expiry_date, qty, created_by
                )
                VALUES (NULL, $1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(req.storage_location_id)
            .bind(req.product_id)
            .bind(req.batch_no.as_deref())
            .bind(req.expiry_date)
            .bind(remaining)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;
            tr_specs.push((req.batch_no.clone(), req.expiry_date, remaining));
        }

        // 開一張「同倉、已核准」TR 移轉單記錄本次上架（未分配 → 目標儲位）並記帳。
        // qty > 0 已驗證，tr_specs 必非空。
        Self::create_putaway_transfer(&mut tx, req, actor, &base_uom, &tr_specs).await?;

        tx.commit().await?;
        Ok(())
    }

    /// 建立一張「同倉、已核准」TR 移轉單記錄未分配上架，並經 [`Self::process_document`] 記帳。
    ///
    /// 每個 FIFO 攤扣 chunk 一行（批號/效期忠實）：`storage_location_from_id = NULL`（來源為
    /// 未分配池）、`storage_location_to_id = 目標儲位`。process_transfer 據此寫
    /// transfer_out(from 倉, null loc) + transfer_in(to 倉, 目標 loc) —— 同倉淨零、僅目標端
    /// upsert `storage_location_inventory` 並更新 `current_count`（故上游不再直接動 sli）；
    /// 另補寫單頭+明細的 HMAC 稽核（原 assign 無稽核，改走 TR 一併補上）。
    async fn create_putaway_transfer(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        req: &AssignUnassignedRequest,
        actor: &ActorContext,
        base_uom: &str,
        tr_specs: &[(Option<String>, Option<chrono::NaiveDate>, Decimal)],
    ) -> Result<()> {
        let created_by = actor
            .actor_user_id()
            .ok_or_else(|| AppError::Forbidden("分配未分配庫存需由已登入使用者觸發".to_string()))?;

        let doc_no = DocumentService::generate_doc_no(tx, DocType::TR).await?;
        let document = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, warehouse_from_id, warehouse_to_id,
                doc_date, remark, created_by, approved_by, approved_at, created_at, updated_at
            )
            VALUES ($1, 'TR', $2, 'approved', $3, $3, $3, $4, $5, $6, $6, NOW(), NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&doc_no)
        .bind(req.warehouse_id)
        .bind(time::today_taiwan_naive())
        .bind("未分配庫存上架至儲位")
        .bind(created_by)
        .fetch_one(&mut **tx)
        .await?;

        let mut lines: Vec<DocumentLine> = Vec::with_capacity(tr_specs.len());
        for (idx, (batch_no, expiry_date, qty)) in tr_specs.iter().enumerate() {
            let line = sqlx::query_as::<_, DocumentLine>(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom,
                    batch_no, expiry_date, storage_location_from_id, storage_location_to_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, $9)
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(document.id)
            .bind((idx + 1) as i32)
            .bind(req.product_id)
            .bind(*qty)
            .bind(base_uom)
            .bind(batch_no.as_deref())
            .bind(*expiry_date)
            .bind(req.storage_location_id)
            .fetch_one(&mut **tx)
            .await?;
            lines.push(line);
        }

        // 記帳：transfer_out/in 淨零 + 目標 sli/current_count（同一交易）。
        Self::process_document(tx, &document, &lines).await?;

        // 稽核：單頭 + 明細作為同一 DOC_CREATE 事件（補原 assign 缺漏的 HMAC 軌跡）。
        let display = format!("{}: {}", document.doc_type.prefix(), document.doc_no);
        let snapshot = DocumentAuditSnapshot {
            document: &document,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_CREATE",
                entity: Some(AuditEntity::new("document", document.id, &display)),
                data_diff: Some(DataDiff::create_only(&snapshot)),
                request_context: None,
            },
        )
        .await?;
        Ok(())
    }

    /// 查詢造成某 (倉庫, 產品) 未分配的來源 GRN 明細（剩餘未上架量 > 0）。
    /// 供「未分配庫存」列展開追溯：這批未分配是哪張採購入庫單造成的。
    pub async fn get_unassigned_sources(
        pool: &PgPool,
        query: &UnassignedSourceQuery,
    ) -> Result<Vec<UnassignedSourceDoc>> {
        let rows = sqlx::query_as::<_, UnassignedSourceDoc>(
            r#"
            SELECT
                v.document_id, v.doc_no, v.doc_date, v.line_no,
                pt.name AS partner_name,
                v.batch_no, v.expiry_date, v.remaining_unshelved
            FROM v_grn_line_unshelved v
            LEFT JOIN partners pt ON pt.id = v.partner_id
            WHERE v.warehouse_id = $1 AND v.product_id = $2
            ORDER BY v.doc_created_at ASC, v.line_no ASC
            "#,
        )
        .bind(query.warehouse_id)
        .bind(query.product_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
