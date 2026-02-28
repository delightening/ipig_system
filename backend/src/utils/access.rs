//! 資源存取權限檢查 helper，用於防範 IDOR（Insecure Direct Object Reference）

use uuid::Uuid;

use crate::{error::AppError, middleware::CurrentUser, Result};

/// 檢查使用者是否有權存取資源
/// - 資源擁有者（resource_owner_id == current_user.id）可存取
/// - 具備 admin_permission 權限者可存取（如 hr.leave.view_all）
/// - 否則回傳 403 Forbidden
pub fn check_resource_access(
    current_user: &CurrentUser,
    resource_owner_id: Uuid,
    admin_permission: &str,
) -> Result<()> {
    if current_user.id == resource_owner_id || current_user.has_permission(admin_permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden("無權存取此資源".into()))
    }
}
