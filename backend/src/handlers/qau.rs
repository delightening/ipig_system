//! QAU (Quality Assurance Unit) Handlers
//!
//! GLP 品質保證：唯讀檢視研究狀態、審查進度、稽核摘要、動物實驗概覽

use axum::{extract::State, Extension, Json};

use crate::{
    middleware::CurrentUser,
    require_permission,
    services::{QauDashboard, QauService},
    AppState, Result,
};

/// 取得 QAU 儀表板
/// GET /qau/dashboard
/// 需 qau.dashboard.view 權限
pub async fn get_qau_dashboard(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<QauDashboard>> {
    require_permission!(current_user, "qau.dashboard.view");

    let dashboard = QauService::get_dashboard(&state.db).await?;
    Ok(Json(dashboard))
}
