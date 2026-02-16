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
-- - 血液檢查系統
-- - 猝死記錄表
-- - 動物轉讓系統
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
    -- 額外動物欄位
    animal_no VARCHAR(50),
    deletion_reason TEXT,
    animal_id UUID,
    breed_other VARCHAR(100),
    -- 實驗分配者
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
    -- 軟刪除
    deleted_at TIMESTAMPTZ,
    deletion_reason TEXT,
    deleted_by UUID REFERENCES users(id),
    -- 緊急給藥
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
    -- 軟刪除
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
    -- 軟刪除
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
    -- 軟刪除
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
-- 16. 動物匯入批次表
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
-- 17. 變更原因記錄表
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
-- 18. 觀察/手術獸醫已讀記錄表
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
-- 19. 血液檢查項目模板主檔
-- ============================================

CREATE TABLE blood_test_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    default_unit VARCHAR(50),
    reference_range VARCHAR(100),
    default_price NUMERIC(10, 2) DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_blood_test_templates_code ON blood_test_templates(code);
CREATE INDEX idx_blood_test_templates_is_active ON blood_test_templates(is_active);

-- ============================================
-- 20. 血液檢查模板 Seed 資料
-- ============================================

INSERT INTO blood_test_templates (code, name, default_unit, default_price, sort_order) VALUES
-- 肝 (Liver)
('T-BIL',  'T. Bilirubin (總膽紅素)',      'mg/dL',   50,   101),
('D-BIL',  'D. Bilirubin (直接膽紅素)',     'mg/dL',   50,   102),
('AST',    'AST (GOT)',                     'U/L',     50,   103),
('ALT',    'ALT (GPT)',                     'U/L',     50,   104),
('ALP',    'ALP (鹼性磷酸酶)',              'U/L',     50,   105),
('GGT',    'γ-GT',                          'U/L',    120,   106),
('TP',     'Total Protein (總蛋白)',        'g/dL',    50,   107),
('ALB',    'ALB (白蛋白)',                  'g/dL',    50,   108),
('GLO',    'GLO (球蛋白)',                  'g/dL',    50,   109),
('AG',     'A/G (白球比)',                  NULL,       0,   110),
-- 血脂 (Lipids)
('CHO',    'CHO (膽固醇)',                  'mg/dL',   50,   201),
('TG',     'TG (三酸甘油酯)',              'mg/dL',   50,   202),
('HDL',    'HDL',                           'mg/dL',  200,   203),
('LDL',    'LDL',                           'mg/dL',  200,   204),
-- 心臟 (Heart)
('CK',     'CK (肌酸激酶)',                'U/L',    140,   301),
('LDH',    'LDH (乳酸脫氫酶)',             'U/L',    150,   302),
('HSCRP',  'hs-CRP (高敏C反應蛋白)',       'mg/L',   400,   303),
('CKMB',   'CK-MB',                        'U/L',    250,   304),
-- 胰臟 (Pancreas)
('AMY',    'Amylase (澱粉酶)',             'U/L',    175,   401),
('LPS',    'Lipase (脂肪酶)',              'U/L',    375,   402),
-- 醣/胰 (Sugar/Pancreas)
('GLU',    'Glucose (血糖)',               'mg/dL',   50,   501),
('HBA1C',  'HbA1c (糖化血色素)',           '%',      390,   502),
('INS',    'Insulin (胰島素)',             'μU/mL',  330,   503),
('CPEP',   'C-Peptide (C-胜肽)',           'ng/mL',  360,   504),
-- 腎 (Kidney)
('BUN',    'BUN (血尿素氮)',               'mg/dL',   50,   601),
('CRE',    'CRE (肌酐酸)',                 'mg/dL',   50,   602),
('UA',     'UA (尿酸)',                    'mg/dL',   50,   603),
-- 泌尿 (Urinary)
('UREA',   'Urea Nitrogen (尿素氮)',       'mg/dL',   50,   701),
('UTP',    'UTP (尿蛋白定量)',             'mg/day', 150,   702),
('MALB',   'Microalbumin (微白蛋白)',      'mg/L',   400,   703),
('URINE',  'Urine routine (尿液常規)',     NULL,     150,   704),
-- 血液/凝血 (Coagulation)
('PT',     'PT (凝血酶原時間)',            'sec',    315,   801),
('APTT',   'APTT (活化部分凝血時間)',      'sec',    390,   802),
('FIB',    'Fibrinogen (纖維蛋白原)',      'mg/dL',  400,   803),
('DDIM',   'D-Dimer (D-二聚體)',           'μg/mL',  400,   804),
('ESR',    'ESR (紅血球沉降速率)',         'mm/hr',  225,   805),
-- 電解質 (Electrolytes)
('NA',     'Na+ (鈉)',                     'mEq/L',   75,   901),
('K',      'K+ (鉀)',                      'mEq/L',   75,   902),
('CL',     'Cl- (氯)',                     'mEq/L',   75,   903),
('CA',     'Ca2+ (鈣)',                    'mg/dL',   75,   904),
('MG',     'Mg2+ (鎂)',                    'mg/dL',  125,   905),
('PHOS',   'P (磷)',                       'mg/dL',   75,   906),
-- 荷爾蒙 (Hormones)
('PRL',    'Prolactine (泌乳素)',          'ng/mL',  360,  1001),
('TESTO',  'Testosterone (睪固酮)',        'ng/dL',  330,  1002),
('FSH',    'FSH (濾泡刺激素)',             'mIU/mL', 300,  1003),
('LH',     'LH (黃體生成素)',              'mIU/mL', 300,  1004),
('PROG',   'Progesterone (黃體素)',        'ng/mL',  330,  1005),
('E2',     'E2 (雌二醇)',                  'pg/mL',  330,  1006),
('AE3',    'uE3',                          NULL,     690,  1007),
-- 其他 (Others)
('LAC',    'Lactate (乳酸)',               'mmol/L', 500,  1101),
('CTX',    'CTx (骨膠原交聯)',             'ng/mL',  940,  1102),
('HBANAL', 'Hb analysis (血色素分析)',     NULL,     400,  1103),
-- 感染 (Infection)
('AFB',    '抗酸菌',                       NULL,       0,  1201),
('IGRA',   'IGRA (干擾素測試)',            NULL,    4320,  1202),
('CULT',   '培養 & 鑑定',                  NULL,     980,  1203),
-- CBC 基礎
('WBC',    'WBC (白血球計數)',             '10³/μL',   0,     2),
('RBC',    'RBC (紅血球計數)',             '10⁶/μL',   0,     3),
('HGB',    'HGB (血紅素)',                 'g/dL',     0,     4),
('HCT',    'HCT (血容比)',                 '%',        0,     5),
('PLT',    'PLT (血小板計數)',             '10³/μL',   0,     6),
-- 採血管 (Blood Collection Tubes)
('EDTA',   'EDTA 採血管',                  '支',       0,  1301),
('HEP',    'Heparin 採血管',               '支',       0,  1302),
('SST',    'SST 採血管',                   '支',       0,  1303),
('SCIT',   'Sodium Citrate 採血管',        '支',       0,  1304)
ON CONFLICT (code) DO UPDATE SET
  name = EXCLUDED.name,
  default_unit = EXCLUDED.default_unit,
  default_price = EXCLUDED.default_price,
  sort_order = EXCLUDED.sort_order;

