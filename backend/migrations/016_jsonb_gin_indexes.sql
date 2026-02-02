-- ============================================
-- Migration 016: JSONB GIN Indexes
-- 
-- 為常用的 JSONB 欄位建立 GIN 索引以提升查詢效能
-- ============================================

-- ============================================
-- 1. 高優先級索引 (頻繁查詢的欄位)
-- ============================================

-- user_activity_logs 表
-- actor_roles: 用於查詢特定角色的活動日誌
CREATE INDEX IF NOT EXISTS idx_user_activity_logs_actor_roles_gin 
ON user_activity_logs USING GIN (actor_roles);

-- before_data/after_data: 用於查詢特定欄位變更，使用 jsonb_path_ops 獲得更好效能
CREATE INDEX IF NOT EXISTS idx_user_activity_logs_before_data_gin 
ON user_activity_logs USING GIN (before_data jsonb_path_ops);

CREATE INDEX IF NOT EXISTS idx_user_activity_logs_after_data_gin 
ON user_activity_logs USING GIN (after_data jsonb_path_ops);

-- audit_logs 表
-- 稽核日誌的變更資料查詢
CREATE INDEX IF NOT EXISTS idx_audit_logs_before_data_gin 
ON audit_logs USING GIN (before_data jsonb_path_ops);

CREATE INDEX IF NOT EXISTS idx_audit_logs_after_data_gin 
ON audit_logs USING GIN (after_data jsonb_path_ops);

-- protocols 表
-- 計畫書內容搜尋
CREATE INDEX IF NOT EXISTS idx_protocols_working_content_gin 
ON protocols USING GIN (working_content);

-- ============================================
-- 2. 中優先級索引 (偶爾查詢的欄位)
-- ============================================

-- pig_observations 表
-- 設備和治療記錄查詢
CREATE INDEX IF NOT EXISTS idx_pig_observations_equipment_gin 
ON pig_observations USING GIN (equipment_used);

CREATE INDEX IF NOT EXISTS idx_pig_observations_treatments_gin 
ON pig_observations USING GIN (treatments);

-- pig_surgeries 表
-- 手術藥物紀錄查詢
CREATE INDEX IF NOT EXISTS idx_pig_surgeries_induction_anesthesia_gin 
ON pig_surgeries USING GIN (induction_anesthesia);

CREATE INDEX IF NOT EXISTS idx_pig_surgeries_pre_surgery_medication_gin 
ON pig_surgeries USING GIN (pre_surgery_medication);

CREATE INDEX IF NOT EXISTS idx_pig_surgeries_anesthesia_maintenance_gin 
ON pig_surgeries USING GIN (anesthesia_maintenance);

CREATE INDEX IF NOT EXISTS idx_pig_surgeries_vital_signs_gin 
ON pig_surgeries USING GIN (vital_signs);

CREATE INDEX IF NOT EXISTS idx_pig_surgeries_post_surgery_medication_gin 
ON pig_surgeries USING GIN (post_surgery_medication);

-- documents 表
-- 盤點範圍查詢
CREATE INDEX IF NOT EXISTS idx_documents_stocktake_scope_gin 
ON documents USING GIN (stocktake_scope);

-- security_alerts 表
-- 安全警報上下文查詢
CREATE INDEX IF NOT EXISTS idx_security_alerts_context_data_gin 
ON security_alerts USING GIN (context_data);

-- ============================================
-- 3. 低優先級索引 (較少查詢的欄位)
-- ============================================

-- user_activity_aggregates 表
-- 聚合資料查詢
CREATE INDEX IF NOT EXISTS idx_user_activity_aggregates_actions_gin 
ON user_activity_aggregates USING GIN (actions_by_category);

CREATE INDEX IF NOT EXISTS idx_user_activity_aggregates_pages_gin 
ON user_activity_aggregates USING GIN (pages_visited);

CREATE INDEX IF NOT EXISTS idx_user_activity_aggregates_entities_gin 
ON user_activity_aggregates USING GIN (entities_modified);

-- scheduled_reports 表
-- 報表參數查詢
CREATE INDEX IF NOT EXISTS idx_scheduled_reports_parameters_gin 
ON scheduled_reports USING GIN (parameters);

-- record_versions 表
-- 版本快照查詢，使用 jsonb_path_ops 因為主要是 containment 查詢
CREATE INDEX IF NOT EXISTS idx_record_versions_snapshot_gin 
ON record_versions USING GIN (snapshot jsonb_path_ops);

-- protocol_versions 表
-- 計畫版本快照查詢
CREATE INDEX IF NOT EXISTS idx_protocol_versions_content_snapshot_gin 
ON protocol_versions USING GIN (content_snapshot jsonb_path_ops);

-- ============================================
-- 索引說明
-- ============================================

COMMENT ON INDEX idx_user_activity_logs_actor_roles_gin IS 'GIN index for querying user activities by role';
COMMENT ON INDEX idx_audit_logs_before_data_gin IS 'GIN index for querying audit log changes using containment operator';
COMMENT ON INDEX idx_protocols_working_content_gin IS 'GIN index for searching protocol content';

-- ============================================
-- 完成
-- ============================================
