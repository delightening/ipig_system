-- GLP 合規功能：軟刪除、變更原因、增強審計日誌
-- Migration: 013_glp_compliance.sql
-- 日期: 2026-01-19

-- ============================================
-- 1. 為動物相關表格新增軟刪除欄位
-- ============================================

-- pigs 表已有 deleted_at，新增刪除原因和刪除者
ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pigs ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_observations 表
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_surgeries 表
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_weights 表
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_vaccinations 表
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- pig_sacrifices 表
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- vet_recommendations 表
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- ============================================
-- 2. 建立變更原因表
-- ============================================

CREATE TABLE IF NOT EXISTS change_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,      -- 'pig', 'observation', 'surgery', etc.
    entity_id VARCHAR(50) NOT NULL,        -- 可以是 UUID 或 INTEGER
    change_type VARCHAR(20) NOT NULL,      -- 'UPDATE', 'DELETE'
    reason TEXT NOT NULL,
    old_values JSONB,                       -- 變更前的值
    new_values JSONB,                       -- 變更後的值
    changed_fields TEXT[],                  -- 變更的欄位名稱
    changed_by UUID NOT NULL REFERENCES users(id),
    changed_at TIMESTAMPTZ DEFAULT NOW()
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_change_reasons_entity ON change_reasons(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_change_reasons_changed_at ON change_reasons(changed_at);
CREATE INDEX IF NOT EXISTS idx_change_reasons_changed_by ON change_reasons(changed_by);

-- ============================================
-- 3. 增強審計日誌表
-- ============================================

-- audit_logs 表增強
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS old_value JSONB;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS new_value JSONB;
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS changed_fields TEXT[];
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS change_reason TEXT;

-- user_activity_logs 表增強
ALTER TABLE user_activity_logs ADD COLUMN IF NOT EXISTS old_value JSONB;
ALTER TABLE user_activity_logs ADD COLUMN IF NOT EXISTS new_value JSONB;
ALTER TABLE user_activity_logs ADD COLUMN IF NOT EXISTS change_reason TEXT;

-- ============================================
-- 4. 為未來的電子簽章和資料鎖定預留欄位
-- ============================================

-- 資料鎖定欄位（Phase 2 使用）
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT false;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT false;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT false;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS locked_at TIMESTAMPTZ;
ALTER TABLE pig_sacrifices ADD COLUMN IF NOT EXISTS locked_by UUID REFERENCES users(id);

-- ============================================
-- 5. 電子簽章表（Phase 2 使用）
-- ============================================

CREATE TABLE IF NOT EXISTS electronic_signatures (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,      -- 'sacrifice', 'protocol_approval'
    entity_id VARCHAR(50) NOT NULL,
    signer_id UUID NOT NULL REFERENCES users(id),
    signature_type VARCHAR(50) NOT NULL,   -- 'APPROVE', 'CONFIRM', 'WITNESS'
    content_hash VARCHAR(64) NOT NULL,     -- SHA-256 of content at signing time
    signature_data TEXT NOT NULL,          -- Encrypted signature (base64)
    ip_address VARCHAR(45),
    user_agent TEXT,
    signed_at TIMESTAMPTZ DEFAULT NOW(),
    is_valid BOOLEAN DEFAULT true,
    invalidated_reason TEXT,
    invalidated_at TIMESTAMPTZ,
    invalidated_by UUID REFERENCES users(id)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_entity ON electronic_signatures(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_signer ON electronic_signatures(signer_id);
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_valid ON electronic_signatures(is_valid) WHERE is_valid = true;

-- 簽章驗證日誌
CREATE TABLE IF NOT EXISTS signature_verification_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature_id UUID NOT NULL REFERENCES electronic_signatures(id),
    verified_by UUID REFERENCES users(id),
    verification_result BOOLEAN NOT NULL,
    verification_method VARCHAR(50) NOT NULL,
    failure_reason TEXT,
    verified_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================
-- 6. 記錄附註/更正表（Phase 2 使用）
-- ============================================

CREATE TABLE IF NOT EXISTS record_annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type VARCHAR(50) NOT NULL,      -- 'observation', 'surgery', 'sacrifice'
    record_id INTEGER NOT NULL,
    annotation_type VARCHAR(20) NOT NULL,  -- 'NOTE', 'CORRECTION', 'ADDENDUM'
    content TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    signature_id UUID REFERENCES electronic_signatures(id)  -- 如果是 CORRECTION，需要簽章
);

CREATE INDEX IF NOT EXISTS idx_record_annotations_record ON record_annotations(record_type, record_id);

-- ============================================
-- 完成
-- ============================================
