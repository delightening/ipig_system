-- R22: Attack Detection & Active Alerting
-- Security alert configuration (thresholds) + notification channels

-- ── security_alert_config ──────────────────────────────────────
-- Key/value store for alert thresholds, cached in-memory by AlertThresholdService

CREATE TABLE IF NOT EXISTS security_alert_config (
    key         VARCHAR(100) PRIMARY KEY,
    value       TEXT NOT NULL,
    description TEXT,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO security_alert_config (key, value, description) VALUES
    ('auth_rate_limit_threshold',   '10',  'Auth rate limit triggers in window before escalation alert'),
    ('auth_rate_limit_window_mins', '5',   'Window (minutes) for auth rate limit escalation'),
    ('idor_403_threshold',          '20',  '403 permission denied count before IDOR probe alert'),
    ('idor_403_window_mins',        '5',   'Window (minutes) for IDOR probe detection'),
    ('brute_force_dedup_mins',      '30',  'Suppress duplicate brute_force alerts within this window'),
    ('alert_escalation_dedup_mins', '30',  'Suppress duplicate escalation alerts within this window')
ON CONFLICT (key) DO NOTHING;

-- ── security_notification_channels ─────────────────────────────
-- Configures how security alerts are pushed to admins

CREATE TABLE IF NOT EXISTS security_notification_channels (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel      VARCHAR(20) NOT NULL,
    is_enabled   BOOLEAN NOT NULL DEFAULT false,
    config_json  JSONB NOT NULL DEFAULT '{}',
    min_severity VARCHAR(20) NOT NULL DEFAULT 'warning',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_channel CHECK (channel IN ('email', 'line', 'webhook')),
    CONSTRAINT chk_min_severity CHECK (min_severity IN ('info', 'warning', 'critical'))
);
