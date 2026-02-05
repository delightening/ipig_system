-- Migration: 033_rename_pig_to_animal.sql
-- Description: 將所有 pig.* 權限代碼重新命名為 animal.*
-- Date: 2026-02-06

-- ============================================
-- 1. 處理重複的權限（pig.export.medical → animal.export.medical）
-- ============================================

-- 將 pig.export.medical 的角色權限轉移給 animal.export.medical
INSERT INTO role_permissions (role_id, permission_id)
SELECT rp.role_id, p2.id
FROM role_permissions rp
JOIN permissions p1 ON rp.permission_id = p1.id
JOIN permissions p2 ON p2.code = 'animal.export.medical'
WHERE p1.code = 'pig.export.medical'
ON CONFLICT DO NOTHING;

-- 刪除 pig.export.medical
DELETE FROM permissions WHERE code = 'pig.export.medical';

-- ============================================
-- 2. 重命名所有 pig.* 權限為 animal.*
-- ============================================

-- pig.pig.* → animal.animal.*
UPDATE permissions SET code = 'animal.animal.view_all', module = 'animal' WHERE code = 'pig.pig.view_all';
UPDATE permissions SET code = 'animal.animal.view_project', module = 'animal' WHERE code = 'pig.pig.view_project';
UPDATE permissions SET code = 'animal.animal.create', module = 'animal' WHERE code = 'pig.pig.create';
UPDATE permissions SET code = 'animal.animal.edit', module = 'animal' WHERE code = 'pig.pig.edit';
UPDATE permissions SET code = 'animal.animal.assign', module = 'animal' WHERE code = 'pig.pig.assign';
UPDATE permissions SET code = 'animal.animal.import', module = 'animal' WHERE code = 'pig.pig.import';
UPDATE permissions SET code = 'animal.animal.delete', module = 'animal' WHERE code = 'pig.pig.delete';

-- pig.record.* → animal.record.*
UPDATE permissions SET code = 'animal.record.view', module = 'animal' WHERE code = 'pig.record.view';
UPDATE permissions SET code = 'animal.record.create', module = 'animal' WHERE code = 'pig.record.create';
UPDATE permissions SET code = 'animal.record.edit', module = 'animal' WHERE code = 'pig.record.edit';
UPDATE permissions SET code = 'animal.record.delete', module = 'animal' WHERE code = 'pig.record.delete';
UPDATE permissions SET code = 'animal.record.observation', module = 'animal' WHERE code = 'pig.record.observation';
UPDATE permissions SET code = 'animal.record.surgery', module = 'animal' WHERE code = 'pig.record.surgery';
UPDATE permissions SET code = 'animal.record.weight', module = 'animal' WHERE code = 'pig.record.weight';
UPDATE permissions SET code = 'animal.record.vaccine', module = 'animal' WHERE code = 'pig.record.vaccine';
UPDATE permissions SET code = 'animal.record.sacrifice', module = 'animal' WHERE code = 'pig.record.sacrifice';

-- pig.export.* → animal.export.*
UPDATE permissions SET code = 'animal.export.observation', module = 'animal' WHERE code = 'pig.export.observation';
UPDATE permissions SET code = 'animal.export.surgery', module = 'animal' WHERE code = 'pig.export.surgery';
UPDATE permissions SET code = 'animal.export.experiment', module = 'animal' WHERE code = 'pig.export.experiment';

-- pig.vet.* → animal.vet.*
UPDATE permissions SET code = 'animal.vet.recommend', module = 'animal' WHERE code = 'pig.vet.recommend';
UPDATE permissions SET code = 'animal.vet.read', module = 'animal' WHERE code = 'pig.vet.read';

-- ============================================
-- 3. 更新描述中的「豬隻」為「動物」
-- ============================================
UPDATE permissions SET description = REPLACE(description, '豬隻', '動物') WHERE code LIKE 'animal.%';

-- ============================================
-- 完成
-- ============================================
