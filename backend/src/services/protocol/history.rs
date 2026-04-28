use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use serde_json::Value;

use super::ProtocolService;
use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        ProtocolActivity, ProtocolActivityResponse, ProtocolActivityType,
        ProtocolStatus, ProtocolVersion,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        AuditService,
    },
    Result,
};

/// 把 `ProtocolActivityType` 對應到全域 audit event_type 字串。
///
/// 抽成獨立函式讓 `record_activity` 與 `record_activity_tx` 共用。
fn event_type_for(activity_type: ProtocolActivityType) -> &'static str {
    match activity_type {
        ProtocolActivityType::Created => "PROTOCOL_CREATE",
        ProtocolActivityType::Updated => "PROTOCOL_UPDATE",
        ProtocolActivityType::Submitted => "PROTOCOL_SUBMIT",
        ProtocolActivityType::Resubmitted => "PROTOCOL_RESUBMIT",
        ProtocolActivityType::Approved => "PROTOCOL_APPROVE",
        ProtocolActivityType::ApprovedWithConditions => "PROTOCOL_APPROVE_CONDITIONAL",
        ProtocolActivityType::Rejected => "PROTOCOL_REJECT",
        ProtocolActivityType::CommentAdded => "PROTOCOL_COMMENT",
        ProtocolActivityType::CommentReplied => "PROTOCOL_COMMENT_REPLY",
        ProtocolActivityType::ReviewerAssigned => "PROTOCOL_REVIEWER_ASSIGN",
        ProtocolActivityType::VetAssigned => "PROTOCOL_VET_ASSIGN",
        ProtocolActivityType::StatusChanged => "PROTOCOL_STATUS_CHANGE",
        ProtocolActivityType::CoeditorAssigned => "PROTOCOL_COEDITOR_ASSIGN",
        ProtocolActivityType::CoeditorRemoved => "PROTOCOL_COEDITOR_REMOVE",
        _ => "PROTOCOL_ACTION",
    }
}

/// 把 status enum 轉成 `ProtocolActivityType`（record_status_change 用）。
fn activity_type_for_status(to_status: ProtocolStatus) -> ProtocolActivityType {
    match to_status {
        ProtocolStatus::Draft => ProtocolActivityType::Created,
        ProtocolStatus::Approved => ProtocolActivityType::Approved,
        ProtocolStatus::ApprovedWithConditions => ProtocolActivityType::ApprovedWithConditions,
        ProtocolStatus::Closed => ProtocolActivityType::Closed,
        ProtocolStatus::Rejected => ProtocolActivityType::Rejected,
        ProtocolStatus::Suspended => ProtocolActivityType::Suspended,
        ProtocolStatus::Deleted => ProtocolActivityType::Deleted,
        ProtocolStatus::Submitted => ProtocolActivityType::Submitted,
        ProtocolStatus::Resubmitted => ProtocolActivityType::Resubmitted,
        _ => ProtocolActivityType::StatusChanged,
    }
}

