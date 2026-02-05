-- ============================================
-- Migration 002: ERP 基礎資料
-- 
-- 包含：
-- - SKU 類別與子類別
-- - 豬隻來源
-- - 倉庫
-- - 供應商
-- - 產品
-- ============================================

-- ============================================
-- 1. SKU 類別資料
-- ============================================

-- 插入 SKU 主類別
INSERT INTO sku_categories (code, name, sort_order) VALUES
    ('MED', '藥品', 1),
    ('MSP', '醫材', 2),
    ('FED', '飼料', 3),
    ('EQP', '器材', 4),
    ('CON', '耗材', 5),
    ('CHM', '化學品', 6),
    ('OTH', '其他', 7)
ON CONFLICT (code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 藥品
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('MED', 'ANE', '麻醉劑', 1),
    ('MED', 'ANT', '抗生素', 2),
    ('MED', 'VAC', '疫苗', 3),
    ('MED', 'PAI', '止痛劑', 4),
    ('MED', 'DEW', '驅蟲劑', 5),
    ('MED', 'OPH', '眼科藥', 6),
    ('MED', 'TOP', '外用藥', 7),
    ('MED', 'INJ', '注射劑', 8),
    ('MED', 'ORL', '口服藥', 9),
    ('MED', 'OTH', '其他', 10)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 醫材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('MSP', 'SYR', '注射器材', 1),
    ('MSP', 'BND', '敷料繃帶', 2),
    ('MSP', 'TUB', '導管管路', 3),
    ('MSP', 'MON', '監測耗材', 4),
    ('MSP', 'SUR', '手術耗材', 5),
    ('MSP', 'OTH', '其他', 6)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 飼料
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('FED', 'PIG', '豬用飼料', 1),
    ('FED', 'MIN', '迷你豬飼料', 2),
    ('FED', 'SUP', '營養補充', 3),
    ('FED', 'OTH', '其他', 4)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 器材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('EQP', 'SUR', '手術器材', 1),
    ('EQP', 'MON', '監測設備', 2),
    ('EQP', 'IMG', '影像設備', 3),
    ('EQP', 'ANE', '麻醉設備', 4),
    ('EQP', 'RES', '保定設備', 5),
    ('EQP', 'WEI', '量測設備', 6),
    ('EQP', 'OTH', '其他', 7)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 耗材
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CON', 'SYR', '針筒注射', 1),
    ('CON', 'GLV', '手套', 2),
    ('CON', 'GAU', '紗布敷料', 3),
    ('CON', 'TUB', '管路耗材', 4),
    ('CON', 'CLN', '清潔消毒', 5),
    ('CON', 'TAG', '標示耗材', 6),
    ('CON', 'LAB', '實驗耗材', 7),
    ('CON', 'OTH', '其他', 8)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 化學品
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('CHM', 'RGT', '試劑', 1),
    ('CHM', 'SOL', '溶劑', 2),
    ('CHM', 'STD', '標準品', 3),
    ('CHM', 'BUF', '緩衝液', 4),
    ('CHM', 'DYE', '染劑', 5),
    ('CHM', 'OTH', '其他', 6)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- 插入 SKU 子類別 - 其他
INSERT INTO sku_subcategories (category_code, code, name, sort_order) VALUES
    ('OTH', 'GEN', '一般', 1)
ON CONFLICT (category_code, code) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order;

-- ============================================
-- 2. 豬隻來源
-- ============================================

INSERT INTO pig_sources (id, code, name, sort_order) VALUES
    (gen_random_uuid(), 'TAITUNG', '台東種畜繁殖場', 1),
    (gen_random_uuid(), 'QINGXIN', '青欣牧場', 2),
    (gen_random_uuid(), 'PIGMODEL', '豬博士畜牧場', 3),
    (gen_random_uuid(), 'PINGSHUN', '平順牧場', 4)
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 3. 倉庫
-- ============================================

INSERT INTO warehouses (id, code, name, address, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'WH001', '主倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH002', '藥物倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH003', '飼料倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH004', '醫療器材倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW()),
    (gen_random_uuid(), 'WH005', '備用倉庫', '苗栗縣後龍鎮外埔里外埔6-15號', true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 4. 供應商
-- ============================================

INSERT INTO partners (id, partner_type, code, name, supplier_category, tax_id, phone, email, address, payment_terms, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'supplier', 'SUP001', '生技藥品股份有限公司', 'drug', '12345678', '02-2345-6789', 'contact@biopharm.com.tw', '台北市信義區信義路五段7號', '月結30天', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP002', '醫療器材供應商', 'consumable', '23456789', '03-3456-7890', 'sales@medsupply.com.tw', '桃園市中壢區中正路100號', '月結45天', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP003', '優質飼料有限公司', 'feed', '34567890', '04-4567-8901', 'info@feedquality.com.tw', '台中市西屯區台灣大道三段99號', '現金交易', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP004', '精密儀器設備公司', 'equipment', '45678901', '06-5678-9012', 'service@precision.com.tw', '台南市東區中華東路200號', '月結60天', true, NOW(), NOW()),
    (gen_random_uuid(), 'supplier', 'SUP005', '綜合醫療用品商行', 'consumable', '56789012', '07-6789-0123', 'order@medsupplies.com.tw', '高雄市前金區中正四路300號', '月結30天', true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 5. 產品
-- ============================================

INSERT INTO products (id, sku, name, spec, base_uom, category_code, subcategory_code, track_batch, track_expiry, default_expiry_days, safety_stock, reorder_point, status, is_active, created_at, updated_at) VALUES
    (gen_random_uuid(), 'MED-ANT-001', '抗生素注射液', '每瓶10ml，濃度100mg/ml，用於治療細菌感染', 'pcs', 'MED', 'ANT', true, true, 365, 50.0000, 20.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'CON-SYR-001', '一次性注射器', '規格：5ml，無菌包裝，每盒100支', 'pcs', 'CON', 'SYR', false, false, NULL, 500.0000, 200.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'FED-PIG-001', '成長期飼料', '蛋白質含量18%，適用於8-20週齡豬隻，每包25kg', 'kg', 'FED', 'PIG', true, true, 180, 1000.0000, 500.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'EQP-WEI-001', '電子體重計', '最大量程500kg，精度0.1kg，適用於豬隻體重測量', 'pcs', 'EQP', 'WEI', false, false, NULL, 2.0000, 1.0000, 'active', true, NOW(), NOW()),
    (gen_random_uuid(), 'CON-GLV-001', '手術手套', '無粉乳膠手套，無菌包裝，尺寸L，每盒100雙', 'pcs', 'CON', 'GLV', false, false, NULL, 200.0000, 100.0000, 'active', true, NOW(), NOW())
ON CONFLICT (sku) DO NOTHING;

-- ============================================
-- 6. 預設月報表排程
-- ============================================

INSERT INTO scheduled_reports (id, report_type, schedule_type, day_of_month, hour_of_day, recipients, is_active, created_at)
VALUES 
    (gen_random_uuid(), 'stock_on_hand', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW()),
    (gen_random_uuid(), 'purchase_summary', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW()),
    (gen_random_uuid(), 'cost_summary', 'monthly', 1, 6, ARRAY[]::uuid[], true, NOW())
ON CONFLICT DO NOTHING;

-- ============================================
-- 完成
-- ============================================
-- ============================================
-- Migration 003: Seed Accounts
-- 
-- ?嚗?-- - 蝟餌絞蝞∠??∪董??-- - ??啣??身撣唾?嚗 Rust ?典???撱箇?嚗?-- ============================================

-- ============================================
-- 1. 撱箇??身蝞∠??∪董??-- 撣唾?: admin@ipig.local
-- 撖Ⅳ: admin123
-- ============================================

INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    'admin@ipig.local',
    '$argon2id$v=19$m=19456,t=2,p=1$Z/2b+2ciQvX6LNhEnutXxA$6h0UrmyUFr2YG1KOWuRQo2kaZUqw/ohhP4+bZblmiZM',
    '蝟餌絞蝞∠???,
    true,
    true,
    false,
    NOW(),
    NOW()
)
ON CONFLICT (email) DO NOTHING;

-- ============================================
-- 2. ?箇恣??晷 admin 閫
-- ============================================

INSERT INTO user_roles (user_id, role_id, assigned_at)
SELECT u.id, r.id, NOW()
FROM users u, roles r
WHERE u.email = 'admin@ipig.local' AND r.code = 'admin'
ON CONFLICT DO NOTHING;

-- ============================================
-- 3. ?箇恣?閫?晷?????-- ============================================

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'admin'
ON CONFLICT DO NOTHING;

-- ============================================
-- 4. ?箇恣?撱箇??閮剖?
-- ============================================

INSERT INTO notification_settings (user_id)
SELECT id FROM users WHERE email = 'admin@ipig.local'
ON CONFLICT (user_id) DO NOTHING;

-- ============================================
-- 5. ??啣??身撣唾?隤芣?
-- ??撣唾???Rust 蝔??典?????撱箇?
-- 隞乩噶甇?Ⅱ?Ｙ? argon2 撖Ⅳ hash
-- ============================================

-- 撣唾??” (??Rust main.rs ?典???撱箇?):
-- 
-- 1. ?∪? <monkey20531@gmail.com>
--    - 撖Ⅳ: test123
--    - 閫: ?瑁?蝘 (IACUC_STAFF), 閰阡?撌乩?鈭箏 (EXPERIMENT_STAFF)
--
-- 2. ?? <lisa82103031@gmail.com>
--    - 撖Ⅳ: test123
--    - 閫: 閰阡?撌乩?鈭箏 (EXPERIMENT_STAFF)
--
-- 3. ?株? <museum1925@gmail.com>
--    - 撖Ⅳ: test123
--    - 閫: 閰阡?撌乩?鈭箏 (EXPERIMENT_STAFF)
--
-- 4. ?? <keytyne@gmail.com>
--    - 撖Ⅳ: test123
--    - 閫: 閰阡?撌乩?鈭箏 (EXPERIMENT_STAFF), ?澈蝞∠???(WAREHOUSE_MANAGER)
--
-- 5. 瘞貊 <raying80@gmail.com>
--    - 撖Ⅳ: test123
--    - 閫: 閰阡?撌乩?鈭箏 (EXPERIMENT_STAFF)
--
-- 6. ?? <smen1971@gmail.com>
--    - 撖Ⅳ: test123
--    - 閫: 閰阡?撌乩?鈭箏 (EXPERIMENT_STAFF), ?澈蝞∠???(WAREHOUSE_MANAGER), ?∟頃鈭箏 (PURCHASING)

-- ============================================
-- 摰?
-- ============================================
-- ============================================
-- Migration 007: Seed Data & Extensibility
-- 
-- ?嚗?-- - ?拍車?身??甈?蝞∠?
-- - ?券?蝞∠?
-- - 雿輻??憟質身摰?-- - 5 ??蝔踹?霅?-- - 3 蝑鞈澆
-- ============================================

-- ============================================
-- 1. ?拍車蝞∠?銵?-- ============================================

CREATE TABLE species (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    name_en VARCHAR(100),
    icon VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default species (pig)
INSERT INTO species (id, code, name, name_en, icon, sort_order, config)
VALUES (
    gen_random_uuid(),
    'pig',
    '鞊?,
    'Pig',
    'pig',
    1,
    '{"breeds": ["Minipig", "White", "Other"], "identifier_label": "?唾?", "identifier_format": "###", "default_pen_prefix": ["A", "B", "C", "D", "E", "F", "G"]}'::jsonb
);

-- ============================================
-- 2. 閮剜撅斤?銵?-- ============================================

-- Facilities
CREATE TABLE facilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    address TEXT,
    phone VARCHAR(50),
    contact_person VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO facilities (id, code, name, address)
VALUES (gen_random_uuid(), 'MAIN', '鞊砍?憯怠??拍??銝剖?', '??蝮?姘?');

-- Buildings
CREATE TABLE buildings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    facility_id UUID NOT NULL REFERENCES facilities(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(facility_id, code)
);

-- Zones
CREATE TABLE zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL,
    name VARCHAR(50),
    color VARCHAR(20),
    is_active BOOLEAN DEFAULT true,
    layout_config JSONB DEFAULT '{}',
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(building_id, code)
);

-- Pens
CREATE TABLE pens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL,
    name VARCHAR(50),
    capacity INTEGER DEFAULT 1,
    current_count INTEGER DEFAULT 0,
    status VARCHAR(20) DEFAULT 'available',
    row_index INTEGER,
    col_index INTEGER,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(zone_id, code)
);

CREATE INDEX idx_pens_zone ON pens(zone_id);
CREATE INDEX idx_pens_code ON pens(code);
CREATE INDEX idx_pens_status ON pens(status);

-- Seed Buildings, Zones, and Pens
DO $$
DECLARE
    v_facility_id UUID;
    v_building_a_id UUID;
    v_building_b_id UUID;
    v_zone_id UUID;
    i INTEGER;
BEGIN
    SELECT id INTO v_facility_id FROM facilities WHERE code = 'MAIN';
    
    -- Create Building A (ACD zones)
    INSERT INTO buildings (facility_id, code, name, sort_order, config)
    VALUES (v_facility_id, 'A', 'A璉?(ACD?)', 1, '{"zones": ["A", "C", "D"]}'::jsonb)
    RETURNING id INTO v_building_a_id;
    
    -- Create Building B (BEFG zones)
    INSERT INTO buildings (facility_id, code, name, sort_order, config)
    VALUES (v_facility_id, 'B', 'B璉?(BEFG?)', 2, '{"zones": ["B", "E", "F", "G"]}'::jsonb)
    RETURNING id INTO v_building_b_id;
    
    -- Zone A
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'A', 'A?', '#4CAF50', 1, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'A' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone C
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'C', 'C?', '#2196F3', 2, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'C' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone D
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'D', 'D?', '#9C27B0', 3, '{"rows": 4, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..33 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'D' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone B
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'B', 'B?', '#FF9800', 1, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'B' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone E
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'E', 'E?', '#E91E63', 2, '{"rows": 2, "cols": 5}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..10 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'E' || LPAD(i::text, 2, '0'), (i-1)/5 + 1, ((i-1) % 5) + 1);
    END LOOP;
    
    -- Zone F
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'F', 'F?', '#FFEB3B', 3, '{"rows": 2, "cols": 3, "special_layout": true}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..3 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index, capacity)
        VALUES (v_zone_id, 'F' || LPAD(i::text, 2, '0'), 1, i, 5);
    END LOOP;
    
    -- Zone G
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'G', 'G?', '#607D8B', 4, '{"rows": 1, "cols": 5}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..5 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'G' || LPAD(i::text, 2, '0'), 1, i);
    END LOOP;
END $$;

-- Add pen_id to pigs table
ALTER TABLE pigs ADD COLUMN IF NOT EXISTS pen_id UUID REFERENCES pens(id);
CREATE INDEX IF NOT EXISTS idx_pigs_pen ON pigs(pen_id);

-- ============================================
-- 3. ?券?蝞∠?銵?-- ============================================

CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES departments(id),
    manager_id UUID REFERENCES users(id),
    is_active BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}',
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_departments_parent ON departments(parent_id);
CREATE INDEX idx_departments_manager ON departments(manager_id);

-- Seed default departments
INSERT INTO departments (id, code, name, sort_order) VALUES
    (gen_random_uuid(), 'ADMIN', '銵?券?', 1),
    (gen_random_uuid(), 'LAB', '撖阡?摰?, 2),
    (gen_random_uuid(), 'VET', '?賊?券?', 3),
    (gen_random_uuid(), 'WAREHOUSE', '??券?', 4);

-- Add department fields to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS department_id UUID REFERENCES departments(id);
ALTER TABLE users ADD COLUMN IF NOT EXISTS direct_manager_id UUID REFERENCES users(id);
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department_id);
CREATE INDEX IF NOT EXISTS idx_users_manager ON users(direct_manager_id);

-- ============================================
-- 4. 閫蝢斤?銵?-- ============================================

CREATE TABLE role_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE role_group_roles (
    role_group_id UUID NOT NULL REFERENCES role_groups(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (role_group_id, role_id)
);

INSERT INTO role_groups (code, name, description) VALUES
    ('INTERNAL_STAFF', '?折?∪極', '???典撌亙?函??箸甈?蝢斤?'),
    ('EXPERIMENT_TEAM', '撖阡???', '撖阡??賊?撌乩?鈭箏???黎蝯?),
    ('ADMIN_TEAM', '蝞∠???', '蝟餌絞蝞∠??賊?甈?蝢斤?');

-- ============================================
-- 5. 雿輻??憟質身摰”
-- ============================================

CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key VARCHAR(100) NOT NULL,
    preference_value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, preference_key)
);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

-- Update trigger
CREATE OR REPLACE FUNCTION update_user_preferences_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_user_preferences_updated_at
    BEFORE UPDATE ON user_preferences
    FOR EACH ROW
    EXECUTE FUNCTION update_user_preferences_updated_at();

COMMENT ON TABLE user_preferences IS '雿輻??憟質身摰”';
COMMENT ON COLUMN user_preferences.preference_key IS '閮剖??萄?嚗? nav_order, dashboard_widgets';
COMMENT ON COLUMN user_preferences.preference_value IS 'JSON ?澆??身摰?;

-- ============================================
-- 6. Pen Details View
-- ============================================

CREATE OR REPLACE VIEW pen_details AS
SELECT 
    p.id,
    p.code,
    p.name,
    p.capacity,
    p.current_count,
    p.status,
    z.id AS zone_id,
    z.code AS zone_code,
    z.name AS zone_name,
    z.color AS zone_color,
    b.id AS building_id,
    b.code AS building_code,
    b.name AS building_name,
    f.id AS facility_id,
    f.code AS facility_code,
    f.name AS facility_name
FROM pens p
JOIN zones z ON p.zone_id = z.id
JOIN buildings b ON z.building_id = b.id
JOIN facilities f ON b.facility_id = f.id
WHERE p.is_active = true;

-- ============================================
-- 7. 憿?甈?
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- Facilities
    (gen_random_uuid(), 'facility.read', '瑼Ｚ?閮剜鞈?', 'facility', '?舀炎閬身??, NOW()),
    (gen_random_uuid(), 'facility.create', '撱箇?閮剜', 'facility', '?臬遣蝡身??, NOW()),
    (gen_random_uuid(), 'facility.update', '?湔閮剜', 'facility', '?舀?啗身??, NOW()),
    (gen_random_uuid(), 'facility.delete', '?芷閮剜', 'facility', '?臬?方身??, NOW()),
    -- Building
    (gen_random_uuid(), 'building.read', '瑼Ｚ?璉?鞈?', 'facility', '?舀炎閬???, NOW()),
    (gen_random_uuid(), 'building.create', '撱箇?璉?', 'facility', '?臬遣蝡???, NOW()),
    (gen_random_uuid(), 'building.update', '?湔璉?', 'facility', '?舀?唳???, NOW()),
    (gen_random_uuid(), 'building.delete', '?芷璉?', 'facility', '?臬?斗???, NOW()),
    -- Zone
    (gen_random_uuid(), 'zone.read', '瑼Ｚ????閮?, 'facility', '?舀炎閬???, NOW()),
    (gen_random_uuid(), 'zone.create', '撱箇????, 'facility', '?臬遣蝡???, NOW()),
    (gen_random_uuid(), 'zone.update', '?湔???, 'facility', '?舀?啣???, NOW()),
    (gen_random_uuid(), 'zone.delete', '?芷???, 'facility', '?臬?文???, NOW()),
    -- Pen
    (gen_random_uuid(), 'pen.read', '瑼Ｚ?甈?鞈?', 'facility', '?舀炎閬?雿?, NOW()),
    (gen_random_uuid(), 'pen.create', '撱箇?甈?', 'facility', '?臬遣蝡?雿?, NOW()),
    (gen_random_uuid(), 'pen.update', '?湔甈?', 'facility', '?舀?唳?雿?, NOW()),
    (gen_random_uuid(), 'pen.delete', '?芷甈?', 'facility', '?臬?斗?雿?, NOW()),
    -- Species
    (gen_random_uuid(), 'species.read', '瑼Ｚ??拍車鞈?', 'species', '?舀炎閬蝔?, NOW()),
    (gen_random_uuid(), 'species.create', '撱箇??拍車', 'species', '?臬遣蝡蝔?, NOW()),
    (gen_random_uuid(), 'species.update', '?湔?拍車', 'species', '?舀?啁蝔?, NOW()),
    (gen_random_uuid(), 'species.delete', '?芷?拍車', 'species', '?臬?斤蝔?, NOW()),
    -- Department
    (gen_random_uuid(), 'department.read', '瑼Ｚ??券?鞈?', 'department', '?舀炎閬?', NOW()),
    (gen_random_uuid(), 'department.create', '撱箇??券?', 'department', '?臬遣蝡?', NOW()),
    (gen_random_uuid(), 'department.update', '?湔?券?', 'department', '?舀?圈?', NOW()),
    (gen_random_uuid(), 'department.delete', '?芷?券?', 'department', '?臬?日?', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 8. 撱箇? 5 ??蝔踹?霅?-- ============================================

DO $$
DECLARE
    v_admin_id UUID;
    v_protocol_id UUID;
    v_protocol_no VARCHAR(50);
    i INTEGER;
BEGIN
    -- Get admin user
    SELECT id INTO v_admin_id FROM users WHERE email = 'admin@ipig.local' LIMIT 1;
    
    IF v_admin_id IS NULL THEN
        RAISE NOTICE 'Admin user not found, skipping protocol creation';
        RETURN;
    END IF;
    
    FOR i IN 1..5 LOOP
        v_protocol_id := gen_random_uuid();
        v_protocol_no := 'AUP-2026-' || LPAD(i::text, 4, '0');
        
        INSERT INTO protocols (
            id, protocol_no, title, status, pi_user_id, created_by,
            working_content, start_date, end_date, created_at, updated_at
        ) VALUES (
            v_protocol_id,
            v_protocol_no,
            CASE i
                WHEN 1 THEN '餈瑚?鞊砍???蝞∪??質?隡啁?蝛?
                WHEN 2 THEN '??蝟?拇??岫撽?
                WHEN 3 THEN '?啣??怨?摰?扯?隡?
                WHEN 4 THEN '?怎??冽???詨捆?扳葫閰?
                WHEN 5 THEN '銝剛??交?????蝛?
            END,
            'DRAFT',
            v_admin_id,
            v_admin_id,
            jsonb_build_object(
                'abstract', '?銝??蝔踹?霅堆??冽皜祈岫蝟餌絞????怎楊?? ' || v_protocol_no,
                'methodology', '?祉?蝛嗡蝙?冽?皞?撖阡?瘚??脰?閰阡?閮剛???,
                'animalCount', i * 10,
                'species', '鞊?,
                'breed', 'Minipig',
                'duration', '6??',
                'createdAt', NOW()
            ),
            CURRENT_DATE + INTERVAL '1 month',
            CURRENT_DATE + INTERVAL '7 month',
            NOW(),
            NOW()
        );
        
        -- Add status history
        INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
        VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_admin_id, '撱箇??阮?降', NOW());
        
        -- Add user-protocol relationship
        INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
        VALUES (v_admin_id, v_protocol_id, 'PI', NOW(), v_admin_id);
        
    END LOOP;
END $$;

-- ============================================
-- 9. 撱箇? 3 蝑?閮剜鞈澆
-- ============================================

DO $$
DECLARE
    v_admin_id UUID;
    v_warehouse_id UUID;
    v_supplier_id UUID;
    v_product_id UUID;
    v_doc_id UUID;
    v_product_sku VARCHAR(50);
    i INTEGER;
BEGIN
    -- Get admin user
    SELECT id INTO v_admin_id FROM users WHERE email = 'admin@ipig.local' LIMIT 1;
    
    IF v_admin_id IS NULL THEN
        RAISE NOTICE 'Admin user not found, skipping PO creation';
        RETURN;
    END IF;
    
    -- Get first warehouse
    SELECT id INTO v_warehouse_id FROM warehouses WHERE code = 'WH001' LIMIT 1;
    
    IF v_warehouse_id IS NULL THEN
        RAISE NOTICE 'Warehouse not found, skipping PO creation';
        RETURN;
    END IF;
    
    FOR i IN 1..3 LOOP
        v_doc_id := gen_random_uuid();
        
        -- Get supplier based on iteration
        SELECT id INTO v_supplier_id FROM partners WHERE code = 'SUP00' || i LIMIT 1;
        
        IF v_supplier_id IS NULL THEN
            CONTINUE;
        END IF;
        
        -- Create PO document
        INSERT INTO documents (
            id, doc_type, doc_no, status, warehouse_id, partner_id,
            doc_date, remark, created_by, created_at, updated_at
        ) VALUES (
            v_doc_id,
            'PO',
            'PO-2026-' || LPAD(i::text, 4, '0'),
            'draft',
            v_warehouse_id,
            v_supplier_id,
            CURRENT_DATE,
            CASE i
                WHEN 1 THEN '撣貉??亙?鋆疏?∟頃'
                WHEN 2 THEN '??鋆??∟頃'
                WHEN 3 THEN '憌潭?摰??∟頃'
            END,
            v_admin_id,
            NOW(),
            NOW()
        );
        
        -- Get products based on iteration
        IF i = 1 THEN
            v_product_sku := 'MED-ANT-001';
        ELSIF i = 2 THEN
            v_product_sku := 'CON-SYR-001';
        ELSE
            v_product_sku := 'FED-PIG-001';
        END IF;
        
        SELECT id INTO v_product_id FROM products WHERE sku = v_product_sku LIMIT 1;
        
        IF v_product_id IS NOT NULL THEN
            -- Create PO line with batch_no and expiry_date
            INSERT INTO document_lines (
                id, document_id, line_no, product_id, qty, uom,
                unit_price, batch_no, expiry_date, remark
            ) VALUES (
                gen_random_uuid(),
                v_doc_id,
                1,
                v_product_id,
                CASE i WHEN 1 THEN 100 WHEN 2 THEN 500 ELSE 200 END,
                CASE i WHEN 3 THEN 'kg' ELSE 'pcs' END,
                CASE i WHEN 1 THEN 150.00 WHEN 2 THEN 2.50 ELSE 45.00 END,
                'BATCH-2026-' || LPAD(i::text, 3, '0'),
                CURRENT_DATE + INTERVAL '6 months',
                '?身?∟頃??'
            );
        END IF;
        
    END LOOP;
END $$;

-- ============================================
-- 摰?
-- ============================================
-- Migration 008: Reset Admin Account
-- This migration resets the admin account password to admin123
-- Email: admin@ipig.local
-- Password: admin123

-- Update admin user password (argon2id hash for 'admin123')
UPDATE users 
SET 
    password_hash = '$argon2id$v=19$m=19456,t=2,p=1$KVnFxLOfhBXicAx+BQ1k7g$9Zz0oFETd5Md5sZdMXI2AHM+XqVQrDpa+zCBukHhf0A',
    is_active = true,
    must_change_password = false,
    updated_at = NOW()
WHERE email = 'admin@ipig.local';

-- Ensure admin user exists (create if not exists)
INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    'admin@ipig.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KVnFxLOfhBXicAx+BQ1k7g$9Zz0oFETd5Md5sZdMXI2AHM+XqVQrDpa+zCBukHhf0A',
    '蝟餌絞蝞∠???,
    true,
    true,
    false,
    NOW(),
    NOW()
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@ipig.local');

-- Ensure admin role assignment
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.email = 'admin@ipig.local' 
  AND (r.code = 'admin' OR r.code = 'SYSTEM_ADMIN')
ON CONFLICT DO NOTHING;
