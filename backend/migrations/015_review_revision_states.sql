-- ============================================
-- Migration 015: AUP 審查流程多輪往返支援
-- 
-- 變更內容：
-- 1. 新增 PRE_REVIEW_REVISION_REQUIRED 狀態（行政預審補件）
-- 2. 新增 VET_REVISION_REQUIRED 狀態（獸醫要求修訂）
-- 3. 新增審查往返歷史記錄表
-- ============================================

-- ============================================
-- 1. 新增新的狀態到 protocol_status enum
-- ============================================

-- 新增行政預審修訂狀態
ALTER TYPE protocol_status ADD VALUE IF NOT EXISTS 'PRE_REVIEW_REVISION_REQUIRED';

-- 新增獸醫審查修訂狀態
ALTER TYPE protocol_status ADD VALUE IF NOT EXISTS 'VET_REVISION_REQUIRED';

-- ============================================
-- 2. 更新 review_comments 的 review_stage 約束
-- ============================================

-- 先移除舊的約束（如果存在）
ALTER TABLE review_comments DROP CONSTRAINT IF EXISTS chk_review_stage;

-- 新增更新後的約束，包含新的審查階段
ALTER TABLE review_comments 
ADD CONSTRAINT chk_review_stage 
CHECK (review_stage IS NULL OR review_stage IN (
    'PRE_REVIEW', 
    'PRE_REVIEW_REVISION_REQUIRED',
    'VET_REVIEW', 
    'VET_REVISION_REQUIRED',
    'UNDER_REVIEW'
));

-- ============================================
-- 3. 新增審查往返歷史記錄表
-- ============================================

CREATE TABLE IF NOT EXISTS review_round_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    protocol_id UUID NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    review_stage VARCHAR(30) NOT NULL,  -- PRE_REVIEW, VET_REVIEW, UNDER_REVIEW
    round_number INTEGER NOT NULL DEFAULT 1,
    action VARCHAR(30) NOT NULL,  -- REQUESTED_REVISION, PI_RESUBMITTED, APPROVED
    actor_id UUID NOT NULL REFERENCES users(id),
    remark TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 審查往返歷史索引
CREATE INDEX IF NOT EXISTS idx_review_round_history_protocol ON review_round_history(protocol_id);
CREATE INDEX IF NOT EXISTS idx_review_round_history_stage ON review_round_history(review_stage);

-- ============================================
-- 完成
-- ============================================
