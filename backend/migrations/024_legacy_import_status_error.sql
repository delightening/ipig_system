-- 024_legacy_import_status_error.sql
-- Up: allow 'error' status on legacy_import_records.status and add a named CHECK
-- Down（手動）：
--   將 CHECK 限縮回原集合（不含 'error'）；見下方註解範例。

-- Up
BEGIN;
-- Drop existing unnamed CHECK constraint on status (generated name), then add a named one
DO $$
DECLARE
    c RECORD;
BEGIN
    FOR c IN
        SELECT conname
        FROM pg_constraint
        WHERE conrelid = 'public.legacy_import_records'::regclass
          AND contype = 'c'
          AND conkey IS NOT NULL
          AND EXISTS (
              SELECT 1
              FROM pg_attribute a
              WHERE a.attrelid = pg_constraint.conrelid
                AND a.attname = 'status'
                AND a.attnum = ANY (pg_constraint.conkey)
          )
    LOOP
        EXECUTE format('ALTER TABLE legacy_import_records DROP CONSTRAINT %I', c.conname);
    END LOOP;
END $$;

ALTER TABLE legacy_import_records
    ADD CONSTRAINT legacy_import_records_status_check
    CHECK (status IN ('pending','approved','rejected','skipped_duplicate','conflict','error'));
COMMIT;

-- Down（手動）：
-- BEGIN;
-- -- 重新建立不含 'error' 的 CHECK（注意先刪除現有 named constraint）
-- ALTER TABLE legacy_import_records
--     DROP CONSTRAINT IF EXISTS legacy_import_records_status_check;
-- ALTER TABLE legacy_import_records
--     ADD CONSTRAINT legacy_import_records_status_check
--     CHECK (status IN ('pending','approved','rejected','skipped_duplicate','conflict'));
-- COMMIT;

