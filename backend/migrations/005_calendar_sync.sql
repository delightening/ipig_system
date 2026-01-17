-- ============================================
-- Migration 005: Google Calendar Sync
-- 
-- 包含：
-- - 全系統日曆設定
-- - 事件同步追蹤
-- - 衝突管理
-- - 同步歷史
-- - 相關觸發器
-- ============================================

-- ============================================
-- 1. 全系統日曆設定表
-- ============================================

CREATE TABLE google_calendar_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Calendar access
    calendar_id VARCHAR(255) NOT NULL,
    calendar_name VARCHAR(100),
    calendar_description TEXT,
    
    -- Authentication
    auth_method VARCHAR(20) DEFAULT 'gmail_account',
    auth_email VARCHAR(255),
    is_configured BOOLEAN DEFAULT false,
    
    -- Sync settings
    sync_enabled BOOLEAN DEFAULT true,
    sync_schedule_morning TIME DEFAULT '08:00:00',
    sync_schedule_evening TIME DEFAULT '18:00:00',
    sync_timezone VARCHAR(50) DEFAULT 'Asia/Taipei',
    
    -- What to sync
    sync_approved_leaves BOOLEAN DEFAULT true,
    sync_overtime BOOLEAN DEFAULT false,
    
    -- Event format
    event_title_template VARCHAR(255) DEFAULT '[請假] {employee_name} - {leave_type}',
    event_color_id VARCHAR(10),
    
    -- Status
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(20),
    last_sync_error TEXT,
    last_sync_events_pushed INTEGER DEFAULT 0,
    last_sync_events_pulled INTEGER DEFAULT 0,
    last_sync_conflicts INTEGER DEFAULT 0,
    last_sync_duration_ms INTEGER,
    
    -- Next scheduled sync
    next_sync_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Only allow one configuration row
CREATE UNIQUE INDEX idx_calendar_config_singleton ON google_calendar_config ((true));

-- Insert default config (disabled until configured)
INSERT INTO google_calendar_config (
    calendar_id, 
    calendar_name, 
    is_configured, 
    sync_enabled
) VALUES (
    'not-configured@placeholder.com',
    '請假行事曆',
    false,
    false
);

-- ============================================
-- 2. 事件同步追蹤表
-- ============================================

CREATE TABLE calendar_event_sync (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Local reference
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    
    -- Google Calendar reference
    google_event_id VARCHAR(255),
    google_event_etag VARCHAR(255),
    google_event_link VARCHAR(500),
    
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    local_updated_at TIMESTAMPTZ NOT NULL,
    google_updated_at TIMESTAMPTZ,
    
    -- Event data
    last_synced_data JSONB,
    
    -- Status
    sync_status VARCHAR(20) DEFAULT 'pending_create',
    
    -- Error tracking
    last_error TEXT,
    error_count INTEGER DEFAULT 0,
    last_error_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(leave_request_id)
);

CREATE INDEX idx_calendar_sync_status ON calendar_event_sync(sync_status);
CREATE INDEX idx_calendar_sync_pending ON calendar_event_sync(sync_status) 
    WHERE sync_status IN ('pending_create', 'pending_update', 'pending_delete');
CREATE INDEX idx_calendar_sync_google ON calendar_event_sync(google_event_id) 
    WHERE google_event_id IS NOT NULL;

-- ============================================
-- 3. 衝突追蹤表
-- ============================================

CREATE TABLE calendar_sync_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- References
    calendar_event_sync_id UUID REFERENCES calendar_event_sync(id) ON DELETE SET NULL,
    leave_request_id UUID REFERENCES leave_requests(id) ON DELETE SET NULL,
    
    -- Conflict details
    conflict_type VARCHAR(50) NOT NULL,
    
    -- Data comparison
    ipig_data JSONB NOT NULL,
    google_data JSONB,
    
    -- Difference summary
    difference_summary TEXT,
    
    -- Resolution
    status VARCHAR(20) DEFAULT 'pending',
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    -- If accepted Google changes
    requires_new_approval BOOLEAN DEFAULT false,
    new_approval_request_id UUID REFERENCES leave_requests(id),
    
    -- Detection
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_conflicts_pending ON calendar_sync_conflicts(status) WHERE status = 'pending';
CREATE INDEX idx_sync_conflicts_leave ON calendar_sync_conflicts(leave_request_id);

-- ============================================
-- 4. 同步歷史表
-- ============================================

CREATE TABLE calendar_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Job details
    job_type VARCHAR(20) NOT NULL,
    triggered_by UUID REFERENCES users(id),
    
    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Results
    status VARCHAR(20) DEFAULT 'running',
    
    -- Push stats
    events_created INTEGER DEFAULT 0,
    events_updated INTEGER DEFAULT 0,
    events_deleted INTEGER DEFAULT 0,
    
    -- Pull stats
    events_checked INTEGER DEFAULT 0,
    conflicts_detected INTEGER DEFAULT 0,
    
    -- Errors
    errors_count INTEGER DEFAULT 0,
    error_messages JSONB DEFAULT '[]',
    
    -- Progress
    progress_percentage INTEGER DEFAULT 0,
    current_operation VARCHAR(100),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_history_date ON calendar_sync_history(started_at DESC);
CREATE INDEX idx_sync_history_status ON calendar_sync_history(status);

-- ============================================
-- 5. 觸發器: 請假狀態變更時排隊同步
-- ============================================

CREATE OR REPLACE FUNCTION queue_calendar_sync_on_leave_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Only sync approved leaves
    IF NEW.status = 'APPROVED' THEN
        INSERT INTO calendar_event_sync (leave_request_id, local_updated_at, sync_status)
        VALUES (NEW.id, NOW(), 'pending_create')
        ON CONFLICT (leave_request_id) DO UPDATE SET
            local_updated_at = NOW(),
            sync_status = CASE 
                WHEN calendar_event_sync.google_event_id IS NULL THEN 'pending_create'
                ELSE 'pending_update'
            END,
            updated_at = NOW();
    
    ELSIF OLD.status = 'APPROVED' AND NEW.status IN ('CANCELLED', 'REVOKED') THEN
        UPDATE calendar_event_sync
        SET sync_status = 'pending_delete',
            local_updated_at = NOW(),
            updated_at = NOW()
        WHERE leave_request_id = NEW.id
          AND google_event_id IS NOT NULL;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_queue_calendar_sync
    AFTER INSERT OR UPDATE OF status ON leave_requests
    FOR EACH ROW
    EXECUTE FUNCTION queue_calendar_sync_on_leave_change();

-- ============================================
-- 6. 日曆同步相關權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'hr.calendar.config', '設定行事曆同步', 'hr', '可設定 Google 日曆同步', NOW()),
    (gen_random_uuid(), 'hr.calendar.view', '檢視行事曆同步狀態', 'hr', '可檢視日曆同步狀態', NOW()),
    (gen_random_uuid(), 'hr.calendar.sync', '手動觸發行事曆同步', 'hr', '可手動觸發日曆同步', NOW()),
    (gen_random_uuid(), 'hr.calendar.conflicts', '處理行事曆同步衝突', 'hr', '可處理日曆同步衝突', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 完成
-- ============================================
