use chrono::{Duration, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::ActorContext,
    models::{
        audit_diff::DataDiff, AnimalStatus, ChairDecisionRequest, CreateEuthanasiaAppealRequest,
        CreateEuthanasiaOrderRequest, EuthanasiaAppeal, EuthanasiaOrder, EuthanasiaOrderResponse,
        EuthanasiaOrderStatus, ExecuteEuthanasiaRequest, PiApproveEuthanasiaRequest,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, NotificationService, SignatureService, SignatureType,
    },
};

// ============================================================
// Constants — entity_type / event_type / decision values
// ============================================================

const ORDER_ENTITY_TYPE: &str = "euthanasia_order";
const APPEAL_ENTITY_TYPE: &str = "euthanasia_appeal";

const EVT_ORDER_CREATED: &str = "EuthanasiaOrderCreated";
const EVT_ORDER_APPROVED: &str = "EuthanasiaOrderApproved";
const EVT_ORDER_APPEALED: &str = "EuthanasiaOrderAppealed";
const EVT_CHAIR_DECIDED: &str = "EuthanasiaChairDecided";
const EVT_ORDER_EXECUTED: &str = "EuthanasiaOrderExecuted";
const EVT_NOTIFY_FAILED: &str = "EuthanasiaNotificationFailed";
const EVT_ORDER_TIMEOUT: &str = "EuthanasiaOrderTimeout";

const DECISION_APPROVE_APPEAL: &str = "approve_appeal";

const CONFLICT_MSG: &str = "此記錄已被其他人修改，請重新載入後再試。";

/// 輔助結構：查詢動物關聯 PI 資訊
#[derive(FromRow)]
struct AnimalPiRecord {
    #[sqlx(rename = "id")]
    _id: Uuid,
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
    /// 建立安樂死單據（獸醫開立）
    ///
    /// R30-A：
    /// - 單 INSERT 原子性，無需顯式 tx；audit 與 INSERT 用同一 tx 包起來保證原子。
    /// - 通知改為 commit 後 fire-and-forget；失敗時 tracing::error! + 寫一筆
    ///   `EuthanasiaNotificationFailed` audit（獨立 tx，業務不受影響）。
    /// - 不需簽章（建單階段，後續 pi_approve / execute 才簽）。
    pub async fn create_order(
        pool: &PgPool,
        actor: &ActorContext,
        req: &CreateEuthanasiaOrderRequest,
    ) -> Result<EuthanasiaOrder, AppError> {
        let user = actor.require_user()?;
        let vet_user_id = user.id;

        // 查詢動物的關聯 PI（tx 外，read-only）
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

        let deadline_at = Utc::now() + Duration::hours(24);

        let mut tx = pool.begin().await?;

        let order = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            INSERT INTO euthanasia_orders (animal_id, vet_user_id, pi_user_id, reason, deadline_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at, version
            "#,
        )
        .bind(req.animal_id)
        .bind(vet_user_id)
        .bind(pi_user_id)
        .bind(&req.reason)
        .bind(deadline_at)
        .fetch_one(&mut *tx)
        .await?;

        let display = format!(
            "{} / {}",
            animal_record.ear_tag,
            animal_record.iacuc_no.as_deref().unwrap_or("-")
        );
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: EVT_ORDER_CREATED,
                entity: Some(AuditEntity::new(ORDER_ENTITY_TYPE, order.id, &display)),
                data_diff: Some(DataDiff::create_only(&order)),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 通知 PI（commit 後 fire-and-forget）
        let order_id = order.id;
        let reason = req.reason.clone();
        let ear_tag = animal_record.ear_tag.clone();
        let iacuc_no = animal_record.iacuc_no.clone();
        let pool_clone = pool.clone();
        let actor_clone = actor.clone();
        tokio::spawn(async move {
            let notification_service = NotificationService::new(pool_clone.clone());
            if let Err(e) = notification_service
                .notify_euthanasia_order(
                    order_id,
                    &ear_tag,
                    iacuc_no.as_deref(),
                    &reason,
                    pi_user_id,
                )
                .await
            {
                tracing::error!("發送安樂死通知失敗: {e}");
                Self::log_notification_failure(&pool_clone, &actor_clone, order_id, &e.to_string())
                    .await;
            }
        });

        Ok(order)
    }

