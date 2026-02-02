-- ============================================
-- Migration 021: Emergency Medication, Euthanasia & Review System
-- 
-- 包含：
-- - 安樂死單據相關表格
-- - 審查委員決議表格
-- - 全員會議請求表格
-- - 通知類型擴充
-- ============================================

-- ============================================
-- 1. 自訂類型 (Custom Types)
-- ============================================

-- 安樂死單據狀態
CREATE TYPE euthanasia_order_status AS ENUM (
    'pending_pi',        -- 等待 PI 回應
    'approved',          -- PI 同意執行
    'appealed',          -- PI 申請暫緩
    'chair_arbitration', -- CHAIR 仲裁中
    'executed',          -- 已執行
    'cancelled'          -- 已取消
);

-- ============================================
-- 2. 安樂死單據表
-- ============================================

CREATE TABLE euthanasia_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pig_id UUID NOT NULL REFERENCES pigs(id) ON DELETE CASCADE,
    vet_user_id UUID NOT NULL REFERENCES users(id),
    pi_user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status euthanasia_order_status NOT NULL DEFAULT 'pending_pi',
    deadline_at TIMESTAMPTZ NOT NULL,           -- 24 小時後
    pi_responded_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ,
    executed_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_euthanasia_orders_pig_id ON euthanasia_orders(pig_id);
CREATE INDEX idx_euthanasia_orders_status ON euthanasia_orders(status);
CREATE INDEX idx_euthanasia_orders_deadline ON euthanasia_orders(deadline_at);
CREATE INDEX idx_euthanasia_orders_pi_user_id ON euthanasia_orders(pi_user_id);

-- ============================================
-- 3. 安樂死暫緩申請表
-- ============================================

CREATE TABLE euthanasia_appeals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES euthanasia_orders(id) ON DELETE CASCADE,
    pi_user_id UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    attachment_path VARCHAR(500),
    chair_user_id UUID REFERENCES users(id),
    chair_decision VARCHAR(20),                  -- 'approve_appeal', 'reject_appeal'
    chair_decided_at TIMESTAMPTZ,
    chair_deadline_at TIMESTAMPTZ,              -- CHAIR 裁決期限
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_euthanasia_appeals_order_id ON euthanasia_appeals(order_id);
CREATE INDEX idx_euthanasia_appeals_chair_deadline ON euthanasia_appeals(chair_deadline_at);

-- ============================================
-- 4. 審查委員決議表（追蹤共識決）
-- ============================================

CREATE TABLE reviewer_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_version_id UUID NOT NULL REFERENCES protocol_versions(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES users(id),
    decision VARCHAR(20) NOT NULL,               -- 'approve', 'revision_required', 'reject'
    comment TEXT,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (protocol_version_id, reviewer_id)
);

CREATE INDEX idx_reviewer_decisions_protocol_version ON reviewer_decisions(protocol_version_id);
CREATE INDEX idx_reviewer_decisions_reviewer ON reviewer_decisions(reviewer_id);

-- ============================================
-- 5. 全員會議請求表
-- ============================================

CREATE TABLE meeting_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    requested_by UUID NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'scheduled', 'completed', 'cancelled'
    meeting_date TIMESTAMPTZ,
    chair_decision TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_meeting_requests_protocol_id ON meeting_requests(protocol_id);
CREATE INDEX idx_meeting_requests_status ON meeting_requests(status);

-- ============================================
-- 6. 擴充通知類型
-- ============================================

-- 使用 DO block 來安全地新增 enum 值（如果不存在）
DO $$
BEGIN
    -- 新增 emergency_medication 通知類型
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'emergency_medication' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'emergency_medication';
    END IF;
END$$;

DO $$
BEGIN
    -- 新增 euthanasia_order 通知類型
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'euthanasia_order' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'euthanasia_order';
    END IF;
END$$;

DO $$
BEGIN
    -- 新增 euthanasia_appeal 通知類型
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'euthanasia_appeal' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'euthanasia_appeal';
    END IF;
END$$;

DO $$
BEGIN
    -- 新增 meeting_request 通知類型
    IF NOT EXISTS (SELECT 1 FROM pg_enum WHERE enumlabel = 'meeting_request' 
                   AND enumtypid = 'notification_type'::regtype) THEN
        ALTER TYPE notification_type ADD VALUE 'meeting_request';
    END IF;
END$$;

-- ============================================
-- 7. 新增 review_comments 回覆欄位（如果不存在）
-- ============================================

-- 新增 parent_comment_id 欄位以支援回覆功能
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'review_comments' AND column_name = 'parent_comment_id') THEN
        ALTER TABLE review_comments ADD COLUMN parent_comment_id UUID REFERENCES review_comments(id);
    END IF;
END$$;

-- 新增 replied_by 欄位
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'review_comments' AND column_name = 'replied_by') THEN
        ALTER TABLE review_comments ADD COLUMN replied_by UUID REFERENCES users(id);
    END IF;
END$$;
