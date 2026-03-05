use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::middleware::etag::etag_middleware;
use crate::middleware::rate_limiter::{
    api_rate_limit_middleware, auth_rate_limit_middleware, upload_rate_limit_middleware,
    write_rate_limit_middleware,
};
use crate::{handlers, middleware::auth_middleware, middleware::csrf_middleware, AppState};

pub fn api_routes(state: AppState) -> Router {
    // Public routes (no auth required) - 移除公開註冊，改為私域註冊
    let public_routes = Router::new()
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/forgot-password", post(handlers::forgot_password))
        .route(
            "/auth/reset-password",
            post(handlers::reset_password_with_token),
        )
        .route("/auth/2fa/verify", post(handlers::verify_2fa_login))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_rate_limit_middleware,
        ))
        .with_state(state.clone());

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Auth
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/confirm-password", post(handlers::confirm_password))
        .route("/auth/2fa/setup", post(handlers::setup_2fa))
        .route("/auth/2fa/confirm", post(handlers::confirm_2fa_setup))
        .route("/auth/2fa/disable", post(handlers::disable_2fa))
        .route("/auth/stop-impersonate", post(handlers::stop_impersonate))
        .route("/auth/heartbeat", post(handlers::heartbeat))
        .route("/me", get(handlers::me).put(handlers::update_me))
        .route("/me/export", get(handlers::export_me))
        .route("/me/account", delete(handlers::delete_me_account))
        .route("/me/account/delete", post(handlers::delete_me_account))
        .route("/me/password", put(handlers::change_own_password))
        // User Preferences
        .route("/me/preferences", get(handlers::get_all_preferences))
        .route(
            "/me/preferences/:key",
            get(handlers::get_preference)
                .put(handlers::upsert_preference)
                .delete(handlers::delete_preference),
        )
        .route("/me/preferences/:key/delete", post(handlers::delete_preference))
        // Users
        .route(
            "/users",
            get(handlers::list_users).post(handlers::create_user),
        )
        .route(
            "/users/:id",
            get(handlers::get_user)
                .put(handlers::update_user)
                .delete(handlers::delete_user),
        )
        .route("/users/:id/delete", post(handlers::delete_user))
        .route("/users/:id/password", put(handlers::reset_user_password))
        .route("/users/:id/impersonate", post(handlers::impersonate_user))
        // Roles
        .route(
            "/roles",
            get(handlers::list_roles).post(handlers::create_role),
        )
        .route(
            "/roles/:id",
            get(handlers::get_role)
                .put(handlers::update_role)
                .delete(handlers::delete_role),
        )
        .route("/roles/:id/delete", post(handlers::delete_role))
        .route("/permissions", get(handlers::list_permissions))
        // Warehouses
        .route(
            "/warehouses",
            get(handlers::list_warehouses).post(handlers::create_warehouse),
        )
        .route(
            "/warehouses/import",
            post(handlers::import_warehouses),
        )
        .route(
            "/warehouses/import/template",
            get(handlers::download_warehouse_import_template),
        )
        .route(
            "/warehouses/:id",
            get(handlers::get_warehouse)
                .put(handlers::update_warehouse)
                .delete(handlers::delete_warehouse),
        )
        .route(
            "/warehouses/:id/delete",
            post(handlers::delete_warehouse),
        )
        .route(
            "/warehouses/:id/layout",
            put(handlers::storage_location::update_warehouse_layout),
        )
        // Storage Locations (倉庫儲位)
        .route(
            "/storage-locations",
            get(handlers::storage_location::list_storage_locations)
                .post(handlers::storage_location::create_storage_location),
        )
        .route(
            "/storage-locations/:id",
            get(handlers::storage_location::get_storage_location)
                .put(handlers::storage_location::update_storage_location)
                .delete(handlers::storage_location::delete_storage_location),
        )
        .route(
            "/storage-locations/:id/delete",
            post(handlers::storage_location::delete_storage_location),
        )
        .route(
            "/storage-locations/:id/inventory",
            get(handlers::storage_location::get_storage_location_inventory)
                .post(handlers::storage_location::create_storage_location_inventory_item),
        )
        .route(
            "/storage-locations/inventory/:item_id",
            put(handlers::storage_location::update_storage_location_inventory_item),
        )
        .route(
            "/storage-locations/inventory/:item_id/transfer",
            post(handlers::storage_location::transfer_storage_location_inventory),
        )
        .route(
            "/storage-locations/generate-code/:warehouse_id",
            get(handlers::storage_location::generate_storage_location_code),
        )
        // Products
        .route(
            "/products",
            get(handlers::list_products).post(handlers::create_product),
        )
        .route(
            "/products/import",
            post(handlers::import_products),
        )
        .route(
            "/products/import/check",
            post(handlers::check_product_import_duplicates),
        )
        .route(
            "/products/import/template",
            get(handlers::download_product_import_template),
        )
        .route(
            "/products/:id",
            get(handlers::get_product)
                .put(handlers::update_product)
                .delete(handlers::delete_product),
        )
        .route(
            "/products/:id/status",
            patch(handlers::update_product_status),
        )
        .route("/products/:id/delete", post(handlers::delete_product))
        .route(
            "/categories",
            get(handlers::list_categories).post(handlers::create_category),
        )
        // SKU (完整 API)
        .route("/sku/categories", get(handlers::get_sku_categories))
        .route(
            "/sku/categories/:code/subcategories",
            get(handlers::get_sku_subcategories),
        )
        .route("/sku/generate", post(handlers::generate_sku))
        .route("/sku/validate", post(handlers::validate_sku))
        .route("/skus/preview", post(handlers::preview_sku))
        .route(
            "/products/with-sku",
            post(handlers::create_product_with_sku),
        )
        // Partners
        .route(
            "/partners",
            get(handlers::list_partners).post(handlers::create_partner),
        )
        .route(
            "/partners/import",
            post(handlers::import_partners),
        )
        .route(
            "/partners/import/template",
            get(handlers::download_partner_import_template),
        )
        .route(
            "/partners/generate-code",
            get(handlers::generate_partner_code),
        )
        .route(
            "/partners/:id",
            get(handlers::get_partner)
                .put(handlers::update_partner)
                .delete(handlers::delete_partner),
        )
        .route("/partners/:id/delete", post(handlers::delete_partner))
        // Documents
        .route(
            "/documents",
            get(handlers::list_documents).post(handlers::create_document),
        )
        .route(
            "/documents/:id",
            get(handlers::get_document)
                .put(handlers::update_document)
                .delete(handlers::delete_document),
        )
        .route("/documents/:id/delete", post(handlers::delete_document))
        .route("/documents/:id/submit", post(handlers::submit_document))
        .route("/documents/:id/approve", post(handlers::approve_document))
        .route("/documents/:id/cancel", post(handlers::cancel_document))
        // Inventory
        .route("/inventory/on-hand", get(handlers::get_inventory_on_hand))
        .route("/inventory/ledger", get(handlers::get_stock_ledger))
        .route("/inventory/low-stock", get(handlers::get_low_stock_alerts))
        // Audit Logs
        .route("/audit-logs", get(handlers::list_audit_logs))
        // Reports
        .route(
            "/reports/stock-on-hand",
            get(handlers::get_stock_on_hand_report),
        )
        .route(
            "/reports/stock-ledger",
            get(handlers::get_stock_ledger_report),
        )
        .route(
            "/reports/purchase-lines",
            get(handlers::get_purchase_lines_report),
        )
        .route(
            "/reports/sales-lines",
            get(handlers::get_sales_lines_report),
        )
        .route(
            "/reports/cost-summary",
            get(handlers::get_cost_summary_report),
        )
        .route(
            "/reports/blood-test-cost",
            get(handlers::get_blood_test_cost_report),
        )
        .route(
            "/reports/blood-test-analysis",
            get(handlers::get_blood_test_analysis),
        )
        // Accounting / 會計
        .route(
            "/accounting/chart-of-accounts",
            get(handlers::get_chart_of_accounts),
        )
        .route(
            "/accounting/trial-balance",
            get(handlers::get_trial_balance),
        )
        .route(
            "/accounting/journal-entries",
            get(handlers::get_journal_entries),
        )
        .route(
            "/accounting/ap-aging",
            get(handlers::get_ap_aging),
        )
        .route(
            "/accounting/ar-aging",
            get(handlers::get_ar_aging),
        )
        .route(
            "/accounting/ap-payments",
            post(handlers::create_ap_payment),
        )
        .route(
            "/accounting/ar-receipts",
            post(handlers::create_ar_receipt),
        )
        // Protocols (AUP 審查系統)
        .route(
            "/protocols",
            get(handlers::list_protocols).post(handlers::create_protocol),
        )
        .route(
            "/protocols/:id",
            get(handlers::get_protocol).put(handlers::update_protocol),
        )
        .route("/protocols/:id/submit", post(handlers::submit_protocol))
        .route(
            "/protocols/:id/status",
            post(handlers::change_protocol_status),
        )
        .route(
            "/protocols/:id/versions",
            get(handlers::get_protocol_versions),
        )
        .route(
            "/protocols/:id/activities",
            get(handlers::get_protocol_activities),
        )
        .route(
            "/protocols/:id/animal-stats",
            get(handlers::get_protocol_animal_stats),
        )
        .route(
            "/protocols/:id/export-pdf",
            get(handlers::export_protocol_pdf),
        )
        // Review
        .route(
            "/reviews/assignments",
            get(handlers::list_review_assignments).post(handlers::assign_reviewer),
        )
        .route(
            "/reviews/comments",
            get(handlers::list_review_comments).post(handlers::create_review_comment),
        )
        .route(
            "/reviews/comments/:id/resolve",
            post(handlers::resolve_review_comment),
        )
        .route(
            "/reviews/comments/reply",
            post(handlers::reply_review_comment),
        )
        // Draft Reply (Coeditor 草稿回覆)
        .route("/reviews/comments/draft", post(handlers::save_reply_draft))
        .route(
            "/reviews/comments/:id/draft",
            get(handlers::get_reply_draft),
        )
        .route(
            "/reviews/comments/submit-draft",
            post(handlers::submit_reply_from_draft),
        )
        // Vet Review Form
        .route("/reviews/vet-form", post(handlers::save_vet_review_form))
        // Co-Editor Assignment
        .route(
            "/protocols/:id/co-editors",
            get(handlers::list_co_editors).post(handlers::assign_co_editor),
        )
        .route(
            "/protocols/:id/co-editors/:user_id",
            delete(handlers::remove_co_editor),
        )
        .route(
            "/protocols/:id/co-editors/:user_id/delete",
            post(handlers::remove_co_editor),
        )
        // My Projects
        .route("/my-projects", get(handlers::get_my_protocols))
        // Animal Sources
        .route(
            "/animal-sources",
            get(handlers::list_animal_sources).post(handlers::create_animal_source),
        )
        .route(
            "/animal-sources/:id",
            put(handlers::update_animal_source).delete(handlers::delete_animal_source),
        )
        .route("/animal-sources/:id/delete", post(handlers::delete_animal_source))
        // Animals (實驗動物管理系統)
        .route(
            "/animals",
            get(handlers::list_animals).post(handlers::create_animal),
        )
        .route("/animals/by-pen", get(handlers::list_animals_by_pen))
        .route(
            "/animals/batch/assign",
            post(handlers::batch_assign_animals),
        )
        .route("/animals/vet-comments", get(handlers::get_vet_comments))
        .route(
            "/animals/:id",
            get(handlers::get_animal)
                .put(handlers::update_animal)
                .delete(handlers::delete_animal),
        )
        .route("/animals/:id/delete", post(handlers::delete_animal))
        .route("/animals/:id/events", get(handlers::get_animal_events))
        .route(
            "/animals/:id/vet-read",
            post(handlers::mark_animal_vet_read),
        )
        // 動物欄位修正申請（耳號、出生日期、性別、品種需 admin 批准）
        .route(
            "/animals/:id/field-corrections",
            get(handlers::list_animal_field_corrections)
                .post(handlers::create_animal_field_correction_request),
        )
        .route(
            "/admin/animal-field-corrections/pending",
            get(handlers::list_pending_animal_field_corrections),
        )
        .route(
            "/admin/animal-field-corrections/:id/review",
            post(handlers::review_animal_field_correction),
        )
        // Animal Records - Observations
        .route(
            "/animals/:id/observations",
            get(handlers::list_animal_observations).post(handlers::create_animal_observation),
        )
        .route(
            "/animals/:id/observations/with-recommendations",
            get(handlers::list_animal_observations_with_recommendations),
        )
        .route(
            "/animals/:id/observations/copy",
            post(handlers::copy_animal_observation),
        )
        .route(
            "/observations/:id",
            get(handlers::get_animal_observation)
                .put(handlers::update_animal_observation)
                .delete(handlers::delete_animal_observation),
        )
        .route("/observations/:id/delete", post(handlers::delete_animal_observation))
        .route(
            "/observations/:id/vet-read",
            post(handlers::mark_observation_vet_read),
        )
        .route(
            "/observations/:id/versions",
            get(handlers::get_observation_versions),
        )
        // Animal Records - Surgeries
        .route(
            "/animals/:id/surgeries",
            get(handlers::list_animal_surgeries).post(handlers::create_animal_surgery),
        )
        .route(
            "/animals/:id/surgeries/with-recommendations",
            get(handlers::list_animal_surgeries_with_recommendations),
        )
        .route(
            "/animals/:id/surgeries/copy",
            post(handlers::copy_animal_surgery),
        )
        .route(
            "/surgeries/:id",
            get(handlers::get_animal_surgery)
                .put(handlers::update_animal_surgery)
                .delete(handlers::delete_animal_surgery),
        )
        .route("/surgeries/:id/delete", post(handlers::delete_animal_surgery))
        .route(
            "/surgeries/:id/vet-read",
            post(handlers::mark_surgery_vet_read),
        )
        .route(
            "/surgeries/:id/versions",
            get(handlers::get_surgery_versions),
        )
        // Animal Records - Weights
        .route(
            "/animals/:id/weights",
            get(handlers::list_animal_weights).post(handlers::create_animal_weight),
        )
        .route(
            "/weights/:id",
            put(handlers::update_animal_weight).delete(handlers::delete_animal_weight),
        )
        .route("/weights/:id/delete", post(handlers::delete_animal_weight))
        // Animal Records - Vaccinations
        .route(
            "/animals/:id/vaccinations",
            get(handlers::list_animal_vaccinations).post(handlers::create_animal_vaccination),
        )
        .route(
            "/vaccinations/:id",
            put(handlers::update_animal_vaccination).delete(handlers::delete_animal_vaccination),
        )
        .route("/vaccinations/:id/delete", post(handlers::delete_animal_vaccination))
        // Animal Records - Sacrifice
        .route(
            "/animals/:id/sacrifice",
            get(handlers::get_animal_sacrifice).post(handlers::upsert_animal_sacrifice),
        )
        // Animal Records - Sudden Death (猝死)
        .route(
            "/animals/:id/sudden-death",
            get(handlers::get_animal_sudden_death).post(handlers::create_animal_sudden_death),
        )
        // Animal Transfers (轉讓)
        .route(
            "/animals/:id/data-boundary",
            get(handlers::get_animal_data_boundary),
        )
        .route(
            "/animals/:id/transfers",
            get(handlers::list_animal_transfers).post(handlers::initiate_transfer),
        )
        .route("/transfers/:id", get(handlers::get_transfer))
        .route(
            "/transfers/:id/vet-evaluate",
            post(handlers::vet_evaluate_transfer),
        )
        .route(
            "/transfers/:id/vet-evaluation",
            get(handlers::get_transfer_vet_evaluation),
        )
        .route(
            "/transfers/:id/assign-plan",
            put(handlers::assign_transfer_plan),
        )
        .route("/transfers/:id/approve", post(handlers::approve_transfer))
        .route("/transfers/:id/complete", post(handlers::complete_transfer))
        .route("/transfers/:id/reject", post(handlers::reject_transfer))
        // Animal Records - Pathology
        .route(
            "/animals/:id/pathology",
            get(handlers::get_animal_pathology_report)
                .post(handlers::upsert_animal_pathology_report),
        )
        // Animal Records - Care Records (疼痛評估紀錄)
        .route(
            "/animals/:id/care-records",
            get(handlers::list_care_records).post(handlers::create_care_record),
        )
        .route(
            "/care-records/:id",
            put(handlers::update_care_record).delete(handlers::delete_care_record),
        )
        .route("/care-records/:id/delete", post(handlers::delete_care_record))
        // Animal Records - Blood Tests (血液檢查)
        .route(
            "/animals/:id/blood-tests",
            get(handlers::list_animal_blood_tests).post(handlers::create_animal_blood_test),
        )
        .route(
            "/blood-tests/:id",
            get(handlers::get_animal_blood_test)
                .put(handlers::update_animal_blood_test)
                .delete(handlers::delete_animal_blood_test),
        )
        .route("/blood-tests/:id/delete", post(handlers::delete_animal_blood_test))
        // Blood Test Templates (項目模板管理)
        .route(
            "/blood-test-templates",
            get(handlers::list_blood_test_templates).post(handlers::create_blood_test_template),
        )
        .route(
            "/blood-test-templates/all",
            get(handlers::list_all_blood_test_templates),
        )
        .route(
            "/blood-test-templates/:id",
            put(handlers::update_blood_test_template).delete(handlers::delete_blood_test_template),
        )
        .route("/blood-test-templates/:id/delete", post(handlers::delete_blood_test_template))
        // Blood Test Panels (檢驗組合管理)
        .route(
            "/blood-test-panels",
            get(handlers::list_blood_test_panels).post(handlers::create_blood_test_panel),
        )
        .route(
            "/blood-test-panels/all",
            get(handlers::list_all_blood_test_panels),
        )
        .route(
            "/blood-test-panels/:id",
            put(handlers::update_blood_test_panel).delete(handlers::delete_blood_test_panel),
        )
        .route("/blood-test-panels/:id/delete", post(handlers::delete_blood_test_panel))
        .route(
            "/blood-test-panels/:id/items",
            put(handlers::update_blood_test_panel_items),
        )
        // Vet Recommendations
        .route(
            "/observations/:id/recommendations",
            get(handlers::get_observation_vet_recommendations)
                .post(handlers::add_observation_vet_recommendation),
        )
        .route(
            "/observations/:id/recommendations/with-attachments",
            post(handlers::add_observation_vet_recommendation_with_attachments),
        )
        .route(
            "/surgeries/:id/recommendations",
            get(handlers::get_surgery_vet_recommendations)
                .post(handlers::add_surgery_vet_recommendation),
        )
        .route(
            "/surgeries/:id/recommendations/with-attachments",
            post(handlers::add_surgery_vet_recommendation_with_attachments),
        )
        // Animal Export
        .route(
            "/animals/:id/export",
            post(handlers::export_animal_medical_data),
        )
        .route(
            "/projects/:iacuc_no/export",
            post(handlers::export_project_medical_data),
        )
        // Import Batches
        .route(
            "/animals/import/batches",
            get(handlers::list_import_batches),
        )
        .route(
            "/animals/import/template/basic",
            get(handlers::download_basic_import_template),
        )
        .route(
            "/animals/import/template/weight",
            get(handlers::download_weight_import_template),
        )
        // Notifications（/settings 須在 /:id 之前，避免 "settings" 被當成 UUID 解析）
        .route("/notifications", get(handlers::list_notifications))
        .route(
            "/notifications/unread-count",
            get(handlers::get_unread_count),
        )
        .route("/notifications/read", post(handlers::mark_as_read))
        .route("/notifications/read-all", post(handlers::mark_all_as_read))
        .route(
            "/notifications/settings",
            get(handlers::get_notification_settings).put(handlers::update_notification_settings),
        )
        .route("/notifications/:id", delete(handlers::delete_notification))
        .route("/notifications/:id/delete", post(handlers::delete_notification))
        // Alerts
        .route("/alerts/low-stock", get(handlers::list_low_stock_alerts))
        .route("/alerts/expiry", get(handlers::list_expiry_alerts))
        // Manual Trigger (Admin only)
        .route(
            "/admin/trigger/low-stock-check",
            post(handlers::trigger_low_stock_check),
        )
        .route(
            "/admin/trigger/expiry-check",
            post(handlers::trigger_expiry_check),
        )
        .route(
            "/admin/trigger/notification-cleanup",
            post(handlers::trigger_notification_cleanup),
        )
        // Scheduled Reports
        .route(
            "/scheduled-reports",
            get(handlers::list_scheduled_reports).post(handlers::create_scheduled_report),
        )
        .route(
            "/scheduled-reports/:id",
            get(handlers::get_scheduled_report)
                .put(handlers::update_scheduled_report)
                .delete(handlers::delete_scheduled_report),
        )
        .route("/scheduled-reports/:id/delete", post(handlers::delete_scheduled_report))
        .route("/report-history", get(handlers::list_report_history))
        .route(
            "/report-history/:id/download",
            get(handlers::download_report),
        )
        // Attachments (read/delete — no upload rate limit)
        .route("/attachments", get(handlers::list_attachments))
        .route(
            "/attachments/:id",
            get(handlers::download_attachment).delete(handlers::delete_attachment),
        )
        .route("/attachments/:id/delete", post(handlers::delete_attachment))
        // ============================================
        // Admin System Settings (系統設定管理)
        // ============================================
        .route(
            "/admin/system-settings",
            get(handlers::get_system_settings).put(handlers::update_system_settings),
        )
        // ============================================
        // Admin Config Warnings (啟動配置警告)
        // ============================================
        .route("/admin/config-warnings", get(handlers::get_config_warnings))
        // ============================================
        // Admin Audit Trail (新增)
        // ============================================
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
        .route("/admin/audit/activities", get(handlers::list_activity_logs))
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
        .route("/admin/audit/alerts", get(handlers::list_security_alerts))
        .route(
            "/admin/audit/alerts/:id/resolve",
            post(handlers::resolve_security_alert),
        )
        .route("/admin/audit/dashboard", get(handlers::get_audit_dashboard))
        // QAU Dashboard (GLP 品質保證唯讀檢視)
        .route("/qau/dashboard", get(handlers::get_qau_dashboard))
        // SSE 安全警報即時推送
        .route(
            "/admin/audit/alerts/sse",
            get(handlers::sse::sse_security_alerts),
        )
        // ============================================
        // Admin Notification Routing (通知路由管理)
        // ============================================
        .route(
            "/admin/notification-routing",
            get(handlers::list_notification_routing).post(handlers::create_notification_routing),
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
        .route("/admin/notification-routing/:id/delete", post(handlers::delete_notification_routing))
        // ============================================
        // Treatment Drug Options (藥物選單管理)
        // ============================================
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
        .route("/admin/treatment-drugs/:id/delete", post(handlers::treatment_drug::delete_treatment_drug))
        .route(
            "/admin/treatment-drugs/import-erp",
            post(handlers::treatment_drug::import_treatment_drugs_from_erp),
        )
        // ============================================
        // HR Attendance (新增)
        // ============================================
        .route("/hr/attendance", get(handlers::list_attendance))
        .route("/hr/attendance/clock-in", post(handlers::clock_in))
        .route("/hr/attendance/clock-out", post(handlers::clock_out))
        .route("/hr/attendance/stats", get(handlers::get_attendance_stats))
        .route("/hr/attendance/:id", put(handlers::correct_attendance))
        // ============================================
        // HR Overtime (新增)
        // ============================================
        .route(
            "/hr/overtime",
            get(handlers::list_overtime).post(handlers::create_overtime),
        )
        .route(
            "/hr/overtime/:id",
            get(handlers::get_overtime)
                .put(handlers::update_overtime)
                .delete(handlers::delete_overtime),
        )
        .route("/hr/overtime/:id/delete", post(handlers::delete_overtime))
        .route("/hr/overtime/:id/submit", post(handlers::submit_overtime))
        .route("/hr/overtime/:id/approve", post(handlers::approve_overtime))
        .route("/hr/overtime/:id/reject", post(handlers::reject_overtime))
        // ============================================
        // HR Leave (新增)
        // ============================================
        .route(
            "/hr/leaves",
            get(handlers::list_leaves).post(handlers::create_leave),
        )
        .route(
            "/hr/leaves/:id",
            get(handlers::get_leave)
                .put(handlers::update_leave)
                .delete(handlers::delete_leave),
        )
        .route("/hr/leaves/:id/delete", post(handlers::delete_leave))
        .route("/hr/leaves/:id/submit", post(handlers::submit_leave))
        .route("/hr/leaves/:id/approve", post(handlers::approve_leave))
        .route("/hr/leaves/:id/reject", post(handlers::reject_leave))
        .route("/hr/leaves/:id/cancel", post(handlers::cancel_leave))
        // ============================================
        // HR Balances (新增)
        // ============================================
        .route(
            "/hr/balances/annual",
            get(handlers::get_annual_leave_balances),
        )
        .route(
            "/hr/balances/comp-time",
            get(handlers::get_comp_time_balances),
        )
        .route("/hr/balances/summary", get(handlers::get_balance_summary))
        .route(
            "/hr/balances/annual-entitlements",
            post(handlers::create_annual_leave_entitlement),
        )
        .route("/hr/balances/:id/adjust", post(handlers::adjust_balance))
        .route(
            "/hr/balances/expired-compensation",
            get(handlers::get_expired_leave_compensation),
        )
        // ============================================
        // HR Dashboard (儀表板)
        // ============================================
        .route(
            "/hr/dashboard/calendar",
            get(handlers::get_dashboard_calendar),
        )
        // ============================================
        // HR Staff List (供下拉選單用)
        // ============================================
        .route("/hr/staff", get(handlers::list_staff_for_proxy))
        .route(
            "/hr/internal-users",
            get(handlers::list_internal_users_for_balance),
        )
        // ============================================
        // Calendar Sync (新增)
        // ============================================
        .route("/hr/calendar/status", get(handlers::get_calendar_status))
        .route(
            "/hr/calendar/config",
            get(handlers::get_calendar_config).put(handlers::update_calendar_config),
        )
        .route("/hr/calendar/connect", post(handlers::connect_calendar))
        .route(
            "/hr/calendar/disconnect",
            post(handlers::disconnect_calendar),
        )
        .route("/hr/calendar/sync", post(handlers::trigger_sync))
        .route("/hr/calendar/history", get(handlers::list_sync_history))
        .route("/hr/calendar/pending", get(handlers::list_pending_syncs))
        .route("/hr/calendar/conflicts", get(handlers::list_conflicts))
        .route("/hr/calendar/conflicts/:id", get(handlers::get_conflict))
        // ============================================
        // Training Records (人員訓練紀錄, GLP 合規)
        // ============================================
        .route(
            "/training-records",
            get(handlers::list_training_records).post(handlers::create_training_record),
        )
        .route(
            "/training-records/:id",
            get(handlers::get_training_record)
                .put(handlers::update_training_record)
                .delete(handlers::delete_training_record),
        )
        .route("/training-records/:id/delete", post(handlers::delete_training_record))
        .route("/hr/calendar/conflicts/:id/resolve", post(handlers::resolve_conflict))
        // ============================================
        // Equipment & Calibrations (設備校準, GLP 合規)
        // ============================================
        .route(
            "/equipment",
            get(handlers::list_equipment).post(handlers::create_equipment),
        )
        .route(
            "/equipment/:id",
            get(handlers::get_equipment)
                .put(handlers::update_equipment)
                .delete(handlers::delete_equipment),
        )
        .route("/equipment/:id/delete", post(handlers::delete_equipment))
        .route(
            "/equipment-calibrations",
            get(handlers::list_calibrations).post(handlers::create_calibration),
        )
        .route(
            "/equipment-calibrations/:id",
            get(handlers::get_calibration)
                .put(handlers::update_calibration)
                .delete(handlers::delete_calibration),
        )
        .route("/equipment-calibrations/:id/delete", post(handlers::delete_calibration))
        .route("/hr/calendar/events", get(handlers::list_calendar_events))
        // ============================================
        // Facility Management (新增)
        // ============================================
        .route(
            "/facilities/species",
            get(handlers::list_species).post(handlers::create_species),
        )
        .route(
            "/facilities/species/:id",
            get(handlers::get_species)
                .put(handlers::update_species)
                .delete(handlers::delete_species),
        )
        .route("/facilities/species/:id/delete", post(handlers::delete_species))
        .route(
            "/facilities",
            get(handlers::list_facilities).post(handlers::create_facility),
        )
        .route(
            "/facilities/:id",
            get(handlers::get_facility)
                .put(handlers::update_facility)
                .delete(handlers::delete_facility),
        )
        .route("/facilities/:id/delete", post(handlers::delete_facility))
        .route(
            "/facilities/buildings",
            get(handlers::list_buildings).post(handlers::create_building),
        )
        .route(
            "/facilities/buildings/:id",
            get(handlers::get_building)
                .put(handlers::update_building)
                .delete(handlers::delete_building),
        )
        .route("/facilities/buildings/:id/delete", post(handlers::delete_building))
        .route(
            "/facilities/zones",
            get(handlers::list_zones).post(handlers::create_zone),
        )
        .route(
            "/facilities/zones/:id",
            get(handlers::get_zone)
                .put(handlers::update_zone)
                .delete(handlers::delete_zone),
        )
        .route("/facilities/zones/:id/delete", post(handlers::delete_zone))
        .route(
            "/facilities/pens",
            get(handlers::list_pens).post(handlers::create_pen),
        )
        .route(
            "/facilities/pens/:id",
            get(handlers::get_pen)
                .put(handlers::update_pen)
                .delete(handlers::delete_pen),
        )
        .route("/facilities/pens/:id/delete", post(handlers::delete_pen))
        .route(
            "/facilities/departments",
            get(handlers::list_departments).post(handlers::create_department),
        )
        .route(
            "/facilities/departments/:id",
            get(handlers::get_department)
                .put(handlers::update_department)
                .delete(handlers::delete_department),
        )
        .route("/facilities/departments/:id/delete", post(handlers::delete_department))
        // ============================================
        // Electronic Signatures & Annotations (GLP Compliance)
        // ============================================
        .route(
            "/signatures/sacrifice/:id",
            post(handlers::sign_sacrifice_record).get(handlers::get_sacrifice_signature_status),
        )
        .route(
            "/signatures/observation/:id",
            post(handlers::sign_observation_record),
        )
        .route(
            "/signatures/euthanasia/:id",
            post(handlers::sign_euthanasia_order).get(handlers::get_euthanasia_signature_status),
        )
        .route(
            "/signatures/transfer/:id",
            post(handlers::sign_transfer_record).get(handlers::get_transfer_signature_status),
        )
        .route(
            "/signatures/protocol/:id",
            post(handlers::sign_protocol_review).get(handlers::get_protocol_signature_status),
        )
        .route(
            "/annotations/:record_type/:record_id",
            get(handlers::get_record_annotations).post(handlers::add_record_annotation),
        )
        // ============================================
        // Euthanasia Orders (安樂死管理)
        // ============================================
        .route(
            "/euthanasia/orders",
            post(handlers::euthanasia::create_order),
        )
        .route(
            "/euthanasia/orders/pending",
            get(handlers::euthanasia::get_pending_orders),
        )
        .route(
            "/euthanasia/orders/:id",
            get(handlers::euthanasia::get_order),
        )
        .route(
            "/euthanasia/orders/:id/approve",
            post(handlers::euthanasia::approve_order),
        )
        .route(
            "/euthanasia/orders/:id/appeal",
            post(handlers::euthanasia::appeal_order),
        )
        .route(
            "/euthanasia/orders/:id/execute",
            post(handlers::euthanasia::execute_order),
        )
        .route(
            "/euthanasia/appeals/:id/decide",
            post(handlers::euthanasia::decide_appeal),
        )
        // ============================================
        // Amendments (變更申請系統)
        // ============================================
        .route(
            "/amendments",
            get(handlers::amendment::list_amendments).post(handlers::amendment::create_amendment),
        )
        .route(
            "/amendments/pending-count",
            get(handlers::amendment::get_pending_count),
        )
        .route(
            "/amendments/:id",
            get(handlers::amendment::get_amendment).patch(handlers::amendment::update_amendment),
        )
        .route(
            "/amendments/:id/submit",
            post(handlers::amendment::submit_amendment),
        )
        .route(
            "/amendments/:id/classify",
            post(handlers::amendment::classify_amendment),
        )
        .route(
            "/amendments/:id/start-review",
            post(handlers::amendment::start_amendment_review),
        )
        .route(
            "/amendments/:id/decision",
            post(handlers::amendment::record_amendment_decision),
        )
        .route(
            "/amendments/:id/status",
            post(handlers::amendment::change_amendment_status),
        )
        .route(
            "/amendments/:id/versions",
            get(handlers::amendment::get_amendment_versions),
        )
        .route(
            "/amendments/:id/history",
            get(handlers::amendment::get_amendment_history),
        )
        .route(
            "/amendments/:id/assignments",
            get(handlers::amendment::get_amendment_assignments),
        )
        .route(
            "/protocols/:id/amendments",
            get(handlers::amendment::list_protocol_amendments),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            write_rate_limit_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    // File upload routes — stricter 30/min rate limit
    let upload_routes = Router::new()
        .route(
            "/protocols/:id/attachments",
            post(handlers::upload_protocol_attachment),
        )
        .route("/animals/:id/photos", post(handlers::upload_animal_photo))
        .route(
            "/animals/:id/pathology/attachments",
            post(handlers::upload_pathology_report),
        )
        .route(
            "/animals/:id/sacrifice/photos",
            post(handlers::upload_sacrifice_photo),
        )
        .route(
            "/vet-recommendations/:record_type/:record_id/attachments",
            post(handlers::upload_vet_recommendation_attachment),
        )
        .route(
            "/observations/:id/attachments",
            post(handlers::upload_observation_attachment),
        )
        .route(
            "/hr/leaves/attachments",
            post(handlers::upload_leave_attachment),
        )
        .route(
            "/animals/import/basic",
            post(handlers::import_basic_data),
        )
        .route(
            "/animals/import/weights",
            post(handlers::import_weight_data),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            upload_rate_limit_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    // 健康檢查 + 指標路由（不受 Rate Limiter 影響，確保監控系統可探測）
    let health_route = Router::new()
        .route("/api/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::metrics_handler))
        .route("/api/metrics/vitals", post(handlers::metrics::vitals_handler))
        .with_state(state.clone());

    // P1-M1: API 版本路徑 — 引入 /api/v1/ 前綴；/api 保留為 deprecated 向後相容
    let api_v1 = public_routes.merge(protected_routes).merge(upload_routes);
    health_route
        .merge(
            Router::new()
                .nest("/api/v1", api_v1.clone())
                .nest("/api", api_v1)
                .route_layer(middleware::from_fn(etag_middleware))
                .route_layer(middleware::from_fn_with_state(
                    state,
                    api_rate_limit_middleware,
                )),
        )
}
