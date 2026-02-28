-- P1-S8: Composite indexes for common query patterns
-- Note: not using CONCURRENTLY because SQLx runs migrations inside a transaction

-- Animals: frequently filtered by status + soft-delete, sorted by created_at
CREATE INDEX IF NOT EXISTS idx_animals_status_deleted_created
ON animals(status, is_deleted, created_at DESC);

-- Protocols: frequently filtered by status + PI, sorted by created_at
CREATE INDEX IF NOT EXISTS idx_protocols_status_pi_created
ON protocols(status, pi_user_id, created_at DESC);

-- Notifications: user's unread notifications, sorted by time
CREATE INDEX IF NOT EXISTS idx_notifications_user_read_created
ON notifications(user_id, is_read, created_at DESC);

-- Audit logs: query history by entity
CREATE INDEX IF NOT EXISTS idx_audit_entity_created
ON user_activity_logs(entity_type, entity_id, created_at DESC);

-- Attachments: lookup by entity
CREATE INDEX IF NOT EXISTS idx_attachments_entity
ON attachments(entity_type, entity_id);
