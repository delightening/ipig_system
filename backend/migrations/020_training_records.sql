-- ============================================
-- Migration 020: 人員訓練紀錄 (GLP 合規)
--
-- 包含：
-- - training_records 表
-- - training.view, training.manage 權限
--
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 人員訓練紀錄表
-- ============================================

CREATE TABLE training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_name VARCHAR(200) NOT NULL,
    completed_at DATE NOT NULL,
    expires_at DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_training_records_user ON training_records(user_id);
CREATE INDEX idx_training_records_expires ON training_records(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================
-- 2. 訓練紀錄權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'training.view', '查看訓練紀錄', 'training', '可查看人員訓練紀錄', NOW()),
    (gen_random_uuid(), 'training.manage', '管理訓練紀錄', 'training', '可新增、編輯、刪除訓練紀錄', NOW())
ON CONFLICT (code) DO NOTHING;
