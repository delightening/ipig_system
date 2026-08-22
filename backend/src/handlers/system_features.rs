//! R30-27b: 對前端公開的 feature flag 端點
//!
//! GET /api/v1/system/features
//!
//! 回傳目前啟用的 feature flag 給前端，前端據此決定是否顯示對應 UI（如
//! role/permission 變更簽章 dialog）。需登入但**不需 admin**：UI 渲染決策對所有
//! 已認證 user 可見即可，flag 本身不是 secret。
//!
//! 設計原則：只回傳「影響 UI 行為」的 flag；config 內其他 secret-adjacent flag
//! （如 audit_hmac_key、jwt_keys）不在此端點暴露。

use axum::{extract::State, Extension, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{middleware::CurrentUser, AppState, Result};

/// 對前端公開的 feature flag 集合（影響 UI 行為）
#[derive(Serialize, ToSchema)]
pub struct SystemFeatures {
    /// R30-27：role / permission 變更是否強制密碼 + 手寫雙因子簽章
    pub role_signature_required: bool,
}

/// GET /system/features
///
/// 回傳目前後端啟用的 feature flag。前端 UI 用以決定是否顯示簽章 dialog 等
/// 條件 UI。已登入即可呼叫（不需 admin）。
#[utoipa::path(
    get,
    path = "/api/v1/system/features",
    responses(
        (status = 200, description = "feature flag 集合", body = SystemFeatures),
        (status = 401, description = "未認證"),
    ),
    tag = "系統"
)]
pub async fn get_system_features(
    State(state): State<AppState>,
    Extension(_current_user): Extension<CurrentUser>,
) -> Result<Json<SystemFeatures>> {
    Ok(Json(SystemFeatures {
        role_signature_required: state.config.role_signature_required,
    }))
}
