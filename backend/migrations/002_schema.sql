--
-- PostgreSQL database dump
--


-- Dumped from database version 16.13
-- Dumped by pg_dump version 16.13

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'SQL_ASCII';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: pg_stat_statements; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_stat_statements WITH SCHEMA public;


--
-- Name: EXTENSION pg_stat_statements; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pg_stat_statements IS 'track planning and execution statistics of all SQL statements executed';


--
-- Name: pg_trgm; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;


--
-- Name: EXTENSION pg_trgm; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pg_trgm IS 'text similarity measurement and index searching based on trigrams';


--
-- Name: account_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.account_type AS ENUM (
    'asset',
    'liability',
    'equity',
    'revenue',
    'expense'
);


--
-- Name: amendment_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.amendment_status AS ENUM (
    'DRAFT',
    'SUBMITTED',
    'CLASSIFIED',
    'UNDER_REVIEW',
    'REVISION_REQUIRED',
    'RESUBMITTED',
    'APPROVED',
    'REJECTED',
    'ADMIN_APPROVED',
    'EFFECTIVE'
);


--
-- Name: amendment_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.amendment_type AS ENUM (
    'MAJOR',
    'MINOR',
    'PENDING'
);


--
-- Name: animal_breed; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.animal_breed AS ENUM (
    'miniature',
    'white',
    'LYD',
    'other'
);


--
-- Name: animal_file_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.animal_file_type AS ENUM (
    'photo',
    'attachment',
    'report'
);


--
-- Name: animal_gender; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.animal_gender AS ENUM (
    'male',
    'female'
);


--
-- Name: animal_record_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.animal_record_type AS ENUM (
    'observation',
    'surgery',
    'sacrifice',
    'pathology',
    'blood_test'
);


--
-- Name: animal_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.animal_status AS ENUM (
    'unassigned',
    'in_experiment',
    'completed',
    'euthanized',
    'sudden_death',
    'transferred'
);


--
-- Name: animal_transfer_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.animal_transfer_status AS ENUM (
    'pending',
    'vet_evaluated',
    'plan_assigned',
    'pi_approved',
    'completed',
    'rejected'
);


--
-- Name: calibration_cycle; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.calibration_cycle AS ENUM (
    'monthly',
    'quarterly',
    'semi_annual',
    'annual'
);


--
-- Name: calibration_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.calibration_type AS ENUM (
    'calibration',
    'validation',
    'inspection'
);


--
-- Name: capa_action_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.capa_action_type AS ENUM (
    'corrective',
    'preventive'
);


--
-- Name: capa_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.capa_status AS ENUM (
    'open',
    'in_progress',
    'completed',
    'verified'
);


--
-- Name: care_record_mode; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.care_record_mode AS ENUM (
    'legacy',
    'pain_assessment'
);


--
-- Name: customer_category; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.customer_category AS ENUM (
    'internal',
    'external',
    'research',
    'other'
);


--
-- Name: disposal_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.disposal_status AS ENUM (
    'pending',
    'approved',
    'rejected'
);


--
-- Name: doc_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.doc_status AS ENUM (
    'draft',
    'submitted',
    'approved',
    'cancelled'
);


--
-- Name: doc_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.doc_type AS ENUM (
    'PO',
    'GRN',
    'PR',
    'SO',
    'DO',
    'SR',
    'TR',
    'STK',
    'ADJ',
    'RM',
    'RTN'
);


--
-- Name: equipment_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.equipment_status AS ENUM (
    'active',
    'inactive',
    'under_repair',
    'decommissioned'
);


--
-- Name: euthanasia_order_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.euthanasia_order_status AS ENUM (
    'pending_pi',
    'appealed',
    'chair_arbitration',
    'approved',
    'rejected',
    'executed',
    'cancelled'
);


--
-- Name: export_format; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.export_format AS ENUM (
    'pdf',
    'excel'
);


--
-- Name: export_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.export_type AS ENUM (
    'medical_summary',
    'observation_records',
    'surgery_records',
    'experiment_records'
);


--
-- Name: import_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.import_status AS ENUM (
    'pending',
    'processing',
    'completed',
    'failed'
);


--
-- Name: import_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.import_type AS ENUM (
    'animal_basic',
    'animal_weight'
);


--
-- Name: leave_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.leave_status AS ENUM (
    'DRAFT',
    'PENDING_L1',
    'PENDING_L2',
    'PENDING_HR',
    'PENDING_GM',
    'APPROVED',
    'REJECTED',
    'CANCELLED',
    'REVOKED',
    'PENDING_PROXY',
    'PENDING_DIRECTOR'
);


--
-- Name: leave_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.leave_type AS ENUM (
    'ANNUAL',
    'PERSONAL',
    'SICK',
    'COMPENSATORY',
    'MARRIAGE',
    'BEREAVEMENT',
    'MATERNITY',
    'PATERNITY',
    'MENSTRUAL',
    'OFFICIAL'
);


--
-- Name: maintenance_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.maintenance_status AS ENUM (
    'pending',
    'in_progress',
    'completed',
    'unrepairable',
    'pending_review'
);


--
-- Name: maintenance_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.maintenance_type AS ENUM (
    'repair',
    'maintenance'
);


--
-- Name: nc_severity; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.nc_severity AS ENUM (
    'critical',
    'major',
    'minor'
);


--
-- Name: nc_source; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.nc_source AS ENUM (
    'inspection',
    'observation',
    'external_audit',
    'self_report'
);


--
-- Name: nc_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.nc_status AS ENUM (
    'open',
    'in_progress',
    'pending_verification',
    'closed'
);


--
-- Name: notification_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.notification_type AS ENUM (
    'low_stock',
    'expiry_warning',
    'document_approval',
    'protocol_status',
    'protocol_submitted',
    'review_assignment',
    'review_comment',
    'leave_approval',
    'overtime_approval',
    'vet_recommendation',
    'system_alert',
    'monthly_report',
    'equipment_overdue',
    'equipment_unrepairable',
    'equipment_disposal'
);


--
-- Name: partner_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.partner_type AS ENUM (
    'supplier',
    'customer'
);


--
-- Name: protocol_activity_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.protocol_activity_type AS ENUM (
    'CREATED',
    'UPDATED',
    'SUBMITTED',
    'RESUBMITTED',
    'APPROVED',
    'APPROVED_WITH_CONDITIONS',
    'CLOSED',
    'REJECTED',
    'SUSPENDED',
    'DELETED',
    'STATUS_CHANGED',
    'REVIEWER_ASSIGNED',
    'VET_ASSIGNED',
    'COEDITOR_ASSIGNED',
    'COEDITOR_REMOVED',
    'COMMENT_ADDED',
    'COMMENT_REPLIED',
    'COMMENT_RESOLVED',
    'ATTACHMENT_UPLOADED',
    'ATTACHMENT_DELETED',
    'VERSION_CREATED',
    'VERSION_RECOVERED',
    'AMENDMENT_CREATED',
    'AMENDMENT_SUBMITTED',
    'ANIMAL_ASSIGNED',
    'ANIMAL_UNASSIGNED',
    'MCP_READ'
);


--
-- Name: protocol_role; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.protocol_role AS ENUM (
    'PI',
    'CLIENT'
);


--
-- Name: protocol_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.protocol_status AS ENUM (
    'DRAFT',
    'SUBMITTED',
    'PRE_REVIEW',
    'PRE_REVIEW_REVISION_REQUIRED',
    'VET_REVIEW',
    'VET_REVISION_REQUIRED',
    'UNDER_REVIEW',
    'REVISION_REQUIRED',
    'RESUBMITTED',
    'APPROVED',
    'APPROVED_WITH_CONDITIONS',
    'DEFERRED',
    'REJECTED',
    'SUSPENDED',
    'CLOSED',
    'DELETED'
);


--
-- Name: qa_inspection_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.qa_inspection_status AS ENUM (
    'draft',
    'submitted',
    'closed'
);


--
-- Name: qa_inspection_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.qa_inspection_type AS ENUM (
    'protocol',
    'equipment',
    'facility',
    'training',
    'general'
);


--
-- Name: qa_item_result; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.qa_item_result AS ENUM (
    'pass',
    'fail',
    'not_applicable'
);


--
-- Name: qa_schedule_item_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.qa_schedule_item_status AS ENUM (
    'planned',
    'in_progress',
    'completed',
    'cancelled',
    'overdue'
);


--
-- Name: qa_schedule_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.qa_schedule_status AS ENUM (
    'planned',
    'in_progress',
    'completed',
    'cancelled'
);


--
-- Name: qa_schedule_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.qa_schedule_type AS ENUM (
    'annual',
    'periodic',
    'ad_hoc'
);


--
-- Name: record_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.record_type AS ENUM (
    'abnormal',
    'experiment',
    'observation'
);


--
-- Name: report_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.report_type AS ENUM (
    'stock_on_hand',
    'stock_ledger',
    'purchase_summary',
    'cost_summary',
    'expiry_report',
    'low_stock_report'
);


--
-- Name: schedule_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.schedule_type AS ENUM (
    'daily',
    'weekly',
    'monthly'
);


--
-- Name: signature_meaning; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.signature_meaning AS ENUM (
    'APPROVE',
    'REVIEW',
    'WITNESS',
    'AUTHOR',
    'INVALIDATE',
    'CONFIRM',
    'LEGACY_PRE_R30_10',
    'ACKNOWLEDGE'
);


--
-- Name: sop_status; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.sop_status AS ENUM (
    'draft',
    'active',
    'obsolete'
);


--
-- Name: stock_direction; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.stock_direction AS ENUM (
    'in',
    'out',
    'transfer_in',
    'transfer_out',
    'adjust_in',
    'adjust_out'
);


--
-- Name: supplier_category; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.supplier_category AS ENUM (
    'drug',
    'consumable',
    'feed',
    'equipment',
    'other'
);


--
-- Name: validation_phase; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.validation_phase AS ENUM (
    'IQ',
    'OQ',
    'PQ'
);


--
-- Name: version_record_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.version_record_type AS ENUM (
    'observation',
    'surgery',
    'weight',
    'vaccination',
    'sacrifice',
    'pathology',
    'blood_test'
);


--
-- Name: vet_record_type; Type: TYPE; Schema: public; Owner: -
--

CREATE TYPE public.vet_record_type AS ENUM (
    'observation',
    'surgery'
);


--
-- Name: animal_record_type_to_text(public.animal_record_type); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.animal_record_type_to_text(public.animal_record_type) RETURNS text
    LANGUAGE sql IMMUTABLE
    AS $_$
    SELECT $1::text;
$_$;


--
-- Name: CAST (public.animal_record_type AS text); Type: CAST; Schema: -; Owner: -
--

CREATE CAST (public.animal_record_type AS text) WITH FUNCTION public.animal_record_type_to_text(public.animal_record_type) AS ASSIGNMENT;


--
-- Name: record_type_to_text(public.record_type); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.record_type_to_text(public.record_type) RETURNS text
    LANGUAGE sql IMMUTABLE
    AS $_$
    SELECT $1::text;
$_$;


--
-- Name: CAST (public.record_type AS text); Type: CAST; Schema: -; Owner: -
--

CREATE CAST (public.record_type AS text) WITH FUNCTION public.record_type_to_text(public.record_type) AS ASSIGNMENT;


--
-- Name: text_to_version_record_type(text); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.text_to_version_record_type(text) RETURNS public.version_record_type
    LANGUAGE sql STABLE
    AS $_$
    SELECT r.v FROM unnest(enum_range(NULL::version_record_type)) AS r(v)
    WHERE version_record_type_to_text(r.v) = $1 LIMIT 1;
$_$;


--
-- Name: CAST (text AS public.version_record_type); Type: CAST; Schema: -; Owner: -
--

CREATE CAST (text AS public.version_record_type) WITH FUNCTION public.text_to_version_record_type(text) AS ASSIGNMENT;


--
-- Name: version_record_type_to_text(public.version_record_type); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.version_record_type_to_text(public.version_record_type) RETURNS text
    LANGUAGE sql STABLE
    AS $_$
    SELECT (SELECT enumlabel FROM pg_enum
            WHERE enumtypid = 'version_record_type'::regtype
            ORDER BY enumsortorder
            OFFSET (array_position(enum_range(NULL::version_record_type), $1) - 1)
            LIMIT 1);
$_$;


--
-- Name: CAST (public.version_record_type AS text); Type: CAST; Schema: -; Owner: -
--

CREATE CAST (public.version_record_type AS text) WITH FUNCTION public.version_record_type_to_text(public.version_record_type) AS ASSIGNMENT;


--
-- Name: check_blood_test_items_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_blood_test_items_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- core fields 永不可動
    IF OLD.id IS DISTINCT FROM NEW.id
       OR OLD.blood_test_id IS DISTINCT FROM NEW.blood_test_id
       OR OLD.template_id IS DISTINCT FROM NEW.template_id
       OR OLD.item_name IS DISTINCT FROM NEW.item_name
       OR OLD.result_value IS DISTINCT FROM NEW.result_value
       OR OLD.result_unit IS DISTINCT FROM NEW.result_unit
       OR OLD.reference_range IS DISTINCT FROM NEW.reference_range
       OR OLD.is_abnormal IS DISTINCT FROM NEW.is_abnormal
       OR OLD.remark IS DISTINCT FROM NEW.remark
       OR OLD.sort_order IS DISTINCT FROM NEW.sort_order
       OR OLD.created_at IS DISTINCT FROM NEW.created_at THEN
        RAISE EXCEPTION 'animal_blood_test_items core fields immutable (GLP §11.10(c))。
血檢結果不可直接修改；如需修正，使用 BloodTestService::correct_item_with_reason 走 supersede 流程。'
            USING ERRCODE = 'P0001';
    END IF;

    -- supersede 相關欄位：only NULL → set 一次性翻轉
    IF OLD.superseded_by_id IS NOT NULL
       OR OLD.superseded_at IS NOT NULL
       OR OLD.corrected_by IS NOT NULL
       OR OLD.correction_reason IS NOT NULL THEN
        RAISE EXCEPTION 'animal_blood_test_items already superseded; 修正紀錄不可二次修改 (GLP §11.70)。'
            USING ERRCODE = 'P0001';
    END IF;

    -- 翻轉時 4 欄必須一致 set（CHECK constraint 也會擋，但 trigger 提前出更清楚的錯訊）
    IF NEW.superseded_by_id IS NULL
       OR NEW.superseded_at IS NULL
       OR NEW.corrected_by IS NULL
       OR NEW.correction_reason IS NULL THEN
        RAISE EXCEPTION 'supersede 必須同時設定 superseded_by_id / superseded_at / corrected_by / correction_reason 4 欄。'
            USING ERRCODE = 'P0001';
    END IF;

    RETURN NEW;
END;
$$;


--
-- Name: FUNCTION check_blood_test_items_immutable(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_blood_test_items_immutable() IS 'GLP §11.10(c)/§11.70：blood test items core fields 永不可改；supersede 4 欄僅允許一次性 NULL→set 翻轉。';


--
-- Name: check_blood_test_items_no_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_blood_test_items_no_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    bypass TEXT;
BEGIN
    -- 取 session-level GUC；若未設定為 NULL（current_setting with missing_ok=true）
    bypass := current_setting('app.bypass_blood_test_items_delete', true);
    IF bypass = 'true' THEN
        RAISE NOTICE 'blood_test_items DELETE bypassed via app.bypass_blood_test_items_delete (id=%, blood_test_id=%)', OLD.id, OLD.blood_test_id;
        RETURN OLD;
    END IF;

    RAISE EXCEPTION 'animal_blood_test_items is append-only (GLP §11.70)。不可 DELETE。
若需修正，使用 BloodTestService::correct_item_with_reason；如為合法 cascade 場景，
service 層需在同 tx 內 SET LOCAL app.bypass_blood_test_items_delete = ''true''.'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_blood_test_items_no_delete(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_blood_test_items_no_delete() IS 'GLP §11.70：blood_test_items 一律不可 DELETE。修正請走 supersede 流程。session GUC app.bypass_blood_test_items_delete 為 escape hatch（會 RAISE NOTICE 留軌跡）。';


--
-- Name: check_brute_force(character varying); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_brute_force(p_email character varying) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_failed_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO v_failed_count
    FROM   login_events
    WHERE  email      = p_email
      AND  event_type = 'login_failure'
      AND  created_at > NOW() - INTERVAL '15 minutes';
    RETURN v_failed_count >= 5;
END;
$$;


--
-- Name: check_documents_system_generated_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_documents_system_generated_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.system_generated IS DISTINCT FROM OLD.system_generated THEN
        RAISE EXCEPTION
            'documents.system_generated 不可變更（單號 %）。此旗標決定是否豁免職務分離守衛，'
            '只能於建立單據時由系統自動產生路徑寫入。', OLD.doc_no
            USING ERRCODE = 'P0001';
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: FUNCTION check_documents_system_generated_immutable(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_documents_system_generated_immutable() IS 'system_generated 一經 INSERT 決定即不可變更：它是職務分離守衛的豁免依據，若能事後翻轉，具 create+approve 權限者即可自行豁免。';


--
-- Name: check_electronic_signatures_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_electronic_signatures_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- core fields (簽章本身的內容) 不可動
    IF OLD.entity_type IS DISTINCT FROM NEW.entity_type
       OR OLD.entity_id IS DISTINCT FROM NEW.entity_id
       OR OLD.signer_id IS DISTINCT FROM NEW.signer_id
       OR OLD.signature_type IS DISTINCT FROM NEW.signature_type
       OR OLD.content_hash IS DISTINCT FROM NEW.content_hash
       OR OLD.signature_data IS DISTINCT FROM NEW.signature_data
       OR OLD.signature_method IS DISTINCT FROM NEW.signature_method
       OR OLD.handwriting_svg IS DISTINCT FROM NEW.handwriting_svg
       OR OLD.stroke_data IS DISTINCT FROM NEW.stroke_data
       OR OLD.signed_at IS DISTINCT FROM NEW.signed_at
       -- bot review #627：meaning（§11.50(a)(3) 簽章意義）與 hmac_version（R30-7）亦為核心
       -- 元數據，須鎖定，否則可竄改簽章法律意義（如 Review→Approve）或降級 HMAC 版本。
       -- 註：041 trigger 早於 043(meaning)/042(hmac_version) 兩欄存在，故原版漏列；此處補上。
       OR OLD.meaning IS DISTINCT FROM NEW.meaning
       OR OLD.hmac_version IS DISTINCT FROM NEW.hmac_version THEN
        RAISE EXCEPTION 'electronic_signatures core fields immutable (GLP §11.70)。
僅 is_valid / invalidated_reason / invalidated_at / invalidated_by 可由 SignatureService::invalidate 修改。'
            USING ERRCODE = 'P0001';
    END IF;
    -- Low-1：is_valid 僅能 true→false（作廢），不可 false→true（復活）。
    IF NEW.is_valid AND NOT OLD.is_valid THEN
        RAISE EXCEPTION '不可將已作廢的電子簽章重新生效 (GLP §11.70：作廢不可復原)。'
            USING ERRCODE = 'P0001';
    END IF;
    -- 走到這裡代表 core fields 都沒動且 is_valid 未被復活 → 通過。
    RETURN NEW;
END;
$$;


--
-- Name: FUNCTION check_electronic_signatures_immutable(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_electronic_signatures_immutable() IS 'GLP §11.70：簽章 core fields 不可動；is_valid 僅能 true→false（作廢不可復原）。僅允許軟失效（is_valid + invalidated_* 4 欄）。';


--
-- Name: check_electronic_signatures_no_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_electronic_signatures_no_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'electronic_signatures is append-only (GLP §11.70)。不可 DELETE。
若需作廢簽章，使用 SignatureService::invalidate（軟失效，保留紀錄）。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_electronic_signatures_no_delete(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_electronic_signatures_no_delete() IS 'GLP §11.70：electronic_signatures 一律不可 DELETE。作廢簽章須走軟失效流程。';


--
-- Name: check_journal_entries_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_journal_entries_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'journal_entries is append-only（帳簿憑證不可竄改）。不可 UPDATE。
帳務更正請開立沖銷傳票（借貸相反的新分錄），保留原始傳票。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_journal_entries_immutable(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_journal_entries_immutable() IS '帳簿憑證不可竄改：journal_entries 為 append-only 傳票表頭，不可 UPDATE。更正走沖銷傳票。';


--
-- Name: check_journal_entries_no_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_journal_entries_no_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'journal_entries is append-only（帳簿憑證不可竄改）。不可 DELETE。
帳務更正請開立沖銷傳票，保留原始傳票。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_journal_entries_no_delete(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_journal_entries_no_delete() IS '帳簿憑證不可竄改：journal_entries 不可 DELETE。子表 journal_entry_lines 的 ON DELETE CASCADE 因父表被擋而不會觸發。';


--
-- Name: check_journal_entries_no_truncate(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_journal_entries_no_truncate() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'journal_entries is append-only（帳簿憑證不可竄改）。不可 TRUNCATE。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_journal_entries_no_truncate(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_journal_entries_no_truncate() IS '帳簿憑證不可竄改：TRUNCATE 不觸發 row-level trigger，需獨立的 STATEMENT 層擋板。';


--
-- Name: check_journal_entry_lines_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_journal_entry_lines_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'journal_entry_lines is append-only（帳簿憑證不可竄改）。不可 UPDATE。
分錄更正請開立沖銷傳票（借貸相反的新分錄），保留原始分錄。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_journal_entry_lines_immutable(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_journal_entry_lines_immutable() IS '帳簿憑證不可竄改：journal_entry_lines 為 append-only 傳票明細，不可 UPDATE。更正走沖銷傳票。';


--
-- Name: check_journal_entry_lines_no_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_journal_entry_lines_no_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'journal_entry_lines is append-only（帳簿憑證不可竄改）。不可 DELETE。
分錄更正請開立沖銷傳票，保留原始分錄。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_journal_entry_lines_no_delete(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_journal_entry_lines_no_delete() IS '帳簿憑證不可竄改：journal_entry_lines 不可 DELETE。含父表 journal_entries 的 ON DELETE CASCADE 路徑。';


--
-- Name: check_journal_entry_lines_no_truncate(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_journal_entry_lines_no_truncate() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'journal_entry_lines is append-only（帳簿憑證不可竄改）。不可 TRUNCATE。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_journal_entry_lines_no_truncate(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_journal_entry_lines_no_truncate() IS '帳簿憑證不可竄改：TRUNCATE 不觸發 row-level trigger，需獨立的 STATEMENT 層擋板；亦擋下父表 TRUNCATE ... CASCADE 的連帶清空。';


--
-- Name: check_stock_ledger_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_stock_ledger_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'stock_ledger is append-only (GLP §11.10(e))。不可 UPDATE。
庫存更正請開立調整單（ADJ）或沖銷單（R84-5），由新流水抵銷舊流水，保留完整軌跡。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_stock_ledger_immutable(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_stock_ledger_immutable() IS 'GLP §11.10(e)：stock_ledger 為 append-only 庫存流水，不可 UPDATE。更正走 ADJ / 沖銷單。';


--
-- Name: check_stock_ledger_no_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_stock_ledger_no_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'stock_ledger is append-only (GLP §11.10(e))。不可 DELETE。
庫存更正請開立調整單（ADJ）或沖銷單（R84-5）。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_stock_ledger_no_delete(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_stock_ledger_no_delete() IS 'GLP §11.10(e)：stock_ledger 不可 DELETE。retention_enforcer 因本表無 deleted_at 欄位而 skip，不受影響。';


--
-- Name: check_stock_ledger_no_truncate(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_stock_ledger_no_truncate() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'stock_ledger is append-only (GLP §11.10(e))。不可 TRUNCATE。'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_stock_ledger_no_truncate(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_stock_ledger_no_truncate() IS 'GLP §11.10(e)：TRUNCATE 不觸發 row-level trigger，需獨立的 STATEMENT 層擋板，否則 DELETE 防線可被一句 TRUNCATE 繞過。';


--
-- Name: check_user_activity_logs_immutable(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_user_activity_logs_immutable() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF OLD.id IS DISTINCT FROM NEW.id
       OR OLD.actor_user_id IS DISTINCT FROM NEW.actor_user_id
       OR OLD.actor_email IS DISTINCT FROM NEW.actor_email
       OR OLD.actor_display_name IS DISTINCT FROM NEW.actor_display_name
       OR (OLD.actor_roles IS DISTINCT FROM NEW.actor_roles)
       OR OLD.session_id IS DISTINCT FROM NEW.session_id
       OR OLD.session_started_at IS DISTINCT FROM NEW.session_started_at
       OR OLD.event_category IS DISTINCT FROM NEW.event_category
       OR OLD.event_type IS DISTINCT FROM NEW.event_type
       OR OLD.event_severity IS DISTINCT FROM NEW.event_severity
       OR OLD.entity_type IS DISTINCT FROM NEW.entity_type
       OR OLD.entity_id IS DISTINCT FROM NEW.entity_id
       OR OLD.entity_display_name IS DISTINCT FROM NEW.entity_display_name
       OR (OLD.before_data IS DISTINCT FROM NEW.before_data)
       OR (OLD.after_data IS DISTINCT FROM NEW.after_data)
       OR (OLD.changed_fields IS DISTINCT FROM NEW.changed_fields)
       OR OLD.ip_address IS DISTINCT FROM NEW.ip_address
       OR OLD.user_agent IS DISTINCT FROM NEW.user_agent
       OR OLD.request_path IS DISTINCT FROM NEW.request_path
       OR OLD.request_method IS DISTINCT FROM NEW.request_method
       OR OLD.response_status IS DISTINCT FROM NEW.response_status
       OR OLD.geo_country IS DISTINCT FROM NEW.geo_country
       OR OLD.geo_city IS DISTINCT FROM NEW.geo_city
       OR OLD.is_suspicious IS DISTINCT FROM NEW.is_suspicious
       OR OLD.suspicious_reason IS DISTINCT FROM NEW.suspicious_reason
       OR OLD.created_at IS DISTINCT FROM NEW.created_at
       OR OLD.partition_date IS DISTINCT FROM NEW.partition_date
    THEN
        RAISE EXCEPTION 'user_activity_logs: direct modification of log payload is not allowed (integrity enforcement)';
    END IF;
    RETURN NEW;
END;
$$;


--
-- Name: check_user_activity_logs_no_delete(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.check_user_activity_logs_no_delete() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'user_activity_logs is append-only (GLP §11.10(e)(1))。不可 DELETE。
若需排除舊資料，使用 partition drop（DETACH PARTITION 不觸發 ROW trigger）'
        USING ERRCODE = 'P0001';
END;
$$;


--
-- Name: FUNCTION check_user_activity_logs_no_delete(); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.check_user_activity_logs_no_delete() IS 'GLP §11.10(e)(1) 雙保險：阻擋任何 DELETE FROM user_activity_logs。partition retention drop 走 DETACH PARTITION 不觸發。';


--
-- Name: create_default_notification_settings(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.create_default_notification_settings() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO notification_settings (user_id)
    VALUES (NEW.id)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$;


--
-- Name: fn_expiry_alerts(integer, integer); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.fn_expiry_alerts(p_warn_days integer DEFAULT 60, p_cutoff_days integer DEFAULT 90) RETURNS TABLE(product_id uuid, sku character varying, product_name character varying, spec text, category_code character varying, warehouse_id uuid, warehouse_code character varying, warehouse_name character varying, batch_no character varying, expiry_date date, on_hand_qty numeric, base_uom character varying, days_until_expiry integer, expiry_status character varying, total_qty numeric)
    LANGUAGE sql STABLE
    AS $$
    SELECT
        p.id                                                          AS product_id,
        p.sku,
        p.name                                                        AS product_name,
        p.spec,
        p.category_code,
        sl.warehouse_id,
        w.code                                                        AS warehouse_code,
        w.name                                                        AS warehouse_name,
        sl.batch_no,
        sl.expiry_date,
        SUM(CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            ELSE -sl.qty_base
        END)                                                          AS on_hand_qty,
        p.base_uom,
        (sl.expiry_date - CURRENT_DATE)::INT                         AS days_until_expiry,
        CASE WHEN sl.expiry_date < CURRENT_DATE
             THEN 'expired'
             ELSE 'expiring_soon'
        END                                                           AS expiry_status,
        COALESCE(inv.on_hand_qty_base, 0)                            AS total_qty
    FROM stock_ledger sl
    JOIN products p    ON sl.product_id   = p.id
    JOIN warehouses w  ON sl.warehouse_id = w.id
    LEFT JOIN inventory_snapshots inv
           ON inv.product_id = p.id AND inv.warehouse_id = sl.warehouse_id
    WHERE p.track_expiry = true
      AND sl.expiry_date IS NOT NULL
      AND p.is_active    = true
      AND sl.expiry_date >= CURRENT_DATE - p_cutoff_days
    GROUP BY p.id, p.sku, p.name, p.spec, p.category_code,
             sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date,
             p.base_uom, inv.on_hand_qty_base
    HAVING
        SUM(CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            ELSE -sl.qty_base
        END) > 0
        AND sl.expiry_date <= CURRENT_DATE + p_warn_days
$$;


--
-- Name: FUNCTION fn_expiry_alerts(p_warn_days integer, p_cutoff_days integer); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.fn_expiry_alerts(p_warn_days integer, p_cutoff_days integer) IS '效期預警查詢函數，支援動態傳入提前預警天數與截止天數';


--
-- Name: get_annual_leave_balance(uuid); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_annual_leave_balance(p_user_id uuid) RETURNS TABLE(entitlement_year integer, entitled_days numeric, used_days numeric, remaining_days numeric, expires_at date, days_until_expiry integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT ale.entitlement_year, ale.entitled_days, ale.used_days,
           (ale.entitled_days - ale.used_days),
           ale.expires_at,
           (ale.expires_at - CURRENT_DATE)::INTEGER
    FROM annual_leave_entitlements ale
    WHERE ale.user_id = p_user_id
      AND NOT ale.is_expired
      AND (ale.entitled_days - ale.used_days) > 0
    ORDER BY ale.expires_at ASC;
END;
$$;


--
-- Name: get_comp_time_balance(uuid); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_comp_time_balance(p_user_id uuid) RETURNS TABLE(id uuid, earned_date date, original_hours numeric, used_hours numeric, remaining_hours numeric, expires_at date, days_until_expiry integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    SELECT ctb.id, ctb.earned_date, ctb.original_hours, ctb.used_hours,
           (ctb.original_hours - ctb.used_hours),
           ctb.expires_at,
           (ctb.expires_at - CURRENT_DATE)::INTEGER
    FROM comp_time_balances ctb
    WHERE ctb.user_id = p_user_id
      AND NOT ctb.is_expired
      AND (ctb.original_hours - ctb.used_hours) > 0
    ORDER BY ctb.earned_date ASC;
END;
$$;


--
-- Name: get_total_comp_time_hours(uuid); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.get_total_comp_time_hours(p_user_id uuid) RETURNS numeric
    LANGUAGE plpgsql
    AS $$
DECLARE v_total NUMERIC;
BEGIN
    SELECT COALESCE(SUM(original_hours - used_hours), 0) INTO v_total
    FROM comp_time_balances
    WHERE user_id = p_user_id
      AND NOT is_expired
      AND (original_hours - used_hours) > 0;
    RETURN v_total;
END;
$$;


--
-- Name: log_activity(uuid, character varying, character varying, character varying, uuid, character varying, jsonb, jsonb, inet, text, uuid, text[], boolean); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.log_activity(p_actor_user_id uuid, p_event_category character varying, p_event_type character varying, p_entity_type character varying, p_entity_id uuid, p_entity_display_name character varying, p_before_data jsonb DEFAULT NULL::jsonb, p_after_data jsonb DEFAULT NULL::jsonb, p_ip_address inet DEFAULT NULL::inet, p_user_agent text DEFAULT NULL::text, p_impersonated_by_user_id uuid DEFAULT NULL::uuid, p_changed_fields text[] DEFAULT NULL::text[], p_is_suspicious boolean DEFAULT false) RETURNS uuid
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_id                 UUID;
    v_actor_email        VARCHAR(255);
    v_actor_display_name VARCHAR(100);
    v_actor_roles        JSONB;
    v_changed_fields     TEXT[];
BEGIN
    SELECT email, display_name
    INTO   v_actor_email, v_actor_display_name
    FROM   users WHERE id = p_actor_user_id;

    SELECT jsonb_agg(r.code)
    INTO   v_actor_roles
    FROM   user_roles ur
    JOIN   roles r ON ur.role_id = r.id
    WHERE  ur.user_id = p_actor_user_id;

    -- changed_fields 來源優先級（同 migration 036，R26-5 修正）：
    --   1. app 層提供 → 直接用
    --   2. before/after 都有 → 取聯集中值不同者
    --   3. 其他 → NULL
    IF p_changed_fields IS NOT NULL THEN
        v_changed_fields := p_changed_fields;
    ELSIF p_before_data IS NOT NULL AND p_after_data IS NOT NULL THEN
        IF jsonb_typeof(p_before_data) = 'object' AND jsonb_typeof(p_after_data) = 'object' THEN
            SELECT array_agg(DISTINCT key ORDER BY key)
            INTO   v_changed_fields
            FROM (
                SELECT jsonb_object_keys(p_before_data) AS key
                UNION
                SELECT jsonb_object_keys(p_after_data) AS key
            ) all_keys
            WHERE  (p_before_data->key) IS DISTINCT FROM (p_after_data->key);
        END IF;
    END IF;

    INSERT INTO user_activity_logs (
        actor_user_id, actor_email, actor_display_name, actor_roles,
        event_category, event_type, event_severity,
        entity_type, entity_id, entity_display_name,
        before_data, after_data, changed_fields,
        ip_address, user_agent,
        impersonated_by_user_id,
        is_suspicious, suspicious_reason
    ) VALUES (
        p_actor_user_id, v_actor_email, v_actor_display_name, v_actor_roles,
        p_event_category, p_event_type,
        CASE WHEN p_is_suspicious THEN 'warning' ELSE 'info' END,
        p_entity_type, p_entity_id, p_entity_display_name,
        p_before_data, p_after_data, v_changed_fields,
        p_ip_address, p_user_agent,
        p_impersonated_by_user_id,
        p_is_suspicious,
        CASE WHEN p_is_suspicious THEN 'Security event: ' || COALESCE(p_event_type, '') ELSE NULL END
    ) RETURNING id INTO v_id;

    RETURN v_id;
END;
$$;


--
-- Name: FUNCTION log_activity(p_actor_user_id uuid, p_event_category character varying, p_event_type character varying, p_entity_type character varying, p_entity_id uuid, p_entity_display_name character varying, p_before_data jsonb, p_after_data jsonb, p_ip_address inet, p_user_agent text, p_impersonated_by_user_id uuid, p_changed_fields text[], p_is_suspicious boolean); Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON FUNCTION public.log_activity(p_actor_user_id uuid, p_event_category character varying, p_event_type character varying, p_entity_type character varying, p_entity_id uuid, p_entity_display_name character varying, p_before_data jsonb, p_after_data jsonb, p_ip_address inet, p_user_agent text, p_impersonated_by_user_id uuid, p_changed_fields text[], p_is_suspicious boolean) IS 'v4 (R28-5 follow-up): 加 p_is_suspicious — SECURITY 事件走 HMAC chain 時保留
     is_suspicious / event_severity=warning / suspicious_reason。沿用 v3 changed_fields 修正。';


--
-- Name: maintenance_vacuum_analyze(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.maintenance_vacuum_analyze() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    ANALYZE animals;
    ANALYZE animal_observations;
    ANALYZE animal_surgeries;
    ANALYZE animal_weights;
    ANALYZE animal_vaccinations;
    ANALYZE vet_recommendations;
    ANALYZE notifications;
    ANALYZE user_activity_logs;
    ANALYZE attachments;
    ANALYZE protocols;
    ANALYZE audit_logs;
    RAISE NOTICE 'maintenance_vacuum_analyze completed at %', NOW();
END;
$$;


--
-- Name: update_thread_last_message_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE FUNCTION public.update_thread_last_message_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- 用 GREATEST：避免歷史訊息匯入或亂序寫入時把 last_message_at 倒退
    UPDATE message_threads
    SET last_message_at = GREATEST(last_message_at, NEW.created_at)
    WHERE id = NEW.thread_id;
    RETURN NEW;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: ai_api_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_api_keys (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    key_hash character varying(64) NOT NULL,
    key_prefix character varying(12) NOT NULL,
    created_by uuid NOT NULL,
    scopes jsonb DEFAULT '["read"]'::jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    expires_at timestamp with time zone,
    last_used_at timestamp with time zone,
    usage_count bigint DEFAULT 0 NOT NULL,
    rate_limit_per_minute integer DEFAULT 60 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_rate_limit_positive CHECK (((rate_limit_per_minute IS NULL) OR (rate_limit_per_minute > 0)))
);


--
-- Name: ai_query_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_query_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    api_key_id uuid NOT NULL,
    endpoint character varying(200) NOT NULL,
    method character varying(10) DEFAULT 'GET'::character varying NOT NULL,
    query_summary jsonb,
    response_status smallint NOT NULL,
    duration_ms integer DEFAULT 0 NOT NULL,
    source_ip character varying(45),
    created_at timestamp with time zone DEFAULT now() NOT NULL
)
PARTITION BY RANGE (created_at);


--
-- Name: ai_query_logs_2026_08; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_query_logs_2026_08 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    api_key_id uuid NOT NULL,
    endpoint character varying(200) NOT NULL,
    method character varying(10) DEFAULT 'GET'::character varying NOT NULL,
    query_summary jsonb,
    response_status smallint NOT NULL,
    duration_ms integer DEFAULT 0 NOT NULL,
    source_ip character varying(45),
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: ai_query_logs_2026_09; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ai_query_logs_2026_09 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    api_key_id uuid NOT NULL,
    endpoint character varying(200) NOT NULL,
    method character varying(10) DEFAULT 'GET'::character varying NOT NULL,
    query_summary jsonb,
    response_status smallint NOT NULL,
    duration_ms integer DEFAULT 0 NOT NULL,
    source_ip character varying(45),
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: amendment_review_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.amendment_review_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    amendment_id uuid NOT NULL,
    reviewer_id uuid,
    assigned_by uuid NOT NULL,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL,
    decision character varying(20),
    decided_at timestamp with time zone,
    comment text,
    reviewer_name text,
    CONSTRAINT chk_amendment_reviewer_identity CHECK (((reviewer_id IS NOT NULL) OR (NULLIF(btrim(reviewer_name), ''::text) IS NOT NULL)))
);


--
-- Name: COLUMN amendment_review_assignments.reviewer_name; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.amendment_review_assignments.reviewer_name IS '院外審查委員姓名（補登歷史變更用，reviewer_id 為 NULL 時填）';


--
-- Name: amendment_status_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.amendment_status_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    amendment_id uuid NOT NULL,
    from_status public.amendment_status,
    to_status public.amendment_status NOT NULL,
    changed_by uuid NOT NULL,
    remark text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: amendment_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.amendment_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    amendment_id uuid NOT NULL,
    version_no integer NOT NULL,
    content_snapshot jsonb NOT NULL,
    submitted_at timestamp with time zone DEFAULT now() NOT NULL,
    submitted_by uuid NOT NULL
);


--
-- Name: amendments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.amendments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    amendment_no character varying(50) NOT NULL,
    revision_number integer DEFAULT 1 NOT NULL,
    amendment_type public.amendment_type DEFAULT 'PENDING'::public.amendment_type NOT NULL,
    status public.amendment_status DEFAULT 'DRAFT'::public.amendment_status NOT NULL,
    title character varying(200) NOT NULL,
    description text,
    change_items character varying(255)[] DEFAULT '{}'::character varying[],
    changes_content jsonb,
    submitted_by uuid,
    submitted_at timestamp with time zone,
    classified_by uuid,
    classified_at timestamp with time zone,
    classification_remark text,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    approved_signature_id uuid,
    rejected_signature_id uuid,
    effective_from timestamp with time zone,
    version integer DEFAULT 1 NOT NULL,
    is_historical boolean DEFAULT false NOT NULL
);


--
-- Name: COLUMN amendments.approved_signature_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.amendments.approved_signature_id IS 'GLP 核准簽章 FK。NOT NULL 後表示已被簽章核准，service 層 update guard 拒絕修改（21 CFR §11.10(e)(1)）。';


--
-- Name: COLUMN amendments.rejected_signature_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.amendments.rejected_signature_id IS 'GLP 否決簽章 FK。語意同 approved_signature_id 但表示被否決。';


--
-- Name: COLUMN amendments.effective_from; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.amendments.effective_from IS 'R30-25 GLP §58：amendment 正式生效時點。NULL = 尚未生效（含 APPROVED 但未啟用）。';


--
-- Name: COLUMN amendments.is_historical; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.amendments.is_historical IS '補登歷史變更（紙本核准回溯）：跳過 live 審查、可由 DRAFT 直接 finalize 至 EFFECTIVE、approved/rejected_signature_id 維持 NULL（無 live 電子簽章）';


--
-- Name: animal_blood_test_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_blood_test_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    blood_test_id uuid NOT NULL,
    template_id uuid,
    item_name character varying(200) NOT NULL,
    result_value character varying(100),
    result_unit character varying(50),
    reference_range character varying(100),
    is_abnormal boolean DEFAULT false NOT NULL,
    remark text,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    superseded_by_id uuid,
    superseded_at timestamp with time zone,
    corrected_by uuid,
    correction_reason text,
    CONSTRAINT chk_blood_test_items_supersede_consistency CHECK ((((superseded_by_id IS NULL) AND (superseded_at IS NULL) AND (corrected_by IS NULL) AND (correction_reason IS NULL)) OR ((superseded_by_id IS NOT NULL) AND (superseded_at IS NOT NULL) AND (corrected_by IS NOT NULL) AND (correction_reason IS NOT NULL) AND (char_length(correction_reason) >= 5))))
);


--
-- Name: COLUMN animal_blood_test_items.superseded_by_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_blood_test_items.superseded_by_id IS 'R30-16: 指向修正後的新 row。NULL 表示此筆為 current（最新版）。';


--
-- Name: COLUMN animal_blood_test_items.superseded_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_blood_test_items.superseded_at IS 'R30-16: 此筆被 supersede 的時間。';


--
-- Name: COLUMN animal_blood_test_items.corrected_by; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_blood_test_items.corrected_by IS 'R30-16: 執行修正的使用者 ID。';


--
-- Name: COLUMN animal_blood_test_items.correction_reason; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_blood_test_items.correction_reason IS 'R30-16: 修正原因（GLP §11.10(e)）。被 supersede 的舊 row 必填，current row 必為 NULL。';


--
-- Name: animal_blood_tests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_blood_tests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    test_date date NOT NULL,
    lab_name character varying(200),
    status character varying(20) DEFAULT 'completed'::character varying NOT NULL,
    remark text,
    vet_read boolean DEFAULT false NOT NULL,
    vet_read_at timestamp with time zone,
    deleted_at timestamp with time zone,
    deleted_by uuid,
    delete_reason text,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    is_locked boolean DEFAULT false NOT NULL,
    locked_at timestamp with time zone,
    locked_by uuid,
    CONSTRAINT chk_blood_test_status CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'completed'::character varying])::text[])))
);


--
-- Name: COLUMN animal_blood_tests.deleted_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_blood_tests.deleted_at IS '軟刪除時間，NULL 表示未刪除';


--
-- Name: COLUMN animal_blood_tests.is_locked; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_blood_tests.is_locked IS 'GLP 簽章鎖。同 animal_observations.is_locked。';


--
-- Name: animal_field_correction_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_field_correction_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    field_name character varying(50) NOT NULL,
    old_value text,
    new_value text NOT NULL,
    reason text NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    requested_by uuid NOT NULL,
    reviewed_by uuid,
    reviewed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_field_name CHECK (((field_name)::text = ANY ((ARRAY['ear_tag'::character varying, 'birth_date'::character varying, 'gender'::character varying, 'breed'::character varying])::text[]))),
    CONSTRAINT chk_status CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'approved'::character varying, 'rejected'::character varying])::text[])))
);


--
-- Name: TABLE animal_field_correction_requests; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.animal_field_correction_requests IS '動物不可變欄位修正申請，需 admin 批准後套用';


--
-- Name: COLUMN animal_field_correction_requests.field_name; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_field_correction_requests.field_name IS '欄位名稱：ear_tag, birth_date, gender, breed';


--
-- Name: COLUMN animal_field_correction_requests.status; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_field_correction_requests.status IS 'pending=待審核, approved=已批准, rejected=已拒絕';


--
-- Name: animal_import_batches; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_import_batches (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    import_type public.import_type NOT NULL,
    file_name character varying(255) NOT NULL,
    total_rows integer DEFAULT 0 NOT NULL,
    success_count integer DEFAULT 0 NOT NULL,
    error_count integer DEFAULT 0 NOT NULL,
    status public.import_status DEFAULT 'pending'::public.import_status NOT NULL,
    error_details jsonb,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone
);


--
-- Name: animal_observations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_observations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    event_date date NOT NULL,
    record_type public.record_type NOT NULL,
    equipment_used jsonb,
    anesthesia_start timestamp with time zone,
    anesthesia_end timestamp with time zone,
    content text NOT NULL,
    no_medication_needed boolean DEFAULT false NOT NULL,
    stop_medication boolean DEFAULT false NOT NULL,
    treatments jsonb,
    remark text,
    vet_read boolean DEFAULT false NOT NULL,
    vet_read_at timestamp with time zone,
    deleted_at timestamp with time zone,
    deletion_reason text,
    deleted_by uuid,
    is_emergency boolean DEFAULT false,
    emergency_status character varying(20),
    emergency_reason text,
    reviewed_by uuid,
    reviewed_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    is_locked boolean DEFAULT false NOT NULL,
    locked_at timestamp with time zone,
    locked_by uuid
);


--
-- Name: COLUMN animal_observations.is_locked; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_observations.is_locked IS 'GLP 簽章鎖。true 後 update/delete 會被 service 層 ensure_not_locked guard 拒絕（409）。';


--
-- Name: animal_pathology_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_pathology_reports (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_record_attachments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_record_attachments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    record_type public.animal_record_type NOT NULL,
    record_id uuid NOT NULL,
    file_type public.animal_file_type NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path character varying(500) NOT NULL,
    file_size integer NOT NULL,
    mime_type character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_sacrifices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_sacrifices (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    sacrifice_date date,
    zoletil_dose character varying(50),
    method_electrocution boolean DEFAULT false NOT NULL,
    method_bloodletting boolean DEFAULT false NOT NULL,
    method_other text,
    sampling text,
    sampling_other text,
    blood_volume_ml numeric(6,1),
    confirmed_sacrifice boolean DEFAULT false NOT NULL,
    is_locked boolean DEFAULT false NOT NULL,
    locked_at timestamp with time zone,
    locked_by uuid,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_sources; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_sources (
    id uuid NOT NULL,
    code character varying(20) NOT NULL,
    name character varying(100) NOT NULL,
    address text,
    contact character varying(100),
    phone character varying(20),
    phone_ext character varying(20),
    is_active boolean DEFAULT true NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_sudden_deaths; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_sudden_deaths (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    discovered_at timestamp with time zone NOT NULL,
    discovered_by uuid NOT NULL,
    probable_cause text,
    iacuc_no character varying(20),
    location character varying(100),
    remark text,
    requires_pathology boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_surgeries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_surgeries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    is_first_experiment boolean DEFAULT true NOT NULL,
    surgery_date date NOT NULL,
    surgery_site character varying(200) NOT NULL,
    induction_anesthesia jsonb,
    pre_surgery_medication jsonb,
    positioning character varying(100),
    anesthesia_maintenance jsonb,
    anesthesia_observation text,
    vital_signs jsonb,
    reflex_recovery text,
    respiration_rate integer,
    post_surgery_medication jsonb,
    remark text,
    no_medication_needed boolean DEFAULT false NOT NULL,
    vet_read boolean DEFAULT false NOT NULL,
    vet_read_at timestamp with time zone,
    deleted_at timestamp with time zone,
    deletion_reason text,
    deleted_by uuid,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    is_locked boolean DEFAULT false NOT NULL,
    locked_at timestamp with time zone,
    locked_by uuid
);


--
-- Name: COLUMN animal_surgeries.is_locked; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_surgeries.is_locked IS 'GLP 簽章鎖。同 animal_observations.is_locked。';


--
-- Name: animal_transfers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_transfers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    from_iacuc_no character varying(20) NOT NULL,
    to_iacuc_no character varying(20),
    status public.animal_transfer_status DEFAULT 'pending'::public.animal_transfer_status NOT NULL,
    initiated_by uuid NOT NULL,
    reason text NOT NULL,
    remark text,
    rejected_by uuid,
    rejected_reason text,
    completed_at timestamp with time zone,
    transfer_type character varying(20) DEFAULT 'internal'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: COLUMN animal_transfers.transfer_type; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.animal_transfers.transfer_type IS 'external: 轉給其他機構; internal: 仍在機構內';


--
-- Name: animal_vaccinations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_vaccinations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    administered_date date NOT NULL,
    vaccine character varying(100),
    deworming_dose character varying(100),
    deleted_at timestamp with time zone,
    deletion_reason text,
    deleted_by uuid,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_vet_advice_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_vet_advice_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    advice_date date DEFAULT CURRENT_DATE NOT NULL,
    observation text DEFAULT ''::text NOT NULL,
    suggested_treatment text DEFAULT ''::text NOT NULL,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    follow_up text DEFAULT ''::text NOT NULL,
    source_vet_patrol_entry_id uuid
);


--
-- Name: animal_vet_advices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_vet_advices (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    sections jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animal_weights; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animal_weights (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    measure_date date NOT NULL,
    weight numeric(5,1) NOT NULL,
    deleted_at timestamp with time zone,
    deletion_reason text,
    deleted_by uuid,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: animals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.animals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    ear_tag character varying(10) NOT NULL,
    status public.animal_status DEFAULT 'unassigned'::public.animal_status NOT NULL,
    breed public.animal_breed NOT NULL,
    source_id uuid,
    gender public.animal_gender NOT NULL,
    birth_date date,
    entry_date date NOT NULL,
    entry_weight numeric(5,1),
    pen_location character varying(10),
    pre_experiment_code character varying(20),
    iacuc_no character varying(20),
    experiment_date date,
    remark text,
    deleted_at timestamp with time zone,
    deleted_by uuid,
    animal_no character varying(50),
    deletion_reason text,
    animal_id uuid,
    breed_other character varying(100),
    experiment_assigned_by uuid,
    lab_animal_id character varying(50),
    glp_study_no character varying(50),
    randomization_group character varying(50),
    dosing_group character varying(50),
    quarantine_end_date date,
    vet_weight_viewed_at timestamp with time zone,
    vet_vaccine_viewed_at timestamp with time zone,
    vet_sacrifice_viewed_at timestamp with time zone,
    vet_last_viewed_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    pen_id uuid,
    species_id uuid,
    reserved_protocol_id uuid,
    reserved_planned_experiment_id uuid,
    CONSTRAINT chk_animal_reservation_single CHECK ((num_nonnulls(reserved_protocol_id, reserved_planned_experiment_id) <= 1))
);


--
-- Name: annual_leave_entitlements; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.annual_leave_entitlements (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    entitlement_year integer NOT NULL,
    entitled_days numeric(5,2) NOT NULL,
    used_days numeric(5,2) DEFAULT 0,
    expires_at date NOT NULL,
    calculation_basis character varying(50),
    seniority_years numeric(4,2),
    is_expired boolean DEFAULT false,
    expired_days numeric(5,2) DEFAULT 0,
    expiry_processed_at timestamp with time zone,
    notes text,
    adjustment_days numeric(5,2) DEFAULT 0,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_calculation_basis CHECK (((calculation_basis IS NULL) OR ((calculation_basis)::text = ANY ((ARRAY['seniority'::character varying, 'prorated'::character varying, 'manual'::character varying, 'carry_forward'::character varying])::text[]))))
);


--
-- Name: ap_payment_no_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ap_payment_no_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ap_payments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ap_payments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    payment_no character varying(50) NOT NULL,
    partner_id uuid NOT NULL,
    payment_date date NOT NULL,
    amount numeric(18,4) NOT NULL,
    reference text,
    journal_entry_id uuid,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: application_notices; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.application_notices (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    version_label text NOT NULL,
    title text NOT NULL,
    content text NOT NULL,
    attachment_id uuid,
    effective_from date NOT NULL,
    is_active boolean DEFAULT false NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: ar_receipt_no_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.ar_receipt_no_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: ar_receipts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ar_receipts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    receipt_no character varying(50) NOT NULL,
    partner_id uuid NOT NULL,
    receipt_date date NOT NULL,
    amount numeric(18,4) NOT NULL,
    reference text,
    journal_entry_id uuid,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: attachments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attachments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    category character varying(50) NOT NULL,
    entity_id character varying(100),
    entity_type character varying(50),
    file_name character varying(255) NOT NULL,
    file_path character varying(500) NOT NULL,
    file_size integer NOT NULL,
    mime_type character varying(100) NOT NULL,
    uploaded_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: attendance_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.attendance_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    work_date date NOT NULL,
    clock_in_time timestamp with time zone,
    clock_out_time timestamp with time zone,
    regular_hours numeric(5,2) DEFAULT 0,
    overtime_hours numeric(5,2) DEFAULT 0,
    status character varying(20) DEFAULT 'normal'::character varying,
    clock_in_source character varying(20),
    clock_in_ip inet,
    clock_out_source character varying(20),
    clock_out_ip inet,
    clock_in_latitude double precision,
    clock_in_longitude double precision,
    clock_out_latitude double precision,
    clock_out_longitude double precision,
    remark text,
    is_corrected boolean DEFAULT false,
    corrected_by uuid,
    corrected_at timestamp with time zone,
    correction_reason text,
    original_clock_in timestamp with time zone,
    original_clock_out timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_status CHECK (((status)::text = ANY ((ARRAY['normal'::character varying, 'late'::character varying, 'early_leave'::character varying, 'absent'::character varying, 'leave'::character varying, 'holiday'::character varying])::text[])))
);


--
-- Name: audit_chain_known_breaks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.audit_chain_known_breaks (
    log_id uuid NOT NULL,
    reason text NOT NULL,
    recorded_at timestamp with time zone DEFAULT now() NOT NULL,
    recorded_by text
);


--
-- Name: TABLE audit_chain_known_breaks; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.audit_chain_known_breaks IS 'HMAC chain 已知斷鏈白名單：寫入 bug/早期 era/CLI bin 無金鑰 產物（非竄改）。verifier 歸類為 acknowledged、不告警；新斷鏈仍偵測。詳見 2026-06-09 調查（migration 095/097、PR #654/#656）。';


--
-- Name: audit_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.audit_logs (
    id uuid NOT NULL,
    actor_user_id uuid NOT NULL,
    action character varying(50) NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id uuid NOT NULL,
    before_data jsonb,
    after_data jsonb,
    ip_address character varying(45),
    user_agent text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: blood_test_panel_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.blood_test_panel_items (
    panel_id uuid NOT NULL,
    template_id uuid NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL
);


--
-- Name: blood_test_panels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.blood_test_panels (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    key character varying(30) NOT NULL,
    name character varying(100) NOT NULL,
    icon character varying(100) DEFAULT '📋'::character varying,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: blood_test_presets; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.blood_test_presets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(100) NOT NULL,
    icon character varying(100) DEFAULT '📋'::character varying,
    panel_keys text[] DEFAULT '{}'::text[] NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: blood_test_templates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.blood_test_templates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code character varying(20) NOT NULL,
    name character varying(200) NOT NULL,
    default_unit character varying(50),
    reference_range character varying(100),
    default_price numeric(10,2) DEFAULT 0,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: buildings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.buildings (
    id uuid NOT NULL,
    facility_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    description text,
    is_active boolean DEFAULT true NOT NULL,
    config jsonb,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: calendar_event_sync; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.calendar_event_sync (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    leave_request_id uuid NOT NULL,
    google_event_id character varying(255),
    google_event_etag character varying(255),
    google_event_link text,
    sync_version integer DEFAULT 0 NOT NULL,
    local_updated_at timestamp with time zone DEFAULT now() NOT NULL,
    google_updated_at timestamp with time zone,
    last_synced_data jsonb,
    sync_status character varying(50) DEFAULT 'pending_create'::character varying NOT NULL,
    last_error text,
    error_count integer DEFAULT 0 NOT NULL,
    last_error_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: calendar_sync_conflicts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.calendar_sync_conflicts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    calendar_event_sync_id uuid,
    leave_request_id uuid,
    conflict_type character varying(50) NOT NULL,
    ipig_data jsonb DEFAULT '{}'::jsonb NOT NULL,
    google_data jsonb,
    difference_summary text,
    status character varying(50) DEFAULT 'pending'::character varying NOT NULL,
    resolved_by uuid,
    resolved_at timestamp with time zone,
    resolution_notes text,
    requires_new_approval boolean DEFAULT false NOT NULL,
    new_approval_request_id uuid,
    detected_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: calendar_sync_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.calendar_sync_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    job_type character varying(50) DEFAULT 'manual'::character varying NOT NULL,
    triggered_by uuid,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    duration_ms integer,
    status character varying(50) DEFAULT 'running'::character varying NOT NULL,
    events_created integer DEFAULT 0 NOT NULL,
    events_updated integer DEFAULT 0 NOT NULL,
    events_deleted integer DEFAULT 0 NOT NULL,
    events_checked integer DEFAULT 0 NOT NULL,
    conflicts_detected integer DEFAULT 0 NOT NULL,
    errors_count integer DEFAULT 0 NOT NULL,
    error_messages jsonb,
    progress_percentage integer DEFAULT 0 NOT NULL,
    current_operation character varying(255),
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: care_medication_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.care_medication_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    record_type public.vet_record_type NOT NULL,
    record_id uuid NOT NULL,
    record_mode public.care_record_mode DEFAULT 'pain_assessment'::public.care_record_mode NOT NULL,
    post_op_days integer,
    time_period character varying(20),
    vet_read boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    deletion_reason text,
    deleted_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    incision smallint,
    attitude_behavior smallint,
    appetite smallint,
    feces smallint,
    urine smallint,
    pain_score smallint,
    injection_ketorolac boolean DEFAULT false NOT NULL,
    injection_meloxicam boolean DEFAULT false NOT NULL,
    oral_meloxicam boolean DEFAULT false NOT NULL,
    post_medications jsonb,
    is_locked boolean DEFAULT false NOT NULL,
    locked_at timestamp with time zone,
    locked_by uuid
);


--
-- Name: COLUMN care_medication_records.deleted_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.deleted_at IS '軟刪除時間';


--
-- Name: COLUMN care_medication_records.deletion_reason; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.deletion_reason IS '刪除原因（GLP 合規）';


--
-- Name: COLUMN care_medication_records.deleted_by; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.deleted_by IS '刪除操作者';


--
-- Name: COLUMN care_medication_records.incision; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.incision IS '傷口狀況 0=正常 1=輕微透明滲出液/鮮紅血液 2=不透明滲出液/暗褐血液 3=膿樣分泌物';


--
-- Name: COLUMN care_medication_records.attitude_behavior; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.attitude_behavior IS '態度/行為 0=正常 1=皮膚外觀改變 2=步態/姿勢異常 3=反應遲鈍持續自殘 4=焦慮緊張 5=叫聲迴避攻擊';


--
-- Name: COLUMN care_medication_records.appetite; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.appetite IS '食慾 0=正常 1=飼料未吃完 2=飼料不吃且對零食無興趣';


--
-- Name: COLUMN care_medication_records.feces; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.feces IS '排便 0=正常 1=量減少 2=排便異常(軟便/下痢/血便) 3=未排便';


--
-- Name: COLUMN care_medication_records.urine; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.urine IS '排尿 0=正常 1=次數增加或減少 2=尿色異常 3=未排尿';


--
-- Name: COLUMN care_medication_records.pain_score; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.pain_score IS '疼痛分數 1=第一級 2=第二級 3=第三級 4=第四級';


--
-- Name: COLUMN care_medication_records.injection_ketorolac; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.injection_ketorolac IS '術後給藥：注射 Ketorolac';


--
-- Name: COLUMN care_medication_records.injection_meloxicam; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.injection_meloxicam IS '術後給藥：注射 Meloxicam';


--
-- Name: COLUMN care_medication_records.oral_meloxicam; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.oral_meloxicam IS '術後給藥：口服 Meloxicam';


--
-- Name: COLUMN care_medication_records.is_locked; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.care_medication_records.is_locked IS 'GLP 簽章鎖。同 animal_observations.is_locked。';


--
-- Name: change_reasons; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.change_reasons (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id text NOT NULL,
    change_type character varying(20) NOT NULL,
    reason text NOT NULL,
    old_values jsonb,
    new_values jsonb,
    changed_fields text[],
    changed_by uuid,
    changed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: change_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.change_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    change_number character varying(50) NOT NULL,
    title character varying(300) NOT NULL,
    change_type character varying(50) NOT NULL,
    description text NOT NULL,
    justification text,
    impact_assessment text,
    status character varying(30) DEFAULT 'draft'::character varying NOT NULL,
    requested_by uuid NOT NULL,
    reviewed_by uuid,
    reviewed_at timestamp with time zone,
    approved_by uuid,
    approved_at timestamp with time zone,
    implemented_at timestamp with time zone,
    verified_by uuid,
    verified_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: chart_of_accounts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.chart_of_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    code character varying(20) NOT NULL,
    name character varying(200) NOT NULL,
    account_type public.account_type NOT NULL,
    parent_id uuid,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: comp_time_balances; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.comp_time_balances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    overtime_record_id uuid NOT NULL,
    original_hours numeric(5,2) NOT NULL,
    used_hours numeric(5,2) DEFAULT 0,
    earned_date date NOT NULL,
    expires_at date NOT NULL,
    is_expired boolean DEFAULT false,
    expired_hours numeric(5,2) DEFAULT 0,
    converted_to_pay boolean DEFAULT false,
    expiry_processed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: competency_assessments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.competency_assessments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    assessment_type character varying(50) NOT NULL,
    skill_area character varying(200) NOT NULL,
    assessment_date date NOT NULL,
    assessor_id uuid NOT NULL,
    result character varying(30) NOT NULL,
    score numeric(5,2),
    method character varying(100),
    valid_until date,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT competency_assessments_result_check CHECK (((result)::text = ANY ((ARRAY['competent'::character varying, 'not_yet_competent'::character varying, 'requires_supervision'::character varying])::text[])))
);


--
-- Name: controlled_documents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.controlled_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    doc_number character varying(50) NOT NULL,
    title character varying(300) NOT NULL,
    doc_type character varying(50) NOT NULL,
    category character varying(100),
    current_version integer DEFAULT 1 NOT NULL,
    status character varying(30) DEFAULT 'draft'::character varying NOT NULL,
    effective_date date,
    review_due_date date,
    owner_id uuid,
    approved_by uuid,
    approved_at timestamp with time zone,
    obsoleted_at timestamp with time zone,
    retention_years integer,
    file_path text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: data_retention_policies; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.data_retention_policies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    table_name text NOT NULL,
    retention_years integer,
    delete_strategy text NOT NULL,
    description text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT data_retention_policies_delete_strategy_check CHECK ((delete_strategy = ANY (ARRAY['hard_delete'::text, 'partition_drop'::text, 'never'::text]))),
    CONSTRAINT data_retention_policies_table_name_check CHECK ((table_name ~ '^[a-z0-9_]+$'::text))
);


--
-- Name: TABLE data_retention_policies; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.data_retention_policies IS 'GLP §11.10(c) + OECD §10 record retention policy。為每個業務表定義保留年限與刪除策略；排程任務 services/retention_enforcer.rs 每日 03:00 UTC 跑，找出 deleted_at + retention_years 已過的 row 實際刪除。retention_years NULL = 永久；delete_strategy = never 表示僅文件用途。';


--
-- Name: COLUMN data_retention_policies.table_name; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.data_retention_policies.table_name IS '對應業務表名稱（與 information_schema.tables 對應）';


--
-- Name: COLUMN data_retention_policies.retention_years; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.data_retention_policies.retention_years IS 'NULL = 永久不刪；數值 = 滿年限後 hard delete';


--
-- Name: COLUMN data_retention_policies.delete_strategy; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.data_retention_policies.delete_strategy IS 'hard_delete / partition_drop / never';


--
-- Name: departments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.departments (
    id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    parent_id uuid,
    manager_id uuid,
    is_active boolean DEFAULT true NOT NULL,
    config jsonb,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: document_acknowledgments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.document_acknowledgments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL,
    user_id uuid NOT NULL,
    acknowledged_at timestamp with time zone DEFAULT now() NOT NULL,
    version_acknowledged integer NOT NULL
);


--
-- Name: document_lines; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.document_lines (
    id uuid NOT NULL,
    document_id uuid NOT NULL,
    line_no integer NOT NULL,
    product_id uuid NOT NULL,
    qty numeric(18,4) NOT NULL,
    uom character varying(20) NOT NULL,
    unit_price numeric(18,4),
    batch_no character varying(50),
    expiry_date date,
    remark text,
    storage_location_id uuid,
    storage_location_from_id uuid,
    storage_location_to_id uuid,
    warehouse_id uuid
);


--
-- Name: COLUMN document_lines.warehouse_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.document_lines.warehouse_id IS 'SO 多倉銷貨：該行所屬倉庫，以 storage_location_id 反推回填（ledger 跟隨儲位，延續 2026-07-18 裁定）。SO 核准逐行照此扣帳；其他單據沿用表頭 documents.warehouse_id（本欄可為 NULL）。';


--
-- Name: document_revisions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.document_revisions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL,
    version integer NOT NULL,
    change_summary text NOT NULL,
    revised_by uuid,
    reviewed_by uuid,
    approved_by uuid,
    approved_at timestamp with time zone,
    file_path text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: documents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.documents (
    id uuid NOT NULL,
    doc_type public.doc_type NOT NULL,
    doc_no character varying(50) NOT NULL,
    status public.doc_status DEFAULT 'draft'::public.doc_status NOT NULL,
    warehouse_id uuid,
    warehouse_from_id uuid,
    warehouse_to_id uuid,
    partner_id uuid,
    source_doc_id uuid,
    doc_date date NOT NULL,
    receipt_status character varying(20),
    stocktake_scope jsonb,
    remark text,
    iacuc_no character varying(20),
    requires_manager_approval boolean DEFAULT false,
    scrap_total_amount numeric,
    manager_approval_status character varying(20),
    manager_approved_by uuid,
    manager_approved_at timestamp with time zone,
    manager_reject_reason text,
    created_by uuid NOT NULL,
    approved_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    approved_at timestamp with time zone,
    protocol_id uuid,
    reverses_doc_id uuid,
    system_generated boolean DEFAULT false NOT NULL,
    CONSTRAINT chk_receipt_status CHECK (((receipt_status IS NULL) OR ((receipt_status)::text = ANY ((ARRAY['pending'::character varying, 'partial'::character varying, 'complete'::character varying])::text[]))))
);


--
-- Name: COLUMN documents.reverses_doc_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.documents.reverses_doc_id IS 'R84-5 紅字沖銷：本單為哪一張原始單據的沖銷單（放在沖銷單身上，指向原單；一般單據為 NULL）。partial unique index 保證一張原單最多一張沖銷單。';


--
-- Name: COLUMN documents.system_generated; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.documents.system_generated IS '單據是否由系統自動產生（非人工開立）。true 時 created_by 只是借用欄位、不代表真正的作者，故職務分離守衛（R97-1「建立者不得自核」）對其豁免。⚠️ 本欄位不得開放給任何 request DTO 設定，只能由 service 內部的自動產生路徑寫入，否則具備 create+approve 權限者可自行標記以繞過守衛。';


--
-- Name: electronic_signatures; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.electronic_signatures (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    entity_type character varying(50) NOT NULL,
    entity_id character varying(100) NOT NULL,
    signer_id uuid NOT NULL,
    signature_type character varying(20) NOT NULL,
    content_hash character varying(64) NOT NULL,
    signature_data character varying(128) NOT NULL,
    ip_address character varying(45),
    user_agent text,
    signed_at timestamp with time zone DEFAULT now() NOT NULL,
    is_valid boolean DEFAULT true NOT NULL,
    invalidated_reason text,
    invalidated_at timestamp with time zone,
    invalidated_by uuid,
    handwriting_svg text,
    stroke_data jsonb,
    signature_method character varying(20) DEFAULT 'password'::character varying,
    hmac_version smallint DEFAULT 1 NOT NULL,
    meaning public.signature_meaning NOT NULL
);


--
-- Name: COLUMN electronic_signatures.hmac_version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.electronic_signatures.hmac_version IS 'signature_data 編碼版本：1=SHA-256 legacy（pre-R30-7），2=HMAC-SHA256+secret。verify 時依此 dispatch。';


--
-- Name: COLUMN electronic_signatures.meaning; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.electronic_signatures.meaning IS '21 CFR §11.50(a)(3) 簽章意義（review/approval/responsibility/authorship 等）。
 由 service 層依 SignatureType 推導或 caller 顯式指定。
 LEGACY_PRE_R30_10 = R30-10 升級前的歷史資料（無顯式 meaning）。';


--
-- Name: environment_monitoring_points; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.environment_monitoring_points (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(200) NOT NULL,
    location_type character varying(50) NOT NULL,
    building_id uuid,
    zone_id uuid,
    parameters jsonb NOT NULL,
    monitoring_interval character varying(30),
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: environment_readings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.environment_readings (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    monitoring_point_id uuid NOT NULL,
    reading_time timestamp with time zone NOT NULL,
    readings jsonb NOT NULL,
    is_out_of_range boolean DEFAULT false NOT NULL,
    out_of_range_params jsonb,
    recorded_by uuid,
    source character varying(20) DEFAULT 'manual'::character varying NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: equipment; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(200) NOT NULL,
    model character varying(200),
    serial_number character varying(100),
    location character varying(200),
    notes text,
    is_active boolean DEFAULT true NOT NULL,
    status public.equipment_status DEFAULT 'active'::public.equipment_status NOT NULL,
    calibration_type public.calibration_type,
    calibration_cycle public.calibration_cycle,
    inspection_cycle public.calibration_cycle,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    purchase_date date,
    warranty_expiry date,
    department character varying(100)
);


--
-- Name: equipment_annual_plans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_annual_plans (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    year integer NOT NULL,
    equipment_id uuid NOT NULL,
    calibration_type public.calibration_type NOT NULL,
    cycle public.calibration_cycle NOT NULL,
    month_1 boolean DEFAULT false NOT NULL,
    month_2 boolean DEFAULT false NOT NULL,
    month_3 boolean DEFAULT false NOT NULL,
    month_4 boolean DEFAULT false NOT NULL,
    month_5 boolean DEFAULT false NOT NULL,
    month_6 boolean DEFAULT false NOT NULL,
    month_7 boolean DEFAULT false NOT NULL,
    month_8 boolean DEFAULT false NOT NULL,
    month_9 boolean DEFAULT false NOT NULL,
    month_10 boolean DEFAULT false NOT NULL,
    month_11 boolean DEFAULT false NOT NULL,
    month_12 boolean DEFAULT false NOT NULL,
    generated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: equipment_calibrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_calibrations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    equipment_id uuid NOT NULL,
    calibrated_at date NOT NULL,
    next_due_at date,
    result character varying(50),
    notes text,
    calibration_type public.calibration_type DEFAULT 'calibration'::public.calibration_type NOT NULL,
    partner_id uuid,
    report_number character varying(100),
    inspector character varying(100),
    equipment_serial_number character varying(100),
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    certificate_number character varying(100),
    performed_by character varying(100),
    acceptance_criteria character varying(200),
    measurement_uncertainty character varying(100),
    validation_phase public.validation_phase,
    protocol_number character varying(100),
    reference_standard_id uuid,
    calibration_lab_name character varying(200),
    calibration_lab_accreditation character varying(100),
    traceability_statement text,
    reading_before character varying(100),
    reading_after character varying(100)
);


--
-- Name: equipment_disposals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_disposals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    equipment_id uuid NOT NULL,
    status public.disposal_status DEFAULT 'pending'::public.disposal_status NOT NULL,
    disposal_date date,
    reason text NOT NULL,
    disposal_method text,
    applied_by uuid NOT NULL,
    applied_at timestamp with time zone DEFAULT now() NOT NULL,
    approved_by uuid,
    approved_at timestamp with time zone,
    rejection_reason text,
    applicant_signature_id uuid,
    approver_signature_id uuid,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: equipment_idle_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_idle_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    equipment_id uuid NOT NULL,
    request_type text NOT NULL,
    reason text NOT NULL,
    status public.disposal_status DEFAULT 'pending'::public.disposal_status NOT NULL,
    applied_by uuid NOT NULL,
    applied_at timestamp with time zone DEFAULT now() NOT NULL,
    applicant_signature_id uuid,
    approved_by uuid,
    approved_at timestamp with time zone,
    approver_signature_id uuid,
    rejection_reason text,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT equipment_idle_requests_request_type_check CHECK ((request_type = ANY (ARRAY['idle'::text, 'restore'::text])))
);


--
-- Name: equipment_maintenance_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_maintenance_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    equipment_id uuid NOT NULL,
    maintenance_type public.maintenance_type NOT NULL,
    status public.maintenance_status DEFAULT 'pending'::public.maintenance_status NOT NULL,
    reported_at date NOT NULL,
    completed_at date,
    problem_description text,
    repair_content text,
    repair_partner_id uuid,
    maintenance_items text,
    performed_by character varying(100),
    notes text,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    reviewed_by uuid,
    reviewed_at timestamp with time zone,
    reviewer_signature_id uuid,
    review_notes text
);


--
-- Name: equipment_status_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_status_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    equipment_id uuid NOT NULL,
    old_status public.equipment_status NOT NULL,
    new_status public.equipment_status NOT NULL,
    changed_by uuid NOT NULL,
    reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: equipment_suppliers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.equipment_suppliers (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    equipment_id uuid NOT NULL,
    partner_id uuid NOT NULL,
    contact_person character varying(100),
    contact_phone character varying(50),
    contact_email character varying(255),
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: euthanasia_appeals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.euthanasia_appeals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    order_id uuid NOT NULL,
    pi_user_id uuid NOT NULL,
    reason text NOT NULL,
    attachment_path character varying(500),
    chair_user_id uuid,
    chair_decision character varying(50),
    chair_decided_at timestamp with time zone,
    chair_deadline_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    version integer DEFAULT 1 NOT NULL
);


--
-- Name: COLUMN euthanasia_appeals.version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_appeals.version IS 'Optimistic lock version。語意同 euthanasia_orders.version。';


--
-- Name: euthanasia_byproduct_samples; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.euthanasia_byproduct_samples (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    euthanasia_id uuid,
    animal_id uuid NOT NULL,
    source_protocol_id uuid NOT NULL,
    sampled_at timestamp with time zone NOT NULL,
    sample_content text NOT NULL,
    requester_user_id uuid,
    collector_id uuid NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    created_by uuid NOT NULL,
    deleted_at timestamp with time zone,
    requester_org_name text,
    requester_contact_name text,
    special_equipment_used text,
    work_started_at timestamp with time zone,
    work_ended_at timestamp with time zone,
    CONSTRAINT byproduct_samples_requester_present CHECK (((requester_user_id IS NOT NULL) OR ((requester_org_name IS NOT NULL) AND (length(TRIM(BOTH FROM requester_org_name)) > 0) AND (requester_contact_name IS NOT NULL) AND (length(TRIM(BOTH FROM requester_contact_name)) > 0)))),
    CONSTRAINT byproduct_samples_work_time_order CHECK (((work_started_at IS NULL) OR (work_ended_at IS NULL) OR (work_ended_at >= work_started_at)))
);


--
-- Name: TABLE euthanasia_byproduct_samples; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.euthanasia_byproduct_samples IS 'R53-1: 廢棄物再利用紀錄 — 結案豬隻組織/血液多採給其他研究方。PI viewer 不可見此 entity_type。';


--
-- Name: COLUMN euthanasia_byproduct_samples.requester_user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.requester_user_id IS 'in-system 研究人員 FK。external researcher 用 requester_text，兩欄至少一個有值。';


--
-- Name: COLUMN euthanasia_byproduct_samples.collector_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.collector_id IS '採樣者（必須為已登入的 vet 角色，ActorContext::User 強制）。';


--
-- Name: COLUMN euthanasia_byproduct_samples.requester_org_name; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.requester_org_name IS 'R53-14: external requester 機構名（如「國防醫學大學」）。FK 為 NULL 時必填，並要求 contact 也填。';


--
-- Name: COLUMN euthanasia_byproduct_samples.requester_contact_name; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.requester_contact_name IS 'R53-14: external requester 聯絡人姓名（如「王教授」）。FK 為 NULL 時必填，並要求 org 也填。';


--
-- Name: COLUMN euthanasia_byproduct_samples.special_equipment_used; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.special_equipment_used IS 'R53-14: 特殊儀器使用紀錄（billing 報表欄位，自由文字）。';


--
-- Name: COLUMN euthanasia_byproduct_samples.work_started_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.work_started_at IS 'R53-14: 採樣 / 處理工作開始時間。total hours 由 report query 即算（end - start），不持久化。';


--
-- Name: COLUMN euthanasia_byproduct_samples.work_ended_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_byproduct_samples.work_ended_at IS 'R53-14: 採樣 / 處理工作結束時間。兩欄都有值時 CHECK 要求 end >= start。';


--
-- Name: euthanasia_orders; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.euthanasia_orders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    animal_id uuid NOT NULL,
    vet_user_id uuid NOT NULL,
    pi_user_id uuid NOT NULL,
    status public.euthanasia_order_status DEFAULT 'pending_pi'::public.euthanasia_order_status NOT NULL,
    reason text NOT NULL,
    deadline_at timestamp with time zone NOT NULL,
    pi_responded_at timestamp with time zone,
    executed_at timestamp with time zone,
    executed_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    version integer DEFAULT 1 NOT NULL
);


--
-- Name: COLUMN euthanasia_orders.version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.euthanasia_orders.version IS 'Optimistic lock version。每次 UPDATE 時 +1，service 層 WHERE version = $n 命中 0 row 即回 409 Conflict。';


--
-- Name: event_outbox; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.event_outbox (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    channel text NOT NULL,
    payload jsonb NOT NULL,
    status text DEFAULT 'PENDING'::text NOT NULL,
    attempt_count integer DEFAULT 0 NOT NULL,
    next_attempt_at timestamp with time zone DEFAULT now() NOT NULL,
    last_error text,
    enqueued_by uuid,
    enqueued_at timestamp with time zone DEFAULT now() NOT NULL,
    started_at timestamp with time zone,
    done_at timestamp with time zone,
    source_entity text,
    source_entity_id uuid,
    CONSTRAINT event_outbox_attempt_count_check CHECK ((attempt_count >= 0)),
    CONSTRAINT event_outbox_status_check CHECK ((status = ANY (ARRAY['PENDING'::text, 'SENDING'::text, 'DONE'::text, 'FAILED'::text, 'DEAD'::text])))
);


--
-- Name: TABLE event_outbox; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.event_outbox IS 'R30-3a: Transactional outbox for guaranteed-delivery side effects (notifications, webhooks). Worker: bin/outbox_worker.rs';


--
-- Name: COLUMN event_outbox.channel; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.event_outbox.channel IS 'Adapter routing key: email / line / webhook / reindex. ChannelRegistry::send 依此分流';


--
-- Name: COLUMN event_outbox.attempt_count; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.event_outbox.attempt_count IS '已失敗次數（default 0）。mark_failed 先 +1 再依新值算 next_attempt_at。6 次失敗後狀態變 DEAD';


--
-- Name: COLUMN event_outbox.next_attempt_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.event_outbox.next_attempt_at IS '下次 worker 嘗試送出的最早時間。claim_batch 用 WHERE next_attempt_at <= NOW()';


--
-- Name: COLUMN event_outbox.source_entity; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.event_outbox.source_entity IS '入隊來源 entity 類型（amendment / euthanasia 等），方便從 outbox row 反查業務紀錄';


--
-- Name: expiry_monthly_snapshots; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.expiry_monthly_snapshots (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    snapshot_ym character(7) NOT NULL,
    product_id uuid NOT NULL,
    sku character varying(80) NOT NULL,
    product_name character varying(255) NOT NULL,
    warehouse_id uuid NOT NULL,
    batch_no character varying(100),
    expiry_date date NOT NULL,
    days_past smallint NOT NULL,
    on_hand_qty numeric(15,4) NOT NULL,
    base_uom character varying(20) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: TABLE expiry_monthly_snapshots; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.expiry_monthly_snapshots IS '每月執行月度效期通知時的品項快照，供下月比對新增/減少';


--
-- Name: COLUMN expiry_monthly_snapshots.snapshot_ym; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.expiry_monthly_snapshots.snapshot_ym IS '快照月份，格式 YYYY-MM';


--
-- Name: COLUMN expiry_monthly_snapshots.days_past; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.expiry_monthly_snapshots.days_past IS '拍照時距效期已過幾天（負值=尚未過期）';


--
-- Name: expiry_notification_config; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.expiry_notification_config (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    warn_days smallint DEFAULT 60 NOT NULL,
    cutoff_days smallint DEFAULT 90 NOT NULL,
    monthly_threshold_days smallint,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_by uuid,
    CONSTRAINT chk_enc_cutoff CHECK (((cutoff_days >= 1) AND (cutoff_days <= 730))),
    CONSTRAINT chk_enc_monthly CHECK (((monthly_threshold_days IS NULL) OR ((monthly_threshold_days >= 1) AND (monthly_threshold_days <= 730)))),
    CONSTRAINT chk_enc_warn CHECK (((warn_days >= 1) AND (warn_days <= 365)))
);


--
-- Name: TABLE expiry_notification_config; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.expiry_notification_config IS '系統層級效期通知範圍設定（全系統僅一列）';


--
-- Name: COLUMN expiry_notification_config.warn_days; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.expiry_notification_config.warn_days IS '提前幾天開始預警（預設 60）';


--
-- Name: COLUMN expiry_notification_config.cutoff_days; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.expiry_notification_config.cutoff_days IS '過期超過幾天後停止通知（預設 90）';


--
-- Name: COLUMN expiry_notification_config.monthly_threshold_days; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.expiry_notification_config.monthly_threshold_days IS '過期超過此天數後轉月度彙整通知；NULL=停用';


--
-- Name: export_jobs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.export_jobs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    export_type public.export_type NOT NULL,
    export_format public.export_format NOT NULL,
    status public.import_status DEFAULT 'pending'::public.import_status NOT NULL,
    file_name character varying(255),
    file_path character varying(500),
    parameters jsonb,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: facilities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.facilities (
    id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    address text,
    phone character varying(50),
    contact_person character varying(100),
    is_active boolean DEFAULT true NOT NULL,
    config jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: formulation_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.formulation_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    product_id uuid NOT NULL,
    protocol_id uuid,
    formulation_date date NOT NULL,
    batch_number character varying(100),
    concentration character varying(100),
    volume character varying(100),
    prepared_by uuid NOT NULL,
    verified_by uuid,
    verified_at timestamp with time zone,
    expiry_date date,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: google_calendar_config; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.google_calendar_config (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    calendar_id character varying(255) DEFAULT ''::character varying NOT NULL,
    calendar_name character varying(255),
    calendar_description text,
    auth_method character varying(50) DEFAULT 'shared_account'::character varying NOT NULL,
    auth_email character varying(255),
    is_configured boolean DEFAULT false NOT NULL,
    sync_enabled boolean DEFAULT false NOT NULL,
    sync_schedule_morning time without time zone,
    sync_schedule_evening time without time zone,
    sync_timezone character varying(50),
    sync_approved_leaves boolean DEFAULT true NOT NULL,
    sync_overtime boolean DEFAULT false NOT NULL,
    event_title_template character varying(255),
    event_color_id character varying(20),
    last_sync_at timestamp with time zone,
    last_sync_status character varying(50),
    last_sync_error text,
    last_sync_events_pushed integer,
    last_sync_events_pulled integer,
    last_sync_conflicts integer,
    last_sync_duration_ms integer,
    next_sync_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: import_jobs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.import_jobs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    import_type public.import_type NOT NULL,
    status public.import_status DEFAULT 'pending'::public.import_status NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path character varying(500) NOT NULL,
    total_rows integer,
    processed_rows integer DEFAULT 0,
    success_rows integer DEFAULT 0,
    error_rows integer DEFAULT 0,
    error_details jsonb,
    started_at timestamp with time zone,
    completed_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: inventory_snapshots; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.inventory_snapshots (
    warehouse_id uuid NOT NULL,
    product_id uuid NOT NULL,
    on_hand_qty_base numeric(18,4) DEFAULT 0 NOT NULL,
    avg_cost numeric(18,4),
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_inventory_snapshots_on_hand_nonneg CHECK ((on_hand_qty_base >= (0)::numeric))
);


--
-- Name: invitation_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.invitation_roles (
    invitation_id uuid NOT NULL,
    role_id uuid NOT NULL
);


--
-- Name: invitations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.invitations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    email character varying(255) NOT NULL,
    organization character varying(255),
    invitation_token character varying(255) NOT NULL,
    invited_by uuid NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    accepted_at timestamp with time zone,
    created_user_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    display_name character varying(100),
    phone character varying(20),
    "position" character varying(100),
    is_internal boolean NOT NULL,
    CONSTRAINT invitations_status_check CHECK (((status)::text = ANY ((ARRAY['pending'::character varying, 'accepted'::character varying, 'expired'::character varying, 'revoked'::character varying])::text[])))
);


--
-- Name: COLUMN invitations.is_internal; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.invitations.is_internal IS '受邀者是否為本場受僱人員。決定接受邀請後建立的 users.is_internal，連帶影響能否被加入部門、以及是否出現在特休/加班/訓練等人事清單。由邀請人在建立邀請時判斷，不由角色推導（同一角色可能是內部或外部人員）。';


--
-- Name: ip_blocklist; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.ip_blocklist (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    ip_address inet NOT NULL,
    reason text NOT NULL,
    source text NOT NULL,
    alert_id uuid,
    blocked_at timestamp with time zone DEFAULT now() NOT NULL,
    blocked_until timestamp with time zone,
    blocked_by uuid,
    hit_count bigint DEFAULT 0 NOT NULL,
    last_hit_at timestamp with time zone,
    unblocked_at timestamp with time zone,
    unblocked_by uuid,
    unblocked_reason text,
    metadata jsonb,
    CONSTRAINT chk_ip_blocklist_source CHECK ((source = ANY (ARRAY['R22-6_idor'::text, 'R22-1_ratelimit'::text, 'honeypot'::text, 'manual'::text])))
);


--
-- Name: journal_entries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.journal_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    entry_no character varying(50) NOT NULL,
    entry_date date NOT NULL,
    description text,
    source_entity_type character varying(50),
    source_entity_id uuid,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: journal_entry_lines; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.journal_entry_lines (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    journal_entry_id uuid NOT NULL,
    line_no integer NOT NULL,
    account_id uuid NOT NULL,
    debit_amount numeric(18,4) DEFAULT 0 NOT NULL,
    credit_amount numeric(18,4) DEFAULT 0 NOT NULL,
    description text,
    CONSTRAINT chk_debit_credit CHECK (((debit_amount >= (0)::numeric) AND (credit_amount >= (0)::numeric) AND (((debit_amount > (0)::numeric) AND (credit_amount = (0)::numeric)) OR ((debit_amount = (0)::numeric) AND (credit_amount > (0)::numeric)))))
);


--
-- Name: journal_entry_no_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.journal_entry_no_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: jwt_blacklist; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.jwt_blacklist (
    jti character varying(64) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    revoked_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: leave_approvals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leave_approvals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    leave_request_id uuid NOT NULL,
    approver_id uuid NOT NULL,
    approval_level character varying(20) NOT NULL,
    action character varying(20) NOT NULL,
    comments text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_action CHECK (((action)::text = ANY ((ARRAY['APPROVE'::character varying, 'REJECT'::character varying, 'REQUEST_REVISION'::character varying, 'ESCALATE'::character varying])::text[])))
);


--
-- Name: leave_balance_usage; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leave_balance_usage (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    leave_request_id uuid NOT NULL,
    source_type character varying(20) NOT NULL,
    annual_leave_entitlement_id uuid,
    comp_time_balance_id uuid,
    days_used numeric(5,2),
    hours_used numeric(5,2),
    action character varying(20) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_balance_action CHECK (((action)::text = ANY ((ARRAY['deduct'::character varying, 'restore'::character varying])::text[]))),
    CONSTRAINT chk_source_type CHECK (((source_type)::text = ANY ((ARRAY['annual'::character varying, 'comp_time'::character varying])::text[])))
);


--
-- Name: leave_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.leave_requests (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    proxy_user_id uuid,
    leave_type public.leave_type NOT NULL,
    start_date date NOT NULL,
    end_date date NOT NULL,
    start_time time without time zone,
    end_time time without time zone,
    total_days numeric(5,2) NOT NULL,
    total_hours numeric(5,2),
    reason text,
    supporting_documents jsonb DEFAULT '[]'::jsonb,
    annual_leave_source_id uuid,
    is_urgent boolean DEFAULT false,
    is_retroactive boolean DEFAULT false,
    status public.leave_status DEFAULT 'DRAFT'::public.leave_status,
    current_approver_id uuid,
    submitted_at timestamp with time zone,
    approved_at timestamp with time zone,
    rejected_at timestamp with time zone,
    cancelled_at timestamp with time zone,
    revoked_at timestamp with time zone,
    cancellation_reason text,
    revocation_reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: line_shelf_allocations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.line_shelf_allocations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_line_id uuid,
    storage_location_id uuid NOT NULL,
    product_id uuid NOT NULL,
    batch_no character varying(50),
    expiry_date date,
    qty numeric(18,4) NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT line_shelf_allocations_qty_check CHECK ((qty > (0)::numeric))
);


--
-- Name: login_events; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.login_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid,
    email character varying(255) NOT NULL,
    event_type character varying(20) NOT NULL,
    ip_address inet,
    user_agent text,
    device_type character varying(50),
    browser character varying(50),
    os character varying(50),
    geo_country character varying(100),
    geo_city character varying(100),
    geo_timezone character varying(50),
    is_unusual_time boolean DEFAULT false,
    is_unusual_location boolean DEFAULT false,
    is_new_device boolean DEFAULT false,
    is_mass_login boolean DEFAULT false,
    device_fingerprint character varying(255),
    failure_reason character varying(100),
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: management_reviews; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.management_reviews (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    review_number character varying(50) NOT NULL,
    title character varying(300) NOT NULL,
    review_date date NOT NULL,
    status character varying(30) DEFAULT 'planned'::character varying NOT NULL,
    agenda text,
    attendees jsonb,
    minutes text,
    decisions jsonb,
    action_items jsonb,
    chaired_by uuid,
    approved_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: message_attachments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_attachments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    message_id uuid,
    uploaded_by uuid NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path text NOT NULL,
    file_size bigint NOT NULL,
    mime_type character varying(100) NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: message_thread_participants; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_thread_participants (
    thread_id uuid NOT NULL,
    user_id uuid NOT NULL,
    joined_at timestamp with time zone DEFAULT now() NOT NULL,
    last_read_at timestamp with time zone,
    left_at timestamp with time zone
);


--
-- Name: message_threads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.message_threads (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    type character varying(20) NOT NULL,
    subject character varying(200),
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_message_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    CONSTRAINT message_threads_type_check CHECK (((type)::text = ANY ((ARRAY['direct'::character varying, 'group'::character varying])::text[])))
);


--
-- Name: messages; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    thread_id uuid NOT NULL,
    sender_id uuid NOT NULL,
    body text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    edited_at timestamp with time zone,
    deleted_at timestamp with time zone
);


--
-- Name: notification_routing; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notification_routing (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    event_type character varying(80) NOT NULL,
    role_code character varying(50),
    channel character varying(20) DEFAULT 'in_app'::character varying NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    description text,
    frequency character varying(20) DEFAULT 'immediate'::character varying NOT NULL,
    hour_of_day smallint DEFAULT 8 NOT NULL,
    day_of_week smallint,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    target_kind character varying(20) DEFAULT 'role'::character varying NOT NULL,
    target_value character varying(80) NOT NULL,
    CONSTRAINT chk_channel CHECK (((channel)::text = ANY ((ARRAY['in_app'::character varying, 'email'::character varying, 'both'::character varying])::text[]))),
    CONSTRAINT chk_nr_dow CHECK (((day_of_week >= 0) AND (day_of_week <= 6))),
    CONSTRAINT chk_nr_frequency CHECK (((frequency)::text = ANY ((ARRAY['immediate'::character varying, 'daily'::character varying, 'weekly'::character varying, 'monthly'::character varying])::text[]))),
    CONSTRAINT chk_nr_hour CHECK (((hour_of_day >= 0) AND (hour_of_day <= 23))),
    CONSTRAINT chk_nr_target_kind CHECK (((target_kind)::text = ANY ((ARRAY['role'::character varying, 'resolver'::character varying])::text[])))
);


--
-- Name: COLUMN notification_routing.frequency; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.notification_routing.frequency IS 'immediate=事件即時觸發, daily/weekly/monthly=批次排程';


--
-- Name: COLUMN notification_routing.hour_of_day; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.notification_routing.hour_of_day IS '批次通知的執行小時（0-23），immediate 時忽略';


--
-- Name: COLUMN notification_routing.day_of_week; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.notification_routing.day_of_week IS 'weekly 時有效：0=週日, 1=週一 ... 6=週六';


--
-- Name: COLUMN notification_routing.target_kind; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.notification_routing.target_kind IS 'role=持有該角色者; resolver=關係型收件人解析器（key 見 services/notification/resolvers.rs）';


--
-- Name: COLUMN notification_routing.target_value; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.notification_routing.target_value IS 'target_kind=role 時為 roles.code；target_kind=resolver 時為 resolver key';


--
-- Name: notification_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notification_settings (
    user_id uuid NOT NULL,
    email_low_stock boolean DEFAULT true NOT NULL,
    email_expiry_warning boolean DEFAULT true NOT NULL,
    email_document_approval boolean DEFAULT true NOT NULL,
    email_protocol_status boolean DEFAULT true NOT NULL,
    email_monthly_report boolean DEFAULT true NOT NULL,
    expiry_warning_days integer DEFAULT 30 NOT NULL,
    low_stock_notify_immediately boolean DEFAULT true NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notifications (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    type public.notification_type NOT NULL,
    title character varying(200) NOT NULL,
    content text,
    is_read boolean DEFAULT false NOT NULL,
    read_at timestamp with time zone,
    related_entity_type character varying(50),
    related_entity_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    priority smallint DEFAULT 0 NOT NULL,
    kind text DEFAULT 'info'::text NOT NULL,
    recipient_role text,
    CONSTRAINT chk_notifications_kind CHECK ((kind = ANY (ARRAY['info'::text, 'action'::text]))),
    CONSTRAINT chk_notifications_priority CHECK (((priority >= 0) AND (priority <= 1))),
    CONSTRAINT chk_notifications_recipient_role CHECK (((recipient_role IS NULL) OR (recipient_role = ANY (ARRAY['proxy'::text, 'approver'::text]))))
);


--
-- Name: COLUMN notifications.priority; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.notifications.priority IS '0=一般；1=緊急置頂（採購未入庫、巡場待填追蹤等待辦，完成後由 hook 降級回 0）';


--
-- Name: observation_vet_reads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.observation_vet_reads (
    observation_id uuid NOT NULL,
    vet_user_id uuid NOT NULL,
    read_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: overtime_approvals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.overtime_approvals (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    overtime_record_id uuid NOT NULL,
    approver_id uuid NOT NULL,
    approval_level character varying(20) NOT NULL,
    action character varying(20) NOT NULL,
    comments text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_overtime_approval_action CHECK (((action)::text = ANY ((ARRAY['APPROVE'::character varying, 'REJECT'::character varying])::text[])))
);


--
-- Name: overtime_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.overtime_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    attendance_id uuid,
    overtime_date date NOT NULL,
    start_time timestamp with time zone NOT NULL,
    end_time timestamp with time zone NOT NULL,
    hours numeric(5,2) NOT NULL,
    overtime_type character varying(20) NOT NULL,
    multiplier numeric(3,2) DEFAULT 1.0,
    comp_time_hours numeric(5,2) NOT NULL,
    comp_time_expires_at date NOT NULL,
    comp_time_used_hours numeric(5,2) DEFAULT 0,
    status character varying(20) DEFAULT 'draft'::character varying,
    submitted_at timestamp with time zone,
    approved_by uuid,
    approved_at timestamp with time zone,
    rejected_by uuid,
    rejected_at timestamp with time zone,
    rejection_reason text,
    reason text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    calc_unit character varying(10) DEFAULT 'hour'::character varying NOT NULL,
    tier1_hours numeric(5,2) DEFAULT 0 NOT NULL,
    tier2_hours numeric(5,2) DEFAULT 0 NOT NULL,
    weighted_hours numeric(6,2) DEFAULT 0 NOT NULL,
    day_count numeric(4,1) DEFAULT 0 NOT NULL,
    voided_by uuid,
    voided_at timestamp with time zone,
    void_reason text,
    CONSTRAINT chk_overtime_calc_unit CHECK (((calc_unit)::text = ANY ((ARRAY['hour'::character varying, 'day'::character varying])::text[]))),
    CONSTRAINT chk_overtime_status CHECK (((status)::text = ANY ((ARRAY['draft'::character varying, 'pending'::character varying, 'pending_admin_staff'::character varying, 'pending_admin'::character varying, 'approved'::character varying, 'rejected'::character varying, 'voided'::character varying])::text[]))),
    CONSTRAINT chk_overtime_type CHECK (((overtime_type)::text = ANY ((ARRAY['A'::character varying, 'B'::character varying, 'C'::character varying, 'D'::character varying])::text[])))
);


--
-- Name: COLUMN overtime_records.calc_unit; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.calc_unit IS '計費單位：hour=平日按時數分段(A)；day=值班按天(B/C/D)';


--
-- Name: COLUMN overtime_records.tier1_hours; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.tier1_hours IS '平日加班前 2 小時時數（×1.33）';


--
-- Name: COLUMN overtime_records.tier2_hours; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.tier2_hours IS '平日加班超過 2 小時時數（×1.66）';


--
-- Name: COLUMN overtime_records.weighted_hours; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.weighted_hours IS '加權係數時數 = tier1×1.33 + tier2×1.66（供薪資模組換算加班費）';


--
-- Name: COLUMN overtime_records.day_count; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.day_count IS '值班天數（calc_unit=day 時使用，B/C/D）';


--
-- Name: COLUMN overtime_records.voided_by; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.voided_by IS 'R86-2 作廢通道：作廢此已核准加班單的管理者（ADMIN 單簽）。';


--
-- Name: COLUMN overtime_records.void_reason; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.overtime_records.void_reason IS 'R86-2 作廢通道：作廢理由（必填，一併寫入 user_activity_logs 稽核鏈）。';


--
-- Name: partners; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.partners (
    id uuid NOT NULL,
    partner_type public.partner_type NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    supplier_category public.supplier_category,
    customer_category public.customer_category,
    tax_id character varying(50),
    phone character varying(50),
    phone_ext character varying(20),
    email character varying(255),
    address text,
    payment_terms character varying(100),
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.password_reset_tokens (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    token_hash character varying(255) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: pdf_artifacts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pdf_artifacts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    resource_type text NOT NULL,
    resource_id text NOT NULL,
    pdf_blob_hash text NOT NULL,
    pdf_size_bytes bigint NOT NULL,
    doc_type text NOT NULL,
    generated_by uuid NOT NULL,
    generated_at timestamp with time zone DEFAULT now() NOT NULL,
    request_ip inet,
    user_agent text,
    electronic_signature_id uuid,
    hmac_chain_link text,
    attachment_id uuid,
    storage_strategy text DEFAULT 'attachment'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT pdf_artifacts_attachment_consistency CHECK ((((storage_strategy = 'attachment'::text) AND (attachment_id IS NOT NULL)) OR ((storage_strategy = 'transient'::text) AND (attachment_id IS NULL)) OR (storage_strategy = 's3'::text))),
    CONSTRAINT pdf_artifacts_pdf_blob_hash_check CHECK ((pdf_blob_hash ~ '^[0-9a-f]{64}$'::text)),
    CONSTRAINT pdf_artifacts_pdf_size_bytes_check CHECK ((pdf_size_bytes >= 0)),
    CONSTRAINT pdf_artifacts_storage_strategy_check CHECK ((storage_strategy = ANY (ARRAY['attachment'::text, 'transient'::text, 's3'::text])))
);


--
-- Name: TABLE pdf_artifacts; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.pdf_artifacts IS 'R32-A6: PDF 匯出永久存證。每次「下載 PDF」按鈕觸發 → 寫一筆。21 CFR §11.50（signature meaning）+ §11.10(c)（record integrity）對齊。';


--
-- Name: COLUMN pdf_artifacts.pdf_blob_hash; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.pdf_artifacts.pdf_blob_hash IS 'SHA-256 hex of PDF binary。日後若 PDF 檔案被竄改（手動編輯 / 重新壓縮），可重算 hash 比對發現不一致 → trigger GLP 警報。';


--
-- Name: COLUMN pdf_artifacts.electronic_signature_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.pdf_artifacts.electronic_signature_id IS 'NULL = 純檢視匯出（如內部 review 用）；NOT NULL = 含 §11.50 簽章（PI/chair 正式提交版本）。caller 負責決定是否觸發簽章流程（產 PDF 前先 SignatureService::sign_record_tx）。';


--
-- Name: COLUMN pdf_artifacts.hmac_chain_link; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.pdf_artifacts.hmac_chain_link IS '對應的 user_activity_logs.id（PDF_EXPORT_REQUESTED 事件）。NULL 罕見 — scheduler / batch 路徑可能省略 chain 寫入，仍保留 artifact 紀錄。';


--
-- Name: COLUMN pdf_artifacts.storage_strategy; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.pdf_artifacts.storage_strategy IS 'attachment = 透過 attachments 表存盤（預設）；transient = 不存盤（即時下載即丟）；s3 = 預留未來雲端儲存實作（migration 別處 add column 時實作）。';


--
-- Name: pens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pens (
    id uuid NOT NULL,
    zone_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200),
    capacity integer DEFAULT 1 NOT NULL,
    current_count integer DEFAULT 0 NOT NULL,
    status character varying(50) DEFAULT 'active'::character varying NOT NULL,
    row_index integer,
    col_index integer,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.permissions (
    id uuid NOT NULL,
    code character varying(100) NOT NULL,
    name character varying(200) NOT NULL,
    module character varying(50),
    description text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: pi_account_invites; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.pi_account_invites (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    pi_user_id uuid NOT NULL,
    email character varying(255) NOT NULL,
    status character varying(20) DEFAULT 'pending'::character varying NOT NULL,
    provisioned_by uuid NOT NULL,
    provisioned_at timestamp with time zone DEFAULT now() NOT NULL,
    approved_by uuid,
    sent_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: planned_experiments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.planned_experiments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    unit text NOT NULL,
    description text,
    demand_count integer DEFAULT 0 NOT NULL,
    protocol_id uuid,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT planned_experiments_demand_count_check CHECK ((demand_count >= 0))
);


--
-- Name: product_categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_categories (
    id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    parent_id uuid,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: product_uom_conversions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.product_uom_conversions (
    id uuid NOT NULL,
    product_id uuid NOT NULL,
    uom character varying(20) NOT NULL,
    factor_to_base numeric(18,6) NOT NULL
);


--
-- Name: products; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.products (
    id uuid NOT NULL,
    sku character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    spec text,
    category_id uuid,
    category_code character(3),
    subcategory_code character(3),
    base_uom character varying(20) DEFAULT 'pcs'::character varying NOT NULL,
    pack_unit character varying(20),
    pack_qty integer,
    track_batch boolean DEFAULT false NOT NULL,
    track_expiry boolean DEFAULT false NOT NULL,
    default_expiry_days integer,
    safety_stock numeric(18,4),
    safety_stock_uom character varying(20),
    reorder_point numeric(18,4),
    reorder_point_uom character varying(20),
    image_url character varying(500),
    license_no character varying(100),
    storage_condition character varying(50),
    barcode character varying(50),
    tags text[],
    status character varying(20) DEFAULT 'active'::character varying NOT NULL,
    remark text,
    is_active boolean DEFAULT true NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    glp_characterization text,
    stability_data jsonb,
    storage_conditions character varying(200),
    cas_number character varying(50),
    is_test_article boolean DEFAULT false NOT NULL,
    is_control_article boolean DEFAULT false NOT NULL,
    cost_price numeric(18,4),
    selling_price numeric(18,4),
    CONSTRAINT chk_product_status CHECK (((status)::text = ANY ((ARRAY['active'::character varying, 'inactive'::character varying, 'discontinued'::character varying])::text[])))
);


--
-- Name: COLUMN products.cost_price; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.products.cost_price IS 'R35-16: 標準成本/最近採購均價（NUMERIC(18,4)）。NULL 代表尚未維護';


--
-- Name: COLUMN products.selling_price; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.products.selling_price IS 'R35-16: 目前對外定價（NUMERIC(18,4)）。NULL 代表尚未維護；於庫存價值計算中不計入';


--
-- Name: protocol_activities; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocol_activities (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    activity_type public.protocol_activity_type NOT NULL,
    actor_id uuid NOT NULL,
    actor_name character varying(100),
    actor_email character varying(255),
    from_value text,
    to_value text,
    target_entity_type character varying(50),
    target_entity_id uuid,
    target_entity_name character varying(255),
    remark text,
    extra_data jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: protocol_ai_reviews; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocol_ai_reviews (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    protocol_version_id uuid,
    review_type character varying(30) NOT NULL,
    rule_result jsonb,
    ai_result jsonb,
    ai_model character varying(50),
    ai_input_tokens integer,
    ai_output_tokens integer,
    total_errors integer DEFAULT 0 NOT NULL,
    total_warnings integer DEFAULT 0 NOT NULL,
    score integer,
    triggered_by uuid,
    duration_ms integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT protocol_ai_reviews_review_type_check CHECK (((review_type)::text = ANY ((ARRAY['client_pre_submit'::character varying, 'staff_pre_review'::character varying])::text[])))
);


--
-- Name: protocol_attachments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocol_attachments (
    id uuid NOT NULL,
    protocol_version_id uuid,
    protocol_id uuid,
    file_name character varying(255) NOT NULL,
    file_path character varying(500) NOT NULL,
    file_size integer NOT NULL,
    mime_type character varying(100) NOT NULL,
    uploaded_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: protocol_notice_acknowledgements; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocol_notice_acknowledgements (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    notice_id uuid NOT NULL,
    signer_id uuid NOT NULL,
    signature_id uuid,
    notice_attachment_id uuid,
    acknowledged_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: protocol_template_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocol_template_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    version_label text NOT NULL,
    effective_date date,
    is_current boolean DEFAULT false NOT NULL,
    notes text,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: protocol_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocol_versions (
    id uuid NOT NULL,
    protocol_id uuid NOT NULL,
    version_no integer NOT NULL,
    content_snapshot jsonb NOT NULL,
    submitted_at timestamp with time zone DEFAULT now() NOT NULL,
    submitted_by uuid NOT NULL
);


--
-- Name: protocols; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.protocols (
    id uuid NOT NULL,
    protocol_no character varying(50) NOT NULL,
    iacuc_no character varying(50),
    title character varying(500) NOT NULL,
    status public.protocol_status DEFAULT 'DRAFT'::public.protocol_status NOT NULL,
    pi_user_id uuid NOT NULL,
    working_content jsonb,
    start_date date,
    end_date date,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    version integer DEFAULT 1 NOT NULL,
    submitted_at date,
    approved_at date,
    application_no text,
    import_pending boolean DEFAULT false NOT NULL,
    original_version_label text,
    study_director_user_id uuid,
    imported_at timestamp with time zone,
    source_form_version text
);


--
-- Name: COLUMN protocols.application_no; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.protocols.application_no IS '申請編號（例 APIG-103001），與 iacuc_no（核准編號 PIG-115001）區分；匯入補登用';


--
-- Name: COLUMN protocols.import_pending; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.protocols.import_pending IS '匯入補登中：status=APPROVED 但允許編輯 working_content；按「完成補登」後清除（import P1）';


--
-- Name: COLUMN protocols.original_version_label; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.protocols.original_version_label IS '原計劃書版本號文字（補登，例 v2.1）；完成補登時填入';


--
-- Name: COLUMN protocols.study_director_user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.protocols.study_director_user_id IS '計劃負責人 / Study Director（本公司員工，自 EXPERIMENT_STAFF 挑選）；補登中可編輯內容並完成補登';


--
-- Name: COLUMN protocols.imported_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.protocols.imported_at IS 'import_approved 建立時間；非 NULL = 補登匯入計劃（永久標記，與暫態 import_pending 區分）。補登歷史變更僅允許於此類計劃';


--
-- Name: COLUMN protocols.source_form_version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.protocols.source_form_version IS '計畫書表單版本鍵（C/D/E/F…）；驅動版本名冊 manifest 渲染；null=最新版；變更升級時更新';


--
-- Name: qa_audit_schedules; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_audit_schedules (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    year integer NOT NULL,
    title character varying(255) NOT NULL,
    schedule_type public.qa_schedule_type DEFAULT 'annual'::public.qa_schedule_type NOT NULL,
    description text,
    status public.qa_schedule_status DEFAULT 'planned'::public.qa_schedule_status NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_capa; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_capa (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    nc_id uuid NOT NULL,
    action_type public.capa_action_type NOT NULL,
    description text NOT NULL,
    assignee_id uuid,
    due_date date,
    completed_at timestamp with time zone,
    status public.capa_status DEFAULT 'open'::public.capa_status NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_inspection_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_inspection_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    inspection_id uuid NOT NULL,
    item_order integer NOT NULL,
    description text NOT NULL,
    result public.qa_item_result DEFAULT 'not_applicable'::public.qa_item_result NOT NULL,
    remarks text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_inspections; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_inspections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    inspection_number character varying(50) NOT NULL,
    title character varying(255) NOT NULL,
    inspection_type public.qa_inspection_type NOT NULL,
    inspection_date date NOT NULL,
    inspector_id uuid NOT NULL,
    related_entity_type character varying(50),
    related_entity_id uuid,
    status public.qa_inspection_status DEFAULT 'draft'::public.qa_inspection_status NOT NULL,
    findings text,
    conclusion text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_non_conformances; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_non_conformances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    nc_number character varying(50) NOT NULL,
    title character varying(255) NOT NULL,
    description text NOT NULL,
    severity public.nc_severity NOT NULL,
    source public.nc_source NOT NULL,
    related_inspection_id uuid,
    assignee_id uuid,
    due_date date,
    status public.nc_status DEFAULT 'open'::public.nc_status NOT NULL,
    root_cause text,
    closure_notes text,
    closed_at timestamp with time zone,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_schedule_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_schedule_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    schedule_id uuid NOT NULL,
    inspection_type public.qa_inspection_type NOT NULL,
    title character varying(255) NOT NULL,
    planned_date date NOT NULL,
    actual_date date,
    responsible_person_id uuid,
    related_inspection_id uuid,
    status public.qa_schedule_item_status DEFAULT 'planned'::public.qa_schedule_item_status NOT NULL,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_sop_acknowledgments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_sop_acknowledgments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    sop_id uuid NOT NULL,
    user_id uuid NOT NULL,
    acknowledged_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: qa_sop_documents; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.qa_sop_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_number character varying(50) NOT NULL,
    title character varying(255) NOT NULL,
    version character varying(20) NOT NULL,
    category character varying(100),
    file_path character varying(500),
    effective_date date,
    review_date date,
    status public.sop_status DEFAULT 'draft'::public.sop_status NOT NULL,
    description text,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    reviewed_by uuid,
    reviewed_at timestamp with time zone,
    approved_by uuid,
    approved_at timestamp with time zone,
    review_due_date date,
    revision_history jsonb
);


--
-- Name: record_annotations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.record_annotations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    record_type character varying(50) NOT NULL,
    record_id integer NOT NULL,
    annotation_type character varying(20) NOT NULL,
    content text NOT NULL,
    created_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    signature_id uuid
);


--
-- Name: record_versions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.record_versions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    record_type public.version_record_type NOT NULL,
    record_id uuid NOT NULL,
    version_no integer NOT NULL,
    snapshot jsonb NOT NULL,
    diff_summary text,
    changed_by uuid,
    changed_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: reference_standards; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.reference_standards (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(200) NOT NULL,
    serial_number character varying(100),
    standard_type character varying(50) DEFAULT 'working'::character varying NOT NULL,
    traceable_to character varying(500),
    national_standard_number character varying(200),
    calibration_lab character varying(200),
    calibration_lab_accreditation character varying(100),
    last_calibrated_at date,
    next_due_at date,
    certificate_number character varying(100),
    measurement_uncertainty character varying(100),
    status character varying(20) DEFAULT 'active'::character varying NOT NULL,
    notes text,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: refresh_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.refresh_tokens (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    token_hash character varying(255) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    revoked_at timestamp with time zone,
    family_id uuid NOT NULL,
    revoked_reason text,
    last_used_at timestamp with time zone DEFAULT now() NOT NULL,
    rotated_at timestamp with time zone,
    last_ip text,
    last_user_agent text
);


--
-- Name: COLUMN refresh_tokens.rotated_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.refresh_tokens.rotated_at IS 'R46-1: normal_rotation 撤銷時的時間戳。reuse detection 時用於 race window 判定（≤5s 視為併發 race，不觸發 family revoke）。';


--
-- Name: COLUMN refresh_tokens.last_ip; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.refresh_tokens.last_ip IS 'R46-2: rotation 當下 client IP。reuse 真案時與當前 request IP 比對，相同則 severity 降為 warning。';


--
-- Name: COLUMN refresh_tokens.last_user_agent; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.refresh_tokens.last_user_agent IS 'R46-2: rotation 當下 client User-Agent。reuse 真案時與當前 request UA 比對，相同則 severity 降為 warning。';


--
-- Name: report_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.report_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    scheduled_report_id uuid,
    report_type public.report_type NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path character varying(500) NOT NULL,
    file_size integer,
    parameters jsonb,
    generated_at timestamp with time zone DEFAULT now() NOT NULL,
    generated_by uuid
);


--
-- Name: review_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.review_assignments (
    id uuid NOT NULL,
    protocol_id uuid NOT NULL,
    reviewer_id uuid NOT NULL,
    assigned_by uuid NOT NULL,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    is_primary_reviewer boolean DEFAULT false NOT NULL,
    review_stage character varying(20) DEFAULT 'UNDER_REVIEW'::character varying,
    decision character varying(20),
    decided_at timestamp with time zone
);


--
-- Name: review_comments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.review_comments (
    id uuid NOT NULL,
    protocol_version_id uuid,
    protocol_id uuid,
    reviewer_id uuid,
    content text NOT NULL,
    is_resolved boolean DEFAULT false NOT NULL,
    resolved_by uuid,
    resolved_at timestamp with time zone,
    review_stage character varying(20),
    parent_comment_id uuid,
    replied_by uuid,
    draft_content text,
    drafted_by uuid,
    draft_updated_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    reviewer_name text,
    section_no character varying(50),
    CONSTRAINT chk_protocol_reference CHECK (((protocol_version_id IS NOT NULL) OR (protocol_id IS NOT NULL))),
    CONSTRAINT chk_review_stage CHECK (((review_stage IS NULL) OR ((review_stage)::text = ANY ((ARRAY['PRE_REVIEW'::character varying, 'PRE_REVIEW_REVISION_REQUIRED'::character varying, 'VET_REVIEW'::character varying, 'VET_REVISION_REQUIRED'::character varying, 'UNDER_REVIEW'::character varying, 'FINAL_REVIEW'::character varying, 'FINAL'::character varying])::text[])))),
    CONSTRAINT review_comments_reviewer_identity_chk CHECK (((reviewer_id IS NOT NULL) OR (NULLIF(btrim(reviewer_name), ''::text) IS NOT NULL)))
);


--
-- Name: COLUMN review_comments.section_no; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.review_comments.section_no IS '審查意見對應的計畫書項次（如 4.1.2），補登審查文件填寫；nullable';


--
-- Name: review_round_history; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.review_round_history (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    review_stage character varying(30) NOT NULL,
    round_number integer DEFAULT 1 NOT NULL,
    action character varying(30) NOT NULL,
    actor_id uuid NOT NULL,
    remark text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: risk_register; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.risk_register (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    risk_number character varying(50) NOT NULL,
    title character varying(300) NOT NULL,
    description text,
    category character varying(50),
    source character varying(50),
    severity integer NOT NULL,
    likelihood integer NOT NULL,
    detectability integer,
    risk_score integer GENERATED ALWAYS AS ((severity * likelihood)) STORED,
    status character varying(30) DEFAULT 'identified'::character varying NOT NULL,
    mitigation_plan text,
    residual_risk_score integer,
    owner_id uuid,
    review_date date,
    related_nc_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT risk_register_detectability_check CHECK (((detectability >= 1) AND (detectability <= 5))),
    CONSTRAINT risk_register_likelihood_check CHECK (((likelihood >= 1) AND (likelihood <= 5))),
    CONSTRAINT risk_register_severity_check CHECK (((severity >= 1) AND (severity <= 5)))
);


--
-- Name: role_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.role_permissions (
    role_id uuid NOT NULL,
    permission_id uuid NOT NULL
);


--
-- Name: role_training_requirements; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.role_training_requirements (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    role_code character varying(50) NOT NULL,
    training_topic character varying(200) NOT NULL,
    is_mandatory boolean DEFAULT true NOT NULL,
    recurrence_months integer,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone
);


--
-- Name: roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.roles (
    id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    is_internal boolean DEFAULT true NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: scheduled_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.scheduled_reports (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    report_type public.report_type NOT NULL,
    schedule_type public.schedule_type NOT NULL,
    day_of_week integer,
    day_of_month integer,
    hour_of_day integer DEFAULT 6 NOT NULL,
    parameters jsonb,
    recipients uuid[] NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    last_run_at timestamp with time zone,
    next_run_at timestamp with time zone,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: security_alert_config; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.security_alert_config (
    key character varying(100) NOT NULL,
    value text NOT NULL,
    description text,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: security_alerts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.security_alerts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    alert_type character varying(50) NOT NULL,
    severity character varying(20) DEFAULT 'warning'::character varying NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    user_id uuid,
    activity_log_id uuid,
    login_event_id uuid,
    context_data jsonb,
    status character varying(20) DEFAULT 'open'::character varying,
    resolved_by uuid,
    resolved_at timestamp with time zone,
    resolution_notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    last_notified_at timestamp with time zone,
    CONSTRAINT chk_alert_status CHECK (((status)::text = ANY ((ARRAY['open'::character varying, 'acknowledged'::character varying, 'investigating'::character varying, 'resolved'::character varying, 'false_positive'::character varying])::text[]))),
    CONSTRAINT chk_severity CHECK (((severity)::text = ANY ((ARRAY['info'::character varying, 'warning'::character varying, 'critical'::character varying])::text[])))
);


--
-- Name: security_notification_channels; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.security_notification_channels (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    channel character varying(20) NOT NULL,
    is_enabled boolean DEFAULT false NOT NULL,
    config_json jsonb DEFAULT '{}'::jsonb NOT NULL,
    min_severity character varying(20) DEFAULT 'warning'::character varying NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_channel CHECK (((channel)::text = ANY ((ARRAY['email'::character varying, 'line'::character varying, 'webhook'::character varying])::text[]))),
    CONSTRAINT chk_min_severity CHECK (((min_severity)::text = ANY ((ARRAY['info'::character varying, 'warning'::character varying, 'critical'::character varying])::text[])))
);


--
-- Name: signature_bridge_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.signature_bridge_sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    mobile_token_hash character varying(255) NOT NULL,
    purpose character varying(50) NOT NULL,
    payload text,
    status character varying(20) DEFAULT 'PENDING'::character varying NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    submitted_at timestamp with time zone,
    consumed_at timestamp with time zone,
    CONSTRAINT signature_bridge_sessions_status_check CHECK (((status)::text = ANY ((ARRAY['PENDING'::character varying, 'COMPLETED'::character varying, 'CONSUMED'::character varying, 'EXPIRED'::character varying])::text[])))
);


--
-- Name: TABLE signature_bridge_sessions; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.signature_bridge_sessions IS 'R30-27c：桌機 ↔ 手機簽名 bridge session。短命（5min TTL）+ 單次使用，避免 QR 截圖被回放。';


--
-- Name: COLUMN signature_bridge_sessions.mobile_token_hash; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.signature_bridge_sessions.mobile_token_hash IS 'bcrypt hash of mobile_token（plaintext 只在 start 端點當下回給桌機，桌機編入 QR 給手機）。';


--
-- Name: COLUMN signature_bridge_sessions.payload; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.signature_bridge_sessions.payload IS 'AEAD 加密信封（XChaCha20-Poly1305，AAD=session_id‖user_id）的 mutation_signature payload。R66-C6 前為 JSONB 明文。';


--
-- Name: sku_categories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sku_categories (
    code character(3) NOT NULL,
    name character varying(50) NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: sku_sequences; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sku_sequences (
    category_code character(3) NOT NULL,
    subcategory_code character(3) NOT NULL,
    last_sequence integer DEFAULT 0 NOT NULL
);


--
-- Name: sku_subcategories; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sku_subcategories (
    id integer NOT NULL,
    category_code character(3) NOT NULL,
    code character(3) NOT NULL,
    name character varying(50) NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: sku_subcategories_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.sku_subcategories_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: sku_subcategories_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.sku_subcategories_id_seq OWNED BY public.sku_subcategories.id;


--
-- Name: slow_queries; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.slow_queries AS
 SELECT queryid,
    "left"(query, 200) AS query_preview,
    calls,
    mean_exec_time AS avg_ms,
    total_exec_time AS total_ms,
    rows
   FROM public.pg_stat_statements
  WHERE (mean_exec_time > (100)::double precision)
  ORDER BY mean_exec_time DESC
 LIMIT 50;


--
-- Name: species; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.species (
    id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(100) NOT NULL,
    name_en character varying(100),
    icon character varying(100),
    is_active boolean DEFAULT true NOT NULL,
    config jsonb,
    sort_order integer DEFAULT 0 NOT NULL,
    parent_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: stock_ledger; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.stock_ledger (
    id uuid NOT NULL,
    warehouse_id uuid NOT NULL,
    product_id uuid NOT NULL,
    trx_date timestamp with time zone NOT NULL,
    doc_type public.doc_type NOT NULL,
    doc_id uuid NOT NULL,
    doc_no character varying(50) NOT NULL,
    line_id uuid,
    direction public.stock_direction NOT NULL,
    qty_base numeric(18,4) NOT NULL,
    unit_cost numeric(18,4),
    batch_no character varying(50),
    expiry_date date,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    storage_location_id uuid
);


--
-- Name: storage_location_inventory; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.storage_location_inventory (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    storage_location_id uuid NOT NULL,
    product_id uuid NOT NULL,
    on_hand_qty numeric(18,4) DEFAULT 0 NOT NULL,
    batch_no character varying(50),
    expiry_date date,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_storage_location_inventory_on_hand_nonneg CHECK ((on_hand_qty >= (0)::numeric))
);


--
-- Name: storage_locations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.storage_locations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    warehouse_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200),
    location_type character varying(50) DEFAULT 'shelf'::character varying NOT NULL,
    row_index integer DEFAULT 0 NOT NULL,
    col_index integer DEFAULT 0 NOT NULL,
    width integer DEFAULT 2 NOT NULL,
    height integer DEFAULT 2 NOT NULL,
    capacity integer,
    current_count integer DEFAULT 0,
    color character varying(20),
    zone character varying(50),
    is_active boolean DEFAULT true NOT NULL,
    config jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: study_final_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.study_final_reports (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    report_number character varying(50) NOT NULL,
    protocol_id uuid NOT NULL,
    title character varying(500) NOT NULL,
    status character varying(30) DEFAULT 'draft'::character varying NOT NULL,
    summary text,
    methods text,
    results text,
    conclusions text,
    deviations text,
    signed_by uuid,
    signed_at timestamp with time zone,
    signature_id uuid,
    qau_statement text,
    qau_signed_by uuid,
    qau_signed_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: surgery_vet_reads; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.surgery_vet_reads (
    surgery_id uuid NOT NULL,
    vet_user_id uuid NOT NULL,
    read_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: system_settings; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.system_settings (
    key character varying(100) NOT NULL,
    value jsonb NOT NULL,
    description text,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_by uuid
);


--
-- Name: training_records; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.training_records (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    course_name character varying(200) NOT NULL,
    completed_at date NOT NULL,
    expires_at date,
    notes text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: transfer_vet_evaluations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.transfer_vet_evaluations (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    transfer_id uuid NOT NULL,
    vet_id uuid NOT NULL,
    health_status text NOT NULL,
    is_fit_for_transfer boolean NOT NULL,
    conditions text,
    evaluated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: treatment_drug_options; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.treatment_drug_options (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(200) NOT NULL,
    display_name character varying(200),
    default_dosage_unit character varying(20),
    available_units text[],
    default_dosage_value character varying(50),
    erp_product_id uuid,
    category character varying(50),
    sort_order integer DEFAULT 0 NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_default_dosage_unit_in_available_units CHECK (((default_dosage_unit IS NULL) OR ((available_units IS NOT NULL) AND ((default_dosage_unit)::text = ANY (available_units)))))
);


--
-- Name: user_activity_aggregates; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_aggregates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid,
    aggregate_date date NOT NULL,
    login_count integer DEFAULT 0,
    failed_login_count integer DEFAULT 0,
    session_count integer DEFAULT 0,
    total_session_minutes integer DEFAULT 0,
    page_view_count integer DEFAULT 0,
    action_count integer DEFAULT 0,
    actions_by_category jsonb DEFAULT '{}'::jsonb,
    pages_visited jsonb DEFAULT '[]'::jsonb,
    entities_modified jsonb DEFAULT '[]'::jsonb,
    unique_ip_count integer DEFAULT 0,
    unusual_activity_count integer DEFAULT 0,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_activity_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
)
PARTITION BY RANGE (partition_date);


--
-- Name: COLUMN user_activity_logs.impersonated_by_user_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_activity_logs.impersonated_by_user_id IS 'When non-NULL: the real admin user_id who impersonated actor_user_id (SEC-11). NULL for direct operations.';


--
-- Name: COLUMN user_activity_logs.hmac_version; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_activity_logs.hmac_version IS 'HMAC chain hash 編碼版本: 1 = legacy string-concat (pre-R26-6, 已棄寫)；2 = length-prefix canonical (R26-6 SDD)；3 = v3 含 entity_type / entity_id / extra_input (R30-9a SIGNATURE_*)';


--
-- Name: COLUMN user_activity_logs.extra_input; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_activity_logs.extra_input IS 'R30-9a: HMAC chain v3 額外輸入。SIGNATURE_CREATE: <sig_id>:<content_hash>; SIGNATURE_INVALIDATED: <sig_id>:<invalidated_reason>（原始字串，非 hash）. v2 row 為 NULL（不參與 hash 計算）';


--
-- Name: user_activity_logs_2026_q1; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2026_q1 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2026_q2; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2026_q2 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2026_q3; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2026_q3 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2026_q4; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2026_q4 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2027_q1; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2027_q1 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2027_q2; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2027_q2 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2027_q3; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2027_q3 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2027_q4; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2027_q4 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2028_q1; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2028_q1 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2028_q2; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2028_q2 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2028_q3; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2028_q3 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_activity_logs_2028_q4; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_activity_logs_2028_q4 (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    actor_user_id uuid,
    actor_email character varying(255),
    actor_display_name character varying(100),
    actor_roles jsonb,
    session_id uuid,
    session_started_at timestamp with time zone,
    event_category character varying(50) NOT NULL,
    event_type character varying(100) NOT NULL,
    event_severity character varying(20) DEFAULT 'info'::character varying,
    entity_type character varying(50),
    entity_id uuid,
    entity_display_name character varying(255),
    before_data jsonb,
    after_data jsonb,
    changed_fields text[],
    ip_address inet,
    user_agent text,
    request_path character varying(500),
    request_method character varying(10),
    response_status integer,
    geo_country character varying(100),
    geo_city character varying(100),
    is_suspicious boolean DEFAULT false,
    suspicious_reason text,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    partition_date date DEFAULT CURRENT_DATE NOT NULL,
    integrity_hash character varying(128),
    previous_hash character varying(128),
    impersonated_by_user_id uuid,
    hmac_version smallint,
    extra_input text
);


--
-- Name: user_aup_profiles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_aup_profiles (
    user_id uuid NOT NULL,
    training_records jsonb,
    research_experience text,
    animal_experience text,
    certifications jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_mcp_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_mcp_keys (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    key_hash text NOT NULL,
    key_prefix character varying(20) NOT NULL,
    name text NOT NULL,
    last_used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    revoked_at timestamp with time zone,
    scopes text[] DEFAULT '{read,write}'::text[] NOT NULL,
    expires_at timestamp with time zone
);


--
-- Name: TABLE user_mcp_keys; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON TABLE public.user_mcp_keys IS '個人 MCP API Key，用於 claude.ai Remote MCP 連線';


--
-- Name: COLUMN user_mcp_keys.key_hash; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_mcp_keys.key_hash IS 'SHA-256 hash，明文金鑰只在建立時回傳一次';


--
-- Name: COLUMN user_mcp_keys.key_prefix; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_mcp_keys.key_prefix IS '顯示於 UI 的前綴，格式 mcp_xxxxxxxx';


--
-- Name: COLUMN user_mcp_keys.scopes; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_mcp_keys.scopes IS 'MCP key 權限範圍：read=唯讀工具、write=可呼叫 mutation 工具（CSO-r3 #5）。新 key 預設僅 {read}';


--
-- Name: COLUMN user_mcp_keys.expires_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.user_mcp_keys.expires_at IS 'key 過期時間；NULL=不過期（grandfather 既有 key）。新 key 預設 +1 年（CSO-r3 #5）';


--
-- Name: user_preferences; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_preferences (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    preference_key character varying(100) NOT NULL,
    preference_value jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: user_protocols; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_protocols (
    user_id uuid NOT NULL,
    protocol_id uuid NOT NULL,
    role_in_protocol public.protocol_role NOT NULL,
    granted_at timestamp with time zone DEFAULT now() NOT NULL,
    granted_by uuid
);


--
-- Name: user_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_roles (
    user_id uuid NOT NULL,
    role_id uuid NOT NULL,
    assigned_at timestamp with time zone DEFAULT now(),
    assigned_by uuid
);


--
-- Name: user_sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    started_at timestamp with time zone DEFAULT now() NOT NULL,
    ended_at timestamp with time zone,
    last_activity_at timestamp with time zone DEFAULT now() NOT NULL,
    refresh_token_id uuid,
    ip_address inet,
    user_agent text,
    device_fingerprint character varying(255),
    page_view_count integer DEFAULT 0,
    action_count integer DEFAULT 0,
    is_active boolean DEFAULT true,
    ended_reason character varying(50)
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id uuid NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    display_name character varying(100) NOT NULL,
    phone character varying(20),
    phone_ext character varying(20),
    organization character varying(200),
    is_internal boolean DEFAULT true NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    must_change_password boolean DEFAULT true NOT NULL,
    last_login_at timestamp with time zone,
    login_attempts integer DEFAULT 0 NOT NULL,
    locked_until timestamp with time zone,
    theme_preference character varying(20) DEFAULT 'light'::character varying NOT NULL,
    language_preference character varying(10) DEFAULT 'zh-TW'::character varying NOT NULL,
    entry_date date,
    "position" character varying(100),
    aup_roles character varying(255)[] DEFAULT '{}'::character varying[],
    years_experience integer DEFAULT 0 NOT NULL,
    trainings jsonb DEFAULT '[]'::jsonb NOT NULL,
    deleted_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    totp_enabled boolean DEFAULT false NOT NULL,
    totp_secret_encrypted text,
    totp_backup_codes text[],
    version integer DEFAULT 1 NOT NULL,
    expires_at timestamp with time zone,
    last_totp_step bigint,
    tokens_valid_after timestamp with time zone,
    work_shift character varying(20) DEFAULT 'standard'::character varying NOT NULL,
    department_id uuid,
    CONSTRAINT chk_language_preference CHECK (((language_preference)::text = ANY ((ARRAY['zh-TW'::character varying, 'en'::character varying])::text[]))),
    CONSTRAINT chk_theme_preference CHECK (((theme_preference)::text = ANY ((ARRAY['light'::character varying, 'dark'::character varying, 'system'::character varying])::text[]))),
    CONSTRAINT chk_work_shift CHECK (((work_shift)::text = ANY ((ARRAY['early'::character varying, 'standard'::character varying])::text[])))
);


--
-- Name: COLUMN users.id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.id IS 'Reserved UUID 00000000-0000-0000-0000-000000000001 = SYSTEM actor for non-user-triggered audit (scheduler/bin/migration).';


--
-- Name: COLUMN users.expires_at; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.expires_at IS '帳號到期日，NULL 表示永不過期。用於實習生等臨時帳號';


--
-- Name: COLUMN users.last_totp_step; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.last_totp_step IS 'CSO #3：最後成功驗證的 TOTP time-step（unix_time / period）。登入 2FA 拒絕 step <= 此值，防同窗重放。';


--
-- Name: COLUMN users.tokens_valid_after; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.tokens_valid_after IS 'access token 須晚於此時間簽發才有效；改密碼/重設/角色變更時設 NOW()（CSO-r3 #3/#4）。NULL=無限制';


--
-- Name: COLUMN users.work_shift; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.work_shift IS '固定班別：early=7:30–16:30，standard=8:30–17:30';


--
-- Name: COLUMN users.department_id; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON COLUMN public.users.department_id IS '所屬部門，NULL 表示未指派；請假兩關審核以此反查 L1 部門主管';


--
-- Name: warehouses; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.warehouses (
    id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200) NOT NULL,
    address text,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: v_expiry_alerts; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_expiry_alerts AS
 SELECT p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.spec,
    p.category_code,
    sl.warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    sl.batch_no,
    sl.expiry_date,
    sum(
        CASE
            WHEN (sl.direction = ANY (ARRAY['in'::public.stock_direction, 'transfer_in'::public.stock_direction, 'adjust_in'::public.stock_direction])) THEN sl.qty_base
            ELSE (- sl.qty_base)
        END) AS on_hand_qty,
    p.base_uom,
    (sl.expiry_date - CURRENT_DATE) AS days_until_expiry,
        CASE
            WHEN (sl.expiry_date < CURRENT_DATE) THEN 'expired'::text
            ELSE 'expiring_soon'::text
        END AS expiry_status,
    COALESCE(inv.on_hand_qty_base, (0)::numeric) AS total_qty
   FROM (((public.stock_ledger sl
     JOIN public.products p ON ((sl.product_id = p.id)))
     JOIN public.warehouses w ON ((sl.warehouse_id = w.id)))
     LEFT JOIN public.inventory_snapshots inv ON (((inv.product_id = p.id) AND (inv.warehouse_id = sl.warehouse_id))))
  WHERE ((p.track_expiry = true) AND (sl.expiry_date IS NOT NULL) AND (p.is_active = true) AND (sl.expiry_date >= (CURRENT_DATE - 90)))
  GROUP BY p.id, p.sku, p.name, p.spec, p.category_code, sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date, p.base_uom, inv.on_hand_qty_base
 HAVING ((sum(
        CASE
            WHEN (sl.direction = ANY (ARRAY['in'::public.stock_direction, 'transfer_in'::public.stock_direction, 'adjust_in'::public.stock_direction])) THEN sl.qty_base
            ELSE (- sl.qty_base)
        END) > (0)::numeric) AND (sl.expiry_date <= (CURRENT_DATE + 60)));


--
-- Name: v_grn_line_unshelved; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_grn_line_unshelved AS
 SELECT dl.id AS document_line_id,
    d.id AS document_id,
    d.doc_no,
    d.doc_date,
    d.warehouse_id,
    d.partner_id,
    d.created_at AS doc_created_at,
    dl.line_no,
    dl.product_id,
    dl.batch_no,
    dl.expiry_date,
    dl.qty AS line_qty,
    (dl.qty - COALESCE(sum(a.qty), (0)::numeric)) AS remaining_unshelved
   FROM ((public.documents d
     JOIN public.document_lines dl ON ((dl.document_id = d.id)))
     LEFT JOIN public.line_shelf_allocations a ON ((a.document_line_id = dl.id)))
  WHERE ((d.doc_type = 'GRN'::public.doc_type) AND (d.status = 'approved'::public.doc_status) AND (dl.storage_location_id IS NULL))
  GROUP BY dl.id, d.id, d.doc_no, d.doc_date, d.warehouse_id, d.partner_id, d.created_at, dl.line_no, dl.product_id, dl.batch_no, dl.expiry_date, dl.qty
 HAVING ((dl.qty - COALESCE(sum(a.qty), (0)::numeric)) > (0)::numeric);


--
-- Name: v_inventory_summary; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_inventory_summary AS
 SELECT w.id AS warehouse_id,
    w.code AS warehouse_code,
    w.name AS warehouse_name,
    p.id AS product_id,
    p.sku,
    p.name AS product_name,
    p.base_uom,
    p.category_code,
    inv.on_hand_qty_base,
    inv.avg_cost,
    p.safety_stock,
    p.reorder_point
   FROM ((public.inventory_snapshots inv
     JOIN public.warehouses w ON ((inv.warehouse_id = w.id)))
     JOIN public.products p ON ((inv.product_id = p.id)))
  WHERE (p.is_active AND w.is_active)
  ORDER BY w.code, p.sku;


--
-- Name: v_low_stock_alerts; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_low_stock_alerts AS
 SELECT inv.warehouse_id,
    w.name AS warehouse_name,
    p.id AS product_id,
    p.sku AS product_sku,
    p.name AS product_name,
    p.base_uom,
    inv.on_hand_qty_base AS qty_on_hand,
    p.safety_stock,
    p.reorder_point,
        CASE
            WHEN (inv.on_hand_qty_base <= (0)::numeric) THEN 'out_of_stock'::text
            WHEN ((p.safety_stock IS NOT NULL) AND (inv.on_hand_qty_base < p.safety_stock)) THEN 'below_safety'::text
            WHEN ((p.reorder_point IS NOT NULL) AND (inv.on_hand_qty_base < p.reorder_point)) THEN 'below_reorder'::text
            ELSE 'below_safety'::text
        END AS stock_status
   FROM ((public.inventory_snapshots inv
     JOIN public.products p ON ((inv.product_id = p.id)))
     JOIN public.warehouses w ON ((inv.warehouse_id = w.id)))
  WHERE (p.is_active AND w.is_active AND ((inv.on_hand_qty_base <= (0)::numeric) OR ((p.safety_stock IS NOT NULL) AND (inv.on_hand_qty_base < p.safety_stock)) OR ((p.reorder_point IS NOT NULL) AND (inv.on_hand_qty_base < p.reorder_point))));


--
-- Name: v_purchase_order_receipt_status; Type: VIEW; Schema: public; Owner: -
--

CREATE VIEW public.v_purchase_order_receipt_status AS
 SELECT po.id AS po_id,
    po.doc_no AS po_no,
    po.status AS po_status,
    po.partner_id,
    po.warehouse_id,
    po.doc_date AS po_date,
    COALESCE(sum(pol.qty), (0)::numeric) AS ordered_qty,
    COALESCE(sum(grnl.received_qty), (0)::numeric) AS received_qty,
        CASE
            WHEN (COALESCE(sum(grnl.received_qty), (0)::numeric) = (0)::numeric) THEN 'pending'::text
            WHEN (COALESCE(sum(grnl.received_qty), (0)::numeric) < COALESCE(sum(pol.qty), (0)::numeric)) THEN 'partial'::text
            ELSE 'complete'::text
        END AS receipt_status
   FROM ((public.documents po
     LEFT JOIN public.document_lines pol ON ((po.id = pol.document_id)))
     LEFT JOIN ( SELECT grn.source_doc_id,
            grnl_1.product_id,
            sum(grnl_1.qty) AS received_qty
           FROM (public.documents grn
             JOIN public.document_lines grnl_1 ON ((grn.id = grnl_1.document_id)))
          WHERE ((grn.doc_type = 'GRN'::public.doc_type) AND (grn.status = 'approved'::public.doc_status))
          GROUP BY grn.source_doc_id, grnl_1.product_id) grnl ON (((po.id = grnl.source_doc_id) AND (pol.product_id = grnl.product_id))))
  WHERE (po.doc_type = 'PO'::public.doc_type)
  GROUP BY po.id, po.doc_no, po.status, po.partner_id, po.warehouse_id, po.doc_date;


--
-- Name: vet_patrol_entries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vet_patrol_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    report_id uuid NOT NULL,
    category character varying(50) NOT NULL,
    animal_id uuid,
    observation text DEFAULT ''::text NOT NULL,
    suggestion text DEFAULT ''::text NOT NULL,
    follow_up text DEFAULT ''::text NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: vet_patrol_entry_animals; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vet_patrol_entry_animals (
    entry_id uuid NOT NULL,
    animal_id uuid NOT NULL
);


--
-- Name: vet_patrol_entry_photos; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vet_patrol_entry_photos (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    entry_id uuid NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path text NOT NULL,
    file_size bigint NOT NULL,
    mime_type character varying(100) NOT NULL,
    caption text DEFAULT ''::text NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: vet_patrol_photos; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vet_patrol_photos (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    report_id uuid NOT NULL,
    file_name character varying(255) NOT NULL,
    file_path text NOT NULL,
    file_size bigint NOT NULL,
    mime_type character varying(100) NOT NULL,
    caption text DEFAULT ''::text NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: vet_patrol_reports; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vet_patrol_reports (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    patrol_date date DEFAULT CURRENT_DATE NOT NULL,
    week_start date,
    week_end date,
    status character varying(30) DEFAULT 'draft'::character varying NOT NULL,
    created_by uuid,
    updated_by uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    deleted_at timestamp with time zone,
    accompanying_personnel text,
    submitted_at timestamp with time zone,
    follow_up_user_id uuid,
    follow_up_submitted_at timestamp with time zone,
    acknowledged_at timestamp with time zone,
    acknowledged_by_id uuid,
    CONSTRAINT vet_patrol_reports_status_check CHECK (((status)::text = ANY (ARRAY[('draft'::character varying)::text, ('awaiting_acknowledgement'::character varying)::text, ('awaiting_follow_up'::character varying)::text, ('completed'::character varying)::text])))
);


--
-- Name: vet_review_assignments; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.vet_review_assignments (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    protocol_id uuid NOT NULL,
    vet_id uuid,
    assigned_by uuid,
    assigned_at timestamp with time zone DEFAULT now() NOT NULL,
    completed_at timestamp with time zone,
    decision character varying(20),
    decision_remark text,
    review_form jsonb,
    vet_name text,
    CONSTRAINT vet_review_assignments_vet_identity_chk CHECK (((vet_id IS NOT NULL) OR (NULLIF(btrim(vet_name), ''::text) IS NOT NULL)))
);


--
-- Name: zones; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.zones (
    id uuid NOT NULL,
    building_id uuid NOT NULL,
    code character varying(50) NOT NULL,
    name character varying(200),
    color character varying(20),
    is_active boolean DEFAULT true NOT NULL,
    layout_config jsonb,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: ai_query_logs_2026_08; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_query_logs ATTACH PARTITION public.ai_query_logs_2026_08 FOR VALUES FROM ('2026-08-01 00:00:00+00') TO ('2026-09-01 00:00:00+00');


--
-- Name: ai_query_logs_2026_09; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_query_logs ATTACH PARTITION public.ai_query_logs_2026_09 FOR VALUES FROM ('2026-09-01 00:00:00+00') TO ('2026-10-01 00:00:00+00');


--
-- Name: user_activity_logs_2026_q1; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2026_q1 FOR VALUES FROM ('2026-01-01') TO ('2026-04-01');


--
-- Name: user_activity_logs_2026_q2; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2026_q2 FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');


--
-- Name: user_activity_logs_2026_q3; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2026_q3 FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');


--
-- Name: user_activity_logs_2026_q4; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2026_q4 FOR VALUES FROM ('2026-10-01') TO ('2027-01-01');


--
-- Name: user_activity_logs_2027_q1; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2027_q1 FOR VALUES FROM ('2027-01-01') TO ('2027-04-01');


--
-- Name: user_activity_logs_2027_q2; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2027_q2 FOR VALUES FROM ('2027-04-01') TO ('2027-07-01');


--
-- Name: user_activity_logs_2027_q3; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2027_q3 FOR VALUES FROM ('2027-07-01') TO ('2027-10-01');


--
-- Name: user_activity_logs_2027_q4; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2027_q4 FOR VALUES FROM ('2027-10-01') TO ('2028-01-01');


--
-- Name: user_activity_logs_2028_q1; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2028_q1 FOR VALUES FROM ('2028-01-01') TO ('2028-04-01');


--
-- Name: user_activity_logs_2028_q2; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2028_q2 FOR VALUES FROM ('2028-04-01') TO ('2028-07-01');


--
-- Name: user_activity_logs_2028_q3; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2028_q3 FOR VALUES FROM ('2028-07-01') TO ('2028-10-01');


--
-- Name: user_activity_logs_2028_q4; Type: TABLE ATTACH; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs ATTACH PARTITION public.user_activity_logs_2028_q4 FOR VALUES FROM ('2028-10-01') TO ('2029-01-01');


--
-- Name: sku_subcategories id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sku_subcategories ALTER COLUMN id SET DEFAULT nextval('public.sku_subcategories_id_seq'::regclass);


--
-- Name: ai_api_keys ai_api_keys_key_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_api_keys
    ADD CONSTRAINT ai_api_keys_key_hash_key UNIQUE (key_hash);


--
-- Name: ai_api_keys ai_api_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_api_keys
    ADD CONSTRAINT ai_api_keys_pkey PRIMARY KEY (id);


--
-- Name: ai_query_logs ai_query_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_query_logs
    ADD CONSTRAINT ai_query_logs_pkey PRIMARY KEY (id, created_at);


--
-- Name: ai_query_logs_2026_08 ai_query_logs_2026_08_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_query_logs_2026_08
    ADD CONSTRAINT ai_query_logs_2026_08_pkey PRIMARY KEY (id, created_at);


--
-- Name: ai_query_logs_2026_09 ai_query_logs_2026_09_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_query_logs_2026_09
    ADD CONSTRAINT ai_query_logs_2026_09_pkey PRIMARY KEY (id, created_at);


--
-- Name: amendment_review_assignments amendment_review_assignments_amendment_id_reviewer_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_review_assignments
    ADD CONSTRAINT amendment_review_assignments_amendment_id_reviewer_id_key UNIQUE (amendment_id, reviewer_id);


--
-- Name: amendment_review_assignments amendment_review_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_review_assignments
    ADD CONSTRAINT amendment_review_assignments_pkey PRIMARY KEY (id);


--
-- Name: amendment_status_history amendment_status_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_status_history
    ADD CONSTRAINT amendment_status_history_pkey PRIMARY KEY (id);


--
-- Name: amendment_versions amendment_versions_amendment_id_version_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_versions
    ADD CONSTRAINT amendment_versions_amendment_id_version_no_key UNIQUE (amendment_id, version_no);


--
-- Name: amendment_versions amendment_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_versions
    ADD CONSTRAINT amendment_versions_pkey PRIMARY KEY (id);


--
-- Name: amendments amendments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_pkey PRIMARY KEY (id);


--
-- Name: amendments amendments_protocol_id_amendment_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_protocol_id_amendment_no_key UNIQUE (protocol_id, amendment_no);


--
-- Name: animal_blood_test_items animal_blood_test_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_test_items
    ADD CONSTRAINT animal_blood_test_items_pkey PRIMARY KEY (id);


--
-- Name: animal_blood_tests animal_blood_tests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_tests
    ADD CONSTRAINT animal_blood_tests_pkey PRIMARY KEY (id);


--
-- Name: animal_field_correction_requests animal_field_correction_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_field_correction_requests
    ADD CONSTRAINT animal_field_correction_requests_pkey PRIMARY KEY (id);


--
-- Name: animal_import_batches animal_import_batches_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_import_batches
    ADD CONSTRAINT animal_import_batches_pkey PRIMARY KEY (id);


--
-- Name: animal_observations animal_observations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_observations
    ADD CONSTRAINT animal_observations_pkey PRIMARY KEY (id);


--
-- Name: animal_pathology_reports animal_pathology_reports_animal_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_pathology_reports
    ADD CONSTRAINT animal_pathology_reports_animal_id_key UNIQUE (animal_id);


--
-- Name: animal_pathology_reports animal_pathology_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_pathology_reports
    ADD CONSTRAINT animal_pathology_reports_pkey PRIMARY KEY (id);


--
-- Name: animal_record_attachments animal_record_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_record_attachments
    ADD CONSTRAINT animal_record_attachments_pkey PRIMARY KEY (id);


--
-- Name: animal_sacrifices animal_sacrifices_animal_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sacrifices
    ADD CONSTRAINT animal_sacrifices_animal_id_key UNIQUE (animal_id);


--
-- Name: animal_sacrifices animal_sacrifices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sacrifices
    ADD CONSTRAINT animal_sacrifices_pkey PRIMARY KEY (id);


--
-- Name: animal_sources animal_sources_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sources
    ADD CONSTRAINT animal_sources_code_key UNIQUE (code);


--
-- Name: animal_sources animal_sources_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sources
    ADD CONSTRAINT animal_sources_pkey PRIMARY KEY (id);


--
-- Name: animal_sudden_deaths animal_sudden_deaths_animal_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sudden_deaths
    ADD CONSTRAINT animal_sudden_deaths_animal_id_key UNIQUE (animal_id);


--
-- Name: animal_sudden_deaths animal_sudden_deaths_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sudden_deaths
    ADD CONSTRAINT animal_sudden_deaths_pkey PRIMARY KEY (id);


--
-- Name: animal_surgeries animal_surgeries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_surgeries
    ADD CONSTRAINT animal_surgeries_pkey PRIMARY KEY (id);


--
-- Name: animal_transfers animal_transfers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_transfers
    ADD CONSTRAINT animal_transfers_pkey PRIMARY KEY (id);


--
-- Name: animal_vaccinations animal_vaccinations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vaccinations
    ADD CONSTRAINT animal_vaccinations_pkey PRIMARY KEY (id);


--
-- Name: animal_vet_advice_records animal_vet_advice_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advice_records
    ADD CONSTRAINT animal_vet_advice_records_pkey PRIMARY KEY (id);


--
-- Name: animal_vet_advices animal_vet_advices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advices
    ADD CONSTRAINT animal_vet_advices_pkey PRIMARY KEY (id);


--
-- Name: animal_weights animal_weights_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_weights
    ADD CONSTRAINT animal_weights_pkey PRIMARY KEY (id);


--
-- Name: animals animals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_pkey PRIMARY KEY (id);


--
-- Name: annual_leave_entitlements annual_leave_entitlements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.annual_leave_entitlements
    ADD CONSTRAINT annual_leave_entitlements_pkey PRIMARY KEY (id);


--
-- Name: annual_leave_entitlements annual_leave_entitlements_user_id_entitlement_year_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.annual_leave_entitlements
    ADD CONSTRAINT annual_leave_entitlements_user_id_entitlement_year_key UNIQUE (user_id, entitlement_year);


--
-- Name: ap_payments ap_payments_payment_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ap_payments
    ADD CONSTRAINT ap_payments_payment_no_key UNIQUE (payment_no);


--
-- Name: ap_payments ap_payments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ap_payments
    ADD CONSTRAINT ap_payments_pkey PRIMARY KEY (id);


--
-- Name: application_notices application_notices_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.application_notices
    ADD CONSTRAINT application_notices_pkey PRIMARY KEY (id);


--
-- Name: application_notices application_notices_version_label_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.application_notices
    ADD CONSTRAINT application_notices_version_label_key UNIQUE (version_label);


--
-- Name: ar_receipts ar_receipts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ar_receipts
    ADD CONSTRAINT ar_receipts_pkey PRIMARY KEY (id);


--
-- Name: ar_receipts ar_receipts_receipt_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ar_receipts
    ADD CONSTRAINT ar_receipts_receipt_no_key UNIQUE (receipt_no);


--
-- Name: attachments attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attachments
    ADD CONSTRAINT attachments_pkey PRIMARY KEY (id);


--
-- Name: attendance_records attendance_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_records
    ADD CONSTRAINT attendance_records_pkey PRIMARY KEY (id);


--
-- Name: attendance_records attendance_records_user_id_work_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_records
    ADD CONSTRAINT attendance_records_user_id_work_date_key UNIQUE (user_id, work_date);


--
-- Name: audit_chain_known_breaks audit_chain_known_breaks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_chain_known_breaks
    ADD CONSTRAINT audit_chain_known_breaks_pkey PRIMARY KEY (log_id);


--
-- Name: audit_logs audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (id);


--
-- Name: blood_test_panel_items blood_test_panel_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_panel_items
    ADD CONSTRAINT blood_test_panel_items_pkey PRIMARY KEY (panel_id, template_id);


--
-- Name: blood_test_panels blood_test_panels_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_panels
    ADD CONSTRAINT blood_test_panels_key_key UNIQUE (key);


--
-- Name: blood_test_panels blood_test_panels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_panels
    ADD CONSTRAINT blood_test_panels_pkey PRIMARY KEY (id);


--
-- Name: blood_test_presets blood_test_presets_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_presets
    ADD CONSTRAINT blood_test_presets_pkey PRIMARY KEY (id);


--
-- Name: blood_test_templates blood_test_templates_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_templates
    ADD CONSTRAINT blood_test_templates_code_key UNIQUE (code);


--
-- Name: blood_test_templates blood_test_templates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_templates
    ADD CONSTRAINT blood_test_templates_pkey PRIMARY KEY (id);


--
-- Name: buildings buildings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.buildings
    ADD CONSTRAINT buildings_pkey PRIMARY KEY (id);


--
-- Name: calendar_event_sync calendar_event_sync_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_event_sync
    ADD CONSTRAINT calendar_event_sync_pkey PRIMARY KEY (id);


--
-- Name: calendar_sync_conflicts calendar_sync_conflicts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_conflicts
    ADD CONSTRAINT calendar_sync_conflicts_pkey PRIMARY KEY (id);


--
-- Name: calendar_sync_history calendar_sync_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_history
    ADD CONSTRAINT calendar_sync_history_pkey PRIMARY KEY (id);


--
-- Name: care_medication_records care_medication_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.care_medication_records
    ADD CONSTRAINT care_medication_records_pkey PRIMARY KEY (id);


--
-- Name: change_reasons change_reasons_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_reasons
    ADD CONSTRAINT change_reasons_pkey PRIMARY KEY (id);


--
-- Name: change_requests change_requests_change_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_requests
    ADD CONSTRAINT change_requests_change_number_key UNIQUE (change_number);


--
-- Name: change_requests change_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_requests
    ADD CONSTRAINT change_requests_pkey PRIMARY KEY (id);


--
-- Name: chart_of_accounts chart_of_accounts_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chart_of_accounts
    ADD CONSTRAINT chart_of_accounts_code_key UNIQUE (code);


--
-- Name: chart_of_accounts chart_of_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chart_of_accounts
    ADD CONSTRAINT chart_of_accounts_pkey PRIMARY KEY (id);


--
-- Name: comp_time_balances comp_time_balances_overtime_record_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comp_time_balances
    ADD CONSTRAINT comp_time_balances_overtime_record_id_key UNIQUE (overtime_record_id);


--
-- Name: comp_time_balances comp_time_balances_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comp_time_balances
    ADD CONSTRAINT comp_time_balances_pkey PRIMARY KEY (id);


--
-- Name: competency_assessments competency_assessments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.competency_assessments
    ADD CONSTRAINT competency_assessments_pkey PRIMARY KEY (id);


--
-- Name: controlled_documents controlled_documents_doc_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.controlled_documents
    ADD CONSTRAINT controlled_documents_doc_number_key UNIQUE (doc_number);


--
-- Name: controlled_documents controlled_documents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.controlled_documents
    ADD CONSTRAINT controlled_documents_pkey PRIMARY KEY (id);


--
-- Name: data_retention_policies data_retention_policies_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_retention_policies
    ADD CONSTRAINT data_retention_policies_pkey PRIMARY KEY (id);


--
-- Name: data_retention_policies data_retention_policies_table_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.data_retention_policies
    ADD CONSTRAINT data_retention_policies_table_name_key UNIQUE (table_name);


--
-- Name: departments departments_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_code_key UNIQUE (code);


--
-- Name: departments departments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_pkey PRIMARY KEY (id);


--
-- Name: document_acknowledgments document_acknowledgments_document_id_user_id_version_acknow_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_acknowledgments
    ADD CONSTRAINT document_acknowledgments_document_id_user_id_version_acknow_key UNIQUE (document_id, user_id, version_acknowledged);


--
-- Name: document_acknowledgments document_acknowledgments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_acknowledgments
    ADD CONSTRAINT document_acknowledgments_pkey PRIMARY KEY (id);


--
-- Name: document_lines document_lines_document_id_line_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_document_id_line_no_key UNIQUE (document_id, line_no);


--
-- Name: document_lines document_lines_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_pkey PRIMARY KEY (id);


--
-- Name: document_revisions document_revisions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_revisions
    ADD CONSTRAINT document_revisions_pkey PRIMARY KEY (id);


--
-- Name: documents documents_doc_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_doc_no_key UNIQUE (doc_no);


--
-- Name: documents documents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_pkey PRIMARY KEY (id);


--
-- Name: electronic_signatures electronic_signatures_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.electronic_signatures
    ADD CONSTRAINT electronic_signatures_pkey PRIMARY KEY (id);


--
-- Name: environment_monitoring_points environment_monitoring_points_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.environment_monitoring_points
    ADD CONSTRAINT environment_monitoring_points_pkey PRIMARY KEY (id);


--
-- Name: environment_readings environment_readings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.environment_readings
    ADD CONSTRAINT environment_readings_pkey PRIMARY KEY (id);


--
-- Name: equipment_annual_plans equipment_annual_plans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_annual_plans
    ADD CONSTRAINT equipment_annual_plans_pkey PRIMARY KEY (id);


--
-- Name: equipment_annual_plans equipment_annual_plans_year_equipment_id_calibration_type_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_annual_plans
    ADD CONSTRAINT equipment_annual_plans_year_equipment_id_calibration_type_key UNIQUE (year, equipment_id, calibration_type);


--
-- Name: equipment_calibrations equipment_calibrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_calibrations
    ADD CONSTRAINT equipment_calibrations_pkey PRIMARY KEY (id);


--
-- Name: equipment_disposals equipment_disposals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_disposals
    ADD CONSTRAINT equipment_disposals_pkey PRIMARY KEY (id);


--
-- Name: equipment_idle_requests equipment_idle_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_idle_requests
    ADD CONSTRAINT equipment_idle_requests_pkey PRIMARY KEY (id);


--
-- Name: equipment_maintenance_records equipment_maintenance_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_maintenance_records
    ADD CONSTRAINT equipment_maintenance_records_pkey PRIMARY KEY (id);


--
-- Name: equipment equipment_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment
    ADD CONSTRAINT equipment_pkey PRIMARY KEY (id);


--
-- Name: equipment_status_logs equipment_status_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_status_logs
    ADD CONSTRAINT equipment_status_logs_pkey PRIMARY KEY (id);


--
-- Name: equipment_suppliers equipment_suppliers_equipment_id_partner_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_suppliers
    ADD CONSTRAINT equipment_suppliers_equipment_id_partner_id_key UNIQUE (equipment_id, partner_id);


--
-- Name: equipment_suppliers equipment_suppliers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_suppliers
    ADD CONSTRAINT equipment_suppliers_pkey PRIMARY KEY (id);


--
-- Name: euthanasia_appeals euthanasia_appeals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_appeals
    ADD CONSTRAINT euthanasia_appeals_pkey PRIMARY KEY (id);


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_pkey PRIMARY KEY (id);


--
-- Name: euthanasia_orders euthanasia_orders_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_orders
    ADD CONSTRAINT euthanasia_orders_pkey PRIMARY KEY (id);


--
-- Name: event_outbox event_outbox_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.event_outbox
    ADD CONSTRAINT event_outbox_pkey PRIMARY KEY (id);


--
-- Name: expiry_monthly_snapshots expiry_monthly_snapshots_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.expiry_monthly_snapshots
    ADD CONSTRAINT expiry_monthly_snapshots_pkey PRIMARY KEY (id);


--
-- Name: expiry_notification_config expiry_notification_config_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.expiry_notification_config
    ADD CONSTRAINT expiry_notification_config_pkey PRIMARY KEY (id);


--
-- Name: export_jobs export_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs
    ADD CONSTRAINT export_jobs_pkey PRIMARY KEY (id);


--
-- Name: facilities facilities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.facilities
    ADD CONSTRAINT facilities_pkey PRIMARY KEY (id);


--
-- Name: formulation_records formulation_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.formulation_records
    ADD CONSTRAINT formulation_records_pkey PRIMARY KEY (id);


--
-- Name: google_calendar_config google_calendar_config_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.google_calendar_config
    ADD CONSTRAINT google_calendar_config_pkey PRIMARY KEY (id);


--
-- Name: import_jobs import_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.import_jobs
    ADD CONSTRAINT import_jobs_pkey PRIMARY KEY (id);


--
-- Name: inventory_snapshots inventory_snapshots_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.inventory_snapshots
    ADD CONSTRAINT inventory_snapshots_pkey PRIMARY KEY (warehouse_id, product_id);


--
-- Name: invitation_roles invitation_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitation_roles
    ADD CONSTRAINT invitation_roles_pkey PRIMARY KEY (invitation_id, role_id);


--
-- Name: invitations invitations_invitation_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitations
    ADD CONSTRAINT invitations_invitation_token_key UNIQUE (invitation_token);


--
-- Name: invitations invitations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitations
    ADD CONSTRAINT invitations_pkey PRIMARY KEY (id);


--
-- Name: ip_blocklist ip_blocklist_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ip_blocklist
    ADD CONSTRAINT ip_blocklist_pkey PRIMARY KEY (id);


--
-- Name: journal_entries journal_entries_entry_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entries
    ADD CONSTRAINT journal_entries_entry_no_key UNIQUE (entry_no);


--
-- Name: journal_entries journal_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entries
    ADD CONSTRAINT journal_entries_pkey PRIMARY KEY (id);


--
-- Name: journal_entry_lines journal_entry_lines_journal_entry_id_line_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entry_lines
    ADD CONSTRAINT journal_entry_lines_journal_entry_id_line_no_key UNIQUE (journal_entry_id, line_no);


--
-- Name: journal_entry_lines journal_entry_lines_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entry_lines
    ADD CONSTRAINT journal_entry_lines_pkey PRIMARY KEY (id);


--
-- Name: jwt_blacklist jwt_blacklist_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.jwt_blacklist
    ADD CONSTRAINT jwt_blacklist_pkey PRIMARY KEY (jti);


--
-- Name: leave_approvals leave_approvals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_approvals
    ADD CONSTRAINT leave_approvals_pkey PRIMARY KEY (id);


--
-- Name: leave_balance_usage leave_balance_usage_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balance_usage
    ADD CONSTRAINT leave_balance_usage_pkey PRIMARY KEY (id);


--
-- Name: leave_requests leave_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_pkey PRIMARY KEY (id);


--
-- Name: line_shelf_allocations line_shelf_allocations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.line_shelf_allocations
    ADD CONSTRAINT line_shelf_allocations_pkey PRIMARY KEY (id);


--
-- Name: login_events login_events_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.login_events
    ADD CONSTRAINT login_events_pkey PRIMARY KEY (id);


--
-- Name: management_reviews management_reviews_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.management_reviews
    ADD CONSTRAINT management_reviews_pkey PRIMARY KEY (id);


--
-- Name: management_reviews management_reviews_review_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.management_reviews
    ADD CONSTRAINT management_reviews_review_number_key UNIQUE (review_number);


--
-- Name: message_attachments message_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_attachments
    ADD CONSTRAINT message_attachments_pkey PRIMARY KEY (id);


--
-- Name: message_thread_participants message_thread_participants_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_thread_participants
    ADD CONSTRAINT message_thread_participants_pkey PRIMARY KEY (thread_id, user_id);


--
-- Name: message_threads message_threads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_threads
    ADD CONSTRAINT message_threads_pkey PRIMARY KEY (id);


--
-- Name: messages messages_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_pkey PRIMARY KEY (id);


--
-- Name: notification_routing notification_routing_event_type_role_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_routing
    ADD CONSTRAINT notification_routing_event_type_role_code_key UNIQUE (event_type, role_code);


--
-- Name: notification_routing notification_routing_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_routing
    ADD CONSTRAINT notification_routing_pkey PRIMARY KEY (id);


--
-- Name: notification_settings notification_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_settings
    ADD CONSTRAINT notification_settings_pkey PRIMARY KEY (user_id);


--
-- Name: notifications notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);


--
-- Name: observation_vet_reads observation_vet_reads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.observation_vet_reads
    ADD CONSTRAINT observation_vet_reads_pkey PRIMARY KEY (observation_id, vet_user_id);


--
-- Name: overtime_approvals overtime_approvals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_approvals
    ADD CONSTRAINT overtime_approvals_pkey PRIMARY KEY (id);


--
-- Name: overtime_records overtime_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_records
    ADD CONSTRAINT overtime_records_pkey PRIMARY KEY (id);


--
-- Name: partners partners_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.partners
    ADD CONSTRAINT partners_code_key UNIQUE (code);


--
-- Name: partners partners_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.partners
    ADD CONSTRAINT partners_pkey PRIMARY KEY (id);


--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (id);


--
-- Name: pdf_artifacts pdf_artifacts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pdf_artifacts
    ADD CONSTRAINT pdf_artifacts_pkey PRIMARY KEY (id);


--
-- Name: pens pens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pens
    ADD CONSTRAINT pens_pkey PRIMARY KEY (id);


--
-- Name: permissions permissions_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permissions
    ADD CONSTRAINT permissions_code_key UNIQUE (code);


--
-- Name: permissions permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.permissions
    ADD CONSTRAINT permissions_pkey PRIMARY KEY (id);


--
-- Name: pi_account_invites pi_account_invites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pi_account_invites
    ADD CONSTRAINT pi_account_invites_pkey PRIMARY KEY (id);


--
-- Name: planned_experiments planned_experiments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.planned_experiments
    ADD CONSTRAINT planned_experiments_pkey PRIMARY KEY (id);


--
-- Name: product_categories product_categories_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_categories
    ADD CONSTRAINT product_categories_code_key UNIQUE (code);


--
-- Name: product_categories product_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_categories
    ADD CONSTRAINT product_categories_pkey PRIMARY KEY (id);


--
-- Name: product_uom_conversions product_uom_conversions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_uom_conversions
    ADD CONSTRAINT product_uom_conversions_pkey PRIMARY KEY (id);


--
-- Name: product_uom_conversions product_uom_conversions_product_id_uom_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_uom_conversions
    ADD CONSTRAINT product_uom_conversions_product_id_uom_key UNIQUE (product_id, uom);


--
-- Name: products products_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_pkey PRIMARY KEY (id);


--
-- Name: products products_sku_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_sku_key UNIQUE (sku);


--
-- Name: protocol_activities protocol_activities_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_activities
    ADD CONSTRAINT protocol_activities_pkey PRIMARY KEY (id);


--
-- Name: protocol_ai_reviews protocol_ai_reviews_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_ai_reviews
    ADD CONSTRAINT protocol_ai_reviews_pkey PRIMARY KEY (id);


--
-- Name: protocol_attachments protocol_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_attachments
    ADD CONSTRAINT protocol_attachments_pkey PRIMARY KEY (id);


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_pkey PRIMARY KEY (id);


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_protocol_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_protocol_id_key UNIQUE (protocol_id);


--
-- Name: protocol_template_versions protocol_template_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_template_versions
    ADD CONSTRAINT protocol_template_versions_pkey PRIMARY KEY (id);


--
-- Name: protocol_template_versions protocol_template_versions_version_label_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_template_versions
    ADD CONSTRAINT protocol_template_versions_version_label_key UNIQUE (version_label);


--
-- Name: protocol_versions protocol_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_versions
    ADD CONSTRAINT protocol_versions_pkey PRIMARY KEY (id);


--
-- Name: protocol_versions protocol_versions_protocol_id_version_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_versions
    ADD CONSTRAINT protocol_versions_protocol_id_version_no_key UNIQUE (protocol_id, version_no);


--
-- Name: protocols protocols_iacuc_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocols
    ADD CONSTRAINT protocols_iacuc_no_key UNIQUE (iacuc_no);


--
-- Name: protocols protocols_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocols
    ADD CONSTRAINT protocols_pkey PRIMARY KEY (id);


--
-- Name: protocols protocols_protocol_no_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocols
    ADD CONSTRAINT protocols_protocol_no_key UNIQUE (protocol_no);


--
-- Name: qa_audit_schedules qa_audit_schedules_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_audit_schedules
    ADD CONSTRAINT qa_audit_schedules_pkey PRIMARY KEY (id);


--
-- Name: qa_capa qa_capa_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_capa
    ADD CONSTRAINT qa_capa_pkey PRIMARY KEY (id);


--
-- Name: qa_inspection_items qa_inspection_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_inspection_items
    ADD CONSTRAINT qa_inspection_items_pkey PRIMARY KEY (id);


--
-- Name: qa_inspections qa_inspections_inspection_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_inspections
    ADD CONSTRAINT qa_inspections_inspection_number_key UNIQUE (inspection_number);


--
-- Name: qa_inspections qa_inspections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_inspections
    ADD CONSTRAINT qa_inspections_pkey PRIMARY KEY (id);


--
-- Name: qa_non_conformances qa_non_conformances_nc_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_non_conformances
    ADD CONSTRAINT qa_non_conformances_nc_number_key UNIQUE (nc_number);


--
-- Name: qa_non_conformances qa_non_conformances_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_non_conformances
    ADD CONSTRAINT qa_non_conformances_pkey PRIMARY KEY (id);


--
-- Name: qa_schedule_items qa_schedule_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_schedule_items
    ADD CONSTRAINT qa_schedule_items_pkey PRIMARY KEY (id);


--
-- Name: qa_sop_acknowledgments qa_sop_acknowledgments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_acknowledgments
    ADD CONSTRAINT qa_sop_acknowledgments_pkey PRIMARY KEY (id);


--
-- Name: qa_sop_acknowledgments qa_sop_acknowledgments_sop_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_acknowledgments
    ADD CONSTRAINT qa_sop_acknowledgments_sop_id_user_id_key UNIQUE (sop_id, user_id);


--
-- Name: qa_sop_documents qa_sop_documents_document_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_documents
    ADD CONSTRAINT qa_sop_documents_document_number_key UNIQUE (document_number);


--
-- Name: qa_sop_documents qa_sop_documents_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_documents
    ADD CONSTRAINT qa_sop_documents_pkey PRIMARY KEY (id);


--
-- Name: record_annotations record_annotations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.record_annotations
    ADD CONSTRAINT record_annotations_pkey PRIMARY KEY (id);


--
-- Name: record_versions record_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.record_versions
    ADD CONSTRAINT record_versions_pkey PRIMARY KEY (id);


--
-- Name: reference_standards reference_standards_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference_standards
    ADD CONSTRAINT reference_standards_pkey PRIMARY KEY (id);


--
-- Name: refresh_tokens refresh_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_pkey PRIMARY KEY (id);


--
-- Name: report_history report_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.report_history
    ADD CONSTRAINT report_history_pkey PRIMARY KEY (id);


--
-- Name: review_assignments review_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_assignments
    ADD CONSTRAINT review_assignments_pkey PRIMARY KEY (id);


--
-- Name: review_assignments review_assignments_protocol_id_reviewer_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_assignments
    ADD CONSTRAINT review_assignments_protocol_id_reviewer_id_key UNIQUE (protocol_id, reviewer_id);


--
-- Name: review_comments review_comments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_pkey PRIMARY KEY (id);


--
-- Name: review_round_history review_round_history_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_round_history
    ADD CONSTRAINT review_round_history_pkey PRIMARY KEY (id);


--
-- Name: risk_register risk_register_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.risk_register
    ADD CONSTRAINT risk_register_pkey PRIMARY KEY (id);


--
-- Name: risk_register risk_register_risk_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.risk_register
    ADD CONSTRAINT risk_register_risk_number_key UNIQUE (risk_number);


--
-- Name: role_permissions role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_pkey PRIMARY KEY (role_id, permission_id);


--
-- Name: role_training_requirements role_training_requirements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_training_requirements
    ADD CONSTRAINT role_training_requirements_pkey PRIMARY KEY (id);


--
-- Name: roles roles_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_code_key UNIQUE (code);


--
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.roles
    ADD CONSTRAINT roles_pkey PRIMARY KEY (id);


--
-- Name: scheduled_reports scheduled_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scheduled_reports
    ADD CONSTRAINT scheduled_reports_pkey PRIMARY KEY (id);


--
-- Name: security_alert_config security_alert_config_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_alert_config
    ADD CONSTRAINT security_alert_config_pkey PRIMARY KEY (key);


--
-- Name: security_alerts security_alerts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_alerts
    ADD CONSTRAINT security_alerts_pkey PRIMARY KEY (id);


--
-- Name: security_notification_channels security_notification_channels_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_notification_channels
    ADD CONSTRAINT security_notification_channels_pkey PRIMARY KEY (id);


--
-- Name: signature_bridge_sessions signature_bridge_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.signature_bridge_sessions
    ADD CONSTRAINT signature_bridge_sessions_pkey PRIMARY KEY (id);


--
-- Name: sku_categories sku_categories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sku_categories
    ADD CONSTRAINT sku_categories_pkey PRIMARY KEY (code);


--
-- Name: sku_sequences sku_sequences_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sku_sequences
    ADD CONSTRAINT sku_sequences_pkey PRIMARY KEY (category_code, subcategory_code);


--
-- Name: sku_subcategories sku_subcategories_category_code_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sku_subcategories
    ADD CONSTRAINT sku_subcategories_category_code_code_key UNIQUE (category_code, code);


--
-- Name: sku_subcategories sku_subcategories_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sku_subcategories
    ADD CONSTRAINT sku_subcategories_pkey PRIMARY KEY (id);


--
-- Name: species species_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.species
    ADD CONSTRAINT species_code_key UNIQUE (code);


--
-- Name: species species_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.species
    ADD CONSTRAINT species_pkey PRIMARY KEY (id);


--
-- Name: stock_ledger stock_ledger_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_ledger
    ADD CONSTRAINT stock_ledger_pkey PRIMARY KEY (id);


--
-- Name: storage_location_inventory storage_location_inventory_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_location_inventory
    ADD CONSTRAINT storage_location_inventory_pkey PRIMARY KEY (id);


--
-- Name: storage_locations storage_locations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_locations
    ADD CONSTRAINT storage_locations_pkey PRIMARY KEY (id);


--
-- Name: storage_locations storage_locations_warehouse_id_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_locations
    ADD CONSTRAINT storage_locations_warehouse_id_code_key UNIQUE (warehouse_id, code);


--
-- Name: study_final_reports study_final_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.study_final_reports
    ADD CONSTRAINT study_final_reports_pkey PRIMARY KEY (id);


--
-- Name: study_final_reports study_final_reports_report_number_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.study_final_reports
    ADD CONSTRAINT study_final_reports_report_number_key UNIQUE (report_number);


--
-- Name: surgery_vet_reads surgery_vet_reads_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.surgery_vet_reads
    ADD CONSTRAINT surgery_vet_reads_pkey PRIMARY KEY (surgery_id, vet_user_id);


--
-- Name: system_settings system_settings_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_settings
    ADD CONSTRAINT system_settings_pkey PRIMARY KEY (key);


--
-- Name: training_records training_records_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.training_records
    ADD CONSTRAINT training_records_pkey PRIMARY KEY (id);


--
-- Name: transfer_vet_evaluations transfer_vet_evaluations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transfer_vet_evaluations
    ADD CONSTRAINT transfer_vet_evaluations_pkey PRIMARY KEY (id);


--
-- Name: transfer_vet_evaluations transfer_vet_evaluations_transfer_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transfer_vet_evaluations
    ADD CONSTRAINT transfer_vet_evaluations_transfer_id_key UNIQUE (transfer_id);


--
-- Name: treatment_drug_options treatment_drug_options_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.treatment_drug_options
    ADD CONSTRAINT treatment_drug_options_pkey PRIMARY KEY (id);


--
-- Name: notification_routing uq_nr_event_target; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_routing
    ADD CONSTRAINT uq_nr_event_target UNIQUE (event_type, target_kind, target_value);


--
-- Name: user_activity_aggregates user_activity_aggregates_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_aggregates
    ADD CONSTRAINT user_activity_aggregates_pkey PRIMARY KEY (id);


--
-- Name: user_activity_aggregates user_activity_aggregates_user_id_aggregate_date_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_aggregates
    ADD CONSTRAINT user_activity_aggregates_user_id_aggregate_date_key UNIQUE (user_id, aggregate_date);


--
-- Name: user_activity_logs user_activity_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs
    ADD CONSTRAINT user_activity_logs_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2026_q1 user_activity_logs_2026_q1_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2026_q1
    ADD CONSTRAINT user_activity_logs_2026_q1_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2026_q2 user_activity_logs_2026_q2_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2026_q2
    ADD CONSTRAINT user_activity_logs_2026_q2_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2026_q3 user_activity_logs_2026_q3_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2026_q3
    ADD CONSTRAINT user_activity_logs_2026_q3_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2026_q4 user_activity_logs_2026_q4_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2026_q4
    ADD CONSTRAINT user_activity_logs_2026_q4_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2027_q1 user_activity_logs_2027_q1_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2027_q1
    ADD CONSTRAINT user_activity_logs_2027_q1_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2027_q2 user_activity_logs_2027_q2_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2027_q2
    ADD CONSTRAINT user_activity_logs_2027_q2_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2027_q3 user_activity_logs_2027_q3_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2027_q3
    ADD CONSTRAINT user_activity_logs_2027_q3_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2027_q4 user_activity_logs_2027_q4_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2027_q4
    ADD CONSTRAINT user_activity_logs_2027_q4_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2028_q1 user_activity_logs_2028_q1_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2028_q1
    ADD CONSTRAINT user_activity_logs_2028_q1_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2028_q2 user_activity_logs_2028_q2_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2028_q2
    ADD CONSTRAINT user_activity_logs_2028_q2_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2028_q3 user_activity_logs_2028_q3_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2028_q3
    ADD CONSTRAINT user_activity_logs_2028_q3_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_activity_logs_2028_q4 user_activity_logs_2028_q4_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_logs_2028_q4
    ADD CONSTRAINT user_activity_logs_2028_q4_pkey PRIMARY KEY (id, partition_date);


--
-- Name: user_aup_profiles user_aup_profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_aup_profiles
    ADD CONSTRAINT user_aup_profiles_pkey PRIMARY KEY (user_id);


--
-- Name: user_mcp_keys user_mcp_keys_key_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_mcp_keys
    ADD CONSTRAINT user_mcp_keys_key_hash_key UNIQUE (key_hash);


--
-- Name: user_mcp_keys user_mcp_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_mcp_keys
    ADD CONSTRAINT user_mcp_keys_pkey PRIMARY KEY (id);


--
-- Name: user_preferences user_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_preferences
    ADD CONSTRAINT user_preferences_pkey PRIMARY KEY (id);


--
-- Name: user_preferences user_preferences_user_id_preference_key_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_preferences
    ADD CONSTRAINT user_preferences_user_id_preference_key_key UNIQUE (user_id, preference_key);


--
-- Name: user_protocols user_protocols_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_protocols
    ADD CONSTRAINT user_protocols_pkey PRIMARY KEY (user_id, protocol_id);


--
-- Name: user_roles user_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_pkey PRIMARY KEY (user_id, role_id);


--
-- Name: user_sessions user_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: vet_patrol_entries vet_patrol_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entries
    ADD CONSTRAINT vet_patrol_entries_pkey PRIMARY KEY (id);


--
-- Name: vet_patrol_entry_animals vet_patrol_entry_animals_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entry_animals
    ADD CONSTRAINT vet_patrol_entry_animals_pkey PRIMARY KEY (entry_id, animal_id);


--
-- Name: vet_patrol_entry_photos vet_patrol_entry_photos_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entry_photos
    ADD CONSTRAINT vet_patrol_entry_photos_pkey PRIMARY KEY (id);


--
-- Name: vet_patrol_photos vet_patrol_photos_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_photos
    ADD CONSTRAINT vet_patrol_photos_pkey PRIMARY KEY (id);


--
-- Name: vet_patrol_reports vet_patrol_reports_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_reports
    ADD CONSTRAINT vet_patrol_reports_pkey PRIMARY KEY (id);


--
-- Name: vet_review_assignments vet_review_assignments_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_review_assignments
    ADD CONSTRAINT vet_review_assignments_pkey PRIMARY KEY (id);


--
-- Name: vet_review_assignments vet_review_assignments_protocol_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_review_assignments
    ADD CONSTRAINT vet_review_assignments_protocol_id_key UNIQUE (protocol_id);


--
-- Name: warehouses warehouses_code_key; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.warehouses
    ADD CONSTRAINT warehouses_code_key UNIQUE (code);


--
-- Name: warehouses warehouses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.warehouses
    ADD CONSTRAINT warehouses_pkey PRIMARY KEY (id);


--
-- Name: zones zones_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.zones
    ADD CONSTRAINT zones_pkey PRIMARY KEY (id);


--
-- Name: idx_ai_query_logs_api_key; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_query_logs_api_key ON ONLY public.ai_query_logs USING btree (api_key_id, created_at);


--
-- Name: ai_query_logs_2026_08_api_key_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX ai_query_logs_2026_08_api_key_id_created_at_idx ON public.ai_query_logs_2026_08 USING btree (api_key_id, created_at);


--
-- Name: ai_query_logs_2026_09_api_key_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX ai_query_logs_2026_09_api_key_id_created_at_idx ON public.ai_query_logs_2026_09 USING btree (api_key_id, created_at);


--
-- Name: idx_activity_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_actor ON ONLY public.user_activity_logs USING btree (actor_user_id, created_at DESC);


--
-- Name: idx_activity_actor_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_actor_created ON ONLY public.user_activity_logs USING btree (actor_user_id, created_at DESC);


--
-- Name: idx_activity_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_category ON ONLY public.user_activity_logs USING btree (event_category, created_at DESC);


--
-- Name: idx_activity_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_date ON ONLY public.user_activity_logs USING btree (partition_date, created_at DESC);


--
-- Name: idx_activity_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_entity ON ONLY public.user_activity_logs USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: idx_activity_event_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_event_type ON ONLY public.user_activity_logs USING btree (event_type, created_at DESC);


--
-- Name: idx_activity_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_ip ON ONLY public.user_activity_logs USING btree (ip_address, created_at DESC);


--
-- Name: idx_activity_logs_integrity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_logs_integrity ON ONLY public.user_activity_logs USING btree (created_at, integrity_hash);


--
-- Name: idx_activity_suspicious; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_activity_suspicious ON ONLY public.user_activity_logs USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: idx_afcr_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_afcr_animal_id ON public.animal_field_correction_requests USING btree (animal_id);


--
-- Name: idx_afcr_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_afcr_created_at ON public.animal_field_correction_requests USING btree (created_at DESC);


--
-- Name: idx_afcr_requested_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_afcr_requested_by ON public.animal_field_correction_requests USING btree (requested_by);


--
-- Name: idx_afcr_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_afcr_status ON public.animal_field_correction_requests USING btree (status);


--
-- Name: idx_aggregates_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aggregates_date ON public.user_activity_aggregates USING btree (aggregate_date DESC);


--
-- Name: idx_aggregates_user_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_aggregates_user_date ON public.user_activity_aggregates USING btree (user_id, aggregate_date DESC);


--
-- Name: idx_ai_api_keys_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_api_keys_active ON public.ai_api_keys USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_ai_api_keys_key_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_api_keys_key_hash ON public.ai_api_keys USING btree (key_hash);


--
-- Name: idx_ai_reviews_protocol_latest; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ai_reviews_protocol_latest ON public.protocol_ai_reviews USING btree (protocol_id, created_at DESC);


--
-- Name: idx_ai_reviews_version_type; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_ai_reviews_version_type ON public.protocol_ai_reviews USING btree (protocol_version_id, review_type) WHERE (protocol_version_id IS NOT NULL);


--
-- Name: idx_alerts_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_alerts_status ON public.security_alerts USING btree (status, created_at DESC);


--
-- Name: idx_amendment_review_assignments_amendment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendment_review_assignments_amendment ON public.amendment_review_assignments USING btree (amendment_id);


--
-- Name: idx_amendment_review_assignments_reviewer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendment_review_assignments_reviewer ON public.amendment_review_assignments USING btree (reviewer_id);


--
-- Name: idx_amendment_status_history_amendment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendment_status_history_amendment_id ON public.amendment_status_history USING btree (amendment_id);


--
-- Name: idx_amendment_versions_amendment_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendment_versions_amendment_id ON public.amendment_versions USING btree (amendment_id);


--
-- Name: idx_amendments_approved_signature; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendments_approved_signature ON public.amendments USING btree (approved_signature_id) WHERE (approved_signature_id IS NOT NULL);


--
-- Name: idx_amendments_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendments_protocol_id ON public.amendments USING btree (protocol_id);


--
-- Name: idx_amendments_rejected_signature; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendments_rejected_signature ON public.amendments USING btree (rejected_signature_id) WHERE (rejected_signature_id IS NOT NULL);


--
-- Name: idx_amendments_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_amendments_status ON public.amendments USING btree (status);


--
-- Name: idx_animal_blood_test_items_blood_test_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_test_items_blood_test_id ON public.animal_blood_test_items USING btree (blood_test_id);


--
-- Name: idx_animal_blood_test_items_current; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_test_items_current ON public.animal_blood_test_items USING btree (blood_test_id) WHERE (superseded_by_id IS NULL);


--
-- Name: idx_animal_blood_test_items_superseded_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_test_items_superseded_by ON public.animal_blood_test_items USING btree (superseded_by_id) WHERE (superseded_by_id IS NOT NULL);


--
-- Name: idx_animal_blood_test_items_template_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_test_items_template_id ON public.animal_blood_test_items USING btree (template_id);


--
-- Name: idx_animal_blood_tests_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_tests_animal_id ON public.animal_blood_tests USING btree (animal_id);


--
-- Name: idx_animal_blood_tests_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_tests_deleted_at ON public.animal_blood_tests USING btree (deleted_at) WHERE (deleted_at IS NULL);


--
-- Name: idx_animal_blood_tests_is_locked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_tests_is_locked ON public.animal_blood_tests USING btree (is_locked) WHERE (is_locked = true);


--
-- Name: idx_animal_blood_tests_test_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_blood_tests_test_date ON public.animal_blood_tests USING btree (test_date);


--
-- Name: idx_animal_import_batches_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_import_batches_created_at ON public.animal_import_batches USING btree (created_at DESC);


--
-- Name: idx_animal_import_batches_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_import_batches_created_by ON public.animal_import_batches USING btree (created_by);


--
-- Name: idx_animal_import_batches_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_import_batches_status ON public.animal_import_batches USING btree (status);


--
-- Name: idx_animal_observations_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_observations_animal_id ON public.animal_observations USING btree (animal_id);


--
-- Name: idx_animal_observations_event_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_observations_event_date ON public.animal_observations USING btree (event_date);


--
-- Name: idx_animal_observations_is_locked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_observations_is_locked ON public.animal_observations USING btree (is_locked) WHERE (is_locked = true);


--
-- Name: idx_animal_pathology_reports_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_pathology_reports_animal_id ON public.animal_pathology_reports USING btree (animal_id);


--
-- Name: idx_animal_record_attachments_record; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_record_attachments_record ON public.animal_record_attachments USING btree (record_type, record_id);


--
-- Name: idx_animal_sacrifices_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_sacrifices_animal_id ON public.animal_sacrifices USING btree (animal_id);


--
-- Name: idx_animal_surgeries_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_surgeries_animal_id ON public.animal_surgeries USING btree (animal_id);


--
-- Name: idx_animal_surgeries_is_locked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_surgeries_is_locked ON public.animal_surgeries USING btree (is_locked) WHERE (is_locked = true);


--
-- Name: idx_animal_surgeries_surgery_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_surgeries_surgery_date ON public.animal_surgeries USING btree (surgery_date);


--
-- Name: idx_animal_vaccinations_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_vaccinations_animal_id ON public.animal_vaccinations USING btree (animal_id);


--
-- Name: idx_animal_vet_advices_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_animal_vet_advices_animal ON public.animal_vet_advices USING btree (animal_id);


--
-- Name: idx_animal_weights_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_weights_animal_id ON public.animal_weights USING btree (animal_id);


--
-- Name: idx_animal_weights_measure_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animal_weights_measure_date ON public.animal_weights USING btree (measure_date);


--
-- Name: idx_animals_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_active ON public.animals USING btree (deleted_at) WHERE (deleted_at IS NULL);


--
-- Name: idx_animals_ear_tag; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_ear_tag ON public.animals USING btree (ear_tag);


--
-- Name: idx_animals_ear_tag_trgm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_ear_tag_trgm ON public.animals USING gin (ear_tag public.gin_trgm_ops);


--
-- Name: idx_animals_glp_study_no; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_glp_study_no ON public.animals USING btree (glp_study_no);


--
-- Name: idx_animals_iacuc_no; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_iacuc_no ON public.animals USING btree (iacuc_no);


--
-- Name: idx_animals_pen_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_pen_id ON public.animals USING btree (pen_id);


--
-- Name: idx_animals_pen_location; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_pen_location ON public.animals USING btree (pen_location);


--
-- Name: idx_animals_pen_location_trgm; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_pen_location_trgm ON public.animals USING gin (pen_location public.gin_trgm_ops);


--
-- Name: idx_animals_reserved_planned; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_reserved_planned ON public.animals USING btree (reserved_planned_experiment_id) WHERE (reserved_planned_experiment_id IS NOT NULL);


--
-- Name: idx_animals_reserved_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_reserved_protocol ON public.animals USING btree (reserved_protocol_id) WHERE (reserved_protocol_id IS NOT NULL);


--
-- Name: idx_animals_species_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_species_id ON public.animals USING btree (species_id);


--
-- Name: idx_animals_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_animals_status ON public.animals USING btree (status);


--
-- Name: idx_annot_record; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_annot_record ON public.record_annotations USING btree (record_type, record_id);


--
-- Name: idx_annual_leave_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_annual_leave_expires ON public.annual_leave_entitlements USING btree (expires_at) WHERE ((NOT is_expired) AND ((entitled_days - used_days) > (0)::numeric));


--
-- Name: idx_annual_leave_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_annual_leave_user ON public.annual_leave_entitlements USING btree (user_id, entitlement_year DESC);


--
-- Name: idx_ap_payments_partner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ap_payments_partner ON public.ap_payments USING btree (partner_id);


--
-- Name: idx_application_notices_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_application_notices_active ON public.application_notices USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_approval_approver; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_approval_approver ON public.leave_approvals USING btree (approver_id, created_at DESC);


--
-- Name: idx_approval_request; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_approval_request ON public.leave_approvals USING btree (leave_request_id, created_at);


--
-- Name: idx_ar_receipts_partner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ar_receipts_partner ON public.ar_receipts USING btree (partner_id);


--
-- Name: idx_attachments_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attachments_category ON public.attachments USING btree (category);


--
-- Name: idx_attachments_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attachments_entity ON public.attachments USING btree (entity_type, entity_id);


--
-- Name: idx_attendance_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attendance_date ON public.attendance_records USING btree (work_date DESC);


--
-- Name: idx_attendance_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attendance_status ON public.attendance_records USING btree (status);


--
-- Name: idx_attendance_user_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_attendance_user_date ON public.attendance_records USING btree (user_id, work_date DESC);


--
-- Name: idx_audit_entity_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_entity_created ON ONLY public.user_activity_logs USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: idx_audit_hmac_null; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_hmac_null ON ONLY public.user_activity_logs USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: INDEX idx_audit_hmac_null; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX public.idx_audit_hmac_null IS 'R28-5：HMAC legacy backfill 監控用 partial index。配合 scheduler register_hmac_legacy_gauge_job 每 10 分鐘 COUNT(*) 走 index-only scan，不掃全 partitioned 表。Backfill 完成後（30 天 = 0）可移除。';


--
-- Name: idx_audit_logs_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_actor ON public.audit_logs USING btree (actor_user_id);


--
-- Name: idx_audit_logs_after_data_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_after_data_gin ON public.audit_logs USING gin (after_data jsonb_path_ops);


--
-- Name: idx_audit_logs_before_data_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_before_data_gin ON public.audit_logs USING gin (before_data jsonb_path_ops);


--
-- Name: idx_audit_logs_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_created_at ON public.audit_logs USING btree (created_at);


--
-- Name: idx_audit_logs_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_audit_logs_entity ON public.audit_logs USING btree (entity_type, entity_id);


--
-- Name: idx_blood_test_panel_items_panel; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_panel_items_panel ON public.blood_test_panel_items USING btree (panel_id);


--
-- Name: idx_blood_test_panels_is_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_panels_is_active ON public.blood_test_panels USING btree (is_active);


--
-- Name: idx_blood_test_panels_sort_order; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_panels_sort_order ON public.blood_test_panels USING btree (sort_order);


--
-- Name: idx_blood_test_presets_is_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_presets_is_active ON public.blood_test_presets USING btree (is_active);


--
-- Name: idx_blood_test_presets_sort_order; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_presets_sort_order ON public.blood_test_presets USING btree (sort_order);


--
-- Name: idx_blood_test_templates_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_templates_code ON public.blood_test_templates USING btree (code);


--
-- Name: idx_blood_test_templates_is_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_blood_test_templates_is_active ON public.blood_test_templates USING btree (is_active);


--
-- Name: idx_buildings_facility_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_buildings_facility_id ON public.buildings USING btree (facility_id);


--
-- Name: idx_byproduct_samples_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_byproduct_samples_animal ON public.euthanasia_byproduct_samples USING btree (animal_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_byproduct_samples_euthanasia; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_byproduct_samples_euthanasia ON public.euthanasia_byproduct_samples USING btree (euthanasia_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_byproduct_samples_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_byproduct_samples_protocol ON public.euthanasia_byproduct_samples USING btree (source_protocol_id) WHERE (deleted_at IS NULL);


--
-- Name: idx_calendar_event_sync_leave; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_calendar_event_sync_leave ON public.calendar_event_sync USING btree (leave_request_id);


--
-- Name: idx_calendar_event_sync_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_calendar_event_sync_status ON public.calendar_event_sync USING btree (sync_status);


--
-- Name: idx_calendar_sync_conflicts_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_calendar_sync_conflicts_status ON public.calendar_sync_conflicts USING btree (status);


--
-- Name: idx_calendar_sync_history_started; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_calendar_sync_history_started ON public.calendar_sync_history USING btree (started_at DESC);


--
-- Name: idx_calendar_sync_history_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_calendar_sync_history_status ON public.calendar_sync_history USING btree (status);


--
-- Name: idx_calibrations_ref_standard; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_calibrations_ref_standard ON public.equipment_calibrations USING btree (reference_standard_id);


--
-- Name: idx_care_medication_records_deleted_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_care_medication_records_deleted_at ON public.care_medication_records USING btree (deleted_at) WHERE (deleted_at IS NULL);


--
-- Name: idx_care_medication_records_is_locked; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_care_medication_records_is_locked ON public.care_medication_records USING btree (is_locked) WHERE (is_locked = true);


--
-- Name: idx_care_medication_records_record; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_care_medication_records_record ON public.care_medication_records USING btree (record_type, record_id);


--
-- Name: idx_change_reasons_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_change_reasons_entity ON public.change_reasons USING btree (entity_type, entity_id);


--
-- Name: idx_change_requests_requester; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_change_requests_requester ON public.change_requests USING btree (requested_by);


--
-- Name: idx_change_requests_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_change_requests_status ON public.change_requests USING btree (status);


--
-- Name: idx_change_requests_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_change_requests_type ON public.change_requests USING btree (change_type);


--
-- Name: idx_chart_of_accounts_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_chart_of_accounts_code ON public.chart_of_accounts USING btree (code);


--
-- Name: idx_comp_time_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comp_time_expires ON public.comp_time_balances USING btree (expires_at) WHERE ((NOT is_expired) AND ((original_hours - used_hours) > (0)::numeric));


--
-- Name: idx_comp_time_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_comp_time_user ON public.comp_time_balances USING btree (user_id, earned_date DESC);


--
-- Name: idx_competency_assessor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_competency_assessor ON public.competency_assessments USING btree (assessor_id);


--
-- Name: idx_competency_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_competency_user ON public.competency_assessments USING btree (user_id);


--
-- Name: idx_competency_valid; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_competency_valid ON public.competency_assessments USING btree (valid_until);


--
-- Name: idx_ctrl_docs_owner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ctrl_docs_owner ON public.controlled_documents USING btree (owner_id);


--
-- Name: idx_ctrl_docs_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ctrl_docs_status ON public.controlled_documents USING btree (status);


--
-- Name: idx_ctrl_docs_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ctrl_docs_type ON public.controlled_documents USING btree (doc_type);


--
-- Name: idx_departments_parent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_departments_parent_id ON public.departments USING btree (parent_id);


--
-- Name: idx_doc_acks_doc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_doc_acks_doc ON public.document_acknowledgments USING btree (document_id);


--
-- Name: idx_doc_acks_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_doc_acks_user ON public.document_acknowledgments USING btree (user_id);


--
-- Name: idx_doc_revisions_doc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_doc_revisions_doc ON public.document_revisions USING btree (document_id);


--
-- Name: idx_document_lines_document_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_lines_document_id ON public.document_lines USING btree (document_id);


--
-- Name: idx_document_lines_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_lines_product_id ON public.document_lines USING btree (product_id);


--
-- Name: idx_document_lines_storage_from_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_lines_storage_from_id ON public.document_lines USING btree (storage_location_from_id) WHERE (storage_location_from_id IS NOT NULL);


--
-- Name: idx_document_lines_storage_location_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_lines_storage_location_id ON public.document_lines USING btree (storage_location_id);


--
-- Name: idx_document_lines_storage_to_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_lines_storage_to_id ON public.document_lines USING btree (storage_location_to_id) WHERE (storage_location_to_id IS NOT NULL);


--
-- Name: idx_document_lines_warehouse_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_document_lines_warehouse_id ON public.document_lines USING btree (warehouse_id);


--
-- Name: idx_documents_doc_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_doc_date ON public.documents USING btree (doc_date);


--
-- Name: idx_documents_doc_no; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_doc_no ON public.documents USING btree (doc_no);


--
-- Name: idx_documents_doc_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_doc_type ON public.documents USING btree (doc_type);


--
-- Name: idx_documents_partner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_partner_id ON public.documents USING btree (partner_id);


--
-- Name: idx_documents_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_protocol_id ON public.documents USING btree (protocol_id);


--
-- Name: idx_documents_reverses_doc_id_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_documents_reverses_doc_id_unique ON public.documents USING btree (reverses_doc_id) WHERE (reverses_doc_id IS NOT NULL);


--
-- Name: idx_documents_source_doc_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_source_doc_id ON public.documents USING btree (source_doc_id);


--
-- Name: idx_documents_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_status ON public.documents USING btree (status);


--
-- Name: idx_documents_system_generated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_system_generated ON public.documents USING btree (system_generated) WHERE system_generated;


--
-- Name: idx_documents_warehouse_from_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_warehouse_from_id ON public.documents USING btree (warehouse_from_id);


--
-- Name: idx_documents_warehouse_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_warehouse_id ON public.documents USING btree (warehouse_id);


--
-- Name: idx_documents_warehouse_to_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_documents_warehouse_to_id ON public.documents USING btree (warehouse_to_id);


--
-- Name: idx_env_points_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_env_points_active ON public.environment_monitoring_points USING btree (is_active);


--
-- Name: idx_env_points_building; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_env_points_building ON public.environment_monitoring_points USING btree (building_id);


--
-- Name: idx_env_readings_oor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_env_readings_oor ON public.environment_readings USING btree (is_out_of_range) WHERE (is_out_of_range = true);


--
-- Name: idx_env_readings_point; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_env_readings_point ON public.environment_readings USING btree (monitoring_point_id);


--
-- Name: idx_env_readings_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_env_readings_time ON public.environment_readings USING btree (reading_time);


--
-- Name: idx_equipment_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_active ON public.equipment USING btree (is_active);


--
-- Name: idx_equipment_annual_plans_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_annual_plans_equipment ON public.equipment_annual_plans USING btree (equipment_id);


--
-- Name: idx_equipment_annual_plans_year; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_annual_plans_year ON public.equipment_annual_plans USING btree (year);


--
-- Name: idx_equipment_calibrations_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_calibrations_equipment ON public.equipment_calibrations USING btree (equipment_id);


--
-- Name: idx_equipment_calibrations_partner_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_calibrations_partner_id ON public.equipment_calibrations USING btree (partner_id);


--
-- Name: idx_equipment_calibrations_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_calibrations_type ON public.equipment_calibrations USING btree (calibration_type);


--
-- Name: idx_equipment_disposals_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_disposals_equipment ON public.equipment_disposals USING btree (equipment_id);


--
-- Name: idx_equipment_disposals_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_disposals_status ON public.equipment_disposals USING btree (status);


--
-- Name: idx_equipment_idle_requests_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_idle_requests_equipment ON public.equipment_idle_requests USING btree (equipment_id);


--
-- Name: idx_equipment_idle_requests_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_idle_requests_status ON public.equipment_idle_requests USING btree (status);


--
-- Name: idx_equipment_maintenance_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_maintenance_equipment ON public.equipment_maintenance_records USING btree (equipment_id);


--
-- Name: idx_equipment_maintenance_repair_partner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_maintenance_repair_partner ON public.equipment_maintenance_records USING btree (repair_partner_id);


--
-- Name: idx_equipment_maintenance_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_maintenance_status ON public.equipment_maintenance_records USING btree (status);


--
-- Name: idx_equipment_maintenance_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_maintenance_type ON public.equipment_maintenance_records USING btree (maintenance_type);


--
-- Name: idx_equipment_name; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_name ON public.equipment USING btree (name);


--
-- Name: idx_equipment_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_status ON public.equipment USING btree (status);


--
-- Name: idx_equipment_status_logs_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_status_logs_created ON public.equipment_status_logs USING btree (created_at DESC);


--
-- Name: idx_equipment_status_logs_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_status_logs_equipment ON public.equipment_status_logs USING btree (equipment_id);


--
-- Name: idx_equipment_suppliers_equipment; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_suppliers_equipment ON public.equipment_suppliers USING btree (equipment_id);


--
-- Name: idx_equipment_suppliers_partner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_equipment_suppliers_partner ON public.equipment_suppliers USING btree (partner_id);


--
-- Name: idx_esig_entity; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_esig_entity ON public.electronic_signatures USING btree (entity_type, entity_id);


--
-- Name: idx_esig_meaning; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_esig_meaning ON public.electronic_signatures USING btree (meaning);


--
-- Name: idx_esig_signer_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_esig_signer_id ON public.electronic_signatures USING btree (signer_id);


--
-- Name: idx_euthanasia_appeals_order; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_euthanasia_appeals_order ON public.euthanasia_appeals USING btree (order_id);


--
-- Name: idx_euthanasia_orders_animal_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_euthanasia_orders_animal_id ON public.euthanasia_orders USING btree (animal_id);


--
-- Name: idx_euthanasia_orders_pi_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_euthanasia_orders_pi_user_id ON public.euthanasia_orders USING btree (pi_user_id);


--
-- Name: idx_euthanasia_orders_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_euthanasia_orders_status ON public.euthanasia_orders USING btree (status);


--
-- Name: idx_euthanasia_orders_vet_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_euthanasia_orders_vet_user_id ON public.euthanasia_orders USING btree (vet_user_id);


--
-- Name: idx_event_outbox_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_pending ON public.event_outbox USING btree (next_attempt_at) WHERE (status = ANY (ARRAY['PENDING'::text, 'FAILED'::text]));


--
-- Name: idx_event_outbox_source; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_source ON public.event_outbox USING btree (source_entity, source_entity_id);


--
-- Name: idx_event_outbox_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_status ON public.event_outbox USING btree (status, enqueued_at);


--
-- Name: idx_event_outbox_stuck_sending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_event_outbox_stuck_sending ON public.event_outbox USING btree (started_at) WHERE (status = 'SENDING'::text);


--
-- Name: idx_expiry_snapshot_ym; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_expiry_snapshot_ym ON public.expiry_monthly_snapshots USING btree (snapshot_ym);


--
-- Name: idx_export_jobs_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_export_jobs_created_by ON public.export_jobs USING btree (created_by);


--
-- Name: idx_export_jobs_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_export_jobs_status ON public.export_jobs USING btree (status);


--
-- Name: idx_final_reports_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_final_reports_protocol ON public.study_final_reports USING btree (protocol_id);


--
-- Name: idx_final_reports_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_final_reports_status ON public.study_final_reports USING btree (status);


--
-- Name: idx_formulation_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_formulation_date ON public.formulation_records USING btree (formulation_date);


--
-- Name: idx_formulation_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_formulation_product ON public.formulation_records USING btree (product_id);


--
-- Name: idx_formulation_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_formulation_protocol ON public.formulation_records USING btree (protocol_id);


--
-- Name: idx_google_calendar_config_singleton; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_google_calendar_config_singleton ON public.google_calendar_config USING btree ((true));


--
-- Name: idx_import_jobs_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_import_jobs_created_by ON public.import_jobs USING btree (created_by);


--
-- Name: idx_import_jobs_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_import_jobs_status ON public.import_jobs USING btree (status);


--
-- Name: idx_inventory_snapshots_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_inventory_snapshots_product_id ON public.inventory_snapshots USING btree (product_id);


--
-- Name: idx_invitation_roles_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_invitation_roles_role_id ON public.invitation_roles USING btree (role_id);


--
-- Name: idx_invitations_email_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_invitations_email_pending ON public.invitations USING btree (email) WHERE ((status)::text = 'pending'::text);


--
-- Name: idx_invitations_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_invitations_expires_at ON public.invitations USING btree (expires_at) WHERE ((status)::text = 'pending'::text);


--
-- Name: idx_invitations_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_invitations_token ON public.invitations USING btree (invitation_token);


--
-- Name: idx_ip_blocklist_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_ip_blocklist_active ON public.ip_blocklist USING btree (ip_address) WHERE (unblocked_at IS NULL);


--
-- Name: idx_ip_blocklist_expiring; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ip_blocklist_expiring ON public.ip_blocklist USING btree (blocked_until) WHERE ((blocked_until IS NOT NULL) AND (unblocked_at IS NULL));


--
-- Name: idx_ip_blocklist_recent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ip_blocklist_recent ON public.ip_blocklist USING btree (blocked_at DESC);


--
-- Name: idx_journal_entries_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_journal_entries_date ON public.journal_entries USING btree (entry_date);


--
-- Name: idx_journal_entry_lines_account_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_journal_entry_lines_account_id ON public.journal_entry_lines USING btree (account_id);


--
-- Name: idx_journal_entry_lines_entry; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_journal_entry_lines_entry ON public.journal_entry_lines USING btree (journal_entry_id);


--
-- Name: idx_jwt_blacklist_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_jwt_blacklist_expires ON public.jwt_blacklist USING btree (expires_at);


--
-- Name: idx_leave_balance_usage_comp_time_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_balance_usage_comp_time_id ON public.leave_balance_usage USING btree (comp_time_balance_id);


--
-- Name: idx_leave_balance_usage_entitlement_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_balance_usage_entitlement_id ON public.leave_balance_usage USING btree (annual_leave_entitlement_id);


--
-- Name: idx_leave_date_range; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_date_range ON public.leave_requests USING btree (start_date, end_date);


--
-- Name: idx_leave_requests_annual_leave_source_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_requests_annual_leave_source_id ON public.leave_requests USING btree (annual_leave_source_id);


--
-- Name: idx_leave_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_status ON public.leave_requests USING btree (status);


--
-- Name: idx_leave_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_leave_user ON public.leave_requests USING btree (user_id, start_date DESC);


--
-- Name: idx_line_shelf_alloc_line; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_line_shelf_alloc_line ON public.line_shelf_allocations USING btree (document_line_id) WHERE (document_line_id IS NOT NULL);


--
-- Name: idx_line_shelf_alloc_loc_prod; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_line_shelf_alloc_loc_prod ON public.line_shelf_allocations USING btree (storage_location_id, product_id);


--
-- Name: idx_login_created_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_login_created_type ON public.login_events USING btree (created_at, event_type);


--
-- Name: idx_login_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_login_email ON public.login_events USING btree (email, created_at DESC);


--
-- Name: idx_login_email_type_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_login_email_type_created ON public.login_events USING btree (email, event_type, created_at DESC);


--
-- Name: idx_login_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_login_ip ON public.login_events USING btree (ip_address, created_at DESC);


--
-- Name: idx_login_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_login_type ON public.login_events USING btree (event_type, created_at DESC);


--
-- Name: idx_login_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_login_user ON public.login_events USING btree (user_id, created_at DESC);


--
-- Name: idx_message_attachments_message; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_attachments_message ON public.message_attachments USING btree (message_id) WHERE (message_id IS NOT NULL);


--
-- Name: idx_message_attachments_orphan; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_attachments_orphan ON public.message_attachments USING btree (uploaded_by, created_at) WHERE (message_id IS NULL);


--
-- Name: idx_message_thread_participants_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_thread_participants_user ON public.message_thread_participants USING btree (user_id, last_read_at) WHERE (left_at IS NULL);


--
-- Name: idx_message_threads_last_message_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_message_threads_last_message_at ON public.message_threads USING btree (last_message_at DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_messages_deleted_for_gc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_deleted_for_gc ON public.messages USING btree (deleted_at) WHERE (deleted_at IS NOT NULL);


--
-- Name: idx_messages_thread_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_messages_thread_created ON public.messages USING btree (thread_id, created_at DESC) WHERE (deleted_at IS NULL);


--
-- Name: idx_mgmt_reviews_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mgmt_reviews_date ON public.management_reviews USING btree (review_date);


--
-- Name: idx_mgmt_reviews_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_mgmt_reviews_status ON public.management_reviews USING btree (status);


--
-- Name: idx_notice_ack_notice; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notice_ack_notice ON public.protocol_notice_acknowledgements USING btree (notice_id);


--
-- Name: idx_notification_routing_event; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notification_routing_event ON public.notification_routing USING btree (event_type, is_active);


--
-- Name: idx_notification_routing_event_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notification_routing_event_target ON public.notification_routing USING btree (event_type, is_active, target_kind);


--
-- Name: idx_notifications_action_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_action_pending ON public.notifications USING btree (user_id, created_at DESC) WHERE ((kind = 'action'::text) AND (priority > 0));


--
-- Name: idx_notifications_created_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_created_at ON public.notifications USING btree (created_at);


--
-- Name: idx_notifications_is_read; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_is_read ON public.notifications USING btree (user_id, is_read);


--
-- Name: idx_notifications_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_type ON public.notifications USING btree (type);


--
-- Name: idx_notifications_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_user_id ON public.notifications USING btree (user_id);


--
-- Name: idx_notifications_user_priority_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_user_priority_created ON public.notifications USING btree (user_id, priority DESC, created_at DESC);


--
-- Name: idx_notifications_user_read_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_notifications_user_read_created ON public.notifications USING btree (user_id, is_read, created_at DESC);


--
-- Name: idx_overtime_approval_approver; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_approval_approver ON public.overtime_approvals USING btree (approver_id, created_at DESC);


--
-- Name: idx_overtime_approval_record; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_approval_record ON public.overtime_approvals USING btree (overtime_record_id, created_at);


--
-- Name: idx_overtime_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_expires ON public.overtime_records USING btree (comp_time_expires_at) WHERE (((status)::text = 'approved'::text) AND (comp_time_used_hours < comp_time_hours));


--
-- Name: idx_overtime_records_attendance_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_records_attendance_id ON public.overtime_records USING btree (attendance_id);


--
-- Name: idx_overtime_records_no_duplicate; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_overtime_records_no_duplicate ON public.overtime_records USING btree (user_id, overtime_date, start_time, end_time) WHERE ((status)::text <> ALL ((ARRAY['rejected'::character varying, 'voided'::character varying])::text[]));


--
-- Name: INDEX idx_overtime_records_no_duplicate; Type: COMMENT; Schema: public; Owner: -
--

COMMENT ON INDEX public.idx_overtime_records_no_duplicate IS 'R86-2 防重：同一人同一天同一起訖時間只能有一筆有效加班單（駁回/作廢的不佔位）。';


--
-- Name: idx_overtime_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_status ON public.overtime_records USING btree (status);


--
-- Name: idx_overtime_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_overtime_user ON public.overtime_records USING btree (user_id, overtime_date DESC);


--
-- Name: idx_partners_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_partners_code ON public.partners USING btree (code);


--
-- Name: idx_partners_partner_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_partners_partner_type ON public.partners USING btree (partner_type);


--
-- Name: idx_password_reset_tokens_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_reset_tokens_token_hash ON public.password_reset_tokens USING btree (token_hash);


--
-- Name: idx_password_reset_tokens_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_password_reset_tokens_user_id ON public.password_reset_tokens USING btree (user_id);


--
-- Name: idx_pdf_artifacts_doc_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pdf_artifacts_doc_type ON public.pdf_artifacts USING btree (doc_type);


--
-- Name: idx_pdf_artifacts_generated_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pdf_artifacts_generated_at ON public.pdf_artifacts USING btree (generated_at DESC);


--
-- Name: idx_pdf_artifacts_generated_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pdf_artifacts_generated_by ON public.pdf_artifacts USING btree (generated_by);


--
-- Name: idx_pdf_artifacts_resource_time; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pdf_artifacts_resource_time ON public.pdf_artifacts USING btree (resource_type, resource_id, generated_at DESC);


--
-- Name: idx_pdf_artifacts_signature; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pdf_artifacts_signature ON public.pdf_artifacts USING btree (electronic_signature_id) WHERE (electronic_signature_id IS NOT NULL);


--
-- Name: idx_pens_zone_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pens_zone_id ON public.pens USING btree (zone_id);


--
-- Name: idx_pi_account_invites_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_pi_account_invites_status ON public.pi_account_invites USING btree (status, created_at DESC);


--
-- Name: idx_planned_experiments_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_planned_experiments_protocol ON public.planned_experiments USING btree (protocol_id) WHERE (protocol_id IS NOT NULL);


--
-- Name: idx_product_categories_parent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_categories_parent_id ON public.product_categories USING btree (parent_id);


--
-- Name: idx_product_uom_conversions_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_product_uom_conversions_product_id ON public.product_uom_conversions USING btree (product_id);


--
-- Name: idx_products_category_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_category_id ON public.products USING btree (category_id);


--
-- Name: idx_products_selling_price_nonnull; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_selling_price_nonnull ON public.products USING btree (id) WHERE (selling_price IS NOT NULL);


--
-- Name: idx_products_sku; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_sku ON public.products USING btree (sku);


--
-- Name: idx_products_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_products_status ON public.products USING btree (status);


--
-- Name: idx_protocol_activities_actor_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_activities_actor_id ON public.protocol_activities USING btree (actor_id);


--
-- Name: idx_protocol_activities_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_activities_protocol_id ON public.protocol_activities USING btree (protocol_id, created_at DESC);


--
-- Name: idx_protocol_activities_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_activities_type ON public.protocol_activities USING btree (activity_type, created_at DESC);


--
-- Name: idx_protocol_attachments_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_attachments_protocol_id ON public.protocol_attachments USING btree (protocol_id);


--
-- Name: idx_protocol_attachments_protocol_version_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_attachments_protocol_version_id ON public.protocol_attachments USING btree (protocol_version_id);


--
-- Name: idx_protocol_template_versions_current; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_protocol_template_versions_current ON public.protocol_template_versions USING btree (is_current) WHERE (is_current = true);


--
-- Name: idx_protocol_versions_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocol_versions_protocol_id ON public.protocol_versions USING btree (protocol_id);


--
-- Name: idx_protocols_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocols_created_by ON public.protocols USING btree (created_by);


--
-- Name: idx_protocols_iacuc_no; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocols_iacuc_no ON public.protocols USING btree (iacuc_no);


--
-- Name: idx_protocols_pi_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocols_pi_user_id ON public.protocols USING btree (pi_user_id);


--
-- Name: idx_protocols_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocols_status ON public.protocols USING btree (status);


--
-- Name: idx_protocols_status_pi_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocols_status_pi_created ON public.protocols USING btree (status, pi_user_id, created_at DESC);


--
-- Name: idx_protocols_working_content; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_protocols_working_content ON public.protocols USING gin (working_content);


--
-- Name: idx_qa_capa_nc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_capa_nc ON public.qa_capa USING btree (nc_id);


--
-- Name: idx_qa_inspection_items_insp; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_inspection_items_insp ON public.qa_inspection_items USING btree (inspection_id);


--
-- Name: idx_qa_inspections_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_inspections_date ON public.qa_inspections USING btree (inspection_date);


--
-- Name: idx_qa_inspections_inspector; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_inspections_inspector ON public.qa_inspections USING btree (inspector_id);


--
-- Name: idx_qa_inspections_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_inspections_status ON public.qa_inspections USING btree (status);


--
-- Name: idx_qa_nc_assignee; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_nc_assignee ON public.qa_non_conformances USING btree (assignee_id);


--
-- Name: idx_qa_nc_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_nc_created_by ON public.qa_non_conformances USING btree (created_by);


--
-- Name: idx_qa_nc_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_nc_status ON public.qa_non_conformances USING btree (status);


--
-- Name: idx_qa_schedule_items_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_schedule_items_date ON public.qa_schedule_items USING btree (planned_date);


--
-- Name: idx_qa_schedule_items_sched; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_schedule_items_sched ON public.qa_schedule_items USING btree (schedule_id);


--
-- Name: idx_qa_schedules_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_schedules_status ON public.qa_audit_schedules USING btree (status);


--
-- Name: idx_qa_schedules_year; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_schedules_year ON public.qa_audit_schedules USING btree (year);


--
-- Name: idx_qa_sop_ack_sop; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_sop_ack_sop ON public.qa_sop_acknowledgments USING btree (sop_id);


--
-- Name: idx_qa_sop_ack_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_sop_ack_user ON public.qa_sop_acknowledgments USING btree (user_id);


--
-- Name: idx_qa_sop_created_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_sop_created_by ON public.qa_sop_documents USING btree (created_by);


--
-- Name: idx_qa_sop_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_qa_sop_status ON public.qa_sop_documents USING btree (status);


--
-- Name: idx_record_versions_record; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_record_versions_record ON public.record_versions USING btree (record_type, record_id);


--
-- Name: idx_reference_standards_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_standards_status ON public.reference_standards USING btree (status);


--
-- Name: idx_reference_standards_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_reference_standards_type ON public.reference_standards USING btree (standard_type);


--
-- Name: idx_refresh_tokens_family_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_tokens_family_id ON public.refresh_tokens USING btree (family_id);


--
-- Name: idx_refresh_tokens_token_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_tokens_token_hash ON public.refresh_tokens USING btree (token_hash);


--
-- Name: idx_refresh_tokens_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_refresh_tokens_user_id ON public.refresh_tokens USING btree (user_id);


--
-- Name: idx_report_history_generated_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_report_history_generated_at ON public.report_history USING btree (generated_at);


--
-- Name: idx_report_history_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_report_history_type ON public.report_history USING btree (report_type);


--
-- Name: idx_review_assignments_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_assignments_protocol_id ON public.review_assignments USING btree (protocol_id);


--
-- Name: idx_review_assignments_reviewer_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_assignments_reviewer_id ON public.review_assignments USING btree (reviewer_id);


--
-- Name: idx_review_comments_drafted_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_comments_drafted_by ON public.review_comments USING btree (drafted_by);


--
-- Name: idx_review_comments_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_comments_parent ON public.review_comments USING btree (parent_comment_id);


--
-- Name: idx_review_comments_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_comments_protocol_id ON public.review_comments USING btree (protocol_id);


--
-- Name: idx_review_comments_protocol_version_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_comments_protocol_version_id ON public.review_comments USING btree (protocol_version_id);


--
-- Name: idx_review_comments_review_stage; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_comments_review_stage ON public.review_comments USING btree (review_stage);


--
-- Name: idx_review_comments_reviewer_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_comments_reviewer_id ON public.review_comments USING btree (reviewer_id);


--
-- Name: idx_review_round_history_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_round_history_protocol ON public.review_round_history USING btree (protocol_id);


--
-- Name: idx_review_round_history_stage; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_review_round_history_stage ON public.review_round_history USING btree (review_stage);


--
-- Name: idx_risk_register_owner; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_risk_register_owner ON public.risk_register USING btree (owner_id);


--
-- Name: idx_risk_register_score; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_risk_register_score ON public.risk_register USING btree (risk_score);


--
-- Name: idx_risk_register_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_risk_register_status ON public.risk_register USING btree (status);


--
-- Name: idx_scheduled_reports_next_run; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_scheduled_reports_next_run ON public.scheduled_reports USING btree (next_run_at) WHERE (is_active = true);


--
-- Name: idx_security_alerts_context_data; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_alerts_context_data ON public.security_alerts USING gin (context_data);


--
-- Name: idx_security_alerts_login_event_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_alerts_login_event_id ON public.security_alerts USING btree (login_event_id);


--
-- Name: idx_security_alerts_status_created; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_alerts_status_created ON public.security_alerts USING btree (status, created_at) WHERE ((status)::text = 'open'::text);


--
-- Name: idx_security_alerts_sweep; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_alerts_sweep ON public.security_alerts USING btree (status, created_at, last_notified_at) WHERE ((status)::text = 'open'::text);


--
-- Name: idx_security_alerts_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_alerts_user_id ON public.security_alerts USING btree (user_id);


--
-- Name: idx_security_notification_channels_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_security_notification_channels_enabled ON public.security_notification_channels USING btree (is_enabled) WHERE (is_enabled = true);


--
-- Name: idx_sessions_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_active ON public.user_sessions USING btree (is_active, last_activity_at DESC) WHERE (is_active = true);


--
-- Name: idx_sessions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sessions_user ON public.user_sessions USING btree (user_id, started_at DESC);


--
-- Name: idx_signature_bridge_sessions_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_signature_bridge_sessions_expires_at ON public.signature_bridge_sessions USING btree (expires_at) WHERE ((status)::text = 'PENDING'::text);


--
-- Name: idx_signature_bridge_sessions_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_signature_bridge_sessions_user_id ON public.signature_bridge_sessions USING btree (user_id);


--
-- Name: idx_species_parent_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_species_parent_id ON public.species USING btree (parent_id);


--
-- Name: idx_stock_ledger_doc_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_doc_id ON public.stock_ledger USING btree (doc_id);


--
-- Name: idx_stock_ledger_line_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_line_id ON public.stock_ledger USING btree (line_id);


--
-- Name: idx_stock_ledger_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_product_id ON public.stock_ledger USING btree (product_id);


--
-- Name: idx_stock_ledger_storage_prod_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_storage_prod_date ON public.stock_ledger USING btree (storage_location_id, product_id, trx_date DESC) WHERE (storage_location_id IS NOT NULL);


--
-- Name: idx_stock_ledger_trx_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_trx_date ON public.stock_ledger USING btree (trx_date);


--
-- Name: idx_stock_ledger_warehouse_product; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_warehouse_product ON public.stock_ledger USING btree (warehouse_id, product_id);


--
-- Name: idx_stock_ledger_wh_prod_date; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_wh_prod_date ON public.stock_ledger USING btree (warehouse_id, product_id, trx_date DESC);


--
-- Name: idx_stock_ledger_wh_prod_dir; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_stock_ledger_wh_prod_dir ON public.stock_ledger USING btree (warehouse_id, product_id, direction);


--
-- Name: idx_storage_location_inventory_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_storage_location_inventory_product_id ON public.storage_location_inventory USING btree (product_id);


--
-- Name: idx_storage_location_inventory_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_storage_location_inventory_unique ON public.storage_location_inventory USING btree (storage_location_id, product_id, COALESCE(batch_no, ''::character varying), COALESCE(expiry_date, '1900-01-01'::date));


--
-- Name: idx_storage_locations_warehouse; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_storage_locations_warehouse ON public.storage_locations USING btree (warehouse_id);


--
-- Name: idx_sudden_deaths_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sudden_deaths_animal ON public.animal_sudden_deaths USING btree (animal_id);


--
-- Name: idx_sudden_deaths_discovered_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_sudden_deaths_discovered_by ON public.animal_sudden_deaths USING btree (discovered_by);


--
-- Name: idx_training_records_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_training_records_expires ON public.training_records USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_training_records_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_training_records_user ON public.training_records USING btree (user_id);


--
-- Name: idx_training_requirements_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_training_requirements_active ON public.role_training_requirements USING btree (role_code, training_topic) WHERE (deleted_at IS NULL);


--
-- Name: idx_transfer_vet_eval; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_transfer_vet_eval ON public.transfer_vet_evaluations USING btree (transfer_id);


--
-- Name: idx_transfers_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_transfers_animal ON public.animal_transfers USING btree (animal_id);


--
-- Name: idx_transfers_from_iacuc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_transfers_from_iacuc ON public.animal_transfers USING btree (from_iacuc_no);


--
-- Name: idx_transfers_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_transfers_status ON public.animal_transfers USING btree (status);


--
-- Name: idx_transfers_to_iacuc; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_transfers_to_iacuc ON public.animal_transfers USING btree (to_iacuc_no);


--
-- Name: idx_treatment_drug_options_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_treatment_drug_options_active ON public.treatment_drug_options USING btree (is_active);


--
-- Name: idx_treatment_drug_options_business_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_treatment_drug_options_business_key ON public.treatment_drug_options USING btree (lower(TRIM(BOTH FROM name)), COALESCE(category, ''::character varying)) WHERE (is_active = true);


--
-- Name: idx_treatment_drug_options_product_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_treatment_drug_options_product_id ON public.treatment_drug_options USING btree (erp_product_id);


--
-- Name: idx_ual_after_data_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ual_after_data_gin ON ONLY public.user_activity_logs USING gin (after_data jsonb_path_ops);


--
-- Name: idx_ual_before_data_gin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_ual_before_data_gin ON ONLY public.user_activity_logs USING gin (before_data jsonb_path_ops);


--
-- Name: idx_usage_request; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_usage_request ON public.leave_balance_usage USING btree (leave_request_id);


--
-- Name: idx_user_activity_logs_hmac_version; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_activity_logs_hmac_version ON ONLY public.user_activity_logs USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: idx_user_activity_logs_impersonated_by; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_activity_logs_impersonated_by ON ONLY public.user_activity_logs USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: idx_user_mcp_keys_hash; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_mcp_keys_hash ON public.user_mcp_keys USING btree (key_hash) WHERE (revoked_at IS NULL);


--
-- Name: idx_user_mcp_keys_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_mcp_keys_user_id ON public.user_mcp_keys USING btree (user_id) WHERE (revoked_at IS NULL);


--
-- Name: idx_user_preferences_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_preferences_user_id ON public.user_preferences USING btree (user_id);


--
-- Name: idx_user_protocols_protocol_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_protocols_protocol_id ON public.user_protocols USING btree (protocol_id);


--
-- Name: idx_user_protocols_protocol_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_protocols_protocol_user ON public.user_protocols USING btree (protocol_id, user_id);


--
-- Name: idx_user_protocols_user_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_protocols_user_id ON public.user_protocols USING btree (user_id);


--
-- Name: idx_user_protocols_user_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_protocols_user_protocol ON public.user_protocols USING btree (user_id, protocol_id);


--
-- Name: idx_user_roles_role_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_roles_role_id ON public.user_roles USING btree (role_id);


--
-- Name: idx_user_sessions_refresh_token_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_user_sessions_refresh_token_id ON public.user_sessions USING btree (refresh_token_id);


--
-- Name: idx_users_department_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_department_id ON public.users USING btree (department_id) WHERE (department_id IS NOT NULL);


--
-- Name: idx_users_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_email ON public.users USING btree (email);


--
-- Name: idx_users_expires_at; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_expires_at ON public.users USING btree (expires_at) WHERE (expires_at IS NOT NULL);


--
-- Name: idx_users_is_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_users_is_active ON public.users USING btree (is_active);


--
-- Name: idx_vet_advice_records_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_advice_records_animal ON public.animal_vet_advice_records USING btree (animal_id);


--
-- Name: idx_vet_patrol_entries_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_entries_animal ON public.vet_patrol_entries USING btree (animal_id);


--
-- Name: idx_vet_patrol_entries_report; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_entries_report ON public.vet_patrol_entries USING btree (report_id);


--
-- Name: idx_vet_patrol_entry_animals_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_entry_animals_animal ON public.vet_patrol_entry_animals USING btree (animal_id);


--
-- Name: idx_vet_patrol_entry_photos_entry; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_entry_photos_entry ON public.vet_patrol_entry_photos USING btree (entry_id);


--
-- Name: idx_vet_patrol_photos_report; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_photos_report ON public.vet_patrol_photos USING btree (report_id);


--
-- Name: idx_vet_patrol_reports_draft_updated; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_reports_draft_updated ON public.vet_patrol_reports USING btree (updated_at) WHERE (((status)::text = 'draft'::text) AND (deleted_at IS NULL));


--
-- Name: idx_vet_patrol_reports_follow_up_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_reports_follow_up_pending ON public.vet_patrol_reports USING btree (follow_up_user_id) WHERE (((status)::text = 'awaiting_follow_up'::text) AND (deleted_at IS NULL));


--
-- Name: idx_vet_patrol_reports_pending_ack; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_patrol_reports_pending_ack ON public.vet_patrol_reports USING btree (follow_up_user_id) WHERE (((status)::text = 'awaiting_acknowledgement'::text) AND (deleted_at IS NULL));


--
-- Name: idx_vet_review_assignments_protocol; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_review_assignments_protocol ON public.vet_review_assignments USING btree (protocol_id);


--
-- Name: idx_vet_review_assignments_vet; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_vet_review_assignments_vet ON public.vet_review_assignments USING btree (vet_id);


--
-- Name: idx_warehouses_code; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_warehouses_code ON public.warehouses USING btree (code);


--
-- Name: idx_zones_building_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX idx_zones_building_id ON public.zones USING btree (building_id);


--
-- Name: uq_buildings_facility_code_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_buildings_facility_code_active ON public.buildings USING btree (facility_id, code) WHERE (is_active = true);


--
-- Name: uq_expiry_snapshot_key; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_expiry_snapshot_key ON public.expiry_monthly_snapshots USING btree (snapshot_ym, product_id, warehouse_id, COALESCE(batch_no, ''::character varying));


--
-- Name: uq_facilities_code_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_facilities_code_active ON public.facilities USING btree (code) WHERE (is_active = true);


--
-- Name: uq_pens_zone_code_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_pens_zone_code_active ON public.pens USING btree (zone_id, code) WHERE (is_active = true);


--
-- Name: uq_pi_account_invites_protocol_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_pi_account_invites_protocol_pending ON public.pi_account_invites USING btree (protocol_id) WHERE ((status)::text = 'pending'::text);


--
-- Name: uq_species_code_lower; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_species_code_lower ON public.species USING btree (lower((code)::text));


--
-- Name: uq_zones_building_code_active; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX uq_zones_building_code_active ON public.zones USING btree (building_id, code) WHERE (is_active = true);


--
-- Name: user_activity_logs_2026_q1_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_actor_user_id_created_at_idx ON public.user_activity_logs_2026_q1 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_actor_user_id_created_at_idx1 ON public.user_activity_logs_2026_q1 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_after_data_idx ON public.user_activity_logs_2026_q1 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q1_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_before_data_idx ON public.user_activity_logs_2026_q1 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q1_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_created_at_integrity_hash_idx ON public.user_activity_logs_2026_q1 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2026_q1_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2026_q1 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2026_q1 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_event_category_created_at_idx ON public.user_activity_logs_2026_q1 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_event_type_created_at_idx ON public.user_activity_logs_2026_q1 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_hmac_version_idx ON public.user_activity_logs_2026_q1 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2026_q1_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_id_idx ON public.user_activity_logs_2026_q1 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2026_q1_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_impersonated_by_user_id_created__idx ON public.user_activity_logs_2026_q1 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2026_q1_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_ip_address_created_at_idx ON public.user_activity_logs_2026_q1 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2026_q1_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_is_suspicious_idx ON public.user_activity_logs_2026_q1 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2026_q1_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q1_partition_date_created_at_idx ON public.user_activity_logs_2026_q1 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_actor_user_id_created_at_idx ON public.user_activity_logs_2026_q2 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_actor_user_id_created_at_idx1 ON public.user_activity_logs_2026_q2 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_after_data_idx ON public.user_activity_logs_2026_q2 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q2_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_before_data_idx ON public.user_activity_logs_2026_q2 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q2_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_created_at_integrity_hash_idx ON public.user_activity_logs_2026_q2 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2026_q2_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2026_q2 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2026_q2 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_event_category_created_at_idx ON public.user_activity_logs_2026_q2 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_event_type_created_at_idx ON public.user_activity_logs_2026_q2 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_hmac_version_idx ON public.user_activity_logs_2026_q2 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2026_q2_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_id_idx ON public.user_activity_logs_2026_q2 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2026_q2_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_impersonated_by_user_id_created__idx ON public.user_activity_logs_2026_q2 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2026_q2_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_ip_address_created_at_idx ON public.user_activity_logs_2026_q2 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2026_q2_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_is_suspicious_idx ON public.user_activity_logs_2026_q2 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2026_q2_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q2_partition_date_created_at_idx ON public.user_activity_logs_2026_q2 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_actor_user_id_created_at_idx ON public.user_activity_logs_2026_q3 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_actor_user_id_created_at_idx1 ON public.user_activity_logs_2026_q3 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_after_data_idx ON public.user_activity_logs_2026_q3 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q3_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_before_data_idx ON public.user_activity_logs_2026_q3 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q3_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_created_at_integrity_hash_idx ON public.user_activity_logs_2026_q3 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2026_q3_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2026_q3 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2026_q3 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_event_category_created_at_idx ON public.user_activity_logs_2026_q3 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_event_type_created_at_idx ON public.user_activity_logs_2026_q3 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_hmac_version_idx ON public.user_activity_logs_2026_q3 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2026_q3_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_id_idx ON public.user_activity_logs_2026_q3 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2026_q3_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_impersonated_by_user_id_created__idx ON public.user_activity_logs_2026_q3 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2026_q3_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_ip_address_created_at_idx ON public.user_activity_logs_2026_q3 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2026_q3_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_is_suspicious_idx ON public.user_activity_logs_2026_q3 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2026_q3_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q3_partition_date_created_at_idx ON public.user_activity_logs_2026_q3 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_actor_user_id_created_at_idx ON public.user_activity_logs_2026_q4 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_actor_user_id_created_at_idx1 ON public.user_activity_logs_2026_q4 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_after_data_idx ON public.user_activity_logs_2026_q4 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q4_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_before_data_idx ON public.user_activity_logs_2026_q4 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2026_q4_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_created_at_integrity_hash_idx ON public.user_activity_logs_2026_q4 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2026_q4_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2026_q4 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2026_q4 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_event_category_created_at_idx ON public.user_activity_logs_2026_q4 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_event_type_created_at_idx ON public.user_activity_logs_2026_q4 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_hmac_version_idx ON public.user_activity_logs_2026_q4 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2026_q4_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_id_idx ON public.user_activity_logs_2026_q4 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2026_q4_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_impersonated_by_user_id_created__idx ON public.user_activity_logs_2026_q4 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2026_q4_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_ip_address_created_at_idx ON public.user_activity_logs_2026_q4 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2026_q4_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_is_suspicious_idx ON public.user_activity_logs_2026_q4 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2026_q4_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2026_q4_partition_date_created_at_idx ON public.user_activity_logs_2026_q4 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_actor_user_id_created_at_idx ON public.user_activity_logs_2027_q1 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_actor_user_id_created_at_idx1 ON public.user_activity_logs_2027_q1 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_after_data_idx ON public.user_activity_logs_2027_q1 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q1_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_before_data_idx ON public.user_activity_logs_2027_q1 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q1_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_created_at_integrity_hash_idx ON public.user_activity_logs_2027_q1 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2027_q1_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2027_q1 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2027_q1 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_event_category_created_at_idx ON public.user_activity_logs_2027_q1 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_event_type_created_at_idx ON public.user_activity_logs_2027_q1 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_hmac_version_idx ON public.user_activity_logs_2027_q1 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2027_q1_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_id_idx ON public.user_activity_logs_2027_q1 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2027_q1_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_impersonated_by_user_id_created__idx ON public.user_activity_logs_2027_q1 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2027_q1_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_ip_address_created_at_idx ON public.user_activity_logs_2027_q1 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2027_q1_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_is_suspicious_idx ON public.user_activity_logs_2027_q1 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2027_q1_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q1_partition_date_created_at_idx ON public.user_activity_logs_2027_q1 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_actor_user_id_created_at_idx ON public.user_activity_logs_2027_q2 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_actor_user_id_created_at_idx1 ON public.user_activity_logs_2027_q2 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_after_data_idx ON public.user_activity_logs_2027_q2 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q2_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_before_data_idx ON public.user_activity_logs_2027_q2 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q2_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_created_at_integrity_hash_idx ON public.user_activity_logs_2027_q2 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2027_q2_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2027_q2 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2027_q2 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_event_category_created_at_idx ON public.user_activity_logs_2027_q2 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_event_type_created_at_idx ON public.user_activity_logs_2027_q2 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_hmac_version_idx ON public.user_activity_logs_2027_q2 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2027_q2_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_id_idx ON public.user_activity_logs_2027_q2 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2027_q2_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_impersonated_by_user_id_created__idx ON public.user_activity_logs_2027_q2 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2027_q2_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_ip_address_created_at_idx ON public.user_activity_logs_2027_q2 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2027_q2_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_is_suspicious_idx ON public.user_activity_logs_2027_q2 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2027_q2_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q2_partition_date_created_at_idx ON public.user_activity_logs_2027_q2 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_actor_user_id_created_at_idx ON public.user_activity_logs_2027_q3 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_actor_user_id_created_at_idx1 ON public.user_activity_logs_2027_q3 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_after_data_idx ON public.user_activity_logs_2027_q3 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q3_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_before_data_idx ON public.user_activity_logs_2027_q3 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q3_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_created_at_integrity_hash_idx ON public.user_activity_logs_2027_q3 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2027_q3_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2027_q3 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2027_q3 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_event_category_created_at_idx ON public.user_activity_logs_2027_q3 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_event_type_created_at_idx ON public.user_activity_logs_2027_q3 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_hmac_version_idx ON public.user_activity_logs_2027_q3 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2027_q3_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_id_idx ON public.user_activity_logs_2027_q3 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2027_q3_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_impersonated_by_user_id_created__idx ON public.user_activity_logs_2027_q3 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2027_q3_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_ip_address_created_at_idx ON public.user_activity_logs_2027_q3 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2027_q3_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_is_suspicious_idx ON public.user_activity_logs_2027_q3 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2027_q3_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q3_partition_date_created_at_idx ON public.user_activity_logs_2027_q3 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_actor_user_id_created_at_idx ON public.user_activity_logs_2027_q4 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_actor_user_id_created_at_idx1 ON public.user_activity_logs_2027_q4 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_after_data_idx ON public.user_activity_logs_2027_q4 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q4_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_before_data_idx ON public.user_activity_logs_2027_q4 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2027_q4_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_created_at_integrity_hash_idx ON public.user_activity_logs_2027_q4 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2027_q4_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2027_q4 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2027_q4 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_event_category_created_at_idx ON public.user_activity_logs_2027_q4 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_event_type_created_at_idx ON public.user_activity_logs_2027_q4 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_hmac_version_idx ON public.user_activity_logs_2027_q4 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2027_q4_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_id_idx ON public.user_activity_logs_2027_q4 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2027_q4_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_impersonated_by_user_id_created__idx ON public.user_activity_logs_2027_q4 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2027_q4_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_ip_address_created_at_idx ON public.user_activity_logs_2027_q4 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2027_q4_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_is_suspicious_idx ON public.user_activity_logs_2027_q4 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2027_q4_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2027_q4_partition_date_created_at_idx ON public.user_activity_logs_2027_q4 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_actor_user_id_created_at_idx ON public.user_activity_logs_2028_q1 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_actor_user_id_created_at_idx1 ON public.user_activity_logs_2028_q1 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_after_data_idx ON public.user_activity_logs_2028_q1 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q1_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_before_data_idx ON public.user_activity_logs_2028_q1 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q1_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_created_at_integrity_hash_idx ON public.user_activity_logs_2028_q1 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2028_q1_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2028_q1 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2028_q1 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_event_category_created_at_idx ON public.user_activity_logs_2028_q1 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_event_type_created_at_idx ON public.user_activity_logs_2028_q1 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_hmac_version_idx ON public.user_activity_logs_2028_q1 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2028_q1_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_id_idx ON public.user_activity_logs_2028_q1 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2028_q1_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_impersonated_by_user_id_created__idx ON public.user_activity_logs_2028_q1 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2028_q1_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_ip_address_created_at_idx ON public.user_activity_logs_2028_q1 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2028_q1_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_is_suspicious_idx ON public.user_activity_logs_2028_q1 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2028_q1_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q1_partition_date_created_at_idx ON public.user_activity_logs_2028_q1 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_actor_user_id_created_at_idx ON public.user_activity_logs_2028_q2 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_actor_user_id_created_at_idx1 ON public.user_activity_logs_2028_q2 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_after_data_idx ON public.user_activity_logs_2028_q2 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q2_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_before_data_idx ON public.user_activity_logs_2028_q2 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q2_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_created_at_integrity_hash_idx ON public.user_activity_logs_2028_q2 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2028_q2_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2028_q2 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2028_q2 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_event_category_created_at_idx ON public.user_activity_logs_2028_q2 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_event_type_created_at_idx ON public.user_activity_logs_2028_q2 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_hmac_version_idx ON public.user_activity_logs_2028_q2 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2028_q2_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_id_idx ON public.user_activity_logs_2028_q2 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2028_q2_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_impersonated_by_user_id_created__idx ON public.user_activity_logs_2028_q2 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2028_q2_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_ip_address_created_at_idx ON public.user_activity_logs_2028_q2 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2028_q2_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_is_suspicious_idx ON public.user_activity_logs_2028_q2 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2028_q2_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q2_partition_date_created_at_idx ON public.user_activity_logs_2028_q2 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_actor_user_id_created_at_idx ON public.user_activity_logs_2028_q3 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_actor_user_id_created_at_idx1 ON public.user_activity_logs_2028_q3 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_after_data_idx ON public.user_activity_logs_2028_q3 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q3_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_before_data_idx ON public.user_activity_logs_2028_q3 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q3_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_created_at_integrity_hash_idx ON public.user_activity_logs_2028_q3 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2028_q3_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2028_q3 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2028_q3 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_event_category_created_at_idx ON public.user_activity_logs_2028_q3 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_event_type_created_at_idx ON public.user_activity_logs_2028_q3 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_hmac_version_idx ON public.user_activity_logs_2028_q3 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2028_q3_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_id_idx ON public.user_activity_logs_2028_q3 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2028_q3_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_impersonated_by_user_id_created__idx ON public.user_activity_logs_2028_q3 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2028_q3_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_ip_address_created_at_idx ON public.user_activity_logs_2028_q3 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2028_q3_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_is_suspicious_idx ON public.user_activity_logs_2028_q3 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2028_q3_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q3_partition_date_created_at_idx ON public.user_activity_logs_2028_q3 USING btree (partition_date, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_actor_user_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_actor_user_id_created_at_idx ON public.user_activity_logs_2028_q4 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_actor_user_id_created_at_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_actor_user_id_created_at_idx1 ON public.user_activity_logs_2028_q4 USING btree (actor_user_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_after_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_after_data_idx ON public.user_activity_logs_2028_q4 USING gin (after_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q4_before_data_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_before_data_idx ON public.user_activity_logs_2028_q4 USING gin (before_data jsonb_path_ops);


--
-- Name: user_activity_logs_2028_q4_created_at_integrity_hash_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_created_at_integrity_hash_idx ON public.user_activity_logs_2028_q4 USING btree (created_at, integrity_hash);


--
-- Name: user_activity_logs_2028_q4_entity_type_entity_id_created_a_idx1; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_entity_type_entity_id_created_a_idx1 ON public.user_activity_logs_2028_q4 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_entity_type_entity_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_entity_type_entity_id_created_at_idx ON public.user_activity_logs_2028_q4 USING btree (entity_type, entity_id, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_event_category_created_at_idx ON public.user_activity_logs_2028_q4 USING btree (event_category, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_event_type_created_at_idx ON public.user_activity_logs_2028_q4 USING btree (event_type, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_hmac_version_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_hmac_version_idx ON public.user_activity_logs_2028_q4 USING btree (hmac_version) WHERE (integrity_hash IS NOT NULL);


--
-- Name: user_activity_logs_2028_q4_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_id_idx ON public.user_activity_logs_2028_q4 USING btree (id) WHERE ((hmac_version IS NULL) AND (integrity_hash IS NOT NULL));


--
-- Name: user_activity_logs_2028_q4_impersonated_by_user_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_impersonated_by_user_id_created__idx ON public.user_activity_logs_2028_q4 USING btree (impersonated_by_user_id, created_at DESC) WHERE (impersonated_by_user_id IS NOT NULL);


--
-- Name: user_activity_logs_2028_q4_ip_address_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_ip_address_created_at_idx ON public.user_activity_logs_2028_q4 USING btree (ip_address, created_at DESC);


--
-- Name: user_activity_logs_2028_q4_is_suspicious_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_is_suspicious_idx ON public.user_activity_logs_2028_q4 USING btree (is_suspicious) WHERE (is_suspicious = true);


--
-- Name: user_activity_logs_2028_q4_partition_date_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_activity_logs_2028_q4_partition_date_created_at_idx ON public.user_activity_logs_2028_q4 USING btree (partition_date, created_at DESC);


--
-- Name: ux_vet_advice_source_entry_animal; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX ux_vet_advice_source_entry_animal ON public.animal_vet_advice_records USING btree (source_vet_patrol_entry_id, animal_id) WHERE (source_vet_patrol_entry_id IS NOT NULL);


--
-- Name: ai_query_logs_2026_08_api_key_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ai_query_logs_api_key ATTACH PARTITION public.ai_query_logs_2026_08_api_key_id_created_at_idx;


--
-- Name: ai_query_logs_2026_08_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.ai_query_logs_pkey ATTACH PARTITION public.ai_query_logs_2026_08_pkey;


--
-- Name: ai_query_logs_2026_09_api_key_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ai_query_logs_api_key ATTACH PARTITION public.ai_query_logs_2026_09_api_key_id_created_at_idx;


--
-- Name: ai_query_logs_2026_09_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.ai_query_logs_pkey ATTACH PARTITION public.ai_query_logs_2026_09_pkey;


--
-- Name: user_activity_logs_2026_q1_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2026_q1_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q1_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2026_q1_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2026_q1_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2026_q1_after_data_idx;


--
-- Name: user_activity_logs_2026_q1_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2026_q1_before_data_idx;


--
-- Name: user_activity_logs_2026_q1_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2026_q1_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2026_q1_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2026_q1_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2026_q1_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2026_q1_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q1_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2026_q1_event_category_created_at_idx;


--
-- Name: user_activity_logs_2026_q1_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2026_q1_event_type_created_at_idx;


--
-- Name: user_activity_logs_2026_q1_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2026_q1_hmac_version_idx;


--
-- Name: user_activity_logs_2026_q1_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2026_q1_id_idx;


--
-- Name: user_activity_logs_2026_q1_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2026_q1_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2026_q1_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2026_q1_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2026_q1_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2026_q1_is_suspicious_idx;


--
-- Name: user_activity_logs_2026_q1_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2026_q1_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2026_q1_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2026_q1_pkey;


--
-- Name: user_activity_logs_2026_q2_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2026_q2_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q2_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2026_q2_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2026_q2_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2026_q2_after_data_idx;


--
-- Name: user_activity_logs_2026_q2_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2026_q2_before_data_idx;


--
-- Name: user_activity_logs_2026_q2_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2026_q2_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2026_q2_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2026_q2_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2026_q2_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2026_q2_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q2_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2026_q2_event_category_created_at_idx;


--
-- Name: user_activity_logs_2026_q2_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2026_q2_event_type_created_at_idx;


--
-- Name: user_activity_logs_2026_q2_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2026_q2_hmac_version_idx;


--
-- Name: user_activity_logs_2026_q2_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2026_q2_id_idx;


--
-- Name: user_activity_logs_2026_q2_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2026_q2_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2026_q2_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2026_q2_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2026_q2_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2026_q2_is_suspicious_idx;


--
-- Name: user_activity_logs_2026_q2_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2026_q2_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2026_q2_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2026_q2_pkey;


--
-- Name: user_activity_logs_2026_q3_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2026_q3_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q3_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2026_q3_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2026_q3_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2026_q3_after_data_idx;


--
-- Name: user_activity_logs_2026_q3_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2026_q3_before_data_idx;


--
-- Name: user_activity_logs_2026_q3_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2026_q3_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2026_q3_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2026_q3_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2026_q3_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2026_q3_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q3_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2026_q3_event_category_created_at_idx;


--
-- Name: user_activity_logs_2026_q3_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2026_q3_event_type_created_at_idx;


--
-- Name: user_activity_logs_2026_q3_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2026_q3_hmac_version_idx;


--
-- Name: user_activity_logs_2026_q3_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2026_q3_id_idx;


--
-- Name: user_activity_logs_2026_q3_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2026_q3_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2026_q3_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2026_q3_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2026_q3_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2026_q3_is_suspicious_idx;


--
-- Name: user_activity_logs_2026_q3_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2026_q3_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2026_q3_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2026_q3_pkey;


--
-- Name: user_activity_logs_2026_q4_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2026_q4_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q4_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2026_q4_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2026_q4_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2026_q4_after_data_idx;


--
-- Name: user_activity_logs_2026_q4_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2026_q4_before_data_idx;


--
-- Name: user_activity_logs_2026_q4_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2026_q4_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2026_q4_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2026_q4_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2026_q4_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2026_q4_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2026_q4_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2026_q4_event_category_created_at_idx;


--
-- Name: user_activity_logs_2026_q4_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2026_q4_event_type_created_at_idx;


--
-- Name: user_activity_logs_2026_q4_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2026_q4_hmac_version_idx;


--
-- Name: user_activity_logs_2026_q4_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2026_q4_id_idx;


--
-- Name: user_activity_logs_2026_q4_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2026_q4_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2026_q4_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2026_q4_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2026_q4_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2026_q4_is_suspicious_idx;


--
-- Name: user_activity_logs_2026_q4_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2026_q4_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2026_q4_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2026_q4_pkey;


--
-- Name: user_activity_logs_2027_q1_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2027_q1_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q1_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2027_q1_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2027_q1_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2027_q1_after_data_idx;


--
-- Name: user_activity_logs_2027_q1_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2027_q1_before_data_idx;


--
-- Name: user_activity_logs_2027_q1_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2027_q1_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2027_q1_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2027_q1_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2027_q1_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2027_q1_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q1_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2027_q1_event_category_created_at_idx;


--
-- Name: user_activity_logs_2027_q1_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2027_q1_event_type_created_at_idx;


--
-- Name: user_activity_logs_2027_q1_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2027_q1_hmac_version_idx;


--
-- Name: user_activity_logs_2027_q1_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2027_q1_id_idx;


--
-- Name: user_activity_logs_2027_q1_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2027_q1_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2027_q1_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2027_q1_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2027_q1_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2027_q1_is_suspicious_idx;


--
-- Name: user_activity_logs_2027_q1_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2027_q1_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2027_q1_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2027_q1_pkey;


--
-- Name: user_activity_logs_2027_q2_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2027_q2_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q2_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2027_q2_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2027_q2_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2027_q2_after_data_idx;


--
-- Name: user_activity_logs_2027_q2_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2027_q2_before_data_idx;


--
-- Name: user_activity_logs_2027_q2_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2027_q2_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2027_q2_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2027_q2_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2027_q2_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2027_q2_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q2_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2027_q2_event_category_created_at_idx;


--
-- Name: user_activity_logs_2027_q2_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2027_q2_event_type_created_at_idx;


--
-- Name: user_activity_logs_2027_q2_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2027_q2_hmac_version_idx;


--
-- Name: user_activity_logs_2027_q2_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2027_q2_id_idx;


--
-- Name: user_activity_logs_2027_q2_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2027_q2_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2027_q2_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2027_q2_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2027_q2_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2027_q2_is_suspicious_idx;


--
-- Name: user_activity_logs_2027_q2_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2027_q2_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2027_q2_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2027_q2_pkey;


--
-- Name: user_activity_logs_2027_q3_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2027_q3_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q3_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2027_q3_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2027_q3_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2027_q3_after_data_idx;


--
-- Name: user_activity_logs_2027_q3_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2027_q3_before_data_idx;


--
-- Name: user_activity_logs_2027_q3_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2027_q3_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2027_q3_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2027_q3_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2027_q3_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2027_q3_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q3_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2027_q3_event_category_created_at_idx;


--
-- Name: user_activity_logs_2027_q3_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2027_q3_event_type_created_at_idx;


--
-- Name: user_activity_logs_2027_q3_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2027_q3_hmac_version_idx;


--
-- Name: user_activity_logs_2027_q3_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2027_q3_id_idx;


--
-- Name: user_activity_logs_2027_q3_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2027_q3_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2027_q3_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2027_q3_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2027_q3_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2027_q3_is_suspicious_idx;


--
-- Name: user_activity_logs_2027_q3_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2027_q3_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2027_q3_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2027_q3_pkey;


--
-- Name: user_activity_logs_2027_q4_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2027_q4_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q4_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2027_q4_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2027_q4_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2027_q4_after_data_idx;


--
-- Name: user_activity_logs_2027_q4_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2027_q4_before_data_idx;


--
-- Name: user_activity_logs_2027_q4_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2027_q4_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2027_q4_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2027_q4_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2027_q4_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2027_q4_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2027_q4_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2027_q4_event_category_created_at_idx;


--
-- Name: user_activity_logs_2027_q4_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2027_q4_event_type_created_at_idx;


--
-- Name: user_activity_logs_2027_q4_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2027_q4_hmac_version_idx;


--
-- Name: user_activity_logs_2027_q4_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2027_q4_id_idx;


--
-- Name: user_activity_logs_2027_q4_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2027_q4_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2027_q4_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2027_q4_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2027_q4_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2027_q4_is_suspicious_idx;


--
-- Name: user_activity_logs_2027_q4_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2027_q4_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2027_q4_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2027_q4_pkey;


--
-- Name: user_activity_logs_2028_q1_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2028_q1_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q1_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2028_q1_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2028_q1_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2028_q1_after_data_idx;


--
-- Name: user_activity_logs_2028_q1_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2028_q1_before_data_idx;


--
-- Name: user_activity_logs_2028_q1_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2028_q1_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2028_q1_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2028_q1_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2028_q1_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2028_q1_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q1_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2028_q1_event_category_created_at_idx;


--
-- Name: user_activity_logs_2028_q1_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2028_q1_event_type_created_at_idx;


--
-- Name: user_activity_logs_2028_q1_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2028_q1_hmac_version_idx;


--
-- Name: user_activity_logs_2028_q1_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2028_q1_id_idx;


--
-- Name: user_activity_logs_2028_q1_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2028_q1_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2028_q1_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2028_q1_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2028_q1_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2028_q1_is_suspicious_idx;


--
-- Name: user_activity_logs_2028_q1_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2028_q1_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2028_q1_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2028_q1_pkey;


--
-- Name: user_activity_logs_2028_q2_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2028_q2_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q2_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2028_q2_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2028_q2_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2028_q2_after_data_idx;


--
-- Name: user_activity_logs_2028_q2_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2028_q2_before_data_idx;


--
-- Name: user_activity_logs_2028_q2_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2028_q2_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2028_q2_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2028_q2_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2028_q2_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2028_q2_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q2_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2028_q2_event_category_created_at_idx;


--
-- Name: user_activity_logs_2028_q2_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2028_q2_event_type_created_at_idx;


--
-- Name: user_activity_logs_2028_q2_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2028_q2_hmac_version_idx;


--
-- Name: user_activity_logs_2028_q2_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2028_q2_id_idx;


--
-- Name: user_activity_logs_2028_q2_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2028_q2_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2028_q2_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2028_q2_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2028_q2_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2028_q2_is_suspicious_idx;


--
-- Name: user_activity_logs_2028_q2_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2028_q2_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2028_q2_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2028_q2_pkey;


--
-- Name: user_activity_logs_2028_q3_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2028_q3_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q3_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2028_q3_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2028_q3_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2028_q3_after_data_idx;


--
-- Name: user_activity_logs_2028_q3_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2028_q3_before_data_idx;


--
-- Name: user_activity_logs_2028_q3_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2028_q3_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2028_q3_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2028_q3_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2028_q3_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2028_q3_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q3_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2028_q3_event_category_created_at_idx;


--
-- Name: user_activity_logs_2028_q3_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2028_q3_event_type_created_at_idx;


--
-- Name: user_activity_logs_2028_q3_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2028_q3_hmac_version_idx;


--
-- Name: user_activity_logs_2028_q3_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2028_q3_id_idx;


--
-- Name: user_activity_logs_2028_q3_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2028_q3_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2028_q3_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2028_q3_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2028_q3_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2028_q3_is_suspicious_idx;


--
-- Name: user_activity_logs_2028_q3_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2028_q3_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2028_q3_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2028_q3_pkey;


--
-- Name: user_activity_logs_2028_q4_actor_user_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor ATTACH PARTITION public.user_activity_logs_2028_q4_actor_user_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q4_actor_user_id_created_at_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_actor_created ATTACH PARTITION public.user_activity_logs_2028_q4_actor_user_id_created_at_idx1;


--
-- Name: user_activity_logs_2028_q4_after_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_after_data_gin ATTACH PARTITION public.user_activity_logs_2028_q4_after_data_idx;


--
-- Name: user_activity_logs_2028_q4_before_data_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_ual_before_data_gin ATTACH PARTITION public.user_activity_logs_2028_q4_before_data_idx;


--
-- Name: user_activity_logs_2028_q4_created_at_integrity_hash_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_logs_integrity ATTACH PARTITION public.user_activity_logs_2028_q4_created_at_integrity_hash_idx;


--
-- Name: user_activity_logs_2028_q4_entity_type_entity_id_created_a_idx1; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_entity_created ATTACH PARTITION public.user_activity_logs_2028_q4_entity_type_entity_id_created_a_idx1;


--
-- Name: user_activity_logs_2028_q4_entity_type_entity_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_entity ATTACH PARTITION public.user_activity_logs_2028_q4_entity_type_entity_id_created_at_idx;


--
-- Name: user_activity_logs_2028_q4_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_category ATTACH PARTITION public.user_activity_logs_2028_q4_event_category_created_at_idx;


--
-- Name: user_activity_logs_2028_q4_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_event_type ATTACH PARTITION public.user_activity_logs_2028_q4_event_type_created_at_idx;


--
-- Name: user_activity_logs_2028_q4_hmac_version_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_hmac_version ATTACH PARTITION public.user_activity_logs_2028_q4_hmac_version_idx;


--
-- Name: user_activity_logs_2028_q4_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_hmac_null ATTACH PARTITION public.user_activity_logs_2028_q4_id_idx;


--
-- Name: user_activity_logs_2028_q4_impersonated_by_user_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_user_activity_logs_impersonated_by ATTACH PARTITION public.user_activity_logs_2028_q4_impersonated_by_user_id_created__idx;


--
-- Name: user_activity_logs_2028_q4_ip_address_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_ip ATTACH PARTITION public.user_activity_logs_2028_q4_ip_address_created_at_idx;


--
-- Name: user_activity_logs_2028_q4_is_suspicious_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_suspicious ATTACH PARTITION public.user_activity_logs_2028_q4_is_suspicious_idx;


--
-- Name: user_activity_logs_2028_q4_partition_date_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_activity_date ATTACH PARTITION public.user_activity_logs_2028_q4_partition_date_created_at_idx;


--
-- Name: user_activity_logs_2028_q4_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.user_activity_logs_pkey ATTACH PARTITION public.user_activity_logs_2028_q4_pkey;


--
-- Name: animal_blood_test_items check_blood_test_items_immutable_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_blood_test_items_immutable_trigger BEFORE UPDATE ON public.animal_blood_test_items FOR EACH ROW EXECUTE FUNCTION public.check_blood_test_items_immutable();


--
-- Name: animal_blood_test_items check_blood_test_items_no_delete_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_blood_test_items_no_delete_trigger BEFORE DELETE ON public.animal_blood_test_items FOR EACH ROW EXECUTE FUNCTION public.check_blood_test_items_no_delete();


--
-- Name: documents check_documents_system_generated_immutable_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_documents_system_generated_immutable_trigger BEFORE UPDATE ON public.documents FOR EACH ROW EXECUTE FUNCTION public.check_documents_system_generated_immutable();


--
-- Name: electronic_signatures check_electronic_signatures_immutable_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_electronic_signatures_immutable_trigger BEFORE UPDATE ON public.electronic_signatures FOR EACH ROW EXECUTE FUNCTION public.check_electronic_signatures_immutable();


--
-- Name: electronic_signatures check_electronic_signatures_no_delete_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_electronic_signatures_no_delete_trigger BEFORE DELETE ON public.electronic_signatures FOR EACH ROW EXECUTE FUNCTION public.check_electronic_signatures_no_delete();


--
-- Name: journal_entries check_journal_entries_immutable_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_journal_entries_immutable_trigger BEFORE UPDATE ON public.journal_entries FOR EACH ROW EXECUTE FUNCTION public.check_journal_entries_immutable();


--
-- Name: journal_entries check_journal_entries_no_delete_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_journal_entries_no_delete_trigger BEFORE DELETE ON public.journal_entries FOR EACH ROW EXECUTE FUNCTION public.check_journal_entries_no_delete();


--
-- Name: journal_entries check_journal_entries_no_truncate_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_journal_entries_no_truncate_trigger BEFORE TRUNCATE ON public.journal_entries FOR EACH STATEMENT EXECUTE FUNCTION public.check_journal_entries_no_truncate();


--
-- Name: journal_entry_lines check_journal_entry_lines_immutable_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_journal_entry_lines_immutable_trigger BEFORE UPDATE ON public.journal_entry_lines FOR EACH ROW EXECUTE FUNCTION public.check_journal_entry_lines_immutable();


--
-- Name: journal_entry_lines check_journal_entry_lines_no_delete_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_journal_entry_lines_no_delete_trigger BEFORE DELETE ON public.journal_entry_lines FOR EACH ROW EXECUTE FUNCTION public.check_journal_entry_lines_no_delete();


--
-- Name: journal_entry_lines check_journal_entry_lines_no_truncate_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_journal_entry_lines_no_truncate_trigger BEFORE TRUNCATE ON public.journal_entry_lines FOR EACH STATEMENT EXECUTE FUNCTION public.check_journal_entry_lines_no_truncate();


--
-- Name: stock_ledger check_stock_ledger_immutable_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_stock_ledger_immutable_trigger BEFORE UPDATE ON public.stock_ledger FOR EACH ROW EXECUTE FUNCTION public.check_stock_ledger_immutable();


--
-- Name: stock_ledger check_stock_ledger_no_delete_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_stock_ledger_no_delete_trigger BEFORE DELETE ON public.stock_ledger FOR EACH ROW EXECUTE FUNCTION public.check_stock_ledger_no_delete();


--
-- Name: stock_ledger check_stock_ledger_no_truncate_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_stock_ledger_no_truncate_trigger BEFORE TRUNCATE ON public.stock_ledger FOR EACH STATEMENT EXECUTE FUNCTION public.check_stock_ledger_no_truncate();


--
-- Name: user_activity_logs check_user_activity_logs_no_delete_trigger; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER check_user_activity_logs_no_delete_trigger BEFORE DELETE ON public.user_activity_logs FOR EACH ROW EXECUTE FUNCTION public.check_user_activity_logs_no_delete();


--
-- Name: users trg_create_notification_settings; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_create_notification_settings AFTER INSERT ON public.users FOR EACH ROW EXECUTE FUNCTION public.create_default_notification_settings();


--
-- Name: messages trg_messages_update_thread_timestamp; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_messages_update_thread_timestamp AFTER INSERT ON public.messages FOR EACH ROW EXECUTE FUNCTION public.update_thread_last_message_at();


--
-- Name: user_activity_logs trg_user_activity_logs_immutable; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER trg_user_activity_logs_immutable BEFORE UPDATE ON public.user_activity_logs FOR EACH ROW EXECUTE FUNCTION public.check_user_activity_logs_immutable();


--
-- Name: ai_api_keys ai_api_keys_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ai_api_keys
    ADD CONSTRAINT ai_api_keys_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: amendment_review_assignments amendment_review_assignments_amendment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_review_assignments
    ADD CONSTRAINT amendment_review_assignments_amendment_id_fkey FOREIGN KEY (amendment_id) REFERENCES public.amendments(id) ON DELETE CASCADE;


--
-- Name: amendment_review_assignments amendment_review_assignments_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_review_assignments
    ADD CONSTRAINT amendment_review_assignments_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id);


--
-- Name: amendment_review_assignments amendment_review_assignments_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_review_assignments
    ADD CONSTRAINT amendment_review_assignments_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: amendment_status_history amendment_status_history_amendment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_status_history
    ADD CONSTRAINT amendment_status_history_amendment_id_fkey FOREIGN KEY (amendment_id) REFERENCES public.amendments(id) ON DELETE CASCADE;


--
-- Name: amendment_versions amendment_versions_amendment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_versions
    ADD CONSTRAINT amendment_versions_amendment_id_fkey FOREIGN KEY (amendment_id) REFERENCES public.amendments(id) ON DELETE CASCADE;


--
-- Name: amendment_versions amendment_versions_submitted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendment_versions
    ADD CONSTRAINT amendment_versions_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id);


--
-- Name: amendments amendments_approved_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_approved_signature_id_fkey FOREIGN KEY (approved_signature_id) REFERENCES public.electronic_signatures(id) ON DELETE RESTRICT;


--
-- Name: amendments amendments_classified_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_classified_by_fkey FOREIGN KEY (classified_by) REFERENCES public.users(id);


--
-- Name: amendments amendments_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: amendments amendments_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: amendments amendments_rejected_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_rejected_signature_id_fkey FOREIGN KEY (rejected_signature_id) REFERENCES public.electronic_signatures(id) ON DELETE RESTRICT;


--
-- Name: amendments amendments_submitted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.amendments
    ADD CONSTRAINT amendments_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id);


--
-- Name: animal_blood_test_items animal_blood_test_items_blood_test_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_test_items
    ADD CONSTRAINT animal_blood_test_items_blood_test_id_fkey FOREIGN KEY (blood_test_id) REFERENCES public.animal_blood_tests(id) ON DELETE CASCADE;


--
-- Name: animal_blood_test_items animal_blood_test_items_corrected_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_test_items
    ADD CONSTRAINT animal_blood_test_items_corrected_by_fkey FOREIGN KEY (corrected_by) REFERENCES public.users(id);


--
-- Name: animal_blood_test_items animal_blood_test_items_superseded_by_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_test_items
    ADD CONSTRAINT animal_blood_test_items_superseded_by_id_fkey FOREIGN KEY (superseded_by_id) REFERENCES public.animal_blood_test_items(id);


--
-- Name: animal_blood_test_items animal_blood_test_items_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_test_items
    ADD CONSTRAINT animal_blood_test_items_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.blood_test_templates(id);


--
-- Name: animal_blood_tests animal_blood_tests_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_tests
    ADD CONSTRAINT animal_blood_tests_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_blood_tests animal_blood_tests_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_tests
    ADD CONSTRAINT animal_blood_tests_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_blood_tests animal_blood_tests_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_tests
    ADD CONSTRAINT animal_blood_tests_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: animal_blood_tests animal_blood_tests_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_blood_tests
    ADD CONSTRAINT animal_blood_tests_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);


--
-- Name: animal_field_correction_requests animal_field_correction_requests_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_field_correction_requests
    ADD CONSTRAINT animal_field_correction_requests_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_field_correction_requests animal_field_correction_requests_requested_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_field_correction_requests
    ADD CONSTRAINT animal_field_correction_requests_requested_by_fkey FOREIGN KEY (requested_by) REFERENCES public.users(id);


--
-- Name: animal_field_correction_requests animal_field_correction_requests_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_field_correction_requests
    ADD CONSTRAINT animal_field_correction_requests_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: animal_import_batches animal_import_batches_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_import_batches
    ADD CONSTRAINT animal_import_batches_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_observations animal_observations_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_observations
    ADD CONSTRAINT animal_observations_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_observations animal_observations_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_observations
    ADD CONSTRAINT animal_observations_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_observations animal_observations_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_observations
    ADD CONSTRAINT animal_observations_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: animal_observations animal_observations_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_observations
    ADD CONSTRAINT animal_observations_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);


--
-- Name: animal_observations animal_observations_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_observations
    ADD CONSTRAINT animal_observations_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: animal_pathology_reports animal_pathology_reports_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_pathology_reports
    ADD CONSTRAINT animal_pathology_reports_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_pathology_reports animal_pathology_reports_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_pathology_reports
    ADD CONSTRAINT animal_pathology_reports_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_sacrifices animal_sacrifices_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sacrifices
    ADD CONSTRAINT animal_sacrifices_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_sacrifices animal_sacrifices_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sacrifices
    ADD CONSTRAINT animal_sacrifices_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_sacrifices animal_sacrifices_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sacrifices
    ADD CONSTRAINT animal_sacrifices_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);


--
-- Name: animal_sudden_deaths animal_sudden_deaths_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sudden_deaths
    ADD CONSTRAINT animal_sudden_deaths_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id);


--
-- Name: animal_sudden_deaths animal_sudden_deaths_discovered_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_sudden_deaths
    ADD CONSTRAINT animal_sudden_deaths_discovered_by_fkey FOREIGN KEY (discovered_by) REFERENCES public.users(id);


--
-- Name: animal_surgeries animal_surgeries_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_surgeries
    ADD CONSTRAINT animal_surgeries_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_surgeries animal_surgeries_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_surgeries
    ADD CONSTRAINT animal_surgeries_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_surgeries animal_surgeries_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_surgeries
    ADD CONSTRAINT animal_surgeries_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: animal_surgeries animal_surgeries_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_surgeries
    ADD CONSTRAINT animal_surgeries_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);


--
-- Name: animal_transfers animal_transfers_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_transfers
    ADD CONSTRAINT animal_transfers_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id);


--
-- Name: animal_transfers animal_transfers_initiated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_transfers
    ADD CONSTRAINT animal_transfers_initiated_by_fkey FOREIGN KEY (initiated_by) REFERENCES public.users(id);


--
-- Name: animal_transfers animal_transfers_rejected_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_transfers
    ADD CONSTRAINT animal_transfers_rejected_by_fkey FOREIGN KEY (rejected_by) REFERENCES public.users(id);


--
-- Name: animal_vaccinations animal_vaccinations_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vaccinations
    ADD CONSTRAINT animal_vaccinations_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_vaccinations animal_vaccinations_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vaccinations
    ADD CONSTRAINT animal_vaccinations_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_vaccinations animal_vaccinations_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vaccinations
    ADD CONSTRAINT animal_vaccinations_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: animal_vet_advice_records animal_vet_advice_records_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advice_records
    ADD CONSTRAINT animal_vet_advice_records_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id);


--
-- Name: animal_vet_advice_records animal_vet_advice_records_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advice_records
    ADD CONSTRAINT animal_vet_advice_records_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_vet_advice_records animal_vet_advice_records_source_vet_patrol_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advice_records
    ADD CONSTRAINT animal_vet_advice_records_source_vet_patrol_entry_id_fkey FOREIGN KEY (source_vet_patrol_entry_id) REFERENCES public.vet_patrol_entries(id) ON DELETE CASCADE;


--
-- Name: animal_vet_advice_records animal_vet_advice_records_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advice_records
    ADD CONSTRAINT animal_vet_advice_records_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: animal_vet_advices animal_vet_advices_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advices
    ADD CONSTRAINT animal_vet_advices_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id);


--
-- Name: animal_vet_advices animal_vet_advices_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advices
    ADD CONSTRAINT animal_vet_advices_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_vet_advices animal_vet_advices_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_vet_advices
    ADD CONSTRAINT animal_vet_advices_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: animal_weights animal_weights_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_weights
    ADD CONSTRAINT animal_weights_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: animal_weights animal_weights_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_weights
    ADD CONSTRAINT animal_weights_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animal_weights animal_weights_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animal_weights
    ADD CONSTRAINT animal_weights_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: animals animals_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: animals animals_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: animals animals_experiment_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_experiment_assigned_by_fkey FOREIGN KEY (experiment_assigned_by) REFERENCES public.users(id);


--
-- Name: animals animals_pen_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_pen_id_fkey FOREIGN KEY (pen_id) REFERENCES public.pens(id);


--
-- Name: animals animals_reserved_planned_experiment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_reserved_planned_experiment_id_fkey FOREIGN KEY (reserved_planned_experiment_id) REFERENCES public.planned_experiments(id) ON DELETE SET NULL;


--
-- Name: animals animals_reserved_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_reserved_protocol_id_fkey FOREIGN KEY (reserved_protocol_id) REFERENCES public.protocols(id) ON DELETE SET NULL;


--
-- Name: animals animals_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_source_id_fkey FOREIGN KEY (source_id) REFERENCES public.animal_sources(id);


--
-- Name: animals animals_species_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.animals
    ADD CONSTRAINT animals_species_id_fkey FOREIGN KEY (species_id) REFERENCES public.species(id);


--
-- Name: annual_leave_entitlements annual_leave_entitlements_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.annual_leave_entitlements
    ADD CONSTRAINT annual_leave_entitlements_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: annual_leave_entitlements annual_leave_entitlements_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.annual_leave_entitlements
    ADD CONSTRAINT annual_leave_entitlements_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: ap_payments ap_payments_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ap_payments
    ADD CONSTRAINT ap_payments_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: ap_payments ap_payments_journal_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ap_payments
    ADD CONSTRAINT ap_payments_journal_entry_id_fkey FOREIGN KEY (journal_entry_id) REFERENCES public.journal_entries(id);


--
-- Name: ap_payments ap_payments_partner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ap_payments
    ADD CONSTRAINT ap_payments_partner_id_fkey FOREIGN KEY (partner_id) REFERENCES public.partners(id);


--
-- Name: application_notices application_notices_attachment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.application_notices
    ADD CONSTRAINT application_notices_attachment_id_fkey FOREIGN KEY (attachment_id) REFERENCES public.attachments(id);


--
-- Name: application_notices application_notices_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.application_notices
    ADD CONSTRAINT application_notices_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: ar_receipts ar_receipts_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ar_receipts
    ADD CONSTRAINT ar_receipts_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: ar_receipts ar_receipts_journal_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ar_receipts
    ADD CONSTRAINT ar_receipts_journal_entry_id_fkey FOREIGN KEY (journal_entry_id) REFERENCES public.journal_entries(id);


--
-- Name: ar_receipts ar_receipts_partner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ar_receipts
    ADD CONSTRAINT ar_receipts_partner_id_fkey FOREIGN KEY (partner_id) REFERENCES public.partners(id);


--
-- Name: attachments attachments_uploaded_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attachments
    ADD CONSTRAINT attachments_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES public.users(id);


--
-- Name: attendance_records attendance_records_corrected_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_records
    ADD CONSTRAINT attendance_records_corrected_by_fkey FOREIGN KEY (corrected_by) REFERENCES public.users(id);


--
-- Name: attendance_records attendance_records_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.attendance_records
    ADD CONSTRAINT attendance_records_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: audit_logs audit_logs_actor_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.audit_logs
    ADD CONSTRAINT audit_logs_actor_user_id_fkey FOREIGN KEY (actor_user_id) REFERENCES public.users(id);


--
-- Name: blood_test_panel_items blood_test_panel_items_panel_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_panel_items
    ADD CONSTRAINT blood_test_panel_items_panel_id_fkey FOREIGN KEY (panel_id) REFERENCES public.blood_test_panels(id) ON DELETE CASCADE;


--
-- Name: blood_test_panel_items blood_test_panel_items_template_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.blood_test_panel_items
    ADD CONSTRAINT blood_test_panel_items_template_id_fkey FOREIGN KEY (template_id) REFERENCES public.blood_test_templates(id) ON DELETE CASCADE;


--
-- Name: buildings buildings_facility_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.buildings
    ADD CONSTRAINT buildings_facility_id_fkey FOREIGN KEY (facility_id) REFERENCES public.facilities(id);


--
-- Name: calendar_event_sync calendar_event_sync_leave_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_event_sync
    ADD CONSTRAINT calendar_event_sync_leave_request_id_fkey FOREIGN KEY (leave_request_id) REFERENCES public.leave_requests(id) ON DELETE CASCADE;


--
-- Name: calendar_sync_conflicts calendar_sync_conflicts_calendar_event_sync_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_conflicts
    ADD CONSTRAINT calendar_sync_conflicts_calendar_event_sync_id_fkey FOREIGN KEY (calendar_event_sync_id) REFERENCES public.calendar_event_sync(id) ON DELETE SET NULL;


--
-- Name: calendar_sync_conflicts calendar_sync_conflicts_leave_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_conflicts
    ADD CONSTRAINT calendar_sync_conflicts_leave_request_id_fkey FOREIGN KEY (leave_request_id) REFERENCES public.leave_requests(id) ON DELETE SET NULL;


--
-- Name: calendar_sync_conflicts calendar_sync_conflicts_new_approval_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_conflicts
    ADD CONSTRAINT calendar_sync_conflicts_new_approval_request_id_fkey FOREIGN KEY (new_approval_request_id) REFERENCES public.leave_requests(id);


--
-- Name: calendar_sync_conflicts calendar_sync_conflicts_resolved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_conflicts
    ADD CONSTRAINT calendar_sync_conflicts_resolved_by_fkey FOREIGN KEY (resolved_by) REFERENCES public.users(id);


--
-- Name: calendar_sync_history calendar_sync_history_triggered_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.calendar_sync_history
    ADD CONSTRAINT calendar_sync_history_triggered_by_fkey FOREIGN KEY (triggered_by) REFERENCES public.users(id);


--
-- Name: care_medication_records care_medication_records_deleted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.care_medication_records
    ADD CONSTRAINT care_medication_records_deleted_by_fkey FOREIGN KEY (deleted_by) REFERENCES public.users(id);


--
-- Name: care_medication_records care_medication_records_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.care_medication_records
    ADD CONSTRAINT care_medication_records_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);


--
-- Name: change_reasons change_reasons_changed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_reasons
    ADD CONSTRAINT change_reasons_changed_by_fkey FOREIGN KEY (changed_by) REFERENCES public.users(id);


--
-- Name: change_requests change_requests_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_requests
    ADD CONSTRAINT change_requests_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: change_requests change_requests_requested_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_requests
    ADD CONSTRAINT change_requests_requested_by_fkey FOREIGN KEY (requested_by) REFERENCES public.users(id);


--
-- Name: change_requests change_requests_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_requests
    ADD CONSTRAINT change_requests_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: change_requests change_requests_verified_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.change_requests
    ADD CONSTRAINT change_requests_verified_by_fkey FOREIGN KEY (verified_by) REFERENCES public.users(id);


--
-- Name: chart_of_accounts chart_of_accounts_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.chart_of_accounts
    ADD CONSTRAINT chart_of_accounts_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.chart_of_accounts(id);


--
-- Name: comp_time_balances comp_time_balances_overtime_record_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comp_time_balances
    ADD CONSTRAINT comp_time_balances_overtime_record_id_fkey FOREIGN KEY (overtime_record_id) REFERENCES public.overtime_records(id) ON DELETE CASCADE;


--
-- Name: comp_time_balances comp_time_balances_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.comp_time_balances
    ADD CONSTRAINT comp_time_balances_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: competency_assessments competency_assessments_assessor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.competency_assessments
    ADD CONSTRAINT competency_assessments_assessor_id_fkey FOREIGN KEY (assessor_id) REFERENCES public.users(id);


--
-- Name: competency_assessments competency_assessments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.competency_assessments
    ADD CONSTRAINT competency_assessments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: controlled_documents controlled_documents_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.controlled_documents
    ADD CONSTRAINT controlled_documents_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: controlled_documents controlled_documents_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.controlled_documents
    ADD CONSTRAINT controlled_documents_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: departments departments_manager_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_manager_id_fkey FOREIGN KEY (manager_id) REFERENCES public.users(id);


--
-- Name: departments departments_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.departments
    ADD CONSTRAINT departments_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.departments(id);


--
-- Name: document_acknowledgments document_acknowledgments_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_acknowledgments
    ADD CONSTRAINT document_acknowledgments_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.controlled_documents(id) ON DELETE CASCADE;


--
-- Name: document_acknowledgments document_acknowledgments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_acknowledgments
    ADD CONSTRAINT document_acknowledgments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: document_lines document_lines_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;


--
-- Name: document_lines document_lines_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: document_lines document_lines_storage_location_from_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_storage_location_from_id_fkey FOREIGN KEY (storage_location_from_id) REFERENCES public.storage_locations(id);


--
-- Name: document_lines document_lines_storage_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_storage_location_id_fkey FOREIGN KEY (storage_location_id) REFERENCES public.storage_locations(id);


--
-- Name: document_lines document_lines_storage_location_to_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_storage_location_to_id_fkey FOREIGN KEY (storage_location_to_id) REFERENCES public.storage_locations(id);


--
-- Name: document_lines document_lines_warehouse_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_lines
    ADD CONSTRAINT document_lines_warehouse_id_fkey FOREIGN KEY (warehouse_id) REFERENCES public.warehouses(id);


--
-- Name: document_revisions document_revisions_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_revisions
    ADD CONSTRAINT document_revisions_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: document_revisions document_revisions_document_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_revisions
    ADD CONSTRAINT document_revisions_document_id_fkey FOREIGN KEY (document_id) REFERENCES public.controlled_documents(id) ON DELETE CASCADE;


--
-- Name: document_revisions document_revisions_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_revisions
    ADD CONSTRAINT document_revisions_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: document_revisions document_revisions_revised_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.document_revisions
    ADD CONSTRAINT document_revisions_revised_by_fkey FOREIGN KEY (revised_by) REFERENCES public.users(id);


--
-- Name: documents documents_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: documents documents_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: documents documents_manager_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_manager_approved_by_fkey FOREIGN KEY (manager_approved_by) REFERENCES public.users(id);


--
-- Name: documents documents_partner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_partner_id_fkey FOREIGN KEY (partner_id) REFERENCES public.partners(id);


--
-- Name: documents documents_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE SET NULL;


--
-- Name: documents documents_reverses_doc_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_reverses_doc_id_fkey FOREIGN KEY (reverses_doc_id) REFERENCES public.documents(id);


--
-- Name: documents documents_source_doc_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_source_doc_id_fkey FOREIGN KEY (source_doc_id) REFERENCES public.documents(id);


--
-- Name: documents documents_warehouse_from_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_warehouse_from_id_fkey FOREIGN KEY (warehouse_from_id) REFERENCES public.warehouses(id);


--
-- Name: documents documents_warehouse_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_warehouse_id_fkey FOREIGN KEY (warehouse_id) REFERENCES public.warehouses(id);


--
-- Name: documents documents_warehouse_to_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT documents_warehouse_to_id_fkey FOREIGN KEY (warehouse_to_id) REFERENCES public.warehouses(id);


--
-- Name: electronic_signatures electronic_signatures_invalidated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.electronic_signatures
    ADD CONSTRAINT electronic_signatures_invalidated_by_fkey FOREIGN KEY (invalidated_by) REFERENCES public.users(id);


--
-- Name: electronic_signatures electronic_signatures_signer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.electronic_signatures
    ADD CONSTRAINT electronic_signatures_signer_id_fkey FOREIGN KEY (signer_id) REFERENCES public.users(id);


--
-- Name: environment_monitoring_points environment_monitoring_points_building_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.environment_monitoring_points
    ADD CONSTRAINT environment_monitoring_points_building_id_fkey FOREIGN KEY (building_id) REFERENCES public.buildings(id);


--
-- Name: environment_monitoring_points environment_monitoring_points_zone_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.environment_monitoring_points
    ADD CONSTRAINT environment_monitoring_points_zone_id_fkey FOREIGN KEY (zone_id) REFERENCES public.zones(id);


--
-- Name: environment_readings environment_readings_monitoring_point_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.environment_readings
    ADD CONSTRAINT environment_readings_monitoring_point_id_fkey FOREIGN KEY (monitoring_point_id) REFERENCES public.environment_monitoring_points(id) ON DELETE CASCADE;


--
-- Name: environment_readings environment_readings_recorded_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.environment_readings
    ADD CONSTRAINT environment_readings_recorded_by_fkey FOREIGN KEY (recorded_by) REFERENCES public.users(id);


--
-- Name: equipment_annual_plans equipment_annual_plans_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_annual_plans
    ADD CONSTRAINT equipment_annual_plans_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_calibrations equipment_calibrations_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_calibrations
    ADD CONSTRAINT equipment_calibrations_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_calibrations equipment_calibrations_partner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_calibrations
    ADD CONSTRAINT equipment_calibrations_partner_id_fkey FOREIGN KEY (partner_id) REFERENCES public.partners(id);


--
-- Name: equipment_calibrations equipment_calibrations_reference_standard_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_calibrations
    ADD CONSTRAINT equipment_calibrations_reference_standard_id_fkey FOREIGN KEY (reference_standard_id) REFERENCES public.reference_standards(id);


--
-- Name: equipment_disposals equipment_disposals_applicant_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_disposals
    ADD CONSTRAINT equipment_disposals_applicant_signature_id_fkey FOREIGN KEY (applicant_signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: equipment_disposals equipment_disposals_applied_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_disposals
    ADD CONSTRAINT equipment_disposals_applied_by_fkey FOREIGN KEY (applied_by) REFERENCES public.users(id);


--
-- Name: equipment_disposals equipment_disposals_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_disposals
    ADD CONSTRAINT equipment_disposals_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: equipment_disposals equipment_disposals_approver_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_disposals
    ADD CONSTRAINT equipment_disposals_approver_signature_id_fkey FOREIGN KEY (approver_signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: equipment_disposals equipment_disposals_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_disposals
    ADD CONSTRAINT equipment_disposals_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_idle_requests equipment_idle_requests_applicant_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_idle_requests
    ADD CONSTRAINT equipment_idle_requests_applicant_signature_id_fkey FOREIGN KEY (applicant_signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: equipment_idle_requests equipment_idle_requests_applied_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_idle_requests
    ADD CONSTRAINT equipment_idle_requests_applied_by_fkey FOREIGN KEY (applied_by) REFERENCES public.users(id);


--
-- Name: equipment_idle_requests equipment_idle_requests_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_idle_requests
    ADD CONSTRAINT equipment_idle_requests_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: equipment_idle_requests equipment_idle_requests_approver_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_idle_requests
    ADD CONSTRAINT equipment_idle_requests_approver_signature_id_fkey FOREIGN KEY (approver_signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: equipment_idle_requests equipment_idle_requests_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_idle_requests
    ADD CONSTRAINT equipment_idle_requests_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_maintenance_records equipment_maintenance_records_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_maintenance_records
    ADD CONSTRAINT equipment_maintenance_records_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: equipment_maintenance_records equipment_maintenance_records_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_maintenance_records
    ADD CONSTRAINT equipment_maintenance_records_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_maintenance_records equipment_maintenance_records_repair_partner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_maintenance_records
    ADD CONSTRAINT equipment_maintenance_records_repair_partner_id_fkey FOREIGN KEY (repair_partner_id) REFERENCES public.partners(id);


--
-- Name: equipment_maintenance_records equipment_maintenance_records_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_maintenance_records
    ADD CONSTRAINT equipment_maintenance_records_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: equipment_maintenance_records equipment_maintenance_records_reviewer_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_maintenance_records
    ADD CONSTRAINT equipment_maintenance_records_reviewer_signature_id_fkey FOREIGN KEY (reviewer_signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: equipment_status_logs equipment_status_logs_changed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_status_logs
    ADD CONSTRAINT equipment_status_logs_changed_by_fkey FOREIGN KEY (changed_by) REFERENCES public.users(id);


--
-- Name: equipment_status_logs equipment_status_logs_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_status_logs
    ADD CONSTRAINT equipment_status_logs_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_suppliers equipment_suppliers_equipment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_suppliers
    ADD CONSTRAINT equipment_suppliers_equipment_id_fkey FOREIGN KEY (equipment_id) REFERENCES public.equipment(id) ON DELETE CASCADE;


--
-- Name: equipment_suppliers equipment_suppliers_partner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.equipment_suppliers
    ADD CONSTRAINT equipment_suppliers_partner_id_fkey FOREIGN KEY (partner_id) REFERENCES public.partners(id) ON DELETE CASCADE;


--
-- Name: euthanasia_appeals euthanasia_appeals_chair_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_appeals
    ADD CONSTRAINT euthanasia_appeals_chair_user_id_fkey FOREIGN KEY (chair_user_id) REFERENCES public.users(id);


--
-- Name: euthanasia_appeals euthanasia_appeals_order_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_appeals
    ADD CONSTRAINT euthanasia_appeals_order_id_fkey FOREIGN KEY (order_id) REFERENCES public.euthanasia_orders(id) ON DELETE CASCADE;


--
-- Name: euthanasia_appeals euthanasia_appeals_pi_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_appeals
    ADD CONSTRAINT euthanasia_appeals_pi_user_id_fkey FOREIGN KEY (pi_user_id) REFERENCES public.users(id);


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE RESTRICT;


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_collector_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_collector_id_fkey FOREIGN KEY (collector_id) REFERENCES public.users(id) ON DELETE RESTRICT;


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE RESTRICT;


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_euthanasia_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_euthanasia_id_fkey FOREIGN KEY (euthanasia_id) REFERENCES public.euthanasia_orders(id) ON DELETE RESTRICT;


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_requester_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_requester_user_id_fkey FOREIGN KEY (requester_user_id) REFERENCES public.users(id) ON DELETE RESTRICT;


--
-- Name: euthanasia_byproduct_samples euthanasia_byproduct_samples_source_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_byproduct_samples
    ADD CONSTRAINT euthanasia_byproduct_samples_source_protocol_id_fkey FOREIGN KEY (source_protocol_id) REFERENCES public.protocols(id) ON DELETE RESTRICT;


--
-- Name: euthanasia_orders euthanasia_orders_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_orders
    ADD CONSTRAINT euthanasia_orders_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: euthanasia_orders euthanasia_orders_executed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_orders
    ADD CONSTRAINT euthanasia_orders_executed_by_fkey FOREIGN KEY (executed_by) REFERENCES public.users(id);


--
-- Name: euthanasia_orders euthanasia_orders_pi_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_orders
    ADD CONSTRAINT euthanasia_orders_pi_user_id_fkey FOREIGN KEY (pi_user_id) REFERENCES public.users(id);


--
-- Name: euthanasia_orders euthanasia_orders_vet_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.euthanasia_orders
    ADD CONSTRAINT euthanasia_orders_vet_user_id_fkey FOREIGN KEY (vet_user_id) REFERENCES public.users(id);


--
-- Name: event_outbox event_outbox_enqueued_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.event_outbox
    ADD CONSTRAINT event_outbox_enqueued_by_fkey FOREIGN KEY (enqueued_by) REFERENCES public.users(id);


--
-- Name: expiry_monthly_snapshots expiry_monthly_snapshots_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.expiry_monthly_snapshots
    ADD CONSTRAINT expiry_monthly_snapshots_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: expiry_monthly_snapshots expiry_monthly_snapshots_warehouse_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.expiry_monthly_snapshots
    ADD CONSTRAINT expiry_monthly_snapshots_warehouse_id_fkey FOREIGN KEY (warehouse_id) REFERENCES public.warehouses(id);


--
-- Name: expiry_notification_config expiry_notification_config_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.expiry_notification_config
    ADD CONSTRAINT expiry_notification_config_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: export_jobs export_jobs_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.export_jobs
    ADD CONSTRAINT export_jobs_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: formulation_records formulation_records_prepared_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.formulation_records
    ADD CONSTRAINT formulation_records_prepared_by_fkey FOREIGN KEY (prepared_by) REFERENCES public.users(id);


--
-- Name: formulation_records formulation_records_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.formulation_records
    ADD CONSTRAINT formulation_records_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: formulation_records formulation_records_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.formulation_records
    ADD CONSTRAINT formulation_records_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id);


--
-- Name: formulation_records formulation_records_verified_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.formulation_records
    ADD CONSTRAINT formulation_records_verified_by_fkey FOREIGN KEY (verified_by) REFERENCES public.users(id);


--
-- Name: import_jobs import_jobs_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.import_jobs
    ADD CONSTRAINT import_jobs_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: inventory_snapshots inventory_snapshots_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.inventory_snapshots
    ADD CONSTRAINT inventory_snapshots_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: inventory_snapshots inventory_snapshots_warehouse_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.inventory_snapshots
    ADD CONSTRAINT inventory_snapshots_warehouse_id_fkey FOREIGN KEY (warehouse_id) REFERENCES public.warehouses(id);


--
-- Name: invitation_roles invitation_roles_invitation_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitation_roles
    ADD CONSTRAINT invitation_roles_invitation_id_fkey FOREIGN KEY (invitation_id) REFERENCES public.invitations(id) ON DELETE CASCADE;


--
-- Name: invitation_roles invitation_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitation_roles
    ADD CONSTRAINT invitation_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE RESTRICT;


--
-- Name: invitations invitations_created_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitations
    ADD CONSTRAINT invitations_created_user_id_fkey FOREIGN KEY (created_user_id) REFERENCES public.users(id);


--
-- Name: invitations invitations_invited_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.invitations
    ADD CONSTRAINT invitations_invited_by_fkey FOREIGN KEY (invited_by) REFERENCES public.users(id);


--
-- Name: ip_blocklist ip_blocklist_blocked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ip_blocklist
    ADD CONSTRAINT ip_blocklist_blocked_by_fkey FOREIGN KEY (blocked_by) REFERENCES public.users(id);


--
-- Name: ip_blocklist ip_blocklist_unblocked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.ip_blocklist
    ADD CONSTRAINT ip_blocklist_unblocked_by_fkey FOREIGN KEY (unblocked_by) REFERENCES public.users(id);


--
-- Name: journal_entries journal_entries_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entries
    ADD CONSTRAINT journal_entries_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: journal_entry_lines journal_entry_lines_account_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entry_lines
    ADD CONSTRAINT journal_entry_lines_account_id_fkey FOREIGN KEY (account_id) REFERENCES public.chart_of_accounts(id);


--
-- Name: journal_entry_lines journal_entry_lines_journal_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.journal_entry_lines
    ADD CONSTRAINT journal_entry_lines_journal_entry_id_fkey FOREIGN KEY (journal_entry_id) REFERENCES public.journal_entries(id) ON DELETE CASCADE;


--
-- Name: leave_approvals leave_approvals_approver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_approvals
    ADD CONSTRAINT leave_approvals_approver_id_fkey FOREIGN KEY (approver_id) REFERENCES public.users(id);


--
-- Name: leave_approvals leave_approvals_leave_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_approvals
    ADD CONSTRAINT leave_approvals_leave_request_id_fkey FOREIGN KEY (leave_request_id) REFERENCES public.leave_requests(id) ON DELETE CASCADE;


--
-- Name: leave_balance_usage leave_balance_usage_annual_leave_entitlement_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balance_usage
    ADD CONSTRAINT leave_balance_usage_annual_leave_entitlement_id_fkey FOREIGN KEY (annual_leave_entitlement_id) REFERENCES public.annual_leave_entitlements(id);


--
-- Name: leave_balance_usage leave_balance_usage_comp_time_balance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balance_usage
    ADD CONSTRAINT leave_balance_usage_comp_time_balance_id_fkey FOREIGN KEY (comp_time_balance_id) REFERENCES public.comp_time_balances(id);


--
-- Name: leave_balance_usage leave_balance_usage_leave_request_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_balance_usage
    ADD CONSTRAINT leave_balance_usage_leave_request_id_fkey FOREIGN KEY (leave_request_id) REFERENCES public.leave_requests(id);


--
-- Name: leave_requests leave_requests_annual_leave_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_annual_leave_source_id_fkey FOREIGN KEY (annual_leave_source_id) REFERENCES public.annual_leave_entitlements(id);


--
-- Name: leave_requests leave_requests_current_approver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_current_approver_id_fkey FOREIGN KEY (current_approver_id) REFERENCES public.users(id);


--
-- Name: leave_requests leave_requests_proxy_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_proxy_user_id_fkey FOREIGN KEY (proxy_user_id) REFERENCES public.users(id);


--
-- Name: leave_requests leave_requests_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.leave_requests
    ADD CONSTRAINT leave_requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: line_shelf_allocations line_shelf_allocations_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.line_shelf_allocations
    ADD CONSTRAINT line_shelf_allocations_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: line_shelf_allocations line_shelf_allocations_document_line_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.line_shelf_allocations
    ADD CONSTRAINT line_shelf_allocations_document_line_id_fkey FOREIGN KEY (document_line_id) REFERENCES public.document_lines(id) ON DELETE SET NULL;


--
-- Name: line_shelf_allocations line_shelf_allocations_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.line_shelf_allocations
    ADD CONSTRAINT line_shelf_allocations_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: line_shelf_allocations line_shelf_allocations_storage_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.line_shelf_allocations
    ADD CONSTRAINT line_shelf_allocations_storage_location_id_fkey FOREIGN KEY (storage_location_id) REFERENCES public.storage_locations(id);


--
-- Name: login_events login_events_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.login_events
    ADD CONSTRAINT login_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: management_reviews management_reviews_chaired_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.management_reviews
    ADD CONSTRAINT management_reviews_chaired_by_fkey FOREIGN KEY (chaired_by) REFERENCES public.users(id);


--
-- Name: message_attachments message_attachments_message_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_attachments
    ADD CONSTRAINT message_attachments_message_id_fkey FOREIGN KEY (message_id) REFERENCES public.messages(id) ON DELETE CASCADE;


--
-- Name: message_attachments message_attachments_uploaded_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_attachments
    ADD CONSTRAINT message_attachments_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES public.users(id);


--
-- Name: message_thread_participants message_thread_participants_thread_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_thread_participants
    ADD CONSTRAINT message_thread_participants_thread_id_fkey FOREIGN KEY (thread_id) REFERENCES public.message_threads(id) ON DELETE CASCADE;


--
-- Name: message_thread_participants message_thread_participants_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_thread_participants
    ADD CONSTRAINT message_thread_participants_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: message_threads message_threads_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.message_threads
    ADD CONSTRAINT message_threads_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: messages messages_sender_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES public.users(id);


--
-- Name: messages messages_thread_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.messages
    ADD CONSTRAINT messages_thread_id_fkey FOREIGN KEY (thread_id) REFERENCES public.message_threads(id) ON DELETE CASCADE;


--
-- Name: notification_routing notification_routing_role_code_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_routing
    ADD CONSTRAINT notification_routing_role_code_fkey FOREIGN KEY (role_code) REFERENCES public.roles(code);


--
-- Name: notification_settings notification_settings_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notification_settings
    ADD CONSTRAINT notification_settings_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: notifications notifications_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: observation_vet_reads observation_vet_reads_vet_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.observation_vet_reads
    ADD CONSTRAINT observation_vet_reads_vet_user_id_fkey FOREIGN KEY (vet_user_id) REFERENCES public.users(id);


--
-- Name: overtime_approvals overtime_approvals_approver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_approvals
    ADD CONSTRAINT overtime_approvals_approver_id_fkey FOREIGN KEY (approver_id) REFERENCES public.users(id);


--
-- Name: overtime_approvals overtime_approvals_overtime_record_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_approvals
    ADD CONSTRAINT overtime_approvals_overtime_record_id_fkey FOREIGN KEY (overtime_record_id) REFERENCES public.overtime_records(id) ON DELETE CASCADE;


--
-- Name: overtime_records overtime_records_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_records
    ADD CONSTRAINT overtime_records_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: overtime_records overtime_records_attendance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_records
    ADD CONSTRAINT overtime_records_attendance_id_fkey FOREIGN KEY (attendance_id) REFERENCES public.attendance_records(id);


--
-- Name: overtime_records overtime_records_rejected_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_records
    ADD CONSTRAINT overtime_records_rejected_by_fkey FOREIGN KEY (rejected_by) REFERENCES public.users(id);


--
-- Name: overtime_records overtime_records_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_records
    ADD CONSTRAINT overtime_records_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: overtime_records overtime_records_voided_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.overtime_records
    ADD CONSTRAINT overtime_records_voided_by_fkey FOREIGN KEY (voided_by) REFERENCES public.users(id);


--
-- Name: password_reset_tokens password_reset_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: pdf_artifacts pdf_artifacts_attachment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pdf_artifacts
    ADD CONSTRAINT pdf_artifacts_attachment_id_fkey FOREIGN KEY (attachment_id) REFERENCES public.attachments(id);


--
-- Name: pdf_artifacts pdf_artifacts_electronic_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pdf_artifacts
    ADD CONSTRAINT pdf_artifacts_electronic_signature_id_fkey FOREIGN KEY (electronic_signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: pdf_artifacts pdf_artifacts_generated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pdf_artifacts
    ADD CONSTRAINT pdf_artifacts_generated_by_fkey FOREIGN KEY (generated_by) REFERENCES public.users(id);


--
-- Name: pens pens_zone_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pens
    ADD CONSTRAINT pens_zone_id_fkey FOREIGN KEY (zone_id) REFERENCES public.zones(id);


--
-- Name: pi_account_invites pi_account_invites_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pi_account_invites
    ADD CONSTRAINT pi_account_invites_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: pi_account_invites pi_account_invites_pi_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pi_account_invites
    ADD CONSTRAINT pi_account_invites_pi_user_id_fkey FOREIGN KEY (pi_user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: pi_account_invites pi_account_invites_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pi_account_invites
    ADD CONSTRAINT pi_account_invites_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: pi_account_invites pi_account_invites_provisioned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.pi_account_invites
    ADD CONSTRAINT pi_account_invites_provisioned_by_fkey FOREIGN KEY (provisioned_by) REFERENCES public.users(id);


--
-- Name: planned_experiments planned_experiments_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.planned_experiments
    ADD CONSTRAINT planned_experiments_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: planned_experiments planned_experiments_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.planned_experiments
    ADD CONSTRAINT planned_experiments_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE SET NULL;


--
-- Name: product_categories product_categories_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_categories
    ADD CONSTRAINT product_categories_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.product_categories(id);


--
-- Name: product_uom_conversions product_uom_conversions_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.product_uom_conversions
    ADD CONSTRAINT product_uom_conversions_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: products products_category_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.product_categories(id);


--
-- Name: products products_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.products
    ADD CONSTRAINT products_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: protocol_activities protocol_activities_actor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_activities
    ADD CONSTRAINT protocol_activities_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: protocol_activities protocol_activities_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_activities
    ADD CONSTRAINT protocol_activities_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: protocol_ai_reviews protocol_ai_reviews_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_ai_reviews
    ADD CONSTRAINT protocol_ai_reviews_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: protocol_ai_reviews protocol_ai_reviews_protocol_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_ai_reviews
    ADD CONSTRAINT protocol_ai_reviews_protocol_version_id_fkey FOREIGN KEY (protocol_version_id) REFERENCES public.protocol_versions(id);


--
-- Name: protocol_ai_reviews protocol_ai_reviews_triggered_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_ai_reviews
    ADD CONSTRAINT protocol_ai_reviews_triggered_by_fkey FOREIGN KEY (triggered_by) REFERENCES public.users(id);


--
-- Name: protocol_attachments protocol_attachments_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_attachments
    ADD CONSTRAINT protocol_attachments_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: protocol_attachments protocol_attachments_protocol_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_attachments
    ADD CONSTRAINT protocol_attachments_protocol_version_id_fkey FOREIGN KEY (protocol_version_id) REFERENCES public.protocol_versions(id) ON DELETE CASCADE;


--
-- Name: protocol_attachments protocol_attachments_uploaded_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_attachments
    ADD CONSTRAINT protocol_attachments_uploaded_by_fkey FOREIGN KEY (uploaded_by) REFERENCES public.users(id);


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_notice_attachment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_notice_attachment_id_fkey FOREIGN KEY (notice_attachment_id) REFERENCES public.attachments(id);


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_notice_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_notice_id_fkey FOREIGN KEY (notice_id) REFERENCES public.application_notices(id);


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_signature_id_fkey FOREIGN KEY (signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: protocol_notice_acknowledgements protocol_notice_acknowledgements_signer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_notice_acknowledgements
    ADD CONSTRAINT protocol_notice_acknowledgements_signer_id_fkey FOREIGN KEY (signer_id) REFERENCES public.users(id);


--
-- Name: protocol_template_versions protocol_template_versions_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_template_versions
    ADD CONSTRAINT protocol_template_versions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: protocol_versions protocol_versions_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_versions
    ADD CONSTRAINT protocol_versions_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: protocol_versions protocol_versions_submitted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocol_versions
    ADD CONSTRAINT protocol_versions_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id);


--
-- Name: protocols protocols_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocols
    ADD CONSTRAINT protocols_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: protocols protocols_pi_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocols
    ADD CONSTRAINT protocols_pi_user_id_fkey FOREIGN KEY (pi_user_id) REFERENCES public.users(id);


--
-- Name: protocols protocols_study_director_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.protocols
    ADD CONSTRAINT protocols_study_director_user_id_fkey FOREIGN KEY (study_director_user_id) REFERENCES public.users(id);


--
-- Name: qa_audit_schedules qa_audit_schedules_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_audit_schedules
    ADD CONSTRAINT qa_audit_schedules_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: qa_capa qa_capa_assignee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_capa
    ADD CONSTRAINT qa_capa_assignee_id_fkey FOREIGN KEY (assignee_id) REFERENCES public.users(id);


--
-- Name: qa_capa qa_capa_nc_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_capa
    ADD CONSTRAINT qa_capa_nc_id_fkey FOREIGN KEY (nc_id) REFERENCES public.qa_non_conformances(id) ON DELETE CASCADE;


--
-- Name: qa_inspection_items qa_inspection_items_inspection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_inspection_items
    ADD CONSTRAINT qa_inspection_items_inspection_id_fkey FOREIGN KEY (inspection_id) REFERENCES public.qa_inspections(id) ON DELETE CASCADE;


--
-- Name: qa_inspections qa_inspections_inspector_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_inspections
    ADD CONSTRAINT qa_inspections_inspector_id_fkey FOREIGN KEY (inspector_id) REFERENCES public.users(id);


--
-- Name: qa_non_conformances qa_non_conformances_assignee_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_non_conformances
    ADD CONSTRAINT qa_non_conformances_assignee_id_fkey FOREIGN KEY (assignee_id) REFERENCES public.users(id);


--
-- Name: qa_non_conformances qa_non_conformances_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_non_conformances
    ADD CONSTRAINT qa_non_conformances_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: qa_non_conformances qa_non_conformances_related_inspection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_non_conformances
    ADD CONSTRAINT qa_non_conformances_related_inspection_id_fkey FOREIGN KEY (related_inspection_id) REFERENCES public.qa_inspections(id);


--
-- Name: qa_schedule_items qa_schedule_items_related_inspection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_schedule_items
    ADD CONSTRAINT qa_schedule_items_related_inspection_id_fkey FOREIGN KEY (related_inspection_id) REFERENCES public.qa_inspections(id);


--
-- Name: qa_schedule_items qa_schedule_items_responsible_person_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_schedule_items
    ADD CONSTRAINT qa_schedule_items_responsible_person_id_fkey FOREIGN KEY (responsible_person_id) REFERENCES public.users(id);


--
-- Name: qa_schedule_items qa_schedule_items_schedule_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_schedule_items
    ADD CONSTRAINT qa_schedule_items_schedule_id_fkey FOREIGN KEY (schedule_id) REFERENCES public.qa_audit_schedules(id) ON DELETE CASCADE;


--
-- Name: qa_sop_acknowledgments qa_sop_acknowledgments_sop_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_acknowledgments
    ADD CONSTRAINT qa_sop_acknowledgments_sop_id_fkey FOREIGN KEY (sop_id) REFERENCES public.qa_sop_documents(id) ON DELETE CASCADE;


--
-- Name: qa_sop_acknowledgments qa_sop_acknowledgments_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_acknowledgments
    ADD CONSTRAINT qa_sop_acknowledgments_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: qa_sop_documents qa_sop_documents_approved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_documents
    ADD CONSTRAINT qa_sop_documents_approved_by_fkey FOREIGN KEY (approved_by) REFERENCES public.users(id);


--
-- Name: qa_sop_documents qa_sop_documents_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_documents
    ADD CONSTRAINT qa_sop_documents_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: qa_sop_documents qa_sop_documents_reviewed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.qa_sop_documents
    ADD CONSTRAINT qa_sop_documents_reviewed_by_fkey FOREIGN KEY (reviewed_by) REFERENCES public.users(id);


--
-- Name: record_annotations record_annotations_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.record_annotations
    ADD CONSTRAINT record_annotations_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: record_annotations record_annotations_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.record_annotations
    ADD CONSTRAINT record_annotations_signature_id_fkey FOREIGN KEY (signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: record_versions record_versions_changed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.record_versions
    ADD CONSTRAINT record_versions_changed_by_fkey FOREIGN KEY (changed_by) REFERENCES public.users(id);


--
-- Name: reference_standards reference_standards_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.reference_standards
    ADD CONSTRAINT reference_standards_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: refresh_tokens refresh_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: report_history report_history_generated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.report_history
    ADD CONSTRAINT report_history_generated_by_fkey FOREIGN KEY (generated_by) REFERENCES public.users(id);


--
-- Name: report_history report_history_scheduled_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.report_history
    ADD CONSTRAINT report_history_scheduled_report_id_fkey FOREIGN KEY (scheduled_report_id) REFERENCES public.scheduled_reports(id) ON DELETE SET NULL;


--
-- Name: review_assignments review_assignments_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_assignments
    ADD CONSTRAINT review_assignments_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id);


--
-- Name: review_assignments review_assignments_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_assignments
    ADD CONSTRAINT review_assignments_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: review_assignments review_assignments_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_assignments
    ADD CONSTRAINT review_assignments_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: review_comments review_comments_drafted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_drafted_by_fkey FOREIGN KEY (drafted_by) REFERENCES public.users(id);


--
-- Name: review_comments review_comments_parent_comment_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_parent_comment_id_fkey FOREIGN KEY (parent_comment_id) REFERENCES public.review_comments(id) ON DELETE CASCADE;


--
-- Name: review_comments review_comments_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id);


--
-- Name: review_comments review_comments_protocol_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_protocol_version_id_fkey FOREIGN KEY (protocol_version_id) REFERENCES public.protocol_versions(id) ON DELETE CASCADE;


--
-- Name: review_comments review_comments_replied_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_replied_by_fkey FOREIGN KEY (replied_by) REFERENCES public.users(id);


--
-- Name: review_comments review_comments_resolved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_resolved_by_fkey FOREIGN KEY (resolved_by) REFERENCES public.users(id);


--
-- Name: review_comments review_comments_reviewer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_comments
    ADD CONSTRAINT review_comments_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id);


--
-- Name: review_round_history review_round_history_actor_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_round_history
    ADD CONSTRAINT review_round_history_actor_id_fkey FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: review_round_history review_round_history_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.review_round_history
    ADD CONSTRAINT review_round_history_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: risk_register risk_register_owner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.risk_register
    ADD CONSTRAINT risk_register_owner_id_fkey FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: role_permissions role_permissions_permission_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public.permissions(id) ON DELETE CASCADE;


--
-- Name: role_permissions role_permissions_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.role_permissions
    ADD CONSTRAINT role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;


--
-- Name: scheduled_reports scheduled_reports_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.scheduled_reports
    ADD CONSTRAINT scheduled_reports_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: security_alerts security_alerts_login_event_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_alerts
    ADD CONSTRAINT security_alerts_login_event_id_fkey FOREIGN KEY (login_event_id) REFERENCES public.login_events(id);


--
-- Name: security_alerts security_alerts_resolved_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_alerts
    ADD CONSTRAINT security_alerts_resolved_by_fkey FOREIGN KEY (resolved_by) REFERENCES public.users(id);


--
-- Name: security_alerts security_alerts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.security_alerts
    ADD CONSTRAINT security_alerts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: signature_bridge_sessions signature_bridge_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.signature_bridge_sessions
    ADD CONSTRAINT signature_bridge_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: sku_subcategories sku_subcategories_category_code_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sku_subcategories
    ADD CONSTRAINT sku_subcategories_category_code_fkey FOREIGN KEY (category_code) REFERENCES public.sku_categories(code) ON DELETE CASCADE;


--
-- Name: species species_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.species
    ADD CONSTRAINT species_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.species(id);


--
-- Name: stock_ledger stock_ledger_doc_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_ledger
    ADD CONSTRAINT stock_ledger_doc_id_fkey FOREIGN KEY (doc_id) REFERENCES public.documents(id);


--
-- Name: stock_ledger stock_ledger_line_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_ledger
    ADD CONSTRAINT stock_ledger_line_id_fkey FOREIGN KEY (line_id) REFERENCES public.document_lines(id);


--
-- Name: stock_ledger stock_ledger_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_ledger
    ADD CONSTRAINT stock_ledger_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id);


--
-- Name: stock_ledger stock_ledger_storage_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_ledger
    ADD CONSTRAINT stock_ledger_storage_location_id_fkey FOREIGN KEY (storage_location_id) REFERENCES public.storage_locations(id);


--
-- Name: stock_ledger stock_ledger_warehouse_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.stock_ledger
    ADD CONSTRAINT stock_ledger_warehouse_id_fkey FOREIGN KEY (warehouse_id) REFERENCES public.warehouses(id);


--
-- Name: storage_location_inventory storage_location_inventory_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_location_inventory
    ADD CONSTRAINT storage_location_inventory_product_id_fkey FOREIGN KEY (product_id) REFERENCES public.products(id) ON DELETE CASCADE;


--
-- Name: storage_location_inventory storage_location_inventory_storage_location_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_location_inventory
    ADD CONSTRAINT storage_location_inventory_storage_location_id_fkey FOREIGN KEY (storage_location_id) REFERENCES public.storage_locations(id) ON DELETE CASCADE;


--
-- Name: storage_locations storage_locations_warehouse_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.storage_locations
    ADD CONSTRAINT storage_locations_warehouse_id_fkey FOREIGN KEY (warehouse_id) REFERENCES public.warehouses(id) ON DELETE CASCADE;


--
-- Name: study_final_reports study_final_reports_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.study_final_reports
    ADD CONSTRAINT study_final_reports_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id);


--
-- Name: study_final_reports study_final_reports_qau_signed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.study_final_reports
    ADD CONSTRAINT study_final_reports_qau_signed_by_fkey FOREIGN KEY (qau_signed_by) REFERENCES public.users(id);


--
-- Name: study_final_reports study_final_reports_signature_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.study_final_reports
    ADD CONSTRAINT study_final_reports_signature_id_fkey FOREIGN KEY (signature_id) REFERENCES public.electronic_signatures(id);


--
-- Name: study_final_reports study_final_reports_signed_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.study_final_reports
    ADD CONSTRAINT study_final_reports_signed_by_fkey FOREIGN KEY (signed_by) REFERENCES public.users(id);


--
-- Name: surgery_vet_reads surgery_vet_reads_vet_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.surgery_vet_reads
    ADD CONSTRAINT surgery_vet_reads_vet_user_id_fkey FOREIGN KEY (vet_user_id) REFERENCES public.users(id);


--
-- Name: system_settings system_settings_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.system_settings
    ADD CONSTRAINT system_settings_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: training_records training_records_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.training_records
    ADD CONSTRAINT training_records_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: transfer_vet_evaluations transfer_vet_evaluations_transfer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transfer_vet_evaluations
    ADD CONSTRAINT transfer_vet_evaluations_transfer_id_fkey FOREIGN KEY (transfer_id) REFERENCES public.animal_transfers(id);


--
-- Name: transfer_vet_evaluations transfer_vet_evaluations_vet_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.transfer_vet_evaluations
    ADD CONSTRAINT transfer_vet_evaluations_vet_id_fkey FOREIGN KEY (vet_id) REFERENCES public.users(id);


--
-- Name: treatment_drug_options treatment_drug_options_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.treatment_drug_options
    ADD CONSTRAINT treatment_drug_options_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: treatment_drug_options treatment_drug_options_erp_product_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.treatment_drug_options
    ADD CONSTRAINT treatment_drug_options_erp_product_id_fkey FOREIGN KEY (erp_product_id) REFERENCES public.products(id);


--
-- Name: user_activity_aggregates user_activity_aggregates_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_activity_aggregates
    ADD CONSTRAINT user_activity_aggregates_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_activity_logs user_activity_logs_actor_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE public.user_activity_logs
    ADD CONSTRAINT user_activity_logs_actor_user_id_fkey FOREIGN KEY (actor_user_id) REFERENCES public.users(id);


--
-- Name: user_activity_logs user_activity_logs_impersonated_by_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE public.user_activity_logs
    ADD CONSTRAINT user_activity_logs_impersonated_by_user_id_fkey FOREIGN KEY (impersonated_by_user_id) REFERENCES public.users(id);


--
-- Name: user_aup_profiles user_aup_profiles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_aup_profiles
    ADD CONSTRAINT user_aup_profiles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_mcp_keys user_mcp_keys_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_mcp_keys
    ADD CONSTRAINT user_mcp_keys_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_preferences user_preferences_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_preferences
    ADD CONSTRAINT user_preferences_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_protocols user_protocols_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_protocols
    ADD CONSTRAINT user_protocols_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id);


--
-- Name: user_protocols user_protocols_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_protocols
    ADD CONSTRAINT user_protocols_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: user_protocols user_protocols_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_protocols
    ADD CONSTRAINT user_protocols_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_roles user_roles_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id);


--
-- Name: user_roles user_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;


--
-- Name: user_roles user_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_roles
    ADD CONSTRAINT user_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_sessions user_sessions_refresh_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_refresh_token_id_fkey FOREIGN KEY (refresh_token_id) REFERENCES public.refresh_tokens(id) ON DELETE SET NULL;


--
-- Name: user_sessions user_sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_sessions
    ADD CONSTRAINT user_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: users users_department_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_department_id_fkey FOREIGN KEY (department_id) REFERENCES public.departments(id) ON DELETE SET NULL;


--
-- Name: vet_patrol_entries vet_patrol_entries_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entries
    ADD CONSTRAINT vet_patrol_entries_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id);


--
-- Name: vet_patrol_entries vet_patrol_entries_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entries
    ADD CONSTRAINT vet_patrol_entries_report_id_fkey FOREIGN KEY (report_id) REFERENCES public.vet_patrol_reports(id) ON DELETE CASCADE;


--
-- Name: vet_patrol_entry_animals vet_patrol_entry_animals_animal_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entry_animals
    ADD CONSTRAINT vet_patrol_entry_animals_animal_id_fkey FOREIGN KEY (animal_id) REFERENCES public.animals(id) ON DELETE CASCADE;


--
-- Name: vet_patrol_entry_animals vet_patrol_entry_animals_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entry_animals
    ADD CONSTRAINT vet_patrol_entry_animals_entry_id_fkey FOREIGN KEY (entry_id) REFERENCES public.vet_patrol_entries(id) ON DELETE CASCADE;


--
-- Name: vet_patrol_entry_photos vet_patrol_entry_photos_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entry_photos
    ADD CONSTRAINT vet_patrol_entry_photos_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: vet_patrol_entry_photos vet_patrol_entry_photos_entry_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_entry_photos
    ADD CONSTRAINT vet_patrol_entry_photos_entry_id_fkey FOREIGN KEY (entry_id) REFERENCES public.vet_patrol_entries(id) ON DELETE CASCADE;


--
-- Name: vet_patrol_photos vet_patrol_photos_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_photos
    ADD CONSTRAINT vet_patrol_photos_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: vet_patrol_photos vet_patrol_photos_report_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_photos
    ADD CONSTRAINT vet_patrol_photos_report_id_fkey FOREIGN KEY (report_id) REFERENCES public.vet_patrol_reports(id) ON DELETE CASCADE;


--
-- Name: vet_patrol_reports vet_patrol_reports_acknowledged_by_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_reports
    ADD CONSTRAINT vet_patrol_reports_acknowledged_by_id_fkey FOREIGN KEY (acknowledged_by_id) REFERENCES public.users(id);


--
-- Name: vet_patrol_reports vet_patrol_reports_created_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_reports
    ADD CONSTRAINT vet_patrol_reports_created_by_fkey FOREIGN KEY (created_by) REFERENCES public.users(id);


--
-- Name: vet_patrol_reports vet_patrol_reports_follow_up_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_reports
    ADD CONSTRAINT vet_patrol_reports_follow_up_user_id_fkey FOREIGN KEY (follow_up_user_id) REFERENCES public.users(id);


--
-- Name: vet_patrol_reports vet_patrol_reports_updated_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_patrol_reports
    ADD CONSTRAINT vet_patrol_reports_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public.users(id);


--
-- Name: vet_review_assignments vet_review_assignments_assigned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_review_assignments
    ADD CONSTRAINT vet_review_assignments_assigned_by_fkey FOREIGN KEY (assigned_by) REFERENCES public.users(id);


--
-- Name: vet_review_assignments vet_review_assignments_protocol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_review_assignments
    ADD CONSTRAINT vet_review_assignments_protocol_id_fkey FOREIGN KEY (protocol_id) REFERENCES public.protocols(id) ON DELETE CASCADE;


--
-- Name: vet_review_assignments vet_review_assignments_vet_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.vet_review_assignments
    ADD CONSTRAINT vet_review_assignments_vet_id_fkey FOREIGN KEY (vet_id) REFERENCES public.users(id);


--
-- Name: zones zones_building_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.zones
    ADD CONSTRAINT zones_building_id_fkey FOREIGN KEY (building_id) REFERENCES public.buildings(id);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: -
--

GRANT USAGE ON SCHEMA public TO grafana_readonly;


--
-- Name: TABLE data_retention_policies; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.data_retention_policies TO grafana_readonly;


--
-- Name: TABLE euthanasia_byproduct_samples; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.euthanasia_byproduct_samples TO grafana_readonly;


--
-- Name: TABLE event_outbox; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.event_outbox TO grafana_readonly;


--
-- Name: TABLE invitation_roles; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.invitation_roles TO grafana_readonly;


--
-- Name: TABLE ip_blocklist; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.ip_blocklist TO grafana_readonly;


--
-- Name: TABLE login_events; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.login_events TO grafana_readonly;


--
-- Name: TABLE message_attachments; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.message_attachments TO grafana_readonly;


--
-- Name: TABLE message_thread_participants; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.message_thread_participants TO grafana_readonly;


--
-- Name: TABLE message_threads; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.message_threads TO grafana_readonly;


--
-- Name: TABLE messages; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.messages TO grafana_readonly;


--
-- Name: TABLE pdf_artifacts; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.pdf_artifacts TO grafana_readonly;


--
-- Name: TABLE security_alerts; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.security_alerts TO grafana_readonly;


--
-- Name: TABLE signature_bridge_sessions; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.signature_bridge_sessions TO grafana_readonly;


--
-- Name: TABLE user_activity_logs; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.user_activity_logs TO grafana_readonly;


--
-- Name: TABLE user_sessions; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.user_sessions TO grafana_readonly;


--
-- Name: TABLE vet_patrol_entry_animals; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.vet_patrol_entry_animals TO grafana_readonly;


--
-- Name: TABLE vet_patrol_entry_photos; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.vet_patrol_entry_photos TO grafana_readonly;


--
-- Name: TABLE vet_patrol_photos; Type: ACL; Schema: public; Owner: -
--

GRANT SELECT ON TABLE public.vet_patrol_photos TO grafana_readonly;


--
-- PostgreSQL database dump complete
--
