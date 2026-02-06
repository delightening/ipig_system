-- ============================================
-- Migration 009: Pig Import Batches Table
-- 
-- 新增豬隻匯入批次記錄表
-- 用於追蹤 CSV/Excel 匯入作業的狀態和結果
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 建立豬隻匯入批次表
-- ============================================

CREATE TABLE IF NOT EXISTS pig_import_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_type import_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    total_rows INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    status import_status NOT NULL DEFAULT 'pending',
    error_details JSONB,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- 建立索引
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_status ON pig_import_batches(status);
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_by ON pig_import_batches(created_by);
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_at ON pig_import_batches(created_at DESC);

-- ============================================
-- 完成
-- ============================================
