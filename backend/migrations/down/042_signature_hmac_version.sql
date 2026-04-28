-- R30-7 rollback: 移除 hmac_version 欄
-- 注意：rollback 後 v2 (HMAC-SHA256) 簽章將無法再 dispatch 驗證，
--       建議僅在新環境尚未產生 v2 資料時 rollback。

ALTER TABLE electronic_signatures
    DROP COLUMN IF EXISTS hmac_version;
