-- Extensibility Foundation: Species, Facilities, and Departments
-- Version: 1.0
-- Created: 2026-01-17
-- Description: Creates abstract tables for species, facilities/buildings/zones/pens, and departments

-- ============================================
-- Species Abstraction
-- ============================================

CREATE TABLE species (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE, -- 'pig', 'rabbit', 'mouse', etc.
    name VARCHAR(100) NOT NULL, -- '豬', '兔', '小鼠'
    name_en VARCHAR(100), -- English name
    icon VARCHAR(50), -- Icon identifier for UI
    is_active BOOLEAN DEFAULT true,
    
    -- Species-specific configuration
    config JSONB DEFAULT '{}', -- {"breeds": [...], "default_fields": [...]}
    
    -- Sorting
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
    '{
        "breeds": ["Minipig", "White", "Other"],
        "identifier_label": "耳號",
        "identifier_format": "###",
        "default_pen_prefix": ["A", "B", "C", "D", "E", "F", "G"]
    }'::jsonb
);

-- ============================================
-- Facilities Hierarchy
-- ============================================

-- Facilities (top level - e.g., research centers)
CREATE TABLE facilities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    address TEXT,
    phone VARCHAR(50),
    contact_person VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    
    -- Configuration
    config JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default facility
INSERT INTO facilities (id, code, name, address)
VALUES (
    gen_random_uuid(),
    'MAIN',
    '豬博士動物科技中心',
    '苗栗縣竹南鎮'
);

-- Buildings (within facilities)
CREATE TABLE buildings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    facility_id UUID NOT NULL REFERENCES facilities(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL, -- 'A', 'B'
    name VARCHAR(100) NOT NULL, -- 'A棟', 'B棟'
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    
    -- Configuration
    config JSONB DEFAULT '{}', -- {"zones": ["A", "C", "D"]}
    
    sort_order INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(facility_id, code)
);

-- Zones (within buildings)
CREATE TABLE zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL, -- 'A', 'B', 'C', 'D', 'E', 'F', 'G'
    name VARCHAR(50), -- 'A區', 'B區'
    color VARCHAR(20), -- Hex color for UI, e.g., '#4CAF50'
    is_active BOOLEAN DEFAULT true,
    
    -- Layout configuration
    layout_config JSONB DEFAULT '{}', -- {"rows": 2, "cols": 10, "special_layout": false}
    
    sort_order INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(building_id, code)
);

-- Pens (within zones)
CREATE TABLE pens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id UUID NOT NULL REFERENCES zones(id) ON DELETE CASCADE,
    code VARCHAR(20) NOT NULL, -- 'A01', 'A02', 'B01'
    name VARCHAR(50), -- Optional display name
    
    -- Capacity
    capacity INTEGER DEFAULT 1,
    current_count INTEGER DEFAULT 0,
    
    -- Status
    status VARCHAR(20) DEFAULT 'available', -- 'available', 'occupied', 'maintenance', 'reserved'
    
    -- Position for UI layout
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

-- ============================================
-- Seed Buildings, Zones, and Pens for Main Facility
-- ============================================

-- Get the main facility ID
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
    
    -- Create Zone A (in Building A)
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'A', 'A區', '#4CAF50', 1, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens A01-A20
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'A' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Create Zone C (in Building A)
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'C', 'C區', '#2196F3', 2, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens C01-C20
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'C' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Create Zone D (in Building A)
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_a_id, 'D', 'D區', '#9C27B0', 3, '{"rows": 4, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens D01-D33
    FOR i IN 1..33 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'D' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Create Zone B (in Building B)
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'B', 'B區', '#FF9800', 1, '{"rows": 2, "cols": 10}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens B01-B20
    FOR i IN 1..20 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'B' || LPAD(i::text, 2, '0'), (i-1)/10 + 1, ((i-1) % 10) + 1);
    END LOOP;
    
    -- Create Zone E (in Building B)
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'E', 'E區', '#E91E63', 2, '{"rows": 2, "cols": 5}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens E01-E10
    FOR i IN 1..10 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'E' || LPAD(i::text, 2, '0'), (i-1)/5 + 1, ((i-1) % 5) + 1);
    END LOOP;
    
    -- Create Zone F (in Building B) - special layout
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'F', 'F區', '#FFEB3B', 3, '{"rows": 2, "cols": 3, "special_layout": true}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens F01-F03
    FOR i IN 1..3 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index, capacity)
        VALUES (v_zone_id, 'F' || LPAD(i::text, 2, '0'), 1, i, 5); -- Higher capacity for F zone
    END LOOP;
    
    -- Create Zone G (in Building B)
    INSERT INTO zones (building_id, code, name, color, sort_order, layout_config)
    VALUES (v_building_b_id, 'G', 'G區', '#607D8B', 4, '{"rows": 1, "cols": 5}'::jsonb)
    RETURNING id INTO v_zone_id;
    
    -- Create pens G01-G05
    FOR i IN 1..5 LOOP
        INSERT INTO pens (zone_id, code, row_index, col_index)
        VALUES (v_zone_id, 'G' || LPAD(i::text, 2, '0'), 1, i);
    END LOOP;
    
