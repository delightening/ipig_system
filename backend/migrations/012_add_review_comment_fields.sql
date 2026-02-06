-- ============================================
-- Migration 012: Add Review Comment Reply and Draft Fields
-- 
-- 補齊 review_comments 表中缺失的欄位，支援回覆與草稿功能
-- ============================================

-- 新增父評論 ID (用於回覆)
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS parent_comment_id UUID REFERENCES review_comments(id) ON DELETE CASCADE;

-- 新增回覆者 ID
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS replied_by UUID REFERENCES users(id);

-- 新增草稿內容
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS draft_content TEXT;

-- 新增草稿撰寫者 ID
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS drafted_by UUID REFERENCES users(id);

-- 新增草稿更新時間
ALTER TABLE review_comments 
ADD COLUMN IF NOT EXISTS draft_updated_at TIMESTAMPTZ;

-- 新增索引
CREATE INDEX IF NOT EXISTS idx_review_comments_parent ON review_comments(parent_comment_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_drafted_by ON review_comments(drafted_by);

-- ============================================
-- 完成
-- ============================================
