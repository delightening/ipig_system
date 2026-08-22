use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::{AuditRedact, DataDiff},
        CreateRoleRequest, MutationSignaturePayload, Permission, PermissionQuery, Role,
        RoleWithPermissions, UpdateRoleRequest,
    },
    repositories,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        signature::SignatureType,
        AuditService, SignatureService,
    },
    AppError, Result,
};

/// R30-27：role/permission mutation 強制簽章 entity_type 常數
const ROLE_SIG_ENTITY: &str = "role";

/// R30-27：依 flag 決定是否要求簽章；要求時呼叫 sign_record_tx 寫入
/// electronic_signatures（密碼 + 手寫雙因子，21 CFR §11.10(d)）。
///
/// `content` 應為 canonical 描述本次變更（role.code、欄位 hash、permission_ids hash 等），
/// 寫入 signature.content_hash 供日後鑑定「簽章對應的內容是否被竄改」。
async fn enforce_role_signature_tx(
    tx: &mut Transaction<'_, Postgres>,
    pool: &PgPool,
    actor: &ActorContext,
    require_signature: bool,
    payload: Option<&MutationSignaturePayload>,
    signer_id: Uuid,
    role_id: Uuid,
    content: &str,
) -> Result<Option<Uuid>> {
    if !require_signature {
        return Ok(None);
    }

    let p = payload.ok_or_else(|| {
        AppError::BadRequest(
            "本操作需電子簽章（密碼 + 手寫）— 缺少 mutation_signature payload".into(),
        )
    })?;
    if p.password.is_empty() {
        return Err(AppError::Validation("簽章密碼不可為空".into()));
    }
    if p.handwriting_svg.is_empty() {
        return Err(AppError::Validation(
            "簽章必須提供手寫簽名（handwriting_svg）".into(),
        ));
    }

    let sig = SignatureService::sign_record_tx(
        tx,
        pool,
        actor,
        ROLE_SIG_ENTITY,
        &role_id.to_string(),
        signer_id,
        SignatureType::Approve,
        content,
        Some(&p.password),
        Some(&p.handwriting_svg),
        p.stroke_data.as_ref(),
    )
    .await?;

    Ok(Some(sig.id))
}

/// R30-27：將 role 的關鍵欄位編碼為 canonical content（簽章 content_hash 來源）。
/// 包 role.id、code、name、is_internal、權限集合 hash — 任何欄位變動都會改 hash。
fn canonical_role_content(
    op: &str,
    role_id: Uuid,
    code: &str,
    name: &str,
    is_internal: bool,
    permission_codes: &[String],
) -> String {
    // 用 \n 分隔欄位（gemini / coderabbit review #293：role.name 是自由文字可含 |，
    // 改用 \n 避免欄位內容與 delimiter 衝突造成 hash collision）。
    // permission_codes 已 sorted（caller 保證），內部用 , 分隔（permission code 不含 ,）。
    format!(
        "role_{op}\nid={role_id}\ncode={code}\nname={name}\ninternal={is_internal}\nperms={perms}",
        perms = permission_codes.join(",")
    )
}

/// Audit-only 輔助型別：記錄角色權限指派的 before/after（用於 ROLE_PERMISSION_CHANGE 子事件）。
/// 權限清單無敏感欄位，空 AuditRedact impl 即可。
#[derive(Serialize)]
struct RolePermissionSnapshot {
    permissions: Vec<String>,
}
impl AuditRedact for RolePermissionSnapshot {}

/// 驗證角色 code 格式：長度 1–50，僅允許英數字與底線。與 DB 與 CreateRoleRequest 約定一致。
pub fn is_valid_role_code(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() || t.len() > 50 {
        return false;
    }
    t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(sqlx::FromRow)]
struct RolePermissionRow {
    role_id: Uuid,
    #[sqlx(flatten)]
    permission: Permission,
}

pub struct RoleService;

