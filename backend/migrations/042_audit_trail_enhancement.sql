-- Audit Trail Enhancement for GLP Compliance
-- Version: 1.0
-- Created: 2026-01-17
-- Description: Enhanced activity logging, login tracking, and session management

-- ============================================
-- User Activity Logs (Partitioned by Date)
-- ============================================

CREATE TABLE user_activity_logs (
    id UUID DEFAULT gen_random_uuid(),
    
    -- Actor information
    actor_user_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    actor_display_name VARCHAR(100),
    actor_roles JSONB, -- Snapshot of roles at time of action
    
    -- Session context
    session_id UUID,
    session_started_at TIMESTAMPTZ,
    
    -- Event classification
    event_category VARCHAR(50) NOT NULL, -- 'auth', 'data', 'admin', 'export', 'navigation'
    event_type VARCHAR(100) NOT NULL, -- 'login_success', 'pig.create', 'protocol.approve', etc.
    event_severity VARCHAR(20) DEFAULT 'info', -- 'info', 'warning', 'critical'
    
    -- Target entity
    entity_type VARCHAR(50), -- 'pig', 'protocol', 'document', 'user', etc.
    entity_id UUID,
    entity_display_name VARCHAR(255), -- Human-readable identifier
    
    -- Change tracking
    before_data JSONB,
    after_data JSONB,
    changed_fields TEXT[], -- Array of field names that changed
    
    -- Context
    ip_address INET,
    user_agent TEXT,
    request_path VARCHAR(500),
    request_method VARCHAR(10),
    response_status INTEGER,
    
    -- Geolocation (populated by background job if enabled)
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    
    -- Security flags
    is_suspicious BOOLEAN DEFAULT false,
    suspicious_reason TEXT,
    
    -- Timestamps (immutable)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Partitioning key for future optimization
    partition_date DATE NOT NULL DEFAULT CURRENT_DATE,
    
    -- Composite primary key (required for partitioned tables - must include partition column)
    PRIMARY KEY (id, partition_date)
) PARTITION BY RANGE (partition_date);

-- Create partitions for 2 years of data (2026-2027)
CREATE TABLE user_activity_logs_2026_q1 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
CREATE TABLE user_activity_logs_2026_q2 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');
CREATE TABLE user_activity_logs_2026_q3 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');
CREATE TABLE user_activity_logs_2026_q4 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2026-10-01') TO ('2027-01-01');
CREATE TABLE user_activity_logs_2027_q1 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-01-01') TO ('2027-04-01');
CREATE TABLE user_activity_logs_2027_q2 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-04-01') TO ('2027-07-01');
CREATE TABLE user_activity_logs_2027_q3 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-07-01') TO ('2027-10-01');
CREATE TABLE user_activity_logs_2027_q4 PARTITION OF user_activity_logs
    FOR VALUES FROM ('2027-10-01') TO ('2028-01-01');

-- Default partition for any dates outside the range
CREATE TABLE user_activity_logs_default PARTITION OF user_activity_logs DEFAULT;

-- Indexes for common query patterns
CREATE INDEX idx_activity_actor ON user_activity_logs(actor_user_id, created_at DESC);
CREATE INDEX idx_activity_entity ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX idx_activity_category ON user_activity_logs(event_category, created_at DESC);
CREATE INDEX idx_activity_event_type ON user_activity_logs(event_type, created_at DESC);
CREATE INDEX idx_activity_suspicious ON user_activity_logs(is_suspicious) WHERE is_suspicious = true;
CREATE INDEX idx_activity_ip ON user_activity_logs(ip_address, created_at DESC);
CREATE INDEX idx_activity_date ON user_activity_logs(partition_date, created_at DESC);

-- ============================================
-- Login Events Table
-- ============================================

CREATE TABLE login_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    email VARCHAR(255) NOT NULL,
    
    -- Event details
    event_type VARCHAR(20) NOT NULL, -- 'login_success', 'login_failure', 'logout', 'session_expired', 'token_refresh'
    
    -- Device/Network
    ip_address INET,
    user_agent TEXT,
    device_type VARCHAR(50), -- 'desktop', 'mobile', 'tablet', 'unknown'
    browser VARCHAR(50),
    os VARCHAR(50),
    
    -- Geolocation
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    geo_timezone VARCHAR(50),
    
    -- Security analysis
    is_unusual_time BOOLEAN DEFAULT false, -- Outside normal working hours (before 7 AM or after 10 PM)
    is_unusual_location BOOLEAN DEFAULT false, -- Different from usual locations
    is_new_device BOOLEAN DEFAULT false, -- First time seeing this device fingerprint
    device_fingerprint VARCHAR(255),
    failure_reason VARCHAR(100), -- 'invalid_password', 'account_locked', 'inactive_user', 'user_not_found', etc.
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_login_user ON login_events(user_id, created_at DESC);
CREATE INDEX idx_login_email ON login_events(email, created_at DESC);
CREATE INDEX idx_login_ip ON login_events(ip_address, created_at DESC);
CREATE INDEX idx_login_type ON login_events(event_type, created_at DESC);
CREATE INDEX idx_login_unusual ON login_events(user_id) 
    WHERE is_unusual_time OR is_unusual_location OR is_new_device;
CREATE INDEX idx_login_failure ON login_events(email, created_at DESC) 
    WHERE event_type = 'login_failure';

-- ============================================
-- User Sessions Table
-- ============================================

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Session details
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Token reference (for invalidation)
    refresh_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    
    -- Device/Network (from login)
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    
    -- Activity summary
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    ended_reason VARCHAR(50) -- 'logout', 'expired', 'forced_logout', 'new_session', 'token_revoked'
);

