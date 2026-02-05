-- 擴展 Schema - 滿足完整規格需求
-- Version: 0.2

-- ============================================
-- 1. 擴展 users 表
-- ============================================

ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
ALTER TABLE users ADD COLUMN IF NOT EXISTS organization VARCHAR(200);
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_internal BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE users ADD COLUMN IF NOT EXISTS must_change_password BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMPTZ;

-- 更新現有用戶為內部人員
UPDATE users SET is_internal = true WHERE is_internal IS NULL;

-- ============================================
-- 2. 擴展 roles 表
-- ============================================

ALTER TABLE roles ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE roles ADD COLUMN IF NOT EXISTS is_internal BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE roles ADD COLUMN IF NOT EXISTS is_system BOOLEAN NOT NULL DEFAULT false;

-- 更新現有角色
UPDATE roles SET is_system = true WHERE code IN ('admin', 'warehouse', 'purchasing', 'sales', 'approver');

-- ============================================
-- 3. 擴展 permissions 表
-- ============================================

ALTER TABLE permissions ADD COLUMN IF NOT EXISTS module VARCHAR(50);
ALTER TABLE permissions ADD COLUMN IF NOT EXISTS description TEXT;

-- 更新現有權限的模組
UPDATE permissions SET module = 'admin' WHERE code LIKE 'user.%' OR code LIKE 'role.%';
UPDATE permissions SET module = 'erp' WHERE code LIKE 'warehouse.%' OR code LIKE 'product.%' OR code LIKE 'partner.%';
UPDATE permissions SET module = 'erp' WHERE code LIKE 'document.%' OR code LIKE 'po.%' OR code LIKE 'grn.%' OR code LIKE 'pr.%';
UPDATE permissions SET module = 'erp' WHERE code LIKE 'so.%' OR code LIKE 'do.%' OR code LIKE 'sr.%';
UPDATE permissions SET module = 'erp' WHERE code LIKE 'tr.%' OR code LIKE 'stk.%' OR code LIKE 'adj.%';
UPDATE permissions SET module = 'erp' WHERE code LIKE 'stock.%' OR code LIKE 'report.%';

-- ============================================
-- 4. 擴展 user_roles 表
-- ============================================

ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS assigned_at TIMESTAMPTZ DEFAULT NOW();
ALTER TABLE user_roles ADD COLUMN IF NOT EXISTS assigned_by UUID REFERENCES users(id);

-- ============================================
-- 5. 擴展 products 表
-- ============================================

ALTER TABLE products ADD COLUMN IF NOT EXISTS pack_unit VARCHAR(20);
ALTER TABLE products ADD COLUMN IF NOT EXISTS pack_qty INTEGER;

-- ============================================
-- 6. 擴展 audit_logs 表
-- ============================================

ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS ip_address VARCHAR(45);
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- ============================================
-- 7. 新增 SKU 類別表
-- ============================================

