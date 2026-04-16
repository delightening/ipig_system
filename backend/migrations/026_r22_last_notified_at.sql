-- R22: Add last_notified_at to security_alerts
-- Prevents sweep_unresolved_alerts from re-notifying the same alert every 6 hours forever.

ALTER TABLE security_alerts
    ADD COLUMN IF NOT EXISTS last_notified_at TIMESTAMPTZ;

-- Index for sweep query: filter on status + created_at + last_notified_at
CREATE INDEX IF NOT EXISTS idx_security_alerts_sweep
    ON security_alerts (status, created_at, last_notified_at)
    WHERE status = 'open';
