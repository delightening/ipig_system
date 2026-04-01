-- ============================================================
-- Migration 005: 設施層級結構（物種、設施、棟舍、區域、欄位、部門）
-- 來源: 010_treatment_drug_final.sql (CREATE TABLE 部分),
--       019_facility_animal_integration.sql (parent_id, seed data),
--       020_fix_soft_delete_unique_constraints.sql (partial unique index),
--       022_add_missing_fk_indexes.sql (FK indexes)
-- ============================================================

-- ── species ──────────────────────────────────────────────────
-- parent_id 支援分層（019 新增，直接合入）
CREATE TABLE species (
    id         UUID        PRIMARY KEY,
    code       VARCHAR(50) NOT NULL UNIQUE,
    name       VARCHAR(100) NOT NULL,
    name_en    VARCHAR(100),
    icon       VARCHAR(100),
    is_active  BOOLEAN     NOT NULL DEFAULT true,
    config     JSONB,
    sort_order INTEGER     NOT NULL DEFAULT 0,
    parent_id  UUID        REFERENCES species(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_species_parent_id ON species(parent_id);

-- ── facilities ───────────────────────────────────────────────
-- 020: 軟刪除後可重新使用相同 code，改為 partial unique index
CREATE TABLE facilities (
    id             UUID        PRIMARY KEY,
    code           VARCHAR(50) NOT NULL,
    name           VARCHAR(200) NOT NULL,
    address        TEXT,
    phone          VARCHAR(50),
    contact_person VARCHAR(100),
    is_active      BOOLEAN     NOT NULL DEFAULT true,
    config         JSONB,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_facilities_code_active ON facilities(code) WHERE is_active = true;

-- ── buildings ────────────────────────────────────────────────
CREATE TABLE buildings (
    id          UUID        PRIMARY KEY,
    facility_id UUID        NOT NULL REFERENCES facilities(id),
    code        VARCHAR(50) NOT NULL,
    name        VARCHAR(200) NOT NULL,
    description TEXT,
    is_active   BOOLEAN     NOT NULL DEFAULT true,
    config      JSONB,
    sort_order  INTEGER     NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_buildings_facility_code_active ON buildings(facility_id, code) WHERE is_active = true;
CREATE INDEX idx_buildings_facility_id ON buildings(facility_id);

-- ── zones ────────────────────────────────────────────────────
CREATE TABLE zones (
    id            UUID        PRIMARY KEY,
    building_id   UUID        NOT NULL REFERENCES buildings(id),
    code          VARCHAR(50) NOT NULL,
    name          VARCHAR(200),
    color         VARCHAR(20),
    is_active     BOOLEAN     NOT NULL DEFAULT true,
    layout_config JSONB,
    sort_order    INTEGER     NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_zones_building_code_active ON zones(building_id, code) WHERE is_active = true;
CREATE INDEX idx_zones_building_id ON zones(building_id);

-- ── pens ─────────────────────────────────────────────────────
CREATE TABLE pens (
    id            UUID        PRIMARY KEY,
    zone_id       UUID        NOT NULL REFERENCES zones(id),
    code          VARCHAR(50) NOT NULL,
    name          VARCHAR(200),
    capacity      INTEGER     NOT NULL DEFAULT 1,
    current_count INTEGER     NOT NULL DEFAULT 0,
    status        VARCHAR(50) NOT NULL DEFAULT 'active',
    row_index     INTEGER,
    col_index     INTEGER,
    is_active     BOOLEAN     NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX uq_pens_zone_code_active ON pens(zone_id, code) WHERE is_active = true;
CREATE INDEX idx_pens_zone_id ON pens(zone_id);

-- ── departments ──────────────────────────────────────────────
CREATE TABLE departments (
    id         UUID        PRIMARY KEY,
    code       VARCHAR(50) NOT NULL UNIQUE,
    name       VARCHAR(200) NOT NULL,
    parent_id  UUID        REFERENCES departments(id),
    manager_id UUID        REFERENCES users(id),
    is_active  BOOLEAN     NOT NULL DEFAULT true,
    config     JSONB,
    sort_order INTEGER     NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_departments_parent_id ON departments(parent_id);

-- ============================================================
-- Seed: 物種、設施、棟舍、區域、欄位、部門
-- 來源: 019_facility_animal_integration.sql
-- ============================================================

-- 物種
INSERT INTO species (id, code, name, name_en, sort_order) VALUES
    ('a0000000-0000-0000-0000-000000000001'::uuid, 'pig',       '豬',   'Pig',       1),
    ('a0000000-0000-0000-0000-000000000004'::uuid, 'other',     '其他', 'Other',     99)
ON CONFLICT (code) DO NOTHING;

INSERT INTO species (id, code, name, name_en, parent_id, sort_order) VALUES
    ('a0000000-0000-0000-0000-000000000002'::uuid, 'miniature', '迷你豬', 'Minipig',
     'a0000000-0000-0000-0000-000000000001'::uuid, 1),
    ('a0000000-0000-0000-0000-000000000003'::uuid, 'white',     '白豬',   'White Pig',
     'a0000000-0000-0000-0000-000000000001'::uuid, 2)
ON CONFLICT (code) DO NOTHING;

-- 設施
INSERT INTO facilities (id, code, name) VALUES
    ('b0000000-0000-0000-0000-000000000001'::uuid, 'PIGMODEL', '豬博士畜牧場')
ON CONFLICT DO NOTHING;

-- 棟舍
INSERT INTO buildings (id, facility_id, code, name, sort_order) VALUES
    ('c0000000-0000-0000-0000-000000000001'::uuid,
     'b0000000-0000-0000-0000-000000000001'::uuid, 'A', 'A 棟', 1),
    ('c0000000-0000-0000-0000-000000000002'::uuid,
     'b0000000-0000-0000-0000-000000000001'::uuid, 'B', 'B 棟', 2)
ON CONFLICT DO NOTHING;

-- 區域
INSERT INTO zones (id, building_id, code, name, color, sort_order) VALUES
    ('d0000000-0000-0000-0000-000000000001'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'A', 'A 區', 'blue',   1),
    ('d0000000-0000-0000-0000-000000000002'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'C', 'C 區', 'yellow', 2),
    ('d0000000-0000-0000-0000-000000000003'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'D', 'D 區', 'cyan',   3),
    ('d0000000-0000-0000-0000-000000000004'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'B', 'B 區', 'orange', 1)
ON CONFLICT DO NOTHING;

INSERT INTO zones (id, building_id, code, name, color, sort_order, layout_config) VALUES
    ('d0000000-0000-0000-0000-000000000005'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'E', 'E 區', 'purple', 2,
     '{"display_group": "EFG", "group_position": "left"}'::jsonb),
    ('d0000000-0000-0000-0000-000000000006'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'F', 'F 區', 'amber',  3,
     '{"display_group": "EFG", "group_position": "right", "group_order": 1}'::jsonb),
    ('d0000000-0000-0000-0000-000000000007'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'G', 'G 區', 'green',  4,
     '{"display_group": "EFG", "group_position": "right", "group_order": 2}'::jsonb)
ON CONFLICT DO NOTHING;

-- 欄位（批次建立）
-- A 區 20 欄：A01~A20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000001'::uuid,
       'A' || LPAD(n::text, 2, '0'), 'A' || LPAD(n::text, 2, '0'), 1,
       CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
       CASE WHEN n <= 10 THEN 0     ELSE 1       END
FROM generate_series(1, 20) n
ON CONFLICT DO NOTHING;

-- C 區 20 欄：C01~C20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000002'::uuid,
       'C' || LPAD(n::text, 2, '0'), 'C' || LPAD(n::text, 2, '0'), 1,
       CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
       CASE WHEN n <= 10 THEN 0     ELSE 1       END
FROM generate_series(1, 20) n
ON CONFLICT DO NOTHING;

-- D 區 33 欄：D01~D33
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000003'::uuid,
       'D' || LPAD(n::text, 2, '0'), 'D' || LPAD(n::text, 2, '0'), 1,
       CASE WHEN n <= 17 THEN n - 1 ELSE n - 18 END,
       CASE WHEN n <= 17 THEN 0     ELSE 1       END
FROM generate_series(1, 33) n
ON CONFLICT DO NOTHING;

-- B 區 20 欄：B01~B20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000004'::uuid,
       'B' || LPAD(n::text, 2, '0'), 'B' || LPAD(n::text, 2, '0'), 1,
       CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
       CASE WHEN n <= 10 THEN 0     ELSE 1       END
FROM generate_series(1, 20) n
ON CONFLICT DO NOTHING;

-- E 區 25 欄：E01~E25
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000005'::uuid,
       'E' || LPAD(n::text, 2, '0'), 'E' || LPAD(n::text, 2, '0'), 1,
       n - 1, 0
FROM generate_series(1, 25) n
ON CONFLICT DO NOTHING;

-- F 區 6 欄：F01~F06
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000006'::uuid,
       'F' || LPAD(n::text, 2, '0'), 'F' || LPAD(n::text, 2, '0'), 1,
       n - 1, 0
FROM generate_series(1, 6) n
ON CONFLICT DO NOTHING;

-- G 區 6 欄：G01~G06
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT gen_random_uuid(),
       'd0000000-0000-0000-0000-000000000007'::uuid,
       'G' || LPAD(n::text, 2, '0'), 'G' || LPAD(n::text, 2, '0'), 1,
       n - 1, 0
FROM generate_series(1, 6) n
ON CONFLICT DO NOTHING;

-- 部門
INSERT INTO departments (id, code, name, sort_order) VALUES
    ('e0000000-0000-0000-0000-000000000001'::uuid, 'EXPERIMENT', '試驗部', 1),
    ('e0000000-0000-0000-0000-000000000002'::uuid, 'ADMIN',      '行政部', 2)
ON CONFLICT (code) DO NOTHING;
