//! 統一通知派送層（P0 骨架）。
//!
//! 目標：**零寫死收件人**。handler / service 只「發出事件 + 提供實體上下文」，
//! 收件人與管道全部由 `notification_routing` 表決定：
//! - `target_kind='role'`  → 持有該角色者（`get_users_by_role`）。
//! - `target_kind='resolver'` → 關係型解析器（見 [`super::resolvers`]）。
//!
//! 站內 in-app 通知一律建立（不受時間窗 / 請假影響）。
//! email 受 routing channel + 個人偏好（`notification_settings`）兩層 AND 控制——
//! **P0 階段僅打通架構、不主動寄 email**（對齊決策：先維持 channel 現狀），
//! email 模板串接於後續 Phase 進行；此處遇 email channel 僅記錄、不送。

use std::collections::HashMap;

use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::ActorContext;
use crate::models::{CreateNotificationRequest, NotificationRouting, NotificationType};
use crate::services::EmailService;

use super::dispatch::StaffEmail;
use super::{resolvers, NotificationService};

/// 解析關係型收件人所需的實體上下文。
///
/// handler 只填入相關實體 ID 與觸發者，**不決定收件人是誰**。
/// 欄位隨 resolver 需求逐步擴充（P0 僅含計畫關係與觸發者）。
#[derive(Debug, Clone, Default)]
pub struct EventContext {
    /// 觸發者 user id：用於排除「通知自己」，並供 actor 型 resolver 使用。
    pub actor_id: Option<Uuid>,
    /// 關聯計畫 id：供 `protocol_pi_sd` / `assigned_reviewers` 等 resolver 使用。
    pub protocol_id: Option<Uuid>,
    /// 直接指定的「當事人」user id（如假單 / 加班申請人本人）：供 `event_subject` resolver 使用。
    pub subject_user_id: Option<Uuid>,
    /// 通用實體 id（如 leave_request id）：供查詢型 resolver（`leave_request_approvers`）使用。
    pub entity_id: Option<Uuid>,
}

/// 一則待派送通知的內容（站內通知用；email 模板於後續 Phase 串接）。
pub struct NotificationPayload {
    pub notification_type: NotificationType,
    pub title: String,
    pub content: Option<String>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
}

/// 收件人聚合後的有效管道（同一 user 因多條 rule 取聯集）。
#[derive(Default, Clone, Copy)]
struct RecipientChannel {
    in_app: bool,
    email: bool,
}

impl RecipientChannel {
    fn merge(&mut self, channel: &str) {
        if channel == "in_app" || channel == "both" {
            self.in_app = true;
        }
        if channel == "email" || channel == "both" {
            self.email = true;
        }
    }
}

impl NotificationService {
    /// 統一派送一個事件：依 `notification_routing` 解析收件人（角色 or resolver）與管道，
    /// 站內通知一律建立為一般通知（`kind='info'`）。回傳建立的站內通知數。
    ///
    /// email：P0 不主動寄（見模組註解）；channel 含 email/both 時僅記錄待後續串接。
    pub async fn dispatch_event(
        &self,
        event_type: &str,
        ctx: &EventContext,
        payload: NotificationPayload,
    ) -> Result<i32, AppError> {
        self.dispatch_event_inner(event_type, ctx, payload, false, None)
            .await
    }

