// 閒置審批（idle requests）

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, ApproveIdleRequestRequest, CreateIdleRequestRequest, DisposalStatus,
        Equipment, EquipmentStatus, IdleRequestQuery, IdleRequestWithDetails, PaginatedResponse,
    },
    repositories,
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    Result,
};

use super::{
    assert_not_self_approval, check_manage_permission, check_view_permission,
    validate_status_transition, EquipmentService,
};

impl EquipmentService {
    // ========== Idle Requests (閒置審批) ==========

    pub async fn list_idle_requests(
        pool: &PgPool,
        query: &IdleRequestQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<IdleRequestWithDetails>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_idle_requests ir
            WHERE ($1::uuid IS NULL OR ir.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR ir.status = $2)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            SELECT ir.id, ir.equipment_id, e.name AS equipment_name,
                   ir.request_type, ir.reason, ir.status,
                   ir.applied_by, u1.display_name AS applicant_name, ir.applied_at,
                   ir.approved_by, u2.display_name AS approver_name, ir.approved_at,
                   ir.rejection_reason, ir.notes, ir.created_at
            FROM equipment_idle_requests ir
            INNER JOIN equipment e ON ir.equipment_id = e.id
            INNER JOIN users u1 ON ir.applied_by = u1.id
            LEFT JOIN users u2 ON ir.approved_by = u2.id
            WHERE ($1::uuid IS NULL OR ir.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR ir.status = $2)
            ORDER BY ir.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    pub async fn create_idle_request(
        pool: &PgPool,
        payload: &CreateIdleRequestRequest,
        current_user: &CurrentUser,
    ) -> Result<IdleRequestWithDetails> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        if payload.request_type != "idle" && payload.request_type != "restore" {
            return Err(AppError::BadRequest(
                "request_type 必須為 'idle' 或 'restore'".into(),
            ));
        }

        let equipment = repositories::equipment::find_equipment_by_id(pool, payload.equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        // 驗證狀態轉換合法性
        let target_status = if payload.request_type == "idle" {
            if equipment.status != EquipmentStatus::Active {
                return Err(AppError::BadRequest("只有啟用中的設備可以申請閒置".into()));
            }
            EquipmentStatus::Inactive
        } else {
            if equipment.status != EquipmentStatus::Inactive {
                return Err(AppError::BadRequest("只有閒置中的設備可以申請恢復".into()));
            }
            EquipmentStatus::Active
        };
        validate_status_transition(&equipment.status, &target_status)?;

        // 檢查是否已有待審批的申請
        let has_pending: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM equipment_idle_requests WHERE equipment_id = $1 AND status = 'pending')",
        )
        .bind(payload.equipment_id)
        .fetch_one(pool)
        .await?;

        if has_pending {
            return Err(AppError::BadRequest(
                "該設備已有待審批的閒置/恢復申請".into(),
            ));
        }

