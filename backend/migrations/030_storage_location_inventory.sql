-- =============================================================================
-- Migration 030: Storage Location Inventory Tracking
-- 新增儲位層級的庫存追蹤功能
-- =============================================================================

-- 1. 在 document_lines 新增儲位欄位
ALTER TABLE document_lines ADD COLUMN IF NOT EXISTS storage_location_id UUID REFERENCES storage_locations(id);

-- 2. 在 stock_ledger 新增儲位欄位
ALTER TABLE stock_ledger ADD COLUMN IF NOT EXISTS storage_location_id UUID REFERENCES storage_locations(id);

-- 3. 建立儲位庫存表
CREATE TABLE IF NOT EXISTS storage_location_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    storage_location_id UUID NOT NULL REFERENCES storage_locations(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    on_hand_qty NUMERIC(18, 4) NOT NULL DEFAULT 0,
    batch_no VARCHAR(50),
    expiry_date DATE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 建立唯一索引（支援 NULL 值）
-- 使用 COALESCE 處理 NULL 值，避免使用 ::text 轉型（非 IMMUTABLE）
CREATE UNIQUE INDEX IF NOT EXISTS idx_storage_location_inventory_unique 
ON storage_location_inventory(storage_location_id, product_id, COALESCE(batch_no, ''), COALESCE(expiry_date, '1900-01-01'::date));

-- 索引
CREATE INDEX IF NOT EXISTS idx_document_lines_storage_location ON document_lines(storage_location_id);
CREATE INDEX IF NOT EXISTS idx_stock_ledger_storage_location ON stock_ledger(storage_location_id);
CREATE INDEX IF NOT EXISTS idx_storage_location_inventory_location ON storage_location_inventory(storage_location_id);
CREATE INDEX IF NOT EXISTS idx_storage_location_inventory_product ON storage_location_inventory(product_id);
