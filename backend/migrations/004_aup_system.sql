-- ============================================
-- Migration 004: AUP System
-- 
-- 包含：
-- - 計畫書主表
-- - 計畫版本與歷程
-- - 審查系統
-- - 附件與用戶關聯
-- - 修正案系統
-- - 使用者 AUP Profile
-- 
-- 編碼: UTF-8 (無 BOM)
-- ============================================

-- ============================================
-- 1. 計畫書主表
-- ============================================

CREATE TABLE protocols (
    id UUID PRIMARY KEY,
    protocol_no VARCHAR(50) NOT NULL UNIQUE,
    iacuc_no VARCHAR(50) UNIQUE,
    title VARCHAR(500) NOT NULL,
    status protocol_status NOT NULL DEFAULT 'DRAFT',
    pi_user_id UUID NOT NULL REFERENCES users(id),
    working_content JSONB,
    start_date DATE,
    end_date DATE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocols_status ON protocols(status);
CREATE INDEX idx_protocols_pi_user_id ON protocols(pi_user_id);
CREATE INDEX idx_protocols_iacuc_no ON protocols(iacuc_no);
CREATE INDEX idx_protocols_created_by ON protocols(created_by);

-- JSONB GIN 索引 for working_content
CREATE INDEX idx_protocols_working_content ON protocols USING GIN (working_content);

-- ============================================
-- 2. 用戶計畫關聯表
-- ============================================

CREATE TABLE user_protocols (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    role_in_protocol protocol_role NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by UUID REFERENCES users(id),
    PRIMARY KEY (user_id, protocol_id)
);

CREATE INDEX idx_user_protocols_user_id ON user_protocols(user_id);
CREATE INDEX idx_user_protocols_protocol_id ON user_protocols(protocol_id);

-- ============================================
-- 3. 計畫版本快照
-- ============================================

CREATE TABLE protocol_versions (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (protocol_id, version_no)
);

CREATE INDEX idx_protocol_versions_protocol_id ON protocol_versions(protocol_id);

-- ============================================
-- 4. 計畫狀態歷程
-- ============================================

CREATE TABLE protocol_status_history (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    from_status protocol_status,
    to_status protocol_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocol_status_history_protocol_id ON protocol_status_history(protocol_id);

-- ============================================
-- 5. 審查人員指派
-- ============================================

CREATE TABLE review_assignments (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    -- 審查增強（原 011）
    is_primary_reviewer BOOLEAN NOT NULL DEFAULT false,
    review_stage VARCHAR(20) DEFAULT 'UNDER_REVIEW',
    UNIQUE (protocol_id, reviewer_id)
);

CREATE INDEX idx_review_assignments_protocol_id ON review_assignments(protocol_id);
CREATE INDEX idx_review_assignments_reviewer_id ON review_assignments(reviewer_id);

-- ============================================
-- 6. 審查意見
-- ============================================

CREATE TABLE review_comments (
    id UUID PRIMARY KEY,
    -- 允許 protocol_version_id 為 NULL（原 011）
    protocol_version_id UUID REFERENCES protocol_versions(id) ON DELETE CASCADE,
    -- 直接關聯 protocol_id（原 011）
    protocol_id UUID REFERENCES protocols(id),
    reviewer_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    -- 審查階段（原 011）
    review_stage VARCHAR(20),
    -- 回覆功能（原 012）
    parent_comment_id UUID REFERENCES review_comments(id) ON DELETE CASCADE,
    replied_by UUID REFERENCES users(id),
    -- 草稿功能（原 012）
    draft_content TEXT,
    drafted_by UUID REFERENCES users(id),
    draft_updated_at TIMESTAMPTZ,
    -- 時間戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- 約束（原 011+015）
    CONSTRAINT chk_review_stage CHECK (review_stage IS NULL OR review_stage IN (
        'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED',
        'VET_REVIEW', 'VET_REVISION_REQUIRED',
        'UNDER_REVIEW'
    )),
    CONSTRAINT chk_protocol_reference CHECK (protocol_version_id IS NOT NULL OR protocol_id IS NOT NULL)
);

CREATE INDEX idx_review_comments_protocol_version_id ON review_comments(protocol_version_id);
CREATE INDEX idx_review_comments_reviewer_id ON review_comments(reviewer_id);
CREATE INDEX idx_review_comments_protocol_id ON review_comments(protocol_id);
CREATE INDEX idx_review_comments_review_stage ON review_comments(review_stage);
CREATE INDEX idx_review_comments_parent ON review_comments(parent_comment_id);
CREATE INDEX idx_review_comments_drafted_by ON review_comments(drafted_by);

-- ============================================
-- 7. 計畫附件表
-- ============================================

CREATE TABLE protocol_attachments (
    id UUID PRIMARY KEY,
    protocol_version_id UUID REFERENCES protocol_versions(id) ON DELETE CASCADE,
    protocol_id UUID REFERENCES protocols(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_protocol_attachments_protocol_id ON protocol_attachments(protocol_id);
CREATE INDEX idx_protocol_attachments_protocol_version_id ON protocol_attachments(protocol_version_id);

-- ============================================
-- 8. 修正案主表
-- ============================================

CREATE TABLE amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    amendment_no VARCHAR(50) NOT NULL,
    revision_number INTEGER NOT NULL DEFAULT 1,
    amendment_type amendment_type NOT NULL DEFAULT 'PENDING',
    status amendment_status NOT NULL DEFAULT 'DRAFT',
    title VARCHAR(200) NOT NULL,
    description TEXT,
    change_items VARCHAR(255)[] DEFAULT '{}',
    changes_content JSONB,
    submitted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    classified_by UUID REFERENCES users(id),
    classified_at TIMESTAMPTZ,
    classification_remark TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (protocol_id, amendment_no)
);

CREATE INDEX idx_amendments_protocol_id ON amendments(protocol_id);
CREATE INDEX idx_amendments_status ON amendments(status);

-- ============================================
-- 9. 修正案審查表
-- ============================================

CREATE TABLE amendment_review_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decision VARCHAR(20),
    decided_at TIMESTAMPTZ,
    comment TEXT,
    UNIQUE (amendment_id, reviewer_id)
);

CREATE INDEX idx_amendment_review_assignments_amendment ON amendment_review_assignments(amendment_id);
CREATE INDEX idx_amendment_review_assignments_reviewer ON amendment_review_assignments(reviewer_id);

-- ============================================
-- 10. 修正案版本快照
-- ============================================

CREATE TABLE amendment_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (amendment_id, version_no)
);

-- ============================================
-- 11. 修正案狀態歷程
-- ============================================

CREATE TABLE amendment_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    from_status amendment_status,
    to_status amendment_status NOT NULL,
    changed_by UUID NOT NULL,
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 10. 使用者 AUP Profile
-- ============================================

CREATE TABLE user_aup_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    training_records JSONB,
    research_experience TEXT,
    animal_experience TEXT,
    certifications JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 11. 定期報表設定
-- ============================================

CREATE TABLE scheduled_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_type report_type NOT NULL,
    schedule_type schedule_type NOT NULL,
    day_of_week INTEGER,
    day_of_month INTEGER,
    hour_of_day INTEGER NOT NULL DEFAULT 6,
    parameters JSONB,
    recipients UUID[] NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_scheduled_reports_next_run ON scheduled_reports(next_run_at) WHERE is_active = true;

-- 報表歷史記錄
CREATE TABLE report_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scheduled_report_id UUID REFERENCES scheduled_reports(id) ON DELETE SET NULL,
    report_type report_type NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size INTEGER,
    parameters JSONB,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    generated_by UUID REFERENCES users(id)
);

CREATE INDEX idx_report_history_type ON report_history(report_type);
CREATE INDEX idx_report_history_generated_at ON report_history(generated_at);

-- ============================================
-- 完成
-- ============================================
