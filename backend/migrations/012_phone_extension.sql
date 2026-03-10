-- 增加電話分機欄位 (Phone Extension)

-- 1. users 表
ALTER TABLE users ADD COLUMN phone_ext VARCHAR(20);

-- 2. partners 表
ALTER TABLE partners ADD COLUMN phone_ext VARCHAR(20);

-- 3. animal_sources 表
ALTER TABLE animal_sources ADD COLUMN phone_ext VARCHAR(20);
