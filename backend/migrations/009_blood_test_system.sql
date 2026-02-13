-- ============================================
-- Migration 009: Blood Test System（血液檢查系統）
-- 
-- 合併自原 024+025+026+027：
-- - 血液檢查項目模板主檔 + seed
-- - 血液檢查主表（預設 completed，無 pending 狀態）
-- - 血液檢查項目明細表
-- - 檢驗組合表 + seed
-- - 組合項目關聯表 + seed
-- 
-- 注意：Enum 擴展（pig_record_type/version_record_type）已回併至 001
--       documents.iacuc_no 已回併至 007
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 血液檢查項目模板主檔
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
-- 2. 模板 Seed 資料（豬博士動物科技生化檢驗項目表）
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
-- 3. 血液檢查主表（status 預設 completed，無需審閱）
-- ============================================

CREATE TABLE pig_blood_tests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
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

CREATE INDEX idx_pig_blood_tests_pig_id ON pig_blood_tests(pig_id);
CREATE INDEX idx_pig_blood_tests_test_date ON pig_blood_tests(test_date);
CREATE INDEX idx_pig_blood_tests_is_deleted ON pig_blood_tests(is_deleted);

-- ============================================
-- 4. 血液檢查項目明細表
-- ============================================

CREATE TABLE pig_blood_test_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blood_test_id UUID NOT NULL REFERENCES pig_blood_tests(id) ON DELETE CASCADE,
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

CREATE INDEX idx_pig_blood_test_items_blood_test_id ON pig_blood_test_items(blood_test_id);
CREATE INDEX idx_pig_blood_test_items_template_id ON pig_blood_test_items(template_id);

-- ============================================
-- 5. 檢驗組合主表
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
-- 6. 組合與模板項目的多對多關聯
-- ============================================

CREATE TABLE IF NOT EXISTS blood_test_panel_items (
    panel_id UUID NOT NULL REFERENCES blood_test_panels(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES blood_test_templates(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (panel_id, template_id)
);

CREATE INDEX IF NOT EXISTS idx_blood_test_panel_items_panel ON blood_test_panel_items(panel_id);

-- ============================================
-- 7. 組合 Seed 資料
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
-- 8. 組合項目關聯 Seed 資料
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
-- 完成
-- ============================================
