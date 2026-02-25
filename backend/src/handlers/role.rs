use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    handlers::user::require_reauth_token,
    middleware::CurrentUser,
    models::{CreateRoleRequest, Permission, PermissionQuery, RoleWithPermissions, UpdateRoleRequest},
    require_permission,
    services::{AuditService, RoleService},
    AppError, AppState, Result,
};

/// 建立角色
#[utoipa::path(
    post,
    path = "/api/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 200, description = "建立成功", body = RoleWithPermissions),
        (status = 400, description = "驗證錯誤", body = ErrorResponse),
    ),
    tag = "角色權限",
    security(("bearer" = []))
)]
pub async fn create_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "dev.role.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let role = RoleService::create(&state.db, &req).await?;
    let response = RoleService::get_by_id(&state.db, role.id).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "SYSTEM", "ROLE_CREATE",
        Some("role"), Some(role.id), Some(&role.name),
        None,
        Some(serde_json::json!({ "name": role.name })),
        None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (ROLE_CREATE): {}", e);
    }

    Ok(Json(response))
}

/// 列出所有角色
#[utoipa::path(
    get,
    path = "/api/roles",
    responses(
        (status = 200, description = "角色清單", body = Vec<RoleWithPermissions>),
    ),
    tag = "角色權限",
    security(("bearer" = []))
)]
pub async fn list_roles(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<RoleWithPermissions>>> {
    require_permission!(current_user, "dev.role.view");
    
    let roles = RoleService::list(&state.db).await?;
    Ok(Json(roles))
}

/// 取得單個角色
#[utoipa::path(
    get,
    path = "/api/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "角色 ID")
    ),
    responses(
        (status = 200, description = "角色資訊", body = RoleWithPermissions),
        (status = 404, description = "角色不存在", body = ErrorResponse),
    ),
    tag = "角色權限",
    security(("bearer" = []))
)]
pub async fn get_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "dev.role.view");
    
    let role = RoleService::get_by_id(&state.db, id).await?;
    Ok(Json(role))
}

/// 更新角色
#[utoipa::path(
    put,
    path = "/api/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "角色 ID")
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "更新成功", body = RoleWithPermissions),
        (status = 400, description = "驗證錯誤", body = ErrorResponse),
    ),
    tag = "角色權限",
    security(("bearer" = []))
)]
pub async fn update_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "dev.role.edit");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let role = RoleService::update(&state.db, id, &req).await?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "SYSTEM", "ROLE_UPDATE",
        Some("role"), Some(id), Some(&role.name),
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (ROLE_UPDATE): {}", e);
    }

    Ok(Json(role))
}

/// 刪除角色
#[utoipa::path(
    delete,
    path = "/api/roles/{id}",
    params(
        ("id" = Uuid, Path, description = "角色 ID")
    ),
    responses(
        (status = 200, description = "刪除成功"),
        (status = 403, description = "需帶 X-Reauth-Token 重新確認密碼", body = ErrorResponse),
        (status = 404, description = "角色不存在", body = ErrorResponse),
    ),
    tag = "角色權限",
    security(("bearer" = []))
)]
pub async fn delete_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dev.role.delete");
    require_reauth_token(&headers, &state, &current_user)?;

    if let Err(e) = AuditService::log_activity(
        &state.db, current_user.id, "SYSTEM", "ROLE_DELETE",
        Some("role"), Some(id), None,
        None, None, None, None,
    ).await {
        tracing::error!("寫入審計日誌失敗 (ROLE_DELETE): {}", e);
    }
    
    RoleService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Role deleted successfully" })))
}

/// 列出所有權限
#[utoipa::path(
    get,
    path = "/api/permissions",
    responses(
        (status = 200, description = "權限清單", body = Vec<Permission>),
    ),
    tag = "角色權限",
    security(("bearer" = []))
)]
pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PermissionQuery>,
) -> Result<Json<Vec<Permission>>> {
    require_permission!(current_user, "dev.role.view");
    
    let permissions = RoleService::list_permissions(&state.db, Some(&query)).await?;
    Ok(Json(permissions))
}
