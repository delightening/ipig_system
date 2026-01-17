-- Google Calendar Sync (Shared Calendar with Dedicated Account)
-- Version: 1.0
-- Created: 2026-01-17
-- Description: Calendar sync configuration, event tracking, and conflict management

-- ============================================
-- System-wide Calendar Configuration
-- ============================================

CREATE TABLE google_calendar_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Calendar access
    calendar_id VARCHAR(255) NOT NULL, -- e.g., 'team-leave@group.calendar.google.com'
    calendar_name VARCHAR(100),
    calendar_description TEXT,
    
    -- Authentication (dedicated Gmail account)
    -- Credentials stored in environment variables, this just tracks if configured
    auth_method VARCHAR(20) DEFAULT 'gmail_account', -- 'gmail_account', 'service_account'
    auth_email VARCHAR(255), -- Email used for authentication (not the password!)
    is_configured BOOLEAN DEFAULT false,
    
    -- Sync settings
    sync_enabled BOOLEAN DEFAULT true,
    sync_schedule_morning TIME DEFAULT '08:00:00', -- Taiwan time
    sync_schedule_evening TIME DEFAULT '18:00:00', -- Taiwan time
    sync_timezone VARCHAR(50) DEFAULT 'Asia/Taipei',
    
    -- What to sync
    sync_approved_leaves BOOLEAN DEFAULT true,
    sync_overtime BOOLEAN DEFAULT false,
    
    -- Event format
    event_title_template VARCHAR(255) DEFAULT '[請假] {employee_name} - {leave_type}',
    event_color_id VARCHAR(10), -- Google Calendar color ID
    
    -- Status
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(20), -- 'success', 'partial', 'failed', 'disabled'
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
-- Event Sync Tracking
-- ============================================

CREATE TABLE calendar_event_sync (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Local reference
    leave_request_id UUID NOT NULL REFERENCES leave_requests(id) ON DELETE CASCADE,
    
    -- Google Calendar reference
    google_event_id VARCHAR(255), -- NULL if not yet synced
    google_event_etag VARCHAR(255), -- For change detection
    google_event_link VARCHAR(500), -- Direct link to event
    
    -- Sync metadata
    sync_version INTEGER DEFAULT 0, -- Incremented on each sync
    local_updated_at TIMESTAMPTZ NOT NULL,
    google_updated_at TIMESTAMPTZ,
    
    -- Event data (for comparison)
    last_synced_data JSONB, -- What was last synced to/from Google
    
    -- Status
    sync_status VARCHAR(20) DEFAULT 'pending_create', 
    -- 'pending_create', 'pending_update', 'pending_delete', 'synced', 'conflict', 'error'
    
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
-- Conflict Tracking for Manual Review
-- ============================================

CREATE TABLE calendar_sync_conflicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- References
    calendar_event_sync_id UUID REFERENCES calendar_event_sync(id) ON DELETE SET NULL,
    leave_request_id UUID REFERENCES leave_requests(id) ON DELETE SET NULL,
    
    -- Conflict details
    conflict_type VARCHAR(50) NOT NULL, 
    -- 'time_mismatch', 'deleted_in_google', 'data_mismatch', 'title_mismatch'
    
    -- Data comparison
    ipig_data JSONB NOT NULL, -- Leave request data from iPig
    google_data JSONB, -- Event data from Google (null if deleted)
    
    -- Difference summary (for quick display)
    difference_summary TEXT,
    
    -- Resolution
    status VARCHAR(20) DEFAULT 'pending', 
    -- 'pending', 'resolved_keep_ipig', 'resolved_accept_google', 'dismissed'
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    
    -- If accepted Google changes, track the approval
    requires_new_approval BOOLEAN DEFAULT false,
    new_approval_request_id UUID REFERENCES leave_requests(id),
    
    -- Detection
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sync_conflicts_pending ON calendar_sync_conflicts(status) WHERE status = 'pending';
CREATE INDEX idx_sync_conflicts_leave ON calendar_sync_conflicts(leave_request_id);

-- ============================================
-- Sync Job History
-- ============================================

CREATE TABLE calendar_sync_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Job details
    job_type VARCHAR(20) NOT NULL, -- 'scheduled_morning', 'scheduled_evening', 'manual'
    triggered_by UUID REFERENCES users(id), -- NULL for scheduled, user ID for manual
    
    -- Timing
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Results
    status VARCHAR(20) DEFAULT 'running', -- 'running', 'success', 'partial', 'failed'
    
    -- Push stats (iPig → Google)
    events_created INTEGER DEFAULT 0,
    events_updated INTEGER DEFAULT 0,
    events_deleted INTEGER DEFAULT 0,
    
    -- Pull stats (Google → iPig comparison)
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
-- Trigger: Queue sync when leave is approved
-- ============================================

CREATE OR REPLACE FUNCTION queue_calendar_sync_on_leave_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Only sync approved leaves
    IF NEW.status = 'APPROVED' THEN
        -- Insert or update sync record
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
        -- Mark for deletion if was previously synced
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
-- Permissions for Calendar Sync
-- ============================================

INSERT INTO permissions (id, code, name, created_at) VALUES
    -- Calendar configuration (admin only)
    (gen_random_uuid(), 'hr.calendar.config', '設定行事曆同步', NOW()),
    
    -- View sync status
    (gen_random_uuid(), 'hr.calendar.view', '檢視行事曆同步狀態', NOW()),
    
    -- Trigger manual sync
    (gen_random_uuid(), 'hr.calendar.sync', '手動觸發行事曆同步', NOW()),
    
    -- Resolve conflicts
    (gen_random_uuid(), 'hr.calendar.conflicts', '處理行事曆同步衝突', NOW())
ON CONFLICT (code) DO NOTHING;

-- Assign to admin
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'admin' AND p.code LIKE 'hr.calendar.%'
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Assign view to IACUC staff
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id 
FROM roles r, permissions p 
WHERE r.code = 'iacuc_staff' AND p.code = 'hr.calendar.view'
ON CONFLICT (role_id, permission_id) DO NOTHING;
