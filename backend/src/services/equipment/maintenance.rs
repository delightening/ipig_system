// 維修 / 保養紀錄（maintenance records）：含 transaction 變體、驗收、簽章

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::DataDiff, CreateMaintenanceRequest, Equipment, EquipmentMaintenanceRecord,
        EquipmentStatus, MaintenanceQuery, MaintenanceRecordWithDetails, MaintenanceStatus,
        MaintenanceType, PaginatedResponse, ReviewMaintenanceRequest, UpdateMaintenanceRequest,
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
    assert_not_self_approval, validate_status_transition, EquipmentService,
    MAINTENANCE_RESIGN_SUPERSEDE_REASON,
};

/// 維修/保養紀錄可排序欄位白名單（query key → SQL 欄位）。
const MAINTENANCE_SORT_COLUMNS: &[(&str, &str)] = &[
    ("reported_at", "m.reported_at"),
    ("completed_at", "m.completed_at"),
    ("equipment_name", "e.name"),
    ("status", "m.status"),
    ("maintenance_type", "m.maintenance_type"),
];

/// 解析 sort param → 安全的 ORDER BY clause（不含 `ORDER BY` 前綴）。
/// 欄位來自白名單常數，方向僅 ASC/DESC，故 format! 插值無注入風險。
/// `sort_by` 不在白名單 / 為 None → 用預設（報修日期新→舊）。
fn resolve_maintenance_order_by(query: &MaintenanceQuery) -> String {
    let column = query.sort_by.as_deref().map(str::trim).and_then(|key| {
        MAINTENANCE_SORT_COLUMNS
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, sql)| *sql)
    });
    let direction = match query.sort_order.as_deref() {
        Some(s) if s.eq_ignore_ascii_case("asc") => "ASC",
        _ => "DESC",
    };
    match column {
        Some(col) => format!("{col} {direction} NULLS LAST, m.created_at DESC"),
        None => "m.reported_at DESC, m.created_at DESC".to_string(),
    }
}

impl EquipmentService {
    // ========== Maintenance Records (維修/保養) ==========

    pub async fn list_maintenance_records(
        pool: &PgPool,
        query: &MaintenanceQuery,
        current_user: &CurrentUser,
    ) -> Result<PaginatedResponse<MaintenanceRecordWithDetails>> {
        super::check_view_permission(current_user)?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(50).min(100);
        let offset = (page - 1) * per_page;

        let total: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM equipment_maintenance_records m
            WHERE ($1::uuid IS NULL OR m.equipment_id = $1)
              AND ($2::maintenance_type IS NULL OR m.maintenance_type = $2)
              AND ($3::maintenance_status IS NULL OR m.status = $3)
            "#,
        )
        .bind(query.equipment_id)
        .bind(&query.maintenance_type)
        .bind(&query.status)
        .fetch_one(pool)
        .await?;

