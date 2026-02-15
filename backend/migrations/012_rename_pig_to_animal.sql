-- ============================================
-- Migration 012: 豬隻→動物命名重構
-- 
-- 將所有 pig_* 命名統一為 animal_*
-- 包含 enum 型別、表名、索引、欄位重命名
-- 
-- ⚠️ 執行前請先備份資料庫！
-- ============================================

-- ============================================
-- 1. Enum 型別重命名
-- ============================================

ALTER TYPE pig_status RENAME TO animal_status;
ALTER TYPE pig_breed RENAME TO animal_breed;
ALTER TYPE pig_gender RENAME TO animal_gender;
ALTER TYPE pig_record_type RENAME TO animal_record_type;
ALTER TYPE pig_file_type RENAME TO animal_file_type;

-- ============================================
-- 2. import_type enum 值更新
-- ============================================

ALTER TYPE import_type RENAME VALUE 'pig_basic' TO 'animal_basic';
ALTER TYPE import_type RENAME VALUE 'pig_weight' TO 'animal_weight';

-- ============================================
-- 3. 表名重命名
-- ============================================

ALTER TABLE pig_sources RENAME TO animal_sources;
ALTER TABLE pigs RENAME TO animals;
ALTER TABLE pig_observations RENAME TO animal_observations;
ALTER TABLE pig_surgeries RENAME TO animal_surgeries;
ALTER TABLE pig_weights RENAME TO animal_weights;
ALTER TABLE pig_vaccinations RENAME TO animal_vaccinations;
ALTER TABLE pig_sacrifices RENAME TO animal_sacrifices;
ALTER TABLE pig_pathology_reports RENAME TO animal_pathology_reports;
ALTER TABLE pig_record_attachments RENAME TO animal_record_attachments;
ALTER TABLE pig_import_batches RENAME TO animal_import_batches;

-- ============================================
-- 4. 欄位重命名 (pig_id → animal_id)
-- ============================================

ALTER TABLE animal_observations RENAME COLUMN pig_id TO animal_id;
ALTER TABLE animal_surgeries RENAME COLUMN pig_id TO animal_id;
ALTER TABLE animal_weights RENAME COLUMN pig_id TO animal_id;
ALTER TABLE animal_vaccinations RENAME COLUMN pig_id TO animal_id;
ALTER TABLE animal_sacrifices RENAME COLUMN pig_id TO animal_id;
ALTER TABLE animal_pathology_reports RENAME COLUMN pig_id TO animal_id;
ALTER TABLE euthanasia_orders RENAME COLUMN pig_id TO animal_id;

-- ============================================
-- 5. 索引重命名
-- ============================================

-- animals 表（原 pigs）
ALTER INDEX idx_pigs_ear_tag RENAME TO idx_animals_ear_tag;
ALTER INDEX idx_pigs_status RENAME TO idx_animals_status;
ALTER INDEX idx_pigs_iacuc_no RENAME TO idx_animals_iacuc_no;
ALTER INDEX idx_pigs_pen_location RENAME TO idx_animals_pen_location;
ALTER INDEX idx_pigs_is_deleted RENAME TO idx_animals_is_deleted;
ALTER INDEX idx_pigs_glp_study_no RENAME TO idx_animals_glp_study_no;

-- animal_observations 表（原 pig_observations）
ALTER INDEX idx_pig_observations_pig_id RENAME TO idx_animal_observations_animal_id;
ALTER INDEX idx_pig_observations_event_date RENAME TO idx_animal_observations_event_date;

-- animal_surgeries 表（原 pig_surgeries）
ALTER INDEX idx_pig_surgeries_pig_id RENAME TO idx_animal_surgeries_animal_id;
ALTER INDEX idx_pig_surgeries_surgery_date RENAME TO idx_animal_surgeries_surgery_date;

-- animal_weights 表（原 pig_weights）
ALTER INDEX idx_pig_weights_pig_id RENAME TO idx_animal_weights_animal_id;
ALTER INDEX idx_pig_weights_measure_date RENAME TO idx_animal_weights_measure_date;

-- animal_vaccinations 表（原 pig_vaccinations）
ALTER INDEX idx_pig_vaccinations_pig_id RENAME TO idx_animal_vaccinations_animal_id;

-- animal_sacrifices 表（原 pig_sacrifices）
ALTER INDEX idx_pig_sacrifices_pig_id RENAME TO idx_animal_sacrifices_animal_id;

-- animal_record_attachments 表（原 pig_record_attachments）
ALTER INDEX idx_pig_record_attachments_record RENAME TO idx_animal_record_attachments_record;

-- animal_import_batches 表（原 pig_import_batches）
ALTER INDEX idx_pig_import_batches_status RENAME TO idx_animal_import_batches_status;
ALTER INDEX idx_pig_import_batches_created_by RENAME TO idx_animal_import_batches_created_by;
ALTER INDEX idx_pig_import_batches_created_at RENAME TO idx_animal_import_batches_created_at;

-- euthanasia_orders 表
ALTER INDEX idx_euthanasia_orders_pig_id RENAME TO idx_euthanasia_orders_animal_id;

-- ============================================
-- 完成
-- ============================================
