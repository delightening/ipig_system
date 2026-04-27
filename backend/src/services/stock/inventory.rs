use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{AssignUnassignedRequest, InventoryOnHand, InventoryQuery, LowStockAlert, UnassignedInventory},
    AppError, Result,
};

use super::StockService;

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
            let f = format!(" AND (p.name ILIKE '%' || ${idx} || '%' OR p.sku ILIKE '%' || ${idx} || '%')");
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
        Self { keyword, product, batch }
    }

    /// 按建構順序 bind 參數（keyword → product_id → batch_no）
    fn bind_all<'q>(
        &self,
        mut q: sqlx::query::QueryAs<'q, sqlx::Postgres, InventoryOnHand, sqlx::postgres::PgArguments>,
        query: &'q InventoryQuery,
    ) -> sqlx::query::QueryAs<'q, sqlx::Postgres, InventoryOnHand, sqlx::postgres::PgArguments> {
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
            kw = filters.keyword, pf = filters.product, bf = filters.batch,
        );
        let q = filters.bind_all(sqlx::query_as::<_, InventoryOnHand>(&sql).bind(loc_id), query);
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
            kw = filters.keyword, pf = filters.product, bf = filters.batch,
        );
        let q = filters.bind_all(sqlx::query_as::<_, InventoryOnHand>(&sql).bind(warehouse_id), query);
        Ok(q.fetch_all(pool).await?)
    }

    /// 全倉庫概覽（無指定 warehouse_id）
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
        let mut q = sqlx::query_as::<_, InventoryOnHand>(&sql);
        if let Some(keyword) = &query.keyword {
            if !keyword.is_empty() {
                q = q.bind(keyword);
            }
        }
        Ok(q.fetch_all(pool).await?)
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
            let f = format!(" AND (p.name ILIKE '%' || ${idx} || '%' OR p.sku ILIKE '%' || ${idx} || '%')");
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
        let mut q = sqlx::query_as::<_, InventoryOnHand>(&sql).bind(days);
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

    /// 查詢低庫存警示
    pub async fn get_low_stock_alerts(pool: &PgPool) -> Result<Vec<LowStockAlert>> {
        let alerts = sqlx::query_as::<_, InventoryOnHand>(
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
                AVG(sl.unit_cost) as avg_cost,
                NULL::varchar as batch_no, NULL::date as expiry_date,
                p.safety_stock, p.reorder_point,
                NULL::timestamptz as last_updated_at
            FROM warehouses w
            CROSS JOIN products p
            LEFT JOIN stock_ledger sl ON w.id = sl.warehouse_id AND p.id = sl.product_id
            WHERE w.is_active = true AND p.is_active = true
              AND p.safety_stock IS NOT NULL
            GROUP BY w.id, w.code, w.name, p.id, p.sku, p.name, p.base_uom, p.category_code, p.safety_stock, p.reorder_point
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
            .map(Self::inventory_to_low_stock_alert)
            .collect();
        Ok(result)
    }

    /// InventoryOnHand → LowStockAlert 轉換
    fn inventory_to_low_stock_alert(inv: InventoryOnHand) -> LowStockAlert {
        let safety_stock = inv.safety_stock;
        let reorder_point = inv.reorder_point;
        let stock_status = if inv.qty_on_hand <= rust_decimal::Decimal::ZERO {
            "out_of_stock".to_string()
        } else if let Some(ss) = safety_stock {
            if inv.qty_on_hand < ss { "low".to_string() } else { "ok".to_string() }
        } else {
            "ok".to_string()
        };
        LowStockAlert {
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
        }
    }

    /// 查詢未分配庫存（倉庫層級有庫存，但尚未分配到任何儲位）
    pub async fn get_unassigned_inventory(
        pool: &PgPool,
        query: &InventoryQuery,
    ) -> Result<Vec<UnassignedInventory>> {
        use sqlx::QueryBuilder;

        let mut qb: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
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

        let rows = qb.build_query_as::<UnassignedInventory>().fetch_all(pool).await?;
        Ok(rows)
    }

    /// 將未分配庫存分配到指定儲位
    ///
    /// - 驗證：qty > 0、儲位屬於該倉庫、分配量 ≤ 目前未分配量
    /// - 動作：UPSERT storage_location_inventory（同 product+batch+expiry 累加），更新 current_count
    /// - 不寫 stock_ledger（倉庫總量不變，僅儲位層級重新分布）
    pub async fn assign_unassigned(
        pool: &PgPool,
        req: &AssignUnassignedRequest,
    ) -> Result<()> {
        if req.qty <= Decimal::ZERO {
            return Err(AppError::Validation("qty 必須大於 0".to_string()));
        }

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

        // 計算目前未分配量（warehouse 總量 - shelf 總量 per product）
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

        // UPSERT：同 product+batch+expiry 累加
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
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(req.storage_location_id)
        .bind(req.product_id)
        .bind(req.qty)
        .bind(req.batch_no.as_deref())
        .bind(req.expiry_date)
        .execute(&mut *tx)
        .await?;

        // 更新儲位 current_count
        sqlx::query(
            r#"
            UPDATE storage_locations
            SET current_count = (
                SELECT COUNT(*) FROM storage_location_inventory
                WHERE storage_location_id = $1 AND on_hand_qty > 0
            ),
            updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(req.storage_location_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
