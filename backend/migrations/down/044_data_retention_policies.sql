-- Down migration for 044_data_retention_policies
-- 用途：staging rollback；移除 R30-17 retention policy 表。
-- 注意：rollback 後 retention_enforcer cron 在 query 時會收到 relation does not
--       exist 錯誤；service 應有對應 fallback log 但 rollback 期間請暫停排程
--       或 disable AUDIT_CHAIN_VERIFY_ACTIVE 等同等開關。

DROP TABLE IF EXISTS data_retention_policies;
