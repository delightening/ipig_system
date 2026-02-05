-- ============================================
-- Migration 007: Seed Data & Extensibility
-- 
-- 包含：
-- - 物種與設施/欄位管理
-- - 部門管理
-- - 使用者偏好設定
-- - 5 個草稿協議
-- - 3 筆採購單
-- ============================================

-- ============================================
-- 1. 物種管理表
-- ============================================

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
    '豬',
    'Pig',
    'pig',
    1,
    '{"breeds": ["Minipig", "White", "Other"], "identifier_label": "耳號", "identifier_format": "###", "default_pen_prefix": ["A", "B", "C", "D", "E", "F", "G"]}'::jsonb
);

-- ============================================
-- 2. 設施層級表
-- ============================================

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
VALUES (gen_random_uuid(), 'MAIN', '豬博士動物科技中心', '苗栗縣竹南鎮');

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
    VALUES (v_facility_id, 'A', 'A棟 (ACD區)', 1, '{"zones": ["A", "C", "D"]}'::jsonb)
    RETURNING id INTO v_building_a_id;
    
    -- Create Building B (BEFG zones)
    INSERT INTO buildings (facility_id, code, name, sort_order, config)
    VALUES (v_facility_id, 'B', 'B棟 (BEFG區)', 2, '{"zones": ["B", "E", "F", "G"]}'::jsonb)
    RETURNING id INTO v_building_b_id;
    
    -- Zone A
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'A', 'A區', '#4CAF50', 1, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'A' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone C
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'C', 'C區', '#2196F3', 2, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'C' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone D
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'D', 'D區', '#9C27B0', 3, '{"rows": 4, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..33 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'D' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone B
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'B', 'B區', '#FF9800', 1, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'B' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Zone E
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'E', 'E區', '#E91E63', 2, '{"rows": 2, "cols": 5}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..10 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'E' || LPAD(i::text, 2, '0'), (i-1)/5 + 1, ((i-1) % 5) + 1);
    END LOOP;
    
    -- Zone F
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'F', 'F區', '#FFEB3B', 3, '{"rows": 2, "cols": 3, "special_layout": true}'::jsonb)
    RETURNING id INTO v_zone_id;
    FOR i IN 1..3 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index, capacity)
        VALUES (v_zone_id, 'F' || LPAD(i::text, 2, '0'), 1, i, 5);
    END LOOP;
    
    -- Zone G
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'G', 'G區', '#607D8B', 4, '{"rows": 1, "cols": 5}'::jsonb)
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
-- 3. 部門管理表
-- ============================================

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
    (gen_random_uuid(), 'ADMIN', '行政部門', 1),
    (gen_random_uuid(), 'LAB', '實驗室', 2),
    (gen_random_uuid(), 'VET', '獸醫部門', 3),
    (gen_random_uuid(), 'WAREHOUSE', '倉儲部門', 4);

-- Add department fields to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS department_id UUID REFERENCES departments(id);
ALTER TABLE users ADD COLUMN IF NOT EXISTS direct_manager_id UUID REFERENCES users(id);
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department_id);
CREATE INDEX IF NOT EXISTS idx_users_manager ON users(direct_manager_id);

-- ============================================
-- 4. 角色群組表
-- ============================================

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
    ('INTERNAL_STAFF', '內部員工', '所有內部員工共用的基本權限群組'),
    ('EXPERIMENT_TEAM', '實驗團隊', '實驗相關工作人員的權限群組'),
    ('ADMIN_TEAM', '管理團隊', '系統管理相關權限群組');

-- ============================================
-- 5. 使用者偏好設定表
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

