-- Migration 008: Reset Admin Account
-- This migration resets the admin account password to admin123
-- Email: admin@ipig.local
-- Password: admin123

-- Update admin user password (argon2id hash for 'admin123')
UPDATE users 
SET 
    password_hash = '$argon2id$v=19$m=19456,t=2,p=1$KVnFxLOfhBXicAx+BQ1k7g$9Zz0oFETd5Md5sZdMXI2AHM+XqVQrDpa+zCBukHhf0A',
    is_active = true,
    must_change_password = false,
    updated_at = NOW()
WHERE email = 'admin@ipig.local';

-- Ensure admin user exists (create if not exists)
INSERT INTO users (id, email, password_hash, display_name, is_internal, is_active, must_change_password, created_at, updated_at)
SELECT 
    gen_random_uuid(),
    'admin@ipig.local',
    '$argon2id$v=19$m=19456,t=2,p=1$KVnFxLOfhBXicAx+BQ1k7g$9Zz0oFETd5Md5sZdMXI2AHM+XqVQrDpa+zCBukHhf0A',
    '系統管理員',
    true,
    true,
    false,
    NOW(),
    NOW()
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@ipig.local');

-- Ensure admin role assignment
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u, roles r
WHERE u.email = 'admin@ipig.local' 
  AND (r.code = 'admin' OR r.code = 'SYSTEM_ADMIN')
ON CONFLICT DO NOTHING;