-- ============================================
-- 21. 血液檢查主表
-- ============================================

CREATE TABLE animal_blood_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
    test_date DATE NOT NULL,
    lab_name VARCHAR(200),
    status VARCHAR(20) NOT NULL DEFAULT 'completed',
    remark TEXT,
    vet_read BOOLEAN NOT NULL DEFAULT false,
    vet_read_at TIMESTAMPTZ,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id),
    delete_reason TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_blood_test_status CHECK (status IN ('pending', 'completed'))
);

CREATE INDEX idx_animal_blood_tests_animal_id ON animal_blood_tests(animal_id);
CREATE INDEX idx_animal_blood_tests_test_date ON animal_blood_tests(test_date);
CREATE INDEX idx_animal_blood_tests_is_deleted ON animal_blood_tests(is_deleted);

-- ============================================
-- 22. 血液檢查項目明細表
-- ============================================

CREATE TABLE animal_blood_test_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blood_test_id UUID NOT NULL REFERENCES animal_blood_tests(id) ON DELETE CASCADE,
    template_id UUID REFERENCES blood_test_templates(id),
    item_name VARCHAR(200) NOT NULL,
    result_value VARCHAR(100),
    result_unit VARCHAR(50),
    reference_range VARCHAR(100),
    is_abnormal BOOLEAN NOT NULL DEFAULT false,
    remark TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_animal_blood_test_items_blood_test_id ON animal_blood_test_items(blood_test_id);
