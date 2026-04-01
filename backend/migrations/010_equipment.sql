-- ============================================================
-- Migration 010: 人員訓練 + 設備維護管理（含廠商、校正、維修、報廢、年度計畫）
-- 來源: 009_glp_extensions.sql (training_records, equipment 初版, 權限種子),
--       018_equipment_maintenance.sql (equipment FINAL 擴充, 所有新表),
--       022_add_missing_fk_indexes.sql (FK indexes)
-- 前置依賴: 002_auth_users.sql (users, roles, permissions),
--           003_notifications.sql (notification_routing, electronic_signatures),
--           009_erp_stock.sql (partners)
-- ============================================================

-- ── training_records ─────────────────────────────────────────
CREATE TABLE training_records (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    course_name  VARCHAR(200) NOT NULL,
    completed_at DATE        NOT NULL,
    expires_at   DATE,
    notes        TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_training_records_user    ON training_records(user_id);
CREATE INDEX idx_training_records_expires ON training_records(expires_at) WHERE expires_at IS NOT NULL;

-- ── equipment ────────────────────────────────────────────────
-- 直接以 018 FINAL 狀態建立（status/calibration_type/calibration_cycle/inspection_cycle 直接加入）
CREATE TABLE equipment (
    id               UUID              PRIMARY KEY DEFAULT gen_random_uuid(),
    name             VARCHAR(200)      NOT NULL,
    model            VARCHAR(200),
    serial_number    VARCHAR(100),
    location         VARCHAR(200),
    notes            TEXT,
    is_active        BOOLEAN           NOT NULL DEFAULT true,
    -- 018: 新增欄位（初版只有 is_active；018 加入 status + calibration 欄位）
    status           equipment_status  NOT NULL DEFAULT 'active',
    calibration_type calibration_type,
    calibration_cycle calibration_cycle,
    inspection_cycle  calibration_cycle,
    created_at       TIMESTAMPTZ       NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ       NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_name   ON equipment(name);
CREATE INDEX idx_equipment_active ON equipment(is_active);
CREATE INDEX idx_equipment_status ON equipment(status);

-- ── equipment_calibrations ───────────────────────────────────
-- 直接以 018 FINAL 狀態建立（calibration_type/partner_id/report_number/inspector/equipment_serial_number）
CREATE TABLE equipment_calibrations (
    id                       UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id             UUID             NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    calibrated_at            DATE             NOT NULL,
    next_due_at              DATE,
    result                   VARCHAR(50),
    notes                    TEXT,
    -- 018: 新增欄位
    calibration_type         calibration_type NOT NULL DEFAULT 'calibration',
    partner_id               UUID             REFERENCES partners(id),
    report_number            VARCHAR(100),
    inspector                VARCHAR(100),
    equipment_serial_number  VARCHAR(100),
    created_at               TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_calibrations_equipment  ON equipment_calibrations(equipment_id);
CREATE INDEX idx_equipment_calibrations_type       ON equipment_calibrations(calibration_type);
CREATE INDEX idx_equipment_calibrations_partner_id ON equipment_calibrations(partner_id);

-- ── equipment_suppliers ──────────────────────────────────────
CREATE TABLE equipment_suppliers (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id   UUID        NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    partner_id     UUID        NOT NULL REFERENCES partners(id) ON DELETE CASCADE,
    contact_person VARCHAR(100),
    contact_phone  VARCHAR(50),
    contact_email  VARCHAR(255),
    notes          TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (equipment_id, partner_id)
);
CREATE INDEX idx_equipment_suppliers_equipment ON equipment_suppliers(equipment_id);
CREATE INDEX idx_equipment_suppliers_partner   ON equipment_suppliers(partner_id);

-- ── equipment_status_logs ────────────────────────────────────
CREATE TABLE equipment_status_logs (
    id           UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID             NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    old_status   equipment_status NOT NULL,
    new_status   equipment_status NOT NULL,
    changed_by   UUID             NOT NULL REFERENCES users(id),
    reason       TEXT,
    created_at   TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_status_logs_equipment ON equipment_status_logs(equipment_id);
CREATE INDEX idx_equipment_status_logs_created   ON equipment_status_logs(created_at DESC);

-- ── equipment_maintenance_records ───────────────────────────
CREATE TABLE equipment_maintenance_records (
    id                  UUID               PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id        UUID               NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    maintenance_type    maintenance_type   NOT NULL,
    status              maintenance_status NOT NULL DEFAULT 'pending',
    reported_at         DATE               NOT NULL,
    completed_at        DATE,
    problem_description TEXT,
    repair_content      TEXT,
    repair_partner_id   UUID               REFERENCES partners(id),
    maintenance_items   TEXT,
    performed_by        VARCHAR(100),
    notes               TEXT,
    created_by          UUID               NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ        NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ        NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_maintenance_equipment     ON equipment_maintenance_records(equipment_id);
CREATE INDEX idx_equipment_maintenance_type          ON equipment_maintenance_records(maintenance_type);
CREATE INDEX idx_equipment_maintenance_status        ON equipment_maintenance_records(status);
CREATE INDEX idx_equipment_maintenance_repair_partner ON equipment_maintenance_records(repair_partner_id);

-- ── equipment_disposals ──────────────────────────────────────
CREATE TABLE equipment_disposals (
    id                     UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id           UUID            NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    status                 disposal_status NOT NULL DEFAULT 'pending',
    disposal_date          DATE,
    reason                 TEXT            NOT NULL,
    disposal_method        TEXT,
    applied_by             UUID            NOT NULL REFERENCES users(id),
    applied_at             TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    approved_by            UUID            REFERENCES users(id),
    approved_at            TIMESTAMPTZ,
    rejection_reason       TEXT,
    applicant_signature_id UUID            REFERENCES electronic_signatures(id),
    approver_signature_id  UUID            REFERENCES electronic_signatures(id),
    notes                  TEXT,
    created_at             TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_disposals_equipment ON equipment_disposals(equipment_id);
CREATE INDEX idx_equipment_disposals_status    ON equipment_disposals(status);

-- ── equipment_annual_plans ───────────────────────────────────
CREATE TABLE equipment_annual_plans (
    id               UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    year             INTEGER          NOT NULL,
    equipment_id     UUID             NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    calibration_type calibration_type NOT NULL,
    cycle            calibration_cycle NOT NULL,
    month_1          BOOLEAN          NOT NULL DEFAULT false,
    month_2          BOOLEAN          NOT NULL DEFAULT false,
    month_3          BOOLEAN          NOT NULL DEFAULT false,
    month_4          BOOLEAN          NOT NULL DEFAULT false,
    month_5          BOOLEAN          NOT NULL DEFAULT false,
    month_6          BOOLEAN          NOT NULL DEFAULT false,
    month_7          BOOLEAN          NOT NULL DEFAULT false,
    month_8          BOOLEAN          NOT NULL DEFAULT false,
    month_9          BOOLEAN          NOT NULL DEFAULT false,
    month_10         BOOLEAN          NOT NULL DEFAULT false,
    month_11         BOOLEAN          NOT NULL DEFAULT false,
    month_12         BOOLEAN          NOT NULL DEFAULT false,
    generated_at     TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    created_at       TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    UNIQUE (year, equipment_id, calibration_type)
);
CREATE INDEX idx_equipment_annual_plans_year      ON equipment_annual_plans(year);
CREATE INDEX idx_equipment_annual_plans_equipment ON equipment_annual_plans(equipment_id);

-- ── 訓練與設備權限種子 ────────────────────────────────────────
-- 來自 009_glp_extensions.sql
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'training.view',       '查看訓練紀錄',       'training',  '可查看人員訓練紀錄',                   NOW()),
    (gen_random_uuid(), 'training.manage',     '管理訓練紀錄',       'training',  '可新增、編輯、刪除訓練紀錄',           NOW()),
    (gen_random_uuid(), 'training.manage_own', '管理自己的訓練紀錄', 'training',  '可新增、編輯、刪除自己的訓練紀錄',     NOW()),
    (gen_random_uuid(), 'equipment.view',      '查看設備',           'equipment', '可查看設備與校準紀錄',                 NOW()),
    (gen_random_uuid(), 'equipment.manage',    '管理設備',           'equipment', '可新增、編輯、刪除設備與校準紀錄',     NOW())
ON CONFLICT (code) DO NOTHING;

-- 來自 018_equipment_maintenance.sql
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'equipment.disposal.approve',  '核准設備報廢',   'equipment', '可核准設備報廢申請',                   NOW()),
    (gen_random_uuid(), 'equipment.maintenance.manage','管理維修保養',   'equipment', '可新增、編輯維修保養紀錄',             NOW()),
    (gen_random_uuid(), 'equipment.plan.manage',       '管理年度計畫',   'equipment', '可產生與編輯年度維護校正計畫表',       NOW())
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EXPERIMENT_STAFF'
  AND p.code IN ('training.view', 'training.manage_own')
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.code = 'EQUIPMENT_MAINTENANCE'
  AND p.code IN (
    'equipment.view', 'equipment.manage',
    'training.view', 'training.manage_own',
    'dashboard.view',
    'equipment.maintenance.manage',
    'equipment.plan.manage'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- ── 通知路由種子（設備事件） ──────────────────────────────────
INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
    ('equipment_overdue',      'EQUIPMENT_MAINTENANCE', 'both',   '設備校正/確效逾期提醒'),
    ('equipment_unrepairable', 'EQUIPMENT_MAINTENANCE', 'both',   '設備無法維修通知'),
    ('equipment_unrepairable', 'admin',                 'both',   '設備無法維修通知（機構負責人）'),
    ('equipment_disposal',     'EQUIPMENT_MAINTENANCE', 'in_app', '設備報廢申請通知'),
    ('equipment_disposal',     'admin',                 'both',   '設備報廢申請通知（機構負責人）')
ON CONFLICT (event_type, role_code) DO NOTHING;
