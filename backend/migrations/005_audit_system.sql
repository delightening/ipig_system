-- ============================================
-- Migration 005: Audit System (合併原 006 + 011)
-- ============================================

CREATE TABLE user_activity_logs (
    id UUID DEFAULT gen_random_uuid(),
    actor_user_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    actor_display_name VARCHAR(100),
    actor_roles JSONB,
    session_id UUID,
    session_started_at TIMESTAMPTZ,
    event_category VARCHAR(50) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_severity VARCHAR(20) DEFAULT 'info',
    entity_type VARCHAR(50),
    entity_id UUID,
    entity_display_name VARCHAR(255),
    before_data JSONB,
    after_data JSONB,
    changed_fields TEXT[],
    ip_address INET,
    user_agent TEXT,
    request_path VARCHAR(500),
    request_method VARCHAR(10),
    response_status INTEGER,
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    is_suspicious BOOLEAN DEFAULT false,
    suspicious_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    partition_date DATE NOT NULL DEFAULT CURRENT_DATE,
    integrity_hash VARCHAR(128),
    previous_hash VARCHAR(128),
    PRIMARY KEY (id, partition_date)
) PARTITION BY RANGE (partition_date);

CREATE TABLE user_activity_logs_2026_q1 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
CREATE TABLE user_activity_logs_2026_q2 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');
CREATE TABLE user_activity_logs_2026_q3 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');
CREATE TABLE user_activity_logs_2026_q4 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-10-01') TO ('2027-01-01');
CREATE TABLE user_activity_logs_2027_q1 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-01-01') TO ('2027-04-01');
CREATE TABLE user_activity_logs_2027_q2 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-04-01') TO ('2027-07-01');
CREATE TABLE user_activity_logs_2027_q3 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-07-01') TO ('2027-10-01');
CREATE TABLE user_activity_logs_2027_q4 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-10-01') TO ('2028-01-01');

CREATE INDEX idx_activity_actor ON user_activity_logs(actor_user_id, created_at DESC);
CREATE INDEX idx_activity_entity ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX idx_activity_category ON user_activity_logs(event_category, created_at DESC);
CREATE INDEX idx_activity_event_type ON user_activity_logs(event_type, created_at DESC);
CREATE INDEX idx_activity_suspicious ON user_activity_logs(is_suspicious) WHERE is_suspicious = true;
CREATE INDEX idx_activity_ip ON user_activity_logs(ip_address, created_at DESC);
CREATE INDEX idx_activity_date ON user_activity_logs(partition_date, created_at DESC);
CREATE INDEX idx_activity_logs_integrity ON user_activity_logs (created_at, integrity_hash);

CREATE TABLE login_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    email VARCHAR(255) NOT NULL,
    event_type VARCHAR(20) NOT NULL,
    ip_address INET,
    user_agent TEXT,
    device_type VARCHAR(50),
    browser VARCHAR(50),
    os VARCHAR(50),
    geo_country VARCHAR(100),
    geo_city VARCHAR(100),
    geo_timezone VARCHAR(50),
    is_unusual_time BOOLEAN DEFAULT false,
    is_unusual_location BOOLEAN DEFAULT false,
    is_new_device BOOLEAN DEFAULT false,
    is_mass_login BOOLEAN DEFAULT false,
    device_fingerprint VARCHAR(255),
    failure_reason VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_login_user ON login_events(user_id, created_at DESC);
CREATE INDEX idx_login_email ON login_events(email, created_at DESC);
CREATE INDEX idx_login_ip ON login_events(ip_address, created_at DESC);
CREATE INDEX idx_login_type ON login_events(event_type, created_at DESC);

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    refresh_token_id UUID REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    ended_reason VARCHAR(50)
);
CREATE INDEX idx_sessions_user ON user_sessions(user_id, started_at DESC);
CREATE INDEX idx_sessions_active ON user_sessions(is_active, last_activity_at DESC) WHERE is_active = true;

CREATE TABLE user_activity_aggregates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    aggregate_date DATE NOT NULL,
    login_count INTEGER DEFAULT 0,
    failed_login_count INTEGER DEFAULT 0,
    session_count INTEGER DEFAULT 0,
    total_session_minutes INTEGER DEFAULT 0,
    page_view_count INTEGER DEFAULT 0,
    action_count INTEGER DEFAULT 0,
    actions_by_category JSONB DEFAULT '{}',
    pages_visited JSONB DEFAULT '[]',
    entities_modified JSONB DEFAULT '[]',
    unique_ip_count INTEGER DEFAULT 0,
    unusual_activity_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, aggregate_date)
);
CREATE INDEX idx_aggregates_user_date ON user_activity_aggregates(user_id, aggregate_date DESC);
CREATE INDEX idx_aggregates_date ON user_activity_aggregates(aggregate_date DESC);

CREATE TABLE security_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'warning',
    title VARCHAR(255) NOT NULL,
    description TEXT,
    user_id UUID REFERENCES users(id),
    activity_log_id UUID,
    login_event_id UUID REFERENCES login_events(id),
    context_data JSONB,
    status VARCHAR(20) DEFAULT 'open',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_severity CHECK (severity IN ('info', 'warning', 'critical')),
    CONSTRAINT chk_alert_status CHECK (status IN ('open', 'acknowledged', 'investigating', 'resolved', 'false_positive'))
);
CREATE INDEX idx_alerts_status ON security_alerts(status, created_at DESC);

CREATE OR REPLACE FUNCTION log_activity(
    p_actor_user_id UUID, p_event_category VARCHAR(50), p_event_type VARCHAR(100),
    p_entity_type VARCHAR(50), p_entity_id UUID, p_entity_display_name VARCHAR(255),
    p_before_data JSONB DEFAULT NULL, p_after_data JSONB DEFAULT NULL,
    p_ip_address INET DEFAULT NULL, p_user_agent TEXT DEFAULT NULL
) RETURNS UUID AS $$
DECLARE v_id UUID; v_actor_email VARCHAR(255); v_actor_display_name VARCHAR(100); v_actor_roles JSONB; v_changed_fields TEXT[];
BEGIN
    SELECT email, display_name INTO v_actor_email, v_actor_display_name FROM users WHERE id = p_actor_user_id;
    SELECT jsonb_agg(r.code) INTO v_actor_roles FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = p_actor_user_id;
    IF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        SELECT array_agg(key) INTO v_changed_fields FROM (SELECT key FROM jsonb_each(p_after_data) EXCEPT SELECT key FROM jsonb_each(p_before_data) WHERE p_before_data->key = p_after_data->key) changed_keys;
    END IF;
    INSERT INTO user_activity_logs (actor_user_id, actor_email, actor_display_name, actor_roles, event_category, event_type, entity_type, entity_id, entity_display_name, before_data, after_data, changed_fields, ip_address, user_agent)
    VALUES (p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles, p_event_category, p_event_type, p_entity_type, p_entity_id, p_entity_display_name, p_before_data, p_after_data, v_changed_fields, p_ip_address, p_user_agent) RETURNING id INTO v_id;
    RETURN v_id;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION check_brute_force(p_email VARCHAR(255)) RETURNS BOOLEAN AS $$
DECLARE v_failed_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_failed_count FROM login_events WHERE email = p_email AND event_type = 'login_failure' AND created_at > NOW() - INTERVAL '15 minutes';
    RETURN v_failed_count >= 5;
END;
$$ LANGUAGE plpgsql;