        // order_by 來自白名單常數 + ASC/DESC，已稽核無注入風險，故 AssertSqlSafe。
        let order_by = resolve_maintenance_order_by(query);
        let data = sqlx::query_as::<_, MaintenanceRecordWithDetails>(sqlx::AssertSqlSafe(format!(
            r#"
            SELECT m.id, m.equipment_id, e.name AS equipment_name,
                   m.maintenance_type, m.status, m.reported_at, m.completed_at,
                   m.problem_description, m.repair_content, m.repair_partner_id,
                   p.name AS repair_partner_name,
                   m.maintenance_items, m.performed_by, m.notes,
                   m.created_by,
                   m.reviewed_by, u2.display_name AS reviewer_name,
                   m.reviewed_at, m.review_notes,
                   m.created_at
            FROM equipment_maintenance_records m
            INNER JOIN equipment e ON m.equipment_id = e.id
            LEFT JOIN partners p ON m.repair_partner_id = p.id
            LEFT JOIN users u2 ON m.reviewed_by = u2.id
            WHERE ($1::uuid IS NULL OR m.equipment_id = $1)
              AND ($2::maintenance_type IS NULL OR m.maintenance_type = $2)
              AND ($3::maintenance_status IS NULL OR m.status = $3)
            ORDER BY {order_by}
            LIMIT $4 OFFSET $5
            "#
        )))
        .bind(query.equipment_id)
        .bind(&query.maintenance_type)
        .bind(&query.status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse::new(data, total.0, page, per_page))
    }

    // ============================================
    // Transaction variants for cross-service atomicity (R26-3 Phase 2)
    // ============================================

    /// Transaction 版本：建立維修紀錄
    pub(in crate::services) async fn create_maintenance_record_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        payload: &CreateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let actor_id = actor.actor_user_id().ok_or_else(|| {
            AppError::Forbidden("Anonymous cannot create maintenance records".into())
        })?;
        payload.validate()?;

        let equipment =
            sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1 FOR UPDATE")
                .bind(payload.equipment_id)
                .fetch_optional(&mut **tx)
                .await?
                .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;

        if payload.maintenance_type == MaintenanceType::Repair
            && equipment.status == EquipmentStatus::Active
        {
            validate_status_transition(&equipment.status, &EquipmentStatus::UnderRepair)?;
            sqlx::query(
                "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, 'under_repair', $3, '建立維修紀錄，自動變更狀態')",
            )
            .bind(payload.equipment_id)
            .bind(&equipment.status)
            .bind(actor_id)
            .execute(&mut **tx)
            .await?;

            sqlx::query("UPDATE equipment SET status = 'under_repair', is_active = false, updated_at = NOW() WHERE id = $1")
                .bind(payload.equipment_id)
                .execute(&mut **tx)
                .await?;
        }

        let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            r#"
            INSERT INTO equipment_maintenance_records
                (equipment_id, maintenance_type, reported_at, completed_at,
                 problem_description, repair_content, repair_partner_id,
                 maintenance_items, performed_by, notes, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(payload.equipment_id)
        .bind(&payload.maintenance_type)
        .bind(payload.reported_at)
        .bind(payload.completed_at)
        .bind(&payload.problem_description)
        .bind(&payload.repair_content)
        .bind(payload.repair_partner_id)
        .bind(&payload.maintenance_items)
        .bind(&payload.performed_by)
        .bind(&payload.notes)
        .bind(actor_id)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("{} {:?}", equipment.name, record.maintenance_type);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_CREATE",
                entity: Some(AuditEntity::new("maintenance_record", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        Ok(record)
    }

    /// Transaction 版本：更新維修紀錄
    pub(in crate::services) async fn update_maintenance_record_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        payload.validate()?;

        let existing = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        // High-2 (#241/#249)：已簽收紀錄不可竄改（21 CFR §11.10(e)(1)）。
        // handler 對前端回報 is_locked=is_signed，後端須一致拒絕修改/刪除。
        if existing.reviewer_signature_id.is_some() {
            return Err(AppError::Conflict("已簽收的維修保養紀錄不可修改".into()));
        }

        let mut new_status = payload.status.clone().unwrap_or(existing.status.clone());
        if new_status == MaintenanceStatus::Completed
            && existing.status != MaintenanceStatus::Completed
            && existing.status != MaintenanceStatus::PendingReview
        {
            new_status = MaintenanceStatus::PendingReview;
        }

        let completed_at = payload.completed_at.or(existing.completed_at);
        let problem_desc = payload
            .problem_description
            .as_ref()
            .or(existing.problem_description.as_ref())
            .cloned();
        let repair_content = payload
            .repair_content
            .as_ref()
            .or(existing.repair_content.as_ref())
            .cloned();
        let repair_partner = payload.repair_partner_id.or(existing.repair_partner_id);
        let maint_items = payload
            .maintenance_items
            .as_ref()
            .or(existing.maintenance_items.as_ref())
            .cloned();
        let performed = payload
            .performed_by
            .as_ref()
            .or(existing.performed_by.as_ref())
            .cloned();
        let notes = payload.notes.as_ref().or(existing.notes.as_ref()).cloned();

        let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            r#"
            UPDATE equipment_maintenance_records
            SET status = $2, completed_at = $3, problem_description = $4,
                repair_content = $5, repair_partner_id = $6,
                maintenance_items = $7, performed_by = $8, notes = $9,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(completed_at)
        .bind(problem_desc)
        .bind(repair_content)
        .bind(repair_partner)
        .bind(maint_items)
        .bind(performed)
        .bind(notes)
        .fetch_one(&mut **tx)
        .await?;

        let display = format!("maintenance {:?}", record.maintenance_type);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_UPDATE",
                entity: Some(AuditEntity::new("maintenance_record", record.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&record))),
                request_context: None,
            },
        )
        .await?;

        Ok(record)
    }

    /// Transaction 版本：刪除維修紀錄
    pub(in crate::services) async fn delete_maintenance_record_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        let before = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        // High-2 (#241/#249)：已簽收紀錄不可刪除（21 CFR §11.10(e)(1)）。
        if before.reviewer_signature_id.is_some() {
            return Err(AppError::Conflict("已簽收的維修保養紀錄不可刪除".into()));
        }

        sqlx::query("DELETE FROM equipment_maintenance_records WHERE id = $1")
            .bind(id)
            .execute(&mut **tx)
            .await?;

        let display = format!("maintenance {:?}", before.maintenance_type);
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_DELETE",
                entity: Some(AuditEntity::new("maintenance_record", before.id, &display)),
                data_diff: Some(DataDiff::delete_only(&before)),
                request_context: None,
            },
        )
        .await?;

        Ok(())
    }

    pub async fn create_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        payload: &CreateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理維修保養紀錄".into()));
        }

        let mut tx = pool.begin().await?;
        let record = Self::create_maintenance_record_tx(&mut tx, actor, payload).await?;
        tx.commit().await?;

        // P2-2: 報修自動通知維修人員（tx 外 fire-and-forget，避免拖延 commit）
        if payload.maintenance_type == MaintenanceType::Repair {
            if let Ok(Some(equipment)) =
                repositories::equipment::find_equipment_by_id(pool, payload.equipment_id).await
            {
                let notification_svc = crate::services::NotificationService::new(pool.clone());
                if let Err(e) = notification_svc
                    .send_equipment_repair_notification(
                        &equipment.name,
                        &current_user.email,
                        payload.problem_description.as_deref().unwrap_or("-"),
                    )
                    .await
                {
                    tracing::warn!("發送報修通知失敗: {e}");
                }
            }
        }

        Ok(record)
    }

    pub async fn update_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &UpdateMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權管理維修保養紀錄".into()));
        }

        let mut tx = pool.begin().await?;
        let record = Self::update_maintenance_record_tx(&mut tx, actor, id, payload).await?;
        let existing_status = record.status.clone();
        tx.commit().await?;

        // 無法維修通知（tx 外 side effect）
        if existing_status == MaintenanceStatus::Unrepairable {
            if let Ok(Some(equip)) =
                repositories::equipment::find_equipment_by_id(pool, record.equipment_id).await
            {
                let notification_svc = crate::services::NotificationService::new(pool.clone());
                let problem = payload.problem_description.as_deref().unwrap_or("-");
                if let Err(e) = notification_svc
                    .send_equipment_unrepairable_notification(
                        &equip.name,
                        equip.serial_number.as_deref().unwrap_or("-"),
                        problem,
                    )
                    .await
                {
                    tracing::warn!("發送無法維修通知失敗: {e}");
                }
            }
        }

        Ok(record)
    }

    pub async fn delete_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<()> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.manage")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權刪除維修保養紀錄".into()));
        }

        let mut tx = pool.begin().await?;
        Self::delete_maintenance_record_tx(&mut tx, actor, id).await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn review_maintenance_record(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        payload: &ReviewMaintenanceRequest,
    ) -> Result<EquipmentMaintenanceRecord> {
        let current_user = actor.require_user()?;
        if !current_user.has_permission("equipment.maintenance.review")
            && !current_user.has_permission("equipment.manage")
        {
            return Err(AppError::Forbidden("無權驗收維修保養紀錄".into()));
        }
        payload.validate()?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        if existing.status != MaintenanceStatus::PendingReview {
            return Err(AppError::BadRequest("此紀錄非待驗收狀態".into()));
        }

        // SEC-SoD (資安稽核 L-2)：登錄者不得驗收自己的維護保養紀錄（與 approve_disposal 一致）。
        assert_not_self_approval(
            existing.created_by,
            current_user.id,
            "登錄者不得驗收自己的維護保養紀錄（職權分離）",
        )?;

        let new_status = if payload.approved {
            MaintenanceStatus::Completed
        } else {
            MaintenanceStatus::Pending
        };

        // 驗收通過 → 設備自動恢復啟用（tx 內執行 + 寫 EQUIPMENT_AUTO_RESTORE audit；
        // Gemini #166 HIGH 修正 tx 原子性 + Gemini #167 MEDIUM 補 audit）
        if payload.approved {
            auto_restore_equipment(&mut tx, actor, existing.equipment_id, current_user.id).await?;
        }

        let record = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            r#"
            UPDATE equipment_maintenance_records
            SET status = $2, reviewed_by = $3, reviewed_at = NOW(),
                review_notes = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&new_status)
        .bind(current_user.id)
        .bind(&payload.review_notes)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "maintenance {:?} → {:?}",
            record.maintenance_type, new_status
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: if payload.approved {
                    "MAINTENANCE_REVIEW_APPROVE"
                } else {
                    "MAINTENANCE_REVIEW_REJECT"
                },
                entity: Some(AuditEntity::new("maintenance_record", record.id, &display)),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&record))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 驗收結果通知登錄人（通過 / 退回都通知；含驗收意見）。
        // 登錄人 created_by ≠ 驗收人（自驗已由 assert_not_self_approval 擋下）。
        let notification_svc = crate::services::NotificationService::new(pool.clone());
        if let Err(e) = notification_svc
            .notify_equipment_maintenance_result(
                record.id,
                record.created_by,
                record.equipment_id,
                payload.approved,
                payload.review_notes.as_deref(),
            )
            .await
        {
            tracing::warn!("發送維修驗收結果通知失敗: {e}");
        }

        Ok(record)
    }

    /// 為維修保養紀錄建立驗收簽章，與 record UPDATE 同 tx 原子。
    ///
    /// 流程（同一 tx 內）：
    ///   1. RBAC：`equipment.maintenance.review` / `equipment.manage`
    ///   2. SELECT FOR UPDATE 鎖 row + 狀態守衛（pending_review / 未簽過）
    ///   3. `SignatureService::sign_record_tx` 寫 electronic_signatures
    ///   4. UPDATE equipment_maintenance_records.reviewer_signature_id
    ///   5. audit log（service 層 log_activity_tx，21 CFR §11.10 audit trail）
    ///
    /// 任何步驟失敗 → 整個 tx rollback，不留簽章孤兒。
    #[allow(clippy::too_many_arguments)]
    pub async fn sign_maintenance_review_tx(
        pool: &PgPool,
        actor: &ActorContext,
        record_id: Uuid,
        sig_type: SignatureType,
        password: Option<&str>,
        handwriting_svg: Option<&str>,
        stroke_data: Option<&serde_json::Value>,
    ) -> Result<ElectronicSignature> {
        let current_user = actor.require_user()?;
        access::require_equipment_review(current_user)?;

        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "SELECT * FROM equipment_maintenance_records WHERE id = $1 FOR UPDATE",
        )
        .bind(record_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("維修保養紀錄不存在".into()))?;

        if existing.status != MaintenanceStatus::PendingReview {
            return Err(AppError::BadRequest("此紀錄非待驗收狀態，無法簽章".into()));
        }

        // SEC-SoD：職權分離守衛提前到「簽章」步驟（與 review_maintenance_record 一致）。
        //
        // 驗收在前端是「先簽章、後 review」兩支獨立請求。SoD 原本只擋在 review：登錄者
        // 自簽時第一步（簽章）會先成功、把 reviewer_signature_id 寫入（= 上鎖），第二步
        // review 才被 SoD 擋下 → 紀錄卡在「已簽章 + 仍待驗收」，先前每次重試都倒在原本的
        // 「已簽章，不得覆寫」硬擋（本次已移除，改為下段的可重簽），真正被 SoD 擋下的原因
        // 反被孤兒簽章蓋掉。提前檢查後，自簽在寫入任何簽章前就被擋，不再產生孤兒、錯誤
        // 訊息也直指職權分離。
        assert_not_self_approval(
            existing.created_by,
            current_user.id,
            "登錄者不得驗收自己的維護保養紀錄（職權分離）",
        )?;

        // 待驗收（pending_review）紀錄若殘留 reviewer_signature_id，必為前一次「簽章成功
        // 但 review 未完成」的孤兒——已完成驗收的 status 會是 completed，由上方狀態守衛
        // 擋掉重簽。因此此處不硬擋覆寫，改為允許重新簽章以完成卡住的驗收：sign_record_tx
        // 會重新驗證密碼、寫入新的 electronic_signatures row（舊 row 保留以維持 HMAC 稽核
        // 鏈不斷鏈），下方 UPDATE 再把 reviewer_signature_id 重指向新簽章。對「已完成」
        // 紀錄的不可覆寫由上方狀態守衛負責；對紀錄內容的不可竄改由 update/delete 的
        // is_signed 鎖（`已簽收的維修保養紀錄不可修改`）負責，均不受本變更影響。

        let content = format!("maintenance_reviewer:{}", record_id);
        let signature = SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
            "maintenance_reviewer",
            &record_id.to_string(),
            current_user.id,
            sig_type,
            &content,
            password,
            handwriting_svg,
            stroke_data,
        )
        .await?;

        // 若待驗收紀錄殘留前次未完成驗收的孤兒簽章，於同 tx 內作廢它（is_valid=false +
        // SIGNATURE_INVALIDATED 稽核事件），避免同一實體同時存在多個有效簽章（21 CFR
        // Part 11 簽章唯一性）。舊 row 仍保留於 electronic_signatures 以維持 HMAC 稽核鏈。
        if let Some(orphan_sig_id) = existing.reviewer_signature_id {
            SignatureService::invalidate_tx(
                &mut tx,
                actor,
                orphan_sig_id,
                MAINTENANCE_RESIGN_SUPERSEDE_REASON,
                current_user.id,
            )
            .await?;
        }

        let updated = sqlx::query_as::<_, EquipmentMaintenanceRecord>(
            "UPDATE equipment_maintenance_records \
             SET reviewer_signature_id = $1, updated_at = NOW() WHERE id = $2 \
             RETURNING *",
        )
        .bind(signature.id)
        .bind(record_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "maintenance_reviewer_signature:{:?}",
            updated.maintenance_type
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "EQUIPMENT",
                event_type: "MAINTENANCE_REVIEWER_SIGNATURE",
                entity: Some(AuditEntity::new("maintenance_record", record_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&existing), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(signature)
    }
}

