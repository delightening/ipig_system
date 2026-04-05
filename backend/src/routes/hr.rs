use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// HR、設備校準、訓練紀錄、設施管理、電子簽章、安樂死路由
pub fn routes() -> Router<AppState> {
    Router::new()
        // HR Attendance
        .route("/hr/attendance", get(handlers::list_attendance))
        .route("/hr/attendance/export", get(handlers::export_attendance))
        .route("/hr/attendance/clock-in", post(handlers::clock_in))
        .route("/hr/attendance/clock-out", post(handlers::clock_out))
        .route("/hr/attendance/stats", get(handlers::get_attendance_stats))
        .route(
            "/hr/attendance/:id",
            put(handlers::correct_attendance),
        )
        // HR Overtime
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
        // 勞基法合規 API
        .route("/hr/overtime/limit-check", get(handlers::check_overtime_limit))
        .route("/hr/overtime/weekday-tiers", get(handlers::calculate_weekday_tiers))
        .route("/hr/work-hours/validate", get(handlers::validate_work_hours))
        // HR Leave
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
        // HR Balances
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
            "/hr/balances/annual-auto-calc",
            post(handlers::auto_calculate_annual_leave),
        )
        .route(
            "/hr/balances/annual-batch-calc",
            post(handlers::batch_auto_calculate_annual_leave),
        )
        .route(
            "/hr/balances/expired-compensation",
            get(handlers::get_expired_leave_compensation),
        )
        // HR Dashboard
        .route(
            "/hr/dashboard/calendar",
            get(handlers::get_dashboard_calendar),
        )
        // HR Staff List
        .route("/hr/staff", get(handlers::list_staff_for_proxy))
        .route(
            "/hr/internal-users",
            get(handlers::list_internal_users_for_balance),
        )
        // Calendar Sync
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
        .route(
            "/hr/calendar/conflicts/:id/resolve",
            post(handlers::resolve_conflict),
        )
        .route("/hr/calendar/events", get(handlers::list_calendar_events))
        // Training Records (GLP 合規)
        .route(
            "/training-records",
            get(handlers::list_training_records)
                .post(handlers::create_training_record),
        )
        .route(
            "/training-records/:id",
            get(handlers::get_training_record)
                .put(handlers::update_training_record)
                .delete(handlers::delete_training_record),
        )
        .route(
            "/training-records/:id/delete",
            post(handlers::delete_training_record),
        )
        // Equipment & Calibrations (GLP 合規)
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
        // Equipment Suppliers
        .route(
            "/equipment-suppliers/summary",
            get(handlers::list_equipment_suppliers_summary),
        )
        .route(
            "/equipment/:id/suppliers",
            get(handlers::list_equipment_suppliers)
                .post(handlers::add_equipment_supplier),
        )
        .route(
            "/equipment-suppliers/:id",
            delete(handlers::remove_equipment_supplier),
        )
        .route(
            "/equipment-suppliers/:id/delete",
            post(handlers::remove_equipment_supplier),
        )
        // Equipment Status Logs
        .route(
            "/equipment/:id/status-logs",
            get(handlers::list_status_logs),
        )
        // Equipment Timeline (設備履歷)
        .route(
            "/equipment/:id/timeline",
            get(handlers::get_equipment_timeline),
        )
        // Calibrations (校正/確效/查核)
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
        .route(
            "/equipment-calibrations/:id/delete",
            post(handlers::delete_calibration),
        )
        // Maintenance Records (維修/保養)
        .route(
            "/equipment-maintenance",
            get(handlers::list_maintenance_records)
                .post(handlers::create_maintenance_record),
        )
        .route(
            "/equipment-maintenance/:id",
            put(handlers::update_maintenance_record)
                .delete(handlers::delete_maintenance_record),
        )
        .route(
            "/equipment-maintenance/:id/delete",
            post(handlers::delete_maintenance_record),
        )
        .route(
            "/equipment-maintenance/:id/review",
            post(handlers::review_maintenance_record),
        )
        .route(
            "/equipment-maintenance/:id/history",
            get(handlers::get_maintenance_history),
        )
        // Disposal Records (報廢)
        .route(
            "/equipment-disposals",
            get(handlers::list_disposals).post(handlers::create_disposal),
        )
        .route(
            "/equipment-disposals/:id/approve",
            post(handlers::approve_disposal),
        )
        .route(
            "/equipment-disposals/:id/restore",
            post(handlers::restore_equipment),
        )
        // Idle Requests (閒置審批)
        .route(
            "/equipment-idle-requests",
            get(handlers::list_idle_requests).post(handlers::create_idle_request),
        )
        .route(
            "/equipment-idle-requests/:id/approve",
            post(handlers::approve_idle_request),
        )
        // Annual Plan (年度計畫)
        .route(
            "/equipment-annual-plans",
            get(handlers::list_annual_plans)
                .post(handlers::create_annual_plan),
        )
        .route(
            "/equipment-annual-plans/generate",
            post(handlers::generate_annual_plan),
        )
        .route(
            "/equipment-annual-plans/execution-summary",
            get(handlers::get_annual_plan_execution_summary),
        )
        .route(
            "/equipment-annual-plans/:id",
            put(handlers::update_annual_plan)
                .delete(handlers::delete_annual_plan),
        )
        .route(
            "/equipment-annual-plans/:id/delete",
            post(handlers::delete_annual_plan),
        )
        // Facility Management
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
        .route(
            "/facilities/species/:id/delete",
            post(handlers::delete_species),
        )
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
        .route(
            "/facilities/buildings/:id/delete",
            post(handlers::delete_building),
        )
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
        .route(
            "/facilities/zones/:id/delete",
            post(handlers::delete_zone),
        )
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
        .route("/facilities/pens/batch", post(handlers::batch_create_pens))
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
        .route(
            "/facilities/departments/:id/delete",
            post(handlers::delete_department),
        )
        // Disposal Signatures
        .route(
            "/signatures/disposal/:id/applicant",
            post(handlers::sign_disposal_applicant),
        )
        .route(
            "/signatures/disposal/:id/approver",
            post(handlers::sign_disposal_approver),
        )
        .route(
            "/signatures/disposal/:id",
            get(handlers::get_disposal_signature_status),
        )
        // Maintenance Review Signatures
        .route(
            "/signatures/maintenance/:id/reviewer",
            post(handlers::sign_maintenance_reviewer),
        )
        .route(
            "/signatures/maintenance/:id",
            get(handlers::get_maintenance_signature_status),
        )
        // Electronic Signatures & Annotations (GLP Compliance)
        .route(
            "/signatures/sacrifice/:id",
            post(handlers::sign_sacrifice_record)
                .get(handlers::get_sacrifice_signature_status),
        )
        .route(
            "/signatures/observation/:id",
            post(handlers::sign_observation_record),
        )
        .route(
            "/signatures/euthanasia/:id",
            post(handlers::sign_euthanasia_order)
                .get(handlers::get_euthanasia_signature_status),
        )
        .route(
            "/signatures/transfer/:id",
            post(handlers::sign_transfer_record)
                .get(handlers::get_transfer_signature_status),
        )
        .route(
            "/signatures/protocol/:id",
            post(handlers::sign_protocol_review)
                .get(handlers::get_protocol_signature_status),
        )
        .route(
            "/annotations/:record_type/:record_id",
            get(handlers::get_record_annotations)
                .post(handlers::add_record_annotation),
        )
        // Euthanasia Orders
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
}
