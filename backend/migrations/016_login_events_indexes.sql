-- R10-L9: login_events composite indexes for common query patterns
-- Covers: email + event_type + created_at (login failure counting, rate limiting)
-- Covers: created_at date + event_type (dashboard daily statistics)

CREATE INDEX IF NOT EXISTS idx_login_email_type_created
    ON login_events (email, event_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_login_created_date_type
    ON login_events ((created_at::date), event_type);
