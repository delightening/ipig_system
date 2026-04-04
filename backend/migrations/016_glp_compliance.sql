-- ============================================================================
-- Migration 016: GLP / ISO 17025 / ISO 9001 合規改進
-- 涵蓋: P0 (角色/校正追溯) + P1 (文件控制/管理審查/風險/變更控制/環境監控/能力評鑑/最終報告/試驗物質)
-- ============================================================================

-- ============================================================================
-- P0-1 & P0-2: GLP 角色 — Study Director + Test Facility Management
-- ============================================================================

INSERT INTO roles (id, code, name, description, is_system, is_internal, created_at) VALUES
    (gen_random_uuid(), 'STUDY_DIRECTOR', '研究主持人 (Study Director)', 'GLP 研究主持人，負責研究之整體執行與最終報告簽署', true, false, NOW()),
    (gen_random_uuid(), 'TEST_FACILITY_MANAGEMENT', '試驗機構管理階層 (TFM)', 'GLP 試驗機構管理階層，負責 GLP 遵循與資源配置', true, true, NOW())
ON CONFLICT (code) DO NOTHING;

-- GLP 專用權限
INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- GLP 核心
    (gen_random_uuid(), 'glp.study_director.designate', '指定 Study Director', 'glp', '可指定研究之 Study Director', NOW()),
    (gen_random_uuid(), 'glp.study_report.sign', '簽署最終報告', 'glp', 'Study Director 簽署最終研究報告', NOW()),
    (gen_random_uuid(), 'glp.compliance.overview', 'GLP 遵循總覽', 'glp', '查看 GLP 遵循狀態儀表板', NOW()),
    -- 管理審查
    (gen_random_uuid(), 'glp.management_review.view', '查看管理審查', 'glp', '檢視管理審查紀錄', NOW()),
    (gen_random_uuid(), 'glp.management_review.manage', '管理管理審查', 'glp', '建立、編輯管理審查', NOW()),
    -- 文件控制
    (gen_random_uuid(), 'dms.document.view', '查看受控文件', 'dms', '檢視受控文件列表與詳情', NOW()),
    (gen_random_uuid(), 'dms.document.manage', '管理受控文件', 'dms', '建立、編輯、審核受控文件', NOW()),
    (gen_random_uuid(), 'dms.document.approve', '核准受控文件', 'dms', '核准受控文件發行', NOW()),
    -- 風險管理
    (gen_random_uuid(), 'risk.register.view', '查看風險登記簿', 'risk', '檢視風險評估紀錄', NOW()),
    (gen_random_uuid(), 'risk.register.manage', '管理風險登記簿', 'risk', '建立、評估、處理風險', NOW()),
    -- 變更控制
    (gen_random_uuid(), 'change.request.view', '查看變更申請', 'change', '檢視變更申請紀錄', NOW()),
    (gen_random_uuid(), 'change.request.manage', '管理變更申請', 'change', '建立、提交變更申請', NOW()),
    (gen_random_uuid(), 'change.request.approve', '核准變更申請', 'change', '審核並核准變更申請', NOW()),
    -- 環境監控
    (gen_random_uuid(), 'env.monitoring.view', '查看環境監控', 'env', '檢視環境監控點與紀錄', NOW()),
    (gen_random_uuid(), 'env.monitoring.manage', '管理環境監控', 'env', '建立監控點、登錄環境數據', NOW()),
    -- 能力評鑑
    (gen_random_uuid(), 'competency.assessment.view', '查看能力評鑑', 'competency', '檢視能力評鑑紀錄', NOW()),
    (gen_random_uuid(), 'competency.assessment.manage', '管理能力評鑑', 'competency', '建立、執行能力評鑑', NOW()),
    -- 最終報告
    (gen_random_uuid(), 'study.report.view', '查看最終報告', 'study', '檢視研究最終報告', NOW()),
    (gen_random_uuid(), 'study.report.manage', '管理最終報告', 'study', '建立、編輯研究最終報告', NOW()),
    -- 配製紀錄
    (gen_random_uuid(), 'formulation.record.view', '查看配製紀錄', 'formulation', '檢視試驗物質配製紀錄', NOW()),
    (gen_random_uuid(), 'formulation.record.manage', '管理配製紀錄', 'formulation', '建立、編輯配製紀錄', NOW())
