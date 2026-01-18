use axum::{
    middleware,
    routing::{get, post, put, delete},
    Router,
};

use crate::{handlers, middleware::auth_middleware, AppState};

pub fn api_routes(state: AppState) -> Router {
    // Public routes (no auth required) - 移除公開註冊，改為私域註冊
    let public_routes = Router::new()
        .route("/auth/login", post(handlers::login))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/forgot-password", post(handlers::forgot_password))
        .route("/auth/reset-password", post(handlers::reset_password_with_token))
        .with_state(state.clone());

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Auth
        .route("/auth/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
        .route("/me/password", put(handlers::change_own_password))
        // User Preferences
        .route("/me/preferences", get(handlers::get_all_preferences))
        .route("/me/preferences/:key", get(handlers::get_preference).put(handlers::upsert_preference).delete(handlers::delete_preference))
        // Users
        .route("/users", get(handlers::list_users).post(handlers::create_user))
        .route("/users/:id", get(handlers::get_user).put(handlers::update_user).delete(handlers::delete_user))
        .route("/users/:id/password", put(handlers::reset_user_password))
        // Roles
        .route("/roles", get(handlers::list_roles).post(handlers::create_role))
        .route("/roles/:id", get(handlers::get_role).put(handlers::update_role).delete(handlers::delete_role))
        .route("/permissions", get(handlers::list_permissions))
        // Warehouses
        .route("/warehouses", get(handlers::list_warehouses).post(handlers::create_warehouse))
        .route("/warehouses/:id", get(handlers::get_warehouse).put(handlers::update_warehouse).delete(handlers::delete_warehouse))
        // Products
        .route("/products", get(handlers::list_products).post(handlers::create_product))
        .route("/products/:id", get(handlers::get_product).put(handlers::update_product).delete(handlers::delete_product))
        .route("/categories", get(handlers::list_categories).post(handlers::create_category))
        // SKU (完整 API)
        .route("/sku/categories", get(handlers::get_sku_categories))
        .route("/sku/categories/:code/subcategories", get(handlers::get_sku_subcategories))
        .route("/sku/generate", post(handlers::generate_sku))
        .route("/sku/validate", post(handlers::validate_sku))
        .route("/skus/preview", post(handlers::preview_sku))
        .route("/products/with-sku", post(handlers::create_product_with_sku))
        // Partners
        .route("/partners", get(handlers::list_partners).post(handlers::create_partner))
        .route("/partners/generate-code", get(handlers::generate_partner_code))
        .route("/partners/:id", get(handlers::get_partner).put(handlers::update_partner).delete(handlers::delete_partner))
        // Documents
        .route("/documents", get(handlers::list_documents).post(handlers::create_document))
        .route("/documents/:id", get(handlers::get_document).put(handlers::update_document).delete(handlers::delete_document))
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
        .route("/reports/stock-on-hand", get(handlers::get_stock_on_hand_report))
        .route("/reports/stock-ledger", get(handlers::get_stock_ledger_report))
        .route("/reports/purchase-lines", get(handlers::get_purchase_lines_report))
        .route("/reports/sales-lines", get(handlers::get_sales_lines_report))
        .route("/reports/cost-summary", get(handlers::get_cost_summary_report))
        // Protocols (AUP 審查系統)
        .route("/protocols", get(handlers::list_protocols).post(handlers::create_protocol))
        .route("/protocols/:id", get(handlers::get_protocol).put(handlers::update_protocol))
        .route("/protocols/:id/submit", post(handlers::submit_protocol))
        .route("/protocols/:id/status", post(handlers::change_protocol_status))
        .route("/protocols/:id/versions", get(handlers::get_protocol_versions))
        .route("/protocols/:id/status-history", get(handlers::get_protocol_status_history))
        .route("/protocols/:id/animal-stats", get(handlers::get_protocol_animal_stats))
        // Review
        .route("/reviews/assignments", get(handlers::list_review_assignments).post(handlers::assign_reviewer))
        .route("/reviews/comments", get(handlers::list_review_comments).post(handlers::create_review_comment))
        .route("/reviews/comments/:id/resolve", post(handlers::resolve_review_comment))
        .route("/reviews/comments/reply", post(handlers::reply_review_comment))
        // Co-Editor Assignment
        .route("/protocols/:id/co-editors", get(handlers::list_co_editors).post(handlers::assign_co_editor))
        .route("/protocols/:id/co-editors/:user_id", delete(handlers::remove_co_editor))
        // My Projects
        .route("/my-projects", get(handlers::get_my_protocols))
        // Pig Sources
        .route("/pig-sources", get(handlers::list_pig_sources).post(handlers::create_pig_source))
        .route("/pig-sources/:id", put(handlers::update_pig_source).delete(handlers::delete_pig_source))
        // Pigs (實驗動物管理系統)
        .route("/pigs", get(handlers::list_pigs).post(handlers::create_pig))
        .route("/pigs/by-pen", get(handlers::list_pigs_by_pen))
        .route("/pigs/batch/assign", post(handlers::batch_assign_pigs))
        .route("/pigs/batch/start-experiment", post(handlers::batch_start_experiment))
        .route("/pigs/vet-comments", get(handlers::get_vet_comments))
        .route("/pigs/:id", get(handlers::get_pig).put(handlers::update_pig).delete(handlers::delete_pig))
        .route("/pigs/:id/vet-read", post(handlers::mark_pig_vet_read))
        // Pig Records - Observations
        .route("/pigs/:id/observations", get(handlers::list_pig_observations).post(handlers::create_pig_observation))
        .route("/pigs/:id/observations/with-recommendations", get(handlers::list_pig_observations_with_recommendations))
        .route("/pigs/:id/observations/copy", post(handlers::copy_pig_observation))
        .route("/observations/:id", get(handlers::get_pig_observation).put(handlers::update_pig_observation).delete(handlers::delete_pig_observation))
        .route("/observations/:id/vet-read", post(handlers::mark_observation_vet_read))
        .route("/observations/:id/versions", get(handlers::get_observation_versions))
        // Pig Records - Surgeries
        .route("/pigs/:id/surgeries", get(handlers::list_pig_surgeries).post(handlers::create_pig_surgery))
        .route("/pigs/:id/surgeries/with-recommendations", get(handlers::list_pig_surgeries_with_recommendations))
        .route("/pigs/:id/surgeries/copy", post(handlers::copy_pig_surgery))
        .route("/surgeries/:id", get(handlers::get_pig_surgery).put(handlers::update_pig_surgery).delete(handlers::delete_pig_surgery))
        .route("/surgeries/:id/vet-read", post(handlers::mark_surgery_vet_read))
        .route("/surgeries/:id/versions", get(handlers::get_surgery_versions))
        // Pig Records - Weights
        .route("/pigs/:id/weights", get(handlers::list_pig_weights).post(handlers::create_pig_weight))
        .route("/weights/:id", put(handlers::update_pig_weight).delete(handlers::delete_pig_weight))
        // Pig Records - Vaccinations
        .route("/pigs/:id/vaccinations", get(handlers::list_pig_vaccinations).post(handlers::create_pig_vaccination))
        .route("/vaccinations/:id", put(handlers::update_pig_vaccination).delete(handlers::delete_pig_vaccination))
        // Pig Records - Sacrifice
        .route("/pigs/:id/sacrifice", get(handlers::get_pig_sacrifice).post(handlers::upsert_pig_sacrifice))
        // Pig Records - Pathology
        .route("/pigs/:id/pathology", get(handlers::get_pig_pathology_report).post(handlers::upsert_pig_pathology_report))
        // Vet Recommendations
        .route("/observations/:id/recommendations", get(handlers::get_observation_vet_recommendations).post(handlers::add_observation_vet_recommendation))
        .route("/observations/:id/recommendations/with-attachments", post(handlers::add_observation_vet_recommendation_with_attachments))
        .route("/surgeries/:id/recommendations", get(handlers::get_surgery_vet_recommendations).post(handlers::add_surgery_vet_recommendation))
        .route("/surgeries/:id/recommendations/with-attachments", post(handlers::add_surgery_vet_recommendation_with_attachments))
        // Pig Export
        .route("/pigs/:id/export", post(handlers::export_pig_medical_data))
        .route("/projects/:iacuc_no/export", post(handlers::export_project_medical_data))
        // Import Batches
        .route("/pigs/import/batches", get(handlers::list_import_batches))
        .route("/pigs/import/template/basic", get(handlers::download_basic_import_template))
        .route("/pigs/import/template/weight", get(handlers::download_weight_import_template))
        .route("/pigs/import/basic", post(handlers::import_basic_data))
        .route("/pigs/import/weights", post(handlers::import_weight_data))
        // Notifications
        .route("/notifications", get(handlers::list_notifications))
        .route("/notifications/unread-count", get(handlers::get_unread_count))
        .route("/notifications/read", post(handlers::mark_as_read))
        .route("/notifications/read-all", post(handlers::mark_all_as_read))
        .route("/notifications/:id", delete(handlers::delete_notification))
        .route("/notifications/settings", get(handlers::get_notification_settings).put(handlers::update_notification_settings))
        // Alerts
        .route("/alerts/low-stock", get(handlers::list_low_stock_alerts))
        .route("/alerts/expiry", get(handlers::list_expiry_alerts))
        // Manual Trigger (Admin only)
        .route("/admin/trigger/low-stock-check", post(handlers::trigger_low_stock_check))
        .route("/admin/trigger/expiry-check", post(handlers::trigger_expiry_check))
        .route("/admin/trigger/notification-cleanup", post(handlers::trigger_notification_cleanup))
        // Scheduled Reports
        .route("/scheduled-reports", get(handlers::list_scheduled_reports).post(handlers::create_scheduled_report))
        .route("/scheduled-reports/:id", get(handlers::get_scheduled_report).put(handlers::update_scheduled_report).delete(handlers::delete_scheduled_report))
        .route("/report-history", get(handlers::list_report_history))
        .route("/report-history/:id/download", get(handlers::download_report))
        // File Upload
        .route("/protocols/:id/attachments", post(handlers::upload_protocol_attachment))
        .route("/pigs/:id/photos", post(handlers::upload_pig_photo))
        .route("/pigs/:id/pathology/attachments", post(handlers::upload_pathology_report))
        .route("/pigs/:id/sacrifice/photos", post(handlers::upload_sacrifice_photo))
        .route("/vet-recommendations/:record_type/:record_id/attachments", post(handlers::upload_vet_recommendation_attachment))
        .route("/attachments", get(handlers::list_attachments))
        .route("/attachments/:id", get(handlers::download_attachment).delete(handlers::delete_attachment))
        // ============================================
        // Admin Audit Trail (新增)
        // ============================================
        .route("/admin/audit/activities", get(handlers::list_activity_logs))
        .route("/admin/audit/activities/user/:user_id", get(handlers::get_user_activity_timeline))
        .route("/admin/audit/activities/entity/:entity_type/:entity_id", get(handlers::get_entity_history))
        .route("/admin/audit/logins", get(handlers::list_login_events))
        .route("/admin/audit/sessions", get(handlers::list_sessions))
        .route("/admin/audit/sessions/:id/logout", post(handlers::force_logout_session))
        .route("/admin/audit/alerts", get(handlers::list_security_alerts))
        .route("/admin/audit/alerts/:id/resolve", post(handlers::resolve_security_alert))
        .route("/admin/audit/dashboard", get(handlers::get_audit_dashboard))
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
        .route("/hr/overtime", get(handlers::list_overtime).post(handlers::create_overtime))
        .route("/hr/overtime/:id", get(handlers::get_overtime).put(handlers::update_overtime).delete(handlers::delete_overtime))
        .route("/hr/overtime/:id/submit", post(handlers::submit_overtime))
        .route("/hr/overtime/:id/approve", post(handlers::approve_overtime))
        .route("/hr/overtime/:id/reject", post(handlers::reject_overtime))
        // ============================================
        // HR Leave (新增)
        // ============================================
        .route("/hr/leaves", get(handlers::list_leaves).post(handlers::create_leave))
        .route("/hr/leaves/:id", get(handlers::get_leave).put(handlers::update_leave).delete(handlers::delete_leave))
        .route("/hr/leaves/:id/submit", post(handlers::submit_leave))
        .route("/hr/leaves/:id/approve", post(handlers::approve_leave))
        .route("/hr/leaves/:id/reject", post(handlers::reject_leave))
        .route("/hr/leaves/:id/cancel", post(handlers::cancel_leave))
        .route("/hr/leaves/attachments", post(handlers::upload_leave_attachment))
        // ============================================
        // HR Balances (新增)
        // ============================================
        .route("/hr/balances/annual", get(handlers::get_annual_leave_balances))
        .route("/hr/balances/comp-time", get(handlers::get_comp_time_balances))
        .route("/hr/balances/summary", get(handlers::get_balance_summary))
        .route("/hr/balances/annual-entitlements", post(handlers::create_annual_leave_entitlement))
        .route("/hr/balances/:id/adjust", post(handlers::adjust_balance))
        // ============================================
        // HR Dashboard (儀表板)
        // ============================================
        .route("/hr/dashboard/calendar", get(handlers::get_dashboard_calendar))
        // ============================================
        // Calendar Sync (新增)
        // ============================================
        .route("/hr/calendar/status", get(handlers::get_calendar_status))
        .route("/hr/calendar/config", get(handlers::get_calendar_config).put(handlers::update_calendar_config))
        .route("/hr/calendar/connect", post(handlers::connect_calendar))
        .route("/hr/calendar/disconnect", post(handlers::disconnect_calendar))
        .route("/hr/calendar/sync", post(handlers::trigger_sync))
        .route("/hr/calendar/history", get(handlers::list_sync_history))
        .route("/hr/calendar/pending", get(handlers::list_pending_syncs))
        .route("/hr/calendar/conflicts", get(handlers::list_conflicts))
        .route("/hr/calendar/conflicts/:id", get(handlers::get_conflict))
        .route("/hr/calendar/conflicts/:id/resolve", post(handlers::resolve_conflict))
        .route("/hr/calendar/events", get(handlers::list_calendar_events))
        // ============================================
        // Facility Management (新增)
        // ============================================
        .route("/facilities/species", get(handlers::list_species).post(handlers::create_species))
        .route("/facilities/species/:id", get(handlers::get_species).put(handlers::update_species).delete(handlers::delete_species))
        .route("/facilities", get(handlers::list_facilities).post(handlers::create_facility))
        .route("/facilities/:id", get(handlers::get_facility).put(handlers::update_facility).delete(handlers::delete_facility))
        .route("/facilities/buildings", get(handlers::list_buildings).post(handlers::create_building))
        .route("/facilities/buildings/:id", get(handlers::get_building).put(handlers::update_building).delete(handlers::delete_building))
        .route("/facilities/zones", get(handlers::list_zones).post(handlers::create_zone))
        .route("/facilities/zones/:id", get(handlers::get_zone).put(handlers::update_zone).delete(handlers::delete_zone))
        .route("/facilities/pens", get(handlers::list_pens).post(handlers::create_pen))
        .route("/facilities/pens/:id", get(handlers::get_pen).put(handlers::update_pen).delete(handlers::delete_pen))
        .route("/facilities/departments", get(handlers::list_departments).post(handlers::create_department))
        .route("/facilities/departments/:id", get(handlers::get_department).put(handlers::update_department).delete(handlers::delete_department))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state);

    Router::new()
        .nest("/api", public_routes.merge(protected_routes))
}
