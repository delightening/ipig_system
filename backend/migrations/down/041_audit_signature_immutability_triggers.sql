-- Down migration for 041_audit_signature_immutability_triggers
-- 用途：staging rollback；移除 R30-20/21 引入的 trigger + function
-- 注意：rollback 後 audit 與 electronic_signatures 的 DB 層保護消失，
--       完全依賴應用層 (services/audit + services/signature) 控制寫入。

DROP TRIGGER IF EXISTS check_electronic_signatures_no_delete_trigger ON electronic_signatures;
DROP FUNCTION IF EXISTS check_electronic_signatures_no_delete();

DROP TRIGGER IF EXISTS check_electronic_signatures_immutable_trigger ON electronic_signatures;
DROP FUNCTION IF EXISTS check_electronic_signatures_immutable();

DROP TRIGGER IF EXISTS check_user_activity_logs_no_delete_trigger ON user_activity_logs;
DROP FUNCTION IF EXISTS check_user_activity_logs_no_delete();
