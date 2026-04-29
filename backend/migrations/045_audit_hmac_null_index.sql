-- R28-5: partial index for HMAC legacy backfill monitoring
--
-- Why:
--   scheduler `register_hmac_legacy_gauge_job` 每 10 分鐘執行
--     SELECT COUNT(*) FROM user_activity_logs
--     WHERE hmac_version IS NULL AND integrity_hash IS NOT NULL
--   user_activity_logs 為 partitioned table，會持續增長；無索引下 COUNT(*)
--   走 seq scan，每次掃全表 + MVCC 可見性檢查，long-tail 後會明顯拖慢監控查詢。
--
-- How:
--   建 partial index 限定「待 backfill」row（剛好 monitor query 的 WHERE）：
--     CREATE INDEX ... ON user_activity_logs(id)
--       WHERE hmac_version IS NULL AND integrity_hash IS NOT NULL
--   1. 索引大小 = 待 backfill row 數，backfill 推進時索引會縮小（最終接近 0）
--   2. monitor query 走 index-only scan，不再 seq scan 全表
--   3. 寫入端（log_activity_tx）每筆都帶 hmac_version=2 + integrity_hash，**不**符合
--      partial 條件 → 不進索引 → 寫入無 overhead

-- partitioned 表（user_activity_logs）的 partial index 會 cascade 到所有子分區。
CREATE INDEX IF NOT EXISTS idx_audit_hmac_null
    ON user_activity_logs (id)
    WHERE hmac_version IS NULL AND integrity_hash IS NOT NULL;

COMMENT ON INDEX idx_audit_hmac_null IS
    'R28-5：HMAC legacy backfill 監控用 partial index。配合 scheduler '
    'register_hmac_legacy_gauge_job 每 10 分鐘 COUNT(*) 走 index-only scan，'
    '不掃全 partitioned 表。Backfill 完成後（30 天 = 0）可移除。';
