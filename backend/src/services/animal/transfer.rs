use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalStatus, AnimalTransfer, AnimalTransferStatus,
        AssignTransferPlanRequest, CreateTransferRequest, DataBoundaryResponse,
        RejectTransferRequest, TransferVetEvaluation, VetEvaluateTransferRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    AppError, Result,
};

pub struct AnimalTransferService;

impl AnimalTransferService {
    // ============================================
    // 動物轉讓流程
    // ============================================

    /// 取得資料隔離時間界線
    /// 回傳該動物最近一筆已完成轉讓的 completed_at
    /// 新 PI 應只看到 created_at > boundary 的紀錄
    pub async fn get_data_boundary(
        pool: &PgPool,
        animal_id: Uuid,
        _current_user_id: Uuid,
        user_roles: &[String],
    ) -> Result<DataBoundaryResponse> {
        // Admin / VET / IACUC_STAFF 可看到所有紀錄
        let privileged = user_roles
            .iter()
            .any(|r| ["ADMIN", crate::constants::ROLE_VET, crate::constants::ROLE_IACUC_STAFF, crate::constants::ROLE_IACUC_CHAIR].contains(&r.as_str()));
        if privileged {
            return Ok(DataBoundaryResponse { boundary: None });
        }

        // 查詢該動物最近一筆已完成轉讓的 completed_at
        let boundary = sqlx::query_scalar::<_, DateTime<Utc>>(
            r#"
            SELECT completed_at FROM animal_transfers
            WHERE animal_id = $1 AND status = 'completed' AND completed_at IS NOT NULL
            ORDER BY completed_at DESC
            LIMIT 1
            "#,
        )
        .bind(animal_id)
        .fetch_optional(pool)
        .await?;

        Ok(DataBoundaryResponse { boundary })
    }

