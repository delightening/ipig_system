use axum::{
    routing::{get, post, put},
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
}
