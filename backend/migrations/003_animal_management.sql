-- ============================================
-- Migration 002: Animal Management System
-- 
-- 包含：
-- - 動物來源表
-- - 動物主表 (UUID 主鍵)
-- - 各類紀錄表 (觀察、手術、體重、疫苗、犧牲、病理)
-- - 獸醫建議與照護紀錄
-- - 匯入匯出表
-- - 緊急安樂死審查表
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 動物來源表
-- ============================================

CREATE TABLE pig_sources (
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

-- 插入預設動物來源
INSERT INTO pig_sources (id, code, name, sort_order) VALUES
    (gen_random_uuid(), 'TAITUNG', '台東種畜繁殖場', 1),
    (gen_random_uuid(), 'QINGXIN', '青欣牧場', 2),
    (gen_random_uuid(), 'PIGMODEL', '豬博士畜牧場', 3),
    (gen_random_uuid(), 'PINGSHUN', '平順牧場', 4)
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 2. 動物主表 (UUID 主鍵)
-- ============================================

CREATE TABLE pigs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ear_tag VARCHAR(10) NOT NULL,
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
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id),
    -- GLP 合規欄位
    lab_animal_id VARCHAR(50),
    glp_study_no VARCHAR(50),
    randomization_group VARCHAR(50),
    dosing_group VARCHAR(50),
    quarantine_end_date DATE,
    -- 獸醫師查看時間戳
    vet_weight_viewed_at TIMESTAMPTZ,
    vet_vaccine_viewed_at TIMESTAMPTZ,
    vet_sacrifice_viewed_at TIMESTAMPTZ,
    vet_last_viewed_at TIMESTAMPTZ,
    -- 建立者與時間
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pigs_ear_tag ON pigs(ear_tag);
CREATE INDEX idx_pigs_status ON pigs(status);
CREATE INDEX idx_pigs_iacuc_no ON pigs(iacuc_no);
CREATE INDEX idx_pigs_pen_location ON pigs(pen_location);
CREATE INDEX idx_pigs_is_deleted ON pigs(is_deleted);
CREATE INDEX idx_pigs_glp_study_no ON pigs(glp_study_no);

-- ============================================
-- 3. 觀察試驗紀錄表
-- ============================================

CREATE TABLE pig_observations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    event_date DATE NOT NULL,
    record_type record_type NOT NULL,
    equipment_used JSONB,
    anesthesia_start TIMESTAMPTZ,
    anesthesia_end TIMESTAMPTZ,
    content TEXT NOT NULL,
    no_medication_needed BOOLEAN NOT NULL DEFAULT false,
    stop_medication BOOLEAN NOT NULL DEFAULT false,
    treatments JSONB,
    remark TEXT,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_observations_pig_id ON pig_observations(pig_id);
CREATE INDEX idx_pig_observations_event_date ON pig_observations(event_date);

-- ============================================
-- 4. 手術紀錄表
-- ============================================

CREATE TABLE pig_surgeries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
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

CREATE INDEX idx_pig_surgeries_pig_id ON pig_surgeries(pig_id);
CREATE INDEX idx_pig_surgeries_surgery_date ON pig_surgeries(surgery_date);

-- ============================================
-- 5. 體重紀錄表
-- ============================================

CREATE TABLE pig_weights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    measure_date DATE NOT NULL,
    weight NUMERIC(5, 1) NOT NULL,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_weights_pig_id ON pig_weights(pig_id);
CREATE INDEX idx_pig_weights_measure_date ON pig_weights(measure_date);

-- ============================================
-- 6. 疫苗/驅蟲紀錄表
-- ============================================

CREATE TABLE pig_vaccinations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    administered_date DATE NOT NULL,
    vaccine VARCHAR(100),
    deworming_dose VARCHAR(100),
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_vaccinations_pig_id ON pig_vaccinations(pig_id);

-- ============================================
-- 7. 犧牲/採樣紀錄表
-- ============================================

CREATE TABLE pig_sacrifices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE UNIQUE,
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

CREATE INDEX idx_pig_sacrifices_pig_id ON pig_sacrifices(pig_id);

-- ============================================
-- 8. 病理組織報告表
-- ============================================

CREATE TABLE pig_pathology_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE UNIQUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 9. 紀錄附件通用表
-- ============================================

CREATE TABLE pig_record_attachments (
    id UUID PRIMARY KEY,
    record_type pig_record_type NOT NULL,
    record_id UUID NOT NULL,
    file_type pig_file_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pig_record_attachments_record ON pig_record_attachments(record_type, record_id);

-- ============================================
-- 10. 獸醫師建議表
-- ============================================

CREATE TABLE vet_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type vet_record_type NOT NULL,
    record_id UUID NOT NULL,
    content TEXT NOT NULL,
    urgency VARCHAR(20) DEFAULT 'normal',
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_urgency CHECK (urgency IN ('normal', 'urgent', 'critical'))
);

CREATE INDEX idx_vet_recommendations_record ON vet_recommendations(record_type, record_id);

-- ============================================
-- 11. 照護給藥紀錄表
-- ============================================

CREATE TABLE care_medication_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type vet_record_type NOT NULL,
    record_id UUID NOT NULL,
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

CREATE INDEX idx_care_medication_records_record ON care_medication_records(record_type, record_id);

-- ============================================
-- 12. 紀錄版本歷史表
-- ============================================

CREATE TABLE record_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type version_record_type NOT NULL,
    record_id UUID NOT NULL,
    version_no INTEGER NOT NULL,
    snapshot JSONB NOT NULL,
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_record_versions_record ON record_versions(record_type, record_id);

-- ============================================
-- 13. 匯入作業表
-- ============================================

CREATE TABLE import_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_type import_type NOT NULL,
    status import_status NOT NULL DEFAULT 'pending',
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    total_rows INTEGER,
    processed_rows INTEGER DEFAULT 0,
    success_rows INTEGER DEFAULT 0,
    error_rows INTEGER DEFAULT 0,
    error_details JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_import_jobs_status ON import_jobs(status);
CREATE INDEX idx_import_jobs_created_by ON import_jobs(created_by);

-- ============================================
-- 14. 匯出作業表
-- ============================================

CREATE TABLE export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    export_type export_type NOT NULL,
    export_format export_format NOT NULL,
    status import_status NOT NULL DEFAULT 'pending',
    file_name VARCHAR(255),
    file_path VARCHAR(500),
    parameters JSONB,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_export_jobs_status ON export_jobs(status);
CREATE INDEX idx_export_jobs_created_by ON export_jobs(created_by);

-- ============================================
-- 15. 緊急安樂死審查表
-- ============================================

CREATE TABLE euthanasia_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    status euthanasia_order_status NOT NULL DEFAULT 'pending_review',
    reason TEXT NOT NULL,
    recommended_by UUID NOT NULL REFERENCES users(id),
    recommended_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    review_notes TEXT,
    executed_at TIMESTAMPTZ,
    executed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_euthanasia_orders_pig_id ON euthanasia_orders(pig_id);
CREATE INDEX idx_euthanasia_orders_status ON euthanasia_orders(status);

-- ============================================
-- 完成
-- ============================================
