-- ============================================
-- Migration 019: 建立缺失的表與修正格式
-- 
-- 問題：
-- 1. change_reasons 表在程式碼中被引用但從未建立
-- 2. observation_vet_reads / surgery_vet_reads 需要確保存在
-- 3. pig_record_attachments.record_id 應為 UUID（與紀錄表 id 一致）
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 建立缺失的 change_reasons 表（程式碼引用但從未建立）
-- ============================================
CREATE TABLE IF NOT EXISTS change_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id TEXT NOT NULL,
    change_type VARCHAR(20) NOT NULL,
    reason TEXT NOT NULL,
    old_values JSONB,
    new_values JSONB,
    changed_fields TEXT[],
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_change_reasons_entity ON change_reasons(entity_type, entity_id);

-- ============================================
-- 2. 確保觀察/手術的獸醫已讀記錄表存在
-- ============================================

CREATE TABLE IF NOT EXISTS observation_vet_reads (
    observation_id UUID NOT NULL,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (observation_id, vet_user_id)
);

CREATE TABLE IF NOT EXISTS surgery_vet_reads (
    surgery_id UUID NOT NULL,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (surgery_id, vet_user_id)
);

-- ============================================
-- 完成
-- ============================================
