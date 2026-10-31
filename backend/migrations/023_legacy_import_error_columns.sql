-- 023_legacy_import_error_columns.sql
-- Up: add error_code and error_message to legacy_import_records for per-animal error reporting
-- Down（手動）：
--   從 legacy_import_records 移除 error_code / error_message 欄位。

-- Up
BEGIN;
ALTER TABLE legacy_import_records
    ADD COLUMN IF NOT EXISTS error_code text,
    ADD COLUMN IF NOT EXISTS error_message text;
COMMIT;

-- Down（手動）：
-- BEGIN;
-- ALTER TABLE legacy_import_records DROP COLUMN IF EXISTS error_code;
-- ALTER TABLE legacy_import_records DROP COLUMN IF EXISTS error_message;
-- COMMIT;

