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
        DECISION_APPROVE_APPEAL, DECISION_REJECT_APPEAL,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService, NotificationService, OutboxService, SignatureService, SignatureType,
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

        let pi_user_id = animal_record
            .pi_user_id
            .filter(|u| !u.is_nil())
            .ok_or_else(|| {
                AppError::BadRequest("該動物尚未關聯至任何計畫，無法開立安樂死單".to_string())
            })?;

        let deadline_at = Utc::now() + Duration::hours(24);

        let mut tx = pool.begin().await?;

        let order = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            INSERT INTO euthanasia_orders (animal_id, vet_user_id, pi_user_id, reason, deadline_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status,
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

        // 生效中的 PI 代理人也要收到通知（migration 010）。**在 commit 前的同一個 tx 內
        // 取**，拿到的才是與這張單同一個時間點的授權狀態。
        //
        // join 路徑與 `lock_order_for_pi` 的授權判準逐字相同——收得到通知的人，
        // 正好就是按得下核准的人。用 `fetch_all` 而非 `fetch_optional`：
        // 部分唯一索引只保證「一份計畫一位生效代理人」，`iacuc_no` 對到多份計畫時
        // 仍可能解出多位，漏掉任何一位都是漏通知。
        let delegate_user_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT d.delegate_user_id
            FROM protocol_pi_delegates d
            JOIN protocols pr ON pr.id = d.protocol_id
            JOIN animals a ON a.iacuc_no = pr.iacuc_no
            JOIN euthanasia_orders eo ON eo.animal_id = a.id
            WHERE eo.id = $1 AND d.revoked_at IS NULL AND d.delegate_user_id <> $2
            "#,
        )
        .bind(order.id)
        .bind(pi_user_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        // 通知 PI 與生效中代理人（commit 後 fire-and-forget）
        let order_id = order.id;
        let reason = req.reason.clone();
        let ear_tag = animal_record.ear_tag.clone();
        let iacuc_no = animal_record.iacuc_no.clone();
        let pool_clone = pool.clone();
        let actor_clone = actor.clone();
        tokio::spawn(async move {
            let notification_service = NotificationService::new(pool_clone.clone());
            // 一位收件人失敗不影響其餘：每位各自記一筆失敗稽核，不 early-return。
            for recipient in std::iter::once(pi_user_id).chain(delegate_user_ids) {
                if let Err(e) = notification_service
                    .notify_euthanasia_order(
                        order_id,
                        &ear_tag,
                        iacuc_no.as_deref(),
                        &reason,
                        recipient,
                    )
                    .await
                {
                    tracing::error!("發送安樂死通知失敗（收件人 {recipient}）: {e}");
                    Self::log_notification_failure(
                        &pool_clone,
                        &actor_clone,
                        order_id,
                        &e.to_string(),
                    )
                    .await;
                }
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
                eo.status,
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

    /// 取得 PI 的待處理安樂死單據。
    ///
    /// 「PI」含**生效中的 PI 代理人**（`protocol_pi_delegates`，migration 010）：
    /// 代理人既然被 `lock_order_for_pi` 允許核准/暫緩，就必須看得到單子，否則等於
    /// 給了一個行使不了的權利——這張單有 24 小時期限且逾時自動核准
    /// （`check_expired_orders`），看不到就只能眼睜睜等它過期。
    ///
    /// 代理人條件與 `lock_order_for_pi` 的授權判準**必須保持一致**（同一條
    /// `animals.iacuc_no = protocols.iacuc_no` 路徑、同樣要求 `revoked_at IS NULL`）：
    /// 看得到的集合與按得下去的集合不一致，就會變成列表有單但一點就 404，
    /// 或反過來有權卻查不到。改一邊要記得改另一邊。
    pub async fn get_pending_orders_for_pi(
        pool: &PgPool,
        pi_user_id: Uuid,
    ) -> Result<Vec<EuthanasiaOrderResponse>, AppError> {
        let orders = sqlx::query_as::<_, EuthanasiaOrderResponse>(
            r#"
            SELECT
                eo.id, eo.animal_id, eo.vet_user_id, eo.pi_user_id, eo.reason,
                eo.status,
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
            WHERE eo.status = 'pending_pi'
              AND (
                eo.pi_user_id = $1
                OR EXISTS (
                  SELECT 1
                  FROM protocol_pi_delegates d
                  JOIN protocols pr ON pr.id = d.protocol_id
                  WHERE pr.iacuc_no = p.iacuc_no
                    AND d.delegate_user_id = $1
                    AND d.revoked_at IS NULL
                )
              )
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
        let acting_user_id = user.id;

        let mut tx = pool.begin().await?;

        let (before, delegation_id) =
            Self::lock_order_for_pi(&mut tx, order_id, acting_user_id).await?;
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
                      status,
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

        // 簽章 — PI 批准必須簽（或持生效中代理授權的代理人代簽，見 lock_order_for_pi）
        let content = format!("euthanasia_pi_approve:{order_id}");
        if let Some(delegation_id) = delegation_id {
            SignatureService::sign_record_delegated_tx(
                &mut tx,
                pool,
                actor,
                ORDER_ENTITY_TYPE,
                &order_id.to_string(),
                acting_user_id,
                delegation_id,
                SignatureType::Approve,
                &content,
                req.password.as_deref(),
                req.handwriting_svg.as_deref(),
                req.stroke_data.as_ref(),
            )
            .await?;
        } else {
            SignatureService::sign_record_tx(
                &mut tx,
                pool,
                actor,
                ORDER_ENTITY_TYPE,
                &order_id.to_string(),
                acting_user_id,
                SignatureType::Approve,
                &content,
                req.password.as_deref(),
                req.handwriting_svg.as_deref(),
                req.stroke_data.as_ref(),
            )
            .await?;
        }

        tx.commit().await?;

        // 通知獸醫（commit 後 fire-and-forget）
        Self::spawn_notify(
            pool,
            actor,
            order_id,
            after.vet_user_id,
            NotifyKind::Approved,
        );

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
        // 代理人申請暫緩時，`euthanasia_appeals.pi_user_id` 記的是實際操作者
        // （代理人自己），與 `euthanasia_orders.pi_user_id`（借位值/真 PI）不必然
        // 相同——這欄本來就是「誰申請的」而非另一張授權快照，語意不變。
        let acting_user_id = user.id;

        let mut tx = pool.begin().await?;

        let (before, _delegation_id) =
            Self::lock_order_for_pi(&mut tx, order_id, acting_user_id).await?;
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
        .bind(acting_user_id)
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
                      status,
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

        let display = format!("申訴: {}", req.reason);
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
                   status,
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

        // 決定 order 終態 — fail-closed 白名單：未知 decision 一律拒絕，
        // 絕不預設走「approved（可執行安樂死）」路徑（Critical / #262 fail-open）。
        // 放在任何 UPDATE 之前，非法值不寫入 appeal / order。
        let new_status = match req.decision.as_str() {
            DECISION_APPROVE_APPEAL => "cancelled", // 暫緩成功，取消安樂死
            DECISION_REJECT_APPEAL => "approved",   // 駁回暫緩，可以執行安樂死
            _ => {
                return Err(AppError::BadRequest(
                    "無效的仲裁決定（須為 approve_appeal 或 reject_appeal）".to_string(),
                ))
            }
        };

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

        // UPDATE order — 根據裁決結果改終態（new_status 已於上方 fail-closed 決定）
        let order_after = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            UPDATE euthanasia_orders
            SET status = ($2::TEXT)::euthanasia_order_status,
                updated_at = NOW(),
                version = version + 1
            WHERE id = $1
            RETURNING id, animal_id, vet_user_id, pi_user_id, reason,
                      status,
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
        let content = format!("euthanasia_chair_decide:{appeal_id}:{}", req.decision);
        SignatureService::sign_record_tx(
            &mut tx,
            pool,
            actor,
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
                   status,
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
                      status,
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
            actor,
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
    /// R30-3b：每筆超時改用 [`approve_timeout_order_tx`] / [`approve_timeout_appeal_tx`]
    /// service fn，把 UPDATE + audit + in-app notification + email outbox 包進同一 tx，
    /// 達成 all-or-nothing。本 cron 函式收斂為 thin loop（拓 SELECT + iterate）。
    /// 詳見 `docs/dev/notification-and-outbox.md`。
    ///
    /// `now` 由 caller 一次取定後 propagate 到 SELECT + per-row UPDATE，避免 app 時鐘
    /// 與 DB `NOW()` 漂移造成 SELECT 選中但 UPDATE WHERE 失敗的浪費（gemini PR #306）。
    pub async fn check_expired_orders(pool: &PgPool) -> Result<i32, AppError> {
        let actor = ActorContext::System {
            reason: "euthanasia_timeout_scheduler",
        };
        let now = Utc::now();
        let mut count = 0;

        // PI 超時未回應的單據 candidates（read-only SELECT）
        // gemini PR #307 medium：LIMIT 100 防超時單據暴量導致排程執行時間過長
        // 與下一輪重疊；剩餘 candidates 由下一輪 cron tick（5min）處理
        let candidates = sqlx::query_as::<_, ExpiredOrderRow>(
            "SELECT id, vet_user_id FROM euthanasia_orders \
             WHERE status = 'pending_pi' AND deadline_at < $1 \
             ORDER BY deadline_at LIMIT 100",
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        for order in &candidates {
            match Self::approve_timeout_order_tx(pool, &actor, order.id, order.vet_user_id, now)
                .await
            {
                Ok(true) => count += 1,
                Ok(false) => {} // race: 已被另一個 worker 處理，noop
                Err(e) => tracing::error!(
                    order_id = %order.id,
                    error = %e,
                    "approve_timeout_order_tx failed; will retry next tick"
                ),
            }
        }

        // CHAIR 超時未裁決的暫緩申請 candidates（同上：LIMIT 100 + 排序，剩餘下輪處理）
        let appeal_candidates = sqlx::query_as::<_, ExpiredAppealRow>(
            r#"
            SELECT ea.id, ea.order_id, eo.vet_user_id
            FROM euthanasia_appeals ea
            JOIN euthanasia_orders eo ON ea.order_id = eo.id
            WHERE ea.chair_decision IS NULL
              AND ea.chair_deadline_at < $1
              AND eo.status = 'chair_arbitration'
            ORDER BY ea.chair_deadline_at LIMIT 100
            "#,
        )
        .bind(now)
        .fetch_all(pool)
        .await?;

        for appeal in &appeal_candidates {
            match Self::approve_timeout_appeal_tx(pool, &actor, appeal, now).await {
                Ok(true) => count += 1,
                Ok(false) => {} // race
                Err(e) => tracing::error!(
                    appeal_id = %appeal.id,
                    order_id = %appeal.order_id,
                    error = %e,
                    "approve_timeout_appeal_tx failed; will retry next tick"
                ),
            }
        }

        Ok(count)
    }

    /// R30-3b：原子處理一筆 PI 超時 order — UPDATE + audit + in-app notification +
    /// email outbox 全在同一 tx。回傳 true = 成功處理；false = race（被別 worker 搶）。
    ///
    /// 與 [`check_expired_orders`] 拆開讓邏輯可獨立測試 + 未來其他 caller（admin
    /// 手動 timeout、batch CLI 等）可複用。
    async fn approve_timeout_order_tx(
        pool: &PgPool,
        actor: &ActorContext,
        order_id: Uuid,
        vet_user_id: Uuid,
        now: chrono::DateTime<Utc>,
    ) -> Result<bool, AppError> {
        let mut tx = pool.begin().await?;

        // CAS UPDATE：confirm 仍在 pending_pi 且已過期，否則 race → noop
        // 用 caller 傳入的 now 取代 DB NOW() 確保與 SELECT candidates 使用同一時間戳
        let updated: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE euthanasia_orders
            SET status = 'approved', updated_at = NOW(), version = version + 1
            WHERE id = $1 AND status = 'pending_pi' AND deadline_at < $2
            RETURNING version
            "#,
        )
        .bind(order_id)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await?;

        if updated.is_none() {
            tx.rollback().await?;
            return Ok(false);
        }

        let audit_display = format!("PI 超時未回應，自動核准 (order_id={})", order_id);
        let email_body = format!(
            "因 PI 超時未回應，order_id={} 已自動核准。請登入系統完成執行。",
            order_id
        );
        finalize_timeout_approval_tx(
            &mut tx,
            actor,
            TimeoutApprovalArgs {
                entity_type: ORDER_ENTITY_TYPE,
                entity_id: order_id,
                order_id,
                vet_user_id,
                audit_display: &audit_display,
                email_body: &email_body,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(true)
    }

    /// R30-3b：原子處理一筆 CHAIR 超時 appeal — 同 [`approve_timeout_order_tx`] 模式。
    async fn approve_timeout_appeal_tx(
        pool: &PgPool,
        actor: &ActorContext,
        appeal: &ExpiredAppealRow,
        now: chrono::DateTime<Utc>,
    ) -> Result<bool, AppError> {
        let mut tx = pool.begin().await?;

        // CAS UPDATE appeal：confirm 仍未裁決且已過期
        let appeal_updated: Option<(i32,)> = sqlx::query_as(
            r#"
            UPDATE euthanasia_appeals
            SET chair_decision = 'timeout_rejected',
                chair_decided_at = NOW(),
                version = version + 1
            WHERE id = $1 AND chair_decision IS NULL AND chair_deadline_at < $2
            RETURNING version
            "#,
        )
        .bind(appeal.id)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await?;

        if appeal_updated.is_none() {
            tx.rollback().await?;
            return Ok(false);
        }

        // 連帶把 order 從 chair_arbitration 推回 approved
        // gemini PR #306 high：必須驗證 rows_affected 否則 appeal/order 狀態會 drift
        let order_update = sqlx::query(
            "UPDATE euthanasia_orders \
             SET status = 'approved', updated_at = NOW(), version = version + 1 \
             WHERE id = $1 AND status = 'chair_arbitration'",
        )
        .bind(appeal.order_id)
        .execute(&mut *tx)
        .await?;

        if order_update.rows_affected() != 1 {
            tx.rollback().await?;
            tracing::error!(
                order_id = %appeal.order_id,
                appeal_id = %appeal.id,
                rows_affected = order_update.rows_affected(),
                "approve_timeout_appeal_tx: order UPDATE expected 1 row but got {}, rolled back to avoid appeal/order drift",
                order_update.rows_affected()
            );
            return Err(AppError::Internal(format!(
                "appeal {} order {} update affected {} rows (expected 1, status may have drifted)",
                appeal.id,
                appeal.order_id,
                order_update.rows_affected()
            )));
        }

        let audit_display = format!(
            "CHAIR 超時未裁決，自動駁回暫緩 (order_id={}, appeal_id={})",
            appeal.order_id, appeal.id
        );
        let email_body = format!(
            "因 CHAIR 超時未裁決暫緩申請，order_id={} 已自動核准。請登入系統完成執行。",
            appeal.order_id
        );
        finalize_timeout_approval_tx(
            &mut tx,
            actor,
            TimeoutApprovalArgs {
                entity_type: APPEAL_ENTITY_TYPE,
                entity_id: appeal.id,
                order_id: appeal.order_id,
                vet_user_id: appeal.vet_user_id,
                audit_display: &audit_display,
                email_body: &email_body,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(true)
    }

    // ============================================================
    // Helpers
    // ============================================================

    /// FOR UPDATE 鎖 order，並驗證 PI 身分。
    /// 鎖定並回傳這張安樂死單，同時解出「操作者是 PI 本人還是代理人」——回傳的
    /// `Option<Uuid>` 非 NULL 時是 `protocol_pi_delegates.id`，供呼叫端綁進
    /// `sign_record_delegated_tx`（PI 本人操作則為 `None`）。
    ///
    /// ⚠️ 這仍是本服務唯一的 IDOR 防線（handler 層不重驗，見 `handlers/euthanasia.rs`
    /// 的 `approve_order`/`appeal_order`）：對不上 `pi_user_id` 本人、也對不上
    /// 任何生效中代理人時一律 `NotFound`（不是 `Forbidden`，避免洩漏單據存在與否）。
    ///
    /// `euthanasia_orders` 本身沒有 `protocol_id` 欄位，代理人比對需經
    /// `animals.iacuc_no = protocols.iacuc_no` 解到計畫（與 `access.rs` 的
    /// `get_animal_protocol_id` 同一個 join 路徑）。`FOR UPDATE OF eo` 只鎖單據列，
    /// 不連帶鎖 animals/protocols/protocol_pi_delegates。
    async fn lock_order_for_pi(
        tx: &mut Transaction<'_, Postgres>,
        order_id: Uuid,
        acting_user_id: Uuid,
    ) -> Result<(EuthanasiaOrder, Option<Uuid>), AppError> {
        let direct = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status,
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at, version
            FROM euthanasia_orders
            WHERE id = $1 AND pi_user_id = $2
            FOR UPDATE
            "#,
        )
        .bind(order_id)
        .bind(acting_user_id)
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(order) = direct {
            return Ok((order, None));
        }

        // sqlx 的 tuple FromRow 是逐欄 Decode，不支援「巢狀 FromRow 結構 + 一個純量欄」
        // 混在同一個 tuple 裡解——分兩句查：先確認代理資格（順帶拿 delegation id），
        // 通過才對單據本身下 FOR UPDATE。
        let delegation_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT d.id
            FROM protocol_pi_delegates d
            JOIN protocols pr ON pr.id = d.protocol_id
            JOIN animals a ON a.iacuc_no = pr.iacuc_no
            JOIN euthanasia_orders eo ON eo.animal_id = a.id
            WHERE eo.id = $1 AND d.delegate_user_id = $2 AND d.revoked_at IS NULL
            "#,
        )
        .bind(order_id)
        .bind(acting_user_id)
        .fetch_optional(&mut **tx)
        .await?;

        let Some(delegation_id) = delegation_id else {
            return Err(AppError::NotFound("找不到指定的安樂死單據".to_string()));
        };

        let order = sqlx::query_as::<_, EuthanasiaOrder>(
            r#"
            SELECT id, animal_id, vet_user_id, pi_user_id, reason,
                   status,
                   deadline_at, pi_responded_at, executed_at, executed_by,
                   created_at, updated_at, version
            FROM euthanasia_orders
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(order_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok((order, Some(delegation_id)))
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

/// R30-3b: 「超時自動核准」共用後置處理參數封裝。
///
/// CLAUDE.md §2「函數參數數量 ≤ 5 個（超過封裝為 struct）」— 8 參數 → 6 個業務欄位
/// 進 struct，主 fn 只剩 `tx` / `actor` / `args`。
struct TimeoutApprovalArgs<'a> {
    /// audit entity_type — `ORDER_ENTITY_TYPE` / `APPEAL_ENTITY_TYPE`
    entity_type: &'a str,
    /// audit entity_id — order_id 或 appeal_id（依 entity_type 對應）
    entity_id: Uuid,
    /// in-app notification + email outbox 的 source — 永遠是 order_id
    order_id: Uuid,
    /// 通知收件人 user
    vet_user_id: Uuid,
    /// audit display 文字
    audit_display: &'a str,
    /// email plain body 文字（caller 已 i18n）
    email_body: &'a str,
}

/// R30-3b: 「超時自動核准」共用後置處理 — audit + in-app notification + email outbox。
///
/// 抽出讓 [`approve_timeout_order_tx`] / [`approve_timeout_appeal_tx`] 共用，
/// 避免重複 + 讓兩主 fn ≤50 行（CLAUDE.md §2）。
async fn finalize_timeout_approval_tx(
    tx: &mut Transaction<'_, Postgres>,
    actor: &ActorContext,
    args: TimeoutApprovalArgs<'_>,
) -> Result<(), AppError> {
    AuditService::log_activity_tx(
        tx,
        actor,
        ActivityLogEntry {
            event_category: "ANIMAL",
            event_type: EVT_ORDER_TIMEOUT,
            entity: Some(AuditEntity::new(
                args.entity_type,
                args.entity_id,
                args.audit_display,
            )),
            data_diff: None,
            request_context: None,
        },
    )
    .await?;

    NotificationService::notify_euthanasia_timeout_approved_tx(tx, args.order_id, args.vet_user_id)
        .await?;

    enqueue_timeout_email_tx(
        tx,
        actor,
        args.vet_user_id,
        args.order_id,
        "[超時核准] 安樂死執行權限已解鎖",
        args.email_body,
    )
    .await?;

    Ok(())
}

/// R30-3b: 把超時通知 email 排進 outbox（同 tx）。
///
/// 查 vet user 的 email 後組 payload；email 缺失（或 user 已停用 / 軟刪除）
/// 時 log warn 但**不中斷 tx**（通知是 best-effort 的延伸，業務 mutation
/// 已成功）。送達由 outbox worker 接手 retry 5 次（exp backoff）+
/// DEAD-letter alert。
///
/// gemini PR #307 medium：query 加 `is_active = true AND deleted_at IS NULL`
/// 守衛，避免發給已停用 / 已刪除的使用者產生無效 outbox 任務。
async fn enqueue_timeout_email_tx(
    tx: &mut Transaction<'_, Postgres>,
    actor: &ActorContext,
    vet_user_id: Uuid,
    order_id: Uuid,
    subject: &str,
    plain_body: &str,
) -> Result<(), AppError> {
    let vet_email: Option<String> = sqlx::query_scalar(
        "SELECT email FROM users \
         WHERE id = $1 AND email IS NOT NULL AND is_active = true AND deleted_at IS NULL",
    )
    .bind(vet_user_id)
    .fetch_optional(&mut **tx)
    .await?;

    let Some(email) = vet_email else {
        tracing::warn!(
            vet_user_id = %vet_user_id,
            order_id = %order_id,
            "outbox: skip euthanasia timeout email — vet has no email on record (or user inactive/deleted)"
        );
        return Ok(());
    };

    // gemini PR #306 security-medium：plain_body 進 HTML 前必須 escape 防 XSS
    // utils::html_escape::html_escape_minimal — 通用 helper，其他 outbox caller 都應用此
    let html_body = format!(
        "<p>{}</p><p>order_id: <code>{}</code></p>",
        crate::utils::html_escape::html_escape_minimal(plain_body),
        order_id
    );
    let payload = serde_json::json!({
        "to": email,
        "subject": subject,
        "plain_body": plain_body,
        "html_body": html_body,
    });

    OutboxService::enqueue_tx(tx, actor, "email", payload, (ORDER_ENTITY_TYPE, order_id)).await?;
    Ok(())
}
