-- 034_audit_impersonation.sql
-- R26-1：audit log 加入 impersonated_by_user_id 欄位（GLP 合規 + SEC-11）
--
-- 背景：
-- - 現行 CurrentUser.impersonated_by (Option<Uuid>) 記錄管理員用 SEC-11
--   impersonate 成別人時，真正執行的管理員 ID
-- - 但 audit log 的 actor_user_id 只記「被模擬者」— 稽核員看不到真正操作的人
-- - 本 migration 補一欄 impersonated_by_user_id，由 AuditService::log_activity_tx
--   自動偵測並填入
--
-- 查詢語意：
--   impersonated_by_user_id IS NULL     → actor 自己執行
--   impersonated_by_user_id IS NOT NULL → actor_user_id 是被模擬者，
--                                          真正執行的管理員是 impersonated_by_user_id

ALTER TABLE user_activity_logs
    ADD COLUMN IF NOT EXISTS impersonated_by_user_id UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_user_activity_logs_impersonated_by
    ON user_activity_logs(impersonated_by_user_id, created_at DESC)
    WHERE impersonated_by_user_id IS NOT NULL;

COMMENT ON COLUMN user_activity_logs.impersonated_by_user_id IS
    'When non-NULL: the real admin user_id who impersonated actor_user_id (SEC-11). NULL for direct operations.';
