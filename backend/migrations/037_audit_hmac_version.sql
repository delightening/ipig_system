-- R26-6: HMAC 編碼版本化 + 驗證端分流
-- Why: PR #158 的 HMAC chain verify cron 目前是停用狀態（AUDIT_CHAIN_VERIFY_ACTIVE=false），
--      因為舊版 `AuditService::log_activity` 用 string-concat 產生 HMAC（碰撞風險），
--      新版 `log_activity_tx` 用 length-prefix canonical encoding。同表兩種編碼混雜，
--      cron 無法區分會對 legacy row 產生 false positive「斷鏈」告警。
-- How:  新增 hmac_version SMALLINT 欄位標示編碼版本，verifier 依此分流計算 expected HMAC。

ALTER TABLE user_activity_logs
ADD COLUMN IF NOT EXISTS hmac_version SMALLINT;

COMMENT ON COLUMN user_activity_logs.hmac_version IS
'HMAC encoding version. 1 = legacy string-concat (AuditService::log_activity / compute_and_store_hmac). 2 = length-prefix canonical (AuditService::log_activity_tx / compute_and_store_hmac_tx, HmacInput). NULL means no HMAC (event_category=SECURITY rows never join the chain, or pre-R26-6 rows before this column existed).';

-- 索引：verifier 掃描特定版本的 row 時用（未來 R26-7 / backfill 工具可能需要）
CREATE INDEX IF NOT EXISTS idx_user_activity_logs_hmac_version
ON user_activity_logs (hmac_version)
WHERE integrity_hash IS NOT NULL;

-- Backfill 策略（保守）：本 migration 不直接 UPDATE 既有資料，原因：
--   1. 既有 row 是 legacy string-concat 寫入的，hmac_version 應為 1，但要區分 NULL（pre-column）
--      與「已 backfill 為 1」，必須分兩階段：先新增 column（本 migration），之後 R26-6
--      backfill 子任務再把既有 row UPDATE 為 1（可於 deploy 後以 idempotent script 執行）。
--   2. 一次 UPDATE 全表會鎖住大量 row，影響線上系統。
-- 驗證端（`AuditService::verify_chain_rows`）遇到 hmac_version IS NULL 時：
--   - 若 integrity_hash 也為 NULL → SECURITY 事件，合法略過
--   - 若 integrity_hash 非 NULL → 視為 legacy v=1（與 backfill 後結果相同）