    /// [`Self::dispatch_event`] 的置頂待辦版本：站內通知建立為
    /// `kind='action', priority>0`（進「待處理」清單），而非一般通知。
    /// email 派送邏輯不變（P0 仍不主動寄）。
    ///
    /// ⚠️ **呼叫端必須自行在對應的終態轉換裡呼叫
    /// [`Self::resolve_pinned_notifications_tx`] 解除**，否則待辦會永久卡住
    /// （待辦依設計不可手動已讀，見 `crud.rs` 的說明）。這一點與單一收件人的
    /// `create_pinned_notification_tx` 用法相同，差別只在這裡的收件人是
    /// 路由/resolver 動態算出的一批人，不是呼叫端已知的單一 user_id。
    /// `recipient_role`：見 [`crate::models::RECIPIENT_ROLE_PROXY`] /
    /// [`crate::models::RECIPIENT_ROLE_APPROVER`]。目前僅 `leave_submitted` 事件使用
    /// （標記這批動態解析出的收件人是「核准人」身分），其餘事件傳 `None`。
    pub async fn dispatch_pinned_event(
        &self,
        event_type: &str,
        ctx: &EventContext,
        payload: NotificationPayload,
        recipient_role: Option<&'static str>,
    ) -> Result<i32, AppError> {
        self.dispatch_event_inner(event_type, ctx, payload, true, recipient_role)
            .await
    }

    async fn dispatch_event_inner(
        &self,
        event_type: &str,
        ctx: &EventContext,
        payload: NotificationPayload,
        pinned: bool,
        recipient_role: Option<&'static str>,
    ) -> Result<i32, AppError> {
        let rules = self.load_active_routing_rules(event_type).await?;

        // 收件人去重：user_id → 有效管道聯集。
        let mut targets: HashMap<Uuid, RecipientChannel> = HashMap::new();
        for rule in &rules {
            for uid in self.resolve_rule_recipients(rule, ctx).await? {
                if ctx.actor_id == Some(uid) {
                    continue; // 不通知觸發者本人
                }
                targets.entry(uid).or_default().merge(&rule.channel);
            }
        }

        let mut count = 0;
        for (uid, ch) in targets {
            if ch.in_app {
                let request = CreateNotificationRequest {
                    user_id: uid,
                    notification_type: payload.notification_type.clone(),
                    title: payload.title.clone(),
                    content: payload.content.clone(),
                    related_entity_type: payload.related_entity_type.clone(),
                    related_entity_id: payload.related_entity_id,
                };
                // 單一收件人失敗不阻斷其他（對齊既有 notify_* 的 warn-and-continue 行為）。
                let created = if pinned {
                    self.create_pinned_notification_with_role(request, recipient_role)
                        .await
                } else {
                    self.create_notification(request).await
                };
                if let Err(e) = created {
                    tracing::warn!("[dispatch_event] 建立站內通知失敗 user={uid}: {e}");
                } else {
                    count += 1;
                }
            }
            if ch.email {
                // email channel：經個人偏好（notification_settings）兩層 AND + 時間窗 / 請假
                // chokepoint（dispatch_staff_email）+ outbox。app_url 未初始化（測試/bin）則跳過。
                self.dispatch_routed_email(uid, event_type, &payload).await;
            }
        }
        Ok(count)
    }

