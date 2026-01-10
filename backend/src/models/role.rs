use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_internal: bool,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub module: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 50, message = "Code must be 1-50 characters"))]
    pub code: String,
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_is_internal")]
    pub is_internal: bool,
    #[serde(default)]
    pub permission_ids: Vec<Uuid>,
}

fn default_is_internal() -> bool {
    true
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_internal: Option<bool>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleWithPermissions {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_internal: bool,
    pub is_system: bool,
    pub is_active: bool,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignUserRoleRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PermissionQuery {
    pub module: Option<String>,
}
