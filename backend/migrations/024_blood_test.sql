-- ============================================
-- Migration 024: Blood Test System
-- 
-- 包含：
-- - 血液檢查項目模板主檔
-- - 血液檢查主表
-- - 血液檢查項目明細表
-- - ERP 單據新增 iacuc_no 欄位
-- - Enum 擴展
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 血液檢查項目模板主檔
-- ============================================

CREATE TABLE blood_test_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    default_unit VARCHAR(50),
    reference_range VARCHAR(100),
    default_price NUMERIC(10, 2) DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blood_test_templates_code ON blood_test_templates(code);
CREATE INDEX idx_blood_test_templates_is_active ON blood_test_templates(is_active);

-- 預設模板種子資料由 025_seed_blood_templates.sql 統一管理

-- ============================================
-- 2. 血液檢查主表
-- ============================================

CREATE TABLE pig_blood_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    test_date DATE NOT NULL,
    lab_name VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    remark TEXT,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id),
    delete_reason TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_blood_test_status CHECK (status IN ('pending', 'completed'))
);

CREATE INDEX idx_pig_blood_tests_pig_id ON pig_blood_tests(pig_id);
CREATE INDEX idx_pig_blood_tests_test_date ON pig_blood_tests(test_date);
CREATE INDEX idx_pig_blood_tests_is_deleted ON pig_blood_tests(is_deleted);

-- ============================================
-- 3. 血液檢查項目明細表
-- ============================================

CREATE TABLE pig_blood_test_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blood_test_id UUID NOT NULL REFERENCES pig_blood_tests(id) ON DELETE CASCADE,
    template_id UUID REFERENCES blood_test_templates(id),
    item_name VARCHAR(200) NOT NULL,
    result_value VARCHAR(100),
    result_unit VARCHAR(50),
    reference_range VARCHAR(100),
    is_abnormal BOOLEAN NOT NULL DEFAULT false,
    remark TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_blood_test_items_blood_test_id ON pig_blood_test_items(blood_test_id);
CREATE INDEX idx_pig_blood_test_items_template_id ON pig_blood_test_items(template_id);

-- ============================================
-- 4. ERP 單據表新增 iacuc_no 欄位
-- ============================================

ALTER TABLE documents ADD COLUMN IF NOT EXISTS iacuc_no VARCHAR(20);
CREATE INDEX IF NOT EXISTS idx_documents_iacuc_no ON documents(iacuc_no);

-- ============================================
-- 5. Enum 擴展
-- ============================================

ALTER TYPE pig_record_type ADD VALUE IF NOT EXISTS 'blood_test';
ALTER TYPE version_record_type ADD VALUE IF NOT EXISTS 'blood_test';

-- ============================================
-- 完成
-- ============================================
