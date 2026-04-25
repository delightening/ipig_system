-- C1: GLP record locks（21 CFR §11.10(e)(1)：簽章記錄不得被竄改）
-- Why: animal_sacrifices 已有 is_locked / locked_at / locked_by（見 006）；但簽章服務
--      也預期支援 observation / surgery / blood_test / care_medication 鎖定（見
--      services/signature/mod.rs::lock_record / lock_record_uuid 的 match arms），
--      然而對應表缺欄位，導致簽章成功也鎖不住記錄、後續仍可被 update / delete。
-- How:  四張表同時 ADD 三欄；NOT NULL DEFAULT false → 既有列 backfill 為 false，
--      不影響既有未簽章記錄。

ALTER TABLE animal_observations
    ADD COLUMN IF NOT EXISTS is_locked  BOOLEAN     NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS locked_at  TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_by  UUID        REFERENCES users(id);

ALTER TABLE animal_surgeries
    ADD COLUMN IF NOT EXISTS is_locked  BOOLEAN     NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS locked_at  TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_by  UUID        REFERENCES users(id);

ALTER TABLE animal_blood_tests
    ADD COLUMN IF NOT EXISTS is_locked  BOOLEAN     NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS locked_at  TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_by  UUID        REFERENCES users(id);

ALTER TABLE care_medication_records
    ADD COLUMN IF NOT EXISTS is_locked  BOOLEAN     NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS locked_at  TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS locked_by  UUID        REFERENCES users(id);

-- 索引：locked 記錄查詢（cron / admin dashboard 可能需要列出已鎖定的記錄做合規審查）
CREATE INDEX IF NOT EXISTS idx_animal_observations_is_locked
    ON animal_observations (is_locked) WHERE is_locked = true;
CREATE INDEX IF NOT EXISTS idx_animal_surgeries_is_locked
    ON animal_surgeries (is_locked) WHERE is_locked = true;
CREATE INDEX IF NOT EXISTS idx_animal_blood_tests_is_locked
    ON animal_blood_tests (is_locked) WHERE is_locked = true;
CREATE INDEX IF NOT EXISTS idx_care_medication_records_is_locked
    ON care_medication_records (is_locked) WHERE is_locked = true;

COMMENT ON COLUMN animal_observations.is_locked IS
'GLP 簽章鎖。true 後 update/delete 會被 service 層 ensure_not_locked guard 拒絕（409）。';
COMMENT ON COLUMN animal_surgeries.is_locked IS
'GLP 簽章鎖。同 animal_observations.is_locked。';
COMMENT ON COLUMN animal_blood_tests.is_locked IS
'GLP 簽章鎖。同 animal_observations.is_locked。';
COMMENT ON COLUMN care_medication_records.is_locked IS
'GLP 簽章鎖。同 animal_observations.is_locked。';