END $$;

-- ============================================
-- Departments for Personnel Management
-- ============================================

CREATE TABLE departments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES departments(id), -- For hierarchy
    manager_id UUID REFERENCES users(id), -- Department head
    is_active BOOLEAN DEFAULT true,
    
    -- Configuration
    config JSONB DEFAULT '{}', -- {"max_leave_percentage": 30}
    
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

-- Add department and manager fields to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS department_id UUID REFERENCES departments(id);
ALTER TABLE users ADD COLUMN IF NOT EXISTS direct_manager_id UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_users_department ON users(department_id);
CREATE INDEX IF NOT EXISTS idx_users_manager ON users(direct_manager_id);

-- ============================================
-- Role Groups for Easier Assignment
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

-- Seed default role groups
INSERT INTO role_groups (code, name, description) VALUES
    ('INTERNAL_STAFF', '內部員工', '所有內部員工共用的基本權限群組'),
    ('EXPERIMENT_TEAM', '實驗團隊', '實驗相關工作人員的權限群組'),
    ('ADMIN_TEAM', '管理團隊', '系統管理相關權限群組');

-- ============================================
-- Add pen_id to pigs table for proper relationship
-- ============================================

ALTER TABLE pigs ADD COLUMN IF NOT EXISTS pen_id UUID REFERENCES pens(id);

-- Create index for the new column
CREATE INDEX IF NOT EXISTS idx_pigs_pen ON pigs(pen_id);

-- ============================================
-- Views for Backward Compatibility
-- ============================================

-- View to get pen info with full hierarchy
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
-- Permissions for New Features
-- ============================================

INSERT INTO permissions (id, code, name, created_at) VALUES
    -- Facilities management
    (gen_random_uuid(), 'facility.read', '檢視設施資訊', NOW()),
    (gen_random_uuid(), 'facility.create', '建立設施', NOW()),
    (gen_random_uuid(), 'facility.update', '更新設施', NOW()),
    (gen_random_uuid(), 'facility.delete', '刪除設施', NOW()),
    
    -- Building management
    (gen_random_uuid(), 'building.read', '檢視棟舍資訊', NOW()),
    (gen_random_uuid(), 'building.create', '建立棟舍', NOW()),
    (gen_random_uuid(), 'building.update', '更新棟舍', NOW()),
    (gen_random_uuid(), 'building.delete', '刪除棟舍', NOW()),
    
    -- Zone management
    (gen_random_uuid(), 'zone.read', '檢視區域資訊', NOW()),
    (gen_random_uuid(), 'zone.create', '建立區域', NOW()),
    (gen_random_uuid(), 'zone.update', '更新區域', NOW()),
    (gen_random_uuid(), 'zone.delete', '刪除區域', NOW()),
    
    -- Pen management
    (gen_random_uuid(), 'pen.read', '檢視欄位資訊', NOW()),
    (gen_random_uuid(), 'pen.create', '建立欄位', NOW()),
    (gen_random_uuid(), 'pen.update', '更新欄位', NOW()),
    (gen_random_uuid(), 'pen.delete', '刪除欄位', NOW()),
    
    -- Species management
    (gen_random_uuid(), 'species.read', '檢視物種資訊', NOW()),
    (gen_random_uuid(), 'species.create', '建立物種', NOW()),
    (gen_random_uuid(), 'species.update', '更新物種', NOW()),
    (gen_random_uuid(), 'species.delete', '刪除物種', NOW()),
    
    -- Department management
    (gen_random_uuid(), 'department.read', '檢視部門資訊', NOW()),
    (gen_random_uuid(), 'department.create', '建立部門', NOW()),
    (gen_random_uuid(), 'department.update', '更新部門', NOW()),
    (gen_random_uuid(), 'department.delete', '刪除部門', NOW())
ON CONFLICT (code) DO NOTHING;

-- Assign to admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'admin' 
  AND p.code IN (
    'facility.read', 'facility.create', 'facility.update', 'facility.delete',
    'building.read', 'building.create', 'building.update', 'building.delete',
    'zone.read', 'zone.create', 'zone.update', 'zone.delete',
    'pen.read', 'pen.create', 'pen.update', 'pen.delete',
    'species.read', 'species.create', 'species.update', 'species.delete',
    'department.read', 'department.create', 'department.update', 'department.delete'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Assign read permissions to experiment staff
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'experiment_staff' 
  AND p.code IN (
    'facility.read', 'building.read', 'zone.read', 'pen.read', 
    'species.read', 'department.read'
  )
ON CONFLICT (role_id, permission_id) DO NOTHING;
