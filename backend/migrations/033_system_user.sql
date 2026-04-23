-- 033_system_user.sql
-- 為 ActorContext::System 提供 audit FK 用的固定 UUID user
--
-- Rust 端常數：middleware::actor::SYSTEM_USER_ID
-- 用途：Scheduler cron job、bin 工具、啟動期 migration 等「非使用者觸發」的
--      操作所寫入的 user_activity_logs.actor_user_id 都指向這個 user，
--      讓 FK 約束有效，同時讓稽核員能一眼區分「系統操作 vs 人為操作」。
--
-- is_active = false 確保無法被當成登入帳號使用。

INSERT INTO users (
    id,
    email,
    password_hash,
    display_name,
    is_active,
    created_at,
    updated_at
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    'system@ipig.internal',
    '!INVALID_HASH_CANNOT_LOGIN!',  -- 非 argon2 格式，登入必定失敗
    'SYSTEM',
    false,
    NOW(),
    NOW()
)
ON CONFLICT (id) DO NOTHING;

COMMENT ON COLUMN users.id IS
  'Reserved UUID 00000000-0000-0000-0000-000000000001 = SYSTEM actor for non-user-triggered audit (scheduler/bin/migration).';