COMMENT ON TABLE user_preferences IS '使用者偏好設定表';
COMMENT ON COLUMN user_preferences.preference_key IS '設定鍵名，如 nav_order, dashboard_widgets';
COMMENT ON COLUMN user_preferences.preference_value IS 'JSON 格式的設定值';

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
-- 7. 額外權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    -- Facilities
    (gen_random_uuid(), 'facility.read', '檢視設施資訊', 'facility', '可檢視設施', NOW()),
    (gen_random_uuid(), 'facility.create', '建立設施', 'facility', '可建立設施', NOW()),
    (gen_random_uuid(), 'facility.update', '更新設施', 'facility', '可更新設施', NOW()),
    (gen_random_uuid(), 'facility.delete', '刪除設施', 'facility', '可刪除設施', NOW()),
    -- Building
    (gen_random_uuid(), 'building.read', '檢視棟舍資訊', 'facility', '可檢視棟舍', NOW()),
    (gen_random_uuid(), 'building.create', '建立棟舍', 'facility', '可建立棟舍', NOW()),
    (gen_random_uuid(), 'building.update', '更新棟舍', 'facility', '可更新棟舍', NOW()),
    (gen_random_uuid(), 'building.delete', '刪除棟舍', 'facility', '可刪除棟舍', NOW()),
    -- Zone
    (gen_random_uuid(), 'zone.read', '檢視區域資訊', 'facility', '可檢視區域', NOW()),
    (gen_random_uuid(), 'zone.create', '建立區域', 'facility', '可建立區域', NOW()),
    (gen_random_uuid(), 'zone.update', '更新區域', 'facility', '可更新區域', NOW()),
    (gen_random_uuid(), 'zone.delete', '刪除區域', 'facility', '可刪除區域', NOW()),
    -- Pen
    (gen_random_uuid(), 'pen.read', '檢視欄位資訊', 'facility', '可檢視欄位', NOW()),
    (gen_random_uuid(), 'pen.create', '建立欄位', 'facility', '可建立欄位', NOW()),
    (gen_random_uuid(), 'pen.update', '更新欄位', 'facility', '可更新欄位', NOW()),
    (gen_random_uuid(), 'pen.delete', '刪除欄位', 'facility', '可刪除欄位', NOW()),
    -- Species
    (gen_random_uuid(), 'species.read', '檢視物種資訊', 'species', '可檢視物種', NOW()),
    (gen_random_uuid(), 'species.create', '建立物種', 'species', '可建立物種', NOW()),
    (gen_random_uuid(), 'species.update', '更新物種', 'species', '可更新物種', NOW()),
    (gen_random_uuid(), 'species.delete', '刪除物種', 'species', '可刪除物種', NOW()),
    -- Department
    (gen_random_uuid(), 'department.read', '檢視部門資訊', 'department', '可檢視部門', NOW()),
    (gen_random_uuid(), 'department.create', '建立部門', 'department', '可建立部門', NOW()),
    (gen_random_uuid(), 'department.update', '更新部門', 'department', '可更新部門', NOW()),
    (gen_random_uuid(), 'department.delete', '刪除部門', 'department', '可刪除部門', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 8. 建立 5 個草稿協議
-- ============================================

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
                WHEN 1 THEN '迷你豬心臟血管功能評估研究'
                WHEN 2 THEN '降血糖藥物效力試驗'
                WHEN 3 THEN '新型疫苗安全性評估'
                WHEN 4 THEN '醫療器材生物相容性測試'
                WHEN 5 THEN '中草藥抗炎效果研究'
            END,
            'DRAFT',
            v_admin_id,
            v_admin_id,
            jsonb_build_object(
                'abstract', '這是一個草稿協議，用於測試系統功能。計畫編號: ' || v_protocol_no,
                'methodology', '本研究使用標準化實驗流程進行試驗設計。',
                'animalCount', i * 10,
                'species', '豬',
                'breed', 'Minipig',
                'duration', '6個月',
                'createdAt', NOW()
            ),
            CURRENT_DATE + INTERVAL '1 month',
            CURRENT_DATE + INTERVAL '7 month',
            NOW(),
            NOW()
        );
        
        -- Add status history
        INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
        VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_admin_id, '建立草稿協議', NOW());
        
        -- Add user-protocol relationship
        INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
        VALUES (v_admin_id, v_protocol_id, 'PI', NOW(), v_admin_id);
        
    END LOOP;
END $$;

-- ============================================
-- 9. 建立 3 筆預設採購單
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
                WHEN 1 THEN '常規藥品補貨採購'
                WHEN 2 THEN '耗材補充採購'
                WHEN 3 THEN '飼料定期採購'
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
                '預設採購品項'
            );
        END IF;
        
    END LOOP;
END $$;

-- ============================================
-- 完成
-- ============================================
