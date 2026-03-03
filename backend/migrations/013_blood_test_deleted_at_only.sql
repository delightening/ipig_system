-- ============================================
-- Migration 013: 血液檢查統一使用 deleted_at（移除 is_deleted）
-- ============================================
-- 與照護紀錄、疫苗紀錄等一致，改為僅使用 deleted_at 判斷軟刪除。

DROP INDEX IF EXISTS idx_animal_blood_tests_is_deleted;

ALTER TABLE animal_blood_tests DROP COLUMN IF EXISTS is_deleted;

CREATE INDEX IF NOT EXISTS idx_animal_blood_tests_deleted_at
    ON animal_blood_tests(deleted_at) WHERE deleted_at IS NULL;

COMMENT ON COLUMN animal_blood_tests.deleted_at IS '軟刪除時間，NULL 表示未刪除';
