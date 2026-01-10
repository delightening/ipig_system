use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::{AuditAction, AuditLog, AuditLogQuery, AuditLogWithActor},
    Result,
};

pub struct AuditService;

impl AuditService {
    /// 記錄稽核日誌
    pub async fn log(
        pool: &PgPool,
        actor_user_id: Uuid,
        action: AuditAction,
        entity_type: &str,
        entity_id: Uuid,
        before: Option<serde_json::Value>,
        after: Option<serde_json::Value>,
    ) -> Result<AuditLog> {
        let log = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (id, actor_user_id, action, entity_type, entity_id, before_data, after_data, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(actor_user_id)
        .bind(action.as_str())
        .bind(entity_type)
        .bind(entity_id)
        .bind(before)
        .bind(after)
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// 查詢稽核日誌
    pub async fn list(pool: &PgPool, query: &AuditLogQuery) -> Result<Vec<AuditLogWithActor>> {
        // 基本查詢
        let mut sql = String::from(
            r#"
            SELECT 
                al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                al.action, al.entity_type, al.entity_id,
                al.before_data, al.after_data, al.created_at
            FROM audit_logs al
            INNER JOIN users u ON al.actor_user_id = u.id
            WHERE 1=1
            "#
        );

        // 動態添加條件
        if query.entity_type.is_some() {
            sql.push_str(" AND al.entity_type = $1");
        }
        if query.action.is_some() {
            sql.push_str(" AND al.action = $2");
        }
        if query.start_date.is_some() {
            sql.push_str(" AND al.created_at >= $3::date");
        }
        if query.end_date.is_some() {
            sql.push_str(" AND al.created_at < ($4::date + interval '1 day')");
        }

        sql.push_str(" ORDER BY al.created_at DESC LIMIT 200");

        // 由於 sqlx 的限制，使用簡化的查詢分支
        let logs = if let Some(ref entity_type) = query.entity_type {
            if let Some(ref action) = query.action {
                sqlx::query_as::<_, AuditLogWithActor>(
                    r#"
                    SELECT 
                        al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                        al.action, al.entity_type, al.entity_id,
                        al.before_data, al.after_data, al.created_at
                    FROM audit_logs al
                    INNER JOIN users u ON al.actor_user_id = u.id
                    WHERE al.entity_type = $1 AND al.action = $2
                    ORDER BY al.created_at DESC
                    LIMIT 200
                    "#
                )
                .bind(entity_type)
                .bind(action)
                .fetch_all(pool)
                .await?
            } else {
                sqlx::query_as::<_, AuditLogWithActor>(
                    r#"
                    SELECT 
                        al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                        al.action, al.entity_type, al.entity_id,
                        al.before_data, al.after_data, al.created_at
                    FROM audit_logs al
                    INNER JOIN users u ON al.actor_user_id = u.id
                    WHERE al.entity_type = $1
                    ORDER BY al.created_at DESC
                    LIMIT 200
                    "#
                )
                .bind(entity_type)
                .fetch_all(pool)
                .await?
            }
        } else if let Some(ref action) = query.action {
            sqlx::query_as::<_, AuditLogWithActor>(
                r#"
                SELECT 
                    al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                    al.action, al.entity_type, al.entity_id,
                    al.before_data, al.after_data, al.created_at
                FROM audit_logs al
                INNER JOIN users u ON al.actor_user_id = u.id
                WHERE al.action = $1
                ORDER BY al.created_at DESC
                LIMIT 200
                "#
            )
            .bind(action)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, AuditLogWithActor>(
                r#"
                SELECT 
                    al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                    al.action, al.entity_type, al.entity_id,
                    al.before_data, al.after_data, al.created_at
                FROM audit_logs al
                INNER JOIN users u ON al.actor_user_id = u.id
                ORDER BY al.created_at DESC
                LIMIT 200
                "#
            )
            .fetch_all(pool)
            .await?
        };

        Ok(logs)
    }

    /// 取得特定實體的稽核歷史
    pub async fn get_entity_history(
        pool: &PgPool,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditLogWithActor>> {
        let logs = sqlx::query_as::<_, AuditLogWithActor>(
            r#"
            SELECT 
                al.id, al.actor_user_id, u.email as actor_email, u.display_name as actor_name,
                al.action, al.entity_type, al.entity_id,
                al.before_data, al.after_data, al.created_at
            FROM audit_logs al
            INNER JOIN users u ON al.actor_user_id = u.id
            WHERE al.entity_type = $1 AND al.entity_id = $2
            ORDER BY al.created_at DESC
            "#
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }
}
