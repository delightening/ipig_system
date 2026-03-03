-- ============================================
-- Migration 012: care_medication_records 軟刪除欄位
-- ============================================
-- 為照護紀錄（疼痛評估）新增軟刪除支援，符合 GLP 稽核規範。

ALTER TABLE care_medication_records
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deletion_reason TEXT,
    ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

COMMENT ON COLUMN care_medication_records.deleted_at IS '軟刪除時間';
COMMENT ON COLUMN care_medication_records.deletion_reason IS '刪除原因（GLP 合規）';
COMMENT ON COLUMN care_medication_records.deleted_by IS '刪除操作者';
