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
        access,
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
        let privileged = user_roles.iter().any(|r| {
            [
                "ADMIN",
                crate::constants::ROLE_VET,
                crate::constants::ROLE_IACUC_STAFF,
                crate::constants::ROLE_IACUC_CHAIR,
            ]
            .contains(&r.as_str())
        });
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
    pub async fn list_transfers(
        pool: &PgPool,
        scope: access::Scoped<access::AnimalRead>,
    ) -> Result<Vec<AnimalTransfer>> {
        let animal_id = scope.id();
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
        _scope: access::Scoped<access::AnimalRead>,
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

    /// R75 歸屬約束：轉讓紀錄須屬於已授權的動物（service 層自我強制，不只信任 handler）。
    fn require_ownership(
        record: &AnimalTransfer,
        scope: &access::Scoped<access::AnimalWrite>,
    ) -> Result<()> {
        if record.animal_id != scope.id() {
            return Err(AppError::Forbidden("此轉讓紀錄不屬於已授權的動物".into()));
        }
        Ok(())
    }

    /// 步驟 1：發起轉讓 — Service-driven audit
    ///
    /// 同 tx：SELECT FOR UPDATE（鎖 animal 列防並發重複發起）、INSERT animal_transfers、log_activity_tx。
    ///
    /// issue #180 起**不再** UPDATE animals.status——動物在簽核期間仍在原欄，狀態維持 `completed`。
    pub async fn initiate_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        scope: access::Scoped<access::AnimalWrite>,
        req: &CreateTransferRequest,
    ) -> Result<AnimalTransfer> {
        let animal_id = scope.id();
        let user = actor.require_user()?;
        let initiated_by = user.id;

        let transfer_type = match req.transfer_type.as_str() {
            "external" | "internal" => req.transfer_type.clone(),
            _ => "internal".to_string(),
        };

        let mut tx = pool.begin().await?;

        // Gemini PR #179 HIGH：狀態檢查同 tx 並鎖 animal row，避免兩個並發請求
        // 都看到 Completed 狀態然後都進入 pending transfer。
        let animal: crate::models::Animal =
            sqlx::query_as("SELECT * FROM animals WHERE id = $1 AND deleted_at IS NULL FOR UPDATE")
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

        // issue #180：**刻意不改動 animals.status**。動物在整個簽核期間（發起 → 獸醫評估 →
        // 指派 → PI 同意）仍實際待在原欄，狀態維持 `completed` 才與現實一致。
        // 舊版在此設 'transferred' 作為中間態，衍生三個問題：欄舍頭數立即偏低（須補償 recalc）、
        // 前端得把「已轉讓」特判成「轉讓申請中」、駁回時須回滾狀態。
        // 「轉讓申請中」改由 `animal_transfers` 未結案列表示（見 `AnimalListItem::pending_transfer_status`）。
        // 連帶：pen count 於本步驟不再變動，原補償性 recalc 一併移除。

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

        let display = format!("[{}] {} - 發起轉讓", from_iacuc, animal.ear_tag,);
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
        scope: access::Scoped<access::AnimalWrite>,
        transfer_id: Uuid,
        req: &VetEvaluateTransferRequest,
    ) -> Result<AnimalTransfer> {
        let user = actor.require_user()?;
        let vet_id = user.id;

        let before = Self::get_transfer(pool, transfer_id).await?;
        Self::require_ownership(&before, &scope)?;

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

        // 更新轉讓狀態（#179：WHERE 帶 status 防並發重複推進）
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'vet_evaluated', updated_at = NOW() WHERE id = $1 AND status = 'pending' RETURNING *"
        )
        .bind(transfer_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("此記錄已被其他人修改，請重新載入後再試。".to_string()))?;

        let display = format!(
            "轉讓獸醫評估：{}",
            if req.is_fit_for_transfer {
                "適合轉讓"
            } else {
                "不適合轉讓"
            }
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
        scope: access::Scoped<access::AnimalWrite>,
        transfer_id: Uuid,
        req: &AssignTransferPlanRequest,
    ) -> Result<AnimalTransfer> {
        let _ = actor.require_user()?;

        let before = Self::get_transfer(pool, transfer_id).await?;
        Self::require_ownership(&before, &scope)?;

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

        // #179：WHERE 帶 status 防並發重複推進
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET to_iacuc_no = $1, status = 'plan_assigned', updated_at = NOW() WHERE id = $2 AND status = 'vet_evaluated' RETURNING *"
        )
        .bind(&req.to_iacuc_no)
        .bind(transfer_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("此記錄已被其他人修改，請重新載入後再試。".to_string()))?;

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
        scope: access::Scoped<access::AnimalWrite>,
        transfer_id: Uuid,
    ) -> Result<AnimalTransfer> {
        let user = actor.require_user()?;

        let before = Self::get_transfer(pool, transfer_id).await?;
        Self::require_ownership(&before, &scope)?;

        if before.status != AnimalTransferStatus::PlanAssigned {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「已指定新計劃」，當前：{}",
                before.status.display_name()
            )));
        }

        // #179 SoD：發起轉讓者不得自行核准（職權分離）。簽署權責（VET / 轉出入計劃 PI）
        // 由 handler 端 check_transfer_signing_authority 把關。
        if before.initiated_by == user.id {
            return Err(AppError::Forbidden(
                "發起轉讓者不得自行核准（職權分離）".into(),
            ));
        }

        let mut tx = pool.begin().await?;

        // #179：WHERE 帶 status 防並發重複推進（TOCTOU）。
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'pi_approved', updated_at = NOW() WHERE id = $1 AND status = 'plan_assigned' RETURNING *"
        )
        .bind(transfer_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("此記錄已被其他人修改，請重新載入後再試。".to_string()))?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: "TRANSFER_APPROVE",
                entity: Some(AuditEntity::new(
                    "animal_transfers",
                    transfer_id,
                    "PI 同意轉讓",
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&updated))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    /// external 轉讓完成：動物交付其他機構、實際離場。
    ///
    /// status 落到 `transferred`（終態）並清空 `pen_id` / `pen_location`；因動物離欄，
    /// 舊欄 `current_count` 需重算（豬離開後該欄少一隻）。
    ///
    /// issue #180 前此分支也寫 `in_experiment`，導致已送出場的動物永遠顯示「實驗中」、
    /// 且被 `repositories/ai.rs` 的 active_animals 統計成在養動物。
    async fn apply_external_completion_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        animal_id: Uuid,
        to_iacuc: &str,
    ) -> Result<()> {
        // 先取舊 pen_id：下一句就會把它清成 NULL，之後拿不到。
        let old_pen_id: Option<Uuid> =
            sqlx::query_scalar("SELECT pen_id FROM animals WHERE id = $1")
                .bind(animal_id)
                .fetch_optional(&mut **tx)
                .await?
                .flatten();

        sqlx::query(
            "UPDATE animals SET iacuc_no = $1, status = 'transferred', pen_location = NULL, pen_id = NULL, updated_at = NOW() WHERE id = $2"
        )
        .bind(to_iacuc)
        .bind(animal_id)
        .execute(&mut **tx)
        .await?;

        if let Some(pid) = old_pen_id {
            sqlx::query(
                "UPDATE pens SET current_count = (SELECT COUNT(*) FROM animals WHERE pen_id = $1 AND deleted_at IS NULL AND status NOT IN ('euthanized', 'sudden_death', 'transferred')) WHERE id = $1"
            )
            .bind(pid)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    /// internal 轉讓完成：動物留在本院，直接由計畫 A 進計畫 B。
    ///
    /// status 落到 `in_experiment`、欄位不動。**刻意不重算 `current_count`**：動物沒離欄，
    /// 且 `completed` 與 `in_experiment` 都計入頭數（排除清單只有 euthanized /
    /// sudden_death / transferred）→ 頭數不變。issue #180 前因中間態使頭數於流程期間偏低，
    /// 才需要在此補回。
    async fn apply_internal_completion_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        animal_id: Uuid,
        to_iacuc: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE animals SET iacuc_no = $1, status = 'in_experiment', updated_at = NOW() WHERE id = $2"
        )
        .bind(to_iacuc)
        .bind(animal_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// 步驟 5：完成轉讓（將動物分配到新計劃） — Service-driven audit
    ///
    /// 同 tx：UPDATE animals（改 IACUC 與狀態）、清舊 pen count（僅 external）、
    /// UPDATE animal_transfers 轉 completed、log_activity_tx。
    /// 避免中途失敗留下「transfer.completed 但 animal 狀態未更新」的不一致。
    ///
    /// issue #180：本步驟是**唯一**改動 animals.status 的轉讓步驟。
    /// internal → `in_experiment`（進計畫 B）；external → `transferred`（離場終態）。
    pub async fn complete_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        scope: access::Scoped<access::AnimalWrite>,
        transfer_id: Uuid,
    ) -> Result<AnimalTransfer> {
        let _ = actor.require_user()?;

        let before = Self::get_transfer(pool, transfer_id).await?;
        Self::require_ownership(&before, &scope)?;

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

        if before.transfer_type.as_str() == "external" {
            Self::apply_external_completion_tx(&mut tx, before.animal_id, &to_iacuc).await?;
        } else {
            Self::apply_internal_completion_tx(&mut tx, before.animal_id, &to_iacuc).await?;
        }

        // 更新轉讓狀態為完成（#179：WHERE 帶 status 防並發；0 row → Conflict → 整 tx 回滾，
        // 連同上方 animal 更新一併還原）
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1 AND status = 'pi_approved' RETURNING *"
        )
        .bind(transfer_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("此記錄已被其他人修改，請重新載入後再試。".to_string()))?;

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
    /// 同 tx：UPDATE animal_transfers → rejected + log_activity_tx。
    /// issue #180 起**無需回滾 animals.status**：發起轉讓本就不改動物狀態，駁回自然是 no-op，
    /// 一併消掉舊版「回復 completed + 補回 pen count」這條容易漏、且與現實不符的回滾路徑。
    pub async fn reject_transfer(
        pool: &PgPool,
        actor: &ActorContext,
        scope: access::Scoped<access::AnimalWrite>,
        transfer_id: Uuid,
        req: &RejectTransferRequest,
    ) -> Result<AnimalTransfer> {
        let user = actor.require_user()?;
        let rejected_by = user.id;

        let before = Self::get_transfer(pool, transfer_id).await?;
        Self::require_ownership(&before, &scope)?;

        if before.status == AnimalTransferStatus::Completed
            || before.status == AnimalTransferStatus::Rejected
        {
            return Err(AppError::BadRequest(format!(
                "轉讓已為終態「{}」，無法拒絕",
                before.status.display_name()
            )));
        }

        let mut tx = pool.begin().await?;

        // issue #180：動物狀態與 pen count 於整個轉讓流程中均未被動過，駁回無須回滾。

        // CodeRabbit #1103：WHERE 帶終態排除防並發（TOCTOU）。上方 `before` 是在 tx 之外讀的，
        // 若 `complete_transfer` 於其間先提交，舊版無條件 UPDATE 會把已完成、`completed_at`
        // 已寫入的轉讓覆寫成 rejected。與同檔其餘四步（#179）的樂觀鎖慣例對齊：0 row → Conflict。
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'rejected', rejected_by = $1, rejected_reason = $2, updated_at = NOW() WHERE id = $3 AND status NOT IN ('completed', 'rejected') RETURNING *"
        )
        .bind(rejected_by)
        .bind(&req.reason)
        .bind(transfer_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict("此記錄已被其他人修改，請重新載入後再試。".to_string()))?;

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
