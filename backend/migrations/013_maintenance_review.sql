-- 維修/保養紀錄加入「待驗收」狀態與簽核欄位

-- 新增 pending_review 狀態值
ALTER TYPE maintenance_status ADD VALUE IF NOT EXISTS 'pending_review';

-- 維修紀錄加入驗收欄位
ALTER TABLE equipment_maintenance_records
  ADD COLUMN IF NOT EXISTS reviewed_by            UUID        REFERENCES users(id),
  ADD COLUMN IF NOT EXISTS reviewed_at             TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS reviewer_signature_id   UUID        REFERENCES electronic_signatures(id),
  ADD COLUMN IF NOT EXISTS review_notes            TEXT;

-- 新增驗收權限
INSERT INTO permissions (id, code, name, module, description) VALUES
  (gen_random_uuid(), 'equipment.maintenance.review', '驗收維修保養紀錄', 'equipment', '可驗收維修保養紀錄並簽核')
ON CONFLICT (code) DO NOTHING;

-- 新增通知路由（帶 description 欄位，避免 NOT NULL 問題）
INSERT INTO notification_routing (event_type, role_code, channel, description)
SELECT 'equipment_maintenance_review', 'EQUIPMENT_MAINTENANCE', 'in_app', '維修/保養待驗收通知'
WHERE NOT EXISTS (
  SELECT 1 FROM notification_routing
  WHERE event_type = 'equipment_maintenance_review' AND role_code = 'EQUIPMENT_MAINTENANCE'
);

INSERT INTO notification_routing (event_type, role_code, channel, description)
SELECT 'equipment_maintenance_review', 'admin', 'both', '維修/保養待驗收通知'
WHERE NOT EXISTS (
  SELECT 1 FROM notification_routing
  WHERE event_type = 'equipment_maintenance_review' AND role_code = 'admin'
);
