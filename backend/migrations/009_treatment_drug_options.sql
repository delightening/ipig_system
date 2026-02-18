-- ============================================
-- Migration 009: 治療方式藥物選項
-- 
-- 包含：
-- - 藥物選單選項表
-- - Seed data（常用藥物）
-- - 清除舊用藥 JSONB 資料（測試階段）
-- ============================================

-- ============================================
-- 1. 藥物選單選項表
-- ============================================

CREATE TABLE treatment_drug_options (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    display_name VARCHAR(200),
    default_dosage_unit VARCHAR(20),
    available_units TEXT[],
    default_dosage_value VARCHAR(50),
    erp_product_id UUID REFERENCES products(id),
    category VARCHAR(50),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_treatment_drug_options_active ON treatment_drug_options(is_active);
CREATE INDEX idx_treatment_drug_options_category ON treatment_drug_options(category);
CREATE INDEX idx_treatment_drug_options_erp ON treatment_drug_options(erp_product_id);

-- ============================================
-- 2. Seed data: 常用藥物
-- ============================================

INSERT INTO treatment_drug_options (name, display_name, default_dosage_unit, available_units, category, sort_order) VALUES
-- 麻醉類
('Atropine', 'Atropine（阿托品）', 'mg', ARRAY['mg', 'ml', 'mg/kg'], '麻醉', 10),
('Stroless', 'Stroless', 'mg', ARRAY['mg', 'ml', 'mg/kg'], '麻醉', 20),
('Zoletil-50', 'Zoletil-50（乙嗪錠）', 'mg', ARRAY['mg', 'ml', 'mg/kg'], '麻醉', 30),
('O2', 'O2（氧氣）', 'L/min', ARRAY['L/min', '%'], '麻醉', 40),
('N2O', 'N2O（氧化亞氮）', 'L/min', ARRAY['L/min', '%'], '麻醉', 50),
('Isoflurane', 'Isoflurane（異氟醚）', '%', ARRAY['%', 'ml'], '麻醉', 60),
-- 止痛類
('Meloxicam', 'Meloxicam（美洛昔康）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg'], '止痛', 100),
('Buprenorphine', 'Buprenorphine（丁丙諾啡）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg'], '止痛', 110),
('Carprofen', 'Carprofen（卡洛芬）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg'], '止痛', 120),
-- 抗生素
('Enrofloxacin', 'Enrofloxacin（恩諾沙星）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg', 'tab'], '抗生素', 200),
('Amoxicillin', 'Amoxicillin（阿莫西林）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg', 'cap', 'tab'], '抗生素', 210),
('Cefazolin', 'Cefazolin（頭孢唑啉）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg'], '抗生素', 220),
-- 鎮靜類
('Xylazine', 'Xylazine（甲苯噻嗪）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg'], '鎮靜', 300),
('Midazolam', 'Midazolam（咪達唑侖）', 'mg/kg', ARRAY['mg', 'ml', 'mg/kg'], '鎮靜', 310),
-- 其他
('優點軟膏', '優點軟膏（眼藥膏）', 'cm', ARRAY['cm', 'g', 'pcs'], '其他', 400);

-- ============================================
-- 3. 清除舊用藥 JSONB 資料（測試階段）
-- ============================================

-- 清除觀察紀錄的 treatments
UPDATE animal_observations SET treatments = '[]'::jsonb WHERE treatments IS NOT NULL AND treatments != '[]'::jsonb;

-- 清除手術紀錄的藥物 JSONB
UPDATE animal_surgeries 
SET induction_anesthesia = NULL, 
    pre_surgery_medication = NULL, 
    anesthesia_maintenance = NULL,
    post_surgery_medication = NULL
WHERE induction_anesthesia IS NOT NULL 
   OR pre_surgery_medication IS NOT NULL 
   OR anesthesia_maintenance IS NOT NULL
   OR post_surgery_medication IS NOT NULL;
