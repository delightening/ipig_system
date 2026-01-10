-- 通知系統擴展
-- Version: 0.5
-- 日期: 2026-01-09

-- ============================================
-- 1. 新增通知類型到 notification_type ENUM
-- ============================================

-- PostgreSQL ENUM 新增值
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'protocol_submitted';
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'review_assignment';
ALTER TYPE notification_type ADD VALUE IF NOT EXISTS 'review_comment';

-- ============================================
-- 2. 新增 link 欄位到 notifications 表
-- ============================================

ALTER TABLE notifications ADD COLUMN IF NOT EXISTS link VARCHAR(500);

-- ============================================
-- 3. 新增通知日誌表（用於追蹤發送失敗）
-- ============================================

CREATE TABLE IF NOT EXISTS notification_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_type VARCHAR(50) NOT NULL,
    recipient_email VARCHAR(255),
    recipient_user_id UUID REFERENCES users(id),
    subject VARCHAR(500),
    status VARCHAR(20) NOT NULL, -- 'pending', 'sent', 'failed', 'retry'
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notification_logs_status ON notification_logs(status);
CREATE INDEX IF NOT EXISTS idx_notification_logs_created_at ON notification_logs(created_at);

-- ============================================
-- 4. 新增通知偏好設定欄位
-- ============================================

ALTER TABLE notification_settings 
    ADD COLUMN IF NOT EXISTS email_review_assignment BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE notification_settings 
    ADD COLUMN IF NOT EXISTS email_vet_recommendation BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE notification_settings 
    ADD COLUMN IF NOT EXISTS in_app_enabled BOOLEAN NOT NULL DEFAULT true;

-- ============================================
-- 5. 新增效期緊急提醒設定
-- ============================================

ALTER TABLE notification_settings 
    ADD COLUMN IF NOT EXISTS expiry_urgent_days INTEGER NOT NULL DEFAULT 7;

-- 檢查約束
ALTER TABLE notification_settings DROP CONSTRAINT IF EXISTS chk_expiry_urgent_days;
ALTER TABLE notification_settings ADD CONSTRAINT chk_expiry_urgent_days 
    CHECK (expiry_urgent_days >= 1 AND expiry_urgent_days <= 30);

-- ============================================
-- 6. 新增通知相關權限
-- ============================================

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'notification.trigger', '觸發通知檢查', 'notification', '可手動觸發低庫存/效期檢查', NOW())
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 7. 更新 WAREHOUSE_MANAGER 角色權限
-- ============================================

-- 確保 WAREHOUSE_MANAGER 有通知觸發權限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'WAREHOUSE_MANAGER' AND p.code = 'notification.trigger'
ON CONFLICT DO NOTHING;

-- SYSTEM_ADMIN 也要有
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.code = 'SYSTEM_ADMIN' AND p.code = 'notification.trigger'
ON CONFLICT DO NOTHING;

-- ============================================
-- 8. 建立效期提醒自動清理 Function
-- ============================================

-- 清理舊通知的 Function（可由排程呼叫）
CREATE OR REPLACE FUNCTION cleanup_old_notifications(days_to_keep INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM notifications
    WHERE is_read = true 
      AND read_at < NOW() - (days_to_keep || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- 清理通知日誌
CREATE OR REPLACE FUNCTION cleanup_notification_logs(days_to_keep INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM notification_logs
    WHERE created_at < NOW() - (days_to_keep || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 9. 更新現有使用者的通知設定
-- ============================================

UPDATE notification_settings
SET 
    email_review_assignment = true,
    email_vet_recommendation = true,
    in_app_enabled = true,
    expiry_urgent_days = 7
WHERE email_review_assignment IS NULL 
   OR email_vet_recommendation IS NULL 
   OR in_app_enabled IS NULL;
