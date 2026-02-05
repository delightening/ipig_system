-- Migration: Role Permissions Update
-- 1. Emergency medication fields for pig observations
-- 2. Scrap approval fields for documents

-- ============================================
-- 1. 緊急給藥相關欄位
-- ============================================

-- 是否為緊急給藥
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS is_emergency BOOLEAN DEFAULT false;

-- 緊急給藥狀態: pending_review (待補簽), approved (已核准), rejected (已駁回)
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS emergency_status VARCHAR(50);

-- 緊急給藥原因
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS emergency_reason TEXT;

-- 審核者 (VET 或 PI)
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS reviewed_by UUID REFERENCES users(id);

-- 審核時間
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ;

-- ============================================
-- 2. 報廢簽核相關欄位
-- ============================================

-- 是否需要主管簽核 (金額超過門檻)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS requires_manager_approval BOOLEAN DEFAULT false;

-- 報廢總金額 (用於檢查是否超過門檻)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS scrap_total_amount NUMERIC(18,4);

-- 主管簽核狀態: pending (待簽核), approved (已核准), rejected (已駁回)
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_approval_status VARCHAR(50);

-- 主管簽核者
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_approved_by UUID REFERENCES users(id);

-- 主管簽核時間
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_approved_at TIMESTAMPTZ;

-- 主管駁回原因
ALTER TABLE documents ADD COLUMN IF NOT EXISTS manager_reject_reason TEXT;

-- ============================================
-- 3. 系統設定: 報廢簽核門檻
-- ============================================

-- 建立系統設定表 (如果不存在)
CREATE TABLE IF NOT EXISTS system_settings (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 報廢簽核門檻: 5000 元
INSERT INTO system_settings (key, value, description)
VALUES ('scrap_approval_threshold', '5000', '報廢金額超過此門檻需主管簽核 (SYSTEM_ADMIN)')
ON CONFLICT (key) DO NOTHING;

-- ============================================
-- 4. 新增緊急給藥權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES (gen_random_uuid(), 'animal.record.emergency', '緊急給藥', 'animal', '允許在緊急情況下執行給藥並待補簽', NOW())
ON CONFLICT (code) DO NOTHING;

-- 將權限賦予 EXPERIMENT_STAFF 角色
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code = 'animal.record.emergency'
ON CONFLICT DO NOTHING;

-- ============================================
-- 5. 確保 WAREHOUSE_MANAGER 沒有 AUP 相關權限
-- ============================================

-- 移除 WAREHOUSE_MANAGER 的 AUP 相關權限 (如果有的話)
DELETE FROM role_permissions
WHERE role_id = (SELECT id FROM roles WHERE code = 'WAREHOUSE_MANAGER')
AND permission_id IN (
    SELECT id FROM permissions WHERE code LIKE 'aup.%'
);
