-- R10-L9: login_events composite indexes for common query patterns
-- Covers: email + event_type + created_at (login failure counting, rate limiting)
-- Covers: created_at + event_type (dashboard daily statistics, range queries)

CREATE INDEX IF NOT EXISTS idx_login_email_type_created
    ON login_events (email, event_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_login_created_type
    ON login_events (created_at, event_type);
