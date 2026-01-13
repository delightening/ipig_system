-- Update permission names to reflect "Animal" instead of "Pig"
UPDATE permissions SET name = '新增動物', description = '建立新的動物資料' WHERE id = 'animal.pig.create';
UPDATE permissions SET name = '查看動物', description = '查看動物列表與詳情' WHERE id = 'animal.pig.view_all';
UPDATE permissions SET name = '編輯動物', description = '修改動物基本資料' WHERE id = 'animal.pig.edit';