    /// 對單一收件人寄出「路由通知 email」（通用樣板）。
    /// 失敗 / 條件不符（無 app_url、個人偏好關閉、查無聯絡資料）皆靜默跳過，不阻斷其他收件人。
    async fn dispatch_routed_email(
        &self,
        user_id: Uuid,
        event_type: &str,
        payload: &NotificationPayload,
    ) {
        let Some(app_url) = super::app_url() else {
            return;
        };
        if !self.email_pref_allows(user_id, event_type).await {
            return;
        }
        let contact = match self.find_active_user_contact(user_id).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(user_id = %user_id, error = %e, "[dispatch_event] 查詢收件人聯絡資料失敗，略過 email");
                return;
            }
        };
        let Some((email, name)) = contact else {
            return;
        };
        let rendered = EmailService::render_generic_notification_email(
            &app_url,
            &name,
            &payload.title,
            payload.content.as_deref().unwrap_or(""),
        );
        let actor = ActorContext::System {
            reason: "routed_notification",
        };
        let source_id = payload.related_entity_id.unwrap_or_else(Uuid::nil);
        let source_type = payload
            .related_entity_type
            .as_deref()
            .unwrap_or("notification");
        if let Err(e) = self
            .dispatch_staff_email(
                &actor,
                (source_type, source_id),
                StaffEmail {
                    to_email: email,
                    to_name: name,
                    recipient_user_id: Some(user_id),
                    email: rendered,
                },
            )
            .await
        {
            tracing::warn!("[dispatch_event] 分派通知 email 失敗 user={user_id}: {e}");
        }
    }

    /// 個人偏好（notification_settings）對該事件是否允許 email（兩層 AND 的個人層）。
    /// 有對應偏好欄位則尊重之；無對應 → 預設允許（admin 啟用 email channel 即寄）。
    /// **DB 查詢失敗 → fail-closed 不寄**（避免暫時性錯誤把信寄給明確關閉的使用者）。
    async fn email_pref_allows(&self, user_id: Uuid, event_type: &str) -> bool {
        // 每個對應偏好欄位用靜態 SQL（不字串拼接，符合禁止拼接 SQL 規則）。
        let result =
            match event_type {
                "low_stock_alert" => {
                    sqlx::query_scalar::<_, bool>(
                        "SELECT email_low_stock FROM notification_settings WHERE user_id = $1",
                    )
                    .bind(user_id)
                    .fetch_optional(&self.db)
                    .await
                }
                "expiry_alert" => {
                    sqlx::query_scalar::<_, bool>(
                        "SELECT email_expiry_warning FROM notification_settings WHERE user_id = $1",
                    )
                    .bind(user_id)
                    .fetch_optional(&self.db)
                    .await
                }
                "document_submitted" => sqlx::query_scalar::<_, bool>(
                    "SELECT email_document_approval FROM notification_settings WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_optional(&self.db)
                .await,
                "protocol_submitted"
                | "protocol_vet_review"
                | "protocol_under_review"
                | "protocol_resubmitted"
                | "protocol_approved"
                | "protocol_rejected" => sqlx::query_scalar::<_, bool>(
                    "SELECT email_protocol_status FROM notification_settings WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_optional(&self.db)
                .await,
                _ => return true, // 無對應個人偏好 → admin 啟用 email channel 即寄
            };
        match result {
            Ok(Some(enabled)) => enabled,
            Ok(None) => true, // 無 settings 列 → 預設寄
            Err(e) => {
                // DB 失敗 → fail-closed 不寄（避免暫時性錯誤把信寄給明確關閉者）。
                tracing::warn!(
                    user_id = %user_id,
                    event_type,
                    error = %e,
                    "[dispatch_event] 讀取 email 偏好失敗，fail-closed 不寄"
                );
                false
            }
        }
    }

    /// 載入某事件所有 active 路由規則。
    async fn load_active_routing_rules(
        &self,
        event_type: &str,
    ) -> Result<Vec<NotificationRouting>, AppError> {
        let rules: Vec<NotificationRouting> = sqlx::query_as(
            r#"
            SELECT id, event_type, role_code, channel, is_active, description,
                   frequency, hour_of_day, day_of_week, created_at, updated_at,
                   target_kind, target_value
            FROM notification_routing
            WHERE event_type = $1 AND is_active = true
            "#,
        )
        .bind(event_type)
        .fetch_all(&self.db)
        .await?;
        Ok(rules)
    }

    /// 依 rule 的目標種類解析出 user_id 清單。
    async fn resolve_rule_recipients(
        &self,
        rule: &NotificationRouting,
        ctx: &EventContext,
    ) -> Result<Vec<Uuid>, AppError> {
        match rule.target_kind.as_str() {
            "role" => Ok(self
                .get_users_by_role(&rule.target_value)
                .await?
                .into_iter()
                .map(|(id, _, _)| id)
                .collect()),
            "resolver" => resolvers::resolve(self, &rule.target_value, ctx).await,
            other => {
                tracing::warn!(
                    target_kind = other,
                    "[dispatch_event] 未知 target_kind，略過此 rule"
                );
                Ok(vec![])
            }
        }
    }
}
