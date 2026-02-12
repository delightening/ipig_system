-- ============================================
-- Migration 026: Blood Test Panels（檢驗組合）
--
-- 提供血液檢查的「組合快速勾選」功能：
-- - 組合主表（如 CBC、肝功能、腎功能...）
-- - 組合與模板項目的多對多關聯
-- - 預設 14 組組合 seed 資料
--
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 組合主表
-- ============================================

CREATE TABLE IF NOT EXISTS blood_test_panels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(30) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(10) DEFAULT '📋',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_blood_test_panels_is_active ON blood_test_panels(is_active);
CREATE INDEX IF NOT EXISTS idx_blood_test_panels_sort_order ON blood_test_panels(sort_order);

-- ============================================
-- 2. 組合與模板項目的多對多關聯
-- ============================================

CREATE TABLE IF NOT EXISTS blood_test_panel_items (
    panel_id UUID NOT NULL REFERENCES blood_test_panels(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES blood_test_templates(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (panel_id, template_id)
);

CREATE INDEX IF NOT EXISTS idx_blood_test_panel_items_panel ON blood_test_panel_items(panel_id);

-- ============================================
-- 3. Seed 預設組合（對應豬博士生化檢驗表單分類）
-- ============================================

INSERT INTO blood_test_panels (key, name, icon, sort_order) VALUES
('TUBE',     '採血管',        '🧪', 1),
('CBC',      'CBC 血球計數',  '🩸', 2),
('LIVER',    '肝功能',        '🫁', 3),
('LIPID',    '血脂',          '💧', 4),
('HEART',    '心臟',          '❤️', 5),
('PANCREAS', '胰臟',          '🔬', 6),
('SUGAR',    '血糖/胰',       '🍬', 7),
('KIDNEY',   '腎功能',        '🫘', 8),
('URINARY',  '泌尿',          '🚿', 9),
('COAG',     '凝血',          '🔴', 10),
('ELECTRO',  '電解質',        '⚡', 11),
('HORMONE',  '荷爾蒙',        '💉', 12),
('INFECT',   '感染',          '🦠', 13),
('OTHER',    '其他',          '📋', 14)
ON CONFLICT (key) DO UPDATE SET
    name = EXCLUDED.name,
    icon = EXCLUDED.icon,
    sort_order = EXCLUDED.sort_order;

-- ============================================
-- 4. Seed 組合項目關聯
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
    -- 胰臟
    ('PANCREAS', 'AMY'), ('PANCREAS', 'LPS'),
    -- 血糖/胰
    ('SUGAR', 'GLU'), ('SUGAR', 'HBA1C'), ('SUGAR', 'INS'), ('SUGAR', 'CPEP'),
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
