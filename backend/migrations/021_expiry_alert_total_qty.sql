-- 021: 更新效期預警 view
--   1. 補齊 ExpiryAlert struct 所需欄位 (spec, category_code, warehouse_name, expiry_status)
--   2. 新增 total_qty: 同品項同倉庫所有批號的合計庫存量

DROP VIEW IF EXISTS v_expiry_alerts;
CREATE VIEW v_expiry_alerts AS
SELECT
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.spec,
    p.category_code,
    sl.warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    sl.batch_no,
    sl.expiry_date,
    SUM(CASE WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base ELSE -sl.qty_base END) AS on_hand_qty,
    p.base_uom,
    sl.expiry_date - CURRENT_DATE AS days_until_expiry,
    CASE WHEN sl.expiry_date < CURRENT_DATE THEN 'expired' ELSE 'expiring_soon' END AS expiry_status,
    -- 同品項同倉庫所有批號的合計庫存量
    COALESCE(inv.on_hand_qty_base, 0) AS total_qty
FROM stock_ledger sl
JOIN products p ON sl.product_id = p.id
JOIN warehouses w ON sl.warehouse_id = w.id
LEFT JOIN inventory_snapshots inv ON inv.product_id = p.id AND inv.warehouse_id = sl.warehouse_id
WHERE p.track_expiry = true AND sl.expiry_date IS NOT NULL AND p.is_active = true
GROUP BY p.id, p.sku, p.name, p.spec, p.category_code, sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date, p.base_uom, inv.on_hand_qty_base
HAVING SUM(CASE WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base ELSE -sl.qty_base END) > 0 AND sl.expiry_date <= CURRENT_DATE + 60;
