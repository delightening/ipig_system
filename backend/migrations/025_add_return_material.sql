-- 新增退料單（RM）功能
-- Version: 0.25

BEGIN;

-- 1. 將 RM 添加到 doc_type 枚舉中
-- 注意：PostgreSQL 不支援直接向 ENUM 中添加值（在舊版本中）
-- 需要創建新的 ENUM 類型並替換
DO $$
BEGIN
    -- 創建新的 ENUM 類型（包含 RM）
    CREATE TYPE doc_type_new AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'TR', 'STK', 'ADJ', 'RM');
    
    -- 替換 ENUM 類型
    ALTER TABLE documents ALTER COLUMN doc_type TYPE doc_type_new USING doc_type::text::doc_type_new;
    ALTER TABLE stock_ledger ALTER COLUMN doc_type TYPE doc_type_new USING doc_type::text::doc_type_new;
    
    -- 刪除舊的 ENUM 類型
    DROP TYPE doc_type;
    
    -- 重命名新類型為原名稱
    ALTER TYPE doc_type_new RENAME TO doc_type;
END $$;

COMMIT;
