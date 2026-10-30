-- 021_surgery_positioning_length.sql
-- Up: widen animal_surgeries.positioning from varchar(100) to varchar(500)
-- Down: shrink back to varchar(100) with safety check

-- Up
BEGIN;
ALTER TABLE animal_surgeries
    ALTER COLUMN positioning TYPE varchar(500);
COMMIT;

-- Down
BEGIN;
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM animal_surgeries
        WHERE positioning IS NOT NULL AND length(positioning) > 100
    ) THEN
        RAISE EXCEPTION 'Cannot downgrade positioning to varchar(100): values longer than 100 exist';
    END IF;
END $$;

ALTER TABLE animal_surgeries
    ALTER COLUMN positioning TYPE varchar(100);
COMMIT;