    /// 紀錄通知失敗事件（獨立 tx；不影響業務）
    async fn log_notification_failure(
        pool: &PgPool,
        actor: &ActorContext,
        order_id: Uuid,
        error_msg: &str,
    ) {
        let display = format!("通知失敗: {error_msg}");
        if let Err(e) = AuditService::log_activity_oneshot(
            pool,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: EVT_NOTIFY_FAILED,
                entity: Some(AuditEntity::new(ORDER_ENTITY_TYPE, order_id, &display)),
                data_diff: None,
                request_context: None,
            },
        )
        .await
        {
            tracing::error!("寫入 EuthanasiaNotificationFailed audit 失敗: {e}");
        }
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
                eo.created_at, eo.updated_at, eo.version,
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
                eo.created_at, eo.updated_at, eo.version,
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
    ///
    /// R30-A：tx + FOR UPDATE + version optimistic lock + audit + sign_record_tx。
    /// 全 tx 原子；簽章失敗整 tx rollback，不留簽章孤兒。
    pub async fn pi_approve(
        pool: &PgPool,
        actor: &ActorContext,
        order_id: Uuid,
        req: &PiApproveEuthanasiaRequest,
    ) -> Result<EuthanasiaOrder, AppError> {
        let user = actor.require_user()?;
        let pi_user_id = user.id;

        let mut tx = pool.begin().await?;

        let before = Self::lock_order_for_pi(&mut tx, order_id, pi_user_id).await?;
        if before.status != EuthanasiaOrderStatus::PendingPi {
            return Err(AppError::BadRequest(format!(
                "單據狀態為「{}」，不可執行此操作",
                before.status.display_name()
            )));
        }

        // version optimistic lock + 一次 UPDATE 到 approved
        let after = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved',
                pi_responded_at = NOW(),
                updated_at = NOW(),
                version = version + 1
            WHERE id = $1
              AND status = 'pending_pi'
              AND ($2::INT IS NULL OR version = $2)
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at, version
            "#,
        )
        .bind(order_id)
        .bind(req.version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict(CONFLICT_MSG.to_string()))?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: EVT_ORDER_APPROVED,
                entity: Some(AuditEntity::new(
                    ORDER_ENTITY_TYPE,
                    order_id,
                    &order_id.to_string(),
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        // 簽章 — PI 批准必須簽
        let content = format!("euthanasia_pi_approve:{order_id}");
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            ORDER_ENTITY_TYPE,
            &order_id.to_string(),
            pi_user_id,
            SignatureType::Approve,
            &content,
            req.password.as_deref(),
            req.handwriting_svg.as_deref(),
            req.stroke_data.as_ref(),
        )
        .await?;

        tx.commit().await?;

        // 通知獸醫（commit 後 fire-and-forget）
        Self::spawn_notify(pool, actor, order_id, after.vet_user_id, NotifyKind::Approved);

        Ok(after)
    }