    /// 取得動物的轉讓記錄
    pub async fn list_transfers(pool: &PgPool, animal_id: Uuid) -> Result<Vec<AnimalTransfer>> {
        let records = sqlx::query_as::<_, AnimalTransfer>(
            "SELECT * FROM animal_transfers WHERE animal_id = $1 ORDER BY created_at DESC",
        )
        .bind(animal_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// 取得單一轉讓記錄
    pub async fn get_transfer(pool: &PgPool, transfer_id: Uuid) -> Result<AnimalTransfer> {
        let record =
            sqlx::query_as::<_, AnimalTransfer>("SELECT * FROM animal_transfers WHERE id = $1")
                .bind(transfer_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| AppError::NotFound("轉讓記錄不存在".to_string()))?;

        Ok(record)
    }

    /// 取得轉讓的獸醫評估
    pub async fn get_transfer_vet_evaluation(
        pool: &PgPool,
        transfer_id: Uuid,
    ) -> Result<Option<TransferVetEvaluation>> {
        let record = sqlx::query_as::<_, TransferVetEvaluation>(
            "SELECT * FROM transfer_vet_evaluations WHERE transfer_id = $1",
        )
        .bind(transfer_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// 步驟 1：發起轉讓 — Service-driven audit
    ///
    /// 同 tx：UPDATE animals.status + INSERT animal_transfers + log_activity_tx。
    /// 舊版 pool-based 多步在 panic 時會留下「animal=transferred 但 transfer row
    /// 缺失」的不一致；本次收歸同 tx。
    pub async fn initiate_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        animal_id: Uuid,
        req: &CreateTransferRequest,
    ) -> Result<AnimalTransfer> {
        let user = actor.require_user()?;
        let initiated_by = user.id;

        let transfer_type = match req.transfer_type.as_str() {
            "external" | "internal" => req.transfer_type.clone(),
            _ => "internal".to_string(),
        };

        let mut tx = pool.begin().await?;

        // Gemini PR #179 HIGH：狀態檢查同 tx 並鎖 animal row，避免兩個並發請求
        // 都看到 Completed 狀態然後都進入 pending transfer。
        let animal: crate::models::Animal = sqlx::query_as(
            "SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(animal_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("動物不存在".to_string()))?;

        if animal.status != AnimalStatus::Completed {
            return Err(AppError::BadRequest(format!(
                "只有「存活完成」狀態的動物可以發起轉讓，當前狀態：{}",
                animal.status.display_name()
            )));
        }

        let from_iacuc = animal.iacuc_no.clone().ok_or_else(|| {
            AppError::BadRequest("動物未指定 IACUC No.，無法發起轉讓".to_string())
        })?;

        // 檢查是否有進行中的轉讓（同 tx 可見性；父層 animal 已鎖）
        let active = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM animal_transfers WHERE animal_id = $1 AND status NOT IN ('completed', 'rejected')"
        )
        .bind(animal_id)
        .fetch_one(&mut *tx)
        .await?;

        if active > 0 {
            return Err(AppError::BadRequest(
                "此動物已有進行中的轉讓申請".to_string(),
            ));
        }

        // 更新動物狀態為「已轉讓」（中間態，表示正在轉讓流程中）
        sqlx::query("UPDATE animals SET status = 'transferred', updated_at = NOW() WHERE id = $1")
            .bind(animal_id)
            .execute(&mut *tx)
            .await?;

        // Gemini PR #179 MED：current_count 排除 transferred，Completed→Transferred
        // 須立即 recalc 避免 pen count 在 workflow 期間持續偏高
        // （長期解法見 issue #180）
        if let Some(pid) = animal.pen_id {
            sqlx::query(
                "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
            )
            .bind(pid)
            .execute(&mut *tx)
            .await?;
        }

        let record = sqlx::query_as::<_, AnimalTransfer>(
            r#"
            INSERT INTO animal_transfers (animal_id, from_iacuc_no, status, transfer_type, initiated_by, reason, remark)
            VALUES ($1, $2, 'pending', $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(animal_id)
        .bind(&from_iacuc)
        .bind(&transfer_type)
        .bind(initiated_by)
        .bind(&req.reason)
        .bind(&req.remark)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "[{}] {} - 發起轉讓",
            from_iacuc,
            animal.ear_tag,
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_INITIATE",
                entity: Some(AuditEntity::new("animal_transfers", record.id, &display)),
                data_diff: Some(DataDiff::create_only(&record)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(record)
    }

    /// 步驟 2：獸醫評估 — Service-driven audit
    pub async fn vet_evaluate_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        transfer_id: Uuid,
        req: &VetEvaluateTransferRequest,
    ) -> Result<AnimalTransfer> {
        let user = actor.require_user()?;
        let vet_id = user.id;

        let before = Self::get_transfer(pool, transfer_id).await?;

        if before.status != AnimalTransferStatus::Pending {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「待審」，當前：{}",
                before.status.display_name()
            )));
        }

        let mut tx = pool.begin().await?;

        // 建立獸醫評估紀錄
        sqlx::query(
            r#"
            INSERT INTO transfer_vet_evaluations (transfer_id, vet_id, health_status, is_fit_for_transfer, conditions)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(transfer_id)
        .bind(vet_id)
        .bind(&req.health_status)
        .bind(req.is_fit_for_transfer)
        .bind(&req.conditions)
        .execute(&mut *tx)
        .await?;

        // 更新轉讓狀態
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'vet_evaluated', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(transfer_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "轉讓獸醫評估：{}",
            if req.is_fit_for_transfer { "適合轉讓" } else { "不適合轉讓" }
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_VET_EVALUATE",
                entity: Some(AuditEntity::new("animal_transfers", transfer_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    /// 步驟 3：指定新計劃 — Service-driven audit
    pub async fn assign_transfer_plan(
        pool: &PgPool,
        actor: &ActorContext,
        transfer_id: Uuid,
        req: &AssignTransferPlanRequest,
    ) -> Result<AnimalTransfer> {
        let _ = actor.require_user()?;

        let before = Self::get_transfer(pool, transfer_id).await?;

        if before.status != AnimalTransferStatus::VetEvaluated {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「獸醫已評估」，當前：{}",
                before.status.display_name()
            )));
        }

        let mut tx = pool.begin().await?;

        // 驗證目標計劃存在
        let plan_exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM protocols WHERE iacuc_no = $1")
                .bind(&req.to_iacuc_no)
                .fetch_one(&mut *tx)
                .await?;

        if plan_exists == 0 {
            return Err(AppError::BadRequest(format!(
                "目標 IACUC No. '{}' 不存在",
                req.to_iacuc_no
            )));
        }

        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET to_iacuc_no = $1, status = 'plan_assigned', updated_at = NOW() WHERE id = $2 RETURNING *"
        )
        .bind(&req.to_iacuc_no)
        .bind(transfer_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("轉讓指定新計劃：{}", req.to_iacuc_no);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_ASSIGN_PLAN",
                entity: Some(AuditEntity::new("animal_transfers", transfer_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    /// 步驟 4：PI 同意 — Service-driven audit
    pub async fn approve_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        transfer_id: Uuid,
    ) -> Result<AnimalTransfer> {
        let _ = actor.require_user()?;

        let before = Self::get_transfer(pool, transfer_id).await?;

        if before.status != AnimalTransferStatus::PlanAssigned {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「已指定新計劃」，當前：{}",
                before.status.display_name()
            )));
        }

        let mut tx = pool.begin().await?;

        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'pi_approved', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(transfer_id)
        .fetch_one(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_APPROVE",
                entity: Some(AuditEntity::new("animal_transfers", transfer_id, "PI 同意轉讓")),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    /// 步驟 5：完成轉讓（將動物分配到新計劃） — Service-driven audit
    ///
    /// 同 tx：UPDATE animals（改 IACUC + 狀態 → in_experiment）+ 清舊 pen count
    /// （external only）+ UPDATE animal_transfers → completed + log_activity_tx。
    /// 避免中途失敗留下「transfer.completed 但 animal 狀態未更新」的不一致。
    pub async fn complete_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        transfer_id: Uuid,
    ) -> Result<AnimalTransfer> {
        let _ = actor.require_user()?;

        let before = Self::get_transfer(pool, transfer_id).await?;

        if before.status != AnimalTransferStatus::PiApproved {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「PI 已同意」，當前：{}",
                before.status.display_name()
            )));
        }

        let to_iacuc = before
            .to_iacuc_no
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("未指定目標 IACUC No.".to_string()))?
            .clone();

        let mut tx = pool.begin().await?;

        // 取得舊 pen_id 用於更新 current_count
        let old_pen_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT pen_id FROM animals WHERE id = $1",
        )
        .bind(before.animal_id)
        .fetch_optional(&mut *tx)
        .await?
        .flatten();