        let record = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            WITH inserted AS (
                INSERT INTO equipment_idle_requests
                    (equipment_id, request_type, reason, applied_by, notes)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
            )
            SELECT i.id, i.equipment_id, e.name AS equipment_name,
                   i.request_type, i.reason, i.status,
                   i.applied_by, u1.display_name AS applicant_name, i.applied_at,
                   i.approved_by, NULL::text AS approver_name, i.approved_at,
                   i.rejection_reason, i.notes, i.created_at
            FROM inserted i
            INNER JOIN equipment e ON i.equipment_id = e.id
            INNER JOIN users u1 ON i.applied_by = u1.id
            "#,
        )
        .bind(payload.equipment_id)
        .bind(&payload.request_type)
        .bind(&payload.reason)
        .bind(current_user.id)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn approve_idle_request(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &ApproveIdleRequestRequest,
    ) -> Result<IdleRequestWithDetails> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.idle.approve") {
            return Err(AppError::Forbidden("無權核准閒置申請".into()));
        }
        payload.validate()?;

        let mut tx = pool.begin().await?;

        // R71-3：鎖 idle_request 列（FOR UPDATE OF ir）並取 before 快照，
        // 防兩個並發核准都通過 pending 檢查後重複套用。
        let before = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            SELECT ir.id, ir.equipment_id, e.name AS equipment_name,
                   ir.request_type, ir.reason, ir.status,
                   ir.applied_by, u1.display_name AS applicant_name, ir.applied_at,
                   ir.approved_by, u2.display_name AS approver_name, ir.approved_at,
                   ir.rejection_reason, ir.notes, ir.created_at
            FROM equipment_idle_requests ir
            INNER JOIN equipment e ON ir.equipment_id = e.id
            INNER JOIN users u1 ON ir.applied_by = u1.id
            LEFT JOIN users u2 ON ir.approved_by = u2.id
            WHERE ir.id = $1
            FOR UPDATE OF ir
            "#,
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("閒置申請不存在".into()))?;

        if before.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此申請已處理".into()));
        }

        // SEC-SoD (資安稽核 M-1)：申請人不得核准自己的閒置申請（與 approve_disposal 一致）。
        assert_not_self_approval(
            before.applied_by,
            current_user.id,
            "申請人不得核准自己的閒置申請（職權分離）",
        )?;

        let new_status = if payload.approved {
            DisposalStatus::Approved
        } else {
            DisposalStatus::Rejected
        };

        sqlx::query(
            r#"
            UPDATE equipment_idle_requests
            SET status = $2, approved_by = $3, approved_at = NOW(),
                rejection_reason = $4, updated_at = NOW()
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(current_user.id)
        .bind(&payload.rejection_reason)
        .execute(&mut *tx)
        .await?;

        if payload.approved {
            // 鎖設備列（同一 tx），避免與其他設備狀態變更競態
            let equipment =
                sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1 FOR UPDATE")
                    .bind(before.equipment_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

            let (target_status, is_active, reason) = if before.request_type == "idle" {
                (EquipmentStatus::Inactive, false, "閒置申請核准")
            } else {
                (EquipmentStatus::Active, true, "閒置恢復申請核准")
            };

            validate_status_transition(&equipment.status, &target_status)?;

            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(before.equipment_id)
            .bind(&equipment.status)
            .bind(&target_status)
            .bind(current_user.id)
            .bind(reason)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "UPDATE equipment SET status = $2, is_active = $3, updated_at = NOW() WHERE id = $1",
            )
            .bind(before.equipment_id)
            .bind(&target_status)
            .bind(is_active)
            .execute(&mut *tx)
            .await?;
        }

        // after 快照（同一 tx 可見變更）
        let after = sqlx::query_as::<_, IdleRequestWithDetails>(
            r#"
            SELECT ir.id, ir.equipment_id, e.name AS equipment_name,
                   ir.request_type, ir.reason, ir.status,
                   ir.applied_by, u1.display_name AS applicant_name, ir.applied_at,
                   ir.approved_by, u2.display_name AS approver_name, ir.approved_at,
                   ir.rejection_reason, ir.notes, ir.created_at
            FROM equipment_idle_requests ir
            INNER JOIN equipment e ON ir.equipment_id = e.id
            INNER JOIN users u1 ON ir.applied_by = u1.id
            LEFT JOIN users u2 ON ir.approved_by = u2.id
            WHERE ir.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        // R71-3：補 in-tx 稽核（與資料變更原子；事件進 user_activity_logs / HMAC chain）
        let display = format!(
            "idle_request {} {:?} → {:?}",
            after.request_type, before.status, new_status
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: if payload.approved {
                    "IDLE_REQUEST_APPROVE"
                } else {
                    "IDLE_REQUEST_REJECT"
                },
                entity: Some(AuditEntity::new(
                    "equipment_idle_request",
                    after.id,
                    &display,
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // P2-3: 通知申請人審批結果（post-commit 側效應，失敗僅警告不影響已 commit 的核准）
        {
            let notification_svc = crate::services::NotificationService::new(pool.clone());
            let action = if payload.approved { "核准" } else { "駁回" };
            let type_label = if after.request_type == "idle" {
                "閒置"
            } else {
                "恢復"
            };
            if let Err(e) = notification_svc
                .create_notification(crate::models::CreateNotificationRequest {
                    user_id: after.applied_by,
                    notification_type: crate::models::NotificationType::SystemAlert,
                    title: format!("設備{}申請已{}", type_label, action),
                    content: Some(format!(
                        "您的設備「{}」{}申請已被{}。",
                        after.equipment_name, type_label, action
                    )),
                    related_entity_type: Some("equipment".to_string()),
                    related_entity_id: None,
                })
                .await
            {
                tracing::warn!("發送閒置審批結果通知失敗: {e}");
            }
        }

        Ok(after)
    }
}
