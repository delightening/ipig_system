use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::{handlers, AppState};

/// 通知、警示、排程報表、附件路由
pub fn routes() -> Router<AppState> {
    Router::new()
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
            get(handlers::get_notification_settings)
                .put(handlers::update_notification_settings),
        )
        .route(
            "/notifications/:id",
            delete(handlers::delete_notification),
        )
        .route(
            "/notifications/:id/delete",
            post(handlers::delete_notification),
        )
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
        .route(
            "/admin/trigger/po-pending-receipt-check",
            post(handlers::trigger_po_pending_receipt_check),
        )
        // Scheduled Reports
        .route(
            "/scheduled-reports",
            get(handlers::list_scheduled_reports)
                .post(handlers::create_scheduled_report),
        )
        .route(
            "/scheduled-reports/:id",
            get(handlers::get_scheduled_report)
                .put(handlers::update_scheduled_report)
                .delete(handlers::delete_scheduled_report),
        )
        .route(
            "/scheduled-reports/:id/delete",
            post(handlers::delete_scheduled_report),
        )
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
        .route(
            "/attachments/:id/delete",
            post(handlers::delete_attachment),
        )
}
