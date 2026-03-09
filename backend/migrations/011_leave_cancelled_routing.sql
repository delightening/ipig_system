-- 011: 新增請假取消通知路由規則
-- 當已核准的請假被取消時，通知 ADMIN_STAFF 與 admin 角色
INSERT INTO notification_routing (event_type, role_code, channel, description) VALUES
('leave_cancelled', 'ADMIN_STAFF', 'in_app', '請假取消'),
('leave_cancelled', 'admin', 'in_app', '請假取消')
ON CONFLICT DO NOTHING;
