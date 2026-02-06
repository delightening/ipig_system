-- ============================================
-- Migration 008: Storage Location Integration
-- 
-- 為單據明細新增儲位欄位，支援 GRN 入庫指定儲位
-- ============================================

-- 為 document_lines 新增 storage_location_id 欄位
ALTER TABLE document_lines 
ADD COLUMN IF NOT EXISTS storage_location_id UUID REFERENCES storage_locations(id);

CREATE INDEX IF NOT EXISTS idx_document_lines_storage_location 
ON document_lines(storage_location_id);

-- ============================================
-- 完成
-- ============================================
