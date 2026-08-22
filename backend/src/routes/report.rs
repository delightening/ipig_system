use axum::{
    routing::{get, post},
    Router,
};

use crate::{handlers, AppState};

/// 稽核日誌、報表、會計路由
pub fn routes() -> Router<AppState> {
    Router::new()
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
        .route(
            "/reports/purchase-sales-monthly",
            get(handlers::get_purchase_sales_monthly),
        )
        .route(
            "/reports/purchase-sales-by-partner",
            get(handlers::get_purchase_sales_by_partner),
        )
        .route(
            "/reports/purchase-sales-by-category",
            get(handlers::get_purchase_sales_by_category),
        )
        // R53-9 豬隻病歷週報（多表 timeline UNION）
        .route(
            "/reports/animal-medical/weekly",
            post(handlers::weekly_medical_report),
        )
        .route(
            "/reports/animal-medical/weekly/xlsx",
            post(handlers::export_weekly_medical_report_xlsx),
        )
        .route(
            "/reports/animal-medical/weekly/pdf",
            post(handlers::export_weekly_medical_report_pdf),
        )
        // R53-15 byproduct 月結報表
        .route(
            "/reports/byproduct-monthly",
            post(handlers::byproduct_monthly_report),
        )
        .route(
            "/reports/byproduct-monthly/xlsx",
            post(handlers::export_byproduct_monthly_xlsx),
        )
        // Accounting
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
        .route("/accounting/ap-aging", get(handlers::get_ap_aging))
        .route("/accounting/ar-aging", get(handlers::get_ar_aging))
        .route("/accounting/ap-payments", post(handlers::create_ap_payment))
        .route("/accounting/ar-receipts", post(handlers::create_ar_receipt))
        .route("/accounting/profit-loss", get(handlers::get_profit_loss))
}
