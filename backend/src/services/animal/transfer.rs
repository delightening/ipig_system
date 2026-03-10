use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::AnimalService;
use crate::{
    models::{
        AnimalStatus, AnimalTransfer, AnimalTransferStatus, AssignTransferPlanRequest,
        CreateTransferRequest, DataBoundaryResponse, RejectTransferRequest, TransferVetEvaluation,
        VetEvaluateTransferRequest,
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
            .any(|r| matches!(r.as_str(), "ADMIN" | "VET" | "IACUC_STAFF" | "IACUC_CHAIR"));
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

    /// 步驟 1：發起轉讓
    pub async fn initiate_transfer(
        pool: &PgPool,
        animal_id: Uuid,
        req: &CreateTransferRequest,
        initiated_by: Uuid,
    ) -> Result<AnimalTransfer> {
        // 驗證動物狀態
        let animal = AnimalService::get_by_id(pool, animal_id).await?;
        if animal.status != AnimalStatus::Completed {
            return Err(AppError::BadRequest(format!(
                "只有「存活完成」狀態的動物可以發起轉讓，當前狀態：{}",
                animal.status.display_name()
            )));
        }

        // 檢查是否有進行中的轉讓
        let active = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM animal_transfers WHERE animal_id = $1 AND status NOT IN ('completed', 'rejected')"
        )
        .bind(animal_id)
        .fetch_one(pool)
        .await?;

        if active > 0 {
            return Err(AppError::BadRequest(
                "此動物已有進行中的轉讓申請".to_string(),
            ));
        }

        let from_iacuc = animal.iacuc_no.ok_or_else(|| {
            AppError::BadRequest("動物未指定 IACUC No.，無法發起轉讓".to_string())
        })?;

        let transfer_type = match req.transfer_type.as_str() {
            "external" | "internal" => req.transfer_type.clone(),
            _ => "internal".to_string(),
        };

        // 更新動物狀態為「已轉讓」（中間態，表示正在轉讓流程中）
        sqlx::query("UPDATE animals SET status = 'transferred', updated_at = NOW() WHERE id = $1")
            .bind(animal_id)
            .execute(pool)
            .await?;

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
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// 步驟 2：獸醫評估
    pub async fn vet_evaluate_transfer(
        pool: &PgPool,
        transfer_id: Uuid,
        req: &VetEvaluateTransferRequest,
        vet_id: Uuid,
    ) -> Result<AnimalTransfer> {
        let transfer = Self::get_transfer(pool, transfer_id).await?;

        if transfer.status != AnimalTransferStatus::Pending {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「待審」，當前：{}",
                transfer.status.display_name()
            )));
        }

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
        .execute(pool)
        .await?;

        // 更新轉讓狀態
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'vet_evaluated', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(transfer_id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    /// 步驟 3：指定新計劃
    pub async fn assign_transfer_plan(
        pool: &PgPool,
        transfer_id: Uuid,
        req: &AssignTransferPlanRequest,
    ) -> Result<AnimalTransfer> {
        let transfer = Self::get_transfer(pool, transfer_id).await?;

        if transfer.status != AnimalTransferStatus::VetEvaluated {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「獸醫已評估」，當前：{}",
                transfer.status.display_name()
            )));
        }

        // 驗證目標計劃存在
        let plan_exists =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM protocols WHERE iacuc_no = $1")
                .bind(&req.to_iacuc_no)
                .fetch_one(pool)
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
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    /// 步驟 4：PI 同意
    pub async fn approve_transfer(pool: &PgPool, transfer_id: Uuid) -> Result<AnimalTransfer> {
        let transfer = Self::get_transfer(pool, transfer_id).await?;

        if transfer.status != AnimalTransferStatus::PlanAssigned {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「已指定新計劃」，當前：{}",
                transfer.status.display_name()
            )));
        }

        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'pi_approved', updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(transfer_id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    /// 步驟 5：完成轉讓（將動物分配到新計劃）
    pub async fn complete_transfer(pool: &PgPool, transfer_id: Uuid) -> Result<AnimalTransfer> {
        let transfer = Self::get_transfer(pool, transfer_id).await?;

        if transfer.status != AnimalTransferStatus::PiApproved {
            return Err(AppError::BadRequest(format!(
                "轉讓狀態不正確，需為「PI 已同意」，當前：{}",
                transfer.status.display_name()
            )));
        }

        let to_iacuc = transfer
            .to_iacuc_no
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("未指定目標 IACUC No.".to_string()))?;

        // 更新動物：新 IACUC No. + 狀態 → in_experiment；若為「轉給其他機構」則清空欄位
        let is_external = transfer.transfer_type.as_str() == "external";
        if is_external {
            sqlx::query(
                "UPDATE animals SET iacuc_no = $1, status = 'in_experiment', pen_location = NULL, updated_at = NOW() WHERE id = $2"
            )
            .bind(to_iacuc)
            .bind(transfer.animal_id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE animals SET iacuc_no = $1, status = 'in_experiment', updated_at = NOW() WHERE id = $2"
            )
            .bind(to_iacuc)
            .bind(transfer.animal_id)
            .execute(pool)
            .await?;
        }

        // 更新轉讓狀態為完成
        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(transfer_id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }

    /// 拒絕轉讓
    pub async fn reject_transfer(
        pool: &PgPool,
        transfer_id: Uuid,
        req: &RejectTransferRequest,
        rejected_by: Uuid,
    ) -> Result<AnimalTransfer> {
        let transfer = Self::get_transfer(pool, transfer_id).await?;

        if transfer.status == AnimalTransferStatus::Completed
            || transfer.status == AnimalTransferStatus::Rejected
        {
            return Err(AppError::BadRequest(format!(
                "轉讓已為終態「{}」，無法拒絕",
                transfer.status.display_name()
            )));
        }

        // 回復動物狀態為 completed
        sqlx::query("UPDATE animals SET status = 'completed', updated_at = NOW() WHERE id = $1")
            .bind(transfer.animal_id)
            .execute(pool)
            .await?;

        let updated = sqlx::query_as::<_, AnimalTransfer>(
            "UPDATE animal_transfers SET status = 'rejected', rejected_by = $1, rejected_reason = $2, updated_at = NOW() WHERE id = $3 RETURNING *"
        )
        .bind(rejected_by)
        .bind(&req.reason)
        .bind(transfer_id)
        .fetch_one(pool)
        .await?;

        Ok(updated)
    }
}
