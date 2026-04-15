use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// 管理後台路由：系統設定、配置警告、稽核軌跡、QAU、SSE、通知路由、治療藥物
pub fn routes() -> Router<AppState> {
    Router::new()
        // Admin System Settings
        .route(
            "/admin/system-settings",
            get(handlers::get_system_settings).put(handlers::update_system_settings),
        )
        // Admin SMTP Test Email
        .route(
            "/admin/system-settings/test-email",
            post(handlers::send_test_email),
        )
        // IACUC 通知測試
        .route(
            "/admin/iacuc/test-notification",
            post(handlers::send_iacuc_test_notification),
        )
        // Admin Config Warnings
        .route(
            "/admin/config-warnings",
            get(handlers::get_config_warnings),
        )
        // Admin Audit Trail
        .route(
            "/admin/audit-logs/export",
            get(handlers::export_audit_logs),
        )
        .route(
            "/admin/data-export",
            get(handlers::data_export::full_database_export),
        )
        .route(
            "/admin/data-import",
            post(handlers::data_export::full_database_import),
        )
        .route(
            "/admin/audit/activities",
            get(handlers::list_activity_logs),
        )
        .route(
            "/admin/audit/activities/export",
            get(handlers::export_activity_logs),
        )
        .route(
            "/admin/audit/activities/user/:user_id",
            get(handlers::get_user_activity_timeline),
        )
        .route(
            "/admin/audit/activities/entity/:entity_type/:entity_id",
            get(handlers::get_entity_history),
        )
        .route("/admin/audit/logins", get(handlers::list_login_events))
        .route("/admin/audit/sessions", get(handlers::list_sessions))
        .route(
            "/admin/audit/sessions/:id/logout",
            post(handlers::force_logout_session),
        )
        .route(
            "/admin/audit/alerts",
            get(handlers::list_security_alerts),
        )
        .route(
            "/admin/audit/alerts/:id/resolve",
            post(handlers::resolve_security_alert),
        )
        .route(
            "/admin/audit/dashboard",
            get(handlers::get_audit_dashboard),
        )
        // R22-17: Security Events (SECURITY category only)
        .route(
            "/admin/audit/security-events",
            get(handlers::list_security_events),
        )
        // 最近告警（輪詢用）
        .route(
            "/admin/audit/alerts/recent",
            get(handlers::list_security_alerts),
        )
        // QAU Dashboard
        .route("/qau/dashboard", get(handlers::get_qau_dashboard))
        // QA 稽查報告
        .route(
            "/qau/inspections",
            get(handlers::qa_plan::list_inspections).post(handlers::qa_plan::create_inspection),
        )
        .route(
            "/qau/inspections/:id",
            get(handlers::qa_plan::get_inspection).put(handlers::qa_plan::update_inspection),
        )
        // QA 不符合事項
        .route(
            "/qau/non-conformances",
            get(handlers::qa_plan::list_non_conformances)
                .post(handlers::qa_plan::create_non_conformance),
        )
        .route(
            "/qau/non-conformances/:id",
            get(handlers::qa_plan::get_non_conformance)
                .put(handlers::qa_plan::update_non_conformance),
        )
        .route(
            "/qau/non-conformances/:nc_id/capa",
            post(handlers::qa_plan::create_capa),
        )
        .route(
            "/qau/non-conformances/:nc_id/capa/:capa_id",
            put(handlers::qa_plan::update_capa),
        )
        // QA SOP 文件
        .route(
            "/qau/sop",
            get(handlers::qa_plan::list_sop_documents).post(handlers::qa_plan::create_sop_document),
        )
        .route(
            "/qau/sop/:id",
            get(handlers::qa_plan::get_sop_document).put(handlers::qa_plan::update_sop_document),
        )
        .route(
            "/qau/sop/:id/acknowledge",
            post(handlers::qa_plan::acknowledge_sop),
        )
        .route(
            "/qau/sop/:id/download",
            get(handlers::download_sop_document),
        )
        // QA 稽查排程
        .route(
            "/qau/schedules",
            get(handlers::qa_plan::list_schedules).post(handlers::qa_plan::create_schedule),
        )
        .route(
            "/qau/schedules/:id",
            get(handlers::qa_plan::get_schedule).put(handlers::qa_plan::update_schedule),
        )
        .route(
            "/qau/schedules/:schedule_id/items/:item_id",
            put(handlers::qa_plan::update_schedule_item),
        )
        // 安全警報 polling 端點（取代 SSE）
        .route(
            "/admin/audit/alerts/recent",
            get(handlers::get_recent_alerts),
        )
        // Admin Notification Routing
        .route(
            "/admin/notification-routing",
            get(handlers::list_notification_routing)
                .post(handlers::create_notification_routing),
        )
        .route(
            "/admin/notification-routing/event-types",
            get(handlers::list_available_event_types),
        )
        .route(
            "/admin/notification-routing/roles",
            get(handlers::list_available_roles),
        )
        .route(
            "/admin/notification-routing/:id",
            put(handlers::update_notification_routing)
                .delete(handlers::delete_notification_routing),
        )
        .route(
            "/admin/notification-routing/:id/delete",
            post(handlers::delete_notification_routing),
        )
        // Expiry Notification Config
        .route(
            "/admin/expiry-config",
            get(handlers::get_expiry_config).put(handlers::update_expiry_config),
        )
        // Treatment Drug Options
        .route(
            "/treatment-drugs",
            get(handlers::treatment_drug::list_treatment_drugs),
        )
        .route(
            "/admin/treatment-drugs",
            get(handlers::treatment_drug::admin_list_treatment_drugs)
                .post(handlers::treatment_drug::create_treatment_drug),
        )
        .route(
            "/admin/treatment-drugs/:id",
            put(handlers::treatment_drug::update_treatment_drug)
                .delete(handlers::treatment_drug::delete_treatment_drug),
        )
        .route(
            "/admin/treatment-drugs/:id/delete",
            post(handlers::treatment_drug::delete_treatment_drug),
        )
        .route(
            "/admin/treatment-drugs/import-erp",
            post(handlers::treatment_drug::import_treatment_drugs_from_erp),
        )
        // ============================================================
        // GLP 合規模組 Routes
        // ============================================================
        // Reference Standards (參考標準器)
        .route(
            "/admin/reference-standards",
            get(handlers::glp_compliance::list_reference_standards)
                .post(handlers::glp_compliance::create_reference_standard),
        )
        .route(
            "/admin/reference-standards/:id",
            get(handlers::glp_compliance::get_reference_standard)
                .put(handlers::glp_compliance::update_reference_standard),
        )
        // Controlled Documents (文件控制)
        .route(
            "/admin/documents",
            get(handlers::glp_compliance::list_controlled_documents)
                .post(handlers::glp_compliance::create_controlled_document),
        )
        .route(
            "/admin/documents/:id",
            get(handlers::glp_compliance::get_controlled_document)
                .put(handlers::glp_compliance::update_controlled_document),
        )
        .route(
            "/admin/documents/:id/approve",
            post(handlers::glp_compliance::approve_controlled_document),
        )
        .route(
            "/admin/documents/:id/revisions",
            post(handlers::glp_compliance::create_revision),
        )
        .route(
            "/admin/documents/:id/acknowledge",
            post(handlers::glp_compliance::acknowledge_document),
        )
        // Management Reviews (管理審查)
        .route(
            "/admin/management-reviews",
            get(handlers::glp_compliance::list_management_reviews)
                .post(handlers::glp_compliance::create_management_review),
        )
        .route(
            "/admin/management-reviews/:id",
            get(handlers::glp_compliance::get_management_review)
                .put(handlers::glp_compliance::update_management_review),
        )
        // Risk Register (風險管理)
        .route(
            "/admin/risks",
            get(handlers::glp_compliance::list_risks)
                .post(handlers::glp_compliance::create_risk),
        )
        .route(
            "/admin/risks/:id",
            get(handlers::glp_compliance::get_risk)
                .put(handlers::glp_compliance::update_risk),
        )
        // Change Requests (變更控制)
        .route(
            "/admin/change-requests",
            get(handlers::glp_compliance::list_change_requests)
                .post(handlers::glp_compliance::create_change_request),
        )
        .route(
            "/admin/change-requests/:id",
            get(handlers::glp_compliance::get_change_request)
                .put(handlers::glp_compliance::update_change_request),
        )
        .route(
            "/admin/change-requests/:id/approve",
            post(handlers::glp_compliance::approve_change_request),
        )
        // Environment Monitoring (環境監控)
        .route(
            "/admin/env-monitoring/points",
            get(handlers::glp_compliance::list_monitoring_points)
                .post(handlers::glp_compliance::create_monitoring_point),
        )
        .route(
            "/admin/env-monitoring/points/:id",
            get(handlers::glp_compliance::get_monitoring_point)
                .put(handlers::glp_compliance::update_monitoring_point),
        )
        .route(
            "/admin/env-monitoring/readings",
            get(handlers::glp_compliance::list_readings)
                .post(handlers::glp_compliance::create_reading),
        )
        // Competency Assessments (能力評鑑)
        .route(
            "/admin/competency-assessments",
            get(handlers::glp_compliance::list_competency_assessments)
                .post(handlers::glp_compliance::create_competency_assessment),
        )
        .route(
            "/admin/competency-assessments/:id",
            put(handlers::glp_compliance::update_competency_assessment),
        )
        .route(
            "/admin/training-requirements",
            get(handlers::glp_compliance::list_training_requirements)
                .post(handlers::glp_compliance::create_training_requirement),
        )
        .route(
            "/admin/training-requirements/:id",
            delete(handlers::glp_compliance::delete_training_requirement),
        )
        // Study Final Reports (最終報告)
        .route(
            "/admin/study-reports",
            get(handlers::glp_compliance::list_study_reports)
                .post(handlers::glp_compliance::create_study_report),
        )
        .route(
            "/admin/study-reports/:id",
            get(handlers::glp_compliance::get_study_report)
                .put(handlers::glp_compliance::update_study_report),
        )
        // Formulation Records (配製紀錄)
        .route(
            "/admin/formulation-records",
            get(handlers::glp_compliance::list_formulation_records)
                .post(handlers::glp_compliance::create_formulation_record),
        )
}