impl RoleService {
    /// 建立角色 — Service-driven audit
    ///
    /// Actor 政策：僅允許 User（dev.role.create 權限）。System 自動化不應建立角色。
    ///
    /// R30-27：`require_signature=true` 時必須帶 `req.mutation_signature` 雙因子簽章。
    pub async fn create(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateRoleRequest,
        ip: Option<&str>,
        user_agent: Option<&str>,
        require_signature: bool,
    ) -> Result<Role> {
        let user = actor.require_user()?;
        if !is_valid_role_code(&req.code) {
            return Err(AppError::Validation(
                "Role code must be 1-50 characters and only contain letters, numbers, and underscore".to_string(),
            ));
        }

        let mut tx = pool.begin().await?;

        // 檢查 code 是否已存在
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1)")
            .bind(&req.code)
            .fetch_one(&mut *tx)
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
        .fetch_one(&mut *tx)
        .await?;

        if !req.permission_ids.is_empty() {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING"
            )
            .bind(role.id)
            .bind(&req.permission_ids)
            .execute(&mut *tx)
            .await?;
        }

        // R30-27：強制簽章（flag=true 時）。content 包剛建立的 role 欄位 + 權限集合 hash。
        let perm_codes: Vec<String> = if req.permission_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query_scalar(
                r#"SELECT p.code FROM permissions p WHERE p.id = ANY($1) ORDER BY p.code"#,
            )
            .bind(&req.permission_ids)
            .fetch_all(&mut *tx)
            .await?
        };
        let sig_content = canonical_role_content(
            "create",
            role.id,
            &role.code,
            &role.name,
            role.is_internal,
            &perm_codes,
        );
        enforce_role_signature_tx(
            &mut tx,
            pool,
            actor,
            require_signature,
            req.mutation_signature.as_ref(),
            user.id,
            role.id,
            &sig_content,
        )
        .await?;

        // SECURITY audit：角色建立屬於權限管理關鍵操作
        let display = format!("{} ({})", role.name, role.code);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "ROLE_CREATE",
                entity: Some(AuditEntity::new("role", role.id, &display)),
                data_diff: Some(DataDiff::create_only(&role)),
                request_context: Some(crate::services::audit::RequestContext {
                    ip_address: ip,
                    user_agent,
                }),
            },
        )
        .await?;

        tx.commit().await?;
        Ok(role)
    }

    /// 取得角色列表（含權限）— 2 次查詢取代 1+N
    pub async fn list(pool: &PgPool) -> Result<Vec<RoleWithPermissions>> {
        let roles =
            sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE is_active = true ORDER BY code")
                .fetch_all(pool)
                .await?;

        let role_ids: Vec<Uuid> = roles.iter().map(|r| r.id).collect();

        let mut perm_map: HashMap<Uuid, Vec<Permission>> = HashMap::new();
        if !role_ids.is_empty() {
            let rows = sqlx::query_as::<_, RolePermissionRow>(
                r#"
                SELECT rp.role_id,
                       p.id, p.code, p.name, p.module, p.description, p.created_at
                FROM permissions p
                INNER JOIN role_permissions rp ON p.id = rp.permission_id
                WHERE rp.role_id = ANY($1)
                ORDER BY p.code
                "#,
            )
            .bind(&role_ids)
            .fetch_all(pool)
            .await?;

            for row in rows {
                perm_map
                    .entry(row.role_id)
                    .or_default()
                    .push(row.permission);
            }
        }

        let result = roles
            .into_iter()
            .map(|role| {
                let permissions = perm_map.remove(&role.id).unwrap_or_default();
                RoleWithPermissions {
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
                }
            })
            .collect();

        Ok(result)
    }

    /// 取得單一角色
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<RoleWithPermissions> {
        let role = repositories::role::find_role_by_id_active(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.* FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1
            ORDER BY p.code
            "#,
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

    /// 更新角色 — Service-driven audit（含權限變更子事件）
    ///
    /// R30-27：`require_signature=true` 時必須帶 `req.mutation_signature` 雙因子簽章。
    pub async fn update(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        req: &UpdateRoleRequest,
        ip: Option<&str>,
        user_agent: Option<&str>,
        require_signature: bool,
    ) -> Result<RoleWithPermissions> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE 取 before
        let before = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE id = $1 AND is_active = true FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        // 更新角色
        let after = sqlx::query_as::<_, Role>(
            r#"
            UPDATE roles SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                is_internal = COALESCE($3, is_internal),
                updated_at = NOW()
            WHERE id = $4 AND is_active = true
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(req.is_internal)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        // 基本 ROLE_UPDATE 事件（SECURITY：角色基本資料變更）
        let display = format!("{} ({})", after.name, after.code);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "SECURITY",
                event_type: "ROLE_UPDATE",
                entity: Some(AuditEntity::new("role", after.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: Some(crate::services::audit::RequestContext {
                    ip_address: ip,
                    user_agent,
                }),
            },
        )
        .await?;

        // 權限變更額外寫一筆 SECURITY 事件（供稽核員篩選「誰何時改了哪個角色的權限」）
        if let Some(ref permission_ids) = req.permission_ids {
            // before 權限
            let before_perms: Vec<String> = sqlx::query_scalar(
                r#"SELECT p.code FROM role_permissions rp
                   INNER JOIN permissions p ON rp.permission_id = p.id
                   WHERE rp.role_id = $1 ORDER BY p.code"#,
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

            // 刪除現有權限
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            if !permission_ids.is_empty() {
                sqlx::query(
                    "INSERT INTO role_permissions (role_id, permission_id) SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING"
                )
                .bind(id)
                .bind(permission_ids)
                .execute(&mut *tx)
                .await?;
            }

            let after_perms: Vec<String> = sqlx::query_scalar(
                r#"SELECT p.code FROM role_permissions rp
                   INNER JOIN permissions p ON rp.permission_id = p.id
                   WHERE rp.role_id = $1 ORDER BY p.code"#,
            )
            .bind(id)
            .fetch_all(&mut *tx)
            .await?;

            if before_perms != after_perms {
                let before_snap = RolePermissionSnapshot {
                    permissions: before_perms,
                };
                let after_snap = RolePermissionSnapshot {
                    permissions: after_perms,
                };
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "SECURITY",
                        event_type: "ROLE_PERMISSION_CHANGE",
                        entity: Some(AuditEntity::new("role", after.id, &display)),
                        data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                        request_context: Some(crate::services::audit::RequestContext {
                            ip_address: ip,
                            user_agent,
                        }),
                    },
                )
                .await?;
            }
        }

        // R30-27：強制簽章（flag=true 時）。content 包 update 後最終 role 欄位 + 權限集合。
        let final_perm_codes: Vec<String> = sqlx::query_scalar(
            r#"SELECT p.code FROM role_permissions rp
               INNER JOIN permissions p ON rp.permission_id = p.id
               WHERE rp.role_id = $1 ORDER BY p.code"#,
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let sig_content = canonical_role_content(
            "update",
            after.id,
            &after.code,
            &after.name,
            after.is_internal,
            &final_perm_codes,
        );
        enforce_role_signature_tx(
            &mut tx,
            pool,
            actor,
            require_signature,
            req.mutation_signature.as_ref(),
            user.id,
            after.id,
            &sig_content,
        )
        .await?;

        tx.commit().await?;
        Self::get_by_id(pool, after.id).await
    }

    /// 刪除角色 — Service-driven audit
    ///
    /// system role 改軟刪除（is_active=false）；非 system role 硬刪除含關聯清理。
    ///
    /// R30-27：`require_signature=true` 時必須帶 `signature` 雙因子簽章。
    pub async fn delete(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        ip: Option<&str>,
        user_agent: Option<&str>,
        require_signature: bool,
        signature: Option<&MutationSignaturePayload>,
    ) -> Result<()> {
        let user = actor.require_user()?;
        let mut tx = pool.begin().await?;

        // SELECT FOR UPDATE 取 before（含 is_active 濾掉已軟刪的）
        let before = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE id = $1 AND is_active = true FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        let display = format!("{} ({})", before.name, before.code);

        // R30-27：強制簽章（flag=true 時）。content 包 delete 前 role 欄位 + 權限集合。
        // 簽章必須先於 delete 動作，以便簽章內容對應的是「被刪掉的 role 樣貌」。
        let perm_codes: Vec<String> = sqlx::query_scalar(
            r#"SELECT p.code FROM role_permissions rp
               INNER JOIN permissions p ON rp.permission_id = p.id
               WHERE rp.role_id = $1 ORDER BY p.code"#,
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let sig_content = canonical_role_content(
            "delete",
            before.id,
            &before.code,
            &before.name,
            before.is_internal,
            &perm_codes,
        );
        enforce_role_signature_tx(
            &mut tx,
            pool,
            actor,
            require_signature,
            signature,
            user.id,
            before.id,
            &sig_content,
        )
        .await?;

        if before.is_system {
            // 軟刪除 system role
            let after = sqlx::query_as::<_, Role>(
                "UPDATE roles SET is_active = false, updated_at = NOW() WHERE id = $1 RETURNING *",
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "SECURITY",
                    event_type: "ROLE_DEACTIVATE",
                    entity: Some(AuditEntity::new("role", before.id, &display)),
                    data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                    request_context: Some(crate::services::audit::RequestContext {
                        ip_address: ip,
                        user_agent,
                    }),
                },
            )
            .await?;
        } else {
            // 非 system role：硬刪除 + 清關聯
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            sqlx::query("DELETE FROM user_roles WHERE role_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            let result = sqlx::query("DELETE FROM roles WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;

            if result.rows_affected() == 0 {
                return Err(AppError::NotFound("Role not found".to_string()));
            }

            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "SECURITY",
                    event_type: "ROLE_DELETE",
                    entity: Some(AuditEntity::new("role", before.id, &display)),
                    data_diff: Some(DataDiff::delete_only(&before)),
                    request_context: Some(crate::services::audit::RequestContext {
                        ip_address: ip,
                        user_agent,
                    }),
                },
            )
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// 取得所有權限（去重，確保每個 code 只返回一條記錄）
    pub async fn list_permissions(
        pool: &PgPool,
        query: Option<&PermissionQuery>,
    ) -> Result<Vec<Permission>> {
        let permissions = if let Some(q) = query {
            if let Some(ref module) = q.module {
                sqlx::query_as::<_, Permission>(
                    r#"
                    SELECT DISTINCT ON (code) *
                    FROM permissions 
                    WHERE module = $1 
                    ORDER BY code, created_at DESC
                    "#,
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
                    "#,
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
                "#,
            )
            .fetch_all(pool)
            .await?
        };

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::{canonical_role_content, is_valid_role_code};
    use uuid::Uuid;

    #[test]
    fn test_canonical_role_content_deterministic() {
        // R30-27：同樣 input → 同樣 content；任一欄位變動 → content 改變。
        // 確保 signature.content_hash 在「事後鑑定」時可重現比對。
        let id = Uuid::nil();
        let perms = vec!["aup.read".to_string(), "aup.write".to_string()];
        let a = canonical_role_content("create", id, "ADMIN", "管理員", true, &perms);
        let b = canonical_role_content("create", id, "ADMIN", "管理員", true, &perms);
        assert_eq!(a, b);

        // 改 perm 集合 → content 變
        let c = canonical_role_content(
            "create",
            id,
            "ADMIN",
            "管理員",
            true,
            &["aup.read".to_string()],
        );
        assert_ne!(a, c);
        // 改 op → content 變（防止 create / update 簽章互相重放）
        let d = canonical_role_content("update", id, "ADMIN", "管理員", true, &perms);
        assert_ne!(a, d);

        // gemini / coderabbit review #293 regression：name 含 `|` 不能造成欄位
        // 邊界混淆。改用 `\n` 分隔後，"a|b" 與 "a" + "b" 兩種 name 應仍產生不同 hash。
        let collide_a = canonical_role_content("create", id, "X", "name|extra=evil", true, &perms);
        let collide_b = canonical_role_content("create", id, "X", "name", true, &perms);
        assert_ne!(collide_a, collide_b);
    }

    #[test]
    fn test_is_valid_role_code_valid() {
        assert!(is_valid_role_code("admin"));
        assert!(is_valid_role_code("role_1"));
        assert!(is_valid_role_code("A1_b"));
        assert!(is_valid_role_code("a"));
        assert!(is_valid_role_code("  editor  "));
    }

    #[test]
    fn test_is_valid_role_code_empty_or_too_long() {
        assert!(!is_valid_role_code(""));
        assert!(!is_valid_role_code("   "));
        assert!(!is_valid_role_code(&"a".repeat(51)));
    }

    #[test]
    fn test_is_valid_role_code_invalid_chars() {
        assert!(!is_valid_role_code("role-code"));
        assert!(!is_valid_role_code("role.code"));
        assert!(!is_valid_role_code("角色"));
        assert!(!is_valid_role_code("a b"));
    }
}
