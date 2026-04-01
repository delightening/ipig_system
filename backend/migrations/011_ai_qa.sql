-- ============================================================
-- Migration 011: AI API Keys、AI 查詢日誌、QA 計畫管理模組
-- 來源: 017_ai_api_keys.sql, 022_add_missing_fk_indexes.sql (ai_api_keys CHECK),
--       029_qa_plan.sql (QA 表格 + 權限種子)
-- 前置依賴: 002_auth_users.sql (users, roles, permissions, role_permissions)
-- 注意: QA ENUMs 已定義於 001_enums.sql
-- ============================================================

-- ── ai_api_keys ──────────────────────────────────────────────
CREATE TABLE ai_api_keys (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name                  VARCHAR(100) NOT NULL,
    key_hash              VARCHAR(64)  NOT NULL UNIQUE,
    key_prefix            VARCHAR(12)  NOT NULL,
    created_by            UUID         NOT NULL REFERENCES users(id),
    scopes                JSONB        NOT NULL DEFAULT '["read"]'::jsonb,
    is_active             BOOLEAN      NOT NULL DEFAULT true,
    expires_at            TIMESTAMPTZ,
    last_used_at          TIMESTAMPTZ,
    usage_count           BIGINT       NOT NULL DEFAULT 0,
    rate_limit_per_minute INT          NOT NULL DEFAULT 60,
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    -- 022: CHECK constraint
    CONSTRAINT chk_rate_limit_positive CHECK (rate_limit_per_minute IS NULL OR rate_limit_per_minute > 0)
);
CREATE INDEX idx_ai_api_keys_key_hash ON ai_api_keys(key_hash);
CREATE INDEX idx_ai_api_keys_active   ON ai_api_keys(is_active) WHERE is_active = true;