impl ProtocolService {
    /// Transaction 版本：取得下一個版本號。舊 pool 版本已於 PR #3 刪除（被此替代）。
    pub(super) async fn get_next_version_no_tx(
        tx: &mut Transaction<'_, Postgres>,
        protocol_id: Uuid,
    ) -> Result<i32> {
        let max_version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version_no) FROM protocol_versions WHERE protocol_id = $1",
        )
        .bind(protocol_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(max_version.unwrap_or(0) + 1)
    }

    /// Transaction 版本：同 `record_activity` 但走 tx + 使用 `log_activity_tx`
    /// 讓 protocol_activities INSERT 與 user_activity_logs + HMAC 全部同 tx。
    ///
    /// Service-driven 重構的 protocol 路徑（例如 `submit`）使用此函式。
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn record_activity_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        protocol_id: Uuid,
        activity_type: ProtocolActivityType,
        from_value: Option<String>,
        to_value: Option<String>,
        target_entity: Option<(&str, Uuid, &str)>,
        remark: Option<String>,
        extra_data: Option<Value>,
    ) -> Result<ProtocolActivity> {
        let actor_id = actor
            .actor_user_id()
            .unwrap_or(crate::middleware::SYSTEM_USER_ID);

        // actor_name / actor_email（可 None，走 fallback）
        let actor_info: Option<(String, String)> = sqlx::query_as(
            "SELECT COALESCE(display_name, email), email FROM users WHERE id = $1",
        )
        .bind(actor_id)
        .fetch_optional(&mut **tx)
        .await?;
        let (actor_name, actor_email) = actor_info.unwrap_or_default();

        let (target_type, target_id, target_name) = target_entity
            .map(|(t, i, n)| (Some(t.to_string()), Some(i), Some(n.to_string())))
            .unwrap_or((None, None, None));

        let activity = sqlx::query_as::<_, ProtocolActivity>(
            r#"
            INSERT INTO protocol_activities (
                protocol_id, activity_type, actor_id, actor_name, actor_email,
                from_value, to_value, target_entity_type, target_entity_id, target_entity_name,
                remark, extra_data, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            RETURNING *
            "#,
        )
        .bind(protocol_id)
        .bind(activity_type)
        .bind(actor_id)
        .bind(&actor_name)
        .bind(&actor_email)
        .bind(&from_value)
        .bind(&to_value)
        .bind(&target_type)
        .bind(target_id)
        .bind(&target_name)
        .bind(&remark)
        .bind(&extra_data)
        .fetch_one(&mut **tx)
        .await?;

        // Protocol 標題（供 audit 顯示）
        let protocol_title: String = sqlx::query_scalar(
            "SELECT title FROM protocols WHERE id = $1",
        )
        .bind(protocol_id)
        .fetch_optional(&mut **tx)
        .await?
        .unwrap_or_else(|| "Unknown Protocol".to_string());

        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type: event_type_for(activity_type),
                entity: Some(AuditEntity::new("protocol", protocol_id, &protocol_title)),
                data_diff: None,
                request_context: None,
            },
        )
        .await?;

        Ok(activity)
    }

    /// Transaction 版本：同 `record_status_change` 但走 tx。
    pub(super) async fn record_status_change_tx(
        tx: &mut Transaction<'_, Postgres>,
        actor: &ActorContext,
        protocol_id: Uuid,
        from_status: Option<ProtocolStatus>,
        to_status: ProtocolStatus,
        remark: Option<String>,
    ) -> Result<()> {
        let activity_type = activity_type_for_status(to_status);
        Self::record_activity_tx(
            tx,
            actor,
            protocol_id,
            activity_type,
            from_status.map(|s| s.as_str().to_string()),
            Some(to_status.as_str().to_string()),
            None,
            remark,
            None,
        )
        .await?;
        Ok(())
    }

    /// 記錄活動
    pub async fn record_activity(
        pool: &PgPool,
        protocol_id: Uuid,
        activity_type: ProtocolActivityType,
        actor_id: Uuid,
        from_value: Option<String>,
        to_value: Option<String>,
        target_entity: Option<(&str, Uuid, &str)>,
        remark: Option<String>,
        extra_data: Option<Value>,
    ) -> Result<ProtocolActivity> {
        // 取得行為者資訊
        let actor_info: Option<(String, String)> = sqlx::query_as(
            "SELECT COALESCE(display_name, email), email FROM users WHERE id = $1"
        )
        .bind(actor_id)
        .fetch_optional(pool)
        .await?;

        let (actor_name, actor_email) = actor_info.unwrap_or_default();
        
        let (target_type, target_id, target_name) = target_entity
            .map(|(t, i, n)| (Some(t.to_string()), Some(i), Some(n.to_string())))
            .unwrap_or((None, None, None));

        let activity = sqlx::query_as::<_, ProtocolActivity>(
            r#"
            INSERT INTO protocol_activities (
                protocol_id, activity_type, actor_id, actor_name, actor_email,
                from_value, to_value, target_entity_type, target_entity_id, target_entity_name,
                remark, extra_data, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            RETURNING *
            "#
        )
        .bind(protocol_id)
        .bind(activity_type)
        .bind(actor_id)
        .bind(&actor_name)
        .bind(&actor_email)
        .bind(&from_value)
        .bind(&to_value)
        .bind(&target_type)
        .bind(target_id)
        .bind(&target_name)
        .bind(&remark)
        .bind(&extra_data)
        .fetch_one(pool)
        .await?;

        // 取得計畫標題用於全域審計日誌
        let protocol_title: String = sqlx::query_scalar(
            "SELECT title FROM protocols WHERE id = $1"
        )
        .bind(protocol_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or_else(|| "Unknown Protocol".to_string());

        // 同步記錄到全域審計日誌
        let event_type = event_type_for(activity_type);

        let actor = ActorContext::User(CurrentUser {
            id: actor_id,
            email: String::new(),
            roles: vec![],
            permissions: vec![],
            jti: String::new(),
            exp: 0,
            impersonated_by: None,
        });

        if let Err(e) = AuditService::log_activity_oneshot(
            pool,
            &actor,
            ActivityLogEntry {
                event_category: "AUP",
                event_type,
                entity: Some(AuditEntity {
                    entity_type: "protocol",
                    entity_id: protocol_id,
                    entity_display_name: &protocol_title,
                }),
                data_diff: None,
                request_context: None,
            },
        ).await {
            tracing::error!("寫入 user_activity_logs 失敗 ({}): {}", event_type, e);
        }

        Ok(activity)
    }

    /// 取得版本列表
    pub async fn get_versions(pool: &PgPool, protocol_id: Uuid) -> Result<Vec<ProtocolVersion>> {
        let versions = sqlx::query_as::<_, ProtocolVersion>(
            "SELECT * FROM protocol_versions WHERE protocol_id = $1 ORDER BY version_no DESC"
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;

        Ok(versions)
    }

    /// 取得活動歷程
    pub async fn get_activities(pool: &PgPool, protocol_id: Uuid) -> Result<Vec<ProtocolActivityResponse>> {
        let activities = sqlx::query_as::<_, ProtocolActivity>(
            "SELECT * FROM protocol_activities WHERE protocol_id = $1 ORDER BY created_at DESC"
        )
        .bind(protocol_id)
        .fetch_all(pool)
        .await?;

        Ok(activities.into_iter().map(ProtocolActivityResponse::from).collect())
    }
}
