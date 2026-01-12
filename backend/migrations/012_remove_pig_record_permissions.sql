-- Remove pig.record.* permissions, keep animal.record.*

INSERT INTO permissions (id, code, name, module, description, created_at) VALUES
    (gen_random_uuid(), 'animal.record.view', '查看紀錄', 'pig', '可查看豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.create', '新增紀錄', 'pig', '可新增豬隻相關紀錄（觀察、手術、體重等）', NOW()),
    (gen_random_uuid(), 'animal.record.edit', '編輯紀錄', 'pig', '可編輯豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.delete', '刪除紀錄', 'pig', '可刪除豬隻相關紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.copy', '複製紀錄', 'pig', '可複製觀察/手術紀錄作為範本', NOW()),
    (gen_random_uuid(), 'animal.record.observation', '觀察紀錄', 'pig', '可新增/編輯觀察試驗紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.surgery', '手術紀錄', 'pig', '可新增/編輯手術紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.weight', '體重紀錄', 'pig', '可新增/編輯體重紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.vaccine', '疫苗紀錄', 'pig', '可新增/編輯疫苗紀錄', NOW()),
    (gen_random_uuid(), 'animal.record.sacrifice', '犧牲紀錄', 'pig', '可新增/編輯犧牲紀錄', NOW())
ON CONFLICT (code) DO UPDATE SET
    name = EXCLUDED.name,
    module = EXCLUDED.module,
    description = EXCLUDED.description;

DELETE FROM role_permissions rp
USING permissions p
WHERE rp.permission_id = p.id
  AND p.code LIKE 'pig.record.%';

DELETE FROM permissions
WHERE code LIKE 'pig.record.%';
