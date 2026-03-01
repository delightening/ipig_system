-- ============================================
-- Migration 010: GLP、設備、會計、訓練權限
-- ============================================

-- 10.1 人員訓練紀錄
CREATE TABLE training_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_name VARCHAR(200) NOT NULL,
    completed_at DATE NOT NULL,
    expires_at DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_training_records_user ON training_records(user_id);
CREATE INDEX idx_training_records_expires ON training_records(expires_at) WHERE expires_at IS NOT NULL;

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'training.view', '查看訓練紀錄', 'training', '可查看人員訓練紀錄', NOW()),
    (gen_random_uuid(), 'training.manage', '管理訓練紀錄', 'training', '可新增、編輯、刪除訓練紀錄', NOW()),
    (gen_random_uuid(), 'training.manage_own', '管理自己的訓練紀錄', 'training', '可新增、編輯、刪除自己的訓練紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EXPERIMENT_STAFF' AND p.code IN ('training.view', 'training.manage_own')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 10.2 設備與校準
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

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'equipment.view', '查看設備', 'equipment', '可查看設備與校準紀錄', NOW()),
    (gen_random_uuid(), 'equipment.manage', '管理設備', 'equipment', '可新增、編輯、刪除設備與校準紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

-- 10.3 QAU 品質保證
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'qau.dashboard.view', '查看 QAU 儀表板', 'qau', 'GLP 品質保證：可查看研究狀態、審查進度、稽核摘要', NOW()),
    (gen_random_uuid(), 'qau.protocol.view', 'QAU 檢視計畫', 'qau', '唯讀檢視所有計畫書', NOW()),
    (gen_random_uuid(), 'qau.audit.view', 'QAU 檢視稽核', 'qau', '唯讀檢視稽核日誌', NOW()),
    (gen_random_uuid(), 'qau.animal.view', 'QAU 檢視動物', 'qau', '唯讀檢視動物紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'QAU', '品質保證單位', 'GLP 合規：獨立於研究執行，可檢視研究狀態、審查進度、稽核摘要、動物紀錄', true, true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'QAU' AND p.code IN (
    'qau.dashboard.view', 'qau.protocol.view', 'qau.audit.view', 'qau.animal.view',
    'aup.protocol.view_all', 'aup.review.view', 'aup.attachment.view', 'aup.attachment.download',
    'aup.version.view', 'audit.logs.view', 'animal.animal.view_all', 'animal.record.view'
)
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- 10.4 會計
CREATE TYPE account_type AS ENUM ('asset', 'liability', 'equity', 'revenue', 'expense');

CREATE TABLE chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    account_type account_type NOT NULL,
    parent_id UUID REFERENCES chart_of_accounts(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_chart_of_accounts_code ON chart_of_accounts(code);

CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_no VARCHAR(50) NOT NULL UNIQUE,
    entry_date DATE NOT NULL,
    description TEXT,
    source_entity_type VARCHAR(50),
    source_entity_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_journal_entries_date ON journal_entries(entry_date);
CREATE SEQUENCE IF NOT EXISTS journal_entry_no_seq START 1;

CREATE TABLE journal_entry_lines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    line_no INTEGER NOT NULL,
    account_id UUID NOT NULL REFERENCES chart_of_accounts(id),
    debit_amount NUMERIC(18, 4) NOT NULL DEFAULT 0,
    credit_amount NUMERIC(18, 4) NOT NULL DEFAULT 0,
    description TEXT,
    UNIQUE (journal_entry_id, line_no),
    CONSTRAINT chk_debit_credit CHECK ((debit_amount >= 0 AND credit_amount >= 0) AND ((debit_amount > 0 AND credit_amount = 0) OR (debit_amount = 0 AND credit_amount > 0)))
);
CREATE INDEX idx_journal_entry_lines_entry ON journal_entry_lines(journal_entry_id);

INSERT INTO chart_of_accounts (id, code, name, account_type) VALUES
(gen_random_uuid(),'1100','現金及約當現金','asset'),(gen_random_uuid(),'1200','應收帳款','asset'),(gen_random_uuid(),'1300','存貨','asset'),
(gen_random_uuid(),'2100','應付帳款','liability'),(gen_random_uuid(),'3100','業主權益','equity'),(gen_random_uuid(),'4100','銷貨收入','revenue'),
(gen_random_uuid(),'5100','進貨成本','expense'),(gen_random_uuid(),'5200','銷貨成本','expense')
ON CONFLICT (code) DO NOTHING;

CREATE TABLE ap_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_no VARCHAR(50) NOT NULL UNIQUE,
    partner_id UUID NOT NULL REFERENCES partners(id),
    payment_date DATE NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    reference TEXT,
    journal_entry_id UUID REFERENCES journal_entries(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE SEQUENCE IF NOT EXISTS ap_payment_no_seq START 1;
CREATE INDEX idx_ap_payments_partner ON ap_payments(partner_id);

CREATE TABLE ar_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    receipt_no VARCHAR(50) NOT NULL UNIQUE,
    partner_id UUID NOT NULL REFERENCES partners(id),
    receipt_date DATE NOT NULL,
    amount NUMERIC(18, 4) NOT NULL,
    reference TEXT,
    journal_entry_id UUID REFERENCES journal_entries(id),
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE SEQUENCE IF NOT EXISTS ar_receipt_no_seq START 1;
CREATE INDEX idx_ar_receipts_partner ON ar_receipts(partner_id);