CREATE INDEX idx_animal_blood_test_items_template_id ON animal_blood_test_items(template_id);

-- ============================================
-- 23. 檢驗組合主表
-- ============================================

CREATE TABLE IF NOT EXISTS blood_test_panels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(30) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(100) DEFAULT '📋',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_blood_test_panels_is_active ON blood_test_panels(is_active);
CREATE INDEX IF NOT EXISTS idx_blood_test_panels_sort_order ON blood_test_panels(sort_order);

-- ============================================
-- 24. 組合與模板項目的多對多關聯
-- ============================================

CREATE TABLE IF NOT EXISTS blood_test_panel_items (
    panel_id UUID NOT NULL REFERENCES blood_test_panels(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES blood_test_templates(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (panel_id, template_id)
);

CREATE INDEX IF NOT EXISTS idx_blood_test_panel_items_panel ON blood_test_panel_items(panel_id);

-- ============================================
-- 25. 檢驗組合 Seed 資料
-- ============================================

INSERT INTO blood_test_panels (key, name, icon, sort_order) VALUES
('TUBE',     '採血管',        '🧪', 1),
('CBC',      'CBC',  '🩸', 2),
('LIVER',    '肝臟',        '/icons/liver.svg', 3),
('LIPID',    '血脂',          '🥩', 4),
('HEART',    '心臟',          '🫀', 5),
('SUGAR',    '胰臟與血糖',       '🍬', 6),
('KIDNEY',   '腎臟',        '/icons/renal.svg', 7),
('URINARY',  '泌尿',          '🚽', 8),
('COAG',     '凝血',          '🩸', 9),
('ELECTRO',  '電解質',        '⚡', 10),
('HORMONE',  '荷爾蒙',        '💉', 11),
('INFECT',   '感染',          '🦠', 12),
('OTHER',    '其他',          '📋', 13)
ON CONFLICT (key) DO UPDATE SET
    name = EXCLUDED.name,
    icon = EXCLUDED.icon,
    sort_order = EXCLUDED.sort_order;

-- ============================================
-- 26. 組合項目關聯 Seed 資料
-- ============================================

INSERT INTO blood_test_panel_items (panel_id, template_id, sort_order)
SELECT p.id, t.id, t.sort_order
FROM blood_test_panels p
CROSS JOIN blood_test_templates t
WHERE (p.key, t.code) IN (
    -- 採血管
    ('TUBE', 'EDTA'), ('TUBE', 'HEP'), ('TUBE', 'SST'), ('TUBE', 'SCIT'),
    -- CBC 血球計數
    ('CBC', 'WBC'), ('CBC', 'RBC'), ('CBC', 'HGB'), ('CBC', 'HCT'), ('CBC', 'PLT'),
    -- 肝功能
    ('LIVER', 'T-BIL'), ('LIVER', 'D-BIL'), ('LIVER', 'AST'), ('LIVER', 'ALT'),
    ('LIVER', 'ALP'), ('LIVER', 'GGT'), ('LIVER', 'TP'), ('LIVER', 'ALB'),
    ('LIVER', 'GLO'), ('LIVER', 'AG'),
    -- 血脂
    ('LIPID', 'CHO'), ('LIPID', 'TG'), ('LIPID', 'HDL'), ('LIPID', 'LDL'),
    -- 心臟
    ('HEART', 'CK'), ('HEART', 'LDH'), ('HEART', 'HSCRP'), ('HEART', 'CKMB'),
    -- 胰臟與血糖
    ('PANCREAS', 'AMY'), ('PANCREAS', 'LPS'), ('SUGAR', 'GLU'), ('SUGAR', 'HBA1C'), ('SUGAR', 'INS'), ('SUGAR', 'CPEP'),
    -- 腎功能
    ('KIDNEY', 'BUN'), ('KIDNEY', 'CRE'), ('KIDNEY', 'UA'),
    -- 泌尿
    ('URINARY', 'UREA'), ('URINARY', 'UTP'), ('URINARY', 'MALB'), ('URINARY', 'URINE'),
    -- 凝血
    ('COAG', 'PT'), ('COAG', 'APTT'), ('COAG', 'FIB'), ('COAG', 'DDIM'), ('COAG', 'ESR'),
    -- 電解質
    ('ELECTRO', 'NA'), ('ELECTRO', 'K'), ('ELECTRO', 'CL'),
    ('ELECTRO', 'CA'), ('ELECTRO', 'MG'), ('ELECTRO', 'PHOS'),
    -- 荷爾蒙
    ('HORMONE', 'PRL'), ('HORMONE', 'TESTO'), ('HORMONE', 'FSH'),
    ('HORMONE', 'LH'), ('HORMONE', 'PROG'), ('HORMONE', 'E2'), ('HORMONE', 'AE3'),
    -- 感染
    ('INFECT', 'AFB'), ('INFECT', 'IGRA'), ('INFECT', 'CULT'),
    -- 其他
    ('OTHER', 'LAC'), ('OTHER', 'CTX'), ('OTHER', 'HBANAL')
)
ON CONFLICT (panel_id, template_id) DO NOTHING;

-- ============================================
-- 27. 猝死記錄表
-- ============================================

CREATE TABLE animal_sudden_deaths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id) UNIQUE,
    discovered_at TIMESTAMPTZ NOT NULL,
    discovered_by UUID NOT NULL REFERENCES users(id),
    probable_cause TEXT,
    iacuc_no VARCHAR(20),
    location VARCHAR(100),
    remark TEXT,
    requires_pathology BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sudden_deaths_animal ON animal_sudden_deaths(animal_id);
CREATE INDEX idx_sudden_deaths_discovered_by ON animal_sudden_deaths(discovered_by);

-- ============================================
-- 28. 動物轉讓記錄表
-- ============================================

CREATE TABLE animal_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    animal_id UUID NOT NULL REFERENCES animals(id),
    from_iacuc_no VARCHAR(20) NOT NULL,
    to_iacuc_no VARCHAR(20),
    status animal_transfer_status NOT NULL DEFAULT 'pending',
    initiated_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    remark TEXT,
    rejected_by UUID REFERENCES users(id),
    rejected_reason TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 29. 轉讓獸醫評估表
-- ============================================

CREATE TABLE transfer_vet_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transfer_id UUID NOT NULL REFERENCES animal_transfers(id) UNIQUE,
    vet_id UUID NOT NULL REFERENCES users(id),
    health_status TEXT NOT NULL,
    is_fit_for_transfer BOOLEAN NOT NULL,
    conditions TEXT,
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transfers_animal ON animal_transfers(animal_id);
CREATE INDEX idx_transfers_from_iacuc ON animal_transfers(from_iacuc_no);
CREATE INDEX idx_transfers_to_iacuc ON animal_transfers(to_iacuc_no);
CREATE INDEX idx_transfers_status ON animal_transfers(status);
CREATE INDEX idx_transfer_vet_eval ON transfer_vet_evaluations(transfer_id);

-- ============================================
-- 完成
-- ============================================
