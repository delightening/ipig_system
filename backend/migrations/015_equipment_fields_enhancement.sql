-- Migration 015: 設備欄位補強（ISO 17025 / GLP 合規）

-- ============================================================
-- 1. 設備基本資訊補充（資產管理必要欄位）
-- ============================================================
ALTER TABLE equipment
    ADD COLUMN IF NOT EXISTS purchase_date    DATE,
    ADD COLUMN IF NOT EXISTS warranty_expiry  DATE,
    ADD COLUMN IF NOT EXISTS department       VARCHAR(100);

-- ============================================================
-- 2. 校正記錄補充（ISO 17025:2017 合規必要欄位）
-- ============================================================
ALTER TABLE equipment_calibrations
    ADD COLUMN IF NOT EXISTS certificate_number      VARCHAR(100),
    ADD COLUMN IF NOT EXISTS performed_by            VARCHAR(100),
    ADD COLUMN IF NOT EXISTS acceptance_criteria     VARCHAR(200),
    ADD COLUMN IF NOT EXISTS measurement_uncertainty VARCHAR(100);

-- ============================================================
-- 3. 確效類型（GMP Annex 15 / OECD GLP 必要欄位）
--    IQ = 安裝確效, OQ = 作業確效, PQ = 效能確效
-- ============================================================
DO $$
BEGIN
    CREATE TYPE validation_phase AS ENUM ('IQ', 'OQ', 'PQ');
EXCEPTION WHEN duplicate_object THEN NULL;
END
$$;

ALTER TABLE equipment_calibrations
    ADD COLUMN IF NOT EXISTS validation_phase  validation_phase,
    ADD COLUMN IF NOT EXISTS protocol_number   VARCHAR(100);
