-- 實驗動物管理系統 - 前端需求補充
-- Version: 0.4
-- 根據前端需求確認補充缺少的欄位和表

-- ============================================
-- 1. pig_observations 補充欄位
-- ============================================

-- 軟刪除支援
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- 複製紀錄來源
ALTER TABLE pig_observations ADD COLUMN IF NOT EXISTS copied_from_id INTEGER REFERENCES pig_observations(id);

-- 建立軟刪除索引
CREATE INDEX IF NOT EXISTS idx_pig_observations_deleted_at ON pig_observations(deleted_at) WHERE deleted_at IS NULL;

-- ============================================
-- 2. pig_surgeries 補充欄位
-- ============================================

-- 軟刪除支援
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- 複製紀錄來源
ALTER TABLE pig_surgeries ADD COLUMN IF NOT EXISTS copied_from_id INTEGER REFERENCES pig_surgeries(id);

-- 建立軟刪除索引
CREATE INDEX IF NOT EXISTS idx_pig_surgeries_deleted_at ON pig_surgeries(deleted_at) WHERE deleted_at IS NULL;

-- ============================================
-- 3. pig_weights 補充欄位
-- ============================================

-- 軟刪除支援
ALTER TABLE pig_weights ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- 建立軟刪除索引
CREATE INDEX IF NOT EXISTS idx_pig_weights_deleted_at ON pig_weights(deleted_at) WHERE deleted_at IS NULL;

-- ============================================
-- 4. pig_vaccinations 補充欄位
-- ============================================

-- 軟刪除支援
ALTER TABLE pig_vaccinations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- 建立軟刪除索引
CREATE INDEX IF NOT EXISTS idx_pig_vaccinations_deleted_at ON pig_vaccinations(deleted_at) WHERE deleted_at IS NULL;

-- ============================================
-- 5. vet_recommendations 補充欄位
-- ============================================

-- 附件支援（含圖片）
ALTER TABLE vet_recommendations ADD COLUMN IF NOT EXISTS attachments JSONB;

-- ============================================
-- 6. 新增匯入批次記錄表
-- ============================================

CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE import_type AS ENUM ('pig_basic', 'pig_weight');

CREATE TABLE IF NOT EXISTS pig_import_batches (
    id UUID PRIMARY KEY,
    import_type import_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    total_rows INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    status import_status NOT NULL DEFAULT 'pending',
    error_details JSONB,  -- 存放錯誤列資訊 [{row: 12, ear_tag: "801", error: "耳號已存在"}]
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_pig_import_batches_status ON pig_import_batches(status);
CREATE INDEX IF NOT EXISTS idx_pig_import_batches_created_by ON pig_import_batches(created_by);

-- ============================================
-- 7. 新增匯出記錄表
-- ============================================

CREATE TYPE export_format AS ENUM ('pdf', 'excel');
CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');

CREATE TABLE IF NOT EXISTS pig_export_records (
    id UUID PRIMARY KEY,
    pig_id INTEGER REFERENCES pigs(id),
    iacuc_no VARCHAR(20),
    export_type export_type NOT NULL,
    export_format export_format NOT NULL,
    file_path VARCHAR(500),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_export_records_pig_id ON pig_export_records(pig_id);
CREATE INDEX IF NOT EXISTS idx_pig_export_records_iacuc_no ON pig_export_records(iacuc_no);

-- ============================================
-- 8. 補充缺少的權限定義
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- 複製紀錄權限
    (gen_random_uuid(), 'animal.record.copy', '複製紀錄', 'animal', '可複製觀察/手術紀錄作為範本', NOW()),
    -- 匯出權限
    (gen_random_uuid(), 'animal.export.observation', '匯出觀察紀錄', 'animal', '可匯出豬隻觀察紀錄', NOW()),
    (gen_random_uuid(), 'animal.export.surgery', '匯出手術紀錄', 'animal', '可匯出豬隻手術紀錄', NOW()),
    (gen_random_uuid(), 'animal.export.experiment', '匯出試驗紀錄', 'animal', '可匯出豬隻試驗紀錄', NOW()),
    -- 病理報告權限
    (gen_random_uuid(), 'animal.pathology.upload', '上傳病理報告', 'animal', '可上傳豬隻病理組織報告', NOW()),
    (gen_random_uuid(), 'animal.pathology.view', '查看病理報告', 'animal', '可查看豬隻病理組織報告', NOW()),
    -- 獸醫師附件權限
    (gen_random_uuid(), 'animal.vet.upload_attachment', '獸醫師上傳附件', 'animal', '可上傳獸醫師建議附件', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 9. 角色權限關聯（為現有角色補充新權限）
-- ============================================

-- 為 SYSTEM_ADMIN 角色補充所有新權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'SYSTEM_ADMIN'
AND p.code IN (
    'animal.record.copy',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment',
    'animal.pathology.upload',
    'animal.pathology.view',
    'animal.vet.upload_attachment'
)
ON CONFLICT DO NOTHING;

-- 為 VET 角色補充獸醫師相關權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'VET'
AND p.code IN (
    'animal.record.copy',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment',
    'animal.pathology.view',
    'animal.vet.upload_attachment'
)
ON CONFLICT DO NOTHING;

-- 為 IACUC_STAFF 角色補充權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'IACUC_STAFF'
AND p.code IN (
    'animal.record.copy',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment',
    'animal.pathology.upload',
    'animal.pathology.view'
)
ON CONFLICT DO NOTHING;

-- 為 EXPERIMENT_STAFF 角色補充權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
AND p.code IN (
    'animal.record.copy',
    'animal.export.observation',
    'animal.export.surgery',
    'animal.export.experiment'
)
ON CONFLICT DO NOTHING;

-- 為 CLIENT 角色補充匯出權限（病歷總表、觀察紀錄、手術紀錄）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'CLIENT'
AND p.code IN (
    'animal.export.medical',
    'animal.export.observation',
    'animal.export.surgery'
)
ON CONFLICT DO NOTHING;

-- 為 PI 角色補充匯出權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'PI'
AND p.code IN (
    'animal.export.medical',
    'animal.export.observation',
    'animal.export.surgery'
)
ON CONFLICT DO NOTHING;

-- ============================================
-- 10. 補充 record_versions 相關功能
-- ============================================

-- 新增差異比對欄位
ALTER TABLE record_versions ADD COLUMN IF NOT EXISTS diff_summary TEXT;

-- 建立複合索引優化版本查詢
CREATE INDEX IF NOT EXISTS idx_record_versions_record_version ON record_versions(record_type, record_id, version_no);

-- ============================================
-- 11. 觀察紀錄已讀標記表（追蹤獸醫師閱讀記錄）
-- ============================================

CREATE TABLE IF NOT EXISTS observation_vet_reads (
    id SERIAL PRIMARY KEY,
    observation_id INTEGER NOT NULL REFERENCES pig_observations(id) ON DELETE CASCADE,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (observation_id, vet_user_id)
);

CREATE INDEX IF NOT EXISTS idx_observation_vet_reads_observation ON observation_vet_reads(observation_id);

-- ============================================
-- 12. 手術紀錄已讀標記表
-- ============================================

CREATE TABLE IF NOT EXISTS surgery_vet_reads (
    id SERIAL PRIMARY KEY,
    surgery_id INTEGER NOT NULL REFERENCES pig_surgeries(id) ON DELETE CASCADE,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (surgery_id, vet_user_id)
);

CREATE INDEX IF NOT EXISTS idx_surgery_vet_reads_surgery ON surgery_vet_reads(surgery_id);
