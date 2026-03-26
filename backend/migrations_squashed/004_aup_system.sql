-- ============================================
-- Migration 004: AUP Protocol System
-- Squashed from: 005, 008(partial), 014
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
    version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_protocols_status ON protocols(status);
CREATE INDEX idx_protocols_pi_user_id ON protocols(pi_user_id);
CREATE INDEX idx_protocols_iacuc_no ON protocols(iacuc_no);
CREATE INDEX idx_protocols_created_by ON protocols(created_by);
CREATE INDEX idx_protocols_working_content ON protocols USING GIN (working_content);
CREATE INDEX idx_protocols_status_pi_created ON protocols(status, pi_user_id, created_at DESC);

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

CREATE TABLE review_assignments (
    id UUID PRIMARY KEY,
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    is_primary_reviewer BOOLEAN NOT NULL DEFAULT false,
    review_stage VARCHAR(20) DEFAULT 'UNDER_REVIEW',
    decision VARCHAR(20),
    decided_at TIMESTAMPTZ,
    UNIQUE (protocol_id, reviewer_id)
);
CREATE INDEX idx_review_assignments_protocol_id ON review_assignments(protocol_id);
CREATE INDEX idx_review_assignments_reviewer_id ON review_assignments(reviewer_id);

CREATE TABLE review_comments (
    id UUID PRIMARY KEY,
    protocol_version_id UUID REFERENCES protocol_versions(id) ON DELETE CASCADE,
    protocol_id UUID REFERENCES protocols(id),
    reviewer_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    review_stage VARCHAR(20),
    parent_comment_id UUID REFERENCES review_comments(id) ON DELETE CASCADE,
    replied_by UUID REFERENCES users(id),
    draft_content TEXT,
    drafted_by UUID REFERENCES users(id),
    draft_updated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_review_stage CHECK (review_stage IS NULL OR review_stage IN (
        'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED', 'VET_REVIEW', 'VET_REVISION_REQUIRED', 'UNDER_REVIEW'
    )),
    CONSTRAINT chk_protocol_reference CHECK (protocol_version_id IS NOT NULL OR protocol_id IS NOT NULL)
);
CREATE INDEX idx_review_comments_protocol_version_id ON review_comments(protocol_version_id);
CREATE INDEX idx_review_comments_reviewer_id ON review_comments(reviewer_id);
CREATE INDEX idx_review_comments_protocol_id ON review_comments(protocol_id);
CREATE INDEX idx_review_comments_review_stage ON review_comments(review_stage);
CREATE INDEX idx_review_comments_parent ON review_comments(parent_comment_id);
CREATE INDEX idx_review_comments_drafted_by ON review_comments(drafted_by);

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

CREATE TABLE amendment_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    UNIQUE (amendment_id, version_no)
);
CREATE INDEX idx_amendment_versions_amendment_id ON amendment_versions(amendment_id);

CREATE TABLE amendment_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    from_status amendment_status,
    to_status amendment_status NOT NULL,
    changed_by UUID NOT NULL,
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_amendment_status_history_amendment_id ON amendment_status_history(amendment_id);

-- user_aup_profiles 已在 002_users_auth.sql 中建立

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

-- system_settings 已在 002_users_auth.sql 中建立（含 seed data）

CREATE TABLE vet_review_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    vet_id UUID NOT NULL REFERENCES users(id),
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    decision VARCHAR(20),
    decision_remark TEXT,
    review_form JSONB,
    UNIQUE (protocol_id)
);
CREATE INDEX idx_vet_review_assignments_protocol ON vet_review_assignments(protocol_id);
CREATE INDEX idx_vet_review_assignments_vet ON vet_review_assignments(vet_id);

CREATE TABLE protocol_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    activity_type protocol_activity_type NOT NULL,
    actor_id UUID NOT NULL REFERENCES users(id),
    actor_name VARCHAR(100),
    actor_email VARCHAR(255),
    from_value TEXT,
    to_value TEXT,
    target_entity_type VARCHAR(50),
    target_entity_id UUID,
    target_entity_name VARCHAR(255),
    remark TEXT,
    extra_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_protocol_activities_protocol_id ON protocol_activities(protocol_id, created_at DESC);
CREATE INDEX idx_protocol_activities_actor_id ON protocol_activities(actor_id);
CREATE INDEX idx_protocol_activities_type ON protocol_activities(activity_type, created_at DESC);

CREATE TABLE review_round_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    review_stage VARCHAR(30) NOT NULL,
    round_number INTEGER NOT NULL DEFAULT 1,
    action VARCHAR(30) NOT NULL,
    actor_id UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_review_round_history_protocol ON review_round_history(protocol_id);
CREATE INDEX idx_review_round_history_stage ON review_round_history(review_stage);
