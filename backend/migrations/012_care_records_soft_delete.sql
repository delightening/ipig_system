-- ============================================
-- Migration 012: 照護紀錄軟刪除（GLP 合規）
-- ============================================
-- 照護紀錄（疼痛評估）改為軟刪除，保留刪除原因與操作者。

ALTER TABLE care_medication_records
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deletion_reason TEXT,
    ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_care_medication_records_deleted_at
    ON care_medication_records(deleted_at) WHERE deleted_at IS NULL;

COMMENT ON COLUMN care_medication_records.deleted_at IS '軟刪除時間';
COMMENT ON COLUMN care_medication_records.deletion_reason IS '刪除原因（GLP 合規）';
COMMENT ON COLUMN care_medication_records.deleted_by IS '刪除操作者';
