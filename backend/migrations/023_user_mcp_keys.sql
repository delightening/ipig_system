-- 023: 個人 MCP API Key（連接 claude.ai MCP Server 用）
-- 有別於系統級 ai_api_keys（程式讀取用），這是個人審查接入憑證

CREATE TABLE user_mcp_keys (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash     TEXT        NOT NULL UNIQUE,   -- SHA-256 hash（明文只顯示一次）
    key_prefix   VARCHAR(20) NOT NULL,           -- 顯示用前綴，e.g. "mcp_a1b2c3d4"
    name         TEXT        NOT NULL,           -- 使用者自訂名稱
    last_used_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at   TIMESTAMPTZ                     -- NULL = 有效
);

CREATE INDEX idx_user_mcp_keys_user_id ON user_mcp_keys (user_id)
    WHERE revoked_at IS NULL;
CREATE INDEX idx_user_mcp_keys_hash ON user_mcp_keys (key_hash)
    WHERE revoked_at IS NULL;

COMMENT ON TABLE user_mcp_keys IS '個人 MCP API Key，用於 claude.ai Remote MCP 連線';
COMMENT ON COLUMN user_mcp_keys.key_hash IS 'SHA-256 hash，明文金鑰只在建立時回傳一次';
COMMENT ON COLUMN user_mcp_keys.key_prefix IS '顯示於 UI 的前綴，格式 mcp_xxxxxxxx';
