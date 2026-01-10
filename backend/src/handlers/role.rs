use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    middleware::CurrentUser,
    models::{CreateRoleRequest, Permission, PermissionQuery, RoleWithPermissions, UpdateRoleRequest},
    require_permission,
    services::RoleService,
    AppError, AppState, Result,
};

/// 建立角色
pub async fn create_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "role.create");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let role = RoleService::create(&state.db, &req).await?;
    let response = RoleService::get_by_id(&state.db, role.id).await?;
    Ok(Json(response))
}

/// 取得角色列表
pub async fn list_roles(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<RoleWithPermissions>>> {
    require_permission!(current_user, "role.read");
    
    let roles = RoleService::list(&state.db).await?;
    Ok(Json(roles))
}

/// 取得單一角色
pub async fn get_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "role.read");
    
    let role = RoleService::get_by_id(&state.db, id).await?;
    Ok(Json(role))
}

/// 更新角色
pub async fn update_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "role.update");
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    
    let role = RoleService::update(&state.db, id, &req).await?;
    Ok(Json(role))
}

/// 刪除角色
pub async fn delete_role(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "role.delete");
    
    RoleService::delete(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "Role deleted successfully" })))
}

/// 取得所有權限
pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<PermissionQuery>,
) -> Result<Json<Vec<Permission>>> {
    require_permission!(current_user, "role.read");
    
    let permissions = RoleService::list_permissions(&state.db, Some(&query)).await?;
    Ok(Json(permissions))
}