-- ── ai_query_logs (月分區) ───────────────────────────────────
CREATE TABLE ai_query_logs (
    id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    api_key_id      UUID        NOT NULL,
    endpoint        VARCHAR(200) NOT NULL,
    method          VARCHAR(10)  NOT NULL DEFAULT 'GET',
    query_summary   JSONB,
    response_status SMALLINT    NOT NULL,
    duration_ms     INT          NOT NULL DEFAULT 0,
    source_ip       VARCHAR(45),
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- 動態建立當月與次月分區
DO $$
DECLARE
    curr_start  DATE := date_trunc('month', CURRENT_DATE);
    next_start  DATE := date_trunc('month', CURRENT_DATE + INTERVAL '1 month');
    after_next  DATE := date_trunc('month', CURRENT_DATE + INTERVAL '2 months');
    curr_suffix TEXT := to_char(CURRENT_DATE, 'YYYY_MM');
    next_suffix TEXT := to_char(CURRENT_DATE + INTERVAL '1 month', 'YYYY_MM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS ai_query_logs_%s PARTITION OF ai_query_logs FOR VALUES FROM (%L) TO (%L)',
        curr_suffix, curr_start, next_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS ai_query_logs_%s PARTITION OF ai_query_logs FOR VALUES FROM (%L) TO (%L)',
        next_suffix, next_start, after_next
    );
END $$;

CREATE INDEX idx_ai_query_logs_api_key ON ai_query_logs(api_key_id, created_at);

-- ============================================================
-- QA 計畫管理（來自 029_qa_plan.sql）
-- ENUMs: qa_inspection_type, qa_inspection_status, qa_item_result,
--        nc_severity, nc_source, nc_status, capa_action_type, capa_status,
--        sop_status, qa_schedule_type, qa_schedule_status, qa_schedule_item_status
--        均已定義於 001_enums.sql
-- ============================================================

-- ── qa_inspections ───────────────────────────────────────────
CREATE TABLE qa_inspections (
    id                  UUID                 PRIMARY KEY DEFAULT gen_random_uuid(),
    inspection_number   VARCHAR(50)          NOT NULL UNIQUE,
    title               VARCHAR(255)         NOT NULL,
    inspection_type     qa_inspection_type   NOT NULL,
    inspection_date     DATE                 NOT NULL,
    inspector_id        UUID                 NOT NULL REFERENCES users(id),
    related_entity_type VARCHAR(50),
    related_entity_id   UUID,
    status              qa_inspection_status NOT NULL DEFAULT 'draft',
    findings            TEXT,
    conclusion          TEXT,
    created_at          TIMESTAMPTZ          NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ          NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_qa_inspections_inspector ON qa_inspections(inspector_id);
CREATE INDEX idx_qa_inspections_date      ON qa_inspections(inspection_date);
CREATE INDEX idx_qa_inspections_status    ON qa_inspections(status);

-- ── qa_inspection_items ──────────────────────────────────────
CREATE TABLE qa_inspection_items (
    id            UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    inspection_id UUID           NOT NULL REFERENCES qa_inspections(id) ON DELETE CASCADE,
    item_order    INT            NOT NULL,
    description   TEXT           NOT NULL,
    result        qa_item_result NOT NULL DEFAULT 'not_applicable',
    remarks       TEXT,
    created_at    TIMESTAMPTZ    NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_qa_inspection_items_insp ON qa_inspection_items(inspection_id);

-- ── qa_non_conformances ──────────────────────────────────────
CREATE TABLE qa_non_conformances (
    id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    nc_number             VARCHAR(50) NOT NULL UNIQUE,
    title                 VARCHAR(255) NOT NULL,
    description           TEXT        NOT NULL,
    severity              nc_severity NOT NULL,
    source                nc_source   NOT NULL,
    related_inspection_id UUID        REFERENCES qa_inspections(id),
    assignee_id           UUID        REFERENCES users(id),
    due_date              DATE,
    status                nc_status   NOT NULL DEFAULT 'open',
    root_cause            TEXT,
    closure_notes         TEXT,
    closed_at             TIMESTAMPTZ,
    created_by            UUID        NOT NULL REFERENCES users(id),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_qa_nc_status     ON qa_non_conformances(status);
CREATE INDEX idx_qa_nc_assignee   ON qa_non_conformances(assignee_id);
CREATE INDEX idx_qa_nc_created_by ON qa_non_conformances(created_by);

-- ── qa_capa ──────────────────────────────────────────────────
CREATE TABLE qa_capa (
    id           UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
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
CREATE INDEX idx_qa_capa_nc ON qa_capa(nc_id);

-- ── qa_sop_documents ─────────────────────────────────────────
CREATE TABLE qa_sop_documents (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    document_number VARCHAR(50) NOT NULL UNIQUE,
    title           VARCHAR(255) NOT NULL,
    version         VARCHAR(20) NOT NULL,
    category        VARCHAR(100),
    file_path       VARCHAR(500),
    effective_date  DATE,
    review_date     DATE,
    status          sop_status  NOT NULL DEFAULT 'draft',
    description     TEXT,
    created_by      UUID        NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_qa_sop_status     ON qa_sop_documents(status);
CREATE INDEX idx_qa_sop_created_by ON qa_sop_documents(created_by);

-- ── qa_sop_acknowledgments ───────────────────────────────────
CREATE TABLE qa_sop_acknowledgments (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    sop_id          UUID        NOT NULL REFERENCES qa_sop_documents(id) ON DELETE CASCADE,
    user_id         UUID        NOT NULL REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sop_id, user_id)
);
CREATE INDEX idx_qa_sop_ack_sop  ON qa_sop_acknowledgments(sop_id);
CREATE INDEX idx_qa_sop_ack_user ON qa_sop_acknowledgments(user_id);

-- ── qa_audit_schedules ───────────────────────────────────────
CREATE TABLE qa_audit_schedules (
    id            UUID               PRIMARY KEY DEFAULT gen_random_uuid(),
    year          INT                NOT NULL,
    title         VARCHAR(255)       NOT NULL,
    schedule_type qa_schedule_type   NOT NULL DEFAULT 'annual',
    description   TEXT,
    status        qa_schedule_status NOT NULL DEFAULT 'planned',
    created_by    UUID               NOT NULL REFERENCES users(id),
    created_at    TIMESTAMPTZ        NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ        NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_qa_schedules_year   ON qa_audit_schedules(year);
CREATE INDEX idx_qa_schedules_status ON qa_audit_schedules(status);

-- ── qa_schedule_items ────────────────────────────────────────
CREATE TABLE qa_schedule_items (
    id                    UUID                    PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_id           UUID                    NOT NULL REFERENCES qa_audit_schedules(id) ON DELETE CASCADE,
    inspection_type       qa_inspection_type      NOT NULL,
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
CREATE INDEX idx_qa_schedule_items_sched ON qa_schedule_items(schedule_id);
CREATE INDEX idx_qa_schedule_items_date  ON qa_schedule_items(planned_date);

-- ── QAU 新增權限種子（來自 029） ─────────────────────────────
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'qau.inspection.view',   'QAU 檢視稽查報告',   'qau', '查看稽查報告列表與詳情',       NOW()),
    (gen_random_uuid(), 'qau.inspection.manage', 'QAU 管理稽查報告',   'qau', '建立、編輯、關閉稽查報告',     NOW()),
    (gen_random_uuid(), 'qau.nc.view',           'QAU 檢視不符合事項', 'qau', '查看 NC 與 CAPA 列表',         NOW()),
    (gen_random_uuid(), 'qau.nc.manage',         'QAU 管理不符合事項', 'qau', '建立、指派、結案不符合事項',   NOW()),
    (gen_random_uuid(), 'qau.sop.view',          'QAU 檢視 SOP',       'qau', '查看 SOP 文件列表',            NOW()),
    (gen_random_uuid(), 'qau.sop.manage',        'QAU 管理 SOP',       'qau', '建立、版本控制 SOP 文件',      NOW()),
    (gen_random_uuid(), 'qau.schedule.view',     'QAU 檢視稽查排程',   'qau', '查看年度稽查計畫',             NOW()),
    (gen_random_uuid(), 'qau.schedule.manage',   'QAU 管理稽查排程',   'qau', '建立、維護年度稽查計畫',       NOW())
ON CONFLICT (code) DO NOTHING;

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
