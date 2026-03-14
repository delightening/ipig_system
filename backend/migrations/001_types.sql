-- ============================================
-- Migration 001: 自訂類型 (Custom Types)
-- ============================================

-- ERP
CREATE TYPE partner_type AS ENUM ('supplier', 'customer');
CREATE TYPE supplier_category AS ENUM ('drug', 'consumable', 'feed', 'equipment', 'other');
CREATE TYPE customer_category AS ENUM ('internal', 'external', 'research', 'other');
CREATE TYPE doc_type AS ENUM ('PO', 'GRN', 'PR', 'SO', 'DO', 'SR', 'TR', 'STK', 'ADJ', 'RM', 'RTN');
CREATE TYPE doc_status AS ENUM ('draft', 'submitted', 'approved', 'cancelled');
CREATE TYPE stock_direction AS ENUM ('in', 'out', 'transfer_in', 'transfer_out', 'adjust_in', 'adjust_out');

-- AUP
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

-- Animal
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

-- Notification & Report
CREATE TYPE notification_type AS ENUM (
    'low_stock', 'expiry_warning', 'document_approval', 'protocol_status', 'protocol_submitted',
    'review_assignment', 'review_comment', 'leave_approval', 'overtime_approval', 'vet_recommendation',
    'system_alert', 'monthly_report'
);
CREATE TYPE schedule_type AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE report_type AS ENUM ('stock_on_hand', 'stock_ledger', 'purchase_summary', 'cost_summary', 'expiry_report', 'low_stock_report');

-- HR
CREATE TYPE leave_type AS ENUM ('ANNUAL', 'PERSONAL', 'SICK', 'COMPENSATORY', 'MARRIAGE', 'BEREAVEMENT', 'MATERNITY', 'PATERNITY', 'MENSTRUAL', 'OFFICIAL');
CREATE TYPE leave_status AS ENUM ('DRAFT', 'PENDING_L1', 'PENDING_L2', 'PENDING_HR', 'PENDING_GM', 'APPROVED', 'REJECTED', 'CANCELLED', 'REVOKED');

-- Amendment
CREATE TYPE amendment_type AS ENUM ('MAJOR', 'MINOR', 'PENDING');
CREATE TYPE amendment_status AS ENUM ('DRAFT', 'SUBMITTED', 'CLASSIFIED', 'UNDER_REVIEW', 'REVISION_REQUIRED', 'RESUBMITTED', 'APPROVED', 'REJECTED', 'ADMIN_APPROVED');

-- Euthanasia
CREATE TYPE euthanasia_order_status AS ENUM ('pending_pi', 'appealed', 'chair_arbitration', 'approved', 'rejected', 'executed', 'cancelled');

-- Import/Export
CREATE TYPE import_type AS ENUM ('animal_basic', 'animal_weight');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');
CREATE TYPE export_type AS ENUM ('medical_summary', 'observation_records', 'surgery_records', 'experiment_records');
CREATE TYPE export_format AS ENUM ('pdf', 'excel');
