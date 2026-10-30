-- 022_legacy_recorder_name.sql
-- Up: add legacy_recorder_name (text NULL) to five formal record tables
-- Down: drop those columns

-- Up
BEGIN;
ALTER TABLE animal_observations ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_surgeries ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_weights ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_vaccinations ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
ALTER TABLE animal_sacrifices ADD COLUMN IF NOT EXISTS legacy_recorder_name text;
COMMIT;

-- Down
BEGIN;
ALTER TABLE animal_observations DROP COLUMN IF EXISTS legacy_recorder_name;
ALTER TABLE animal_surgeries DROP COLUMN IF EXISTS legacy_recorder_name;
ALTER TABLE animal_weights DROP COLUMN IF EXISTS legacy_recorder_name;
ALTER TABLE animal_vaccinations DROP COLUMN IF EXISTS legacy_recorder_name;
ALTER TABLE animal_sacrifices DROP COLUMN IF EXISTS legacy_recorder_name;
COMMIT;

