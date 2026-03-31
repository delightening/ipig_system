-- Migration 029: QA 計畫管理（稽查報告、不符合事項、SOP 文件、稽查排程）

-- ============================================================
-- 29.1 Enum Types
-- ============================================================

CREATE TYPE qa_inspection_type AS ENUM (
    'protocol',
    'equipment',
    'facility',
    'training',
    'general'
);

CREATE TYPE qa_inspection_status AS ENUM (
    'draft',
    'submitted',
    'closed'
);

CREATE TYPE qa_item_result AS ENUM (
    'pass',
    'fail',
    'not_applicable'
);

CREATE TYPE nc_severity AS ENUM (
    'critical',
    'major',
    'minor'
);

CREATE TYPE nc_source AS ENUM (
    'inspection',
    'observation',
    'external_audit',
    'self_report'
);

CREATE TYPE nc_status AS ENUM (
    'open',
    'in_progress',
    'pending_verification',
    'closed'
);

CREATE TYPE capa_action_type AS ENUM (
    'corrective',
    'preventive'
);

CREATE TYPE capa_status AS ENUM (
    'open',
    'in_progress',
    'completed',
    'verified'
);

CREATE TYPE sop_status AS ENUM (
    'draft',
    'active',
    'obsolete'
);

CREATE TYPE qa_schedule_type AS ENUM (
    'annual',
    'periodic',
    'ad_hoc'
);

CREATE TYPE qa_schedule_status AS ENUM (
    'planned',
    'in_progress',
    'completed',
    'cancelled'
);

CREATE TYPE qa_schedule_item_status AS ENUM (
    'planned',
    'in_progress',
    'completed',
    'cancelled',
    'overdue'
);

-- ============================================================
-- 29.2 稽查報告（QA Inspection Report）
-- ============================================================

