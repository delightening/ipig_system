-- ============================================
-- Migration 013: 稽核日誌 HMAC 完整性強制（High 4）
-- 僅允許 UPDATE 寫入 integrity_hash / previous_hash，禁止竄改日誌內容
-- ============================================

CREATE OR REPLACE FUNCTION check_user_activity_logs_immutable()
RETURNS TRIGGER AS $$
BEGIN
  -- 僅允許更新 integrity_hash 與 previous_hash，其餘欄位不得變更
  IF OLD.id IS DISTINCT FROM NEW.id
     OR OLD.actor_user_id IS DISTINCT FROM NEW.actor_user_id
     OR OLD.actor_email IS DISTINCT FROM NEW.actor_email
     OR OLD.actor_display_name IS DISTINCT FROM NEW.actor_display_name
     OR (OLD.actor_roles IS DISTINCT FROM NEW.actor_roles)
     OR OLD.session_id IS DISTINCT FROM NEW.session_id
     OR OLD.session_started_at IS DISTINCT FROM NEW.session_started_at
     OR OLD.event_category IS DISTINCT FROM NEW.event_category
     OR OLD.event_type IS DISTINCT FROM NEW.event_type
     OR OLD.event_severity IS DISTINCT FROM NEW.event_severity
     OR OLD.entity_type IS DISTINCT FROM NEW.entity_type
     OR OLD.entity_id IS DISTINCT FROM NEW.entity_id
     OR OLD.entity_display_name IS DISTINCT FROM NEW.entity_display_name
     OR (OLD.before_data IS DISTINCT FROM NEW.before_data)
     OR (OLD.after_data IS DISTINCT FROM NEW.after_data)
     OR (OLD.changed_fields IS DISTINCT FROM NEW.changed_fields)
     OR OLD.ip_address IS DISTINCT FROM NEW.ip_address
     OR OLD.user_agent IS DISTINCT FROM NEW.user_agent
     OR OLD.request_path IS DISTINCT FROM NEW.request_path
     OR OLD.request_method IS DISTINCT FROM NEW.request_method
     OR OLD.response_status IS DISTINCT FROM NEW.response_status
     OR OLD.geo_country IS DISTINCT FROM NEW.geo_country
     OR OLD.geo_city IS DISTINCT FROM NEW.geo_city
     OR OLD.is_suspicious IS DISTINCT FROM NEW.is_suspicious
     OR OLD.suspicious_reason IS DISTINCT FROM NEW.suspicious_reason
     OR OLD.created_at IS DISTINCT FROM NEW.created_at
     OR OLD.partition_date IS DISTINCT FROM NEW.partition_date
  THEN
    RAISE EXCEPTION 'user_activity_logs: direct modification of log payload is not allowed (integrity enforcement)';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_user_activity_logs_immutable ON user_activity_logs;
CREATE TRIGGER trg_user_activity_logs_immutable
  BEFORE UPDATE ON user_activity_logs
  FOR EACH ROW
  EXECUTE FUNCTION check_user_activity_logs_immutable();
