use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;

use super::ProtocolService;
use crate::{
    models::{
        AssignCoEditorRequest, AssignReviewerRequest, CoEditorAssignmentResponse, ProtocolActivityType,
        ReviewAssignment, UserProtocol,
    },
    AppError, Result,
};

impl ProtocolService {
    /// 指派審查人員
    pub async fn assign_reviewer(
        pool: &PgPool,
        req: &AssignReviewerRequest,
        assigned_by: Uuid,
    ) -> Result<ReviewAssignment> {
        // H1: 防止自我審核（提交者 ≠ 審查者），確保 IACUC 獨立審查原則
        let submitter_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT pi_user_id FROM protocols WHERE id = $1",
        )
        .bind(req.protocol_id)
        .fetch_optional(pool)
        .await?;

        if let Some(pi_id) = submitter_id {
            if pi_id == req.reviewer_id {
                return Err(AppError::Validation(
                    "審查委員不得為計畫提交者（PI），違反 IACUC 獨立審查原則".to_string(),
                ));
            }
        }

        let assignment = sqlx::query_as::<_, ReviewAssignment>(
            r#"
            INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by, assigned_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (protocol_id, reviewer_id) DO UPDATE SET assigned_at = NOW()
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(req.protocol_id)
        .bind(req.reviewer_id)
        .bind(assigned_by)
        .fetch_one(pool)
        .await?;

        Self::record_activity(
            pool,
            req.protocol_id,
            ProtocolActivityType::ReviewerAssigned,
            assigned_by,
            None,
            None,
            Some(("user", req.reviewer_id, "Reviewer")),
            Some(format!("Assigned reviewer {}", req.reviewer_id)),
            None,
        ).await?;

        Ok(assignment)
    }

    /// Transaction 版本：在既有 transaction 內指派正式審查委員（R26-8 Phase 2 需用）
    pub(super) async fn assign_primary_reviewer_tx(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
        reviewer_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<ReviewAssignment> {
        let assignment = sqlx::query_as::<_, ReviewAssignment>(
            r#"
            INSERT INTO review_assignments (id, protocol_id, reviewer_id, assigned_by, assigned_at, is_primary_reviewer, review_stage)
            VALUES ($1, $2, $3, $4, NOW(), true, 'UNDER_REVIEW')
            ON CONFLICT (protocol_id, reviewer_id) DO UPDATE SET
                assigned_at = NOW(),
                is_primary_reviewer = true,
                review_stage = 'UNDER_REVIEW'
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_id)
        .bind(reviewer_id)
        .bind(assigned_by)
        .fetch_one(&mut **tx)
        .await?;

        Ok(assignment)
    }

    /// Transaction 版本：在既有 transaction 內指派獸醫審查員（R26-8 Phase 2 需用）
    pub(super) async fn assign_vet_reviewer_tx(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
        vet_id: Option<Uuid>,
        assigned_by: Uuid,
    ) -> Result<()> {
        // 如果未指定獸醫師，從系統設定取得預設獸醫師
        let vet_user_id = if let Some(id) = vet_id {
            id
        } else {
            // 從系統設定取得預設獸醫師
            let setting: Option<serde_json::Value> = sqlx::query_scalar(
                "SELECT value FROM system_settings WHERE key = 'default_vet_reviewer'"
            )
            .fetch_optional(&mut **tx)
            .await?;

            let default_vet_id = setting
                .and_then(|v| v.get("user_id")?.as_str().map(|s| s.to_string()))
                .and_then(|s| Uuid::parse_str(&s).ok());

            match default_vet_id {
                Some(id) => id,
                None => {
                    // 如果沒有設定預設獸醫師，查找第一個具有 VET 角色的使用者
                    let first_vet: Option<Uuid> = sqlx::query_scalar(
                        r#"
                        SELECT u.id FROM users u
                        INNER JOIN user_roles ur ON u.id = ur.user_id
                        INNER JOIN roles r ON ur.role_id = r.id
                        WHERE r.code = 'VET' AND u.is_active = true
                        LIMIT 1
                        "#
                    )
                    .fetch_optional(&mut **tx)
                    .await?;

                    first_vet.ok_or_else(|| {
                        AppError::BusinessRule("系統中沒有可用的獸醫師".to_string())
                    })?
                }
            }
        };

        // 驗證指定的使用者是獸醫師
        let is_vet: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1 AND r.code = 'VET'
            )
            "#
        )
        .bind(vet_user_id)
        .fetch_one(&mut **tx)
        .await?;

        if !is_vet.0 {
            return Err(AppError::Validation("指定的使用者不具有獸醫師角色".to_string()));
        }

