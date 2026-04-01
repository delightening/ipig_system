-- ============================================================
-- Migration 001: 所有自訂 ENUM 類型 + CAST 函式
-- 來源: 001_types.sql, 009_glp_extensions.sql,
--       018_equipment_maintenance.sql, 029_qa_plan.sql
-- ============================================================

-- ── ERP ─────────────────────────────────────────────────────
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');
CREATE TYPE customer_category AS ENUM ('internal', 'external', 'research', 'other');
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RM', 'RTN');
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');

-- ── AUP ─────────────────────────────────────────────────────
CREATE TYPE protocol_role AS ENUM ('PI', 'CLIENT', 'CO_EDITOR');
CREATE TYPE protocol_activity_type AS ENUM (
    'CREATED', 'UPDATED', 'SUBMITTED', 'RESUBMITTED', 'APPROVED', 'APPROVED_WITH_CONDITIONS',
    'CLOSED', 'REJECTED', 'SUSPENDED', 'DELETED', 'STATUS_CHANGED', 'REVIEWER_ASSIGNED',
    'VET_ASSIGNED', 'COEDITOR_ASSIGNED', 'COEDITOR_REMOVED', 'COMMENT_ADDED', 'COMMENT_REPLIED',
    'COMMENT_RESOLVED', 'ATTACHMENT_UPLOADED', 'ATTACHMENT_DELETED', 'VERSION_CREATED',
    'VERSION_RECOVERED', 'AMENDMENT_CREATED', 'AMENDMENT_SUBMITTED', 'ANIMAL_ASSIGNED', 'ANIMAL_UNASSIGNED'
);
CREATE TYPE protocol_status AS ENUM (
    'DRAFT', 'SUBMITTED', 'PRE_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED',
    'VET_REVIEW', 'VET_REVISION_REQUIRED', 'UNDER_REVIEW', 'REVISION_REQUIRED',
    'RESUBMITTED', 'APPROVED', 'APPROVED_WITH_CONDITIONS', 'DEFERRED', 'REJECTED',
    'SUSPENDED', 'CLOSED', 'DELETED'
);

-- ── Animal ───────────────────────────────────────────────────
CREATE TYPE animal_status AS ENUM ('unassigned', 'in_experiment', 'completed', 'euthanized', 'sudden_death', 'transferred');
CREATE TYPE animal_breed AS ENUM ('miniature', 'white', 'LYD', 'other');
CREATE TYPE animal_gender AS ENUM ('male', 'female');
CREATE TYPE record_type AS ENUM ('abnormal', 'experiment', 'observation');
CREATE TYPE animal_record_type AS ENUM ('observation', 'surgery', 'sacrifice', 'pathology', 'blood_test');
CREATE TYPE animal_file_type AS ENUM ('photo', 'attachment', 'report');
CREATE TYPE vet_record_type AS ENUM ('observation', 'surgery');
CREATE TYPE care_record_mode AS ENUM ('legacy', 'pain_assessment');
CREATE TYPE version_record_type AS ENUM ('observation', 'surgery', 'weight', 'vaccination', 'sacrifice', 'pathology', 'blood_test');
CREATE TYPE animal_transfer_status AS ENUM ('pending', 'vet_evaluated', 'plan_assigned', 'pi_approved', 'completed', 'rejected');

-- ── Notification & Report ────────────────────────────────────
-- 包含 018 新增的 equipment_overdue / equipment_unrepairable / equipment_disposal
CREATE TYPE notification_type AS ENUM (
    'low_stock', 'expiry_warning', 'document_approval', 'protocol_status', 'protocol_submitted',
    'review_assignment', 'review_comment', 'leave_approval', 'overtime_approval', 'vet_recommendation',
    'system_alert', 'monthly_report',
    'equipment_overdue', 'equipment_unrepairable', 'equipment_disposal'
);
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE report_type AS ENUM ('stock_on_hand', 'stock_ledger', 'purchase_summary', 'cost_summary', 'expiry_report', 'low_stock_report');

-- ── HR ───────────────────────────────────────────────────────
CREATE TYPE leave_type AS ENUM ('ANNUAL', 'PERSONAL', 'SICK', 'COMPENSATORY', 'MARRIAGE', 'BEREAVEMENT', 'MATERNITY', 'PATERNITY', 'MENSTRUAL', 'OFFICIAL');
CREATE TYPE leave_status AS ENUM ('DRAFT', 'PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM', 'APPROVED', 'REJECTED', 'CANCELLED', 'REVOKED');

-- ── Amendment ────────────────────────────────────────────────
CREATE TYPE amendment_type AS ENUM ('MAJOR', 'MINOR', 'PENDING');
CREATE TYPE amendment_status AS ENUM ('DRAFT', 'SUBMITTED', 'CLASSIFIED', 'UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED', 'APPROVED', 'REJECTED', 'ADMIN_APPROVED');

