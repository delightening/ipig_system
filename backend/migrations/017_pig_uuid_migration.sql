-- Migration: 017_pig_uuid_migration.sql
-- Purpose: Migrate pig module from INTEGER to UUID primary keys
-- Created: 2026-02-02
-- 
-- This migration:
-- 1. Adds UUID as the new primary key for pigs table
-- 2. Adds pig_no (SERIAL) for user-friendly display
-- 3. Updates all related tables to use UUID for pig references

-- ============================================
-- STEP 1: Add new columns to pigs table
-- ============================================

-- Add uuid column (will become the new primary key)
ALTER TABLE pigs ADD COLUMN uuid UUID DEFAULT gen_random_uuid();

-- Ensure all existing rows have UUIDs
UPDATE pigs SET uuid = gen_random_uuid() WHERE uuid IS NULL;

-- Make uuid NOT NULL
ALTER TABLE pigs ALTER COLUMN uuid SET NOT NULL;

-- Add pig_no column for display purposes (auto-increment)
-- We'll use a sequence to continue from the max existing id
DO $$
DECLARE
    max_id INTEGER;
BEGIN
    SELECT COALESCE(MAX(id), 0) INTO max_id FROM pigs;
    EXECUTE format('CREATE SEQUENCE IF NOT EXISTS pig_no_seq START WITH %s', max_id + 1);
END $$;

ALTER TABLE pigs ADD COLUMN pig_no INTEGER;

-- Copy existing id values to pig_no
UPDATE pigs SET pig_no = id;

-- Set pig_no to use the sequence for new records
ALTER TABLE pigs ALTER COLUMN pig_no SET DEFAULT nextval('pig_no_seq');
ALTER TABLE pigs ALTER COLUMN pig_no SET NOT NULL;

-- Create unique index on pig_no
CREATE UNIQUE INDEX idx_pigs_pig_no ON pigs(pig_no);

-- ============================================
-- STEP 2: Add pig_uuid columns to related tables
-- ============================================

-- pig_observations
ALTER TABLE pig_observations ADD COLUMN pig_uuid UUID;

-- pig_surgeries
ALTER TABLE pig_surgeries ADD COLUMN pig_uuid UUID;

-- pig_weights
ALTER TABLE pig_weights ADD COLUMN pig_uuid UUID;

-- pig_vaccinations
ALTER TABLE pig_vaccinations ADD COLUMN pig_uuid UUID;

-- pig_sacrifices
ALTER TABLE pig_sacrifices ADD COLUMN pig_uuid UUID;

-- pig_pathology_reports
ALTER TABLE pig_pathology_reports ADD COLUMN pig_uuid UUID;

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'pig_export_records') THEN
        EXECUTE 'ALTER TABLE pig_export_records ADD COLUMN pig_uuid UUID';
    END IF;
END $$;

-- ============================================
-- STEP 3: Migrate foreign key data
-- ============================================

UPDATE pig_observations po 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = po.pig_id);

UPDATE pig_surgeries ps 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = ps.pig_id);

UPDATE pig_weights pw 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pw.pig_id);

UPDATE pig_vaccinations pv 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pv.pig_id);

UPDATE pig_sacrifices ps 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = ps.pig_id);

UPDATE pig_pathology_reports pp 
SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pp.pig_id);

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'pig_export_records' AND column_name = 'pig_uuid') THEN
        EXECUTE 'UPDATE pig_export_records pe 
                 SET pig_uuid = (SELECT uuid FROM pigs WHERE id = pe.pig_id)
                 WHERE pig_id IS NOT NULL';
    END IF;
END $$;

-- ============================================
-- STEP 4: Make pig_uuid NOT NULL where appropriate
-- ============================================

ALTER TABLE pig_observations ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_surgeries ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_weights ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_vaccinations ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_sacrifices ALTER COLUMN pig_uuid SET NOT NULL;
ALTER TABLE pig_pathology_reports ALTER COLUMN pig_uuid SET NOT NULL;

-- ============================================
-- STEP 5: Drop old foreign key constraints
-- ============================================

ALTER TABLE pig_observations DROP CONSTRAINT IF EXISTS pig_observations_pig_id_fkey;
ALTER TABLE pig_surgeries DROP CONSTRAINT IF EXISTS pig_surgeries_pig_id_fkey;
ALTER TABLE pig_weights DROP CONSTRAINT IF EXISTS pig_weights_pig_id_fkey;
ALTER TABLE pig_vaccinations DROP CONSTRAINT IF EXISTS pig_vaccinations_pig_id_fkey;
ALTER TABLE pig_sacrifices DROP CONSTRAINT IF EXISTS pig_sacrifices_pig_id_fkey;
ALTER TABLE pig_pathology_reports DROP CONSTRAINT IF EXISTS pig_pathology_reports_pig_id_fkey;

