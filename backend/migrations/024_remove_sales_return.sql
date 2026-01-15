-- 移除銷售退貨（SR）功能
-- Version: 0.24

BEGIN;

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

COMMIT;
