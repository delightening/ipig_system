-- 血液檢查常用組合（分析頁一鍵選取用）
-- 與 blood_test_panels 不同：panels 為分類、presets 為可組合多個 panel 的快捷鍵

CREATE TABLE blood_test_presets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(100) DEFAULT '📋',
    panel_keys TEXT[] NOT NULL DEFAULT '{}',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_blood_test_presets_is_active ON blood_test_presets(is_active);
CREATE INDEX idx_blood_test_presets_sort_order ON blood_test_presets(sort_order);

-- 預設常用組合
INSERT INTO blood_test_presets (name, icon, panel_keys, sort_order) VALUES
('肝腎功能', '🩺', ARRAY['LIVER','KIDNEY'], 1),
('血球分析', '🩸', ARRAY['CBC'], 2),
('發炎指標', '🦠', ARRAY['INFECT'], 3),
('肝臟', '/icons/liver.svg', ARRAY['LIVER'], 4),
('血脂', '🥩', ARRAY['LIPID'], 5),
('電解質', '⚡', ARRAY['ELECTRO'], 6);
