-- ============================================
-- Migration 011: AUP Review Flow Enhancement
-- 
-- 變更內容：
-- 1. 新增 VET_REVIEW 狀態
-- 2. review_assignments 新增 is_primary_reviewer
-- 3. review_comments 支援多階段意見
-- 4. 新增 system_settings 表
-- 5. 新增 vet_assignments 表
-- ============================================

-- ============================================
-- 1. 新增 VET_REVIEW 狀態到 protocol_status enum
-- ============================================
-- 注意：PostgreSQL 的 ADD VALUE 無法指定位置，新值會加在最後
-- 但在 Rust 端的 enum 定義中需要調整順序
ALTER TYPE protocol_status ADD VALUE IF NOT EXISTS 'VET_REVIEW';

-- ============================================
-- 2. 修改 review_assignments 表
-- ============================================
-- 新增 is_primary_reviewer 欄位
-- true = 正式審查委員（可撰寫意見，限 2-3 位）
-- false = 其他審查委員（唯讀）
ALTER TABLE review_assignments 
ADD COLUMN IF NOT EXISTS is_primary_reviewer BOOLEAN NOT NULL DEFAULT false;

-- 新增 review_stage 欄位，記錄指派發生在哪個審查階段
ALTER TABLE review_assignments
ADD COLUMN IF NOT EXISTS review_stage VARCHAR(20) DEFAULT 'UNDER_REVIEW';

-- ============================================
-- 3. 修改 review_comments 表，支援多階段意見
-- ============================================
-- 允許 protocol_version_id 為空（預審階段尚無版本）
ALTER TABLE review_comments 
ALTER COLUMN protocol_version_id DROP NOT NULL;

-- 新增 protocol_id 直接關聯（用於預審意見）
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS protocol_id UUID REFERENCES protocols(id);

-- 新增 review_stage 欄位
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS review_stage VARCHAR(20);

-- 新增約束確保 review_stage 值有效
ALTER TABLE review_comments 
ADD CONSTRAINT chk_review_stage 
CHECK (review_stage IS NULL OR review_stage IN ('PRE_REVIEW', 'VET_REVIEW', 'UNDER_REVIEW'));

-- 確保至少有一個關聯（protocol_version_id 或 protocol_id）
ALTER TABLE review_comments 
ADD CONSTRAINT chk_protocol_reference 
CHECK (protocol_version_id IS NOT NULL OR protocol_id IS NOT NULL);

-- 新增索引
CREATE INDEX IF NOT EXISTS idx_review_comments_protocol_id ON review_comments(protocol_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_review_stage ON review_comments(review_stage);

-- ============================================
-- 4. 新增系統設定表
-- ============================================
CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id)
);

-- 新增預設獸醫師設定
INSERT INTO system_settings (key, value, description) VALUES 
('default_vet_reviewer', '{"user_id": null}', '預設獸醫審查員，VET_REVIEW 階段會自動指派此獸醫師')
ON CONFLICT (key) DO NOTHING;

-- ============================================
-- 5. 新增獸醫審查指派表
-- ============================================
CREATE TABLE IF NOT EXISTS vet_review_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    vet_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    decision VARCHAR(20),
    decision_remark TEXT,
    UNIQUE (protocol_id)
);

CREATE INDEX IF NOT EXISTS idx_vet_review_assignments_protocol ON vet_review_assignments(protocol_id);
CREATE INDEX IF NOT EXISTS idx_vet_review_assignments_vet ON vet_review_assignments(vet_id);

-- ============================================
-- 6. 更新現有資料
-- ============================================
-- 將現有的 review_comments 設定預設 review_stage
UPDATE review_comments 
SET review_stage = 'UNDER_REVIEW' 
WHERE review_stage IS NULL AND protocol_version_id IS NOT NULL;

-- ============================================
-- 完成
-- ============================================