        // 建立獸醫審查指派記錄
        sqlx::query(
            r#"
            INSERT INTO vet_review_assignments (id, protocol_id, vet_id, assigned_by, assigned_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (protocol_id) DO UPDATE SET
                vet_id = $3,
                assigned_by = $4,
                assigned_at = NOW(),
                completed_at = NULL,
                decision = NULL,
                decision_remark = NULL
            "#
        )
        .bind(Uuid::new_v4())
        .bind(protocol_id)
        .bind(vet_user_id)
        .bind(assigned_by)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 指派 co-editor（試驗工作人員）
    pub async fn assign_co_editor(
        pool: &PgPool,
        req: &AssignCoEditorRequest,
        assigned_by: Uuid,
    ) -> Result<UserProtocol> {
        // 驗證協議存在
        let protocol_exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM protocols WHERE id = $1)"
        )
        .bind(req.protocol_id)
        .fetch_one(pool)
        .await?;

        if !protocol_exists.0 {
            return Err(AppError::NotFound("Protocol not found".to_string()));
        }

        // SEC-BIZ-1: 驗證操作者是否為該計畫的 PI/CLIENT/CO_EDITOR 或具 view_all 權限
        // 防止任何有 aup.review.assign 權限的人把自己或他人加進不屬於自己的計畫
        let is_pi_or_member = crate::services::access::is_pi_or_coeditor(
            pool, req.protocol_id, assigned_by,
        ).await?;
        let has_view_all: (bool,) = sqlx::query_as(
            r#"SELECT EXISTS(
                SELECT 1 FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1
                  AND r.code IN ('SYSTEM_ADMIN', 'admin', 'IACUC_STAFF', 'IACUC_CHAIR')
            )"#,
        )
        .bind(assigned_by)
        .fetch_one(pool)
        .await?;

        if !is_pi_or_member && !has_view_all.0 {
            return Err(AppError::Forbidden(
                "只有計畫 PI、共同編輯者或 IACUC 人員可以指派 co-editor".to_string(),
            ));
        }

        // 驗證用戶存在且是 EXPERIMENT_STAFF 角色
        let user_has_role: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = $1 AND r.code = 'EXPERIMENT_STAFF'
            )
            "#
        )
        .bind(req.user_id)
        .fetch_one(pool)
        .await?;

        if !user_has_role.0 {
            return Err(AppError::Validation("User must have EXPERIMENT_STAFF role to be assigned as co-editor".to_string()));
        }

        // 指派為 co-editor
        let assignment = sqlx::query_as::<_, UserProtocol>(
            r#"
            INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
            VALUES ($1, $2, 'CO_EDITOR', NOW(), $3)
            ON CONFLICT (user_id, protocol_id) 
            DO UPDATE SET 
                role_in_protocol = 'CO_EDITOR',
                granted_at = NOW(),
                granted_by = $3
            RETURNING *
            "#
        )
        .bind(req.user_id)
        .bind(req.protocol_id)
        .bind(assigned_by)
        .fetch_one(pool)
        .await?;

        Self::record_activity(
            pool,
            req.protocol_id,
            ProtocolActivityType::CoeditorAssigned,
            assigned_by,
            None,
            None,
            Some(("user", req.user_id, "Co-editor")),
            Some(format!("Assigned co-editor {}", req.user_id)),
            None,
        ).await?;

        Ok(assignment)
    }

    /// 取得 co-editor 列表
    pub async fn list_co_editors(
        pool: &PgPool,
        protocol_id: Uuid,
    ) -> Result<Vec<CoEditorAssignmentResponse>> {
        let co_editors = sqlx::query_as::<_, CoEditorAssignmentResponse>(
            r#"
            SELECT 
                up.user_id,
                up.protocol_id,
                up.role_in_protocol,
                up.granted_at,
                up.granted_by,
                COALESCE(u.display_name, u.email) as user_name,
                u.email as user_email,
                granted_by_user.display_name as granted_by_name
            FROM user_protocols up
            INNER JOIN users u ON up.user_id = u.id
            LEFT JOIN users granted_by_user ON up.granted_by = granted_by_user.id
            WHERE up.protocol_id = $1 AND up.role_in_protocol = 'CO_EDITOR'
            ORDER BY up.granted_at DESC
            "#
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;

        Ok(co_editors)
    }

    /// 移除 co-editor
    pub async fn remove_co_editor(
        pool: &PgPool,
        protocol_id: Uuid,
        user_id: Uuid,
        removed_by: Uuid,
    ) -> Result<()> {
        // 驗證該用戶確實是此協議的 co-editor
        let is_co_editor: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_protocols
                WHERE protocol_id = $1 
                AND user_id = $2 
                AND role_in_protocol = 'CO_EDITOR'
            )
            "#
        )
        .bind(protocol_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        if !is_co_editor.0 {
            return Err(AppError::NotFound("Co-editor assignment not found".to_string()));
        }

        // 移除 co-editor
        sqlx::query(
            r#"
            DELETE FROM user_protocols
            WHERE protocol_id = $1 AND user_id = $2 AND role_in_protocol = 'CO_EDITOR'
            "#
        )
        .bind(protocol_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Self::record_activity(
            pool,
            protocol_id,
            ProtocolActivityType::CoeditorRemoved,
            removed_by,
            None,
            None,
            Some(("user", user_id, "Co-editor")),
            Some(format!("Removed co-editor {}", user_id)),
            None,
        ).await?;

        Ok(())
    }
}