-- Drop unique constraint on pig_sacrifices if exists
ALTER TABLE pig_sacrifices DROP CONSTRAINT IF EXISTS pig_sacrifices_pig_id_key;
ALTER TABLE pig_pathology_reports DROP CONSTRAINT IF EXISTS pig_pathology_reports_pig_id_key;

-- ============================================
-- STEP 6: Drop old indexes that reference pig_id
-- ============================================

DROP INDEX IF EXISTS idx_pig_observations_pig_id;
DROP INDEX IF EXISTS idx_pig_surgeries_pig_id;
DROP INDEX IF EXISTS idx_pig_weights_pig_id;
DROP INDEX IF EXISTS idx_pig_vaccinations_pig_id;
DROP INDEX IF EXISTS idx_pig_sacrifices_pig_id;

-- ============================================
-- STEP 7: Drop old pig_id columns from related tables
-- ============================================

ALTER TABLE pig_observations DROP COLUMN pig_id;
ALTER TABLE pig_surgeries DROP COLUMN pig_id;
ALTER TABLE pig_weights DROP COLUMN pig_id;
ALTER TABLE pig_vaccinations DROP COLUMN pig_id;
ALTER TABLE pig_sacrifices DROP COLUMN pig_id;
ALTER TABLE pig_pathology_reports DROP COLUMN pig_id;

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'pig_export_records' AND column_name = 'pig_id') THEN
        EXECUTE 'ALTER TABLE pig_export_records DROP COLUMN pig_id';
    END IF;
END $$;

-- ============================================
-- STEP 8: Rename pig_uuid to pig_id
-- ============================================

ALTER TABLE pig_observations RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_surgeries RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_weights RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_vaccinations RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_sacrifices RENAME COLUMN pig_uuid TO pig_id;
ALTER TABLE pig_pathology_reports RENAME COLUMN pig_uuid TO pig_id;

-- pig_export_records (if exists)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'pig_export_records' AND column_name = 'pig_uuid') THEN
        EXECUTE 'ALTER TABLE pig_export_records RENAME COLUMN pig_uuid TO pig_id';
    END IF;
END $$;

-- ============================================
-- STEP 9: Update pigs table primary key
-- ============================================

-- Drop old primary key
ALTER TABLE pigs DROP CONSTRAINT pigs_pkey;

-- Drop old id column
ALTER TABLE pigs DROP COLUMN id;

-- Rename uuid to id
ALTER TABLE pigs RENAME COLUMN uuid TO id;

-- Add new primary key
ALTER TABLE pigs ADD PRIMARY KEY (id);

-- ============================================
-- STEP 10: Add new foreign key constraints
-- ============================================

ALTER TABLE pig_observations 
    ADD CONSTRAINT pig_observations_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_surgeries 
    ADD CONSTRAINT pig_surgeries_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_weights 
    ADD CONSTRAINT pig_weights_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_vaccinations 
    ADD CONSTRAINT pig_vaccinations_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_sacrifices 
    ADD CONSTRAINT pig_sacrifices_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

ALTER TABLE pig_pathology_reports 
    ADD CONSTRAINT pig_pathology_reports_pig_id_fkey 
    FOREIGN KEY (pig_id) REFERENCES pigs(id) ON DELETE CASCADE;

-- Add unique constraint for pig_sacrifices and pig_pathology_reports (one per pig)
ALTER TABLE pig_sacrifices ADD CONSTRAINT pig_sacrifices_pig_id_key UNIQUE (pig_id);
ALTER TABLE pig_pathology_reports ADD CONSTRAINT pig_pathology_reports_pig_id_key UNIQUE (pig_id);

-- ============================================
-- STEP 11: Create new indexes
-- ============================================

CREATE INDEX idx_pig_observations_pig_id ON pig_observations(pig_id);
CREATE INDEX idx_pig_surgeries_pig_id ON pig_surgeries(pig_id);
CREATE INDEX idx_pig_weights_pig_id ON pig_weights(pig_id);
CREATE INDEX idx_pig_vaccinations_pig_id ON pig_vaccinations(pig_id);
CREATE INDEX idx_pig_sacrifices_pig_id ON pig_sacrifices(pig_id);

-- ============================================
-- STEP 12: Update pig_record_attachments
-- (Uses record_type + record_id pattern, no direct pig_id FK)
-- No changes needed for this table
-- ============================================

-- ============================================
-- STEP 13: Update deleted_by reference (already UUID, no change needed)
-- ============================================

-- ============================================
-- STEP 14: Verify migration
-- ============================================

-- Verify pigs table structure
DO $$
BEGIN
    -- Check if id is now UUID
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'pigs' 
        AND column_name = 'id' 
        AND data_type = 'uuid'
    ) THEN
        RAISE EXCEPTION 'Migration failed: pigs.id is not UUID';
    END IF;
    
    -- Check if pig_no exists
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'pigs' 
        AND column_name = 'pig_no'
    ) THEN
        RAISE EXCEPTION 'Migration failed: pigs.pig_no does not exist';
    END IF;
    
    RAISE NOTICE 'Migration verification passed!';
END $$;
