use chrono::{Duration, Utc};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        ChairDecisionRequest, CreateEuthanasiaAppealRequest, CreateEuthanasiaOrderRequest,
        EuthanasiaAppeal, EuthanasiaOrder, EuthanasiaOrderResponse, AnimalStatus,
    },
    services::NotificationService,
};

/// 輔助結構：查詢動物關聯 PI 資訊
#[derive(FromRow)]
struct AnimalPiRecord {
    #[allow(dead_code)]
    id: Uuid,
    ear_tag: String,
    iacuc_no: Option<String>,
    pi_user_id: Option<Uuid>,
}

/// 輔助結構：超時安樂死單 RETURNING
#[derive(FromRow)]
struct ExpiredOrderRow {
    id: Uuid,
    vet_user_id: Uuid,
}

/// 輔助結構：超時暫緩申請
#[derive(FromRow)]
struct ExpiredAppealRow {
    id: Uuid,
    order_id: Uuid,
    vet_user_id: Uuid,
}

pub struct EuthanasiaService;

impl EuthanasiaService {
    /// 建立安樂死單據 (獸醫開立)
    pub async fn create_order(
        pool: &PgPool,
        req: &CreateEuthanasiaOrderRequest,
        vet_user_id: Uuid,
    ) -> Result<EuthanasiaOrder, AppError> {
        // 查詢動物的關聯 PI
        let animal_record = sqlx::query_as::<_, AnimalPiRecord>(
            r#"
            SELECT p.id, p.ear_tag, p.iacuc_no, pr.pi_user_id
            FROM animals p
            LEFT JOIN protocols pr ON p.iacuc_no = pr.iacuc_no
            WHERE p.id = $1 AND p.deleted_at IS NULL
            "#,
        )
        .bind(req.animal_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到指定的動物".to_string()))?;

        let pi_user_id = animal_record.pi_user_id.filter(|u| !u.is_nil()).ok_or_else(|| {
            AppError::BadRequest("該動物尚未關聯至任何計畫，無法開立安樂死單".to_string())
        })?;

        // 設定 24 小時後過期
        let deadline_at = Utc::now() + Duration::hours(24);

        let order = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            INSERT INTO euthanasia_orders (animal_id, vet_user_id, pi_user_id, reason, deadline_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at
            "#,
        )
        .bind(req.animal_id)
        .bind(vet_user_id)
        .bind(pi_user_id)
        .bind(&req.reason)
        .bind(deadline_at)
        .fetch_one(pool)
        .await?;

        // 發送通知給 PI
        let notification_service = NotificationService::new(pool.clone());
        if let Err(e) = notification_service
            .notify_euthanasia_order(
                order.id,
                &animal_record.ear_tag,
                animal_record.iacuc_no.as_deref(),
                &req.reason,
                pi_user_id,
            )
            .await {
            tracing::warn!("發送安樂死通知失敗: {e}");
        }

        Ok(order)
    }

    /// 取得安樂死單據詳情
    pub async fn get_order_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<EuthanasiaOrderResponse, AppError> {
        let order = sqlx::query_as::<_, EuthanasiaOrderResponse>(
            r#"
            SELECT 
                eo.id, eo.animal_id, eo.vet_user_id, eo.pi_user_id, eo.reason,
                eo.status as "status: EuthanasiaOrderStatus",
                eo.deadline_at, eo.pi_responded_at, eo.executed_at, eo.executed_by,
                eo.created_at, eo.updated_at,
                p.ear_tag as animal_ear_tag,
                p.iacuc_no as animal_iacuc_no,
                uv.display_name as vet_name,
                up.display_name as pi_name
            FROM euthanasia_orders eo
            JOIN animals p ON eo.animal_id = p.id
            JOIN users uv ON eo.vet_user_id = uv.id
            JOIN users up ON eo.pi_user_id = up.id
            WHERE eo.id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到安樂死單據".to_string()))?;

        Ok(order)
    }

    /// 取得 PI 的待處理安樂死單據
    pub async fn get_pending_orders_for_pi(
        pool: &PgPool,
        pi_user_id: Uuid,
    ) -> Result<Vec<EuthanasiaOrderResponse>, AppError> {
        let orders = sqlx::query_as::<_, EuthanasiaOrderResponse>(
            r#"
            SELECT 
                eo.id, eo.animal_id, eo.vet_user_id, eo.pi_user_id, eo.reason,
                eo.status as "status: EuthanasiaOrderStatus",
                eo.deadline_at, eo.pi_responded_at, eo.executed_at, eo.executed_by,
                eo.created_at, eo.updated_at,
                p.ear_tag as animal_ear_tag,
                p.iacuc_no as animal_iacuc_no,
                uv.display_name as vet_name,
                up.display_name as pi_name
            FROM euthanasia_orders eo
            JOIN animals p ON eo.animal_id = p.id
            JOIN users uv ON eo.vet_user_id = uv.id
            JOIN users up ON eo.pi_user_id = up.id
            WHERE eo.pi_user_id = $1 AND eo.status = 'pending_pi'
            ORDER BY eo.deadline_at ASC
            "#,
        )
        .bind(pi_user_id)
        .fetch_all(pool)
        .await?;

        Ok(orders)
    }

    /// PI 同意執行安樂死
    pub async fn pi_approve(
        pool: &PgPool,
        order_id: Uuid,
        pi_user_id: Uuid,
    ) -> Result<EuthanasiaOrder, AppError> {
        // 驗證權限
        let order = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at
            FROM euthanasia_orders
            WHERE id = $1 AND pi_user_id = $2 AND status = 'pending_pi'
            "#,
        )
        .bind(order_id)
        .bind(pi_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到待處理的安樂死單據".to_string()))?;

        // 更新狀態為已同意
        let updated = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved', pi_responded_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at
            "#,
        )
        .bind(order_id)
        .fetch_one(pool)
        .await?;

        // 通知獸醫可以執行
        let notification_service = NotificationService::new(pool.clone());
        if let Err(e) = notification_service
            .notify_euthanasia_approved(order_id, order.vet_user_id)
            .await {
            tracing::warn!("發送安樂死通知失敗: {e}");
        }

        Ok(updated)
    }

    /// PI 申請暫緩
    pub async fn pi_appeal(
        pool: &PgPool,
        order_id: Uuid,
        pi_user_id: Uuid,
        req: &CreateEuthanasiaAppealRequest,
    ) -> Result<EuthanasiaAppeal, AppError> {
        // 驗證權限
        let _exists: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM euthanasia_orders
            WHERE id = $1 AND pi_user_id = $2 AND status = 'pending_pi'
            "#,
        )
        .bind(order_id)
        .bind(pi_user_id)
        .fetch_optional(pool)
        .await?;

        if _exists.is_none() {
            return Err(AppError::NotFound("找不到待處理的安樂死單據".to_string()));
        }

        // 查找 CHAIR 用戶
        let chair: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE r.code = 'IACUC_CHAIR' AND u.is_active = true
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        let chair_user_id = chair.map(|c| c.0);
        let chair_deadline = Utc::now() + Duration::hours(24);

        // 建立暫緩申請
        let appeal = sqlx::query_as::<_, EuthanasiaAppeal>(
            r#"
            INSERT INTO euthanasia_appeals (order_id, pi_user_id, reason, attachment_path, chair_user_id, chair_deadline_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                      chair_decision, chair_decided_at, chair_deadline_at, created_at
            "#,
        )
        .bind(order_id)
        .bind(pi_user_id)
        .bind(&req.reason)
        .bind(&req.attachment_path)
        .bind(chair_user_id)
        .bind(chair_deadline)
        .fetch_one(pool)
        .await?;

        // 更新安樂死單據狀態
        sqlx::query(
            r#"
            UPDATE euthanasia_orders
            SET status = 'appealed', pi_responded_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(order_id)
        .execute(pool)
        .await?;

        // 如果有 CHAIR，通知進行仲裁
        if let Some(chair_id) = chair_user_id {
            // 更新為 CHAIR 仲裁中
            sqlx::query(
                r#"
                UPDATE euthanasia_orders
                SET status = 'chair_arbitration', updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(order_id)
            .execute(pool)
            .await?;

            let notification_service = NotificationService::new(pool.clone());
            if let Err(e) = notification_service
                .notify_euthanasia_appeal(appeal.id, order_id, chair_id, &req.reason)
                .await {
                tracing::warn!("發送安樂死通知失敗: {e}");
            }
        }

        Ok(appeal)
    }

    /// CHAIR 裁決
    pub async fn chair_decide(
        pool: &PgPool,
        appeal_id: Uuid,
        chair_user_id: Uuid,
        req: &ChairDecisionRequest,
    ) -> Result<EuthanasiaAppeal, AppError> {
        // 驗證權限：必須是 CHAIR 且為該 appeal 的裁決者
        let _appeal = sqlx::query_as::<_, EuthanasiaAppeal>(
            r#"
            SELECT id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                   chair_decision, chair_decided_at, chair_deadline_at, created_at
            FROM euthanasia_appeals
            WHERE id = $1 AND chair_user_id = $2 AND chair_decision IS NULL
            "#,
        )
        .bind(appeal_id)
        .bind(chair_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到待裁決的暫緩申請".to_string()))?;

        // 更新裁決結果
        let updated = sqlx::query_as::<_, EuthanasiaAppeal>(
            r#"
            UPDATE euthanasia_appeals
            SET chair_decision = $1, chair_decided_at = NOW()
            WHERE id = $2
            RETURNING id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                      chair_decision, chair_decided_at, chair_deadline_at, created_at
            "#,
        )
        .bind(&req.decision)
        .bind(appeal_id)
        .fetch_one(pool)
        .await?;

        // 根據裁決結果更新安樂死單據
        let new_status = if req.decision == "approve_appeal" {
            "cancelled" // 暫緩成功，取消安樂死
        } else {
            "approved" // 駁回暫緩，可以執行安樂死
        };

        sqlx::query(
            r#"
            UPDATE euthanasia_orders
            SET status = ($1::TEXT)::euthanasia_order_status, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(new_status)
        .bind(updated.order_id)
        .execute(pool)
        .await?;

        Ok(updated)
    }

    /// 執行安樂死
    pub async fn execute(
        pool: &PgPool,
        order_id: Uuid,
        executor_id: Uuid,
    ) -> Result<EuthanasiaOrder, AppError> {
        // 驗證單據狀態為可執行
        let order = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at
            FROM euthanasia_orders
            WHERE id = $1 AND status = 'approved'
            "#,
        )
        .bind(order_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到可執行的安樂死單據".to_string()))?;

        // 更新安樂死單據為已執行
        let updated = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = 'executed', executed_at = NOW(), executed_by = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at
            "#,
        )
        .bind(executor_id)
        .bind(order_id)
        .fetch_one(pool)
        .await?;

        // 更新動物狀態為 Euthanized（安樂死）
        sqlx::query(
            r#"
            UPDATE animals SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(AnimalStatus::Euthanized as AnimalStatus)
        .bind(order.animal_id)
        .execute(pool)
        .await?;

        // 自動建立空犧牲紀錄供執行者填寫
        sqlx::query(
            r#"
            INSERT INTO animal_sacrifices (
                animal_id, sacrifice_date, zoletil_dose,
                method_electrocution, method_bloodletting,
                confirmed_sacrifice, created_by, created_at, updated_at
            )
            VALUES ($1, CURRENT_DATE, NULL, false, false, false, $2, NOW(), NOW())
            ON CONFLICT (animal_id) DO NOTHING
            "#
        )
        .bind(order.animal_id)
        .bind(executor_id)
        .execute(pool)
        .await?;

        Ok(updated)
    }

    /// 檢查並處理超時的安樂死單據（供排程器呼叫）
    pub async fn check_expired_orders(pool: &PgPool) -> Result<i32, AppError> {
        let now = Utc::now();
        let mut count = 0;

        // 處理 PI 超時未回應的單據
        let expired_pending = sqlx::query_as::<_, ExpiredOrderRow>(
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved', updated_at = NOW()
            WHERE status = 'pending_pi' AND deadline_at < $1
            RETURNING id, vet_user_id
            "#,
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        count += expired_pending.len() as i32;

        // 通知獸醫可以執行
        let notification_service = NotificationService::new(pool.clone());
        for order in expired_pending {
            if let Err(e) = notification_service
                .notify_euthanasia_timeout_approved(order.id, order.vet_user_id)
                .await {
                tracing::warn!("發送安樂死通知失敗: {e}");
            }
        }

        // 處理 CHAIR 超時未裁決的暫緩申請
        let expired_appeals = sqlx::query_as::<_, ExpiredAppealRow>(
            r#"
            SELECT ea.id, ea.order_id, eo.vet_user_id
            FROM euthanasia_appeals ea
            JOIN euthanasia_orders eo ON ea.order_id = eo.id
            WHERE ea.chair_decision IS NULL 
              AND ea.chair_deadline_at < $1
              AND eo.status = 'chair_arbitration'
            "#,
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        for appeal in expired_appeals {
            // 駁回暫緩，更新為可執行
            sqlx::query(
                r#"
                UPDATE euthanasia_appeals
                SET chair_decision = 'timeout_rejected', chair_decided_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(appeal.id)
            .execute(pool)
            .await?;

            sqlx::query(
                r#"
                UPDATE euthanasia_orders
                SET status = 'approved', updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(appeal.order_id)
            .execute(pool)
            .await?;

            // 通知獸醫
            if let Err(e) = notification_service
                .notify_euthanasia_timeout_approved(appeal.order_id, appeal.vet_user_id)
                .await {
                tracing::warn!("發送安樂死通知失敗: {e}");
            }

            count += 1;
        }

        Ok(count)
    }
}
