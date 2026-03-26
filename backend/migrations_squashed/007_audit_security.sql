-- ============================================
-- Migration 007: Audit, Security & AI
-- Squashed from: 007(7.1), 008(partial), 013, 016, 017, 022(partial)
-- ============================================

-- ============================================================
-- 7.1 使用者活動日誌（分區表）
-- ============================================================
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

-- 季度分區：2026 Q1–Q4 + 2027 Q1–Q4
CREATE TABLE user_activity_logs_2026_q1 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');
CREATE TABLE user_activity_logs_2026_q2 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');
CREATE TABLE user_activity_logs_2026_q3 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');
CREATE TABLE user_activity_logs_2026_q4 PARTITION OF user_activity_logs FOR VALUES FROM ('2026-10-01') TO ('2027-01-01');
CREATE TABLE user_activity_logs_2027_q1 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-01-01') TO ('2027-04-01');
CREATE TABLE user_activity_logs_2027_q2 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-04-01') TO ('2027-07-01');
CREATE TABLE user_activity_logs_2027_q3 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-07-01') TO ('2027-10-01');
CREATE TABLE user_activity_logs_2027_q4 PARTITION OF user_activity_logs FOR VALUES FROM ('2027-10-01') TO ('2028-01-01');

-- 索引
CREATE INDEX idx_activity_actor ON user_activity_logs(actor_user_id, created_at DESC);
CREATE INDEX idx_activity_entity ON user_activity_logs(entity_type, entity_id, created_at DESC);
CREATE INDEX idx_activity_category ON user_activity_logs(event_category, created_at DESC);
CREATE INDEX idx_activity_event_type ON user_activity_logs(event_type, created_at DESC);
CREATE INDEX idx_activity_suspicious ON user_activity_logs(is_suspicious) WHERE is_suspicious = true;
CREATE INDEX idx_activity_ip ON user_activity_logs(ip_address, created_at DESC);
CREATE INDEX idx_activity_date ON user_activity_logs(partition_date, created_at DESC);
CREATE INDEX idx_activity_logs_integrity ON user_activity_logs(created_at, integrity_hash);
CREATE INDEX idx_audit_entity_created ON user_activity_logs(entity_type, entity_id, created_at DESC);
-- 複合索引：改善 partition pruning (from 022)
CREATE INDEX idx_activity_actor_created ON user_activity_logs(actor_user_id, created_at DESC);

-- ============================================================
-- 7.2 HMAC 完整性強制 (from 013)
-- 僅允許 UPDATE 寫入 integrity_hash / previous_hash，禁止竄改日誌內容
-- ============================================================
CREATE OR REPLACE FUNCTION check_user_activity_logs_immutable()
RETURNS TRIGGER AS $$
BEGIN
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

CREATE TRIGGER trg_user_activity_logs_immutable
  BEFORE UPDATE ON user_activity_logs
  FOR EACH ROW
  EXECUTE FUNCTION check_user_activity_logs_immutable();

-- ============================================================
-- 7.3 登入事件
-- ============================================================
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

-- 複合索引 (from 016) — DO block 處理 non-C locale collation 問題
DO $$ BEGIN
    CREATE INDEX IF NOT EXISTS idx_login_email_type_created
        ON login_events (email, event_type, created_at DESC);
EXCEPTION WHEN feature_not_supported THEN
    CREATE INDEX IF NOT EXISTS idx_login_email_type_created
        ON login_events ((email COLLATE "C"), (event_type COLLATE "C"), created_at DESC);
END $$;

DO $$ BEGIN
    CREATE INDEX IF NOT EXISTS idx_login_created_type
        ON login_events (created_at, event_type);
EXCEPTION WHEN feature_not_supported THEN
    CREATE INDEX IF NOT EXISTS idx_login_created_type
        ON login_events (created_at, (event_type COLLATE "C"));
END $$;

-- ============================================================
-- 7.4 使用者 Session
-- ============================================================
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
CREATE INDEX idx_user_sessions_refresh_token_id ON user_sessions(refresh_token_id);

-- ============================================================
-- 7.5 使用者活動統計
-- ============================================================
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

-- ============================================================
-- 7.6 安全警報
-- ============================================================
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
-- FK 索引 (from 022)
CREATE INDEX idx_security_alerts_user_id ON security_alerts(user_id);
CREATE INDEX idx_security_alerts_login_event_id ON security_alerts(login_event_id);

-- ============================================================
-- 7.7 稽核日誌函數
-- ============================================================
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

-- ============================================================
-- 7.8 AI API Keys (from 017)
-- ============================================================
CREATE TABLE ai_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    -- SHA-256 hash of the key (不儲存明文)
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    -- key 前綴（用於辨識，如 "ipig_ai_abc1..."）
    key_prefix VARCHAR(12) NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    -- 允許的權限範圍（JSON 陣列，如 ["animal.read", "protocol.read"]）
    scopes JSONB NOT NULL DEFAULT '["read"]'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT true,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    usage_count BIGINT NOT NULL DEFAULT 0,
    rate_limit_per_minute INT NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_rate_limit_positive CHECK (rate_limit_per_minute IS NULL OR rate_limit_per_minute > 0)
);
CREATE INDEX idx_ai_api_keys_key_hash ON ai_api_keys(key_hash);
CREATE INDEX idx_ai_api_keys_active ON ai_api_keys(is_active) WHERE is_active = true;

-- AI 查詢日誌（分區表，按月分區）
CREATE TABLE ai_query_logs (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL,
    endpoint VARCHAR(200) NOT NULL,
    method VARCHAR(10) NOT NULL DEFAULT 'GET',
    query_summary JSONB,
    response_status SMALLINT NOT NULL,
    duration_ms INT NOT NULL DEFAULT 0,
    source_ip VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- 動態建立當月 + 下月分區
DO $$
DECLARE
    curr_start DATE := date_trunc('month', CURRENT_DATE);
    next_start DATE := date_trunc('month', CURRENT_DATE + INTERVAL '1 month');
    after_next DATE := date_trunc('month', CURRENT_DATE + INTERVAL '2 months');
    curr_suffix TEXT := to_char(CURRENT_DATE, 'YYYY_MM');
    next_suffix TEXT := to_char(CURRENT_DATE + INTERVAL '1 month', 'YYYY_MM');
BEGIN
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS ai_query_logs_%s PARTITION OF ai_query_logs FOR VALUES FROM (%L) TO (%L)',
        curr_suffix, curr_start, next_start
    );
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS ai_query_logs_%s PARTITION OF ai_query_logs FOR VALUES FROM (%L) TO (%L)',
        next_suffix, next_start, after_next
    );
END $$;

CREATE INDEX idx_ai_query_logs_api_key ON ai_query_logs(api_key_id, created_at);

-- ============================================================
-- 7.9 效能維護函數 (from 008)
-- ============================================================
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
