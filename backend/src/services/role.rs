use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{CreateRoleRequest, Permission, PermissionQuery, Role, RoleWithPermissions, UpdateRoleRequest},
    AppError, Result,
};

pub struct RoleService;

impl RoleService {
    /// 建立角色
    pub async fn create(pool: &PgPool, req: &CreateRoleRequest) -> Result<Role> {
        // 檢查 code 是否已存在
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1)"
        )
        .bind(&req.code)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(AppError::Conflict("Role code already exists".to_string()));
        }

        // 建立角色
        let role = sqlx::query_as::<_, Role>(
            r#"
            INSERT INTO roles (id, code, name, description, is_internal, is_system, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, false, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&req.code)
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.is_internal)
        .fetch_one(pool)
        .await?;

        // 指派權限
        for permission_id in &req.permission_ids {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(role.id)
                .bind(permission_id)
                .execute(pool)
                .await?;
        }

        Ok(role)
    }

    /// 取得角色列表（含權限）
    pub async fn list(pool: &PgPool) -> Result<Vec<RoleWithPermissions>> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE is_active = true ORDER BY code"
        )
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for role in roles {
            let permissions = sqlx::query_as::<_, Permission>(
                r#"
                SELECT p.* FROM permissions p
                INNER JOIN role_permissions rp ON p.id = rp.permission_id
                WHERE rp.role_id = $1
                ORDER BY p.code
                "#
            )
            .bind(role.id)
            .fetch_all(pool)
            .await?;

            result.push(RoleWithPermissions {
                id: role.id,
                code: role.code,
                name: role.name,
                description: role.description,
                is_internal: role.is_internal,
                is_system: role.is_system,
                is_active: role.is_active,
                permissions,
                created_at: role.created_at,
                updated_at: role.updated_at,
            });
        }

        Ok(result)
    }

    /// 取得單一角色
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<RoleWithPermissions> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.* FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.code
            "#
        )
        .bind(role.id)
        .fetch_all(pool)
        .await?;

        Ok(RoleWithPermissions {
            id: role.id,
            code: role.code,
            name: role.name,
            description: role.description,
            is_internal: role.is_internal,
            is_system: role.is_system,
            is_active: role.is_active,
            permissions,
            created_at: role.created_at,
            updated_at: role.updated_at,
        })
    }

    /// 更新角色
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateRoleRequest) -> Result<RoleWithPermissions> {
        // 檢查角色是否存在
        let existing = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1 AND is_active = true")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        // 更新角色
        let role = sqlx::query_as::<_, Role>(
            r#"
            UPDATE roles SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                is_internal = COALESCE($3, is_internal),
                updated_at = NOW()
            WHERE id = $4 AND is_active = true
            RETURNING *
            "#
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.is_internal)
        .bind(id)
        .fetch_one(pool)
        .await?;

        // 如果要更新權限
        if let Some(ref permission_ids) = req.permission_ids {
            // 刪除現有權限
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(id)
                .execute(pool)
                .await?;

            // 指派新權限
            for permission_id in permission_ids {
                sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
                    .bind(id)
                    .bind(permission_id)
                    .execute(pool)
                    .await?;
            }
        }

        Self::get_by_id(pool, role.id).await
    }

    /// 刪除角色
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        // 檢查是否為系統角色
        let role = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1 AND is_active = true")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        if role.is_system {
            sqlx::query("UPDATE roles SET is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            return Ok(());
        }

        // 先刪除關聯
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        sqlx::query("DELETE FROM user_roles WHERE role_id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        let result = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Role not found".to_string()));
        }

        Ok(())
    }

    /// 取得所有權限（去重，確保每個 code 只返回一條記錄）
    pub async fn list_permissions(pool: &PgPool, query: Option<&PermissionQuery>) -> Result<Vec<Permission>> {
        let permissions = if let Some(q) = query {
            if let Some(ref module) = q.module {
                sqlx::query_as::<_, Permission>(
                    r#"
                    SELECT DISTINCT ON (code) *
                    FROM permissions 
                    WHERE module = $1 
                    ORDER BY code, created_at DESC
                    "#
                )
                .bind(module)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, Permission>(
                    r#"
                    SELECT DISTINCT ON (code) *
                    FROM permissions 
                    ORDER BY code, created_at DESC
                    "#
                )
                .fetch_all(pool)
                .await?
            }
        } else {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT DISTINCT ON (code) *
                FROM permissions 
                ORDER BY code, created_at DESC
                "#
            )
            .fetch_all(pool)
            .await?
        };

        Ok(permissions)
    }
}