CREATE TABLE qa_inspections (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inspection_number   VARCHAR(50)  NOT NULL UNIQUE,
    title               VARCHAR(255) NOT NULL,
    inspection_type     qa_inspection_type NOT NULL,
    inspection_date     DATE         NOT NULL,
    inspector_id        UUID         NOT NULL REFERENCES users(id),
    related_entity_type VARCHAR(50),
    related_entity_id   UUID,
    status              qa_inspection_status NOT NULL DEFAULT 'draft',
    findings            TEXT,
    conclusion          TEXT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE qa_inspection_items (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    inspection_id UUID NOT NULL REFERENCES qa_inspections(id) ON DELETE CASCADE,
    item_order    INT  NOT NULL,
    description   TEXT NOT NULL,
    result        qa_item_result NOT NULL DEFAULT 'not_applicable',
    remarks       TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_qa_inspections_inspector    ON qa_inspections(inspector_id);
CREATE INDEX idx_qa_inspections_date         ON qa_inspections(inspection_date);
CREATE INDEX idx_qa_inspections_status       ON qa_inspections(status);
CREATE INDEX idx_qa_inspection_items_insp    ON qa_inspection_items(inspection_id);

-- ============================================================
-- 29.3 不符合事項（Non-Conformance & CAPA）
-- ============================================================

CREATE TABLE qa_non_conformances (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nc_number              VARCHAR(50)  NOT NULL UNIQUE,
    title                  VARCHAR(255) NOT NULL,
    description            TEXT         NOT NULL,
    severity               nc_severity  NOT NULL,
    source                 nc_source    NOT NULL,
    related_inspection_id  UUID         REFERENCES qa_inspections(id),
    assignee_id            UUID         REFERENCES users(id),
    due_date               DATE,
    status                 nc_status    NOT NULL DEFAULT 'open',
    root_cause             TEXT,
    closure_notes          TEXT,
    closed_at              TIMESTAMPTZ,
    created_by             UUID         NOT NULL REFERENCES users(id),
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE qa_capa (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nc_id        UUID             NOT NULL REFERENCES qa_non_conformances(id) ON DELETE CASCADE,
    action_type  capa_action_type NOT NULL,
    description  TEXT             NOT NULL,
    assignee_id  UUID             REFERENCES users(id),
    due_date     DATE,
    completed_at TIMESTAMPTZ,
    status       capa_status      NOT NULL DEFAULT 'open',
    created_at   TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_qa_nc_status     ON qa_non_conformances(status);
CREATE INDEX idx_qa_nc_assignee   ON qa_non_conformances(assignee_id);
CREATE INDEX idx_qa_nc_created_by ON qa_non_conformances(created_by);
CREATE INDEX idx_qa_capa_nc       ON qa_capa(nc_id);

-- ============================================================
-- 29.4 SOP 文件管理
-- ============================================================

CREATE TABLE qa_sop_documents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_number VARCHAR(50)  NOT NULL UNIQUE,
    title           VARCHAR(255) NOT NULL,
    version         VARCHAR(20)  NOT NULL,
    category        VARCHAR(100),
    file_path       VARCHAR(500),
    effective_date  DATE,
    review_date     DATE,
    status          sop_status   NOT NULL DEFAULT 'draft',
    description     TEXT,
    created_by      UUID         NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE qa_sop_acknowledgments (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sop_id         UUID        NOT NULL REFERENCES qa_sop_documents(id) ON DELETE CASCADE,
    user_id        UUID        NOT NULL REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sop_id, user_id)
);

CREATE INDEX idx_qa_sop_status      ON qa_sop_documents(status);
CREATE INDEX idx_qa_sop_created_by  ON qa_sop_documents(created_by);
CREATE INDEX idx_qa_sop_ack_sop     ON qa_sop_acknowledgments(sop_id);
CREATE INDEX idx_qa_sop_ack_user    ON qa_sop_acknowledgments(user_id);

-- ============================================================
-- 29.5 QA 稽查排程（Audit Schedule）
-- ============================================================

CREATE TABLE qa_audit_schedules (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    year          INT              NOT NULL,
    title         VARCHAR(255)     NOT NULL,
    schedule_type qa_schedule_type NOT NULL DEFAULT 'annual',
    description   TEXT,
    status        qa_schedule_status NOT NULL DEFAULT 'planned',
    created_by    UUID             NOT NULL REFERENCES users(id),
    created_at    TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE qa_schedule_items (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id           UUID                    NOT NULL REFERENCES qa_audit_schedules(id) ON DELETE CASCADE,
    inspection_type       qa_inspection_type       NOT NULL,
    title                 VARCHAR(255)             NOT NULL,
    planned_date          DATE                     NOT NULL,
    actual_date           DATE,
    responsible_person_id UUID                     REFERENCES users(id),
    related_inspection_id UUID                     REFERENCES qa_inspections(id),
    status                qa_schedule_item_status  NOT NULL DEFAULT 'planned',
    notes                 TEXT,
    created_at            TIMESTAMPTZ              NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ              NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_qa_schedules_year       ON qa_audit_schedules(year);
CREATE INDEX idx_qa_schedules_status     ON qa_audit_schedules(status);
CREATE INDEX idx_qa_schedule_items_sched ON qa_schedule_items(schedule_id);
CREATE INDEX idx_qa_schedule_items_date  ON qa_schedule_items(planned_date);

-- ============================================================
-- 29.6 新增 QAU 權限
-- ============================================================

INSERT INTO permissions (id, code, name, module, description, created_at)
VALUES
    (gen_random_uuid(), 'qau.inspection.view',     'QAU 檢視稽查報告',   'qau', '查看稽查報告列表與詳情',         NOW()),
    (gen_random_uuid(), 'qau.inspection.manage',   'QAU 管理稽查報告',   'qau', '建立、編輯、關閉稽查報告',        NOW()),
    (gen_random_uuid(), 'qau.nc.view',             'QAU 檢視不符合事項', 'qau', '查看 NC 與 CAPA 列表',           NOW()),
    (gen_random_uuid(), 'qau.nc.manage',           'QAU 管理不符合事項', 'qau', '建立、指派、結案不符合事項',      NOW()),
    (gen_random_uuid(), 'qau.sop.view',            'QAU 檢視 SOP',       'qau', '查看 SOP 文件列表',              NOW()),
    (gen_random_uuid(), 'qau.sop.manage',          'QAU 管理 SOP',       'qau', '建立、版本控制 SOP 文件',        NOW()),
    (gen_random_uuid(), 'qau.schedule.view',       'QAU 檢視稽查排程',   'qau', '查看年度稽查計畫',               NOW()),
    (gen_random_uuid(), 'qau.schedule.manage',     'QAU 管理稽查排程',   'qau', '建立、維護年度稽查計畫',         NOW())
ON CONFLICT (code) DO NOTHING;

-- 將所有新 QAU 權限綁定至 QAU 角色
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.code = 'QAU'
  AND p.code IN (
    'qau.inspection.view', 'qau.inspection.manage',
    'qau.nc.view',         'qau.nc.manage',
    'qau.sop.view',        'qau.sop.manage',
    'qau.schedule.view',   'qau.schedule.manage'
  )
ON CONFLICT DO NOTHING;
