-- ============================================
-- Migration 020: 新增動物紀錄表缺失欄位與權限
-- 
-- 問題：
-- 1. Rust 程式碼引用了多個欄位，但從未在 migration 中建立
-- 2. 多個權限代碼在程式碼中被使用但從未定義
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- Part A: 新增缺失的表欄位
-- ============================================

-- 1. pig_observations 軟刪除欄位
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- 2. pig_observations 緊急給藥欄位
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS is_emergency BOOLEAN DEFAULT false;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS emergency_status VARCHAR(20);
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS emergency_reason TEXT;
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS reviewed_by UUID REFERENCES users(id);
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ;

-- 3. pig_surgeries 軟刪除欄位
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- 4. pig_weights 軟刪除欄位
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- 5. pig_vaccinations 軟刪除欄位
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deletion_reason TEXT;
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- 6. vet_recommendations 新增缺失欄位
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS is_urgent BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS attachments JSONB;

-- 7. record_versions 新增缺失欄位
ALTER TABLE record_versions ADD COLUMN IF NOT EXISTS diff_summary TEXT;

-- ============================================
-- Part B: 新增缺失的權限定義
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES
    (gen_random_uuid(), 'animal.pathology.view', '查看病理報告', 'animal', '可查看動物病理報告', NOW()),
    (gen_random_uuid(), 'animal.pathology.upload', '上傳病理報告', 'animal', '可上傳動物病理報告', NOW()),
    (gen_random_uuid(), 'animal.info.assign', '分配動物資訊', 'animal', '可分配動物至計畫', NOW()),
    (gen_random_uuid(), 'animal.info.edit', '編輯動物資訊', 'animal', '可編輯動物資訊', NOW()),
    (gen_random_uuid(), 'animal.record.emergency', '緊急給藥', 'animal', '可執行緊急給藥', NOW()),
    (gen_random_uuid(), 'animal.record.copy', '複製紀錄', 'animal', '可複製動物紀錄', NOW()),
    (gen_random_uuid(), 'animal.vet.upload_attachment', '上傳獸醫附件', 'animal', '可上傳獸醫建議附件', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- Part C: 分配權限給 EXPERIMENT_STAFF 角色
-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    'animal.pathology.view',
    'animal.pathology.upload',
    'animal.info.assign',
    'animal.info.edit',
    'animal.record.emergency',
    'animal.record.copy',
    'animal.vet.upload_attachment'
)
ON CONFLICT DO NOTHING;

-- 也分配給 VET（獸醫師）角色
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'VET'
AND p.code IN (
    'animal.pathology.view',
    'animal.pathology.upload',
    'animal.info.assign',
    'animal.info.edit',
    'animal.record.emergency',
    'animal.record.copy',
    'animal.vet.upload_attachment'
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 完成
-- ============================================