/// 設備自動恢復啟用（維修驗收通過時呼叫）。
///
/// 接受 `&mut Transaction`，讓 caller（`review_maintenance_record`）能將本函式
/// 的 status_log INSERT + equipment UPDATE **與同 tx 的 maintenance record 狀態
/// 更新 + audit** 一起原子落地。
///
/// Gemini PR #166 HIGH 指出：原 `pool`-based 版本在後續 maintenance record UPDATE
/// 失敗時，equipment status 已獨立 commit，產生資料不一致（設備已「恢復」但
/// 維修紀錄卻沒通過驗收）。本修復將函式簽名改為接 tx，由 caller 保證同 tx。
async fn auto_restore_equipment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &ActorContext,
    equipment_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    // tx 內 SELECT FOR UPDATE：避免與其他 equipment status 變更併發衝突
    let before = sqlx::query_as::<_, Equipment>("SELECT * FROM equipment WHERE id = $1 FOR UPDATE")
        .bind(equipment_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("設備不存在".into()))?;
    validate_status_transition(&before.status, &EquipmentStatus::Active)?;

    sqlx::query(
        "INSERT INTO equipment_status_logs (equipment_id, old_status, new_status, changed_by, reason) VALUES ($1, $2, 'active', $3, '維修驗收通過，自動恢復狀態')",
    )
    .bind(equipment_id)
    .bind(&before.status)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    let after = sqlx::query_as::<_, Equipment>(
        "UPDATE equipment SET status = 'active', is_active = true, updated_at = NOW() WHERE id = $1 RETURNING *",
    )
    .bind(equipment_id)
    .fetch_one(&mut **tx)
    .await?;

    // Gemini PR #167 MEDIUM：R26 DoD-1 要求所有 mutation（含狀態轉換）同 tx 寫 audit。
    let display = format!("{} → active", after.name);
    AuditService::log_activity_tx(
        tx,
        actor,
        ActivityLogEntry {
            event_category: "EQUIPMENT",
            event_type: "EQUIPMENT_AUTO_RESTORE",
            entity: Some(AuditEntity::new("equipment", after.id, &display)),
            data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
            request_context: None,
        },
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod maintenance_sort_tests {
    use super::resolve_maintenance_order_by;
    use crate::models::MaintenanceQuery;

    fn query(sort_by: Option<&str>, sort_order: Option<&str>) -> MaintenanceQuery {
        MaintenanceQuery {
            equipment_id: None,
            maintenance_type: None,
            status: None,
            sort_by: sort_by.map(str::to_string),
            sort_order: sort_order.map(str::to_string),
            page: None,
            per_page: None,
        }
    }

    #[test]
    fn defaults_to_reported_at_desc_when_unset() {
        assert_eq!(
            resolve_maintenance_order_by(&query(None, None)),
            "m.reported_at DESC, m.created_at DESC"
        );
    }

    #[test]
    fn whitelisted_column_with_asc() {
        assert_eq!(
            resolve_maintenance_order_by(&query(Some("completed_at"), Some("asc"))),
            "m.completed_at ASC NULLS LAST, m.created_at DESC"
        );
    }

    #[test]
    fn whitelisted_column_defaults_to_desc() {
        assert_eq!(
            resolve_maintenance_order_by(&query(Some("equipment_name"), None)),
            "e.name DESC NULLS LAST, m.created_at DESC"
        );
    }

    #[test]
    fn injection_attempt_falls_back_to_default() {
        // 非白名單 key（含 SQL 片段）一律忽略，回退預設，杜絕注入。
        let malicious = "reported_at; DROP TABLE equipment_maintenance_records";
        assert_eq!(
            resolve_maintenance_order_by(&query(Some(malicious), Some("asc"))),
            "m.reported_at DESC, m.created_at DESC"
        );
    }
}
