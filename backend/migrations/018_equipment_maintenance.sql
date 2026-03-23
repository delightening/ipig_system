-- ============================================
-- Migration 018: 設備維護管理系統擴充
-- 設備狀態、廠商關聯、校正/確效/查核、維修/保養、報廢、年度計畫
-- ============================================

-- 18.1 新增 Enum 型別
CREATE TYPE equipment_status AS ENUM ('active', 'inactive', 'under_repair', 'decommissioned');
CREATE TYPE calibration_type AS ENUM ('calibration', 'validation', 'inspection');
CREATE TYPE calibration_cycle AS ENUM ('monthly', 'quarterly', 'semi_annual', 'annual');
CREATE TYPE maintenance_type AS ENUM ('repair', 'maintenance');
CREATE TYPE maintenance_status AS ENUM ('pending', 'in_progress', 'completed', 'unrepairable');
CREATE TYPE disposal_status AS ENUM ('pending', 'approved', 'rejected');

-- 18.2 擴充 equipment 表
-- 新增狀態欄位（取代 is_active 的簡單布林）
ALTER TABLE equipment ADD COLUMN status equipment_status NOT NULL DEFAULT 'active';
-- 設備的校正/確效類型（二擇一）
ALTER TABLE equipment ADD COLUMN calibration_type calibration_type;
-- 校正/確效週期
ALTER TABLE equipment ADD COLUMN calibration_cycle calibration_cycle;
-- 查核週期（可選）
ALTER TABLE equipment ADD COLUMN inspection_cycle calibration_cycle;

-- 同步既有 is_active 資料到新 status 欄位
UPDATE equipment SET status = CASE WHEN is_active THEN 'active'::equipment_status ELSE 'inactive'::equipment_status END;

-- 建立索引
CREATE INDEX idx_equipment_status ON equipment(status);

-- 18.3 設備-廠商關聯表（多對多，關聯 partners 表）
CREATE TABLE equipment_suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    partner_id UUID NOT NULL REFERENCES partners(id) ON DELETE CASCADE,
    contact_person VARCHAR(100),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (equipment_id, partner_id)
);
CREATE INDEX idx_equipment_suppliers_equipment ON equipment_suppliers(equipment_id);
CREATE INDEX idx_equipment_suppliers_partner ON equipment_suppliers(partner_id);

-- 18.4 擴充 equipment_calibrations 表（校正/確效/查核 三種措施）
ALTER TABLE equipment_calibrations ADD COLUMN calibration_type calibration_type NOT NULL DEFAULT 'calibration';
-- 校正/確效廠商（關聯 partners 表）
ALTER TABLE equipment_calibrations ADD COLUMN partner_id UUID REFERENCES partners(id);
-- 校正報告編號
ALTER TABLE equipment_calibrations ADD COLUMN report_number VARCHAR(100);
-- 查核人員（自由輸入）
ALTER TABLE equipment_calibrations ADD COLUMN inspector VARCHAR(100);
-- 設備序號（denormalized，方便查詢顯示）
ALTER TABLE equipment_calibrations ADD COLUMN equipment_serial_number VARCHAR(100);

CREATE INDEX idx_equipment_calibrations_type ON equipment_calibrations(calibration_type);

-- 18.5 設備狀態變更紀錄（audit trail）
CREATE TABLE equipment_status_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    old_status equipment_status NOT NULL,
    new_status equipment_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_status_logs_equipment ON equipment_status_logs(equipment_id);
CREATE INDEX idx_equipment_status_logs_created ON equipment_status_logs(created_at DESC);

-- 18.6 維修/保養紀錄
CREATE TABLE equipment_maintenance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    maintenance_type maintenance_type NOT NULL,
    status maintenance_status NOT NULL DEFAULT 'pending',
    -- 維修欄位
    reported_at DATE NOT NULL,
    completed_at DATE,
    problem_description TEXT,
    repair_content TEXT,
    -- 維修廠商（關聯 partners 表）
    repair_partner_id UUID REFERENCES partners(id),
    -- 保養欄位
    maintenance_items TEXT,
    performed_by VARCHAR(100),
    -- 共用
    notes TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_maintenance_equipment ON equipment_maintenance_records(equipment_id);
