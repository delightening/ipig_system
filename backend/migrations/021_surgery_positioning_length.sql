-- 021_surgery_positioning_length.sql
-- Up: widen animal_surgeries.positioning from varchar(100) to varchar(500)
-- Down（手動）：
--   檢查長度後再縮回 varchar(100)；見下方註解範例。

-- Up
BEGIN;
ALTER TABLE animal_surgeries
    ALTER COLUMN positioning TYPE varchar(500);
COMMIT;

-- Down（手動）：
-- BEGIN;
-- DO $$
-- BEGIN
--     IF EXISTS (
--         SELECT 1 FROM animal_surgeries
--         WHERE positioning IS NOT NULL AND length(positioning) > 100
--     ) THEN
--         RAISE EXCEPTION 'Cannot downgrade positioning to varchar(100): values longer than 100 exist';
--     END IF;
-- END $$;
-- ALTER TABLE animal_surgeries
--     ALTER COLUMN positioning TYPE varchar(100);
-- COMMIT;

