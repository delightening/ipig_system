-- ============================================
-- Migration 019: 設施管理與動物模組串接
-- 1. species 加 parent_id 支援分層
-- 2. seed 設施基礎資料（設施/棟舍/區域/欄位/物種/部門）
-- 3. animals 表新增 pen_id / species_id FK
-- 4. 從現有 pen_location / breed 自動對應
-- ============================================

-- ─── 1. Species 加 parent_id 支援分層 ───────────────────────────────

ALTER TABLE species ADD COLUMN IF NOT EXISTS parent_id UUID REFERENCES species(id);

-- ─── 2. Seed 物種資料 ──────────────────────────────────────────────

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

-- ─── 3. Seed 設施資料 ──────────────────────────────────────────────

-- 設施：豬博士畜牧場
INSERT INTO facilities (id, code, name)
VALUES ('b0000000-0000-0000-0000-000000000001'::uuid, 'PIGMODEL', '豬博士畜牧場')
ON CONFLICT (code) DO NOTHING;

-- 棟舍：A 棟
INSERT INTO buildings (id, facility_id, code, name, sort_order)
VALUES ('c0000000-0000-0000-0000-000000000001'::uuid,
        'b0000000-0000-0000-0000-000000000001'::uuid, 'A', 'A 棟', 1)
ON CONFLICT (facility_id, code) DO NOTHING;

-- 棟舍：B 棟
INSERT INTO buildings (id, facility_id, code, name, sort_order)
VALUES ('c0000000-0000-0000-0000-000000000002'::uuid,
        'b0000000-0000-0000-0000-000000000001'::uuid, 'B', 'B 棟', 2)
ON CONFLICT (facility_id, code) DO NOTHING;

-- ─── 4. Seed 區域資料 ──────────────────────────────────────────────

-- A 棟的區域
INSERT INTO zones (id, building_id, code, name, color, sort_order) VALUES
('d0000000-0000-0000-0000-000000000001'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'A', 'A 區', 'blue', 1),
('d0000000-0000-0000-0000-000000000002'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'C', 'C 區', 'yellow', 2),
('d0000000-0000-0000-0000-000000000003'::uuid, 'c0000000-0000-0000-0000-000000000001'::uuid, 'D', 'D 區', 'cyan', 3)
ON CONFLICT (building_id, code) DO NOTHING;

-- B 棟的區域
INSERT INTO zones (id, building_id, code, name, color, sort_order, layout_config) VALUES
('d0000000-0000-0000-0000-000000000004'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'B', 'B 區', 'orange', 1, NULL),
('d0000000-0000-0000-0000-000000000005'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'E', 'E 區', 'purple', 2,
 '{"display_group": "EFG", "group_position": "left"}'::jsonb),
('d0000000-0000-0000-0000-000000000006'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'F', 'F 區', 'amber', 3,
 '{"display_group": "EFG", "group_position": "right", "group_order": 1}'::jsonb),
('d0000000-0000-0000-0000-000000000007'::uuid, 'c0000000-0000-0000-0000-000000000002'::uuid, 'G', 'G 區', 'green', 4,
 '{"display_group": "EFG", "group_position": "right", "group_order": 2}'::jsonb)
ON CONFLICT (building_id, code) DO NOTHING;

-- ─── 5. Seed 欄位資料（使用 generate_series 批次建立）──────────────

-- A 區 20 欄：A01~A20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000001'::uuid,
    'A' || LPAD(n::text, 2, '0'),
    'A' || LPAD(n::text, 2, '0'),
    1,
    CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
    CASE WHEN n <= 10 THEN 0 ELSE 1 END
FROM generate_series(1, 20) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- C 區 20 欄：C01~C20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000002'::uuid,
    'C' || LPAD(n::text, 2, '0'),
    'C' || LPAD(n::text, 2, '0'),
    1,
    CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
    CASE WHEN n <= 10 THEN 0 ELSE 1 END
FROM generate_series(1, 20) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- D 區 33 欄：D01~D33
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000003'::uuid,
    'D' || LPAD(n::text, 2, '0'),
    'D' || LPAD(n::text, 2, '0'),
    1,
    CASE WHEN n <= 17 THEN n - 1 ELSE n - 18 END,
    CASE WHEN n <= 17 THEN 0 ELSE 1 END
FROM generate_series(1, 33) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- B 區 20 欄：B01~B20
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000004'::uuid,
    'B' || LPAD(n::text, 2, '0'),
    'B' || LPAD(n::text, 2, '0'),
    1,
    CASE WHEN n <= 10 THEN n - 1 ELSE n - 11 END,
    CASE WHEN n <= 10 THEN 0 ELSE 1 END
FROM generate_series(1, 20) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- E 區 25 欄：E01~E25
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000005'::uuid,
    'E' || LPAD(n::text, 2, '0'),
    'E' || LPAD(n::text, 2, '0'),
    1,
    n - 1,
    0
FROM generate_series(1, 25) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- F 區 6 欄：F01~F06
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000006'::uuid,
    'F' || LPAD(n::text, 2, '0'),
    'F' || LPAD(n::text, 2, '0'),
    1,
    n - 1,
    0
FROM generate_series(1, 6) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- G 區 6 欄：G01~G06
INSERT INTO pens (id, zone_id, code, name, capacity, row_index, col_index)
SELECT
    gen_random_uuid(),
    'd0000000-0000-0000-0000-000000000007'::uuid,
    'G' || LPAD(n::text, 2, '0'),
    'G' || LPAD(n::text, 2, '0'),
    1,
    n - 1,
    0
FROM generate_series(1, 6) AS n
ON CONFLICT (zone_id, code) DO NOTHING;

-- ─── 6. Seed 部門資料 ──────────────────────────────────────────────

INSERT INTO departments (id, code, name, sort_order) VALUES
('e0000000-0000-0000-0000-000000000001'::uuid, 'EXPERIMENT', '試驗部', 1),
('e0000000-0000-0000-0000-000000000002'::uuid, 'ADMIN', '行政部', 2)
ON CONFLICT (code) DO NOTHING;

-- ─── 7. Animals 表新增 pen_id / species_id ─────────────────────────

ALTER TABLE animals ADD COLUMN IF NOT EXISTS pen_id UUID REFERENCES pens(id);
ALTER TABLE animals ADD COLUMN IF NOT EXISTS species_id UUID REFERENCES species(id);

-- 建立索引
CREATE INDEX IF NOT EXISTS idx_animals_pen_id ON animals(pen_id);
CREATE INDEX IF NOT EXISTS idx_animals_species_id ON animals(species_id);

-- ─── 8. 從現有 pen_location 對應到 pen_id ──────────────────────────

UPDATE animals
SET pen_id = pens.id
FROM pens
WHERE pens.code = animals.pen_location
  AND animals.pen_location IS NOT NULL
  AND animals.pen_location != ''
  AND animals.pen_id IS NULL;

-- ─── 9. 從現有 breed 對應到 species_id ─────────────────────────────

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

-- ─── 10. 同步 pens.current_count ───────────────────────────────────

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