-- ── Euthanasia ───────────────────────────────────────────────
CREATE TYPE euthanasia_order_status AS ENUM ('pending_pi', 'appealed', 'chair_arbitration', 'approved', 'rejected', 'executed', 'cancelled');

-- ── Import / Export ──────────────────────────────────────────
CREATE TYPE import_type AS ENUM ('animal_basic', 'animal_weight');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
CREATE TYPE export_format AS ENUM ('pdf', 'excel');

-- ── Accounting (009) ─────────────────────────────────────────
CREATE TYPE account_type AS ENUM ('asset', 'liability', 'equity', 'revenue', 'expense');

-- ── Equipment Maintenance (018) ──────────────────────────────
CREATE TYPE equipment_status   AS ENUM ('active', 'inactive', 'under_repair', 'decommissioned');
CREATE TYPE calibration_type   AS ENUM ('calibration', 'validation', 'inspection');
CREATE TYPE calibration_cycle  AS ENUM ('monthly', 'quarterly', 'semi_annual', 'annual');
CREATE TYPE maintenance_type   AS ENUM ('repair', 'maintenance');
CREATE TYPE maintenance_status AS ENUM ('pending', 'in_progress', 'completed', 'unrepairable');
CREATE TYPE disposal_status    AS ENUM ('pending', 'approved', 'rejected');

-- ── QA Plan (029) ────────────────────────────────────────────
CREATE TYPE qa_inspection_type       AS ENUM ('protocol', 'equipment', 'facility', 'training', 'general');
CREATE TYPE qa_inspection_status     AS ENUM ('draft', 'submitted', 'closed');
CREATE TYPE qa_item_result           AS ENUM ('pass', 'fail', 'not_applicable');
CREATE TYPE nc_severity              AS ENUM ('critical', 'major', 'minor');
CREATE TYPE nc_source                AS ENUM ('inspection', 'observation', 'external_audit', 'self_report');
CREATE TYPE nc_status                AS ENUM ('open', 'in_progress', 'pending_verification', 'closed');
CREATE TYPE capa_action_type         AS ENUM ('corrective', 'preventive');
CREATE TYPE capa_status              AS ENUM ('open', 'in_progress', 'completed', 'verified');
CREATE TYPE sop_status               AS ENUM ('draft', 'active', 'obsolete');
CREATE TYPE qa_schedule_type         AS ENUM ('annual', 'periodic', 'ad_hoc');
CREATE TYPE qa_schedule_status       AS ENUM ('planned', 'in_progress', 'completed', 'cancelled');
CREATE TYPE qa_schedule_item_status  AS ENUM ('planned', 'in_progress', 'completed', 'cancelled', 'overdue');

-- ── ENUM ↔ text CAST 函式 (008_supplementary) ───────────────
-- 供 sqlx/Rust 層 enum 轉換使用

CREATE OR REPLACE FUNCTION version_record_type_to_text(version_record_type) RETURNS text AS $$
    SELECT (SELECT enumlabel FROM pg_enum
            WHERE enumtypid = 'version_record_type'::regtype
            ORDER BY enumsortorder
            OFFSET (array_position(enum_range(NULL::version_record_type), $1) - 1)
            LIMIT 1);
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION text_to_version_record_type(text) RETURNS version_record_type AS $$
    SELECT r.v FROM unnest(enum_range(NULL::version_record_type)) AS r(v)
    WHERE version_record_type_to_text(r.v) = $1 LIMIT 1;
$$ LANGUAGE SQL STABLE;

CREATE OR REPLACE FUNCTION animal_record_type_to_text(animal_record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

CREATE OR REPLACE FUNCTION record_type_to_text(record_type) RETURNS text AS $$
    SELECT $1::text;
$$ LANGUAGE SQL IMMUTABLE;

DROP CAST IF EXISTS (version_record_type AS text);
DROP CAST IF EXISTS (text AS version_record_type);
DROP CAST IF EXISTS (animal_record_type AS text);
DROP CAST IF EXISTS (record_type AS text);

CREATE CAST (version_record_type AS text)   WITH FUNCTION version_record_type_to_text(version_record_type)   AS ASSIGNMENT;
CREATE CAST (text AS version_record_type)   WITH FUNCTION text_to_version_record_type(text)                  AS ASSIGNMENT;
CREATE CAST (animal_record_type AS text)    WITH FUNCTION animal_record_type_to_text(animal_record_type)     AS ASSIGNMENT;
CREATE CAST (record_type AS text)           WITH FUNCTION record_type_to_text(record_type)                   AS ASSIGNMENT;
