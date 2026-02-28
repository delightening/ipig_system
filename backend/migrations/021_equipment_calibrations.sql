-- ============================================
-- Migration 021: 設備與校準紀錄 (實驗室 GLP)
--
-- 包含：
-- - equipment 表
-- - equipment_calibrations 表
-- - equipment.view, equipment.manage 權限
--
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 設備表
-- ============================================

CREATE TABLE equipment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    model VARCHAR(200),
    serial_number VARCHAR(100),
    location VARCHAR(200),
    notes TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_equipment_name ON equipment(name);
CREATE INDEX idx_equipment_active ON equipment(is_active);

-- ============================================
-- 2. 設備校準紀錄表
-- ============================================

CREATE TABLE equipment_calibrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    calibrated_at DATE NOT NULL,
    next_due_at DATE,
    result VARCHAR(50),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_equipment_calibrations_equipment ON equipment_calibrations(equipment_id);
CREATE INDEX idx_equipment_calibrations_next_due ON equipment_calibrations(next_due_at) WHERE next_due_at IS NOT NULL;

-- ============================================
-- 3. 設備權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'equipment.view', '查看設備', 'equipment', '可查看設備與校準紀錄', NOW()),
    (gen_random_uuid(), 'equipment.manage', '管理設備', 'equipment', '可新增、編輯、刪除設備與校準紀錄', NOW())
ON CONFLICT (code) DO NOTHING;
