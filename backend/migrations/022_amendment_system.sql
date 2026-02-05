-- ============================================
-- Migration 022: Amendment System
-- 
-- 變更申請系統：
-- - 變更申請主表 (amendments)
-- - 變更申請版本 (amendment_versions)
-- - 變更申請審查指派 (amendment_review_assignments)
-- - 變更申請狀態歷程 (amendment_status_history)
-- - 修改 review_comments 表支援草稿功能
-- ============================================

-- ============================================
-- 1. 新增自訂類型
-- ============================================

-- 變更申請類型
CREATE TYPE amendment_type AS ENUM (
    'MAJOR',    -- 重大變更（需全體審查委員審查）
    'MINOR',    -- 小變更（行政審查）
    'PENDING'   -- 待分類
);

-- 變更申請狀態
CREATE TYPE amendment_status AS ENUM (
    'DRAFT',              -- 草稿
    'SUBMITTED',          -- 已提交，待分類
    'CLASSIFIED',         -- 已分類，待審查
    'UNDER_REVIEW',       -- 審查中（重大變更）
    'REVISION_REQUIRED',  -- 需修訂
    'RESUBMITTED',        -- 已重送
    'APPROVED',           -- 核准（重大變更）
    'REJECTED',           -- 否決
    'ADMIN_APPROVED'      -- 行政審查核准（小變更）
);

-- ============================================
-- 2. 變更申請主表
-- ============================================

CREATE TABLE amendments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    amendment_no VARCHAR(30) NOT NULL UNIQUE,
    revision_number INTEGER NOT NULL,
    amendment_type amendment_type NOT NULL DEFAULT 'PENDING',
    status amendment_status NOT NULL DEFAULT 'DRAFT',
    title VARCHAR(200) NOT NULL,
    description TEXT,
    change_items TEXT[],
    changes_content JSONB,
    submitted_by UUID REFERENCES users(id),
    submitted_at TIMESTAMPTZ,
    classified_by UUID REFERENCES users(id),
    classified_at TIMESTAMPTZ,
    classification_remark TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 確保同一計畫的 revision_number 唯一
    CONSTRAINT amendments_protocol_revision_unique UNIQUE (protocol_id, revision_number)
);

CREATE INDEX idx_amendments_protocol_id ON amendments(protocol_id);
CREATE INDEX idx_amendments_status ON amendments(status);
CREATE INDEX idx_amendments_amendment_type ON amendments(amendment_type);
CREATE INDEX idx_amendments_submitted_by ON amendments(submitted_by);

COMMENT ON TABLE amendments IS '變更申請主表';
COMMENT ON COLUMN amendments.amendment_no IS '變更編號，格式：PIG-114001-R01';
COMMENT ON COLUMN amendments.revision_number IS '第幾次變更 (1, 2, 3...)';
COMMENT ON COLUMN amendments.amendment_type IS '變更類型：MAJOR=重大變更, MINOR=小變更, PENDING=待分類';
COMMENT ON COLUMN amendments.change_items IS '變更項目清單';
COMMENT ON COLUMN amendments.changes_content IS '變更詳細內容（JSONB 結構）';

-- ============================================
-- 3. 變更申請版本表
-- ============================================

CREATE TABLE amendment_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    version_no INTEGER NOT NULL,
    content_snapshot JSONB NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_by UUID NOT NULL REFERENCES users(id),
    
    CONSTRAINT amendment_versions_unique UNIQUE (amendment_id, version_no)
);

CREATE INDEX idx_amendment_versions_amendment_id ON amendment_versions(amendment_id);

COMMENT ON TABLE amendment_versions IS '變更申請版本快照';

-- ============================================
-- 4. 變更申請審查指派表
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
    
    CONSTRAINT amendment_review_assignments_unique UNIQUE (amendment_id, reviewer_id),
    CONSTRAINT amendment_decision_check CHECK (decision IS NULL OR decision IN ('APPROVE', 'REJECT', 'REVISION'))
);

CREATE INDEX idx_amendment_review_assignments_amendment_id ON amendment_review_assignments(amendment_id);
CREATE INDEX idx_amendment_review_assignments_reviewer_id ON amendment_review_assignments(reviewer_id);

