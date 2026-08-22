// 報廢紀錄（disposal records）：申請、簽章、核准、恢復

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, ApproveDisposalRequest, CreateDisposalRequest, DisposalQuery,
        DisposalStatus, DisposalWithDetails, Equipment, EquipmentDisposal, EquipmentStatus,
        PaginatedResponse,
    },
    repositories,
    services::{
        access,
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, ElectronicSignature, SignatureService, SignatureType,
    },
    Result,
};

use super::{
    assert_not_self_approval, check_manage_permission, check_view_permission,
    validate_status_transition, EquipmentService,
};

impl EquipmentService {
    // ========== Disposal Records (報廢) ==========

    pub async fn list_disposals(
        pool: &PgPool,
        query: &DisposalQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<DisposalWithDetails>> {
        check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_disposals d
            WHERE ($1::uuid IS NULL OR d.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR d.status = $2)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        let data = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            SELECT d.id, d.equipment_id, e.name AS equipment_name,
                   d.status, d.disposal_date, d.reason, d.disposal_method,
                   d.applied_by, u1.display_name AS applicant_name, d.applied_at,
                   d.approved_by, u2.display_name AS approver_name, d.approved_at,
                   d.rejection_reason, d.notes, d.created_at
            FROM equipment_disposals d
            INNER JOIN equipment e ON d.equipment_id = e.id
            INNER JOIN users u1 ON d.applied_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE ($1::uuid IS NULL OR d.equipment_id = $1)
              AND ($2::disposal_status IS NULL OR d.status = $2)
            ORDER BY d.applied_at DESC
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

    pub async fn create_disposal(
        pool: &PgPool,
        payload: &CreateDisposalRequest,
        current_user: &CurrentUser,
    ) -> Result<DisposalWithDetails> {
        check_manage_permission(current_user)?;
        payload.validate()?;

        // 驗證設備存在
        repositories::equipment::find_equipment_by_id(pool, payload.equipment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        let record = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            WITH ins AS (
                INSERT INTO equipment_disposals (equipment_id, disposal_date, reason, disposal_method, applied_by, notes)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *
            )
            SELECT ins.id, ins.equipment_id, e.name AS equipment_name,
                   ins.status, ins.disposal_date, ins.reason, ins.disposal_method,
                   ins.applied_by, u1.display_name AS applicant_name, ins.applied_at,
                   ins.approved_by, NULL::text AS approver_name, ins.approved_at,
                   ins.rejection_reason, ins.notes, ins.created_at
            FROM ins
            INNER JOIN equipment e ON ins.equipment_id = e.id
            INNER JOIN users u1 ON ins.applied_by = u1.id
            "#,
        )
        .bind(payload.equipment_id)
        .bind(payload.disposal_date)
        .bind(&payload.reason)
        .bind(&payload.disposal_method)
        .bind(current_user.id)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await?;

        // 發送報廢申請通知
        let notification_svc = crate::services::NotificationService::new(pool.clone());
        if let Err(e) = notification_svc
            .send_equipment_disposal_notification(
                &record.equipment_name,
                &record.applicant_name,
                &payload.reason,
            )
            .await
        {
            tracing::warn!("發送報廢申請通知失敗: {e}");
        }

        Ok(record)
    }

    /// 為報廢申請建立申請人簽章，與 record UPDATE 同 tx 原子（21 CFR §11.10(e)(1)）。
    ///
    /// 流程（同一 tx 內）：
    ///   1. RBAC：`equipment.manage`（同 create_disposal）
    ///   2. SELECT FOR UPDATE 鎖 row + 狀態守衛（pending / 未簽過）+ 自簽檢查
    ///      （applied_by == current_user.id；申請人不能由他人代簽）
    ///   3. `SignatureService::sign_record_tx` 寫 electronic_signatures
    ///   4. UPDATE applicant_signature_id
    ///   5. audit log（log_activity_tx）
    pub async fn sign_disposal_applicant_tx(
        pool: &PgPool,
        actor: &ActorContext,
        disposal_id: Uuid,
        sig_type: SignatureType,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<ElectronicSignature> {
        let current_user = actor.require_user()?;
        access::require_equipment_manage(current_user)?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentDisposal>(
            "SELECT * FROM equipment_disposals WHERE id = $1 FOR UPDATE",
        )
        .bind(disposal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此報廢申請非待處理狀態".into()));
        }
        if existing.applicant_signature_id.is_some() {
            return Err(AppError::Conflict(
                "此報廢申請的申請人已簽章，不得覆寫".into(),
            ));
        }
        if existing.applied_by != current_user.id {
            return Err(AppError::Forbidden(
                "只有申請人本人可以簽章（不得代簽）".into(),
            ));
        }

        let content = format!("disposal_applicant:{}", disposal_id);
        let signature = SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            "disposal_applicant",
            &disposal_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        let updated = sqlx::query_as::<_, EquipmentDisposal>(
            "UPDATE equipment_disposals \
             SET applicant_signature_id = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING *",
        )
        .bind(signature.id)
        .bind(disposal_id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "DISPOSAL_APPLICANT_SIGNATURE",
                entity: Some(AuditEntity::new(
                    "equipment_disposal",
                    disposal_id,
                    "disposal_applicant_signature",
                )),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(signature)
    }

    /// 為報廢申請建立核准人簽章，與 record UPDATE 同 tx 原子（21 CFR §11.10(e)(1)）。
    ///
    /// 流程（同一 tx 內）：
    ///   1. RBAC：`equipment.disposal.approve` 或 `equipment.manage`
    ///   2. SELECT FOR UPDATE 鎖 row + 狀態守衛（pending / 未簽過）+ 申請人不能自核
    ///      （applied_by != current_user.id；防止 self-approve 提權）
    ///   3. `SignatureService::sign_record_tx` 寫 electronic_signatures
    ///   4. UPDATE approver_signature_id
    ///   5. audit log（log_activity_tx）
    pub async fn sign_disposal_approver_tx(
        pool: &PgPool,
        actor: &ActorContext,
        disposal_id: Uuid,
        sig_type: SignatureType,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<ElectronicSignature> {
        let current_user = actor.require_user()?;
        access::require_equipment_disposal_approve(current_user)?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentDisposal>(
            "SELECT * FROM equipment_disposals WHERE id = $1 FOR UPDATE",
        )
        .bind(disposal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此報廢申請非待處理狀態".into()));
        }
        if existing.approver_signature_id.is_some() {
            return Err(AppError::Conflict(
                "此報廢申請的核准人已簽章，不得覆寫".into(),
            ));
        }
        if existing.applied_by == current_user.id {
            return Err(AppError::Forbidden("申請人不得自核自簽（職權分離）".into()));
        }

        let content = format!("disposal_approver:{}", disposal_id);
        let signature = SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            "disposal_approver",
            &disposal_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        let updated = sqlx::query_as::<_, EquipmentDisposal>(
            "UPDATE equipment_disposals \
             SET approver_signature_id = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING *",
        )
        .bind(signature.id)
        .bind(disposal_id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "DISPOSAL_APPROVER_SIGNATURE",
                entity: Some(AuditEntity::new(
                    "equipment_disposal",
                    disposal_id,
                    "disposal_approver_signature",
                )),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(signature)
    }

    pub async fn approve_disposal(
        pool: &PgPool,
        id: Uuid,
        payload: &ApproveDisposalRequest,
        current_user: &CurrentUser,
    ) -> Result<DisposalWithDetails> {
        if !current_user.has_permission("equipment.disposal.approve") {
            return Err(AppError::Forbidden("無權核准報廢申請".into()));
        }
        payload.validate()?;

        let existing = repositories::equipment::find_disposal_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Pending {
            return Err(AppError::BadRequest("此報廢申請已處理".into()));
        }

        // SEC-SoD: 申請人不得自核自簽（與簽章路徑 sign_disposal_approver_tx 一致；
        // 原 legacy approve 端點缺此守衛 → 同時持 manage + approve 者可自核報廢設備）
        assert_not_self_approval(
            existing.applied_by,
            current_user.id,
            "申請人不得核准自己的報廢申請（職權分離）",
        )?;

        let new_status = if payload.approved {
            DisposalStatus::Approved
        } else {
            DisposalStatus::Rejected
        };

        // #167：包進 transaction + status 守衛 + 同 tx 稽核（原為多筆裸 pool 寫入、無 audit、無原子性）
        let actor = ActorContext::User(current_user.clone());
        let mut tx = pool.begin().await?;

        // 核准要報廢時，先在 tx 內 FOR UPDATE 鎖定設備列再驗證狀態轉換，避免 TOCTOU
        // （原本在 tx 外用 pool 讀取驗證，驗證與寫入之間設備狀態可能被其他請求改動）
        if payload.approved {
            let equipment =
                sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1 FOR UPDATE")
                    .bind(existing.equipment_id)
                    .fetch_optional(&mut *tx)
                    .await?
                    .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;
            validate_status_transition(&equipment.status, &EquipmentStatus::Decommissioned)?;
        }

        let rows = sqlx::query(
            r#"
            UPDATE equipment_disposals
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
        .await?
        .rows_affected();
        if rows == 0 {
            return Err(AppError::Conflict(
                "此報廢申請已被處理，請重新載入後再試。".into(),
            ));
        }

        // 核准後自動將設備狀態變為「報廢」
        if payload.approved {
            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, (SELECT status FROM equipment WHERE id = $1), 'decommissioned', $2, '報廢申請核准')",
            )
            .bind(existing.equipment_id)
            .bind(current_user.id)
            .execute(&mut *tx)
            .await?;

            sqlx::query("UPDATE equipment SET status = 'decommissioned', is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(existing.equipment_id)
                .execute(&mut *tx)
                .await?;
        }

        let display = format!(
            "設備報廢{}（disposal {}）",
            if payload.approved { "核准" } else { "駁回" },
            id
        );
        AuditService::log_activity_tx(
            &mut tx,
            &actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: if payload.approved {
                    "DISPOSAL_APPROVE"
                } else {
                    "DISPOSAL_REJECT"
                },
                entity: Some(AuditEntity::new("equipment_disposals", id, &display)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;

        // 重新查詢完整紀錄
        let record = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            SELECT d.id, d.equipment_id, e.name AS equipment_name,
                   d.status, d.disposal_date, d.reason, d.disposal_method,
                   d.applied_by, u1.display_name AS applicant_name, d.applied_at,
                   d.approved_by, u2.display_name AS approver_name, d.approved_at,
                   d.rejection_reason, d.notes, d.created_at
            FROM equipment_disposals d
            INNER JOIN equipment e ON d.equipment_id = e.id
            INNER JOIN users u1 ON d.applied_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE d.id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        // 審核結果通知申請人（核准 / 駁回都通知；含駁回原因）。
        // 申請人 applied_by ≠ 核准人（自核已由 assert_not_self_approval 擋下）。
        let notification_svc = crate::services::NotificationService::new(pool.clone());
        if let Err(e) = notification_svc
            .notify_equipment_disposal_result(
                record.id,
                existing.applied_by,
                record.equipment_id,
                payload.approved,
                payload.rejection_reason.as_deref(),
            )
            .await
        {
            tracing::warn!("發送報廢審核結果通知失敗: {e}");
        }

        Ok(record)
    }

    /// 管理員恢復已報廢設備（將 status 改回 active、is_active 改回 true）
    pub async fn restore_equipment(
        pool: &PgPool,
        disposal_id: Uuid,
        current_user: &CurrentUser,
    ) -> Result<DisposalWithDetails> {
        if !current_user.has_permission("equipment.disposal.approve") {
            return Err(AppError::Forbidden("無權恢復報廢設備".into()));
        }

        let existing = repositories::equipment::find_disposal_by_id(pool, disposal_id)
            .await?
            .ok_or_else(|| AppError::NotFound("報廢紀錄不存在".into()))?;

        if existing.status != DisposalStatus::Approved {
            return Err(AppError::BadRequest("只能恢復已核准的報廢設備".into()));
        }

        // P2-1: 二次審批 — 恢復人不得與原核准人相同
        if let Some(approver) = existing.approved_by {
            if approver == current_user.id {
                return Err(AppError::BadRequest(
                    "報廢恢復需由原核准人以外的管理員執行（二次審批）".into(),
                ));
            }
        }

        // #167：包進 transaction + status 守衛 + 同 tx 稽核
        let actor = ActorContext::User(current_user.clone());
        let mut tx = pool.begin().await?;

        // 在 tx 內 FOR UPDATE 鎖定設備列，動態驗證其「目前實際狀態」可轉回 Active，
        // 避免原本靜態 validate(Decommissioned→Active) 與 hardcoded old_status 與真實狀態脫節（TOCTOU + 稽核失真）
        let equipment =
            sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1 FOR UPDATE")
                .bind(existing.equipment_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;
        validate_status_transition(&equipment.status, &EquipmentStatus::Active)?;

        // 將報廢紀錄狀態改為 rejected（表示已撤銷）— status 守衛防並發重入
        let rows = sqlx::query(
            "UPDATE equipment_disposals SET status = 'rejected', rejection_reason = '管理員恢復設備', approved_by = $2, approved_at = NOW(), updated_at = NOW() WHERE id = $1 AND status = 'approved'",
        )
        .bind(disposal_id)
        .bind(current_user.id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if rows == 0 {
            return Err(AppError::Conflict(
                "此報廢紀錄已被處理，請重新載入後再試。".into(),
            ));
        }

        // 記錄狀態變更日誌（old_status 用鎖定讀到的實際狀態，非 hardcode）
        sqlx::query(
            "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, 'active', $3, '管理員恢復報廢設備（二次審批）')",
        )
        .bind(existing.equipment_id)
        .bind(&equipment.status)
        .bind(current_user.id)
        .execute(&mut *tx)
        .await?;

        // 恢復設備狀態
        sqlx::query(
            "UPDATE equipment SET status = 'active', is_active = true, updated_at = NOW() WHERE id = $1",
        )
        .bind(existing.equipment_id)
        .execute(&mut *tx)
        .await?;

        let display = format!("設備報廢恢復（disposal {}）", disposal_id);
        AuditService::log_activity_tx(
            &mut tx,
            &actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "DISPOSAL_RESTORE",
                entity: Some(AuditEntity::new(
                    "equipment_disposals",
                    disposal_id,
                    &display,
                )),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;
        tx.commit().await?;

        let record = sqlx::query_as::<_, DisposalWithDetails>(
            r#"
            SELECT d.id, d.equipment_id, e.name AS equipment_name,
                   d.status, d.disposal_date, d.reason, d.disposal_method,
                   d.applied_by, u1.display_name AS applicant_name, d.applied_at,
                   d.approved_by, u2.display_name AS approver_name, d.approved_at,
                   d.rejection_reason, d.notes, d.created_at
            FROM equipment_disposals d
            INNER JOIN equipment e ON d.equipment_id = e.id
            INNER JOIN users u1 ON d.applied_by = u1.id
            LEFT JOIN users u2 ON d.approved_by = u2.id
            WHERE d.id = $1
            "#,
        )
        .bind(disposal_id)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}
