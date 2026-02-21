-- ============================================
-- Migration 010: JWT 黑名單持久化（SEC-33）
--
-- 將 JWT 撤銷紀錄持久化到 DB，
-- 避免 API 重啟時黑名單遺失
-- ============================================

CREATE TABLE IF NOT EXISTS jwt_blacklist (
    jti VARCHAR(64) PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 用於背景清理已過期項目
CREATE INDEX idx_jwt_blacklist_expires ON jwt_blacklist(expires_at);
