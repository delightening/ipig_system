-- 023: 新增 INTERN 角色 + 使用者帳號到期日
-- INTERN 擁有與 EXPERIMENT_STAFF 相同的權限，但帳號有到期日限制

-- 23.1 新增 INTERN 角色
INSERT INTO roles (id, code, name, description, is_internal, is_system, created_at, updated_at) VALUES
    (gen_random_uuid(), 'INTERN', '實習生', '實習人員，權限同試驗工作人員，帳號有到期日', true, true, NOW(), NOW())
ON CONFLICT (code) DO NOTHING;

-- 23.2 使用者帳號到期日欄位
ALTER TABLE users ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ DEFAULT NULL;

-- 為 expires_at 建立索引（登入時檢查過期帳號）
CREATE INDEX IF NOT EXISTS idx_users_expires_at ON users (expires_at) WHERE expires_at IS NOT NULL;

COMMENT ON COLUMN users.expires_at IS '帳號到期日，NULL 表示永不過期。用於實習生等臨時帳號';
