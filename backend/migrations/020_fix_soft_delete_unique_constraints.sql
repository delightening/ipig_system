-- 修復軟刪除與唯一約束衝突問題
-- 問題：軟刪除的記錄（is_active = false）仍受 UNIQUE 約束限制，
--       導致無法新增與已停用記錄相同代碼的新記錄。
-- 方案：將 UNIQUE 約束改為 partial unique index，只限制 active 記錄。

-- ============================================
-- buildings: UNIQUE (facility_id, code) → partial
-- ============================================
ALTER TABLE buildings DROP CONSTRAINT IF EXISTS buildings_facility_id_code_key;
CREATE UNIQUE INDEX IF NOT EXISTS uq_buildings_facility_code_active
    ON buildings (facility_id, code) WHERE is_active = true;

-- ============================================
-- zones: UNIQUE (building_id, code) → partial
-- ============================================
ALTER TABLE zones DROP CONSTRAINT IF EXISTS zones_building_id_code_key;
CREATE UNIQUE INDEX IF NOT EXISTS uq_zones_building_code_active
    ON zones (building_id, code) WHERE is_active = true;

-- ============================================
-- pens: UNIQUE (zone_id, code) → partial
-- ============================================
ALTER TABLE pens DROP CONSTRAINT IF EXISTS pens_zone_id_code_key;
CREATE UNIQUE INDEX IF NOT EXISTS uq_pens_zone_code_active
    ON pens (zone_id, code) WHERE is_active = true;

-- ============================================
-- facilities: UNIQUE (code) → partial (if soft-deletable)
-- ============================================
ALTER TABLE facilities DROP CONSTRAINT IF EXISTS facilities_code_key;
CREATE UNIQUE INDEX IF NOT EXISTS uq_facilities_code_active
    ON facilities (code) WHERE is_active = true;
