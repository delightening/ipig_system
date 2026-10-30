-- 020_legacy_import_staging.sql
-- Staging tables for one-time legacy import from ipig.pigmodel.asia
-- Up: create legacy_import_batches and legacy_import_records
-- Down（手動）：
--   DROP TABLE IF EXISTS legacy_import_records;
--   DROP TABLE IF EXISTS legacy_import_batches;

-- Up
BEGIN;

CREATE TABLE IF NOT EXISTS legacy_import_batches (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source          text NOT NULL DEFAULT 'ipig.pigmodel.asia',
    description     text,
    created_by      uuid REFERENCES users(id),
    created_at      timestamptz NOT NULL DEFAULT now()
);

-- record_kind enumerates the target formal table category
-- record_type mirrors new-system observation record type when applicable
-- status: pending -> approved/rejected, or skipped_duplicate/conflict at staging
CREATE TABLE IF NOT EXISTS legacy_import_records (
    id                      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    legacy_source           text NOT NULL,
    record_kind             text NOT NULL CHECK (record_kind IN ('observation','surgery','weight','vaccine','sacrifice')),
    legacy_record_id        text NOT NULL,
    legacy_pig_system_id    integer,
    matched_animal_id       uuid NULL REFERENCES animals(id),
    legacy_record_type_text text, -- literal label from old system (e.g. '-', '觀察記錄')
    original_recorder_name  text,
    mapped_user_id          uuid NULL REFERENCES users(id),
    legacy_created_at       timestamptz NULL,
    record_type             text NULL CHECK (record_type IN ('abnormal','experiment','observation')),
    status                  text NOT NULL CHECK (status IN ('pending','approved','rejected','skipped_duplicate','conflict')),
    payload                 jsonb NOT NULL,
    import_batch_id         uuid NOT NULL REFERENCES legacy_import_batches(id) ON DELETE CASCADE,
    target_record_id        uuid NULL,
    created_at              timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_legacy_import_idem
    ON legacy_import_records(legacy_source, record_kind, legacy_record_id);

-- Helpful indexes for reports
CREATE INDEX IF NOT EXISTS idx_legacy_import_records_batch ON legacy_import_records(import_batch_id);
CREATE INDEX IF NOT EXISTS idx_legacy_import_records_status ON legacy_import_records(status);
CREATE INDEX IF NOT EXISTS idx_legacy_import_records_kind ON legacy_import_records(record_kind);

COMMENT ON TABLE legacy_import_batches IS 'One-time legacy import batches (staging).';
COMMENT ON TABLE legacy_import_records IS 'Staging rows for legacy import records before approval.';
COMMENT ON COLUMN legacy_import_records.legacy_record_type_text IS 'Original textual record type from legacy system (e.g. \"-\").';
COMMENT ON COLUMN legacy_import_records.record_type IS 'Mapped new-system observation record_type (abnormal/experiment/observation) when applicable.';
COMMENT ON COLUMN legacy_import_records.status IS 'pending | approved | rejected | skipped_duplicate | conflict';
COMMENT ON COLUMN legacy_import_records.target_record_id IS 'UUID of the created formal record after approval; table inferred from record_kind.';

COMMIT;

-- Down（手動）：
-- BEGIN;
-- DROP TABLE IF EXISTS legacy_import_records;
-- DROP TABLE IF EXISTS legacy_import_batches;
-- COMMIT;

