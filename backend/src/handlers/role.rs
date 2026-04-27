use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::HeaderMap,
    Extension, Json,
};
use std::net::SocketAddr;
use uuid::Uuid;
use validator::Validate;

use crate::error::ErrorResponse;
use crate::{
    handlers::user::require_reauth_token,
    middleware::{extract_real_ip_with_trust, ActorContext, CurrentUser},
    models::{CreateRoleRequest, Permission, PermissionQuery, RoleWithPermissions, UpdateRoleRequest},
    require_permission,
    services::RoleService,
    AppState, Result,
};

/// 建立角色
#[utoipa::path(
    post,
    path = "/api/v1/roles",
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "dev.role.create");
    req.validate()?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let actor = ActorContext::User(current_user.clone());
    let role = RoleService::create(&state.db, &actor, &req, Some(&ip), user_agent).await?;
    let response = RoleService::get_by_id(&state.db, role.id).await?;

    Ok(Json(response))
}

/// 列出所有角色
#[utoipa::path(
    get,
    path = "/api/v1/roles",
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
    path = "/api/v1/roles/{id}",
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
    path = "/api/v1/roles/{id}",
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<Json<RoleWithPermissions>> {
    require_permission!(current_user, "dev.role.edit");
    req.validate()?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let actor = ActorContext::User(current_user.clone());
    let role = RoleService::update(&state.db, &actor, id, &req, Some(&ip), user_agent).await?;

    // H-01: 角色權限變更後清空全部快取（無法預知哪些使用者持有此角色）
    state.permission_cache.invalidate_all();

    Ok(Json(role))
}

/// 刪除角色
#[utoipa::path(
    delete,
    path = "/api/v1/roles/{id}",
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(current_user): Extension<CurrentUser>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    require_permission!(current_user, "dev.role.delete");
    require_reauth_token(&headers, &state, &current_user)?;

    let ip = extract_real_ip_with_trust(&headers, &addr, state.config.trust_proxy_headers);
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let actor = ActorContext::User(current_user.clone());
    RoleService::delete(&state.db, &actor, id, Some(&ip), user_agent).await?;
    // H-01: 角色刪除後清空全部快取
    state.permission_cache.invalidate_all();
    Ok(Json(serde_json::json!({ "message": "Role deleted successfully" })))
}

/// 列出所有權限
#[utoipa::path(
    get,
    path = "/api/v1/permissions",
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
