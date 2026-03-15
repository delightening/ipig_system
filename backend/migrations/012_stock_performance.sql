-- no-transaction
-- ============================================
-- Migration 012: 庫存查詢效能優化
-- 日期: 2026-03-15
-- 目的: 新增索引、重寫視圖、初始化 inventory_snapshots
-- ============================================

-- 新增關鍵索引 (CONCURRENTLY 避免鎖表)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_stock_ledger_wh_prod_dir
  ON stock_ledger(warehouse_id, product_id, direction);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_stock_ledger_wh_prod_date
  ON stock_ledger(warehouse_id, product_id, trx_date DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_stock_ledger_doc_id
  ON stock_ledger(doc_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_document_lines_product_id
  ON document_lines(product_id);

-- 重寫低庫存警告視圖 (移除 CROSS JOIN,改用 inventory_snapshots)
DROP VIEW IF EXISTS v_low_stock_alerts;

CREATE OR REPLACE VIEW v_low_stock_alerts AS
SELECT
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    inv.warehouse_id,
    w.code AS warehouse_code,
    inv.on_hand_qty_base AS on_hand_qty,
    p.base_uom,
    CASE
        WHEN inv.on_hand_qty_base <= 0 THEN 'out_of_stock'
        WHEN p.safety_stock IS NOT NULL AND inv.on_hand_qty_base < p.safety_stock THEN 'below_safety'
        WHEN p.reorder_point IS NOT NULL AND inv.on_hand_qty_base < p.reorder_point THEN 'below_reorder'
    END AS stock_status
FROM inventory_snapshots inv
JOIN products p ON inv.product_id = p.id
JOIN warehouses w ON inv.warehouse_id = w.id
WHERE p.is_active AND w.is_active
  AND (
    inv.on_hand_qty_base <= 0
    OR (p.safety_stock IS NOT NULL AND inv.on_hand_qty_base < p.safety_stock)
    OR (p.reorder_point IS NOT NULL AND inv.on_hand_qty_base < p.reorder_point)
  );

-- 重寫庫存總覽視圖
DROP VIEW IF EXISTS v_inventory_summary;

CREATE OR REPLACE VIEW v_inventory_summary AS
SELECT
    w.id AS warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.base_uom,
    p.category_code,
    inv.on_hand_qty_base,
    inv.avg_cost,
    p.safety_stock,
    p.reorder_point
FROM inventory_snapshots inv
JOIN warehouses w ON inv.warehouse_id = w.id
JOIN products p ON inv.product_id = p.id
WHERE p.is_active AND w.is_active
ORDER BY w.code, p.sku;

-- 初始化 inventory_snapshots (Backfill 從 stock_ledger 計算當前庫存)
INSERT INTO inventory_snapshots (warehouse_id, product_id, on_hand_qty_base, avg_cost, updated_at)
SELECT
    sl.warehouse_id,
    sl.product_id,
    COALESCE(SUM(
        CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
            ELSE 0
        END
    ), 0) AS on_hand_qty_base,
    AVG(sl.unit_cost) AS avg_cost,
    NOW() AS updated_at
FROM stock_ledger sl
WHERE EXISTS (
    SELECT 1 FROM warehouses w WHERE w.id = sl.warehouse_id AND w.is_active
)
AND EXISTS (
    SELECT 1 FROM products p WHERE p.id = sl.product_id AND p.is_active
)
GROUP BY sl.warehouse_id, sl.product_id
HAVING SUM(
    CASE
        WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
        WHEN sl.direction IN ('out', 'transfer_out', 'adjust_out') THEN -sl.qty_base
        ELSE 0
    END
) != 0
ON CONFLICT (warehouse_id, product_id) DO UPDATE
SET
    on_hand_qty_base = EXCLUDED.on_hand_qty_base,
    avg_cost = EXCLUDED.avg_cost,
    updated_at = NOW();

-- 驗證初始化結果
DO $$
DECLARE
    snapshot_count INTEGER;
    ledger_distinct_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO snapshot_count FROM inventory_snapshots;
    SELECT COUNT(DISTINCT (warehouse_id, product_id)) INTO ledger_distinct_count
    FROM stock_ledger;

    RAISE NOTICE '初始化完成: inventory_snapshots 有 % 筆記錄', snapshot_count;
    RAISE NOTICE 'stock_ledger 有 % 個唯一倉庫-產品組合', ledger_distinct_count;

    IF snapshot_count = 0 THEN
        RAISE WARNING '警告: inventory_snapshots 為空,可能 stock_ledger 沒有資料或所有庫存為 0';
    END IF;
END $$;
