-- 022_legacy_recorder_name.sql
-- Up: add legacy_recorder_name (text NULL) to five formal record tables
-- Down（手動）：
--   從五張正式表移除該欄位。

-- Up
BEGIN;
ALTER TABLE animal_observations ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_surgeries ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_weights ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_vaccinations ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_sacrifices ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
COMMIT;

-- Down（手動）：
-- BEGIN;
-- ALTER TABLE animal_observations DROP COLUMN IF EXISTS legacy_recorder_name;
-- ALTER TABLE animal_surgeries DROP COLUMN IF EXISTS legacy_recorder_name;
-- ALTER TABLE animal_weights DROP COLUMN IF EXISTS legacy_recorder_name;
-- ALTER TABLE animal_vaccinations DROP COLUMN IF EXISTS legacy_recorder_name;
-- ALTER TABLE animal_sacrifices DROP COLUMN IF EXISTS legacy_recorder_name;
-- COMMIT;

