-- Down migration for 045_audit_hmac_null_index
-- 用途：staging rollback；移除 R28-5 partial index。
-- 注意：rollback 後 register_hmac_legacy_gauge_job 仍可運作但 COUNT(*) 走
--       seq scan，partition 大時會慢；不影響功能正確性，僅監控查詢延遲。

DROP INDEX IF EXISTS idx_audit_hmac_null;