    /// PI 申請暫緩
    ///
    /// R30-A 設計決策 D2：取消「先 appealed → 再 chair_arbitration」中間態，
    /// 改為「無 chair → appealed」「有 chair → chair_arbitration」一次到位，
    /// 消除 race window。整 fn 包一個 tx；不需簽章（PI 申訴不是非否認性節點，
    /// chair_decide 才是）。
    pub async fn pi_appeal(
        pool: &PgPool,
        actor: &ActorContext,
        order_id: Uuid,
        req: &CreateEuthanasiaAppealRequest,
    ) -> Result<EuthanasiaAppeal, AppError> {
        let user = actor.require_user()?;
        let pi_user_id = user.id;

        let mut tx = pool.begin().await?;

        let before = Self::lock_order_for_pi(&mut tx, order_id, pi_user_id).await?;
        if before.status != EuthanasiaOrderStatus::PendingPi {
            return Err(AppError::BadRequest(format!(
                "單據狀態為「{}」，不可申請暫緩",
                before.status.display_name()
            )));
        }

        // 查找 CHAIR 用戶（在 tx 內，但 read-only — 與業務 row 無關）
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
        .fetch_optional(&mut *tx)
        .await?;

        let chair_user_id = chair.map(|c| c.0);
        let chair_deadline = Utc::now() + Duration::hours(24);

        // 建立暫緩申請
        let appeal = sqlx::query_as::<_, EuthanasiaAppeal>(
            r#"
            INSERT INTO euthanasia_appeals (
                order_id, pi_user_id, reason, attachment_path, chair_user_id, chair_deadline_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                      chair_decision, chair_decided_at, chair_deadline_at, created_at, version
            "#,
        )
        .bind(order_id)
        .bind(pi_user_id)
        .bind(&req.reason)
        .bind(&req.attachment_path)
        .bind(chair_user_id)
        .bind(chair_deadline)
        .fetch_one(&mut *tx)
        .await?;

        // D2：一次 UPDATE 到正確終態 — 有 chair → chair_arbitration，否則 → appealed
        // 同時 version optimistic lock。
        let target_status = if chair_user_id.is_some() {
            "chair_arbitration"
        } else {
            "appealed"
        };
        let after = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = ($2::TEXT)::euthanasia_order_status,
                pi_responded_at = NOW(),
                updated_at = NOW(),
                version = version + 1
            WHERE id = $1
              AND status = 'pending_pi'
              AND ($3::INT IS NULL OR version = $3)
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at, version
            "#,
        )
        .bind(order_id)
        .bind(target_status)
        .bind(req.version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict(CONFLICT_MSG.to_string()))?;

        let display = format!("申訴: {}", &req.reason);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: EVT_ORDER_APPEALED,
                entity: Some(AuditEntity::new(ORDER_ENTITY_TYPE, order_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 通知 CHAIR（commit 後 fire-and-forget）
        if let Some(chair_id) = chair_user_id {
            let appeal_id = appeal.id;
            let reason = req.reason.clone();
            let pool_clone = pool.clone();
            let actor_clone = actor.clone();
            tokio::spawn(async move {
                let svc = NotificationService::new(pool_clone.clone());
                if let Err(e) = svc
                    .notify_euthanasia_appeal(appeal_id, order_id, chair_id, &reason)
                    .await
                {
                    tracing::error!("發送暫緩通知失敗: {e}");
                    Self::log_notification_failure(
                        &pool_clone,
                        &actor_clone,
                        order_id,
                        &e.to_string(),
                    )
                    .await;
                }
            });
        }

        Ok(appeal)
    }

    /// CHAIR 裁決
    ///
    /// R30-A：tx + FOR UPDATE + version optimistic lock + audit + sign_record_tx。
    pub async fn chair_decide(
        pool: &PgPool,
        actor: &ActorContext,
        appeal_id: Uuid,
        req: &ChairDecisionRequest,
    ) -> Result<EuthanasiaAppeal, AppError> {
        let user = actor.require_user()?;
        let chair_user_id = user.id;

        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖 appeal row
        let appeal_before: EuthanasiaAppeal = sqlx::query_as::<_, EuthanasiaAppeal>(
            r#"
            SELECT id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                   chair_decision, chair_decided_at, chair_deadline_at, created_at, version
            FROM euthanasia_appeals
            WHERE id = $1 AND chair_user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(appeal_id)
        .bind(chair_user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到待裁決的暫緩申請".to_string()))?;

        if appeal_before.chair_decision.is_some() {
            return Err(AppError::Conflict("此暫緩申請已被裁決".to_string()));
        }

        // 同 tx 內鎖 order row（後面要 UPDATE order.status）
        let order_id = appeal_before.order_id;
        let order_before: EuthanasiaOrder = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at, version
            FROM euthanasia_orders
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(order_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到對應的安樂死單據".to_string()))?;

        // UPDATE appeal — version optimistic lock
        let appeal_after = sqlx::query_as::<_, EuthanasiaAppeal>(
            r#"
            UPDATE euthanasia_appeals
            SET chair_decision = $1,
                chair_decided_at = NOW(),
                version = version + 1
            WHERE id = $2
              AND chair_decision IS NULL
              AND ($3::INT IS NULL OR version = $3)
            RETURNING id, order_id, pi_user_id, reason, attachment_path, chair_user_id,
                      chair_decision, chair_decided_at, chair_deadline_at, created_at, version
            "#,
        )
        .bind(&req.decision)
        .bind(appeal_id)
        .bind(req.version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict(CONFLICT_MSG.to_string()))?;

        // UPDATE order — 根據裁決結果改終態
        let new_status = if req.decision == DECISION_APPROVE_APPEAL {
            "cancelled" // 暫緩成功，取消安樂死
        } else {
            "approved" // 駁回暫緩，可以執行安樂死
        };

        let order_after = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = ($2::TEXT)::euthanasia_order_status,
                updated_at = NOW(),
                version = version + 1
            WHERE id = $1
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at, version
            "#,
        )
        .bind(order_id)
        .bind(new_status)
        .fetch_one(&mut *tx)
        .await?;

        // Audit — 1 筆 chair decided（覆蓋 appeal + order 兩個變更）
        let display = format!("仲裁決定: {} → order={}", req.decision, new_status);
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: EVT_CHAIR_DECIDED,
                entity: Some(AuditEntity::new(APPEAL_ENTITY_TYPE, appeal_id, &display)),
                data_diff: Some(DataDiff::compute(Some(&order_before), Some(&order_after))),
                request_context: None,
            },
        )
        .await?;

        // 簽章 — chair 決定為 21 CFR §11 非否認性終決
        let content = format!(
            "euthanasia_chair_decide:{appeal_id}:{}",
            req.decision
        );
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            APPEAL_ENTITY_TYPE,
            &appeal_id.to_string(),
            chair_user_id,
            SignatureType::Approve,
            &content,
            req.password.as_deref(),
            req.handwriting_svg.as_deref(),
            req.stroke_data.as_ref(),
        )
        .await?;

        tx.commit().await?;

        Ok(appeal_after)
    }

    /// 執行安樂死
    ///
    /// R30-A：tx + FOR UPDATE + version + audit + sign_record_tx + animal status
    /// update + sacrifice insert 全部同 tx。執行為不可逆操作，必須簽章。
    pub async fn execute(
        pool: &PgPool,
        actor: &ActorContext,
        order_id: Uuid,
        req: &ExecuteEuthanasiaRequest,
    ) -> Result<EuthanasiaOrder, AppError> {
        let user = actor.require_user()?;
        let executor_id = user.id;

        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖 order
        let before: EuthanasiaOrder = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at, version
            FROM euthanasia_orders
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(order_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到安樂死單據".to_string()))?;

        if before.status != EuthanasiaOrderStatus::Approved {
            return Err(AppError::BadRequest(format!(
                "單據狀態為「{}」，不可執行",
                before.status.display_name()
            )));
        }

        // UPDATE order — version optimistic lock
        let after = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = 'executed',
                executed_at = NOW(),
                executed_by = $1,
                updated_at = NOW(),
                version = version + 1
            WHERE id = $2
              AND status = 'approved'
              AND ($3::INT IS NULL OR version = $3)
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status as "status: EuthanasiaOrderStatus",
                      deadline_at, pi_responded_at, executed_at, executed_by,
                      created_at, updated_at, version
            "#,
        )
        .bind(executor_id)
        .bind(order_id)
        .bind(req.version)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Conflict(CONFLICT_MSG.to_string()))?;

        // 動物狀態 → Euthanized；移出欄位
        sqlx::query(
            r#"
            UPDATE animals
            SET status = $1, pen_location = NULL, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(AnimalStatus::Euthanized as AnimalStatus)
        .bind(before.animal_id)
        .execute(&mut *tx)
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
            "#,
        )
        .bind(before.animal_id)
        .bind(executor_id)
        .execute(&mut *tx)
        .await?;

        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ANIMAL",
                event_type: EVT_ORDER_EXECUTED,
                entity: Some(AuditEntity::new(
                    ORDER_ENTITY_TYPE,
                    order_id,
                    &order_id.to_string(),
                )),
                data_diff: Some(DataDiff::compute(Some(&before), Some(&after))),
                request_context: None,
            },
        )
        .await?;

        let content = format!("euthanasia_execute:{order_id}");
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            ORDER_ENTITY_TYPE,
            &order_id.to_string(),
            executor_id,
            SignatureType::Confirm,
            &content,
            req.password.as_deref(),
            req.handwriting_svg.as_deref(),
            req.stroke_data.as_ref(),
        )
        .await?;

        tx.commit().await?;

        Ok(after)
    }

    /// 檢查並處理超時的安樂死單據（供排程器呼叫）
    ///
    /// R30-A：scheduler 呼叫，actor 為 System；不需簽章。
    /// 補上每筆超時自動轉態的 audit log（pool-level，逐筆獨立 tx）。
    pub async fn check_expired_orders(pool: &PgPool) -> Result<i32, AppError> {
        let actor = ActorContext::System {
            reason: "euthanasia_timeout_scheduler",
        };
        let now = Utc::now();
        let mut count = 0;

        // PI 超時未回應的單據 → approved
        let expired_pending = sqlx::query_as::<_, ExpiredOrderRow>(
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved', updated_at = NOW(), version = version + 1
            WHERE status = 'pending_pi' AND deadline_at < $1
            RETURNING id, vet_user_id
            "#,
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        count += expired_pending.len() as i32;

        let notification_service = NotificationService::new(pool.clone());
        for order in &expired_pending {
            let display = format!("PI 超時未回應，自動核准 (order_id={})", order.id);
            if let Err(e) = AuditService::log_activity_oneshot(
                pool,
                &actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: EVT_ORDER_TIMEOUT,
                    entity: Some(AuditEntity::new(ORDER_ENTITY_TYPE, order.id, &display)),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await
            {
                tracing::error!("寫入 EuthanasiaOrderTimeout audit 失敗: {e}");
            }

            if let Err(e) = notification_service
                .notify_euthanasia_timeout_approved(order.id, order.vet_user_id)
                .await
            {
                tracing::warn!("發送安樂死通知失敗: {e}");
            }
        }

        // CHAIR 超時未裁決的暫緩申請
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
            sqlx::query(
                r#"
                UPDATE euthanasia_appeals
                SET chair_decision = 'timeout_rejected',
                    chair_decided_at = NOW(),
                    version = version + 1
                WHERE id = $1
                "#,
            )
            .bind(appeal.id)
            .execute(pool)
            .await?;

            sqlx::query(
                r#"
                UPDATE euthanasia_orders
                SET status = 'approved',
                    updated_at = NOW(),
                    version = version + 1
                WHERE id = $1
                "#,
            )
            .bind(appeal.order_id)
            .execute(pool)
            .await?;

            let display = format!(
                "CHAIR 超時未裁決，自動駁回暫緩 (order_id={}, appeal_id={})",
                appeal.order_id, appeal.id
            );
            if let Err(e) = AuditService::log_activity_oneshot(
                pool,
                &actor,
                ActivityLogEntry {
                    event_category: "ANIMAL",
                    event_type: EVT_ORDER_TIMEOUT,
                    entity: Some(AuditEntity::new(APPEAL_ENTITY_TYPE, appeal.id, &display)),
                    data_diff: None,
                    request_context: None,
                },
            )
            .await
            {
                tracing::error!("寫入 EuthanasiaOrderTimeout audit 失敗: {e}");
            }

            if let Err(e) = notification_service
                .notify_euthanasia_timeout_approved(appeal.order_id, appeal.vet_user_id)
                .await
            {
                tracing::warn!("發送安樂死通知失敗: {e}");
            }

            count += 1;
        }

        Ok(count)
    }

    // ============================================================
    // Helpers
    // ============================================================

    /// FOR UPDATE 鎖 order，並驗證 PI 身分。
    async fn lock_order_for_pi(
        tx: &mut Transaction<'_, Postgres>,
        order_id: Uuid,
        pi_user_id: Uuid,
    ) -> Result<EuthanasiaOrder, AppError> {
        sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status as "status: EuthanasiaOrderStatus",
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at, version
            FROM euthanasia_orders
            WHERE id = $1 AND pi_user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(order_id)
        .bind(pi_user_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("找不到指定的安樂死單據".to_string()))
    }

    fn spawn_notify(
        pool: &PgPool,
        actor: &ActorContext,
        order_id: Uuid,
        vet_user_id: Uuid,
        kind: NotifyKind,
    ) {
        let pool_clone = pool.clone();
        let actor_clone = actor.clone();
        tokio::spawn(async move {
            let svc = NotificationService::new(pool_clone.clone());
            let res = match kind {
                NotifyKind::Approved => svc.notify_euthanasia_approved(order_id, vet_user_id).await,
            };
            if let Err(e) = res {
                tracing::error!("發送安樂死通知失敗: {e}");
                Self::log_notification_failure(&pool_clone, &actor_clone, order_id, &e.to_string())
                    .await;
            }
        });
    }
}

enum NotifyKind {
    Approved,
}