ON CONFLICT (code) DO NOTHING;

-- STUDY_DIRECTOR 角色權限（繼承 PI 核心 + GLP 專用）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'STUDY_DIRECTOR' AND p.code = ANY(ARRAY[
    -- PI 核心權限
    'aup.protocol.view_own', 'aup.protocol.create', 'aup.protocol.edit',
    'aup.protocol.submit', 'aup.protocol.delete',
    'aup.review.view', 'aup.review.reply',
    'aup.attachment.view', 'aup.attachment.download', 'aup.attachment.upload', 'aup.attachment.delete',
    'aup.version.view', 'aup.version.restore',
    'animal.animal.view_project', 'animal.record.view',
    'animal.export.medical', 'animal.export.observation', 'animal.export.surgery',
    'dashboard.view',
    -- GLP 專用
    'glp.study_report.sign', 'glp.compliance.overview',
    'study.report.view', 'study.report.manage',
    'formulation.record.view', 'formulation.record.manage'
])
ON CONFLICT DO NOTHING;

-- TEST_FACILITY_MANAGEMENT 角色權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'TEST_FACILITY_MANAGEMENT' AND p.code = ANY(ARRAY[
    -- GLP 管理
    'glp.study_director.designate', 'glp.compliance.overview',
    'glp.management_review.view', 'glp.management_review.manage',
    -- 全域唯讀
    'aup.protocol.view_all', 'aup.review.view',
    'aup.attachment.view', 'aup.attachment.download', 'aup.version.view',
    'animal.animal.view_all', 'animal.record.view',
    'qau.dashboard.view', 'qau.inspection.view', 'qau.nc.view', 'qau.sop.view', 'qau.schedule.view',
    'audit.logs.view',
    -- 管理系統
    'dms.document.view', 'dms.document.manage', 'dms.document.approve',
    'risk.register.view', 'risk.register.manage',
    'change.request.view', 'change.request.manage', 'change.request.approve',
    'env.monitoring.view',
    'competency.assessment.view',
    'study.report.view',
    'dashboard.view'
])
ON CONFLICT DO NOTHING;

-- QAU 角色補充權限（合規模組唯讀）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'QAU' AND p.code = ANY(ARRAY[
    'dms.document.view',
    'risk.register.view',
    'change.request.view',
    'env.monitoring.view',
    'competency.assessment.view',
    'study.report.view',
    'formulation.record.view',
    'glp.compliance.overview'
])
ON CONFLICT DO NOTHING;

-- ADMIN_STAFF 角色補充權限（管理系統完整存取）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'ADMIN_STAFF' AND p.code = ANY(ARRAY[
    'dms.document.view', 'dms.document.manage', 'dms.document.approve',
    'risk.register.view', 'risk.register.manage',
    'change.request.view', 'change.request.manage', 'change.request.approve',
    'env.monitoring.view', 'env.monitoring.manage',
    'competency.assessment.view', 'competency.assessment.manage',
    'glp.management_review.view', 'glp.management_review.manage'
])
ON CONFLICT DO NOTHING;

-- EXPERIMENT_STAFF 補充（環境監控登錄、配製紀錄）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'EXPERIMENT_STAFF' AND p.code = ANY(ARRAY[
    'env.monitoring.view', 'env.monitoring.manage',
    'formulation.record.view', 'formulation.record.manage',
    'competency.assessment.view'
])
ON CONFLICT DO NOTHING;

-- EQUIPMENT_MAINTENANCE 補充（變更控制）
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.code = 'EQUIPMENT_MAINTENANCE' AND p.code = ANY(ARRAY[
    'change.request.view', 'change.request.manage',
    'env.monitoring.view'
])
ON CONFLICT DO NOTHING;

-- ============================================================================
-- P0-3: 校正計量追溯鏈 — Reference Standards + Calibration 擴充
-- ============================================================================

