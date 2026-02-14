use sqlx::PgPool;
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    models::ProtocolListItem,
    AppError, Result,
};

impl ProtocolService {
    /// 取得我的計畫列表（依使用者）
    /// 支援委託單位主管：如果用戶是 CLIENT 角色且為主管，可查看同組織下所有用戶的計畫
    /// 支援特權角色：如果用戶是管理員、獸醫師或 IACUC 工作人員，可查看所有計畫
    pub async fn get_my_protocols(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProtocolListItem>> {
        // 檢查用戶角色與權限
        let user_info: Option<(String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT 
                COALESCE(string_agg(DISTINCT r.code, ','), '') as roles,
                COALESCE(string_agg(DISTINCT p.code, ','), '') as permissions,
                u.organization
            FROM users u
            LEFT JOIN user_roles ur ON u.id = ur.user_id
            LEFT JOIN roles r ON ur.role_id = r.id
            LEFT JOIN role_permissions rp ON r.id = rp.role_id
            LEFT JOIN permissions p ON rp.permission_id = p.id
            WHERE u.id = $1
            GROUP BY u.id, u.organization
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let (roles_str, permissions_str, organization) = user_info.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        let roles: Vec<&str> = roles_str.split(',').filter(|s| !s.is_empty()).collect();
        let permissions: Vec<&str> = permissions_str.split(',').filter(|s| !s.is_empty()).collect();
        
        let has_view_all_permission = permissions.contains(&"aup.protocol.view_all");
        let has_client_role = roles.contains(&"CLIENT");
        
        let is_vet_or_reviewer = roles.contains(&"VET") || roles.contains(&"REVIEWER");
        let has_other_privileged_role = roles.iter().any(|&r| ["admin", "IACUC_STAFF", "IACUC_CHAIR"].contains(&r));
        
        // VET 與 REVIEWER 角色特殊處理：在「我的計畫」中，預設點進來只看受指派的任務（除非具備其他管理員角色）
        // 這樣能將 AUP 管理清單（全面查看）與我的計畫（任務導向）明確區分
        let has_broad_access = (has_view_all_permission || has_other_privileged_role) && (!is_vet_or_reviewer || has_other_privileged_role);

        // 如果具備特權角色或廣泛查看權限，則查看所有計畫
        if has_broad_access {
            return sqlx::query_as::<_, ProtocolListItem>(
                r#"
                SELECT DISTINCT
                    p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                    p.pi_user_id, u.display_name as pi_name, u.organization as pi_organization,
                    p.start_date, p.end_date, p.created_at,
                    NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number
                FROM protocols p
                LEFT JOIN users u ON p.pi_user_id = u.id
                WHERE p.status != 'DELETED'
                ORDER BY p.created_at DESC
                "#
            )
            .fetch_all(pool)
            .await
            .map_err(Into::into);
        }

        // 如果是 CLIENT 角色且有組織，則查看同組織下所有用戶的計畫（委託單位主管權限）
        let protocols = if has_client_role && organization.is_some() {
            sqlx::query_as::<_, ProtocolListItem>(
                r#"
                SELECT DISTINCT
                    p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                    p.pi_user_id, u.display_name as pi_name, u.organization as pi_organization,
                    p.start_date, p.end_date, p.created_at,
                    NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number
                FROM protocols p
                LEFT JOIN users u ON p.pi_user_id = u.id
                WHERE (
                    -- 用戶直接相關的計畫
                    EXISTS (
                        SELECT 1 FROM user_protocols up 
                        WHERE up.protocol_id = p.id AND up.user_id = $1
                    )
                    OR
                    -- 同組織下所有用戶的計畫（委託單位主管權限）
                    (u.organization = $2 AND p.status != 'DELETED')
                )
                AND p.status != 'DELETED'
                ORDER BY p.created_at DESC
                "#
            )
            .bind(user_id)
            .bind(organization.as_deref())
            .fetch_all(pool)
            .await?
        } else {
            // 一般用戶：查看自己相關的計畫 + 被指派審查的計畫
            sqlx::query_as::<_, ProtocolListItem>(
                r#"
                SELECT DISTINCT
                    p.id, p.protocol_no, p.iacuc_no, p.title, p.status,
                    p.pi_user_id, u.display_name as pi_name, u.organization as pi_organization,
                    p.start_date, p.end_date, p.created_at,
                    NULLIF(p.working_content->'basic'->>'apply_study_number', '') as apply_study_number
                FROM protocols p
                LEFT JOIN users u ON p.pi_user_id = u.id
                WHERE p.status != 'DELETED'
                AND (
                    -- 用戶是 PI、CLIENT 或 CO_EDITOR
                    EXISTS (
                        SELECT 1 FROM user_protocols up 
                        WHERE up.protocol_id = p.id AND up.user_id = $1
                    )
                    OR
                    -- 用戶是被指派的審查委員（只能看非草稿狀態）
                    (
                        EXISTS (
                            SELECT 1 FROM review_assignments ra 
                            WHERE ra.protocol_id = p.id AND ra.reviewer_id = $1
                        )
                        AND p.status NOT IN ('DRAFT', 'REVISION_REQUIRED')
                    )
                    OR
                    -- 用戶是被指派的獸醫審查員
                    EXISTS (
                        SELECT 1 FROM vet_review_assignments vra
                        WHERE vra.protocol_id = p.id AND vra.vet_id = $1
                    )
                )
                ORDER BY p.created_at DESC
                "#
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        };

    Ok(protocols)
    }

    /// 儲存獸醫審查表
    pub async fn save_vet_review_form(
        pool: &sqlx::PgPool,
        protocol_id: uuid::Uuid,
        vet_id: uuid::Uuid,
        review_form: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO vet_review_assignments (id, protocol_id, vet_id, assigned_at, review_form)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (protocol_id) DO UPDATE SET
                review_form = $4,
                completed_at = NOW()
            "#
        )
        .bind(uuid::Uuid::new_v4())
        .bind(protocol_id)
        .bind(vet_id)
        .bind(review_form)
        .execute(pool)
        .await?;

        Ok(())
    }
}
