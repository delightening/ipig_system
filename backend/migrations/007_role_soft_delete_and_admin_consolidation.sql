-- Add soft-delete flag for roles and consolidate admin roles

ALTER TABLE roles ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- Consolidate SYSTEM_ADMIN into admin
WITH admin_role AS (
    SELECT id FROM roles WHERE code = 'admin' LIMIT 1
),
system_admin_role AS (
    SELECT id FROM roles WHERE code = 'SYSTEM_ADMIN' LIMIT 1
)
INSERT INTO user_roles (user_id, role_id, assigned_at, assigned_by)
SELECT ur.user_id, (SELECT id FROM admin_role), NOW(), ur.assigned_by
FROM user_roles ur
WHERE ur.role_id = (SELECT id FROM system_admin_role)
ON CONFLICT DO NOTHING;

-- Remove SYSTEM_ADMIN role if it exists
DELETE FROM role_permissions
WHERE role_id = (SELECT id FROM roles WHERE code = 'SYSTEM_ADMIN');

DELETE FROM user_roles
WHERE role_id = (SELECT id FROM roles WHERE code = 'SYSTEM_ADMIN');

DELETE FROM roles
WHERE code = 'SYSTEM_ADMIN';