-- SKU 主類別
CREATE TABLE IF NOT EXISTS sku_categories (
    code CHAR(3) PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SKU 子類別
CREATE TABLE IF NOT EXISTS sku_subcategories (
    id SERIAL PRIMARY KEY,
    category_code CHAR(3) NOT NULL REFERENCES sku_categories(code) ON DELETE CASCADE,
    code CHAR(3) NOT NULL,
    name VARCHAR(50) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (category_code, code)
);

-- SKU 流水號追蹤
CREATE TABLE IF NOT EXISTS sku_sequences (
    category_code CHAR(3) NOT NULL,
    subcategory_code CHAR(3) NOT NULL,
    last_sequence INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (category_code, subcategory_code)
);

-- 插入 SKU 類別資料
INSERT INTO sku_categories (code, name, sort_order) VALUES
    ('MED', '藥品', 1),
    ('FED', '飼料', 2),
    ('EQP', '器材', 3),
    ('CON', '耗材', 4),
    ('OTH', '其他', 5)
ON CONFLICT (code) DO NOTHING;

-- 插入 SKU 子類別資料 - 藥品
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('MED', 'ANE', '麻醉劑', 1),
    ('MED', 'ANT', '抗生素', 2),
    ('MED', 'VAC', '疫苗', 3),
    ('MED', 'PAI', '止痛劑', 4),
    ('MED', 'DEW', '驅蟲劑', 5),
    ('MED', 'OPH', '眼科藥', 6),
    ('MED', 'TOP', '外用藥', 7),
    ('MED', 'INJ', '注射劑', 8),
    ('MED', 'ORL', '口服藥', 9),
    ('MED', 'OTH', '其他', 10)
ON CONFLICT (category_code, code) DO NOTHING;

-- 插入 SKU 子類別資料 - 飼料
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('FED', 'PIG', '豬用飼料', 1),
    ('FED', 'MIN', '迷你豬飼料', 2),
    ('FED', 'SUP', '營養補充', 3),
    ('FED', 'OTH', '其他', 4)
ON CONFLICT (category_code, code) DO NOTHING;

-- 插入 SKU 子類別資料 - 器材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('EQP', 'SUR', '手術器材', 1),
    ('EQP', 'MON', '監測設備', 2),
    ('EQP', 'IMG', '影像設備', 3),
    ('EQP', 'ANE', '麻醉設備', 4),
    ('EQP', 'RES', '保定設備', 5),
    ('EQP', 'WEI', '量測設備', 6),
    ('EQP', 'OTH', '其他', 7)
ON CONFLICT (category_code, code) DO NOTHING;

-- 插入 SKU 子類別資料 - 耗材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CON', 'SYR', '針筒注射', 1),
    ('CON', 'GLV', '手套', 2),
    ('CON', 'GAU', '紗布敷料', 3),
    ('CON', 'TUB', '管路耗材', 4),
    ('CON', 'CLN', '清潔消毒', 5),
    ('CON', 'TAG', '標示耗材', 6),
    ('CON', 'OTH', '其他', 7)
ON CONFLICT (category_code, code) DO NOTHING;

-- 插入 SKU 子類別資料 - 其他
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('OTH', 'GEN', '一般', 1)
ON CONFLICT (category_code, code) DO NOTHING;

-- ============================================
-- 8. 新增密碼重設 Token 表
-- ============================================

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);

-- ============================================
-- 9. 新增 user_protocols 表（外部人員計畫關聯）
-- ============================================

-- 計畫中角色類型
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT');

CREATE TABLE IF NOT EXISTS user_protocols (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    protocol_id UUID NOT NULL,
    role_in_protocol protocol_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, protocol_id)
);

CREATE INDEX IF NOT EXISTS idx_user_protocols_user_id ON user_protocols(user_id);
CREATE INDEX IF NOT EXISTS idx_user_protocols_protocol_id ON user_protocols(protocol_id);

-- ============================================
-- 10. AUP 審查系統表
-- ============================================

-- 計畫狀態類型
CREATE TYPE protocol_status AS ENUM (
    'DRAFT',
    'SUBMITTED',
    'PRE_REVIEW',
    'UNDER_REVIEW',
    'REVISION_REQUIRED',
    'RESUBMITTED',
    'APPROVED',
    'APPROVED_WITH_CONDITIONS',
    'DEFERRED',
    'REJECTED',
    'SUSPENDED',
    'CLOSED'
);

