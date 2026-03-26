-- 022: 補建高頻查詢路徑的 FK 索引
-- 來源：2026-03-26 Code Review (H2)
--
-- 策略：只為高頻查詢路徑建立索引，跳過低頻的 created_by/deleted_by -> users(id)
-- 這些 FK 主要用於 ON DELETE CASCADE，反向查詢極少。

-- ============================================================
-- ERP: documents 表 — 進銷存單據查詢頻繁
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_documents_warehouse_id ON documents(warehouse_id);
CREATE INDEX IF NOT EXISTS idx_documents_warehouse_from_id ON documents(warehouse_from_id);
CREATE INDEX IF NOT EXISTS idx_documents_warehouse_to_id ON documents(warehouse_to_id);
CREATE INDEX IF NOT EXISTS idx_documents_partner_id ON documents(partner_id);
CREATE INDEX IF NOT EXISTS idx_documents_source_doc_id ON documents(source_doc_id);

-- ============================================================
-- ERP: document_lines / stock_ledger
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_document_lines_storage_location_id ON document_lines(storage_location_id);
CREATE INDEX IF NOT EXISTS idx_stock_ledger_line_id ON stock_ledger(line_id);
CREATE INDEX IF NOT EXISTS idx_product_uom_conversions_product_id ON product_uom_conversions(product_id);

-- ============================================================
-- 動物管理: 安樂死流程 — 獸醫/PI 查詢
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_euthanasia_orders_vet_user_id ON euthanasia_orders(vet_user_id);
CREATE INDEX IF NOT EXISTS idx_euthanasia_orders_pi_user_id ON euthanasia_orders(pi_user_id);
CREATE INDEX IF NOT EXISTS idx_animal_pathology_reports_animal_id ON animal_pathology_reports(animal_id);

-- ============================================================
-- 計畫書: amendment 查詢鏈
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_amendment_versions_amendment_id ON amendment_versions(amendment_id);
CREATE INDEX IF NOT EXISTS idx_amendment_status_history_amendment_id ON amendment_status_history(amendment_id);

-- ============================================================
-- HR: 請假/加班 關聯查詢
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_overtime_records_attendance_id ON overtime_records(attendance_id);
CREATE INDEX IF NOT EXISTS idx_leave_requests_annual_leave_source_id ON leave_requests(annual_leave_source_id);
CREATE INDEX IF NOT EXISTS idx_leave_balance_usage_entitlement_id ON leave_balance_usage(annual_leave_entitlement_id);
CREATE INDEX IF NOT EXISTS idx_leave_balance_usage_comp_time_id ON leave_balance_usage(comp_time_balance_id);

-- ============================================================
-- 安全: security_alerts 查詢
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_security_alerts_user_id ON security_alerts(user_id);
CREATE INDEX IF NOT EXISTS idx_security_alerts_login_event_id ON security_alerts(login_event_id);

-- ============================================================
-- Session: refresh_token 關聯
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_user_sessions_refresh_token_id ON user_sessions(refresh_token_id);

-- ============================================================
-- 設施層級: 查詢子項時用到 parent FK
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_buildings_facility_id ON buildings(facility_id);
CREATE INDEX IF NOT EXISTS idx_zones_building_id ON zones(building_id);
CREATE INDEX IF NOT EXISTS idx_pens_zone_id ON pens(zone_id);
CREATE INDEX IF NOT EXISTS idx_departments_parent_id ON departments(parent_id);
CREATE INDEX IF NOT EXISTS idx_species_parent_id ON species(parent_id);
CREATE INDEX IF NOT EXISTS idx_product_categories_parent_id ON product_categories(parent_id);

-- ============================================================
-- 電子簽章: 簽署人查詢
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_electronic_signatures_signer_id ON electronic_signatures(signer_id);

-- ============================================================
-- 設備維護: 供應商/校正查詢
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_equipment_calibrations_partner_id ON equipment_calibrations(partner_id);
CREATE INDEX IF NOT EXISTS idx_equipment_maintenance_repair_partner ON equipment_maintenance_records(repair_partner_id);

-- ============================================================
-- 藥品選項: 產品關聯
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_treatment_drug_options_product_id ON treatment_drug_options(erp_product_id);

-- ============================================================
-- M5: AI API Key rate_limit CHECK constraint
-- ============================================================
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'chk_rate_limit_positive') THEN
        ALTER TABLE ai_api_keys
            ADD CONSTRAINT chk_rate_limit_positive CHECK (rate_limit_per_minute IS NULL OR rate_limit_per_minute > 0);
    END IF;
END $$;

-- ============================================================
-- M12: Audit log 複合索引（改善 partition pruning）
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_activity_actor_created ON user_activity_logs(actor_user_id, created_at DESC);

-- ============================================================
-- M4: Boolean 索引改 partial index（更高效的 soft delete 查詢）
-- ============================================================
-- 先刪除原有的全量 boolean 索引，再建立 partial index
DROP INDEX IF EXISTS idx_animals_is_deleted;
CREATE INDEX IF NOT EXISTS idx_animals_not_deleted ON animals(id) WHERE is_deleted = false;
DROP INDEX IF EXISTS idx_animals_deleted_at;
CREATE INDEX IF NOT EXISTS idx_animals_active ON animals(deleted_at) WHERE deleted_at IS NULL;
