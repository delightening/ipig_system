// 通知路由規則 CRUD 操作

use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateNotificationRoutingRequest, EventTypeCategory, EventTypeInfo, FixedNotification,
        NotificationRouting, RecipientMember, RoleInfo, RoutingRecipientsPreview,
        RoutingRuleRecipients, UpdateNotificationRoutingRequest,
    },
};

use super::{resolvers, NotificationService};

impl NotificationService {
    /// 列出所有通知路由規則
    pub async fn list_notification_routing(&self) -> Result<Vec<NotificationRouting>, AppError> {
        let rules: Vec<NotificationRouting> = sqlx::query_as(
            r#"
            SELECT id, event_type, role_code, channel, is_active, description,
                   frequency, hour_of_day, day_of_week,
                   created_at, updated_at, target_kind, target_value
            FROM notification_routing
            ORDER BY event_type, target_kind, target_value
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rules)
    }

    /// 建立通知路由規則
    pub async fn create_notification_routing(
        &self,
        request: CreateNotificationRoutingRequest,
    ) -> Result<NotificationRouting, AppError> {
        let channel = request.channel.unwrap_or_else(|| "in_app".to_string());
        let frequency = request.frequency.unwrap_or_else(|| "immediate".to_string());
        let hour_of_day = request.hour_of_day.unwrap_or(8);

        Self::validate_channel(&channel)?;
        Self::validate_frequency(&frequency, request.day_of_week)?;

        let role_exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM roles WHERE code = $1")
            .bind(&request.role_code)
            .fetch_optional(&self.db)
            .await?;

        if role_exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "角色代碼 '{}' 不存在",
                request.role_code
            )));
        }

        // 此 API 建立的皆為「角色型」收件人列（target_kind='role'，target_value=role_code）。
        // resolver 型列由 seed / 後續專用 API 建立。
        let rule: NotificationRouting = sqlx::query_as(
            r#"
            INSERT INTO notification_routing
                (event_type, role_code, channel, description, frequency, hour_of_day, day_of_week,
                 target_kind, target_value)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'role', $2)
            RETURNING *
            "#,
        )
        .bind(&request.event_type)
        .bind(&request.role_code)
        .bind(&channel)
        .bind(&request.description)
        .bind(&frequency)
        .bind(hour_of_day)
        .bind(request.day_of_week)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::BadRequest(format!(
                    "事件 '{}' 與角色 '{}' 的路由規則已存在",
                    request.event_type, request.role_code
                ))
            } else {
                AppError::from(e)
            }
        })?;

        tracing::info!(
            "[NotificationRouting] 新增路由規則: {} → {} ({}, {})",
            request.event_type,
            request.role_code,
            channel,
            frequency
        );

        Ok(rule)
    }

    /// 更新通知路由規則
    pub async fn update_notification_routing(
        &self,
        id: Uuid,
        request: UpdateNotificationRoutingRequest,
    ) -> Result<NotificationRouting, AppError> {
        if let Some(ref channel) = request.channel {
            Self::validate_channel(channel)?;
        }
        if let Some(ref frequency) = request.frequency {
            Self::validate_frequency(frequency, request.day_of_week)?;
        }

        let rule: NotificationRouting = sqlx::query_as(
            r#"
            UPDATE notification_routing
            SET channel     = COALESCE($2, channel),
                is_active   = COALESCE($3, is_active),
                description = COALESCE($4, description),
                frequency   = COALESCE($5, frequency),
                hour_of_day = COALESCE($6, hour_of_day),
                day_of_week = CASE WHEN $5 IS NOT NULL THEN $7 ELSE day_of_week END,
                updated_at  = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&request.channel)
        .bind(request.is_active)
        .bind(&request.description)
        .bind(&request.frequency)
        .bind(request.hour_of_day)
        .bind(request.day_of_week)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("通知路由規則不存在".to_string()))?;

        tracing::info!(
            "[NotificationRouting] 更新路由規則 {}: {} → {} ({})",
            id,
            rule.event_type,
            rule.target_value,
            rule.target_kind
        );

        Ok(rule)
    }

    /// 刪除通知路由規則
    pub async fn delete_notification_routing(&self, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM notification_routing WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("通知路由規則不存在".to_string()));
        }

        tracing::info!("[NotificationRouting] 刪除路由規則 {}", id);
        Ok(())
    }

    /// 取得所有可用的事件類型（含分類與中文名稱）
    /// 分組：AUP | Animal | ERP | HR
    pub fn list_available_event_types() -> Vec<EventTypeCategory> {
        vec![
            EventTypeCategory {
                group: "AUP".to_string(),
                category: "AUP 計畫審查".to_string(),
                event_types: vec![
                    EventTypeInfo {
                        code: "protocol_submitted".to_string(),
                        name: "計畫提交".to_string(),
                    },
                    EventTypeInfo {
                        code: "protocol_vet_review".to_string(),
                        name: "獸醫審查".to_string(),
                    },
                    EventTypeInfo {
                        code: "protocol_under_review".to_string(),
                        name: "委員審查".to_string(),
                    },
                    EventTypeInfo {
                        code: "protocol_resubmitted".to_string(),
                        name: "重新提交".to_string(),
                    },
                    EventTypeInfo {
                        code: "protocol_approved".to_string(),
                        name: "計畫核准".to_string(),
                    },
                    EventTypeInfo {
                        code: "protocol_rejected".to_string(),
                        name: "計畫駁回".to_string(),
                    },
                    EventTypeInfo {
                        code: "review_comment_created".to_string(),
                        name: "新審查意見".to_string(),
                    },
                    EventTypeInfo {
                        code: "all_reviews_completed".to_string(),
                        name: "所有審查意見送出".to_string(),
                    },
                    EventTypeInfo {
                        code: "all_comments_resolved".to_string(),
                        name: "所有意見已解決".to_string(),
                    },
                ],
            },
            EventTypeCategory {
                group: "AUP".to_string(),
                category: "修正案".to_string(),
                event_types: vec![
                    EventTypeInfo {
                        code: "amendment_submitted".to_string(),
                        name: "修正案提交".to_string(),
                    },
                    EventTypeInfo {
                        code: "amendment_decision_recorded".to_string(),
                        name: "修正案審查決定".to_string(),
                    },
                    EventTypeInfo {
                        code: "amendment_approved".to_string(),
                        name: "修正案核准".to_string(),
                    },
                    EventTypeInfo {
                        code: "amendment_rejected".to_string(),
                        name: "修正案駁回".to_string(),
                    },
                ],
            },
            EventTypeCategory {
                group: "Animal".to_string(),
                category: "動物健康".to_string(),
                event_types: vec![
                    EventTypeInfo {
                        code: "emergency_medication".to_string(),
                        name: "緊急給藥".to_string(),
                    },
                    EventTypeInfo {
                        code: "animal_abnormal_record".to_string(),
                        name: "動物異常紀錄".to_string(),
                    },
                    EventTypeInfo {
                        code: "animal_sudden_death".to_string(),
                        name: "動物猝死".to_string(),
                    },
                    EventTypeInfo {
                        code: "euthanasia_order_created".to_string(),
                        name: "安樂死申請".to_string(),
                    },
                ],
            },
            EventTypeCategory {
                group: "ERP".to_string(),
                category: "ERP 進銷存".to_string(),
                event_types: vec![
                    EventTypeInfo {
                        code: "document_submitted".to_string(),
                        name: "採購單提交".to_string(),
                    },
                    EventTypeInfo {
                        code: "po_pending_receipt".to_string(),
                        name: "採購單未入庫提醒".to_string(),
                    },
                    EventTypeInfo {
                        code: "low_stock_alert".to_string(),
                        name: "低庫存預警".to_string(),
                    },
                    EventTypeInfo {
                        code: "expiry_alert".to_string(),
                        name: "效期預警".to_string(),
                    },
                ],
            },
            EventTypeCategory {
                group: "HR".to_string(),
                category: "HR 人事".to_string(),
                event_types: vec![
                    EventTypeInfo {
                        code: "leave_submitted".to_string(),
                        name: "請假申請".to_string(),
                    },
                    EventTypeInfo {
                        code: "overtime_submitted".to_string(),
                        name: "加班申請".to_string(),
                    },
                    EventTypeInfo {
                        code: "leave_approved".to_string(),
                        name: "請假核准".to_string(),
                    },
                    EventTypeInfo {
                        code: "leave_cancelled".to_string(),
                        name: "請假取消".to_string(),
                    },
                    EventTypeInfo {
                        code: "overtime_approved".to_string(),
                        name: "加班核准".to_string(),
                    },
                ],
            },
            EventTypeCategory {
                group: "Equipment".to_string(),
                category: "設備管理".to_string(),
                event_types: vec![
                    EventTypeInfo {
                        code: "equipment_overdue".to_string(),
                        name: "設備校正/確效逾期提醒".to_string(),
                    },
                    EventTypeInfo {
                        code: "equipment_unrepairable".to_string(),
                        name: "設備無法維修通知".to_string(),
                    },
                    EventTypeInfo {
                        code: "equipment_maintenance_review".to_string(),
                        name: "維修/保養待驗收通知".to_string(),
                    },
                    EventTypeInfo {
                        code: "equipment_disposal".to_string(),
                        name: "設備報廢申請通知".to_string(),
                    },
                ],
            },
        ]
    }

    /// 「固定通知」目錄：收件人由系統流程決定、不經 notification_routing、不可調整。
    /// 供 admin 路由頁顯示為唯讀 notes（讓管理者看見有什麼狀況會通知誰）。
    ///
    /// 新增固定通知路徑時請同步更新本目錄，維持路由頁透明完整。
    pub fn list_fixed_notifications() -> Vec<FixedNotification> {
        let f = |event_label: &str,
                 trigger: &str,
                 recipient: &str,
                 channel: &str,
                 group: &str,
                 note: &str| FixedNotification {
            event_label: event_label.to_string(),
            trigger: trigger.to_string(),
            recipient_label: recipient.to_string(),
            channel: channel.to_string(),
            group: group.to_string(),
            note: note.to_string(),
        };
        vec![
            f(
                "安樂死單開立",
                "獸醫開立安樂死單",
                "該單所屬計畫的 PI",
                "in_app",
                "Animal",
                "24 小時審核流程；收件人由該單決定，不可調整",
            ),
            f(
                "安樂死 — PI 同意執行",
                "PI 同意執行安樂死",
                "該單開立的獸醫",
                "in_app",
                "Animal",
                "流程決定，不可調整",
            ),
            f(
                "安樂死 — 暫緩申請",
                "PI 申請暫緩",
                "IACUC 委員會主席",
                "in_app",
                "Animal",
                "仲裁流程；不可調整",
            ),
            f(
                "安樂死 — 超時自動核准",
                "24 小時無人回應",
                "該單開立的獸醫",
                "in_app",
                "Animal",
                "與安樂死狀態變更同一資料庫交易（原子）；不可調整",
            ),
            f(
                "獸醫師建議",
                "獸醫對動物新增照護建議",
                "該動物所屬計畫的 PI 與計劃負責人(SD)",
                "in_app",
                "Animal",
                "一般建議僅站內；緊急建議加發 Email。目前固定，未來可能納入路由",
            ),
            f(
                "審查指派",
                "計畫指派審查委員",
                "被指派審查該計畫的委員",
                "in_app",
                "AUP",
                "Email 視「委員審查」事件是否啟用；收件人由指派決定，不可調整",
            ),
        ]
    }

    // ── 內部驗證 helpers ──

    fn validate_channel(channel: &str) -> Result<(), AppError> {
        if !["in_app", "email", "both"].contains(&channel) {
            return Err(AppError::BadRequest(
                "channel 必須是 'in_app'、'email' 或 'both'".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_frequency(frequency: &str, day_of_week: Option<i16>) -> Result<(), AppError> {
        if !["immediate", "daily", "weekly", "monthly"].contains(&frequency) {
            return Err(AppError::BadRequest(
                "frequency 必須是 'immediate'、'daily'、'weekly' 或 'monthly'".to_string(),
            ));
        }
        if frequency == "weekly" && day_of_week.is_none() {
            return Err(AppError::BadRequest(
                "frequency 為 'weekly' 時，day_of_week（0-6）為必填".to_string(),
            ));
        }
        Ok(())
    }

    /// 取得所有可用角色列表
    pub async fn list_available_roles(&self) -> Result<Vec<RoleInfo>, AppError> {
        let roles: Vec<RoleInfo> = sqlx::query_as(
            r#"
            SELECT code, name
            FROM roles
            WHERE is_active = true
            ORDER BY code
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(roles)
    }

    /// 各事件路由規則的「具體收件人」預覽：
    /// - 角色型規則 → 列出實際持有該角色的使用者（label 為角色中文名）。
    /// - resolver 型規則 → 由事件動態決定，回傳人類可讀說明 + `fixed=true`（不可調），不列舉成員。
    ///
    /// 供 admin 路由頁巢狀收合顯示「實際是誰」。
    pub async fn routing_recipients_preview(
        &self,
    ) -> Result<Vec<RoutingRecipientsPreview>, AppError> {
        let rules = self.list_notification_routing().await?;
        let role_names: HashMap<String, String> = self
            .list_available_roles()
            .await?
            .into_iter()
            .map(|r| (r.code, r.name))
            .collect();

        let mut by_event: Vec<RoutingRecipientsPreview> = Vec::new();
        for rule in rules {
            let rule_recipients = if rule.target_kind == "resolver" {
                let (label, desc, fixed) = resolvers::resolver_meta(&rule.target_value);
                RoutingRuleRecipients {
                    rule_id: rule.id,
                    target_kind: rule.target_kind,
                    target_value: rule.target_value,
                    channel: rule.channel,
                    is_active: rule.is_active,
                    label: label.to_string(),
                    description: Some(desc.to_string()),
                    fixed,
                    members: Vec::new(),
                }
            } else {
                let members = self
                    .get_users_by_role(&rule.target_value)
                    .await?
                    .into_iter()
                    .map(|(id, email, display_name)| RecipientMember {
                        id,
                        display_name,
                        email,
                    })
                    .collect();
                let label = role_names
                    .get(&rule.target_value)
                    .cloned()
                    .unwrap_or_else(|| rule.target_value.clone());
                RoutingRuleRecipients {
                    rule_id: rule.id,
                    target_kind: rule.target_kind,
                    target_value: rule.target_value,
                    channel: rule.channel,
                    is_active: rule.is_active,
                    label,
                    description: None,
                    fixed: false,
                    members,
                }
            };

            match by_event
                .iter_mut()
                .find(|g| g.event_type == rule.event_type)
            {
                Some(grp) => grp.rules.push(rule_recipients),
                None => by_event.push(RoutingRecipientsPreview {
                    event_type: rule.event_type,
                    rules: vec![rule_recipients],
                }),
            }
        }
        Ok(by_event)
    }
}
