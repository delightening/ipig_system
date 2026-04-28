-- R30-10 rollback: 移除 meaning 欄與 signature_meaning enum
-- 注意：rollback 後新建立的 meaning 資料會遺失。建議僅在新環境尚未產生
--       R30-10 後簽章資料時 rollback。

DROP INDEX IF EXISTS idx_esig_meaning;

ALTER TABLE electronic_signatures
    DROP COLUMN IF EXISTS meaning;

DROP TYPE IF EXISTS signature_meaning;
