-- ============================================
-- Migration 008: Facility, Equipment & Maintenance
-- Squashed from: 010(partial), 018, 019, 020, 022(partial)
-- ============================================

-- ─── 8.1 Enum 型別 已在 001_types.sql 中定義 ─────────────────────────
-- equipment_status, calibration_type, calibration_cycle, maintenance_type,
-- maintenance_status, disposal_status

-- ─── 8.2 物種 (Species) ─────────────────────────────────────────────
-- 直接包含 parent_id (from 019)

CREATE TABLE species (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    name_en VARCHAR(100),
    icon VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,
    sort_order INTEGER NOT NULL DEFAULT 0,
    parent_id UUID REFERENCES species(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_species_parent_id ON species(parent_id);

-- ─── 8.3 設施 (Facility) ────────────────────────────────────────────
-- 使用 partial unique index (from 020)

CREATE TABLE facilities (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    address TEXT,
    phone VARCHAR(50),
    contact_person VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_facilities_code_active ON facilities (code) WHERE is_active = true;

-- ─── 8.4 建築 (Building) ────────────────────────────────────────────
-- 使用 partial unique index (from 020)

CREATE TABLE buildings (
    id UUID PRIMARY KEY,
    facility_id UUID NOT NULL REFERENCES facilities(id),
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_buildings_facility_code_active ON buildings (facility_id, code) WHERE is_active = true;
CREATE INDEX idx_buildings_facility_id ON buildings(facility_id);

-- ─── 8.5 分區 (Zone) ────────────────────────────────────────────────
-- 使用 partial unique index (from 020)

CREATE TABLE zones (
    id UUID PRIMARY KEY,
    building_id UUID NOT NULL REFERENCES buildings(id),
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200),
    color VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT true,
    layout_config JSONB,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_zones_building_code_active ON zones (building_id, code) WHERE is_active = true;
CREATE INDEX idx_zones_building_id ON zones(building_id);

-- ─── 8.6 欄位 (Pen) ─────────────────────────────────────────────────
-- 使用 partial unique index (from 020)

CREATE TABLE pens (
    id UUID PRIMARY KEY,
    zone_id UUID NOT NULL REFERENCES zones(id),
    code VARCHAR(50) NOT NULL,
    name VARCHAR(200),
    capacity INTEGER NOT NULL DEFAULT 1,
    current_count INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    row_index INTEGER,
    col_index INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_pens_zone_code_active ON pens (zone_id, code) WHERE is_active = true;
CREATE INDEX idx_pens_zone_id ON pens(zone_id);

-- ─── 8.7 部門 (Department) ──────────────────────────────────────────

CREATE TABLE departments (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    parent_id UUID REFERENCES departments(id),
    manager_id UUID REFERENCES users(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    config JSONB,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_departments_parent_id ON departments(parent_id);

-- ─── 8.8 設備 (Equipment) ───────────────────────────────────────────
-- 直接包含 018 的 status, calibration_type, calibration_cycle, inspection_cycle

CREATE TABLE equipment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(200) NOT NULL,
    model VARCHAR(200),
    serial_number VARCHAR(100),
    location VARCHAR(200),
    notes TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    status equipment_status NOT NULL DEFAULT 'active',
    calibration_type calibration_type,
    calibration_cycle calibration_cycle,
    inspection_cycle calibration_cycle,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_name ON equipment(name);
CREATE INDEX idx_equipment_active ON equipment(is_active);
CREATE INDEX idx_equipment_status ON equipment(status);

-- ─── 8.9 設備校準 (Equipment Calibrations) ──────────────────────────
-- 直接包含 018 的 calibration_type, partner_id, report_number, inspector, equipment_serial_number

CREATE TABLE equipment_calibrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    calibrated_at DATE NOT NULL,
    next_due_at DATE,
    result VARCHAR(50),
    notes TEXT,
    calibration_type calibration_type NOT NULL DEFAULT 'calibration',
    partner_id UUID REFERENCES partners(id),
    report_number VARCHAR(100),
    inspector VARCHAR(100),
    equipment_serial_number VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_calibrations_equipment ON equipment_calibrations(equipment_id);
CREATE INDEX idx_equipment_calibrations_type ON equipment_calibrations(calibration_type);
CREATE INDEX idx_equipment_calibrations_partner_id ON equipment_calibrations(partner_id);

-- ─── 8.10 設備-廠商關聯表 (from 018) ────────────────────────────────

CREATE TABLE equipment_suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    partner_id UUID NOT NULL REFERENCES partners(id) ON DELETE CASCADE,
    contact_person VARCHAR(100),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (equipment_id, partner_id)
);
CREATE INDEX idx_equipment_suppliers_equipment ON equipment_suppliers(equipment_id);
CREATE INDEX idx_equipment_suppliers_partner ON equipment_suppliers(partner_id);

-- ─── 8.11 設備狀態變更紀錄 (from 018) ───────────────────────────────

CREATE TABLE equipment_status_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    old_status equipment_status NOT NULL,
    new_status equipment_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_status_logs_equipment ON equipment_status_logs(equipment_id);
CREATE INDEX idx_equipment_status_logs_created ON equipment_status_logs(created_at DESC);

-- ─── 8.12 維修/保養紀錄 (from 018) ──────────────────────────────────

CREATE TABLE equipment_maintenance_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    maintenance_type maintenance_type NOT NULL,
    status maintenance_status NOT NULL DEFAULT 'pending',
    reported_at DATE NOT NULL,
    completed_at DATE,
    problem_description TEXT,
    repair_content TEXT,
    repair_partner_id UUID REFERENCES partners(id),
    maintenance_items TEXT,
    performed_by VARCHAR(100),
    notes TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_maintenance_equipment ON equipment_maintenance_records(equipment_id);
CREATE INDEX idx_equipment_maintenance_type ON equipment_maintenance_records(maintenance_type);
CREATE INDEX idx_equipment_maintenance_status ON equipment_maintenance_records(status);
CREATE INDEX idx_equipment_maintenance_repair_partner ON equipment_maintenance_records(repair_partner_id);

-- ─── 8.13 報廢紀錄 (from 018) ───────────────────────────────────────

CREATE TABLE equipment_disposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    status disposal_status NOT NULL DEFAULT 'pending',
    disposal_date DATE,
    reason TEXT NOT NULL,
    disposal_method TEXT,
    applied_by UUID NOT NULL REFERENCES users(id),
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    rejection_reason TEXT,
    applicant_signature_id UUID REFERENCES electronic_signatures(id),
    approver_signature_id UUID REFERENCES electronic_signatures(id),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_equipment_disposals_equipment ON equipment_disposals(equipment_id);
CREATE INDEX idx_equipment_disposals_status ON equipment_disposals(status);

-- ─── 8.14 年度維護校正計畫表 (from 018) ─────────────────────────────

CREATE TABLE equipment_annual_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    year INTEGER NOT NULL,
    equipment_id UUID NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    calibration_type calibration_type NOT NULL,
    cycle calibration_cycle NOT NULL,
    month_1 BOOLEAN NOT NULL DEFAULT false,
    month_2 BOOLEAN NOT NULL DEFAULT false,
    month_3 BOOLEAN NOT NULL DEFAULT false,
    month_4 BOOLEAN NOT NULL DEFAULT false,
    month_5 BOOLEAN NOT NULL DEFAULT false,
    month_6 BOOLEAN NOT NULL DEFAULT false,
    month_7 BOOLEAN NOT NULL DEFAULT false,
    month_8 BOOLEAN NOT NULL DEFAULT false,
    month_9 BOOLEAN NOT NULL DEFAULT false,
    month_10 BOOLEAN NOT NULL DEFAULT false,
    month_11 BOOLEAN NOT NULL DEFAULT false,
    month_12 BOOLEAN NOT NULL DEFAULT false,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (year, equipment_id, calibration_type)
);
CREATE INDEX idx_equipment_annual_plans_year ON equipment_annual_plans(year);
CREATE INDEX idx_equipment_annual_plans_equipment ON equipment_annual_plans(equipment_id);

-- ─── 8.15 Animals 表 FK 約束 (from 019) ─────────────────────────────
-- animals 在 003/004 定義，pen_id/species_id 欄位已存在，此處加上 FK

ALTER TABLE animals ADD CONSTRAINT animals_pen_id_fkey FOREIGN KEY (pen_id) REFERENCES pens(id);
ALTER TABLE animals ADD CONSTRAINT animals_species_id_fkey FOREIGN KEY (species_id) REFERENCES species(id);
-- 索引已在 003_animal_management.sql 中建立

-- ─── 8.16 Seed 物種資料 (from 019) ──────────────────────────────────

-- 大類：豬
INSERT INTO species (id, code, name, name_en, sort_order)
VALUES ('a0000000-0000-0000-0000-000000000001'::uuid, 'pig', '豬', 'Pig', 1)
ON CONFLICT (code) DO NOTHING;

-- 品種：迷你豬（對應 animal_breed 的 miniature）
INSERT INTO species (id, code, name, name_en, parent_id, sort_order)
VALUES ('a0000000-0000-0000-0000-000000000002'::uuid, 'miniature', '迷你豬', 'Minipig',
        'a0000000-0000-0000-0000-000000000001'::uuid, 1)
ON CONFLICT (code) DO NOTHING;

-- 品種：白豬（對應 animal_breed 的 white + LYD）
INSERT INTO species (id, code, name, name_en, parent_id, sort_order)
VALUES ('a0000000-0000-0000-0000-000000000003'::uuid, 'white', '白豬', 'White Pig',
        'a0000000-0000-0000-0000-000000000001'::uuid, 2)
ON CONFLICT (code) DO NOTHING;

-- 大類：其他
INSERT INTO species (id, code, name, name_en, sort_order)
VALUES ('a0000000-0000-0000-0000-000000000004'::uuid, 'other', '其他', 'Other', 99)
ON CONFLICT (code) DO NOTHING;

-- ─── 8.17 Seed 設施資料 (from 019) ──────────────────────────────────

-- 設施：豬博士畜牧場
INSERT INTO facilities (id, code, name)
VALUES ('b0000000-0000-0000-0000-000000000001'::uuid, 'PIGMODEL', '豬博士畜牧場')
ON CONFLICT DO NOTHING;

-- 棟舍：A 棟
INSERT INTO buildings (id, facility_id, code, name, sort_order)
VALUES ('c0000000-0000-0000-0000-000000000001'::uuid,
        'b0000000-0000-0000-0000-000000000001'::uuid, 'A', 'A 棟', 1)
ON CONFLICT DO NOTHING;

-- 棟舍：B 棟
INSERT INTO buildings (id, facility_id, code, name, sort_order)
VALUES ('c0000000-0000-0000-0000-000000000002'::uuid,
        'b0000000-0000-0000-0000-000000000001'::uuid, 'B', 'B 棟', 2)
ON CONFLICT DO NOTHING;

-- ─── 8.18 Seed 區域資料 (from 019) ──────────────────────────────────

-- A 棟區域
INSERT INTO zones (id, building_id, code, name, color, sort_order) VALUES
('d0000000-0000-0000-0000-000000000001'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'A', 'A 區', 'blue', 1),
('d0000000-0000-0000-0000-000000000002'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'C', 'C 區', 'yellow', 2),
('d0000000-0000-0000-0000-000000000003'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'D', 'D 區', 'cyan', 3)
ON CONFLICT DO NOTHING;

-- B 棟區域
INSERT INTO zones (id, building_id, code, name, color, sort_order, layout_config) VALUES
('d0000000-0000-0000-0000-000000000004'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'B', 'B 區', 'orange', 1, NULL),
('d0000000-0000-0000-0000-000000000005'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'E', 'E 區', 'purple', 2,
 '{"display_group": "EFG", "group_position": "left"}'::jsonb),
('d0000000-0000-0000-0000-000000000006'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'F', 'F 區', 'amber', 3,
 '{"display_group": "EFG", "group_position": "right", "group_order": 1}'::jsonb),
('d0000000-0000-0000-0000-000000000007'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'G', 'G 區', 'green', 4,
 '{"display_group": "EFG", "group_position": "right", "group_order": 2}'::jsonb)
ON CONFLICT DO NOTHING;

-- ─── 8.19 Seed 欄位資料 (from 019, generate_series 批次建立) ────────

-- A 區 20 欄：A01~A20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000001'::uuid,
    'A' || LPAD(n::text, 2, '0'), 'A' || LPAD(n::text, 2, '0'), 1,
    CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
    CASE WHEN n <= 10 THEN 0 ELSE 1 END
FROM generate_series(1, 20) AS n
ON CONFLICT DO NOTHING;

-- C 區 20 欄：C01~C20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000002'::uuid,
    'C' || LPAD(n::text, 2, '0'), 'C' || LPAD(n::text, 2, '0'), 1,
    CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
    CASE WHEN n <= 10 THEN 0 ELSE 1 END
FROM generate_series(1, 20) AS n
ON CONFLICT DO NOTHING;

-- D 區 33 欄：D01~D33
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000003'::uuid,
    'D' || LPAD(n::text, 2, '0'), 'D' || LPAD(n::text, 2, '0'), 1,
    CASE WHEN n <= 17 THEN n - 1 ELSE n - 18 END,
    CASE WHEN n <= 17 THEN 0 ELSE 1 END
FROM generate_series(1, 33) AS n
ON CONFLICT DO NOTHING;

-- B 區 20 欄：B01~B20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000004'::uuid,
    'B' || LPAD(n::text, 2, '0'), 'B' || LPAD(n::text, 2, '0'), 1,
    CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
    CASE WHEN n <= 10 THEN 0 ELSE 1 END
FROM generate_series(1, 20) AS n
ON CONFLICT DO NOTHING;

-- E 區 25 欄：E01~E25
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000005'::uuid,
    'E' || LPAD(n::text, 2, '0'), 'E' || LPAD(n::text, 2, '0'), 1, n - 1, 0
FROM generate_series(1, 25) AS n
ON CONFLICT DO NOTHING;

-- F 區 6 欄：F01~F06
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000006'::uuid,
    'F' || LPAD(n::text, 2, '0'), 'F' || LPAD(n::text, 2, '0'), 1, n - 1, 0
FROM generate_series(1, 6) AS n
ON CONFLICT DO NOTHING;

-- G 區 6 欄：G01~G06
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(), 'd0000000-0000-0000-0000-000000000007'::uuid,
    'G' || LPAD(n::text, 2, '0'), 'G' || LPAD(n::text, 2, '0'), 1, n - 1, 0
FROM generate_series(1, 6) AS n
ON CONFLICT DO NOTHING;

-- ─── 8.20 Seed 部門資料 (from 019) ──────────────────────────────────

INSERT INTO departments (id, code, name, sort_order) VALUES
('e0000000-0000-0000-0000-000000000001'::uuid, 'EXPERIMENT', '試驗部', 1),
('e0000000-0000-0000-0000-000000000002'::uuid, 'ADMIN', '行政部', 2)
ON CONFLICT (code) DO NOTHING;

-- ─── 8.21 從現有 pen_location 對應到 pen_id (from 019) ──────────────

UPDATE animals
SET pen_id = pens.id
FROM pens
WHERE pens.code = animals.pen_location
  AND animals.pen_location IS NOT NULL
  AND animals.pen_location != ''
  AND animals.pen_id IS NULL;

-- ─── 8.22 從現有 breed 對應到 species_id (from 019) ─────────────────

-- miniature → 迷你豬
UPDATE animals
SET species_id = 'a0000000-0000-0000-0000-000000000002'::uuid
WHERE breed::text = 'miniature'
  AND species_id IS NULL;

-- white → 白豬
UPDATE animals
SET species_id = 'a0000000-0000-0000-0000-000000000003'::uuid
WHERE breed::text = 'white'
  AND species_id IS NULL;

-- LYD → 白豬（LYD 等同白豬）
UPDATE animals
SET species_id = 'a0000000-0000-0000-0000-000000000003'::uuid
WHERE breed::text = 'LYD'
  AND species_id IS NULL;

-- other → 其他
UPDATE animals
SET species_id = 'a0000000-0000-0000-0000-000000000004'::uuid
WHERE breed::text = 'other'
  AND species_id IS NULL;

-- ─── 8.23 同步 pens.current_count (from 019) ───────────────────────

UPDATE pens
SET current_count = sub.cnt
FROM (
    SELECT pen_id, COUNT(*) AS cnt
    FROM animals
    WHERE pen_id IS NOT NULL
      AND is_deleted = false
      AND status NOT IN ('euthanized', 'sudden_death', 'transferred')
    GROUP BY pen_id
) sub
WHERE pens.id = sub.pen_id;

-- 通知類型 enum values 已在 001_types.sql 中定義
-- 權限、角色、通知路由 seed 已在 002_users_auth.sql 中建立
