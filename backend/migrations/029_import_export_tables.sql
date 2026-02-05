-- =============================================================================
-- Migration 029: Import/Export Tables for Animal Management
-- 新增豬隻匯入批次記錄和匯出記錄相關表格和 enum 類型
-- =============================================================================

-- 匯入類型 enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'import_type') THEN
        CREATE TYPE import_type AS ENUM ('pig_basic', 'pig_weight');
    END IF;
END $$;

-- 匯入狀態 enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'import_status') THEN
        CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
    END IF;
END $$;

-- 匯出類型 enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'export_type') THEN
        CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
    END IF;
END $$;

-- 匯出格式 enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'export_format') THEN
        CREATE TYPE export_format AS ENUM ('pdf', 'excel');
    END IF;
END $$;

-- 豬隻匯入批次記錄表
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

-- 豬隻匯出記錄表
CREATE TABLE IF NOT EXISTS pig_export_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID REFERENCES pigs(id) ON DELETE SET NULL,
    iacuc_no VARCHAR(50),
    export_type export_type NOT NULL,
    export_format export_format NOT NULL,
    file_path VARCHAR(500),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_by ON pig_import_batches(created_by);
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_at ON pig_import_batches(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_pig_id ON pig_export_records(pig_id);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_iacuc_no ON pig_export_records(iacuc_no);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_created_at ON pig_export_records(created_at DESC);

-- 授予相關權限
INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), 'animal.info.import', '匯入動物基本資料', '實驗動物管理', '可批次匯入豬隻基本資料', NOW()
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE code = 'animal.info.import');

INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), 'animal.weight.import', '匯入動物體重資料', '實驗動物管理', '可批次匯入豬隻體重資料', NOW()
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE code = 'animal.weight.import');

INSERT INTO permissions (id, code, name, module, description, created_at)
SELECT gen_random_uuid(), 'animal.export.medical', '匯出動物醫療資料', '實驗動物管理', '可匯出豬隻病歷資料', NOW()
WHERE NOT EXISTS (SELECT 1 FROM permissions WHERE code = 'animal.export.medical');

-- 授予 SYSTEM_ADMIN 角色這些權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'SYSTEM_ADMIN'
  AND p.code IN ('animal.info.import', 'animal.weight.import', 'animal.export.medical')
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );

-- 授予 ADMIN_STAFF 角色這些權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'ADMIN_STAFF'
  AND p.code IN ('animal.info.import', 'animal.weight.import', 'animal.export.medical')
  AND NOT EXISTS (
    SELECT 1 FROM role_permissions rp 
    WHERE rp.role_id = r.id AND rp.permission_id = p.id
  );