CREATE INDEX idx_sessions_user ON user_sessions(user_id, started_at DESC);
CREATE INDEX idx_sessions_active ON user_sessions(is_active, last_activity_at DESC) WHERE is_active = true;
CREATE INDEX idx_sessions_token ON user_sessions(refresh_token_id);

-- ============================================
-- User Activity Aggregates (Daily Summary)
-- ============================================

CREATE TABLE user_activity_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    aggregate_date DATE NOT NULL,
    
    -- Counts
    login_count INTEGER DEFAULT 0,
    failed_login_count INTEGER DEFAULT 0,
    session_count INTEGER DEFAULT 0,
    total_session_minutes INTEGER DEFAULT 0,
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    
    -- Breakdowns
    actions_by_category JSONB DEFAULT '{}', -- {"auth": 5, "data": 10, ...}
    pages_visited JSONB DEFAULT '[]', -- ["/pigs", "/protocols", ...]
    entities_modified JSONB DEFAULT '[]', -- [{"type": "pig", "count": 3}, ...]
    
    -- Security
    unique_ip_count INTEGER DEFAULT 0,
    unusual_activity_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(user_id, aggregate_date)
);

CREATE INDEX idx_aggregates_user_date ON user_activity_aggregates(user_id, aggregate_date DESC);
CREATE INDEX idx_aggregates_date ON user_activity_aggregates(aggregate_date DESC);

-- ============================================
-- Security Alerts Table
-- ============================================

CREATE TABLE security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Alert details
    alert_type VARCHAR(50) NOT NULL, -- 'brute_force', 'unusual_login', 'high_volume', 'suspicious_pattern'
    severity VARCHAR(20) NOT NULL DEFAULT 'warning', -- 'info', 'warning', 'critical'
    title VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Related entities
    user_id UUID REFERENCES users(id),
    activity_log_id UUID,
    login_event_id UUID REFERENCES login_events(id),
    
    -- Context
    context_data JSONB, -- Additional context (IP, patterns, etc.)
    
    -- Resolution
    status VARCHAR(20) DEFAULT 'open', -- 'open', 'acknowledged', 'investigating', 'resolved', 'dismissed'
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alerts_status ON security_alerts(status, created_at DESC);
CREATE INDEX idx_alerts_user ON security_alerts(user_id, created_at DESC);
CREATE INDEX idx_alerts_severity ON security_alerts(severity, status) WHERE status IN ('open', 'acknowledged', 'investigating');

-- ============================================
-- Comments on Retention Policy
-- ============================================

COMMENT ON TABLE user_activity_logs IS 'GLP Compliance: Retention policy - 2 years hot storage, 5 years cold archive, 7 years total. Partitioned by quarter for efficient archival.';
COMMENT ON TABLE login_events IS 'GLP Compliance: Retention policy - 2 years hot storage, 5 years cold archive, 7 years total.';
COMMENT ON TABLE user_sessions IS 'Session tracking for security analysis. Sessions older than 90 days can be archived.';
COMMENT ON TABLE user_activity_aggregates IS 'Daily aggregates for dashboard. Can be regenerated from activity logs if needed.';

-- ============================================
-- Helper Functions
-- ============================================

-- Function to log an activity (for use in triggers or application code)
CREATE OR REPLACE FUNCTION log_activity(
    p_actor_user_id UUID,
    p_event_category VARCHAR(50),
    p_event_type VARCHAR(100),
    p_entity_type VARCHAR(50),
    p_entity_id UUID,
    p_entity_display_name VARCHAR(255),
    p_before_data JSONB DEFAULT NULL,
    p_after_data JSONB DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    v_id UUID;
    v_actor_email VARCHAR(255);
    v_actor_display_name VARCHAR(100);
    v_actor_roles JSONB;
    v_changed_fields TEXT[];
BEGIN
    -- Get actor info
    SELECT email, display_name INTO v_actor_email, v_actor_display_name
    FROM users WHERE id = p_actor_user_id;
    
    -- Get actor roles
    SELECT jsonb_agg(r.code) INTO v_actor_roles
    FROM user_roles ur
    JOIN roles r ON ur.role_id = r.id
    WHERE ur.user_id = p_actor_user_id;
    
    -- Calculate changed fields
    IF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        SELECT array_agg(key) INTO v_changed_fields
        FROM (
            SELECT key FROM jsonb_each(p_after_data)
            EXCEPT
            SELECT key FROM jsonb_each(p_before_data) WHERE p_before_data->key = p_after_data->key
        ) changed_keys;
    END IF;
    
    -- Insert log entry
    INSERT INTO user_activity_logs (
        actor_user_id, actor_email, actor_display_name, actor_roles,
        event_category, event_type,
        entity_type, entity_id, entity_display_name,
        before_data, after_data, changed_fields,
        ip_address, user_agent
    ) VALUES (
        p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles,
        p_event_category, p_event_type,
        p_entity_type, p_entity_id, p_entity_display_name,
        p_before_data, p_after_data, v_changed_fields,
        p_ip_address, p_user_agent
    ) RETURNING id INTO v_id;
    
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

-- Function to check for brute force attacks (5+ failed logins in 15 minutes)
CREATE OR REPLACE FUNCTION check_brute_force(p_email VARCHAR(255)) RETURNS BOOLEAN AS $$
DECLARE
    v_failed_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_failed_count
    FROM login_events
    WHERE email = p_email
      AND event_type = 'login_failure'
      AND created_at > NOW() - INTERVAL '15 minutes';
    
    RETURN v_failed_count >= 5;
END;
$$ LANGUAGE plpgsql;