COMMENT ON TABLE amendment_review_assignments IS '變更申請審查委員指派';
COMMENT ON COLUMN amendment_review_assignments.decision IS '審查決定：APPROVE=核准, REJECT=否決, REVISION=需修訂';

-- ============================================
-- 5. 變更申請狀態歷程表
-- ============================================

CREATE TABLE amendment_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    amendment_id UUID NOT NULL REFERENCES amendments(id) ON DELETE CASCADE,
    from_status amendment_status,
    to_status amendment_status NOT NULL,
    changed_by UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_amendment_status_history_amendment_id ON amendment_status_history(amendment_id);

COMMENT ON TABLE amendment_status_history IS '變更申請狀態變更歷程';

-- ============================================
-- 6. 修改 review_comments 表支援草稿功能
-- ============================================

-- 新增草稿相關欄位
ALTER TABLE review_comments 
ADD COLUMN draft_content TEXT,
ADD COLUMN drafted_by UUID REFERENCES users(id),
ADD COLUMN draft_updated_at TIMESTAMPTZ;

CREATE INDEX idx_review_comments_drafted_by ON review_comments(drafted_by) WHERE drafted_by IS NOT NULL;

COMMENT ON COLUMN review_comments.draft_content IS '回覆草稿內容（僅 PI/Coeditor 可見）';
COMMENT ON COLUMN review_comments.drafted_by IS '草稿撰寫者';
COMMENT ON COLUMN review_comments.draft_updated_at IS '草稿最後更新時間';

-- ============================================
-- 7. 新增權限
-- ============================================

-- 變更申請相關權限
INSERT INTO permissions (id, code, name, module, description) VALUES
    (gen_random_uuid(), 'amendment.create', '建立變更申請', 'AUP', '允許建立變更申請（PI）'),
    (gen_random_uuid(), 'amendment.read', '檢視變更申請', 'AUP', '允許檢視變更申請'),
    (gen_random_uuid(), 'amendment.update', '更新變更申請', 'AUP', '允許更新變更申請'),
    (gen_random_uuid(), 'amendment.classify', '分類變更申請', 'AUP', '允許分類變更申請為重大/小變更（IACUC_STAFF）'),
    (gen_random_uuid(), 'amendment.review', '審查變更申請', 'AUP', '允許審查變更申請（審查委員）'),
    (gen_random_uuid(), 'amendment.approve', '核准變更申請', 'AUP', '允許核准變更申請'),
    (gen_random_uuid(), 'amendment.admin_approve', '行政審查變更申請', 'AUP', '允許行政審查核准小變更（IACUC_STAFF）');

-- 為角色分配權限
-- PI
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'PI' AND p.code IN ('amendment.create', 'amendment.read', 'amendment.update');

-- IACUC_STAFF
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'IACUC_STAFF' AND p.code IN ('amendment.read', 'amendment.classify', 'amendment.admin_approve');

-- REVIEWER
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'REVIEWER' AND p.code IN ('amendment.read', 'amendment.review');

-- VET
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'VET' AND p.code IN ('amendment.read', 'amendment.review');

-- CHAIR
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'CHAIR' AND p.code IN ('amendment.read', 'amendment.review', 'amendment.approve');

-- SYSTEM_ADMIN
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'admin' AND p.code LIKE 'amendment.%';

-- ============================================
-- 8. 建立視圖：變更申請列表
-- ============================================

CREATE OR REPLACE VIEW amendment_list_view AS
SELECT 
    a.id,
    a.protocol_id,
    a.amendment_no,
    a.revision_number,
    a.amendment_type,
    a.status,
    a.title,
    a.description,
    a.change_items,
    a.submitted_at,
    a.classified_at,
    a.created_at,
    a.updated_at,
    p.iacuc_no AS protocol_iacuc_no,
    p.title AS protocol_title,
    u.display_name AS submitted_by_name,
    c.display_name AS classified_by_name
FROM amendments a
JOIN protocols p ON a.protocol_id = p.id
LEFT JOIN users u ON a.submitted_by = u.id
LEFT JOIN users c ON a.classified_by = c.id;

COMMENT ON VIEW amendment_list_view IS '變更申請列表視圖（含關聯資訊）';