-- 計畫書主表
CREATE TABLE IF NOT EXISTS protocols (
    id UUID PRIMARY KEY,
    protocol_no VARCHAR(50) NOT NULL UNIQUE,
    iacuc_no VARCHAR(50) UNIQUE,
    title VARCHAR(500) NOT NULL,
    status protocol_status NOT NULL DEFAULT 'DRAFT',
    pi_user_id UUID NOT NULL REFERENCES users(id),
    working_content JSONB,
    start_date DATE,
    end_date DATE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_protocols_status ON protocols(status);
CREATE INDEX IF NOT EXISTS idx_protocols_pi_user_id ON protocols(pi_user_id);
CREATE INDEX IF NOT EXISTS idx_protocols_iacuc_no ON protocols(iacuc_no);

-- 計畫版本快照
CREATE TABLE IF NOT EXISTS protocol_versions (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (protocol_id, version_no)
);

CREATE INDEX IF NOT EXISTS idx_protocol_versions_protocol_id ON protocol_versions(protocol_id);

-- 計畫狀態歷程
CREATE TABLE IF NOT EXISTS protocol_status_history (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    from_status protocol_status,
    to_status protocol_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_protocol_status_history_protocol_id ON protocol_status_history(protocol_id);

-- 審查人員指派
CREATE TABLE IF NOT EXISTS review_assignments (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    UNIQUE (protocol_id, reviewer_id)
);

CREATE INDEX IF NOT EXISTS idx_review_assignments_protocol_id ON review_assignments(protocol_id);
CREATE INDEX IF NOT EXISTS idx_review_assignments_reviewer_id ON review_assignments(reviewer_id);

-- 審查意見
CREATE TABLE IF NOT EXISTS review_comments (
    id UUID PRIMARY KEY,
    protocol_version_id UUID NOT NULL REFERENCES protocol_versions(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_review_comments_protocol_version_id ON review_comments(protocol_version_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_reviewer_id ON review_comments(reviewer_id);

-- 附件表
CREATE TABLE IF NOT EXISTS protocol_attachments (
    id UUID PRIMARY KEY,
    protocol_version_id UUID REFERENCES protocol_versions(id) ON DELETE CASCADE,
    protocol_id UUID REFERENCES protocols(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_protocol_attachments_protocol_id ON protocol_attachments(protocol_id);
CREATE INDEX IF NOT EXISTS idx_protocol_attachments_protocol_version_id ON protocol_attachments(protocol_version_id);

-- ============================================
-- 11. 實驗動物管理系統表
-- ============================================

-- 豬隻狀態類型
CREATE TYPE pig_status AS ENUM ('unassigned', 'assigned', 'in_experiment', 'completed');

-- 豬隻品種類型
CREATE TYPE pig_breed AS ENUM ('miniature', 'white', 'other');

-- 豬隻性別類型
CREATE TYPE pig_gender AS ENUM ('male', 'female');

-- 紀錄類型
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');

-- 豬隻來源表
CREATE TABLE IF NOT EXISTS pig_sources (
    id UUID PRIMARY KEY,
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    address TEXT,
    contact VARCHAR(100),
    phone VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 插入預設豬隻來源
INSERT INTO pig_sources (id, code, name, sort_order) VALUES
    (gen_random_uuid(), 'TAITUNG', '台東種畜繁殖場', 1),
    (gen_random_uuid(), 'QINGXIN', '青欣牧場', 2),
    (gen_random_uuid(), 'PIGMODEL', '豬博士畜牧場', 3),
    (gen_random_uuid(), 'PINGSHUN', '平順牧場', 4)
ON CONFLICT (code) DO NOTHING;

-- 豬隻主表
CREATE TABLE IF NOT EXISTS pigs (
    id SERIAL PRIMARY KEY,
    ear_tag VARCHAR(10) NOT NULL UNIQUE,
    status pig_status NOT NULL DEFAULT 'unassigned',
    breed pig_breed NOT NULL,
    source_id UUID REFERENCES pig_sources(id),
    gender pig_gender NOT NULL,
    birth_date DATE,
    entry_date DATE NOT NULL,
    entry_weight NUMERIC(5, 1),
    pen_location VARCHAR(10),
    pre_experiment_code VARCHAR(20),
    iacuc_no VARCHAR(20),
    experiment_date DATE,
    remark TEXT,
    vet_weight_viewed_at TIMESTAMPTZ,
    vet_vaccine_viewed_at TIMESTAMPTZ,
    vet_sacrifice_viewed_at TIMESTAMPTZ,
    vet_last_viewed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pigs_ear_tag ON pigs(ear_tag);
CREATE INDEX IF NOT EXISTS idx_pigs_status ON pigs(status);
CREATE INDEX IF NOT EXISTS idx_pigs_iacuc_no ON pigs(iacuc_no);
CREATE INDEX IF NOT EXISTS idx_pigs_pen_location ON pigs(pen_location);

-- 觀察試驗紀錄表
CREATE TABLE IF NOT EXISTS pig_observations (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    event_date DATE NOT NULL,
    record_type record_type NOT NULL,
    equipment_used JSONB,
    anesthesia_start TIMESTAMPTZ,
    anesthesia_end TIMESTAMPTZ,
    content TEXT NOT NULL,
    no_medication_needed BOOLEAN NOT NULL DEFAULT false,
    treatments JSONB,
    remark TEXT,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_observations_pig_id ON pig_observations(pig_id);
CREATE INDEX IF NOT EXISTS idx_pig_observations_event_date ON pig_observations(event_date);

-- 手術紀錄表
CREATE TABLE IF NOT EXISTS pig_surgeries (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    is_first_experiment BOOLEAN NOT NULL DEFAULT true,
    surgery_date DATE NOT NULL,
    surgery_site VARCHAR(200) NOT NULL,
    induction_anesthesia JSONB,
    pre_surgery_medication JSONB,
    positioning VARCHAR(100),
    anesthesia_maintenance JSONB,
    anesthesia_observation TEXT,
    vital_signs JSONB,
    reflex_recovery TEXT,
    respiration_rate INTEGER,
    post_surgery_medication JSONB,
    remark TEXT,
    no_medication_needed BOOLEAN NOT NULL DEFAULT false,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_surgeries_pig_id ON pig_surgeries(pig_id);
CREATE INDEX IF NOT EXISTS idx_pig_surgeries_surgery_date ON pig_surgeries(surgery_date);

-- 體重紀錄表
CREATE TABLE IF NOT EXISTS pig_weights (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    measure_date DATE NOT NULL,
    weight NUMERIC(5, 1) NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_weights_pig_id ON pig_weights(pig_id);
CREATE INDEX IF NOT EXISTS idx_pig_weights_measure_date ON pig_weights(measure_date);

-- 疫苗/驅蟲紀錄表
CREATE TABLE IF NOT EXISTS pig_vaccinations (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    administered_date DATE NOT NULL,
    vaccine VARCHAR(100),
    deworming_dose VARCHAR(100),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_vaccinations_pig_id ON pig_vaccinations(pig_id);

-- 犧牲/採樣紀錄表
CREATE TABLE IF NOT EXISTS pig_sacrifices (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE UNIQUE,
    sacrifice_date DATE,
    zoletil_dose VARCHAR(50),
    method_electrocution BOOLEAN NOT NULL DEFAULT false,
    method_bloodletting BOOLEAN NOT NULL DEFAULT false,
    method_other TEXT,
    sampling TEXT,
    sampling_other TEXT,
    blood_volume_ml NUMERIC(6, 1),
    confirmed_sacrifice BOOLEAN NOT NULL DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_sacrifices_pig_id ON pig_sacrifices(pig_id);

-- 病理組織報告表
CREATE TABLE IF NOT EXISTS pig_pathology_reports (
    id SERIAL PRIMARY KEY,
    pig_id INTEGER NOT NULL REFERENCES pigs(id) ON DELETE CASCADE UNIQUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 紀錄附件通用表
CREATE TYPE pig_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology');
CREATE TYPE pig_file_type AS ENUM ('photo', 'attachment', 'report');

CREATE TABLE IF NOT EXISTS pig_record_attachments (
    id UUID PRIMARY KEY,
    record_type pig_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    file_type pig_file_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pig_record_attachments_record ON pig_record_attachments(record_type, record_id);

-- 獸醫師建議表
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');

CREATE TABLE IF NOT EXISTS vet_recommendations (
    id SERIAL PRIMARY KEY,
    record_type vet_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vet_recommendations_record ON vet_recommendations(record_type, record_id);

-- 照護給藥紀錄表
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');

CREATE TABLE IF NOT EXISTS care_medication_records (
    id SERIAL PRIMARY KEY,
    record_type vet_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    record_mode care_record_mode NOT NULL DEFAULT 'pain_assessment',
    post_op_days INTEGER,
    time_period VARCHAR(20),
    spirit VARCHAR(50),
    appetite VARCHAR(50),
    mobility_standing VARCHAR(50),
    mobility_walking VARCHAR(50),
    attitude_behavior VARCHAR(50),
    vet_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_care_medication_records_record ON care_medication_records(record_type, record_id);

-- 紀錄版本歷史表
CREATE TYPE version_record_type AS ENUM ('observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology');

CREATE TABLE IF NOT EXISTS record_versions (
    id SERIAL PRIMARY KEY,
    record_type version_record_type NOT NULL,
    record_id INTEGER NOT NULL,
    version_no INTEGER NOT NULL,
    snapshot JSONB NOT NULL,
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_record_versions_record ON record_versions(record_type, record_id);

-- ============================================
-- 12. 新增統一角色定義
-- ============================================

-- 插入系統預設角色（符合 role.md 規格）
INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'SYSTEM_ADMIN', '系統管理員', '全系統最高權限，使用者管理、系統維運', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'WAREHOUSE_MANAGER', '倉庫管理員', '專責 ERP 進銷存系統（採購、庫存、盤點、報表）', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'PROGRAM_ADMIN', '程式管理員', '系統程式層級管理', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'PI', '計畫主持人', '提交計畫、管理自己的計畫與豬隻', false, true, NOW(), NOW()),
    (gen_random_uuid(), 'VET', '獸醫師', '審查計畫、豬隻健康管理、提供建議', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'REVIEWER', '審查委員', 'IACUC 計畫審查', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'CHAIR', 'IACUC 主席', '主導審查決策', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'IACUC_STAFF', '執行秘書', '行政流程管理、管理所有計劃進度', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'EXPERIMENT_STAFF', '試驗工作人員', '執行實驗操作、記錄數據、查詢 ERP 物資現況', true, true, NOW(), NOW()),
    (gen_random_uuid(), 'CLIENT', '委託人', '查看委託計畫與豬隻紀錄', false, true, NOW(), NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    is_internal = EXCLUDED.is_internal,
    is_system = EXCLUDED.is_system,
    updated_at = NOW();

-- ============================================
-- 13. 新增完整權限定義
-- ============================================

-- AUP 系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'aup.protocol.view_all', '查看所有計畫', 'aup', '可查看系統中所有計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.view_own', '查看自己的計畫', 'aup', '可查看自己相關的計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.create', '建立計畫', 'aup', '可建立新計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.edit', '編輯計畫', 'aup', '可編輯計畫草稿', NOW()),
    (gen_random_uuid(), 'aup.protocol.submit', '提交計畫', 'aup', '可提交計畫送審', NOW()),
    (gen_random_uuid(), 'aup.protocol.review', '審查計畫', 'aup', '可審查計畫並提供意見', NOW()),
    (gen_random_uuid(), 'aup.protocol.approve', '核准/否決', 'aup', '可核准或否決計畫', NOW()),
    (gen_random_uuid(), 'aup.protocol.change_status', '變更狀態', 'aup', '可變更計畫狀態', NOW()),
    (gen_random_uuid(), 'aup.review.assign', '指派審查人員', 'aup', '可指派審查人員', NOW()),
    (gen_random_uuid(), 'aup.review.comment', '新增審查意見', 'aup', '可新增審查意見', NOW())
ON CONFLICT (code) DO NOTHING;

-- 實驗動物管理系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'animal.pig.view_all', '查看所有豬隻', 'animal', '可查看所有豬隻資料', NOW()),
    (gen_random_uuid(), 'animal.pig.view_project', '查看計畫內豬隻', 'animal', '可查看計畫內的豬隻', NOW()),
    (gen_random_uuid(), 'animal.pig.create', '新增豬隻', 'animal', '可新增豬隻', NOW()),
    (gen_random_uuid(), 'animal.pig.edit', '編輯豬隻資料', 'animal', '可編輯豬隻基本資料', NOW()),
    (gen_random_uuid(), 'animal.pig.assign', '分配豬隻至計畫', 'animal', '可將豬隻分配至計畫', NOW()),
    (gen_random_uuid(), 'animal.pig.import', '匯入豬隻資料', 'animal', '可批次匯入豬隻資料', NOW()),
    (gen_random_uuid(), 'animal.record.create', '新增紀錄', 'animal', '可新增豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.edit', '編輯紀錄', 'animal', '可編輯豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.delete', '刪除紀錄', 'animal', '可刪除豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.vet.recommend', '新增獸醫師建議', 'animal', '可新增獸醫師建議', NOW()),
    (gen_random_uuid(), 'animal.vet.read', '標記獸醫師已讀', 'animal', '可標記紀錄已讀', NOW()),
    (gen_random_uuid(), 'animal.export.medical', '匯出病歷', 'animal', '可匯出豬隻病歷', NOW())
ON CONFLICT (code) DO NOTHING;

-- 系統管理權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'admin.user.view', '查看使用者', 'admin', '可查看使用者列表', NOW()),
    (gen_random_uuid(), 'admin.user.create', '建立使用者', 'admin', '可建立新使用者帳號', NOW()),
    (gen_random_uuid(), 'admin.user.edit', '編輯使用者', 'admin', '可編輯使用者資料', NOW()),
    (gen_random_uuid(), 'admin.user.delete', '停用使用者', 'admin', '可停用使用者帳號', NOW()),
    (gen_random_uuid(), 'admin.user.reset_password', '重設密碼', 'admin', '可重設他人密碼', NOW()),
    (gen_random_uuid(), 'admin.role.manage', '管理角色', 'admin', '可管理角色定義', NOW()),
    (gen_random_uuid(), 'admin.permission.manage', '管理權限', 'admin', '可管理權限定義', NOW()),
    (gen_random_uuid(), 'admin.audit.view', '查看稽核紀錄', 'admin', '可查看系統稽核紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

-- ERP 系統權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'erp.inventory.view', '查看庫存', 'erp', '可查看庫存現況', NOW()),
    (gen_random_uuid(), 'erp.purchase.create', '建立採購單', 'erp', '可建立採購單', NOW()),
    (gen_random_uuid(), 'erp.purchase.approve', '核准採購單', 'erp', '可核准採購單', NOW()),
    (gen_random_uuid(), 'erp.stock.in', '入庫操作', 'erp', '可執行入庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.out', '出庫操作', 'erp', '可執行出庫操作', NOW()),
    (gen_random_uuid(), 'erp.stock.adjust', '庫存調整', 'erp', '可執行庫存調整', NOW()),
    (gen_random_uuid(), 'erp.stock.transfer', '調撥', 'erp', '可執行庫存調撥', NOW()),
    (gen_random_uuid(), 'erp.stocktake.create', '盤點', 'erp', '可執行庫存盤點', NOW()),
    (gen_random_uuid(), 'erp.report.view', '查看報表', 'erp', '可查看 ERP 報表', NOW()),
    (gen_random_uuid(), 'erp.report.export', '匯出報表', 'erp', '可匯出 ERP 報表', NOW())
ON CONFLICT (code) DO NOTHING;
