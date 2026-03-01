-- ============================================
-- Migration 009: Performance (合併原 017, 018)
-- ============================================

CREATE INDEX IF NOT EXISTS idx_animals_status_deleted_created ON animals(status, is_deleted, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_protocols_status_pi_created ON protocols(status, pi_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_user_read_created ON notifications(user_id, is_read, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_entity_created ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_attachments_entity ON attachments(entity_type, entity_id);

CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

CREATE OR REPLACE FUNCTION maintenance_vacuum_analyze() RETURNS void AS $$
BEGIN
    ANALYZE animals; ANALYZE animal_observations; ANALYZE animal_surgeries; ANALYZE animal_weights;
    ANALYZE animal_vaccinations; ANALYZE vet_recommendations; ANALYZE notifications;
    ANALYZE user_activity_logs; ANALYZE attachments; ANALYZE protocols; ANALYZE audit_logs;
    RAISE NOTICE 'maintenance_vacuum_analyze completed at %', NOW();
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE VIEW slow_queries AS
SELECT queryid, LEFT(query, 200) AS query_preview, calls, mean_exec_time AS avg_ms, total_exec_time AS total_ms, rows
FROM pg_stat_statements WHERE mean_exec_time > 100 ORDER BY mean_exec_time DESC LIMIT 50;
