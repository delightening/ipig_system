-- 移除銷售退貨（SR）功能
-- Version: 0.24

-- 1. 刪除 SR 相關權限
DELETE FROM role_permissions 
WHERE permission_id IN (
    SELECT id FROM permissions WHERE code LIKE 'erp.sr.%' OR code LIKE 'sr.%'
);

DELETE FROM permissions 
WHERE code LIKE 'erp.sr.%' OR code LIKE 'sr.%';

-- 2. 從 doc_type 枚舉中移除 SR
-- 注意：PostgreSQL 不支援直接從 ENUM 中刪除值
-- 需要創建新的 ENUM 類型並替換
-- 先刪除依賴於 doc_type 的視圖
DROP VIEW IF EXISTS v_purchase_order_receipt_status;

DO $$
BEGIN
    -- 創建新的 ENUM 類型（不包含 SR）
    CREATE TYPE doc_type_new AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'TR', 'STK', 'ADJ');
    
    -- 刪除所有 SR 類型的單據（如果存在）
    DELETE FROM document_lines WHERE document_id IN (
        SELECT id FROM documents WHERE doc_type::text = 'SR'
    );
    DELETE FROM documents WHERE doc_type::text = 'SR';
    
    -- 刪除所有 SR 類型的庫存流水記錄
    DELETE FROM stock_ledger WHERE doc_type::text = 'SR';
    
    -- 替換 ENUM 類型
    ALTER TABLE documents ALTER COLUMN doc_type TYPE doc_type_new USING doc_type::text::doc_type_new;
    ALTER TABLE stock_ledger ALTER COLUMN doc_type TYPE doc_type_new USING doc_type::text::doc_type_new;
    
    -- 刪除舊的 ENUM 類型
    DROP TYPE doc_type;
    
    -- 重命名新類型為原名稱
    ALTER TYPE doc_type_new RENAME TO doc_type;
END $$;

-- 3. 重新創建視圖
CREATE OR REPLACE VIEW v_purchase_order_receipt_status AS
SELECT 
    po.id AS po_id,
    po.doc_no AS po_no,
    po.status AS po_status,
    po.partner_id,
    po.warehouse_id,
    po.doc_date AS po_date,
    COALESCE(SUM(pol.qty), 0) AS ordered_qty,
    COALESCE(SUM(grnl.received_qty), 0) AS received_qty,
    CASE 
        WHEN COALESCE(SUM(grnl.received_qty), 0) = 0 THEN 'pending'
        WHEN COALESCE(SUM(grnl.received_qty), 0) < COALESCE(SUM(pol.qty), 0) THEN 'partial'
        ELSE 'complete'
    END AS receipt_status
FROM documents po
LEFT JOIN document_lines pol ON po.id = pol.document_id
LEFT JOIN (
    SELECT 
        grn.source_doc_id,
        grnl.product_id,
        SUM(grnl.qty) AS received_qty
    FROM documents grn
    JOIN document_lines grnl ON grn.id = grnl.document_id
    WHERE grn.doc_type = 'GRN' AND grn.status = 'approved'
    GROUP BY grn.source_doc_id, grnl.product_id
) grnl ON po.id = grnl.source_doc_id AND pol.product_id = grnl.product_id
WHERE po.doc_type = 'PO'
GROUP BY po.id, po.doc_no, po.status, po.partner_id, po.warehouse_id, po.doc_date;
