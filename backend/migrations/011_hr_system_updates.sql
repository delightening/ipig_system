-- ============================================
-- Migration 011: HR System Updates
-- 
-- 包含：
-- 1. 清除 AUP、產品、倉庫、供應商資料
-- 2. 更新加班類型為 A/B/C/D
-- ============================================

-- ============================================
-- 1. 清除 ERP 與 AUP 資料
-- ============================================

-- 先清除相依資料 (由子到父)

-- 清除庫存相關
DELETE FROM inventory_snapshots;
DELETE FROM stock_ledger;

-- 清除單據相關
DELETE FROM document_lines;
DELETE FROM documents;

-- 清除產品相關
DELETE FROM product_uom_conversions;
DELETE FROM products;
DELETE FROM product_categories;

-- 清除 SKU 序號追蹤
DELETE FROM sku_sequences;

-- 清除供應商
DELETE FROM partners;

-- 清除倉庫
DELETE FROM warehouses;

-- 清除 AUP 協議相關
DELETE FROM review_comments;
DELETE FROM review_assignments;
DELETE FROM protocol_attachments;
DELETE FROM protocol_status_history;
DELETE FROM protocol_versions;
DELETE FROM user_protocols;
DELETE FROM protocols;

-- ============================================
-- 2. 更新加班類型為 A/B/C/D
-- ============================================

-- 更新現有資料的 overtime_type
-- weekday -> A (平日加班)
-- weekend -> B (假日加班)
-- holiday -> C (國定假日加班)
UPDATE overtime_records SET overtime_type = 'A' WHERE overtime_type = 'weekday';
UPDATE overtime_records SET overtime_type = 'B' WHERE overtime_type = 'weekend';
UPDATE overtime_records SET overtime_type = 'C' WHERE overtime_type = 'holiday';

-- 注意：現有已核准的加班記錄的 comp_time_hours 保持不變
-- 新規則（只有 C 和 D 有補休時數）只適用於新建的加班申請

-- ============================================
-- 完成
-- ============================================