        // 更新動物：新 IACUC No. + 狀態 → in_experiment；若為「轉給其他機構」則清空欄位
        let is_external = before.transfer_type.as_str() == "external";
        if is_external {
            sqlx::query(
                "UPDATE animals SET iacuc_no = $1, status = 'in_experiment', pen_location = NULL, pen_id = NULL, updated_at = NOW() WHERE id = $2"
            )
            .bind(&to_iacuc)
            .bind(before.animal_id)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                "UPDATE animals SET iacuc_no = $1, status = 'in_experiment', updated_at = NOW() WHERE id = $2"
            )
            .bind(&to_iacuc)
            .bind(before.animal_id)
            .execute(&mut *tx)
            .await?;
        }

        // 更新舊 pen 的 current_count（Gemini PR #179）
        // External：動物離開 pen（pen_id 已清為 NULL），count 減 1
        // Internal：動物狀態從 'transferred' → 'in_experiment'，重新計入 count（count 增 1）
        // 兩者都需 recalc；不再以 is_external gating。
        // 註：此為短期修補；移除 'transferred' 中間狀態的較大重構，見
        // <https://github.com/delightening/ipig_system/issues/180>。
        if let Some(pid) = old_pen_id {
            sqlx::query(
                "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
            )
            .bind(pid)
            .execute(&mut *tx)
            .await?;
        }

        // 更新轉讓狀態為完成
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(transfer_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "轉讓完成：{} → {}",
            updated.from_iacuc_no,
            updated.to_iacuc_no.as_deref().unwrap_or("未知")
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_COMPLETE",
                entity: Some(AuditEntity::new("animal_transfers", transfer_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    /// 拒絕轉讓 — Service-driven audit
    ///
    /// 同 tx：UPDATE animals.status 回 completed + UPDATE animal_transfers → rejected
    /// + log_activity_tx。
    pub async fn reject_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        transfer_id: Uuid,
        req: &RejectTransferRequest,
    ) -> Result<AnimalTransfer> {
        let user = actor.require_user()?;
        let rejected_by = user.id;

        let before = Self::get_transfer(pool, transfer_id).await?;

        if before.status == AnimalTransferStatus::Completed
            || before.status == AnimalTransferStatus::Rejected
        {
            return Err(AppError::BadRequest(format!(
                "轉讓已為終態「{}」，無法拒絕",
                before.status.display_name()
            )));
        }

        let mut tx = pool.begin().await?;

        // 回復動物狀態為 completed
        sqlx::query("UPDATE animals SET status = 'completed', updated_at = NOW() WHERE id = $1")
            .bind(before.animal_id)
            .execute(&mut *tx)
            .await?;

        // Gemini PR #179 MED：Transferred→Completed 須 recalc pen count（completed
        // 狀態應計入總數；initiate 已扣、reject 須加回）。長期解法見 issue #180。
        let pen_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT pen_id FROM animals WHERE id = $1",
        )
        .bind(before.animal_id)
        .fetch_optional(&mut *tx)
        .await?
        .flatten();
        if let Some(pid) = pen_id {
            sqlx::query(
                "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
            )
            .bind(pid)
            .execute(&mut *tx)
            .await?;
        }

        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'rejected', rejected_by = $1, rejected_reason = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
        )
        .bind(rejected_by)
        .bind(&req.reason)
        .bind(transfer_id)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!("拒絕轉讓：{}", req.reason);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_REJECT",
                entity: Some(AuditEntity::new("animal_transfers", transfer_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }
}
