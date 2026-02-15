-- ============================================
-- Migration 003: Animal Management System
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

CREATE TABLE animal_sources (
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
INSERT INTO animal_sources (id, code, name, sort_order) VALUES
    (gen_random_uuid(), 'TAITUNG', '台東種畜繁殖場', 1),
    (gen_random_uuid(), 'QINGXIN', '青欣牧場', 2),
    (gen_random_uuid(), 'PIGMODEL', '豬博士畜牧場', 3),
    (gen_random_uuid(), 'PINGSHUN', '平順牧場', 4)
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 2. 動物主表 (UUID 主鍵)
-- ============================================

CREATE TABLE animals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ear_tag VARCHAR(10) NOT NULL,
    status animal_status NOT NULL DEFAULT 'unassigned',
    breed animal_breed NOT NULL,
    source_id UUID REFERENCES animal_sources(id),
    gender animal_gender NOT NULL,
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
    -- 額外動物欄位（原 010）
    animal_no VARCHAR(50),
    deletion_reason TEXT,
    animal_id UUID,
    breed_other VARCHAR(100),
    -- 實驗分配者（原 023）
    experiment_assigned_by UUID REFERENCES users(id),
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

CREATE INDEX idx_animals_ear_tag ON animals(ear_tag);
CREATE INDEX idx_animals_status ON animals(status);
CREATE INDEX idx_animals_iacuc_no ON animals(iacuc_no);
CREATE INDEX idx_animals_pen_location ON animals(pen_location);
CREATE INDEX idx_animals_is_deleted ON animals(is_deleted);
CREATE INDEX idx_animals_glp_study_no ON animals(glp_study_no);

-- ============================================
-- 3. 觀察試驗紀錄表
-- ============================================

CREATE TABLE animal_observations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
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
    -- 軟刪除（原 020）
    deleted_at TIMESTAMPTZ,
    deletion_reason TEXT,
    deleted_by UUID REFERENCES users(id),
    -- 緊急給藥（原 020）
    is_emergency BOOLEAN DEFAULT false,
    emergency_status VARCHAR(20),
    emergency_reason TEXT,
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    -- 建立者與時間
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animal_observations_animal_id ON animal_observations(animal_id);
CREATE INDEX idx_animal_observations_event_date ON animal_observations(event_date);

-- ============================================
-- 4. 手術紀錄表
-- ============================================

CREATE TABLE animal_surgeries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
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
    -- 軟刪除（原 020）
    deleted_at TIMESTAMPTZ,
    deletion_reason TEXT,
    deleted_by UUID REFERENCES users(id),
    -- 建立者與時間
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animal_surgeries_animal_id ON animal_surgeries(animal_id);
CREATE INDEX idx_animal_surgeries_surgery_date ON animal_surgeries(surgery_date);

-- ============================================
-- 5. 體重紀錄表
-- ============================================

CREATE TABLE animal_weights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    measure_date DATE NOT NULL,
    weight NUMERIC(5, 1) NOT NULL,
    -- 軟刪除（原 020）
    deleted_at TIMESTAMPTZ,
    deletion_reason TEXT,
    deleted_by UUID REFERENCES users(id),
    -- 建立者與時間
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animal_weights_animal_id ON animal_weights(animal_id);
CREATE INDEX idx_animal_weights_measure_date ON animal_weights(measure_date);

-- ============================================
-- 6. 疫苗/驅蟲紀錄表
-- ============================================

CREATE TABLE animal_vaccinations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    administered_date DATE NOT NULL,
    vaccine VARCHAR(100),
    deworming_dose VARCHAR(100),
    -- 軟刪除（原 020）
    deleted_at TIMESTAMPTZ,
    deletion_reason TEXT,
    deleted_by UUID REFERENCES users(id),
    -- 建立者與時間
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animal_vaccinations_animal_id ON animal_vaccinations(animal_id);

-- ============================================
-- 7. 犧牲/採樣紀錄表
-- ============================================

CREATE TABLE animal_sacrifices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE UNIQUE,
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

CREATE INDEX idx_animal_sacrifices_animal_id ON animal_sacrifices(animal_id);

-- ============================================
-- 8. 病理組織報告表
-- ============================================

CREATE TABLE animal_pathology_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE UNIQUE,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 9. 紀錄附件通用表
-- ============================================

CREATE TABLE animal_record_attachments (
    id UUID PRIMARY KEY,
    record_type animal_record_type NOT NULL,
    record_id UUID NOT NULL,
    file_type animal_file_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animal_record_attachments_record ON animal_record_attachments(record_type, record_id);

-- ============================================
-- 10. 獸醫師建議表
-- ============================================

CREATE TABLE vet_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_type vet_record_type NOT NULL,
    record_id UUID NOT NULL,
    content TEXT NOT NULL,
    urgency VARCHAR(20) DEFAULT 'normal',
    -- 新增欄位（原 020）
    is_urgent BOOLEAN NOT NULL DEFAULT false,
    attachments JSONB,
    -- 建立者與時間
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
    diff_summary TEXT,
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
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    pi_user_id UUID NOT NULL REFERENCES users(id),
    status euthanasia_order_status NOT NULL DEFAULT 'pending_pi',
    reason TEXT NOT NULL,
    deadline_at TIMESTAMPTZ NOT NULL,
    pi_responded_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ,
    executed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE euthanasia_appeals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES euthanasia_orders(id) ON DELETE CASCADE,
    pi_user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    attachment_path VARCHAR(500),
    chair_user_id UUID REFERENCES users(id),
    chair_decision VARCHAR(50),
    chair_decided_at TIMESTAMPTZ,
    chair_deadline_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_euthanasia_orders_animal_id ON euthanasia_orders(animal_id);
CREATE INDEX idx_euthanasia_orders_status ON euthanasia_orders(status);
CREATE INDEX idx_euthanasia_appeals_order ON euthanasia_appeals(order_id);

-- ============================================
-- 16. 動物匯入批次表（原 009）
-- ============================================

CREATE TABLE IF NOT EXISTS animal_import_batches (
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

CREATE INDEX IF NOT EXISTS idx_animal_import_batches_status ON animal_import_batches(status);
CREATE INDEX IF NOT EXISTS idx_animal_import_batches_created_by ON animal_import_batches(created_by);
CREATE INDEX IF NOT EXISTS idx_animal_import_batches_created_at ON animal_import_batches(created_at DESC);

-- ============================================
-- 17. 變更原因記錄表（原 019）
-- ============================================

CREATE TABLE IF NOT EXISTS change_reasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    entity_id TEXT NOT NULL,
    change_type VARCHAR(20) NOT NULL,
    reason TEXT NOT NULL,
    old_values JSONB,
    new_values JSONB,
    changed_fields TEXT[],
    changed_by UUID REFERENCES users(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_change_reasons_entity ON change_reasons(entity_type, entity_id);

-- ============================================
-- 18. 觀察/手術獸醫已讀記錄表（原 019）
-- ============================================

CREATE TABLE IF NOT EXISTS observation_vet_reads (
    observation_id UUID NOT NULL,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (observation_id, vet_user_id)
);

CREATE TABLE IF NOT EXISTS surgery_vet_reads (
    surgery_id UUID NOT NULL,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (surgery_id, vet_user_id)
);

-- ============================================
-- 完成
-- ============================================
