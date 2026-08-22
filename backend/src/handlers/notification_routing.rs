// 通知路由規則管理 Handler（Admin only）

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::CurrentUser,
    models::{
        CreateNotificationRoutingRequest, NotificationRouting, UpdateNotificationRoutingRequest,
    },
    services::{access::require_notification_routing_manage, NotificationService},
    AppState,
};

/// 列出所有通知路由規則
pub async fn list_notification_routing(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<NotificationRouting>>, AppError> {
    require_notification_routing_manage(&current_user)?;
    let service = NotificationService::new(state.db.clone());
    let rules = service.list_notification_routing().await?;
    Ok(Json(rules))
}

/// 建立通知路由規則
pub async fn create_notification_routing(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(request): Json<CreateNotificationRoutingRequest>,
) -> Result<(StatusCode, Json<NotificationRouting>), AppError> {
    require_notification_routing_manage(&current_user)?;
    request
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let service = NotificationService::new(state.db.clone());
    let rule = service.create_notification_routing(request).await?;
    Ok((StatusCode::CREATED, Json(rule)))
}

/// 更新通知路由規則
pub async fn update_notification_routing(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateNotificationRoutingRequest>,
) -> Result<Json<NotificationRouting>, AppError> {
    require_notification_routing_manage(&current_user)?;
    let service = NotificationService::new(state.db.clone());
    let rule = service.update_notification_routing(id, request).await?;
    Ok(Json(rule))
}

/// 刪除通知路由規則
pub async fn delete_notification_routing(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    require_notification_routing_manage(&current_user)?;
    let service = NotificationService::new(state.db.clone());
    service.delete_notification_routing(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 取得所有可用事件類型（含分類）
pub async fn list_available_event_types(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<crate::models::EventTypeCategory>>, AppError> {
    require_notification_routing_manage(&current_user)?;
    Ok(Json(
        crate::services::NotificationService::list_available_event_types(),
    ))
}

/// 取得各事件路由規則的「具體收件人」預覽（供 admin 路由頁巢狀收合顯示）。
pub async fn list_routing_recipients(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<crate::models::RoutingRecipientsPreview>>, AppError> {
    require_notification_routing_manage(&current_user)?;
    let service = NotificationService::new(state.db.clone());
    let preview = service.routing_recipients_preview().await?;
    Ok(Json(preview))
}

/// 取得「固定通知」目錄（流程決定收件人、不經路由、不可調整），供路由頁顯示為唯讀 notes。
pub async fn list_fixed_notifications(
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<crate::models::FixedNotification>>, AppError> {
    require_notification_routing_manage(&current_user)?;
    Ok(Json(
        crate::services::NotificationService::list_fixed_notifications(),
    ))
}

/// 取得所有可用角色
pub async fn list_available_roles(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<crate::models::RoleInfo>>, AppError> {
    require_notification_routing_manage(&current_user)?;
    let service = NotificationService::new(state.db.clone());
    let roles = service.list_available_roles().await?;
    Ok(Json(roles))
}