CREATE INDEX idx_equipment_maintenance_type ON equipment_maintenance_records(maintenance_type);
CREATE INDEX idx_equipment_maintenance_status ON equipment_maintenance_records(status);

-- 18.7 報廢紀錄（含簽核流程）
CREATE TABLE equipment_disposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    status disposal_status NOT NULL DEFAULT 'pending',
    -- 申請資訊
    disposal_date DATE,
    reason TEXT NOT NULL,
    disposal_method TEXT,
    applied_by UUID NOT NULL REFERENCES users(id),
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 核准資訊
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    -- 電子簽章
    applicant_signature_id UUID REFERENCES electronic_signatures(id),
    approver_signature_id UUID REFERENCES electronic_signatures(id),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_disposals_equipment ON equipment_disposals(equipment_id);
CREATE INDEX idx_equipment_disposals_status ON equipment_disposals(status);

-- 18.8 年度維護校正計畫表
CREATE TABLE equipment_annual_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    year INTEGER NOT NULL,
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    calibration_type calibration_type NOT NULL,
    cycle calibration_cycle NOT NULL,
    -- 12 個月的排定狀態（true = 該月需執行）
    month_1 BOOLEAN NOT NULL DEFAULT false,
    month_2 BOOLEAN NOT NULL DEFAULT false,
    month_3 BOOLEAN NOT NULL DEFAULT false,
    month_4 BOOLEAN NOT NULL DEFAULT false,
    month_5 BOOLEAN NOT NULL DEFAULT false,
    month_6 BOOLEAN NOT NULL DEFAULT false,
    month_7 BOOLEAN NOT NULL DEFAULT false,
    month_8 BOOLEAN NOT NULL DEFAULT false,
    month_9 BOOLEAN NOT NULL DEFAULT false,
    month_10 BOOLEAN NOT NULL DEFAULT false,
    month_11 BOOLEAN NOT NULL DEFAULT false,
    month_12 BOOLEAN NOT NULL DEFAULT false,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (year, equipment_id, calibration_type)
);
CREATE INDEX idx_equipment_annual_plans_year ON equipment_annual_plans(year);
CREATE INDEX idx_equipment_annual_plans_equipment ON equipment_annual_plans(equipment_id);

-- 18.9 新增通知類型（擴充 notification_type enum）
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'equipment_overdue';
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'equipment_unrepairable';
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'equipment_disposal';

-- 18.10 新增權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'equipment.disposal.approve', '核准設備報廢', 'equipment', '可核准設備報廢申請', NOW()),
    (gen_random_uuid(), 'equipment.maintenance.manage', '管理維修保養', 'equipment', '可新增、編輯維修保養紀錄', NOW()),
    (gen_random_uuid(), 'equipment.plan.manage', '管理年度計畫', 'equipment', '可產生與編輯年度維護校正計畫表', NOW())
ON CONFLICT (code) DO NOTHING;

-- 賦予 EQUIPMENT_MAINTENANCE 角色新權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EQUIPMENT_MAINTENANCE' AND p.code IN (
    'equipment.maintenance.manage',
    'equipment.plan.manage'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 18.11 新增通知路由
INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
    ('equipment_overdue', 'EQUIPMENT_MAINTENANCE', 'both', '設備校正/確效逾期提醒'),
    ('equipment_unrepairable', 'EQUIPMENT_MAINTENANCE', 'both', '設備無法維修通知'),
    ('equipment_unrepairable', 'admin', 'both', '設備無法維修通知（機構負責人）'),
    ('equipment_disposal', 'EQUIPMENT_MAINTENANCE', 'in_app', '設備報廢申請通知'),
    ('equipment_disposal', 'admin', 'both', '設備報廢申請通知（機構負責人）')
ON CONFLICT (event_type, role_code) DO NOTHING;
