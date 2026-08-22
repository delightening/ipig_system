//! 動物試驗申請須知版本 handlers（院區層級，admin 管理 + 申請人讀取生效版本）。

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{ApplicationNotice, ApplicationNoticeResponse, CreateApplicationNoticeRequest},
    require_permission,
    services::ApplicationNoticeService,
    AppState, Result,
};

const PERM: &str = "aup.application_notice.manage";

/// 當前生效須知（任一登入使用者可讀，供申請人填表/簽署）。
/// 路由位於認證 router 下（已登入即可）；須知正文為院區層級參考資料，無須額外權限。
pub async fn get_active_notice(
    State(state): State<AppState>,
) -> Result<Json<Option<ApplicationNotice>>> {
    Ok(Json(ApplicationNoticeService::get_active(&state.db).await?))
}

/// 須知版本列表（admin）。
pub async fn list_notices(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ApplicationNoticeResponse>>> {
    require_permission!(current_user, PERM);
    Ok(Json(ApplicationNoticeService::list(&state.db).await?))
}

/// 新增須知版本（admin）。
pub async fn create_notice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateApplicationNoticeRequest>,
) -> Result<(StatusCode, Json<ApplicationNotice>)> {
    require_permission!(current_user, PERM);
    req.validate()?;
    let actor = ActorContext::User(current_user);
    let notice = ApplicationNoticeService::create(&state.db, &actor, &req).await?;
    Ok((StatusCode::CREATED, Json(notice)))
}

/// 設為當前生效版本（admin）。
pub async fn activate_notice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApplicationNotice>> {
    require_permission!(current_user, PERM);
    let actor = ActorContext::User(current_user);
    let notice = ApplicationNoticeService::activate(&state.db, &actor, id).await?;
    Ok(Json(notice))
}
