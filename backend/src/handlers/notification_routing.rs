// 通知路由規則管理 Handler（Admin only）

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    models::{
        NotificationRouting, CreateNotificationRoutingRequest,
        UpdateNotificationRoutingRequest,
    },
    services::NotificationService,
    AppState,
};

/// 列出所有通知路由規則
pub async fn list_notification_routing(
    State(state): State<AppState>,
) -> Result<Json<Vec<NotificationRouting>>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let rules = service.list_notification_routing().await?;
    Ok(Json(rules))
}

/// 建立通知路由規則
pub async fn create_notification_routing(
    State(state): State<AppState>,
    Json(request): Json<CreateNotificationRoutingRequest>,
) -> Result<(StatusCode, Json<NotificationRouting>), AppError> {
    request.validate().map_err(|e| AppError::BadRequest(e.to_string()))?;

    let service = NotificationService::new(state.db.clone());
    let rule = service.create_notification_routing(request).await?;
    Ok((StatusCode::CREATED, Json(rule)))
}

/// 更新通知路由規則
pub async fn update_notification_routing(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateNotificationRoutingRequest>,
) -> Result<Json<NotificationRouting>, AppError> {
    let service = NotificationService::new(state.db.clone());
    let rule = service.update_notification_routing(id, request).await?;
    Ok(Json(rule))
}

/// 刪除通知路由規則
pub async fn delete_notification_routing(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let service = NotificationService::new(state.db.clone());
    service.delete_notification_routing(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