CREATE TABLE IF NOT EXISTS reference_standards (
    id                              UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name                            VARCHAR(200)    NOT NULL,
    serial_number                   VARCHAR(100),
    standard_type                   VARCHAR(50)     NOT NULL DEFAULT 'working',
    traceable_to                    VARCHAR(500),
    national_standard_number        VARCHAR(200),
    calibration_lab                 VARCHAR(200),
    calibration_lab_accreditation   VARCHAR(100),
    last_calibrated_at              DATE,
    next_due_at                     DATE,
    certificate_number              VARCHAR(100),
    measurement_uncertainty         VARCHAR(100),
    status                          VARCHAR(20)     NOT NULL DEFAULT 'active',
    notes                           TEXT,
    created_by                      UUID            REFERENCES users(id),
    created_at                      TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at                      TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_reference_standards_status ON reference_standards(status);
CREATE INDEX IF NOT EXISTS idx_reference_standards_type ON reference_standards(standard_type);

-- 擴充 equipment_calibrations
ALTER TABLE equipment_calibrations
    ADD COLUMN IF NOT EXISTS reference_standard_id      UUID REFERENCES reference_standards(id),
    ADD COLUMN IF NOT EXISTS calibration_lab_name        VARCHAR(200),
    ADD COLUMN IF NOT EXISTS calibration_lab_accreditation VARCHAR(100),
    ADD COLUMN IF NOT EXISTS traceability_statement      TEXT,
    ADD COLUMN IF NOT EXISTS reading_before              VARCHAR(100),
    ADD COLUMN IF NOT EXISTS reading_after               VARCHAR(100);

CREATE INDEX IF NOT EXISTS idx_calibrations_ref_standard ON equipment_calibrations(reference_standard_id);

-- ============================================================================
-- P1-1: 通用文件控制系統 (DMS)
-- ============================================================================

CREATE TABLE IF NOT EXISTS controlled_documents (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_number          VARCHAR(50)     NOT NULL UNIQUE,
    title               VARCHAR(300)    NOT NULL,
    doc_type            VARCHAR(50)     NOT NULL,
    category            VARCHAR(100),
    current_version     INTEGER         NOT NULL DEFAULT 1,
    status              VARCHAR(30)     NOT NULL DEFAULT 'draft',
    effective_date      DATE,
    review_due_date     DATE,
    owner_id            UUID            REFERENCES users(id),
    approved_by         UUID            REFERENCES users(id),
    approved_at         TIMESTAMPTZ,
    obsoleted_at        TIMESTAMPTZ,
    retention_years     INTEGER,
    file_path           TEXT,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ctrl_docs_status ON controlled_documents(status);
CREATE INDEX IF NOT EXISTS idx_ctrl_docs_type ON controlled_documents(doc_type);
CREATE INDEX IF NOT EXISTS idx_ctrl_docs_owner ON controlled_documents(owner_id);

CREATE TABLE IF NOT EXISTS document_revisions (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id         UUID            NOT NULL REFERENCES controlled_documents(id) ON DELETE CASCADE,
    version             INTEGER         NOT NULL,
    change_summary      TEXT            NOT NULL,
    revised_by          UUID            REFERENCES users(id),
    reviewed_by         UUID            REFERENCES users(id),
    approved_by         UUID            REFERENCES users(id),
    approved_at         TIMESTAMPTZ,
    file_path           TEXT,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_doc_revisions_doc ON document_revisions(document_id);

CREATE TABLE IF NOT EXISTS document_acknowledgments (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id             UUID        NOT NULL REFERENCES controlled_documents(id) ON DELETE CASCADE,
    user_id                 UUID        NOT NULL REFERENCES users(id),
    acknowledged_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version_acknowledged    INTEGER     NOT NULL,
    UNIQUE (document_id, user_id, version_acknowledged)
);

CREATE INDEX IF NOT EXISTS idx_doc_acks_doc ON document_acknowledgments(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_acks_user ON document_acknowledgments(user_id);

-- ============================================================================
-- P1-2: 管理審查模組
-- ============================================================================

CREATE TABLE IF NOT EXISTS management_reviews (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    review_number       VARCHAR(50)     NOT NULL UNIQUE,
    title               VARCHAR(300)    NOT NULL,
    review_date         DATE            NOT NULL,
    status              VARCHAR(30)     NOT NULL DEFAULT 'planned',
    agenda              TEXT,
    attendees           JSONB,
    minutes             TEXT,
    decisions           JSONB,
    action_items        JSONB,
    chaired_by          UUID            REFERENCES users(id),
    approved_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mgmt_reviews_status ON management_reviews(status);
CREATE INDEX IF NOT EXISTS idx_mgmt_reviews_date ON management_reviews(review_date);

-- ============================================================================
-- P1-3: 風險管理模組
-- ============================================================================

CREATE TABLE IF NOT EXISTS risk_register (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    risk_number         VARCHAR(50)     NOT NULL UNIQUE,
    title               VARCHAR(300)    NOT NULL,
    description         TEXT,
    category            VARCHAR(50),
    source              VARCHAR(50),
    severity            INTEGER         NOT NULL CHECK (severity BETWEEN 1 AND 5),
    likelihood          INTEGER         NOT NULL CHECK (likelihood BETWEEN 1 AND 5),
    detectability       INTEGER         CHECK (detectability BETWEEN 1 AND 5),
    risk_score          INTEGER         GENERATED ALWAYS AS (severity * likelihood) STORED,
    status              VARCHAR(30)     NOT NULL DEFAULT 'identified',
    mitigation_plan     TEXT,
    residual_risk_score INTEGER,
    owner_id            UUID            REFERENCES users(id),
    review_date         DATE,
    related_nc_id       UUID,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_risk_register_status ON risk_register(status);
CREATE INDEX IF NOT EXISTS idx_risk_register_owner ON risk_register(owner_id);
CREATE INDEX IF NOT EXISTS idx_risk_register_score ON risk_register(risk_score);

-- ============================================================================
-- P1-4: 通用變更控制
-- ============================================================================

CREATE TABLE IF NOT EXISTS change_requests (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    change_number       VARCHAR(50)     NOT NULL UNIQUE,
    title               VARCHAR(300)    NOT NULL,
    change_type         VARCHAR(50)     NOT NULL,
    description         TEXT            NOT NULL,
    justification       TEXT,
    impact_assessment   TEXT,
    status              VARCHAR(30)     NOT NULL DEFAULT 'draft',
    requested_by        UUID            NOT NULL REFERENCES users(id),
    reviewed_by         UUID            REFERENCES users(id),
    reviewed_at         TIMESTAMPTZ,
    approved_by         UUID            REFERENCES users(id),
    approved_at         TIMESTAMPTZ,
    implemented_at      TIMESTAMPTZ,
    verified_by         UUID            REFERENCES users(id),
    verified_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_change_requests_status ON change_requests(status);
CREATE INDEX IF NOT EXISTS idx_change_requests_type ON change_requests(change_type);
CREATE INDEX IF NOT EXISTS idx_change_requests_requester ON change_requests(requested_by);

-- ============================================================================
-- P1-5: SOP 審核簽署強化
-- ============================================================================

ALTER TABLE qa_sop_documents
    ADD COLUMN IF NOT EXISTS reviewed_by        UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS reviewed_at         TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS approved_by         UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS approved_at         TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS review_due_date     DATE,
    ADD COLUMN IF NOT EXISTS revision_history    JSONB;

-- ============================================================================
-- P1-6: 環境監控模組
-- ============================================================================

CREATE TABLE IF NOT EXISTS environment_monitoring_points (
    id                      UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    name                    VARCHAR(200)    NOT NULL,
    location_type           VARCHAR(50)     NOT NULL,
    building_id             UUID            REFERENCES buildings(id),
    zone_id                 UUID            REFERENCES zones(id),
    parameters              JSONB           NOT NULL,
    monitoring_interval     VARCHAR(30),
    is_active               BOOLEAN         NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_env_points_active ON environment_monitoring_points(is_active);
CREATE INDEX IF NOT EXISTS idx_env_points_building ON environment_monitoring_points(building_id);

CREATE TABLE IF NOT EXISTS environment_readings (
    id                      UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    monitoring_point_id     UUID            NOT NULL REFERENCES environment_monitoring_points(id) ON DELETE CASCADE,
    reading_time            TIMESTAMPTZ     NOT NULL,
    readings                JSONB           NOT NULL,
    is_out_of_range         BOOLEAN         NOT NULL DEFAULT false,
    out_of_range_params     JSONB,
    recorded_by             UUID            REFERENCES users(id),
    source                  VARCHAR(20)     NOT NULL DEFAULT 'manual',
    notes                   TEXT,
    created_at              TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_env_readings_point ON environment_readings(monitoring_point_id);
CREATE INDEX IF NOT EXISTS idx_env_readings_time ON environment_readings(reading_time);
CREATE INDEX IF NOT EXISTS idx_env_readings_oor ON environment_readings(is_out_of_range) WHERE is_out_of_range = true;

-- ============================================================================
-- P1-7: 能力評鑑框架
-- ============================================================================

CREATE TABLE IF NOT EXISTS competency_assessments (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID            NOT NULL REFERENCES users(id),
    assessment_type     VARCHAR(50)     NOT NULL,
    skill_area          VARCHAR(200)    NOT NULL,
    assessment_date     DATE            NOT NULL,
    assessor_id         UUID            NOT NULL REFERENCES users(id),
    result              VARCHAR(30)     NOT NULL,
    score               NUMERIC(5,2),
    method              VARCHAR(100),
    valid_until         DATE,
    notes               TEXT,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_competency_user ON competency_assessments(user_id);
CREATE INDEX IF NOT EXISTS idx_competency_assessor ON competency_assessments(assessor_id);
CREATE INDEX IF NOT EXISTS idx_competency_valid ON competency_assessments(valid_until);

CREATE TABLE IF NOT EXISTS role_training_requirements (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    role_code           VARCHAR(50)     NOT NULL,
    training_topic      VARCHAR(200)    NOT NULL,
    is_mandatory        BOOLEAN         NOT NULL DEFAULT true,
    recurrence_months   INTEGER,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    UNIQUE (role_code, training_topic)
);

-- ============================================================================
-- P1-8: 最終報告模組
-- ============================================================================

CREATE TABLE IF NOT EXISTS study_final_reports (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    report_number       VARCHAR(50)     NOT NULL UNIQUE,
    protocol_id         UUID            NOT NULL REFERENCES protocols(id),
    title               VARCHAR(500)    NOT NULL,
    status              VARCHAR(30)     NOT NULL DEFAULT 'draft',
    summary             TEXT,
    methods             TEXT,
    results             TEXT,
    conclusions         TEXT,
    deviations          TEXT,
    signed_by           UUID            REFERENCES users(id),
    signed_at           TIMESTAMPTZ,
    signature_id        UUID            REFERENCES electronic_signatures(id),
    qau_statement       TEXT,
    qau_signed_by       UUID            REFERENCES users(id),
    qau_signed_at       TIMESTAMPTZ,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_final_reports_protocol ON study_final_reports(protocol_id);
CREATE INDEX IF NOT EXISTS idx_final_reports_status ON study_final_reports(status);

-- ============================================================================
-- P1-9: 試驗物質管理強化
-- ============================================================================

ALTER TABLE products
    ADD COLUMN IF NOT EXISTS glp_characterization   TEXT,
    ADD COLUMN IF NOT EXISTS stability_data          JSONB,
    ADD COLUMN IF NOT EXISTS storage_conditions      VARCHAR(200),
    ADD COLUMN IF NOT EXISTS cas_number              VARCHAR(50),
    ADD COLUMN IF NOT EXISTS is_test_article         BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_control_article      BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS formulation_records (
    id                  UUID            PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id          UUID            NOT NULL REFERENCES products(id),
    protocol_id         UUID            REFERENCES protocols(id),
    formulation_date    DATE            NOT NULL,
    batch_number        VARCHAR(100),
    concentration       VARCHAR(100),
    volume              VARCHAR(100),
    prepared_by         UUID            NOT NULL REFERENCES users(id),
    verified_by         UUID            REFERENCES users(id),
    verified_at         TIMESTAMPTZ,
    expiry_date         DATE,
    notes               TEXT,
    created_at          TIMESTAMPTZ     NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_formulation_product ON formulation_records(product_id);
CREATE INDEX IF NOT EXISTS idx_formulation_protocol ON formulation_records(protocol_id);
CREATE INDEX IF NOT EXISTS idx_formulation_date ON formulation_records(formulation_date);
