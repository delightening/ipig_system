use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        ChairDecisionRequest, CreateEuthanasiaAppealRequest, CreateEuthanasiaOrderRequest,
        EuthanasiaAppeal, EuthanasiaAppealResponse, EuthanasiaOrder, EuthanasiaOrderResponse,
        EuthanasiaOrderStatus, PigStatus,
    },
    services::NotificationService,
};

pub struct EuthanasiaService;

impl EuthanasiaService {
    /// 建立安樂死單據 (獸醫開立)
    pub async fn create_order(
        pool: &PgPool,
        req: &CreateEuthanasiaOrderRequest,
        vet_user_id: Uuid,
    ) -> Result<EuthanasiaOrder, AppError> {
        // 查詢豬隻的關聯 PI
        let pig = sqlx::query!(
            r#"
            SELECT p.id, p.ear_tag, p.iacuc_no, pr.pi_user_id as "pi_user_id?"
            FROM pigs p
            LEFT JOIN protocols pr ON p.iacuc_no = pr.iacuc_no
            WHERE p.id = $1 AND p.is_deleted = false
            "#,
            req.pig_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到指定的豬隻".to_string()))?;

        let pi_user_id = pig.pi_user_id.ok_or_else(|| {
            AppError::BadRequest("該豬隻尚未關聯至任何計畫，無法開立安樂死單".to_string())
        })?;

        // 設定 24 小時後過期
        let deadline_at = Utc::now() + Duration::hours(24);

        let order = sqlx::query_as!(
            EuthanasiaOrder,
            r#"
            INSERT INTO euthanasia_orders (pig_id, vet_user_id, pi_user_id, reason, deadline_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, pig_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at
            "#,
            req.pig_id,
            vet_user_id,
            pi_user_id,
            req.reason,
            deadline_at
        )
        .fetch_one(pool)
        .await?;

        // 發送通知給 PI
        let notification_service = NotificationService::new(pool.clone());
        let _ = notification_service
            .notify_euthanasia_order(
                order.id,
                &pig.ear_tag,
                pig.iacuc_no.as_deref(),
                &req.reason,
                pi_user_id,
            )
            .await;

        Ok(order)
    }

    /// 取得安樂死單據詳情
    pub async fn get_order_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<EuthanasiaOrderResponse, AppError> {
        let order = sqlx::query_as!(
            EuthanasiaOrderResponse,
            r#"
            SELECT 
                eo.id, eo.pig_id, eo.vet_user_id, eo.pi_user_id, eo.reason,
                eo.status as "status: EuthanasiaOrderStatus",
                eo.deadline_at, eo.pi_responded_at, eo.executed_at, eo.executed_by,
                eo.created_at, eo.updated_at,
                p.ear_tag as pig_ear_tag,
                p.iacuc_no as pig_iacuc_no,
                uv.display_name as vet_name,
                up.display_name as pi_name
            FROM euthanasia_orders eo
            JOIN pigs p ON eo.pig_id = p.id
            JOIN users uv ON eo.vet_user_id = uv.id
            JOIN users up ON eo.pi_user_id = up.id
            WHERE eo.id = $1
            "#,
            id
        )
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
        let orders = sqlx::query_as!(
            EuthanasiaOrderResponse,
            r#"
            SELECT 
                eo.id, eo.pig_id, eo.vet_user_id, eo.pi_user_id, eo.reason,
                eo.status as "status: EuthanasiaOrderStatus",
                eo.deadline_at, eo.pi_responded_at, eo.executed_at, eo.executed_by,
                eo.created_at, eo.updated_at,
                p.ear_tag as pig_ear_tag,
                p.iacuc_no as pig_iacuc_no,
                uv.display_name as vet_name,
                up.display_name as pi_name
            FROM euthanasia_orders eo
            JOIN pigs p ON eo.pig_id = p.id
            JOIN users uv ON eo.vet_user_id = uv.id
            JOIN users up ON eo.pi_user_id = up.id
            WHERE eo.pi_user_id = $1 AND eo.status = 'pending_pi'
            ORDER BY eo.deadline_at ASC
            "#,
            pi_user_id
        )
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
        let order = sqlx::query_as!(
            EuthanasiaOrder,
            r#"
            SELECT id, pig_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at
            FROM euthanasia_orders
            WHERE id = $1 AND pi_user_id = $2 AND status = 'pending_pi'
            "#,
            order_id,
            pi_user_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到待處理的安樂死單據".to_string()))?;

        // 更新狀態為已同意
        let updated = sqlx::query_as!(
            EuthanasiaOrder,
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved', pi_responded_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING id, pig_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at
            "#,
            order_id
        )
        .fetch_one(pool)
        .await?;

        // 通知獸醫可以執行
        let notification_service = NotificationService::new(pool.clone());
        let _ = notification_service
            .notify_euthanasia_approved(order_id, order.vet_user_id)
            .await;

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
        let _order = sqlx::query!(
            r#"
            SELECT id FROM euthanasia_orders
            WHERE id = $1 AND pi_user_id = $2 AND status = 'pending_pi'
            "#,
            order_id,
            pi_user_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到待處理的安樂死單據".to_string()))?;

        // 查找 CHAIR 用戶
        let chair = sqlx::query!(
            r#"
            SELECT u.id
            FROM users u
            JOIN user_roles ur ON u.id = ur.user_id
            JOIN roles r ON ur.role_id = r.id
            WHERE r.code = 'IACUC_CHAIR' AND u.is_active = true
            LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await?;

        let chair_user_id = chair.map(|c| c.id);
        let chair_deadline = Utc::now() + Duration::hours(24);

        // 建立暫緩申請
        let appeal = sqlx::query_as!(
            EuthanasiaAppeal,
            r#"
            INSERT INTO euthanasia_appeals (order_id, pi_user_id, reason, attachment_path, chair_user_id, chair_deadline_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                      chair_decision, chair_decided_at, chair_deadline_at, created_at
            "#,
            order_id,
            pi_user_id,
            req.reason,
            req.attachment_path,
            chair_user_id,
            chair_deadline
        )
        .fetch_one(pool)
        .await?;

        // 更新安樂死單據狀態
        sqlx::query!(
            r#"
            UPDATE euthanasia_orders
            SET status = 'appealed', pi_responded_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            order_id
        )
        .execute(pool)
        .await?;

        // 如果有 CHAIR，通知進行仲裁
        if let Some(chair_id) = chair_user_id {
            // 更新為 CHAIR 仲裁中
            sqlx::query!(
                r#"
                UPDATE euthanasia_orders
                SET status = 'chair_arbitration', updated_at = NOW()
                WHERE id = $1
                "#,
                order_id
            )
            .execute(pool)
            .await?;

            let notification_service = NotificationService::new(pool.clone());
            let _ = notification_service
                .notify_euthanasia_appeal(appeal.id, order_id, chair_id, &req.reason)
                .await;
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
        let appeal = sqlx::query_as!(
            EuthanasiaAppeal,
            r#"
            SELECT id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                   chair_decision, chair_decided_at, chair_deadline_at, created_at
            FROM euthanasia_appeals
            WHERE id = $1 AND chair_user_id = $2 AND chair_decision IS NULL
            "#,
            appeal_id,
            chair_user_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到待裁決的暫緩申請".to_string()))?;

        // 更新裁決結果
        let updated = sqlx::query_as!(
            EuthanasiaAppeal,
            r#"
            UPDATE euthanasia_appeals
            SET chair_decision = $1, chair_decided_at = NOW()
            WHERE id = $2
            RETURNING id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                      chair_decision, chair_decided_at, chair_deadline_at, created_at
            "#,
            req.decision,
            appeal_id
        )
        .fetch_one(pool)
        .await?;

        // 根據裁決結果更新安樂死單據
        let new_status = if req.decision == "approve_appeal" {
            "cancelled" // 暫緩成功，取消安樂死
        } else {
            "approved" // 駁回暫緩，可以執行安樂死
        };

        sqlx::query!(
            r#"
            UPDATE euthanasia_orders
            SET status = ($1::TEXT)::euthanasia_order_status, updated_at = NOW()
            WHERE id = $2
            "#,
            new_status,
            appeal.order_id
        )
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
        let order = sqlx::query_as!(
            EuthanasiaOrder,
            r#"
            SELECT id, pig_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at
            FROM euthanasia_orders
            WHERE id = $1 AND status = 'approved'
            "#,
            order_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到可執行的安樂死單據".to_string()))?;

        // 更新安樂死單據為已執行
        let updated = sqlx::query_as!(
            EuthanasiaOrder,
            r#"
            UPDATE euthanasia_orders
            SET status = 'executed', executed_at = NOW(), executed_by = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, pig_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at
            "#,
            executor_id,
            order_id
        )
        .fetch_one(pool)
        .await?;

        // 更新豬隻狀態為 Deceased
        sqlx::query!(
            r#"
            UPDATE pigs SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            PigStatus::Deceased as PigStatus,
            order.pig_id
        )
        .execute(pool)
        .await?;

        Ok(updated)
    }

    /// 檢查並處理超時的安樂死單據（供排程器呼叫）
    pub async fn check_expired_orders(pool: &PgPool) -> Result<i32, AppError> {
        let now = Utc::now();
        let mut count = 0;

        // 處理 PI 超時未回應的單據
        let expired_pending = sqlx::query!(
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved', updated_at = NOW()
            WHERE status = 'pending_pi' AND deadline_at < $1
            RETURNING id, vet_user_id
            "#,
            now
        )
        .fetch_all(pool)
        .await?;

        count += expired_pending.len() as i32;

        // 通知獸醫可以執行
        let notification_service = NotificationService::new(pool.clone());
        for order in expired_pending {
            let _ = notification_service
                .notify_euthanasia_timeout_approved(order.id, order.vet_user_id)
                .await;
        }

        // 處理 CHAIR 超時未裁決的暫緩申請
        let expired_appeals = sqlx::query!(
            r#"
            SELECT ea.id, ea.order_id, eo.vet_user_id
            FROM euthanasia_appeals ea
            JOIN euthanasia_orders eo ON ea.order_id = eo.id
            WHERE ea.chair_decision IS NULL 
              AND ea.chair_deadline_at < $1
              AND eo.status = 'chair_arbitration'
            "#,
            now
        )
        .fetch_all(pool)
        .await?;

        for appeal in expired_appeals {
            // 駁回暫緩，更新為可執行
            sqlx::query!(
                r#"
                UPDATE euthanasia_appeals
                SET chair_decision = 'timeout_rejected', chair_decided_at = NOW()
                WHERE id = $1
                "#,
                appeal.id
            )
            .execute(pool)
            .await?;

            sqlx::query!(
                r#"
                UPDATE euthanasia_orders
                SET status = 'approved', updated_at = NOW()
                WHERE id = $1
                "#,
                appeal.order_id
            )
            .execute(pool)
            .await?;

            // 通知獸醫
            let _ = notification_service
                .notify_euthanasia_timeout_approved(appeal.order_id, appeal.vet_user_id)
                .await;

            count += 1;
        }

        Ok(count)
    }
}
